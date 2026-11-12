#!/usr/bin/env python3
# 施工自检：对比 before/after 字体与源字体在指定字符上的字形有无（只读）。
import os, sys
from fontTools.ttLib import TTFont

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
chars = ["\u2103", "\u2286", "\u25c7", "\u53c9", "\u6094", "\u60dc", "\u633d",
         "\u6382", "\u6735", "\u762a", "\u8273", "\u95f9", "\u7b4b"]

def probe(path, label):
    f = TTFont(path)
    cmap = f.getBestCmap()
    out = " ".join(
        "U+%04X:%s" % (ord(c), "Y" if ord(c) in cmap else "N") for c in chars
    )
    print(label, out)

probe(os.path.join(ROOT, "fonts", "NotoSansSC-src.ttf"), "SRC ")
probe(os.path.join(ROOT, "assets", "font.ttf"), "AFTER")
if os.path.exists("/tmp/font_before.ttf"):
    probe("/tmp/font_before.ttf", "BEFR")
