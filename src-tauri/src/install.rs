//! 从 Git 仓库安装技能
use crate::config;
use crate::file_ops::move_tree_or_copy;
use std::fs;
use std::path::PathBuf;

fn resolve_install_target_path(raw: &str) -> Result<PathBuf, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("安装目标路径为空".to_string());
    }
    if let Some(rest) = raw.strip_prefix("~/") {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| "无法解析用户主目录，请在「来源」中配置技能目录或使用绝对路径".to_string())?;
        return Ok(PathBuf::from(home).join(rest));
    }
    if raw == "~" {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| "无法解析用户主目录".to_string())?;
        return Ok(PathBuf::from(home));
    }
    Ok(PathBuf::from(raw))
}

fn infer_skill_folder_name(repo_url: &str, skill_dir: &PathBuf, staging: &PathBuf) -> String {
    let dir_name = skill_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("skill");
    let is_staging_root = skill_dir == staging;
    let looks_like_temp = dir_name.starts_with(".skills-manager-clone-")
        || dir_name.starts_with("skills-manager-");
    let generic_skills_folder =
        dir_name == "skills" && skill_dir.parent().map(|p| p == staging).unwrap_or(false);

    if !is_staging_root && !looks_like_temp && !generic_skills_folder && !dir_name.is_empty() {
        return dir_name.to_string();
    }

    let cleaned = strip_url_query_and_fragment(repo_url);
    let base = cleaned
        .trim()
        .trim_end_matches('/')
        .trim_end_matches(".git");
    let name = base
        .rsplit('/')
        .find(|s| !s.is_empty())
        .unwrap_or("skill")
        .to_string();
    if name.is_empty() {
        "skill".to_string()
    } else {
        name
    }
}

/// Git 会把「Cloning into …」打在 stderr 最前面，真正原因多在 `fatal:` / `error:` 或末尾行
fn git_clone_stderr_hint(stderr: &str) -> String {
    let lines: Vec<String> = stderr
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    let is_clone_progress = |s: &str| {
        let t = s.trim();
        t.starts_with("Cloning into")
            || t.starts_with("正克隆到")
            || (t.starts_with("remote:") && !t.to_lowercase().contains("fatal"))
    };

    let meaningful: Vec<&String> = lines.iter().filter(|l| !is_clone_progress(l)).collect();

    if let Some(line) = meaningful.iter().find(|l| l.to_lowercase().contains("fatal:")) {
        return format!("\n{}", line);
    }
    if let Some(line) = meaningful
        .iter()
        .find(|l| l.to_lowercase().contains("error:"))
    {
        return format!("\n{}", line);
    }

    let tail: Vec<&str> = meaningful
        .iter()
        .rev()
        .take(5)
        .rev()
        .map(|s| s.as_str())
        .collect();
    if !tail.is_empty() {
        return format!("\n{}", tail.join("\n"));
    }

    let raw = stderr.trim();
    if raw.is_empty() {
        return String::new();
    }
    const MAX: usize = 800;
    if raw.len() > MAX {
        format!("\n{}…", &raw[..MAX])
    } else {
        format!("\n{}", raw)
    }
}

fn stderr_suggests_network(stderr: &str) -> bool {
    let s = stderr.to_lowercase();
    s.contains("could not resolve host")
        || s.contains("connection timed out")
        || s.contains("connection refused")
        || s.contains("network is unreachable")
        || s.contains("operation timed out")
        || s.contains("failed to connect")
        || s.contains("could not read from remote repository")
        || s.contains("empty reply from server")
        || s.contains("connection reset")
        || s.contains("ssl_connect")
        || s.contains("gnutls")
        || s.contains("secure connection")
        || s.contains("certificate verify failed")
        || s.contains("ssl certificate problem")
        || s.contains("http2 framing")
        || s.contains("error in the http2")
}

