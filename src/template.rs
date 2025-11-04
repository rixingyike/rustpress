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
    content_dir: std::path::PathBuf,
}

impl TemplateEngine {
    /// 将 toml::Value 递归转换为 serde_json::Value
    fn toml_to_json(value: &toml::Value) -> serde_json::Value {
        use serde_json::Value as JsonValue;
        match value {
            toml::Value::String(s) => JsonValue::String(s.clone()),
            toml::Value::Integer(i) => JsonValue::Number((*i).into()),
            toml::Value::Float(f) => serde_json::Number::from_f64(*f)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null),
            toml::Value::Boolean(b) => JsonValue::Bool(*b),
            toml::Value::Datetime(dt) => JsonValue::String(dt.to_string()),
            toml::Value::Array(arr) => {
                let items = arr.iter().map(Self::toml_to_json).collect::<Vec<_>>();
                JsonValue::Array(items)
            }
            toml::Value::Table(table) => {
                let mut map = serde_json::Map::new();
                for (k, v) in table.iter() {
                    map.insert(k.clone(), Self::toml_to_json(v));
                }
                JsonValue::Object(map)
            }
        }
    }
    /// 创建新的模板引擎
    pub fn new<P: AsRef<std::path::Path>>(config: Config, content_dir: P) -> Result<Self> {
        let tera = Tera::new("src/themes/default/templates/**/*")?;
        
        Ok(TemplateEngine { tera, config, content_dir: content_dir.as_ref().to_path_buf() })
    }
    
    /// 创建基础上下文
    fn create_base_context(&self) -> Context {
        use serde_json::Value as JsonValue;

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
        
        // 合并广告配置到site.ads（支持嵌套，如 ads.google）
        if let Some(ads) = self.config.data.get("ads") {
            if let toml::Value::Table(ads_table) = ads {
                let mut ads_config = serde_json::Map::new();
                for (key, value) in ads_table {
                    let json_value = match value {
                        toml::Value::String(s) => JsonValue::String(s.clone()),
                        toml::Value::Integer(i) => JsonValue::Number((*i).into()),
                        toml::Value::Float(f) => JsonValue::Number(serde_json::Number::from_f64(*f).unwrap_or_else(|| 0.into())),
                        toml::Value::Boolean(b) => JsonValue::Bool(*b),
                        toml::Value::Table(_) | toml::Value::Array(_) => Self::toml_to_json(value),
                        _ => JsonValue::Null,
                    };
                    ads_config.insert(key.clone(), json_value);
                }
                site_config.insert("ads".to_string(), JsonValue::Object(ads_config));
            }
        }

        // 合并菜单配置到 site.menu，并按 weight 排序 main 菜单
        if let Some(menu) = self.config.data.get("menu") {
            if let toml::Value::Table(menu_table) = menu {
                if let Some(toml::Value::Array(main_arr)) = menu_table.get("main") {
                    let mut main_items: Vec<JsonValue> = Vec::new();
                    for item in main_arr {
                        if let toml::Value::Table(tbl) = item {
                            let name = tbl.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let url = tbl.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let weight = tbl.get("weight").and_then(|v| v.as_integer()).unwrap_or(i64::MAX);
                            let mut map = serde_json::Map::new();
                            map.insert("name".to_string(), JsonValue::String(name));
                            map.insert("url".to_string(), JsonValue::String(url));
                            map.insert("weight".to_string(), JsonValue::Number(weight.into()));
                            main_items.push(JsonValue::Object(map));
                        }
                    }
                    // 按 weight 升序排序
                    main_items.sort_by(|a, b| {
                        let wa = a.as_object().and_then(|m| m.get("weight")).and_then(|v| v.as_i64()).unwrap_or(i64::MAX);
                        let wb = b.as_object().and_then(|m| m.get("weight")).and_then(|v| v.as_i64()).unwrap_or(i64::MAX);
                        wa.cmp(&wb)
                    });
                    let mut menu_obj = serde_json::Map::new();
                    menu_obj.insert("main".to_string(), JsonValue::Array(main_items));
                    site_config.insert("menu".to_string(), JsonValue::Object(menu_obj));
                } else {
                    // 非预期结构时直接递归转换
                    site_config.insert("menu".to_string(), Self::toml_to_json(menu));
                }
            } else {
                site_config.insert("menu".to_string(), Self::toml_to_json(menu));
            }
        }
        
        // 如果存在 md_dir/build.toml，则读取其中的 sidebar 数据并注入 site.sidebar
        let build_path = self.content_dir.join("build.toml");
        if build_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&build_path) {
                if let Ok(build_value) = content.parse::<toml::Value>() {
                    if let Some(sidebar_value) = build_value.get("sidebar") {
                        let sidebar_json = Self::toml_to_json(sidebar_value);
                        if let JsonValue::Object(obj) = sidebar_json {
                            site_config.insert("sidebar".to_string(), JsonValue::Object(obj));
                        } else {
                            site_config.insert("sidebar".to_string(), sidebar_json);
                        }
                    }
                }
            }
        }

        // 合并功能开关到 site.features
        if let Some(features) = self.config.data.get("features") {
            site_config.insert("features".to_string(), Self::toml_to_json(features));
        }

        // 合并 Analytics 配置到 site.analytics
        if let Some(analytics) = self.config.data.get("analytics") {
            site_config.insert("analytics".to_string(), Self::toml_to_json(analytics));
        }

        // 合并评论（giscus）配置到 site.comments
        if let Some(comments) = self.config.data.get("comments") {
            site_config.insert("comments".to_string(), Self::toml_to_json(comments));
        }

        // 插入合并后的站点配置
        context.insert("site", &JsonValue::Object(site_config));
        
        // 插入当前时间
        let now = Utc::now();
        context.insert("now", &now);
        
        context
    }
    
    /// 渲染首页（第1页）
    pub fn render_index(&self, posts: &[Post], all_tags: &[Value], all_categories: &Value) -> Result<String> {
        let mut context = self.create_base_context();
        
        // 获取每页文章数量配置
        let posts_per_page = self.config.data.get("homepage")
            .and_then(|v| v.get("posts_per_page"))
            .and_then(|v| v.as_integer())
            .unwrap_or(10) as usize;
        
        // 倒序排列：按日期降序（最新的在前）
        let mut sorted_posts: Vec<&Post> = posts.iter().collect();
        sorted_posts.sort_by(|a, b| {
            let date_a = a.date().unwrap_or("");
            let date_b = b.date().unwrap_or("");
            date_b.cmp(date_a) // 降序排列，最新的在前
        });
        
        // 计算总页数
        let total_pages = (posts.len() + posts_per_page - 1) / posts_per_page;
        
        // 首页显示最新文章，不一定是满页
        // 只显示最新的posts_per_page篇文章
        let page_posts: Vec<Value> = sorted_posts
            .iter()
            .take(posts_per_page)
            .map(|p| p.data.clone())
            .collect();
        
        // 首页不显示页码，显示为"最新"
        context.insert("posts", &page_posts);
        context.insert("all_tags", all_tags);
        context.insert("all_categories", all_categories);
        context.insert("current_page", &0); // 0表示"最新"页
        context.insert("total_pages", &total_pages);
        // 范围显示：只有一页时为第1到N条
        let start_display = if sorted_posts.is_empty() { 0 } else { 1 };
        let end_display = std::cmp::min(posts_per_page, sorted_posts.len());
        context.insert("start_index", &start_display);
        context.insert("end_index", &end_display);
        
        // 导航逻辑：单页没有上一页/下一页
        context.insert("has_previous_page", &false);
        context.insert("has_next_page", &false);
        
        self.tera.render("index.html", &context)
            .map_err(Error::Template)
    }
    
    /// 渲染首页分页页面
    pub fn render_index_page(&self, posts: &[Post], all_tags: &[Value], all_categories: &Value, page: usize) -> Result<String> {
        let mut context = self.create_base_context();
        
        // 获取每页文章数量配置
        let posts_per_page = self.config.data.get("homepage")
            .and_then(|v| v.get("posts_per_page"))
            .and_then(|v| v.as_integer())
            .unwrap_or(10) as usize;
        
        // 倒序排列：按日期降序（最新的在前）
        let mut sorted_posts: Vec<&Post> = posts.iter().collect();
        sorted_posts.sort_by(|a, b| {
            let date_a = a.date().unwrap_or("");
            let date_b = b.date().unwrap_or("");
            date_b.cmp(date_a) // 降序排列，最新的在前
        });
        
        // 计算总页数
        let total_pages = (posts.len() + posts_per_page - 1) / posts_per_page;
        
        // 验证页码
        if page < 1 || page > total_pages {
            return Err(Error::Other(format!("无效的页码: {}，总页数: {}", page, total_pages)));
        }
        
        // 计算分页范围（倒分页，余数放到首页）：
        // page=1（最旧）总是满 posts_per_page 条；page=total_pages（最新）可能是余数 r 条
        let total_posts = sorted_posts.len();
        let start_index = total_posts.saturating_sub(page * posts_per_page);
        let end_index = std::cmp::min(total_posts.saturating_sub((page - 1) * posts_per_page), total_posts);
        
        // 获取当前页的文章
        let page_posts: Vec<Value> = sorted_posts[start_index..end_index]
            .iter()
            .map(|p| p.data.clone())
            .collect();
        
        context.insert("posts", &page_posts);
        context.insert("all_tags", all_tags);
        context.insert("all_categories", all_categories);
        context.insert("current_page", &page);
        context.insert("total_pages", &total_pages);
        // 范围显示（按最旧为1编号）：
        // 对于按日期降序的列表，显示区间需转换为升序编号：
        // 最小显示 = total_posts - end_index + 1；最大显示 = total_posts - start_index
        let start_display = if sorted_posts.is_empty() { 0 } else { total_posts.saturating_sub(end_index) + 1 };
        let end_display = if sorted_posts.is_empty() { 0 } else { total_posts.saturating_sub(start_index) };
        context.insert("start_index", &start_display);
        context.insert("end_index", &end_display);
        
        // 导航逻辑（倒分页）：
        // 第1页（最旧）：上一页到更“新”的第2页；下一页不可用
        // 中间页：上一页到更“新”的页（页码更大）；下一页到更“旧”的页（页码更小）
        // 最大页（最新）：上一页不可用；下一页到更“旧”的第(total_pages-1)页
        context.insert("has_previous_page", &(page < total_pages)); // 左边（上一页）指向更新内容
        context.insert("has_next_page", &(page > 1)); // 右边（下一页）指向更旧内容
        
        self.tera.render("index.html", &context)
            .map_err(Error::Template)
    }
    
    /// 渲染文章页面
    pub fn render_post(&self, post: &Post, all_posts: &[Post]) -> Result<String> {
        let mut context = self.create_base_context();
        context.insert("page", &post.data);
        
        // 计算相关文章
        let related_posts = self.calculate_related_posts(post, all_posts);
        context.insert("related_posts", &related_posts);
        
        self.tera.render("post.html", &context)
            .map_err(Error::Template)
    }
    
    /// 计算相关文章
    fn calculate_related_posts(&self, current_post: &Post, all_posts: &[Post]) -> Vec<Value> {
        let mut scored_posts: Vec<(f64, &Post)> = Vec::new();
        
        // 获取当前文章的标签和分类
        let current_tags = current_post.tags();
        let current_categories = current_post.categories();
        
        for post in all_posts {
            // 跳过当前文章本身
            if post.slug() == current_post.slug() {
                continue;
            }
            
            let mut score = 0.0;
            
            // 计算标签相似度（权重较高）
            let post_tags = post.tags();
            for tag in &current_tags {
                if post_tags.contains(tag) {
                    score += 2.0; // 每个匹配标签加2分
                }
            }
            
            // 计算分类相似度（权重中等）
            let post_categories = post.categories();
            for category in &current_categories {
                if post_categories.contains(category) {
                    score += 1.5; // 每个匹配分类加1.5分
                }
            }
            
            // 如果没有任何匹配，给予基础分（基于发布日期相近度）
            if score == 0.0 {
                // 简单的日期相近度计算（越近的文章分数越高）
                if let (Some(current_date), Some(post_date)) = (current_post.date(), post.date()) {
                    if let (Ok(current_dt), Ok(post_dt)) = (
                        NaiveDate::parse_from_str(current_date, "%Y-%m-%d"),
                        NaiveDate::parse_from_str(post_date, "%Y-%m-%d")
                    ) {
                        let days_diff = current_dt.signed_duration_since(post_dt).num_days().abs();
                        score = 1.0 / (days_diff as f64 + 1.0); // 日期越近分数越高
                    }
                }
            }
            
            if score > 0.0 {
                scored_posts.push((score, post));
            }
        }
        
        // 按分数降序排序
        scored_posts.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        
        // 取前5个相关文章
        scored_posts
            .into_iter()
            .take(5)
            .map(|(_, post)| post.data.clone())
            .collect()
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

    /// 渲染单标签文章列表分页页面（倒分页：最大页为最新）
    pub fn render_tag_page(&self, posts: &[&Post], tag_name: &str, page: usize, posts_per_page: usize) -> Result<String> {
        let mut context = self.create_base_context();

        // 总文章数与总页数
        let total_posts = posts.len();
        let total_pages = if total_posts == 0 { 1 } else { (total_posts + posts_per_page - 1) / posts_per_page };

        // 页码边界检查
        if page < 1 || page > total_pages {
            return Err(Error::Other(format!("无效的页码: {}，总页数: {}", page, total_pages)));
        }

        // 当前页文章切片
        let start = (page - 1) * posts_per_page;
        let end = std::cmp::min(start + posts_per_page, total_posts);
        let page_posts: Vec<Value> = posts[start..end].iter().map(|p| p.data.clone()).collect();

        // 构建分页信息
        let start_index = if total_posts == 0 { 0 } else { start + 1 };
        let end_index = if total_posts == 0 { 0 } else { end };

        // 页面 URL 构建函数（倒分页）：最大页（最新）对应 index.html
        let page_url = |n: usize| -> String {
            if n == total_pages {
                format!("/tags/{}/index.html", tag_name)
            } else {
                format!("/tags/{}/index{}.html", tag_name, n)
            }
        };

        // 构建页码列表（显示倒序页码：最大页数在左侧）
        let mut pages_vec: Vec<Value> = Vec::new();
        for n in 1..=total_pages {
            let display_number = total_pages - n + 1; // 倒序显示
            let page_obj = serde_json::json!({
                "number": display_number,
                "url": page_url(n),
                "is_current": n == page
            });
            pages_vec.push(page_obj);
        }

        // 上一页/下一页链接（倒分页一致）：上一页为更“新”（页码加1），下一页为更“旧”（页码减1）
        let previous = if page < total_pages { Some(page_url(page + 1)) } else { None };
        let next = if page > 1 { Some(page_url(page - 1)) } else { None };

        let paginator = serde_json::json!({
            "previous": previous,
            "next": next,
            "total_posts": total_posts,
            "start_index": start_index,
            "end_index": end_index,
            "pages": pages_vec
        });

        context.insert("posts", &page_posts);
        context.insert("tag_name", &tag_name);
        context.insert("paginator", &paginator);

        self.tera.render("tag.html", &context)
            .map_err(Error::Template)
    }
    
    /// 渲染分类页面
    pub fn render_categories(&self, posts: &[Post], all_categories: &Value, all_tags: &[Value]) -> Result<String> {
        let mut context = self.create_base_context();
        
        let posts_json: Vec<Value> = posts.iter().map(|p| p.data.clone()).collect();
        context.insert("posts", &posts_json);
        context.insert("all_categories", all_categories);
        
        // 获取热门标签（前10个）
        let tags: Vec<Value> = all_tags.iter().take(10).cloned().collect();
        context.insert("tags", &tags);
        
        // 获取最新文章（前5篇）
        let mut sorted_posts: Vec<&Post> = posts.iter().collect();
        sorted_posts.sort_by(|a, b| {
            let date_a = a.date().unwrap_or("");
            let date_b = b.date().unwrap_or("");
            date_b.cmp(date_a) // 降序排列，最新的在前
        });
        let recent_posts: Vec<Value> = sorted_posts
            .iter()
            .take(5)
            .map(|p| p.data.clone())
            .collect();
        context.insert("recent_posts", &recent_posts);
        
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
    
    /// 渲染年份归档页面
    pub fn render_year_archive(&self, posts: &[&Post], year: &str) -> Result<String> {
        let mut context = self.create_base_context();
        
        let posts_json: Vec<Value> = posts.iter().map(|p| p.data.clone()).collect();
        context.insert("posts", &posts_json);
        context.insert("year", year);
        
        self.tera.render("year_archive.html", &context)
            .map_err(Error::Template)
    }
    
    /// 渲染关于页面
    pub fn render_about(&self, posts: &[Post], all_tags: &[Value], all_categories: &Value) -> Result<String> {
        let mut context = self.create_base_context();

        // 基础数据（用于侧边栏/站点信息）
        let posts_json: Vec<Value> = posts.iter().map(|p| p.data.clone()).collect();
        context.insert("posts", &posts_json);
        context.insert("all_tags", all_tags);
        context.insert("all_categories", all_categories);

        // 读取并解析 about.md，注入页面内容与 frontmatter
        let about_path = std::path::Path::new("source/about.md");
        if about_path.exists() {
            if let Ok(content) = std::fs::read_to_string(about_path) {
                if let Ok(Some(about_data)) = crate::post::PostParser::parse_file_content(&content, about_path, std::path::Path::new("source")) {
                    // 注入完整的 page 数据，供模板访问 frontmatter（如 toc）
                    context.insert("page", &about_data);
                    // 单独注入 page_content 以保持现有模板兼容
                    if let Some(page_content) = about_data.get("content") {
                        context.insert("page_content", page_content);
                    }
                }
            }
        }

        self.tera.render("about.html", &context)
            .map_err(Error::Template)
    }
    
    /// 渲染友链页面
    pub fn render_friends(&self) -> Result<String> {
        let mut context = self.create_base_context();
        
        // 读取friends.md文件
        let friends_path = std::path::Path::new("source/friends.md");
        if friends_path.exists() {
            let content = std::fs::read_to_string(friends_path)
                .map_err(|e| Error::Other(format!("读取friends.md失败: {}", e)))?;
            
            // 解析friends.md文件（通过公开包装方法）
            if let Ok(Some(friends_data)) = crate::post::PostParser::parse_file_content(&content, friends_path, std::path::Path::new("source")) {
                if let Some(friends_list) = friends_data.get("friends") {
                    context.insert("friends", friends_list);
                }
                
                // 添加页面内容
                if let Some(page_content) = friends_data.get("content") {
                    context.insert("page_content", page_content);
                }
            }
        }
        
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

    /// 渲染单分类文章列表分页页面（倒分页：最大页为最新）
    pub fn render_category_page(
        &self,
        posts: &[&Post],
        category_path: &[String],
        page: usize,
        posts_per_page: usize,
    ) -> Result<String> {
        let mut context = self.create_base_context();

        // 总文章数与总页数
        let total_posts = posts.len();
        let total_pages = if total_posts == 0 { 1 } else { (total_posts + posts_per_page - 1) / posts_per_page };

        // 页码边界检查
        if page < 1 || page > total_pages {
            return Err(Error::Other(format!("无效的页码: {}，总页数: {}", page, total_pages)));
        }

        // 当前页文章切片（按传入顺序）
        let start = (page - 1) * posts_per_page;
        let end = std::cmp::min(start + posts_per_page, total_posts);
        let page_posts: Vec<Value> = posts[start..end].iter().map(|p| p.data.clone()).collect();

        // 构建分页信息（范围编号按最旧为1的顺序展示）
        let start_index = if total_posts == 0 { 0 } else { start + 1 };
        let end_index = if total_posts == 0 { 0 } else { end };

        // 分类名称（用于标题等展示）
        let category_name = category_path.last().cloned().unwrap_or_default();

        // 页面 URL 构建函数（倒分页）：最大页（最新）对应 index.html
        let base_path = format!("/{}", category_path.join("/"));
        let page_url = |n: usize| -> String {
            if n == total_pages {
                format!("{}/index.html", base_path)
            } else {
                format!("{}/index{}.html", base_path, n)
            }
        };

        // 构建页码列表（显示倒序页码：最大页数在左侧）
        let mut pages_vec: Vec<Value> = Vec::new();
        for n in 1..=total_pages {
            let display_number = total_pages - n + 1; // 倒序显示
            let page_obj = serde_json::json!({
                "number": display_number,
                "url": page_url(n),
                "is_current": n == page
            });
            pages_vec.push(page_obj);
        }

        // 上一页/下一页链接（倒分页）：上一页为更“新”（页码加1），下一页为更“旧”（页码减1）
        let previous = if page < total_pages { Some(page_url(page + 1)) } else { None };
        let next = if page > 1 { Some(page_url(page - 1)) } else { None };

        let paginator = serde_json::json!({
            "previous": previous,
            "next": next,
            "total_posts": total_posts,
            "start_index": start_index,
            "end_index": end_index,
            "pages": pages_vec
        });

        context.insert("posts", &page_posts);
        context.insert("category_name", &category_name);
        context.insert("paginator", &paginator);

        self.tera.render("category.html", &context)
            .map_err(Error::Template)
    }
    
    /// 渲染带侧边栏的分类页面
    pub fn render_category_with_sidebar(
        &self, 
        posts: &[&Post], 
        category_name: &str,
        category_path: &[String],
        subcategories: &[Value],
        related_tags: &[Value]
    ) -> Result<String> {
        let mut context = self.create_base_context();
        
        // 转换 posts 为 JSON 值
        let posts_json: Vec<Value> = posts.iter().map(|p| p.data.clone()).collect();
        context.insert("posts", &posts_json);
        context.insert("category_name", category_name);
        context.insert("category_path", category_path);
        context.insert("subcategories", subcategories);
        context.insert("related_tags", related_tags);
        
        // 构建面包屑导航
        let breadcrumbs: Vec<Value> = category_path.iter()
            .enumerate()
            .map(|(i, cat)| {
                let mut map = serde_json::Map::new();
                map.insert("name".to_string(), Value::String(cat.clone()));
                map.insert("url".to_string(), Value::String(format!("/{}/", category_path[0..=i].join("/"))));
                Value::Object(map)
            })
            .collect();
        context.insert("breadcrumbs", &breadcrumbs);
        
        self.tera.render("category_with_sidebar.html", &context)
            .map_err(Error::Template)
    }
}
