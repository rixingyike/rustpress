//! 模板引擎模块
//! 
//! 负责模板的加载和渲染

use crate::config::Config;
use crate::error::{Error, Result};
use crate::post::Post;
use chrono::prelude::*;
use serde_json::Value;
use tera::{Context, Tera};

/// 模板引擎
pub struct TemplateEngine {
    tera: Tera,
    config: Config,
}

impl TemplateEngine {
    /// 创建新的模板引擎
    pub fn new(config: Config) -> Result<Self> {
        let tera = Tera::new("src/themes/default/templates/**/*")?;
        
        Ok(TemplateEngine { tera, config })
    }
    
    /// 创建基础上下文
    fn create_base_context(&self) -> Context {
        use serde_json::{json, Value as JsonValue};
        
        let mut context = Context::new();
        
        // 创建合并的站点配置
        let mut site_config = serde_json::Map::new();
        
        // 插入基本的站点配置
        if let toml::Value::Table(site_table) = &self.config.site() {
            for (key, value) in site_table {
                // 将toml::Value转换为serde_json::Value
                let json_value = match value {
                    toml::Value::String(s) => JsonValue::String(s.clone()),
                    toml::Value::Integer(i) => JsonValue::Number((*i).into()),
                    toml::Value::Float(f) => JsonValue::Number(serde_json::Number::from_f64(*f).unwrap_or_else(|| 0.into())),
                    toml::Value::Boolean(b) => JsonValue::Bool(*b),
                    toml::Value::Array(arr) => {
                        let json_arr: Vec<JsonValue> = arr.iter().map(|v| {
                            match v {
                                toml::Value::String(s) => JsonValue::String(s.clone()),
                                toml::Value::Integer(i) => JsonValue::Number((*i).into()),
                                toml::Value::Float(f) => JsonValue::Number(serde_json::Number::from_f64(*f).unwrap_or_else(|| 0.into())),
                                toml::Value::Boolean(b) => JsonValue::Bool(*b),
                                _ => JsonValue::Null,
                            }
                        }).collect();
                        JsonValue::Array(json_arr)
                    },
                    toml::Value::Table(table) => {
                        let mut json_map = serde_json::Map::new();
                        for (k, v) in table {
                            let json_v = match v {
                                toml::Value::String(s) => JsonValue::String(s.clone()),
                                toml::Value::Integer(i) => JsonValue::Number((*i).into()),
                                toml::Value::Float(f) => JsonValue::Number(serde_json::Number::from_f64(*f).unwrap_or_else(|| 0.into())),
                                toml::Value::Boolean(b) => JsonValue::Bool(*b),
                                _ => JsonValue::Null,
                            };
                            json_map.insert(k.clone(), json_v);
                        }
                        JsonValue::Object(json_map)
                    },
                    _ => JsonValue::Null,
                };
                site_config.insert(key.clone(), json_value);
            }
        }
        
        // 合并作者配置到site.author
        if let Some(author) = self.config.data.get("author") {
            if let toml::Value::Table(author_table) = author {
                let mut author_config = serde_json::Map::new();
                for (key, value) in author_table {
                    let json_value = match value {
                        toml::Value::String(s) => JsonValue::String(s.clone()),
                        toml::Value::Integer(i) => JsonValue::Number((*i).into()),
                        toml::Value::Float(f) => JsonValue::Number(serde_json::Number::from_f64(*f).unwrap_or_else(|| 0.into())),
                        toml::Value::Boolean(b) => JsonValue::Bool(*b),
                        _ => JsonValue::Null,
                    };
                    author_config.insert(key.clone(), json_value);
                }
                site_config.insert("author".to_string(), JsonValue::Object(author_config));
            }
        }
        
        // 合并社交链接配置到site.social
        if let Some(social) = self.config.data.get("social") {
            if let toml::Value::Table(social_table) = social {
                let mut social_config = serde_json::Map::new();
                for (key, value) in social_table {
                    let json_value = match value {
                        toml::Value::String(s) => JsonValue::String(s.clone()),
                        toml::Value::Integer(i) => JsonValue::Number((*i).into()),
                        toml::Value::Float(f) => JsonValue::Number(serde_json::Number::from_f64(*f).unwrap_or_else(|| 0.into())),
                        toml::Value::Boolean(b) => JsonValue::Bool(*b),
                        _ => JsonValue::Null,
                    };
                    social_config.insert(key.clone(), json_value);
                }
                site_config.insert("social".to_string(), JsonValue::Object(social_config));
            }
        }
        
        // 合并首页配置到site.homepage
        if let Some(homepage) = self.config.data.get("homepage") {
            if let toml::Value::Table(homepage_table) = homepage {
                let mut homepage_config = serde_json::Map::new();
                for (key, value) in homepage_table {
                    let json_value = match value {
                        toml::Value::String(s) => JsonValue::String(s.clone()),
                        toml::Value::Integer(i) => JsonValue::Number((*i).into()),
                        toml::Value::Float(f) => JsonValue::Number(serde_json::Number::from_f64(*f).unwrap_or_else(|| 0.into())),
                        toml::Value::Boolean(b) => JsonValue::Bool(*b),
                        _ => JsonValue::Null,
                    };
                    homepage_config.insert(key.clone(), json_value);
                }
                site_config.insert("homepage".to_string(), JsonValue::Object(homepage_config));
            }
        }
        
        // 合并广告配置到site.ads
        if let Some(ads) = self.config.data.get("ads") {
            if let toml::Value::Table(ads_table) = ads {
                let mut ads_config = serde_json::Map::new();
                for (key, value) in ads_table {
                    let json_value = match value {
                        toml::Value::String(s) => JsonValue::String(s.clone()),
                        toml::Value::Integer(i) => JsonValue::Number((*i).into()),
                        toml::Value::Float(f) => JsonValue::Number(serde_json::Number::from_f64(*f).unwrap_or_else(|| 0.into())),
                        toml::Value::Boolean(b) => JsonValue::Bool(*b),
                        _ => JsonValue::Null,
                    };
                    ads_config.insert(key.clone(), json_value);
                }
                site_config.insert("ads".to_string(), JsonValue::Object(ads_config));
            }
        }
        
        // 插入合并后的站点配置
        context.insert("site", &JsonValue::Object(site_config));
        
        // 插入当前时间
        let now = Utc::now();
        context.insert("now", &now);
        
        context
    }
    
