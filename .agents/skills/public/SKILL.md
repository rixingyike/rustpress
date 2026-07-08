---
name: public
description: 发布新 tag 并发布 Rust 包到 crates.io。当用户请求 "/public" 或要求发布新版本、Git Tag、或发布包到 crates.io 时激活此技能。
---

# Public Skill

本技能用于指导 Agent 协助用户或在宿主 macOS 开发机上执行新版本打包、自动标记并推送 Git Tag 以及发布 Rust 包至 crates.io。

## 核心机制

仓库根目录下已内置完备的自动化发布脚本 [publish_to_crates.sh](file:///workspace/rustpress/publish_to_crates.sh)。它通过 `cargo-release` 链式完成：生成变更日志 -> 提升版本 -> 提交并打标签 -> 推送远程 Git -> 发布 crates.io。

## 使用步骤

1. **前提检查**：
   - 当前分支必须为 `main` 或 `master`。
   - 宿主机上必须已配置好 crates.io 的发布 Token（通过 `cargo login` 写入本地凭据文件 `~/.cargo/credentials.toml`，或者在终端中导出 `CARGO_REGISTRY_TOKEN` 环境变量）。

2. **运行发布命令**：
   - 引导用户在宿主开发机终端（或者在具备 Rust 工具链的环境中）运行以下命令：
     ```bash
     # 1. 默认升级小版本号并发布至 crates.io
     bash publish_to_crates.sh
     
     # 2. 升级次要版本号（minor）并自动提交未暂存改动
     LEVEL=minor AUTO_COMMIT=1 bash publish_to_crates.sh
     
     # 3. 仅提升版本、标记并推送 Git Tag，跳过 crates.io 发布
     SKIP_PUBLISH=1 bash publish_to_crates.sh
     ```

3. **脚本自动化逻辑**：
   - 自动扫描 git log 并在 [CHANGELOG.md](file:///workspace/rustpress/CHANGELOG.md) 头部生成并追加最新版本日志；
   - 自动提交工作区改动以保持 Git Tree 干净；
   - 调用 `cargo release` 修改版本、打 tag 并推送至远端仓库；
   - 执行编译并发布包至 [crates.io/crates/rustpress](https://crates.io/crates/rustpress)。
