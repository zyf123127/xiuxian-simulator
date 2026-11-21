# -*- coding: utf-8 -*-
"""v1.16.34 单1 β+γ 刀等价性验收（只读）：git HEAD 的四簇 match 原臂字面量
（SIM_DEMON_SCENE/SETTLE_CODA/FINALE_SUB/REALM_TITLE）+ DualGate::ThisYear 单臂
对照 当前 N 表 + text_source 真相源行 —— 逐 slot 全等校验（special_sweep_gen.py
v1.16.33 同型扩面）。索引映射随簇钉死：sim_demon=target-11 槽（_ 臂落槽 3）、
settle_coda=min(14)/5 槽、finale=i 直槽（_ 落槽 3）、realm_title=0..13 直槽
（_ 落槽 14）。本脚本只读仓库、只打印，不写任何文件。
纪律：循环变量带后缀。"""
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


def head_file(path):
    return subprocess.run(['git', 'show', 'HEAD:' + path], capture_output=True,
                          encoding='utf-8').stdout


def brace_span(text, m_open):
    """m_open 指向 '{'，字符串/注释感知配平，返回 (起, 止含'})。"""
    dep = 0
    k = m_open
    while k < len(text):
        c = text[k]
        if c == '"':
            k += 1
            while True:
                if text[k] == '\\':
                    k += 2
                    continue
                if text[k] == '"':
                    break
                k += 1
            k += 1
            continue
        if c == '/' and text[k:k + 2] == '//':
            k = text.find('\n', k)
            continue
        if c == '/' and text[k:k + 2] == '/*':
            k = text.find('*/', k + 2) + 2
            continue
        if c == '{':
            dep += 1
        elif c == '}':
            dep -= 1
            if dep == 0:
                return (m_open, k + 1)
        k += 1
    raise AssertionError('brace 配平失败')


rows = {}
for line in open('assets/text_source.txt', encoding='utf-8'):
    if not line.strip():
        continue
    n_s, _, raw = line.rstrip('\n').partition('\t')
    rows[int(n_s)] = raw

fail = []


def check_table(tag, head_src, head_anchor_re, cur_src, table_re, slots_map):
    """slots_map: 槽号 → HEAD 原臂 raw（含 None 兜底臂引用的闭包外值由调用方拼）"""
    mt = re.search(table_re, cur_src, re.S)
    assert mt, '%s: N 表缺失' % tag
    slots = [int(x) for x in re.findall(r'\d+', mt.group(2))]
    assert len(slots) == int(mt.group(1)), '%s: 表长 %d != %d' % (tag, len(slots), mt.group(1))
    ma = re.search(head_anchor_re, head_src, re.S)
    assert ma, '%s: HEAD match 锚缺失' % tag
    h0, h1 = brace_span(head_src, head_src.index('{', ma.end() - 1))
    span = head_src[h0:h1]
    ok = 0
    for slot, want_raw in slots_map(span):
        got = decode_raw(rows[slots[slot]])
        want = decode_raw(want_raw)
        if got == want:
            ok += 1
        else:
            fail.append('%s slot=%d N=%d want=%r got=%r' % (tag, slot, slots[slot], want, got))
    print('%s: %d/%d slot 全等（表 [u16;%d]）' % (tag, ok, int(mt.group(1)), int(mt.group(1))))


def num_arms(span):
    """数字臂 {k: raw} + _ 臂 raw 二元组。"""
    arms = {int(am.group(1)): am.group(2)
            for am in re.finditer(r'(\d+)\s*=>\s*"((?:[^"\\]|\\.)*)"', span)}
    fb = re.search(r'_\s*=>\s*"((?:[^"\\]|\\.)*)"', span).group(1)
    return arms, fb


cur = {}
for f in ('game.rs', 'data.rs', 'draw.rs'):
    cur[f] = open('src/' + f, encoding='utf-8').read()
head_game = head_file('src/game.rs')
head_data = head_file('src/data.rs')
head_draw = head_file('src/draw.rs')

