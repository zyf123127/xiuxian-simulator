# -*- coding: utf-8 -*-
"""title_recheck.py —— v1.16.30 单1 主刀②静态纪律钉（card.title 查表化对账）。

背景（v1.16.30 单1）：build_event 弹卡一次性路径的 card.title 曾是 317 臂各写
一行 `card.title = "...".into();` 字面量，与 data.rs EVENT_NAMES（textpack 名段）
构成「双源」。单1 主刀②删除 317 臂字面量行，构造改 `title: EVENT_NAMES[id].into()`
单源化——本脚本固化为常驻静态钉，看守三件事：

  钉A（臂内禁 card.title 赋值）：build_event 函数体内不得再出现任何
     `card.title = ...` 行（字面量或动态都禁）——标题唯一来源是构造行查表。
     若未来某臂需要「与表不同的标题」，必须先回评审改设计，不许静默回填字面量。
  钉B（名段覆盖断言）：EVENT_NAMES 名段（段一..段九基址 × EV_SEG 段界，段八自
     v1.16.32 单3 扩池批起、段九自 v1.16.33 单3 扩池批起，另有钉B-2 段八/段九
     首尾内容钉——错行非空串三查抓不住，内容钉兜底）对
     id 0..=N_EVENTS-1 逐条可解析、非空、行号=N 不错位——段基址/段界漂移当场红。
  钉C（构造行单源断言）：臂内零赋值形态下，EventCard 构造行必须存在
     `title: EVENT_NAMES[id].into()`——标题单源链断不得。

兼容模式（--reconcile）：若臂内仍存在赋值行（手术前状态或回滚验证），按
「id→名段 N 映射」逐 id 全等比对，并断言 id 全覆盖、每臂恰 1 次赋值——
这正是单1 手术前做过的一次性对账，留作复跑证据入口。

对账口径（手术前已全绿一次，证据见当轮施工汇报）：
  - 名段映射：id<EV_SEG1 → TXT_NAME_BASE+id；EV_SEG1..EV_SEG2 →
    TXT_NAME2_BASE+id−EV_SEG1；EV_SEG2..EV_SEG3 → TXT_NAME3_BASE+id−EV_SEG2；
    EV_SEG3..EV_SEG4 → TXT_NAME4_BASE+id−EV_SEG3（v1.16.29 起五段时补段四
    ——本注当时滞后只写到段三口径，v1.16.31 随批补全）；
    EV_SEG4..EV_SEG5 → TXT_NAME5_BASE+id−EV_SEG4；EV_SEG5..EV_SEG6 →
    TXT_NAME6_BASE+id−EV_SEG5；≥EV_SEG6 →
    TXT_NAME7_BASE+id−EV_SEG6；≥EV_SEG7 → TXT_NAME8_BASE+id−EV_SEG7（段八）；
    ≥EV_SEG8 → TXT_NAME9_BASE+id−EV_SEG8（段九，v1.16.33 随批补全——本注
    此前滞后只写到段七口径）；≥EV_SEG9 → TXT_NAME10_BASE+id−EV_SEG9
    （段十，v1.16.34 随批补全）；≥EV_SEG10 → TXT_NAME11_BASE+id−EV_SEG10
    （段十一，v1.16.35 随批补全）；≥EV_SEG11 → TXT_NAME12_BASE+id−EV_SEG11
    （段十二，v1.16.36 随批补全）；≥EV_SEG12 → TXT_NAME13_BASE+id−EV_SEG12
    （段十三，v1.16.37 随批补全）。段界/基址常量直接从
    src/data.rs 与 src/text_offsets.inc 解析，不硬编码。
  - text_source.txt 行号=N（0 基），真相源权威。

纪律注：
  - 本脚本只读 src/ 与 assets/，不写任何产物。
  - 循环变量一律 k/eid（勿用 i——v1.16.28 pack_text.py 循环变量泄漏压行事故纪律）。
  - 字面量解析要求纯形 `card.title = "...".into();`（grep 已证手术前 317 行全为
    纯形；标题域禁 \ 转义与动态拼接——动态赋值行在 reconcile 下报非纯形红）。

用法：PYTHONIOENCODING=utf-8 python tools/title_recheck.py [--reconcile]
"""
import re
import sys
from pathlib import Path

