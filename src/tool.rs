use serde_toml_merge::merge_into_table;
use std::env;
use toml::Table;

use crate::{conf, path};

pub fn handle(tool: conf::Tool) {
    let current_dir = env::current_dir().unwrap();
    let mut merged_config: Table = path::read_toml(&tool.rootconfig.0);
    for subconfig in tool.subconfigs {
        if path::is_subdir(&subconfig.directory.0, &current_dir) {
            let additional_config: Table = path::read_toml(&subconfig.config.0);
            merge_into_table(&mut merged_config, additional_config).unwrap();
        }
    }
    for injection in tool.inject {
        path::write_toml(&injection.location, &merged_config);
        match injection.inject_type {
            conf::InjectType::Env => {
                println!(
                    "export {}={:#?}",
                    injection.name.unwrap(),
                    injection.location.to_string_lossy()
                )
            }
            conf::InjectType::File => {}
        }
    }
}
