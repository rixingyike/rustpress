use rustpress::config::Config;
use rustpress::post::PostParser;
use rustpress::template::TemplateEngine;
use std::fs;
use std::path::Path;

#[test]
fn test_config_dev_and_test_domains() {
    // 1. 测试正式环境域名 -> 不包含草稿
    let prod_toml = r#"
    [site]
    name = "Test Site"
    description = "Test Desc"
    domain = "https://yishulun.com"
    base_url = "https://yishulun.com"
    "#;
    let config = Config { data: toml::from_str(prod_toml).unwrap() };
    assert!(!config.is_dev_or_test_domain());

    // 2. 测试测试环境域名 dev.yishulun.com -> 包含草稿
    let dev_toml = r#"
    [site]
    name = "Test Site"
    description = "Test Desc"
    domain = "https://dev.yishulun.com"
    base_url = "https://dev.yishulun.com"
    "#;
    let config = Config { data: toml::from_str(dev_toml).unwrap() };
    assert!(config.is_dev_or_test_domain());

    // 3. 测试本地 localhost -> 包含草稿
    let local_toml = r#"
    [site]
    name = "Test Site"
    description = "Test Desc"
    domain = "https://yishulun.com"
    base_url = "http://localhost:1111"
    "#;
    let config = Config { data: toml::from_str(local_toml).unwrap() };
    assert!(config.is_dev_or_test_domain());

    // 4. 测试 127.0.0.1 -> 包含草稿
    let ip_toml = r#"
    [site]
    name = "Test Site"
    description = "Test Desc"
    domain = "http://127.0.0.1:8080"
    "#;
    let config = Config { data: toml::from_str(ip_toml).unwrap() };
    assert!(config.is_dev_or_test_domain());
}

#[test]
fn test_list_posts_with_draft_options() -> Result<(), Box<dyn std::error::Error>> {
    let test_dir = std::env::temp_dir().join(format!("rustpress_test_draft_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos()));
    fs::create_dir_all(&test_dir)?;
    let content_dir = &test_dir;

    // 写入常规公开文章
    let post1_path = content_dir.join("post1.md");
    fs::write(&post1_path, "---\ntitle: \"公开文章\"\ndate: 2026-08-01\n---\n公开内容")?;

    // 写入草稿文章
    let draft_path = content_dir.join("draft_post.md");
    fs::write(&draft_path, "---\ntitle: \"草稿文章\"\ndraft: true\ndate: 2026-08-02\n---\n草稿内容")?;

    // 写入位于 draft 目录下的文章
    let doc_dir = content_dir.join("docs").join("draft_book");
    fs::create_dir_all(&doc_dir)?;
    fs::write(doc_dir.join("README.md"), "---\ntitle: \"草稿书籍\"\nlayout: doc\ndraft: true\n---\n书籍介绍")?;
    fs::write(doc_dir.join("chapter1.md"), "---\ntitle: \"第一章\"\n---\n章节内容")?;

    // 1. 默认/生产模式（include_drafts = false）
    let prod_posts = PostParser::list_posts_with_options(content_dir, false)?;
    assert_eq!(prod_posts.len(), 1);
    assert_eq!(prod_posts[0].title(), Some("公开文章"));

    // 2. 开发/测试模式（include_drafts = true）
    let dev_posts = PostParser::list_posts_with_options(content_dir, true)?;
    assert!(dev_posts.iter().any(|p| p.title() == Some("公开文章") && !p.is_draft()));
    assert!(dev_posts.iter().any(|p| p.title() == Some("草稿文章") && p.is_draft()));
    assert!(dev_posts.iter().any(|p| p.title() == Some("第一章")));

    let _ = fs::remove_dir_all(&test_dir);
    Ok(())
}

#[test]
fn test_template_renders_draft_badge() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let source_dir = Path::new(&workspace_root).join("source");
    let config_path = source_dir.join("config.toml");

    let config = Config::from_file(&config_path)?;
    let engine = TemplateEngine::new(config, &source_dir)?;

    let draft_post_path = source_dir.join("test_draft_badge_dummy.md");
    let draft_md = "---\ntitle: \"测试草稿文章\"\ndraft: true\ndate: 2026-08-01\n---\n草稿正文";
    fs::write(&draft_post_path, draft_md)?;

    let dev_posts = PostParser::list_posts_with_options(&source_dir, true)?;
    let post_obj = dev_posts.iter().find(|p| p.title() == Some("测试草稿文章")).expect("Found test draft post");
    assert!(post_obj.is_draft());

    let html = engine.render_post(post_obj, &dev_posts)?;
    let _ = fs::remove_file(&draft_post_path);

    assert!(html.contains("草稿"), "Rendered post should contain draft badge");

    Ok(())
}
