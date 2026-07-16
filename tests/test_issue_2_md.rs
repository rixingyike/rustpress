use rustpress::PostParser;
use std::fs;
use std::path::Path;

#[test]
fn test_parse_issue_2_md() {
    let workspace_root = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let md_path = Path::new(&workspace_root).join("source/2026/test_issue_2_dummy.md");

    // 写入含有缺少空格 frontmatter 的测试文件
    let content = r#"---
title:"手机和相机的区别是什么？"
createTime: 2026/01/03 00:55:10
tags: ["手机摄影", "相机"]
description:"测试"
---
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
            assert_eq!(title, "手机和相机的区别是什么？", "标题解析不匹配");

            println!("✅ 2.md 解析测试通过！");
        }
        Ok(None) => panic!("解析结果为空 (未识别到 Frontmatter)"),
        Err(e) => panic!("解析出错: {:?}", e),
    }
}
