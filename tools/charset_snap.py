# -*- coding: utf-8 -*-
# 施工辅助：按 tools/make_font.py 同款口径快照 src/*.rs 字面量字符集（newChars 差集用）
import glob, re, sys

PAT_STR = re.compile(r'"((?:[^"\\]|\\.)*)"')
PAT_CHR = re.compile(r"'([^'\\\n]|\\.)'")

chars = set()
for f in glob.glob('src/*.rs'):
    src = open(f, encoding='utf-8').read()
    for m in PAT_STR.finditer(src):
        chars.update(m.group(1))
    for m in PAT_CHR.finditer(src):
        chars.update(m.group(1))

out = sys.argv[1] if len(sys.argv) > 1 else '.mumu/chars_before.json'
open(out, 'w', encoding='utf-8').write(''.join(sorted(chars)))
print('unique literal chars:', len(chars), '->', out)
