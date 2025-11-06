#!/usr/bin/env bash
set -euo pipefail

# RustPress 发布脚本（更简单、自动）
# 功能：
# - 使用 cargo-release 提升版本号（默认 patch）
# - 发布到 crates.io（需本机已登录或设置 CARGO_REGISTRY_TOKEN）
# - 将提交与标签推送到 Git 远端
# 默认参数：LEVEL=patch, TAG_PREFIX=v, REMOTE=origin

LEVEL="${LEVEL:-patch}"
REMOTE="${REMOTE:-origin}"
TAG_PREFIX="${TAG_PREFIX:-v}"
NO_CONFIRM="${NO_CONFIRM:-1}"
ALLOW_DIRTY="${ALLOW_DIRTY:-0}"
CLEAN_NODE_MODULES="${CLEAN_NODE_MODULES:-0}"
SKIP_PUBLISH="${SKIP_PUBLISH:-0}"

echo ":: 发布级别: ${LEVEL}"
echo ":: Git 远端: ${REMOTE}"
echo ":: 标签前缀: ${TAG_PREFIX}"
echo ":: 无交互模式: ${NO_CONFIRM}"
echo ":: 允许脏工作区: ${ALLOW_DIRTY}"
echo ":: 清理 node_modules: ${CLEAN_NODE_MODULES}"
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

# 检查 crates.io 发布凭据：优先环境变量，其次本地登录
if [[ -z "${CARGO_REGISTRY_TOKEN:-}" && ! -f "${HOME}/.cargo/credentials" ]]; then
  echo "警告: 未检测到 crates.io 凭据。请运行 'cargo login <token>' 或导出 CARGO_REGISTRY_TOKEN。" >&2
  echo "      将继续执行，但发布到 crates.io 可能失败。" >&2
fi

# 可选：清理依赖目录以避免脏工作区（默认不清理，设置 CLEAN_NODE_MODULES=1 启用）
if [[ "$CLEAN_NODE_MODULES" == "1" ]]; then
  echo ":: 清理 themes/*/node_modules 以保证发布检查通过"
  while IFS= read -r -d '' nm; do
    echo "   - 删除: $nm"
    rm -rf "$nm"
  done < <(find themes -type d -name node_modules -prune -print0)
fi

# 处理脏工作区：直接提交并推送（检测包含未跟踪文件）
if [[ -n "$(git status --porcelain)" ]]; then
  echo ":: 检测到未提交改动（包含未跟踪文件），正在自动提交到 ${REMOTE}"
  git add -A || true
  if ! git diff --cached --quiet; then
    pre_commit_msg="chore: pre-release auto commit $(date -Iseconds)"
    git commit -m "$pre_commit_msg" || true
    echo ":: 推送预提交到 ${REMOTE}"
    git push "$REMOTE" || true
  else
    echo ":: 无改动需要提交"
  fi
fi

# 不再使用暂存/恢复流程，发布从干净工作区进行

# Show current version
current_version=$(sed -n 's/^version\s*=\s*"\([^"]\+\)"/\1/p' Cargo.toml | head -n 1 || true)
echo ":: 当前 Cargo.toml 版本: ${current_version:-unknown}"

echo ":: 开始运行 cargo-release（将发布到 crates.io 并推送到 Git）"
# Build flags for non-interactive release if requested
release_flags=("$LEVEL" --execute --publish --push --tag-prefix "$TAG_PREFIX" --push-remote "$REMOTE")
if [[ "$NO_CONFIRM" == "1" ]]; then
  release_flags+=(--no-confirm)
fi
if [[ "$ALLOW_DIRTY" == "1" ]]; then
  release_flags+=(--allow-dirty)
fi
if [[ "$SKIP_PUBLISH" == "1" ]]; then
  # 覆盖默认的 --publish，改为 --no-publish
  # 先移除数组中的 --publish（若存在）
  for i in "${!release_flags[@]}"; do
    if [[ "${release_flags[$i]}" == "--publish" ]]; then
      unset 'release_flags[$i]'
    fi
  done
  release_flags+=(--no-publish)
fi
cargo release "${release_flags[@]}"

# 显式推送提交与标签（以防某些环境下 --push 未生效）
echo ":: 显式推送提交与标签到 ${REMOTE}"
git push "$REMOTE" || true
git push "$REMOTE" --tags || true

# 显示最新标签
last_tag=$(git tag --list "${TAG_PREFIX}*" --sort=-version:refname | head -n 1 || true)
echo ":: 发布完成。最新标签（前缀 '${TAG_PREFIX}'）: ${last_tag:-<none>}"

echo ":: 提示: 可用环境变量覆盖默认值，例如："
echo "   TAG_PREFIX=v LEVEL=patch REMOTE=origin bash publish.sh"
