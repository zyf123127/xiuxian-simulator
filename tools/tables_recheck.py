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
# 照段四→段五先例），故段七起于 1954）；
# v1.16.32 单3 扩池起八段连号——段八基址 TXT_NAME8_BASE/TXT_BRIEF8_BASE=
# 2338/2344，与 data.rs EV_SEG7=338 同批登记（单1/单2 已用 N1971..2337 故
# 段八顺延），故段八起于 2338。
# v1.16.33 单3 扩池起九段连号——段九基址 TXT_NAME9_BASE/TXT_BRIEF9_BASE=
# 2576/2581，与 data.rs EV_SEG8=344 快照常量同批登记（单1 N 化批已用
# N2368..2575 故段九顺延），故段九起于 2576。
# v1.16.34 单3 扩池起十段连号——段十基址 TXT_NAME10_BASE/TXT_BRIEF10_BASE=
# 2647/2649，与 data.rs EV_SEG9=349 快照常量同批登记（单1 精度/臂表批已用
# N2601..2646 故段十顺延），故段十起于 2647。
# v1.16.35 单3 扩池起十一段连号——段十一基址 TXT_NAME11_BASE/TXT_BRIEF11_BASE=
# 2657/2659，与 data.rs EV_SEG10=351 快照常量同批登记（单2 加载链批未用正文 N
# 故段十一顺延），故段十一起于 2657。
# v1.16.36 单3 扩池起十二段连号——段十二基址 TXT_NAME12_BASE/TXT_BRIEF12_BASE=
# 2667/2670，与 data.rs EV_SEG11=353 快照常量同批登记（单1 快照预建+单2 零改动
# 未用正文 N 故段十二顺延），故段十二起于 2667。
# v1.16.37 单2 扩池起十三段连号——段十三基址 TXT_NAME13_BASE/TXT_BRIEF13_BASE=
# 2684/2687，与 data.rs EV_SEG12=356 快照常量同批登记（v1.16.36 正文收编至
# N2683 故段十三顺延；另手写检定副标行 N2690「受缶一问」不入名/简段），故段
# 十三起于 2684。
# 对账：段行数 == EVENTS 行数 == 359、NAMES 去「·续」后互异、基址段界内。只读。
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
_SEG7 = 338
_SEG8 = 344
_SEG9 = 349
_SEG10 = 351
# v1.16.36 单3 扩段十二：EV_SEG11=353 快照段界（单1 预建本批引用）
_SEG11 = 353
# v1.16.37 单2 扩段十三：EV_SEG12=356 快照段界（v1.16.36 单3 预建本批引用）
_SEG12 = 356
# v1.16.38 单2 扩段十四：EV_SEG13=359 快照段界（v1.16.37 单2 预建本批引用）
_SEG13 = 359
# v1.16.40 单3 扩段十五：EV_SEG14=362 快照段界（v1.16.38 单2 预建本批引用）
_SEG14 = 362
# v1.16.43 单3 扩段十六：EV_SEG15=365 快照段界（v1.16.40 单3 预建本批引用）
_SEG15 = 365
# v1.16.44 扩段十七：EV_SEG16=368 快照段界（v1.16.43 单3 预建本批引用）
_SEG16 = 368
# v1.16.45 施工单三扩段十八：EV_SEG17=371 快照段界（v1.16.44 预建本批引用；
# 段十七 len17 本批改差兑现——_seg_rows 段十七块同步改有界）
_SEG17 = 371
# v1.16.46 段十九扩段十九：EV_SEG18=374 快照段界（v1.16.45 施工单三预建本批引
# 用；段十八 len18 本批改差兑现——_seg_rows 段十八块同步改有界）
# v1.16.48 段二十一错位修复：本值原误写 377（各批把「新 N_EVENTS」误写进上一
# 位 _SEG 的存量 bug——v1.16.46 引入 377、v1.16.47 引入 380 一档连错），总行数
# 恰 380 使 aligned 断言巧合通过而 N2804..2806（段十八）与 N2832..2837（段二十
# 名/简）零覆盖；本批修直回 _SEGN=EV_SEGN 一一对应，勿再把新 N_EVENTS 写进 _SEG
_SEG18 = 374
# v1.16.47 段二十扩段二十：EV_SEG19=377 快照段界（v1.16.46 段十九预建本批引
# 用；段十九 len19 本批改差兑现——_seg_rows 段十九块同步改有界）
_SEG19 = 377
# v1.16.48 段二十一扩段二十一：EV_SEG20=380 快照段界（v1.16.47 段二十预建本
# 批引用；段二十 len20 本批改差兑现——_seg_rows 段二十块同步改有界）
_SEG20 = 380
# v1.16.49 段二十二扩段二十二：EV_SEG21=383 快照段界（v1.16.48 段二十一预建本
# 批引用；段二十一 len21 本批改差兑现——_seg_rows 段二十一块同步改有界）
_SEG21 = 383
# v1.16.49 段二十二开设：EV_SEG22=386 快照段界（本批预建下批引用；照 v1.16.48
# 修直纪律 _SEGN=EV_SEGN 一一对应，勿把新 N_EVENTS 写进 _SEG）
_SEG22 = 386
# v1.16.50 段二十三开设：EV_SEG23=389 快照段界（本批预建下批引用；照 v1.16.48
# 修直纪律 _SEGN=EV_SEGN 一一对应，勿把新 N_EVENTS 写进 _SEG）
_SEG23 = 389
# v1.16.51 段二十四开设：EV_SEG24=392 快照段界（本批预建下批引用；照 v1.16.48
# 修直纪律 _SEGN=EV_SEGN 一一对应，勿把新 N_EVENTS 写进 _SEG）
_SEG24 = 392
# v1.16.36 单5 low 清账：对账段数常量（段一..段十五）——标签行随此动态写，
# 下批扩段十六时随批 +1（勘察建议「随段数常量动态改写」款）
# v1.16.40 单3 扩段十五：SEGS_COUNT 14→15 随批
# v1.16.43 单3 扩段十六：SEGS_COUNT 15→16 随批
# v1.16.44 扩段十七：SEGS_COUNT 16→17 随批
# v1.16.45 施工单三扩段十八：SEGS_COUNT 17→18 随批
# v1.16.46 段十九扩池：SEGS_COUNT 18→19 随批
# v1.16.47 段二十扩池：SEGS_COUNT 19→20 随批
# v1.16.48 段二十一扩池：SEGS_COUNT 20→21 随批
# v1.16.49 段二十二扩池：SEGS_COUNT 21→22 随批
# v1.16.50 段二十三扩池：SEGS_COUNT 22→23 随批
# v1.16.51 段二十四扩池：SEGS_COUNT 23→24 随批
SEGS_COUNT = 24
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
_name8 = re.search(r"pub static TXT_NAME8_BASE: u32 = (\d+);", inc)
_brief8 = re.search(r"pub static TXT_BRIEF8_BASE: u32 = (\d+);", inc)
name8_base = int(_name8.group(1)) if _name8 else None
brief8_base = int(_brief8.group(1)) if _brief8 else None
_name9 = re.search(r"pub static TXT_NAME9_BASE: u32 = (\d+);", inc)
_brief9 = re.search(r"pub static TXT_BRIEF9_BASE: u32 = (\d+);", inc)
name9_base = int(_name9.group(1)) if _name9 else None
brief9_base = int(_brief9.group(1)) if _brief9 else None
_name10 = re.search(r"pub static TXT_NAME10_BASE: u32 = (\d+);", inc)
_brief10 = re.search(r"pub static TXT_BRIEF10_BASE: u32 = (\d+);", inc)
name10_base = int(_name10.group(1)) if _name10 else None
brief10_base = int(_brief10.group(1)) if _brief10 else None
_name11 = re.search(r"pub static TXT_NAME11_BASE: u32 = (\d+);", inc)
_brief11 = re.search(r"pub static TXT_BRIEF11_BASE: u32 = (\d+);", inc)
name11_base = int(_name11.group(1)) if _name11 else None
brief11_base = int(_brief11.group(1)) if _brief11 else None
# v1.16.36 单3 扩段十二：段十二基址（.inc 回读同款）
_name12 = re.search(r"pub static TXT_NAME12_BASE: u32 = (\d+);", inc)
_brief12 = re.search(r"pub static TXT_BRIEF12_BASE: u32 = (\d+);", inc)
name12_base = int(_name12.group(1)) if _name12 else None
brief12_base = int(_brief12.group(1)) if _brief12 else None
# v1.16.37 单2 扩段十三：段十三基址（.inc 回读同款）
_name13 = re.search(r"pub static TXT_NAME13_BASE: u32 = (\d+);", inc)
_brief13 = re.search(r"pub static TXT_BRIEF13_BASE: u32 = (\d+);", inc)
name13_base = int(_name13.group(1)) if _name13 else None
brief13_base = int(_brief13.group(1)) if _brief13 else None
# v1.16.38 单2 扩段十四：段十四基址（.inc 回读同款）
_name14 = re.search(r"pub static TXT_NAME14_BASE: u32 = (\d+);", inc)
_brief14 = re.search(r"pub static TXT_BRIEF14_BASE: u32 = (\d+);", inc)
name14_base = int(_name14.group(1)) if _name14 else None
brief14_base = int(_brief14.group(1)) if _brief14 else None

