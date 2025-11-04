//! 工具函数模块
//! 
//! 提供各种实用的辅助函数

use crate::error::{Error, Result};
use crate::post::{Post, PostParser};
use std::path::Path;
use walkdir::WalkDir;

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

/// 记录编译信息到 md_dir/build.toml 文件
pub fn log_build_info<P: AsRef<std::path::Path>>(md_dir: P) -> Result<()> {
    use chrono::{DateTime, Local};
    use std::path::Path;
    
    // 获取当前时间
    let now: DateTime<Local> = Local::now();
    let beijing_time = now.format("%Y-%m-%d %H:%M:%S").to_string();

    // 读取已有的 build.toml 并更新 last_build_time，保留其他键
    let build_path = Path::new(md_dir.as_ref()).join("build.toml");
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

/// 构建主题 CSS
pub fn build_theme_css() -> Result<()> {
    let theme_dir = "src/themes/default";
    let package_json_path = format!("{}/package.json", theme_dir);
    
    // 检查主题是否需要 CSS 编译
    if !std::path::Path::new(&package_json_path).exists() {
        println!("主题不需要 CSS 编译，跳过...");
        return Ok(());
    }
    
    println!("检测到主题需要 CSS 编译，正在构建...");
    
    let npm_cmd = get_npm_command();
    
    // 检查是否安装了依赖
    let node_modules_path = format!("{}/node_modules", theme_dir);
    if !std::path::Path::new(&node_modules_path).exists() {
        println!("正在安装主题依赖...");
        let install_status = std::process::Command::new(npm_cmd)
            .args(&["install"])
            .current_dir(theme_dir)
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
        .current_dir(theme_dir)
        .status()
        .map_err(|e| Error::Other(format!("无法执行 npm run build-css 命令: {}", e)))?;
    
    if !build_status.success() {
        return Err(Error::Other("CSS 构建失败".to_string()));
    }
    
    println!("主题 CSS 编译完成");
    Ok(())
}

/// 计算并确保首次生成侧边栏数据到 md_dir/build.toml（如果缺失）
pub fn ensure_sidebar_data<P: AsRef<std::path::Path>>(md_dir: P, posts: &[Post]) -> Result<()> {
    use std::path::Path;
    let build_path = Path::new(md_dir.as_ref()).join("build.toml");
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

/// 使用当前内容重新生成并覆盖 md_dir/build.toml 中的侧边栏数据
pub fn regenerate_sidebar<P: AsRef<std::path::Path>>(md_dir: P, posts: &[Post]) -> Result<()> {
    // 简单复用 ensure_sidebar_data 的逻辑：删除现有 sidebar 后重新生成
    use std::path::Path;
    let build_path = Path::new(md_dir.as_ref()).join("build.toml");
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

/// 从 source/build.toml 读取编译模式；默认增量
pub fn read_build_mode<P: AsRef<std::path::Path>>(md_dir: P) -> BuildMode {
    use std::path::Path;
    let build_path = Path::new(md_dir.as_ref()).join("build.toml");
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
