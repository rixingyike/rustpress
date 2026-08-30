//! 内容创建与专栏运维命令行子指令模块

use crate::error::{Error, Result};
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

/// 自然排序键
fn natural_sort_key(s: &str) -> Vec<(bool, i64, String)> {
    let re = regex::Regex::new(r"(\d+)").unwrap();
    let mut parts = Vec::new();
    let mut last_end = 0;
    for mat in re.find_iter(s) {
        if mat.start() > last_end {
            parts.push((false, 0, s[last_end..mat.start()].to_lowercase()));
        }
        let num = mat.as_str().parse::<i64>().unwrap_or(0);
        parts.push((true, num, String::new()));
        last_end = mat.end();
    }
    if last_end < s.len() {
        parts.push((false, 0, s[last_end..].to_lowercase()));
    }
    parts
}

/// 检查 source 目录是否存在的前置校验
pub fn check_source_dir(md_dir: &str) -> Result<PathBuf> {
    let p = Path::new(md_dir);
    if !p.exists() || !p.is_dir() {
        eprintln!("❌ 错误: 当前目录下未找到 '{}' 目录，请在博客项目根目录下运行此命令！", md_dir);
        exit(1);
    }
    Ok(p.to_path_buf())
}

/// 查找专栏目录（支持精确与模糊/别名匹配）
fn find_column_dir(columns_dir: &Path, column_name: &str) -> PathBuf {
    let direct = columns_dir.join(column_name);
    if direct.exists() && direct.is_dir() {
        return direct;
    }

    // 模糊匹配：搜索现有子目录
    if let Ok(entries) = fs::read_dir(columns_dir) {
        let col_lower = column_name.to_lowercase();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                let name_lower = name.to_lowercase();
                if name_lower == col_lower || name_lower.contains(&col_lower) || col_lower.contains(&name_lower) {
                    return path;
                }
            }
        }
    }

    direct
}

/// 获取所有专栏目录
fn get_all_column_dirs(columns_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut dirs = Vec::new();
    if let Ok(entries) = fs::read_dir(columns_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if name.eq_ignore_ascii_case("assets") {
                    continue;
                }
                dirs.push(path);
            }
        }
    }
    dirs.sort_by(|a, b| {
        let na = a.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let nb = b.file_name().and_then(|s| s.to_str()).unwrap_or("");
        natural_sort_key(na).cmp(&natural_sort_key(nb))
    });
    Ok(dirs)
}

/// 1. 创建日常博客文章: rustpress new-blog [title]
pub fn new_blog(md_dir: &str, title: &str) -> Result<()> {
    let source_dir = check_source_dir(md_dir)?;
    let year = Local::now().format("%Y").to_string();
    let target_dir = source_dir.join(&year);
    fs::create_dir_all(&target_dir)?;

    let raw_title = if title.trim().is_empty() { "新标题" } else { title.trim() };

    // 查找当前年份目录下的最大数字文件名
    let mut max_num = 0u64;
    if let Ok(entries) = fs::read_dir(&target_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(num) = stem.parse::<u64>() {
                        if num > max_num {
                            max_num = num;
                        }
                    }
                }
            }
        }
    }

    let next_num = max_num + 1;
    let target_file = target_dir.join(format!("{}.md", next_num));

    if target_file.exists() {
        eprintln!("❌ 错误: 目标文件已存在: {}", target_file.display());
        exit(1);
    }

    let now_str = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let content = format!(
        r#"---
title: {}
date: {}
layout: blog
---

# {}

"#,
        raw_title, now_str, raw_title
    );

    fs::write(&target_file, content)?;
    println!("✅ 成功创建博客文件: {}", target_file.display());
    Ok(())
}

/// 2. 创建专栏/连载文章: rustpress new-article <column> [title]
pub fn new_article(md_dir: &str, column: &str, title: &str) -> Result<()> {
    let source_dir = check_source_dir(md_dir)?;
    let columns_dir = source_dir.join("columns");
    fs::create_dir_all(&columns_dir)?;

    let col_str = column.trim();
    if col_str.is_empty() {
        eprintln!("❌ 错误: 请指定专栏目录名或编号！例如: rustpress new-article rustpress 我的新文章");
        exit(1);
    }

    let target_col_dir = find_column_dir(&columns_dir, col_str);
    fs::create_dir_all(&target_col_dir)?;

    let mut clean_title = if title.trim().is_empty() { "新标题" } else { title.trim() };
    if let Some(stripped) = clean_title.strip_suffix(".md") {
        clean_title = stripped;
    }

    let filename = format!("{}.md", clean_title);
    let target_file = target_col_dir.join(&filename);

    if target_file.exists() {
        eprintln!("❌ 错误: 目标文件已存在: {}", target_file.display());
        exit(1);
    }

    let now_str = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let content = format!(
        r#"---
title: {}
date: {}
layout: doc-item
---

# {}

"#,
        clean_title, now_str, clean_title
    );

    fs::write(&target_file, content)?;
    println!("✅ 成功创建专栏文章: {}", target_file.display());

    // 同步更新专栏 README.md 中的 catalog
    sync_article_to_catalog(&target_col_dir, &filename, &now_str, col_str)?;

    Ok(())
}

