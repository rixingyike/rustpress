//! 工具函数模块
//! 
//! 提供各种实用的辅助函数

use crate::error::{Error, Result};
use crate::config::Config;
use crate::post::{Post, PostParser};
use std::path::Path;
use walkdir::WalkDir;
use std::borrow::Cow;

// 将主题静态资源打包进二进制（主题的 public 目录，包含 static 子目录）
#[derive(rust_embed::RustEmbed)]
#[folder = "themes/default/public"]
pub struct ThemeStaticAssets;

// 将主题模板打包进二进制
#[derive(rust_embed::RustEmbed)]
#[folder = "themes/default/templates"]
pub struct ThemeTemplates;

/// 运行时路径信息
#[derive(Debug, Clone)]
pub struct RuntimePaths {
    pub build_toml_path: std::path::PathBuf,
    pub theme_dir: std::path::PathBuf,
    pub theme_templates_dir: std::path::PathBuf,
    pub theme_static_dir: std::path::PathBuf,
}

/// 运行时路径构建器（Builder 模式）
#[derive(Debug, Default, Clone)]
pub struct RuntimePathsBuilder {
    md_dir: Option<std::path::PathBuf>,
    theme_name: Option<String>,
}

impl RuntimePathsBuilder {
    pub fn new() -> Self { Self::default() }
    pub fn md_dir<P: AsRef<std::path::Path>>(mut self, md_dir: P) -> Self {
        self.md_dir = Some(md_dir.as_ref().to_path_buf());
        self
    }
    pub fn theme_name<S: Into<String>>(mut self, name: S) -> Self {
        self.theme_name = Some(name.into());
        self
    }
    pub fn build(self) -> RuntimePaths {
        let md = self.md_dir.unwrap_or_else(|| std::path::PathBuf::from("."));
        let theme = self.theme_name.unwrap_or_else(|| "default".to_string());
        // build.toml 路径优先项目根，其次 md_dir
        let build_toml_path = {
            let root = std::path::PathBuf::from("build.toml");
            if root.exists() { root } else { md.join("build.toml") }
        };

        // 优先使用项目根目录 themes/<theme>
        let theme_dir_in_root = std::path::PathBuf::from("themes").join(&theme);
        // 次选在 md_dir 下 themes/<theme>
        let theme_dir_in_md = md.join("themes").join(&theme);

        // 选择存在的主题目录
        let (theme_dir, theme_templates_dir, theme_static_dir) = if theme_dir_in_root.exists() {
            let templates_dir = theme_dir_in_root.join("templates");
            // 优先主题 public 目录（例如 themes/default/public/static），否则回退到 themes/default/static
            let public_dir = theme_dir_in_root.join("public");
            let static_dir = if public_dir.exists() { public_dir } else { theme_dir_in_root.join("static") };
            (theme_dir_in_root, templates_dir, static_dir)
        } else if theme_dir_in_md.exists() {
            let templates_dir = theme_dir_in_md.join("templates");
            let public_dir = theme_dir_in_md.join("public");
            let static_dir = if public_dir.exists() { public_dir } else { theme_dir_in_md.join("static") };
            (theme_dir_in_md, templates_dir, static_dir)
        } else {
            // 如果都不存在，默认回退到根目录路径
            let templates_dir = theme_dir_in_root.join("templates");
            let public_dir = theme_dir_in_root.join("public");
            let static_dir = if public_dir.exists() { public_dir } else { theme_dir_in_root.join("static") };
            (theme_dir_in_root, templates_dir, static_dir)
        };

        RuntimePaths { build_toml_path, theme_dir, theme_templates_dir, theme_static_dir }
    }
}

/// 解析 build.toml 的读取路径：优先项目根目录，其次 md_dir 下
pub fn resolve_build_toml_path_read<P: AsRef<std::path::Path>>(md_dir: P) -> std::path::PathBuf {
    let root = std::path::PathBuf::from("build.toml");
    let in_md = md_dir.as_ref().join("build.toml");
    if root.exists() { root } else { in_md }
}

/// 解析 build.toml 的写入路径：优先写到项目根目录；若根不存在且 md_dir 已存在历史文件，则写回 md_dir
pub fn resolve_build_toml_path_write<P: AsRef<std::path::Path>>(md_dir: P) -> std::path::PathBuf {
    let root = std::path::PathBuf::from("build.toml");
    let in_md = md_dir.as_ref().join("build.toml");
    // 若根已存在，或 md_dir 不存在该文件，则写根；否则保持写回 md_dir 以兼容旧项目
    if root.exists() || !in_md.exists() { root } else { in_md }
}

