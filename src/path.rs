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

pub fn read_toml(path: &PathBuf) -> Result<Table> {
    read(path)?
        .parse()
        .wrap_err(format!("could not parse file {path:#?} as toml"))
}

pub fn write_toml(path: &PathBuf, table: &Table) -> Result<()> {
    let path = permissive_normalize(path);
    let parent = path
        .parent()
        .wrap_err(format!("unable to determine parent of {path:#?}"))?;
    fs::create_dir_all(parent).wrap_err(format!("failed to create directory {parent:#?}"))?;
    fs::write(&path, table.to_string())
        .wrap_err(format!("unable to write merged config to {path:#?}"))
}

pub fn read_yaml(path: &PathBuf) -> Result<serde_yaml::Value> {
    serde_yaml::from_str(&read(path)?).wrap_err(format!("could not parse file {path:#?} as yaml"))
}

pub fn write_yaml(path: &PathBuf, value: &serde_yaml::Value) -> Result<()> {
    let path = permissive_normalize(path);
    let parent = path
        .parent()
        .wrap_err(format!("unable to determine parent of {path:#?}"))?;
    fs::create_dir_all(parent).wrap_err(format!("failed to create directory {parent:#?}"))?;
    fs::write(
        &path,
        serde_yaml::to_string(value).wrap_err(format!(
            "unable to serialize and write merged yaml to {path:#?}"
        ))?,
    )
    .wrap_err(format!("unable to write merged config to {path:#?}"))
}


pub fn read_json(path: &PathBuf) -> Result<serde_json::Value> {
    serde_yaml::from_str(&read(path)?).wrap_err(format!("could not parse file {path:#?} as yaml"))
}

pub fn write_json(path: &PathBuf, value: &serde_json::Value) -> Result<()> {
    let path = permissive_normalize(path);
    let parent = path
        .parent()
        .wrap_err(format!("unable to determine parent of {path:#?}"))?;
    fs::create_dir_all(parent).wrap_err(format!("failed to create directory {parent:#?}"))?;
    fs::write(
        &path,
        serde_json::to_string(value).wrap_err(format!(
            "unable to serialize and write merged json to {path:#?}"
        ))?,
    )
    .wrap_err(format!("unable to write merged config to {path:#?}"))
}
