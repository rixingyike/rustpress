//! 工具函数模块
//!
//! 提供各种实用的辅助函数

use crate::config::Config;
use crate::error::{Error, Result};
use crate::post::{Post, PostParser};
use std::borrow::Cow;
use std::path::Path;
use walkdir::WalkDir;
// 使派生的 RustEmbed trait 在作用域内，从而可调用 ::get()
use rust_embed::RustEmbed;
// 将根目录的配置编译进二进制，作为默认模板来源
pub const EMBEDDED_ROOT_CONFIG_TOML: &str = include_str!("../config.toml.example");
pub const EMBEDDED_ROOT_BUILD_TOML: &str = include_str!("../source/build.toml");
pub const EMBEDDED_DEPLOY_YML: &str = include_str!("../deploy.yml.example.yml");

// 将主题静态资源打包进二进制（主题的 public 目录，包含 static 子目录）
#[derive(RustEmbed)]
#[folder = "themes/default/public"]
pub struct ThemeStaticAssets;

// 将主题模板打包进二进制
#[derive(RustEmbed)]
#[folder = "themes/default/templates"]
pub struct ThemeTemplates;

// 将默认页面（home.md/about.md/friends.md）打包进二进制
#[derive(RustEmbed)]
#[folder = "themes/default/pages"]
pub struct DefaultPages;

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
    root_dir: Option<std::path::PathBuf>,
    theme_name: Option<String>,
}

impl RuntimePathsBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn md_dir<P: AsRef<std::path::Path>>(mut self, md_dir: P) -> Self {
        self.md_dir = Some(md_dir.as_ref().to_path_buf());
        self
    }
    pub fn theme_name<S: Into<String>>(mut self, name: S) -> Self {
        self.theme_name = Some(name.into());
        self
    }
    pub fn root_dir<P: AsRef<std::path::Path>>(mut self, root_dir: P) -> Self {
        self.root_dir = Some(root_dir.as_ref().to_path_buf());
        self
    }
    pub fn build(self) -> RuntimePaths {
        let theme = self.theme_name.unwrap_or_else(|| "default".to_string());
        // build.toml 路径：优先 md_dir，其次项目根
        let md_dir_for_resolve = self
            .md_dir
            .clone()
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let build_toml_path = resolve_build_toml_path_read(&md_dir_for_resolve);

        // 确定基础目录：优先使用 root_dir，否则使用 md_dir 的父目录，最后回退到当前目录
        let base_dir = if let Some(root) = self.root_dir {
            root
        } else if let Some(md) = &self.md_dir {
            md.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| std::path::PathBuf::from("."))
        } else {
            std::path::PathBuf::from(".")
        };

        // 优先使用项目根目录下的 themes/<theme>
        let theme_dir_in_root = base_dir.join("themes").join(&theme);
        // 兼容历史：若根目录不存在主题目录，可回退到 md_dir 下的 themes/<theme>
        let theme_dir_in_md = md_dir_for_resolve.join("themes").join(&theme);

        // 选择存在的主题目录
        let (theme_dir, theme_templates_dir, theme_static_dir) = if theme_dir_in_root.exists() {
            let templates_dir = theme_dir_in_root.join("templates");
            // 优先主题 public 目录（例如 themes/default/public/static），否则回退到 themes/default/static
            let public_dir = theme_dir_in_root.join("public");
            let static_dir = if public_dir.exists() {
                public_dir
            } else {
                theme_dir_in_root.join("static")
            };
            (theme_dir_in_root, templates_dir, static_dir)
        } else if theme_dir_in_md.exists() {
            let templates_dir = theme_dir_in_md.join("templates");
            let public_dir = theme_dir_in_md.join("public");
            let static_dir = if public_dir.exists() {
                public_dir
            } else {
                theme_dir_in_md.join("static")
            };
            (theme_dir_in_md, templates_dir, static_dir)
        } else {
            // 如果都不存在，默认回退到根目录路径
            let templates_dir = theme_dir_in_root.join("templates");
            let public_dir = theme_dir_in_root.join("public");
            let static_dir = if public_dir.exists() {
                public_dir
            } else {
                theme_dir_in_root.join("static")
            };
            (theme_dir_in_root, templates_dir, static_dir)
        };

        RuntimePaths {
            build_toml_path,
            theme_dir,
            theme_templates_dir,
            theme_static_dir,
        }
    }
}

