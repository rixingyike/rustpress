//! 文章处理模块
//!
//! 负责解析 Markdown 文件，提取元数据和内容

use crate::error::{Error, Result};
use pulldown_cmark::{Options, Parser, html};
use regex::Regex;
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
        let mut tags: Vec<String> = self
            .data
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        // 去重（保持顺序）
        let mut seen = std::collections::HashSet::new();
        tags.retain(|t| seen.insert(t.clone()));
        tags
    }

    /// 获取文章日期
    pub fn date(&self) -> Option<&str> {
        self.data.get("date_ymd").and_then(|v| v.as_str())
    }

    /// 获取文章完整创建时间（包含时分秒）
    pub fn create_time(&self) -> Option<&str> {
        self.data.get("createTime").and_then(|v| v.as_str())
    }

    /// 获取文章 URL
    pub fn url(&self) -> Option<&str> {
        self.data.get("url").and_then(|v| v.as_str())
    }

    /// 获取源文件路径
    pub fn source_path(&self) -> Option<&str> {
        self.data.get("source_path").and_then(|v| v.as_str())
    }

    /// 获取源文件的最后修改时间（UNIX秒）
    pub fn modified_epoch(&self) -> Option<i64> {
        self.data.get("modified_epoch").and_then(|v| v.as_i64())
    }
}

/// 文章解析器
pub struct PostParser;

