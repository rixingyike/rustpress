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
}
