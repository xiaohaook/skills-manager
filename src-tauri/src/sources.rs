//! 自定义来源与 GitHub Token（写入 config）
use crate::config;
use crate::models::SourceConfig;
use serde_json;
use std::fs;
use std::path::PathBuf;

#[tauri::command]
pub fn add_custom_source(name: String, path: String) -> Result<String, String> {
    let mut sources = config::load_custom_sources();

    if sources.iter().any(|s| s.name == name || s.path == path) {
        return Err("Source already exists".to_string());
    }

    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    sources.push(SourceConfig {
        name,
        path,
        enabled: true,
    });

    config::save_custom_sources(&sources)?;
    Ok("Source added successfully".to_string())
}

#[tauri::command]
pub fn remove_custom_source(name: String) -> Result<String, String> {
    let mut sources = config::load_custom_sources();
    sources.retain(|s| s.name != name);
    config::save_custom_sources(&sources)?;
    Ok("Source removed successfully".to_string())
}

#[tauri::command]
pub fn get_custom_sources() -> Result<Vec<SourceConfig>, String> {
    Ok(config::load_custom_sources())
}

#[tauri::command]
pub fn set_github_token(token: String) -> Result<String, String> {
    let config_path = config::get_config_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut cfg_doc = if config_path.exists() {
        fs::read_to_string(&config_path)
            .ok()
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
            .unwrap_or_else(|| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if token.is_empty() {
        if let Some(obj) = cfg_doc.as_object_mut() {
            obj.remove("github_token");
        }
    } else {
        cfg_doc["github_token"] = serde_json::json!(token);
    }

    fs::write(&config_path, cfg_doc.to_string()).map_err(|e| e.to_string())?;

    if token.is_empty() {
        Ok("GitHub Token 已清除".to_string())
    } else {
        Ok("GitHub Token 已配置（认证请求限流 5000 次/小时）".to_string())
    }
}
