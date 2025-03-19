use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize};

use crate::path::{is_dir, is_file};

use color_eyre::Result;

#[derive(Deserialize, Debug)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[allow(clippy::module_name_repetitions)]
pub struct RelConf {
    pub tools: Vec<Tool>,
}


#[derive(Serialize, Debug)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum InjectConfig {
    Path {
        path: FilePath,
        #[serde(rename = "command", default, skip_serializing_if = "Option::is_none")]
        command: Option<()>,
    },
    Template {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<()>,
        #[serde(rename = "command")]
        command: String,
    },
}

impl<'de> Deserialize<'de> for InjectConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            path: Option<FilePath>,
            #[serde(rename = "command")]
            command: Option<String>,
        }

        let helper = Helper::deserialize(deserializer)?;

        match (helper.path, helper.command) {
            (Some(config), None) => Ok(InjectConfig::Path {
                path: config,
                command: None
            }),
            (None, Some(cmd)) => Ok(InjectConfig::Template {
                path: None,
                command: cmd
            }),
            (Some(_), Some(_)) => Err(serde::de::Error::custom(
                "cannot specify both 'path' and 'command' on config"
            )),
            (None, None) => Err(serde::de::Error::custom(
                "must specify either 'path' or 'command' on config"
            )),
        }
    }
}

#[derive(Deserialize, Debug)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Tool {
    pub name: String,
    pub format: Format,
    pub inject: Vec<Inject>,
    pub configs: Vec<Config>,
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
pub struct Config {
    #[serde(flatten)]
    pub config: InjectConfig,
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

#[derive(Serialize, Debug)]
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
