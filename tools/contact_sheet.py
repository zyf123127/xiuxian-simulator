# -*- coding: utf-8 -*-
"""把 shots/ 40 张缩略拼成联络表，便于挑选 TapTap 实机截图。"""
from PIL import Image, ImageDraw
import glob

paths = sorted(glob.glob("shots/shot_*.png"))
assert paths, "no shots"
cols, rows = 5, 9
tw, th = 256, 144
sheet = Image.new("RGB", (cols * tw, rows * (th + 18)), (20, 20, 30))
d = ImageDraw.Draw(sheet)
for i, p in enumerate(paths):
    im = Image.open(p).resize((tw, th))
    x, y = (i % cols) * tw, (i // cols) * (th + 18)
    sheet.paste(im, (x, y + 18))
    d.text((x + 4, y + 2), p.split("/")[-1], fill=(255, 255, 120))
sheet.save("shots/_contact.png")
print("saved shots/_contact.png", len(paths))
