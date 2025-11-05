# RustPress — 增量编译倒分页无后端 Rust 纯静态博客程序

一个用 Rust 构建的无后端静态博客程序，支持增量编译与倒分页。每次构建只重建受影响的页面（首页及相关标签/分类/年份页），无论文章是 1 篇还是几千篇，构建速度都保持稳定。内置 Tera 模板、纯前端搜索（search.json + JS）、RSS 与 Sitemap；并可通过 `source/build.toml` 的 `compile_mode` 在本地与 CI 中按需切换增量或全量构建。

## 特性

- 🚀 **快速**：使用Rust语言编写，编译速度快，生成网站高效
- 📝 **支持Markdown**：使用Markdown格式编写文章，简单易用
- 🎨 **模板系统**：使用Tera模板引擎，支持自定义网站外观
- 📦 **轻量级**：无运行时依赖，生成的网站可以直接部署
- 🔧 **简单易用**：提供直观的命令行界面

## 设计理念

RustPress采用**倒分页设计**，整体按日期降序、固定每页 `M = homepage.posts_per_page`（可配置）。命名规则：`index.html` 表示第 `n` 页（最新页），`index(n-1).html`、…、`index1.html` 依次更旧，其中 `index1.html` 为最早页。余数分配策略为“余数放到首页”：当总数 `len` 不是 `M` 的整数倍时，首页 `index.html` 展示余数 `r = len % M` 条，其余各页（含 `index1.html`）均满 `M` 条。范围文案统一按“最旧为 1 开始编号”显示：例如 `index1.html` 显示“第 1 到 M 条”、`index2.html` 显示“第 M+1 到 2M 条”、首页显示“第 len - r + 1 到 len 条”（若 `r = 0` 则为“第 (n-1)M+1 到 nM 条”）。导航语义也采用倒分页：上一页指向更“新”，下一页指向更“旧”，首页无“上一页”，`index1.html` 无“下一页”。该设计支持增量编译：新增文章只影响最新页及其上游页面，旧页面保持稳定，构建效率随文章数量线性增长。单标签文章列表页沿用相同的倒分页与余数策略。

**归档页设计**：按年归档，archives.html为归档首页，包含年份索引链接（倒序排列）。年份归档页位于archives/2024.html等路径，文章不分页，已过完年份的归档页无需重新生成，每次编译只生成总归档页和当前年份归档页。

**分类页设计**：categories.html展示source目录结构，便于结构化写作。分类页只显示分类关系和文章数量，点击分类跳转到子分类文章列表页。子分类文章列表页不分页，直接展示该分类下所有文章。采用扁平URL结构：categories/技术架构/微服务架构/index.html、categories/技术架构/index.html。

**标签页设计**：tags.html展示所有标签及文章数目，不展示文章列表。点击标签跳转到单标签文章列表页，与首页相同，也采用倒分页设计：第1页包含最旧文章，最大页数包含最新文章。URL结构例如：tags/rust/index.html（第1页）、tags/rust/index3.html（第3页）、tags/rust/index2.html（第2页）。

**搜索页设计**：search.html作为搜索功能入口，采用纯前端实现方案。构建时预生成search.json索引文件，客户端通过JavaScript在内存中完成实时搜索，无需后端支持，实现静态网站的高效搜索体验。

**侧边栏设计**：采用组件化设计，在以下页面展示侧边栏：
- **首页**（index.html）- 已实现，展示网站概览和导航

侧边栏内容包含：
- **作者信息** - 姓名、头像、简介、位置、社交链接
- **广告位2** - 可配置的推广内容
- **热门文章** - 基于标签和分类相似度计算的智能推荐文章列表（已实现）
- **热门分类** - 最近更新的一些分类
- **热门标签** - 文章最多的一些标签
- **广告位3** - 可配置的推广内容

设计原则是保持内容稳定性与用户体验的平衡，侧边栏内容以相对稳定信息为主，同时包含必要的动态更新内容。相关文章功能采用智能算法，根据当前文章的标签和分类计算相似度，为用户提供真正有价值的内容推荐。

## 文件复制策略（重要）

RustPress 仅解析并渲染 `source` 目录中的 Markdown（`.md`）为 HTML。对非 Markdown 的静态文件，按如下策略复制到输出目录（默认 `public`）：

