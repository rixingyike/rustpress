#!/bin/bash
# 启动本地开发预览服务器

# 获取脚本所在目录的绝对路径
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🌐 启动本地预览服务器 (http://localhost:1111)..."
cargo run --manifest-path "$ROOT_DIR/Cargo.toml" -- -m "$SCRIPT_DIR/source" serve --output-dir "$SCRIPT_DIR/public"
