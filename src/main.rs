//! RustPress - 一个快速的静态博客生成器
//!
//! 主程序入口

use clap::Parser;
use rustpress::{
    cli::{Cli, Commands},
    config::Config,
    error::Result,
    generator::Generator,
    server::DevServer,
    utils::{build_theme_css, ensure_initial_setup, read_template_file},
};
use std::path::Path;
use std::process::exit;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::New { name, force } => new_project(name, *force),
        Commands::Init => {
            rustpress::utils::init_source_dir(&cli.md_dir, &cli.config)
        }
        Commands::Build {
            output_dir,
            incremental,
        } => build_site(&cli.md_dir, output_dir, &cli.config, *incremental),
        Commands::BuildDev {
            output_dir,
            incremental,
        } => build_dev_site(&cli.md_dir, output_dir, &cli.config, *incremental),
        Commands::BuildCss => {
            // 为 CSS 构建加载配置以确定主题名称
            // 启动时初始化（themes/config.toml/build.toml 及示例页）
            ensure_initial_setup(&cli.md_dir, &cli.config)?;
            // 配置读取优先 source 目录
            let config_path =
                rustpress::utils::resolve_config_toml_path_read(&cli.md_dir, &cli.config);
            let config = Config::from_file(&config_path)?;
            build_theme_css(&cli.md_dir, &config)
        }
        Commands::Serve {
            port,
            output_dir,
            incremental: _,
            hotreload,
        } => {
            if *hotreload {
                dev_site_hotreload(*port, &cli.md_dir, output_dir, &cli.config, false)
            } else {
                // 即使不显式开启 hotreload，当前 serve 也默认调用同步预览（单次构建后 serve）
                println!("以静态模式启动预览服务器...");
                // 此处复用之前的同步预览逻辑
                let config_path = rustpress::utils::resolve_config_toml_path_read(
                    std::path::Path::new(&cli.md_dir),
                    &cli.config,
                );
                let config = Config::from_file(&config_path)?;
                build_site(&cli.md_dir, output_dir, &cli.config, false)?;
                DevServer::serve_sync(*port, output_dir, Some(&config))
            }
        }
        Commands::BuildSidebar => build_sidebar(&cli.md_dir, &cli.config),
    }
}

