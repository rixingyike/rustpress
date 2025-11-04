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

## 安装

确保您已安装Rust和Cargo，然后执行以下命令：

```bash
# 克隆项目（假设已有仓库）
git clone https://github.com/rixingyike/rustpress.git
cd rustpress

# 构建项目
cargo build --release

# 将可执行文件复制到系统路径（可选）
cp target/release/rustpress /usr/local/bin/
```

## 使用方法

### 创建新的博客项目

```bash
cargo run -- new my-blog
# 或者如果已安装到系统路径
# rustpress new my-blog
```

这将创建一个名为`my-blog`的新博客项目，包含以下目录结构：

```
my-blog/
├── content/       # 存放Markdown文章
├── templates/     # 存放模板文件
├── static/        # 存放静态资源（CSS、JS、图片等）
├── public/        # 生成的静态网站文件
└── config.toml    # 配置文件
```

### 编写文章

在`content`目录下创建Markdown文件，例如`my-first-post.md`，包含以下内容：

```markdown
+++
title: "我的第一篇文章"
createTime: 2023-01-01
categories: ["技术"]
tags: ["Rust", "博客"]
+++

# 标题

这是一篇使用RustPress创建的博客文章。

## 二级标题

- 列表项1
- 列表项2
```

### 构建网站

```bash
cd my-blog
cargo run -- build
# 或者如果已安装到系统路径
# rustpress build
```

这将把Markdown文章编译成HTML文件，并输出到`public`目录。

### 本地预览

```bash
cd my-blog
cargo run -- serve
# 或者指定端口
# cargo run -- serve --port 8080
# 或者如果已安装到系统路径
# rustpress serve
```

这将在本地启动一个Web服务器，您可以在浏览器中访问`http://localhost:1111`来预览您的博客。

### 示例：指定源目录、输出目录并启用增量模式

在某些场景（例如兼容已有目录结构或在 CI 中区分构建目录），可以显式指定源目录与输出目录，并开启增量编译：

```bash
# 增量构建（指定源与输出目录）
cargo run -- -m old_source -c source/config.toml build -o old_public --incremental

# 启动本地预览（增量构建 + 指定端口）
cargo run -- -m old_source -c source/config.toml serve -o old_public -p 1118 --incremental
```

- `-m` 指定 Markdown 源目录，`-o` 指定输出目录，`-c` 指向配置文件路径；`--incremental` 显式启用增量构建（优先级高于 `build.toml`）。

## 配置

编辑`config.toml`文件来自定义您的博客：

```toml
[site]
name = "我的博客"           # 博客名称
description = "使用RustPress创建的博客"  # 博客描述
author = "作者"            # 作者名称
base_url = "https://example.com"  # 博客的基础URL

[taxonomies]
category = "categories"    # 分类
tag = "tags"               # 标签

# 分类分页配置（单分类文章列表每页显示多少条）
[categories]
posts_per_page = 8

# 标签分页配置（单标签文章列表每页显示多少条）
[tags]
posts_per_page = 8
```

## 模板自定义

您可以修改`templates`目录下的模板文件来自定义网站的外观：

- `base.html`：基础模板，包含HTML结构和CSS样式
- `index.html`：首页模板，显示文章列表
- `post.html`：文章详情页模板

## 部署到GitHub Pages

1. 构建您的网站：`cargo run -- build`
2. 进入`public`目录：`cd public`
3. 初始化git仓库：`git init`
4. 添加GitHub Pages远程仓库：`git remote add origin https://github.com/your-username/your-username.github.io.git`
5. 提交并推送：`git add . && git commit -m "Deploy blog" && git push -u origin master`

等待几分钟后，您的博客将可以在`https://your-username.github.io`访问。

## 开发

如果您想为RustPress贡献代码，请按照以下步骤：

1. Fork并克隆仓库
2. 创建功能分支：`git checkout -b feature/my-feature`
3. 提交更改：`git commit -am 'Add some feature'`
4. 推送到分支：`git push origin feature/my-feature`
5. 提交Pull Request

## 许可证

本项目使用MIT许可证 - 详见[LICENSE](LICENSE)文件

## 致谢

受到以下项目的启发：
- [Zola](https://www.getzola.org/)
- [Hugo](https://gohugo.io/)
