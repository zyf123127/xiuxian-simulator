# -*- coding: utf-8 -*-
"""只读小工具：按 N 打印 text_source.txt 行的解码值（单1 钉测串提取用）。"""
import io
import sys

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
        if d == 'u':
            close = raw.index('}', k + 2)
            out.append(chr(int(raw[k + 2:close], 16)))
            k = close + 1
        elif d in ESC:
            out.append(ESC[d])
            k += 1
        else:
            return None
    return ''.join(out)


rows = {}
for line in open('assets/text_source.txt', encoding='utf-8'):
    if not line.strip():
        continue
    n_s, _, raw = line.rstrip('\n').partition('\t')
    rows[int(n_s)] = raw
for arg in sys.argv[1:]:
    n = int(arg)
    print(n, repr(dec(rows[n])))
