#!/usr/bin/env bash
set -euo pipefail

# 默认生产端口为 1111，可通过参数指定自定义端口，例如 `./prod.sh 8080`
PORT="${1:-1111}"

echo "=== [Production] 正在 Release 模式下编译 RustPress ==="
cargo build --release

echo "=== [Production] 正在生成静态博客网站 HTML/CSS/JS ==="
./target/release/rustpress build --output-dir public

echo "=== [Production] 正在启动生产静态文件服务器，端口：$PORT ==="
# 启动内置的静态服务器，关闭开发热重载 (--no-hotreload)
dev=pushpen ./target/release/rustpress serve --port "$PORT" --no-hotreload --output-dir public
