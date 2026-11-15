# -*- coding: utf-8 -*-
"""验证 tables2 第三批真相源新增行（N1971..2337）的全部字符都在 font cmap 内。"""
import io
import sys
from fontTools.ttLib import TTFont

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
ESC = {'n': '\n', 't': '\t', 'r': '\r', '"': '"', '\\': '\\', "'": "'", '0': '\0'}


def dec(raw):
    if '\\' not in raw:
        return raw
    out = []
    k = 0
    while k < len(raw):
        c = raw[k]
        if c != '\\':
            out.append(c)
            k += 1
            continue
        k += 1
        d = raw[k]
        if d in ESC:
            out.append(ESC[d])
            k += 1
        else:
            raise ValueError(raw[k:k + 4])
    return ''.join(out)


chars = set()
for line in open('assets/text_source.txt', encoding='utf-8'):
    n, _, raw = line.rstrip('\n').partition('\t')
    if 1971 <= int(n) <= 2337:
        chars.update(dec(raw))
chars.discard('\n')
font = TTFont('assets/font.ttf')
cmap = font.getBestCmap()
miss = sorted(c for c in chars if ord(c) not in cmap)
print('new-row distinct chars:', len(chars))
print('missing from cmap:', miss if miss else 'NONE — 全部在字体 2321 cmap 内')
