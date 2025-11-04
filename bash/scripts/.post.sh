#!/bin/bash

# 目标目录
TARGET_DIR="$HOME/work/yishulun_blog_mdandcode"

# 检查目标目录是否存在
if [ ! -d "$TARGET_DIR" ]; then
  echo -e "\n错误：目标目录 $TARGET_DIR 不存在！\n"
  exit 1
fi

# 导航到目标目录
cd "$TARGET_DIR" || { echo -e "\n错误：无法进入目录 $TARGET_DIR！\n"; exit 1; }

# 执行 push.sh 脚本
"$HOME/.push.sh" "自动提交：$(date +'%Y-%m-%d %H:%M:%S')"