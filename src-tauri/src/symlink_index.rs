use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

/// 家目录下已知 skills 布局：软链目标路径字符串（trim 尾 `/`）→ 哪些来源名称指向它。
pub fn build_symlink_ref_index() -> HashMap<String, HashSet<String>> {
    let mut index: HashMap<String, HashSet<String>> = HashMap::new();
    let Ok(home) = std::env::var("HOME") else {
        return index;
    };
    let home_path = PathBuf::from(&home);
    let Ok(entries) = fs::read_dir(&home_path) else {
        return index;
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }
        let dir_name = entry.file_name().to_string_lossy().to_string();
        let skills_dirs: Vec<PathBuf> = if entry_path.join("skills").exists() {
            vec![entry_path.join("skills")]
        } else if dir_name.contains("skills") {
            vec![entry_path.clone()]
        } else {
            vec![]
        };
        let formatted_source = format_claw_source_label(&dir_name);

        for skills_subdir in skills_dirs {
            if !skills_subdir.exists() || !skills_subdir.is_dir() {
                continue;
            }
            let Ok(skill_entries) = fs::read_dir(&skills_subdir) else {
                continue;
            };
            for skill_entry in skill_entries.flatten() {
                let skill_path = skill_entry.path();
                let Ok(metadata) = fs::symlink_metadata(&skill_path) else {
                    continue;
                };
                if !metadata.file_type().is_symlink() {
                    continue;
                }
                let Ok(link_target) = fs::read_link(&skill_path) else {
                    continue;
                };
                let normalized_link = link_target
                    .to_string_lossy()
                    .trim_end_matches('/')
                    .to_string();
                if normalized_link.is_empty() {
                    continue;
                }
                index
                    .entry(normalized_link)
                    .or_default()
                    .insert(formatted_source.clone());
            }
        }
    }
    index
}

fn format_claw_source_label(dir_name: &str) -> String {
    if dir_name.starts_with(".cursor") || dir_name.starts_with("cursor") {
        "Cursor".to_string()
    } else if dir_name.starts_with(".claude") || dir_name.starts_with("claude") {
        "Claude Code".to_string()
    } else if dir_name.starts_with(".openclaw") || dir_name.starts_with("openclaw") {
        "OpenClaw".to_string()
    } else if dir_name.starts_with(".dewuclaw") || dir_name.starts_with("dewuclaw") {
        "DewuClaw".to_string()
    } else if dir_name.starts_with(".comate") || dir_name.starts_with("comate") {
        "Comate".to_string()
    } else if dir_name.starts_with(".codex") || dir_name.starts_with("codex") {
        "Codex".to_string()
    } else {
        dir_name.to_string()
    }
}

fn normalized_skill_dir_key(skill_dir: &PathBuf) -> String {
    let real_target = if let Ok(real) = skill_dir.canonicalize() {
        real
    } else {
        skill_dir.clone()
    };
    real_target
        .to_string_lossy()
        .trim_end_matches('/')
        .to_string()
}

pub fn ref_count_from_symlink_index(
    skill_dir: &PathBuf,
    index: &HashMap<String, HashSet<String>>,
) -> u32 {
    let key = normalized_skill_dir_key(skill_dir);
    index
        .get(&key)
        .map(|s| s.len() as u32)
        .unwrap_or(0)
}
