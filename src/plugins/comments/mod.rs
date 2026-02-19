//! 评论插件入口
//!
//! 自包含的 GitHub Issue 评论系统
//! 通过 linkme 分布式切片自动注册到全局插件列表

pub mod api;

use crate::config::Config;
use crate::error::Result;
use crate::plugins::PluginDescriptor;
use tera::Context;

/// 评论模板（HTML + CSS + JS）
const COMMENT_TEMPLATE: &str = include_str!("comments.html");

// ---- 自动注册 ----

#[linkme::distributed_slice(crate::plugins::PLUGINS)]
static COMMENTS_PLUGIN: PluginDescriptor = PluginDescriptor {
    name: "Comments",
    on_post_render: Some(comments_on_post_render),
    api_routes: Some(comments_api_routes),
};

// ---- on_post_render 钩子 ----

fn comments_on_post_render(config: &Config, context: &mut Context) -> Result<()> {
    // 检查全局开关
    let comments_enabled = config
        .data
        .get("comments")
        .and_then(|v| v.get("enabled"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !comments_enabled {
        return Ok(());
    }

    // 检查单篇文章开关
    if let Some(page) = context.get("page") {
        if let Some(comments) = page.get("comments") {
            if let Some(enabled) = comments.as_bool() {
                if !enabled {
                    return Ok(());
                }
            }
        }
    }

    // 读取配置
    let repo = config
        .data
        .get("comments")
        .and_then(|v| v.get("repo"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if repo.is_empty() {
        return Ok(());
    }

    // client_id 可选：没有则进入只读模式
    let client_id = config
        .data
        .get("comments")
        .and_then(|v| v.get("github_client_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // 模板替换
    let script = COMMENT_TEMPLATE
        .replace("{{REPO}}", repo)
        .replace("{{CLIENT_ID}}", client_id);

    context.insert("comment_system_script", &script);

    Ok(())
}

// ---- API 路由工厂 ----

fn comments_api_routes(config: &Config) -> Option<(&'static str, axum::Router)> {
    let comments = config.data.get("comments")?;
    let enabled = comments.get("enabled")?.as_bool()?;
    if !enabled {
        return None;
    }

    let repo = comments.get("repo")?.as_str()?.to_string();
    let client_id = comments.get("github_client_id")?.as_str()?.to_string();
    let client_secret = comments.get("github_client_secret")?.as_str()?.to_string();
    let site_url = config
        .data
        .get("site")
        .and_then(|v| v.get("base_url"))
        .and_then(|v| v.as_str())
        .unwrap_or("http://localhost:1111")
        .to_string();

    if repo.is_empty() || client_id.is_empty() || client_secret.is_empty() {
        return None;
    }

    let cfg = api::CommentsConfig {
        repo,
        github_client_id: client_id,
        github_client_secret: client_secret,
        site_url,
    };

    Some(("/api/comments", api::api_routes(cfg)))
}
