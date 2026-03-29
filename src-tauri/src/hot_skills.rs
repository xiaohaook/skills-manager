//! 社区热榜内置列表、缓存与合并逻辑
use crate::models::HotSkill;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// 获取热榜技能（支持多平台：GitHub + ClawHub + 内置）

/// 热榜内置列表或平台字段逻辑变更时递增，缓存文件名随之变化，避免长期命中旧 JSON。
const HOT_SKILLS_CACHE_VERSION: u32 = 4;

/// 热榜合并：每平台最多保留条数（按 stars），再全局排序截断，避免 GitHub 大包独占列表。
const HOT_LIST_PER_PLATFORM_CAP: usize = 25;
const HOT_LIST_MAX_TOTAL: usize = 120;

pub(crate) fn merge_hot_skills_with_platform_caps(skills: Vec<HotSkill>) -> Vec<HotSkill> {
    let mut by_platform: HashMap<String, Vec<HotSkill>> = HashMap::new();
    for skill in skills {
        let p = skill.platform.clone();
        by_platform.entry(p).or_default().push(skill);
    }
    let mut picked: Vec<HotSkill> = Vec::new();
    for mut list in by_platform.into_values() {
        list.sort_by(|a, b| b.stars.cmp(&a.stars));
        list.truncate(HOT_LIST_PER_PLATFORM_CAP);
        picked.extend(list);
    }
    picked.sort_by(|a, b| b.stars.cmp(&a.stars));
    let mut seen = HashSet::new();
    picked.retain(|s| seen.insert(s.id.clone()));
    picked.sort_by(|a, b| b.stars.cmp(&a.stars));
    picked.truncate(HOT_LIST_MAX_TOTAL);
    picked
}

// 获取热榜技能（支持多平台：GitHub + ClawHub）
#[tauri::command]
pub async fn get_hot_skills() -> Result<Vec<HotSkill>, String> {
    let cache_dir = get_cache_dir();
    let cache_file = cache_dir.join(format!("hot_skills_v{}.json", HOT_SKILLS_CACHE_VERSION));
    
    // 尝试读取缓存（有效期 1 小时）
    if cache_file.exists() {
        if let Ok(metadata) = fs::metadata(&cache_file) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(age_secs) = modified.duration_since(UNIX_EPOCH) {
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                    if now - age_secs.as_secs() < 3600 {
                        if let Ok(content) = fs::read_to_string(&cache_file) {
                            if let Ok(mut cached) = serde_json::from_str::<Vec<HotSkill>>(&content) {
                                normalize_hot_skills_list(&mut cached);
                                return Ok(cached);
                            }
                        }
                    }
                }
            }
        }
    }
    
    let mut all_skills = Vec::new();
    
    // 获取 GitHub 热门技能（内置 fallback）
    let github_skills = get_fallback_hot_skills();
    for mut skill in github_skills {
        normalize_hot_skill_platform(&mut skill);
        all_skills.push(skill);
    }
    
    all_skills = merge_hot_skills_with_platform_caps(all_skills);
    
    // 更新缓存
    fs::create_dir_all(&cache_dir).ok();
    if let Ok(json) = serde_json::to_string_pretty(&all_skills) {
        if fs::write(&cache_file, json).is_ok() {
            // 旧版固定名缓存；若仍存在会挡住用户对「新热榜」的感知
            let legacy = cache_dir.join("hot_skills.json");
            let _ = fs::remove_file(legacy);
        }
    }
    
    Ok(all_skills)
}

#[tauri::command]
pub fn clear_hot_skills_cache() -> Result<(), String> {
    let cache_dir = get_cache_dir();
    let cache_file = cache_dir.join(format!("hot_skills_v{}.json", HOT_SKILLS_CACHE_VERSION));
    if cache_file.exists() {
        fs::remove_file(&cache_file).map_err(|e| e.to_string())?;
    }
    let legacy = cache_dir.join("hot_skills.json");
    if legacy.exists() {
        let _ = fs::remove_file(legacy);
    }
    Ok(())
}

pub fn get_cache_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".skills-manager/cache")
}