# v1.16.40 单3 扩段十五：段十五基址（.inc 回读同款）
_name15 = re.search(r"pub static TXT_NAME15_BASE: u32 = (\d+);", inc)
_brief15 = re.search(r"pub static TXT_BRIEF15_BASE: u32 = (\d+);", inc)
name15_base = int(_name15.group(1)) if _name15 else None
brief15_base = int(_brief15.group(1)) if _brief15 else None

# v1.16.43 单3 扩段十六：段十六基址（.inc 回读同款）
_name16 = re.search(r"pub static TXT_NAME16_BASE: u32 = (\d+);", inc)
_brief16 = re.search(r"pub static TXT_BRIEF16_BASE: u32 = (\d+);", inc)
name16_base = int(_name16.group(1)) if _name16 else None
brief16_base = int(_brief16.group(1)) if _brief16 else None

# v1.16.44 扩段十七：段十七基址（.inc 回读同款）
_name17 = re.search(r"pub static TXT_NAME17_BASE: u32 = (\d+);", inc)
_brief17 = re.search(r"pub static TXT_BRIEF17_BASE: u32 = (\d+);", inc)
name17_base = int(_name17.group(1)) if _name17 else None
brief17_base = int(_brief17.group(1)) if _brief17 else None
# v1.16.45 施工单三扩段十八
_name18 = re.search(r"pub static TXT_NAME18_BASE: u32 = (\d+);", inc)
_brief18 = re.search(r"pub static TXT_BRIEF18_BASE: u32 = (\d+);", inc)
name18_base = int(_name18.group(1)) if _name18 else None
brief18_base = int(_brief18.group(1)) if _brief18 else None

