use crate::models::SourceConfig;
use serde_json;
use std::fs;
use std::path::PathBuf;

pub fn get_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".skills-manager/config.json")
}

pub fn load_custom_sources() -> Vec<SourceConfig> {
    let config_path = get_config_path();
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(sources) = config.get("custom_sources").and_then(|s| s.as_array()) {
                    return sources
                        .iter()
                        .filter_map(|s| serde_json::from_value::<SourceConfig>(s.clone()).ok())
                        .collect();
                }
            }
        }
    }
    vec![]
}

pub fn save_custom_sources(sources: &[SourceConfig]) -> Result<(), String> {
    let config_path = get_config_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let config = serde_json::json!({
        "custom_sources": sources
    });

    fs::write(&config_path, config.to_string()).map_err(|e| e.to_string())
}

pub fn get_github_token() -> Option<String> {
    let config_path = get_config_path();
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(token) = config.get("github_token").and_then(|v| v.as_str()) {
                    if !token.is_empty() {
                        return Some(token.to_string());
                    }
                }
            }
        }
    }
    std::env::var("GITHUB_TOKEN").ok()
}
