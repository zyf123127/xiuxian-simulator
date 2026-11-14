#!/usr/bin/env python3
# 施工自检：扫描 src/*.rs 字面量 + textpack 真相源所需字符集 vs assets/font.ttf
# 现有 cmap，列缺字。与 tools/make_font.py 同款口径（字符串字面量 + 字符字面量
# + 兜底集；v1.16.25 追加 assets/text_source.txt 解码值——收编文案的字运行期
# 仍要渲染，纯字面量口径会在重建时漏掉它们），只读对账不写字体——并形仍以
# make_font.py 单一产出口为准。
import glob, os, re
from fontTools.ttLib import TTFont

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "src")
PAT_STR = re.compile(r'"((?:[^"\\]|\\.)*)"')
PAT_CHR = re.compile(r"'([^'\\\n]|\\.)'")

chars = set()
for c in range(0x20, 0x7F):
    chars.add(chr(c))
extras = "，。：；、！？（）《》「」·…—＋×÷％℃。【】〖〗※☆★◇◆□■▲▼"
extras += "一二三四五六七八九十百千万亿兆京零壹贰叁肆伍陆柒捌玖"
extras += "上中下前后左右天地玄黄宇宙洪荒甲乙丙丁戊己庚辛壬癸子丑寅卯辰巳午未申酉戌亥"
extras += "初终大小快慢停常疾极"
chars.update(extras)

for f in glob.glob(os.path.join(SRC, "*.rs")):
    with open(f, encoding="utf-8") as fp:
        src = fp.read()
    for m in PAT_STR.finditer(src):
        chars.update(m.group(1))
    for m in PAT_CHR.finditer(src):
        chars.update(m.group(1))

# v1.16.25 追加：textpack 真相源逐行解码（与 make_font.py 同款 decode 口径）
_ESC = {'n': '\n', 't': '\t', 'r': '\r', '"': '"', '\\': '\\', "'": "'", '0': '\0'}


def _decode_raw(raw):
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
        if k >= len(raw):
            return None
        d = raw[k]
        if d == 'u':
            if raw[k + 1:k + 2] != '{':
                return None
            close = raw.index('}', k + 2)
            try:
                out.append(chr(int(raw[k + 2:close], 16)))
            except ValueError:
                return None
            k = close + 1
        elif d in _ESC:
            out.append(_ESC[d])
            k += 1
        else:
            return None
    return ''.join(out)


_SRC_TXT = os.path.join(ROOT, "assets", "text_source.txt")
if os.path.exists(_SRC_TXT):
    for _line in open(_SRC_TXT, encoding="utf-8").read().splitlines():
        if not _line.strip():
            continue
        _, _, _raw = _line.partition('\t')
        _val = _decode_raw(_raw)
        if _val:
            chars.update(_val)

font = TTFont(os.path.join(ROOT, "assets", "font.ttf"))
cmap = font.getBestCmap()
missing = sorted(c for c in chars if ord(c) not in cmap)
print("needed:", len(chars), "font cmap:", len(cmap))
print("missing count:", len(missing))
print("missing:", "".join(missing).encode("unicode_escape").decode("ascii"))