# v1.16.46 段十九扩池：段十九基址（.inc 回读同款）
_name19 = re.search(r"pub static TXT_NAME19_BASE: u32 = (\d+);", inc)
_brief19 = re.search(r"pub static TXT_BRIEF19_BASE: u32 = (\d+);", inc)
name19_base = int(_name19.group(1)) if _name19 else None
brief19_base = int(_brief19.group(1)) if _brief19 else None

# v1.16.47 段二十扩池：段二十基址（.inc 回读同款）
_name20 = re.search(r"pub static TXT_NAME20_BASE: u32 = (\d+);", inc)
_brief20 = re.search(r"pub static TXT_BRIEF20_BASE: u32 = (\d+);", inc)
name20_base = int(_name20.group(1)) if _name20 else None
brief20_base = int(_brief20.group(1)) if _brief20 else None

# v1.16.48 段二十一扩池：段二十一基址（.inc 回读同款）
_name21 = re.search(r"pub static TXT_NAME21_BASE: u32 = (\d+);", inc)
_brief21 = re.search(r"pub static TXT_BRIEF21_BASE: u32 = (\d+);", inc)
name21_base = int(_name21.group(1)) if _name21 else None
brief21_base = int(_brief21.group(1)) if _brief21 else None
# v1.16.49 段二十二扩池：段二十二基址（.inc 回读同款）
_name22 = re.search(r"pub static TXT_NAME22_BASE: u32 = (\d+);", inc)
_brief22 = re.search(r"pub static TXT_BRIEF22_BASE: u32 = (\d+);", inc)
name22_base = int(_name22.group(1)) if _name22 else None
brief22_base = int(_brief22.group(1)) if _brief22 else None
# v1.16.50 段二十三扩池：段二十三基址（.inc 回读同款）
_name23 = re.search(r"pub static TXT_NAME23_BASE: u32 = (\d+);", inc)
_brief23 = re.search(r"pub static TXT_BRIEF23_BASE: u32 = (\d+);", inc)
name23_base = int(_name23.group(1)) if _name23 else None
brief23_base = int(_brief23.group(1)) if _brief23 else None
# v1.16.51 段二十四扩池：段二十四基址（.inc 回读同款）
_name24 = re.search(r"pub static TXT_NAME24_BASE: u32 = (\d+);", inc)
_brief24 = re.search(r"pub static TXT_BRIEF24_BASE: u32 = (\d+);", inc)
name24_base = int(_name24.group(1)) if _name24 else None
brief24_base = int(_brief24.group(1)) if _brief24 else None


