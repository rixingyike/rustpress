---
description: 发布 RustPress 新版本 (自动化: Changelog -> Crates.io -> GitHub Release)
---

# 🚀 一键发布流程 (One-Click Release)

// turbo-all

1. **自动更新 Changelog**
   - 提取自上次 Tag 以来的 Git 提交记录，简单总结，追加到 `CHANGELOG.md` 顶部。如果没有合适的提交记录，以"修改若干已知问题"代替。
   ```bash
   # 获取最近 Tag
   LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
   if [ -z "$LAST_TAG" ]; then
     LOGS=$(git log --pretty=format:"- %s")
   else
     LOGS=$(git log ${LAST_TAG}..HEAD --pretty=format:"- %s")
   fi
   
   TODAY=$(date +%Y-%m-%d)
   
   # 临时生成新日志段落
   echo "## [Unreleased] - $TODAY" > changelog_tmp
   echo "" >> changelog_tmp
   echo "$LOGS" >> changelog_tmp
   echo "" >> changelog_tmp
   echo "---" >> changelog_tmp
   echo "" >> changelog_tmp
   
   # 追加旧日志
   if [ -f CHANGELOG.md ]; then
     cat CHANGELOG.md >> changelog_tmp
   fi
   mv changelog_tmp CHANGELOG.md
   
   echo "✅ CHANGELOG.md 已更新。"
   ```

2. **执行发布脚本 (发布到 Crates.io)**
   - 使用 `AUTO_COMMIT=1` 自动提交 CHANGELOG 的变更并清理工作区。
   - 默认提升 Patch 版本。如需其他级别，请在运行 Workflow 前手动修改此命令。
   ```bash
   # AUTO_COMMIT=1: 自动提交未提交的变更 (如 CHANGELOG.md)
   # LEVEL=patch (默认)
   AUTO_COMMIT=1 bash publish_to_crates.sh
   ```

3. **创建 GitHub Release**
   - 读取新版本号，创建 GitHub Release。
   - 自动生成 Release Notes (GitHub 风格)。
   ```bash
   # 获取 Cargo.toml 中的最新版本 (由 publish_to_crates.sh 更新)
   CURRENT_VERSION=$(sed -n 's/^version[ ]*=[ ]*"\([^"]\+\)"/\1/p' Cargo.toml | head -n 1)
   echo "🚀 检测到新版本: v$CURRENT_VERSION"
   
   # 确保推送到远端后再创建 Release
   git push origin main --tags || true

   # 创建 GitHub Release
   gh release create "v$CURRENT_VERSION" --generate-notes --title "v$CURRENT_VERSION"
   
   echo "🎉 发布完成！"
   RELEASE_URL=$(gh release view "v$CURRENT_VERSION" --json url -q ".url")
   echo "🔗 Release Link: $RELEASE_URL"
   ```