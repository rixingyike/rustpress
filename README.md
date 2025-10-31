# RustPress

一个使用Rust语言编写的个人静态博客生成器，类似于Zola和Hugo，可以快速将Markdown格式的文章编译成HTML文件，方便部署到GitHub Pages等静态网站托管服务。

## 开发计划

### 已完成 ✅
- ✅ 项目初始化：确定目标与基础目录结构
- ✅ 集成 Tera 模板：`base.html`、`index.html`、`post.html`
- ✅ 实现 Markdown 编译：生成 `public/` 静态页
- ✅ 增加标签与归档模板：`tags.html`、`archives.html`
- ✅ 添加示例内容：`mdsource/` 与 `public/` 示例文件
- ✅ 完成首次提交并整理 `.gitignore`
- ✅ **完整的CLI框架**：支持 `new`、`build`、`serve` 命令
- ✅ **Markdown解析**：使用 pulldown-cmark 解析Markdown到HTML
- ✅ **模板系统**：集成Tera模板引擎，支持多种模板
- ✅ **配置文件支持**：config.toml 配置系统
- ✅ **静态网站生成**：完整的构建流程
- ✅ **本地预览服务器**：使用Axum提供HTTP服务
- ✅ **标签和归档功能**：支持按标签和时间归档
- ✅ **Front Matter解析**：支持YAML格式的文章元数据
- ✅ **完整的子目录分类功能**：支持按分类自动生成子目录和索引页面

### 🎯 近期任务
- [ ] **测试和验证现有功能**
  - [ ] 测试 `cargo run -- build` 命令
  - [ ] 测试 `cargo run -- serve` 命令  
  - [ ] 验证模板渲染是否正常
  - [ ] 检查生成的HTML文件质量

- [ ] **完善全局配置文件**
  - [x] 扩展 config.toml 配置项
    - [x] 添加作者信息配置
    - [x] 添加站点基础地址配置（协议+域名）
    - [x] 添加ICP备案号配置
    - [x] 添加起始年份配置
  - [x] 在模板中集成配置变量
  - [ ] 添加配置文件验证和默认值处理

- [ ] **完善错误处理和用户体验**
  - [ ] 添加更详细的错误信息和堆栈跟踪
  - [ ] 改进命令行输出和进度提示
  - [ ] 添加构建过程的详细日志
  - [ ] 优化错误恢复机制

- [ ] **添加静态资源处理**
  - [ ] CSS/JS/图片文件的复制和处理
  - [ ] 静态资源的优化和压缩
  - [ ] 支持 `static/` 目录自动复制到 `public/`
  - [ ] 添加资源文件监听和热重载

### 🔧 中期任务
- [ ] **增强功能**
  - [ ] 添加草稿功能（draft posts）
  - [ ] 支持更多Front Matter格式（JSON、TOML）
  - [ ] 添加分页功能
  - [ ] RSS/Atom feed生成
  - [ ] 站点地图（sitemap.xml）生成
  - [ ] 搜索功能支持

- [ ] **性能优化**
  - [ ] 增量构建支持（只重建修改的文件）
  - [ ] 并行处理文件
  - [ ] 缓存机制实现
  - [ ] 大型网站构建优化

- [ ] **模板和主题**
  - [ ] 完善默认主题样式
  - [ ] 支持自定义CSS和JavaScript
  - [ ] 响应式设计优化
  - [ ] 暗色主题支持

### 📚 长期任务
- [ ] **测试和文档**
  - [ ] 编写单元测试覆盖核心功能
  - [ ] 集成测试和端到端测试
  - [ ] 完善README和使用文档
  - [ ] 添加示例项目和教程
  - [ ] API文档生成

- [ ] **高级功能**
  - [ ] 主题系统和主题市场
  - [ ] 插件架构设计
  - [ ] 多语言支持（i18n）
  - [ ] 评论系统集成
  - [ ] 数据分析和统计

- [ ] **部署和集成**
  - [ ] GitHub Actions 自动部署
  - [ ] Docker 容器化支持
  - [ ] Vercel/Netlify 一键部署
  - [ ] CDN 集成优化

### 🚀 下一步行动建议
1. **优先级1**：测试当前实现，确保基本功能正常工作
2. **优先级2**：完善错误处理，提升用户体验
3. **优先级3**：添加静态资源处理，完善构建流程
4. **优先级4**：根据测试结果，确定具体的功能增强方向

## 特性

- 🚀 **快速**：使用Rust语言编写，编译速度快，生成网站高效
- 📝 **支持Markdown**：使用Markdown格式编写文章，简单易用
- 🎨 **模板系统**：使用Tera模板引擎，支持自定义网站外观
- 📦 **轻量级**：无运行时依赖，生成的网站可以直接部署
- 🔧 **简单易用**：提供直观的命令行界面

## 设计理念

RustPress采用**倒分页设计**：第1页包含最早的文章，最大页数包含最新的文章。这种设计支持增量编译，新文章只影响最新页，旧页面保持稳定，构建效率随文章数量增加而缓慢增长。

**归档页设计**：按年归档，archives.html为归档首页，包含年份索引链接（倒序排列）。年份归档页位于archives/2024.html等路径，已过完年份的归档页无需重新生成，每次编译只生成总归档页和当前年份归档页。

**分类页设计**：categories.html展示source目录结构，便于结构化写作。分类页只显示分类关系和文章数量，点击分类跳转到子分类文章列表页。子分类文章列表页不分页，直接展示该分类下所有文章。采用扁平URL结构：categories/技术架构/微服务架构.html。

**标签页设计**：tags.html展示所有标签及文章数目，不展示文章列表。点击标签跳转到单标签文章列表页，采用倒分页设计：第1页包含最旧文章，最大页数包含最新文章。URL结构：tags/rust.html（第1页）、tags/rust/2.html（第2页）、tags/rust/3.html（第3页）。需要创建tag.html模板，包含标签标题、倒分页文章列表和分页导航。

**搜索页设计**：search.html作为搜索功能入口，采用纯前端实现方案。构建时预生成search.json索引文件，客户端通过JavaScript在内存中完成实时搜索，无需后端支持，实现静态网站的高效搜索体验。

**侧边栏设计**：采用组件化设计，在以下页面展示侧边栏：
- **首页**（index.html）- 已实现，展示网站概览和导航
- **分类列表页**（categories.html）及子分类文章列表页
- **标签列表页**（tags.html）及子标签文章列表页  
- **归档页**（archives.html）及年归档页

侧边栏内容包含：
- **作者信息** - 姓名、头像、简介、位置、社交链接
- **广告位2** - 可配置的推广内容
- **相关文章** - 最新发布的文章列表
- **热门分类** - 最近更新的一些分类
- **热门标签** - 文章最多的一些标签
- **广告位3** - 可配置的推广内容

设计原则是保持内容稳定性与用户体验的平衡，侧边栏内容以相对稳定信息为主，同时包含必要的动态更新内容。

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
