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

fn parse_from_str_with_path(content: &str, path: Option<&PathBuf>) -> Result<serde_yaml::Value> {
    let format = if let Some(p) = path {
        if let Some(ext_format) = detect_format_from_extension(p) {
            ext_format
        } else {
            detect_format_from_content(content)?
        }
    } else {
        detect_format_from_content(content)?
    };

    parse_content_to_yaml_value(content, format)
}

fn to_string_with_path(value: &serde_yaml::Value, path: &PathBuf) -> Result<String> {
    if let Some(format) = detect_format_from_extension(path) {
        serialize_yaml_value_to_format(value, format)
    } else {
        eprintln!("Warning: Unable to determine format from file extension for {}, defaulting to JSON", path.display());
        serialize_yaml_value_to_format(value, DetectedFormat::Json)
    }
}

pub fn read(path: Option<&PathBuf>, command: Option<String>) -> Result<serde_yaml::Value> {
    let config: String = match (&path, &command) {
        (Some(p), None) => crate::path::read(p)?,
        (None, Some(command)) => crate::path::run_command(command)?,
        // if the validation works correctly, these match-branches will be unreachable
        (None, None) => unreachable!("must specify either 'path' or 'command' on config"),
        (Some(_), Some(_)) => {
            unreachable!("cannot specify both 'path' and 'command' on config")
        }
    };

    match (path, command) {
        (Some(p), None) => parse_from_str_with_path(&config, Some(p)),
        (None, Some(_)) => parse_from_str_with_path(&config, None),
        _ => unreachable!(),
    }.wrap_err(format!(
        "could not parse file {path:#?}"
    ))
}

pub fn write(path: &PathBuf, value: &serde_yaml::Value) -> Result<()> {
    let path = crate::path::permissive_normalize(path);
    let parent = path
        .parent()
        .wrap_err(format!("unable to determine parent of {path:#?}"))?;
    std::fs::create_dir_all(parent)
        .wrap_err(format!("failed to create directory {parent:#?}"))?;
    std::fs::write(
        &path,
        to_string_with_path(value, &path).wrap_err(format!(
            "unable to serialize and write merged unified config to {path:#?}"
        ))?,
    )
    .wrap_err(format!("unable to write merged config to {path:#?}"))
}

pub fn perform_injection(inject: crate::conf::Inject, config: &serde_yaml::Value) -> Result<()> {
    write(&inject.path, config)?;
    if let Some(env_name) = inject.env_name {
        println!(
            "export {}={:#?}",
            env_name,
            crate::path::normalize(&inject.path)?.to_string_lossy()
        );
    }
    Ok(())
}

pub fn handle_tool(tool: crate::conf::Tool) -> Result<()> {
    let mut merged_config: serde_yaml::Value = serde_yaml::Value::default();
    for config in tool.configs {
        if crate::tool::should_run(&config)? {
            let additional_config: serde_yaml::Value = match &config.config {
                crate::conf::InjectConfig::Path { path, .. } => {
                    read(Some(&path.0.clone()), None)
                }
                crate::conf::InjectConfig::Template { command, .. } => {
                    read(None, Some(command.into()))
                }
            }?;
            let stringified_config: String = match &config.config {
                crate::conf::InjectConfig::Path { path, .. } => {
                    crate::path::normalize(&path.0)?.to_string_lossy().into()
                }
                crate::conf::InjectConfig::Template { command, .. } => command.into(),
            };

            crate::merge::yaml(&mut merged_config, additional_config).map_err(|e| {
                color_eyre::eyre::eyre!(format!(
                    "unable to merge config from {:#?} for tool {}: {e}",
                    stringified_config, tool.name
                ))
            })?;
        }
    }
    for inject in tool.inject {
        perform_injection(inject, &merged_config)?;
    }
    Ok(())
}
