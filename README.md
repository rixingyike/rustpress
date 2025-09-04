# RustPress

一个使用Rust语言编写的静态博客生成器，类似于Zola和Hugo，可以快速将Markdown格式的文章编译成HTML文件，方便部署到GitHub Pages等静态网站托管服务。

## 开发历史

 - 项目初始化：确定目标与基础目录结构
 - 集成 Tera 模板：`base.html`、`index.html`、`post.html`
 - 实现 Markdown 编译：生成 `public/` 静态页
 - 增加标签与归档模板：`tags.html`、`archives.html`
 - 添加示例内容：`mdsource/` 与 `public/` 示例文件
 - 完成首次提交并整理 `.gitignore`

## 特性

- 🚀 **快速**：使用Rust语言编写，编译速度快，生成网站高效
- 📝 **支持Markdown**：使用Markdown格式编写文章，简单易用
- 🎨 **模板系统**：使用Tera模板引擎，支持自定义网站外观
- 📦 **轻量级**：无运行时依赖，生成的网站可以直接部署
- 🔧 **简单易用**：提供直观的命令行界面

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
title = "我的第一篇文章"
date = 2023-01-01
categories = ["技术"]
tags = ["Rust", "博客"]
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