/// 解析 build.toml 的读取路径：优先 `md_dir/build.toml`，否则回退到项目根 `build.toml`
pub fn resolve_build_toml_path_read<P: AsRef<std::path::Path>>(md_dir: P) -> std::path::PathBuf {
    let md_build = md_dir.as_ref().join("build.toml");
    if md_build.exists() {
        md_build
    } else {
        std::path::PathBuf::from("build.toml")
    }
}

/// 解析 build.toml 的写入路径：
/// - 若 `md_dir/build.toml` 存在则写入该处；
/// - 若项目根存在 `build.toml` 则写入根；
/// - 若都不存在，选择在 `md_dir` 下创建（符合首次处理 source 的策略）。
pub fn resolve_build_toml_path_write<P: AsRef<std::path::Path>>(md_dir: P) -> std::path::PathBuf {
    let md_build = md_dir.as_ref().join("build.toml");
    if md_build.exists() {
        md_build
    } else {
        let root_build = std::path::PathBuf::from("build.toml");
        if root_build.exists() {
            root_build
        } else {
            md_build
        }
    }
}

/// 解析 config.toml 的读取路径：优先 `md_dir/<config_filename>`，否则回退到项目根 `<config_filename>`
pub fn resolve_config_toml_path_read<P: AsRef<std::path::Path>>(
    md_dir: P,
    config_filename: &str,
) -> std::path::PathBuf {
    let md_config = md_dir.as_ref().join(config_filename);
    if md_config.exists() {
        md_config
    } else {
        std::path::PathBuf::from(config_filename)
    }
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
        let relative_path = src_file
            .strip_prefix(src_path)
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
            std::fs::copy(src_file, &dst_file).map_err(|e| {
                Error::Other(format!(
                    "无法复制文件 {:?} -> {:?}: {}",
                    src_file, dst_file, e
                ))
            })?;
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

    // 获取当前时间
    let now: DateTime<Local> = Local::now();
    let beijing_time = now.format("%Y-%m-%d %H:%M:%S").to_string();

    // 读取已有的 build.toml 并更新 last_build_time，保留其他键
    let build_path = resolve_build_toml_path_write(md_dir.as_ref());
    let mut root = if build_path.exists() {
        match std::fs::read_to_string(&build_path) {
            Ok(content) => content
                .parse::<toml::Value>()
                .unwrap_or(toml::Value::Table(toml::value::Table::new())),
            Err(_) => toml::Value::Table(toml::value::Table::new()),
        }
    } else {
        toml::Value::Table(toml::value::Table::new())
    };

    // 设置 last_build_time
    if let toml::Value::Table(ref mut table) = root {
        table.insert(
            "last_build_time".to_string(),
            toml::Value::String(beijing_time),
        );
    }

    // 写回 build.toml
    let toml_str = toml::to_string(&root)
        .map_err(|e| Error::Other(format!("序列化 build.toml 失败: {}", e)))?;
    std::fs::write(&build_path, toml_str)
        .map_err(|e| Error::Other(format!("写入 build.toml 失败: {}", e)))?;
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
            Ok(content) => content
                .parse::<toml::Value>()
                .unwrap_or(toml::Value::Table(toml::value::Table::new())),
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
            if let Some(slug) = p.slug() {
                item.insert("slug".to_string(), toml::Value::String(slug.to_string()));
            }
            if let Some(title) = p.title() {
                item.insert("title".to_string(), toml::Value::String(title.to_string()));
            }
            if let Some(date) = p.date() {
                item.insert(
                    "date_ymd".to_string(),
                    toml::Value::String(date.to_string()),
                );
            }
            let cats = p.categories();
            if !cats.is_empty() {
                item.insert(
                    "categories".to_string(),
                    toml::Value::Array(cats.into_iter().map(toml::Value::String).collect()),
                );
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
            let name = v
                .get("name")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let count = v.get("count").and_then(|x| x.as_i64()).unwrap_or(0);
            let mut item = toml::value::Table::new();
            item.insert("name".to_string(), toml::Value::String(name));
            item.insert("count".to_string(), toml::Value::Integer(count));
            toml::Value::Table(item)
        })
        .collect();

    // 计算热门分类（按出现次数，取前8，顶层名统计）
    let mut category_count: std::collections::HashMap<String, i64> =
        std::collections::HashMap::new();
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
    sidebar.insert(
        "hot_categories".to_string(),
        toml::Value::Array(hot_categories),
    );

    if let toml::Value::Table(ref mut table) = root {
        table.insert("sidebar".to_string(), toml::Value::Table(sidebar));
    }

    let toml_str = toml::to_string(&root)
        .map_err(|e| Error::Other(format!("序列化 build.toml 失败: {}", e)))?;
    std::fs::write(&build_path, toml_str)
        .map_err(|e| Error::Other(format!("写入 build.toml 失败: {}", e)))?;
    println!("已生成侧边栏数据到 {}（可手动修改）", build_path.display());
    Ok(())
}

