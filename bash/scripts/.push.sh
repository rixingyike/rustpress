#!/bin/bash

"$HOME/.pull.sh"

# 获取当前分支名称
BRANCH=$(git branch --show-current)

# 检查是否在 Git 仓库中
if [ -z "$BRANCH" ]; then
  echo "错误：当前目录不是 Git 仓库！"
  exit 1
fi

# 生成提交信息
if [ -z "$1" ]; then
  # 如果没有提供提交信息，则使用当前时间生成默认信息
  COMMENT="自动提交: $(date +%Y-%m-%d_%H:%M:%S)"
else
  # 如果提供了提交信息，则使用传递的字符串
  COMMENT="$1"
fi

# 执行 Git 操作
git add ./
git commit -m "$COMMENT"
git push origin "$BRANCH"

# 输出完成信息
echo -e "\n提交完成！分支: $BRANCH，提交信息: $COMMENT"