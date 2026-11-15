# -*- coding: utf-8 -*-
"""只读勘察：draw.rs/charpanel.rs 延后批（帧路径纪律）>=40B 中文串池上界。"""
import io
import re
import sys

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
LIT = re.compile('"((?:[^"\\\\]|\\\\.)*)"')

total_b = 0
total_n = 0
for f in ('src/draw.rs', 'src/charpanel.rs'):
    text = open(f, encoding='utf-8').read()
    n = 0
    b = 0
    for m in LIT.finditer(text):
        raw = m.group(1)
        if not any(ord(c) > 127 for c in raw):
            continue
        eb = len(raw.encode('utf-8'))
        if eb < 40:
            continue
        n += 1
        b += eb
    print('%s: >=40B 中文串 %d 站 %dB raw（含 const 表内——保守上界）' % (f, n, b))
    total_b += b
    total_n += n
print('TOTAL (rough upper bound):', total_n, 'sites', total_b, 'B raw')
