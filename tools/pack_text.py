#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
pack_text.py —— v1.16.23 减包刀②（文案资产化）：把 game.rs build_event 块内的
长中文常量串（>=MIN_B）收编进 assets/text_source.txt（真相源，每行 N<TAB>raw，
raw=源码转义形态单行），源码侧原地替换为 evtx(N)/* 原文：<raw> */（查池函数
data::evtx，避让 ui.rs::txt 绘制函数重名），并产出：
  - assets/textpack.bin    raw-deflate(wbits=-15, level 9) 的串接 blob
  - src/text_offsets.inc   pub static TXT_ANCH: [u32; A]（每 K32=32 条一个绝对锚）
                           + TXT_DELTA: [u16; N]（组内 delta，v1.16.28 刀C 两级表）
                           + 双表六段基址十常量（data.rs include！）
运行期 data.rs::init_text() boot 一次 miniz_oxide::inflate::decompress_to_vec
（与 ui.rs 字体同款通路），txt(i) O(1) 切片零分配。

v1.16.27 减包刀①（format! 模板收编通道）：块内 format!("模板{}", a)（含多行）
整调用替换为 evf(N, &[a])/* 原文：模板 */——模板位串（含 {} 占位）进 blob，
运行期 data::evf 依序 split("{}") 拼 Display，输出与 format! 逐字节等价。
模板纪律（违者留原地不收编，脚本逐条打印跳过原因）：
  - 占位只许 "{}"：无 {:spec}、无 {{}} 转义，剥净 {} 后无残留花括号；
  - 占位数 == 顶层实参数（字符串/char/注释感知括号配平；实参原样入 &[...]）；
  - 配平失败 / 模板含 */ / 往返不等价 → 跳过。
v1.16.23 旧 is_tmpl 判定因 pre 窗口尾含开引号从未生效（历史 bug，本轮实证
tail 恒含引号字符）——本版以括号回扫判定替代：format! 模板进收编通道；其余
宏（panic!/assert!/write! …，带 !）紧邻字面量一律跳过（宏要求字面量模板，
不可收编）；方法调用前缀（push_str 等）维持旧行为照常收编。

v1.16.27 减包刀②/③（data.rs 双表收编通道）：扫描 data.rs 的
`pub const EVENT_NAMES/EVENT_BRIEF: [&str; N]`，整表连号收编进 blob（不过
MIN_B 门——4 字名也收），基址以 TXT_NAME_BASE/TXT_BRIEF_BASE 写进
text_offsets.inc（data.rs 侧 const 数组由施工方换查表器 static，本脚本只管
行与基址）。基址回读自上次产出的 .inc（N 追加不变号 → 基址稳定幂等）。

等价性纪律（本脚本是唯一真相）：
  1. 每个替换串先做解码 -> re-encode 往返，往返不等价者跳过（留在原地）。
  2. blob 字节 == 原串解码值逐条 assert（脚本内自检）。
  3. inflate(textpack.bin) == blob assert（压缩回环自检）。
  4. 文案编辑工作流：改 text_source.txt 对应行 -> 重跑本脚本 -> 重编 wasm。
     新文案直接写在 game.rs 臂里（裸字面量功能照常，只是不压缩）-> 重跑本脚本
     自动收编（append 新 N，旧 N 永不变号）。

