#!/bin/bash

# 获取当前年份
CURRENT_YEAR=$(date +"%Y")

# 目标目录
TARGET_DIR="$HOME/work/yishulun_blog_mdandcode/src/blog/$CURRENT_YEAR"

# 检查目标目录是否存在
if [ ! -d "$TARGET_DIR" ]; then
  echo -e "\n错误：目标目录 $TARGET_DIR 不存在！\n"
  exit 1
fi

# 获取当前最大数字文件名
MAX_NUM=$(ls "$TARGET_DIR" | grep -E '^[0-9]+\.md$' | sed 's/\.md//' | sort -n | tail -1)

# 如果没有找到数字文件，默认从 1 开始
if [ -z "$MAX_NUM" ]; then
  MAX_NUM=0
fi

# 新文件名
NEW_NUM=$((MAX_NUM + 1))
NEW_FILE="$TARGET_DIR/$NEW_NUM.md"

# 获取当前时间
CREATE_TIME=$(date +"%Y/%m/%d %H:%M:%S")

# 获取标题
TITLE="$1"
if [ -z "$TITLE" ]; then
  TITLE="标题"
fi

# 创建文件并写入默认内容
cat <<EOF > "$NEW_FILE"
---
createTime: $CREATE_TIME
tags: ["Default"]
draft: true
---

# $TITLE
EOF

# 输出成功信息
echo -e "\n文件已创建：$NEW_FILE\n"

# 使用 Typora 打开文件
# typora "$NEW_FILE"

# 使用同目录下的 .open_typora.sh 脚本打开文件
"$HOME/.open_typora.sh" $NEW_FILE