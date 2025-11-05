# RustPress 开发记录

## 现有功能

- 支持通过 `new` 命令一键生成新博客项目，包含内容、模板、配置等目录结构。
- 支持 `build` 命令将 Markdown 文件（支持 YAML front matter）渲染为 HTML，输出到 public 目录。
- 支持 `serve` 命令本地预览，自动加载当前目录下的模板（支持 per-project 模板）。
- Markdown front matter 支持 YAML 风格（---），兼容旧文章。
- front matter 字段允许缺省，title、createTime、slug 均有合理 fallback 逻辑。
- 模板文件命名简洁（base.html、index.html、post.html），支持自定义。
- CLI 支持 `--force` 参数覆盖生成新项目。
- 代码和模板均已去除未用的默认/冗余文件，保持项目结构简洁。

## 近期开发与修复

- 修复依赖重复、缺失等问题，完善 Cargo.toml。
- 修正 Tera 模板语法错误，完善模板变量 fallback。
- 支持 YAML front matter 解析，兼容旧文章格式。
- 优化 slug 生成逻辑，优先 front matter，否则用文件名。
- 模板加载路径改为相对当前目录，支持每个项目自定义模板。
- 新增 `--force` 支持，允许覆盖生成新项目。
- 清理所有未用模板、空目录，保持仓库整洁。

## 发布日志

### 0.1.6
- 修复默认主题按钮在悬停时文字不可见的问题：移除全局 `a:hover` 文本颜色，避免覆盖按钮上的 `text-white` 等样式。
- 调整全局链接样式：`a { @apply text-primary-600 transition-colors; }`，让组件/按钮的文本颜色优先级更高。
- 重新编译主题 CSS：`themes/default/src/tailwind.input.css` → `public/static/css/main.css`。
- 发布与标记：版本 `0.1.6`，Git 标签 `0.1.6`（无 `v` 前缀），并已发布到 crates.io。
- 升级建议：`cargo install rustpress --force` 或在项目中更新依赖为 `rustpress = "0.1.6"`。

注：本次修复影响的页面包括使用蓝色按钮样式的模板（如 `search.html`、`404.html`、`archives.html`、`year_archive.html` 等），无需改动模板，CSS 修复即可生效。

## 未来计划

- 增加多语言支持
- 增加更多内置模板和主题
- 支持自定义内容类型
- 增强静态资源管理

---

如需详细开发历史，请查阅 git commit 日志。
