#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
微信公众号封面生成脚本

功能：
- 从markdown文件中自动提取标题
- 生成天蓝色背景、白色标题的微信公众号封面
- 自动换行和居中排版

依赖：
  pip install Pillow

使用方法：
  基础用法（默认处理 2026/14/publish.md）：
    python scripts/generate_cover.py
  
  指定文件：
    python scripts/generate_cover.py /path/to/file.md

输出：
  生成的图片保存在markdown文件同目录下，名为 cover.png
  尺寸：900x500 像素（标准微信公众号封面比例）
"""

import os
import sys
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
import textwrap
import re
from pathlib import Path

# 配置
COVER_WIDTH = 900
COVER_HEIGHT = 500
GRADIENT_TOP = (193, 34, 42)     # 故宫红 (深)
GRADIENT_BOTTOM = (220, 60, 50)  # 故宫红 (浅)
TEXT_COLOR = (255, 255, 255)     # 白色
BOLD_FLAG = True
PADDING = 50
LINE_SPACING = 1.5

def get_latest_publish_md(base_dir_path):
    """
    在指定的 base_dir_path 下查找所有纯数字的子目录，
    找出数字最大的那个目录，并返回其中的 publish.md 的路径。
    """
    base_path = Path(base_dir_path)
    if not base_path.exists() or not base_path.is_dir():
        print(f"警告：基础目录不存在或不是目录: {base_dir_path}")
        return None

    # 获取所有纯数字的子目录名称，并转换为整数列表
    numeric_dirs = []
    for item in base_path.iterdir():
        if item.is_dir() and item.name.isdigit():
            numeric_dirs.append(int(item.name))

    if not numeric_dirs:
        print(f"警告：在 {base_dir_path} 下没有找到纯数字的子目录。")
        return None

    # 找到最大的数字
    max_dir_num = max(numeric_dirs)
    
    # 构建目标文件的完整路径
    target_file = base_path / str(max_dir_num) / "publish.md"
    
    return str(target_file)

def get_title_from_markdown(md_path):
    """从markdown文件中提取标题"""
    try:
        with open(md_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # 查找第一个# 开头的标题
        lines = content.split('\n')
        for line in lines:
            if line.strip().startswith('#'):
                # 移除#和空白
                title = re.sub(r'^#+\s+', '', line.strip())
                return title
        
        return None
    except Exception as e:
        print(f"读取文件出错: {e}")
        return None

def wrap_text(text, max_width, font, draw):
    """根据字体宽度自动换行"""
    words = text
    lines = []
    current_line = ""
    
    for char in words:
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

def generate_cover(title, output_path):
    """生成封面图片"""

    # 创建渐变背景：故宫红从上到下由深到浅
    img = Image.new('RGB', (COVER_WIDTH, COVER_HEIGHT))
    for y in range(COVER_HEIGHT):
        ratio = y / COVER_HEIGHT
        r = int(GRADIENT_TOP[0] + (GRADIENT_BOTTOM[0] - GRADIENT_TOP[0]) * ratio)
        g = int(GRADIENT_TOP[1] + (GRADIENT_BOTTOM[1] - GRADIENT_TOP[1]) * ratio)
        b = int(GRADIENT_TOP[2] + (GRADIENT_BOTTOM[2] - GRADIENT_TOP[2]) * ratio)
        for x in range(COVER_WIDTH):
            img.putpixel((x, y), (r, g, b))
    draw = ImageDraw.Draw(img)
    
    # 尝试加载系统字体（macOS），优先粗体
    # 从大到小试，选文字能塞进封面（≤3行）的最大字号
    font_sizes = [96, 88, 80, 72, 64, 56, 48, 40]
    font = None
    chosen_size = None

    # 字体优先级：粗体 > 常规
    if BOLD_FLAG:
        font_paths = [
            "/System/Library/Fonts/STHeiti Medium.ttc",
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        ]
    else:
        font_paths = [
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/STHeiti Medium.ttc",
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        ]

    available_width = COVER_WIDTH - 2 * PADDING
    max_height = COVER_HEIGHT - 2 * PADDING

    for size in font_sizes:
        for font_path in font_paths:
            if not os.path.exists(font_path):
                continue
            try:
                test_font = ImageFont.truetype(font_path, size)
            except:
                continue
            lines = wrap_text(title, available_width, test_font, draw)
            if len(lines) == 1:
                # 单行：算实际行高
                test_bbox = draw.textbbox((0, 0), lines[0], font=test_font)
                test_h = test_bbox[3] - test_bbox[1]
                if test_h <= max_height:
                    font = test_font
                    chosen_size = size
                    break
            elif len(lines) <= 3:
                # 2-3行：行高 = 实际字体高度 * 行距系数
                test_bbox = draw.textbbox((0, 0), "测", font=test_font)
                line_h = (test_bbox[3] - test_bbox[1]) * LINE_SPACING
                total_h = len(lines) * line_h
                if total_h <= max_height:
                    font = test_font
                    chosen_size = size
                    break
        if font is not None:
            break
    
    # 如果没有找到系统字体，使用默认字体
    if font is None:
        font = ImageFont.load_default()
        print("警告：未找到中文字体，使用默认字体可能显示不正常")
    
    # 计算可用的文本宽度
    available_width = COVER_WIDTH - 2 * PADDING
    
    # 自动换行
    lines = wrap_text(title, available_width, font, draw)

    # 按字体实际高度计算行高
    sample_bbox = draw.textbbox((0, 0), "测", font=font)
    font_height = sample_bbox[3] - sample_bbox[1]
    line_height = int(font_height * LINE_SPACING)
    total_height = len(lines) * line_height
    
    # 垂直居中开始位置
    start_y = (COVER_HEIGHT - total_height) // 2
    
    # 逐行绘制文本
    for i, line in enumerate(lines):
        y = start_y + i * line_height
        
        # 水平居中
        bbox = draw.textbbox((0, 0), line, font=font)
        text_width = bbox[2] - bbox[0]
        x = (COVER_WIDTH - text_width) // 2
        
        draw.text((x, y), line, fill=TEXT_COLOR, font=font)
    
    # 保存图片
    img.save(output_path)
    print(f"✅ 封面已生成: {output_path}")
    if chosen_size:
        print(f"   字号: {chosen_size}px, 行数: {len(lines)}, 行高: {line_height}px")
    return output_path

def main():
    # 检查参数
    # 如果传入的参数至少有 2 个，并且第一个参数不是以 .md 结尾，
    # 那么判定为：sys.argv[1] 是图片存储路径，sys.argv[2] 是标题
    if len(sys.argv) > 2 and not sys.argv[1].endswith('.md'):
        output_path = sys.argv[1]
        title = sys.argv[2]
        
        # 确保输出目录的包含目录存在
        output_dir = os.path.dirname(output_path)
        if output_dir and not os.path.exists(output_dir):
            os.makedirs(output_dir, exist_ok=True)
            
        print(f"📝 自定义模式: 标题 = {title}")
        print(f"📁 目标存储路径 = {output_path}")
        
        generate_cover(title, output_path)
        return

    # 否则，按照原有从 markdown 提取标题模式工作
    if len(sys.argv) > 1:
        md_file = sys.argv[1]
    else:
        # 默认查找 2026/14/publish.md
        base_dir = "/Users/jsmn/work/hotmp/2026"
        md_file = get_latest_publish_md(base_dir)
    
    if not os.path.exists(md_file):
        print(f"❌ 文件不存在: {md_file}")
        sys.exit(1)
    
    # 提取标题
    title = get_title_from_markdown(md_file)
    if not title:
        print("❌ 未找到markdown标题")
        sys.exit(1)
    
    print(f"📝 提取的标题: {title}")
    
    # 生成输出路径
    if len(sys.argv) > 2:
        output_path = sys.argv[2]
        # 确保输出目录的包含目录存在
        output_dir = os.path.dirname(output_path)
        if output_dir and not os.path.exists(output_dir):
            os.makedirs(output_dir, exist_ok=True)
    else:
        md_dir = os.path.dirname(md_file)
        output_path = os.path.join(md_dir, "cover.png")
    
    # 生成封面
    generate_cover(title, output_path)

if __name__ == "__main__":
    main()
