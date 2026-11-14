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
# v1.16.28 清账：EVENT_BRIEF 自 v1.16.27 刀③起收编进 textpack（data.rs 已改
# EvTable 查表器，pub const 数组不存在）——旧按 data.rs 数组解析必崩，改为按
# 段基址读真相源 assets/text_source.txt 三段连号行（与 tools/tables_recheck.py 同口径）
ESC = {"n": "\n", "t": "\t", "r": "\r", '"': '"', "\\": "\\", "'": "'", "0": "\0"}


def decode_raw(raw):
    if "\\" not in raw:
        return raw
    out = []
    k = 0
    while k < len(raw):
        c = raw[k]
        if c != "\\":
            out.append(c)
            k += 1
            continue
        k += 1
        d = raw[k]
        if d == "u":
            close = raw.index("}", k + 2)
            out.append(chr(int(raw[k + 2:close], 16)))
            k = close + 1
        elif d in ESC:
            out.append(ESC[d])
            k += 1
        else:
            return None
    return "".join(out)


d = open('src/data.rs', encoding='utf-8').read()
src_rows = {}
for line in open('assets/text_source.txt', encoding='utf-8'):
    if not line.strip():
        continue
    n_s, _, raw = line.rstrip('\n').partition('\t')
    src_rows[int(n_s)] = raw
inc = open('src/text_offsets.inc', encoding='utf-8').read()
_SEG1, _SEG2 = 291, 299
bases = []
for key in ('TXT_BRIEF_BASE', 'TXT_BRIEF2_BASE', 'TXT_BRIEF3_BASE'):
    m = re.search(r'pub static %s: u32 = (\d+);' % key, inc)
    bases.append(int(m.group(1)) if m else None)
n_events = int(re.search(r'pub const N_EVENTS: usize = (\d+);', d).group(1))
briefs = []
for i in range(n_events):
    if i < _SEG1:
        n = bases[0] + i
    elif i < _SEG2:
        assert bases[1] is not None, '扩池后段二基址缺失'
        n = bases[1] + (i - _SEG1)
    else:
        assert bases[2] is not None, '扩池后段三基址缺失'
        n = bases[2] + (i - _SEG2)
    assert n in src_rows, f'EVENT_BRIEF 段缺行 N={n}'
    briefs.append(decode_raw(src_rows[n]))
lens = [(len(b), b) for b in briefs]
lens.sort(reverse=True)
print(f'EVENT_BRIEF {n_events} 条；最长 {lens[0][0]} 字：{lens[0][1]}')
top = sorted(set(l for l, _ in lens), reverse=True)[:5]
print('长度分布 top5:', top)
over18 = [(l, b) for l, b in lens if l > 18]
print('超过 18 字的简：', over18 if over18 else '无')
