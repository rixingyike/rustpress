//! RustPress - 一个快速的静态博客生成器
//!
//! 这个库提供了构建静态博客网站的核心功能，包括：
//! - Markdown 文章解析
//! - 模板渲染
//! - 静态文件生成
//! - 开发服务器

pub mod cli;
pub mod config;
pub mod error;
pub mod generator;
pub mod post;
pub mod server;
pub mod template;
pub mod utils;

// 重新导出主要的公共类型和函数
pub use cli::{Cli, Commands};
pub use config::Config;
pub use error::{Error, Result};
pub use generator::Generator;
pub use post::{Post, PostParser};
pub use server::DevServer;
pub use template::TemplateEngine;
pub use utils::*;
