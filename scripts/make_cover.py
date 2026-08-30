#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
make_cover.py - 专栏封面图片 (cover.png) 自动化生成脚本。

特性与规则：
1. 智能提取专栏信息：
   - 标题：优先从 README.md frontmatter 中的 title 提取，若无则取 markdown 首个 # 标题。
   - 作者与站点署名：优先从 README.md frontmatter 的 author 提取，次之从 config.toml 中读取 [site.name] · [site.author]，亦支持命令行 --author 自定义。
   - 封面路径：读取 frontmatter 中的 cover 字段（默认 assets/cover.png），若缺失则自动在 frontmatter 中补齐。
2. 典雅设计与站点主题协调（支持两种风格）：
   - theme（默认）：根据专栏序号自适应 8 组高雅浅色底板，深墨色 (moss-ink) 标题，薄荷绿 (cyber-mint) 装饰线，底部作者与站点署名。
   - red（故宫红渐变）：故宫红上下渐变背景，白色标题与作者署名。
3. 灵活的执行模式：
   - 支持指定专栏序号（./make_cover.sh 1）、指定路径（./make_cover.sh source/columns/1）、或批量一键生成全部专栏封面。
   - 支持 --dry-run 演练预览。
"""

import os
import sys
import re
import hashlib
import argparse
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

try:
    import toml
except ImportError:
    toml = None

# 主题浅色调色板（与站点风格一致）
PALETTES = [
    (245, 247, 245),    # 0: 极浅薄荷白
    (232, 240, 236),    # 1: 浅薄荷绿灰
    (240, 244, 248),    # 2: 浅云蓝白
    (248, 245, 240),    # 3: 暖米白
    (240, 248, 244),    # 4: 清水绿
    (245, 240, 250),    # 5: 淡薰衣草
    (248, 242, 238),    # 6: 浅杏色
    (238, 245, 248),    # 7: 浅冰蓝
]

FG_COLOR = (24, 36, 34)       # 深墨前景色 (moss-ink)
FG_LIGHT = (72, 100, 96)      # 次级文字颜色 (mist-green)
ACCENT   = (0, 195, 145)      # 薄荷绿装饰线 (cyber-mint)

# 故宫红渐变色配置
RED_GRADIENT_TOP = (193, 34, 42)
RED_GRADIENT_BOTTOM = (220, 60, 50)


def get_palette_by_id(column_id: str):
    """根据专栏 ID 选取对应的背景色"""
    try:
        idx = int(column_id) % len(PALETTES)
    except (ValueError, TypeError):
        h = hashlib.md5(str(column_id).encode('utf-8')).hexdigest()
        idx = int(h, 16) % len(PALETTES)
    return PALETTES[idx]


def get_system_font(size: int, bold: bool = True):
    """加载跨平台的优质中文字体"""
    if bold:
        paths = [
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/STHeiti Medium.ttc",
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/Library/Fonts/Songti.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Bold.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/Library/Fonts/Arial Unicode.ttf",
        ]
    else:
        paths = [
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/System/Library/Fonts/PingFang.ttc",
            "/Library/Fonts/Songti.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/Library/Fonts/Arial Unicode.ttf",
        ]

    for p in paths:
        if os.path.exists(p):
            try:
                return ImageFont.truetype(p, size)
            except Exception:
                pass
    return ImageFont.load_default()


def wrap_text(text: str, max_width: int, font: ImageFont.ImageFont, draw: ImageDraw.ImageDraw):
    """根据字形宽度自适应换行"""
    lines = []
    current_line = ""
    for char in text:
        test_line = current_line + char
        bbox = draw.textbbox((0, 0), test_line, font=font)
        text_width = bbox[2] - bbox[0]
        if text_width > max_width:
            if current_line:
                lines.append(current_line)
                current_line = char
            else:
                lines.append(char)
        else:
            current_line = test_line
    if current_line:
        lines.append(current_line)
    return lines


def read_site_config(repo_root: str):
    """从 config.toml 中读取站点名称与作者信息"""
    config_paths = [
        os.path.join(repo_root, "source", "config.toml"),
        os.path.join(repo_root, "config.toml"),
    ]
    site_name = "一树仑"
    author_name = "金石碼农"

    for cp in config_paths:
        if os.path.exists(cp):
            try:
                with open(cp, "r", encoding="utf-8") as f:
                    content = f.read()
                # 优先使用 toml 库解析，若无则正则提取
                if toml is not None:
                    data = toml.loads(content)
                    site_dict = data.get("site", {})
                    author_dict = data.get("author", {})
                    site_name = site_dict.get("name") or site_name
                    author_name = site_dict.get("author") or author_dict.get("name") or author_name
                else:
                    m_site = re.search(r'^\s*name\s*=\s*["\']([^"\']+)["\']', content, re.MULTILINE)
                    m_auth = re.search(r'^\s*author\s*=\s*["\']([^"\']+)["\']', content, re.MULTILINE)
                    if m_site:
                        site_name = m_site.group(1).strip()
                    if m_auth:
                        author_name = m_auth.group(1).strip()
                break
            except Exception:
                pass

    return site_name, author_name


def draw_cover_image(
    title: str,
    author_text: str,
    column_id: str,
    output_path: str,
    style: str = "theme",
    width: int = 800,
    height: int = 450,
):
    """
    绘制完整的专栏封面图片：
    包含专栏标题、品牌装饰线、底部作者与站点署名
    """
    # 1. 创建背景
    if style == "red":
        img = Image.new("RGB", (width, height))
        for y in range(height):
            ratio = y / height
            r = int(RED_GRADIENT_TOP[0] + (RED_GRADIENT_BOTTOM[0] - RED_GRADIENT_TOP[0]) * ratio)
            g = int(RED_GRADIENT_TOP[1] + (RED_GRADIENT_BOTTOM[1] - RED_GRADIENT_TOP[1]) * ratio)
            b = int(RED_GRADIENT_TOP[2] + (RED_GRADIENT_BOTTOM[2] - RED_GRADIENT_TOP[2]) * ratio)
            for x in range(width):
                img.putpixel((x, y), (r, g, b))
        fg_color = (255, 255, 255)
        accent_color = (255, 255, 255)
        footer_color = (240, 240, 240)
    else:
        bg_color = get_palette_by_id(column_id)
        img = Image.new("RGB", (width, height), bg_color)
        fg_color = FG_COLOR
        accent_color = ACCENT
        footer_color = FG_LIGHT

    draw = ImageDraw.Draw(img)

    # 2. 计算标题字体与排版
    font_sizes = [56, 48, 42, 36, 30]
    max_w = width - 120
    chosen_font = None
    chosen_lines = []

    for sz in font_sizes:
        f = get_system_font(sz, bold=True)
        lines = wrap_text(title, max_w, f, draw)
        if len(lines) <= 2:
            chosen_font = f
            chosen_lines = lines
            break

    if not chosen_font:
        chosen_font = get_system_font(30, bold=True)
        chosen_lines = wrap_text(title, max_w, chosen_font, draw)[:3]

    sample_bbox = draw.textbbox((0, 0), "测", font=chosen_font)
    font_h = sample_bbox[3] - sample_bbox[1]
    line_h = int(font_h * 1.4)
    total_title_h = len(chosen_lines) * line_h

    # 垂直居中开始位置（预留底部作者署名空间）
    title_start_y = (height - total_title_h - 80) // 2 + 10

    # 绘制标题文字（水平居中）
    for idx, line in enumerate(chosen_lines):
        bbox = draw.textbbox((0, 0), line, font=chosen_font)
        line_w = bbox[2] - bbox[0]
        x = (width - line_w) // 2
        y = title_start_y + idx * line_h
        draw.text((x, y), line, fill=fg_color, font=chosen_font)

    # 3. 绘制装饰分割线
    line_y = title_start_y + total_title_h + 18
    line_len = 100
    draw.line(
        [((width - line_len) // 2, line_y), ((width + line_len) // 2, line_y)],
        fill=accent_color,
        width=3,
    )

    # 4. 绘制底部作者与站点署名
    if author_text:
        footer_font = get_system_font(18, bold=False)
        bbox = draw.textbbox((0, 0), author_text, font=footer_font)
        footer_w = bbox[2] - bbox[0]
        draw.text(
            ((width - footer_w) // 2, height - 60),
            author_text,
            fill=footer_color,
            font=footer_font,
        )

    # 5. 确保目录并保存图片
    os.makedirs(os.path.dirname(os.path.abspath(output_path)), exist_ok=True)
    img.save(output_path, "PNG")
    return output_path


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

    if os.path.isdir(abs_path):
        has_readme = any(f.lower() == 'readme.md' for f in os.listdir(abs_path))
        other_mds = [f for f in os.listdir(abs_path) if f.endswith('.md') and f.lower() != 'readme.md']
        if has_readme and other_mds:
            return [abs_path]

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


def extract_column_info(readme_path: str):
    """从专栏 README.md 中提取标题、作者和封面路径"""
    with open(readme_path, 'r', encoding='utf-8') as f:
        content = f.read()

    title = None
    author = None
    cover = None

    fm_match = re.match(r'^---\r?\n(.*?)\r?\n---', content, re.DOTALL)
    if fm_match:
        fm = fm_match.group(1)
        t_m = re.search(r'^title:\s*["\']?(.*?)["\']?\s*$', fm, re.MULTILINE)
        if t_m:
            title = t_m.group(1).strip()
        a_m = re.search(r'^author:\s*["\']?(.*?)["\']?\s*$', fm, re.MULTILINE)
        if a_m:
            author = a_m.group(1).strip()
        c_m = re.search(r'^cover:\s*["\']?(.*?)["\']?\s*$', fm, re.MULTILINE)
        if c_m:
            cover = c_m.group(1).strip()

    if not title:
        for line in content.splitlines():
            trimmed = line.strip()
            if trimmed.startswith('#'):
                extracted = re.sub(r'^#+\s*', '', trimmed).strip()
                if extracted:
                    title = extracted
                    break

    return title, author, cover, content


def ensure_cover_in_frontmatter(readme_path: str, cover_val: str = "assets/cover.png", dry_run: bool = False):
    """确保 README.md 的 frontmatter 中包含 cover 字段"""
    with open(readme_path, 'r', encoding='utf-8') as f:
        content = f.read()

    fm_match = re.match(r'^---\r?\n(.*?)\r?\n---\r?\n?(.*)$', content, re.DOTALL)
    if not fm_match:
        return

    fm_text = fm_match.group(1)
    body_text = fm_match.group(2) or ''

    if re.search(r'(^|\n)cover:\s*', fm_text):
        return

    cover_line = f'cover: "{cover_val}"'
    if re.search(r'(^|\n)layout:', fm_text):
        new_fm = re.sub(r'(^|\n)(layout:)', rf'\1{cover_line}\n\2', fm_text, count=1)
    elif re.search(r'(^|\n)title:', fm_text):
        new_fm = re.sub(r'(^|\n)(title:[^\n]*)', rf'\1\2\n{cover_line}', fm_text, count=1)
    else:
        new_fm = fm_text.rstrip() + '\n' + cover_line

    new_content = f"---\n{new_fm.strip()}\n---\n{body_text}"
    if not dry_run:
        with open(readme_path, 'w', encoding='utf-8') as f:
            f.write(new_content)
        print(f"   📝 已在 {os.path.basename(readme_path)} 的 frontmatter 中补充 cover: \"{cover_val}\"")


def process_column_cover(
    column_dir: str,
    repo_root: str,
    custom_title: str = None,
    custom_author: str = None,
    style: str = "theme",
    dry_run: bool = False,
    verbose: bool = False,
):
    """为指定专栏生成封面图片"""
    column_dir = os.path.abspath(column_dir)
    if not os.path.isdir(column_dir):
        print(f"❌ 目录不存在: {column_dir}", file=sys.stderr)
        return False

    readme_name = None
    for name in os.listdir(column_dir):
        if name.lower() == 'readme.md':
            readme_name = name
            break

    if not readme_name:
        print(f"⚠️ 目录下未找到 README.md，跳过: {column_dir}")
        return False

    readme_path = os.path.join(column_dir, readme_name)
    title, fm_author, cover_rel, _ = extract_column_info(readme_path)

    if custom_title:
        title = custom_title

    if not title:
        title = os.path.basename(column_dir)

    # 确定作者署名（仅显示作者名）
    _, default_author = read_site_config(repo_root)
    author_val = custom_author or fm_author or default_author or "金石碼农"
    footer_text = author_val

    # 确定输出封面路径
    if cover_rel:
        if not os.path.isabs(cover_rel) and not cover_rel.startswith('http'):
            cover_output_path = os.path.join(column_dir, cover_rel)
        else:
            cover_output_path = os.path.join(column_dir, "assets", "cover.png")
    else:
        cover_output_path = os.path.join(column_dir, "assets", "cover.png")
        ensure_cover_in_frontmatter(readme_path, "assets/cover.png", dry_run=dry_run)

    col_id = os.path.basename(column_dir)

    print(f"🎨 正在为专栏 [{col_id}] 生成封面:")
    print(f"   - 标题: {title}")
    print(f"   - 署名: {footer_text}")
    print(f"   - 目标路径: {cover_output_path}")

    if dry_run:
        print("   [Dry-run] 演练模式，未生成图片。")
        return True

    try:
        draw_cover_image(
            title=title,
            author_text=footer_text,
            column_id=col_id,
            output_path=cover_output_path,
            style=style,
        )
        print(f"   ✅ 封面生成成功")
        return True
    except Exception as e:
        print(f"❌ 生成封面失败: {e}", file=sys.stderr)
        return False


def main():
    parser = argparse.ArgumentParser(
        description="自动根据专栏 README.md 标题与作者署名为专栏生成封面图片 (cover.png)。"
    )
    parser.add_argument(
        "paths",
        nargs="*",
        help="专栏目录路径（例如 1 或 source/columns/1 或 source/columns）。若不指定，则自动扫描全部专栏。"
    )
    parser.add_argument(
        "-a", "--all",
        action="store_true",
        help="扫描并为 source/columns 下的所有专栏生成封面"
    )
    parser.add_argument(
        "-t", "--title",
        help="自定义封面标题（仅当指定单个专栏时有效）"
    )
    parser.add_argument(
        "-u", "--author",
        help="自定义作者与署名（例如 '金石碼农'）"
    )
    parser.add_argument(
        "-s", "--style",
        choices=["theme", "red"],
        default="theme",
        help="封面视觉风格: theme (站点浅色调，默认), red (故宫红渐变)"
    )
    parser.add_argument(
        "-n", "--dry-run",
        action="store_true",
        help="演练模式，仅打印将要生成的标题、作者与路径，不实际生成文件"
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

    seen = set()
    unique_columns = []
    for cd in column_dirs:
        if cd not in seen:
            seen.add(cd)
            unique_columns.append(cd)

    if not unique_columns:
        print("未找到任何专栏目录可供处理。")
        sys.exit(0)

    print(f"🚀 开始为专栏生成封面 (共 {len(unique_columns)} 个专栏)...")
    success_count = 0
    for cd in unique_columns:
        custom_title = args.title if len(unique_columns) == 1 else None
        ok = process_column_cover(
            column_dir=cd,
            repo_root=repo_root,
            custom_title=custom_title,
            custom_author=args.author,
            style=args.style,
            dry_run=args.dry_run,
            verbose=args.verbose,
        )
        if ok:
            success_count += 1

    print(f"\n✨ 处理完成！共处理 {len(unique_columns)} 个专栏，成功生成 {success_count} 个封面。")


if __name__ == "__main__":
    main()
