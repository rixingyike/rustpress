//! 开发服务器模块
//!
//! 提供本地预览功能，自动挂载插件 API 路由

use crate::config::Config;
use crate::error::{Error, Result};
use crate::plugins;
use axum::Router;
use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;
use tokio;
use tokio::sync::mpsc;
use tower_http::services::{ServeDir, ServeFile};
use notify::{Watcher, RecursiveMode};

/// 开发服务器
pub struct DevServer;

impl DevServer {
    /// 启动服务器（支持优雅关闭）
    pub async fn serve<P: AsRef<std::path::Path>>(
        port: u16,
        output_dir: P,
        config: Option<&Config>,
        shutdown: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> Result<()> {
        let output_dir = output_dir.as_ref().to_path_buf();
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        // 静态文件服务
        let static_service = ServeDir::new(&output_dir)
            .not_found_service(ServeFile::new(output_dir.join("index.html")));

        // 创建路由，自动收集所有插件的 API 路由
        let mut app = if let Some(cfg) = config {
            plugins::collect_api_routes(cfg)
        } else {
            Router::new()
        };

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
                if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() {
                    let _ = tx_clone.blocking_send(());
                }
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
            // 这里使用 tokio::spawn 异步执行，不阻塞 server 启动
            tokio::spawn(async move {
                // 等待一小会儿确保 server 已就绪
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
            res = Self::serve(port, &output_dir, Some(&config), shutdown) => res,
            _ = async {
                while let Some(_) = rx.recv().await {
                    // 防抖 200ms
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    while let Ok(_) = rx.try_recv() {}
                    
                    println!("检测到内容或模板变动，正在自动重构...");
                    if let Ok(new_cfg) = Config::from_file(&config_path) {
                        if let Ok(generator_inc) = crate::generator::Generator::new(new_cfg, &md_dir) {
                            if let Err(e) = generator_inc.build_incremental(&md_dir, &output_dir) {
                                eprintln!("自动重构失败: {}", e);
                            } else {
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

        rt.block_on(Self::serve(port, output_dir, config_owned.as_ref(), std::future::pending()))
    }
}

/// 中间件：强制禁用浏览器缓存（仅用于预览开发服务）
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