def _seg_rows(base, base2, base3, base4, base5, base6, base7, base8, base9,
              base10, base11, base12, base13, base14, base15, base16, base17,
              base18, base19, base20, base21, base22, base23, base24, n):
    """段一 0..min(n,291) + 段二..段十四续号（扩池批）；返回行号列表"""
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
        out += [base7 + i for i in range(min(n, _SEG7) - _SEG6)]
    if n > _SEG7:
        assert base8 is not None, "扩池后段八基址缺失（pack_text.py 未登记）"
        out += [base8 + i for i in range(min(n, _SEG8) - _SEG7)]
    if n > _SEG8:
        assert base9 is not None, "扩池后段九基址缺失（pack_text.py 未登记）"
        # v1.16.34 单3：段九块改有界 min(n,_SEG9)−_SEG8（段十开设后段九不再
        # 是末段——旧行为 n−_SEG8 在扩池后会把段十行错读进段九，红线②口径）
        out += [base9 + i for i in range(min(n, _SEG9) - _SEG8)]
    if n > _SEG9:
        assert base10 is not None, "扩池后段十基址缺失（pack_text.py 未登记）"
        # v1.16.35 单3：段十块改有界 min(n,_SEG10)−_SEG9（段十一开设后段十不再
        # 是末段——旧行为 n−_SEG9 在扩池后会把段十一行错读进段十，红线②口径）
        out += [base10 + i for i in range(min(n, _SEG10) - _SEG9)]
    if n > _SEG10:
        assert base11 is not None, "扩池后段十一基址缺失（pack_text.py 未登记）"
        # v1.16.36 单3：段十一块改有界 min(n,_SEG11)−_SEG10（段十二开设后段十一
        # 不再是末段——旧行为 n−_SEG10 在扩池后会把段十二行错读进段十一，红线②口径）
        out += [base11 + i for i in range(min(n, _SEG11) - _SEG10)]
    if n > _SEG11:
        assert base12 is not None, "扩池后段十二基址缺失（pack_text.py 未登记）"
        # v1.16.37 单2：段十二块改有界 min(n,_SEG12)−_SEG11（段十三开设后段十二
        # 不再是末段——旧行为 n−_SEG11 在扩池后会把段十三行错读进段十二，红线②口径）
        out += [base12 + i for i in range(min(n, _SEG12) - _SEG11)]
    if n > _SEG12:
        assert base13 is not None, "扩池后段十三基址缺失（pack_text.py 未登记）"
        # v1.16.38 单2：段十三块改有界 min(n,_SEG13)−_SEG12（段十四开设后段十三
        # 不再是末段——旧行为 n−_SEG12 在扩池后会把段十四行错读进段十三，红线②口径）
        out += [base13 + i for i in range(min(n, _SEG13) - _SEG12)]
    if n > _SEG13:
        assert base14 is not None, "扩池后段十四基址缺失（pack_text.py 未登记）"
        # v1.16.40 单3 红线②：段十四由无界末段改有界（段十五开设后段十四
        # 不再是末段——旧行为 n−_SEG13 在扩池后会把段十五行错读进段十四）
        out += [base14 + i for i in range(min(n, _SEG14) - _SEG13)]
    if n > _SEG14:
        assert base15 is not None, "扩池后段十五基址缺失（pack_text.py 未登记）"
        # v1.16.43 单3 红线②：段十五由无界末段改有界（段十六开设后段十五
        # 不再是末段——旧行为 n−_SEG14 在扩池后会把段十六行错读进段十五）
        out += [base15 + k for k in range(min(n, _SEG15) - _SEG14)]
    if n > _SEG15:
        assert base16 is not None, "扩池后段十六基址缺失（pack_text.py 未登记）"
        # v1.16.44 红线②：段十六由无界末段改有界（段十七开设后段十六不再
        # 是末段——旧行为 n−_SEG15 在扩池后会把段十七行错读进段十六）
        out += [base16 + k for k in range(min(n, _SEG16) - _SEG15)]
    if n > _SEG16:
        assert base17 is not None, "扩池后段十七基址缺失（pack_text.py 未登记）"
        # v1.16.45 施工单三红线②：段十七由无界末段改有界（段十八开设后段十七
        # 不再是末段——旧行为 n−_SEG16 在扩池后会把段十八行错读进段十七）
        out += [base17 + k for k in range(min(n, _SEG17) - _SEG16)]
    if n > _SEG17:
        assert base18 is not None, "扩池后段十八基址缺失（pack_text.py 未登记）"
        # v1.16.46 段十九红线②：段十八由无界末段改有界（段十九开设后段十八
        # 不再是末段——旧行为 n−_SEG17 在扩池后会把段十九行错读进段十八）
        out += [base18 + k for k in range(min(n, _SEG18) - _SEG17)]
    if n > _SEG18:
        assert base19 is not None, "扩池后段十九基址缺失（pack_text.py 未登记）"
        # v1.16.47 段二十红线②：段十九由无界末段改有界（段二十开设后段十九
        # 不再是末段——旧行为 n−_SEG18 在扩池后会把段二十行错读进段十九）
        out += [base19 + k for k in range(min(n, _SEG19) - _SEG18)]
    if n > _SEG19:
        assert base20 is not None, "扩池后段二十基址缺失（pack_text.py 未登记）"
        # v1.16.48 段二十一红线②：段二十由无界末段改有界（段二十一开设后段
        # 二十不再是末段——旧行为 n−_SEG19 在扩池后会把段二十一行错读进段二
        # 十）；修复前本块因 _SEG19 错位恒不触发、修复后随 min(n,_SEG20) 界内
        # 回真行
        out += [base20 + k for k in range(min(n, _SEG20) - _SEG19)]
    if n > _SEG20:
        assert base21 is not None, "扩池后段二十一基址缺失（pack_text.py 未登记）"
        # v1.16.49 段二十二红线②：段二十一由无界末段改有界（段二十二开设后段
        # 二十一不再是末段——旧行为 n−_SEG20 在扩池后会把段二十二行错读进段二
        # 十一）
        out += [base21 + k for k in range(min(n, _SEG21) - _SEG20)]
    if n > _SEG21:
        assert base22 is not None, "扩池后段二十二基址缺失（pack_text.py 未登记）"
        # v1.16.50 段二十三红线：段二十二由无界末段改有界（段二十三开设后段
        # 二十二不再末段——旧行为 n−_SEG21 在扩池后会把段二十三行错读进段二
        # 十二，照 v1.16.49 段二十一块同款纪律）
        out += [base22 + k for k in range(min(n, _SEG22) - _SEG21)]
    if n > _SEG22:
        assert base23 is not None, "扩池后段二十三基址缺失（pack_text.py 未登记）"
        # v1.16.51 段二十四红线：段二十三由无界末段改有界（段二十四开设后段
        # 二十三不再末段——旧行为 n−_SEG22 在扩池后会把段二十四行错读进段二
        # 十三，照 v1.16.50 段二十二块同款纪律）
        out += [base23 + k for k in range(min(n, _SEG23) - _SEG22)]
    if n > _SEG23:
        assert base24 is not None, "扩池后段二十四基址缺失（pack_text.py 未登记）"
        # v1.16.51 段二十四开设：段二十四无界末段（下批扩段二十五时改有界）
        out += [base24 + k for k in range(n - _SEG23)]
    return out
