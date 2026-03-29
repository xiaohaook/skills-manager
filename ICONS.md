# Skills Manager 图标安装指南

## 🎨 已设计的 Logo

已在 `src-tauri/icons/icon.svg` 创建了应用 logo：
- **绿色渐变背景** - 象征成长和连接
- **拼图元素** - 代表 Skills 的连接和组合
- **简洁现代** - 符合 Mac 应用风格

## 📐 图标规格

Mac 应用需要以下尺寸的 PNG 图标：
- `icon.png` (512x512)
- `icon.icns` (macOS 标准格式)

## 🔧 生成图标方法

### 方法 1：使用在线工具（推荐）

1. 访问 https://cloudconvert.com/svg-to-png
2. 上传 `src-tauri/icons/icon.svg`
3. 设置尺寸为 512x512
4. 下载并保存为 `src-tauri/icons/icon.png`

### 方法 2：使用 macOS 预览

1. 双击打开 `icon.svg`
2. 文件 → 导出
3. 格式选择 PNG
4. 分辨率设置为 512x512
5. 保存为 `icon.png`

### 方法 3：使用命令行（需要安装工具）

```bash
# 安装 ImageMagick
brew install imagemagick

# 生成 PNG
cd src-tauri/icons
convert -density 300 -resize 512x512 icon.svg icon.png

# 生成 ICNS（macOS 图标）
mkdir icon.iconset
sips -z 512 512 icon.png --out icon.iconset/icon_512x512.png
sips -z 256 256 icon.png --out icon.iconset/icon_256x256.png
sips -z 128 128 icon.png --out icon.iconset/icon_128x128.png
iconutil -c icns icon.iconset -o icon.icns
```

## ⚙️ 配置 Tauri

编辑 `src-tauri/tauri.conf.json`：

```json
{
  "bundle": {
    "icon": [
      "icons/icon.png",
      "icons/icon.icns"
    ],
    "identifier": "com.openclaw.skills-manager",
    "active": true
  }
}
```

## 🚀 重新构建

```bash
cd /Users/Zhuanz1/.openclaw/workspace/skills-manager
npm run tauri build
```

构建后的应用会显示新图标！

---

_提示：如果暂时不想生成图标，可以跳过，应用会使用默认图标。_
