# -*- coding: utf-8 -*-
# 施工辅助：真字体 hmtx 度量实算（非运行游戏）——章签安全隙 + EVENT_BRIEF 最长简
# advance 口径：advance_units / unitsPerEm × font_size（macroquad measure_text 同源近似，
# 无 kerning——macroquad 默认不启用 kerning）
from fontTools.ttLib import TTFont
import re

f = TTFont('assets/font.ttf')
upem = f['head'].unitsPerEm
cmap = f.getBestCmap()
hmtx = f['hmtx']

def width_px(text, size):
    w = 0.0
    for ch in text:
        g = cmap.get(ord(ch))
        if g is None:
            return None
        w += hmtx[g][0] / upem * size
    return w

# ---- 章签安全隙：终局叶（guide.rs:91）@18px 自 x=444，章签 num 锚 992 ----
leaf = "已登仙界：讨牒 6/6 方见终战卡，落款四结局（书志页灰行指路）"
w = width_px(leaf, 18)
full = sum(1 for c in leaf if ord(c) > 0x2000)
half = len(leaf) - full
print(f'终局叶 {len(leaf)} 字符（全角 {full} + 半角 {half}）@18px 宽 = {w:.1f}px')
print(f'右端 = 444 + {w:.1f} = {444+w:.1f}px；章签 num 锚 992 → 安全隙 = {992-444-w:.1f}px')
# 供对照：其余叶的最长者（guide.rs 全部 HINTS text @18px）
src = open('src/guide.rs', encoding='utf-8').read()
texts = re.findall(r'text: "([^"]+)",', src)
best = []
for t in texts:
    ww = width_px(t, 18)
    if ww is not None:
        best.append((ww, t))
best.sort(reverse=True)
for ww, t in best[:3]:
    print(f'  叶宽 {ww:6.1f}px  {t}')

# ---- EVENT_BRIEF 实测最长简（A① 注释定稿依据）----
d = open('src/data.rs', encoding='utf-8').read()
m = re.search(r'pub const EVENT_BRIEF: \[&str; (\d+)\] = \[(.*?)\n\];', d, re.S)
n = int(m.group(1))
briefs = re.findall(r'"([^"]+)"', m.group(2))
briefs = [b for b in briefs if not b.startswith('v1.')]  # 剔注释混入（正则只吃串，注释在 // 内不会命中——防御）
assert len(briefs) == n, f'brief 数 {len(briefs)} != 声明 {n}'
lens = [(len(b), b) for b in briefs]
lens.sort(reverse=True)
print(f'EVENT_BRIEF {n} 条；最长 {lens[0][0]} 字：{lens[0][1]}')
top = sorted(set(l for l, _ in lens), reverse=True)[:5]
print('长度分布 top5:', top)
over18 = [(l, b) for l, b in lens if l > 18]
print('超过 18 字的简：', over18 if over18 else '无')