brief_base = int(re.search(r"pub static TXT_BRIEF_BASE: u32 = (\d+);", inc).group(1))

tables = {}
for tname, base, base2, base3, base4, base5, base6, base7, base8, base9, base10, base11, base12, base13, base14, base15, base16, base17, base18, base19, base20, base21, base22, base23, base24 in (
        ("EVENT_NAMES", name_base, name2_base, name3_base, name4_base, name5_base, name6_base, name7_base, name8_base, name9_base, name10_base, name11_base, name12_base, name13_base, name14_base, name15_base, name16_base, name17_base, name18_base, name19_base, name20_base, name21_base, name22_base, name23_base, name24_base),
        ("EVENT_BRIEF", brief_base, brief2_base, brief3_base, brief4_base, brief5_base, brief6_base, brief7_base, brief8_base, brief9_base, brief10_base, brief11_base, brief12_base, brief13_base, brief14_base, brief15_base, brief16_base, brief17_base, brief18_base, brief19_base, brief20_base, brief21_base, brief22_base, brief23_base, brief24_base)):
    entries = []
    for n in _seg_rows(base, base2, base3, base4, base5, base6, base7, base8,
                       base9, base10, base11, base12, base13, base14, base15, base16, base17, base18, base19, base20, base21, base22, base23, base24, n_events):
        assert n in rows, f"{tname} 段缺行 N={n}（基址漂移/收编段不齐）"
        val = decode_raw(rows[n])
        assert val is not None, f"{tname} N={n} 不可解码"
        entries.append(val)
    tables[tname] = entries
    # v1.16.36 单3 low 清账：打印行补 base10 列（v1.16.34 起实缺——base9 后直接
    # base11）并随扩段加 base12 列；v1.16.37 单2 随扩段加 base13 列；v1.16.38
    # 单2 随扩段加 base14 列；aligned 硬编码 359→362 随批
    print(tname, "base", base, "base2", base2, "base3", base3, "base4", base4,
          "base5", base5, "base6", base6, "base7", base7, "base8", base8,
          "base9", base9, "base10", base10, "base11", base11, "base12", base12,
          "base13", base13, "base14", base14, "base15", base15, "base16", base16,
          "base17", base17, "base18", base18, "base19", base19,
          "base20", base20, "base21", base21, "base22", base22,
          # v1.16.50 段二十三：打印行随扩段 +1 列 base23（照 base22 随批款）
          "base23", base23,
          # v1.16.51 段二十四：打印行随扩段 +1 列 base24（同上款）
          "base24", base24,
          "rows", len(entries), "aligned:",
          # v1.16.48：aligned 去硬编码 380 改随 N_EVENTS 动态（380 硬编码在
          # 扩池批每轮过期，且曾掩盖 _SEG 错位巧合——对账本身即行数全等断言）
          len(entries) == n_events)

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
        # v1.16.32 单1：EvTable 字段窄化 u16、段界差括号化（(N_EVENTS - EV_SEG6)
        # as u16）——形态同步
        r"pub static %s: EvTable = EvTable \{" % tname + _C +
        r"\s*base: \w+ as u16," + _C + r"\s*len: \w+ as u16," + _C +
        r"\s*base2: \w+ as u16," + _C + r"\s*len2: \(?\w+ - \w+\)? as u16," + _C +
        r"\s*base3: \w+ as u16," + _C + r"\s*len3: \(?\w+ - \w+\)? as u16," + _C +
        r"\s*base4: \w+ as u16," + _C + r"\s*len4: \(?\w+ - \w+\)? as u16," + _C +
        r"\s*base5: \w+ as u16," + _C + r"\s*len5: \(?\w+ - \w+\)? as u16," + _C +
        r"\s*base6: \w+ as u16," + _C + r"\s*len6: \(?\w+ - \w+\)? as u16," + _C +
        # v1.16.32 单3 扩段八：base8/len8 双段登记（len8 段界差 EV_SEG8−EV_SEG7）
        # v1.16.33 单3 扩段九：base9/len9 双段登记（len9 末段 N_EVENTS−EV_SEG8）
        r"\s*base7: \w+ as u16," + _C + r"\s*len7: \(?\w+ - \w+\)? as u16," + _C +
        r"\s*base8: \w+ as u16," + _C + r"\s*len8: \(?\w+ - \w+\)? as u16," + _C +
        r"\s*base9: \w+ as u16," + _C + r"\s*len9: \(?\w+ - \w+\)? as u16," + _C +
        # v1.16.34 单3 扩段十：base10/len10 双段登记（len10 末段差写法形态）
        r"\s*base10: \w+ as u16," + _C + r"\s*len10: \(?\w+ - \w+\)? as u16," + _C +
        # v1.16.35 单3 扩段十一：base11/len11 双段登记（len11 末段差写法形态）
        r"\s*base11: \w+ as u16," + _C + r"\s*len11: \(?\w+ - \w+\)? as u16," + _C +
        # v1.16.36 单3 扩段十二：base12/len12 双段登记（len12 末段差写法形态）
        r"\s*base12: \w+ as u16," + _C + r"\s*len12: \(?\w+ - \w+\)? as u16," + _C +
        # v1.16.37 单2 扩段十三：base13/len13 双段登记（len13 末段差写法形态）
        r"\s*base13: \w+ as u16," + _C + r"\s*len13: \(?\w+ - \w+\)? as u16," + _C +
        # v1.16.38 单2 扩段十四：base14/len14 双段登记（len14 末段差写法形态）
        # v1.16.40 单3 扩段十五：base15/len15 双段登记（len15 末段差写法形态）
        r"\s*base14: \w+ as u16," + _C + r"\s*len14: \(?\w+ - \w+\)? as u16," + _C +
        r"\s*base15: \w+ as u16," + _C + r"\s*len15: \(?\w+ - \w+\)? as u16," + _C +
        # v1.16.43 单3 扩段十六：base16/len16 双段登记（len16 末段差写法形态）
        r"\s*base16: \w+ as u16," + _C + r"\s*len16: \(?\w+ - \w+\)? as u16," + _C +
        # v1.16.44 扩段十七：base17/len17 双段登记（len17 末段差写法形态）
        r"\s*base17: \w+ as u16," + _C + r"\s*len17: \(?\w+ - \w+\)? as u16," + _C +
        # v1.16.45 施工单三扩段十八：base18/len18 双段登记（len18 末段差写法形态）
        r"\s*base18: \w+ as u16," + _C + r"\s*len18: \(?\w+ - \w+\)? as u16," + _C +
        # v1.16.46 段十九扩段十九：base19/len19 双段登记（len19 末段差写法形态）
        # v1.16.47 段二十扩段二十：base20/len20 双段登记随批（len20 末段差写法
        # 形态——本批改差兑现 (EV_SEG20−EV_SEG19)，正则 \w+ - \w+ 两形态通吃）
        # v1.16.48 段二十一扩段二十一：base21/len21 双段登记随批（len21 末段差
        # 写法形态）
        # v1.16.49 段二十二扩段二十二：base22/len22 双段登记随批（len22 末段差
        # 写法形态——len21 本批改差兑现 (EV_SEG21−EV_SEG20)，正则 \w+ - \w+ 两
        # 形态通吃）
        # v1.16.50 段二十三扩段二十三：base23/len23 双段登记随批（len23 末段差
        # 写法形态——len22 本批改差兑现 (EV_SEG22−EV_SEG21)，正则两形态通吃）
        # v1.16.51 段二十四扩段：base24/len24 双段登记随批（len23 本批改差兑现
        # (EV_SEG23−EV_SEG22)，正则两形态通吃）
        r"\s*base19: \w+ as u16," + _C + r"\s*len19: \(?\w+ - \w+\)? as u16," + _C +
        r"\s*base20: \w+ as u16," + _C + r"\s*len20: \(?\w+ - \w+\)? as u16," + _C +
        r"\s*base21: \w+ as u16," + _C + r"\s*len21: \(?\w+ - \w+\)? as u16," + _C +
        r"\s*base22: \w+ as u16," + _C + r"\s*len22: \(?\w+ - \w+\)? as u16," + _C +
        r"\s*base23: \w+ as u16," + _C + r"\s*len23: \(?\w+ - \w+\)? as u16," + _C +
        r"\s*base24: \w+ as u16," + _C + r"\s*len24: \(?\w+ - \w+\)? as u16,\s*\};"
    )
    m = re.search(pat, src)
    assert m, f"{tname} EvTable 声明缺失"
