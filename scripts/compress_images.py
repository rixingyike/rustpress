import os
import sys
from PIL import Image

def compress_single_image(file_path):
    if not os.path.exists(file_path):
        print(f"错误: 文件 {file_path} 不存在。")
        return False
        
    before_size = os.path.getsize(file_path)
    if before_size == 0:
        return False

    try:
        img = Image.open(file_path)
        ext = os.path.splitext(file_path)[1].lower()
        
        # 根据格式选择不同的优化保存配置
        if ext in ['.jpg', '.jpeg']:
            # JPEG: 启用优化并将质量调为 80%
            img.save(file_path, optimize=True, quality=80)
        elif ext == '.png':
            # PNG: 如果文件大小大于 50KB，我们采用自适应调色板（Quantize）压缩
            if before_size > 50 * 1024:
                try:
                    if img.mode in ('RGBA', 'LA') or (img.mode == 'P' and 'transparency' in img.info):
                        # 保留透明度通道的情况下进行 8 位量化压缩
                        quantized = img.quantize(colors=256, method=Image.Quantize.MAXCOVERAGE)
                    else:
                        # 无透明度通道，直接转为自适应 256 色调色板模式
                        quantized = img.convert("P", palette=Image.Palette.ADAPTIVE, colors=256)
                    quantized.save(file_path, optimize=True)
                except Exception as quant_err:
                    # 降级方案：常规优化保存
                    img.save(file_path, optimize=True)
            else:
                img.save(file_path, optimize=True)
        else:
            # 其它格式默认优化保存
            img.save(file_path, optimize=True)
            
        after_size = os.path.getsize(file_path)
        saved_bytes = before_size - after_size
        saved_pct = (saved_bytes / before_size) * 100
        
        if saved_bytes > 0:
            print(f"成功压缩 {file_path}:")
            print(f"  压缩前: {before_size / 1024:.2f} KB")
            print(f"  压缩后: {after_size / 1024:.2f} KB")
            print(f"  减小了: {saved_bytes / 1024:.2f} KB ({saved_pct:.2f}%)")
        else:
            print(f"已是最佳状态 {file_path} (压缩后体积无减小)")
        return True
    except Exception as e:
        print(f"压缩 {file_path} 失败: {e}", file=sys.stderr)
        return False

def compress_directory(dir_path):
    print(f"开始扫描并压缩目录下的所有图片: {dir_path}")
    image_exts = {'.png', '.jpg', '.jpeg'}
    total_before = 0
    total_after = 0
    
    for root, dirs, files in os.walk(dir_path):
        for file in files:
            ext = os.path.splitext(file)[1].lower()
            if ext in image_exts:
                file_path = os.path.join(root, file)
                before = os.path.getsize(file_path)
                
                compress_single_image(file_path)
                
                after = os.path.getsize(file_path)
                total_before += before
                total_after += after
                
    total_saved = total_before - total_after
    if total_before > 0:
        total_pct = (total_saved / total_before) * 100
        print(f"\n== 整体压缩总结 ==")
        print(f"  总压缩前: {total_before / 1024 / 1024:.2f} MB")
        print(f"  总压缩后: {total_after / 1024 / 1024:.2f} MB")
        print(f"  总共节省: {total_saved / 1024 / 1024:.2f} MB ({total_pct:.2f}%)")

if __name__ == "__main__":
    if len(sys.argv) > 1:
        # 实验/指定单文件模式
        target = sys.argv[1]
        if os.path.isdir(target):
            compress_directory(target)
        else:
            compress_single_image(target)
    else:
        # 默认遍历 source 目录
        compress_directory("source")
