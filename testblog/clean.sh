#!/usr/bin/env bash
set -euo pipefail

# 清扫 testblog 测试目录的输出与临时文件
# 使用方式：
#  - 强制清扫（默认）：删除 source/config.toml、source/build.toml 以及 themes 目录
#      bash clean.sh
#  - 保留配置的清扫（不删除 source/config.toml 与 source/build.toml 且保留 themes）：
#      HARD=0 bash clean.sh

ROOT="$(cd "$(dirname "$0")" && pwd)"
echo ":: 清扫 testblog 输出与临时文件"

# 删除测试输出目录
rm -rf "$ROOT/public"

# 删除默认三页以测试程序自动生成（home/about/friends）
rm -f "$ROOT/source/home.md" "$ROOT/source/about.md" "$ROOT/source/friends.md"

# 清理系统残留文件
find "$ROOT" -name ".DS_Store" -type f -delete || true

# 可选：硬重置（默认启用），删除 source 下的配置与构建文件，并清理 themes 目录（下次运行会自动初始化并写入嵌入模板与静态）
if [[ "${HARD:-1}" == "1" ]]; then
  echo ":: 启用硬重置，删除 source/config.toml 与 source/build.toml"
  rm -f "$ROOT/source/config.toml" "$ROOT/source/build.toml"
  echo ":: 删除 themes 目录"
  rm -rf "$ROOT/themes"
fi

echo ":: 清扫完成"

# HARD=1 bash clean.sh
# cargo run -- -m ./testblog/source build -o ./testblog/public
# cargo run -- -m ./testblog/source serve -o ./testblog/public
# cargo run -- dev --hotreload
