#!/bin/bash

# 获取当前分支名称
BRANCH=$(git branch --show-current)

# 检查是否在 Git 仓库中
if [ -z "$BRANCH" ]; then
  echo -e "\n错误：当前目录不是 Git 仓库！\n"
  exit 1
fi

# 提示用户确认操作
echo -e "\n警告：这将强制从远程仓库覆盖本地修改，所有未提交的更改将被丢弃！"
read -p "是否继续？(y/n): " CONFIRM

if [ "$CONFIRM" != "y" ] && [ "$CONFIRM" != "Y" ]; then
  echo -e "\n操作已取消。\n"
  exit 0
fi

# 强制从远程仓库拉取最新代码
echo -e "\n正在强制从远程仓库拉取最新代码并覆盖本地修改..."
git fetch origin
git reset --hard origin/"$BRANCH"
git clean -fd

# 输出完成信息
echo -e "\n操作完成！本地分支 $BRANCH 已强制更新为远程仓库的最新状态。\n"