# v1.16.36 单5 low 清账：标签随 SEGS_COUNT 动态写（上轮「十段声明在位」实对账
# 十一段漂移先例——硬编码字符串随段数过期）
print(f"EvTable statics {SEGS_COUNT}段声明在位，len() == n_events:", n_events)

# ---------------- v1.16.29 单1刀B：表通道扩容登记 ----------------
# 新三段连号整表（NPC_TALES/HELP_LINES/HINTS text）：基址在 .inc，行数钉死，
# 行全在真相源且可解码非空；结构体表（CHAPTERS/ENDINGS/LOCATIONS）字段级 _n N
# 全部在真相源可解码（逐字段回读 data.rs/game.rs 表体）。
_TALES_BASE = int(re.search(r"pub static TXT_TALES_BASE: u32 = (\d+);", inc).group(1))
_HELP_BASE = int(re.search(r"pub static TXT_HELP_BASE: u32 = (\d+);", inc).group(1))
_HINT_BASE = int(re.search(r"pub static TXT_HINT_BASE: u32 = (\d+);", inc).group(1))
# v1.16.32 单1 第三批 ARR 基址（9 表 63 行）——基址在 .inc，行数钉死，逐行可解码
_ARR2 = {
    _k: int(re.search(r"pub static %s: u32 = (\d+);" % _k, inc).group(1))
    for _k in ("TXT_MINOR_BASE", "TXT_COMPT_BASE", "TXT_NPCBRIEF_BASE",
               "TXT_DFD_BASE", "TXT_WFLAV_BASE", "TXT_EXPCHO_BASE",
               "TXT_ADCREA_BASE", "TXT_DONE_BASE", "TXT_CHAIN_BASE")
}
for _tname, _base, _cnt in (("NPC_TALES", _TALES_BASE, 12),
                            ("HELP_LINES", _HELP_BASE, 17),
                            ("HINTS.text", _HINT_BASE, 14),
                            ("MINOR_LINES", _ARR2["TXT_MINOR_BASE"], 10),
                            ("COMP_TALES", _ARR2["TXT_COMPT_BASE"], 3),
                            ("NPC_BRIEF", _ARR2["TXT_NPCBRIEF_BASE"], 4),
                            ("DONGFU_FAC_DESC", _ARR2["TXT_DFD_BASE"], 3),
                            ("WEATHER_FLAVOR", _ARR2["TXT_WFLAV_BASE"], 5),
                            ("EXPED_CHOICE_LABELS", _ARR2["TXT_EXPCHO_BASE"], 8),
                            ("AD_CREATIVES", _ARR2["TXT_ADCREA_BASE"], 3),
                            ("DONE_NOTE", _ARR2["TXT_DONE_BASE"], 9),
                            ("CHAIN_NAMES", _ARR2["TXT_CHAIN_BASE"], 18)):
    for _i in range(_cnt):
        _n = _base + _i
        assert _n in rows, f"{_tname} 段缺行 N={_n}"
        _v = decode_raw(rows[_n])
        assert _v is not None and _v != "", f"{_tname} N={_n} 空/不可解码"
    print(_tname, "base", _base, "rows", _cnt, "all decode: OK")

