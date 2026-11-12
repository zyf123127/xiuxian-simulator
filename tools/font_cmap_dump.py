#!/usr/bin/env python3
# 施工自检：dump assets/font.ttf 的 cmap 字符集到 stdout（逗号分隔码点+字符）。
# 供并形前后 diff 得 newChars（只读，不写字体）。
import os, sys
from fontTools.ttLib import TTFont

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
font = TTFont(os.path.join(ROOT, "assets", "font.ttf"))
cmap = font.getBestCmap()
chars = sorted(chr(cp) for cp in cmap if cp >= 0x80)
out = ",".join(chars)
sys.stdout.buffer.write(out.encode("utf-8"))
sys.stdout.buffer.write(b"\n")
