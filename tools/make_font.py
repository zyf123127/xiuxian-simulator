#!/usr/bin/env python3
# 扫描 src/*.rs 中实际用到的字符 → 生成 OFL 字体子集 assets/font.ttf
# 注意：文案更新后必须重新运行本脚本，否则缺字
import glob, os, sys
from fontTools import subset

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "src")
OUT = os.path.join(ROOT, "assets", "font.ttf")
SRC_FONT = r"E:\Rust_Game\GetFish_Game\fonts\NotoSansSC-700.ttf"

chars = set()
# ASCII 可打印
for c in range(0x20, 0x7F):
    chars.add(chr(c))
# 常用中文标点与符号兜底
extras = "，。：；、！？（）《》「」·…—＋×÷％℃。【】〖〗※☆★◇◆□■▲▼"
extras += "一二三四五六七八九十百千万亿兆京零壹贰叁肆伍陆柒捌玖"
extras += "上中下前后左右天地玄黄宇宙洪荒甲乙丙丁戊己庚辛壬癸子丑寅卯辰巳午未申酉戌亥"
extras += "初终大小快慢停常疾极"
chars.update(extras)

for f in glob.glob(os.path.join(SRC, "*.rs")):
    with open(f, encoding="utf-8") as fp:
        chars.update(fp.read())

text = "".join(sorted(chars))
print(f"unique chars: {len(text)}")

os.makedirs(os.path.dirname(OUT), exist_ok=True)
args = [
    SRC_FONT,
    f"--text={text}",
    "--layout-features=*",
    "--no-hinting",
    "--desubroutinize",
    f"--output-file={OUT}",
]
subset.main(args)
sz = os.path.getsize(OUT)
print(f"written {OUT} ({sz/1024:.0f} KB)")