/// 与前端筛选一致；可按 URL 推断 gitlab / codeberg / huggingface
pub(crate) fn normalize_hot_skill_platform(skill: &mut HotSkill) {
    let p = skill.platform.trim().to_lowercase();
    match p.as_str() {
        "clawhub" => {
            skill.platform = "clawhub".to_string();
            return;
        }
        "cursor" => {
            skill.platform = "cursor".to_string();
            return;
        }
        "gitlab" => {
            skill.platform = "gitlab".to_string();
            return;
        }
        "codeberg" => {
            skill.platform = "codeberg".to_string();
            return;
        }
        "huggingface" | "hf" => {
            skill.platform = "huggingface".to_string();
            return;
        }
        _ => {}
    }
    let url = skill.github_url.to_lowercase();
    if url.contains("gitlab.com") {
        skill.platform = "gitlab".to_string();
        return;
    }
    if url.contains("codeberg.org") {
        skill.platform = "codeberg".to_string();
        return;
    }
    if url.contains("huggingface.co") {
        skill.platform = "huggingface".to_string();
        return;
    }
    skill.platform = "github".to_string();
}

pub(crate) fn normalize_hot_skills_list(skills: &mut [HotSkill]) {
    for skill in skills.iter_mut() {
        normalize_hot_skill_platform(skill);
    }
}


