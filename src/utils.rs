//! 工具函数模块
//! 
//! 提供各种实用的辅助函数

use crate::error::{Error, Result};
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