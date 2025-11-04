#!/bin/bash

# Typora 可执行文件路径
TYPORA_PATH="$HOME/AppData/Local/Programs/Typora/Typora.exe"

# 检查是否传递了文件名参数
if [ -z "$1" ]; then
  # 如果没有传递参数，直接打开 Typora
  start "" "$TYPORA_PATH"
else
  # 如果传递了参数，打开指定文件
  start "" "$TYPORA_PATH" "$1"
fi