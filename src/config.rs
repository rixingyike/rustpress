//! 配置管理模块
//!
//! 处理配置文件的读取和解析

use crate::error::{Error, Result};
use std::path::Path;

/// 站点配置
#[derive(Debug, Clone)]
pub struct Config {
    /// 原始配置数据
    pub data: toml::Value,
}

impl Config {
    /// 从文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        // 直接读取传入的配置路径（项目根 config.toml）
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("无法读取配置文件 {:?}: {}", path, e)))?;

        Self::from_toml_str(&content)
    }

    /// 从 TOML 字符串加载配置
    pub fn from_toml_str(content: &str) -> Result<Self> {
        let data: toml::Value = toml::from_str(content)
            .map_err(|e| Error::Config(format!("配置文件格式错误: {}", e)))?;

        Ok(Config { data })
    }

    /// 获取站点配置
    pub fn site(&self) -> toml::Value {
        self.data
            .get("site")
            .cloned()
            .unwrap_or_else(|| toml::Value::Table(toml::map::Map::new()))
    }

    /// 获取分类法配置
    pub fn taxonomies(&self) -> Option<&toml::Value> {
        self.data.get("taxonomies")
    }

    /// 获取分类法配置结构体
    pub fn taxonomies_config(&self) -> TaxonomiesConfig {
        TaxonomiesConfig::from_config(self)
    }

    /// 获取主题配置
    pub fn theme(&self) -> Option<&toml::Value> {
        self.data.get("theme")
    }

    /// 获取主题名称（默认 "default"）
    pub fn theme_name(&self) -> String {
        self.theme()
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string()
    }

    /// 获取作者配置
    pub fn author(&self) -> Option<toml::Value> {
        self.site().get("author").cloned()
    }

    /// 获取社交链接配置
    pub fn social(&self) -> Option<toml::Value> {
        self.site().get("social").cloned()
    }

    /// 检查是否为开发或测试环境（如 https://dev.yishulun.com、localhost、127.0.0.1 或 config 中配置的 dev_domains 等）
    ///
    /// 对于测试域名或本地开发环境，需要忽略 draft: true 展示草稿并在首页显示短动态发布框。
    pub fn is_dev_or_test_domain(&self) -> bool {
        let mut custom_dev_domains: Vec<String> = Vec::new();
        let site_val = self.site();

        // 1. 从 [site.dev_domains] 或 [site.test_domains] 读取
        if let Some(domains_val) = site_val.get("dev_domains").or_else(|| site_val.get("test_domains")) {
            if let Some(arr) = domains_val.as_array() {
                for v in arr {
                    if let Some(s) = v.as_str() {
                        custom_dev_domains.push(s.trim().to_lowercase());
                    }
                }
            } else if let Some(s) = domains_val.as_str() {
                for part in s.split(',') {
                    custom_dev_domains.push(part.trim().to_lowercase());
                }
            }
        }

        // 2. 从 [dev.domains] 读取
        if let Some(dev_tab) = self.data.get("dev") {
            if let Some(domains_val) = dev_tab.get("domains").or_else(|| dev_tab.get("dev_domains")) {
                if let Some(arr) = domains_val.as_array() {
                    for v in arr {
                        if let Some(s) = v.as_str() {
                            custom_dev_domains.push(s.trim().to_lowercase());
                        }
                    }
                } else if let Some(s) = domains_val.as_str() {
                    for part in s.split(',') {
                        custom_dev_domains.push(part.trim().to_lowercase());
                    }
                }
            }
        }

        let is_match = |s: &str| -> bool {
            let s_lower = s.to_lowercase();
            // 自定义配置域名匹配
            for d in &custom_dev_domains {
                if !d.is_empty() && s_lower.contains(d) {
                    return true;
                }
            }
            // 常见默认测试域名与特征匹配
            s_lower.contains("dev.yishulun.com")
                || s_lower.contains("localhost")
                || s_lower.contains("127.0.0.1")
                || s_lower.contains("0.0.0.0")
                || s_lower.contains("::1")
                || s_lower.contains("dev.")
                || s_lower.contains("test.")
                || s_lower.contains("staging.")
        };

        if let Some(domain) = self.site().get("domain").and_then(|v| v.as_str()) {
            if is_match(domain) {
                return true;
            }
        }
        if let Some(base_url) = self.site().get("base_url").and_then(|v| v.as_str()) {
            if is_match(base_url) {
                return true;
            }
        }
        if let Ok(val) = std::env::var("RUSTPRESS_INCLUDE_DRAFTS") {
            if val == "1" || val.eq_ignore_ascii_case("true") {
                return true;
            }
        }
        if let Ok(val) = std::env::var("dev").or_else(|_| std::env::var("env")).or_else(|_| std::env::var("ENV")) {
            if val == "pushpen" || val == "1" || val.eq_ignore_ascii_case("true") {
                return true;
            }
        }
        if let Ok(val) = std::env::var("DEV_DOMAINS") {
            for part in val.split(',') {
                if is_match(part.trim()) {
                    return true;
                }
            }
        }
        false
    }
}

