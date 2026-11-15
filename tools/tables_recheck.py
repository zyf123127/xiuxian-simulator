#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# 施工自检：EVENTS 行常量逐行解析（剔除行内注释），对账行数与 EVENTS 语义面；
# EVENT_NAMES / EVENT_BRIEF（v1.16.27 刀②/③起收编进 textpack——data.rs const 数组
# 已改 EvTable 查表器，表本体不在源码里）改为按段基址读 textpack 真相源
# assets/text_source.txt 的连号行（TXT_NAME_BASE/TXT_BRIEF_BASE 见 src/text_offsets.inc；
# v1.16.28 扩池起三段连号——段三基址 TXT_NAME3_BASE/TXT_BRIEF3_BASE=1442/1452；
# v1.16.29 扩池起四段连号——段四基址 TXT_NAME4_BASE/TXT_BRIEF4_BASE=1825/1833，
# 单1/单2 已用 N1492..1824 故段四顺延；
# v1.16.30 扩池起五段连号——段五基址 TXT_NAME5_BASE/TXT_BRIEF5_BASE=1865/1873，
# 单2 改写即减字后下一 N=1865 故段五起于 1865；
# v1.16.31 扩池起六段连号——段六基址 TXT_NAME6_BASE/TXT_BRIEF6_BASE=1905/1915，
# 与 data.rs EV_SEG5=325 同批登记，故段六起于 1905；
# v1.16.31 新功能批起七段连号——段七基址 TXT_NAME7_BASE/TXT_BRIEF7_BASE=
# 1954/1957，与 data.rs EV_SEG6=335 同批登记（段六名/简紧连封死，续批开新段
# 照段四→段五先例），故段七起于 1954），
# 对账：段行数 == EVENTS 行数 == 338、NAMES 去「·续」后互异、基址段界内。只读。
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
# 段二（v1.16.27 扩池起）/段三（v1.16.28 扩池起）/段四（v1.16.29 扩池起）：段一
# 291 条满员后新批连号追加；缺段基址视为未扩池
_SEG1 = 291
_SEG2 = 299
_SEG3 = 309
_SEG4 = 317
_SEG5 = 325
_SEG6 = 335
_name2 = re.search(r"pub static TXT_NAME2_BASE: u32 = (\d+);", inc)
_brief2 = re.search(r"pub static TXT_BRIEF2_BASE: u32 = (\d+);", inc)
_name3 = re.search(r"pub static TXT_NAME3_BASE: u32 = (\d+);", inc)
_brief3 = re.search(r"pub static TXT_BRIEF3_BASE: u32 = (\d+);", inc)
_name4 = re.search(r"pub static TXT_NAME4_BASE: u32 = (\d+);", inc)
_brief4 = re.search(r"pub static TXT_BRIEF4_BASE: u32 = (\d+);", inc)
_name5 = re.search(r"pub static TXT_NAME5_BASE: u32 = (\d+);", inc)
_brief5 = re.search(r"pub static TXT_BRIEF5_BASE: u32 = (\d+);", inc)
_name6 = re.search(r"pub static TXT_NAME6_BASE: u32 = (\d+);", inc)
_brief6 = re.search(r"pub static TXT_BRIEF6_BASE: u32 = (\d+);", inc)
_name7 = re.search(r"pub static TXT_NAME7_BASE: u32 = (\d+);", inc)
_brief7 = re.search(r"pub static TXT_BRIEF7_BASE: u32 = (\d+);", inc)
name2_base = int(_name2.group(1)) if _name2 else None
brief2_base = int(_brief2.group(1)) if _brief2 else None
name3_base = int(_name3.group(1)) if _name3 else None
brief3_base = int(_brief3.group(1)) if _brief3 else None
name4_base = int(_name4.group(1)) if _name4 else None
brief4_base = int(_brief4.group(1)) if _brief4 else None
name5_base = int(_name5.group(1)) if _name5 else None
brief5_base = int(_brief5.group(1)) if _brief5 else None
name6_base = int(_name6.group(1)) if _name6 else None
brief6_base = int(_brief6.group(1)) if _brief6 else None
name7_base = int(_name7.group(1)) if _name7 else None
brief7_base = int(_brief7.group(1)) if _brief7 else None


def _seg_rows(base, base2, base3, base4, base5, base6, base7, n):
    """段一 0..min(n,291) + 段二..段七续号（扩池批）；返回行号列表"""
    out = [base + i for i in range(min(n, _SEG1))]
    if n > _SEG1:
        assert base2 is not None, "扩池后段二基址缺失（pack_text.py 未登记）"
        out += [base2 + i for i in range(min(n, _SEG2) - _SEG1)]
    if n > _SEG2:
        assert base3 is not None, "扩池后段三基址缺失（pack_text.py 未登记）"
        out += [base3 + i for i in range(min(n, _SEG3) - _SEG2)]
    if n > _SEG3:
        assert base4 is not None, "扩池后段四基址缺失（pack_text.py 未登记）"
        out += [base4 + i for i in range(min(n, _SEG4) - _SEG3)]
    if n > _SEG4:
        assert base5 is not None, "扩池后段五基址缺失（pack_text.py 未登记）"
        out += [base5 + i for i in range(min(n, _SEG5) - _SEG4)]
    if n > _SEG5:
        assert base6 is not None, "扩池后段六基址缺失（pack_text.py 未登记）"
        out += [base6 + i for i in range(min(n, _SEG6) - _SEG5)]
    if n > _SEG6:
        assert base7 is not None, "扩池后段七基址缺失（pack_text.py 未登记）"
        out += [base7 + i for i in range(n - _SEG6)]
    return out
