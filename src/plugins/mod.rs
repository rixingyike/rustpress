//! 插件系统模块
//!
//! 使用 linkme 分布式切片实现编译时自动注册。
//! 插件只需在自己的模块中用 `#[linkme::distributed_slice(PLUGINS)]` 标注，
//! 即可被主程序自动发现，无需手动注册。

pub mod comments;

use crate::config::Config;
use crate::error::Result;
use axum::Router;
use tera::Context;

/// 插件描述符（静态，linkme 兼容）
pub struct PluginDescriptor {
    /// 插件名称
    pub name: &'static str,

    /// 文章渲染钩子：在模板渲染之前修改上下文
    pub on_post_render: Option<fn(&Config, &mut Context) -> Result<()>>,

    /// API 路由工厂：返回 (路径前缀, Router)
    pub api_routes: Option<fn(&Config) -> Option<(&'static str, Router)>>,
}

/// 全局插件注册表（编译时自动收集）
#[linkme::distributed_slice]
pub static PLUGINS: [PluginDescriptor];

/// 遍历所有插件，执行 on_post_render 钩子
pub fn run_post_render_hooks(config: &Config, context: &mut Context) -> Result<()> {
    for plugin in PLUGINS.iter() {
        if let Some(hook) = plugin.on_post_render {
            hook(config, context)?;
        }
    }
    Ok(())
}

/// 遍历所有插件，收集 API 路由并挂载到 Router
pub fn collect_api_routes(config: &Config) -> Router {
    let mut router = Router::new();
    for plugin in PLUGINS.iter() {
        if let Some(factory) = plugin.api_routes {
            if let Some((prefix, sub_router)) = factory(config) {
                println!("插件 [{}] API 已挂载: {}/*", plugin.name, prefix);
                router = router.nest(prefix, sub_router);
            }
        }
    }
    router
}