- 递归复制：遍历 `source` 下所有子目录，将“非隐藏且非 `.md`”文件按原相对路径复制到 `public`。
- 根层文本文件：如 `CNAME`、`robots.txt` 等放在 `source` 根层，构建后会出现在 `public` 根层。
- 资产与附件：图片（如 `.png`、`.jpg`、`.jpeg`、`.gif`）、文本（如 `.txt`）、其它附件，保持原有相对目录层次，方便子目录独立移动；不再仅限于 `source/assets`，子目录中的 `assets` 也会被扫描并复制。
- 隐藏文件：以 `.` 开头的文件将跳过复制（例如 `.DS_Store`）。

示例：

- `source/assets/img.png` -> `public/assets/img.png`
- `source/posts/abc/assets/logo.jpg` -> `public/posts/abc/assets/logo.jpg`
- `source/CNAME` -> `public/CNAME`

## 如何使用

下面提供四种使用方式，按你的场景选择其一即可：

### 1）通过 crates.io 安装并使用（推荐）

- 已发布到 crates.io，推荐直接使用 `cargo install` 安装。
- 快速开始：

```bash
# 安装（固定版本）
cargo install rustpress

# 构建静态站点
rustpress -m source build -o public -c config.toml
或 rustpress build

# 开发预览（含热重载与主题编译）
rustpress dev --hotreload -m source -c config.toml -p 1111 -o public
或 rustpress dev --hotreload
```

- 热重载（模板实时预览）：如需监听模板变化自动重建，请使用 CLI：

```bash
cargo dev --hotreload --md-dir source --config config.toml -p 8000
```

热重载适合在编写主题模板时使用。

### 2）作为工程依赖使用

在你自己的项目中添加依赖，并以代码方式调用构建与预览：

```toml
# Cargo.toml
[dependencies]
rustpress = "0.1.5" 
```

```rust
// src/main.rs
fn main() -> rustpress::Result<()> {
    let config = rustpress::Config::from_file(std::path::Path::new("config.toml"))?;
    let gen = rustpress::Generator::new(config.clone(), std::path::Path::new("source"))?;

    // 构建站点到 public
    gen.build("source", "public")?;

    // 启动本地预览
    rustpress::server::DevServer::serve_sync(1111, "public")?;
    Ok(())
}
```

### 3）Fork 源码自由定制

- Fork 本仓库并克隆到本地，按需修改源码与模板：
  - 模板路径：`themes/default/templates/`
  - 静态资源（主题）：`themes/default/static/`
- 本地开发建议：

```bash
# 开发环境构建（编译 CSS + 构建站点）
cargo run -- build-dev

# 开发模式（构建 + 启动预览）
cargo run -- dev

# 开启模板热重载
cargo run -- dev --hotreload
```

### 4）通过 GitHub Actions 自动部署（示例 deploy.yml）

将以下文件保存为 `.github/workflows/deploy.yml`，每次推送到 `main` 分支时自动构建并部署到 GitHub Pages：

```yaml
name: Deploy RustPress to GitHub Pages

on:
  push:
    branches: [ main ]
  workflow_dispatch:

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Build site
        run: |
          cargo install rustpress
          rustpress build

      - name: Upload artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: public

      - name: Deploy to external repository
        uses: cpina/github-action-push-to-another-repository@v1.7.2
        env:
          API_TOKEN_GITHUB: ${{ secrets.EXTERNAL_REPOSITORY_PERSONAL_ACCESS_TOKEN }}
        with:
          source-directory: public/
          destination-github-username: rixingyike
          destination-repository-name: rixingyike.github.io
          target-branch: main
          user-email: 9830131@qq.com
```

> 注：站点的静态文件输出目录为 `public`；根层文本（如 `CNAME`、`robots.txt`）与所有非 `.md` 附件会按原相对路径递归复制到 `public`。

## 配置

编辑`config.toml`文件来自定义您的博客：

```toml
[site]
name = "我的博客"           # 博客名称
description = "使用RustPress创建的博客"  # 博客描述
author = "作者"            # 作者名称
base_url = "https://example.com"  # 博客的基础URL

# 分类分页配置（单分类文章列表每页显示多少条）
[categories]
posts_per_page = 8

# 标签分页配置（单标签文章列表每页显示多少条）
[tags]
posts_per_page = 8
……
```

## 模板主题自定义

您可以修改`templates`目录下默认的 default 主题文件来自定义网站的外观：

- `base.html`：基础模板，包含HTML结构和CSS样式
- `index.html`：首页模板，显示文章列表
- `post.html`：文章详情页模板
- 其他等，使用 hotreload 模式方便修改主题

## 许可证

本项目使用MIT许可证 - 详见[LICENSE](LICENSE)文件

## 致谢

受到以下项目的启发：
- [Zola](https://www.getzola.org/)
- [Hugo](https://gohugo.io/)
 
