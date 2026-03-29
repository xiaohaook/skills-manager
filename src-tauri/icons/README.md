# 🎨 Skills Manager 图标生成指南

## 问题原因

SVG 文件包含渐变和复杂元素，转换工具可能不支持，导致图标显示不正确。

## ✅ 解决方案（3 选 1）

### 方法 1：使用 macOS 预览（最简单）

1. **打开 SVG 文件**
   ```bash
   open /Users/Zhuanz1/.openclaw/workspace/skills-manager/src-tauri/icons/icon.svg
   ```

2. **导出为 PNG**
   - 文件 → 导出（File > Export）
   - 格式选择：**PNG**
   - 尺寸：**512 x 512**
   - 保存为：`icon.png`

3. **验证**
   ```bash
   open /Users/Zhuanz1/.openclaw/workspace/skills-manager/src-tauri/icons/icon.png
   ```

### 方法 2：使用在线工具

1. **访问转换网站**
   - https://cloudconvert.com/svg-to-png
   - https://svgtopng.com/

2. **上传并转换**
   - 上传：`icon.svg`
   - 设置尺寸：512x512
   - 下载：`icon.png`

3. **保存位置**
   ```bash
   mv ~/Downloads/icon.png src-tauri/icons/
   ```

### 方法 3：运行生成脚本

```bash
cd /Users/Zhuanz1/.openclaw/workspace/skills-manager/src-tauri/icons
chmod +x generate-icon.sh
./generate-icon.sh
```

## 📐 图标规格

| 文件 | 尺寸 | 用途 |
|------|------|------|
| `icon.png` | 512x512 | Mac App Store / DMG |
| `icon@2x.png` | 1024x1024 | Retina 显示屏 |
| `icon.icns` | 多尺寸 | macOS 应用包 |

## 🎯 当前图标设计

- **绿色背景** (#10B981) - 象征成长和连接
- **白色拼图块** - 代表 Skills 的组合
- **中心圆点** - 表示核心枢纽
- **四角装饰** - 象征多个 Skills

## ✅ 验证图标

```bash
# 查看图标
qlmanage -p src-tauri/icons/icon.png

# 检查尺寸
sips -g pixelWidth -g pixelHeight src-tauri/icons/icon.png
```

## 🚀 应用图标

设置好 `icon.png` 后，重新构建应用：

```bash
cd /Users/Zhuanz1/.openclaw/workspace/skills-manager
npm run tauri build
```

构建后的应用会显示新图标！

---

_提示：如果图标还是显示不正确，请检查 PNG 文件是否损坏，或尝试重新导出。_
