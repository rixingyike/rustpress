//! 开发服务器模块
//!
//! 提供本地预览功能，自动挂载插件 API 路由

use crate::config::Config;
use crate::error::{Error, Result};
use crate::plugins;
use axum::Router;
use std::net::SocketAddr;
use tokio;
use tower_http::services::{ServeDir, ServeFile};

/// 开发服务器
pub struct DevServer;

impl DevServer {
    /// 启动服务器
    pub async fn serve<P: AsRef<std::path::Path>>(
        port: u16,
        output_dir: P,
        config: Option<&Config>,
    ) -> Result<()> {
        let output_dir = output_dir.as_ref();
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        println!("正在启动本地服务器，端口: {}", port);
        println!("请在浏览器中访问: http://localhost:{}", port);
        println!("按 Ctrl+C 停止服务器");

        // 静态文件服务
        let static_service = ServeDir::new(output_dir)
            .not_found_service(ServeFile::new(output_dir.join("index.html")));

        // 创建路由，自动收集所有插件的 API 路由
        let mut app = if let Some(cfg) = config {
            plugins::collect_api_routes(cfg)
        } else {
            Router::new()
        };

        // 静态文件路由放在最后作为 fallback
        app = app.fallback_service(static_service);

        // 启动服务器
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| Error::Server(format!("无法绑定地址 {}: {}", addr, e)))?;

        axum::serve(listener, app)
            .await
            .map_err(|e| Error::Server(format!("服务器运行错误: {}", e)))?;

        Ok(())
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

        rt.block_on(Self::serve(port, output_dir, config_owned.as_ref()))
    }
}
