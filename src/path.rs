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

fn run_command(command: String) -> Result<String> {
    let (shell, arg) = if cfg!(target_os = "windows") {
        ("powershell", "-Command")
    } else {
        ("sh", "-c")
    };
    let output = std::process::Command::new(shell)
        .arg(arg)
        .arg(&command)
        .output()
        .wrap_err(format!("failed to execute command {command}"))?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout).wrap_err(format!("failed to parse output of command {command} as utf8"))?)
    } else {
        Err(color_eyre::eyre::eyre!(
            "{}",
            String::from_utf8(output.stderr).unwrap_or_else(|_| format!("unknown error executing command {command}").to_string())
        ))
    }
}

macro_rules! define_read_function {
    ($name_suffix:ident, $read_call:path, $return_type:ty, $error_type:expr) => {
        paste::paste! {
            pub fn [<read_ $name_suffix>](path: &Option<PathBuf>, command: Option<String>) -> Result<$return_type> {
                let config: String = match (path, command) {
                    (Some(p), None) => read(p)?,
                    (None, Some(command)) => run_command(command)?,
                    // if the validation works correctly, these match-branches will be unreachable
                    (None, None) => unreachable!("must specify either 'path' or 'command' on config"),
                    (Some(_), Some(_)) => unreachable!("cannot specify both 'path' and 'command' on config")
                };
                $read_call(&config).wrap_err(format!("could not parse file {path:#?} as {}", $error_type))
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
