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
    utils::{build_theme_css, read_template_file},
};
use std::path::Path;
use std::process::exit;

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::New { name, force } => new_project(name, *force),
        Commands::Build { output_dir } => build_site(&cli.md_dir, output_dir, &cli.config),
        Commands::BuildDev { output_dir } => build_dev_site(&cli.md_dir, output_dir, &cli.config),
        Commands::BuildCss => build_theme_css(),
        Commands::Serve { port, output_dir } => serve_site(*port, &cli.md_dir, output_dir, &cli.config),
        Commands::Dev { port, output_dir } => dev_site(*port, &cli.md_dir, output_dir, &cli.config),
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
    let example_post = r#"+++
title = "第一篇文章"
date = 2023-01-01
categories = ["技术"]
tags = ["Rust", "博客"]
+++

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
    let base_template = read_template_file("src/themes/default/templates/base.html")?;
    let index_template = read_template_file("src/themes/default/templates/index.html")?;
    let post_template = read_template_file("src/themes/default/templates/post.html")?;
    
    // 创建模板文件
    std::fs::write(project_path.join("templates/base.html"), base_template)?;
    std::fs::write(project_path.join("templates/index.html"), index_template)?;
    std::fs::write(project_path.join("templates/post.html"), post_template)?;
    
    println!("成功创建博客项目: {}", name);
    println!("项目结构:");
    println!("  {}/content/       - 存放Markdown文章", name);
    println!("  {}/templates/     - 存放模板文件", name);
    println!("  {}/static/        - 存放静态资源（CSS、JS、图片等）", name);
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
fn build_site(md_dir: &str, output_dir: &str, config_file: &str) -> Result<()> {
    let config = Config::from_file(config_file)?;
    let generator = Generator::new(config)?;
    generator.build(md_dir, output_dir)
}

/// 开发环境构建（包含 CSS 编译）
fn build_dev_site(md_dir: &str, output_dir: &str, config_file: &str) -> Result<()> {
    println!("开发环境构建中...");
    
    // 先构建 CSS
    println!("正在构建主题 CSS...");
    build_theme_css()?;
    
    // 再构建网站
    println!("正在构建网站...");
    build_site(md_dir, output_dir, config_file)?;
    
    println!("开发环境构建完成！");
    Ok(())
}

/// 在本地预览博客
fn serve_site(port: u16, md_dir: &str, output_dir: &str, config_file: &str) -> Result<()> {
    // 首先构建网站
    build_site(md_dir, output_dir, config_file)?;
    
    // 启动服务器
    DevServer::serve_sync(port, output_dir)
}

/// 开发模式：构建并启动本地预览服务器
fn dev_site(port: u16, md_dir: &str, output_dir: &str, config_file: &str) -> Result<()> {
    println!("开发模式启动中...");
    
    // 先进行开发环境构建
    build_dev_site(md_dir, output_dir, config_file)?;
    
    // 启动服务器
    DevServer::serve_sync(port, output_dir)
}