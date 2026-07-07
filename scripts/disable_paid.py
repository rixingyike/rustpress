#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os
import re

def process_file(file_path, is_readme):
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
        
    original = content
    
    # 1. 替换 paid: true 为 paid: false
    # 匹配 frontmatter (以 --- 开头并结束的块) 中的 paid: true
    parts = content.split('---', 2)
    if len(parts) >= 3:
        frontmatter = parts[1]
        
        # 替换 paid: true 为 paid: false
        frontmatter = re.sub(r'^paid:\s*true\b', 'paid: false', frontmatter, flags=re.MULTILINE)
        
        # 如果是 README.md，将 price: <任何数值> 替换为 price: 0.0
        if is_readme:
            frontmatter = re.sub(r'^price:\s*[\d\.]+', 'price: 0.0', frontmatter, flags=re.MULTILINE)
            
        parts[1] = frontmatter
        content = '---'.join(parts)
        
    if content != original:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"✓ 已修改文件: {file_path}")

def main():
    base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    columns_dir = os.path.join(base_dir, "source", "columns")
    
    if not os.path.exists(columns_dir):
        print("未找到 columns 目录")
        return
        
    for root, dirs, files in os.walk(columns_dir):
        for file in files:
            if file.endswith('.md'):
                file_path = os.path.join(root, file)
                is_readme = (file == "README.md")
                process_file(file_path, is_readme)

if __name__ == "__main__":
    main()