/// 分类法路径配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaxonomiesConfig {
    pub tweets: String,
    pub columns: String,
    pub categories: String,
    pub tags: String,
    pub projects: String,
    pub works: String,
    pub archives: String,
    pub friends: String,
    pub about: String,
}

impl Default for TaxonomiesConfig {
    fn default() -> Self {
        Self {
            tweets: "/t".to_string(),
            columns: "/c".to_string(),
            categories: "/cat".to_string(),
            tags: "/tag".to_string(),
            projects: "/p".to_string(),
            works: "/w".to_string(),
            archives: "/a".to_string(),
            friends: "/f".to_string(),
            about: "/about.html".to_string(),
        }
    }
}

impl TaxonomiesConfig {
    pub fn from_config(config: &Config) -> Self {
        let mut t = Self::default();
        if let Some(val) = config.taxonomies() {
            if let Some(table) = val.as_table() {
                if let Some(v) = table.get("tweets").or_else(|| table.get("tweet")).and_then(|v| v.as_str()) {
                    t.tweets = v.to_string();
                }
                if let Some(v) = table.get("columns").or_else(|| table.get("column")).and_then(|v| v.as_str()) {
                    t.columns = v.to_string();
                }
                if let Some(v) = table.get("categories").or_else(|| table.get("category")).and_then(|v| v.as_str()) {
                    t.categories = v.to_string();
                }
                if let Some(v) = table.get("tags").or_else(|| table.get("tag")).and_then(|v| v.as_str()) {
                    t.tags = v.to_string();
                }
                if let Some(v) = table.get("projects").or_else(|| table.get("project")).and_then(|v| v.as_str()) {
                    t.projects = v.to_string();
                }
                if let Some(v) = table.get("works").or_else(|| table.get("work")).and_then(|v| v.as_str()) {
                    t.works = v.to_string();
                }
                if let Some(v) = table.get("archives").or_else(|| table.get("archive")).and_then(|v| v.as_str()) {
                    t.archives = v.to_string();
                }
                if let Some(v) = table.get("friends").or_else(|| table.get("friend")).and_then(|v| v.as_str()) {
                    t.friends = v.to_string();
                }
                if let Some(v) = table.get("about").and_then(|v| v.as_str()) {
                    t.about = v.to_string();
                }
            }
        }
        t
    }

    /// 获取规范化的 URL 前缀（例如 "/c", "/t", ""），根目录 "/" 或空字符串返回 ""
    pub fn get_prefix(&self, taxonomy_name: &str) -> String {
        let raw = match taxonomy_name {
            "tweets" | "tweet" | "short" => &self.tweets,
            "columns" | "column" => &self.columns,
            "categories" | "category" => &self.categories,
            "tags" | "tag" => &self.tags,
            "projects" | "project" => &self.projects,
            "works" | "work" => &self.works,
            "archives" | "archive" => &self.archives,
            "friends" | "friend" => &self.friends,
            "about" => &self.about,
            _ => return format!("/{}", taxonomy_name.trim_matches('/')),
        };
        let trimmed = raw.trim().trim_matches('/');
        if trimmed.is_empty() {
            String::new()
        } else {
            format!("/{}", trimmed)
        }
    }

    /// 获取目录名（例如 "c", "t", "columns"），用于文件系统输出目录
    pub fn get_dir(&self, taxonomy_name: &str) -> String {
        let prefix = self.get_prefix(taxonomy_name);
        let trimmed = prefix.trim_matches('/').to_string();
        if trimmed.is_empty() {
            // 如果映射到了根目录 "/"，对聚合页目录回退到自身名称，避免与根目录冲突
            taxonomy_name.to_string()
        } else {
            trimmed
        }
    }

    /// 转换为 serde_json::Map
    pub fn to_json_map(&self) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        map.insert("tweets".to_string(), serde_json::Value::String(self.tweets.clone()));
        map.insert("columns".to_string(), serde_json::Value::String(self.columns.clone()));
        map.insert("categories".to_string(), serde_json::Value::String(self.categories.clone()));
        map.insert("tags".to_string(), serde_json::Value::String(self.tags.clone()));
        map.insert("projects".to_string(), serde_json::Value::String(self.projects.clone()));
        map.insert("works".to_string(), serde_json::Value::String(self.works.clone()));
        map.insert("archives".to_string(), serde_json::Value::String(self.archives.clone()));
        map.insert("friends".to_string(), serde_json::Value::String(self.friends.clone()));
        map.insert("about".to_string(), serde_json::Value::String(self.about.clone()));
        map
    }
}

