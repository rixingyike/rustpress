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

## 未来计划

- 增加多语言支持
- 增加更多内置模板和主题
- 支持自定义内容类型
- 增强静态资源管理

---

如需详细开发历史，请查阅 git commit 日志。