/// 递归复制目录
pub fn copy_dir_recursive<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Result<()> {
    let src_path = src.as_ref();
    let dst_path = dst.as_ref();
    
    if !src_path.exists() {
        return Ok(());
    }
    
    for entry in WalkDir::new(src_path).into_iter().filter_map(|e| e.ok()) {
        let src_file = entry.path();
        let relative_path = src_file.strip_prefix(src_path)
            .map_err(|e| Error::Other(format!("无法获取相对路径 {:?}: {}", src_file, e)))?;
        let dst_file = dst_path.join(relative_path);
        
        if src_file.is_dir() {
            std::fs::create_dir_all(&dst_file)
                .map_err(|e| Error::Other(format!("无法创建目录 {:?}: {}", dst_file, e)))?;
        } else {
            if let Some(parent) = dst_file.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| Error::Other(format!("无法创建父目录 {:?}: {}", parent, e)))?;
            }
            std::fs::copy(src_file, &dst_file)
                .map_err(|e| Error::Other(format!("无法复制文件 {:?} -> {:?}: {}", src_file, dst_file, e)))?;
        }
    }
    
    Ok(())
}

/// 读取模板文件
pub fn read_template_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();
    std::fs::read_to_string(path)
        .map_err(|e| Error::Other(format!("无法读取模板文件 {:?}: {}", path, e)))
}

/// 简单的HTML标签移除函数
pub fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    
    // 清理多余的空白字符
    result.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// 获取 npm 命令名称（跨平台）
pub fn get_npm_command() -> &'static str {
    if cfg!(target_os = "windows") {
        "npm.cmd"
    } else {
        "npm"
    }
}

/// 记录编译信息到 build.toml 文件（优先项目根）
pub fn log_build_info<P: AsRef<std::path::Path>>(md_dir: P) -> Result<()> {
    use chrono::{DateTime, Local};
    use std::path::Path;
    
    // 获取当前时间
    let now: DateTime<Local> = Local::now();
    let beijing_time = now.format("%Y-%m-%d %H:%M:%S").to_string();

    // 读取已有的 build.toml 并更新 last_build_time，保留其他键
    let build_path = resolve_build_toml_path_write(md_dir.as_ref());
    let mut root = if build_path.exists() {
        match std::fs::read_to_string(&build_path) {
            Ok(content) => content.parse::<toml::Value>().unwrap_or(toml::Value::Table(toml::value::Table::new())),
            Err(_) => toml::Value::Table(toml::value::Table::new()),
        }
    } else {
        toml::Value::Table(toml::value::Table::new())
    };

    // 设置 last_build_time
    if let toml::Value::Table(ref mut table) = root {
        table.insert("last_build_time".to_string(), toml::Value::String(beijing_time));
    }

    // 写回 build.toml
    let toml_str = toml::to_string(&root).map_err(|e| Error::Other(format!("序列化 build.toml 失败: {}", e)))?;
    std::fs::write(&build_path, toml_str).map_err(|e| Error::Other(format!("写入 build.toml 失败: {}", e)))?;
    println!("编译信息已更新到: {}", build_path.display());
    Ok(())
}

/// 构建主题 CSS（按配置动态选择主题目录，位于 md_dir/themes/{theme}）
pub fn build_theme_css<P: AsRef<std::path::Path>>(md_dir: P, config: &Config) -> Result<()> {
    let paths = RuntimePathsBuilder::new()
        .md_dir(md_dir.as_ref())
        .theme_name(config.theme_name())
        .build();
    let theme_dir = paths.theme_dir;
    let package_json_path = format!("{}/package.json", theme_dir.display());
    
    // 检查主题是否需要 CSS 编译
    if !std::path::Path::new(&package_json_path).exists() {
        println!("主题不需要 CSS 编译，跳过...");
        return Ok(());
    }
    
    println!("检测到主题需要 CSS 编译，正在构建...");
    
    let npm_cmd = get_npm_command();
    
    // 检查是否安装了依赖
    let node_modules_path = format!("{}/node_modules", theme_dir.display());
    if !std::path::Path::new(&node_modules_path).exists() {
        println!("正在安装主题依赖...");
        let install_status = std::process::Command::new(npm_cmd)
            .args(&["install"]) 
            .current_dir(&theme_dir)
            .status()
            .map_err(|e| Error::Other(format!("无法执行 npm install 命令: {}", e)))?;
        
        if !install_status.success() {
            return Err(Error::Other("npm install 失败".to_string()));
        }
        println!("主题依赖安装完成");
    }
    
    // 运行 CSS 构建命令
    println!("正在编译主题 CSS...");
    let build_status = std::process::Command::new(npm_cmd)
        .args(&["run", "build-css"]) 
        .current_dir(&theme_dir)
        .status()
        .map_err(|e| Error::Other(format!("无法执行 npm run build-css 命令: {}", e)))?;
    
    if !build_status.success() {
        return Err(Error::Other("CSS 构建失败".to_string()));
    }
    
    println!("主题 CSS 编译完成");
    Ok(())
}

