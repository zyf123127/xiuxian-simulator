# 立绘预处理：抠背景(边连通域) → 裁人物包围盒 → 等比缩放 → PNG 优化
# 用法：python tools/process_portraits.py [桌面集高度，默认 1024]
# 就地覆盖 assets/portraits/*.png（原始大图在 git 历史 v1.8.0 提交可找回）；
# 并同步生成 assets/portraits_wasm/（640px 高 + 256 色量化，供 wasm 内嵌，体积约 1/3）
import glob
import io as _io
import os
import struct as _struct
import sys
import zlib

import numpy as np
from PIL import Image, ImageFilter

TARGET_H = int(sys.argv[1]) if len(sys.argv) > 1 else 1024
WASM_DIR = "assets/portraits_wasm"
WASM_H = 640
T_DIST = 34  # 与背景色的 RGB 距离和阈值：低于视为背景


# ---------------- v1.16.47 加载优化（施工单三/勘察一提名一）：PNG IDAT zopfli 无损再压 ----------------
# PIL optimize=True 仅 zlib level 9+滤器选择；PNG 的 IDAT 本就是 zlib(RFC1950) 容器，
# 无需剥离头尾——整流 zopfli 重压后按原 chunk 顺序重组（IDAT 单块重发+CRC 重算）。
# zopfli 缺装 graceful skip 照 make_font/_zopfli/wasm-opt 先例（管线不断，产物=原
# PIL 字节）；zopfli 输出确定（无时间戳入流），重跑幂等。装机后重跑本脚本生成的
# 桌面集/部署集（portraits_wasm）即自动再压，防「重跑覆盖回 PIL 压缩」。
try:
    import zopfli.zlib as _zopfli_zlib

    def _png_idat_zopfli(data: bytes) -> bytes:
        assert data[:8] == b"\x89PNG\r\n\x1a\n", "PNG 签名缺失"
        idat = bytearray()
        pos = 8
        while pos < len(data):
            (ln,) = _struct.unpack(">I", data[pos : pos + 4])
            typ = data[pos + 4 : pos + 8]
            body = data[pos + 8 : pos + 8 + ln]
            pos += 12 + ln
            if typ == b"IDAT":
                idat += body
            if typ == b"IEND":
                break
        assert idat, "PNG 无 IDAT 数据块"
        raw = zlib.decompress(bytes(idat))  # 回环防线一：IDAT 整流可解
        new_stream = _zopfli_zlib.compress(raw)
        assert zlib.decompress(new_stream) == raw, "PNG IDAT zopfli 回环失败"  # 防线二
        # 单次遍历重组（块序原样）：IDAT 新流只在首个 IDAT 位输出一次（原 PNG 多
        # IDAT 块时其余位跳过——IDAT 必须连续，拼接流=新流恰一份）
        out = bytearray(data[:8])
        idat_written = False
        pos = 8
        while pos < len(data):
            (ln,) = _struct.unpack(">I", data[pos : pos + 4])
            typ = data[pos + 4 : pos + 8]
            body = data[pos + 8 : pos + 8 + ln]
            pos += 12 + ln
            if typ == b"IDAT":
                if idat_written:
                    continue
                body = new_stream
                idat_written = True
            out += _struct.pack(">I", len(body)) + typ + body
            out += _struct.pack(">I", zlib.crc32(typ + body) & 0xFFFFFFFF)
            if typ == b"IEND":
                break
        return bytes(out)

    print("zopfli(png): enabled")
except ImportError:

    def _png_idat_zopfli(data: bytes) -> bytes:
        return data  # graceful skip：缺装回落原 PIL 字节（wasm-opt graceful-skip 先例）


def _save_png(im, path: str) -> tuple[int, int]:
    """PNG 保存统一口：PIL optimize → IDAT zopfli 再压 → decode-equality 断言。
    decode-equality：新旧字节 PIL 解码 RGBA 逐像素相等（IDAT 重压只许改字节不许改
    画面——桌面/网页解码链同 png crate→miniz_oxide，portraits_png_decode_chain
    内容级钉照过）；新字节若不小于原字节则落原字节（防倒退）。返回 (原, 新) 字节数。"""
    buf = _io.BytesIO()
    im.save(buf, "PNG", optimize=True)
    old = buf.getvalue()
    new = _png_idat_zopfli(old)
    if new != old:
        a = Image.open(_io.BytesIO(old)).convert("RGBA")
        b = Image.open(_io.BytesIO(new)).convert("RGBA")
        assert a.size == b.size and list(a.getdata()) == list(b.getdata()), (
            f"decode-equality mismatch: {path}"
        )
    with open(path, "wb") as f:
        f.write(new if len(new) < len(old) else old)
    return len(old), len(new)


def process(path: str) -> tuple[str, int, int, int, int]:
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
    # v1.16.47：保存改走 _save_png 统一口（IDAT zopfli 再压+decode-equality 断言）
    _save_png(im3, path)
    # wasm 内嵌集：再降到 640px 高 + 256 色量化（游戏内最大显示高度 ~270 物理像素，
    # 640 仍有 2.4 倍冗余；量化后单张 ~200KB，16 张约 3.5MB）
    os.makedirs(WASM_DIR, exist_ok=True)
    ws = os.path.join(WASM_DIR, os.path.basename(path))
    scale2 = WASM_H / im3.height
    imw = im3.resize((max(1, round(im3.width * scale2)), WASM_H), Image.LANCZOS)
    # v1.16.47：保存改走 _save_png 统一口（IDAT zopfli 再压+decode-equality 断言）
    _save_png(imw.quantize(colors=256, method=Image.FASTOCTREE), ws)
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