/// 使用当前内容重新生成并覆盖 build.toml 中的侧边栏数据（优先项目根）
pub fn regenerate_sidebar<P: AsRef<std::path::Path>>(md_dir: P, posts: &[Post]) -> Result<()> {
    // 简单复用 ensure_sidebar_data 的逻辑：删除现有 sidebar 后重新生成
    let build_path = resolve_build_toml_path_write(md_dir.as_ref());
    let mut root = if build_path.exists() {
        match std::fs::read_to_string(&build_path) {
            Ok(content) => content
                .parse::<toml::Value>()
                .unwrap_or(toml::Value::Table(toml::value::Table::new())),
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
        // full_compile=true 表示全量编译，false 表示增量编译
        if let Some(v) = tbl.get("full_compile") {
            if let Some(b) = v.as_bool() {
                return if b {
                    BuildMode::Full
                } else {
                    BuildMode::Incremental
                };
            }
        }
        // 兼容旧字段
        if let Some(v) = tbl.get("compile_mode").or_else(|| tbl.get("build_mode")) {
            match v {
                toml::Value::String(s) => return read_string_mode(s),
                toml::Value::Boolean(b) => {
                    return if *b {
                        BuildMode::Incremental
                    } else {
                        BuildMode::Full
                    };
                }
                _ => {}
            }
        }
        if let Some(v) = tbl.get("incremental") {
            if let Some(b) = v.as_bool() {
                return if b {
                    BuildMode::Incremental
                } else {
                    BuildMode::Full
                };
            }
        }
    }
    BuildMode::Incremental
}

/// 复制源目录根层的非 Markdown 且非隐藏文件到输出目录（用于拷贝 CNAME 等）
pub fn copy_root_non_md_non_hidden<P: AsRef<Path>, Q: AsRef<Path>>(
    md_dir: P,
    output_dir: Q,
) -> Result<()> {
    use std::fs;
    let md_dir = md_dir.as_ref();
    let output_dir = output_dir.as_ref();
    if !md_dir.exists() {
        return Ok(());
    }
    let rd = fs::read_dir(md_dir)
        .map_err(|e| Error::Other(format!("无法读取源目录 {:?}: {}", md_dir, e)))?;
    for entry in rd.flatten() {
        let path = entry.path();
        // 仅处理根层文件
        if path.is_file() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with('.') {
                continue;
            }
            if path.extension().map_or(false, |ext| ext == "md") {
                continue;
            }
            let dst = output_dir.join(name_str.as_ref());
            fs::copy(&path, &dst).map_err(|e| {
                Error::Other(format!("无法复制文件 {:?} -> {:?}: {}", path, dst, e))
            })?;
        }
    }
    Ok(())
}

