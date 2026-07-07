#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os
import re

def main():
    base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    columns_dir = os.path.join(base_dir, "source", "columns")
    
    if not os.path.exists(columns_dir):
        print("未找到 columns 目录")
        return
        
    for name in os.listdir(columns_dir):
        path = os.path.join(columns_dir, name)
        if os.path.isdir(path):
            readme_path = os.path.join(path, "README.md")
            if os.path.exists(readme_path):
                # 确定专栏序号
                column_num = name
                new_product_id = f"yishulun.com_columns_{column_num}"
                
                with open(readme_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                
                # 替换 product_id: "..." 或 product_id: ...
                original = content
                # 用正则匹配 frontmatter 中的 product_id 字段并修改
                parts = content.split('---', 2)
                if len(parts) >= 3:
                    frontmatter = parts[1]
                    frontmatter = re.sub(
                        r'^product_id:\s*["\']?[^"\'\n\r]+["\']?',
                        f'product_id: "{new_product_id}"',
                        frontmatter,
                        flags=re.MULTILINE
                    )
                    parts[1] = frontmatter
                    content = '---'.join(parts)
                
                if content != original:
                    with open(readme_path, 'w', encoding='utf-8') as f:
                        f.write(content)
                    print(f"✓ 专栏 {column_num} product_id 已更新为: {new_product_id}")

if __name__ == "__main__":
    main()