/// 计算并确保首次生成侧边栏数据到 build.toml（如果缺失，优先项目根）
pub fn ensure_sidebar_data<P: AsRef<std::path::Path>>(md_dir: P, posts: &[Post]) -> Result<()> {
    let build_path = resolve_build_toml_path_write(md_dir.as_ref());
    let mut root = if build_path.exists() {
        match std::fs::read_to_string(&build_path) {
            Ok(content) => content.parse::<toml::Value>().unwrap_or(toml::Value::Table(toml::value::Table::new())),
            Err(_) => toml::Value::Table(toml::value::Table::new()),
        }
    } else {
        toml::Value::Table(toml::value::Table::new())
    };

    let sidebar_missing = match &root {
        toml::Value::Table(t) => !t.contains_key("sidebar"),
        _ => true,
    };

    if !sidebar_missing {
        return Ok(());
    }

    // 计算热门文章（按日期倒序，取前10）
    let mut sorted_posts: Vec<&Post> = posts.iter().collect();
    sorted_posts.sort_by(|a, b| {
        let da = a.date().unwrap_or("");
        let db = b.date().unwrap_or("");
        db.cmp(da)
    });
    let hot_posts: Vec<toml::Value> = sorted_posts
        .into_iter()
        .take(10)
        .map(|p| {
            let mut item = toml::value::Table::new();
            if let Some(slug) = p.slug() { item.insert("slug".to_string(), toml::Value::String(slug.to_string())); }
            if let Some(title) = p.title() { item.insert("title".to_string(), toml::Value::String(title.to_string())); }
            if let Some(date) = p.date() { item.insert("date_ymd".to_string(), toml::Value::String(date.to_string())); }
            let cats = p.categories();
            if !cats.is_empty() {
                item.insert("categories".to_string(), toml::Value::Array(cats.into_iter().map(toml::Value::String).collect()));
            }
            toml::Value::Table(item)
        })
        .collect();

    // 计算热门标签（按出现次数，取前20）
    let all_tags = PostParser::collect_tags(posts);
    let hot_tags: Vec<toml::Value> = all_tags
        .into_iter()
        .take(20)
        .map(|v| {
            let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("").to_string();
            let count = v.get("count").and_then(|x| x.as_i64()).unwrap_or(0);
            let mut item = toml::value::Table::new();
            item.insert("name".to_string(), toml::Value::String(name));
            item.insert("count".to_string(), toml::Value::Integer(count));
            toml::Value::Table(item)
        })
        .collect();

    // 计算热门分类（按出现次数，取前8，顶层名统计）
    let mut category_count: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for post in posts {
        let cats = post.categories();
        if let Some(top) = cats.first() {
            *category_count.entry(top.clone()).or_insert(0) += 1;
        }
    }
    let mut cats_sorted: Vec<(String, i64)> = category_count.into_iter().collect();
    cats_sorted.sort_by(|a, b| b.1.cmp(&a.1));
    let hot_categories: Vec<toml::Value> = cats_sorted
        .into_iter()
        .take(8)
        .map(|(name, count)| {
            let mut item = toml::value::Table::new();
            item.insert("name".to_string(), toml::Value::String(name));
            item.insert("count".to_string(), toml::Value::Integer(count));
            toml::Value::Table(item)
        })
        .collect();

    // 写入到 build.toml 的 sidebar
    let mut sidebar = toml::value::Table::new();
    sidebar.insert("hot_posts".to_string(), toml::Value::Array(hot_posts));
    sidebar.insert("hot_tags".to_string(), toml::Value::Array(hot_tags));
    sidebar.insert("hot_categories".to_string(), toml::Value::Array(hot_categories));

    if let toml::Value::Table(ref mut table) = root {
        table.insert("sidebar".to_string(), toml::Value::Table(sidebar));
    }

    let toml_str = toml::to_string(&root).map_err(|e| Error::Other(format!("序列化 build.toml 失败: {}", e)))?;
    std::fs::write(&build_path, toml_str).map_err(|e| Error::Other(format!("写入 build.toml 失败: {}", e)))?;
    println!("已生成侧边栏数据到 {}（可手动修改）", build_path.display());
    Ok(())
}

