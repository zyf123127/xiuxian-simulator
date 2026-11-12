#!/usr/bin/env python3
# 施工自检：扫描 src/*.rs 字面量所需字符集 vs assets/font.ttf 现有 cmap，列缺字。
# 与 tools/make_font.py 同款正则口径（字符串字面量 + 字符字面量 + 兜底集），
# 只读对账不写字体——并形仍以 make_font.py 单一产出口为准。
import glob, os, re
from fontTools.ttLib import TTFont

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "src")
PAT_STR = re.compile(r'"((?:[^"\\]|\\.)*)"')
PAT_CHR = re.compile(r"'([^'\\\n]|\\.)'")

chars = set()
for c in range(0x20, 0x7F):
    chars.add(chr(c))
extras = "，。：；、！？（）《》「」·…—＋×÷％℃。【】〖〗※☆★◇◆□■▲▼"
extras += "一二三四五六七八九十百千万亿兆京零壹贰叁肆伍陆柒捌玖"
extras += "上中下前后左右天地玄黄宇宙洪荒甲乙丙丁戊己庚辛壬癸子丑寅卯辰巳午未申酉戌亥"
extras += "初终大小快慢停常疾极"
chars.update(extras)

for f in glob.glob(os.path.join(SRC, "*.rs")):
    with open(f, encoding="utf-8") as fp:
        src = fp.read()
    for m in PAT_STR.finditer(src):
        chars.update(m.group(1))
    for m in PAT_CHR.finditer(src):
        chars.update(m.group(1))

font = TTFont(os.path.join(ROOT, "assets", "font.ttf"))
cmap = font.getBestCmap()
missing = sorted(c for c in chars if ord(c) not in cmap)
print("needed:", len(chars), "font cmap:", len(cmap))
print("missing count:", len(missing))
print("missing:", "".join(missing).encode("unicode_escape").decode("ascii"))
