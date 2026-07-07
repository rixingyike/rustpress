#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os
import re
import hashlib
from PIL import Image, ImageDraw, ImageFont

# 明亮纯色背景色板，与站点"极光薄荷"风格一致，搭配深墨前景色
PALETTES = [
    (245, 247, 245),    # 0: 极浅薄荷白（接近 air-bg）
    (232, 240, 236),    # 1: 浅薄荷绿灰
    (240, 244, 248),    # 2: 浅云蓝白
    (248, 245, 240),    # 3: 暖米白
    (240, 248, 244),    # 4: 清水绿
    (245, 240, 250),    # 5: 淡薰衣草
    (248, 242, 238),    # 6: 浅杏色
    (238, 245, 248),    # 7: 浅冰蓝
]

# 深墨前景色（站点 moss-ink）
FG_COLOR = (24, 36, 34)
FG_LIGHT  = (72, 100, 96)       # 次级文字，稍浅
ACCENT    = (0, 195, 145)       # 薄荷绿装饰线（cyber-mint 近似）

def get_palette_by_id(column_id):
    try:
        idx = int(column_id) % len(PALETTES)
    except ValueError:
        h = hashlib.md5(column_id.encode('utf-8')).hexdigest()
        idx = int(h, 16) % len(PALETTES)
    return PALETTES[idx]

def get_system_font(size):
    paths = [
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/Library/Fonts/Songti.ttc",
        "/System/Library/Fonts/Cache/PingFang.ttc",
        "/Library/Fonts/Arial Unicode.ttf",
    ]
    for p in paths:
        if os.path.exists(p):
            try:
                return ImageFont.truetype(p, size)
            except Exception:
                pass
    return ImageFont.load_default()

def draw_cover(title, column_id, output_path):
    width, height = 800, 450
    bg_color = get_palette_by_id(column_id)

    # 1. 纯色背景
    img = Image.new("RGB", (width, height), bg_color)
    draw = ImageDraw.Draw(img)

    # 2. 获取字体
    title_font  = get_system_font(56)
    footer_font = get_system_font(18)

    # 3. 绘制专栏标题（若标题较长，拆成两行）
    max_len = 10
    if len(title) > max_len and len(title) <= 20:
        title_lines = [title[:max_len], title[max_len:]]
    elif len(title) > 20:
        title_lines = [title[:max_len], title[max_len:18] + "..."]
    else:
        title_lines = [title]

    y_offset = 110 if len(title_lines) > 1 else 160
    for idx, line in enumerate(title_lines):
        try:
            line_w = title_font.getbbox(line)[2] - title_font.getbbox(line)[0]
        except AttributeError:
            line_w = title_font.getsize(line)[0]
        draw.text(((width - line_w) // 2, y_offset + idx * 70), line, fill=FG_COLOR, font=title_font)

    # 4. 薄荷绿装饰分割线
    line_y = y_offset + len(title_lines) * 70 + 20
    line_len = 120
    draw.line([((width - line_len) // 2, line_y), ((width + line_len) // 2, line_y)], fill=ACCENT, width=3)

    # 5. 底部签名
    footer_text = "一树仑 · 金石碼农"
    try:
        footer_w = footer_font.getbbox(footer_text)[2] - footer_font.getbbox(footer_text)[0]
    except AttributeError:
        footer_w = footer_font.getsize(footer_text)[0]
    draw.text(((width - footer_w) // 2, height - 70), footer_text, fill=FG_LIGHT, font=footer_font)

    # 6. 保存图片
    img.save(output_path, "PNG")
    print(f"✓ 专栏 {column_id} 封面生成成功: {output_path}")

def update_readme_frontmatter(readme_path, column_id):
    with open(readme_path, 'r', encoding='utf-8') as f:
        content = f.read()
        
    # 提取标题
    title_m = re.search(r'title:\s*"([^"]+)"', content)
    if not title_m:
        title_m = re.search(r'title:\s*([^\n\r]+)', content)
    title = title_m.group(1).strip() if title_m else f"专栏 {column_id}"
    
    # 检查是否有 cover 行
    cover_m = re.search(r'^cover:\s*([^\n\r]+)', content, re.MULTILINE)
    
    # 将 cover 指向我们的 /columns/<id>/cover.png
    target_cover_path = f"/columns/{column_id}/cover.png"
    
    if not cover_m:
        # 在第一个 --- 块中插入 cover 行
        parts = content.split('---', 2)
        if len(parts) >= 3:
            # parts[1] 是 frontmatter 内容
            yaml_lines = parts[1].strip('\n').split('\n')
            yaml_lines.insert(1, f'cover: "{target_cover_path}"')
            parts[1] = '\n' + '\n'.join(yaml_lines) + '\n'
            content = '---'.join(parts)
            
            with open(readme_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"✓ 已自动将 cover 配置追加至 README 前言: {readme_path}")
    else:
        # 如果已经存在了，但路径不对，我们进行更正
        existing_path = cover_m.group(1).strip().strip('"').strip("'")
        if existing_path != target_cover_path:
            content = re.sub(r'^cover:\s*.*$', f'cover: "{target_cover_path}"', content, flags=re.MULTILINE)
            with open(readme_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"✓ 已更正已有的 cover 路径为: {target_cover_path}")
            
    return title

def main():
    base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    columns_dir = os.path.join(base_dir, "source", "columns")
    
    if not os.path.exists(columns_dir):
        print(f"未找到专栏文件夹: {columns_dir}")
        return
        
    for name in os.listdir(columns_dir):
        path = os.path.join(columns_dir, name)
        if os.path.isdir(path):
            readme_path = os.path.join(path, "README.md")
            if os.path.exists(readme_path):
                # 1. 解析并修改 README 中的 Frontmatter，获取专栏标题
                title = update_readme_frontmatter(readme_path, name)
                
                # 2. 在同级目录下生成封面图
                cover_path = os.path.join(path, "cover.png")
                draw_cover(title, name, cover_path)

if __name__ == "__main__":
    main()
