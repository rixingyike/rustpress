//! 评论 API 路由
//!
//! 提供 GitHub OAuth 回调和评论代理接口

use axum::{
    Json, Router,
    extract::{Path, Query},
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
    pub github_client_secret: Option<String>,
    pub site_url: String,
}

/// 创建评论 API 路由
pub fn api_routes(config: CommentsConfig) -> Router {
    let config = Arc::new(config);
    let has_secret = config.github_client_secret.is_some();

    let mut router = Router::new()
        .route("/user", get(get_user))
        .route(
            "/reactions/{issue_number}",
            get({
                let cfg = Arc::clone(&config);
                move |path: Path<i64>| get_reactions(path, cfg)
            }),
        )
        .route(
            "/post",
            post({
                let cfg = Arc::clone(&config);
                move |headers, body| post_comment(headers, body, cfg)
            }),
        )
        .route(
            "/like",
            post({
                let cfg = Arc::clone(&config);
                move |headers, body| like_issue(headers, body, cfg)
            }),
        );

    // 只有配置了 client_secret 才注册 OAuth 相关回调路由
    if has_secret {
        router = router.route(
            "/callback",
            get({
                let cfg = Arc::clone(&config);
                move |query| oauth_callback(query, cfg)
            }),
        );
    }

    router
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
            "client_secret": config.github_client_secret.as_deref().unwrap_or_default(),
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
    #[serde(default)]
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
                "title": format!("{} | {}", title, body.pathname),
                "body": format!("Comment thread for {}\n\n{}{}", body.pathname, config.site_url.trim_end_matches('/'), body.pathname),
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

// ---- 点赞/取消点赞 ----

#[derive(Deserialize)]
pub struct LikeBody {
    pub issue_number: Option<i64>,
    pub title: Option<String>,
    pub pathname: Option<String>,
}

async fn like_issue(
    headers: HeaderMap,
    Json(body): Json<LikeBody>,
    config: Arc<CommentsConfig>,
) -> impl IntoResponse {
    let token = match extract_token_from_cookie(&headers) {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"success": false, "message": "请先登录"}))).into_response(),
    };

    let client = reqwest::Client::new();

    // If issue_number is missing/0 and title+pathname provided, create issue first
    let issue_number = if let Some(num) = body.issue_number {
        if num > 0 {
            num
        } else if let (Some(title), Some(pathname)) = (&body.title, &body.pathname) {
            let res = client
                .post(format!("https://api.github.com/repos/{}/issues", config.repo))
                .header("Authorization", format!("Bearer {}", token))
                .header("Accept", "application/vnd.github.v3+json")
                .header("User-Agent", "RustPress-Comments")
                .json(&serde_json::json!({
                    "title": format!("{} | {}", title, pathname),
                    "body": format!("Comment thread for {}\n\n{}{}", pathname, config.site_url.trim_end_matches('/'), pathname),
                }))
                .send()
                .await;

            match res {
                Ok(r) if r.status().is_success() => {
                    if let Ok(data) = r.json::<serde_json::Value>().await {
                        data["number"].as_i64().unwrap_or(0)
                    } else {
                        return (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"success": false, "message": "创建 Issue 失败: 解析响应失败"}))).into_response();
                    }
                }
                Ok(r) => {
                    return (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"success": false, "message": format!("创建 Issue 失败: HTTP {}", r.status())}))).into_response();
                }
                Err(e) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "message": format!("网络错误: {}", e)}))).into_response();
                }
            }
        } else {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"success": false, "message": "缺少参数: issue_number 或 (title + pathname)"}))).into_response();
        }
    } else if let (Some(title), Some(pathname)) = (body.title, body.pathname) {
        let res = client
            .post(format!("https://api.github.com/repos/{}/issues", config.repo))
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "RustPress-Comments")
            .json(&serde_json::json!({
                "title": format!("{} | {}", title, pathname),
                "body": format!("Comment thread for {}\n\n{}{}", pathname, config.site_url.trim_end_matches('/'), pathname),
            }))
            .send()
            .await;

        match res {
            Ok(r) if r.status().is_success() => {
                if let Ok(data) = r.json::<serde_json::Value>().await {
                    data["number"].as_i64().unwrap_or(0)
                } else {
                    return (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"success": false, "message": "创建 Issue 失败: 解析响应失败"}))).into_response();
                }
            }
            Ok(r) => {
                return (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"success": false, "message": format!("创建 Issue 失败: HTTP {}", r.status())}))).into_response();
            }
            Err(e) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "message": format!("网络错误: {}", e)}))).into_response();
            }
        }
    } else {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"success": false, "message": "缺少参数: issue_number 或 (title + pathname)"}))).into_response();
    };

    if issue_number == 0 {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "message": "创建 Issue 失败: 编号为 0"}))).into_response();
    }

    // React with +1
    let res = client
        .post(format!(
            "https://api.github.com/repos/{}/issues/{}/reactions",
            config.repo, issue_number
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github.squirrel-girl-preview+json")
        .header("User-Agent", "RustPress-Comments")
        .json(&serde_json::json!({"content": "+1"}))
        .send()
        .await;

    match res {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                Json(serde_json::json!({"success": true, "issue_number": issue_number}))
            } else {
                Json(serde_json::json!({"success": false, "message": format!("HTTP {}", status)}))
            }
        }
        Err(e) => Json(serde_json::json!({"success": false, "message": format!("网络错误: {}", e)})),
    }
    .into_response()
}

async fn get_reactions(
    Path(issue_number): Path<i64>,
    config: Arc<CommentsConfig>,
) -> impl IntoResponse {
    let client = reqwest::Client::new();
    let res = client
        .get(format!(
            "https://api.github.com/repos/{}/issues/{}/reactions?per_page=100",
            config.repo, issue_number
        ))
        .header("Accept", "application/vnd.github.squirrel-girl-preview+json")
        .header("User-Agent", "RustPress-Comments")
        .send()
        .await;

    match res {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            (status, body).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{{\"error\":\"{}\"}}", e),
        )
            .into_response(),
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
