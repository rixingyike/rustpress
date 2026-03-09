#!/bin/bash
# 编译示例博客到 test_blog_pub

# 获取脚本所在目录的绝对路径
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🚀 正在编译示例博客..."
cargo run --manifest-path "$ROOT_DIR/Cargo.toml" -- -m "$SCRIPT_DIR/source" build-dev --output-dir "$SCRIPT_DIR/public"
