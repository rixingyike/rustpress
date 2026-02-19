//! 评论插件入口
//!
//! 集成 Giscus 评论系统
//! 通过 linkme 分布式切片自动注册到全局插件列表

use crate::config::Config;
use crate::error::Result;
use crate::plugins::PluginDescriptor;
use tera::Context;

// ---- 自动注册 ----

#[linkme::distributed_slice(crate::plugins::PLUGINS)]
static COMMENTS_PLUGIN: PluginDescriptor = PluginDescriptor {
    name: "Comments",
    on_post_render: Some(comments_on_post_render),
    api_routes: None, // Giscus 托管模式不需要后端 API
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
    let comments_config = match config.data.get("comments") {
        Some(c) => c,
        None => return Ok(()),
    };

    let repo = comments_config
        .get("repo")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let repo_id = comments_config
        .get("repo_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let category = comments_config
        .get("category")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let category_id = comments_config
        .get("category_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if repo.is_empty() || repo_id.is_empty() || category.is_empty() || category_id.is_empty() {
        // 缺少必要配置，无法加载 Giscus
        return Ok(());
    }

    // 可选配置
    let mapping = comments_config
        .get("mapping")
        .and_then(|v| v.as_str())
        .unwrap_or("pathname");
    let strict = comments_config
        .get("strict")
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .to_string(); // config可能读为int/bool
    let theme = comments_config
        .get("theme")
        .and_then(|v| v.as_str())
        .unwrap_or("preferred_color_scheme");
    let lang = comments_config
        .get("lang")
        .and_then(|v| v.as_str())
        .unwrap_or("zh-CN");
    let reactions_enabled = comments_config
        .get("reactions_enabled")
        .and_then(|v| v.as_str())
        .unwrap_or("1");
    let emit_metadata = comments_config
        .get("emit_metadata")
        .and_then(|v| v.as_str())
        .unwrap_or("0");
    let input_position = comments_config
        .get("input_position")
        .and_then(|v| v.as_str())
        .unwrap_or("bottom");

    // Giscus 脚本模板
    let script = format!(
        r#"
<script src="https://giscus.app/client.js"
        data-repo="{}"
        data-repo-id="{}"
        data-category="{}"
        data-category-id="{}"
        data-mapping="{}"
        data-strict="{}"
        data-reactions-enabled="{}"
        data-emit-metadata="{}"
        data-input-position="{}"
        data-theme="{}"
        data-lang="{}"
        crossorigin="anonymous"
        async>
</script>
"#,
        repo,
        repo_id,
        category,
        category_id,
        mapping,
        strict,
        reactions_enabled,
        emit_metadata,
        input_position,
        theme,
        lang
    );

    context.insert("comment_system_script", &script);

    Ok(())
}
