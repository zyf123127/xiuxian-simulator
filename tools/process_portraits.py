# 立绘预处理：抠背景(边连通域) → 裁人物包围盒 → 等比缩放到统一高度 → PNG 优化
# 用法：python tools/process_portraits.py [目标高度，默认 1024]
# 原图在 git 历史中（v1.8.0 提交），本脚本就地覆盖 assets/portraits/*.png
import glob
import sys

import numpy as np
from PIL import Image, ImageFilter

TARGET_H = int(sys.argv[1]) if len(sys.argv) > 1 else 1024
T_DIST = 34  # 与背景色的 RGB 距离和阈值：低于视为背景


def process(path: str) -> tuple[str, int, int]:
    im = Image.open(path).convert("RGBA")
    w, h = im.size
    a = np.asarray(im).astype(np.int16)
    rgb = a[:, :, :3]
    # 背景色：四角 12px 均值（渐变背景取局部角，抗暗角）
    corners = np.concatenate(
        [
            rgb[:12, :12].reshape(-1, 3),
            rgb[:12, -12:].reshape(-1, 3),
            rgb[-12:, :12].reshape(-1, 3),
            rgb[-12:, -12:].reshape(-1, 3),
        ]
    )
    bg = corners.mean(axis=0)
    dist = np.abs(rgb - bg).sum(axis=2)
    fg = dist > T_DIST
    # 边连通域：只把与画布边缘相连的"背景色区域"置透明（保留人物内部的近背景色）
    from scipy import ndimage

    lab, n = ndimage.label(~fg)
    border_labels = set(lab[0, :]) | set(lab[-1, :]) | set(lab[:, 0]) | set(lab[:, -1])
    border_labels.discard(0)
    bgmask = np.isin(lab, list(border_labels))
    alpha = np.where(bgmask, 0, 255).astype(np.uint8)
    # 边缘 1px 收边 + 轻羽化抑制白边
    am = Image.fromarray(alpha, "L").filter(ImageFilter.MinFilter(3)).filter(ImageFilter.GaussianBlur(0.8))
    a[:, :, 3] = np.asarray(am)
    im2 = Image.fromarray(a.astype(np.uint8), "RGBA")
    # 人物包围盒（alpha>12），四周留 1.5% padding
    ys, xs = np.where(np.asarray(im2)[:, :, 3] > 12)
    if len(xs) == 0:
        return (path, 0, 0)
    pad_x = int((xs.max() - xs.min()) * 0.015) + 4
    pad_y = int((ys.max() - ys.min()) * 0.015) + 4
    box = (
        max(0, xs.min() - pad_x),
        max(0, ys.min() - pad_y),
        min(w, xs.max() + pad_x),
        min(h, ys.max() + pad_y),
    )
    im3 = im2.crop(box)
    # 等比缩放到目标高度
    scale = TARGET_H / im3.height
    im3 = im3.resize((max(1, round(im3.width * scale)), TARGET_H), Image.LANCZOS)
    im3.save(path, "PNG", optimize=True)
    return (path, im.size, im3.size)


total_old = total_new = 0
for p in sorted(glob.glob("assets/portraits/*.png")):
    old = __import__("os").path.getsize(p)
    path, before, after = process(p)
    new = __import__("os").path.getsize(p)
    total_old += old
    total_new += new
    print(f"{path}: {before[0]}x{before[1]} -> {after[0]}x{after[1]}  {old//1024}KB -> {new//1024}KB")
print(f"TOTAL: {total_old/1048576:.1f}MB -> {total_new/1048576:.1f}MB")
