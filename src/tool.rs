use std::env;
use toml::Table;

use crate::{
    conf::{self, Inject, Subconfig},
    merge, path,
};

use color_eyre::{
    eyre::{eyre, Context, Ok},
    Result,
};

pub fn handle(tool: conf::Tool) -> Result<()> {
    match tool.format {
        conf::Format::Toml => handle_toml(tool),
        conf::Format::Yaml => handle_yaml(tool),
        conf::Format::Json => handle_json(tool),
    }
}

macro_rules! define_handle_function {
    ($name_suffix:ident, $config_type:ty, $read_call:path, $merge_call:path, $perform_injection_call:path) => {
        paste::paste! {
            pub fn [<handle_ $name_suffix>](tool: conf::Tool) -> Result<()> {
                let mut merged_config: $config_type = $read_call(&tool.rootconfig.0)?;
                for subconfig in tool.subconfigs {
                    if shold_run(&subconfig)? {
                        let additional_config: $config_type = $read_call(&subconfig.path.0)?;
                        $merge_call(&mut merged_config, additional_config).map_err(|e| {
                            eyre!(format!(
                                "unable to merge subconfig from {:#?} for tool {}: {e}",
                                subconfig.path.0, tool.name
                            ))
                        })?;
                    }
                }
                for inject in tool.inject {
                    $perform_injection_call(inject, &merged_config)?;
                }
                Ok(())
            }
        }
    };
}

define_handle_function!(
    toml,
    Table,
    path::read_toml,
    merge::toml,
    perform_injection_toml
);
define_handle_function!(
    yaml,
    serde_yaml::Value,
    path::read_yaml,
    merge::yaml,
    perform_injection_yaml
);
define_handle_function!(
    json,
    serde_json::Value,
    path::read_json,
    merge::json,
    perform_injection_json
);

fn shold_run(subconfig: &Subconfig) -> Result<bool> {
    let current_dir =
        env::current_dir().wrap_err("cannot determine current directory or doesn't exist")?;
    let empty = Vec::new();
    let when_vec = subconfig.when.as_ref().unwrap_or(&empty);
    if when_vec.is_empty() {
        return Ok(true);
    };
    for when in when_vec {
        if when.match_subdirectories && path::is_subdir(&when.directory.0, &current_dir)? {
            return Ok(true);
        }
        if when.directory.0 == current_dir {
            return Ok(true);
        }
    }
    Ok(false)
}

macro_rules! define_perform_injection_function {
    ($name_suffix:ident, $config_type:ty) => {
        paste::paste! {
            fn [<perform_injection_ $name_suffix>](inject: Inject, config: &$config_type) -> Result<()> {
                path::[<write_ $name_suffix>](&inject.path, config)?;
                if let Some(env_name) = inject.env_name {
                    println!(
                        "export {}={:#?}",
                        env_name,
                        path::normalize(&inject.path)?.to_string_lossy()
                    );
                }
                Ok(())
            }
        }
    };
}

define_perform_injection_function!(toml, Table);
define_perform_injection_function!(yaml, serde_yaml::Value);
define_perform_injection_function!(json, serde_json::Value);
