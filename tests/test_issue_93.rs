use rustpress::PostParser;
use std::fs;
use std::path::Path;

#[test]
fn test_parse_issue_93_md() {
    // 定位目标文件
    let workspace_root = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let md_path = Path::new(&workspace_root).join("source/2025/93.md");

    // 确保文件存在
    assert!(md_path.exists(), "测试文件不存在: {:?}", md_path);

    // 读取文件内容
    let content = fs::read_to_string(&md_path).expect("无法读取文件");

    // 调用解析器
    let source_dir = Path::new(&workspace_root).join("source");
    let result = PostParser::parse_file_content(&content, md_path.as_path(), source_dir.as_path());

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
