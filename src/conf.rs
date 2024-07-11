use std::path::PathBuf;

use serde::{Deserialize, Deserializer};

use crate::path::{is_dir, is_file};

#[derive(Deserialize, Debug)]
pub struct RelConf {
    pub tools: Vec<Tool>,
}

#[derive(Deserialize, Debug)]
pub struct Tool {
    pub name: String,
    pub format: Format,
    pub rootconfig: FilePath,
    pub inject: Vec<Inject>,
    pub subconfigs: Vec<Subconfig>,
}

#[derive(Deserialize, Debug)]
pub struct Inject {
    #[serde(rename = "type")]
    pub inject_type: InjectType,
    pub location: PathBuf,
    pub name: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum InjectType {
    Env,
    File,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Toml,
}

#[derive(Deserialize, Debug)]
pub struct Subconfig {
    pub directory: DirectoryPath,
    pub config: FilePath,
    #[serde(rename = "match-subdirectories")]
    pub match_subdirectories: bool,
}

#[derive(Debug)]
pub struct FilePath(pub PathBuf);

impl<'de> Deserialize<'de> for FilePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let path = PathBuf::from(s);

        if !is_file(&path) {
            return Err(serde::de::Error::custom(format!(
                "Expected a file path or symlink to a file, received {}",
                path.to_string_lossy()
            )));
        }

        Ok(FilePath(path))
    }
}

#[derive(Debug)]
pub struct DirectoryPath(pub PathBuf);

impl<'de> Deserialize<'de> for DirectoryPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let path = PathBuf::from(s);

        if !is_dir(&path) {
            return Err(serde::de::Error::custom(
                "Expected a directory path or symlink to a directory",
            ));
        }

        Ok(DirectoryPath(path))
    }
}
