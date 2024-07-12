use std::{collections::HashSet, path::PathBuf};

use tool::handle;

mod conf;
mod path;
mod tool;
mod merge;

use clap::Parser;

use color_eyre::{
    eyre::{eyre, Context, Ok, OptionExt},
    Result,
};

#[derive(Parser)]
#[command(name = "relconf")]
#[command(author = "Kevin F. Konrad")]
#[command(version = "0.1")]
#[command(about = "Generate config files depending on the current path", long_about = None)]
#[command(disable_version_flag = true)]
struct Cli {
    #[arg(short = 'v', short_alias = 'V', long, action = clap::builder::ArgAction::Version)]
    version: (),
    #[arg(
        long = "config",
        short = 'c',
        help = "Override location of relconf config file"
    )]
    config_file: Option<String>,
    #[arg(
        long = "only",
        short = 'o',
        value_delimiter = ',',
        help = "Only generate config for listed tool(s)"
    )]
    tool_names: Vec<String>,
    #[cfg(feature = "schema")]
    #[arg(long, help = "Generate JSON schema")]
    generate_schema: bool,
}

fn env_or_default_config_path() -> Result<PathBuf> {
    std::env::var_os("RELCONF_CONFIG")
        .map_or_else(default_config_path, |config_path| Ok(config_path.into()))
}

fn default_config_path() -> Result<PathBuf> {
    let mut config_dir = dirs::config_dir().ok_or_eyre(eyre!(
        "could not determine default directory for refconf config"
    ))?;
    config_dir.push("relconf/config.yaml");
    Ok(config_dir)
}

fn main() -> color_eyre::Result<()> {
    color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .display_location_section(false)
        .install()?;
    let cli = Cli::parse();
    let config_path = cli
        .config_file
        .map_or_else(env_or_default_config_path, |override_filename| {
            path::normalize(&override_filename)
        })?;

    #[cfg(feature = "schema")]
    if cli.generate_schema {
        use schemars::schema_for;
        let schema = schema_for!(conf::RelConf);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
        return Ok(());
    }

    let raw_config = path::read(&config_path)?;
    let tool_name_set: HashSet<String> = cli.tool_names.into_iter().collect();

    let config: conf::RelConf = serde_yaml::from_str(raw_config.as_str()).wrap_err(format!(
        "error parsing relconf config from {config_path:#?}"
    ))?;
    for tool in config.tools {
        if tool_name_set.is_empty() || tool_name_set.contains(&tool.name) {
            handle(tool)?;
        }
    }
    Ok(())
}
