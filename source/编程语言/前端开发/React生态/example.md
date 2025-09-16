---
title: "欢迎使用RustPress"
createTime: 2023-07-15 10:00:00

tags: ["Rust", "博客", "静态网站"]
---

# 欢迎使用 RustPress

这是一个使用 RustPress 创建的示例页面。RustPress 是一个基于 Rust 语言的静态博客生成器，可以将 Markdown 文件转换为精美的 HTML 网站。

## 特性

- 🚀 **快速** - 使用 Rust 语言编写，性能出色
- 📝 **支持 Markdown** - 轻松撰写内容
- 🎨 **可定制模板** - 使用 Tera 模板引擎
- 🌐 **本地预览** - 内置 web 服务器，方便预览
- 📦 **易于部署** - 生成的静态文件可以部署到任何 web 服务器

## 快速开始

1. 在 `mdsource` 目录下创建 Markdown 文件
2. 运行 `cargo run -- build` 生成网站
3. 运行 `cargo run -- serve` 在本地预览
4. 将 `public` 目录下的文件部署到服务器

## Markdown 示例

### 代码块

```rust
fn main() {
    println!("Hello, RustPress!");
}
```

### 表格

| 特性 | 描述 |
|------|------|
| 快速 | 基于 Rust 的高性能 |
| 简单 | 易于使用的命令行界面 |
| 灵活 | 可定制的模板系统 |

### 列表

- 项目 1
- 项目 2
- 项目 3

> 这个示例文档展示了 RustPress 的基本功能。