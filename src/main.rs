use clap::{Parser, Subcommand};
use std::path::Path;
use std::process::exit;
use anyhow::{Context, Result};
use serde_yaml;
use serde_json;
use walkdir;
use pulldown_cmark;
use pulldown_cmark::html;
use tera;
use std::io::Read;
use axum::{routing::get, Router};
use tower_http::services::{ServeDir, ServeFile};
use tokio;
use std::net::SocketAddr;
use axum::response::IntoResponse;
use axum::body::Body;
use hyper::Response;

// 命令行参数解析
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// 指定Markdown源文件目录
    #[arg(short, long, default_value = "mdsource")]
    md_dir: String,
    
    /// 指定配置文件
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

#[derive(Subcommand)]
enum Commands {
    /// 创建新的博客项目
    New {
        /// 项目名称
        name: String,
        /// 覆盖已存在目录
        #[arg(short, long, default_value_t = false)]
        force: bool,
    },
    
    /// 构建博客网站
    Build {
        /// 指定输出目录
        #[arg(short, long, default_value = "public")]
        output_dir: String,
    },
    
    /// 在本地预览博客
    Serve {
        /// 服务器端口
        #[arg(short, long, default_value_t = 1111)]
        port: u16,
        
        /// 指定输出目录
        #[arg(short, long, default_value = "public")]
        output_dir: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::New { name, force } => new_project(name, *force),
        Commands::Build { output_dir } => build_site(&cli.md_dir, output_dir, &cli.config),
        Commands::Serve { port, output_dir } => serve_site(*port, &cli.md_dir, output_dir, &cli.config),
    }
}

// 读取模板文件的辅助函数
fn read_template_file(path: &str) -> Result<String> {
    let mut file = std::fs::File::open(path)
        .with_context(|| format!("无法打开模板文件: {}", path))?;
    
    let mut content = String::new();
    file.read_to_string(&mut content)
        .with_context(|| format!("无法读取模板文件内容: {}", path))?;
    
    Ok(content)
}

// 创建新的博客项目
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
    let config_content = "[site]\nname = \"我的博客\"\ndescription = \"使用RustPress创建的博客\"\nauthor = \"作者\"\nbase_url = \"https://example.com\"\n\n[taxonomies]\ncategory = \"categories\"\ntag = \"tags\"\n";
    
    std::fs::write(project_path.join("config.toml"), config_content)?;
    
    // 创建示例文章
    let example_post = "+++\ntitle = \"第一篇文章\"\ndate = 2023-01-01\ncategories = [\"技术\"]\ntags = [\"Rust\", \"博客\"]\n+++\n\n# 欢迎使用 RustPress\n\n这是使用 [RustPress](https://github.com/example/rustpress) 创建的第一篇博客文章。\n\n## 特性\n\n- 快速的静态博客生成器\n- 支持 Markdown 格式\n- 简单易用的模板系统\n- 使用 Rust 语言编写\n\n## 开始使用\n\n1. 创建新的文章\n2. 编辑 Markdown 内容\n3. 运行 `rustpress build` 生成静态网站\n4. 部署到 GitHub Pages 或其他静态网站托管服务\n";
    
    std::fs::write(project_path.join("content/first-post.md"), example_post)?;
    
    // 读取默认模板文件
    let base_template = read_template_file("src/templates/base.html")?;
    let index_template = read_template_file("src/templates/index.html")?;
    let post_template = read_template_file("src/templates/post.html")?;
    
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

// 构建博客网站
fn build_site(md_dir: &str, output_dir: &str, config_file: &str) -> Result<()> {
    println!("正在构建网站...");
    
    // 确保输出目录存在
    std::fs::create_dir_all(output_dir)?;
    
    // 读取配置文件
    let config_content = std::fs::read_to_string(config_file)
        .with_context(|| format!("无法读取配置文件 {}", config_file))?;
    
    let config: toml::Value = toml::from_str(&config_content)
        .context("配置文件格式错误")?;
    
    // 初始化模板引擎
    let tera = tera::Tera::new("templates/**/*")?;
    
    // 添加全局上下文变量
    let mut context = tera::Context::new();
    context.insert("site", &config.get("site").unwrap_or(&toml::Value::Table(toml::map::Map::new())));
    // 插入当前时间戳变量 now
    use chrono::prelude::*;
    let now = Utc::now();
    context.insert("now", &now);
    
    // 列出指定目录下的所有Markdown文件
    let posts = list_posts(md_dir)?;
    
    // 渲染首页
    let mut index_context = context.clone();
    index_context.insert("posts", &posts);
    
    let rendered = tera.render("index.html", &index_context)?;
    std::fs::write(format!("{}/index.html", output_dir), rendered)
        .with_context(|| format!("无法写入首页文件到 {}", output_dir))?;
    
    // 渲染每篇文章
    for post in &posts {
        let mut post_context = context.clone();
        post_context.insert("page", &post);
        
    let rendered = tera.render("post.html", &post_context)?;
        if let Some(slug) = post.get("slug").and_then(|s| s.as_str()) {
            let filename = format!("{}/{}.html", output_dir, slug);
            std::fs::write(filename, rendered)
                .with_context(|| format!("无法写入文章文件: {}", slug))?;
        }
    }
    
    println!("网站构建成功！静态文件已生成到 {} 目录。", output_dir);
    
    Ok(())
}

