use std::env;

use crate::{conf, format::Config, path};

use color_eyre::{
    eyre::{Context, Ok},
    Result,
};

pub fn handle(tool: conf::Tool) -> Result<()> {
    match tool.format {
        conf::Format::Toml => toml::Table::handle_tool(tool),
        conf::Format::Yaml => serde_yaml::Value::handle_tool(tool),
        conf::Format::Json => serde_json::Value::handle_tool(tool),
    }
}

pub fn should_run(config: &conf::Config) -> Result<bool> {
    let current_dir =
        env::current_dir().wrap_err("cannot determine current directory or doesn't exist")?;
    let empty = Vec::new();
    let when_vec = config.when.as_ref().unwrap_or(&empty);
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
