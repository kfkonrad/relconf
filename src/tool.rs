use serde_toml_merge::merge_into_table;
use std::env;
use toml::Table;

use crate::{
    conf::{self, Inject, Subconfig}, merge, path
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

pub fn handle_toml(tool: conf::Tool) -> Result<()> {
    let mut merged_config: Table = path::read_toml(&tool.rootconfig.0)?;
    for subconfig in tool.subconfigs {
        if shold_run(&subconfig)? {
            let additional_config: Table = path::read_toml(&subconfig.path.0)?;
            merge_into_table(&mut merged_config, additional_config).map_err(|e| {
                eyre!(format!(
                    "unable to merge subconfig from {:#?} for tool {}: {e}",
                    subconfig.path.0, tool.name
                ))
            })?;
        }
    }
    for inject in tool.inject {
        perform_injection_toml(inject, &merged_config)?;
    }
    Ok(())
}

pub fn handle_yaml(tool: conf::Tool) -> Result<()> {
    let mut merged_config: serde_yaml::Value = path::read_yaml(&tool.rootconfig.0)?;
    for subconfig in tool.subconfigs {
        if shold_run(&subconfig)? {
            let additional_config: serde_yaml::Value = path::read_yaml(&subconfig.path.0)?;
            merge::yaml(&mut merged_config, additional_config).map_err(|e| {
                eyre!(format!(
                    "unable to merge subconfig from {:#?} for tool {}: {e}",
                    subconfig.path.0, tool.name
                ))
            })?;
        }
    }
    for inject in tool.inject {
        perform_injection_yaml(inject, &merged_config)?;
    }
    Ok(())
}

pub fn handle_json(tool: conf::Tool) -> Result<()> {
    let mut merged_config: serde_json::Value = path::read_json(&tool.rootconfig.0)?;
    for subconfig in tool.subconfigs {
        if shold_run(&subconfig)? {
            let additional_config: serde_json::Value = path::read_json(&subconfig.path.0)?;
            merge::json(&mut merged_config, additional_config).map_err(|e| {
                eyre!(format!(
                    "unable to merge subconfig from {:#?} for tool {}: {e}",
                    subconfig.path.0, tool.name
                ))
            })?;
        }
    }
    for inject in tool.inject {
        perform_injection_json(inject, &merged_config)?;
    }
    Ok(())
}

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

fn perform_injection_generic(inject: Inject) -> Result<()> {
    if let Some(env_name) = inject.env_name {
        println!(
            "export {}={:#?}",
            env_name,
            path::normalize(&inject.path)?.to_string_lossy()
        );
    }
    Ok(())
}

fn perform_injection_toml(inject: Inject, table: &Table) -> Result<()> {
    path::write_toml(&inject.path, table)?;
    perform_injection_generic(inject)
}


fn perform_injection_yaml(inject: Inject, value: &serde_yaml::Value) -> Result<()> {
    path::write_yaml(&inject.path, value)?;
    perform_injection_generic(inject)
}


fn perform_injection_json(inject: Inject, value: &serde_json::Value) -> Result<()> {
    path::write_json(&inject.path, value)?;
    perform_injection_generic(inject)
}