// 在本地预览博客
fn serve_site(port: u16, md_dir: &str, output_dir: &str, config_file: &str) -> Result<()> {
    // 首先构建网站
    build_site(md_dir, output_dir, config_file)?;
    
    println!("正在启动本地服务器，端口: {}", port);
    println!("请在浏览器中访问: http://localhost:{}", port);
    println!("按 Ctrl+C 停止服务器");
    
    // 创建一个异步运行时
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("无法创建异步运行时")?
        .block_on(async {
            let addr = SocketAddr::from(([127, 0, 0, 1], port));
            
            // 创建路由
            let app = Router::new()
                    .fallback_service(
                        ServeDir::new(output_dir)
                            .not_found_service(ServeFile::new(format!("{}/index.html", output_dir)))
                    );
            
            // 启动服务器
                let listener = tokio::net::TcpListener::bind(addr).await.with_context(|| format!("无法启动服务器: {}", addr))?;
                axum::serve(listener, app)
                    .await
                    .with_context(|| format!("无法启动服务器: {}", addr))?;
            
            Ok::<(), anyhow::Error>(())
        })?;
    
    Ok(())
}

// 列出所有文章
fn list_posts(md_dir: &str) -> Result<Vec<serde_json::Value>> {
    let mut posts = Vec::new();
    
    // 遍历指定目录下的所有.md文件
    let content_dir = Path::new(md_dir);
    
    // 检查目录是否存在
    if !content_dir.exists() {
        println!("警告: Markdown目录 '{}' 不存在，创建空目录...", md_dir);
        std::fs::create_dir_all(content_dir)?;
    }
    
    for entry in walkdir::WalkDir::new(content_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.path().extension().map_or(false, |ext| ext == "md") {
            let content = std::fs::read_to_string(entry.path())
                .with_context(|| format!("无法读取文件 {:?}", entry.path()))?;
            // 解析文章元数据和内容
            if let Some(mut post) = parse_post(&content, entry.path())? {
                // 自动生成 createDate 字段（仅日期部分，供模板 date filter 使用）
                // 如果没有 title 字段，用 slug 作为 title
                let slug = post.get("slug").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let has_title = post.get("title").is_some();
                if !has_title {
                    if let Some(obj) = post.as_object_mut() {
                        obj.insert("title".to_string(), serde_json::Value::String(slug.clone()));
                    }
                }
                posts.push(post);
            }
        }
    }
    
    // 按日期排序（最新的在前）
    posts.sort_by(|a, b| {
        let date_a = a.get("date").and_then(|d| d.as_str()).unwrap_or("");
        let date_b = b.get("date").and_then(|d| d.as_str()).unwrap_or("");
        date_b.cmp(date_a)
    });
    
    Ok(posts)
}

// 解析单篇文章
fn parse_post(content: &str, path: &Path) -> Result<Option<serde_json::Value>> {
    // 检查 front matter 类型
    let (fm_marker, end_marker) = if content.starts_with("+++") {
        ("+++", "+++\n")
    } else if content.starts_with("---") {
        ("---", "---\n")
    } else {
        return Ok(None);
    };

    // 查找 front matter 结束位置，支持 --- 或 ---\n 结尾
    let start = fm_marker.len();
    let end = if let Some(pos) = content[start..].find(end_marker) {
        start + pos
    } else if let Some(pos) = content[start..].find(fm_marker) {
        start + pos
    } else {
        return Ok(None);
    };

    let front_matter = &content[start..end];
    let body = &content[end + fm_marker.len()..];

    // 解析front matter（YAML）
    let metadata: serde_yaml::Value = serde_yaml::from_str(front_matter)
        .with_context(|| format!("解析front matter失败: {:?}", path))?;

    // 转换元数据为JSON
    let metadata_json = serde_json::to_value(&metadata)?;

    // 解析Markdown为HTML
    let html = markdown_to_html(body);

    // 优先使用 front matter 中的 slug 字段，否则用文件名
    let mut slug = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
    if let serde_json::Value::Object(ref obj) = metadata_json {
        if let Some(serde_json::Value::String(s)) = obj.get("slug") {
            if !s.is_empty() {
                slug = s.clone();
            }
        }
    }

    // 创建完整的文章对象
    let post = match metadata_json {
        serde_json::Value::Object(mut obj) => {
            obj.insert("content".to_string(), serde_json::Value::String(html));
            obj.insert("slug".to_string(), serde_json::Value::String(slug));
            serde_json::Value::Object(obj)
        },
        _ => {
            let mut obj = serde_json::Map::new();
            obj.insert("content".to_string(), serde_json::Value::String(html));
            obj.insert("slug".to_string(), serde_json::Value::String(slug));
            serde_json::Value::Object(obj)
        }
    };

    Ok(Some(post))
}

// 将Markdown转换为HTML
fn markdown_to_html(markdown: &str) -> String {
    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_TABLES);
    options.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
    options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    options.insert(pulldown_cmark::Options::ENABLE_TASKLISTS);
    
    let parser = pulldown_cmark::Parser::new_ext(markdown, options);
    let mut html = String::new();
    html::push_html(&mut html, parser);
    
    html
}