# 结构体表字段级：表体内 `field_n: N` / `lore_ns: S` 全部界内可解码
# v1.16.32 单1 第三批：字段级面扩至 376（data+game 327 = CHAPTERS intro/open/
# reward_note/couplet 36 + num 9 + name 134 + desc 136 + ENDINGS alias/body/gain
# 12；text 40 = CHAPTERS goals 18 + EXPED_STEPS 8 + HINTS 14 + NPC_EVENTS 4；
# mech 5 = REGION_MECH；title 4 = NPC_EVENTS）
# v1.16.40 长老委托：字段级面扩至 392（title 4→12 = NPC_EVENTS 4 + data.rs
# CommDef title_n 8；desc 136→144 = CommDef desc_n 8——title_n 扫描面同步
# data_txt 使 8 条委托名同走界内可解码校验）
data_txt = src  # data.rs（上文已读）
game_src = open("src/game.rs", encoding="utf-8").read()
npc_src = open("src/npc.rs", encoding="utf-8").read()
guide_src = open("src/guide.rs", encoding="utf-8").read()
_field_ns = [int(x) for x in re.findall(r"\b(?:intro_n|open_n|reward_note_n|couplet_n|desc_n|name_n|alias_n|body_n|gain_n|num_n): (\d+)", data_txt + game_src)]
_field_ns += [int(x) for x in re.findall(r"\btext_n: (\d+)", guide_src + data_txt + npc_src)]
_field_ns += [int(x) for x in re.findall(r"\bmech_n: (\d+)", data_txt)]
_field_ns += [int(x) for x in re.findall(r"\btitle_n: (\d+)", npc_src + data_txt)]
_lore_starts = [int(x) for x in re.findall(r"\blore_ns: (\d+)", data_txt)]
assert len(_field_ns) == 327 + 40 + 5 + 12 + 8, \
    f"字段级 N 总数 {len(_field_ns)} != 392（v1.16.32 单1 376 + v1.16.40 CommDef title 8 + desc 8，见上注）"
for _n in _field_ns + _lore_starts:
    assert _n in rows, f"结构体表字段 N={_n} 缺行"
    _v = decode_raw(rows[_n])
    assert _v is not None and _v != "", f"结构体表字段 N={_n} 空/不可解码"
print("struct-table field Ns:", len(_field_ns), "+ lore starts:", len(_lore_starts), "all decode: OK")


# ---------------- v1.16.33 单1 NIZE 通道登记 ----------------
# SPECIAL_SUB（game.rs eff_desc fn 内 fn 级 const）+ 8 张 N 表 + 2 条 N 常量：
# 槽数钉死防半收编，N 全部在真相源可解码非空；稀疏槽/兜底行同 N（dedup alloc）
# 的结构由 pack_text.py 产侧 assert 固化，此处只做行存在性与可解码对账。
_nize_game = open("src/game.rs", encoding="utf-8").read()
_mt0 = re.search(r"const SPECIAL_SUB: \[u16; (\d+)\] = \[(.*?)\];", _nize_game, re.S)
assert _mt0, "SPECIAL_SUB 表缺失"
_nize_size = int(_mt0.group(1))
# v1.16.37 单5 修复：剥表内行注释后再取数（单2 槽 76 登记注含「v1.16.37/2368/
# N2690」等数字，原 findall 直抓 body 会把注释数字混入槽数/槽位——槽数断言
# len!=77 与 76 槽错位双红，且当时管道 head 截断吞退出码漏看，本单清账）
_nize_body = re.sub(r"//[^\n]*", "", _mt0.group(2))
_nize_slots = [int(x) for x in re.findall(r"\d+", _nize_body)]
# v1.16.50 段二十三：槽数 79→80 随批（special 覆盖 0..=79——收册人问 386 检定
# 副标「合卷一问」N2885 启用槽 79）
assert len(_nize_slots) == _nize_size == 80, (
    f"SPECIAL_SUB 槽数 {len(_nize_slots)}/{_nize_size} != 80（special 覆盖 0..=79；扩臂须同步改表）")
