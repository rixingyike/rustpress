//! 评论 API 路由
//!
//! 提供 GitHub OAuth 回调和评论代理接口

use axum::{
    Json, Router,
    extract::Query,
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 评论 API 所需的配置
#[derive(Clone)]
pub struct CommentsConfig {
    pub repo: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub site_url: String,
}

/// 创建评论 API 路由
pub fn api_routes(config: CommentsConfig) -> Router {
    let config = Arc::new(config);

    Router::new()
        .route(
            "/callback",
            get({
                let cfg = Arc::clone(&config);
                move |query| oauth_callback(query, cfg)
            }),
        )
        .route(
            "/post",
            post({
                let cfg = Arc::clone(&config);
                move |headers, body| post_comment(headers, body, cfg)
            }),
        )
        .route("/user", get(get_user))
}

// ---- OAuth 回调 ----

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
    #[serde(default)]
    state: String,
}

async fn oauth_callback(
    Query(query): Query<CallbackQuery>,
    config: Arc<CommentsConfig>,
) -> impl IntoResponse {
    // 用 code 换 access_token
    let client = reqwest::Client::new();
    let res = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .json(&serde_json::json!({
            "client_id": config.github_client_id,
            "client_secret": config.github_client_secret,
            "code": query.code,
        }))
        .send()
        .await;

    match res {
        Ok(resp) => {
            if let Ok(token_data) = resp.json::<TokenResponse>().await {
                if let Some(token) = token_data.access_token {
                    // 返回一个页面，将 token 通过 postMessage 传回父窗口
                    let redirect_path = if query.state.is_empty() {
                        "/".to_string()
                    } else {
                        query.state.clone()
                    };
                    let html = format!(
                        r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>登录成功</title></head>
<body>
<script>
  document.cookie = "gh_token={}; path=/; max-age=2592000; SameSite=Lax";
  window.location.href = "{}";
</script>
<p>登录成功，正在跳转...</p>
</body></html>"#,
                        token, redirect_path
                    );
                    return Html(html).into_response();
                }
            }
            Html("<p>授权失败，请重试。</p>".to_string()).into_response()
        }
        Err(_) => Html("<p>网络错误，请重试。</p>".to_string()).into_response(),
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    #[allow(dead_code)]
    token_type: Option<String>,
}

// ---- 发表评论 ----

#[derive(Deserialize)]
pub struct PostCommentBody {
    pub issue_number: Option<i64>,
    pub title: Option<String>,
    pub body: String,
    pub pathname: String,
}

#[derive(Serialize)]
pub struct PostCommentResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_number: Option<i64>,
}

async fn post_comment(
    headers: HeaderMap,
    Json(body): Json<PostCommentBody>,
    config: Arc<CommentsConfig>,
) -> impl IntoResponse {
    // 从 cookie 中提取 token
    let token = extract_token_from_cookie(&headers);
    let token = match token {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(PostCommentResponse {
                    success: false,
                    message: "请先登录 GitHub".to_string(),
                    comment_url: None,
                    issue_number: None,
                }),
            );
        }
    };

    let client = reqwest::Client::new();

    // 如果没有 issue_number，先创建 Issue
    let issue_number = if let Some(num) = body.issue_number {
        num
    } else {
        // 创建新 Issue
        let title = body
            .title
            .clone()
            .unwrap_or_else(|| format!("评论: {}", body.pathname));
        let create_res = client
            .post(format!(
                "https://api.github.com/repos/{}/issues",
                config.repo
            ))
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "RustPress-Comments")
            .json(&serde_json::json!({
                "title": title,
                "body": format!("评论页面: {}", body.pathname),
                "labels": ["comment"]
            }))
            .send()
            .await;

        match create_res {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(issue) = resp.json::<serde_json::Value>().await {
                    issue["number"].as_i64().unwrap_or(0)
                } else {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(PostCommentResponse {
                            success: false,
                            message: "创建 Issue 失败".to_string(),
                            comment_url: None,
                            issue_number: None,
                        }),
                    );
                }
            }
            _ => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(PostCommentResponse {
                        success: false,
                        message: "创建 Issue 失败".to_string(),
                        comment_url: None,
                        issue_number: None,
                    }),
                );
            }
        }
    };

    // 在 Issue 下发评论
    let comment_res = client
        .post(format!(
            "https://api.github.com/repos/{}/issues/{}/comments",
            config.repo, issue_number
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "RustPress-Comments")
        .json(&serde_json::json!({
            "body": body.body,
        }))
        .send()
        .await;

    match comment_res {
        Ok(resp) if resp.status().is_success() => {
            let url = resp
                .json::<serde_json::Value>()
                .await
                .ok()
                .and_then(|v| v["html_url"].as_str().map(|s| s.to_string()));
            (
                StatusCode::OK,
                Json(PostCommentResponse {
                    success: true,
                    message: "评论发表成功".to_string(),
                    comment_url: url,
                    issue_number: Some(issue_number),
                }),
            )
        }
        Ok(resp) => {
            let status = resp.status();
            let err_body = resp.text().await.unwrap_or_default();
            (
                StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(PostCommentResponse {
                    success: false,
                    message: format!("发评论失败: {}", err_body),
                    comment_url: None,
                    issue_number: None,
                }),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(PostCommentResponse {
                success: false,
                message: format!("网络错误: {}", e),
                comment_url: None,
                issue_number: None,
            }),
        ),
    }
}

// ---- 获取当前用户信息 ----

async fn get_user(headers: HeaderMap) -> impl IntoResponse {
    let token = extract_token_from_cookie(&headers);
    let token = match token {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"logged_in": false})),
            );
        }
    };

    let client = reqwest::Client::new();
    let res = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "RustPress-Comments")
        .send()
        .await;

    match res {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(user) = resp.json::<serde_json::Value>().await {
                (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "logged_in": true,
                        "login": user["login"],
                        "avatar_url": user["avatar_url"],
                        "name": user["name"],
                    })),
                )
            } else {
                (
                    StatusCode::OK,
                    Json(serde_json::json!({"logged_in": false})),
                )
            }
        }
        _ => (
            StatusCode::OK,
            Json(serde_json::json!({"logged_in": false})),
        ),
    }
}

// ---- 工具函数 ----

fn extract_token_from_cookie(headers: &HeaderMap) -> Option<String> {
    headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|c| {
                let c = c.trim();
                if c.starts_with("gh_token=") {
                    Some(c["gh_token=".len()..].to_string())
                } else {
                    None
                }
            })
        })
}
