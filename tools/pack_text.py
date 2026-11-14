#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
pack_text.py —— v1.16.23 减包刀②（文案资产化）：把 game.rs build_event 块内的
长中文常量串（>=MIN_B）收编进 assets/text_source.txt（真相源，每行 N<TAB>raw，
raw=源码转义形态单行），源码侧原地替换为 evtx(N)/* 原文：<raw> */（查池函数
data::evtx，避让 ui.rs::txt 绘制函数重名），并产出：
  - assets/textpack.bin    raw-deflate(wbits=-15, level 9) 的串接 blob
  - src/text_offsets.inc   pub static TXT_OFFS: [u32; K] = [...];（data.rs include!）
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

dry = '--dry-run' in sys.argv
collect_tables = '--tables' in sys.argv  # 刀②/③通道开关（单刀量测用；表收编后自动幂等）

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
       % (n, ', '.join('&(%s)' % a for a in args), r))
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
        for key in ('TXT_NAME_BASE', 'TXT_BRIEF_BASE', 'TXT_NAME2_BASE',
                    'TXT_BRIEF2_BASE'):
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

# ---- 源码替换（从后往前，位置不失效）----
new_block = block
for s0, e0, repl in sorted(kept, key=lambda t: -t[0]):
    new_block = new_block[:s0] + repl + new_block[e0:]

new_src = src[:i] + new_block + src[end:]

# ---- 源码引用校验：块内 evtx(N)/evf(N) 的 N 全部界内且都指向可解码行 ----
refs = set(int(x) for x in re.findall(r'evtx\((\d+)\)', new_block))
refs |= set(int(x) for x in re.findall(r'evf\((\d+),\s*&\[', new_block))
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
inc = ('pub static TXT_OFFS: [u32; %d] = [\n' % len(offs)
       + ', '.join(str(o) for o in offs)
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
OFFS.write_text(inc, encoding='utf-8')
GAME.write_text(new_src, encoding='utf-8')
print('written: %s / %s / %s; game.rs updated' % (SOURCE.name, PACK.name, OFFS.name))
if name_base is not None:
    print('TXT_NAME_BASE=%d  TXT_BRIEF_BASE=%s'
          % (name_base, brief_base if brief_base is not None else '-'))
