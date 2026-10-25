#!/usr/bin/env python3
# 临时脚本：从 fontsource 全量切片中找 ●(U+25CF) 渟(U+6E1F) 所在切片，
# 与 GetFish merged.ttf 合并成 Daoism 专用完整源字体 fonts/NotoSansSC-src.ttf
import glob, os
from fontTools.ttLib import TTFont
from fontTools.merge import Merger

PKG = r"E:\Rust_Game\GetFish_Game\fonts\package\files"
OUT_DIR = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "fonts")
os.makedirs(OUT_DIR, exist_ok=True)

need = {0x25CF, 0x6E1F, 0x6D07, 0x8D4D}  # ● 渟 洇 赍（v1.12.0 书道换皮补字）
slices = []
for p in sorted(glob.glob(os.path.join(PKG, "*.woff2"))):
    if not need:
        break
    cm = TTFont(p).getBestCmap()
    hit = {cp for cp in need if cp in cm}
    if hit:
        sid = os.path.basename(p).split("-")[3]
        ttf = os.path.join(OUT_DIR, f"slice-{sid}.ttf")
        if not os.path.exists(ttf):
            f = TTFont(p)
            f.flavor = None
            f.save(ttf)
        slices.append(ttf)
        need -= hit
        print(f"hit slice {sid}: {[chr(c) for c in hit]} -> {os.path.basename(ttf)}")

if need:
    print(f"仍缺: {[chr(c) for c in need]}")
    raise SystemExit(1)

inputs = slices + [r"E:\Rust_Game\GetFish_Game\fonts\merged.ttf"]
print("合并输入:", [os.path.basename(p) for p in inputs])
merged = Merger().merge(inputs)
out = os.path.join(OUT_DIR, "NotoSansSC-src.ttf")
merged.save(out)
print(f"written {out} ({os.path.getsize(out)//1024} KB)")

cm = TTFont(out).getBestCmap()
for ch in "※→①②④●渟洇赍":
    print(f"  {ch}: {'OK' if ord(ch) in cm else 'MISSING'}")
