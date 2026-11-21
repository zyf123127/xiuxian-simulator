# -*- coding: utf-8 -*-
"""只读勘察（单1 刀3 前置）：NIZE 候选 const 串表元素计数——不写任何文件。"""
import io
import re
import sys

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
LIT = re.compile(r'"((?:[^"\\]|\\.)*)"')

specs = [
    ('COMP_PAGE_PREWARM', 'src/charpanel.rs'),
    ('COMP_PAGE_PREWARM_16', 'src/charpanel.rs'),
    ('HIST_PREWARM_STRS', 'src/charpanel.rs'),
    ('DUAL_CULT_HINT', 'src/charpanel.rs'),
    ('PREWARM_STRS', 'src/gallery.rs'),
    ('EV_NOTE_PREWARM', 'src/gallery.rs'),
    ('RECALL_CHROME', 'src/gallery.rs'),
    ('GUIDE_PANEL_LINES', 'src/draw.rs'),
    ('GUIDE_PANEL_NOTE', 'src/draw.rs'),
    ('GUIDE_TABS', 'src/draw.rs'),
]
for name, f in specs:
    text = open(f, encoding='utf-8').read()
    m = re.search(r'(?m)^[ \t]*const[ \t]+' + name + r'\b', text)
    if not m:
        print(name, 'NOT FOUND')
        continue
    k = m.end()
    depth = 0
    end = None
    while k < len(text):
        c = text[k]
        if c == '"':
            k += 1
            while True:
                if text[k] == '\\':
                    k += 2
                    continue
                if text[k] == '"':
                    break
                k += 1
            k += 1
            continue
        if c in '([{':
            depth += 1
        elif c in ')]}':
            depth -= 1
        elif c == ';' and depth == 0:
            end = k + 1
            break
        k += 1
    item = text[m.start():end]
    lits = LIT.findall(item)
    nb = sum(len(x.encode('utf-8')) for x in lits)
    print('%s: item %d..%d lits=%d rawB=%d' % (name, m.start(), end, len(lits), nb))
