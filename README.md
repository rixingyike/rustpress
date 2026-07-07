# RustPress — 增量编译倒分页无后端 Rust 纯静态博客程序

一个用 Rust 构建的无后端静态博客程序，支持增量编译与倒分页。每次构建只重建受影响的页面（首页及相关标签/分类/年份页），无论文章是 1 篇还是几千篇，构建速度都保持稳定。内置 Tera 模板、纯前端搜索（search.json + JS）、RSS 与 Sitemap；并可通过 `source/build.toml` 的 `compile_mode` 在本地与 CI 中按需切换增量或全量构建。

compile_mode：
incremental
full

cd "/Users/liyi/Library/Application\ Support/app.pushpen.writer/"
cargo run --manifest-path "/Users/liyi/workspace/rustpress/Cargo.toml" -- -m "source" serve --output-dir "public"


## 特性

- 🚀 **快速**：使用Rust语言编写，编译速度快，生成网站高效
- 📝 **支持Markdown**：使用Markdown格式编写文章，简单易用
- 🎨 **模板系统**：使用Tera模板引擎，支持自定义网站外观
- 📦 **轻量级**：无运行时依赖，生成的网站可以直接部署
- 🔧 **简单易用**：提供直观的命令行界面

## 设计理念与页面系统

### 1）网站导航与搜索系统
* **极简顶部导航**：在最新主题（如 `light`）中，采用了精致的极简顶栏设计。主要页面链接（如 首页、关于、专栏、著作、项目、归档等）无缝整合在**全局搜索输入框**中。
* **搜索快捷导航**：用户只需点击搜索框，或输入拼音/中英文关键词（例如「作者」、「专栏」、「作品」、「项目」、「博客」），即可通过下拉面板中的**快捷指向链接**一键跳转至相应版块，免去了传统繁杂菜单的视觉拥堵。
* **无后端全文搜索**：基于编译时预生成的 `search.json` 索引，纯前端 JavaScript 毫秒级匹配并呈现文章标题与摘要。

### 2）多元内容与页面系统
* **专栏系统 (Columns)**：
  - 存放于 `source/columns/{ID}/` 目录中，主配置文件为 `README.md`。
  - 通过 Frontmatter 中的 `catalog` 定义多级嵌套章节（支持缩进空格表示级别，最大支持三级缩进），专栏主页自动按树形目录渲染。
  - 支持收费/免费模式配置。专栏详情页在侧边栏渲染全书大纲，支持点击平滑锚点定位。
* **著作/图书系统 (Works)**：
  - 存放于 `source/works/{ID}.md`，编译为平扁的 `/works/{ID}.html`。
  - Frontmatter 中通过 `cover` 声明封面路径（存放于 `source/works/assets/`），呈现精致的图书详情卡片。
* **项目展示系统 (Projects)**：
  - 存放于 `source/projects/{ID}/README.md`，编译为 `/projects/{ID}/index.html`。
  - 支持多平台构建包下载、客户端截图画廊、技术栈标签等丰富的项目元数据展示。
* **闲言碎语 (Tweets)**：
  - 存放于推文页面的微型朋友圈/推特卡片流，支持多图混排、移动端适配与纯前端懒加载。
* **倒分页博客流**：
  - 整体按日期降序分发，每页固定 `M` 条（在 `config.toml` 中通过 `homepage.posts_per_page` 配置）。
  - 采用独创的“余数放首页”倒分页策略。新增文章只会影响首页及上游页面，使得绝大多数旧页面在编译时保持完全不变，从而完美契合**增量编译**机制，无论文章体量多大都能保持毫秒级构建。
* **按年/月归档 (Archives)**：
  - `/archives/` 作为总归档首页，呈现年份索引。年份归档页（如 `/archives/2026/`）不分页，只在当前年份发生文章新增时增量重构。
* **分类树 (Categories)**：
  - `/categories/` 结构化呈现物理目录树，文件夹展开/收起时配备平滑的动效和交互式文件夹图标。
* **标签云 (Tags)**：
  - `/tags/` 呈现所有文章标签的云集与文章计数，点击即可进入标签专属倒分页列表。

### 3）侧边栏设计 (Sidebar)
采用模块化和组件化设计，在列表和归档等页面展示侧边栏：
- **作者信息卡片**：展示作者姓名、头像、信仰座右铭、社交矩阵（GitHub、X、Zhihu、WeChat 二维码等）。
- **智能推荐（相关内容）**：详情页中根据文章分类和标签的重合度，通过 TF-IDF 相似算法自动推荐 5 篇关联度最高的内容。
- **热门排行榜**：根据配置展示站内高频热门分类与最热标签云。
- **推荐广告位 (Ads)**：提供 header、侧边栏顶部、侧边栏底部等多个可配置的广告卡片栏位。

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
name: deploy

on:
  # 每当 push 到 main 分支时触发部署
  # Deployment is triggered whenever a push is made to the main branch.
  push:
    branches: [main]
  # 手动触发部署
  # Manually trigger deployment
  workflow_dispatch:

# 统一版本号，后续只需改这里
env:
  RUSTPRESS_VERSION: "0.1.12"

jobs:
  docs:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4
        with:
          # “最近更新时间” 等 git 日志相关信息，需要拉取全部提交记录
          # "Last updated time" and other git log-related information require fetching all commit records.
          fetch-depth: 0

      - name: Setup Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      # 缓存 rustpress 二进制，命中后跳过安装
      - name: Cache rustpress binary
        id: cache-rustpress
        uses: actions/cache@v3
        with:
          path: ~/.cargo/bin/rustpress
          key: rustpress-bin-${{ runner.os }}-${{ env.RUSTPRESS_VERSION }}

      # 可选：缓存 crates 索引与源码，加速首次安装或版本升级
      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
          key: cargo-registry-${{ runner.os }}-stable

      # 仅在未命中二进制缓存时安装固定版本的 RustPress
      - name: Install RustPress (if missing)
        if: steps.cache-rustpress.outputs.cache-hit != 'true'
        run: cargo install rustpress --version ${{ env.RUSTPRESS_VERSION }} --locked

      - name: Verify RustPress
        run: rustpress -V

      - name: Build site with RustPress
        run: rustpress -m source build -o public

      # 上传构建结果到 GitHub Pages
      - name: Upload build artifacts
        uses: actions/upload-artifact@v4
        with:
          path: public

      # 部署到外部仓库
      - name: Deploy to external repository
        uses: cpina/github-action-push-to-another-repository@v1.7.2
        env:
          API_TOKEN_GITHUB: ${{ secrets.EXTERNAL_REPOSITORY_PERSONAL_ACCESS_TOKEN }}
        with:
          source-directory: public/
          destination-github-username: rixingyike
          destination-repository-name: rixingyike.github.io
          user-email: 9830131@qq.com
          target-branch: "main"
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

本项目使用 Apache 2.0 许可证 - 详见[LICENSE](LICENSE)文件

## 致谢

受到以下项目的启发：
- [Zola](https://www.getzola.org/)
- [Hugo](https://gohugo.io/)
 
