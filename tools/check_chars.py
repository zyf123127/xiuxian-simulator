#!/usr/bin/env python3
# 验收辅助：检查 assets/font.ttf 是否覆盖指定字符（缺字实证）
import sys
from fontTools.ttLib import TTFont

font_path = sys.argv[1] if len(sys.argv) > 1 else "assets/font.ttf"
# 验收单点名的 18 个缺字
chars = "河源峰贡征步党盯琴做节示稳拢船刺园墟"
f = TTFont(font_path)
cmap = f.getBestCmap()
missing = [c for c in chars if ord(c) not in cmap]
print(f"{font_path}: 覆盖 {len(chars) - len(missing)}/{len(chars)}")
if missing:
    print("missing:", "".join(missing))
else:
    print("missing: none")