# v1.16.37 单2 槽 76 启用（哑缶一声 356 检定副标「受缶一问」N2690 手写行）：
# 原断言 0/19/76 三槽同值兜底随启用过时——改钉 0/19 兜底 + 76 启用值反向钉
#（2690 N 不变号纪律下稳定；误改回兜底当场红）
assert _nize_slots[0] == _nize_slots[19], (
    "SPECIAL_SUB 稀疏槽 0/19 未统一指兜底行（原 _ 臂「机缘」）")
assert _nize_slots[76] != _nize_slots[0] and _nize_slots[76] == 2690, (
    f"SPECIAL_SUB 槽 76 应为启用值 2690（受缶一问），实为 {_nize_slots[76]}")
# v1.16.44 槽 77 启用（漏刻无声 369 检定副标「漏停一问」N2783 手写行）
assert _nize_slots[77] != _nize_slots[0] and _nize_slots[77] == 2783, (
    f"SPECIAL_SUB 槽 77 应为启用值 2783（漏停一问），实为 {_nize_slots[77]}")
# v1.16.47 槽 78 启用（晒书使巡 377 检定副标「卷面候勘」N2838 手写行）
assert _nize_slots[78] != _nize_slots[0] and _nize_slots[78] == 2838, (
    f"SPECIAL_SUB 槽 78 应为启用值 2838（卷面候勘），实为 {_nize_slots[78]}")
# v1.16.50 槽 79 启用（收册人问 386 检定副标「合卷一问」N2885 手写行）
assert _nize_slots[79] != _nize_slots[0] and _nize_slots[79] == 2885, (
    f"SPECIAL_SUB 槽 79 应为启用值 2885（合卷一问），实为 {_nize_slots[79]}")
for _n in _nize_slots:
    assert _n in rows, f"SPECIAL_SUB N={_n} 缺行"
    _v = decode_raw(rows[_n])
    assert _v is not None and _v != "", f"SPECIAL_SUB N={_n} 空/不可解码"
print("SPECIAL_SUB slots:", _nize_size, "fallback N:", _nize_slots[0],
      "slot76 N:", _nize_slots[76], "all decode: OK")

_NIZE_TABLES = (
    ("COMP_PAGE_PREWARM_NS", "src/charpanel.rs", 30),
    ("COMP_PAGE_PREWARM_16_NS", "src/charpanel.rs", 3),
    ("HIST_PREWARM_NS", "src/charpanel.rs", 14),
    ("PREWARM_STRS_NS", "src/gallery.rs", 21),
    ("EV_NOTE_PREWARM_NS", "src/gallery.rs", 24),
    ("RECALL_CHROME_NS", "src/gallery.rs", 3),
    ("GUIDE_PANEL_LINES_NS", "src/draw.rs", 5),
    ("GUIDE_TABS_NS", "src/draw.rs", 3),  # v1.16.40 长老委托扩第三签「门中差事」（N2727）
    # v1.16.34 单1 β+γ（--nize=arms）：四簇 match 臂 N 表随批扩槽（槽数钉死
    # 防半收编；等价性验收 tools/arms_sweep_check.py git HEAD 逐 slot 全等）
    ("SIM_DEMON_SCENE_NS", "src/game.rs", 4),
    ("SETTLE_CODA_NS", "src/game.rs", 3),
    ("FINALE_SUB_NS", "src/draw.rs", 4),
    ("REALM_TITLE_NS", "src/data.rs", 15),
)
for _tname, _tfile, _tcnt in _NIZE_TABLES:
    _tsrc = open(_tfile, encoding="utf-8").read()
    _mt = re.search(r"const %s: \[u16; (\d+)\] = \[(.*?)\];" % _tname, _tsrc, re.S)
    assert _mt, f"{_tname} N 表缺失"
    _ns2 = [int(x) for x in re.findall(r"\d+", _mt.group(2))]
    assert len(_ns2) == int(_mt.group(1)) == _tcnt, (
        f"{_tname} 槽数 {len(_ns2)}/{_mt.group(1)} != 期望 {_tcnt}")
    for _n in _ns2:
        assert _n in rows, f"{_tname} N={_n} 缺行"
        _v = decode_raw(rows[_n])
        assert _v is not None and _v != "", f"{_tname} N={_n} 空/不可解码"
    print(_tname, "rows", _tcnt, "all decode: OK")

for _cname, _cfile in (("DUAL_CULT_HINT_N", "src/charpanel.rs"),
                       ("GUIDE_PANEL_NOTE_N", "src/draw.rs")):
    _csrc = open(_cfile, encoding="utf-8").read()
    _mc = re.search(r"const %s: u16 = (\d+);" % _cname, _csrc)
    assert _mc, f"{_cname} 常量缺失"
    _n = int(_mc.group(1))
    assert _n in rows, f"{_cname} N={_n} 缺行"
    _v = decode_raw(rows[_n])
    assert _v is not None and _v != "", f"{_cname} N={_n} 空/不可解码"
    print(_cname, "N", _n, "decode: OK")