/// 递归复制 `md_dir` 下的所有非 Markdown 且非隐藏文件到 `output_dir`
///
/// 复制行为：
/// 1. 默认保持源码相对路径（如 source/columns/rustpress/assets/img.png -> public/columns/rustpress/assets/img.png）
/// 2. 若文件位于分类法目录（如 columns, projects, works, friends, tweets），
///    且该分类法配置了别名路径（如 columns -> /c, projects -> /p 等），
///    同步复制一份到别名路径（如 public/c/rustpress/assets/img.png），
///    确保 Markdown 中使用相对路径（如 assets/img.png）在编译后的各层级页面中均能正确加载。
pub fn copy_non_md_recursive_preserve_paths<P: AsRef<Path>, Q: AsRef<Path>>(
    md_dir: P,
    output_dir: Q,
    taxonomies: Option<&crate::config::TaxonomiesConfig>,
) -> Result<()> {
    use std::fs;
    let md_dir = md_dir.as_ref();
    let output_dir = output_dir.as_ref();
    if !md_dir.exists() {
        return Ok(());
    }
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)
            .map_err(|e| Error::Other(format!("无法创建输出目录 {:?}: {}", output_dir, e)))?;
    }

    let default_taxonomies;
    let tax_cfg = match taxonomies {
        Some(t) => t,
        None => {
            default_taxonomies = crate::post::PostParser::resolve_taxonomies_from_dir(md_dir);
            &default_taxonomies
        }
    };

    for entry in WalkDir::new(md_dir).into_iter().filter_map(|e| e.ok()) {
        let src_path = entry.path();
        if src_path.is_file() {
            // 跳过隐藏文件（文件名以 '.' 开头）
            let name = src_path
                .file_name()
                .map(|s| s.to_string_lossy())
                .unwrap_or(std::borrow::Cow::Borrowed(""));
            if name.starts_with('.') {
                continue;
            }
            // 跳过 Markdown 文件
            if src_path.extension().map_or(false, |ext| ext == "md") {
                continue;
            }

            // 计算相对路径并复制
            let rel = src_path
                .strip_prefix(md_dir)
                .map_err(|e| Error::Other(format!("无法计算相对路径 {:?}: {}", src_path, e)))?;
            let dst_path = output_dir.join(rel);
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| Error::Other(format!("无法创建父目录 {:?}: {}", parent, e)))?;
            }
            fs::copy(src_path, &dst_path).map_err(|e| {
                Error::Other(format!(
                    "无法复制文件 {:?} -> {:?}: {}",
                    src_path, dst_path, e
                ))
            })?;

            // 映射并复制到分类法别名路径
            let mut components = rel.components();
            if let Some(std::path::Component::Normal(first_os)) = components.next() {
                if let Some(top_cat) = first_os.to_str() {
                    let is_taxonomy = matches!(
                        top_cat,
                        "columns"
                            | "column"
                            | "projects"
                            | "project"
                            | "works"
                            | "work"
                            | "friends"
                            | "friend"
                            | "tweets"
                            | "tweet"
                            | "short"
                    );
                    if is_taxonomy {
                        let tax_prefix = tax_cfg.get_prefix(top_cat);
                        let tax_dir = tax_prefix.trim_matches('/');
                        let sub_path: std::path::PathBuf = components.as_path().to_path_buf();
                        if !sub_path.as_os_str().is_empty() {
                            let mapped_dst = if !tax_dir.is_empty() {
                                if tax_dir != top_cat {
                                    Some(output_dir.join(tax_dir).join(&sub_path))
                                } else {
                                    None
                                }
                            } else {
                                // tax_prefix 为根 "/"
                                Some(output_dir.join(&sub_path))
                            };

                            if let Some(mapped_dst) = mapped_dst {
                                if let Some(parent) = mapped_dst.parent() {
                                    fs::create_dir_all(parent).map_err(|e| {
                                        Error::Other(format!("无法创建父目录 {:?}: {}", parent, e))
                                    })?;
                                }
                                fs::copy(src_path, &mapped_dst).map_err(|e| {
                                    Error::Other(format!(
                                        "无法复制别名文件 {:?} -> {:?}: {}",
                                        src_path, mapped_dst, e
                                    ))
                                })?;
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// 将打包在二进制中的主题静态资源写出到输出目录（覆盖写出）
pub fn write_embedded_theme_static<P: AsRef<Path>>(output_dir: P) -> Result<()> {
    use std::fs;
    let output_dir = output_dir.as_ref();
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)
            .map_err(|e| Error::Other(format!("无法创建输出目录 {:?}: {}", output_dir, e)))?;
    }

    for file in ThemeStaticAssets::iter() {
        let rel: &str = file.as_ref();
        if let Some(content) = ThemeStaticAssets::get(rel) {
            let bytes: Cow<'static, [u8]> = content.data;
            let dst = output_dir.join(rel);
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| Error::Other(format!("无法创建父目录 {:?}: {}", parent, e)))?;
            }
            fs::write(&dst, &bytes)
                .map_err(|e| Error::Other(format!("无法写入嵌入静态文件 {:?}: {}", dst, e)))?;
        }
    }
    Ok(())
}

/// 将打包在二进制中的主题模板写出到项目根目录的 `themes/default/templates`（仅在缺失时写入）
pub fn write_embedded_theme_templates_to_root(root: &Path) -> Result<()> {
    use std::fs;
    let base = root.join("themes/default/templates");
    if !base.exists() {
        fs::create_dir_all(&base)
            .map_err(|e| Error::Other(format!("无法创建模板目录 {:?}: {}", base, e)))?;
    }

    for file in ThemeTemplates::iter() {
        let rel: &str = file.as_ref();
        if let Some(content) = ThemeTemplates::get(rel) {
            let bytes: Cow<'static, [u8]> = content.data;
            let dst = base.join(rel);
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| Error::Other(format!("无法创建父目录 {:?}: {}", parent, e)))?;
            }
            
            // 策略：如果文件不存在则写入；如果文件已存在但包含旧的逻辑，则强制覆盖以自愈
            let mut should_write = !dst.exists();
            if !should_write {
                if let Ok(existing_content) = fs::read_to_string(&dst) {
                    let outdated_patterns = [
                        "is starting_with",
                        "| starting_with",
                        "| contains(pat=",
                        "{% set is_img ="
                    ];
                    if outdated_patterns.iter().any(|p| existing_content.contains(p)) {
                        println!("[DEBUG] 检测到陈旧模板语法或重写逻辑，准备自愈覆盖: {:?}", dst);
                        should_write = true;
                    }
                }
            }

            if should_write {
                fs::write(&dst, &bytes)
                    .map_err(|e| Error::Other(format!("无法写入嵌入模板文件 {:?}: {}", dst, e)))?;
            }
        }
    }
    Ok(())
}

