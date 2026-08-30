//! 命令行参数处理模块

use clap::{Parser, Subcommand};

/// RustPress 命令行工具
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// 指定Markdown源文件目录
    #[arg(short, long, default_value = "source")]
    pub md_dir: String,

    /// 指定配置文件（默认从 md_dir 下解析）
    #[arg(short, long, default_value = "config.toml")]
    pub config: String,
}

/// 可用的命令
#[derive(Subcommand)]
pub enum Commands {
    /// 创建新的博客项目（包含 content/templates/static 等完整项目结构）
    New {
        /// 项目名称
        name: String,
        /// 覆盖已存在目录
        #[arg(short, long, default_value_t = false)]
        force: bool,
    },

    /// 初始化博客项目（自动识别根目录，生成骨架、示例文章及 GitHub Action 部署脚本，支持对现有目录查漏补缺）
    Init,

    /// 生产环境构建（快速，只处理 Markdown）
    Build {
        /// 指定输出目录
        #[arg(short, long, default_value = "public")]
        output_dir: String,

        /// 开启增量编译（基于 build.toml 的 last_build_time）
        #[arg(long, default_value_t = false)]
        incremental: bool,
    },

    /// 开发环境构建（包含 CSS 编译）
    BuildDev {
        /// 指定输出目录
        #[arg(short, long, default_value = "public")]
        output_dir: String,

        /// 开启增量编译（构建前端资源后，按增量渲染文章）
        #[arg(long, default_value_t = false)]
        incremental: bool,
    },

    /// 构建主题 CSS
    BuildCss,

    /// 开发模式：构建并启动具备热重载功能的本地预览服务器
    Serve {
        /// 服务器端口
        #[arg(short, long, default_value_t = 1111)]
        port: u16,

        /// 指定输出目录
        #[arg(short, long, default_value = "public")]
        output_dir: String,

        /// 启动前执行增量编译
        #[arg(long, default_value_t = false)]
        incremental: bool,

        /// 关闭 hotreload（不监听模板文件变化）
        #[arg(long, default_value_t = false)]
        no_hotreload: bool,
    },

    /// 重新生成首页侧边栏数据到 build.toml
    BuildSidebar,

    /// 创建新的日常博客文章 (source/YYYY/N.md)
    #[command(alias = "new_blog", alias = "new-post", alias = "new_post")]
    NewBlog {
        /// 文章标题
        #[arg(default_value = "新标题")]
        title: String,
    },

    /// 创建新的专栏/连载章节文章并自动同步 catalog 目录索引
    #[command(alias = "new_article", alias = "new-doc", alias = "new_doc")]
    NewArticle {
        /// 专栏编号或目录名 (如 1, rustpress)
        column: String,

        /// 文章标题 (如 "1.3.进阶特性")
        #[arg(default_value = "新标题")]
        title: String,
    },

    /// 发布简短闲言/动态 (source/tweets/YYYY/MM/YYYYMMDDHHMMSS.md)
    #[command(alias = "new_tweet", alias = "new-status", alias = "new_status")]
    NewTweet {
        /// 闲言内容
        #[arg(default_value = "")]
        content: String,
    },

    /// 自动检查并更新专栏 catalog 目录索引
    #[command(alias = "make_catalog")]
    MakeCatalog {
        /// 指定专栏编号或目录名 (留空处理全部专栏)
        column: Option<String>,

        /// 强制处理所有专栏
        #[arg(short, long, default_value_t = false)]
        all: bool,
    },

    /// 自动生成专栏高清封面图片 (assets/cover.png)
    #[command(alias = "make_cover")]
    MakeCover {
        /// 指定专栏编号或目录名 (留空处理全部专栏)
        column: Option<String>,

        /// 强制处理所有专栏
        #[arg(short, long, default_value_t = false)]
        all: bool,

        /// 封面风格 (theme 浅色高雅 或 red 故宫红)
        #[arg(short, long, default_value = "theme")]
        style: String,
    },
}