/// 同步单篇文章到 catalog 列表
fn sync_article_to_catalog(col_dir: &Path, filename: &str, now_str: &str, col_id: &str) -> Result<()> {
    let readme_path = col_dir.join("README.md");
    if !readme_path.exists() {
        let initial_readme = format!(
            r#"---
title: "专栏 {}"
layout: columns
catalog:
  - "{}"
date: "{}"
---

# 专栏 {}
"#,
            col_id, filename, now_str, col_id
        );
        fs::write(&readme_path, initial_readme)?;
        println!("✅ 已创建并初始化专栏 README: {}", readme_path.display());
        return Ok(());
    }

    let content = fs::read_to_string(&readme_path)?;
    let fm_re = regex::Regex::new(r"(?s)^---\r?\n(.*?)\r?\n---\r?\n?(.*)$").unwrap();
    if let Some(caps) = fm_re.captures(&content) {
        let fm_text = caps.get(1).unwrap().as_str();
        let body = caps.get(2).unwrap().as_str();

        let mut lines: Vec<String> = fm_text.lines().map(|s| s.to_string()).collect();
        let mut key_idx = None;
        for (i, line) in lines.iter().enumerate() {
            if regex::Regex::new(r"^\s*catalog\s*:").unwrap().is_match(line) {
                key_idx = Some(i);
                break;
            }
        }

        if let Some(k_idx) = key_idx {
            let mut end_idx = k_idx + 1;
            let mut items = Vec::new();
            while end_idx < lines.len() {
                let line = &lines[end_idx];
                if line.trim().is_empty() {
                    end_idx += 1;
                    continue;
                }
                if line.starts_with("  -") || line.starts_with("   -") || line.starts_with("    -") || line.starts_with("\t-") {
                    let trimmed = line.trim_start_matches(|c: char| c == ' ' || c == '\t' || c == '-').trim();
                    let clean_val = trimmed.trim_matches('"').trim_matches('\'');
                    items.push(clean_val.to_string());
                    end_idx += 1;
                } else if line.starts_with(" ") || line.starts_with("\t") {
                    end_idx += 1;
                } else {
                    break;
                }
            }

            if !items.iter().any(|it| it == filename) {
                items.push(filename.to_string());
            }

            let mut new_cat_lines = vec!["catalog:".to_string()];
            for it in items {
                new_cat_lines.push(format!("  - \"{}\"", it));
            }

            lines.splice(k_idx..end_idx, new_cat_lines);
        } else {
            lines.push("catalog:".to_string());
            lines.push(format!("  - \"{}\"", filename));
        }

        let new_fm = lines.join("\n");
        let new_content = format!("---\n{}\n---\n{}", new_fm.trim(), body);
        fs::write(&readme_path, new_content)?;
        println!("✅ 已将 '{}' 同步至 {} 的 catalog 列表中", filename, readme_path.display());
    }

    Ok(())
}

/// 3. 创建简短闲言/动态: rustpress new-tweet [content]
pub fn new_tweet(md_dir: &str, content: &str) -> Result<()> {
    let source_dir = check_source_dir(md_dir)?;
    let now = Local::now();
    let year = now.format("%Y").to_string();
    let month = now.format("%m").to_string();
    let slug = now.format("%Y%m%d%H%M%S").to_string();

    let target_dir = source_dir.join("tweets").join(&year).join(&month);
    fs::create_dir_all(&target_dir)?;

    let target_file = target_dir.join(format!("{}.md", slug));
    let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let tweet_body = content.trim();

    let file_content = format!(
        r#"---
date: {}
layout: tweet
---
{}
"#,
        now_str, tweet_body
    );

    fs::write(&target_file, file_content)?;
    println!("✅ 成功创建闲言动态: {}", target_file.display());
    Ok(())
}

