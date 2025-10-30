//! 静态文件生成器模块
//! 
//! 负责生成静态网站文件

use crate::config::Config;
use crate::error::{Error, Result};
use crate::post::{Post, PostParser};
use crate::template::TemplateEngine;
use crate::utils::{copy_dir_recursive, strip_html_tags};

use std::path::Path;

/// 静态文件生成器
pub struct Generator {
    #[allow(dead_code)]
    config: Config,
    template_engine: TemplateEngine,
}

impl Generator {
    /// 创建新的生成器
    pub fn new(config: Config) -> Result<Self> {
        let template_engine = TemplateEngine::new(config.clone())?;
        
        Ok(Generator {
            config,
            template_engine,
        })
    }
    
    /// 构建网站
    pub fn build<P: AsRef<Path>, Q: AsRef<Path>>(&self, md_dir: P, output_dir: Q) -> Result<()> {
        let md_dir = md_dir.as_ref();
        let output_dir = output_dir.as_ref();
        
        println!("正在构建网站...");
        
        // 清理并重新创建输出目录
        if output_dir.exists() {
            std::fs::remove_dir_all(output_dir)
                .map_err(|e| Error::Other(format!("无法清理输出目录 {:?}: {}", output_dir, e)))?;
        }
        std::fs::create_dir_all(output_dir)?;
        
        // 复制主题静态资源到输出目录
        let theme_static_dir = "src/themes/default/static";
        if Path::new(theme_static_dir).exists() {
            copy_dir_recursive(theme_static_dir, output_dir)?;
        }
        
        // 列出所有文章
        let posts = PostParser::list_posts(md_dir)?;
        
        // 统计标签、年份和分类
        let all_tags = PostParser::collect_tags(&posts);
        let all_years = PostParser::collect_years(&posts);
        let all_categories = PostParser::generate_hierarchical_categories(&posts);
        
        // 渲染首页
        let index_html = self.template_engine.render_index(&posts, &all_tags)?;
        std::fs::write(output_dir.join("index.html"), index_html)
            .map_err(|e| Error::Other(format!("无法写入首页文件: {}", e)))?;
        
        // 渲染每篇文章
        for post in &posts {
            let post_html = self.template_engine.render_post(post)?;
            
            if let Some(slug) = post.slug() {
                let categories = post.categories();
                let rel_dir = if categories.is_empty() { 
                    String::new() 
                } else { 
                    categories.join("/") 
                };
                
                let out_path = if rel_dir.is_empty() {
                    output_dir.join(format!("{}.html", slug))
                } else {
                    output_dir.join(format!("{}/{}.html", rel_dir, slug))
                };
                
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| Error::Other(format!("无法创建文章输出目录 {:?}: {}", parent, e)))?;
                }
                
                std::fs::write(&out_path, post_html)
                    .map_err(|e| Error::Other(format!("无法写入文章文件 {:?}: {}", out_path, e)))?;
            }
        }
        
        // 渲染标签页
        let tags_html = self.template_engine.render_tags(&posts, &all_tags)?;
        std::fs::write(output_dir.join("tags.html"), tags_html)
            .map_err(|e| Error::Other(format!("无法写入标签页: {}", e)))?;
        
        // 渲染分类页
        let categories_html = self.template_engine.render_categories(&posts, &all_categories)?;
        std::fs::write(output_dir.join("categories.html"), categories_html)
            .map_err(|e| Error::Other(format!("无法写入分类页: {}", e)))?;
        
        // 渲染归档页
        let archives_html = self.template_engine.render_archives(&posts, &all_years)?;
        std::fs::write(output_dir.join("archives.html"), archives_html)
            .map_err(|e| Error::Other(format!("无法写入归档页: {}", e)))?;
        
        // 渲染关于页面
        let about_html = self.template_engine.render_about(&posts, &all_tags, &all_categories)?;
        std::fs::write(output_dir.join("about.html"), about_html)
            .map_err(|e| Error::Other(format!("无法写入关于页面: {}", e)))?;
        
        // 渲染友链页面
        let friends_html = self.template_engine.render_friends()?;
        std::fs::write(output_dir.join("friends.html"), friends_html)
            .map_err(|e| Error::Other(format!("无法写入友链页面: {}", e)))?;
        
        // 渲染搜索页面
        let search_html = self.template_engine.render_search(&posts, &all_tags)?;
        std::fs::write(output_dir.join("search.html"), search_html)
            .map_err(|e| Error::Other(format!("无法写入搜索页面: {}", e)))?;
        
        // 渲染404页面
        let not_found_html = self.template_engine.render_404(&posts, &all_tags, &all_categories)?;
        std::fs::write(output_dir.join("404.html"), not_found_html)
            .map_err(|e| Error::Other(format!("无法写入404页面: {}", e)))?;
        
        // 生成搜索索引
        self.generate_search_index(&posts, output_dir)?;
        
        println!("网站构建成功！静态文件已生成到 {:?} 目录。", output_dir);
        
        Ok(())
    }
    
    /// 生成搜索索引
    fn generate_search_index<P: AsRef<Path>>(&self, posts: &[Post], output_dir: P) -> Result<()> {
        let output_dir = output_dir.as_ref();
        let mut search_data = Vec::new();
        
        for (i, post) in posts.iter().enumerate() {
            // 提取文章内容，移除HTML标签
            let content = post.content().unwrap_or("");
            let content_with_spaces = content
                .replace("<", " <")  // 在标签前添加空格
                .replace(">", "> "); // 在标签后添加空格
            
            let content_text = strip_html_tags(&content_with_spaces);
            
            // 生成与输出目录结构一致的 URL
            let slug = post.slug().unwrap_or("");
            let categories = post.categories();
            let url = if categories.is_empty() {
                format!("{}.html", slug)
            } else {
                format!("{}/{}.html", categories.join("/"), slug)
            };

            let search_item = serde_json::json!({
                "id": i,
                "title": post.title().unwrap_or(""),
                "content": content_text,
                "tags": post.tags(),
                "categories": post.categories(),
                "slug": slug,
                "date": post.date().unwrap_or(""),
                "url": url
            });
            search_data.push(search_item);
        }
        
        let search_json = serde_json::to_string_pretty(&search_data)
            .map_err(|e| Error::Other(format!("无法序列化搜索数据: {}", e)))?;
        
        std::fs::write(output_dir.join("search.json"), search_json)
            .map_err(|e| Error::Other(format!("无法写入搜索索引文件: {}", e)))?;
        
        println!("搜索索引已生成：{:?}/search.json", output_dir);
        
        Ok(())
    }
}