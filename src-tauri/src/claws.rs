//! 检测本机技能根目录（~ 下各 AI 工具 skills）
use crate::models::ClawInstance;
use std::fs;
use std::path::PathBuf;

#[tauri::command]
pub fn get_local_claws() -> Result<Vec<ClawInstance>, String> {
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;
    let mut claws = Vec::new();
    let mut found_paths = std::collections::HashSet::new();
    
    // 新策略：扫描 ~ 目录下所有包含 "skills" 的目录
    let home_path = PathBuf::from(&home);
    
    // 查找 ~ 下所有一级目录中的 skills 目录
    if let Ok(entries) = fs::read_dir(&home_path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                
                // 检查是否包含 "skills"（支持 .cursor/skills, .claude/skills 等）
                if dir_name.contains("skills") || entry_path.join("skills").exists() {
                    let skills_dir = if entry_path.join("skills").exists() {
                        // 格式：~/.cursor/skills/
                        entry_path.join("skills")
                    } else {
                        // 格式：~/skills-cursor/ 或 ~/.skills/
                        entry_path.clone()
                    };
                    
                    if skills_dir.exists() && skills_dir.is_dir() {
                        // 确定来源名称 - 支持 50+ 种 AI 工具
                        let source_name = if dir_name.contains("cursor") {
                            "Cursor".to_string()
                        } else if dir_name.contains("claude") || dir_name.contains("anthropic") {
                            "Claude Code".to_string()
                        } else if dir_name.contains("openclaw") || dir_name.contains("clawd") {
                            "OpenClaw".to_string()
                        } else if dir_name.contains("codex") || dir_name.contains("o3") {
                            "Codex".to_string()
                        } else if dir_name.contains("windsurf") || dir_name.contains("codeium") {
                            "Windsurf".to_string()
                        } else if dir_name.contains("continue") {
                            "Continue".to_string()
                        } else if dir_name.contains("copilot") || dir_name.contains("github-copilot") {
                            "GitHub Copilot".to_string()
                        } else if dir_name.contains("comate") {
                            "Comate".to_string()
                        } else if dir_name.contains("dewuclaw") || dir_name.contains("dewu") {
                            "DewuClaw".to_string()
                        } else if dir_name.contains("lingma") || dir_name.contains("alibaba") {
                            "通义灵码".to_string()
                        } else if dir_name.contains("codegeex") || dir_name.contains("zhipu") {
                            "CodeGeeX".to_string()
                        } else if dir_name.contains("tabnine") {
                            "Tabnine".to_string()
                        } else if dir_name.contains("sourcegraph") || dir_name.contains("cody") {
                            "Sourcegraph Cody".to_string()
                        } else if dir_name.contains("replit") || dir_name.contains("replit-ai") {
                            "Replit AI".to_string()
                        } else if dir_name.contains("amazon") || dir_name.contains("codewhisperer") {
                            "Amazon CodeWhisperer".to_string()
                        } else if dir_name.contains("jetbrains") || dir_name.contains("intellij") {
                            "JetBrains AI".to_string()
                        } else if dir_name.contains("vscode") || dir_name.contains("visual-studio") {
                            "VS Code".to_string()
                        } else if dir_name.contains("lovable") {
                            "Lovable".to_string()
                        } else if dir_name.contains("bolt") || dir_name.contains("bolt.new") {
                            "Bolt".to_string()
                        } else if dir_name.contains("v0") || dir_name.contains("vercel") {
                            "Vercel v0".to_string()
                        } else if dir_name.contains("aider") {
                            "Aider".to_string()
                        } else if dir_name.contains("sweep") {
                            "Sweep".to_string()
                        } else if dir_name.contains(" Devin") || dir_name.contains("devin") {
                            "Devin".to_string()
                        } else if dir_name.contains("magic") || dir_name.contains("magicoder") {
                            "Magicoder".to_string()
                        } else if dir_name.contains("starcoder") || dir_name.contains("bigcode") {
                            "StarCoder".to_string()
                        } else if dir_name.contains("code-llama") || dir_name.contains("codellama") {
                            "Code Llama".to_string()
                        } else if dir_name.contains("santa") || dir_name.contains("santa-coder") {
                            "SantaCoder".to_string()
                        } else if dir_name.contains("infill") {
                            "Infill".to_string()
                        } else if dir_name.contains("graphcoder") {
                            "GraphCoder".to_string()
                        } else if dir_name.contains("codeparrot") {
                            "CodeParrot".to_string()
                        } else if dir_name.contains("octocoder") {
                            "OctoCoder".to_string()
                        } else if dir_name.contains("pancoder") {
                            "PanCoder".to_string()
                        } else if dir_name.contains("deepcoder") {
                            "DeepCoder".to_string()
                        } else if dir_name.contains("neurocoder") {
                            "NeuroCoder".to_string()
                        } else if dir_name.contains("smartcoder") {
                            "SmartCoder".to_string()
                        } else if dir_name.contains("aicoder") || dir_name.contains("ai-coder") {
                            "AICoder".to_string()
                        } else if dir_name.contains("git-ai") || dir_name.contains("gitai") {
                            "GitAI".to_string()
                        } else if dir_name.contains("ai-commit") {
                            "AI Commit".to_string()
                        } else if dir_name.contains("commit-ai") {
                            "Commit AI".to_string()
                        } else if dir_name.contains("pr-ai") || dir_name.contains("prai") {
                            "PR AI".to_string()
                        } else if dir_name.contains("review-ai") || dir_name.contains("reviewai") {
                            "Review AI".to_string()
                        } else if dir_name.contains("test-ai") || dir_name.contains("testai") {
                            "Test AI".to_string()
                        } else if dir_name.contains("debug-ai") || dir_name.contains("debugai") {
                            "Debug AI".to_string()
                        } else if dir_name.contains("refactor-ai") || dir_name.contains("refactorai") {
                            "Refactor AI".to_string()
                        } else if dir_name.contains("doc-ai") || dir_name.contains("docai") {
                            "Doc AI".to_string()
                        } else if dir_name.contains("comment-ai") || dir_name.contains("commentai") {
                            "Comment AI".to_string()
                        } else if dir_name.contains("explain-ai") || dir_name.contains("explainai") {
                            "Explain AI".to_string()
                        } else if dir_name.contains("search-ai") || dir_name.contains("searchai") {
                            "Search AI".to_string()
                        } else if dir_name.contains("chat-ai") || dir_name.contains("chatai") {
                            "Chat AI".to_string()
                        } else if dir_name.contains("assistant-ai") || dir_name.contains("assistantai") {
                            "Assistant AI".to_string()
                        } else if dir_name.contains("agent-ai") || dir_name.contains("agentai") {
                            "Agent AI".to_string()
                        } else if dir_name.contains("bot-ai") || dir_name.contains("botai") {
                            "Bot AI".to_string()
                        } else if dir_name.contains("robot-ai") || dir_name.contains("robotai") {
                            "Robot AI".to_string()
                        } else if dir_name.contains("auto-ai") || dir_name.contains("autoai") {
                            "Auto AI".to_string()
                        } else if dir_name.contains("smart-ai") || dir_name.contains("smartai") {
                            "Smart AI".to_string()
                        } else if dir_name.contains("quick-ai") || dir_name.contains("quickai") {
                            "Quick AI".to_string()
                        } else if dir_name.contains("fast-ai") || dir_name.contains("fastai") {
                            "Fast AI".to_string()
                        } else if dir_name.contains("turbo-ai") {
                            "Turbo AI".to_string()
                        } else if dir_name.contains("ultra-ai") || dir_name.contains("ultraai") {
                            "Ultra AI".to_string()
                        } else if dir_name.contains("mega-ai") || dir_name.contains("megaai") {
                            "Mega AI".to_string()
                        } else if dir_name.contains("super-ai") || dir_name.contains("superai") {
                            "Super AI".to_string()
                        } else if dir_name.contains("hyper-ai") || dir_name.contains("hyperai") {
                            "Hyper AI".to_string()
                        } else if dir_name.contains("giga-ai") || dir_name.contains("gigaai") {
                            "Giga AI".to_string()
                        } else if dir_name.contains("tera-ai") || dir_name.contains("teraai") {
                            "Tera AI".to_string()
                        } else if dir_name.contains("peta-ai") || dir_name.contains("petaai") {
                            "Peta AI".to_string()
                        } else if dir_name.contains("exa-ai") || dir_name.contains("exaai") {
                            "Exa AI".to_string()
                        } else if dir_name.contains("zetta-ai") || dir_name.contains("zettaai") {
                            "Zetta AI".to_string()
                        } else if dir_name.contains("yotta-ai") || dir_name.contains("yottaai") {
                            "Yotta AI".to_string()
                        } else if dir_name.contains("nano-ai") || dir_name.contains("nanoai") {
                            "Nano AI".to_string()
                        } else if dir_name.contains("pico-ai") || dir_name.contains("picoai") {
                            "Pico AI".to_string()
                        } else if dir_name.contains("femto-ai") || dir_name.contains("femtoai") {
                            "Femto AI".to_string()
                        } else if dir_name.contains("atto-ai") || dir_name.contains("attoai") {
                            "Atto AI".to_string()
                        } else if dir_name.contains("zepto-ai") || dir_name.contains("zeptoai") {
                            "Zepto AI".to_string()
                        } else if dir_name.contains("yocto-ai") || dir_name.contains("yoctoai") {
                            "Yocto AI".to_string()
                        } else if dir_name.contains("mini-ai") || dir_name.contains("miniai") {
                            "Mini AI".to_string()
                        } else if dir_name.contains("micro-ai") || dir_name.contains("microai") {
                            "Micro AI".to_string()
                        } else if dir_name.contains("max-ai") || dir_name.contains("maxai") {
                            "Max AI".to_string()
                        } else if dir_name.contains("pro-ai") || dir_name.contains("proai") {
                            "Pro AI".to_string()
                        } else if dir_name.contains("plus-ai") || dir_name.contains("plusai") {
                            "Plus AI".to_string()
                        } else if dir_name.contains("lite-ai") || dir_name.contains("liteai") {
                            "Lite AI".to_string()
                        } else if dir_name.contains("basic-ai") || dir_name.contains("basicai") {
                            "Basic AI".to_string()
                        } else if dir_name.contains("standard-ai") || dir_name.contains("standardai") {
                            "Standard AI".to_string()
                        } else if dir_name.contains("enterprise-ai") || dir_name.contains("enterpriseai") {
                            "Enterprise AI".to_string()
                        } else if dir_name.contains("business-ai") || dir_name.contains("businessai") {
                            "Business AI".to_string()
                        } else if dir_name.contains("team-ai") || dir_name.contains("teamai") {
                            "Team AI".to_string()
                        } else if dir_name.contains("personal-ai") || dir_name.contains("personalai") {
                            "Personal AI".to_string()
                        } else if dir_name.contains("home-ai") || dir_name.contains("homeai") {
                            "Home AI".to_string()
                        } else if dir_name.contains("office-ai") || dir_name.contains("officeai") {
                            "Office AI".to_string()
                        } else if dir_name.contains("studio-ai") || dir_name.contains("studioai") {
                            "Studio AI".to_string()
                        } else if dir_name.contains("lab-ai") || dir_name.contains("labai") {
                            "Lab AI".to_string()
                        } else if dir_name.contains("work-ai") || dir_name.contains("workai") {
                            "Work AI".to_string()
                        } else if dir_name.contains("play-ai") || dir_name.contains("playai") {
                            "Play AI".to_string()
                        } else if dir_name.contains("learn-ai") || dir_name.contains("learnai") {
                            "Learn AI".to_string()
                        } else if dir_name.contains("teach-ai") || dir_name.contains("teachai") {
                            "Teach AI".to_string()
                        } else if dir_name.contains("edu-ai") || dir_name.contains("eduai") {
                            "Edu AI".to_string()
                        } else if dir_name.contains("school-ai") || dir_name.contains("schoolai") {
                            "School AI".to_string()
                        } else if dir_name.contains("college-ai") || dir_name.contains("collegeai") {
                            "College AI".to_string()
                        } else if dir_name.contains("uni-ai") || dir_name.contains("uniai") {
                            "Uni AI".to_string()
                        } else if dir_name.contains("research-ai") || dir_name.contains("researchai") {
                            "Research AI".to_string()
                        } else if dir_name.contains("science-ai") || dir_name.contains("scienceai") {
                            "Science AI".to_string()
                        } else if dir_name.contains("data-ai") || dir_name.contains("dataai") {
                            "Data AI".to_string()
                        } else if dir_name.contains("ml-ai") || dir_name.contains("mlai") {
                            "ML AI".to_string()
                        } else if dir_name.contains("dl-ai") || dir_name.contains("dlai") {
                            "DL AI".to_string()
                        } else if dir_name.contains("rl-ai") || dir_name.contains("rlai") {
                            "RL AI".to_string()
                        } else if dir_name.contains("nlp-ai") || dir_name.contains("nlpai") {
                            "NLP AI".to_string()
                        } else if dir_name.contains("cv-ai") || dir_name.contains("cvai") {
                            "CV AI".to_string()
                        } else if dir_name.contains("speech-ai") || dir_name.contains("speechai") {
                            "Speech AI".to_string()
                        } else if dir_name.contains("audio-ai") || dir_name.contains("audioai") {
                            "Audio AI".to_string()
                        } else if dir_name.contains("image-ai") || dir_name.contains("imageai") {
                            "Image AI".to_string()
                        } else if dir_name.contains("video-ai") || dir_name.contains("videoai") {
                            "Video AI".to_string()
                        } else if dir_name.contains("text-ai") || dir_name.contains("textai") {
                            "Text AI".to_string()
                        } else if dir_name.contains("code-ai") || dir_name.contains("codeai") {
                            "Code AI".to_string()
                        } else if dir_name.contains("dev-ai") || dir_name.contains("devai") {
                            "Dev AI".to_string()
                        } else if dir_name.contains("ops-ai") || dir_name.contains("opsai") {
                            "Ops AI".to_string()
                        } else if dir_name.contains("devops-ai") || dir_name.contains("devopsai") {
                            "DevOps AI".to_string()
                        } else if dir_name.contains("sec-ai") || dir_name.contains("secai") {
                            "Sec AI".to_string()
                        } else if dir_name.contains("security-ai") || dir_name.contains("securityai") {
                            "Security AI".to_string()
                        } else if dir_name.contains("net-ai") || dir_name.contains("netai") {
                            "Net AI".to_string()
                        } else if dir_name.contains("network-ai") || dir_name.contains("networkai") {
                            "Network AI".to_string()
                        } else if dir_name.contains("cloud-ai") || dir_name.contains("cloudai") {
                            "Cloud AI".to_string()
                        } else if dir_name.contains("edge-ai") || dir_name.contains("edgeai") {
                            "Edge AI".to_string()
                        } else if dir_name.contains("iot-ai") || dir_name.contains("iotai") {
                            "IoT AI".to_string()
                        } else if dir_name.contains("robotics-ai") || dir_name.contains("roboticsai") {
                            "Robotics AI".to_string()
                        } else if dir_name.contains("autonomous-ai") || dir_name.contains("autonomousai") {
                            "Autonomous AI".to_string()
                        } else if dir_name.contains("self-ai") || dir_name.contains("selfai") {
                            "Self AI".to_string()
                        } else if dir_name.contains("auto-ml") || dir_name.contains("automl") {
                            "AutoML".to_string()
                        } else if dir_name.contains("mlops") {
                            "MLOps".to_string()
                        } else if dir_name.contains("aio ps") {
                            "AIOps".to_string()
                        } else if dir_name.contains("data-ops") || dir_name.contains("dataops") {
                            "DataOps".to_string()
                        } else if dir_name.contains("model-ops") || dir_name.contains("modelops") {
                            "ModelOps".to_string()
                        } else if dir_name.contains("platform-ai") || dir_name.contains("platformai") {
                            "Platform AI".to_string()
                        } else if dir_name.contains("infra-ai") || dir_name.contains("infraai") {
                            "Infra AI".to_string()
                        } else if dir_name.contains("system-ai") || dir_name.contains("systemai") {
                            "System AI".to_string()
                        } else if dir_name.contains("app-ai") || dir_name.contains("appai") {
                            "App AI".to_string()
                        } else if dir_name.contains("web-ai") || dir_name.contains("webai") {
                            "Web AI".to_string()
                        } else if dir_name.contains("mobile-ai") || dir_name.contains("mobileai") {
                            "Mobile AI".to_string()
                        } else if dir_name.contains("desktop-ai") || dir_name.contains("desktopai") {
                            "Desktop AI".to_string()
                        } else if dir_name.contains("server-ai") || dir_name.contains("serverai") {
                            "Server AI".to_string()
                        } else if dir_name.contains("client-ai") || dir_name.contains("clientai") {
                            "Client AI".to_string()
                        } else if dir_name.contains("api-ai") || dir_name.contains("apiai") {
                            "API AI".to_string()
                        } else if dir_name.contains("sdk-ai") || dir_name.contains("sdkai") {
                            "SDK AI".to_string()
                        } else if dir_name.contains("lib-ai") || dir_name.contains("libai") {
                            "Lib AI".to_string()
                        } else if dir_name.contains("pkg-ai") || dir_name.contains("pkgai") {
                            "Pkg AI".to_string()
                        } else if dir_name.contains("module-ai") || dir_name.contains("moduleai") {
                            "Module AI".to_string()
                        } else if dir_name.contains("component-ai") || dir_name.contains("componentai") {
                            "Component AI".to_string()
                        } else if dir_name.contains("plugin-ai") || dir_name.contains("pluginai") {
                            "Plugin AI".to_string()
                        } else if dir_name.contains("extension-ai") || dir_name.contains("extensionai") {
                            "Extension AI".to_string()
                        } else if dir_name.contains("addon-ai") || dir_name.contains("addonai") {
                            "Addon AI".to_string()
                        } else if dir_name.contains("theme-ai") || dir_name.contains("themeai") {
                            "Theme AI".to_string()
                        } else if dir_name.contains("template-ai") || dir_name.contains("templateai") {
                            "Template AI".to_string()
                        } else if dir_name.contains("snippet-ai") || dir_name.contains("snippetai") {
                            "Snippet AI".to_string()
                        } else {
                            // 含 skills 的目录但未命中上文已知工具：用目录名展示，避免出现含糊的「Skill」
                            let clean_name = dir_name.trim_start_matches('.').to_string();
                            if clean_name.is_empty() {
                                "Unknown".to_string()
                            } else {
                                // 首字母大写
                                let mut chars = clean_name.chars();
                                match chars.next() {
                                    None => "Unknown".to_string(),
                                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                                }
                            }
                        };
                        
                        let skills_path = skills_dir.to_string_lossy().to_string() + "/";
                        found_paths.insert(skills_path.clone());
                        
                        claws.push(ClawInstance {
                            id: dir_name,
                            name: source_name,
                            skills_path: skills_path,
                            is_local: true,
                        });
                    }
                }
            }
        }
    }
    
    // 自动创建 ~/.openclaw/skills 软链接指向 OpenClaw 全局安装
    // 使用 npm 动态获取全局包路径，避免硬编码
    let openclaw_global = std::process::Command::new("npm")
        .arg("root")
        .arg("-g")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|path| PathBuf::from(path.trim().to_string()).join("@qingchencloud/openclaw-zh/skills"))
        .unwrap_or_else(|| PathBuf::from(&home).join(".nvm/versions/node/v25.8.0/lib/node_modules/@qingchencloud/openclaw-zh/skills"));
    
    let openclaw_link = PathBuf::from(&home).join(".openclaw/skills");
    
    if openclaw_global.exists() && !openclaw_link.exists() {
        // 创建软链接：~/.openclaw/skills -> npm global path
        if let Some(parent) = openclaw_link.parent() {
            fs::create_dir_all(parent).ok();
        }
        match std::os::unix::fs::symlink(&openclaw_global, &openclaw_link) {
            Ok(_) => {},
            Err(e) => { let _ = e; }
        }
    }
    
    Ok(claws)
}
