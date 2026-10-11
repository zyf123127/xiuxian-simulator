# -*- coding: utf-8 -*-
"""全量字体覆盖检查：扫描 src/*.rs 全部用字，对照 assets/font.ttf 的 cmap。只读检查。"""
import glob
import re
from fontTools.ttLib import TTFont

chars = set()
for p in glob.glob("src/*.rs"):
    with open(p, encoding="utf-8") as f:
        for line in f:
            line = re.sub(r"//.*", "", line)  # 去注释（注释用字不入游戏）
            for ch in line:
                if ord(ch) > 0x2E80:  # CJK 及以上，忽略 ASCII/标点
                    chars.add(ch)

cmap = TTFont("assets/font.ttf").getBestCmap()
missing = sorted(c for c in chars if ord(c) not in cmap)
print(f"src/*.rs 非注释 CJK 用字 {len(chars)} 种；assets/font.ttf 缺 {len(missing)} 个")
if missing:
    print("missing:", "".join(missing))
