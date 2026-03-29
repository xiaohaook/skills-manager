# Skills Manager 安装指南

## 📦 快速安装

### 1. 获取源码

```bash
git clone <repository-url> skills-manager
cd skills-manager
```

### 2. 安装依赖

```bash
cd skills-manager  # 若尚未进入项目根目录

# 安装前端依赖
npm install

# 安装 Rust 依赖（会自动执行）
cd src-tauri
cargo build
cd ..
```

### 3. 开发模式运行

```bash
npm run tauri dev
```

### 4. 构建生产版本

```bash
# 构建 macOS 应用
npm run tauri build

# 构建后的应用位置：
# - src-tauri/target/release/skills-manager
# - src-tauri/target/release/bundle/macos/Skills Manager.app
```

---

## 🔧 系统要求

### 必需
- **Node.js** >= 18.x
- **npm** >= 9.x
- **Rust** >= 1.70
- **macOS** >= 12.0 (Monterey)

### 检查环境

```bash
# 检查 Node.js
node -v  # 应该显示 v18.x 或更高

# 检查 npm
npm -v   # 应该显示 9.x 或更高

# 检查 Rust
rustc --version  # 应该显示 1.70 或更高

# 如果 Rust 未安装
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## 🚀 使用方法

### 启动应用

```bash
cd skills-manager
npm run tauri dev
```

### 功能说明

1. **左侧边栏**
   - 显示所有 Skill 来源（Claude Code、Cursor、OpenClaw 等）
   - 点击来源过滤右侧卡片
   - 数字表示该来源的 Skill 数量

2. **右侧卡片**
   - 显示 Skill 详细信息
   - 左下角显示引用计数标签（被其他来源引用的次数）
   - 右上角圆圈用于批量选择

3. **右键菜单**
   - 复制到其他来源（自动创建软链接）
   - 在 Finder 中显示
   - 编辑 SKILL.md
   - 删除 Skill

4. **批量操作**
   - 勾选多个 Skill
   - 批量复制到目标来源
   - 批量删除

5. **刷新按钮**
   - 点击顶部"刷新"按钮重新加载 Skills
   - 通过 Finder 手动添加/删除 Skills 后使用

---

## 📁 目录结构

```
skills-manager/
├── src/                      # 前端源码（React + TypeScript）
│   ├── App.tsx
│   ├── main.tsx
│   └── index.css
├── src-tauri/                # 后端源码（Rust + Tauri）
│   ├── src/
│   │   └── main.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json              # 前端依赖
├── skills-manager.config.json # 配置文件
└── README.md                 # 本文档
```

---

## ⚙️ 配置文件

编辑 `skills-manager.config.json` 自定义扫描目录：

```json
{
  "scanRoots": [
    "~/.openclaw",
    "~/.claude",
    "~/.cursor",
    "~/projects"
  ],
  "exclude": [
    "**/node_modules/**",
    "**/.git/**",
    "**/dist/**"
  ],
  "maxDepth": 10
}
```

---

## 🐛 常见问题

### Q: 应用无法启动
```bash
# 清理并重新安装
rm -rf node_modules package-lock.json
npm install
cd src-tauri && cargo clean && cd ..
npm run tauri dev
```

### Q: Rust 编译错误
```bash
# 更新 Rust 工具链
rustup update
cargo clean
npm run tauri dev
```

### Q: Skills 不显示
1. 检查 Skills 目录是否存在
2. 确认每个 Skill 目录下有 `SKILL.md` 文件
3. 点击顶部"刷新"按钮

### Q: 软链接创建失败
- 检查目标目录是否有写权限
- 确保源文件和目标在同一文件系统

---

## 📞 技术支持

如有问题，请查看：
- 终端错误日志
- 浏览器开发者工具控制台（Cmd+Option+I）

---

_最后更新：2026-03-27_