/// 克隆到「技能目录」旁的隐藏 staging，完成后 rename 到目标（同卷时避免整树复制）
fn git_clone_install_sync(
    clone_url: String,
    repo_url: String,
    target_base: PathBuf,
) -> Result<String, String> {
    use std::process::{Command, Stdio};

    fs::create_dir_all(&target_base).map_err(|e| format!("无法创建技能目录：{}", e))?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let staging = target_base.join(format!(".skills-manager-clone-{}", timestamp));
    if staging.exists() {
        let _ = fs::remove_dir_all(&staging);
    }

    // http.version=HTTP/1.1：规避 libcurl 在 HTTP/2 下常见的「Error in the HTTP2 framing layer」（代理/部分网络）
    // 不使用 git protocol v2：与上文解耦，减少旧环境异常
    let output = Command::new("git")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_LFS_SKIP_SMUDGE", "1")
        .arg("-c")
        .arg("core.compression=0")
        .arg("-c")
        .arg("http.version=HTTP/1.1")
        .arg("clone")
        .arg("--depth=1")
        .arg("--single-branch")
        .arg("--no-tags")
        .arg("--recurse-submodules=no")
        .arg(&clone_url)
        .arg(&staging)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| {
            let _ = fs::remove_dir_all(&staging);
            format!("Git clone 启动失败：{}", e)
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = fs::remove_dir_all(&staging);

        if stderr_suggests_network(&stderr) {
            let hint = git_clone_stderr_hint(&stderr);
            return Err(format!(
                "网络错误：无法连接远程 Git 仓库。{}\n\n💡 请检查：本机网络与 DNS；系统/Clash 等代理是否开启；Git 是否走代理（可试 `git config --global http.proxy` / 环境变量 HTTPS_PROXY）。桌面应用有时不会自动继承终端代理。",
                hint
            ));
        }
        let hint = git_clone_stderr_hint(&stderr);
        return Err(format!(
            "Git clone 失败，请检查仓库地址、私有仓库权限或网络。{}",
            hint
        ));
    }

    let skill_dir = match find_skill_directory(&staging) {
        Ok(dir) => dir,
        Err(_) => {
            let _ = fs::remove_dir_all(&staging);
            return Err(format!(
                "仓库 {} 不可作为技能安装（未找到 SKILL.md 或 AGENTS.md）\n\n💡 OpenClaw 标准技能一般为 SKILL.md；部分 Agent 项目仅有 AGENTS.md，二者满足其一即可从热榜安装。",
                repo_url
            ));
        }
    };

    let skill_name = infer_skill_folder_name(&repo_url, &skill_dir, &staging);
    let dest_dir = target_base.join(&skill_name);

    if skill_dir == staging {
        move_tree_or_copy(&staging, &dest_dir)?;
    } else {
        move_tree_or_copy(&skill_dir, &dest_dir)?;
        let _ = fs::remove_dir_all(&staging);
    }

    Ok(format!("Successfully installed {} from {}", skill_name, repo_url))
}

// 从远程 Git 仓库安装技能（GitHub / GitLab / Codeberg / Hugging Face 等 HTTPS 地址）
#[tauri::command]
pub async fn install_from_github(repo_url: String, target_path: String) -> Result<String, String> {
    use tokio::task::spawn_blocking;

    let target_base = resolve_install_target_path(&target_path)?;
    let clone_url = resolve_clone_url(&repo_url)?;

    spawn_blocking(move || git_clone_install_sync(clone_url, repo_url, target_base))
        .await
        .map_err(|_| "Git 安装线程执行失败".to_string())?
}

fn parse_github_repo(url: &str) -> Result<(String, String), String> {
    let url = url.trim();
    
    // 处理 author/repo 格式
    if !url.starts_with("http") {
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() == 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
        return Err("Invalid repo format. Use 'author/repo' or full GitHub URL".to_string());
    }
    
    // 处理完整 URL 格式
    let url = url.trim_end_matches('/').trim_end_matches(".git");
    let parts: Vec<&str> = url.split('/').collect();
    
    if parts.len() >= 5 && parts[2] == "github.com" {
        return Ok((parts[3].to_string(), parts[4].to_string()));
    }
    
    Err("Invalid GitHub URL".to_string())
}

fn github_authenticated_clone_url(owner: &str, repo: &str) -> String {
    if let Some(token) = config::get_github_token() {
        format!("https://{}@github.com/{}/{}.git", token, owner, repo)
    } else {
        format!("https://github.com/{}/{}.git", owner, repo)
    }
}

fn strip_url_query_and_fragment(url: &str) -> String {
    let s = url.trim();
    let s = s.split('#').next().unwrap_or(s);
    s.split('?').next().unwrap_or(s).trim().to_string()
}

/// 解析「技能来源」字段中的地址为可 git clone 的 URL（支持 GitHub / GitLab / Codeberg / Hugging Face 等）
fn resolve_clone_url(repo_url: &str) -> Result<String, String> {
    let cleaned = strip_url_query_and_fragment(repo_url);
    let trimmed = cleaned.trim();
    if !trimmed.to_lowercase().starts_with("http") {
        let (owner, repo) = parse_github_repo(trimmed)?;
        return Ok(github_authenticated_clone_url(&owner, &repo));
    }
    let lower = trimmed.to_lowercase();
    let base = trimmed.trim_end_matches('/').trim_end_matches(".git");
    if lower.contains("github.com") {
        let (owner, repo) = parse_github_repo(trimmed)?;
        return Ok(github_authenticated_clone_url(&owner, &repo));
    }
    Ok(format!("{}.git", base))
}

/// OpenClaw 约定为 SKILL.md；不少 Cursor / Agent 仓库仅提供 AGENTS.md（如 mcporter）
fn dir_has_skill_manifest(dir: &PathBuf) -> bool {
    dir.join("SKILL.md").exists() || dir.join("AGENTS.md").exists()
}

fn find_skill_directory(dir: &PathBuf) -> Result<PathBuf, String> {
    if dir_has_skill_manifest(dir) {
        return Ok(dir.clone());
    }

    // 多技能目录：整夹拷贝（内部子项各自可有 SKILL.md）
    let skills_dir = dir.join("skills");
    if skills_dir.exists() {
        return Ok(skills_dir);
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if dir_has_skill_manifest(&path) {
                    return Ok(path);
                }
                if let Ok(found) = find_skill_directory(&path) {
                    return Ok(found);
                }
            }
        }
    }

    Err("No SKILL.md or AGENTS.md found in repository".to_string())
}

