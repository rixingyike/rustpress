---
title: "首页"
subtitle: "欢迎使用 RustPress"
layout: home
home_navs:
  - title: "浏览分类"
    description: "按主题分类查看文章"
    url: "/categories.html"
    emoji: "�"
  - title: "标签云"
    description: "通过标签快速找到相关内容"
    url: "/tags.html"
    emoji: "🏷️"
  - title: "文章归档"
    description: "按时间顺序浏览所有文章"
    url: "/archives.html"
    emoji: "📚"
  - title: "友链"
    description: "提交你的站点，加入友链列表"
    url: "/friends.html"
    emoji: "🤝"
  - title: "关于作者"
    description: "查看关于页面，联系作者沟通"
    url: "/about.html"
    emoji: "📬"
  - title: "RustPress"
    description: "增量编译倒分页无后端 Rust 纯静态博客程序"
    url: "https://github.com/rixingyike/rustpress"
    emoji: "🚀"
    new_tab: true
    cta: "获取 RustPress"
---

# 欢迎使用RustPress

这是从 `home.md` 注入到首页的内容区块。你可以在这里写欢迎词、公告、推荐内容或放置任意 Markdown 文案。上方是最新文章列表与分页，下方是 6 个导航卡片。

## 可用字段（front matter）

- `home_navs`: 一个数组，支持字段 `title`、`description`、`url`、`emoji`（或 `icon`）、`new_tab`、`cta`。
- `title`/`subtitle`: 可选的页面标题信息（模板暂未直接使用，可用于扩展）。

> 提示：如不提供 `home_navs`，模板会使用默认的 6 张卡片。
