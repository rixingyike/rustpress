//! 静态文件生成器模块
//!
//! 负责生成静态网站文件

use crate::config::Config;
use crate::error::{Error, Result};
use crate::post::{Post, PostParser};
use crate::template::TemplateEngine;
use crate::utils::{copy_dir_recursive, strip_html_tags};
use chrono::TimeZone;

use serde_json::Value;
use std::path::Path;

/// 静态文件生成器
pub struct Generator {
    #[allow(dead_code)]
    config: Config,
    template_engine: TemplateEngine,
}

impl Generator {
    /// 创建新的生成器
    pub fn new<P: AsRef<std::path::Path>>(config: Config, md_dir: P) -> Result<Self> {
        let template_engine = TemplateEngine::new(config.clone(), md_dir)?;

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

        // 写出打包在二进制中的主题静态资源到输出目录（若存在本地静态目录，随后拷贝以覆盖）
        crate::utils::write_embedded_theme_static(output_dir)?;
        // 若本地存在主题静态目录，拷贝以覆盖（保留用户覆盖能力）
        let runtime_paths = crate::utils::RuntimePathsBuilder::new()
            .md_dir(md_dir)
            .theme_name(self.config.theme_name())
            .build();
        let theme_static_dir = runtime_paths.theme_static_dir;
        if theme_static_dir.exists() {
            copy_dir_recursive(&theme_static_dir, output_dir)?;
        }

        // 递归复制源目录下的所有非 Markdown 且非隐藏文件，保持相对路径（覆盖原有顶层 assets 与根层非 md 的拷贝策略）
        crate::utils::copy_non_md_recursive_preserve_paths(md_dir, output_dir)?;

        // 列出所有文章
        let posts = PostParser::list_posts(md_dir)?;

        // 首次构建时生成侧边栏数据（可手动编辑，写入优先项目根）
        crate::utils::ensure_sidebar_data(md_dir, &posts)?;

        // 统计标签、年份和分类
        let all_tags = PostParser::collect_tags(&posts);
        let all_years = PostParser::collect_years(&posts);
        let all_categories = PostParser::generate_hierarchical_categories(&posts);

        // 获取每页文章数量配置
        let posts_per_page = self
            .config
            .data
            .get("homepage")
            .and_then(|v| v.get("posts_per_page"))
            .and_then(|v| v.as_integer())
            .unwrap_or(10) as usize;

        // 计算总页数
        let total_pages = (posts.len() + posts_per_page - 1) / posts_per_page;

        // 渲染首页（Home 布局，集成 home.md 内容与导航）
        let index_html = if total_pages > 1 {
            // 有分页时，首页显示最大页（最新）
            self.template_engine.render_home_page(
                &posts,
                &all_tags,
                &all_categories,
                total_pages,
            )?
        } else {
            // 仅1页时，显示 Home 第1页
            self.template_engine
                .render_home(&posts, &all_tags, &all_categories)?
        };
        std::fs::write(output_dir.join("index.html"), index_html)
            .map_err(|e| Error::Other(format!("无法写入首页文件: {}", e)))?;

        // 生成首页分页页面（根目录 index{n}.html，最大页为 index.html）
        self.generate_index_pages(&posts, &all_tags, &all_categories, output_dir)?;

        // 渲染每篇文章
        for post in &posts {
            let post_html = self.template_engine.render_post(post, &posts)?;

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
                    std::fs::create_dir_all(parent).map_err(|e| {
                        Error::Other(format!("无法创建文章输出目录 {:?}: {}", parent, e))
                    })?;
                }

                std::fs::write(&out_path, post_html)
                    .map_err(|e| Error::Other(format!("无法写入文章文件 {:?}: {}", out_path, e)))?;
            }
        }

        // 渲染标签页
        let tags_html = self.template_engine.render_tags(&posts, &all_tags)?;
        std::fs::write(output_dir.join("tags.html"), tags_html)
            .map_err(|e| Error::Other(format!("无法写入标签页: {}", e)))?;

        // 生成单标签文章列表分页页面
        self.generate_tag_pages(&posts, output_dir)?;

        // 渲染分类页
        let categories_html =
            self.template_engine
                .render_categories(&posts, &all_categories, &all_tags)?;
        std::fs::write(output_dir.join("categories.html"), categories_html)
            .map_err(|e| Error::Other(format!("无法写入分类页: {}", e)))?;

        // 为每个分类生成分类索引页面
        self.generate_category_pages(&posts, output_dir)?;

        // 渲染归档页
        let archives_html = self.template_engine.render_archives(&posts, &all_years)?;
        std::fs::write(output_dir.join("archives.html"), archives_html)
            .map_err(|e| Error::Other(format!("无法写入归档页: {}", e)))?;

        // 生成年份归档页
        self.generate_year_archive_pages(&posts, output_dir)?;

        // 渲染关于页面
        let about_html = self
            .template_engine
            .render_about(&posts, &all_tags, &all_categories)?;
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
        let not_found_html = self
            .template_engine
            .render_404(&posts, &all_tags, &all_categories)?;
        std::fs::write(output_dir.join("404.html"), not_found_html)
            .map_err(|e| Error::Other(format!("无法写入404页面: {}", e)))?;

        // 生成搜索索引
        self.generate_search_index(&posts, output_dir)?;

        // 生成 RSS（按开关）
        let rss_enabled = self
            .config
            .data
            .get("features")
            .and_then(|v| v.get("rss"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        if rss_enabled {
            self.generate_rss(&posts, output_dir)?;
        }

        // 生成 Sitemap（按开关）
        let sitemap_enabled = self
            .config
            .data
            .get("features")
            .and_then(|v| v.get("sitemap"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        if sitemap_enabled {
            self.generate_sitemap(&posts, output_dir)?;
        }

        println!("网站构建成功！静态文件已生成到 {:?} 目录。", output_dir);

        Ok(())
    }

    // 已移除：get_subcategories（未使用）

    /// 生成单标签分页页面（URL: tags/{tag}/index.html, index2.html, index3.html ...）
    fn generate_tag_pages<P: AsRef<Path>>(&self, posts: &[Post], output_dir: P) -> Result<()> {
        let output_dir = output_dir.as_ref();

        // 每页数量从 config.tags.posts_per_page 读取，回退到 homepage.posts_per_page，再回退到默认 8
        let posts_per_page = self
            .config
            .data
            .get("tags")
            .and_then(|v| v.get("posts_per_page"))
            .and_then(|v| v.as_integer())
            .or_else(|| {
                self.config
                    .data
                    .get("homepage")
                    .and_then(|v| v.get("posts_per_page"))
                    .and_then(|v| v.as_integer())
            })
            .unwrap_or(8) as usize;

        // 收集所有唯一标签名
        let mut tag_set: std::collections::HashSet<String> = std::collections::HashSet::new();
        for post in posts {
            for tag in post.tags() {
                tag_set.insert(tag.clone());
            }
        }

        // 为每个标签生成分页页面
        for tag_name in tag_set.into_iter() {
            // 筛选并按日期降序排序该标签下的文章
            let mut tag_posts: Vec<&Post> = posts
                .iter()
                .filter(|p| p.tags().contains(&tag_name))
                .collect();
            tag_posts.sort_by(|a, b| {
                let date_a = a.date().unwrap_or("");
                let date_b = b.date().unwrap_or("");
                date_b.cmp(date_a)
            });

            let total_posts = tag_posts.len();
            let total_pages = if total_posts == 0 {
                1
            } else {
                (total_posts + posts_per_page - 1) / posts_per_page
            };

            // 创建输出目录：public/tags/{tag_name}/
            let tag_dir = output_dir.join("tags").join(&tag_name);
            std::fs::create_dir_all(&tag_dir)
                .map_err(|e| Error::Other(format!("无法创建标签目录 {:?}: {}", tag_dir, e)))?;

            // 逐页渲染并输出（倒分页）：最大页（最新）输出为 index.html，其余为 indexN.html
            for page in 1..=total_pages {
                let html = self.template_engine.render_tag_page(
                    &tag_posts,
                    &tag_name,
                    page,
                    posts_per_page,
                )?;
                let file_name = if page == total_pages {
                    "index.html".to_string()
                } else {
                    format!("index{}.html", page)
                };
                let out_path = tag_dir.join(file_name);
                std::fs::write(&out_path, html).map_err(|e| {
                    Error::Other(format!("无法写入标签分页文件 {:?}: {}", out_path, e))
                })?;
            }
        }

        println!("已生成标签分页页面：路径模式 tags/<name>/index[.N].html");
        Ok(())
    }

    /// 仅为指定标签集合生成分页页面（倒分页），并输出详细路径日志
    fn generate_tag_pages_for<P: AsRef<Path>>(
        &self,
        posts: &[Post],
        tags: &std::collections::HashSet<String>,
        output_dir: P,
    ) -> Result<()> {
        let output_dir = output_dir.as_ref();

        let posts_per_page = self
            .config
            .data
            .get("tags")
            .and_then(|v| v.get("posts_per_page"))
            .and_then(|v| v.as_integer())
            .or_else(|| {
                self.config
                    .data
                    .get("homepage")
                    .and_then(|v| v.get("posts_per_page"))
                    .and_then(|v| v.as_integer())
            })
            .unwrap_or(8) as usize;

        for tag_name in tags.iter() {
            let mut rebuilt_paths: Vec<String> = Vec::new();
            let mut tag_posts: Vec<&Post> = posts
                .iter()
                .filter(|p| p.tags().contains(tag_name))
                .collect();
            tag_posts.sort_by(|a, b| {
                let da = a.date().unwrap_or("");
                let db = b.date().unwrap_or("");
                db.cmp(da)
            });

            let total_posts = tag_posts.len();
            let total_pages = if total_posts == 0 {
                1
            } else {
                (total_posts + posts_per_page - 1) / posts_per_page
            };

            let tag_dir = output_dir.join("tags").join(tag_name);
            std::fs::create_dir_all(&tag_dir)
                .map_err(|e| Error::Other(format!("无法创建标签目录 {:?}: {}", tag_dir, e)))?;

            for page in 1..=total_pages {
                let html = self.template_engine.render_tag_page(
                    &tag_posts,
                    tag_name,
                    page,
                    posts_per_page,
                )?;
                let file_name = if page == total_pages {
                    "index.html".to_string()
                } else {
                    format!("index{}.html", page)
                };
                let out_path = tag_dir.join(file_name);
                std::fs::write(&out_path, html).map_err(|e| {
                    Error::Other(format!("无法写入标签分页文件 {:?}: {}", out_path, e))
                })?;
                let rel = out_path.strip_prefix(output_dir).unwrap_or(&out_path);
                rebuilt_paths.push(format!("/{}", rel.to_string_lossy()));
            }
            println!(
                "标签重建: '{}' 共 {} 页 -> {}",
                tag_name,
                total_pages,
                rebuilt_paths.join(", ")
            );
        }
        Ok(())
    }

    // 已移除：get_related_tags（未使用）

    /// 为每个分类生成分页页面（倒分页）：最大页（最新）为 index.html，其余为 indexN.html
    fn generate_category_pages<P: AsRef<Path>>(&self, posts: &[Post], output_dir: P) -> Result<()> {
        let output_dir = output_dir.as_ref();

        // 每页数量从 config.categories.posts_per_page 读取，回退到 homepage.posts_per_page，再回退到默认 8
        let posts_per_page = self
            .config
            .data
            .get("categories")
            .and_then(|v| v.get("posts_per_page"))
            .and_then(|v| v.as_integer())
            .or_else(|| {
                self.config
                    .data
                    .get("homepage")
                    .and_then(|v| v.get("posts_per_page"))
                    .and_then(|v| v.as_integer())
            })
            .unwrap_or(8) as usize;

        // 收集所有分类路径（包括多层级分类）
        let mut category_paths: std::collections::HashSet<Vec<String>> =
            std::collections::HashSet::new();
        for post in posts {
            let cats = post.categories();
            if !cats.is_empty() {
                // 为每个分类路径的每个层级生成索引页面
                for i in 1..=cats.len() {
                    let path = cats[0..i].to_vec();
                    category_paths.insert(path);
                }
            }
        }

        // 为每个分类路径生成分页页面（倒分页）
        for category_path in category_paths.into_iter() {
            // 获取该分类路径下的所有文章，并按日期降序排序（最新在前）
            let mut category_posts: Vec<&Post> = posts
                .iter()
                .filter(|post| {
                    let post_cats = post.categories();
                    post_cats.len() >= category_path.len()
                        && post_cats[0..category_path.len()] == category_path
                })
                .collect();
            category_posts.sort_by(|a, b| {
                let date_a = a.date().unwrap_or("");
                let date_b = b.date().unwrap_or("");
                date_b.cmp(date_a)
            });

            // 构建分类目录路径
            let category_dir = output_dir.join(category_path.join("/"));
            std::fs::create_dir_all(&category_dir)
                .map_err(|e| Error::Other(format!("无法创建分类目录 {:?}: {}", category_dir, e)))?;

            // 计算总页数
            let total_posts = category_posts.len();
            let total_pages = if total_posts == 0 {
                1
            } else {
                (total_posts + posts_per_page - 1) / posts_per_page
            };

            // 逐页渲染并输出（倒分页）：最大页（最新）输出为 index.html，其余为 indexN.html
            for page in 1..=total_pages {
                let html = self.template_engine.render_category_page(
                    &category_posts,
                    &category_path,
                    page,
                    posts_per_page,
                )?;
                let file_name = if page == total_pages {
                    "index.html".to_string()
                } else {
                    format!("index{}.html", page)
                };
                let out_path = category_dir.join(file_name);
                std::fs::write(&out_path, html).map_err(|e| {
                    Error::Other(format!("无法写入分类分页文件 {:?}: {}", out_path, e))
                })?;
            }
        }

        println!("已生成分类分页页面：路径模式 <category-path>/index[.N].html");
        Ok(())
    }

    /// 仅为指定分类路径集合生成分页页面（倒分页），并输出详细路径日志
    fn generate_category_pages_for<P: AsRef<Path>>(
        &self,
        posts: &[Post],
        category_paths: &std::collections::HashSet<Vec<String>>,
        output_dir: P,
    ) -> Result<()> {
        let output_dir = output_dir.as_ref();

        let posts_per_page = self
            .config
            .data
            .get("categories")
            .and_then(|v| v.get("posts_per_page"))
            .and_then(|v| v.as_integer())
            .or_else(|| {
                self.config
                    .data
                    .get("homepage")
                    .and_then(|v| v.get("posts_per_page"))
                    .and_then(|v| v.as_integer())
            })
            .unwrap_or(8) as usize;

        for category_path in category_paths.iter() {
            let mut rebuilt_paths: Vec<String> = Vec::new();
            let mut category_posts: Vec<&Post> = posts
                .iter()
                .filter(|post| {
                    let pc = post.categories();
                    pc.len() >= category_path.len() && pc[0..category_path.len()] == *category_path
                })
                .collect();
            category_posts.sort_by(|a, b| {
                let da = a.date().unwrap_or("");
                let db = b.date().unwrap_or("");
                db.cmp(da)
            });

            let category_dir = output_dir.join(category_path.join("/"));
            std::fs::create_dir_all(&category_dir)
                .map_err(|e| Error::Other(format!("无法创建分类目录 {:?}: {}", category_dir, e)))?;

            let total_posts = category_posts.len();
            let total_pages = if total_posts == 0 {
                1
            } else {
                (total_posts + posts_per_page - 1) / posts_per_page
            };

            for page in 1..=total_pages {
                let html = self.template_engine.render_category_page(
                    &category_posts,
                    category_path,
                    page,
                    posts_per_page,
                )?;
                let file_name = if page == total_pages {
                    "index.html".to_string()
                } else {
                    format!("index{}.html", page)
                };
                let out_path = category_dir.join(file_name);
                std::fs::write(&out_path, html).map_err(|e| {
                    Error::Other(format!("无法写入分类分页文件 {:?}: {}", out_path, e))
                })?;
                let rel = out_path.strip_prefix(output_dir).unwrap_or(&out_path);
                rebuilt_paths.push(format!("/{}", rel.to_string_lossy()));
            }
            println!(
                "分类重建: '{}' 共 {} 页 -> {}",
                category_path.join("/"),
                total_pages,
                rebuilt_paths.join(", ")
            );
        }
        Ok(())
    }

    /// 生成首页分页页面
    fn generate_index_pages<P: AsRef<Path>>(
        &self,
        posts: &[Post],
        all_tags: &[Value],
        all_categories: &Value,
        output_dir: P,
    ) -> Result<()> {
        let output_dir = output_dir.as_ref();

        // 获取每页文章数量配置
        let posts_per_page = self
            .config
            .data
            .get("homepage")
            .and_then(|v| v.get("posts_per_page"))
            .and_then(|v| v.as_integer())
            .unwrap_or(10) as usize;

        // 计算总页数
        let total_pages = (posts.len() + posts_per_page - 1) / posts_per_page;

        // 如果只有1页，不需要生成分页页面
        if total_pages <= 1 {
            return Ok(());
        }

        // 生成所有分页页面为根目录文件 index{n}.html
        // 注意：最大页为首页 index.html，因此仅生成到 total_pages-1
        for page in 1..=std::cmp::max(1, total_pages.saturating_sub(1)) {
            let page_html = self
                .template_engine
                .render_home_page(posts, all_tags, all_categories, page)
                .map_err(|e| Error::Other(format!("无法渲染第{}页: {}", page, e)))?;

            // 根目录下命名为 index{n}.html
            let page_file = output_dir.join(format!("index{}.html", page));

            std::fs::write(&page_file, page_html)
                .map_err(|e| Error::Other(format!("无法写入分页文件 {:?}: {}", page_file, e)))?;
        }

        println!(
            "已生成首页分页页面：共{}页，路径 index{{n}}.html（首页为 index.html）",
            total_pages
        );

        Ok(())
    }

    /// 仅为指定页码集合生成首页分页页面（倒分页），并输出详细路径日志
    fn generate_index_pages_for<P: AsRef<Path>>(
        &self,
        posts: &[Post],
        all_tags: &[Value],
        all_categories: &Value,
        pages: &std::collections::HashSet<usize>,
        output_dir: P,
    ) -> Result<()> {
        let output_dir = output_dir.as_ref();

        // 每页数量
        let posts_per_page = self
            .config
            .data
            .get("homepage")
            .and_then(|v| v.get("posts_per_page"))
            .and_then(|v| v.as_integer())
            .unwrap_or(10) as usize;

        let total_pages = (posts.len() + posts_per_page - 1) / posts_per_page;
        if total_pages == 0 || pages.is_empty() {
            return Ok(());
        }

        let mut list: Vec<usize> = pages
            .iter()
            .cloned()
            .filter(|p| *p >= 1 && *p <= total_pages)
            .collect();
        list.sort_unstable();

        let mut rebuilt_paths: Vec<String> = Vec::new();
        for page in list {
            if total_pages == 1 && page == 1 {
                let html = self
                    .template_engine
                    .render_home(posts, all_tags, all_categories)?;
                let out_path = output_dir.join("index.html");
                std::fs::write(&out_path, html)
                    .map_err(|e| Error::Other(format!("无法写入首页文件 {:?}: {}", out_path, e)))?;
                rebuilt_paths.push("/index.html".to_string());
            } else {
                let html = self
                    .template_engine
                    .render_home_page(posts, all_tags, all_categories, page)
                    .map_err(|e| Error::Other(format!("无法渲染第{}页: {}", page, e)))?;
                let out_path = if page == total_pages {
                    output_dir.join("index.html")
                } else {
                    output_dir.join(format!("index{}.html", page))
                };
                std::fs::write(&out_path, html)
                    .map_err(|e| Error::Other(format!("无法写入分页文件 {:?}: {}", out_path, e)))?;
                let rel = out_path.strip_prefix(output_dir).unwrap_or(&out_path);
                rebuilt_paths.push(format!("/{}", rel.to_string_lossy()));
            }
        }
        println!(
            "首页分页重建 {} 个 -> {}",
            rebuilt_paths.len(),
            rebuilt_paths.join(", ")
        );
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
                .replace("<", " <") // 在标签前添加空格
                .replace(">", "> "); // 在标签后添加空格

            let content_text = strip_html_tags(&content_with_spaces);

            // 生成与输出目录结构一致的 URL
            let slug = post.slug().unwrap_or("");
            let categories = post.categories();
            let url = if categories.is_empty() {
                format!("/{}.html", slug)
            } else {
                format!("/{}/{}.html", categories.join("/"), slug)
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

    /// 生成 RSS (RSS 2.0 简版)
    fn generate_rss<P: AsRef<Path>>(&self, posts: &[Post], output_dir: P) -> Result<()> {
        let output_dir = output_dir.as_ref();

        // 站点信息
        let site = self.config.data.get("site");
        let site_name = site
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("RustPress Blog");
        let site_desc = site
            .and_then(|v| v.get("description"))
            .and_then(|v| v.as_str())
            .unwrap_or("A RustPress site");
        // 优先使用自定义域名字段，其次回退 base_url
        let base_url = site
            .and_then(|v| v.get("domain"))
            .and_then(|v| v.as_str())
            .or_else(|| {
                site.and_then(|v| v.get("base_url"))
                    .and_then(|v| v.as_str())
            })
            .unwrap_or("");
        let base = base_url.trim_end_matches('/');

        // 简单 XML 转义函数
        fn escape_xml(s: &str) -> String {
            s.replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
                .replace('\'', "&apos;")
        }

        let mut items_xml = String::new();
        for post in posts {
            let title = escape_xml(post.title().unwrap_or(""));
            let slug = post.slug().unwrap_or("");
            let cats = post.categories();
            let path = if cats.is_empty() {
                format!("/{}.html", slug)
            } else {
                format!("/{}/{}.html", cats.join("/"), slug)
            };
            let link = format!("{}{}", base, path);

            let content = post.content().unwrap_or("");
            let content_with_spaces = content.replace('<', " <").replace('>', "> ");
            let text = strip_html_tags(&content_with_spaces);
            let summary: String = text.chars().take(400).collect();
            let description = escape_xml(summary.trim());

            let pub_date = post.date().unwrap_or("");
            let guid = &link;

            items_xml.push_str(&format!(
                "  <item>\n    <title>{}</title>\n    <link>{}</link>\n    <guid isPermaLink=\"true\">{}</guid>\n    <description>{}</description>\n    <pubDate>{}</pubDate>\n  </item>\n",
                title, link, guid, description, pub_date
            ));
        }

        let rss_xml = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rss version=\"2.0\">\n<channel>\n  <title>{}</title>\n  <link>{}</link>\n  <description>{}</description>\n{}\n</channel>\n</rss>\n",
            escape_xml(site_name),
            escape_xml(base_url),
            escape_xml(site_desc),
            items_xml
        );

        std::fs::write(output_dir.join("rss.xml"), rss_xml)
            .map_err(|e| Error::Other(format!("无法写入RSS文件: {}", e)))?;
        println!("RSS 已生成：{:?}/rss.xml", output_dir);
        Ok(())
    }

    /// 生成 Sitemap (包含首页分页、文章、标签分页、分类分页、年份归档与主要静态页)
    fn generate_sitemap<P: AsRef<Path>>(&self, posts: &[Post], output_dir: P) -> Result<()> {
        let output_dir = output_dir.as_ref();

        // 基础 URL
        let base_url = self
            .config
            .data
            .get("site")
            .and_then(|v| v.get("domain"))
            .and_then(|v| v.as_str())
            .or_else(|| {
                self.config
                    .data
                    .get("site")
                    .and_then(|v| v.get("base_url"))
                    .and_then(|v| v.as_str())
            })
            .unwrap_or("");
        let base = base_url.trim_end_matches('/');

        let mut urls: Vec<String> = Vec::new();

        // 首页与分页
        let posts_per_page = self
            .config
            .data
            .get("homepage")
            .and_then(|v| v.get("posts_per_page"))
            .and_then(|v| v.as_integer())
            .unwrap_or(10) as usize;
        let total_pages = (posts.len() + posts_per_page - 1) / posts_per_page;
        urls.push(format!("{}/index.html", base));
        for page in 1..=std::cmp::max(1, total_pages.saturating_sub(1)) {
            urls.push(format!("{}/index{}.html", base, page));
        }

        // 文章页
        for post in posts {
            let slug = post.slug().unwrap_or("");
            let cats = post.categories();
            let path = if cats.is_empty() {
                format!("/{}.html", slug)
            } else {
                format!("/{}/{}.html", cats.join("/"), slug)
            };
            urls.push(format!("{}{}", base, path));
        }

        // 标签分页（倒分页）
        let mut tag_set: std::collections::HashSet<String> = std::collections::HashSet::new();
        for p in posts {
            for t in p.tags() {
                tag_set.insert(t.clone());
            }
        }
        let tags_dir = "tags";
        for tag in tag_set.into_iter() {
            let mut tag_posts: Vec<&Post> =
                posts.iter().filter(|p| p.tags().contains(&tag)).collect();
            tag_posts.sort_by(|a, b| {
                let da = a.date().unwrap_or("");
                let db = b.date().unwrap_or("");
                db.cmp(da)
            });
            let total_posts = tag_posts.len();
            let posts_per_page = self
                .config
                .data
                .get("tags")
                .and_then(|v| v.get("posts_per_page"))
                .and_then(|v| v.as_integer())
                .or_else(|| {
                    self.config
                        .data
                        .get("homepage")
                        .and_then(|v| v.get("posts_per_page"))
                        .and_then(|v| v.as_integer())
                })
                .unwrap_or(8) as usize;
            let total_pages = if total_posts == 0 {
                1
            } else {
                (total_posts + posts_per_page - 1) / posts_per_page
            };
            for page in 1..=total_pages {
                let file_name = if page == total_pages {
                    "index.html".to_string()
                } else {
                    format!("index{}.html", page)
                };
                urls.push(format!("{}/{}/{}/{}", base, tags_dir, tag, file_name));
            }
        }

        // 分类分页（倒分页）
        let mut category_paths: std::collections::HashSet<Vec<String>> =
            std::collections::HashSet::new();
        for post in posts {
            let cats = post.categories();
            if !cats.is_empty() {
                for i in 1..=cats.len() {
                    category_paths.insert(cats[0..i].to_vec());
                }
            }
        }
        let posts_per_page_cat = self
            .config
            .data
            .get("categories")
            .and_then(|v| v.get("posts_per_page"))
            .and_then(|v| v.as_integer())
            .or_else(|| {
                self.config
                    .data
                    .get("homepage")
                    .and_then(|v| v.get("posts_per_page"))
                    .and_then(|v| v.as_integer())
            })
            .unwrap_or(8) as usize;
        for path in category_paths.into_iter() {
            let mut cat_posts: Vec<&Post> = posts
                .iter()
                .filter(|post| {
                    let pc = post.categories();
                    pc.len() >= path.len() && pc[0..path.len()] == path
                })
                .collect();
            cat_posts.sort_by(|a, b| {
                let da = a.date().unwrap_or("");
                let db = b.date().unwrap_or("");
                db.cmp(da)
            });
            let total_posts = cat_posts.len();
            let total_pages = if total_posts == 0 {
                1
            } else {
                (total_posts + posts_per_page_cat - 1) / posts_per_page_cat
            };
            for page in 1..=total_pages {
                let file_name = if page == total_pages {
                    "index.html".to_string()
                } else {
                    format!("index{}.html", page)
                };
                urls.push(format!("{}/{}/{}", base, path.join("/"), file_name));
            }
        }

        // 年份归档页
        let mut years: std::collections::HashSet<String> = std::collections::HashSet::new();
        for post in posts {
            if let Some(date) = post.date() {
                if date.len() >= 4 {
                    years.insert(date[0..4].to_string());
                }
            }
        }
        for year in years.into_iter() {
            urls.push(format!("{}/archives/{}.html", base, year));
        }

        // 主要静态页面
        for static_page in [
            "tags.html",
            "categories.html",
            "archives.html",
            "about.html",
            "friends.html",
            "search.html",
        ] {
            urls.push(format!("{}/{}", base, static_page));
        }

        // 生成XML
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");
        for u in urls {
            xml.push_str(&format!("  <url><loc>{}</loc></url>\n", u));
        }
        xml.push_str("</urlset>\n");

        std::fs::write(output_dir.join("sitemap.xml"), xml)
            .map_err(|e| Error::Other(format!("无法写入Sitemap文件: {}", e)))?;
        println!("Sitemap 已生成：{:?}/sitemap.xml", output_dir);
        Ok(())
    }

    /// 生成年份归档页面
    fn generate_year_archive_pages<P: AsRef<Path>>(
        &self,
        posts: &[Post],
        output_dir: P,
    ) -> Result<()> {
        let output_dir = output_dir.as_ref();

        // 创建archives目录
        let archives_dir = output_dir.join("archives");
        std::fs::create_dir_all(&archives_dir)?;

        // 按年份分组文章
        let mut year_posts: std::collections::HashMap<String, Vec<&Post>> =
            std::collections::HashMap::new();

        for post in posts {
            if let Some(date) = post.date() {
                // 从日期中提取年份
                let year = if date.len() >= 4 {
                    date[0..4].to_string()
                } else {
                    continue;
                };

                year_posts.entry(year).or_insert_with(Vec::new).push(post);
            }
        }

        // 为每个年份生成归档页
        for (year, year_post_list) in year_posts {
            let year_archive_html = self
                .template_engine
                .render_year_archive(&year_post_list, &year)?;
            let year_file_path = archives_dir.join(format!("{}.html", year));

            std::fs::write(&year_file_path, year_archive_html)
                .map_err(|e| Error::Other(format!("无法写入年份归档页 {}: {}", year, e)))?;

            println!("年份归档页已生成：{:?}", year_file_path);
        }

        Ok(())
    }

    /// 仅为指定年份集合生成归档页面，并输出详细路径日志
    fn generate_year_archive_pages_for<P: AsRef<Path>>(
        &self,
        posts: &[Post],
        years: &std::collections::HashSet<String>,
        output_dir: P,
    ) -> Result<()> {
        let output_dir = output_dir.as_ref();

        let archives_dir = output_dir.join("archives");
        std::fs::create_dir_all(&archives_dir)?;

        let mut year_posts: std::collections::HashMap<String, Vec<&Post>> =
            std::collections::HashMap::new();
        for post in posts {
            if let Some(date) = post.date() {
                if date.len() >= 4 {
                    let y = &date[0..4];
                    if years.contains(y) {
                        year_posts
                            .entry(y.to_string())
                            .or_insert_with(Vec::new)
                            .push(post);
                    }
                }
            }
        }

        for (year, list) in year_posts {
            let html = self.template_engine.render_year_archive(&list, &year)?;
            let file_path = archives_dir.join(format!("{}.html", year));
            std::fs::write(&file_path, html)
                .map_err(|e| Error::Other(format!("无法写入年份归档页 {}: {}", year, e)))?;
            let rel = file_path.strip_prefix(output_dir).unwrap_or(&file_path);
            println!("年份重建: {} -> /{}", year, rel.to_string_lossy());
        }

        Ok(())
    }

    /// 增量构建网站：仅渲染 last_build_time 之后修改的文章页面，派生页全量刷新
    pub fn build_incremental<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        md_dir: P,
        output_dir: Q,
    ) -> Result<()> {
        let md_dir = md_dir.as_ref();
        let output_dir = output_dir.as_ref();

        println!("正在进行增量构建...");

        // 确保输出目录存在（不清理）
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }

        // 写出二进制内置的主题静态资源（覆盖更新），再拷贝本地以覆盖
        crate::utils::write_embedded_theme_static(output_dir)?;
        let runtime_paths = crate::utils::RuntimePathsBuilder::new()
            .md_dir(md_dir)
            .theme_name(self.config.theme_name())
            .build();
        let theme_static_dir = runtime_paths.theme_static_dir;
        if theme_static_dir.exists() {
            copy_dir_recursive(&theme_static_dir, output_dir)?;
        }

        // 递归复制源目录下的所有非 Markdown 且非隐藏文件，保持相对路径（增量模式也执行，以便更新附件）
        crate::utils::copy_non_md_recursive_preserve_paths(md_dir, output_dir)?;

        // 列出所有文章（用于派生页计算）
        let posts = PostParser::list_posts(md_dir)?;

        // 首次构建时生成侧边栏数据（可手动编辑）
        crate::utils::ensure_sidebar_data(md_dir, &posts)?;

        // 读取 last_build_time -> epoch（使用 NaiveDateTime 解析，并按本地时区转换）
        let last_build_epoch: i64 = {
            let build_path = crate::utils::resolve_build_toml_path_read(md_dir);
            if !build_path.exists() {
                0
            } else if let Ok(content) = std::fs::read_to_string(&build_path) {
                if let Ok(root) = content.parse::<toml::Value>() {
                    if let Some(s) = root.get("last_build_time").and_then(|v| v.as_str()) {
                        if let Ok(naive) =
                            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                        {
                            if let Some(dt) = naive.and_local_timezone(chrono::Local).single() {
                                dt.timestamp()
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                } else {
                    0
                }
            } else {
                0
            }
        };

        // 过滤出修改过的文章
        let changed_posts: Vec<&Post> = posts
            .iter()
            .filter(|p| p.modified_epoch().unwrap_or(0) > last_build_epoch)
            .collect();

        println!(
            "检测到修改文章 {} 篇（自 {} 之后）",
            changed_posts.len(),
            if last_build_epoch == 0 {
                "初始构建".to_string()
            } else {
                chrono::Local
                    .timestamp_opt(last_build_epoch, 0)
                    .single()
                    .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| last_build_epoch.to_string())
            }
        );

        // 渲染被修改的文章
        for &post in &changed_posts {
            let post_html = self.template_engine.render_post(post, &posts)?;

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
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&out_path, post_html)
                    .map_err(|e| Error::Other(format!("无法写入文章文件 {:?}: {}", out_path, e)))?;
            }
        }

        // 计算受影响集合
        let mut changed_tags: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut changed_categories: std::collections::HashSet<Vec<String>> =
            std::collections::HashSet::new();
        let mut changed_years: std::collections::HashSet<String> = std::collections::HashSet::new();

        for p in &changed_posts {
            for t in p.tags() {
                changed_tags.insert(t.clone());
            }
            let cats = p.categories();
            if !cats.is_empty() {
                for i in 1..=cats.len() {
                    changed_categories.insert(cats[0..i].to_vec());
                }
            }
            if let Some(date) = p.date() {
                if date.len() >= 4 {
                    changed_years.insert(date[0..4].to_string());
                }
            }
        }

        // 统计全集用于首页与一些模板数据
        let all_tags = PostParser::collect_tags(&posts);
        let all_categories = PostParser::generate_hierarchical_categories(&posts);

        let posts_per_page = self
            .config
            .data
            .get("homepage")
            .and_then(|v| v.get("posts_per_page"))
            .and_then(|v| v.as_integer())
            .unwrap_or(10) as usize;
        let total_pages = (posts.len() + posts_per_page - 1) / posts_per_page;

        // 仅重建受影响的首页分页页码
        let mut posts_sorted: Vec<&Post> = posts.iter().collect();
        posts_sorted.sort_by(|a, b| {
            let da = a.date().unwrap_or("");
            let db = b.date().unwrap_or("");
            db.cmp(da)
        });
        let mut affected_pages: std::collections::HashSet<usize> = std::collections::HashSet::new();
        for cp in &changed_posts {
            let cs = cp.slug().unwrap_or("");
            let cc = cp.categories();
            if let Some(pos) = posts_sorted
                .iter()
                .position(|p| p.slug().unwrap_or("") == cs && p.categories() == cc)
            {
                let page = if total_pages == 0 {
                    1
                } else {
                    total_pages - (pos / posts_per_page)
                };
                affected_pages.insert(std::cmp::max(1, page));
            } else {
                affected_pages.insert(std::cmp::max(1, total_pages));
            }
        }
        // 如果总页数增加，需要补充新增的 indexN.html 页
        let mut existing_max_n: usize = 0;
        if let Ok(rd) = std::fs::read_dir(output_dir) {
            for entry in rd.flatten() {
                let name_os = entry.file_name();
                if let Ok(name) = name_os.into_string() {
                    if let Some(rest) = name
                        .strip_prefix("index")
                        .and_then(|s| s.strip_suffix(".html"))
                    {
                        if let Ok(n) = rest.parse::<usize>() {
                            existing_max_n = existing_max_n.max(n);
                        }
                    }
                }
            }
        }
        let expected_max_n = total_pages.saturating_sub(1);
        if expected_max_n > existing_max_n {
            for n in (existing_max_n + 1)..=expected_max_n {
                affected_pages.insert(n);
            }
        }
        if !affected_pages.is_empty() {
            self.generate_index_pages_for(
                &posts,
                &all_tags,
                &all_categories,
                &affected_pages,
                output_dir,
            )?;
        }

        // 仅生成受影响标签分页
        if !changed_tags.is_empty() {
            // 当新标签首次出现时，更新标签总览页
            let mut need_tags_overview = false;
            for tag in &changed_tags {
                let dir = output_dir.join("tags").join(tag);
                if !dir.exists() {
                    need_tags_overview = true;
                    break;
                }
            }
            if need_tags_overview {
                let tags_html = self.template_engine.render_tags(&posts, &all_tags)?;
                std::fs::write(output_dir.join("tags.html"), tags_html)
                    .map_err(|e| Error::Other(format!("无法写入标签页: {}", e)))?;
            }
            self.generate_tag_pages_for(&posts, &changed_tags, output_dir)?;
        }

        // 仅生成受影响分类分页（含祖先路径）
        if !changed_categories.is_empty() {
            self.generate_category_pages_for(&posts, &changed_categories, output_dir)?;
        }

        // 仅生成受影响年份归档页
        if !changed_years.is_empty() {
            self.generate_year_archive_pages_for(&posts, &changed_years, output_dir)?;
        }

        // 静态导航页（about、friends、search、404）在增量模式下不变更时跳过重建

        self.generate_search_index(&posts, output_dir)?;

        let rss_enabled = self
            .config
            .data
            .get("features")
            .and_then(|v| v.get("rss"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        if rss_enabled {
            self.generate_rss(&posts, output_dir)?;
        }

        let sitemap_enabled = self
            .config
            .data
            .get("features")
            .and_then(|v| v.get("sitemap"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        if sitemap_enabled {
            self.generate_sitemap(&posts, output_dir)?;
        }

        // 更细致的集合日志
        if !changed_tags.is_empty() {
            let mut tags_list: Vec<&String> = changed_tags.iter().collect();
            tags_list.sort();
            let joined = tags_list.iter().fold(String::new(), |mut acc, s| {
                if !acc.is_empty() {
                    acc.push_str(", ");
                }
                acc.push_str(s);
                acc
            });
            println!("受影响标签 {} 个：{}", tags_list.len(), joined);
        }
        if !changed_categories.is_empty() {
            let mut cat_list: Vec<String> =
                changed_categories.iter().map(|p| p.join("/")).collect();
            cat_list.sort();
            println!(
                "受影响分类路径 {} 个：{}",
                cat_list.len(),
                cat_list.join(", ")
            );
        }
        if !changed_years.is_empty() {
            let mut years_list: Vec<&String> = changed_years.iter().collect();
            years_list.sort();
            let joined = years_list.iter().fold(String::new(), |mut acc, s| {
                if !acc.is_empty() {
                    acc.push_str(", ");
                }
                acc.push_str(s);
                acc
            });
            println!("受影响年份 {} 个：{}", years_list.len(), joined);
        }
        println!("增量构建完成！已更新文章与受影响派生页。");
        Ok(())
    }
}
