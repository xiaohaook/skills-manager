//! 尝试用 Homebrew 补齐 SKILL.md `bins` 声明的命令行依赖（macOS 为主）

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// 常见 bin 名 → Homebrew formula（无映射时假定与 bin 同名）
fn brew_formula_for_bin(bin: &str) -> String {
    match bin {
        "rg" => "ripgrep".into(),
        "fd" => "fd".into(),
        "bat" => "bat".into(),
        "gdu" => "gdu".into(),
        "eza" => "eza".into(),
        "exa" => "eza".into(),
        "jq" => "jq".into(),
        "yq" => "yq".into(),
        "ffmpeg" => "ffmpeg".into(),
        "magick" | "convert" => "imagemagick".into(),
        "gh" => "gh".into(),
        "wget" => "wget".into(),
        "fzf" => "fzf".into(),
        "delta" => "git-delta".into(),
        "hx" | "helix" => "helix".into(),
        "nvim" => "neovim".into(),
        "lazygit" => "lazygit".into(),
        "dust" => "dust".into(),
        "sd" => "sd".into(),
        "hyperfine" => "hyperfine".into(),
        "just" => "just".into(),
        "tokei" => "tokei".into(),
        "graphviz" | "dot" => "graphviz".into(),
        "pandoc" => "pandoc".into(),
        "rsync" => "rsync".into(),
        "sqlite3" => "sqlite".into(),
        "uv" => "uv".into(),
        "ruff" => "ruff".into(),
        "openclaw" => "openclaw".into(),
        other => other.to_string(),
    }
}

fn resolve_brew_executable() -> Result<std::path::PathBuf, String> {
    let candidates = [
        Path::new("/opt/homebrew/bin/brew"),
        Path::new("/usr/local/bin/brew"),
        Path::new("/home/linuxbrew/.linuxbrew/bin/brew"),
    ];
    for p in candidates {
        if p.is_file() {
            return Ok(p.to_path_buf());
        }
    }
    // 依赖 PATH（GUI 应用可能未包含 brew）
    which_brew_fallback()
}

fn which_brew_fallback() -> Result<std::path::PathBuf, String> {
    let out = std::process::Command::new("which")
        .arg("brew")
        .output()
        .map_err(|e| format!("执行 which brew 失败：{e}"))?;
    if !out.status.success() {
        return Err("未找到 Homebrew（brew）。请先安装 https://brew.sh ，或在终端配置好 PATH 后重试。".into());
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        return Err("未找到 Homebrew（brew）。".into());
    }
    Ok(PathBuf::from(s))
}

/// 将缺失的 bin 名解析为要去 `brew install` 的 formula 列表（去重）
fn formulas_for_bins(bins: &[String]) -> Vec<String> {
    let mut set: HashSet<String> = HashSet::new();
    for b in bins {
        set.insert(brew_formula_for_bin(b));
    }
    let mut v: Vec<String> = set.into_iter().collect();
    v.sort();
    v
}

/// GUI 启动时 PATH 常缺 Homebrew，与 `which` / `brew` 子进程共用
fn augmented_path_env() -> String {
    let mut path = std::env::var("PATH").unwrap_or_default();
    for prefix in ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin", "/bin"] {
        if !path.contains(prefix) {
            path = format!("{prefix}:{path}");
        }
    }
    path
}

/// 安装前探测：避免把 SKILL.md 里写错的 bins（如 `blu`）整批传给 `brew install` 导致全失败
async fn brew_formula_exists(brew: &Path, path_env: &str, formula: &str) -> bool {
    let output = tokio::process::Command::new(brew)
        .args(["info", "--formula", formula])
        .env("PATH", path_env)
        .env("HOMEBREW_NO_AUTO_UPDATE", "1")
        .env("HOMEBREW_NO_ENV_HINTS", "1")
        .output()
        .await;
    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

#[tauri::command]
pub async fn install_bins_with_homebrew(bins: Vec<String>) -> Result<String, String> {
    if bins.is_empty() {
        return Ok("没有需要安装的依赖。".into());
    }

    let brew = resolve_brew_executable()?;
    let formulas = formulas_for_bins(&bins);
    if formulas.is_empty() {
        return Ok("没有需要安装的依赖。".into());
    }

    let path_env = augmented_path_env();
    let mut to_install: Vec<String> = Vec::new();
    let mut unknown: Vec<String> = Vec::new();
    for f in formulas {
        if brew_formula_exists(&brew, &path_env, &f).await {
            to_install.push(f);
        } else {
            unknown.push(f);
        }
    }

    if to_install.is_empty() {
        return Err(format!(
            "没有在 Homebrew 里找到对应的 formula，已全部跳过，未执行安装。\n\n\
             无效的 formula 名称：{}\n\n\
             这通常说明 SKILL.md / AGENTS.md 里 **bins:** 写成了机器上不存在、或与 Homebrew 包名不一致的名字（例如截图里的 `blu` 并不是官方 formula）。\n\
             请改成真实的可执行文件名，或自行用 brew search / 官网查找正确包名后再手动安装。",
            unknown.join(", ")
        ));
    }

    let mut cmd = tokio::process::Command::new(&brew);
    cmd.arg("install");
    for f in &to_install {
        cmd.arg(f);
    }
    cmd.env("PATH", path_env.clone());
    cmd.env("HOMEBREW_NO_AUTO_UPDATE", "1");

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("执行 brew 失败：{e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        let skipped_hint = if unknown.is_empty() {
            String::new()
        } else {
            format!(
                "\n\n（另外，以下名称未找到对应 formula，已在安装前跳过：{}）",
                unknown.join(", ")
            )
        };
        return Err(format!(
            "brew install 未完成（退出码 {:?}）。{skipped_hint}\n\n{stderr}\n{stdout}",
            output.status.code()
        ));
    }

    let mut msg = format!("已请求安装：{}。", to_install.join(", "));
    if !unknown.is_empty() {
        msg.push_str(&format!(
            "\n\n已在安装前跳过（Homebrew 无对应 formula，请检查技能里 bins 是否写对或手动安装）：{}",
            unknown.join(", ")
        ));
    }
    if !stdout.trim().is_empty() || !stderr.trim().is_empty() {
        msg.push_str("\n\n");
        msg.push_str(&stdout);
        if !stderr.trim().is_empty() {
            msg.push_str(&stderr);
        }
    }

    Ok(msg)
}