/// 创建新的博客项目
fn new_project(name: &str, force: bool) -> Result<()> {
    let project_path = Path::new(name);

    // 检查目录是否已存在
    if project_path.exists() {
        if force {
            std::fs::remove_dir_all(project_path)?;
        } else {
            eprintln!("错误: 目录 '{}' 已存在 (如需覆盖请加 -f)", name);
            exit(1);
        }
    }

    // 创建必要的目录结构
    std::fs::create_dir_all(project_path.join("content"))?;
    std::fs::create_dir_all(project_path.join("templates"))?;
    std::fs::create_dir_all(project_path.join("static"))?;
    std::fs::create_dir_all(project_path.join("public"))?;

    // 创建配置文件
    let config_content = r#"[site]
name = "我的博客"
description = "使用RustPress创建的博客"
author = "作者"
base_url = "https://example.com"

[taxonomies]
category = "categories"
tag = "tags"
"#;

    std::fs::write(project_path.join("config.toml"), config_content)?;

    // 创建示例文章
    let example_post = r#"---
title: "第一篇文章"
createTime: 2023-01-01 12:00:00
categories: ["技术"]
tags: ["Rust", "博客"]
---

# 欢迎使用 RustPress

这是使用 [RustPress](https://github.com/example/rustpress) 创建的第一篇博客文章。

## 特性

- 快速的静态博客生成器
- 支持 Markdown 格式
- 简单易用的模板系统
- 使用 Rust 语言编写

## 开始使用

1. 创建新的文章
2. 编辑 Markdown 内容
3. 运行 `rustpress build` 生成静态网站
4. 部署到 GitHub Pages 或其他静态网站托管服务
"#;

    std::fs::write(project_path.join("content/first-post.md"), example_post)?;

    // 读取默认模板文件
    let base_template = read_template_file("themes/default/templates/base.html")?;
    let index_template = read_template_file("themes/default/templates/index.html")?;
    let post_template = read_template_file("themes/default/templates/post.html")?;

    // 创建模板文件
    std::fs::write(project_path.join("templates/base.html"), base_template)?;
    std::fs::write(project_path.join("templates/index.html"), index_template)?;
    std::fs::write(project_path.join("templates/post.html"), post_template)?;

    println!("成功创建博客项目: {}", name);
    println!("项目结构:");
    println!("  {}/content/       - 存放Markdown文章", name);
    println!("  {}/templates/     - 存放模板文件", name);
    println!(
        "  {}/static/        - 存放静态资源（CSS、JS、图片等）",
        name
    );
    println!("  {}/public/        - 生成的静态网站文件", name);
    println!("  {}/config.toml    - 配置文件", name);
    println!();
    println!("接下来的步骤:");
    println!("  1. 编辑 {} 目录下的配置文件和文章", name);
    println!("  2. 运行 `cd {} && cargo run -- build` 生成网站", name);
    println!("  3. 运行 `cd {} && cargo run -- serve` 在本地预览", name);

    Ok(())
}

/// 构建博客网站
fn build_site(md_dir: &str, output_dir: &str, config_file: &str, incremental: bool) -> Result<()> {
    use std::path::Path;
    // 启动时初始化（themes/config.toml/build.toml 及示例页）
    ensure_initial_setup(Path::new(md_dir), config_file)?;
    // 配置解析：优先从 source 目录读取，否则回退到项目根
    let config_path =
        rustpress::utils::resolve_config_toml_path_read(Path::new(md_dir), config_file);

    let config = Config::from_file(&config_path)?;
    let generator = Generator::new(config, Path::new(md_dir))?;

    // 根据 build.toml 的编译模式决定默认行为；命令行 --incremental 显式开启则覆盖为增量
    let file_mode = rustpress::utils::read_build_mode(std::path::Path::new(md_dir));
    let effective_incremental = if incremental {
        true
    } else {
        matches!(file_mode, rustpress::utils::BuildMode::Incremental)
    };

    // 首次构建（输出目录不存在或为空）强制全量生成
    let output_path = Path::new(output_dir);
    let is_first_build = if !output_path.exists() {
        true
    } else {
        std::fs::read_dir(output_path)
            .map(|mut rd| rd.next().is_none())
            .unwrap_or(true)
    };
    let final_incremental = if is_first_build {
        false
    } else {
        effective_incremental
    };

    // 记录编译日志到 md_dir/build.toml
    if final_incremental {
        generator.build_incremental(md_dir, output_dir)?;
    } else {
        generator.build(md_dir, output_dir)?;
    }

    // 最后更新构建时间
    rustpress::utils::log_build_info(Path::new(md_dir))
}

/// 开发环境构建（包含 CSS 编译）
fn build_dev_site(
    md_dir: &str,
    output_dir: &str,
    config_file: &str,
    incremental: bool,
) -> Result<()> {
    println!("开发环境构建中...");

    // 先构建 CSS
    println!("正在构建主题 CSS...");
    // 加载配置以确定主题名称
    // 启动时初始化（themes/config.toml/build.toml 及示例页）
    ensure_initial_setup(std::path::Path::new(md_dir), config_file)?;
    let config_path =
        rustpress::utils::resolve_config_toml_path_read(std::path::Path::new(md_dir), config_file);
    let config = Config::from_file(&config_path)?;
    build_theme_css(md_dir, &config)?;

    // 再构建网站
    println!("正在构建网站...");
    build_site(md_dir, output_dir, config_file, incremental)?;

    println!("开发环境构建完成！");
    Ok(())
}

/// 开发模式（hotreload）：构建并启动服务器，同时监听模板与内容变化自动重建
fn dev_site_hotreload(
    port: u16,
    md_dir: &str,
    output_dir: &str,
    config_file: &str,
    _incremental: bool,
) -> Result<()> {
    println!("开发模式（hotreload）启动中...");

    // 先进行一轮全量构建确保基础环境就绪
    build_dev_site(md_dir, output_dir, config_file, false)?;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| rustpress::error::Error::Server(format!("无法创建异步运行时: {}", e)))?;

    rt.block_on(async {
        DevServer::serve_live(
            port,
            Path::new(md_dir),
            Path::new(output_dir),
            config_file,
            true, // 自动打开浏览器
            std::future::pending(),
        ).await
    })
}

/// 重新生成首页侧边栏数据到 build.toml（热门文章/标签/分类）
fn build_sidebar(md_dir: &str, _config_file: &str) -> Result<()> {
    // 列出所有文章，基于当前内容重新生成侧边栏数据
    let posts = rustpress::post::PostParser::list_posts(md_dir)?;
    rustpress::utils::regenerate_sidebar(std::path::Path::new(md_dir), &posts)?;
    println!("已根据当前内容重新生成 build.toml 的侧边栏数据");
    Ok(())
}
