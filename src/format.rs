use color_eyre::{
    eyre::{Context, ContextCompat},
    Result,
};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
enum DetectedFormat {
    Yaml,
    Json,
    Toml,
}

fn detect_format_from_extension(path: &PathBuf) -> Option<DetectedFormat> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(|ext| match ext.to_lowercase().as_str() {
            "yaml" | "yml" => Some(DetectedFormat::Yaml),
            "json" | "json5" | "jsonc" => Some(DetectedFormat::Json),
            "toml" | "tml" => Some(DetectedFormat::Toml),
            _ => None,
        })
}

fn detect_format_from_content(content: &str) -> Result<DetectedFormat> {
    // Try TOML first (most restrictive)
    if toml::from_str::<toml::Table>(content).is_ok() {
        return Ok(DetectedFormat::Toml);
    }

    // Try YAML (which also handles JSON)
    if serde_yaml::from_str::<serde_yaml::Value>(content).is_ok() {
        return Ok(DetectedFormat::Yaml);
    }

    Err(color_eyre::eyre::eyre!("Unable to parse content as TOML or YAML"))
}

fn parse_content_to_yaml_value(content: &str, format: DetectedFormat) -> Result<serde_yaml::Value> {
    match format {
        DetectedFormat::Yaml => {
            serde_yaml::from_str(content)
                .map_err(|e| color_eyre::eyre::eyre!("Error parsing YAML: {}", e))
        }
        DetectedFormat::Json => {
            let json_value: serde_json::Value = serde_json::from_str(content)
                .map_err(|e| color_eyre::eyre::eyre!("Error parsing JSON: {}", e))?;
            serde_yaml::to_value(json_value)
                .map_err(|e| color_eyre::eyre::eyre!("Error converting JSON to YAML: {}", e))
        }
        DetectedFormat::Toml => {
            let toml_value: toml::Table = toml::from_str(content)
                .map_err(|e| color_eyre::eyre::eyre!("Error parsing TOML: {}", e))?;
            serde_yaml::to_value(toml_value)
                .map_err(|e| color_eyre::eyre::eyre!("Error converting TOML to YAML: {}", e))
        }
    }
}

fn serialize_yaml_value_to_format(value: &serde_yaml::Value, format: DetectedFormat) -> Result<String> {
    match format {
        DetectedFormat::Yaml => {
            serde_yaml::to_string(value)
                .map_err(|e| color_eyre::eyre::eyre!("Error serializing YAML: {}", e))
        }
        DetectedFormat::Json => {
            let json_value: serde_json::Value = serde_yaml::from_value(value.clone())
                .map_err(|e| color_eyre::eyre::eyre!("Error converting YAML to JSON: {}", e))?;
            serde_json::to_string_pretty(&json_value)
                .map_err(|e| color_eyre::eyre::eyre!("Error serializing JSON: {}", e))
        }
        DetectedFormat::Toml => {
            let toml_value: toml::Table = serde_yaml::from_value(serde_yaml::to_value(value)?)
                .map_err(|e| color_eyre::eyre::eyre!("Error converting YAML to TOML: {}", e))?;
            Ok(toml_value.to_string())
        }
    }
}

pub trait Config: std::default::Default {
    fn format_name() -> &'static str;

    fn parse_from_str(content: &str) -> Result<Self>;

    fn to_string(value: &Self) -> Result<String>;

    fn to_string_with_path(value: &Self, _path: &PathBuf) -> Result<String> {
        Self::to_string(value)
    }

    fn merge(a: &mut Self, b: Self) -> Result<()>;

    fn read(path: Option<&PathBuf>, command: Option<String>) -> Result<Self> {
        let config: String = match (path, command) {
            (Some(p), None) => crate::path::read(p)?,
            (None, Some(command)) => crate::path::run_command(&command)?,
            // if the validation works correctly, these match-branches will be unreachable
            (None, None) => unreachable!("must specify either 'path' or 'command' on config"),
            (Some(_), Some(_)) => {
                unreachable!("cannot specify both 'path' and 'command' on config")
            }
        };
        Self::parse_from_str(&config).wrap_err(format!(
            "could not parse file {path:#?} as {}",
            Self::format_name()
        ))
    }

    fn write(path: &PathBuf, value: &Self) -> Result<()> {
        let path = crate::path::permissive_normalize(path);
        let parent = path
            .parent()
            .wrap_err(format!("unable to determine parent of {path:#?}"))?;
        std::fs::create_dir_all(parent)
            .wrap_err(format!("failed to create directory {parent:#?}"))?;
        std::fs::write(
            &path,
            Self::to_string_with_path(value, &path).wrap_err(format!(
                "unable to serialize and write merged {} to {path:#?}",
                Self::format_name()
            ))?,
        )
        .wrap_err(format!("unable to write merged config to {path:#?}"))
    }

    fn perform_injection(inject: crate::conf::Inject, config: &Self) -> Result<()> {
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
        let mut merged_config: Self = Self::default();
        for config in tool.configs {
            if crate::tool::should_run(&config)? {
                let additional_config: Self = match &config.config {
                    crate::conf::InjectConfig::Path { path, .. } => {
                        Self::read(Some(&path.0.clone()), None)
                    }
                    crate::conf::InjectConfig::Template { command, .. } => {
                        Self::read(None, Some(command.into()))
                    }
                }?;
                let stringified_config: String = match &config.config {
                    crate::conf::InjectConfig::Path { path, .. } => {
                        crate::path::normalize(&path.0)?.to_string_lossy().into()
                    }
                    crate::conf::InjectConfig::Template { command, .. } => command.into(),
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

impl Config for serde_yaml::Value {
    fn format_name() -> &'static str {
        "unified config"
    }

    fn parse_from_str(content: &str) -> Result<Self> {
        let format = detect_format_from_content(content)?;
        parse_content_to_yaml_value(content, format)
    }

    fn to_string(value: &Self) -> Result<String> {
        serde_yaml::to_string(value)
            .map_err(|e| color_eyre::eyre::eyre!("Error serializing YAML: {}", e))
    }

    fn to_string_with_path(value: &Self, path: &PathBuf) -> Result<String> {
        if let Some(format) = detect_format_from_extension(path) {
            serialize_yaml_value_to_format(value, format)
        } else {
            eprintln!("Warning: Unable to determine format from file extension for {}, defaulting to JSON", path.display());
            serialize_yaml_value_to_format(value, DetectedFormat::Json)
        }
    }

    fn merge(a: &mut Self, b: Self) -> Result<()> {
        crate::merge::yaml(a, b)
    }
}
