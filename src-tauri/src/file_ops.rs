//! 复制 / 移动 / 删除 / 打开路径 / 存在性检查
use std::fs;
use std::path::PathBuf;

#[tauri::command]
pub fn copy_skill(
    sourcePath: String,
    targetPath: String,
    skillName: String,
) -> Result<String, String> {
    let src_dir = PathBuf::from(&sourcePath);
    let dst_dir = PathBuf::from(&targetPath).join(&skillName);

    if !src_dir.exists() {
        return Err(format!("Source skill not found: {}", sourcePath));
    }

    if dst_dir.exists() {
        fs::remove_dir_all(&dst_dir)
            .map_err(|e| format!("Failed to remove existing directory: {}", e))?;
    }

    match std::os::unix::fs::symlink(&src_dir, &dst_dir) {
        Ok(_) => Ok(format!("Successfully linked {} to {}", skillName, targetPath)),
        Err(_link_err) => {
            fs::create_dir_all(&dst_dir)
                .map_err(|e| format!("Failed to create destination: {}", e))?;
            copy_dir_recursive(&src_dir, &dst_dir)?;
            Ok(format!("Successfully copied {} to {}", skillName, targetPath))
        }
    }
}

pub fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
    if !src.exists() {
        return Err(format!("Source does not exist: {:?}", src));
    }

    if !dst.exists() {
        fs::create_dir_all(dst).map_err(|e| format!("Failed to create dir: {}", e))?;
    }

    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy file: {}", e))?;
        }
    }

    Ok(())
}

/// 同盘则 rename（极快），跨盘或失败则回退为复制
pub fn move_tree_or_copy(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
    if dst.exists() {
        fs::remove_dir_all(dst).map_err(|e| format!("移除旧技能目录失败：{}", e))?;
    }
    match fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(_) => {
            copy_dir_recursive(src, dst)?;
            fs::remove_dir_all(src).map_err(|e| format!("清理临时克隆目录失败：{}", e))?;
            Ok(())
        }
    }
}

#[tauri::command]
pub fn show_in_finder(path: String) -> Result<(), String> {
    let output = std::process::Command::new("open").arg("-R").arg(&path).output();

    match output {
        Ok(out) => {
            if out.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                Err(format!("Finder 显示失败：{}", stderr))
            }
        }
        Err(e) => Err(format!("执行失败：{}", e)),
    }
}

#[tauri::command]
pub fn delete_skill(path: String) -> Result<String, String> {
    let skill_dir = PathBuf::from(&path);
    if !skill_dir.exists() {
        return Err(format!("Skill not found: {}", path));
    }

    let real_path = if let Ok(real) = skill_dir.canonicalize() {
        real
    } else {
        skill_dir.clone()
    };
    let normalized_real_path = real_path.to_string_lossy().to_string();

    fs::remove_dir_all(&skill_dir)
        .map_err(|e| format!("Failed to delete skill: {}", e))?;

    cleanup_broken_links(&normalized_real_path)?;

    Ok(format!("Successfully deleted skill: {}", path))
}

fn cleanup_broken_links(deleted_path: &str) -> Result<(), String> {
    if let Ok(home) = std::env::var("HOME") {
        let home_path = PathBuf::from(&home);

        if let Ok(entries) = fs::read_dir(&home_path) {
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

                for skills_dir in skills_dirs {
                    if !skills_dir.exists() || !skills_dir.is_dir() {
                        continue;
                    }

                    if let Ok(skill_entries) = fs::read_dir(&skills_dir) {
                        for skill_entry in skill_entries.flatten() {
                            let skill_path = skill_entry.path();

                            if let Ok(metadata) = fs::symlink_metadata(&skill_path) {
                                if metadata.file_type().is_symlink() {
                                    if let Ok(link_target) = fs::read_link(&skill_path) {
                                        let link_target_str = link_target.to_string_lossy().to_string();

                                        if link_target_str == deleted_path {
                                            fs::remove_file(&skill_path).map_err(|e| {
                                                format!("Failed to remove broken link: {}", e)
                                            })?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub fn open_file_in_editor(path: String) -> Result<(), String> {
    let output = std::process::Command::new("open")
        .arg(&path)
        .output()
        .map_err(|e| format!("Failed to open file: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to open file: {}", stderr));
    }

    Ok(())
}

#[tauri::command]
pub fn check_path_exists(path: String) -> bool {
    PathBuf::from(&path).exists()
}

#[tauri::command]
pub fn is_symlink(path: String) -> bool {
    let path_buf = PathBuf::from(&path);
    if let Ok(metadata) = fs::symlink_metadata(&path_buf) {
        metadata.file_type().is_symlink()
    } else {
        false
    }
}