impl PostParser {
    /// 从 Markdown 文本中提取标题：优先首个 H1（`# 标题`），否则首个任意级别标题
    fn extract_title_from_markdown(markdown: &str) -> Option<String> {
        // 先扫描首个 H1
        let mut in_code_fence = false;
        for line in markdown.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_fence = !in_code_fence;
                continue;
            }
            if in_code_fence {
                continue;
            }
            if trimmed.starts_with("# ") {
                let title = trimmed[2..].trim();
                if !title.is_empty() {
                    return Some(title.to_string());
                }
            }
        }
        // 若没有 H1，则退而求其次，找任意级别标题
        in_code_fence = false;
        for line in markdown.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_fence = !in_code_fence;
                continue;
            }
            if in_code_fence {
                continue;
            }
            if trimmed.starts_with('#') {
                let hashes = trimmed.chars().take_while(|c| *c == '#').count();
                if hashes >= 1 {
                    let title = trimmed[hashes..].trim();
                    if !title.is_empty() {
                        return Some(title.to_string());
                    }
                }
            }
        }
        None
    }

    /// 列出指定目录下的所有文章
    pub fn list_posts<P: AsRef<Path>>(md_dir: P) -> Result<Vec<Post>> {
        let mut posts = Vec::new();
        let content_dir = md_dir.as_ref();

        // 检查目录是否存在
        if !content_dir.exists() {
            println!(
                "警告: Markdown目录 '{}' 不存在，创建空目录...",
                content_dir.display()
            );
            std::fs::create_dir_all(content_dir)?;
        }

        // 预扫描：查找所有 docs 下 README.md 标记为 draft 的目录，以及显式标记为 layout: doc 的书籍目录
        let mut draft_dirs = std::collections::HashSet::new();
        let mut book_dirs = std::collections::HashMap::new(); // 存储目录 -> 封面路径的映射
        for entry in WalkDir::new(content_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.file_name().map_or(false, |n| n == "README.md") {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Ok(Some(post_data)) = Self::parse_post(&content, path, content_dir) {
                        let is_draft = post_data.get("draft").and_then(|v| v.as_bool()).unwrap_or(false);
                        if is_draft {
                            if let Some(parent) = path.parent() {
                                draft_dirs.insert(parent.to_path_buf());
                            }
                        }

                        // 识别书籍目录：在 docs/ 下且 README.md 的 layout 为 doc
                        let cats = Self::extract_categories_from_path(path, content_dir);
                        if cats.len() == 2 && cats[0] == "docs" {
                            if let Some("doc") = post_data.get("layout").and_then(|v| v.as_str()) {
                                if let Some(parent) = path.parent() {
                                    let mut cover_path = post_data.get("cover").and_then(|v| v.as_str()).map(|s| s.to_string());
                                    
                                    // 1. 如果手动设置了封面且是相对路径，转换为站点绝对路径
                                    if let Some(cp) = cover_path.as_mut() {
                                        if !cp.starts_with('/') && !cp.starts_with("http") {
                                            if let Ok(rel_dir) = parent.strip_prefix(content_dir) {
                                                *cp = format!("/{}", rel_dir.join(&cp).to_string_lossy());
                                            }
                                        }
                                    }

                                    // 2. 自动探测同级目录或 assets/ 下的 cover.jpg 或 cover.png
                                    if cover_path.is_none() {
                                        let candidates = [
                                            parent.join("cover.jpg"), parent.join("cover.png"),
                                            parent.join("assets").join("cover.jpg"), parent.join("assets").join("cover.png")
                                        ];
                                        for cand in candidates {
                                            if cand.exists() {
                                                if let Ok(rel) = cand.strip_prefix(content_dir) {
                                                    cover_path = Some(format!("/{}", rel.to_string_lossy()));
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    book_dirs.insert(parent.to_path_buf(), cover_path);
                                }
                            }
                        }
                    }
                }
            }
        }

        for entry in WalkDir::new(content_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().map_or(false, |ext| ext == "md") {
                // 跳过根层下的 README.md（它是主页本身的配置文件，不作为文章列表项）
                if entry.path() == content_dir.join("README.md") {
                    continue;
                }
                // 如果文件在被禁用的 draft 目录下，则跳过
                if draft_dirs.iter().any(|d| entry.path().starts_with(d)) {
                    continue;
                }

                // 跳过隐藏的 Markdown 文件（文件名以点开头）
                let hidden = entry.file_name().to_string_lossy().starts_with('.');
                if hidden {
                    continue;
                }
                let content = std::fs::read_to_string(entry.path())
                    .map_err(|e| Error::Other(format!("无法读取文件 {:?}: {}", entry.path(), e)))?;
                    if let Ok(Some(mut post)) = Self::parse_post(&content, entry.path(), content_dir) {
                        // 检查 draft 字段，如果是 true 则跳过
                        let is_draft = post
                            .get("draft")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        if is_draft {
                            continue;
                        }

                        // 处理布局与封面数据逻辑
                        let cats = Post::from_value(post.clone()).categories();
                        if let Some(obj) = post.as_object_mut() {
                            // 1. 自动应用分支逻辑（仅在未设置布局时）
                            if !obj.contains_key("layout") {
                                if cats.first().map(|c| c == "projects").unwrap_or(false) {
                                    obj.insert("layout".to_string(), Value::String("project".to_string()));
                                } else if book_dirs.iter().any(|(d, _)| entry.path().starts_with(d)) {
                                    obj.insert("layout".to_string(), Value::String("doc".to_string()));
                                }
                            }

                            // 2. 注入探测到的书籍封面（仅限 README.md）
                            if entry.path().file_name().map_or(false, |n| n == "README.md") {
                                if let Some((_, cover_opt)) = book_dirs.iter().find(|(d, _)| entry.path().starts_with(d)) {
                                    if let Some(cp) = cover_opt {
                                        // 始终使用 pre-scan 阶段处理过的标准化路径（绝对路径）
                                        obj.insert("cover".to_string(), Value::String(cp.clone()));
                                    }
                                }
                            }
                        }
                        posts.push(Post::from_value(post));
                    }
            }
        }

        // 按时间排序（最新的在前，包含时分秒）
        posts.sort_by(|a, b| {
            let time_a = a.create_time().unwrap_or("");
            let time_b = b.create_time().unwrap_or("");
            time_b.cmp(time_a)
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

        // 解析front matter
        let metadata_json = if fm_marker == "+++" {
            let metadata: toml::Value = toml::from_str(front_matter).map_err(|e| {
                Error::Markdown(format!("解析TOML front matter失败 {:?}: {}", path, e))
            })?;
            serde_json::to_value(metadata)?
        } else {
            // 针对 YAML 做鲁棒性处理：
            // 1. 修复中文冒号为英文冒号
            let mut fixed_front_matter = front_matter.replace('：', ":");
            
            // 2. 修复缺少空格的键值对 (e.g. "key:value" -> "key: value")
            let re = Regex::new(r"(?m)^([ \t]*[a-zA-Z0-9_-]+):([^\s].*)$").unwrap();
            fixed_front_matter = re.replace_all(&fixed_front_matter, "${1}: ${2}").to_string();

            let metadata: serde_yaml::Value =
                serde_yaml::from_str(&fixed_front_matter).map_err(|e| {
                    // 如果解析失败，打印原始内容以便调试
                    Error::Markdown(format!("解析YAML front matter失败 {:?}: {}", path, e))
                })?;
            serde_json::to_value(metadata)?
        };

        // 解析Markdown为HTML（不在解析阶段追加任何额外内容）
        let html = Self::markdown_to_html(body);

        // 优先使用 front matter 中的 slug 字段，否则用文件名
        let mut slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        if slug == "README" {
            slug = "index".to_string();
        }

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
            .iter()
            .map(|cat| Value::String(cat.clone()))
            .collect();

        // 生成 URL
        let url = if categories.is_empty() {
            format!("/{}.html", slug)
        } else {
            if categories[0] == "works" && categories.len() == 2 && slug == "index" {
                format!("/works/{}.html", categories[1])
            } else {
                format!("/{}/{}.html", categories.join("/"), slug)
            }
        };

        // 创建完整的文章对象
        let mut post = match metadata_json {
            Value::Object(mut obj) => {
                if let Some(orig_url) = obj.get("url").cloned() {
                    obj.insert("buy_url".to_string(), orig_url);
                }
                obj.insert("content".to_string(), Value::String(html));
                obj.insert("slug".to_string(), Value::String(slug));
                obj.insert("url".to_string(), Value::String(url));
                obj.insert("categories".to_string(), Value::Array(categories_json.clone()));
                Value::Object(obj)
            }
            _ => {
                let mut obj = serde_json::Map::new();
                obj.insert("content".to_string(), Value::String(html));
                obj.insert("slug".to_string(), Value::String(slug));
                obj.insert("url".to_string(), Value::String(url));
                obj.insert("categories".to_string(), Value::Array(categories_json.clone()));
                Value::Object(obj)
            }
        };

        // 处理日期相关字段
        if let Some(obj) = post.as_object_mut() {
            // 记录源文件路径与修改时间戳（用于增量编译）
            obj.insert(
                "source_path".to_string(),
                Value::String(path.to_string_lossy().to_string()),
            );
            obj.insert(
                "file_name".to_string(),
                Value::String(path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string()),
            );
            let modified_epoch = std::fs::metadata(path)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|st| st.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            obj.insert(
                "modified_epoch".to_string(),
                Value::Number(modified_epoch.into()),
            );
            
            // 如果属于某个专栏 (首个分类是 "columns" 且分类数 >= 2)，提取专栏标题 (即同级 README.md 的 title) 并注入 column_title
            if categories.first().map(|c| c == "columns").unwrap_or(false) && categories.len() >= 2 {
                if let Some(parent) = path.parent() {
                    let readme_path = parent.join("README.md");
                    if readme_path.exists() {
                        if let Ok(readme_content) = std::fs::read_to_string(&readme_path) {
                            let re_title = Regex::new(r#"(?m)^title:\s*['"]?([^'"\n]+)['"]?"#).unwrap();
                            if let Some(caps) = re_title.captures(&readme_content) {
                                let col_title = caps.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
                                obj.insert("column_title".to_string(), Value::String(col_title.clone()));

                            }
                        }
                    }
                }
            }

            // 如果没有 title 字段，尝试从 Markdown 内容提取标题
            if !obj.contains_key("title") {
                let content_md_title = Self::extract_title_from_markdown(body).or_else(|| {
                    // 兜底：使用 slug
                    obj.get("slug")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                });
                if let Some(title) = content_md_title {
                    obj.insert("title".to_string(), Value::String(title));
                }
            }

            // 处理创建时间字段（兼容多分隔符并归一化为 YYYY-MM-DD HH:MM:SS）
            // 优先使用 createTime，如果不存在则尝试使用 date，若都不存在则使用当前时间，若仍不存在/失败则以 2025-11-05 08:00:00 兜底
            let time_val = obj
                .get("createTime")
                .or_else(|| obj.get("date"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let mut create_time_str = match time_val {
                Some(t) => t,
                None => {
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
                }
            };
            
            // 归一化分隔符并在对象中统一为 createTime
            create_time_str = create_time_str.replace('/', "-").replace('.', "-");
            obj.insert("createTime".to_string(), Value::String(create_time_str.clone()));

            let date_only = if create_time_str.len() >= 10 {
                &create_time_str[0..10]
            } else {
                &create_time_str
            };
            let mut normalized = date_only.to_string();
            // 确保格式长度为10且分隔符在位置4和7
            if normalized.len() == 10 {
                let bytes = normalized.as_bytes();
                let is_digit = |c: u8| c.is_ascii_digit();
                if !(is_digit(bytes[0])
                    && is_digit(bytes[1])
                    && is_digit(bytes[2])
                    && is_digit(bytes[3])
                    && bytes[4] == b'-'
                    && is_digit(bytes[5])
                    && is_digit(bytes[6])
                    && bytes[7] == b'-'
                    && is_digit(bytes[8])
                    && is_digit(bytes[9]))
                {
                    // 尝试强制重组为 YYYY-MM-DD
                    let digits: Vec<char> =
                        date_only.chars().filter(|c| c.is_ascii_digit()).collect();
                    if digits.len() >= 8 {
                        let year: String = digits[0..4].iter().collect();
                        let month: String = digits[4..6].iter().collect();
                        let day: String = digits[6..8].iter().collect();
                        normalized = format!("{}-{}-{}", year, month, day);
                    } else {
                        normalized = "2025-11-05".to_string();
                    }
                }
            } else {
                let digits: Vec<char> =
                    date_only.chars().filter(|c| c.is_ascii_digit()).collect();
                if digits.len() >= 8 {
                    let year: String = digits[0..4].iter().collect();
                    let month: String = digits[4..6].iter().collect();
                    let day: String = digits[6..8].iter().collect();
                    normalized = format!("{}-{}-{}", year, month, day);
                } else {
                    normalized = "2025-11-05".to_string();
                }
            }
            let hm = if create_time_str.len() >= 16 {
                create_time_str[11..16].to_string()
            } else {
                "08:00".to_string()
            };
            obj.insert("date_ymd".to_string(), Value::String(normalized.clone()));
            obj.insert("create_time_hm".to_string(), Value::String(hm));
            // 仅当 front matter 未显式指定 year 时才自动派生（如著作页可自定出版年份）
            if !obj.contains_key("year") && normalized.len() >= 7 {
                let auto_year = &normalized[0..4];
                let ym = &normalized[0..7];
                obj.insert("year".to_string(), Value::String(auto_year.to_string()));
                obj.insert("year_month".to_string(), Value::String(ym.to_string()));
            }

            // 清洗标签：去除空字符串和仅空白的标签；若为空则移除
            if let Some(tags_val) = obj.get("tags") {
                if let Some(arr) = tags_val.as_array() {
                    let mut sanitized: Vec<Value> = arr
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|s| Value::String(s.to_string()))
                        .collect();
                    // 去重（保持顺序）
                    let mut seen = std::collections::HashSet::new();
                    sanitized.retain(|v| {
                        if let Some(s) = v.as_str() {
                            seen.insert(s.to_string())
                        } else {
                            false
                        }
                    });
                    if sanitized.is_empty() {
                        obj.remove("tags");
                    } else {
                        obj.insert("tags".to_string(), Value::Array(sanitized));
                    }
                } else {
                    // 非数组字段的非法标签，移除以避免渲染层误用
                    obj.remove("tags");
                }
            }

            // 标准化图片路径：相对路径转为基于分类目录的绝对路径
            let base_path = if !categories.is_empty() {
                format!("/{}/", categories.join("/"))
            } else {
                String::new()
            };
            let normalize_path = |s: &str| -> String {
                if s.starts_with('/') || s.starts_with("http") {
                    s.to_string()
                } else {
                    format!("{}{}", base_path, s)
                }
            };
            // 处理数组字段：images, screenshots
            for key in &["images", "screenshots"] {
                if let Some(val) = obj.get(*key) {
                    if let Some(arr) = val.as_array() {
                        let normalized: Vec<Value> = arr.iter().map(|v| {
                            v.as_str().map(|s| Value::String(normalize_path(s))).unwrap_or_else(|| v.clone())
                        }).collect();
                        obj.insert(key.to_string(), Value::Array(normalized));
                    }
                }
            }
            // 处理单值字段：cover, icon, avatar
            for key in &["cover", "icon", "avatar"] {
                if let Some(val) = obj.get(*key) {
                    if let Some(s) = val.as_str() {
                        obj.insert(key.to_string(), Value::String(normalize_path(s)));
                    }
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
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

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
                obj.insert(
                    "path".to_string(),
                    Value::Array(
                        self.full_path
                            .iter()
                            .map(|s| Value::String(s.clone()))
                            .collect(),
                    ),
                );

                if !self.children.is_empty() {
                    let mut children: Vec<Value> = self
                        .children
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

        // 排除的特殊目录（这些是内容类型而不是文章分类）
        let excluded: std::collections::HashSet<&str> =
            ["columns", "friends", "projects", "tweets", "works"].into();

        let mut root = CategoryNode::new("root".to_string(), vec![]);

        // 遍历所有文章，构建分类树
        for post in posts {
            let categories = post.categories();
            if !categories.is_empty() {
                // 跳过特殊内容类型目录（首个分类为 excluded 中的值）
                if categories.first().map(|c| excluded.contains(c.as_str())).unwrap_or(false) {
                    continue;
                }
                // 在分类路径上的每个节点都增加计数
                let mut current = &mut root;
                let mut current_path = vec![];

                for category in &categories {
                    current_path.push(category.clone());
                    current = current.children.entry(category.clone()).or_insert_with(|| {
                        CategoryNode::new(category.clone(), current_path.clone())
                    });
                    current.count += 1;
                }
            }
        }

        // 转换为JSON格式
        if root.children.is_empty() {
            Value::Array(vec![])
        } else {
            let mut categories: Vec<Value> = root
                .children
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

    /// 对外公开的单文件Markdown解析包装方法
    ///
    /// 用途：当需要解析一个具体的Markdown文件内容（例如 friends.md）时，
    /// 在模板渲染阶段可调用此方法以获得其 front matter 和 HTML 内容。
    pub fn parse_file_content<P: AsRef<Path>>(
        content: &str,
        path: P,
        md_dir: P,
    ) -> Result<Option<Value>> {
        Self::parse_post(content, path, md_dir)
    }

    /// 传入一个 md 绝对/相对路径，以及可选的文件内容（用于提取 slug），返回 url 路径信息（不包括域名），作为 id
    pub fn get_url_from_path<P: AsRef<Path>>(source_path: P, content_dir: P, content: Option<&str>) -> String {
        let path = source_path.as_ref();
        let md_dir = content_dir.as_ref();

        // 尝试解析内容（优先使用传入的内容，否则从磁盘读取）以获取最准确的 slug
        let content_to_use = match content {
            Some(c) => Some(c.to_string()),
            None => std::fs::read_to_string(path).ok(),
        };

        if let Some(c) = content_to_use {
            if let Ok(Some(post_val)) = Self::parse_post(&c, path, md_dir) {
                if let Some(url) = post_val.get("url").and_then(|v| v.as_str()) {
                    return url.to_string();
                }
            }
        }

        // 降级方案：手动计算
        let mut slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        if slug == "README" {
            slug = "index".to_string();
        }

        let categories = Self::extract_categories_from_path(path, md_dir);
        if categories.is_empty() {
            format!("/{}.html", slug)
        } else {
            if categories[0] == "works" && categories.len() == 2 && slug == "index" {
                format!("/works/{}.html", categories[1])
            } else {
                format!("/{}/{}.html", categories.join("/"), slug)
            }
        }
    }
}
