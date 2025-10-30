//! 文章处理模块
//! 
//! 负责解析 Markdown 文件，提取元数据和内容

use crate::error::{Error, Result};
use pulldown_cmark::{html, Options, Parser};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;
use walkdir::WalkDir;

/// 文章结构
#[derive(Debug, Clone)]
pub struct Post {
    /// 文章元数据和内容
    pub data: Value,
}

impl Post {
    /// 从 JSON 值创建文章
    pub fn from_value(data: Value) -> Self {
        Post { data }
    }
    
    /// 获取文章标题
    pub fn title(&self) -> Option<&str> {
        self.data.get("title").and_then(|v| v.as_str())
    }
    
    /// 获取文章 slug
    pub fn slug(&self) -> Option<&str> {
        self.data.get("slug").and_then(|v| v.as_str())
    }
    
    /// 获取文章内容
    pub fn content(&self) -> Option<&str> {
        self.data.get("content").and_then(|v| v.as_str())
    }
    
    /// 获取文章分类
    pub fn categories(&self) -> Vec<String> {
        self.data
            .get("categories")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// 获取文章标签
    pub fn tags(&self) -> Vec<String> {
        self.data
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// 获取文章日期
    pub fn date(&self) -> Option<&str> {
        self.data.get("date_ymd").and_then(|v| v.as_str())
    }
}

/// 文章解析器
pub struct PostParser;

impl PostParser {
    /// 列出指定目录下的所有文章
    pub fn list_posts<P: AsRef<Path>>(md_dir: P) -> Result<Vec<Post>> {
        let mut posts = Vec::new();
        let content_dir = md_dir.as_ref();
        
        // 检查目录是否存在
        if !content_dir.exists() {
            println!("警告: Markdown目录 '{}' 不存在，创建空目录...", content_dir.display());
            std::fs::create_dir_all(content_dir)?;
        }
        
        for entry in WalkDir::new(content_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().map_or(false, |ext| ext == "md") {
                let content = std::fs::read_to_string(entry.path())
                    .map_err(|e| Error::Other(format!("无法读取文件 {:?}: {}", entry.path(), e)))?;
                
                if let Some(post_data) = Self::parse_post(&content, entry.path(), content_dir)? {
                    posts.push(Post::from_value(post_data));
                }
            }
        }
        
        // 按日期排序（最新的在前）
        posts.sort_by(|a, b| {
            let date_a = a.date().unwrap_or("");
            let date_b = b.date().unwrap_or("");
            date_b.cmp(date_a)
        });
        
        Ok(posts)
    }
    
    /// 解析单篇文章
    fn parse_post<P: AsRef<Path>>(content: &str, path: P, md_dir: P) -> Result<Option<Value>> {
        let path = path.as_ref();
        let md_dir = md_dir.as_ref();
        
        // 检查 front matter 类型
        let (fm_marker, end_marker) = if content.starts_with("+++") {
            ("+++", "+++\n")
        } else if content.starts_with("---") {
            ("---", "---\n")
        } else {
            return Ok(None);
        };

        // 查找 front matter 结束位置
        let start = fm_marker.len();
        let end = if let Some(pos) = content[start..].find(end_marker) {
            start + pos
        } else if let Some(pos) = content[start..].find(fm_marker) {
            start + pos
        } else {
            return Ok(None);
        };

        let front_matter = &content[start..end];
        let body = &content[end + fm_marker.len()..];

        // 解析front matter（YAML）
        let metadata: serde_yaml::Value = serde_yaml::from_str(front_matter)
            .map_err(|e| Error::Markdown(format!("解析front matter失败 {:?}: {}", path, e)))?;

        // 转换元数据为JSON
        let metadata_json = serde_json::to_value(&metadata)?;

        // 解析Markdown为HTML
        let html = Self::markdown_to_html(body);

        // 优先使用 front matter 中的 slug 字段，否则用文件名
        let mut slug = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        
        if let Value::Object(ref obj) = metadata_json {
            if let Some(Value::String(s)) = obj.get("slug") {
                if !s.is_empty() {
                    slug = s.clone();
                }
            }
        }

        // 从文件路径提取分类信息
        let categories = Self::extract_categories_from_path(path, md_dir);
        let categories_json: Vec<Value> = categories
            .into_iter()
            .map(|cat| Value::String(cat))
            .collect();

        // 创建完整的文章对象
        let mut post = match metadata_json {
            Value::Object(mut obj) => {
                obj.insert("content".to_string(), Value::String(html));
                obj.insert("slug".to_string(), Value::String(slug));
                obj.insert("categories".to_string(), Value::Array(categories_json));
                Value::Object(obj)
            },
            _ => {
                let mut obj = serde_json::Map::new();
                obj.insert("content".to_string(), Value::String(html));
                obj.insert("slug".to_string(), Value::String(slug));
                obj.insert("categories".to_string(), Value::Array(categories_json));
                Value::Object(obj)
            }
        };

        // 处理日期相关字段
        if let Some(obj) = post.as_object_mut() {
            // 如果没有 title 字段，用 slug 作为 title
            if !obj.contains_key("title") {
                let slug = obj.get("slug").and_then(|v| v.as_str()).unwrap_or("").to_string();
                obj.insert("title".to_string(), Value::String(slug));
            }
            
            // 处理创建时间字段
            if let Some(create_time) = obj.get("createTime").and_then(|v| v.as_str()) {
                let create_time_str = create_time.to_string(); // 复制字符串避免借用问题
                let date_only = if create_time_str.len() >= 10 { 
                    &create_time_str[0..10] 
                } else { 
                    &create_time_str 
                };
                
                let date_only_string = date_only.to_string();
                obj.insert("date_ymd".to_string(), Value::String(date_only_string.clone()));
                
                if date_only.len() >= 7 {
                    let year = &date_only[0..4];
                    let ym = &date_only[0..7];
                    obj.insert("year".to_string(), Value::String(year.to_string()));
                    obj.insert("year_month".to_string(), Value::String(ym.to_string()));
                }
            }
        }

        Ok(Some(post))
    }
    
    /// 从文件路径提取分类信息
    fn extract_categories_from_path<P: AsRef<Path>>(path: P, md_dir: P) -> Vec<String> {
        let path = path.as_ref();
        let md_dir = md_dir.as_ref();
        let mut categories = Vec::new();
        
        // 获取相对于md_dir的路径
        if let Ok(relative_path) = path.strip_prefix(md_dir) {
            // 获取父目录路径
            if let Some(parent) = relative_path.parent() {
                // 将路径组件转换为分类
                for component in parent.components() {
                    if let std::path::Component::Normal(os_str) = component {
                        if let Some(category) = os_str.to_str() {
                            categories.push(category.to_string());
                        }
                    }
                }
            }
        }
        
        categories
    }
    
    /// 将Markdown转换为HTML
    fn markdown_to_html(markdown: &str) -> String {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        
        let parser = Parser::new_ext(markdown, options);
        let mut html = String::new();
        html::push_html(&mut html, parser);
        
        html
    }
    
    /// 统计所有标签及计数
    pub fn collect_tags(posts: &[Post]) -> Vec<Value> {
        let mut tag_to_count: BTreeMap<String, usize> = BTreeMap::new();
        
        for post in posts {
            for tag in post.tags() {
                *tag_to_count.entry(tag).or_insert(0) += 1;
            }
        }
        
        tag_to_count
            .into_iter()
            .map(|(name, count)| {
                let mut obj = serde_json::Map::new();
                obj.insert("name".to_string(), Value::String(name));
                obj.insert("count".to_string(), Value::from(count as u64));
                Value::Object(obj)
            })
            .collect()
    }
    
    /// 统计所有年份及计数
    pub fn collect_years(posts: &[Post]) -> Vec<Value> {
        let mut year_to_count: BTreeMap<String, usize> = BTreeMap::new();
        
        for post in posts {
            if let Some(year) = post.data.get("year").and_then(|v| v.as_str()) {
                *year_to_count.entry(year.to_string()).or_insert(0) += 1;
            }
        }
        
        year_to_count
            .into_iter()
            .map(|(name, count)| {
                let mut obj = serde_json::Map::new();
                obj.insert("name".to_string(), Value::String(name));
                obj.insert("count".to_string(), Value::from(count as u64));
                Value::Object(obj)
            })
            .collect()
    }
    
    /// 生成层次化的分类结构
    pub fn generate_hierarchical_categories(posts: &[Post]) -> Value {
        use std::collections::HashMap;
        
        // 构建分类树结构
        #[derive(Debug)]
        struct CategoryNode {
            name: String,
            count: usize,
            children: HashMap<String, CategoryNode>,
            full_path: Vec<String>,
        }
        
        impl CategoryNode {
            fn new(name: String, full_path: Vec<String>) -> Self {
                Self {
                    name,
                    count: 0,
                    children: HashMap::new(),
                    full_path,
                }
            }
            
            fn to_json(&self) -> Value {
                let mut obj = serde_json::Map::new();
                obj.insert("name".to_string(), Value::String(self.name.clone()));
                obj.insert("count".to_string(), Value::from(self.count as u64));
                obj.insert("path".to_string(), Value::Array(
                    self.full_path.iter().map(|s| Value::String(s.clone())).collect()
                ));
                
                if !self.children.is_empty() {
                    let mut children: Vec<Value> = self.children
                        .values()
                        .map(|child| child.to_json())
                        .collect();
                    children.sort_by(|a, b| {
                        let name_a = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let name_b = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        name_a.cmp(name_b)
                    });
                    obj.insert("children".to_string(), Value::Array(children));
                }
                
                Value::Object(obj)
            }
        }
        
        let mut root = CategoryNode::new("root".to_string(), vec![]);
        
        // 遍历所有文章，构建分类树
        for post in posts {
            let categories = post.categories();
            if !categories.is_empty() {
                // 在分类路径上的每个节点都增加计数
                let mut current = &mut root;
                let mut current_path = vec![];
                
                for category in &categories {
                    current_path.push(category.clone());
                    current = current.children
                        .entry(category.clone())
                        .or_insert_with(|| CategoryNode::new(category.clone(), current_path.clone()));
                    current.count += 1;
                }
            }
        }
        
        // 转换为JSON格式
        if root.children.is_empty() {
            Value::Array(vec![])
        } else {
            let mut categories: Vec<Value> = root.children
                .values()
                .map(|child| child.to_json())
                .collect();
            categories.sort_by(|a, b| {
                let name_a = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let name_b = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
                name_a.cmp(name_b)
            });
            Value::Array(categories)
        }
    }
}