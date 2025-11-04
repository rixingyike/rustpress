#!/bin/bash
# setup_honkit.sh - HonKit项目快速初始化脚本

set -e  # 遇到错误立即退出

# 定义源目录和目标目录
SOURCE_DIR="../wangwen"
TARGET_DIR="."  # 当前目录

# 打印带颜色的状态信息
RED='\033[31m'
GREEN='\033[32m'
YELLOW='\033[33m'
NC='\033[0m' # 无颜色

echo -e "${GREEN}▶ 开始初始化 HonKit 环境...${NC}"

# 检查源目录是否存在
if [ ! -d "$SOURCE_DIR" ]; then
  echo -e "${RED}✗ 错误：上级目录中找不到 wangwen 文件夹${NC}"
  exit 1
fi

# 文件拷贝清单（支持批量模式）
COPY_ITEMS=(
  ".github"
  ".gitignore"
  ".bookignore"
  "book.json"
  "package.json"
  "_layouts"
)

# 执行文件拷贝操作
echo -e "${YELLOW}➤ 正在复制配置文件...${NC}"
for item in "${COPY_ITEMS[@]}"; do
  source_path="$SOURCE_DIR/$item"
  
  # 处理目录拷贝
  if [ -d "$source_path" ]; then
    echo -e "▷ 复制目录: $item"
    cp -rf "$source_path" "$TARGET_DIR"
  # 处理文件拷贝
  elif [ -f "$source_path" ]; then
    echo -e "▷ 复制文件: $item"
    cp -f "$source_path" "$TARGET_DIR"
  else
    echo -e "${RED}✗ 找不到文件/目录: $item${NC}"
    exit 2
  fi
done

# 安装依赖
echo -e "\n${YELLOW}➤ 正在安装 Node.js 依赖...${NC}"
npm install --silent  # 静默模式安装

# 启动服务
echo -e "\n${GREEN}▶ 启动开发服务器...${NC}"
npm run serve

echo -e "\n${GREEN}✅ 环境初始化完成！${NC}"