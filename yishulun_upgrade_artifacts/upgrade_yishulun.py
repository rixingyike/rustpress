import os
import shutil
import re

def merge_dirs(src, dst):
    """递归合并两个目录，防止覆盖已有的用户内容"""
    if not os.path.exists(dst):
        os.makedirs(dst, exist_ok=True)
    for item in os.listdir(src):
        s = os.path.join(src, item)
        d = os.path.join(dst, item)
        if os.path.isdir(s):
            merge_dirs(s, d)
        else:
            if not os.path.exists(d):
                shutil.move(s, d)
                print(f"    - 移动: {s} -> {d}")
            else:
                print(f"    - 跳过冲突文件: {d} 已存在")

def clean_empty_dir(path):
    """安全递归删除空目录"""
    if os.path.exists(path) and os.path.isdir(path):
        for root, dirs, files in os.walk(path, topdown=False):
            for d in dirs:
                dir_path = os.path.join(root, d)
                try:
                    if not os.listdir(dir_path):
                        os.rmdir(dir_path)
                        print(f"    - 清理空目录: {dir_path}")
                except Exception:
                    pass
        try:
            if not os.listdir(path):
                os.rmdir(path)
                print(f"    - 清理根空目录: {path}")
        except Exception:
            pass

def main():
    print("=== 开始升级 yishulun_mdsrc 博客源码架构 ===")

    # 0. 覆盖复制 config.toml 与 prod.sh
    print("\n0. 正在复制最新的 config.toml 配置文件与 prod.sh 启动脚本...")
    script_dir = os.path.dirname(os.path.abspath(__file__))
    
    # 复制 config.toml
    source_config = os.path.join(script_dir, "config.toml")
    target_config = "source/config.toml"
    if os.path.exists(source_config):
        shutil.copy(source_config, target_config)
        print("  - 已成功将最新的 config.toml 复制覆盖至 source/config.toml")
    else:
        print("  - 警告: 未在当前脚本目录下找到 config.toml，跳过覆盖")
        
    # 复制 prod.sh
    source_prod = os.path.join(script_dir, "prod.sh")
    target_prod = "prod.sh"
    if os.path.exists(source_prod):
        shutil.copy(source_prod, target_prod)
        os.chmod(target_prod, 0o755)
        print("  - 已成功将最新的 prod.sh 复制覆盖至根目录并赋权")
    else:
        print("  - 警告: 未在当前脚本目录下找到 prod.sh，跳过覆盖")

    # 1. 升级 blog/ 下的年份目录至根目录
    print("\n1. 正在将 source/blog/ 下的年份目录提至 source/ 根层...")
    blog_dir = "source/blog"
    if os.path.exists(blog_dir):
        for item in os.listdir(blog_dir):
            if re.match(r"^\d{4}$", item):  # 匹配年份，如 2026
                src_year = os.path.join(blog_dir, item)
                dst_year = os.path.join("source", item, "blog")
                print(f"  - 合并: {src_year} -> {dst_year}")
                merge_dirs(src_year, dst_year)
        clean_empty_dir(blog_dir)

    # 2. 升级 posts/ 下的年份目录至根目录
    print("\n2. 正在将 source/posts/ 下的年份目录提至 source/ 根层...")
    posts_dir = "source/posts"
    if os.path.exists(posts_dir):
        for item in os.listdir(posts_dir):
            if re.match(r"^\d{4}$", item):  # 匹配年份，如 2026
                src_year = os.path.join(posts_dir, item)
                dst_year = os.path.join("source", item, "posts")
                print(f"  - 合并: {src_year} -> {dst_year}")
                merge_dirs(src_year, dst_year)
        clean_empty_dir(posts_dir)

    # 3. 将 docs/ 目录重命名为 columns/ 并整理封面图片
    print("\n3. 正在升级 docs 目录为 columns 并整理专栏封面...")
    old_docs_dir = "source/docs"
    columns_dir = "source/columns"
    if os.path.exists(old_docs_dir):
        if not os.path.exists(columns_dir):
            shutil.move(old_docs_dir, columns_dir)
            print("  - 已将 source/docs 文件夹重命名为 source/columns")
        else:
            merge_dirs(old_docs_dir, columns_dir)
            clean_empty_dir(old_docs_dir)

    # 4. 专栏内部封面迁移与 Frontmatter 修复
    if os.path.exists(columns_dir):
        for item in os.listdir(columns_dir):
            col_path = os.path.join(columns_dir, item)
            if os.path.isdir(col_path) and re.match(r"^\d+$", item):
                assets_dir = os.path.join(col_path, "assets")
                os.makedirs(assets_dir, exist_ok=True)
                
                # 移动 cover.png -> assets/cover.png
                old_cover = os.path.join(col_path, "cover.png")
                new_cover = os.path.join(assets_dir, "cover.png")
                if os.path.exists(old_cover):
                    shutil.move(old_cover, new_cover)
                    print(f"  - 专栏 {item}: 封面图移动至 assets/")
                
                # 更新 README.md frontmatter cover 属性
                readme = os.path.join(col_path, "README.md")
                if os.path.exists(readme):
                    with open(readme, "r", encoding="utf-8") as f:
                        txt = f.read()
                    # 替换 cover 属性
                    txt = re.sub(
                        r'cover:\s*"/docs/(\d+)/cover\.png"',
                        f'cover: "/columns/{item}/assets/cover.png"',
                        txt
                    )
                    txt = re.sub(
                        r'cover:\s*"/columns/(\d+)/cover\.png"',
                        f'cover: "/columns/{item}/assets/cover.png"',
                        txt
                    )
                    with open(readme, "w", encoding="utf-8") as f:
                        f.write(txt)
                    print(f"  - 专栏 {item}: 更新了 README.md 里的 cover 配置")

    # 5. docs.md -> columns/README.md 改造
    print("\n5. 正在将 docs.md 改造为 columns/README.md...")
    old_docs_md = "source/docs.md"
    if os.path.exists(old_docs_md):
        os.makedirs(columns_dir, exist_ok=True)
        new_columns_readme = os.path.join(columns_dir, "README.md")
        with open(old_docs_md, "r", encoding="utf-8") as f:
            content = f.read()
        
        # 将 layout: docs 改为 layout: columns
        content = re.sub(r'layout:\s*docs\b', 'layout: columns', content)
        with open(new_columns_readme, "w", encoding="utf-8") as f:
            f.write(content)
        os.remove(old_docs_md)
        print("  - 已成功改造：source/docs.md -> source/columns/README.md (layout 变更为 columns)")

    # 6. friends.md -> friends/README.md 改造
    print("\n6. 正在将 friends.md 改造为 friends/README.md...")
    friends_md = "source/friends.md"
    friends_dir = "source/friends"
    if os.path.exists(friends_md):
        os.makedirs(friends_dir, exist_ok=True)
        new_friends_readme = os.path.join(friends_dir, "README.md")
        with open(friends_md, "r", encoding="utf-8") as f:
            content = f.read()
        
        # 确保 layout 为 friends
        content = re.sub(r'layout:\s*\w+', 'layout: friends', content)
        with open(new_friends_readme, "w", encoding="utf-8") as f:
            f.write(content)
        os.remove(friends_md)
        print("  - 已成功改造：source/friends.md -> source/friends/README.md (layout 变更为 friends)")

    # 7. projects.md -> projects/README.md 改造
    print("\n7. 正在将 projects.md 改造为 projects/README.md...")
    projects_md = "source/projects.md"
    projects_dir = "source/projects"
    if os.path.exists(projects_md):
        os.makedirs(projects_dir, exist_ok=True)
        new_projects_readme = os.path.join(projects_dir, "README.md")
        with open(projects_md, "r", encoding="utf-8") as f:
            content = f.read()
        
        # 确保 layout 为 projects
        content = re.sub(r'layout:\s*\w+', 'layout: projects', content)
        with open(new_projects_readme, "w", encoding="utf-8") as f:
            f.write(content)
        os.remove(projects_md)
        print("  - 已成功改造：source/projects.md -> source/projects/README.md (layout 变更为 projects)")

    # 8. 著作 (Works) 架构改造
    print("\n8. 正在对著作进行架构整理...")
    works_dir = "source/works"
    if os.path.exists(works_dir):
        works_assets = os.path.join(works_dir, "assets")
        os.makedirs(works_assets, exist_ok=True)
        for i in range(1, 10):
            old_work_dir = os.path.join(works_dir, str(i))
            if os.path.exists(old_work_dir):
                # 移动 README.md -> i.md
                old_readme = os.path.join(old_work_dir, "README.md")
                new_md = os.path.join(works_dir, f"{i}.md")
                if os.path.exists(old_readme):
                    shutil.move(old_readme, new_md)
                    print(f"  - 著作 {i}: 页面文件已移动并重命名为 source/works/{i}.md")
                
                # 移动封面
                for ext in ["png", "jpg", "jpeg"]:
                    old_cover = os.path.join(old_work_dir, f"cover.{ext}")
                    if os.path.exists(old_cover):
                        shutil.move(old_cover, os.path.join(works_assets, f"{i}.{ext}"))
                        print(f"  - 著作 {i}: 封面图已移动并规范命名为 works/assets/{i}.{ext}")
                        break
                
                # 清理空子目录
                try:
                    os.rmdir(old_work_dir)
                except Exception:
                    pass

    # 9. 项目 (Projects) 详情与资产整理
    print("\n9. 正在整理项目内页及局部资产...")
    if os.path.exists(projects_dir):
        # 9.1 恩言
        enyan_md = os.path.join(projects_dir, "enyan.md")
        proj1_dir = os.path.join(projects_dir, "1")
        if os.path.exists(enyan_md):
            os.makedirs(proj1_dir, exist_ok=True)
            new_readme = os.path.join(proj1_dir, "README.md")
            shutil.move(enyan_md, new_readme)
            print("  - 项目 1 (恩言): md 主页已移动至 projects/1/README.md")
            
            with open(new_readme, "r", encoding="utf-8") as f:
                txt = f.read()
            txt = txt.replace("/assets/enyan/", "/projects/1/assets/")
            with open(new_readme, "w", encoding="utf-8") as f:
                f.write(txt)
            print("  - 项目 1 (恩言): Frontmatter 图片资产指向路径已更新")
                
            old_assets = "source/assets/enyan"
            new_assets = os.path.join(proj1_dir, "assets")
            if os.path.exists(old_assets):
                shutil.move(old_assets, new_assets)
                print("  - 项目 1 (恩言): 图片截图资产已全部归入 projects/1/assets/ 中")
                
        # 9.2 RustPress
        rustpress_md = os.path.join(projects_dir, "rustpress.md")
        proj2_dir = os.path.join(projects_dir, "2")
        if os.path.exists(rustpress_md):
            os.makedirs(proj2_dir, exist_ok=True)
            new_readme = os.path.join(proj2_dir, "README.md")
            shutil.move(rustpress_md, new_readme)
            print("  - 项目 2 (RustPress): md 主页已移动至 projects/2/README.md")
            
            with open(new_readme, "r", encoding="utf-8") as f:
                txt = f.read()
            txt = txt.replace("/assets/rustpress_logo.png", "/projects/2/assets/rustpress_logo.png")
            with open(new_readme, "w", encoding="utf-8") as f:
                f.write(txt)
            print("  - 项目 2 (RustPress): Frontmatter 图标配置已更新")
                
            old_logo = "source/assets/rustpress_logo.png"
            new_assets = os.path.join(proj2_dir, "assets")
            os.makedirs(new_assets, exist_ok=True)
            if os.path.exists(old_logo):
                shutil.move(old_logo, os.path.join(new_assets, "rustpress_logo.png"))
                print("  - 项目 2 (RustPress): Logo 图标已成功移动至 projects/2/assets/ 中")

    print("\n=== 所有升级与迁移工作已全部在保护内容的前提下安全完成！ ===")

if __name__ == "__main__":
    main()
