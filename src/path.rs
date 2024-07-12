use std::{
    fs,
    path::{Path, PathBuf},
};

use shellexpand::path::tilde;
use toml::Table;

use color_eyre::{
    eyre::{Context, ContextCompat, Ok},
    Result,
};

pub fn normalize<SP>(path: &SP) -> Result<PathBuf>
where
    SP: ?Sized + AsRef<Path> + std::fmt::Debug,
{
    tilde(path)
        .canonicalize()
        .wrap_err(format!("no such path {path:#?}"))
}

// this means the path needn't exist (only existing paths can be canonicalized)
pub fn permissive_normalize<SP>(path: &SP) -> PathBuf
where
    SP: ?Sized + AsRef<Path>,
{
    let potential_normal_path = tilde(path);
    potential_normal_path
        .canonicalize()
        .unwrap_or_else(|_| potential_normal_path.into())
}

pub fn is_subdir(parent: &PathBuf, subdir: &PathBuf) -> Result<bool> {
    let parent_canon = normalize(parent)?;
    let subdir_canon = normalize(subdir)?;

    Ok(subdir_canon.starts_with(parent_canon))
}

pub fn is_file(path: &Path) -> Result<bool> {
    Ok(normalize(&path)?.is_file())
}

pub fn is_dir(path: &Path) -> Result<bool> {
    Ok(normalize(&path)?.is_dir())
}

pub fn read(path: &PathBuf) -> Result<String> {
    std::fs::read_to_string(normalize(path)?).wrap_err(format!("could not read file {path:#?}"))
}

macro_rules! define_read_function {
    ($name_suffix:ident, $read_call:path, $return_type:ty, $error_type:expr) => {
        paste::paste! {
            pub fn [<read_ $name_suffix>](path: &PathBuf) -> Result<$return_type> {
                $read_call(&read(path)?).wrap_err(format!("could not parse file {path:#?} as {}", $error_type))
            }
        }
    };
}

macro_rules! define_write_function {
    ($name_suffix:ident, $value_type:ty, $to_string_call:path, $serialize_err:expr) => {
        paste::paste! {
            pub fn [<write_ $name_suffix>](path: &PathBuf, value: &$value_type) -> Result<()> {
                let path = permissive_normalize(path);
                let parent = path
                    .parent()
                    .wrap_err(format!("unable to determine parent of {path:#?}"))?;
                fs::create_dir_all(parent).wrap_err(format!("failed to create directory {parent:#?}"))?;
                fs::write(
                    &path,
                    $to_string_call(value).wrap_err(format!(
                        "unable to serialize and write merged $serialize_err to {path:#?}"
                    ))?,
                )
                .wrap_err(format!("unable to write merged config to {path:#?}"))
            }
        }
    };
}

define_read_function!(json, serde_json::from_str, serde_json::Value, "json");
define_read_function!(yaml, serde_yaml::from_str, serde_yaml::Value, "yaml");
define_read_function!(toml, toml::from_str, toml::Table, "toml");

fn table_to_string(table: &Table) -> Result<String> {
    Ok(Table::to_string(table))
}
define_write_function!(toml, Table, table_to_string, "toml");
define_write_function!(yaml, serde_yaml::Value, serde_yaml::to_string, "yaml");
define_write_function!(json, serde_json::Value, serde_json::to_string, "json");