# GBK 控制台直跑在末行 print ✓ 处 UnicodeEncodeError 崩（v1.16.31 实证：-I 隔离
# 模式吞 PYTHONIOENCODING）——补 tables_recheck.py/pack_text.py 同款 stdout 包装
sys.stdout = __import__("io").TextIOWrapper(sys.stdout.buffer, encoding="utf-8")

ROOT = Path(__file__).resolve().parent.parent
GAME = ROOT / "src" / "game.rs"
DATA = ROOT / "src" / "data.rs"
INC = ROOT / "src" / "text_offsets.inc"
SRC = ROOT / "assets" / "text_source.txt"

# 构造行锚（钉C）：EventCard 构造里 title 单源查表
CTOR_ANCHOR = re.compile(r"title:\s*EVENT_NAMES\[id\]\.into\(\)")
# 臂内纯形赋值行：`card.title = "...".into();` 独立行
ARM_TITLE_PURE = re.compile(r'^\s*card\.title\s*=\s*"([^"]*)"\.into\(\);\s*$')
ARM_TITLE_ANY = re.compile(r"card\.title\s*=")
# 外层 match 臂首（恰好 12 空格缩进）：`28 => {`
ARM_START = re.compile(r"^( {12})(\d+) => \{\s*$")
# 外层 match 臂尾（恰好 12 空格缩进）：`}` 或 `},`
ARM_END = re.compile(r"^ {12}\},?\s*$")


def parse_const(text, name):
    m = re.search(r"(?:pub\s+)?const\s+" + name + r":\s*usize\s*=\s*(\d+)", text)
    if not m:
        raise SystemExit("FATAL: 常量 " + name + " 在 data.rs 解析不到")
    return int(m.group(1))


def parse_inc_base(name):
    pat = re.compile(r"pub\s+static\s+" + name + r":\s*u32\s*=\s*(\d+);")
    for ln in INC.read_text(encoding="utf-8").splitlines():
        m = pat.search(ln)
        if m:
            return int(m.group(1))
    raise SystemExit("FATAL: " + name + " 在 text_offsets.inc 解析不到")


def build_event_span(lines):
    """返回 (start,end) 0 基半开区间：fn build_event 行 到 下一个同级 fn 前。"""
    st = None
    for k, ln in enumerate(lines):
        if ln.startswith("    fn build_event("):
            st = k
            break
    if st is None:
        raise SystemExit("FATAL: build_event 找不到")
    for k in range(st + 1, len(lines)):
        if re.match(r"    (pub )?fn \w+", lines[k]):
            return st, k
    raise SystemExit("FATAL: build_event 终界找不到")


