# -*- coding: utf-8 -*-
# 施工辅助：EVENTS 表账本复算（照施工单 awk 口径）——核对 data.rs 批注数字
import re

src = open('src/data.rs', encoding='utf-8').read()
# v1.16.23 减包刀③：EVENTS 元组收窄 (usize,i32,..)→(u16,u8,..)，正则放宽兼容两代口径
m = re.search(r'pub const EVENTS: \[\((?:usize|u16), (?:i32|u8), (?:usize|u8), bool\); (\d+)\] = \[(.*?)\n\];', src, re.S)
decl_n = int(m.group(1))
body = m.group(2)
rows = re.findall(r'\((\d+), (-?\d+), (\d+), (true|false)\)', body)
assert len(rows) == decl_n, f'声明 {decl_n} != 实析 {len(rows)}'
ev = [(int(a), int(w), int(mr), good == 'true') for a, w, mr, good in rows]

total = sum(w for _, w, _, _ in ev)
w0 = sum(1 for _, w, _, _ in ev if w == 0)
REGULAR_W = 2
rare = [(i, w) for i, (_, w, _, _) in enumerate(ev) if 0 < w <= REGULAR_W]
rare_w = sum(w for _, w in rare)
bad = [i for i, (_, w, _, g) in enumerate(ev) if not g]
bad_reg = [(i, w) for i, (_, w, _, g) in enumerate(ev) if not g and w > 0]
bad_reg_w = sum(w for _, w in bad_reg)

mr_all = {k: 0 for k in range(6)}
for _, w, mr, _ in ev:
    mr_all[mr] = mr_all.get(mr, 0) + 1
mr_reg_cnt = {k: 0 for k in range(6)}
mr_reg_w = {k: 0 for k in range(6)}
for _, w, mr, _ in ev:
    if w > 0:
        mr_reg_cnt[mr] = mr_reg_cnt.get(mr, 0) + 1
        mr_reg_w[mr] = mr_reg_w.get(mr, 0) + w

# mr0 开局头部：常态池按权降序前四条基权合计占比（无伴口径——伴随白名单 183
# 计入 w>0 表，但门控期白丁无伴不入轮盘；两种口径都算）
mr0_reg = sorted([(i, w) for i, (_, w, m, _) in enumerate(ev) if w > 0 and m == 0], key=lambda t: -t[1])
top4 = sum(w for _, w in mr0_reg[:4])
mr0_nocomp = [(i, w) for i, (_, w, m, _) in enumerate(ev) if w > 0 and m == 0 and i not in set(ev_idx for ev_idx, in [])]
# 无伴口径剔除伴随白名单成员
# v1.16.18 扩池伴随 +1：203 云海拾简（与 game.rs COMPANION_EVENTS 白名单同批同步）
# v1.16.19 伴随 +3：210 雨檐听铃/211 云上放灯/212 相扶看雪（白名单 14→17，
# 与 game.rs COMPANION_EVENTS 同批同步——漏一处账本口径对不上）
# v1.16.20 扩池伴随 +2：225 同数流星（mr6 有伴层首条）/226 霜晨共伞（mr0 有伴
# 第 2 条）——白名单 17→19，与 game.rs COMPANION_EVENTS 同批同步（漏一处
# 账本口径对不上）
# v1.16.22 伴随 +2：240 药圃赌芽（mr3 第 2 条）/241 踏云题字（mr7 有伴首条）
# ——白名单 19→21，与 game.rs COMPANION_EVENTS 同批同步（漏一处账本口径对不上）
# v1.16.23 伴随 +1：257 灯市挑灯（mr4 第 3 条）——白名单 21→22，与 game.rs 同批同步
# v1.16.24 伴随 +1：262 同研一笺（mr1 层首条）——白名单 22→23，与 game.rs 同批同步
COMPANION = {53, 54, 55, 82, 83, 84, 156, 157, 158, 182, 183, 184, 185, 203,
             210, 211, 212, 225, 226, 240, 241, 257, 262}
mr0_nocomp = sorted([(i, w) for i, (_, w, m, _) in enumerate(ev) if w > 0 and m == 0 and i not in COMPANION], key=lambda t: -t[1])
top4_nc = sum(w for _, w in mr0_nocomp[:4])
tot_nc = sum(w for _, w in mr0_nocomp)

print(f'rows={len(ev)} 总权={total} w0链内段={w0}')
print(f'稀有 {len(rare)} 条 {rare_w} 权 ≈{rare_w/total*100:.2f}%')
print(f'恶缘 全行 {len(bad)} 条；常规(w>0) {len(bad_reg)} 条 {bad_reg_w} 权 ≈{bad_reg_w/total*100:.1f}%')
print('mr 全行分布:', ' '.join(f'{k}:{v}' for k, v in sorted(mr_all.items())))
print('常态池(w>0):', ' '.join(f'{k}:{mr_reg_cnt[k]}条{mr_reg_w[k]}权' for k in sorted(mr_reg_cnt)))
print(f'mr0 开局头部 top4(含伴)={top4}/{mr_reg_w[0]}≈{top4/mr_reg_w[0]*100:.1f}%；无伴口径={top4_nc}/{tot_nc}≈{top4_nc/tot_nc*100:.1f}%')
print('mr5 常态池条数:', mr_reg_cnt[5], '权:', mr_reg_w[5])
print('mr3 常态池条数:', mr_reg_cnt[3], '权:', mr_reg_w[3])