/// 更新专栏目录 catalog
fn update_column_catalog(col_dir: &Path, readme_content: &str) -> Result<(usize, String, usize)> {
    let fm_re = regex::Regex::new(r"(?s)^---\r?\n(.*?)\r?\n---\r?\n?(.*)$").unwrap();
    let caps = match fm_re.captures(readme_content) {
        Some(c) => c,
        None => return Ok((0, readme_content.to_string(), 0)),
    };

    let fm_text = caps.get(1).unwrap().as_str();
    let body = caps.get(2).unwrap().as_str();

    // 收集磁盘上的所有实际 .md 文件（排除 README.md）
    let mut disk_files = Vec::new();
    if let Ok(entries) = fs::read_dir(col_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if !name.eq_ignore_ascii_case("readme.md") {
                    disk_files.push(name.to_string());
                }
            }
        }
    }

    disk_files.sort_by(|a, b| natural_sort_key(a).cmp(&natural_sort_key(b)));

    let mut lines: Vec<String> = fm_text.lines().map(|s| s.to_string()).collect();
    let mut key_idx = None;
    for (i, line) in lines.iter().enumerate() {
        if regex::Regex::new(r"^\s*catalog\s*:").unwrap().is_match(line) {
            key_idx = Some(i);
            break;
        }
    }

    let mut existing_items = Vec::new();
    let mut end_idx = 0;

    if let Some(k_idx) = key_idx {
        end_idx = k_idx + 1;
        while end_idx < lines.len() {
            let line = &lines[end_idx];
            if line.trim().is_empty() {
                end_idx += 1;
                continue;
            }
            if line.starts_with("  -") || line.starts_with("   -") || line.starts_with("    -") || line.starts_with("\t-") {
                let trimmed = line.trim_start_matches(|c: char| c == ' ' || c == '\t' || c == '-').trim();
                let clean_val = trimmed.trim_matches('"').trim_matches('\'');
                existing_items.push(clean_val.to_string());
                end_idx += 1;
            } else if line.starts_with(" ") || line.starts_with("\t") {
                end_idx += 1;
            } else {
                break;
            }
        }
    }

    let existing_count = existing_items.len();
    let mut added_count = 0;

    // 找出未在 catalog 中的文件
    for f in &disk_files {
        if !existing_items.iter().any(|it| it == f) {
            existing_items.push(f.clone());
            added_count += 1;
        }
    }

    if added_count == 0 && key_idx.is_some() {
        return Ok((existing_count, readme_content.to_string(), 0));
    }

    let mut new_cat_lines = vec!["catalog:".to_string()];
    for it in existing_items {
        new_cat_lines.push(format!("  - \"{}\"", it));
    }

    if let Some(k_idx) = key_idx {
        lines.splice(k_idx..end_idx, new_cat_lines);
    } else {
        lines.push("catalog:".to_string());
        for it in &disk_files {
            lines.push(format!("  - \"{}\"", it));
        }
    }

    let new_fm = lines.join("\n");
    let new_content = format!("---\n{}\n---\n{}", new_fm.trim(), body);
    Ok((existing_count, new_content, added_count))
}

/// 4. 自动检查并更新专栏 catalog: rustpress make-catalog [column] [--all]
pub fn make_catalog(md_dir: &str, column: Option<&str>, all: bool) -> Result<()> {
    let source_dir = check_source_dir(md_dir)?;
    let columns_dir = source_dir.join("columns");
    if !columns_dir.exists() || !columns_dir.is_dir() {
        eprintln!("❌ 错误: 未在 {} 下找到 columns 专栏目录！", source_dir.display());
        exit(1);
    }

    let col_dirs = if all || column.is_none() {
        get_all_column_dirs(&columns_dir)?
    } else {
        vec![find_column_dir(&columns_dir, column.unwrap())]
    };

    println!("🚀 开始检查/更新专栏 catalog (共 {} 个专栏)...", col_dirs.len());

    let mut updated_count = 0;
    for col_dir in col_dirs {
        let col_name = col_dir.file_name().and_then(|s| s.to_str()).unwrap_or("未知");
        let readme_path = col_dir.join("README.md");
        if !readme_path.exists() {
            println!("⚠️ 专栏 {} 缺少 README.md，跳过", col_name);
            continue;
        }

        let readme_content = fs::read_to_string(&readme_path)?;
        let (existing_items, updated_content, added) = update_column_catalog(&col_dir, &readme_content)?;
        if added > 0 {
            fs::write(&readme_path, updated_content)?;
            println!("✅ {}: 已追加 {} 篇新文章至 catalog 列表 (当前共 {} 项)", col_name, added, existing_items + added);
            updated_count += 1;
        } else {
            println!("✅ {}: catalog 已经是最新的（包含 {} 项）", col_name, existing_items);
        }
    }

    println!("\n✨ 处理完成！共检查并更新了 {} 个专栏。", updated_count);
    Ok(())
}

/// 5. 自动生成专栏封面: rustpress make-cover [column] [--all] [--style]
pub fn make_cover(md_dir: &str, column: Option<&str>, all: bool, style: &str) -> Result<()> {
    let source_dir = check_source_dir(md_dir)?;
    let project_root = source_dir.parent().unwrap_or(&source_dir);

    // 优先查找 make_cover.py 脚本
    let python_script = if project_root.join("scripts/make_cover.py").exists() {
        Some(project_root.join("scripts/make_cover.py"))
    } else if Path::new("scripts/make_cover.py").exists() {
        Some(PathBuf::from("scripts/make_cover.py"))
    } else {
        None
    };

    if let Some(script_path) = python_script {
        let mut cmd = Command::new("python3");
        cmd.arg(&script_path);
        if all || column.is_none() {
            cmd.arg("--all");
        } else if let Some(col) = column {
            cmd.arg(col);
        }
        cmd.arg("--style").arg(style);

        println!("🎨 正在调用专栏封面生成引擎 (python3 {})...", script_path.display());
        let status = cmd.status().map_err(|e| Error::Io(e))?;
        if !status.success() {
            eprintln!("❌ 封面生成脚本执行失败，退出代码: {:?}", status.code());
            exit(1);
        }
    } else {
        eprintln!("⚠️ 未找到 scripts/make_cover.py 封面生成脚本，请确保 scripts 目录完整。");
    }

    Ok(())
}
