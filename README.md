# RustPress

<p align="center">
  <strong>🚀 极速、纯静态、零后端依赖的现代静态博客与数字花园生成器</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/rustpress"><img src="https://img.shields.io/crates/v/rustpress.svg?style=flat-square" alt="Crates.io"></a>
  <a href="https://github.com/rixingyike/rustpress/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue.svg?style=flat-square" alt="License"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.85%2B-orange.svg?style=flat-square" alt="Rust Edition"></a>
</p>

---

## 🔗 在线示例与完整文档

- 🌐 **在线示例站点 (Demo)**：[https://yishulun.com/](https://yishulun.com/)
- 📖 **官方完整使用与配置指南 (Docs)**：[https://yishulun.com/c/rustpress/index.html](https://yishulun.com/c/rustpress/index.html)

---

## ✨ 核心优势与特色

- ⚡ **独创倒分页算法（Reverse Pagination）**  
  最新内容固定在最新页（`index.html`），历史分页（`index1.html`、`index2.html`...）生成后内容与路径永久不变。配合 CDN/Cloudflare 边缘缓存，历史页面免刷缓存、零回源成本。

- 🔄 **毫秒级增量编译（Incremental Build）**  
  基于智能指纹比对与依赖图谱，每次构建仅更新受变动影响的文章与派生页。编译 10 篇文章与编译 10,000 篇文章速度无异，彻底解决静态博客“文章越多、构建越慢”的痛点。

- 📚 **全能内容矩阵体系（Content Matrix）**  
  开箱即用支持 6 大内容模型：
  - **日常博文 (Blog)**：经典瀑布流文章与归档；
  - **系列专栏 (Columns)**：多级树形目录、沉浸式阅读版式；
  - **短动态闲言 (Tweets)**：微语录朋友圈、多图自适应九宫格；
  - **出版著作 (Works)**：图书书架与立体封面大纲展示；
  - **开源项目 (Projects)**：软件发布页、多平台下载与截图展厅；
  - **友情链接 (Friends)**：友链卡片网格与申请说明。

- 🎨 **现代化主题与实时热重载（Modern Theme & Hot Reload）**  
  基于 **Tera** 模板引擎与 **Tailwind CSS**，预置极简高雅的 Light 主题。本地开发服务器支持模板、CSS 与 Markdown 变更即时热重载（Live Reload）。

- 🔍 **纯静态全功能体验（Zero-backend Fullstack）**  
  内置毫秒级客户端全文检索（`search.json` + Lunr.js）、RSS 订阅、Sitemap 搜索引擎收录、GitHub 原生无服务器评论与点赞系统（基于 Cloudflare Worker & GitHub API）。

- 🛠️ **高效 CLI 创作工作流（CLI Workflow）**  
  提供 `new-blog`、`new-tweet`、`new-article`、`make-catalog`、`make-cover` 等一键辅助指令，创作体验流畅自如。

---

## 🚀 快速上手

### 1. 安装 RustPress

```bash
# 从 crates.io 安装最新版本
cargo install rustpress --locked
```

### 2. 创作内容

在站点根目录（包含 `source/` 目录）下运行：

```bash
# 新建博客文章
rustpress new-blog "我的第一篇博客"

# 新建短动态闲言
rustpress new-tweet "今天天气不错，RustPress 很好用！"

# 为专栏新建章节
rustpress new-article rustpress "1.1.快速上手"
```

### 3. 本地预览与实时热重载

```bash
# 启动本地开发预览服务器（默认端口 1111，包含实时热重载）
rustpress dev
```
打开浏览器访问 `http://localhost:1111` 即可实时预览。

### 4. 生产构建

```bash
# 编译生成纯静态网站至 public/ 目录
rustpress build
```

---

## 📁 典型项目结构

```text
.
├── config.toml           # 站点核心配置文件
├── source/               # 内容源文件目录
│   ├── YYYY/             # 博客文章（按年份归档）
│   ├── tweets/           # 短动态闲言（按 YYYY/MM 归档）
│   ├── columns/          # 系列专栏（支持多级 catalog 目录）
│   ├── works/            # 出版著作
│   ├── projects/         # 开源项目
│   ├── friends/          # 友情链接
│   ├── about.md          # 作者关于页
│   └── assets/           # 全局图片与多媒体静态资源
├── themes/               # 前端主题模板与样式
│   └── light/
│       ├── templates/    # Tera HTML 模板
│       └── public/       # 主题 CSS/JS 静态文件
└── public/               # 生产构建静态输出目录（可直接部署）
```

---

## ⚙️ 基础配置示例 (`config.toml`)

```toml
[site]
name = "我的博客"
base_url = "https://example.com"
description = "记录思考与技术实践"
theme = "light"
dev_domains = ["localhost", "127.0.0.1", "dev.example.com"]

[site.author]
name = "创作者"
avatar = "/static/images/avatar.png"
bio = "终身学习者 / 软件开发者"

[taxonomies]
tweets = "/t"
columns = "/c"
categories = "/cat"
tags = "/tag"
```

---

## 📄 开源许可证

本项目基于 [Apache-2.0 License](LICENSE) 协议开源。

- 💻 GitHub 仓库：[https://github.com/rixingyike/rustpress](https://github.com/rixingyike/rustpress)
- 📦 Crates.io 类库：[https://crates.io/crates/rustpress](https://crates.io/crates/rustpress)
- 📖 专栏文档：[https://yishulun.com/c/rustpress/index.html](https://yishulun.com/c/rustpress/index.html)
