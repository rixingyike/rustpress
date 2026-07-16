use rustpress::PostParser;
use std::fs;
use std::path::Path;

#[test]
fn test_parse_issue_93_md() {
    let workspace_root = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let md_path = Path::new(&workspace_root).join("source/2025/test_issue_93_dummy.md");

    // 确保父目录存在
    if let Some(parent) = md_path.parent() {
        fs::create_dir_all(parent).expect("无法创建父目录");
    }

    // 写入含有缺少空格 frontmatter 的测试文件
    let content = r#"---
title:"墨问终端脚本发布独立版 2.1，支持多图片上传"
createTime: 2025/12/25 19:21:11
tags: ["macOS", "终端脚本", "墨问笔记", "开源软件", "应用程序"]
description:"墨问终端脚本发布独立版 2.1，支持多图片上传，重构为 macOS App 以简化安装和减少系统污染，提供下载链接和安装指南。"
---
# 墨问终端脚本发布独立版 2.1，支持多图片上传
测试内容
"#;

    fs::write(&md_path, content).expect("无法写入测试文件");

    // 调用解析器
    let source_dir = Path::new(&workspace_root).join("source");
    let result = PostParser::parse_file_content(content, md_path.as_path(), source_dir.as_path());

    // 清理测试文件
    let _ = fs::remove_file(&md_path);

    // 验证结果
    match result {
        Ok(Some(value)) => {
            // 验证标题 (自动修复了缺失的空格)
            let title = value
                .get("title")
                .and_then(|v| v.as_str())
                .expect("未找到 title 字段");
            assert_eq!(
                title, "墨问终端脚本发布独立版 2.1，支持多图片上传",
                "标题解析不匹配"
            );

            // 验证描述 (自动修复了缺失的空格)
            let description = value
                .get("description")
                .and_then(|v| v.as_str())
                .expect("未找到 description 字段");
            assert_eq!(description, "墨问终端脚本发布独立版 2.1，支持多图片上传，重构为 macOS App 以简化安装和减少系统污染，提供下载链接和安装指南。", "描述解析不匹配");

            println!("✅ 93.md 解析测试通过！");
        }
        Ok(None) => panic!("解析结果为空 (未识别到 Frontmatter)"),
        Err(e) => panic!("解析出错: {:?}", e),
    }
}
