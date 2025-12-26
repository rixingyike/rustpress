//! 开发服务器模块
//!
//! 提供本地预览功能

use crate::error::{Error, Result};
use axum::Router;
use std::net::SocketAddr;
use tokio;
use tower_http::services::{ServeDir, ServeFile};

/// 开发服务器
pub struct DevServer;

impl DevServer {
    /// 启动服务器
    pub async fn serve<P: AsRef<std::path::Path>>(port: u16, output_dir: P) -> Result<()> {
        let output_dir = output_dir.as_ref();
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        println!("正在启动本地服务器，端口: {}", port);
        println!("请在浏览器中访问: http://localhost:{}", port);
        println!("按 Ctrl+C 停止服务器");

        // 创建路由
        let app = Router::new().fallback_service(
            ServeDir::new(output_dir)
                .not_found_service(ServeFile::new(output_dir.join("index.html"))),
        );

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
    pub fn serve_sync<P: AsRef<std::path::Path>>(port: u16, output_dir: P) -> Result<()> {
        let output_dir = output_dir.as_ref().to_path_buf();

        // 创建异步运行时
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| Error::Server(format!("无法创建异步运行时: {}", e)))?;

        rt.block_on(Self::serve(port, output_dir))
    }
}
