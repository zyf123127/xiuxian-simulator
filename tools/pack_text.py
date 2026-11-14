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
SOURCE = ROOT / 'assets' / 'text_source.txt'
PACK = ROOT / 'assets' / 'textpack.bin'
OFFS = ROOT / 'src' / 'text_offsets.inc'
TARGET_FN = 'build_event'
MIN_B = 24  # utf-8 字节数阈值：只收编长串（短串留给原地，缩小替换面）

dry = '--dry-run' in sys.argv

src = GAME.read_text(encoding='utf-8')

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

# ---- 收集块内合格串 ----
LIT = re.compile(r'"((?:[^"\\]|\\.)*)"')
candidates = []  # (char_pos_start_in_block, raw, val_bytes)
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
    pre = block[max(0, m2.start(1) - 48):m2.start(1)]
    is_tmpl = False
    for mac in ('format!', 'write!', 'writeln!', 'panic!', 'assert!', 'assert_eq!',
                'assert_ne!', 'unreachable!'):
        idx = pre.rfind(mac)
        if idx != -1:
            between = pre[idx + len(mac):]
            if '(' in between and between[between.index('(') + 1:].strip() == '':
                is_tmpl = True
                break
    if is_tmpl:                 # format! 模板必须字面量——跳过
        continue
    if '{' in raw or '}' in raw:  # 潜在动态模板——保守跳过
        continue
    val = decode_raw(raw)
    if val is None or encode_val(val) != raw:  # 往返不等价——跳过（等价性纪律 1）
        continue
    if '*/' in raw or '/*' in raw:  # 回填注释会破——跳过
        continue
    candidates.append((m2.start(1), raw, val.encode('utf-8')))

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

new_items = []  # (pos, raw, val_bytes, N)
for pos, raw, vb in candidates:
    if raw in by_raw:
        n = by_raw[raw]
    else:
        n = next_n
        next_n += 1
        by_raw[raw] = n
        table[n] = raw
    new_items.append((pos, raw, vb, n))

# ---- blob 与 offsets ----
blob = bytearray()
offs = [0]
seen = set()
for n in sorted(table):
    if n in seen:
        continue
    seen.add(n)
    vb = table[n].encode('utf-8')  # raw 是源码形态，blob 存解码值
    val = decode_raw(table[n])
    assert val is not None, 'table row %d undecodable' % n
    blob += val.encode('utf-8')
    offs.append(len(blob))

assert offs[0] == 0 and len(offs) == len(table) + 1, 'offset table mismatch'
for k in range(len(offs) - 1):
    assert offs[k] <= offs[k + 1], 'offsets not monotonic at %d' % k

# ---- 等价性纪律 2：blob 切片逐条 == 原串解码值 ----
for pos, raw, vb, n in new_items:
    assert blob[offs[n]:offs[n + 1]] == vb, 'blob slice mismatch at N=%d' % n

comp = zlib.compressobj(9, zlib.DEFLATED, -15)
packed = comp.compress(bytes(blob)) + comp.flush()

# ---- 等价性纪律 3：压缩回环 ----
dec = zlib.decompressobj(-15)
assert dec.decompress(packed) + dec.flush() == bytes(blob), 'deflate roundtrip failed'

# ---- 源码替换（从后往前，位置不失效）----
new_block = block
new_items_sorted = sorted(new_items, key=lambda t: -t[0])
for pos, raw, vb, n in new_items_sorted:
    repl = 'evtx(%d)/* 原文：%s */' % (n, raw)
    # pos 指向 raw 首字符；字面量含两侧引号，各退一位
    s0 = pos - 1
    e0 = pos + len(raw) + 1
    assert new_block[s0] == '"' and new_block[e0 - 1] == '"', 'quote bounds lost at N=%d' % n
    new_block = new_block[:s0] + repl + new_block[e0:]

new_src = src[:i] + new_block + src[end:]

# ---- 源码引用校验：块内 evtx(N) 的 N 全部界内且都指向同值 ----
refs = set(int(x) for x in re.findall(r'evtx\((\d+)\)', new_block))
assert all(0 <= n < len(offs) - 1 for n in refs), 'evtx(N) out of range'
for n in refs:
    assert decode_raw(table[n]) is not None, 'ref N=%d bad row' % n

replaced_bytes = sum(len(vb) for _, _, vb, _ in new_items)
uniq_bytes = len(blob)
print('block scan: %d literals eligible (dedup -> %d unique rows)'
      % (len(new_items), len(table)))
print('replaced: %d sites, %d B moved into blob' % (len(new_items), replaced_bytes))
print('blob=%d B  raw-deflate=%d B (%.1f%%)  offsets=%d x u32 = %d B'
      % (uniq_bytes, len(packed), len(packed) * 100.0 / max(1, uniq_bytes),
         len(offs), len(offs) * 4))
print('net wasm estimate: %d - %d - %d = %+d B (source literals -> pack+offsets)'
      % (replaced_bytes, len(packed), len(offs) * 4,
         replaced_bytes - len(packed) - len(offs) * 4))

if dry:
    print('[dry-run] no files written')
    sys.exit(0)

# ---- 落盘 ----
SOURCE.write_text(''.join('%d\t%s\n' % (n, table[n]) for n in sorted(table)),
                  encoding='utf-8')
PACK.write_bytes(packed)
OFFS.write_text('pub static TXT_OFFS: [u32; %d] = [\n' % len(offs)
                + ', '.join(str(o) for o in offs)
                + '\n];\n', encoding='utf-8')
GAME.write_text(new_src, encoding='utf-8')
print('written: %s / %s / %s; game.rs updated' % (SOURCE.name, PACK.name, OFFS.name))
