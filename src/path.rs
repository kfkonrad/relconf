use std::path::{Path, PathBuf};

use shellexpand::path::tilde;

use color_eyre::{
    eyre::{Context, Ok},
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

pub fn run_command(command: &str) -> Result<String> {
    let (shell, arg) = if cfg!(target_os = "windows") {
        ("powershell", "-Command")
    } else {
        ("sh", "-c")
    };
    let output = std::process::Command::new(shell)
        .arg(arg)
        .arg(command)
        .output()
        .wrap_err(format!("failed to execute command {command}"))?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout).wrap_err(format!(
            "failed to parse output of command {command} as utf8"
        ))?)
    } else {
        Err(color_eyre::eyre::eyre!(
            "{}",
            String::from_utf8(output.stderr)
                .unwrap_or_else(|_| format!("unknown error executing command {command}"))
        ))
    }
}
