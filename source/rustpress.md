---
title: RustPress - 纯静态博客引擎
layout: project
version: 0.1.21
description: 增量编译、倒分页、无后端的 Rust 纯静态博客程序。支持 Markdown 写作、主题定制、插件系统和热更新开发模式。
createTime: 2026-02-19 15:00:00
icon: /assets/rustpress_logo.png
screenshots: []
downloads:
  - platform: Cargo 安装
    url: https://github.com/rixingyike/rustpress
    icon: link
    size: cargo install rustpress
  - platform: 源码 (GitHub)
    url: https://github.com/rixingyike/rustpress
    icon: link
    size: git clone
tags: [rust, blog, static-site, ssg, 开源]
comments: false
toc: false
---

## 功能特性

- 🚀 **纯静态生成** — 零后端依赖，生成纯 HTML/CSS/JS 静态站点
- ⚡ **增量编译** — 仅重新编译变更文件，毫秒级构建
- 📄 **Markdown 写作** — 完整的 Markdown + frontmatter 支持
- 🔄 **倒分页** — 独创倒序分页方案，新文章永远在首页
- 🎨 **主题系统** — Tailwind CSS 主题，支持自定义模板
- 🔌 **插件系统** — 基于 linkme 自动注册的插件架构
- 🔥 **热更新** — 开发模式下文件变更自动重新构建
- 💬 **评论系统** — 基于 GitHub Issues 的无后端评论
- 🔍 **全文搜索** — 基于 Lunr.js 的客户端全文搜索
- 📡 **RSS & Sitemap** — 自动生成订阅源和站点地图

## 快速开始

```bash
# 安装
cargo install rustpress

# 创建新博客
mkdir my-blog && cd my-blog
rustpress init

# 开发模式（热更新）
rustpress dev --hotreload

# 构建发布
rustpress build
```

## 技术栈

| 组件 | 技术 |
|------|------|
| 核心引擎 | Rust |
| 模板引擎 | Tera |
| Markdown | pulldown-cmark |
| CSS 框架 | Tailwind CSS |
| 搜索引擎 | Lunr.js |
| 插件注册 | linkme |

## 项目结构

```
my-blog/
├── source/           # Markdown 文章
│   ├── config.toml   # 站点配置
│   └── about.md      # 关于页面
├── themes/           # 主题目录
│   └── default/      # 默认主题
├── public/           # 生成的静态文件
└── config.toml       # 构建配置
```

## 开源协议

Apache License 2.0 — 可自由使用、修改和分发。
