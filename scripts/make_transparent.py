import sys
from collections import deque

from PIL import Image


def make_border_transparent(
    img_path, output_path, threshold=250
):  # 【修改点】调高默认阈值，防止“漏水”
    try:
        img = Image.open(img_path)
        img = img.convert("RGBA")
        width, height = img.size
        pixels = img.load()

        visited = set()
        queue = deque()

        # 判断像素是否属于背景白色
        def is_background_white(x, y):
            r, g, b, a = pixels[x, y]
            # 必须 RGB 全部大于等于 threshold 才算白色
            return r >= threshold and g >= threshold and b >= threshold

        # 1. 将图片四周边缘的白色像素作为 BFS 的起点入队
        for x in range(width):
            if is_background_white(x, 0):
                queue.append((x, 0))
                visited.add((x, 0))
            if is_background_white(x, height - 1):
                queue.append((x, height - 1))
                visited.add((x, height - 1))

        for y in range(1, height - 1):
            if is_background_white(0, y):
                queue.append((0, y))
                visited.add((0, y))
            if is_background_white(width - 1, y):
                queue.append((width - 1, y))
                visited.add((width - 1, y))

        # 2. BFS 泛洪填充（像水一样只在外部蔓延，进不到被包围的内部）
        while queue:
            cx, cy = queue.popleft()
            # 将该连通的外部白色背景像素设为完全透明
            pixels[cx, cy] = (255, 255, 255, 0)

            # 遍历 4 邻域
            for dx, dy in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
                nx, ny = cx + dx, cy + dy
                if 0 <= nx < width and 0 <= ny < height:
                    if (nx, ny) not in visited and is_background_white(nx, ny):
                        visited.add((nx, ny))
                        queue.append((nx, ny))

        img.save(output_path, "PNG")
        print(f"成功通过泛洪算法去除了边缘白色背景并保存至 {output_path}。")
    except Exception as e:
        print(f"处理失败: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    target_img = "source/works/2/cover.png"
    # 【关键调试点】
    # 调高 threshold。如果中间的白色还是被去掉了，说明 250 依然能穿透你的图片边界，请改成 254 甚至 255 再试。
    make_border_transparent(target_img, target_img, threshold=255)
