use std::path::PathBuf;
use color_eyre::{
    eyre::{Context, ContextCompat},
    Result,
};

pub trait Config: Sized {
    type Value: Default;

    fn format_name() -> &'static str;

    fn parse_from_str(content: &str) -> Result<Self::Value>;

    fn to_string(value: &Self::Value) -> Result<String>;

    fn merge(a: &mut Self::Value, b: Self::Value) -> Result<()>;

    fn read(path: Option<&PathBuf>, command: Option<String>) -> Result<Self::Value> {
        let config: String = match (path, command) {
            (Some(p), None) => crate::path::read(p)?,
            (None, Some(command)) => crate::path::run_command(&command)?,
            // if the validation works correctly, these match-branches will be unreachable
            (None, None) => unreachable!("must specify either 'path' or 'command' on config"),
            (Some(_), Some(_)) => unreachable!("cannot specify both 'path' and 'command' on config")
        };
        Self::parse_from_str(&config).wrap_err(format!(
            "could not parse file {path:#?} as {}",
            Self::format_name()
        ))
    }

    fn write(path: &PathBuf, value: &Self::Value) -> Result<()> {
        let path = crate::path::permissive_normalize(path);
        let parent = path
            .parent()
            .wrap_err(format!("unable to determine parent of {path:#?}"))?;
        std::fs::create_dir_all(parent).wrap_err(format!("failed to create directory {parent:#?}"))?;
        std::fs::write(
            &path,
            Self::to_string(value).wrap_err(format!(
                "unable to serialize and write merged {} to {path:#?}",
                Self::format_name()
            ))?,
        )
        .wrap_err(format!("unable to write merged config to {path:#?}"))
    }

    fn perform_injection(inject: crate::conf::Inject, config: &Self::Value) -> Result<()> {
        Self::write(&inject.path, config)?;
        if let Some(env_name) = inject.env_name {
            println!(
                "export {}={:#?}",
                env_name,
                crate::path::normalize(&inject.path)?.to_string_lossy()
            );
        }
        Ok(())
    }

    fn handle_tool(tool: crate::conf::Tool) -> Result<()> {
        let mut merged_config: Self::Value = Self::Value::default();
        for config in tool.configs {
            if crate::tool::should_run(&config)? {
                let additional_config: Self::Value = match &config.config {
                    crate::conf::InjectConfig::Path {path, ..} => Self::read(Some(&path.0.clone()), None),
                    crate::conf::InjectConfig::Template {command, ..} => Self::read(None, Some(command.into())),
                }?;
                let stringified_config: String = match &config.config {
                    crate::conf::InjectConfig::Path {path, ..} => crate::path::normalize(&path.0)?.to_string_lossy().into(),
                    crate::conf::InjectConfig::Template {command, ..} => command.into(),
                };

                Self::merge(&mut merged_config, additional_config).map_err(|e| {
                    color_eyre::eyre::eyre!(format!(
                        "unable to merge config from {:#?} for tool {}: {e}",
                        stringified_config, tool.name
                    ))
                })?;
            }
        }
        for inject in tool.inject {
            Self::perform_injection(inject, &merged_config)?;
        }
        Ok(())
    }
}

pub struct Toml;

impl Config for Toml {
    type Value = toml::Table;

    fn format_name() -> &'static str {
        "toml"
    }

    fn parse_from_str(content: &str) -> Result<Self::Value> {
        toml::from_str(content).map_err(|e| color_eyre::eyre::eyre!("Error parsing TOML: {}", e))
    }

    fn to_string(value: &Self::Value) -> Result<String> {
        Ok(toml::Table::to_string(value))
    }

    fn merge(a: &mut Self::Value, b: Self::Value) -> Result<()> {
        crate::merge::toml(a, b)
    }
}

pub struct Yaml;

impl Config for Yaml {
    type Value = serde_yaml::Value;

    fn format_name() -> &'static str {
        "yaml"
    }

    fn parse_from_str(content: &str) -> Result<Self::Value> {
        serde_yaml::from_str(content).map_err(|e| color_eyre::eyre::eyre!("Error parsing YAML: {}", e))
    }

    fn to_string(value: &Self::Value) -> Result<String> {
        serde_yaml::to_string(value).map_err(|e| color_eyre::eyre::eyre!("Error serializing YAML: {}", e))
    }

    fn merge(a: &mut Self::Value, b: Self::Value) -> Result<()> {
        crate::merge::yaml(a, b)
    }
}

pub struct Json;

impl Config for Json {
    type Value = serde_json::Value;

    fn format_name() -> &'static str {
        "json"
    }

    fn parse_from_str(content: &str) -> Result<Self::Value> {
        serde_json::from_str(content).map_err(|e| color_eyre::eyre::eyre!("Error parsing JSON: {}", e))
    }

    fn to_string(value: &Self::Value) -> Result<String> {
        serde_json::to_string(value).map_err(|e| color_eyre::eyre::eyre!("Error serializing JSON: {}", e))
    }

    fn merge(a: &mut Self::Value, b: Self::Value) -> Result<()> {
        crate::merge::json(a, b)
    }
}
