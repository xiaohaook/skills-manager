//! 扫描各 claw 技能目录
use crate::models::{ClawInstance, Skill};
use crate::skill_md::{extract_array, extract_field, missing_bins_list};
use crate::symlink_index::{build_symlink_ref_index, ref_count_from_symlink_index};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[tauri::command]
pub fn scan_skills(claws: Vec<ClawInstance>) -> Result<Vec<Skill>, String> {
    let symlink_index = build_symlink_ref_index();
    let mut all_skills = Vec::new();
    let mut seen_paths = HashSet::new();

    for claw in &claws {
        let skills_dir = PathBuf::from(&claw.skills_path);
        if let Ok(skills) = scan_skills_in_dir(&skills_dir, &claw.name, &claws, &symlink_index) {
            for skill in skills {
                if seen_paths.insert(skill.path.clone()) {
                    all_skills.push(skill);
                }
            }
        }
    }

    Ok(all_skills)
}

fn count_other_claw_installs(skill_name: &str, skill_source: &str, claws: &[ClawInstance]) -> u32 {
    let mut n = 0u32;
    for claw in claws {
        if claw.name == skill_source {
            continue;
        }
        let p = PathBuf::from(&claw.skills_path).join(skill_name);
        if p.exists() {
            n += 1;
        }
    }
    n
}

fn scan_skills_in_dir(
    skills_dir: &PathBuf,
    source: &str,
    claws: &[ClawInstance],
    symlink_index: &HashMap<String, HashSet<String>>,
) -> Result<Vec<Skill>, String> {
    let mut skills = Vec::new();

    if let Ok(entries) = fs::read_dir(skills_dir) {
        for entry in entries.flatten() {
            let skill_dir = entry.path();
            let skill_md = skill_dir.join("SKILL.md");
            let agents_md = skill_dir.join("AGENTS.md");

            if let Ok(metadata) = fs::symlink_metadata(&skill_dir) {
                if metadata.file_type().is_symlink() {
                    continue;
                }
            }

            if skill_md.exists() {
                if let Ok(content) = fs::read_to_string(&skill_md) {
                    let name = skill_dir
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let description = extract_field(&content, "description")
                        .unwrap_or_else(|| "No description".to_string());

                    let emoji = extract_field(&content, "emoji").unwrap_or_else(|| "📦".to_string());

                    let requires = extract_array(&content, "bins");
                    let missing_bins = missing_bins_list(&requires);
                    let ready = requires.is_empty() || missing_bins.is_empty();

                    let ref_count = ref_count_from_symlink_index(&skill_dir, symlink_index);
                    let other_claw_count = count_other_claw_installs(&name, source, claws);

                    skills.push(Skill {
                        name,
                        description,
                        emoji,
                        path: skill_dir.to_string_lossy().to_string(),
                        requires,
                        missing_bins,
                        ready,
                        source: source.to_string(),
                        ref_count,
                        other_claw_count,
                    });
                }
            } else if agents_md.exists() {
                if let Ok(content) = fs::read_to_string(&agents_md) {
                    let name = skill_dir
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let description = extract_field(&content, "description")
                        .or_else(|| {
                            content
                                .lines()
                                .map(|l| l.trim())
                                .find(|l| {
                                    !l.is_empty()
                                        && !l.starts_with('#')
                                        && !l.starts_with("---")
                                        && !l.starts_with("```")
                                })
                                .map(|l| l.chars().take(160).collect::<String>())
                        })
                        .unwrap_or_else(|| "AGENTS.md 项目".to_string());
                    let emoji = extract_field(&content, "emoji").unwrap_or_else(|| "📋".to_string());
                    let requires = extract_array(&content, "bins");
                    let missing_bins = missing_bins_list(&requires);
                    let ready = requires.is_empty() || missing_bins.is_empty();
                    let ref_count = ref_count_from_symlink_index(&skill_dir, symlink_index);
                    let other_claw_count = count_other_claw_installs(&name, source, claws);
                    skills.push(Skill {
                        name,
                        description,
                        emoji,
                        path: skill_dir.to_string_lossy().to_string(),
                        requires,
                        missing_bins,
                        ready,
                        source: source.to_string(),
                        ref_count,
                        other_claw_count,
                    });
                }
            }
        }
    }

    Ok(skills)
}

#[tauri::command]
pub fn scan_remote_claw(remote_path: String) -> Result<Vec<Skill>, String> {
    let skills_dir = PathBuf::from(&remote_path);
    if !skills_dir.exists() {
        return Err(format!("Remote path does not exist: {}", remote_path));
    }
    let symlink_index = build_symlink_ref_index();
    scan_skills_in_dir(&skills_dir, "Remote", &[], &symlink_index)
}
