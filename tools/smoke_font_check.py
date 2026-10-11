#!/usr/bin/env python3
# 冒烟验收辅助：提取 src/*.rs 字符串字面量中的非 ASCII 字符，比对 assets/font.ttf 覆盖
import glob
from fontTools.ttLib import TTFont

cmap = TTFont('assets/font.ttf').getBestCmap()

lit_chars = set()
comment_chars = set()

for p in sorted(glob.glob('src/*.rs')):
    src = open(p, encoding='utf-8').read()
    i, n = 0, len(src)
    in_block_comment = False
    while i < n:
        c = src[i]
        if in_block_comment:
            if src.startswith('*/', i):
                in_block_comment = False
                i += 2
            else:
                if ord(c) > 127:
                    comment_chars.add(c)
                i += 1
            continue
        if src.startswith('//', i):
            j = src.find('\n', i)
            seg = src[i:j if j != -1 else n]
            for ch in seg:
                if ord(ch) > 127:
                    comment_chars.add(ch)
            i = j if j != -1 else n
            continue
        if src.startswith('/*', i):
            in_block_comment = True
            i += 2
            continue
        if c == '"':
            j = i + 1
            buf = []
            while j < n:
                if src[j] == '\\':
                    buf.append(src[j:j + 2])
                    j += 2
                    continue
                if src[j] == '"':
                    break
                buf.append(src[j])
                j += 1
            for ch in ''.join(buf):
                if ord(ch) > 127:
                    lit_chars.add(ch)
            i = j + 1
            continue
        if c == 'r' and i + 1 < n and src[i + 1] in '#r' and src[i:i + 2] != 'ru':
            m = None
            for k in range(i + 1, min(i + 5, n)):
                if src[i + 1:k].count('#') == k - i - 1 and src[k] == '"':
                    m = k
                    break
            if m:
                end = src.find('"' + src[i + 1:m], m + 1)
                j = end if end != -1 else n
                for ch in src[m + 1:j]:
                    if ord(ch) > 127:
                        lit_chars.add(ch)
                i = j + 1 + (m - i - 1)
                continue
        i += 1

missing_lit = sorted(c for c in lit_chars if ord(c) not in cmap)
print('literal distinct non-ascii:', len(lit_chars))
print('MISSING IN LITERALS (%d):' % len(missing_lit), ''.join(missing_lit) if missing_lit else 'none')
