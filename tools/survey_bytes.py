# -*- coding: utf-8 -*-
"""单1 施工批前勘察：逐表统计 utf-8 raw 字节（只读，不改任何文件）。"""
import re
import io
import sys
from pathlib import Path

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
ROOT = Path(__file__).resolve().parent.parent
LIT = re.compile(r'"((?:[^"\\]|\\.)*)"')
BACKSLASH = chr(92)
ESC = {'n': '\n', 't': '\t', 'r': '\r', '"': '"', chr(92): chr(92), "'": "'", '0': '\0'}


def dec(raw):
    if BACKSLASH not in raw:
        return raw
    out = []
    k = 0
    while k < len(raw):
        c = raw[k]
        if c != BACKSLASH:
            out.append(c)
            k += 1
            continue
        k += 1
        d = raw[k]
        if d in ESC:
            out.append(ESC[d])
            k += 1
        else:
            return None
    return ''.join(out)


def region(text, tname):
    m = re.search(r'pub const ' + tname + r'\b.*?=\s*\[', text, re.S)
    if not m:
        return None
    bi = m.end() - 1
    depth = 0
    k = bi
    while k < len(text):
        c = text[k]
        if c == '"':
            k += 1
            while k < len(text):
                if text[k] == BACKSLASH:
                    k += 2
                    continue
                if text[k] == '"':
                    break
                k += 1
            k += 1
            continue
        if c in '[{':
            depth += 1
        elif c in ']}':
            depth -= 1
            if depth == 0:
                return (bi, k + 1)
        k += 1
    return None


def raws(text, tname):
    r = region(text, tname)
    if r is None:
        return None
    return LIT.findall(text[r[0]:r[1]])


src = (ROOT / 'src' / 'data.rs').read_text(encoding='utf-8')
game = (ROOT / 'src' / 'game.rs').read_text(encoding='utf-8')
npc = (ROOT / 'src' / 'npc.rs').read_text(encoding='utf-8')
tables = {'data.rs': src, 'game.rs': game, 'npc.rs': npc}

cands = [
    ('TECHS', 'data.rs'), ('RELICS', 'data.rs'), ('PILLS', 'data.rs'), ('TALENTS', 'data.rs'),
    ('UPGRADES', 'data.rs'), ('ACHS', 'data.rs'), ('ORIGINS', 'data.rs'), ('MAP_THEMES', 'data.rs'),
    ('REGION_MECH', 'data.rs'), ('SECT_GOODS', 'data.rs'),
    ('MINOR_LINES', 'data.rs'), ('COMP_TALES', 'data.rs'), ('NPC_BRIEF', 'data.rs'),
    ('DONGFU_FAC_DESC', 'data.rs'), ('WEATHER_FLAVOR', 'data.rs'),
    ('EXPED_CHOICE_LABELS', 'data.rs'),
    ('AD_CREATIVES', 'data.rs'), ('NPC_NAMES', 'data.rs'), ('NPC_TIER_NAMES', 'data.rs'),
    ('NPC_DESC', 'data.rs'), ('SLOTS', 'data.rs'), ('WEATHERS', 'data.rs'),
    ('TIER_NAMES', 'data.rs'),
    ('PORTRAIT_STAGE_NAMES', 'data.rs'), ('COMPANION_NAMES', 'data.rs'),
    ('NPC_EVENTS', 'npc.rs'), ('INHERITS', 'game.rs'), ('DONE_NOTE', 'game.rs'),
    ('CHAIN_NAMES', 'game.rs'), ('NEME_TITLES', 'game.rs'),
]
total_raw = 0
for t, f in cands:
    ls = raws(tables[f], t)
    if ls is None:
        print('%-22s REGION NOT FOUND' % t)
        continue
    b = sum(len(dec(r).encode('utf-8')) for r in ls)
    total_raw += b
    print('%-22s lits=%3d  raw=%5dB  packed~%4dB  net_data~%4dB'
          % (t, len(ls), b, int(b * 0.472), b - int(b * 0.472)))
print('total raw (all cands):', total_raw)

ch = region(src, 'CHAPTERS')
body = src[ch[0]:ch[1]]
nums = re.findall(r'\bnum\s*:\s*"((?:[^"\\]|\\.)*)"', body)
names = re.findall(r'\bname\s*:\s*"((?:[^"\\]|\\.)*)"', body)
goals = re.findall(r'\btext\s*:\s*"((?:[^"\\]|\\.)*)"', body)
bn = sum(len(dec(r).encode()) for r in nums)
bna = sum(len(dec(r).encode()) for r in names)
bg = sum(len(dec(r).encode()) for r in goals)
print('CHAPTERS num=%d (%dB) name=%d (%dB) goals.text=%d (%dB) residual=%dB'
      % (len(nums), bn, len(names), bna, len(goals), bg, bn + bna + bg))

for t in ('L12', 'L4', 'L1'):
    ls = raws(src, t)
    if ls:
        print('%s: lits=%d raw=%dB' % (t, len(ls), sum(len(dec(r).encode()) for r in ls)))

# ---- 收编前快照（等价性复核用：收编后逐行对账 text_source.txt）----
import json
snap = {}
for t, f in cands:
    ls = raws(tables[f], t)
    if ls is not None:
        snap[t] = [dec(r) for r in ls]
ch_body = src[region(src, 'CHAPTERS')[0]:region(src, 'CHAPTERS')[1]]
snap['CHAPTERS_num'] = [dec(r) for r in re.findall(r'\bnum\s*:\s*"((?:[^"\\]|\\.)*)"', ch_body)]
snap['CHAPTERS_name'] = [dec(r) for r in re.findall(r'\bname\s*:\s*"((?:[^"\\]|\\.)*)"', ch_body)]
snap['CHAPTERS_text'] = [dec(r) for r in re.findall(r'\btext\s*:\s*"((?:[^"\\]|\\.)*)"', ch_body)]
out = ROOT / 'target' / 'tables2_before.json'
out.write_text(json.dumps(snap, ensure_ascii=False, indent=0), encoding='utf-8')
print('snapshot ->', out, '(%d tables)' % len(snap))
