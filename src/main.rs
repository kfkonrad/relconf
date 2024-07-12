use std::collections::HashSet;

use tool::handle;

mod conf;
mod path;
mod tool;

use clap::Parser;

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

fn main() {
    dirs::config_dir().unwrap();
    let cli = Cli::parse();
    let config_path = match cli.config_file {
        Some(override_filename) => path::normalize(&override_filename),
        None => match std::env::var_os("RELCONF_CONFIG") {
            Some(relconf_config) => relconf_config.into(),
            None => {
                let mut config_dir = dirs::config_dir().unwrap();
                config_dir.push("relconf/config.yaml");
                config_dir
            }
        },
    };

    #[cfg(feature = "schema")]
    if cli.generate_schema {
        use schemars::schema_for;
        let schema = schema_for!(conf::RelConf);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
        return;
    }

    let raw_config = path::read(&config_path);
    let tool_name_set: HashSet<String> = cli.tool_names.into_iter().collect();

    let config: conf::RelConf = serde_yaml::from_str(raw_config.as_str()).unwrap(); // TODO: remove unwraps (later)
    config.tools.into_iter().for_each(|tool| {
        if tool_name_set.is_empty() || tool_name_set.contains(&tool.name) {
            handle(tool);
        }
    });
}