/// 将打包在二进制中的主题静态资源写出到项目根目录的 `themes/default/public`（仅在缺失时写入缺失文件）
pub fn write_embedded_theme_static_to_root(root: &Path) -> Result<()> {
    use std::fs;
    let base = root.join("themes/default/public");
    if !base.exists() {
        fs::create_dir_all(&base)
            .map_err(|e| Error::Other(format!("无法创建主题静态目录 {:?}: {}", base, e)))?;
    }

    for file in ThemeStaticAssets::iter() {
        let rel: &str = file.as_ref();
        if let Some(content) = ThemeStaticAssets::get(rel) {
            let bytes: Cow<'static, [u8]> = content.data;
            let dst = base.join(rel);
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| Error::Other(format!("无法创建父目录 {:?}: {}", parent, e)))?;
            }
            // 仅在文件不存在时写入，避免覆盖用户修改
            if !dst.exists() {
                fs::write(&dst, &bytes)
                    .map_err(|e| Error::Other(format!("无法写入嵌入静态文件 {:?}: {}", dst, e)))?;
            }
        }
    }
    Ok(())
}

/// 在项目根保障 `config.toml` 与 `build.toml` 存在：
/// - 若根不存在且 `md_dir` 下存在，则复制到根
/// - 若都不存在，则写入内嵌（编译进二进制）的根默认配置
pub fn ensure_root_config_and_build<P: AsRef<Path>>(
    md_dir: P,
    config_filename: &str,
) -> Result<()> {
    use std::fs;
    let md_dir = md_dir.as_ref();

    // 处理 config.toml
    let root_config = std::path::Path::new(config_filename);
    if !root_config.exists() {
        let md_config = md_dir.join(config_filename);
        if md_config.exists() {
            fs::copy(&md_config, &root_config).map_err(|e| {
                Error::Other(format!(
                    "复制配置文件失败 {:?} -> {:?}: {}",
                    md_config, root_config, e
                ))
            })?;
            println!("已从源目录复制配置到根: {}", root_config.display());
        } else {
            // 使用嵌入的根默认配置写出
            fs::write(&root_config, EMBEDDED_ROOT_CONFIG_TOML)
                .map_err(|e| Error::Other(format!("写入默认配置失败 {:?}: {}", root_config, e)))?;
            println!("已在根目录创建默认配置: {}", root_config.display());
        }
    }

    // 处理 build.toml
    let root_build = std::path::Path::new("build.toml");
    if !root_build.exists() {
        let md_build = md_dir.join("build.toml");
        if md_build.exists() {
            fs::copy(&md_build, &root_build).map_err(|e| {
                Error::Other(format!(
                    "复制构建文件失败 {:?} -> {:?}: {}",
                    md_build, root_build, e
                ))
            })?;
            println!("已从源目录复制构建配置到根: {}", root_build.display());
        } else {
            // 使用嵌入的根默认构建文件写出
            fs::write(&root_build, EMBEDDED_ROOT_BUILD_TOML).map_err(|e| {
                Error::Other(format!("写入默认构建文件失败 {:?}: {}", root_build, e))
            })?;
            println!("已在根目录创建默认构建文件: {}", root_build.display());
        }
    }

    Ok(())
}

