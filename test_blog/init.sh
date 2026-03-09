#!/bin/bash
# test_blog/test_init.sh

echo "开始测试 rustpress init 功能..."

# 1. 切换到项目根目录
cd "$(dirname "$0")/.." || exit

# 2. 清理 test_blog/source 目录下的内容
echo "清理 test_blog/source 子内容..."
rm -rf test_blog/source/*

# 3. 调用 rustpress init 初始化 test_blog
echo "运行 rustpress -m test_blog/source init..."
cargo run -- -m test_blog/source init

# 4. 验证关键文件是否已生成
echo "验证文件生成情况..."

FILES=(
    "test_blog/source/blog/$(date +%Y)/1.md"
    "test_blog/source/projects/rustpress.md"
    "test_blog/source/docs/guide/1.2.1.站点信息配置.md"
    "test_blog/source/about.md"
    "test_blog/themes/default/public/static/images/gongyi.jpg"
    "test_blog/themes/default/public/static/images/donate_qrcode.png"
)

for file in "${FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "✅ $file [存在]"
    else
        echo "❌ $file [缺失]"
    fi
done

# 5. 执行一次预览构建
echo "执行预览构建..."
cargo run -- -m test_blog/source build -o test_blog/public

echo "测试完成！请刷新浏览器预览效果。"
