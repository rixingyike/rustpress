//! 开发服务器模块
//!
//! 提供本地预览功能，自动挂载插件 API 路由

use crate::config::Config;
use crate::error::{Error, Result};
use crate::plugins;
use axum::{
    Router,
    extract::Multipart,
    response::Json,
    routing::post,
};
use serde_json::json;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio;
use tokio::sync::mpsc;
use tower_http::services::{ServeDir, ServeFile};
use notify::{Watcher, RecursiveMode};

/// 应用状态
struct AppState {
    md_dir: Option<PathBuf>,
}

/// 开发服务器
pub struct DevServer;

impl DevServer {
    /// 启动服务器（支持优雅关闭）
    pub async fn serve<P: AsRef<std::path::Path>>(
        port: u16,
        output_dir: P,
        config: Option<&Config>,
        md_dir: Option<PathBuf>,
        shutdown: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> Result<()> {
        let output_dir = output_dir.as_ref().to_path_buf();
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let state = Arc::new(AppState { md_dir });

        // 静态文件服务
        let static_service = ServeDir::new(&output_dir)
            .not_found_service(ServeFile::new(output_dir.join("index.html")));

        // 创建路由，自动收集所有插件的 API 路由
        let mut app = if let Some(cfg) = config {
            plugins::collect_api_routes(cfg)
        } else {
            Router::new()
        };

        // Pushpen tweet 发表 API
        app = app.route(
            "/api/tweets",
            post({
                let state = Arc::clone(&state);
                move |multipart: Multipart| {
                    let state = Arc::clone(&state);
                    async move { handle_post_tweet(multipart, state).await }
                }
            }),
        );

        // 静态文件路由放在最后作为 fallback
        app = app
            .fallback_service(static_service)
            .layer(axum::middleware::from_fn(set_no_cache_headers));

        // 启动服务器
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| Error::Server(format!("无法绑定地址 {}: {}", addr, e)))?;

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown)
            .await
            .map_err(|e| Error::Server(format!("服务器运行错误: {}", e)))?;

        Ok(())
    }

    /// 启动实时预览服务器（边改边看模式）
    pub async fn serve_live<P: AsRef<Path>, Q: AsRef<Path>>(
        port: u16,
        md_dir: P,
        output_dir: Q,
        config_file: &str,
        open_browser: bool,
        shutdown: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> Result<()> {
        let md_dir = md_dir.as_ref().to_path_buf();
        let output_dir = output_dir.as_ref().to_path_buf();
        let config_file_owned = config_file.to_string();

        // 1. 初始化配置与构建
        let config_path = crate::utils::resolve_config_toml_path_read(&md_dir, &config_file_owned);
        let config = Config::from_file(&config_path)?;
        let generator = crate::generator::Generator::new(config.clone(), &md_dir)?;
        
        // 首次构建
        generator.build(&md_dir, &output_dir)?;

        // 2. 准备文件监听
        let (tx, mut rx) = mpsc::channel(100);
        let tx_clone = tx.clone();
        
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                if !(event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove()) {
                    return;
                }
                // 过滤 build.toml 变更（增量编译自身写入会触发循环）
                if event.paths.iter().any(|p| p.file_name().and_then(|n| n.to_str()) == Some("build.toml")) {
                    return;
                }
                let _ = tx_clone.blocking_send(());
            }
        }).map_err(|e| Error::Server(format!("无法初始化监听器: {}", e)))?;

        // 监听 MD 内容目录
        watcher.watch(&md_dir, RecursiveMode::Recursive)
            .map_err(|e| Error::Server(format!("监听目录失败: {}", e)))?;
            
        // 监听模板目录（若存在）
        let runtime_paths = crate::utils::RuntimePathsBuilder::new()
            .md_dir(&md_dir)
            .theme_name(config.theme_name())
            .build();
        if runtime_paths.theme_templates_dir.exists() {
            watcher.watch(&runtime_paths.theme_templates_dir, RecursiveMode::Recursive)
                .map_err(|e| Error::Server(format!("监听模板目录失败: {}", e)))?;
        }

        // 3. 并行运行服务与监听循环
        println!("预览服务器正在运行，已开启实时热重载 (Hot Reload)...");

        // 如果需要，启动时自动打开网页
        if open_browser {
            let url = format!("http://localhost:{}", port);
            println!("正在自动为您打开预览网页: {}", url);
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(300)).await;
                #[cfg(target_os = "macos")]
                let _ = std::process::Command::new("open").arg(&url).status();
                #[cfg(target_os = "linux")]
                let _ = std::process::Command::new("xdg-open").arg(&url).status();
                #[cfg(target_os = "windows")]
                let _ = std::process::Command::new("cmd").args(["/C", "start", &url]).status();
            });
        }
        
        tokio::select! {
            res = Self::serve(port, &output_dir, Some(&config), Some(md_dir.clone()), shutdown) => res,
            _ = async {
                while let Some(_) = rx.recv().await {
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    while let Ok(_) = rx.try_recv() {}
                    
                    println!("检测到内容或模板变动，正在自动重构...");
                    if let Ok(new_cfg) = Config::from_file(&config_path) {
                        if let Ok(generator_inc) = crate::generator::Generator::new(new_cfg, &md_dir) {
                            if let Err(e) = generator_inc.build_incremental(&md_dir, &output_dir) {
                                eprintln!("自动重构失败: {}", e);
                            } else {
                                let _ = crate::utils::log_build_info(&md_dir);
                                println!("自动重构完成！");
                            }
                        }
                    }
                }
            } => Ok(()),
        }
    }

    /// 同步启动服务器（用于阻塞调用）
    pub fn serve_sync<P: AsRef<std::path::Path>>(
        port: u16,
        output_dir: P,
        config: Option<&Config>,
    ) -> Result<()> {
        let output_dir = output_dir.as_ref().to_path_buf();
        let config_owned = config.cloned();

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| Error::Server(format!("无法创建异步运行时: {}", e)))?;

        rt.block_on(Self::serve(port, output_dir, config_owned.as_ref(), None, std::future::pending()))
    }
}

