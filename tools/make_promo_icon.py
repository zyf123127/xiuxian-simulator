# -*- coding: utf-8 -*-
"""TapTap 宣传图与图标生成（管线对齐 GetFish tools/make_promo.py：游戏画面 + 内嵌 OFL 字体标题）。

- promo_1920x1080.png：山河图志地图纯景带（去掉底部 UI）最近邻放大铺满 16:9，
  压暗 + 上下渐变遮罩，仅叠游戏名《凡人修仙·轮回模拟器》（画面不放宣传语/合规文案）。
- icon_512.png：主菜单打坐修士立绘（含法阵光晕）最近邻光栅化放大到 512x512。

用法：python tools/make_promo_icon.py （先用 GF_CLEAN=1 GF_DEBUG=1 冒烟产出 shots/）
"""
import os
from PIL import Image, ImageDraw, ImageFont, ImageEnhance

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FONT = os.path.join(ROOT, "assets", "font.ttf")
SHOT_MAP = os.path.join(ROOT, "shots", "shot_032.png")   # 山河图志（青云宗选中）
SHOT_MENU = os.path.join(ROOT, "shots", "shot_000.png")  # 主菜单（立绘居中）
OUT_PROMO = os.path.join(ROOT, "taptap", "promo_1920x1080.png")
OUT_ICON = os.path.join(ROOT, "taptap", "icon_512.png")

GOLD = (255, 208, 64, 255)
GOLD_DIM = (255, 232, 160, 255)
SHADOW = (8, 6, 24, 240)

# ---------- 宣传图 1920x1080（海报式：游戏名在上部暗区，地图景带在下方） ----------
CANVAS = (1920, 1080)
DARK = (10, 8, 24)
promo = Image.new("RGB", CANVAS, DARK)
map_im = Image.open(SHOT_MAP).convert("RGB")
band = map_im.crop((0, 58, 1280, 536))  # 纯地图景带，去掉顶栏与底部节点面板
band = band.resize((1920, round(band.height * 1.4)), Image.NEAREST)  # 1920x668
band = ImageEnhance.Brightness(band).enhance(0.72)
by = CANVAS[1] - band.height  # 412，贴到画布底部
promo.paste(band, (0, by))
# 景带顶部 150px 渐变融入暗底，标题区干净
fade = Image.new("L", (1, 150), 0)
for y in range(150):
    fade.putpixel((0, y), int(255 * (1 - y / 150)))
fade = fade.resize((1920, 150))
band_top = promo.crop((0, by, 1920, by + 150))
promo.paste(Image.composite(Image.new("RGB", (1920, 150), DARK), band_top, fade), (0, by))

d = ImageDraw.Draw(promo)

def text(cx, y, s, size, fill, stroke=8):
    f = ImageFont.truetype(FONT, size)
    w = d.textlength(s, font=f)
    d.text((cx - w / 2, y), s, font=f, fill=fill, stroke_width=stroke, stroke_fill=SHADOW)

text(960, 36, "凡 人 修 仙", 190, GOLD, 12)
text(960, 296, "·  轮 回 模 拟 器  ·", 82, GOLD_DIM, 6)
promo.save(OUT_PROMO)
print("saved", OUT_PROMO, promo.size)

# ---------- 图标 512x512 ----------
menu = Image.open(SHOT_MENU).convert("RGB")
# 主菜单立绘 + 法阵光晕（圆心约 639,388，半径约 103），取 240x240 方框
crop = menu.crop((519, 268, 759, 508))
icon = crop.resize((512, 512), Image.NEAREST)
icon.save(OUT_ICON)
print("saved", OUT_ICON, icon.size)
