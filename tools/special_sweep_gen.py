# -*- coding: utf-8 -*-
"""单1 刀1 等价性验收（只读）：git HEAD 的 eff_desc special match 66 臂原文
对照 当前 SPECIAL_SUB N 表 + text_source 真相源行 —— 逐 slot 全等校验，
并产出 game.rs 全表钉测代码（special_subtitle_fix_tests 扩面用）。
纪律：循环变量带后缀；本脚本只读仓库、只打印，不写任何文件。"""
import io
import re
import subprocess
import sys

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')

ESC = {'n': '\n', 't': '\t', 'r': '\r', '"': '"', '\\': '\\', "'": "'", '0': '\0'}


def decode_raw(raw):
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
        d = raw[k]
        if d == 'u':
            close = raw.index('}', k + 2)
            out.append(chr(int(raw[k + 2:close], 16)))
            k = close + 1
        elif d in ESC:
            out.append(ESC[d])
            k += 1
        else:
            return None
    return ''.join(out)


head = subprocess.run(['git', 'show', 'HEAD:src/game.rs'], capture_output=True,
                      encoding='utf-8').stdout
m0 = re.search(r'let s = match self\.special \{', head)
b0 = head.index('{', m0.start())
dep = 0
k = b0
close = None
while k < len(head):
    c = head[k]
    if c == '"':
        k += 1
        while True:
            if head[k] == '\\':
                k += 2
                continue
            if head[k] == '"':
                break
            k += 1
        k += 1
        continue
    if c == '{':
        dep += 1
    elif c == '}':
        dep -= 1
        if dep == 0:
            close = k
            break
    k += 1
span = head[m0.start():head.index(';', close) + 1]
arms = {}
for am in re.finditer(r'(\d+)\s*=>\s*"((?:[^"\\]|\\.)*)"', span):
    arms[int(am.group(1))] = am.group(2)
fb = re.search(r'_\s*=>\s*"((?:[^"\\]|\\.)*)"', span).group(1)
print('HEAD arms:', len(arms), 'fallback:', decode_raw(fb))

cur = open('src/game.rs', encoding='utf-8').read()
mtab = re.search(r'const SPECIAL_SUB: \[u16; (\d+)\] = \[(.*?)\];', cur, re.S)
size = int(mtab.group(1))
slots = [int(x) for x in re.findall(r'\d+', mtab.group(2))]
assert len(slots) == size, 'SPECIAL_SUB 解析 %d != %d' % (len(slots), size)

rows = {}
for line in open('assets/text_source.txt', encoding='utf-8'):
    if not line.strip():
        continue
    n_s, _, raw = line.rstrip('\n').partition('\t')
    rows[int(n_s)] = raw

fb_n = None
ok = 0
bad = []
for slot in range(size):
    n = slots[slot]
    if slot in arms:
        want = decode_raw(arms[slot])
    else:
        want = decode_raw(fb)
        if fb_n is None:
            fb_n = n
        assert n == fb_n, 'slot %d 兜底 N=%d != 兜底臂首见 N=%d' % (slot, n, fb_n)
    got = decode_raw(rows[n])
    if got == want:
        ok += 1
    else:
        bad.append((slot, n, want, got))
print('slot 全等校验: %d/%d OK' % (ok, size))
for b in bad:
    print('  MISMATCH slot=%d N=%d want=%r got=%r' % b)
assert not bad, '等价性校验失败'

# 产出钉测代码（66 臂 + 兜底抽查 + 越界抽查）
lines = ['    // v1.16.33 单1 副标表通道全表钉：special→eff_desc 副标逐 slot 与']
lines.append('    // 原臂字面量（git HEAD 原文，special_sweep_gen.py 生成）逐字节等价——')
lines.append('    // N 错行/兜底漂移/越界钳制错位在此必红。')
lines.append('    #[test]')
lines.append('    fn special_sub_table_full_sweep() {')
lines.append('        for (sp, want) in [')
for slot in sorted(arms):
    lines.append('            (%du8, "%s"),' % (slot, arms[slot]))
lines.append('        ] {')
lines.append('            let mut e = Eff::new();')
lines.append('            e.special = sp;')
lines.append('            assert_eq!(e.eff_desc(), want, "special {} 副标漂移", sp);')
lines.append('        }')
lines.append('        // 稀疏窗位/槽 0/未登记值（>76）恒兜底「机缘」（= 原 _ 臂）')
lines.append('        let mut e = Eff::new();')
lines.append('        for sp in [0u8, 19, 51, 52, 55, 58, 59, 65, 71, 72, 76, 77, 255] {')
lines.append('            e.special = sp;')
lines.append('            assert_eq!(e.eff_desc(), "机缘", "special {} 兜底漂移", sp);')
lines.append('        }')
lines.append('    }')
print('\n'.join(lines))
