use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub emoji: String,
    pub path: String,
    pub requires: Vec<String>,
    /// 当前机器上未在 PATH 中找到的二进制名（`bins` 子集）
    #[serde(rename = "missingBins", default)]
    pub missing_bins: Vec<String>,
    pub ready: bool,
    pub source: String,
    #[serde(rename = "refCount")]
    pub ref_count: u32,
    #[serde(rename = "otherClawCount")]
    pub other_claw_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClawInstance {
    pub id: String,
    pub name: String,
    pub skills_path: String,
    pub is_local: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HotSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub emoji: String,
    pub author: String,
    pub stars: u32,
    pub installs: u32,
    pub tags: Vec<String>,
    pub github_url: String,
    #[serde(default)]
    pub platform: String,
    /// 克隆体积/时间可能明显偏大，前端可提示用户
    #[serde(rename = "largeClone", default)]
    pub large_clone: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Platform {
    #[serde(rename = "github")]
    GitHub,
    #[serde(rename = "clawhub")]
    ClawHub,
    #[serde(rename = "builtin")]
    BuiltIn,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceConfig {
    pub name: String,
    pub path: String,
    pub enabled: bool,
}
