#!/usr/bin/env bash
set -euo pipefail

# RustPress 发布脚本（自动发布至 crates.io & 同步 Git 标签）
# 功能：
# - 自动配置 Cargo / Rust 工具链环境
# - 生成变更日志并在 CHANGELOG.md 头部更新
# - 使用 cargo-release 提升版本号（默认 patch）
# - 发布到 crates.io（需本地已登录或设置 CARGO_REGISTRY_TOKEN）
# - 将提交与标签推送到 Git 远端
#
# 常用用法：
#   bash publish-to-crates.sh                    # 升级 patch 版本并发布
#   LEVEL=minor bash publish-to-crates.sh        # 升级 minor 版本并发布
#   SKIP_PUBLISH=1 bash publish-to-crates.sh     # 仅打 tag 并推送到 Git，跳过 crates.io

# 1. 确保 Rust / Cargo 环境变量就绪
if [[ -f "$HOME/.cargo/env" ]]; then
  # shellcheck source=/dev/null
  source "$HOME/.cargo/env" 2>/dev/null || true
fi
export PATH="$HOME/.cargo/bin:$PATH"

# 2. 发布参数配置
LEVEL="${LEVEL:-patch}"
REMOTE="${REMOTE:-origin}"
TAG_PREFIX="${TAG_PREFIX:-v}"
NO_CONFIRM="${NO_CONFIRM:-1}"
SKIP_PUBLISH="${SKIP_PUBLISH:-0}"
STRICT_CLEAN="${STRICT_CLEAN:-1}"
AUTO_COMMIT="${AUTO_COMMIT:-1}"

echo ":: 发布级别: ${LEVEL}"
echo ":: Git 远端: ${REMOTE}"
echo ":: 标签前缀: ${TAG_PREFIX}"
echo ":: 无交互模式: ${NO_CONFIRM}"
echo ":: 严格要求干净工作区: ${STRICT_CLEAN}"
echo ":: 自动提交未跟踪/改动文件: ${AUTO_COMMIT}"
echo ":: 跳过 crates.io 发布: ${SKIP_PUBLISH}"

# 3. 检查仓库根目录
if [[ ! -f "Cargo.toml" ]]; then
  echo "错误: 请在包含 Cargo.toml 的仓库根目录运行此脚本" >&2
  exit 1
fi

# 4. 检查分支策略
branch=$(git rev-parse --abbrev-ref HEAD)
if [[ "$branch" != "main" && "$branch" != "master" ]]; then
  echo "错误: 当前分支 '$branch' 不允许发布（需在 main 或 master 分支）" >&2
  exit 1
fi

# 5. 检查 Rust & cargo-release 工具链
if ! command -v cargo &>/dev/null; then
  echo "错误: 未找到 cargo 命令。请确保已安装 Rust 工具链。" >&2
  exit 1
fi

if ! cargo release --help &>/dev/null; then
  echo ":: 未检测到 cargo-release，正在自动安装..."
  cargo install cargo-release
fi

# 6. 检查 Git 用户身份（避免未配置用户名邮箱时报错）
if [[ -z "$(git config user.name 2>/dev/null || true)" ]]; then
  git config user.name "金石碼农"
fi
if [[ -z "$(git config user.email 2>/dev/null || true)" ]]; then
  git config user.email "jinshimanong@gmail.com"
fi

# 7. 检查 crates.io 发布凭据（仅当不跳过发布时）
if [[ "$SKIP_PUBLISH" != "1" ]]; then
  CARGO_HOME_DIR="${CARGO_HOME:-$HOME/.cargo}"
  has_token="0"
  if [[ -n "${CARGO_REGISTRY_TOKEN:-}" ]]; then
    has_token="1"
  fi
  if [[ -f "${CARGO_HOME_DIR}/credentials" || -f "${CARGO_HOME_DIR}/credentials.toml" ]]; then
    has_token="1"
  fi
  if [[ "$has_token" != "1" ]]; then
    echo "错误: 未检测到 crates.io 凭据。请运行 'cargo login' 或导出 CARGO_REGISTRY_TOKEN。" >&2
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

# 8. 生成/更新变更日志 (CHANGELOG.md)
echo ":: 生成变更日志 (CHANGELOG.md)..."
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
if [[ -z "$LAST_TAG" ]]; then
  LOGS=$(git log --pretty=format:"- %s" | grep -v "chore: pre-release auto commit" | grep -v "chore: Release" || true)
else
  LOGS=$(git log "${LAST_TAG}..HEAD" --pretty=format:"- %s" | grep -v "chore: pre-release auto commit" | grep -v "chore: Release" || true)
fi

if [[ -n "$LOGS" ]]; then
  TODAY=$(date +%Y-%m-%d)
  TEMP_CHANGELOG=$(mktemp)
  echo "## [Unreleased] - $TODAY" > "$TEMP_CHANGELOG"
  echo "" >> "$TEMP_CHANGELOG"
  echo "$LOGS" >> "$TEMP_CHANGELOG"
  echo "" >> "$TEMP_CHANGELOG"
  echo "---" >> "$TEMP_CHANGELOG"
  echo "" >> "$TEMP_CHANGELOG"

  if [[ -f CHANGELOG.md ]]; then
    cat CHANGELOG.md >> "$TEMP_CHANGELOG"
  fi
  mv "$TEMP_CHANGELOG" CHANGELOG.md
  echo ":: CHANGELOG.md 更新完成"
else
  echo ":: 无新增提交日志，保持 CHANGELOG.md"
fi

ensure_clean_worktree

# 9. 执行 cargo-release
current_version=$(sed -n 's/^version[ ]*=[ ]*"\([^"]*\)"/\1/p' Cargo.toml | head -n 1 || true)
echo ":: 当前 Cargo.toml 版本: ${current_version:-unknown}"
echo ":: 开始运行 cargo-release（提升版本、发布 crates.io 并推送到 Git）..."

release_flags=("$LEVEL" --execute)
if [[ "$NO_CONFIRM" == "1" ]]; then
  release_flags+=(--no-confirm)
fi
if [[ "$SKIP_PUBLISH" == "1" ]]; then
  release_flags+=(--no-publish)
fi

cargo release "${release_flags[@]}"

echo ":: 发布成功！"

