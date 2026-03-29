# Skills Manager（OpenClaw 技能管理器）

基于 **Tauri 2 + React + Rust** 的桌面应用，用于浏览、安装和管理本机 **OpenClaw / Cursor** 等多来源下的 Skills（技能包）。  

## 功能概览

| 能力 | 说明 |
|------|------|
| 多来源扫描 | 自动发现常见 Claw 技能目录，并合并自定义路径；去重后统一扫描 |
| 技能列表 | 搜索、按来源筛选、卡片展示就绪 / 缺依赖状态 |
| 社区热榜 | 拉取热门技能；与本地目录名比对后显示 **已安装**，避免重复安装 |
| 从 Git 安装 | 支持热榜一键安装或粘贴 GitHub 等仓库地址安装到选定目标目录 |
| 依赖检测 | 解析 `SKILL.md` / `AGENTS.md` 中的 `bins:`，标出 PATH 中缺失项（`missingBins`） |
| Homebrew 补齐 | 一键 / 单卡 / 详情内尝试用 `brew install` 安装缺失 CLI（会先校验 formula；无法进 brew 的需手动安装） |
| 批量操作 | 多选后复制到其他来源或批量删除 |
| 无障碍与交互 | 侧栏与对话框 roles、确认框、快捷键 `/` 聚焦搜索等 |

详细安装步骤见 [`INSTALL.md`](./INSTALL.md)。

## 环境要求

- **Node.js** ≥ 18  
- **Rust**（rustup 安装，详见 INSTALL.md）  
- **macOS**（主要开发与运行平台；其他平台需自行验证 Tauri 与系统路径行为）  
- **可选**：已安装 [Homebrew](https://brew.sh)，用于「补齐依赖」功能  

## 快速开始

```bash
git clone https://github.com/xiaohaook/skills-manager.git skills-manager
cd skills-manager
npm install
npm run tauri dev
```

（也可 fork 后替换为 fork 地址。）

## 常用脚本

| 命令 | 作用 |
|------|------|
| `npm run dev` | 仅前端 Vite（调 UI 时可用） |
| `npm run tauri dev` | 桌面应用开发模式 |
| `npm run build` | 前端 TypeScript 检查 + Vite 生产构建 |
| `npm run tauri build` | 打正式桌面包（见 INSTALL.md） |
| `npm test` | 前端单元测试（Vitest）；在 macOS/Linux 下将 `TMPDIR` 指向项目内 `.tmp`，避免权限问题 |
| `cd src-tauri && cargo test` | Rust 单元测试（`skill_md` 解析等） |

> **Windows**：`npm test` 默认脚本依赖 POSIX `mkdir`/`TMPDIR`。若在 PowerShell 下遇到问题，可手动设置临时目录后执行 `npx vitest run`。

## 项目结构（节选）

```
skills-manager/
├── src/                      # React 前端
│   ├── App.tsx
│   ├── components/           # Toast、侧栏、详情、右键菜单、确认框等
│   ├── lib/                  # 工具（如热榜与本地安装匹配、菜单位置等）
│   ├── types/domain.ts
│   └── hotTypes.ts
├── src-tauri/                # Tauri / Rust
│   ├── src/
│   │   ├── main.rs
│   │   ├── scan.rs          # 扫描技能与 symlink 引用
│   │   ├── skill_md.rs      # front matter / bins 解析与测试
│   │   ├── bin_install.rs   # Homebrew 安装缺失 bins
│   │   ├── hot_skills.rs
│   │   └── ...
│   └── Cargo.toml
├── package.json
├── vite.config.ts
├── INSTALL.md
└── README.md
```

## 测试

```bash
npm test
cd src-tauri && cargo test
```

## 参与贡献

欢迎 Issue / PR。提交前建议在本地执行 `npm run build`、`npm test` 与 `cd src-tauri && cargo test`。

## 许可证

若仓库内未包含 `LICENSE` 文件，默认以项目所有者的声明为准；添加开源许可证时建议在本目录放置 `LICENSE` 并在此更新说明。