# --- 簇1 sim_demon：HEAD `let scene = match target {`，槽 k=0..2↔臂 11+k、槽3↔_ ---
def map1(span):
    arms, fb = num_arms(span)
    assert set(arms) == {11, 12, 13}, '簇1 HEAD 臂 %s' % sorted(arms)
    return [(0, arms[11]), (1, arms[12]), (2, arms[13]), (3, fb)]


check_table('SIM_DEMON_SCENE_NS', head_game, r'let scene = match target \{',
            cur['game.rs'], r'const SIM_DEMON_SCENE_NS: \[u16; (\d+)\] = \[(.*?)\];', map1)

# --- 簇2 settle_coda：HEAD `match max_realm.min(14) {`，槽0↔0..=4、槽1↔5..=9、槽2↔_ ---
def map2(span):
    rngs = {am.group(1): am.group(2)
            for am in re.finditer(r'(\d+\.\.=\d+)\s*=>\s*"((?:[^"\\]|\\.)*)"', span)}
    fb = re.search(r'_\s*=>\s*"((?:[^"\\]|\\.)*)"', span).group(1)
    assert set(rngs) == {'0..=4', '5..=9'}, '簇2 HEAD 区间臂 %s' % sorted(rngs)
    return [(0, rngs['0..=4']), (1, rngs['5..=9']), (2, fb)]


check_table('SETTLE_CODA_NS', head_game, r'match max_realm\.min\(14\) \{',
            cur['game.rs'], r'const SETTLE_CODA_NS: \[u16; (\d+)\] = \[(.*?)\];', map2)

# --- 簇3 finale：HEAD draw.rs `match i {`，槽0..2↔臂0..2、槽3↔_ ---
def map3(span):
    arms, fb = num_arms(span)
    assert set(arms) == {0, 1, 2}, '簇3 HEAD 臂 %s' % sorted(arms)
    return [(0, arms[0]), (1, arms[1]), (2, arms[2]), (3, fb)]


check_table('FINALE_SUB_NS', head_draw, r'match i \{',
            cur['draw.rs'], r'const FINALE_SUB_NS: \[u16; (\d+)\] = \[(.*?)\];', map3)

# --- 簇4 realm_title：HEAD data.rs `match max_realm {`，槽0..13↔臂0..13、槽14↔_ ---
def map4(span):
    arms, fb = num_arms(span)
    assert set(arms) == set(range(14)), '簇4 HEAD 臂 %s' % sorted(arms)
    return [(k, arms[k]) for k in range(14)] + [(14, fb)]


check_table('REALM_TITLE_NS', head_data, r'match max_realm \{',
            cur['data.rs'], r'const REALM_TITLE_NS: \[u16; (\d+)\] = \[(.*?)\];', map4)

# --- 簇5 单臂：HEAD charpanel.rs `DualGate::ThisYear => "<raw>".to_string(),` ↔
#     cur charpanel.rs evtx(N)/* 原文：… */.to_string(), —— N 行解码 == 原臂 ---
head_cp = head_file('src/charpanel.rs')
mh = re.search(r'DualGate::ThisYear => "((?:[^"\\]|\\.)*)"\.to_string\(\),', head_cp)
assert mh, '簇5: HEAD 单臂缺失'
cur_cp = open('src/charpanel.rs', encoding='utf-8').read()
mc = re.search(r'DualGate::ThisYear => evtx\((\d+)\)/\* 原文：', cur_cp)
assert mc, '簇5: charpanel.rs evtx 单臂缺失'
want = decode_raw(mh.group(1))
got_row = decode_raw(rows[int(mc.group(1))])
if got_row == want:
    print('DualGate::ThisYear 单臂: 1/1 slot 全等（N=%s，dedup 共享既有行）' % mc.group(1))
else:
    fail.append('簇5 N=%s want=%r got=%r' % (mc.group(1), want, got_row))

print('----')
if fail:
    for f in fail:
        print('MISMATCH', f)
    raise SystemExit('等价性校验失败: %d 处' % len(fail))
print('β+γ 全部通过 —— git HEAD 原臂 ↔ N 表 ↔ 真相源逐 slot 全等')