用法：python tools/pack_text.py [--dry-run]
"""
import io
import re
import sys
import zlib
from pathlib import Path

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')

ROOT = Path(__file__).resolve().parent.parent
GAME = ROOT / 'src' / 'game.rs'
DATA = ROOT / 'src' / 'data.rs'
SOURCE = ROOT / 'assets' / 'text_source.txt'
PACK = ROOT / 'assets' / 'textpack.bin'
OFFS = ROOT / 'src' / 'text_offsets.inc'
TARGET_FN = 'build_event'
MIN_B = 24  # utf-8 字节数阈值：只收编长串（短串留给原地，缩小替换面）。
            # v1.16.27 刀④实测：24→18 下调为净增（33 站 634B raw 收编，wasm-opt 后
            # opt +289B——短串每站 ~15B evtx/evf 调用码 + offsets 开销吃掉压缩收益，
            # 短串收编不划算），已回退；阈值维持 24 勿再下调。
# 刀②/③ 双表段长（EVENTS 行数；tables_recheck.py 同口径对账）
NAME_TABLE = 'EVENT_NAMES'
BRIEF_TABLE = 'EVENT_BRIEF'

# v1.16.27 扩池起：名/简段一（291 条）满员后新批连号追加为段二，基址在此登记
#（首轮施工批手分；此后每次重跑回读 .inc 幂等 emit；append-only 旧 N 不变号）
NAME2_BASE_MANIFEST = 1402
BRIEF2_BASE_MANIFEST = 1410
# v1.16.28 扩池起：段二（8 条）满员后新批连号追加为段三（10 条：名 1442..=1451/
# 简 1452..=1461，真相源手写连号段先于正文；与 data.rs EV_SEG2=299 同批登记）
NAME3_BASE_MANIFEST = 1442
BRIEF3_BASE_MANIFEST = 1452
# v1.16.29 扩池起：段三（10 条）满员后新批连号追加为段四（8 条：名 1825..=1832/
# 简 1833..=1840，真相源手写连号段先于正文；单1/单2 已用 N1492..1824，段四基址
# 顺延——与 data.rs EV_SEG3=309 同批登记）
NAME4_BASE_MANIFEST = 1825
BRIEF4_BASE_MANIFEST = 1833
# v1.16.30 扩池起：段四（8 条）满员后新批连号追加为段五（8 条：名 1865..=1872/
# 简 1873..=1880，真相源手写连号段先于正文；单2 改写即减字后下一 N=1865——
# 与 data.rs EV_SEG4=317 同批登记）
NAME5_BASE_MANIFEST = 1865
BRIEF5_BASE_MANIFEST = 1873

dry = '--dry-run' in sys.argv
collect_tables = '--tables' in sys.argv  # 刀②/③通道开关（单刀量测用；表收编后自动幂等）
# v1.16.29 单1（wasm 源级减包）三刀开关——均 append-only 旧 N 不变号，逐刀可量测：
do_block_tmpl = '--block-tmpl' in sys.argv  # 刀A：块外（build_event 之外）format! 纯{}模板收编 evf
do_block_lit = '--block-lit' in sys.argv    # 刀C：块外 ≥40B 散件字面量 evtx 化（24-39B 档勘察实测净负，不做）
do_tables2 = '--tables2' in sys.argv        # 刀B：表通道扩容（结构体表字段级 `_n` + 整表连号基址）

# 块外扫描域：帧路径 draw.rs/charpanel.rs 延后第二批（勘察纪律——evf 分配与
# format! 等价，但每帧重画站点先缓存化或延后，守帧路径纪律）
BLOCK_SCAN_FILES = ['game.rs', 'data.rs', 'guide.rs', 'main.rs', 'menu.rs',
                    'npc.rs', 'world.rs', 'gallery.rs', 'ui.rs', 'art.rs',
                    'cam.rs', 'pixel.rs', 'save.rs', 'sounds.rs']
SRC_DIR = ROOT / 'src'

# v1.16.29 刀B 表通道扩容清单。
# ARR_TABLES：整表连号（行主序 alloc_fresh，基址 emit 进 text_offsets.inc；表本体
# 随后由施工方换查表 fn——重跑时表已不在源码，走 .inc 基址回读，绝不二次收编）。
#   (表名, 文件, 基址键, 期望条数)
ARR_TABLES = [
    ('NPC_TALES', 'data.rs', 'TXT_TALES_BASE', 12),   # [[&str;3];4] 行主序 npc*3+seg
    ('HELP_LINES', 'draw.rs', 'TXT_HELP_BASE', 17),   # [&str;17]
]
# STRUCT_TABLES：结构体表字段级——表体内 `field: "raw"` 原位换 `field_n: N`
#（alloc_fresh 行主序，同字段各行连号）；`lore: &[...]` 类对子字段换
# `lore_ns: S, lore_len: L`（段内连号）。字段声明与 impl 访问器由施工方手改
#（脚本只动表体与真相源，与刀②/③分工同款）；期望条数按行数钉死防半收编。
#   (表名, 文件, 单值字段列表, 对子字段列表, 行数)
STRUCT_TABLES = [
    ('CHAPTERS', 'data.rs', ['intro', 'open', 'reward_note', 'couplet'], ['lore'], 9),
    ('ENDINGS', 'game.rs', ['name', 'alias', 'body', 'gain'], [], 4),
    ('LOCATIONS', 'data.rs', ['desc'], [], 6),
    ('HINTS', 'guide.rs', ['text'], [], 14),
]
# HINTS text 单值连号 → 附发基址（static_measure.py 指引叶宽度口径改走真相源）
BASE_KEYS_TABLES2 = ('TXT_TALES_BASE', 'TXT_HELP_BASE', 'TXT_HINT_BASE')

src = GAME.read_text(encoding='utf-8')
data_src = DATA.read_text(encoding='utf-8')

# ---- 定位目标 fn 块（brace matching，含 fn 签名行） ----
m = re.search(r'fn ' + TARGET_FN + r'\b', src)
assert m, 'fn %s not found' % TARGET_FN
i = src.index('{', m.start())
depth = 0
end = None
for j in range(i, len(src)):
    c = src[j]
    if c == '{':
        depth += 1
    elif c == '}':
        depth -= 1
        if depth == 0:
            end = j + 1
            break
assert end, 'unbalanced braces in %s' % TARGET_FN
block = src[i:end]

# ---- 解码/编码辅助：Rust 源码转义形态 <-> 真实文本 ----
ESC = {'n': '\n', 't': '\t', 'r': '\r', '"': '"', '\\': '\\', "'": "'", '0': '\0'}

def decode_raw(raw: str):
    """Rust 字符串字面量 raw -> 真实文本；失败返回 None。"""
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
        elif d in ESC:
            out.append(ESC[d])
            k += 1
        else:
            return None  # 未知转义（\x.. 等）——保守跳过该串
    return ''.join(out)

def encode_val(val: str):
    """真实文本 -> 源码转义形态（往返验证用；unicode 转义只在非打印位出现）。"""
    out = []
    for c in val:
        if c == '"':
            out.append('\\"')
        elif c == '\\':
            out.append('\\\\')
        elif c == '\n':
            out.append('\\n')
        elif c == '\t':
            out.append('\\t')
        elif c == '\r':
            out.append('\\r')
        else:
            out.append(c)
    return ''.join(out)

# ---- 读取/初始化真相源 ----
table = {}  # N -> raw
if SOURCE.exists():
    for line in SOURCE.read_text(encoding='utf-8').splitlines():
        if not line.strip():
            continue
        n_s, _, raw = line.partition('\t')
        table[int(n_s)] = raw
next_n = (max(table) + 1) if table else 0

# ---- 分配索引（去重：同值串共享 N）----
by_raw = {}
for n, raw in table.items():
    by_raw.setdefault(raw, n)

def alloc(raw: str) -> int:
    """分配 N：同值共享既有 N（build_event 去重）；新值 append（旧 N 永不变号）。"""
    global next_n
    if raw in by_raw:
        return by_raw[raw]
    n = next_n
    next_n += 1
    by_raw[raw] = n
    table[n] = raw
    return n

def alloc_fresh(raw: str) -> int:
    """整表收编专用：无条件新 N（连号段保证 base+i 寻址；与既有串重复也各占一行）。"""
    global next_n
    n = next_n
    next_n += 1
    by_raw[raw] = n
    table[n] = raw
    return n

# ---- 宏前缀识别（v1.16.27 修正旧 is_tmpl 引号 bug 的替代判定）----
def macro_before(blk: str, pos: int):
    """pos 指向模板/字面量 raw 首 char。回扫（跳空白）找紧邻 '('，再取其前标识符。
    返回 (名字, 名字起点, '(' 位置)：宏名带 '!'（如 'format!'），方法名不带
    （如 'push_str'），无前缀返回 (None, -1, -1)。format!(\\n    "…"（多行）同样识别。"""
    k = pos - 2  # pos-1 是开引号，跳过
    while k >= 0 and blk[k] in ' \t\r\n':
        k -= 1
    if k < 0 or blk[k] != '(':
        return None, -1, -1
    p_open = k
    k -= 1
    is_macro = k >= 0 and blk[k] == '!'
    if is_macro:
        k -= 1
    while k >= 0 and blk[k] in ' \t\r\n':
        k -= 1
    j2 = k
    while j2 >= 0 and (blk[j2].isalnum() or blk[j2] == '_'):
        j2 -= 1
    name = blk[j2 + 1:k + 1]
    if is_macro:
        return name + '!', j2 + 1, p_open
    return (name or None), j2 + 1, p_open

CHAR_LIT = re.compile(r"'(?:\\u\{[^}]*\}|\\.|[^'\\\n])'")

def skip_str(blk: str, k: int, ch: str):
    """k 指向引号/定界起始字符，返回越界后的下一扫描位置（转义感知）。"""
    k += 1
    n = len(blk)
    while k < n:
        if blk[k] == '\\':
            k += 2
            continue
        if blk[k] == ch:
            return k + 1
        k += 1
    return n

def scan_call_end(blk: str, p_open: int):
    """字符串/char/注释感知括号配平：p_open='(' 位置，返回其配平 ')' 位置，失败 -1。"""
    d = 0
    k = p_open
    n = len(blk)
    while k < n:
        c = blk[k]
        if c == '"':
            k = skip_str(blk, k, '"')
            continue
        if c == 'r' and blk[k:k + 2] == 'r"':
            # raw string：罕见，按普通扫描兜底会漏 #——保守失败（模板留原地）
            return -1
        if c == "'":
            mm = CHAR_LIT.match(blk, k)
            if mm:  # char 字面量；否则按生命周期引号跳过单字符
                k = mm.end()
                continue
        if c == '/' and blk[k:k + 2] == '//':
            k = blk.find('\n', k)
            if k == -1:
                return -1
            continue
        if c == '/' and blk[k:k + 2] == '/*':
            e = blk.find('*/', k + 2)
            if e == -1:
                return -1
            k = e + 2
            continue
        if c == '(':
            d += 1
        elif c == ')':
            d -= 1
            if d == 0:
                return k
        k += 1
    return -1

def split_args(blk: str, p_open: int, close: int):
    """format!( 括号内第一参数（模板字面量）之后的顶层实参逐个切出
    （字符串/char/注释感知；每个实参 verbatim 去首尾空白）。
    返回 (实参文本列表, 实参数)；异常 (None, -1)。"""
    inner = blk[p_open + 1:close]
    k = 0
    n = len(inner)
    while k < n and inner[k] in ' \t\r\n':
        k += 1
    if k >= n or inner[k] != '"':
        return None, -1
    k = skip_str(inner, k, '"')  # 越过模板字面量
    rest = inner[k:]
    stripped = rest.lstrip(' \t\r\n')
    if not stripped:
        return [], 0
    if not stripped.startswith(','):
        return None, -1  # 模板后无逗号且非空——异常
    body = stripped[1:]          # 剥首个逗号
    body_t = body.rstrip()
    while body_t.endswith(','):  # 剥尾逗号（&[...] 尾逗号合法但会虚增计数）
        body_t = body_t[:-1].rstrip()
    if not body_t:
        return [], 0
    parts = []
    d = 0
    last = 0
    k2 = 0
    n2 = len(body_t)
    while k2 < n2:
        c = body_t[k2]
        if c == '"':
            k2 = skip_str(body_t, k2, '"')
            continue
        if c == "'":
            mm = CHAR_LIT.match(body_t, k2)
            if mm:
                k2 = mm.end()
                continue
        if c == '/' and body_t[k2:k2 + 2] == '//':
            k2 = body_t.find('\n', k2)
            if k2 == -1:
                break
            continue
        if c == '/' and body_t[k2:k2 + 2] == '/*':
            e = body_t.find('*/', k2 + 2)
            if e == -1:
                return None, -1
            k2 = e + 2
            continue
        if c in '([{':
            d += 1
        elif c in ')]}':
            d -= 1
        elif c == ',' and d == 0:
            parts.append(body_t[last:k2].strip())
            last = k2 + 1
        k2 += 1
    parts.append(body_t[last:].strip())
    return parts, len(parts)

# ---- 收集块内合格串（evtx 通道 + evf 模板通道）----
LIT = re.compile(r'"((?:[^"\\]|\\.)*)"')
lit_items = []    # (pos_raw首, raw, val_bytes, N)          —— evtx 通道
tmpl_items = []   # (m_start, close, raw, val_bytes, N)     —— evf 通道
tmpl_skips = []   # (pos, raw[:40], 原因)

for m2 in LIT.finditer(block):
    raw = m2.group(1)
    if not any(ord(c) > 127 for c in raw):
        continue
    b = raw.encode('utf-8')
    if len(b) < MIN_B:
        continue
    line_start = block.rfind('\n', 0, m2.start(1))
    before = block[line_start:m2.start(1)]
    if '//' in before:          # 行内注释尾——不是正文
        continue
    if re.search(r'=>\s*$', before):  # match 模式位——不可替换
        continue
    name, m_start, p_open = macro_before(block, m2.start(1))
    if name == 'format!':
        # ---- 刀①模板通道 ----
        ph = raw.count('{}')
        residue = raw.replace('{}', '')
        if ph < 1 or '{' in residue or '}' in residue:
            tmpl_skips.append((m2.start(1), raw[:40], '占位非纯{}'))
            continue
        val = decode_raw(raw)
        if val is None or encode_val(val) != raw:
            tmpl_skips.append((m2.start(1), raw[:40], '往返不等价'))
            continue
        if '*/' in raw or '/*' in raw:
            tmpl_skips.append((m2.start(1), raw[:40], '含块注释界'))
            continue
        close = scan_call_end(block, p_open)
        if close == -1:
            tmpl_skips.append((m2.start(1), raw[:40], '括号配平失败'))
            continue
        args, nargs = split_args(block, p_open, close)
        if nargs == -1:
            tmpl_skips.append((m2.start(1), raw[:40], '实参解析失败'))
            continue
        if nargs != ph:
            tmpl_skips.append((m2.start(1), raw[:40], '占位%d≠实参%d' % (ph, nargs)))
            continue
        n = alloc(raw)
        tmpl_items.append((m_start, close, raw, val.encode('utf-8'), n, args))
        continue
    if name is not None and name.endswith('!'):
        tmpl_skips.append((m2.start(1), raw[:40], '宏 %s 后字面量不收编' % name))
        continue
    if '{' in raw or '}' in raw:  # 潜在动态模板——保守跳过
        continue
    val = decode_raw(raw)
    if val is None or encode_val(val) != raw:  # 往返不等价——跳过（等价性纪律 1）
        continue
    if '*/' in raw or '/*' in raw:  # 回填注释会破——跳过
        continue
    lit_items.append((m2.start(1), raw, val.encode('utf-8'), alloc(raw)))

# ---- 重叠消解：被外层模板 span 包住的内层收编（嵌套 format! / 实参内字面量）
#       只留内层——外层 format! 留原地（内层替换后照样编译），位置算术不被破坏 ----
all_edits = (
    [(p - 1, p + len(r) + 1, 'evtx(%d)/* 原文：%s */' % (n, r)) for p, r, _vb, n in lit_items]
    + [(ms, cl + 1, 'evf(%d, &[%s])/* 原文：%s */'
       % (n, ', '.join('&(%s) as &dyn core::fmt::Display' % a for a in args), r))
       for ms, cl, r, _vb, n, args in tmpl_items]
)
kept = []
for e in all_edits:
    s, e0, _r = e
    contained = any(o[0] <= s and e0 <= o[1] and (o[0], o[1]) != (s, e0) for o in all_edits)
    if contained:
        print('drop outer edit @%d (contains another replacement; left in place)' % s)
        continue
    kept.append(e)

# ================= v1.16.29 单1：块外第二扫描域（勘察刀A/刀C） =================
# 纪律：循环变量一律带后缀（p4/k7/m4…），绝不裸用 i/j——模块级 i 是 build_event
# 定位索引、j 是其括号扫描游标（:83-96），泄漏覆盖会把尾部拼接 src[:i] 截到文件头
#（v1.16.28 施工事故，见 :496 注）。

def cfg_test_ranges(text):
    """#[cfg(test)] mod 块区间（brace matching）——测试串永不收编（blob 是 wasm
    体量，测试文案入池即白装）。"""
    out = []
    for am in re.finditer(r'#\[cfg\(test\)\]', text):
        mm = re.search(r'mod\s+\w+', text[am.end():am.end() + 120])
        if not mm:
            continue
        try:
            bi2 = text.index('{', am.end() + mm.end())
        except ValueError:
            continue
        depth = 0
        for j2 in range(bi2, len(text)):
            c = text[j2]
            if c == '{':
                depth += 1
            elif c == '}':
                depth -= 1
                if depth == 0:
                    out.append((am.start(), j2 + 1))
                    break
    return out

def comment_ranges(text):
    """行/块注释区间——块外 LIT 扫描严禁命中注释内引号文本（含「原文」回填注释）。"""
    out = []
    k4 = 0
    n4 = len(text)
    while k4 < n4:
        c = text[k4]
        if c == '"':
            k4 = skip_str(text, k4, '"')
            continue
        if c == "'":
            mm4 = CHAR_LIT.match(text, k4)
            if mm4:
                k4 = mm4.end()
                continue
        if c == '/' and text[k4:k4 + 2] == '//':
            e4 = text.find('\n', k4)
            e4 = n4 if e4 == -1 else e4
            out.append((k4, e4))
            k4 = e4
            continue
        if c == '/' and text[k4:k4 + 2] == '/*':
            e4 = text.find('*/', k4 + 2)
            e4 = n4 if e4 == -1 else e4 + 2
            out.append((k4, e4))
            k4 = e4
            continue
        k4 += 1
    return out

def const_item_ranges(text):
    """模块级 const/static 项区间（列 0 起）——初始化是 const 语境，evtx/evf
    调用不可入内（刀C 排除面；表体字面量归刀B 通道）。"""
    out = []
    # 任意缩进（impl 块内 pub const / fn 内局部 const 同为 const 语境，一并排除）
    for m4 in re.finditer(r'(?m)^[\t ]*(pub(\([^)]*\))?(\s+in\s+\([^)]*\))?\s+)?(const|static)\s', text):
        depth = 0
        k5 = m4.end()
        n5 = len(text)
        while k5 < n5:
            c = text[k5]
            if c == '"':
                k5 = skip_str(text, k5, '"')
                continue
            if c == "'":
                mm5 = CHAR_LIT.match(text, k5)
                if mm5:
                    k5 = mm5.end()
                    continue
            if c in '([{':
                depth += 1
            elif c in ')]}':
                depth -= 1
            elif c == ';' and depth == 0:
                out.append((m4.start(), k5 + 1))
                break
            k5 += 1
    return out

def in_ranges(pos, ranges):
    for a2, b2 in ranges:
        if a2 <= pos < b2:
            return True
    return False

def prev_code_pos(text, pos):
    """pos-2 起回扫（pos-1 视作开引号跳过）第一个非空白位。"""
    k6 = pos - 2
    while k6 >= 0 and text[k6] in ' \t\r\n':
        k6 -= 1
    return k6

def after_fat_arrow(text, pos):
    """回扫紧邻 `=>` 或 `=> {`——match 臂体位（与块内通道同纪律，留原地）。"""
    k7 = prev_code_pos(text, pos)
    if k7 < 1:
        return False
    if text[k7] == '{':
        k7 = prev_code_pos(text, k7 + 1)
    return k7 >= 1 and text[k7] == '>' and text[k7 - 1] == '='

def before_fat_arrow(text, pos_end):
    """前扫紧邻 `=>`——match 模式位（字符串模式是 const 语境，evtx 不可入）。"""
    k8 = pos_end
    n8 = len(text)
    while k8 < n8 and text[k8] in ' \t\r\n':
        k8 += 1
    return text[k8:k8 + 2] == '=>'

def find_table_region(text, tname):
    """`pub const T: … = [` 起括号配平（字符串感知），返回 (起, 止) 或 None。"""
    mm6 = re.search(r'pub const ' + tname + r'\b.*?=\s*\[', text, re.S)
    if not mm6:
        return None
    bi3 = mm6.end() - 1
    depth = 0
    k9 = bi3
    n9 = len(text)
    while k9 < n9:
        c = text[k9]
        if c == '"':
            k9 = skip_str(text, k9, '"')
            continue
        if c in '[{':
            depth += 1
        elif c in ']}':
            depth -= 1
            if depth == 0:
                return (bi3, k9 + 1)
        k9 += 1
    return None

def resolve_contained(edits):
    """同一文件内嵌套消解：被外层替换 span 包住的内层保留、外层留原地
    （与块内 all_edits 同款判定，块外嵌套 format! 同理）。"""
    kept2 = []
    for e7 in edits:
        s2, e2, _r = e7
        contained = any(o[0] <= s2 and e2 <= o[1] and (o[0], o[1]) != (s2, e2)
                        for o in edits)
        if contained:
            print('drop outer edit @%s:%d (contains another replacement; left in place)'
                  % (e7[3], s2) if len(e7) > 3 else
                  'drop outer edit @%d (contains another replacement; left in place)' % s2)
            continue
        kept2.append(e7)
    return kept2

def apply_edits(text, edits):
    """从后往前应用替换（位置不失效）。"""
    new_text = text
    for s3, e3, repl in sorted(edits, key=lambda t: -t[0]):
        new_text = new_text[:s3] + repl + new_text[e3:]
    return new_text

out_edits = {}    # fname -> [(s, e, repl)]（全文件坐标；game.rs 块内账稍后并入）
FTXT = {'game.rs': src}
out_tmpl_cnt = out_lit_cnt = out_tmpl_b = out_lit_b = 0
if do_block_tmpl or do_block_lit:
    for fname2 in BLOCK_SCAN_FILES:
        if fname2 not in FTXT:
            FTXT[fname2] = (SRC_DIR / fname2).read_text(encoding='utf-8')
        ftext = FTXT[fname2]
        excl = cfg_test_ranges(ftext) + comment_ranges(ftext)
        if fname2 == 'game.rs':
            excl.append((i, end))       # build_event 块内走既有通道（上方扫描）
        if do_block_lit:
            excl += const_item_ranges(ftext)
        edits_f = []
        for m7 in LIT.finditer(ftext):
            p7 = m7.start(1)
            if in_ranges(p7, excl):
                continue
            raw7 = m7.group(1)
            if not any(ord(c) > 127 for c in raw7):
                continue
            b7 = len(raw7.encode('utf-8'))
            name7, ms7, po7 = macro_before(ftext, p7)
            if name7 == 'format!':
                if not do_block_tmpl or b7 < MIN_B:
                    continue
                ph7 = raw7.count('{}')
                res7 = raw7.replace('{}', '')
                if ph7 < 1 or '{' in res7 or '}' in res7:
                    continue
                val7 = decode_raw(raw7)
                if (val7 is None or encode_val(val7) != raw7
                        or '*/' in raw7 or '/*' in raw7):
                    continue
                if after_fat_arrow(ftext, p7):
                    continue
                cl7 = scan_call_end(ftext, po7)
                if cl7 == -1:
                    continue
                args7, nargs7 = split_args(ftext, po7, cl7)
                if nargs7 == -1 or nargs7 != ph7:
                    continue
                n7 = alloc(raw7)
                # 实参逐位显式 unsize：&[&dyn Display] 元素型一统，不吃推断顺序运气
                #（v1.16.29 main.rs 自检行 i32/u32 混排实证：隐式依赖首元素定型，
                #  异型即 E0308——新收编站一律显式，codegen 等价零字节差）
                edits_f.append((ms7, cl7 + 1,
                                'evf(%d, &[%s])/* 原文：%s */'
                                % (n7, ', '.join('&(%s) as &dyn core::fmt::Display' % a
                                                 for a in args7), raw7),
                                fname2))
                out_tmpl_cnt += 1
                out_tmpl_b += b7
                continue
            if name7 is not None and name7.endswith('!'):
                continue
            if not do_block_lit or b7 < 40:
                continue
            if '{' in raw7 or '}' in raw7:
                continue
            val7 = decode_raw(raw7)
            if (val7 is None or encode_val(val7) != raw7
                    or '*/' in raw7 or '/*' in raw7):
                continue
            if after_fat_arrow(ftext, p7) or before_fat_arrow(ftext, m7.end(1)):
                continue
            n7 = alloc(raw7)
            edits_f.append((p7 - 1, m7.end(1) + 1,
                            'evtx(%d)/* 原文：%s */' % (n7, raw7), fname2))
            out_lit_cnt += 1
            out_lit_b += b7
        for ed7 in edits_f:
            out_edits.setdefault(ed7[3], []).append((ed7[0], ed7[1], ed7[2]))
if do_block_tmpl or do_block_lit:
    print('block-out scan: %d evf templates (%d B) + %d evtx lits >=40B (%d B)'
          % (out_tmpl_cnt, out_tmpl_b, out_lit_cnt, out_lit_b))

# ---- data.rs 双表收编（刀②/③）----
PAT_TABLE = r'pub const %s: \[&str; (\d+)\] = \[(.*?)\n\];'

def collect_table(dsrc, tname):
    """返回 (entries, decl)；表已收编（数组不存在）返回 None。"""
    mm = re.search(PAT_TABLE % tname, dsrc, re.S)
    if not mm:
        return None
    decl = int(mm.group(1))
    entries = LIT.findall(mm.group(2))
    assert decl == len(entries), '%s decl %d != parsed %d' % (tname, decl, len(entries))
    return entries, decl

def table_bases_from_inc():
    """回读上次 .inc 里的双表基址（N 追加不变号 → 基址幂等稳定）。"""
    out = {}
    if OFFS.exists():
        txt = OFFS.read_text(encoding='utf-8')
        for key in (('TXT_NAME_BASE', 'TXT_BRIEF_BASE', 'TXT_NAME2_BASE',
                     'TXT_BRIEF2_BASE', 'TXT_NAME3_BASE', 'TXT_BRIEF3_BASE',
                     'TXT_NAME4_BASE', 'TXT_BRIEF4_BASE', 'TXT_NAME5_BASE',
                     'TXT_BRIEF5_BASE')
                    + BASE_KEYS_TABLES2):
            mm = re.search(r'pub static %s: u32 = (\d+);' % key, txt)
            if mm:
                out[key] = int(mm.group(1))
    return out

def emit_base(key, n):
    return 'pub static %s: u32 = %d;\n' % (key, n)

name_entries = collect_table(data_src, NAME_TABLE) if collect_tables else None
brief_entries = collect_table(data_src, BRIEF_TABLE) if collect_tables else None
inc_bases = table_bases_from_inc()

name_base = None
brief_base = None
table_raw_bytes = 0
if name_entries is not None:
    ent, decl = name_entries
    name_base = next_n
    for raw in ent:
        val = decode_raw(raw)
        assert val is not None and encode_val(val) == raw, '%s 行往返失败: %r' % (NAME_TABLE, raw[:20])
        assert '*/' not in raw and '/*' not in raw
        table_raw_bytes += len(val.encode('utf-8'))
        alloc_fresh(raw)
    print('%s collected: %d rows, base N=%d, %d B'
          % (NAME_TABLE, decl, name_base, table_raw_bytes))
elif 'TXT_NAME_BASE' in inc_bases:
    name_base = inc_bases['TXT_NAME_BASE']
    assert next_n > name_base and next_n - 1 >= name_base + 290, 'name 段基址越界'
if brief_entries is not None:
    ent, decl = brief_entries
    brief_base = next_n
    bbytes = 0
    for raw in ent:
        val = decode_raw(raw)
        assert val is not None and encode_val(val) == raw, '%s 行往返失败: %r' % (BRIEF_TABLE, raw[:20])
        assert '*/' not in raw and '/*' not in raw
        bbytes += len(val.encode('utf-8'))
        alloc_fresh(raw)
    print('%s collected: %d rows, base N=%d, %d B'
          % (BRIEF_TABLE, decl, brief_base, bbytes))
elif 'TXT_BRIEF_BASE' in inc_bases:
    brief_base = inc_bases['TXT_BRIEF_BASE']
    assert next_n > brief_base and next_n - 1 >= brief_base + 290, 'brief 段基址越界'
# 幂等守卫：两段基址必须不同且连号区间不相交
if name_base is not None and brief_base is not None:
    assert name_base != brief_base, '双表基址重合'

# ================= v1.16.29 单1刀B：表通道扩容（--tables2） =================
# 结构体表字段级原位改写（field: "raw" → field_n: N）+ 整表连号基址 emit；
# 通道纪律与刀②/③同款：脚本只管行与基址，字段声明/impl 访问器/查表 fn 由施工方
# 手改；重跑幂等（表体已换 → 无命中跳过；基址走 .inc 回读，绝不二次 alloc_fresh）。
new_base_emits = {}
tables2_report = []
if do_tables2:
    for (tname2, fkey2, sfields2, pfields2, nrows2) in STRUCT_TABLES:
        if fkey2 not in FTXT:
            FTXT[fkey2] = (SRC_DIR / fkey2).read_text(encoding='utf-8')
        ftext2 = FTXT[fkey2]
        reg2 = find_table_region(ftext2, tname2)
        if reg2 is None:
            tables2_report.append('%s: 表体未找到（应已换访问器，幂等跳过）' % tname2)
            continue
        region2 = ftext2[reg2[0]:reg2[1]]
        edits2 = []
        for fld2 in sfields2:
            pat2 = re.compile(r'\b' + fld2 + r'\s*:\s*"((?:[^"\\]|\\.)*)"')
            hits2 = list(pat2.finditer(region2))
            if not hits2:
                tables2_report.append('%s.%s: 无命中（应已收编，幂等跳过）' % (tname2, fld2))
                continue
            assert len(hits2) == nrows2, ('%s.%s 命中 %d != 期望 %d（防半收编）'
                                          % (tname2, fld2, len(hits2), nrows2))
            first_n2 = None
            for h2 in hits2:
                raw2 = h2.group(1)
                val2 = decode_raw(raw2)
                assert val2 is not None and encode_val(val2) == raw2, \
                    '%s.%s 往返失败: %r' % (tname2, fld2, raw2[:20])
                assert '*/' not in raw2 and '/*' not in raw2, '%s.%s 含注释界' % (tname2, fld2)
                n2 = alloc_fresh(raw2)
                if first_n2 is None:
                    first_n2 = n2
                edits2.append((reg2[0] + h2.start(), reg2[0] + h2.end(),
                               '%s_n: %d' % (fld2, n2)))
            if tname2 == 'HINTS' and fld2 == 'text':
                new_base_emits['TXT_HINT_BASE'] = first_n2
            tables2_report.append('%s.%s: %d 行 → field_n 连号（首 N=%d）'
                                  % (tname2, fld2, nrows2, first_n2))
        for fld3 in pfields2:
            pat3 = re.compile(r'\b' + fld3 + r'\s*:\s*&\[')
            hits3 = list(pat3.finditer(region2))
            if not hits3:
                tables2_report.append('%s.%s: 无命中（应已收编，幂等跳过）' % (tname2, fld3))
                continue
            assert len(hits3) == nrows2, ('%s.%s 命中 %d != 期望 %d（防半收编）'
                                          % (tname2, fld3, len(hits3), nrows2))
            for h3 in hits3:
                d3 = 1
                k3 = h3.end()          # region 局部，指向 '[' 后一位
                nreg = len(region2)
                while d3 > 0 and k3 < nreg:
                    c3 = region2[k3]
                    if c3 == '"':
                        k3 = skip_str(region2, k3, '"')
                        continue
                    if c3 == '[':
                        d3 += 1
                    elif c3 == ']':
                        d3 -= 1
                        if d3 == 0:
                            break
                    k3 += 1
                assert d3 == 0, '%s.%s 段括号配平失败' % (tname2, fld3)
                inner3 = region2[h3.end():k3]
                lits3 = LIT.findall(inner3)
                assert 1 <= len(lits3) <= 3, '%s.%s 段内 %d 条越预算(1..=3)' \
                    % (tname2, fld3, len(lits3))
                start3 = None
                for raw3 in lits3:
                    val3 = decode_raw(raw3)
                    assert val3 is not None and encode_val(val3) == raw3, \
                        '%s.%s 往返失败: %r' % (tname2, fld3, raw3[:20])
                    assert '*/' not in raw3 and '/*' not in raw3
                    n3 = alloc_fresh(raw3)
                    if start3 is None:
                        start3 = n3
                edits2.append((reg2[0] + h3.start(), reg2[0] + k3 + 1,
                               '%s_ns: %d, %s_len: %d' % (fld3, start3, fld3, len(lits3))))
            tables2_report.append('%s.%s: %d 行 → (ns, len) 对，段内连号'
                                  % (tname2, fld3, nrows2))
        if edits2:
            out_edits.setdefault(fkey2, []).extend(edits2)
    for (tname4, fkey4, basekey4, expect4) in ARR_TABLES:
        if basekey4 in inc_bases:
            b4v = inc_bases[basekey4]
            assert next_n - 1 >= b4v + expect4 - 1, '%s 基址段越界' % basekey4
            new_base_emits[basekey4] = b4v
            tables2_report.append('%s: 基址回读 %d（表本体应已换 fn，幂等）' % (basekey4, b4v))
            continue
        if fkey4 not in FTXT:
            FTXT[fkey4] = (SRC_DIR / fkey4).read_text(encoding='utf-8')
        ftext4 = FTXT[fkey4]
        reg4 = find_table_region(ftext4, tname4)
        if reg4 is None:
            tables2_report.append('%s: 表体未找到且无基址登记——跳过（人工核对）' % tname4)
            continue
        lits4 = LIT.findall(ftext4[reg4[0]:reg4[1]])
        assert len(lits4) == expect4, '%s 解析 %d 条 != 期望 %d' % (tname4, len(lits4), expect4)
        base4 = next_n
        for raw4b in lits4:
            assert any(ord(c) > 127 for c in raw4b), \
                '%s 含 ASCII 项会破 base+i 寻址' % tname4
            val4b = decode_raw(raw4b)
            assert val4b is not None and encode_val(val4b) == raw4b, \
                '%s 往返失败: %r' % (tname4, raw4b[:20])
            assert '*/' not in raw4b and '/*' not in raw4b
            alloc_fresh(raw4b)
        new_base_emits[basekey4] = base4
        tables2_report.append('%s collected: %d rows, base N=%d（表本体由施工方换查表 fn）'
                              % (tname4, expect4, base4))
if tables2_report:
    print('---- tables2 report ----')
    for line2 in tables2_report:
        print('  ' + line2)
# 刀B 基址回读兜底：非 --tables2 运行也必须补齐（.inc 缺基址 = include! 断链）
ARR_EXPECT = {'TXT_TALES_BASE': 12, 'TXT_HELP_BASE': 17, 'TXT_HINT_BASE': 14}
for bk3, exp3 in ARR_EXPECT.items():
    if bk3 not in new_base_emits and bk3 in inc_bases:
        assert next_n - 1 >= inc_bases[bk3] + exp3 - 1, '%s 基址段越界' % bk3
        new_base_emits[bk3] = inc_bases[bk3]

# ---- blob 与 offsets ----
blob = bytearray()
offs = [0]
seen = set()
for n in sorted(table):
    if n in seen:
        continue
    seen.add(n)
    val = decode_raw(table[n])
    assert val is not None, 'table row %d undecodable' % n
    blob += val.encode('utf-8')  # raw 是源码形态，blob 存解码值
    offs.append(len(blob))

assert offs[0] == 0 and len(offs) == len(table) + 1, 'offset table mismatch'
for k in range(len(offs) - 1):
    assert offs[k] <= offs[k + 1], 'offsets not monotonic at %d' % k

# v1.16.28 减包刀C：offsets 产物 [u32;N]（5772B）改「u32 锚 + u16 组内 delta」两级
# 表——每 K32=32 条一个绝对锚 TXT_ANCH（offs[k*32]），TXT_DELTA[i]=offs[i]−组首。
# 运行期 evtx 解码 = ANCH[i>>5] + DELTA[i]，O(1) 纯算术零分配（热路径等价）；
# 组内 32 条串长和 ≤64KB 恒成立（单条 <200B×32），u16 域 assert 固化。
K32 = 32
anchors = offs[::K32]
assert len(anchors) == (len(offs) + K32 - 1) // K32, 'anchor count mismatch'
deltas = []
# 纪律：循环变量勿用 i——模块级 i 是 build_event 定位索引（:83），泄漏覆盖会把
# :521 的 src[:i] 截断到文件头（v1.16.28 施工现场事故：4059 行被压成 1 行，
# 修复后循环变量改名 di 绝不复用）
for di, o in enumerate(offs):
    a = offs[(di // K32) * K32]
    assert 0 <= o - a <= 0xFFFF, 'delta overflow at %d (%d-%d)' % (di, o, a)
    deltas.append(o - a)

# ---- 等价性纪律 2：blob 切片逐条 == 原串解码值 ----
for pos, raw, vb, n in lit_items:
    assert blob[offs[n]:offs[n + 1]] == vb, 'blob slice mismatch at N=%d' % n
for _ms, _cl, raw, vb, n, _a in tmpl_items:
    assert blob[offs[n]:offs[n + 1]] == vb, 'blob slice mismatch at N=%d (tmpl)' % n

comp = zlib.compressobj(9, zlib.DEFLATED, -15)
packed = comp.compress(bytes(blob)) + comp.flush()

# ---- 等价性纪律 3：压缩回环 ----
dec = zlib.decompressobj(-15)
assert dec.decompress(packed) + dec.flush() == bytes(blob), 'deflate roundtrip failed'

# ---- 源码替换（从后往前，位置不失效）——v1.16.29 起多文件账本 ----
# build_event 块内账（kept 为块局部坐标）换算全文件坐标后并入 game.rs；
# 块外通道（out_edits）与表通道（tables2 写进 out_edits）按文件合并消解后统一应用。
file_edits = {}
for fname8, edits8 in out_edits.items():
    file_edits[fname8] = list(edits8)
blk_shift = [(s0 + i, e0 + i, repl) for s0, e0, repl in kept]
if blk_shift:
    file_edits.setdefault('game.rs', []).extend(blk_shift)

final_texts = {}
for fname8, edits8 in file_edits.items():
    if not edits8:
        continue
    ftext8 = FTXT.get(fname8)
    if ftext8 is None:
        ftext8 = (SRC_DIR / fname8).read_text(encoding='utf-8')
        FTXT[fname8] = ftext8
    kept8 = resolve_contained(edits8)
    if not kept8:
        continue
    final_texts[fname8] = apply_edits(ftext8, kept8)

# ---- 源码引用校验：全部被改文件里 evtx(N)/evf(N) 的 N 界内且都指向可解码行 ----
refs = set()
for ftext9 in final_texts.values():
    refs |= set(int(x) for x in re.findall(r'evtx\((\d+)\)', ftext9))
    refs |= set(int(x) for x in re.findall(r'evf\((\d+),\s*&\[', ftext9))
assert all(0 <= n < len(offs) - 1 for n in refs), 'evtx/evf(N) out of range'
for n in refs:
    assert decode_raw(table[n]) is not None, 'ref N=%d bad row' % n

replaced_bytes = sum(len(vb) for _, _, vb, _ in lit_items)
replaced_bytes += sum(len(vb) for _, _, _, vb, _, _a in tmpl_items)
tmpl_bytes = sum(len(vb) for _, _, _, vb, _, _a in tmpl_items)
print('block scan: %d evtx lits + %d format! templates collected'
      % (len(lit_items), len(tmpl_items)))
if tmpl_skips:
    print('template skips: %d' % len(tmpl_skips))
    for p, r, why in tmpl_skips[:12]:
        print('   skip @%d %s: %s' % (p, r, why))
print('replaced: %d sites, %d B moved into blob' % (len(kept), replaced_bytes))
print('blob=%d B  raw-deflate=%d B (%.1f%%)  offsets=%d x u32 = %d B'
      % (len(blob), len(packed), len(packed) * 100.0 / max(1, len(blob)),
         len(offs), len(offs) * 4))

if dry:
    print('[dry-run] no files written')
    sys.exit(0)

# ---- 落盘 ----
SOURCE.write_text(''.join('%d\t%s\n' % (n, table[n]) for n in sorted(table)),
                  encoding='utf-8')
PACK.write_bytes(packed)
# v1.16.28 刀C：TXT_OFFS [u32;N] → TXT_ANCH [u32;A]+TXT_DELTA [u16;N] 两级表
#（data.rs evtx 解码 = ANCH[i>>5]+DELTA[i]，语义与旧 offs 逐值等价）
inc = ('pub static TXT_ANCH: [u32; %d] = [\n' % len(anchors)
       + ', '.join(str(a) for a in anchors)
       + '\n];\n'
       + 'pub static TXT_DELTA: [u16; %d] = [\n' % len(deltas)
       + ', '.join(str(d) for d in deltas)
       + '\n];\n')
if name_base is not None:
    inc += emit_base('TXT_NAME_BASE', name_base)
if brief_base is not None:
    inc += emit_base('TXT_BRIEF_BASE', brief_base)
if name_base is not None:
    inc += emit_base('TXT_NAME2_BASE',
                     inc_bases.get('TXT_NAME2_BASE', NAME2_BASE_MANIFEST))
if brief_base is not None:
    inc += emit_base('TXT_BRIEF2_BASE',
                     inc_bases.get('TXT_BRIEF2_BASE', BRIEF2_BASE_MANIFEST))
# v1.16.28 段三基址（首轮 emit 取 manifest 手分值；此后回读 .inc 幂等）
if name_base is not None:
    inc += emit_base('TXT_NAME3_BASE',
                     inc_bases.get('TXT_NAME3_BASE', NAME3_BASE_MANIFEST))
if brief_base is not None:
    inc += emit_base('TXT_BRIEF3_BASE',
                     inc_bases.get('TXT_BRIEF3_BASE', BRIEF3_BASE_MANIFEST))
# v1.16.29 段四基址（首轮 emit 取 manifest 手分值；此后回读 .inc 幂等）
if name_base is not None:
    inc += emit_base('TXT_NAME4_BASE',
                     inc_bases.get('TXT_NAME4_BASE', NAME4_BASE_MANIFEST))
if brief_base is not None:
    inc += emit_base('TXT_BRIEF4_BASE',
                     inc_bases.get('TXT_BRIEF4_BASE', BRIEF4_BASE_MANIFEST))
# v1.16.30 段五基址（首轮 emit 取 manifest 手分值；此后回读 .inc 幂等）
if name_base is not None:
    inc += emit_base('TXT_NAME5_BASE',
                     inc_bases.get('TXT_NAME5_BASE', NAME5_BASE_MANIFEST))
if brief_base is not None:
    inc += emit_base('TXT_BRIEF5_BASE',
                     inc_bases.get('TXT_BRIEF5_BASE', BRIEF5_BASE_MANIFEST))
# v1.16.29 刀B 整表基址（首轮 emit 收编 N 段基址；此后每次运行从 .inc 回读重发，
# 与段一/二/三同款幂等——非 --tables2 运行也不许丢基址，否则 include! 断链）
for bk2 in BASE_KEYS_TABLES2:
    if bk2 in new_base_emits:
        inc += emit_base(bk2, new_base_emits[bk2])
OFFS.write_text(inc, encoding='utf-8')
for fname10 in sorted(final_texts):
    (SRC_DIR / fname10).write_text(final_texts[fname10], encoding='utf-8')
print('written: %s / %s / %s; sources updated: %s'
      % (SOURCE.name, PACK.name, OFFS.name,
         ', '.join(sorted(final_texts)) or '(none)'))
if name_base is not None:
    print('TXT_NAME_BASE=%d  TXT_BRIEF_BASE=%s'
          % (name_base, brief_base if brief_base is not None else '-'))