pub(crate) fn get_fallback_hot_skills() -> Vec<HotSkill> {
    // GitHub + ClawHub 热门技能
    vec![
        // 🦞 ClawHub 热门技能 (按下载量排序)
        HotSkill {
            id: "self-improving-agent".to_string(),
            name: "self-improving-agent".to_string(),
            description: "自我改进 Agent | 捕获错误和修正，持续学习和改进".to_string(),
            emoji: "🔄".to_string(),
            author: "pskoett".to_string(),
            stars: 2800,
            installs: 320000,
            tags: vec!["AI".to_string(), "Learning".to_string(), "Automation".to_string()],
            github_url: "https://github.com/pskoett/self-improving-agent".to_string(),
            platform: "clawhub".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "ontology".to_string(),
            name: "ontology".to_string(),
            description: "本体知识图谱 | 结构化 Agent 记忆和可组合技能".to_string(),
            emoji: "🕸️".to_string(),
            author: "oswalpalash".to_string(),
            stars: 419,
            installs: 140000,
            tags: vec!["AI".to_string(), "Knowledge".to_string(), "Graph".to_string()],
            github_url: "https://github.com/oswalpalash/ontology".to_string(),
            platform: "clawhub".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "self-improving-proactive".to_string(),
            name: "self-improving-proactive".to_string(),
            description: "自我反思 + 自我批评 + 自我学习 + 自组织记忆".to_string(),
            emoji: "🤖".to_string(),
            author: "ivangdavila".to_string(),
            stars: 740,
            installs: 128000,
            tags: vec!["AI".to_string(), "Learning".to_string(), "Proactive".to_string()],
            github_url: "https://github.com/ivangdavila/self-improving-proactive".to_string(),
            platform: "clawhub".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "nano-banana-pro".to_string(),
            name: "nano-banana-pro".to_string(),
            description: "图像生成/编辑 | 使用 Gemini 3 Pro Image，支持 1K/2K/4K".to_string(),
            emoji: "🍌".to_string(),
            author: "steipete".to_string(),
            stars: 288,
            installs: 71400,
            tags: vec!["Media".to_string(), "Image".to_string(), "AI".to_string()],
            github_url: "https://github.com/steipete/nano-banana-pro".to_string(),
            platform: "clawhub".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "obsidian".to_string(),
            name: "obsidian".to_string(),
            description: "Obsidian 知识库 | 操作 Obsidian vault，自动化笔记管理".to_string(),
            emoji: "📝".to_string(),
            author: "steipete".to_string(),
            stars: 280,
            installs: 68400,
            tags: vec!["Productivity".to_string(), "Knowledge".to_string(), "Markdown".to_string()],
            github_url: "https://github.com/steipete/obsidian".to_string(),
            platform: "clawhub".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "api-gateway".to_string(),
            name: "api-gateway".to_string(),
            description: "API 网关 | 连接 100+ API (Google/Microsoft/GitHub/Notion 等)".to_string(),
            emoji: "🔌".to_string(),
            author: "byungkyu".to_string(),
            stars: 285,
            installs: 57600,
            tags: vec!["Backend".to_string(), "API".to_string(), "Integration".to_string()],
            github_url: "https://github.com/byungkyu/api-gateway".to_string(),
            platform: "clawhub".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "agent-browser".to_string(),
            name: "agent-browser".to_string(),
            description: "无头浏览器自动化 | 为 AI Agent 优化的 CLI 工具".to_string(),
            emoji: "🌐".to_string(),
            author: "matrixy".to_string(),
            stars: 180,
            installs: 53900,
            tags: vec!["Frontend".to_string(), "Automation".to_string(), "Browser".to_string()],
            github_url: "https://github.com/matrixy/agent-browser".to_string(),
            platform: "clawhub".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "mcporter-official".to_string(),
            name: "mcporter-official".to_string(),
            description: "MCP 服务器管理 | 配置、认证和调用 MCP 工具".to_string(),
            emoji: "🔧".to_string(),
            author: "steipete".to_string(),
            stars: 144,
            installs: 48100,
            tags: vec!["Backend".to_string(), "MCP".to_string(), "DevOps".to_string()],
            github_url: "https://github.com/steipete/mcporter".to_string(),
            platform: "clawhub".to_string(),
            large_clone: false,
        },
        // 🐙 GitHub 热门技能 (按 stars 排序)
        HotSkill {
            id: "notebooklm-py".to_string(),
            name: "notebooklm-py".to_string(),
            description: "Google NotebookLM Python API | 支持 CLI 和 AI Agent 调用".to_string(),
            emoji: "🐍".to_string(),
            author: "teng-lin".to_string(),
            stars: 8041,
            installs: 12000,
            tags: vec!["Python".to_string(), "API".to_string(), "CLI".to_string(), "AI".to_string()],
            github_url: "https://github.com/teng-lin/notebooklm-py".to_string(),
            platform: "cursor".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "Auto-claude-code-research".to_string(),
            name: "Auto-claude-code-research-in-sleep".to_string(),
            description: "ARIS — 自动化 ML 研究的 Markdown 技能 | 跨模型审查、实验自动化".to_string(),
            emoji: "🔍".to_string(),
            author: "wanshuiyin".to_string(),
            stars: 4533,
            installs: 8000,
            tags: vec!["AI".to_string(), "Research".to_string(), "Automation".to_string(), "Python".to_string()],
            github_url: "https://github.com/wanshuiyin/Auto-claude-code-research-in-sleep".to_string(),
            platform: "cursor".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "edict".to_string(),
            name: "edict".to_string(),
            description: "三省六部制 | OpenClaw 多 Agent 编排系统，9 个专用 AI Agent".to_string(),
            emoji: "🏛️".to_string(),
            author: "cft0808".to_string(),
            stars: 1800,
            installs: 5200,
            tags: vec!["AI".to_string(), "Multi-Agent".to_string(), "Python".to_string()],
            github_url: "https://github.com/cft0808/edict".to_string(),
            platform: "github".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "cc-switch".to_string(),
            name: "cc-switch".to_string(),
            description: "跨平台 AI 助手切换 | 支持 Claude Code/Codex/OpenClaw/Gemini CLI".to_string(),
            emoji: "🔄".to_string(),
            author: "farion1231".to_string(),
            stars: 1650,
            installs: 4800,
            tags: vec!["Productivity".to_string(), "CLI".to_string(), "Rust".to_string()],
            github_url: "https://github.com/farion1231/cc-switch".to_string(),
            platform: "cursor".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "nanoclaw".to_string(),
            name: "nanoclaw".to_string(),
            description: "轻量级 OpenClaw | 容器化运行，支持 WhatsApp/Telegram/Slack 等".to_string(),
            emoji: "🦞".to_string(),
            author: "qwibitai".to_string(),
            stars: 1500,
            installs: 4500,
            tags: vec!["AI".to_string(), "Lightweight".to_string(), "Backend".to_string()],
            github_url: "https://github.com/qwibitai/nanoclaw".to_string(),
            platform: "github".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "zeroclaw".to_string(),
            name: "zeroclaw".to_string(),
            description: "快速轻量的 AI 个人助理 | 任何 OS，任何平台".to_string(),
            emoji: "⚡".to_string(),
            author: "zeroclaw-labs".to_string(),
            stars: 1350,
            installs: 4200,
            tags: vec!["AI".to_string(), "Assistant".to_string(), "Rust".to_string()],
            github_url: "https://github.com/zeroclaw-labs/zeroclaw".to_string(),
            platform: "github".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "hermes-agent".to_string(),
            name: "hermes-agent".to_string(),
            description: "与你一起成长的 Agent | 持续学习和进化".to_string(),
            emoji: "🦅".to_string(),
            author: "NousResearch".to_string(),
            stars: 1280,
            installs: 3900,
            tags: vec!["AI".to_string(), "Agent".to_string(), "Python".to_string()],
            github_url: "https://github.com/NousResearch/hermes-agent".to_string(),
            platform: "github".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "last30days-skill".to_string(),
            name: "last30days-skill".to_string(),
            description: "AI 研究助手 | 跨 Reddit/X/YouTube/HN 搜索并合成摘要".to_string(),
            emoji: "📰".to_string(),
            author: "mvanhorn".to_string(),
            stars: 1150,
            installs: 3600,
            tags: vec!["AI".to_string(), "Search".to_string(), "Research".to_string(), "Python".to_string()],
            github_url: "https://github.com/mvanhorn/last30days-skill".to_string(),
            platform: "cursor".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "obsidian-skills".to_string(),
            name: "obsidian-skills".to_string(),
            description: "Obsidian AI 技能 | 教会 Agent 使用 Markdown/JSON Canvas/CLI".to_string(),
            emoji: "📝".to_string(),
            author: "kepano".to_string(),
            stars: 1080,
            installs: 3400,
            tags: vec!["AI".to_string(), "Productivity".to_string(), "Markdown".to_string()],
            github_url: "https://github.com/kepano/obsidian-skills".to_string(),
            platform: "cursor".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "ui-ux-pro-max-skill".to_string(),
            name: "ui-ux-pro-max-skill".to_string(),
            description: "专业 UI/UX 设计智能 | 支持多平台专业界面构建".to_string(),
            emoji: "🎨".to_string(),
            author: "nextlevelbuilder".to_string(),
            stars: 950,
            installs: 3100,
            tags: vec!["Frontend".to_string(), "UI".to_string(), "Design".to_string(), "AI".to_string()],
            github_url: "https://github.com/nextlevelbuilder/ui-ux-pro-max-skill".to_string(),
            platform: "cursor".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "claude-task-master".to_string(),
            name: "claude-task-master".to_string(),
            description: "AI 任务管理 | 可集成到 Cursor/Lovable/Windsurf 等".to_string(),
            emoji: "✅".to_string(),
            author: "eyaltoledano".to_string(),
            stars: 880,
            installs: 2900,
            tags: vec!["Productivity".to_string(), "Task".to_string(), "AI".to_string(), "JavaScript".to_string()],
            github_url: "https://github.com/eyaltoledano/claude-task-master".to_string(),
            platform: "cursor".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "flutter-ai-rules".to_string(),
            name: "flutter-ai-rules".to_string(),
            description: "Flutter AI 规则 | 支持 Cursor/Copilot 等 AI IDE".to_string(),
            emoji: "📱".to_string(),
            author: "evanca".to_string(),
            stars: 820,
            installs: 2700,
            tags: vec!["Frontend".to_string(), "Flutter".to_string(), "Mobile".to_string()],
            github_url: "https://github.com/evanca/flutter-ai-rules".to_string(),
            platform: "cursor".to_string(),
            large_clone: false,
        },
        // 其他托管平台热榜示例（克隆为通用 Git 项目；未必含 OpenClaw SKILL.md）
        HotSkill {
            id: "glab-cli".to_string(),
            name: "gitlab-cli-glab".to_string(),
            description: "GitLab 官方 CLI（glab）· GitLab 热榜向 Go 项目（通用 Git 仓库；作 OpenClaw 技能需含 SKILL.md）".to_string(),
            emoji: "🦊".to_string(),
            author: "gitlab-org".to_string(),
            stars: 3200,
            installs: 8900,
            tags: vec!["DevOps".to_string(), "Backend".to_string(), "Go".to_string()],
            github_url: "https://gitlab.com/gitlab-org/cli".to_string(),
            platform: "gitlab".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "codeberg-community".to_string(),
            name: "Codeberg-Community".to_string(),
            description: "Codeberg 社区讨论与维基 · Codeberg 热榜向（元社区仓库，非标准技能包）".to_string(),
            emoji: "🔷".to_string(),
            author: "Codeberg".to_string(),
            stars: 480,
            installs: 2100,
            tags: vec!["Frontend".to_string(), "Productivity".to_string()],
            github_url: "https://codeberg.org/Codeberg/Community".to_string(),
            platform: "codeberg".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "hf-setfit-sst2".to_string(),
            name: "SetFit-sst2-dataset".to_string(),
            description: "SetFit SST-2 数据集 · Hugging Face Hub 热榜向（数据集 Git 仓库，适用于 ML 工作流）".to_string(),
            emoji: "🤗".to_string(),
            author: "SetFit".to_string(),
            stars: 1860,
            installs: 12000,
            tags: vec!["AI".to_string(), "Data".to_string(), "Python".to_string()],
            github_url: "https://huggingface.co/datasets/SetFit/sst2".to_string(),
            platform: "huggingface".to_string(),
            large_clone: false,
        },
        // ─── 追加：ClawHub ───
        HotSkill {
            id: "clawhub-mcp-bridge".to_string(),
            name: "mcp-bridge".to_string(),
            description: "多 MCP 服务桥接与负载均衡 · 面向 Agent 的工具聚合".to_string(),
            emoji: "🌉".to_string(),
            author: "openclaw".to_string(),
            stars: 412,
            installs: 42000,
            tags: vec!["MCP".to_string(), "Backend".to_string(), "AI".to_string()],
            github_url: "https://github.com/punkpeye/awesome-mcp-servers".to_string(),
            platform: "clawhub".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "clawhub-context-engine".to_string(),
            name: "context-engine".to_string(),
            description: "长上下文压缩与检索 · 为技能执行节省 token".to_string(),
            emoji: "📚".to_string(),
            author: "skillforge".to_string(),
            stars: 355,
            installs: 31500,
            tags: vec!["AI".to_string(), "Learning".to_string(), "Productivity".to_string()],
            github_url: "https://github.com/run-llama/llama_index".to_string(),
            platform: "clawhub".to_string(),
            large_clone: true,
        },
        HotSkill {
            id: "clawhub-doc-ingest".to_string(),
            name: "doc-ingest".to_string(),
            description: "文档批量入库与分块 · PDF/Markdown/HTML 一键索引".to_string(),
            emoji: "📄".to_string(),
            author: "parsekit".to_string(),
            stars: 268,
            installs: 28300,
            tags: vec!["Data".to_string(), "Python".to_string(), "CLI".to_string()],
            github_url: "https://github.com/Unstructured-IO/unstructured".to_string(),
            platform: "clawhub".to_string(),
            large_clone: true,
        },
        HotSkill {
            id: "clawhub-git-automation".to_string(),
            name: "git-worktrees".to_string(),
            description: "Git worktree 与多分支自动化 · 适合并行技能开发".to_string(),
            emoji: "🌿".to_string(),
            author: "devflow".to_string(),
            stars: 198,
            installs: 22100,
            tags: vec!["DevOps".to_string(), "CLI".to_string(), "Productivity".to_string()],
            github_url: "https://github.com/tj/git-extras".to_string(),
            platform: "clawhub".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "clawhub-skill-synth".to_string(),
            name: "skill-synthesizer".to_string(),
            description: "从对话记录半自动生成 SKILL.md 草稿".to_string(),
            emoji: "🧪".to_string(),
            author: "agentlab".to_string(),
            stars: 175,
            installs: 19600,
            tags: vec!["AI".to_string(), "Productivity".to_string(), "Automation".to_string()],
            github_url: "https://github.com/BerriAI/litellm".to_string(),
            platform: "clawhub".to_string(),
            large_clone: false,
        },
        // ─── 追加：GitHub（通用技能向仓库）───
        HotSkill {
            id: "mcp-servers".to_string(),
            name: "mcp-servers".to_string(),
            description: "Model Context Protocol 参考服务器集合 · 文件/数据库/Git 等".to_string(),
            emoji: "🔧".to_string(),
            author: "modelcontextprotocol".to_string(),
            stars: 26800,
            installs: 156000,
            tags: vec!["MCP".to_string(), "Backend".to_string(), "DevOps".to_string()],
            github_url: "https://github.com/modelcontextprotocol/servers".to_string(),
            platform: "github".to_string(),
            large_clone: true,
        },
        HotSkill {
            id: "aider".to_string(),
            name: "aider".to_string(),
            description: "终端 AI 编程助手 · 多模型、Git 感知、适合配套技能".to_string(),
            emoji: "🤖".to_string(),
            author: "aider-ai".to_string(),
            stars: 24600,
            installs: 98000,
            tags: vec!["AI".to_string(), "CLI".to_string(), "Python".to_string()],
            github_url: "https://github.com/aider-ai/aider".to_string(),
            platform: "github".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "continue".to_string(),
            name: "continue".to_string(),
            description: "开源 AI 代码助手 VS Code/Cursor 插件 · 可接本地/OSS 模型".to_string(),
            emoji: "⏩".to_string(),
            author: "continuedev".to_string(),
            stars: 19800,
            installs: 72000,
            tags: vec!["Frontend".to_string(), "AI".to_string(), "JavaScript".to_string()],
            github_url: "https://github.com/continuedev/continue".to_string(),
            platform: "github".to_string(),
            large_clone: true,
        },
        HotSkill {
            id: "anthropic-cookbook".to_string(),
            name: "anthropic-cookbook".to_string(),
            description: "Anthropic API 示例与模式 · 提示词与工具调用参考".to_string(),
            emoji: "📖".to_string(),
            author: "anthropics".to_string(),
            stars: 9800,
            installs: 41000,
            tags: vec!["AI".to_string(), "Python".to_string(), "API".to_string()],
            github_url: "https://github.com/anthropics/anthropic-cookbook".to_string(),
            platform: "github".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "langchain".to_string(),
            name: "langchain".to_string(),
            description: "构建 LLM 应用框架 · 链、Agent、工具与记忆".to_string(),
            emoji: "⛓️".to_string(),
            author: "langchain-ai".to_string(),
            stars: 102000,
            installs: 500000,
            tags: vec!["AI".to_string(), "Python".to_string(), "Data".to_string()],
            github_url: "https://github.com/langchain-ai/langchain".to_string(),
            platform: "github".to_string(),
            large_clone: true,
        },
        HotSkill {
            id: "open-interpreter".to_string(),
            name: "open-interpreter".to_string(),
            description: "自然语言驱动本地代码执行 · 适合自动化技能原型".to_string(),
            emoji: "💬".to_string(),
            author: "OpenInterpreter".to_string(),
            stars: 58200,
            installs: 210000,
            tags: vec!["AI".to_string(), "Python".to_string(), "CLI".to_string()],
            github_url: "https://github.com/OpenInterpreter/open-interpreter".to_string(),
            platform: "github".to_string(),
            large_clone: true,
        },
        HotSkill {
            id: "mem0".to_string(),
            name: "mem0".to_string(),
            description: "长期记忆层 for AI · 可嵌入各 Agent 技能".to_string(),
            emoji: "🧠".to_string(),
            author: "mem0ai".to_string(),
            stars: 29600,
            installs: 88000,
            tags: vec!["AI".to_string(), "Backend".to_string(), "Python".to_string()],
            github_url: "https://github.com/mem0ai/mem0".to_string(),
            platform: "github".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "dify".to_string(),
            name: "dify".to_string(),
            description: "LLM 应用开发平台 · 工作流、RAG、Agent".to_string(),
            emoji: "🐝".to_string(),
            author: "langgenius".to_string(),
            stars: 102000,
            installs: 380000,
            tags: vec!["AI".to_string(), "Backend".to_string(), "DevOps".to_string()],
            github_url: "https://github.com/langgenius/dify".to_string(),
            platform: "github".to_string(),
            large_clone: true,
        },
        // ─── 追加：Cursor 生态（规则/指南类，多为 GitHub 镜像）───
        HotSkill {
            id: "awesome-cursorrules".to_string(),
            name: "awesome-cursorrules".to_string(),
            description: "精选 .cursorrules 与项目规则范例合集".to_string(),
            emoji: "📋".to_string(),
            author: "PatrickJS".to_string(),
            stars: 12400,
            installs: 52000,
            tags: vec!["Productivity".to_string(), "Frontend".to_string(), "Learning".to_string()],
            github_url: "https://github.com/PatrickJS/awesome-cursorrules".to_string(),
            platform: "cursor".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "system-prompts-ai-tools".to_string(),
            name: "system-prompts-ai-tools".to_string(),
            description: "主流 AI 工具系统提示与模型配置逆向整理".to_string(),
            emoji: "🔐".to_string(),
            author: "x1xhlol".to_string(),
            stars: 41200,
            installs: 125000,
            tags: vec!["AI".to_string(), "Learning".to_string(), "Security".to_string()],
            github_url: "https://github.com/x1xhlol/system-prompts-and-models-of-ai-tools".to_string(),
            platform: "cursor".to_string(),
            large_clone: true,
        },
        HotSkill {
            id: "cursor-tools-directory".to_string(),
            name: "cursor-tools".to_string(),
            description: "Cursor 向工具链与 CLI 合集索引".to_string(),
            emoji: "🛠️".to_string(),
            author: "getcursor".to_string(),
            stars: 8900,
            installs: 34000,
            tags: vec!["CLI".to_string(), "Productivity".to_string(), "JavaScript".to_string()],
            github_url: "https://github.com/getcursor/cursor".to_string(),
            platform: "cursor".to_string(),
            large_clone: true,
        },
        HotSkill {
            id: "windsurf-rules".to_string(),
            name: "windsurf-de/rules".to_string(),
            description: "Codeium Windsurf 规则与最佳实践（IDE 技能向）".to_string(),
            emoji: "🏄".to_string(),
            author: "Windsurf-AI".to_string(),
            stars: 2100,
            installs: 9100,
            tags: vec!["Frontend".to_string(), "Productivity".to_string(), "AI".to_string()],
            github_url: "https://github.com/Exafunction/windsurf.nvim".to_string(),
            platform: "cursor".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "v0-system-prompt".to_string(),
            name: "v0-system-prompts".to_string(),
            description: "Vercel v0 / 前端生成向提示与组件规范参考".to_string(),
            emoji: "▲".to_string(),
            author: "vercel-labs".to_string(),
            stars: 5600,
            installs: 22000,
            tags: vec!["Frontend".to_string(), "AI".to_string(), "JavaScript".to_string()],
            github_url: "https://github.com/vercel/ai".to_string(),
            platform: "cursor".to_string(),
            large_clone: false,
        },
        // ─── 追加：GitLab ───
        HotSkill {
            id: "gitlab-foss".to_string(),
            name: "gitlab".to_string(),
            description: "GitLab CE 单体仓库 · DevSecOps 平台核心".to_string(),
            emoji: "🦊".to_string(),
            author: "gitlab-org".to_string(),
            stars: 25600,
            installs: 190000,
            tags: vec!["Backend".to_string(), "DevOps".to_string(), "Ruby".to_string()],
            github_url: "https://gitlab.com/gitlab-org/gitlab".to_string(),
            platform: "gitlab".to_string(),
            large_clone: true,
        },
        HotSkill {
            id: "gitlab-cli-more".to_string(),
            name: "gitlab-release-cli".to_string(),
            description: "GitLab Release CLI · 发版与制品流水线辅助".to_string(),
            emoji: "📦".to_string(),
            author: "gitlab-org".to_string(),
            stars: 890,
            installs: 4500,
            tags: vec!["DevOps".to_string(), "CLI".to_string(), "Go".to_string()],
            github_url: "https://gitlab.com/gitlab-org/release-cli".to_string(),
            platform: "gitlab".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "inkscape".to_string(),
            name: "inkscape".to_string(),
            description: "开源矢量图编辑器 · 可与媒体类技能联用".to_string(),
            emoji: "✒️".to_string(),
            author: "inkscape".to_string(),
            stars: 4100,
            installs: 28000,
            tags: vec!["Media".to_string(), "Frontend".to_string(), "C++".to_string()],
            github_url: "https://gitlab.com/inkscape/inkscape".to_string(),
            platform: "gitlab".to_string(),
            large_clone: true,
        },
        HotSkill {
            id: "f-droid".to_string(),
            name: "fdroidclient".to_string(),
            description: "F-Droid 安卓应用商店客户端".to_string(),
            emoji: "🤖".to_string(),
            author: "fdroid".to_string(),
            stars: 2200,
            installs: 15000,
            tags: vec!["Frontend".to_string(), "Mobile".to_string(), "Java".to_string()],
            github_url: "https://gitlab.com/fdroid/fdroidclient".to_string(),
            platform: "gitlab".to_string(),
            large_clone: false,
        },
        // ─── 追加：Codeberg ───
        HotSkill {
            id: "forgejo".to_string(),
            name: "forgejo".to_string(),
            description: "Forgejo 自托管 Git · Gitea 分支（社区治理）".to_string(),
            emoji: "🔷".to_string(),
            author: "forgejo".to_string(),
            stars: 9200,
            installs: 48000,
            tags: vec!["Backend".to_string(), "DevOps".to_string(), "Go".to_string()],
            github_url: "https://codeberg.org/forgejo/forgejo".to_string(),
            platform: "codeberg".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "codeberg-gadgetbridge".to_string(),
            name: "Gadgetbridge".to_string(),
            description: "手环/手表与手机桥接（开源可穿戴）".to_string(),
            emoji: "⌚".to_string(),
            author: "Freeyourgadget".to_string(),
            stars: 3100,
            installs: 12000,
            tags: vec!["Mobile".to_string(), "Java".to_string(), "Productivity".to_string()],
            github_url: "https://codeberg.org/Freeyourgadget/Gadgetbridge".to_string(),
            platform: "codeberg".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "codeberg-woodpecker".to_string(),
            name: "woodpecker-ci".to_string(),
            description: "Woodpecker CI · 轻量持续集成（Codeberg 常见组合）".to_string(),
            emoji: "🪵".to_string(),
            author: "woodpecker-ci".to_string(),
            stars: 6200,
            installs: 31000,
            tags: vec!["DevOps".to_string(), "Backend".to_string(), "Go".to_string()],
            github_url: "https://codeberg.org/woodpecker-ci/woodpecker".to_string(),
            platform: "codeberg".to_string(),
            large_clone: false,
        },
        // ─── 追加：Hugging Face ───
        HotSkill {
            id: "hf-transformers".to_string(),
            name: "transformers".to_string(),
            description: "Transformers 库 · 预训练模型与推理流水线（列为 HF 生态）".to_string(),
            emoji: "🤗".to_string(),
            author: "huggingface".to_string(),
            stars: 142000,
            installs: 800000,
            tags: vec!["AI".to_string(), "Python".to_string(), "Data".to_string()],
            github_url: "https://github.com/huggingface/transformers".to_string(),
            platform: "huggingface".to_string(),
            large_clone: true,
        },
        HotSkill {
            id: "hf-squad".to_string(),
            name: "SQuAD-dataset".to_string(),
            description: "机器阅读理解经典数据集 SQuAD".to_string(),
            emoji: "📊".to_string(),
            author: "rajpurkar".to_string(),
            stars: 540,
            installs: 8900,
            tags: vec!["Data".to_string(), "AI".to_string(), "Learning".to_string()],
            github_url: "https://huggingface.co/datasets/rajpurkar/squad".to_string(),
            platform: "huggingface".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "hf-gsm8k".to_string(),
            name: "gsm8k".to_string(),
            description: "小学数学推理评测集 GSM8K · Agent 算术基准".to_string(),
            emoji: "🔢".to_string(),
            author: "openai".to_string(),
            stars: 1200,
            installs: 15600,
            tags: vec!["AI".to_string(), "Data".to_string(), "Test".to_string()],
            github_url: "https://huggingface.co/datasets/gsm8k".to_string(),
            platform: "huggingface".to_string(),
            large_clone: false,
        },
        HotSkill {
            id: "hf-datasets-hub".to_string(),
            name: "datasets-library".to_string(),
            description: "Hugging Face datasets 库源码 · 流式与大数据集".to_string(),
            emoji: "🤗".to_string(),
            author: "huggingface".to_string(),
            stars: 19800,
            installs: 92000,
            tags: vec!["Data".to_string(), "Python".to_string(), "AI".to_string()],
            github_url: "https://github.com/huggingface/datasets".to_string(),
            platform: "huggingface".to_string(),
            large_clone: true,
        },
    ]
}
