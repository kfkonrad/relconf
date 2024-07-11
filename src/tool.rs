use serde_toml_merge::merge_into_table;
use std::env;
use toml::Table;

use crate::{
    conf::{self, Inject},
    path,
};

pub fn handle(tool: conf::Tool) {
    let current_dir = env::current_dir().unwrap();
    let mut merged_config: Table = path::read_toml(&tool.rootconfig.0);
    for subconfig in tool.subconfigs {
        if (subconfig.match_subdirectories && path::is_subdir(&subconfig.directory.0, &current_dir))
            || subconfig.directory.0 == current_dir
        {
            let additional_config: Table = path::read_toml(&subconfig.config.0);
            merge_into_table(&mut merged_config, additional_config).unwrap();
        }
    }
    for inject in tool.inject {
        perform_injection(inject, &merged_config);
    }
}

fn perform_injection(inject: Inject, table: &Table) {
    path::write_toml(&inject.location, &table);
    match inject.inject_type {
        conf::InjectType::Env => {
            println!(
                "export {}={:#?}",
                inject.name.unwrap(),
                path::normalize(&inject.location).to_string_lossy()
            )
        }
        conf::InjectType::File => {}
    };
}