    /// 渲染首页
    pub fn render_index(&self, posts: &[Post], all_tags: &[Value], all_categories: &Value) -> Result<String> {
        let mut context = self.create_base_context();
        
        // 转换 posts 为 JSON 值
        let posts_json: Vec<Value> = posts.iter().map(|p| p.data.clone()).collect();
        context.insert("posts", &posts_json);
        context.insert("all_tags", all_tags);
        context.insert("all_categories", all_categories);
        
        self.tera.render("index.html", &context)
            .map_err(Error::Template)
    }
    
    /// 渲染文章页面
    pub fn render_post(&self, post: &Post) -> Result<String> {
        let mut context = self.create_base_context();
        context.insert("page", &post.data);
        
        self.tera.render("post.html", &context)
            .map_err(Error::Template)
    }
    
    /// 渲染标签页面
    pub fn render_tags(&self, posts: &[Post], all_tags: &[Value]) -> Result<String> {
        let mut context = self.create_base_context();
        
        let posts_json: Vec<Value> = posts.iter().map(|p| p.data.clone()).collect();
        context.insert("posts", &posts_json);
        context.insert("all_tags", all_tags);
        
        self.tera.render("tags.html", &context)
            .map_err(Error::Template)
    }
    
    /// 渲染分类页面
    pub fn render_categories(&self, posts: &[Post], all_categories: &Value) -> Result<String> {
        let mut context = self.create_base_context();
        
        let posts_json: Vec<Value> = posts.iter().map(|p| p.data.clone()).collect();
        context.insert("posts", &posts_json);
        context.insert("all_categories", all_categories);
        
        self.tera.render("categories.html", &context)
            .map_err(Error::Template)
    }
    
    /// 渲染归档页面
    pub fn render_archives(&self, posts: &[Post], all_years: &[Value]) -> Result<String> {
        let mut context = self.create_base_context();
        
        let posts_json: Vec<Value> = posts.iter().map(|p| p.data.clone()).collect();
        context.insert("posts", &posts_json);
        context.insert("all_years", all_years);
        
        self.tera.render("archives.html", &context)
            .map_err(Error::Template)
    }
    
    /// 渲染关于页面
    pub fn render_about(&self, posts: &[Post], all_tags: &[Value], all_categories: &Value) -> Result<String> {
        let mut context = self.create_base_context();
        
        let posts_json: Vec<Value> = posts.iter().map(|p| p.data.clone()).collect();
        context.insert("posts", &posts_json);
        context.insert("all_tags", all_tags);
        context.insert("all_categories", all_categories);
        
        self.tera.render("about.html", &context)
            .map_err(Error::Template)
    }
    
    /// 渲染友链页面
    pub fn render_friends(&self) -> Result<String> {
        let context = self.create_base_context();
        
        self.tera.render("friends.html", &context)
            .map_err(Error::Template)
    }
    
    /// 渲染搜索页面
    pub fn render_search(&self, posts: &[Post], all_tags: &[Value]) -> Result<String> {
        let mut context = self.create_base_context();
        
        let posts_json: Vec<Value> = posts.iter().map(|p| p.data.clone()).collect();
        context.insert("posts", &posts_json);
        context.insert("all_tags", all_tags);
        
        self.tera.render("search.html", &context)
            .map_err(Error::Template)
    }
    
    /// 渲染404页面
    pub fn render_404(&self, posts: &[Post], all_tags: &[Value], all_categories: &Value) -> Result<String> {
        let mut context = self.create_base_context();
        
        let posts_json: Vec<Value> = posts.iter().map(|p| p.data.clone()).collect();
        context.insert("posts", &posts_json);
        context.insert("all_tags", all_tags);
        context.insert("all_categories", all_categories);
        
        self.tera.render("404.html", &context)
            .map_err(Error::Template)
    }
    
    /// 渲染分类页面
    pub fn render_category(&self, posts: &[&Post], category_name: &str) -> Result<String> {
        let mut context = self.create_base_context();
        
        // 转换 posts 为 JSON 值
        let posts_json: Vec<Value> = posts.iter().map(|p| p.data.clone()).collect();
        context.insert("posts", &posts_json);
        context.insert("category_name", category_name);
        
        self.tera.render("category.html", &context)
            .map_err(Error::Template)
    }
}