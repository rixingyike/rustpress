#!/bin/bash
# 用于为 yishulun_mdsrc 项目创建 .agents/skills 软链接的脚本

# 优先使用绝对路径
YISHULUN_DIR="/Users/jsmn/workspace/yishulun_mdsrc"

# 如果绝对路径不存在，则尝试使用脚本上一级目录作为项目根目录
if [ ! -d "$YISHULUN_DIR" ]; then
    SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
    YISHULUN_DIR="$( cd "$SCRIPT_DIR/.." && pwd )"
fi

TARGET_DIR="$YISHULUN_DIR/system/skills"
LINK_DIR="$YISHULUN_DIR/.agents"
LINK_PATH="$LINK_DIR/skills"

# 确保目标存在
if [ ! -d "$TARGET_DIR" ]; then
    echo "❌ 错误: 目标文件夹 $TARGET_DIR 不存在！"
    exit 1
fi

# 确保链接目录存在
mkdir -p "$LINK_DIR"

# 清理旧链接
if [ -L "$LINK_PATH" ] || [ -e "$LINK_PATH" ]; then
    rm -rf "$LINK_PATH"
fi

# 创建软链接
ln -s "$TARGET_DIR" "$LINK_PATH"

if [ $? -eq 0 ]; then
    echo "✅ 成功: 已创建软链接 $LINK_PATH -> $TARGET_DIR"
else
    echo "❌ 错误: 创建软链接失败！"
    exit 1
fi
