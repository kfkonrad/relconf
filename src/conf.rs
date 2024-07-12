use std::path::PathBuf;

use serde::{Deserialize, Deserializer};

use crate::path::{is_dir, is_file};

use color_eyre::Result;

#[derive(Deserialize, Debug)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[allow(clippy::module_name_repetitions)]
pub struct RelConf {
    pub tools: Vec<Tool>,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Tool {
    pub name: String,
    pub format: Format,
    pub rootconfig: FilePath,
    pub inject: Vec<Inject>,
    pub subconfigs: Vec<Subconfig>,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Inject {
    pub path: PathBuf,
    #[serde(rename = "env-name")]
    pub env_name: Option<String>,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Toml,
    Yaml,
    Json,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Subconfig {
    pub path: FilePath,
    pub when: Option<Vec<When>>,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct When {
    pub directory: DirectoryPath,
    #[serde(rename = "match-subdirectories", default)]
    pub match_subdirectories: bool,
}

fn errconvert<E, R>(e: Result<R>) -> Result<R, E>
where
    E: serde::de::Error,
{
    e.map_err(|e| serde::de::Error::custom(e.to_string()))
}

#[derive(Debug)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FilePath(pub PathBuf);

impl<'de> Deserialize<'de> for FilePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let path = PathBuf::from(s);

        if !errconvert(is_file(&path))? {
            return Err(serde::de::Error::custom(format!(
                "Expected a file path or symlink to a file, received {}",
                path.to_string_lossy()
            )));
        }

        Ok(Self(path))
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DirectoryPath(pub PathBuf);

impl<'de> Deserialize<'de> for DirectoryPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let path = PathBuf::from(s);

        if !errconvert(is_dir(&path))? {
            return Err(serde::de::Error::custom(
                "Expected a directory path or symlink to a directory",
            ));
        }

        Ok(Self(path))
    }
}