brief_base = int(re.search(r"pub static TXT_BRIEF_BASE: u32 = (\d+);", inc).group(1))

tables = {}
for tname, base, base2, base3, base4, base5, base6, base7 in (
        ("EVENT_NAMES", name_base, name2_base, name3_base, name4_base, name5_base, name6_base, name7_base),
        ("EVENT_BRIEF", brief_base, brief2_base, brief3_base, brief4_base, brief5_base, brief6_base, brief7_base)):
    entries = []
    for n in _seg_rows(base, base2, base3, base4, base5, base6, base7, n_events):
        assert n in rows, f"{tname} 段缺行 N={n}（基址漂移/收编段不齐）"
        val = decode_raw(rows[n])
        assert val is not None, f"{tname} N={n} 不可解码"
        entries.append(val)
    tables[tname] = entries
    print(tname, "base", base, "base2", base2, "base3", base3, "base4", base4,
          "base5", base5, "base6", base6, "base7", base7, "rows", len(entries),
          "aligned:", len(entries) == n_events == 338)

names = tables["EVENT_NAMES"]
seen = set()
dup = []
for e in names:
    base = e.replace("·续", "")
    if base in seen:
        dup.append(base)
    seen.add(base)
print("NAMES distinct bases:", len(seen), "dup:", dup if dup else "none")

# EvTable 静态声明与段长一致（len 七段：len2..len6 段界常量差、len7 末段
# N_EVENTS−EV_SEG6——len 写 N_EVENTS−EV_SEGx 会随扩池静默错位，正则钉段界
# 常量形态；字段间容忍行注释）
_C = r"(?:\s*//[^\n]*)*"
for tname in ("EVENT_NAMES", "EVENT_BRIEF"):
    pat = (
        r"pub static %s: EvTable = EvTable \{" % tname + _C +
        r"\s*base: \w+ as usize," + _C + r"\s*len: \w+," + _C +
        r"\s*base2: \w+ as usize," + _C + r"\s*len2: \w+ - \w+," + _C +
        r"\s*base3: \w+ as usize," + _C + r"\s*len3: \w+ - \w+," + _C +
        r"\s*base4: \w+ as usize," + _C + r"\s*len4: \w+ - \w+," + _C +
        r"\s*base5: \w+ as usize," + _C + r"\s*len5: \w+ - \w+," + _C +
        r"\s*base6: \w+ as usize," + _C + r"\s*len6: \w+ - \w+," + _C +
        r"\s*base7: \w+ as usize," + _C + r"\s*len7: \w+ - \w+,\s*\};"
    )
    m = re.search(pat, src)
    assert m, f"{tname} EvTable 声明缺失"
print("EvTable statics 七段声明在位，len() == n_events:", n_events)

# ---------------- v1.16.29 单1刀B：表通道扩容登记 ----------------
# 新三段连号整表（NPC_TALES/HELP_LINES/HINTS text）：基址在 .inc，行数钉死，
# 行全在真相源且可解码非空；结构体表（CHAPTERS/ENDINGS/LOCATIONS）字段级 _n N
# 全部在真相源可解码（逐字段回读 data.rs/game.rs 表体）。
_TALES_BASE = int(re.search(r"pub static TXT_TALES_BASE: u32 = (\d+);", inc).group(1))
_HELP_BASE = int(re.search(r"pub static TXT_HELP_BASE: u32 = (\d+);", inc).group(1))
_HINT_BASE = int(re.search(r"pub static TXT_HINT_BASE: u32 = (\d+);", inc).group(1))
for _tname, _base, _cnt in (("NPC_TALES", _TALES_BASE, 12),
                            ("HELP_LINES", _HELP_BASE, 17),
                            ("HINTS.text", _HINT_BASE, 14)):
    for _i in range(_cnt):
        _n = _base + _i
        assert _n in rows, f"{_tname} 段缺行 N={_n}"
        _v = decode_raw(rows[_n])
        assert _v is not None and _v != "", f"{_tname} N={_n} 空/不可解码"
    print(_tname, "base", _base, "rows", _cnt, "all decode: OK")

# 结构体表字段级：表体内 `field_n: N` / `lore_ns: S` 全部界内可解码
data_txt = src  # data.rs（上文已读）
game_src = open("src/game.rs", encoding="utf-8").read()
_field_ns = [int(x) for x in re.findall(r"\b(?:intro_n|open_n|reward_note_n|couplet_n|desc_n|name_n|alias_n|body_n|gain_n): (\d+)", data_txt + game_src)]
_field_ns += [int(x) for x in re.findall(r"\btext_n: (\d+)", open("src/guide.rs", encoding="utf-8").read())]
_lore_starts = [int(x) for x in re.findall(r"\blore_ns: (\d+)", data_txt)]
assert len(_field_ns) == 9 * 4 + 4 * 4 + 6 + 14, \
    f"字段级 N 总数 {len(_field_ns)} != 72（CHAPTERS 36 + ENDINGS 16 + desc 6 + HINTS text 14）"
for _n in _field_ns + _lore_starts:
    assert _n in rows, f"结构体表字段 N={_n} 缺行"
    _v = decode_raw(rows[_n])
    assert _v is not None and _v != "", f"结构体表字段 N={_n} 空/不可解码"
print("struct-table field Ns:", len(_field_ns), "+ lore starts:", len(_lore_starts), "all decode: OK")