/// 使用当前内容重新生成并覆盖 build.toml 中的侧边栏数据（优先项目根）
pub fn regenerate_sidebar<P: AsRef<std::path::Path>>(md_dir: P, posts: &[Post]) -> Result<()> {
    // 简单复用 ensure_sidebar_data 的逻辑：删除现有 sidebar 后重新生成
    let build_path = resolve_build_toml_path_write(md_dir.as_ref());
    let mut root = if build_path.exists() {
        match std::fs::read_to_string(&build_path) {
            Ok(content) => content.parse::<toml::Value>().unwrap_or(toml::Value::Table(toml::value::Table::new())),
            Err(_) => toml::Value::Table(toml::value::Table::new()),
        }
    } else {
        toml::Value::Table(toml::value::Table::new())
    };

    if let toml::Value::Table(ref mut table) = root {
        table.remove("sidebar");
    }
    std::fs::write(&build_path, toml::to_string(&root).unwrap_or_default())
        .map_err(|e| Error::Other(format!("更新 build.toml 失败: {}", e)))?;
    ensure_sidebar_data(md_dir, posts)
}

// 构建模式：增量或全量
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildMode {
    Incremental,
    Full,
}

/// 从 build.toml 读取编译模式；默认增量（优先项目根，其次 md_dir）
pub fn read_build_mode<P: AsRef<std::path::Path>>(md_dir: P) -> BuildMode {
    let build_path = resolve_build_toml_path_read(md_dir.as_ref());
    if !build_path.exists() {
        return BuildMode::Incremental;
    }
    let content = match std::fs::read_to_string(&build_path) {
        Ok(s) => s,
        Err(_) => return BuildMode::Incremental,
    };
    let value: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(_) => return BuildMode::Incremental,
    };
    let read_string_mode = |s: &str| match s.to_lowercase().as_str() {
        "full" | "normal" | "all" => BuildMode::Full,
        _ => BuildMode::Incremental,
    };
    if let toml::Value::Table(tbl) = value {
        if let Some(v) = tbl.get("compile_mode").or_else(|| tbl.get("build_mode")) {
            match v {
                toml::Value::String(s) => return read_string_mode(s),
                toml::Value::Boolean(b) => return if *b { BuildMode::Incremental } else { BuildMode::Full },
                _ => {}
            }
        }
        if let Some(v) = tbl.get("incremental") {
            if let Some(b) = v.as_bool() {
                return if b { BuildMode::Incremental } else { BuildMode::Full };
            }
        }
    }
    BuildMode::Incremental
}

/// 复制源目录根层的非 Markdown 且非隐藏文件到输出目录（用于拷贝 CNAME 等）
pub fn copy_root_non_md_non_hidden<P: AsRef<Path>, Q: AsRef<Path>>(md_dir: P, output_dir: Q) -> Result<()> {
    use std::fs;
    let md_dir = md_dir.as_ref();
    let output_dir = output_dir.as_ref();
    if !md_dir.exists() { return Ok(()); }
    let rd = fs::read_dir(md_dir)
        .map_err(|e| Error::Other(format!("无法读取源目录 {:?}: {}", md_dir, e)))?;
    for entry in rd.flatten() {
        let path = entry.path();
        // 仅处理根层文件
        if path.is_file() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with('.') { continue; }
            if path.extension().map_or(false, |ext| ext == "md") { continue; }
            let dst = output_dir.join(name_str.as_ref());
            fs::copy(&path, &dst)
                .map_err(|e| Error::Other(format!("无法复制文件 {:?} -> {:?}: {}", path, dst, e)))?;
        }
    }
    Ok(())
}

/// 将打包在二进制中的主题静态资源写出到输出目录（覆盖写出）
pub fn write_embedded_theme_static<P: AsRef<Path>>(output_dir: P) -> Result<()> {
    use std::fs;
    let output_dir = output_dir.as_ref();
    if !output_dir.exists() { fs::create_dir_all(output_dir).map_err(|e| Error::Other(format!("无法创建输出目录 {:?}: {}", output_dir, e)))?; }

    for file in ThemeStaticAssets::iter() {
        let rel: &str = file.as_ref();
        if let Some(content) = ThemeStaticAssets::get(rel) {
            let bytes: Cow<'static, [u8]> = content.data;
            let dst = output_dir.join(rel);
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent).map_err(|e| Error::Other(format!("无法创建父目录 {:?}: {}", parent, e)))?;
            }
            fs::write(&dst, &bytes).map_err(|e| Error::Other(format!("无法写入嵌入静态文件 {:?}: {}", dst, e)))?;
        }
    }
    Ok(())
}
