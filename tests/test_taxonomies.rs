use rustpress::config::{Config, TaxonomiesConfig};
use rustpress::PostParser;

#[test]
fn test_taxonomies_config_deserialization() {
    let toml_content = r#"
    [taxonomies]
    tweets = "/t"
    columns = "/c"
    categories = "/cat"
    tags = "/tag"
    projects = "/p"
    works = "/w"
    archives = "/a"
    friends = "/f"
    about = "/about.html"
    "#;

    let config = Config::from_toml_str(toml_content).expect("Failed to parse config");
    let tax = config.taxonomies_config();

    assert_eq!(tax.tweets, "/t");
    assert_eq!(tax.columns, "/c");
    assert_eq!(tax.categories, "/cat");
    assert_eq!(tax.tags, "/tag");
    assert_eq!(tax.projects, "/p");
    assert_eq!(tax.works, "/w");
    assert_eq!(tax.archives, "/a");
    assert_eq!(tax.friends, "/f");
    assert_eq!(tax.about, "/about.html");

    assert_eq!(tax.get_dir("columns"), "c");
    assert_eq!(tax.get_prefix("columns"), "/c");
    assert_eq!(tax.get_dir("tags"), "tag");
    assert_eq!(tax.get_prefix("tags"), "/tag");
}

#[test]
fn test_column_routing_with_c_prefix() {
    let mut tax = TaxonomiesConfig::default();
    tax.columns = "/c".to_string();

    // Column overview
    let url = PostParser::compute_post_url(&["columns".to_string()], "README", &tax);
    assert_eq!(url, "/c/index.html");

    // Column single index
    let url = PostParser::compute_post_url(&["columns".to_string(), "harness".to_string()], "README", &tax);
    assert_eq!(url, "/c/harness/index.html");

    // Column chapter post with slug
    let url = PostParser::compute_post_url(&["columns".to_string(), "harness".to_string()], "1.认识GPT", &tax);
    assert_eq!(url, "/c/harness/1.认识GPT.html");
}

#[test]
fn test_column_routing_with_root_prefix() {
    let mut tax = TaxonomiesConfig::default();
    tax.columns = "/".to_string();

    // Column overview MUST NOT collide with site homepage /index.html
    let url = PostParser::compute_post_url(&["columns".to_string()], "README", &tax);
    assert_eq!(url, "/columns/index.html");

    // Column single index becomes 2nd-level route on domain
    let url = PostParser::compute_post_url(&["columns".to_string(), "harness".to_string()], "README", &tax);
    assert_eq!(url, "/harness/index.html");

    // Column chapter post
    let url = PostParser::compute_post_url(&["columns".to_string(), "harness".to_string()], "1.认识GPT", &tax);
    assert_eq!(url, "/harness/1.认识GPT.html");
}

#[test]
fn test_other_taxonomies_routing() {
    let mut tax = TaxonomiesConfig::default();
    tax.projects = "/p".to_string();
    tax.works = "/w".to_string();
    tax.friends = "/f".to_string();
    tax.tweets = "/t".to_string();
    tax.categories = "/cat".to_string();

    // Projects
    let url = PostParser::compute_post_url(&["projects".to_string()], "enyan", &tax);
    assert_eq!(url, "/p/enyan.html");
    let url = PostParser::compute_post_url(&["projects".to_string(), "rustpress".to_string()], "README", &tax);
    assert_eq!(url, "/p/rustpress/index.html");

    // Works
    let url = PostParser::compute_post_url(&["works".to_string()], "miniprogram-0-1", &tax);
    assert_eq!(url, "/w/miniprogram-0-1.html");

    // Friends
    let url = PostParser::compute_post_url(&["friends".to_string()], "1", &tax);
    assert_eq!(url, "/f/1.html");

    // Tweets
    let url = PostParser::compute_post_url(&["tweets".to_string()], "123", &tax);
    assert_eq!(url, "/t/123.html");

    // Ordinary blog posts
    let url = PostParser::compute_post_url(&["tech".to_string(), "rust".to_string()], "hello", &tax);
    assert_eq!(url, "/tech/rust/hello.html");
}

#[test]
fn test_taxonomy_assets_copying() {
    use std::fs;
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let base = std::env::temp_dir().join(format!("rustpress_test_assets_{}", ts));
    let src_dir = base.join("src");
    let out_dir = base.join("public");

    let col_assets = src_dir.join("columns/rustpress/assets");
    fs::create_dir_all(&col_assets).unwrap();
    fs::write(col_assets.join("screenshot.png"), b"fake_png_data").unwrap();

    let tweet_assets = src_dir.join("tweets/2026/08/assets");
    fs::create_dir_all(&tweet_assets).unwrap();
    fs::write(tweet_assets.join("photo.jpg"), b"fake_jpg_data").unwrap();

    let mut tax = TaxonomiesConfig::default();
    tax.columns = "/c".to_string();
    tax.tweets = "/t".to_string();

    rustpress::utils::copy_non_md_recursive_preserve_paths(&src_dir, &out_dir, Some(&tax)).unwrap();

    // 1. Columns assets: both /columns/rustpress/assets/ and /c/rustpress/assets/ exist
    assert!(out_dir.join("columns/rustpress/assets/screenshot.png").exists());
    assert!(out_dir.join("c/rustpress/assets/screenshot.png").exists());

    // 2. Tweets assets: both /tweets/2026/08/assets/ and /t/2026/08/assets/ exist
    assert!(out_dir.join("tweets/2026/08/assets/photo.jpg").exists());
    assert!(out_dir.join("t/2026/08/assets/photo.jpg").exists());

    let _ = fs::remove_dir_all(base);
}

#[test]
fn test_tweet_and_post_image_paths_normalization() {
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let src_dir = std::env::temp_dir().join(format!("rustpress_test_norm_{}", ts));
    let tweet_file = src_dir.join("tweets/2026/08/20260828143000.md");
    std::fs::create_dir_all(tweet_file.parent().unwrap()).unwrap();

    let tweet_content = r#"---
date: "2026-08-28 14:30:00"
layout: tweet
images:
  - "/tweets/2026/08/assets/abs.jpg"
  - "assets/rel.jpg"
  - "./assets/dot_rel.jpg"
  - "https://images.unsplash.com/photo-1.jpg"
  - "http://example.com/photo-2.jpg"
  - "//cdn.example.com/photo-3.jpg"
---

测试闲言
"#;

    let mut tax = TaxonomiesConfig::default();
    tax.tweets = "/t".to_string();

    let post_val = PostParser::parse_post_with_taxonomies(tweet_content, &tweet_file, &src_dir, &tax)
        .unwrap()
        .expect("Post should be parsed");

    let images = post_val.get("images").and_then(|v| v.as_array()).expect("images array");
    let image_strings: Vec<&str> = images.iter().filter_map(|v| v.as_str()).collect();

    assert_eq!(
        image_strings,
        vec![
            "/tweets/2026/08/assets/abs.jpg",
            "/tweets/2026/08/assets/rel.jpg",
            "/tweets/2026/08/assets/dot_rel.jpg",
            "https://images.unsplash.com/photo-1.jpg",
            "http://example.com/photo-2.jpg",
            "//cdn.example.com/photo-3.jpg",
        ]
    );

    let _ = std::fs::remove_dir_all(src_dir);
}

