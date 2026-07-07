import os
import re
from datetime import datetime, timedelta

def parse_datetime(val):
    for fmt in ("%Y-%m-%d %H:%M:%S", "%Y/%m/%d %H:%M:%S", "%Y-%m-%d", "%Y/%m/%d"):
        try:
            return datetime.strptime(val.strip(), fmt)
        except ValueError:
            pass
    return None

def main():
    source_dir = "/workspace/rustpress/source"
    print("=== 开始调整 source 目录文章的 createTime ===")

    # 1. 扫描所有年份目录下的 markdown 文件
    # 格式如：source/{Year}/...
    year_groups = {}
    
    for root, dirs, files in os.walk(source_dir):
        for file in files:
            if file.endswith(".md"):
                path = os.path.join(root, file)
                # 检查是否在年份目录下
                rel_path = os.path.relpath(path, source_dir)
                parts = rel_path.split(os.sep)
                if len(parts) >= 2 and re.match(r"^\d{4}$", parts[0]):
                    year = int(parts[0])
                    year_groups.setdefault(year, []).append(path)

    # 2. 依次处理每个年份组
    for year, paths in sorted(year_groups.items()):
        print(f"\n处理年份目录: source/{year} (共 {len(paths)} 篇文章)...")
        
        # 解析每篇文章的当前时间，以便排序
        parsed_posts = []
        for path in paths:
            with open(path, "r", encoding="utf-8") as f:
                content = f.read()
            
            fm_match = re.match(r"^---\s*\n(.*?)\n---\s*\n", content, re.DOTALL)
            ctime = None
            if fm_match:
                fm = fm_match.group(1)
                ctime_line = re.search(r"createTime:\s*([^\n]+)", fm)
                if ctime_line:
                    ctime = ctime_line.group(1).strip().replace("\"", "").replace("'", "")
            
            dt = parse_datetime(ctime) if ctime else None
            # 如果没有解析到时间，使用文件名或修改时间作为降级排序依据
            sort_key = dt if dt else datetime.fromtimestamp(os.path.getmtime(path))
            parsed_posts.append({
                "path": path,
                "filename": os.path.basename(path),
                "original_dt": sort_key,
                "content": content
            })
            
        # 根据原始时间排序（如果是 2008 年且文件名是 01.md, 02.md，则按文件名数字排序）
        if year == 2008:
            def sort_2008(p):
                name_match = re.match(r"^(\d+)\.md$", p["filename"])
                if name_match:
                    return int(name_match.group(1))
                return 0
            parsed_posts.sort(key=sort_2008)
        else:
            parsed_posts.sort(key=lambda x: (x["original_dt"], x["path"]))

        # 3. 分配新的 createTime
        n = len(parsed_posts)
        if n == 0:
            continue
            
        # 根据年份确定起止时间
        if year == 2026:
            # 2026年，上限为当前时间 (2026-07-06 12:00:00)，避免产生未来的文章
            start_dt = datetime(2026, 1, 1, 9, 0, 0)
            end_dt = datetime(2026, 7, 6, 12, 0, 0)
        else:
            start_dt = datetime(year, 1, 1, 9, 0, 0)
            end_dt = datetime(year, 12, 31, 18, 0, 0)

        # 针对 2008 的特殊优化：01.md -> 01月，02.md -> 02月
        if year == 2008:
            for post in parsed_posts:
                name_match = re.match(r"^(\d+)\.md$", post["filename"])
                if name_match:
                    month = int(name_match.group(1))
                    if 1 <= month <= 12:
                        new_dt = datetime(2008, month, 1, 9, 0, 0)
                    else:
                        new_dt = datetime(2008, 5, 1, 9, 0, 0)
                else:
                    new_dt = datetime(2008, 5, 1, 9, 0, 0)
                post["new_dt"] = new_dt
        else:
            if n == 1:
                parsed_posts[0]["new_dt"] = start_dt
            else:
                total_seconds = (end_dt - start_dt).total_seconds()
                interval = total_seconds / (n - 1)
                for i, post in enumerate(parsed_posts):
                    post["new_dt"] = start_dt + timedelta(seconds=i * interval)

        # 4. 写回文件
        for post in parsed_posts:
            path = post["path"]
            content = post["content"]
            new_dt = post["new_dt"]
            new_ctime_str = new_dt.strftime("%Y/%m/%d %H:%M:%S")
            
            # 替换或添加 frontmatter 中的 createTime
            fm_match = re.match(r"^---\s*\n(.*?)\n---\s*\n", content, re.DOTALL)
            if fm_match:
                fm = fm_match.group(1)
                if "createTime" in fm:
                    # 替换已有的 createTime
                    new_fm = re.sub(r"createTime:\s*[^\n]+", f"createTime: {new_ctime_str}", fm)
                else:
                    # 添加 createTime
                    new_fm = fm + f"\ncreateTime: {new_ctime_str}"
                
                new_content = content[:fm_match.start(1)] + new_fm + content[fm_match.end(1):]
                
                with open(path, "w", encoding="utf-8") as f:
                    f.write(new_content)
                print(f"  - 修改: {os.path.basename(path)} -> createTime: {new_ctime_str}")

if __name__ == "__main__":
    main()
