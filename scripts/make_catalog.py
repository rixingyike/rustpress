#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
make_catalog.py - 自动将专栏目录下的 Markdown 文件按创建时间排序并更新到 README.md 的 frontmatter catalog 中。

规则：
1. 提取当前专栏目录下的所有 .md 文件（排除 README.md / readme.md）。
2. 保留 README.md frontmatter catalog 中已有的文件顺序（防止覆盖作者手动调整的次序）。
3. 对未在 catalog 中的新 md 文件，按创建时间（优先读取 frontmatter 中的 createTime/date，若无则使用文件系统诞生时间 birthtime / mtime）排序，并追加到 catalog 列表末尾。
4. 写回 README.md，保持原有 frontmatter 格式与正文内容不变。
"""

import os
import sys
import re
import argparse
from datetime import datetime


def natural_sort_key(s: str):
    """自然排序键（例如 '2.md' 会排在 '10.md' 前面）"""
    return [int(text) if text.isdigit() else text.lower() for text in re.split(r'(\d+)', s)]


def get_file_creation_time(file_path: str) -> float:
    """
    获取文件的创建时间戳：
    1. 优先解析 Markdown frontmatter 中的 createTime / date / create_time 字段
    2. 若未解析到，则尝试获取 macOS / 文件系统的 st_birthtime
    3. 若不支持 birthtime，降级使用 st_mtime
    """
    # 1. 尝试从 frontmatter 解析
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            header = f.read(2048)
        fm_m = re.match(r'^---\s*\n(.*?)\n---', header, re.DOTALL)
        if fm_m:
            fm = fm_m.group(1)
            for key in ['createTime', 'date', 'create_time', 'created']:
                m = re.search(rf'^{key}:\s*["\']?([^"\'\n\r]+)["\']?', fm, re.MULTILINE)
                if m:
                    time_str = m.group(1).strip()
                    for fmt in (
                        '%Y/%m/%d %H:%M:%S',
                        '%Y-%m-%d %H:%M:%S',
                        '%Y/%m/%d %H:%M',
                        '%Y-%m-%d %H:%M',
                        '%Y/%m/%d',
                        '%Y-%m-%d',
                    ):
                        try:
                            return datetime.strptime(time_str, fmt).timestamp()
                        except ValueError:
                            pass
    except Exception:
        pass

    # 2. 文件系统时间
    try:
        st = os.stat(file_path)
        birthtime = getattr(st, 'st_birthtime', None)
        if birthtime and birthtime > 0:
            return birthtime
        return st.st_mtime
    except Exception:
        return 0.0


def parse_catalog_block(fm_text: str):
    """
    解析 frontmatter 中的 catalog 列表。
    返回: (catalog_items: list[str], start_char_index: int, end_char_index: int)
    若不存在 catalog 键，则返回 ([], -1, -1)
    """
    lines = fm_text.splitlines(keepends=True)
    catalog_line_idx = -1
    for idx, line in enumerate(lines):
        if re.match(r'^\s*catalog\s*:', line):
            catalog_line_idx = idx
            break

    if catalog_line_idx == -1:
        return [], -1, -1

    cat_line = lines[catalog_line_idx]
    after_colon = cat_line.split(':', 1)[1].strip()

    items = []
    end_line_idx = catalog_line_idx + 1

    if after_colon.startswith('[') and after_colon.endswith(']'):
        # 行内列表形式: catalog: ["a.md", "b.md"]
        raw_items = re.findall(r'["\']([^"\']+)["\']|([^,\[\]\s]+)', after_colon)
        items = [m[0] or m[1] for m in raw_items if m[0] or m[1]]
    else:
        # 多行列表形式
        while end_line_idx < len(lines):
            line = lines[end_line_idx]
            if line.strip() == '':
                end_line_idx += 1
                continue
            if re.match(r'^[ \t]+-', line):
                val_match = re.search(r'^[ \t]+-[ \t]*["\']?(.*?)["\']?\s*$', line)
                if val_match:
                    item_val = val_match.group(1).strip()
                    if (item_val.startswith('"') and item_val.endswith('"')) or (
                        item_val.startswith("'") and item_val.endswith("'")
                    ):
                        item_val = item_val[1:-1]
                    items.append(item_val)
                end_line_idx += 1
            elif re.match(r'^[ \t]+', line):
                end_line_idx += 1
            else:
                break

    start_pos = sum(len(l) for l in lines[:catalog_line_idx])
    end_pos = sum(len(l) for l in lines[:end_line_idx])
    return items, start_pos, end_pos


def format_catalog_yaml(catalog_items: list[str]) -> str:
    """生成 YAML 格式的 catalog 代码块"""
    lines = ["catalog:"]
    for item in catalog_items:
        clean_item = item.strip().strip('"').strip("'")
        escaped_item = clean_item.replace('"', '\\"')
        lines.append(f'  - "{escaped_item}"')
    return "\n".join(lines) + "\n"


def process_column_directory(column_dir: str, dry_run: bool = False, verbose: bool = False) -> bool:
    """
    处理单个专栏目录。
    返回是否进行了修改 (True/False)。
    """
    column_dir = os.path.abspath(column_dir)
    if not os.path.isdir(column_dir):
        print(f"❌ 目录不存在: {column_dir}", file=sys.stderr)
        return False

    # 寻找 README.md
    readme_name = None
    for name in os.listdir(column_dir):
        if name.lower() == 'readme.md':
            readme_name = name
            break

    if not readme_name:
        print(f"⚠️ 目录下未找到 README.md，跳过: {column_dir}")
        return False

    readme_path = os.path.join(column_dir, readme_name)
    with open(readme_path, 'r', encoding='utf-8') as f:
        readme_content = f.read()

    # 匹配 frontmatter
    fm_match = re.match(r'^---\r?\n(.*?)\r?\n---\r?\n?(.*)$', readme_content, re.DOTALL)
    if not fm_match:
        print(f"⚠️ {readme_path} 没有有效的 frontmatter 结构，跳过。")
        return False

    fm_text = fm_match.group(1)
    body_text = fm_match.group(2) or ''

    existing_catalog, s_pos, e_pos = parse_catalog_block(fm_text)

    # 收集当前目录下所有非 README 的 .md 文件
    all_files = os.listdir(column_dir)
    md_files = [
        f for f in all_files
        if f.endswith('.md') and f.lower() != 'readme.md' and not os.path.isdir(os.path.join(column_dir, f))
    ]

    if not md_files:
        if verbose:
            print(f"ℹ️ {os.path.basename(column_dir)}: 没有额外的 .md 文章文件。")
        return False

    # 查找尚未在 catalog 中的文件（保持原有 catalog 中的文件及其顺序不动）
    existing_set = set(existing_catalog)
    new_files = [f for f in md_files if f not in existing_set]

    if not new_files:
        print(f"✅ {os.path.basename(column_dir)}: catalog 已经是最新的（包含 {len(existing_catalog)} 篇文章）。")
        return False

    # 对新文件按照创建时间 + 自然文件名排序
    new_files.sort(key=lambda f: (
        get_file_creation_time(os.path.join(column_dir, f)),
        natural_sort_key(f)
    ))

    final_catalog = list(existing_catalog) + new_files
    new_catalog_yaml = format_catalog_yaml(final_catalog)

    if s_pos != -1 and e_pos != -1:
        # 替换已有 catalog 块
        new_fm_text = fm_text[:s_pos] + new_catalog_yaml + fm_text[e_pos:]
    else:
        # 没有 catalog，在 createTime 前插入，或追加在 frontmatter 末尾
        ctime_match = re.search(r'(^|\n)(createTime:)', fm_text)
        if ctime_match:
            insert_idx = ctime_match.start(2)
            new_fm_text = fm_text[:insert_idx] + new_catalog_yaml + fm_text[insert_idx:]
        else:
            new_fm_text = fm_text.rstrip() + '\n' + new_catalog_yaml

    # 组合新 README 内容
    new_readme_content = f"---\n{new_fm_text.strip()}\n---\n{body_text}"

    print(f"📝 专栏 [{os.path.basename(column_dir)}] 目录更新:")
    if existing_catalog:
        print(f"   - 保留已有目录 ({len(existing_catalog)} 篇): {existing_catalog}")
    print(f"   - 新增文章 ({len(new_files)} 篇，按创建时间排序): {new_files}")
    print(f"   - 更新后总计: {len(final_catalog)} 篇")

    if dry_run:
        print("   [Dry-run] 未写入文件。")
    else:
        with open(readme_path, 'w', encoding='utf-8') as f:
            f.write(new_readme_content)
        print(f"   ✅ 已成功写回 {readme_path}")

    return True


def find_column_dirs(target_path: str, repo_root: str = None):
    """寻找专栏目录列表（支持专栏编号、相对路径、绝对路径及智能模糊/别名匹配）"""
    candidates = [target_path]
    if repo_root:
        candidates.append(os.path.join(repo_root, target_path))
        candidates.append(os.path.join(repo_root, "source", "columns", target_path))
        candidates.append(os.path.join(repo_root, "source", target_path))

    resolved_path = None
    for cand in candidates:
        abs_p = os.path.abspath(cand)
        if os.path.exists(abs_p):
            resolved_path = abs_p
            break

    # 若未直接找到，尝试在 source/columns 下进行智能模糊与元数据匹配
    if not resolved_path and repo_root:
        columns_root = os.path.join(repo_root, "source", "columns")
        if os.path.exists(columns_root):
            target_clean = target_path.strip().lower().replace("-", "").replace("_", "")
            for item in os.listdir(columns_root):
                sub_p = os.path.join(columns_root, item)
                if not os.path.isdir(sub_p) or item.startswith('.'):
                    continue
                item_clean = item.lower().replace("-", "").replace("_", "")
                
                # 1. 目录名容错匹配 (例如 rustpress 与 rustpess)
                if target_clean in item_clean or item_clean in target_clean:
                    resolved_path = sub_p
                    break
                
                # 2. README.md 元数据深度匹配 (title / product_id / tags)
                readme_file = os.path.join(sub_p, "README.md")
                if os.path.exists(readme_file):
                    try:
                        with open(readme_file, "r", encoding="utf-8") as f:
                            c = f.read()
                        if target_clean in c.lower().replace("-", "").replace("_", ""):
                            resolved_path = sub_p
                            break
                    except Exception:
                        pass

    if not resolved_path:
        return []

    abs_path = resolved_path

    # 如果目标目录直接包含 README.md 和其他 md 文件，它本身就是一个专栏目录
    if os.path.isdir(abs_path):
        has_readme = any(f.lower() == 'readme.md' for f in os.listdir(abs_path))
        other_mds = [f for f in os.listdir(abs_path) if f.endswith('.md') and f.lower() != 'readme.md']
        if has_readme and other_mds:
            return [abs_path]

        # 检查子目录是否为专栏目录
        sub_columns = []
        for item in sorted(os.listdir(abs_path)):
            sub_path = os.path.join(abs_path, item)
            if os.path.isdir(sub_path) and item != 'assets' and not item.startswith('.'):
                if any(f.lower() == 'readme.md' for f in os.listdir(sub_path)):
                    sub_columns.append(sub_path)

        if sub_columns:
            return sub_columns

        if has_readme:
            return [abs_path]

    return []


def main():
    parser = argparse.ArgumentParser(
        description="自动将专栏目录下的 Markdown 文章按创建时间排序并写入 README.md 的 frontmatter catalog 中。"
    )
    parser.add_argument(
        "paths",
        nargs="*",
        help="专栏目录路径（例如 source/columns/1 或 source/columns）。若不指定，则自动扫描 source/columns。"
    )
    parser.add_argument(
        "-a", "--all",
        action="store_true",
        help="扫描并更新 source/columns 下的所有专栏"
    )
    parser.add_argument(
        "-n", "--dry-run",
        action="store_true",
        help="演练模式，仅打印变更，不实际写入文件"
    )
    parser.add_argument(
        "-v", "--verbose",
        action="store_true",
        help="输出详细日志"
    )

    args = parser.parse_args()

    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    default_columns_dir = os.path.join(repo_root, "source", "columns")

    target_paths = args.paths
    if not target_paths:
        if args.all or os.path.exists(default_columns_dir):
            target_paths = [default_columns_dir]
        else:
            target_paths = ["."]

    column_dirs = []
    for p in target_paths:
        found = find_column_dirs(p, repo_root=repo_root)
        if found:
            column_dirs.extend(found)
        else:
            print(f"⚠️ 未找到有效专栏目录: {p}", file=sys.stderr)

    # 去重
    seen = set()
    unique_columns = []
    for cd in column_dirs:
        if cd not in seen:
            seen.add(cd)
            unique_columns.append(cd)

    if not unique_columns:
        print("未找到任何专栏目录可供处理。")
        sys.exit(0)

    print(f"🚀 开始检查/更新专栏 catalog (共 {len(unique_columns)} 个专栏)...")
    updated_count = 0
    for cd in unique_columns:
        changed = process_column_directory(cd, dry_run=args.dry_run, verbose=args.verbose)
        if changed:
            updated_count += 1

    print(f"\n✨ 处理完成！共检查 {len(unique_columns)} 个专栏，更新了 {updated_count} 个专栏。")


if __name__ == "__main__":
    main()