def main():
    want_reconcile = "--reconcile" in sys.argv
    data_txt = DATA.read_text(encoding="utf-8")
    n_events = parse_const(data_txt, "N_EVENTS")
    seg1 = parse_const(data_txt, "EV_SEG1")
    seg2 = parse_const(data_txt, "EV_SEG2")
    seg3 = parse_const(data_txt, "EV_SEG3")
    seg4 = parse_const(data_txt, "EV_SEG4")
    seg5 = parse_const(data_txt, "EV_SEG5")
    seg6 = parse_const(data_txt, "EV_SEG6")
    seg7 = parse_const(data_txt, "EV_SEG7")
    seg8 = parse_const(data_txt, "EV_SEG8")
    seg9 = parse_const(data_txt, "EV_SEG9")
    seg10 = parse_const(data_txt, "EV_SEG10")
    # v1.16.36 单3 扩段十二：EV_SEG11=353 快照段界（单1 预建本批引用）
    seg11 = parse_const(data_txt, "EV_SEG11")
    # v1.16.37 单2 扩段十三：EV_SEG12=356 快照段界（v1.16.36 单3 预建本批引用）
    seg12 = parse_const(data_txt, "EV_SEG12")
    # v1.16.38 单2 扩段十四：EV_SEG13=359 快照段界（v1.16.37 单2 预建本批引用）
    seg13 = parse_const(data_txt, "EV_SEG13")
    # v1.16.40 单3 扩段十五：EV_SEG14=362 快照段界（v1.16.38 单2 预建本批引用）
    seg14 = parse_const(data_txt, "EV_SEG14")
    # v1.16.43 单3 扩段十六：EV_SEG15=365 快照段界（v1.16.40 单3 预建本批引用）
    seg15 = parse_const(data_txt, "EV_SEG15")
    # v1.16.44 扩段十七：EV_SEG16=368 快照段界（v1.16.43 单3 预建本批引用）
    seg16 = parse_const(data_txt, "EV_SEG16")
    # v1.16.45 施工单三扩段十八：EV_SEG17=371 快照段界（v1.16.44 预建本批引用）
    seg17 = parse_const(data_txt, "EV_SEG17")
    # v1.16.46 段十九扩池：EV_SEG18=374 快照段界（v1.16.45 施工单三预建本批引用）
    seg18 = parse_const(data_txt, "EV_SEG18")
    # v1.16.47 段二十扩池：EV_SEG19=377 快照段界（v1.16.46 段十九预建本批引用）
    seg19 = parse_const(data_txt, "EV_SEG19")
    # v1.16.48 段二十一扩池：EV_SEG20=380 快照段界（v1.16.47 段二十预建本批引用）
    seg20 = parse_const(data_txt, "EV_SEG20")
    seg21 = parse_const(data_txt, "EV_SEG21")
    b1 = parse_inc_base("TXT_NAME_BASE")
    b2 = parse_inc_base("TXT_NAME2_BASE")
    b3 = parse_inc_base("TXT_NAME3_BASE")
    b4 = parse_inc_base("TXT_NAME4_BASE")
    b5 = parse_inc_base("TXT_NAME5_BASE")
    b6 = parse_inc_base("TXT_NAME6_BASE")
    b7 = parse_inc_base("TXT_NAME7_BASE")
    b8 = parse_inc_base("TXT_NAME8_BASE")
    b9 = parse_inc_base("TXT_NAME9_BASE")
    b10 = parse_inc_base("TXT_NAME10_BASE")
    b11 = parse_inc_base("TXT_NAME11_BASE")
    # v1.16.36 单3 扩段十二：段十二基址 TXT_NAME12_BASE
    b12 = parse_inc_base("TXT_NAME12_BASE")
    # v1.16.37 单2 扩段十三：段十三基址 TXT_NAME13_BASE
    b13 = parse_inc_base("TXT_NAME13_BASE")
    # v1.16.38 单2 扩段十四：段十四基址 TXT_NAME14_BASE
    b14 = parse_inc_base("TXT_NAME14_BASE")
    # v1.16.40 单3 扩段十五：段十五基址 TXT_NAME15_BASE
    b15 = parse_inc_base("TXT_NAME15_BASE")
    # v1.16.43 单3 扩段十六：段十六基址 TXT_NAME16_BASE
    b16 = parse_inc_base("TXT_NAME16_BASE")
    # v1.16.44 扩段十七：段十七基址 TXT_NAME17_BASE
    b17 = parse_inc_base("TXT_NAME17_BASE")
    # v1.16.45 施工单三扩段十八：段十八基址 TXT_NAME18_BASE
    b18 = parse_inc_base("TXT_NAME18_BASE")
    # v1.16.46 段十九扩池：段十九基址 TXT_NAME19_BASE
    b19 = parse_inc_base("TXT_NAME19_BASE")
    # v1.16.47 段二十扩池：段二十基址 TXT_NAME20_BASE
    b20 = parse_inc_base("TXT_NAME20_BASE")
    # v1.16.48 段二十一扩池：段二十一基址 TXT_NAME21_BASE
    b21 = parse_inc_base("TXT_NAME21_BASE")

    # id -> 名段 N 映射（与 data.rs EvTable base8+base9 双段九段连寻同口径）
    def name_n(eid):
        if eid < seg1:
            return b1 + eid
        if eid < seg2:
            return b2 + (eid - seg1)
        if eid < seg3:
            return b3 + (eid - seg2)
        if eid < seg4:
            return b4 + (eid - seg3)
        if eid < seg5:
            return b5 + (eid - seg4)
        if eid < seg6:
            return b6 + (eid - seg5)
        if eid < seg7:
            return b7 + (eid - seg6)
        if eid < seg8:
            # v1.16.33 复审必修①：扩段九时段八分支整行被段九 return 顶替——
            # id 338..343 被映射到 charpanel 延后批串 N2570..2575，钉B「行存在/
            # 行号/非空」三查抓不住错行非空串，段八名段覆盖静默失效；恢复本
            # 分支，与 data.rs EvTable base8 段八+base9 段九双段连寻同口径
            return b8 + (eid - seg7)
        # v1.16.33 段九：EV_SEG8=344 快照段界，段九基址 TXT_NAME9_BASE
        if eid < seg9:
            return b9 + (eid - seg8)
        # v1.16.34 段十：EV_SEG9=349 快照段界，段十基址 TXT_NAME10_BASE
        #（新增分支勿替换段九 return——v1.16.33 段八分支被整行顶替先例红线）
        if eid < seg10:
            return b10 + (eid - seg9)
        # v1.16.35 段十一：EV_SEG10=351 快照段界，段十一基址 TXT_NAME11_BASE
        #（新增分支勿替换段十 return——同上红线；段十分支已由无卫语句末段
        # return 改带 if eid<seg10 卫语句，覆盖面原样保留）
        if eid < seg11:
            return b11 + (eid - seg10)
        # v1.16.36 段十二：EV_SEG11=353 快照段界，段十二基址 TXT_NAME12_BASE
        #（v1.16.37 单2：段十二分支由无卫语句末段 return 改带 if eid<seg12
        # 卫语句——段十三开设后段十二不再末段；新增分支勿替换本行——同上
        # 红线，v1.16.33 段八分支被整行顶替先例）
        if eid < seg12:
            return b12 + (eid - seg11)
        # v1.16.37 段十三：EV_SEG12=356 快照段界，段十三基址 TXT_NAME13_BASE
        #（v1.16.38 单2：段十三分支由无卫语句末段 return 改带 if eid<seg13
        # 卫语句——段十四开设后段十三不再末段；新增分支勿替换本行——同上
        # 红线，v1.16.33 段八分支被整行顶替先例）
        if eid < seg13:
            return b13 + (eid - seg12)
        # v1.16.38 段十四：EV_SEG13=359 快照段界，段十四基址 TXT_NAME14_BASE
        #（v1.16.40 单3：段十四分支由无卫语句末段 return 改带 if eid<seg14
        # 卫语句——段十五开设后段十四不再末段；新增分支勿替换本行——
        # 同上红线，v1.16.33 段八分支被整行顶替先例）
        if eid < seg14:
            return b14 + (eid - seg13)
        # v1.16.40 段十五：EV_SEG14=362 快照段界，段十五基址 TXT_NAME15_BASE
        #（新增分支勿替换段十四 return——同上红线）
        # v1.16.43 段十六：段十五分支改卫语句（新增分支勿替换段十五 return——
        # v1.16.33 段八分支被整行顶替红线，:146-201 沿革注同款）
        if eid < seg15:
            return b15 + (eid - seg14)
        # v1.16.43 段十六：EV_SEG15=365 快照段界，段十六基址 TXT_NAME16_BASE
        #（v1.16.44：段十六分支由无卫语句末段 return 改带 if eid<seg16 卫语句
        # ——段十七开设后段十六不再末段；新增分支勿替换本行——同上红线，
        # v1.16.33 段八分支被整行顶替先例）
        if eid < seg16:
            return b16 + (eid - seg15)
        # v1.16.44 段十七：EV_SEG16=368 快照段界，段十七基址 TXT_NAME17_BASE
        #（v1.16.45：段十七分支由无卫语句末段 return 改带 if eid<seg17 卫语句
        # ——段十八开设后段十七不再末段；新增分支勿替换本行——v1.16.33 段八
        # 分支被整行顶替先例红线）
        if eid < seg17:
            return b17 + (eid - seg16)
        # v1.16.45 段十八：EV_SEG17=371 快照段界，段十八基址 TXT_NAME18_BASE
        #（v1.16.46 段十九：段十八分支由无卫语句末段 return 改带 if eid<seg18
        # 卫语句——段十九开设后段十八不再末段；新增分支勿替换本行——v1.16.33
        # 段八分支被整行顶替先例红线）
        if eid < seg18:
            return b18 + (eid - seg17)
        # v1.16.46 段十九：EV_SEG18=374 快照段界，段十九基址 TXT_NAME19_BASE
        #（v1.16.47 段二十：段十九分支由无卫语句末段 return 改带 if eid<seg19
        # 卫语句——段二十开设后段十九不再末段；新增分支勿替换本行——v1.16.33
        # 段八分支被整行顶替先例红线）
        if eid < seg19:
            return b19 + (eid - seg18)
        # v1.16.48 段二十一：EV_SEG20=380 快照段界，段二十基址 TXT_NAME20_BASE
        #（v1.16.48 段二十一：段二十分支由无卫语句末段 return 改带 if eid<seg20
        # 卫语句——段二十一开设后段二十不再末段；新增分支勿替换本行——v1.16.33
        # 段八分支被整行顶替先例红线）
        if eid < seg20:
            return b20 + (eid - seg19)
        # v1.16.48 段二十一：EV_SEG21=383 快照段界，段二十一基址 TXT_NAME21_BASE
        return b21 + (eid - seg20)

    # 钉B：名段覆盖断言（真相源逐条可解析、非空、行号=N）
    src_lines = SRC.read_text(encoding="utf-8").splitlines()
    names = {}
    bad = []
    for eid in range(n_events):
        n = name_n(eid)
        if n >= len(src_lines):
            bad.append("id%d: N%d 越真相源界" % (eid, n))
            continue
        m = re.match(r"^(\d+)\t(.*)$", src_lines[n])
        if not m:
            bad.append("id%d: N%d 行格式异常" % (eid, n))
            continue
        if int(m.group(1)) != n:
            bad.append("id%d: N%d 行号错位（行首 %s）" % (eid, n, m.group(1)))
            continue
        if not m.group(2):
            bad.append("id%d: N%d 空串" % (eid, n))
            continue
        names[eid] = m.group(2)
    if bad:
        raise SystemExit("FATAL 钉B 名段覆盖断言失败:\n  " + "\n  ".join(bad[:8]))

    # 钉B-2 内容级自证（v1.16.33 复审必修①随批加固）：段映射漂移在钉B 三查
    # 下不可见（错行非空串与真名段同样过查）——段八/段九首尾逐字钉与
    # data.rs ev_tables_section_bounds get() 内容钉同源，漂移当场红
    for _eid, _want in ((338, "暮坛对酌"), (343, "老炉现纹"),
                        (344, "同裱旧稿"), (348, "霜橘分甘"),
                        (349, "结绳记事"), (350, "云隙借光"),
                        (351, "共拓崖铭"), (352, "苔阶履痕"),
                        (353, "岭上补松"), (354, "器铺走水"),
                        (355, "磨镜还光"),
                        (356, "哑缶一声"), (357, "同扎一篱"),
                        (358, "灶膛煨薯"),
                        (359, "移竹当窗"), (360, "灵仓亏空"),
                        (361, "熏笼共暖"),
                        (362, "芦絮暖枕"), (363, "同修暖榻"),
                        (364, "山摇地动"),
                        (365, "院墙夜塌"), (366, "共扫叶径"),
                        (367, "檐冰垂光"),
                        (368, "暖阁晒背"), (369, "漏刻无声"),
                        (370, "同制新历"),
                        (371, "牒案翻旧"), (372, "堂前领差"),
                        (373, "残页对笔"),
                        (374, "三纸同檐"), (375, "同页生隙"),
                        (376, "空阶一蝶"),
                        # v1.16.47 段二十扩池：三新元组追加（旧行零替换照款）
                        (377, "晒书使巡"), (378, "巷尾淘井"),
                        (379, "夜渡加价"),
                        # v1.16.48 段二十一扩池：三新元组追加（旧行零替换照款）
                        (380, "补行代笔"), (381, "页脚账浅"),
                        (382, "退稿索酬")):
        if names.get(_eid) != _want:
            raise SystemExit("FATAL 钉B-2 名段内容钉失败: id%d → %r (期望 %r)"
                             % (_eid, names.get(_eid), _want))
    # v1.16.39 遗留清账：段十二/十三/十四补中钉（354/357/360）——首尾钉升全钉，
    # 段八起 24 卡逐卡内容钉全覆盖（v1.16.38 low「360 居中未单钉」清账；
    # 354/357 同批补齐，三段连钉与段九/十/十一全卡钉口径对齐）
    print("OK 钉B-2 段八..段二十一内容钉全钉 ✓（338/343/344/348/349/350/351/352/353/354/355/356/357/358/359/360/361/362/363/364/365/366/367/368/369/370/371/372/373/374/375/376/377/378/379/380/381/382）")

    game_lines = GAME.read_text(encoding="utf-8").splitlines()
    st, en = build_event_span(game_lines)
    body = game_lines[st:en]

    # 钉A 扫描：臂内 card.title 赋值行（带臂归属）
    hits = []  # (行号1基, 臂id 或 None, 行文本)
    cur = None
    for k, ln in enumerate(body):
        mstart = ARM_START.match(ln)
        if mstart:
            cur = int(mstart.group(2))
            continue
        if ARM_END.match(ln):
            cur = None
            continue
        if ln.lstrip().startswith("//"):
            continue  # 注释行不算赋值（含 card.title==EVENT_NAMES 字样的钉注不误报）
        if ARM_TITLE_ANY.search(ln):
            hits.append((st + k + 1, cur, ln.rstrip()))

    ctor_ok = any(CTOR_ANCHOR.search(ln) for ln in body)

    if not hits:
        # 手术后形态：钉A（臂内零赋值）+ 钉C（构造单源）
        if not ctor_ok:
            raise SystemExit(
                "FATAL 钉C: 臂内已无 card.title 赋值，但构造行查不到 "
                "`title: EVENT_NAMES[id].into()`——标题单源链断了"
            )
        print(
            "OK 钉A 臂内 card.title 赋值=0（build_event 行 %d..%d，%d 行）；"
            "钉C 构造行查表单源 ✓" % (st + 1, en, en - st)
        )
    else:
        if not want_reconcile:
            raise SystemExit(
                "FATAL 钉A: build_event 臂内存在 %d 处 card.title 赋值"
                "（手术后应为 0；若为回滚态请加 --reconcile 走全等对账）:\n  " % len(hits)
                + "\n  ".join("%s: %s" % (p, t) for p, _e, t in hits[:6])
            )
        # 兼容模式：逐臂全等对账（手术前/回滚验证）
        noarm = [(p, t) for p, e, t in hits if e is None]
        if noarm:
            raise SystemExit(
                "FATAL reconcile: 有赋值行归属不到臂（臂结构解析失配，先修解析）:\n  "
                + "\n  ".join("%s: %s" % (p, t) for p, t in noarm[:6])
            )
        # 覆盖断言（每臂至少出现一次赋值行才算臂存在）
        seen = set(e for _p, e, _t in hits)
        missing = [e for e in range(n_events) if e not in seen]
        if missing:
            raise SystemExit("FATAL reconcile: 臂缺失 id %s" % missing[:12])
        multi = {}
        for p, e, _t in hits:
            multi.setdefault(e, []).append(p)
        over = {e: v for e, v in multi.items() if len(v) != 1}
        if over:
            raise SystemExit("FATAL reconcile: 臂内 title 赋值数≠1: %s" % over)
        # 全等比对
        mismatch = []
        for p, e, t in hits:
            mp = ARM_TITLE_PURE.match(t)
            if not mp:
                mismatch.append('id%d: 行 %s 非纯形 `card.title = "..".into();`' % (e, p))
                continue
            if mp.group(1) != names.get(e):
                mismatch.append(
                    'id%d: card「%s」≠ 名段「%s」' % (e, mp.group(1), names.get(e))
                )
        if mismatch:
            raise SystemExit("FATAL reconcile 双源漂移:\n  " + "\n  ".join(mismatch[:12]))
        print(
            "OK reconcile %d 臂 title 字面量 == 名段 全等 0 不符；id 0..=%d 全覆盖、"
            "每臂恰 1 次赋值" % (len(hits), n_events - 1)
        )
    # v1.16.46 段十九随批补齐滞后：原 OK 行段界仅列到 seg16/基址仅列到 b17
    #（v1.16.45 批未随批 +1 的存量滞后），本批一次补到全现势 seg1..seg18/b1..b19
    # v1.16.47 段二十：随批 +1 到 seg1..seg19/b1..b20（照 v1.16.46 补齐口径）
    # v1.16.48 段二十一：随批 +1 到 seg1..seg20/b1..b21（照 v1.16.47 口径）
    print(
        "OK 钉B 名段覆盖 ✓（N_EVENTS=%d 段界 %d/%d/%d/%d/%d/%d/%d/%d/%d/%d/%d/%d/%d/%d/%d/%d/%d/%d/%d/%d 基址 %d/%d/%d/%d/%d/%d/%d/%d/%d/%d/%d/%d/%d/%d/%d/%d/%d/%d/%d/%d/%d）"
        % (n_events, seg1, seg2, seg3, seg4, seg5, seg6, seg7, seg8, seg9, seg10,
           seg11, seg12, seg13, seg14, seg15, seg16, seg17, seg18, seg19, seg20,
           b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16, b17, b18, b19, b20, b21)
    )
    print("title_recheck 全绿")


if __name__ == "__main__":
    main()
