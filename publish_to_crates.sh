#!/usr/bin/env bash
set -euo pipefail

# AUTO_COMMIT=1 bash publish_to_crates.sh

# RustPress 发布脚本（更简单、自动）
# 功能：
# - 使用 cargo-release 提升版本号（默认 patch）
# - 发布到 crates.io（需本机已登录或设置 CARGO_REGISTRY_TOKEN）
# - 将提交与标签推送到 Git 远端
# 默认参数：LEVEL=patch, TAG_PREFIX=v, REMOTE=origin

# 发布参数
# 说明：默认不清理 .gitignore 指定的文件，以保留本地预览输出（如 public/）和测试目录（如 testblog/）。
# 如需清理可在运行时设置 CLEAN_IGNORED=1（保留列表由 PRESERVE_IGNORED 控制）。
LEVEL="${LEVEL:-patch}"
REMOTE="${REMOTE:-origin}"
TAG_PREFIX="${TAG_PREFIX:-v}"
NO_CONFIRM="${NO_CONFIRM:-1}"
SKIP_PUBLISH="${SKIP_PUBLISH:-0}"
STRICT_CLEAN="${STRICT_CLEAN:-1}"
AUTO_COMMIT="${AUTO_COMMIT:-0}"

echo ":: 发布级别: ${LEVEL}"
echo ":: Git 远端: ${REMOTE}"
echo ":: 标签前缀: ${TAG_PREFIX}"
echo ":: 无交互模式: ${NO_CONFIRM}"
echo ":: 严格要求干净工作区: ${STRICT_CLEAN}"
echo ":: 自动提交未跟踪/改动文件: ${AUTO_COMMIT}"
echo ":: 跳过 crates.io 发布: ${SKIP_PUBLISH}"

# Ensure we are at repo root
if [[ ! -f "Cargo.toml" ]]; then
  echo "错误: 请在包含 Cargo.toml 的仓库根目录运行此脚本" >&2
  exit 1
fi

# Check branch policy (aligns with release.toml allow-branch)
branch=$(git rev-parse --abbrev-ref HEAD)
if [[ "$branch" != "main" && "$branch" != "master" ]]; then
  echo "错误: 当前分支 '$branch' 不允许（需在 main 或 master 分支）" >&2
  exit 1
fi

# 检查 crates.io 发布凭据（仅当不跳过发布时）
if [[ "$SKIP_PUBLISH" != "1" ]]; then
  CARGO_HOME_DIR="${CARGO_HOME:-$HOME/.cargo}"
  has_token="0"
  # 环境变量方式
  if [[ -n "${CARGO_REGISTRY_TOKEN:-}" ]]; then
    has_token="1"
  fi
  # 文件方式（兼容新的 credentials.toml 与旧的 credentials）
  if [[ -f "${CARGO_HOME_DIR}/credentials" || -f "${CARGO_HOME_DIR}/credentials.toml" ]]; then
    has_token="1"
  fi
  if [[ "$has_token" != "1" ]]; then
    echo "错误: 未检测到 crates.io 凭据。请运行 'cargo login'（推荐从 stdin 输入 token）或导出 CARGO_REGISTRY_TOKEN。" >&2
    echo "提示: 可设置 SKIP_PUBLISH=1 仅做版本、标签与推送。" >&2
    exit 1
  fi
fi

ensure_clean_worktree() {
  if [[ -n "$(git status --porcelain)" ]]; then
    if [[ "$AUTO_COMMIT" == "1" ]]; then
      echo ":: 自动提交改动以保证干净工作区"
      git add -A || true
      if ! git diff --cached --quiet; then
        pre_commit_msg="chore: pre-release auto commit $(date -Iseconds)"
        git commit -m "$pre_commit_msg" || true
        echo ":: 推送预提交到 ${REMOTE}"
        git push "$REMOTE" || true
      fi
    else
      if [[ "$STRICT_CLEAN" == "1" ]]; then
        echo "错误: 工作区存在未提交改动。请提交或清理后再发布。" >&2
        git status --porcelain || true
        echo "提示: 可设置 AUTO_COMMIT=1 自动提交。" >&2
        exit 1
      fi
    fi
  fi
}

ensure_clean_worktree

# Show current version
current_version=$(sed -n 's/^version[ ]*=[ ]*"\([^"]*\)"/\1/p' Cargo.toml | head -n 1 || true)
echo ":: 当前 Cargo.toml 版本: ${current_version:-unknown}"

echo ":: 开始运行 cargo-release（将发布到 crates.io 并推送到 Git）"
# 构建 release 参数（显式执行，不传 tag-prefix，避免重复 v）
# 加上 --no-push 确保本地只打 tag，不推送到远端（由 workflow 处理）
release_flags=("$LEVEL" --execute --no-push --registry crates-io --allow-dirty)
if [[ "$NO_CONFIRM" == "1" ]]; then
  release_flags+=(--no-confirm)
fi
if [[ "$SKIP_PUBLISH" == "1" ]]; then
  release_flags+=(--no-publish)
fi
cargo release "${release_flags[@]}"
