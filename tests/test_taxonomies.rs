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