/// 中间件：强制禁用浏览器缓存
async fn set_no_cache_headers(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::http::{header, HeaderValue};
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-store, must-revalidate"),
    );
    headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    headers.insert(header::EXPIRES, HeaderValue::from_static("0"));
    
    response
}

/// 处理发表 tweet
async fn handle_post_tweet(
    mut multipart: Multipart,
    state: Arc<AppState>,
) -> Json<serde_json::Value> {
    let md_dir = match &state.md_dir {
        Some(d) => d.clone(),
        None => {
            return Json(json!({"ok": false, "error": "未配置源目录"}));
        }
    };

    let mut text = String::new();
    let mut images: Vec<(String, Vec<u8>)> = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "text" {
            text = field.text().await.unwrap_or_default();
        } else if name == "images" {
            let filename = field
                .file_name()
                .unwrap_or("image.jpg")
                .to_string();
            let data = field.bytes().await.unwrap_or_default().to_vec();
            if !data.is_empty() {
                images.push((filename, data));
            }
        }
    }

    if text.trim().is_empty() && images.is_empty() {
        return Json(json!({"ok": false, "error": "内容和图片不能同时为空"}));
    }

    // 生成时间戳
    let now = chrono::Local::now();
    let date_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let year = now.format("%Y").to_string();
    let month = now.format("%m").to_string();
    let ts = now.format("%Y%m%d%H%M%S").to_string();

    // 目标路径: source/tweets/YYYY/MM/
    let tweets_dir = md_dir.join("tweets").join(&year).join(&month);
    if let Err(e) = std::fs::create_dir_all(&tweets_dir) {
        return Json(json!({"ok": false, "error": format!("创建目录失败: {}", e)}));
    }

    // 保存图片到 tweets/YYYY/MM/assets/
    let mut image_urls: Vec<String> = Vec::new();
    let assets_dir = tweets_dir.join("assets");
    if !images.is_empty() {
        if let Err(e) = std::fs::create_dir_all(&assets_dir) {
            return Json(json!({"ok": false, "error": format!("创建 assets 目录失败: {}", e)}));
        }
        for (i, (filename, data)) in images.iter().enumerate() {
            let ext = Path::new(filename)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("jpg");
            let save_name = format!("{}_{}.{}", ts, i, ext);
            let save_path = assets_dir.join(&save_name);
            if let Err(e) = std::fs::write(&save_path, data) {
                return Json(json!({"ok": false, "error": format!("保存图片失败: {}", e)}));
            }
            image_urls.push(format!("/tweets/{}/{}/assets/{}", year, month, save_name));
        }
    }

    // 生成 Markdown 文件
    let slug = format!("{}.md", &ts);
    let mut front = format!("---\ndate: {}\nlayout: tweet\n", date_str);
    if !image_urls.is_empty() {
        front.push_str("images:\n");
        for url in &image_urls {
            front.push_str(&format!("  - \"{}\"\n", url));
        }
    }
    front.push_str("---\n\n");
    front.push_str(&text);

    let md_path = tweets_dir.join(&slug);
    if let Err(e) = std::fs::write(&md_path, &front) {
        return Json(json!({"ok": false, "error": format!("保存推文失败: {}", e)}));
    }

    Json(json!({"ok": true, "slug": slug, "image_count": image_urls.len()}))
}