/// 在 `md_dir` 目录下保障首页、关于、友链三类页面存在，缺失则补全示例文件（YAML front matter）
pub fn ensure_default_pages<P: AsRef<Path>>(md_dir: P) -> Result<()> {
    println!("[DEBUG] 正在同步补全示例页面到 {:?}", md_dir.as_ref());
    use std::fs;
    let md_dir = md_dir.as_ref();
    if !md_dir.exists() {
        fs::create_dir_all(md_dir)
            .map_err(|e| Error::Other(format!("无法创建源目录 {:?}: {}", md_dir, e)))?;
    }

    // 获取当前日期字符串
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    // 优先从嵌入资源写出，若缺失则回退到内置字符串
    let write_if_missing = |name: &str, fallback: &str| -> Result<()> {
        let path = md_dir.join(name);
        if path.exists() {
            return Ok(());
        }
        
        // 自动创建父目录
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| Error::Other(format!("无法创建目录 {:?}: {}", parent, e)))?;
            }
        }

        if let Some(file) = DefaultPages::get(name) {
            std::fs::write(&path, file.data)
                .map_err(|e| Error::Other(format!("写入嵌入默认页失败 {:?}: {}", path, e)))?;
        } else {
            std::fs::write(&path, fallback)
                .map_err(|e| Error::Other(format!("写入内置示例失败 {:?}: {}", path, e)))?;
        }
        println!("已生成示例: {}", path.display());
        Ok(())
    };

    // --- 骨架基础页 ---
    write_if_missing("home.md", r#"---
title: "首页"
layout: home
home_navs:
  - text: "关于我"
    emoji: "👤"
    url: "/about.html"
  - text: "友链"
    emoji: "🤝"
    url: "/friends/"
---

# 欢迎来到我的博客

这里是首页的自定义内容区域。你可以在此添加简介或导航按钮（通过 front matter 的 `home_navs` 字段）。
"#)?;

    write_if_missing("about.md", r#"---
title: "关于我"
layout: about
toc: true
---

# 关于我

这里写你的简介、技能、经历、联系方式等内容。TOC（目录）可根据内容自动生成。
"#)?;

    write_if_missing("friends.md", r#"---
title: "友链"
layout: friends
friends:
  - name: "Rust 官网"
    url: "https://www.rust-lang.org/"
    description: "Rust 编程语言"
  - name: "Crates.io"
    url: "https://crates.io/"
    description: "Rust 包管理平台"
---

# 友情链接

欢迎在此添加你的朋友站点或推荐网站。
"#)?;

    write_if_missing("docs.md", "---\ntitle: \"文档专栏\"\nlayout: docs\n---\n\n这里展示所有书籍和专栏。")?;
    write_if_missing("projects.md", "---\ntitle: \"开源项目\"\nlayout: projects\n---\n\n这里展示所有的开源项目。")?;

    // --- 各分类目录示例 (每样 2 个) ---
    
    // Blog 示例 (按年份组织，数字递增命名)
    let current_year = chrono::Local::now().format("%Y").to_string();
    write_if_missing(&format!("blog/{}/1.md", current_year), &format!("---\ntitle: \"欢迎来到我的博客\"\ndate: \"{}\"\ncategories: [\"生活\"]\n---\n\n这是一个按年份组织的博客示例文章。", today))?;
    write_if_missing(&format!("blog/{}/2.md", current_year), &format!("---\ntitle: \"技术分享：如何使用 Rust\"\ndate: \"{}\"\ncategories: [\"技术\"]\n---\n\n这是一篇按数字递增命名的技术博客示例。", today))?;

    // Projects 示例
    write_if_missing("projects/my-first-project.md", "---\ntitle: \"我的第一个项目\"\nlayout: project\ndescription: \"这是一个使用 Rust 编写的开源项目\"\nicon: \"🦀\"\nversion: \"0.1.0\"\ntags: [\"Rust\", \"Open Source\"]\n---\n\n项目详细介绍内容。")?;
    write_if_missing("projects/rustpress.md", "---\ntitle: \"RustPress\"\nlayout: project\ndescription: \"静态网站生成器\"\nicon: \"🚀\"\nversion: \"0.1.28\"\n---\n\n这是 RustPress 项目本身的示例展示。")?;

    // Docs (书籍) 示例 - 增加章节嵌套和缩进体现
    write_if_missing("docs/guide/README.md", "---\ntitle: \"用户指南\"\nlayout: doc\n---\n\n欢迎阅读本专栏手册。")?;
    write_if_missing("docs/guide/1.安装与配置.md", "---\ntitle: \"1. 安装与配置\"\n---\n\n这是专栏的第一个章节。")?;
    write_if_missing("docs/guide/1.1.快速开始.md", "---\ntitle: \"1.1. 快速开始\"\n---\n\n这是二级章节示例。")?;
    write_if_missing("docs/guide/1.2.配置文件说明.md", "---\ntitle: \"1.2. 配置文件说明\"\n---\n\n详细介绍 config.toml 内容。")?;
    write_if_missing("docs/guide/1.2.1.站点信息配置.md", "---\ntitle: \"1.2.1. 站点信息配置\"\n---\n\n这是三级章节展示缩进。")?;

    write_if_missing("docs/novel/README.md", "---\ntitle: \"科幻小说集\"\nlayout: doc\n---\n\n这是另一个文档专栏的示例。")?;
    write_if_missing("docs/novel/1.第一章：启程.md", "---\ntitle: \"1. 第一章：启程\"\n---\n\n小说开篇内容...")?;
    write_if_missing("docs/novel/2.第二章：星辰.md", "---\ntitle: \"2. 第二章：星辰\"\n---\n\n茫茫宇宙中...")?;

    Ok(())
}

/// 初始化博客源目录骨架
pub fn init_source_dir<P: AsRef<Path>>(md_dir: P, config_filename: &str) -> Result<()> {
    let md_dir = md_dir.as_ref();
    
    // 1. 创建标准子目录
    let dirs = ["blog", "docs", "projects", "assets"];
    for dir in dirs {
        let path = md_dir.join(dir);
        if !path.exists() {
            std::fs::create_dir_all(&path)
                .map_err(|e| Error::Other(format!("无法创建子目录 {:?}: {}", path, e)))?;
        }
    }

    // 2. 调用标准初始化流程（补全配置、模板、静态资源）
    ensure_initial_setup(md_dir, config_filename)?;
    
    // 3. 显式补全示例页面 (只在 init 时执行，防止 serve 时重生已删除的示例)
    ensure_default_pages(md_dir)?;

    // 4. 生成 GitHub Action Workflow
    // 识别项目根目录（md_dir 的父目录，若无则为当前目录）
    let (root_dir, md_rel_name) = if let Some(parent) = md_dir.parent() {
        if parent.as_os_str().is_empty() {
             (Path::new("."), md_dir.file_name().and_then(|n| n.to_str()).unwrap_or("source"))
        } else {
             (parent, md_dir.file_name().and_then(|n| n.to_str()).unwrap_or("source"))
        }
    } else {
        (Path::new("."), ".")
    };

    let workflow_dir = root_dir.join(".github/workflows");
    if !workflow_dir.exists() {
        std::fs::create_dir_all(&workflow_dir)
            .map_err(|e| Error::Other(format!("无法创建目录 {:?}: {}", workflow_dir, e)))?;
    }
    
    let deploy_yml_path = workflow_dir.join("deploy.yml");
    if !deploy_yml_path.exists() {
        // 定制化 deploy.yml：替换默认的 -m source 为实际的目录名
        let deploy_content = EMBEDDED_DEPLOY_YML.replace(
            "rustpress -m source build -o public",
            &format!("rustpress -m {} build -o public", md_rel_name)
        );
        
        std::fs::write(&deploy_yml_path, deploy_content)
            .map_err(|e| Error::Other(format!("无法写入部署脚本 {:?}: {}", deploy_yml_path, e)))?;
        println!("已生成 GitHub Action 部署脚本: {}", deploy_yml_path.display());
    }

    // 4. 生成 .gitignore
    let gitignore_path = root_dir.join(".gitignore");
    if !gitignore_path.exists() {
        let gitignore_content = "node_modules\n.DS_Store\n.pushpen/\npublic\n";
        std::fs::write(&gitignore_path, gitignore_content)
            .map_err(|e| Error::Other(format!("无法写入 .gitignore: {}", e)))?;
        println!("已生成 .gitignore 文件: {}", gitignore_path.display());
    }

    println!("✅ 博客项目初始化完成（包含骨架、示例内容、GitHub Action 及 .gitignore）");
    Ok(())
}

/// 在源目录 md_dir 保障 `config.toml` 与 `build.toml` 存在：
/// - 若 md_dir 下不存在且项目根存在，则复制到 md_dir
/// - 若都不存在，则在 md_dir 写入内嵌（编译进二进制）的根默认配置
pub fn ensure_source_config_and_build<P: AsRef<Path>>(
    md_dir: P,
    config_filename: &str,
) -> Result<()> {
    use std::fs;
    let md_dir = md_dir.as_ref();

    if !md_dir.exists() {
        fs::create_dir_all(md_dir)
            .map_err(|e| Error::Other(format!("无法创建源目录 {:?}: {}", md_dir, e)))?;
    }

    // 处理 config.toml（优先生成到 source）
    let md_config = md_dir.join(config_filename);
    if !md_config.exists() {
        let root_config = std::path::Path::new(config_filename);
        if root_config.exists() {
            fs::copy(&root_config, &md_config).map_err(|e| {
                Error::Other(format!(
                    "复制配置文件失败 {:?} -> {:?}: {}",
                    root_config, md_config, e
                ))
            })?;
            println!("已从根目录复制配置到源目录: {}", md_config.display());
        } else {
            // 使用嵌入的根默认配置写出到源目录
            fs::write(&md_config, EMBEDDED_ROOT_CONFIG_TOML)
                .map_err(|e| Error::Other(format!("写入默认配置失败 {:?}: {}", md_config, e)))?;
            println!("已在源目录创建默认配置: {}", md_config.display());
        }
    }

    // 处理 build.toml（优先生成到 source）
    let md_build = md_dir.join("build.toml");
    if !md_build.exists() {
        let root_build = std::path::Path::new("build.toml");
        if root_build.exists() {
            fs::copy(&root_build, &md_build).map_err(|e| {
                Error::Other(format!(
                    "复制构建文件失败 {:?} -> {:?}: {}",
                    root_build, md_build, e
                ))
            })?;
            println!("已从根目录复制构建配置到源目录: {}", md_build.display());
        } else {
            // 使用嵌入的根默认构建文件写出到源目录
            fs::write(&md_build, EMBEDDED_ROOT_BUILD_TOML)
                .map_err(|e| Error::Other(format!("写入默认构建文件失败 {:?}: {}", md_build, e)))?;
            println!("已在源目录创建默认构建文件: {}", md_build.display());
        }
    }

    Ok(())
}

/// 启动时初始化：在源目录补全 config.toml 与 build.toml；在项目根写出主题资源
/// 注意：此函数已不再生成示例页面（home/about 等），以防止 serve 时重生已删除文件。
pub fn ensure_initial_setup<P: AsRef<Path>>(md_dir: P, config_filename: &str) -> Result<()> {
    let md_path = md_dir.as_ref();
    let root_path = md_path.parent().unwrap_or(Path::new("."));
    
    println!("[DEBUG] 进入 ensure_initial_setup，md_dir: {:?}, root: {:?}", md_path, root_path);
    // 1) 不再在项目根保障配置与构建文件，统一使用源目录（md_dir）下的配置文件
    // ensure_root_config_and_build(md_path, config_filename)?;
    // 2) 在源目录保障配置与构建文件（若缺失，写出内嵌默认或从根复制）
    ensure_source_config_and_build(md_path, config_filename)?;
    // 3) 写出嵌入的主题模板与静态资源到根 themes（缺失或过时时生成/更新）
    write_embedded_theme_templates_to_root(root_path)?;
    write_embedded_theme_static_to_root(root_path)?;
    Ok(())
}
