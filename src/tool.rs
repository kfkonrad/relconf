use serde_toml_merge::merge_into_table;
use std::env;
use toml::Table;

use crate::{
    conf::{self, Inject, Subconfig},
    path,
};

pub fn handle(tool: conf::Tool) {
    let mut merged_config: Table = path::read_toml(&tool.rootconfig.0);
    for subconfig in tool.subconfigs {
        if shold_run(&subconfig) {
            let additional_config: Table = path::read_toml(&subconfig.config.0);
            merge_into_table(&mut merged_config, additional_config).unwrap();
        }
    }
    for inject in tool.inject {
        perform_injection(inject, &merged_config);
    }
}

fn shold_run(subconfig: &Subconfig) -> bool {
    let current_dir = env::current_dir().unwrap();
    let empty = Vec::new();
    let when_vec = subconfig.when.as_ref().unwrap_or(&empty);
    if when_vec.is_empty() {
        return true;
    };
    for when in when_vec {
        if when.match_subdirectories && path::is_subdir(&when.directory.0, &current_dir) {
            return true;
        }
        if when.directory.0 == current_dir {
            return true;
        }
    }
    false
}
fn perform_injection(inject: Inject, table: &Table) {
    path::write_toml(&inject.path, &table);
    if let Some(env_name) = inject.env_name {
        println!(
            "export {}={:#?}",
            env_name,
            path::normalize(&inject.path).to_string_lossy()
        )
    }
}
