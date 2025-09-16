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
        let mut context = Context::new();
        
        // 插入站点配置
        context.insert("site", &self.config.site());
        
        // 插入当前时间
        let now = Utc::now();
        context.insert("now", &now);
        
        context
    }
    
    /// 渲染首页
    pub fn render_index(&self, posts: &[Post], all_tags: &[Value]) -> Result<String> {
        let mut context = self.create_base_context();
        
        // 转换 posts 为 JSON 值
        let posts_json: Vec<Value> = posts.iter().map(|p| p.data.clone()).collect();
        context.insert("posts", &posts_json);
        context.insert("all_tags", all_tags);
        
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
}