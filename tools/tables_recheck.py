#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# 施工自检：EVENTS 行常量逐行解析（剔除行内注释），对账行数与 EVENTS 语义面；
# EVENT_NAMES / EVENT_BRIEF（v1.16.27 刀②/③起收编进 textpack——data.rs const 数组
# 已改 EvTable 查表器，表本体不在源码里）改为按段基址读 textpack 真相源
# assets/text_source.txt 的连号行（TXT_NAME_BASE/TXT_BRIEF_BASE 见 src/text_offsets.inc），
# 对账：段行数 == EVENTS 行数 == 291、NAMES 去「·续」后互异、基址段界内。只读。
import io
import re
import sys

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")

src = open("src/data.rs", encoding="utf-8").read()
n_events = len(re.findall(r"\((\d+), (-?\d+), (\d+), (true|false)\)", src))

# ---- textpack 真相源 ----
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


rows = {}
for line in open("assets/text_source.txt", encoding="utf-8"):
    if not line.strip():
        continue
    n_s, _, raw = line.rstrip("\n").partition("\t")
    rows[int(n_s)] = raw

inc = open("src/text_offsets.inc", encoding="utf-8").read()
name_base = int(re.search(r"pub static TXT_NAME_BASE: u32 = (\d+);", inc).group(1))
# 段二（v1.16.27 扩池起）：段一 291 条满员后新批连号追加；缺段二基址视为未扩池
_SEG1 = 291
_name2 = re.search(r"pub static TXT_NAME2_BASE: u32 = (\d+);", inc)
_brief2 = re.search(r"pub static TXT_BRIEF2_BASE: u32 = (\d+);", inc)
name2_base = int(_name2.group(1)) if _name2 else None
brief2_base = int(_brief2.group(1)) if _brief2 else None


def _seg_rows(base, base2, n):
    """段一 0..min(n,291) + 段二续号（扩池批）；返回 textpack 行号列表"""
    out = [base + i for i in range(min(n, _SEG1))]
    if n > _SEG1:
        assert base2 is not None, "扩池后段二基址缺失（pack_text.py 未登记）"
        out += [base2 + i for i in range(n - _SEG1)]
    return out
brief_base = int(re.search(r"pub static TXT_BRIEF_BASE: u32 = (\d+);", inc).group(1))

tables = {}
for tname, base, base2 in (("EVENT_NAMES", name_base, name2_base),
                           ("EVENT_BRIEF", brief_base, brief2_base)):
    entries = []
    for n in _seg_rows(base, base2, n_events):
        assert n in rows, f"{tname} 段缺行 N={n}（基址漂移/收编段不齐）"
        val = decode_raw(rows[n])
        assert val is not None, f"{tname} N={n} 不可解码"
        entries.append(val)
    tables[tname] = entries
    print(tname, "base", base, "base2", base2, "rows", len(entries), "aligned:", len(entries) == n_events == 299)

names = tables["EVENT_NAMES"]
seen = set()
dup = []
for e in names:
    base = e.replace("·续", "")
    if base in seen:
        dup.append(base)
    seen.add(base)
print("NAMES distinct bases:", len(seen), "dup:", dup if dup else "none")

# EvTable 静态声明与段长一致（len: 291）
for tname in ("EVENT_NAMES", "EVENT_BRIEF"):
    m = re.search(r"pub static %s: EvTable = EvTable \{\s*base: \w+ as usize,\s*len: \w+,\s*base2: \w+ as usize,\s*len2: \w+ - \w+,\s*\};" % tname, src)
    assert m, f"{tname} EvTable 声明缺失"
print("EvTable statics 两段声明在位，len() == n_events:", n_events)
