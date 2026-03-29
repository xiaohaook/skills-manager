#!/bin/bash

# Skills Manager 图标生成脚本
cd "$(dirname "$0")"

echo "🎨 Generating Skills Manager icons..."

# 检查是否安装了 node 和 canvas
if ! command -v node &> /dev/null; then
    echo "❌ Node.js not found. Please install Node.js first."
    exit 1
fi

# 使用 node 生成 PNG
cat > generate-icon.js << 'EOF'
const fs = require('fs');
const { createCanvas } = require('canvas');

const canvas = createCanvas(512, 512);
const ctx = canvas.getContext('2d');

// 背景（绿色）
ctx.fillStyle = '#10B981';
ctx.beginPath();
ctx.roundRect(0, 0, 512, 512, 120);
ctx.fill();

// 拼图块 1（左上）
ctx.fillStyle = 'rgba(255, 255, 255, 0.95)';
ctx.beginPath();
ctx.roundRect(140, 140, 120, 120, 20);
ctx.fill();

// 拼图块 2（右下）
ctx.fillStyle = 'rgba(255, 255, 255, 0.85)';
ctx.beginPath();
ctx.roundRect(232, 232, 120, 120, 20);
ctx.fill();

// 中心连接点
ctx.fillStyle = 'white';
ctx.beginPath();
ctx.arc(256, 256, 15, 0, Math.PI * 2);
ctx.fill();

// 装饰圆点
ctx.fillStyle = 'rgba(255, 255, 255, 0.6)';
[[150, 150], [362, 150], [150, 362], [362, 362]].forEach(([x, y]) => {
    ctx.beginPath();
    ctx.arc(x, y, 6, 0, Math.PI * 2);
    ctx.fill();
});

// 保存
const buffer = canvas.toBuffer('image/png');
fs.writeFileSync('icon.png', buffer);
console.log('✅ Generated icon.png (512x512)');
EOF

# 安装 canvas 库
echo "📦 Installing canvas library..."
npm install --save-dev canvas 2>/dev/null || {
    echo "❌ Failed to install canvas. Try manual method below."
    exit 1
}

# 生成图标
echo "🖼️  Generating PNG..."
node generate-icon.js

# 清理
rm -f generate-icon.js

echo ""
echo "✅ Icon generation complete!"
echo "📍 Location: src-tauri/icons/icon.png"
echo ""
echo "📝 Manual method (if script failed):"
echo "   1. Open icon.svg in Safari or Preview"
echo "   2. File > Export"
echo "   3. Format: PNG, Size: 512x512"
echo "   4. Save as icon.png"
