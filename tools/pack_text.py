#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
pack_text.py —— v1.16.23 减包刀②（文案资产化）：把 game.rs build_event 块内的
长中文常量串（>=MIN_B）收编进 assets/text_source.txt（真相源，每行 N<TAB>raw，
raw=源码转义形态单行），源码侧原地替换为 evtx(N)/* 原文：<raw> */（查池函数
data::evtx，避让 ui.rs::txt 绘制函数重名），并产出：
  - assets/textpack.bin    raw-deflate(wbits=-15, level 9) 的串接 blob
  - src/text_offsets.inc   pub static TXT_ANCH: [u32; A]（每 K32=128 条一个绝对锚；
                            v1.16.32 单1 自 32 粗化至 128——锚数省 4B×3/32 行，
                            组内 delta u16 域由产侧 assert 固化）
                           + TXT_DELTA: [u16; N]（组内 delta，v1.16.28 刀C 两级表）
                           + 双表基址常量（data.rs include!；v1.16.32 单3 末态
                           八对十六常量：TXT_NAME6/TXT_BRIEF6 后增 TXT_NAME7/
                           TXT_BRIEF7、TXT_NAME8/TXT_BRIEF8——历史注「六段基址
                           十常量」与全角 include！均为笔误，已随段清账；
                           v1.16.33..36 扩池随批续至十二对二十四常量——TXT_NAME9/
                           TXT_BRIEF9（2576/2581）..TXT_NAME12/TXT_BRIEF12
                           （2667/2670，段十二，v1.16.36 单3 登记））
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

v1.16.33 单1 NIZE 通道（--nize）：special 副标 match 表通道 + const 串表 N 化批。
  - special：eff_desc 内 `let s = match self.special {..}` 整体换 fn 内 const
    SPECIAL_SUB: [u16;77] N 表 + evtx 查表（稀疏窗位/未登记值统一落「机缘」
    兜底 N——与原 _ 臂逐值等价；臂原文注随替身上移保留）；
  - tables：9 张 const 串表/2 条单串 const 整体换 [u16;K] N 表 / N 常量——
    alloc dedup 同值共享 N（_16 双档、RECALL_CHROME∩EV_NOTE_PREWARM、
    COMP_PAGE_PREWARM 内 DUAL_CULT_HINT 由此共享），访问器/循环由施工方手改。
  两个子域（--nize / --nize=special / --nize=tables）均幂等：域已换出即跳过。

用法：python tools/pack_text.py [--dry-run] [--tables] [--block-tmpl]
             [--block-lit] [--tables2] [--nize[=special|tables]]
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
# v1.16.31 扩池起：段五（8 条）满员后新批连号追加为段六（10 条：名 1905..=1914/
# 简 1915..=1924，真相源手写连号段先于正文；v1.16.30 末态下一 N=1905——
# 与 data.rs EV_SEG5=325 同批登记）
NAME6_BASE_MANIFEST = 1905
BRIEF6_BASE_MANIFEST = 1915
# v1.16.31 新功能 3 条起：段六（10 条 N1905..1914/N1915..1924 紧连封死）满员后
# 新批连号追加为段七（3 条：名 1954..=1956/简 1957..=1959，真相源手写连号段
# 先于正文；第十批改写后下一空 N=1954——与 data.rs EV_SEG6=335 同批登记）
NAME7_BASE_MANIFEST = 1954
BRIEF7_BASE_MANIFEST = 1957
# v1.16.32 单3 扩池起：段七（3 条）满员后新批连号追加为段八（6 条：名
# 2338..=2343/简 2344..=2349，真相源手写连号段先于正文；单1/单2 已用
# N1971..2337 故段八顺延 2338/2344——与 data.rs EV_SEG7=338 同批登记）
NAME8_BASE_MANIFEST = 2338
BRIEF8_BASE_MANIFEST = 2344
# v1.16.33 单3 扩池起：段八（6 条）满员后新批连号追加为段九（5 条：名
# 2576..=2580/简 2581..=2585，真相源手写连号段先于正文；单1 N 化批已用
# N2368..2575 故段九顺延 2576/2581——与 data.rs EV_SEG8=344 同批登记）
NAME9_BASE_MANIFEST = 2576
BRIEF9_BASE_MANIFEST = 2581
# v1.16.34 单3 扩池起：段九（5 条）满员后新批连号追加为段十（2 条：名
# 2647..=2648/简 2649..=2650，真相源手写连号段先于正文；单1 精度/臂表批已
# 用 N2601..2646 故段十顺延 2647/2649——与 data.rs EV_SEG9=349 同批登记）
NAME10_BASE_MANIFEST = 2647
BRIEF10_BASE_MANIFEST = 2649
# v1.16.35 单3 扩池起：段十（2 条）满员后新批连号追加为段十一（2 条：名
# 2657..=2658/简 2659..=2660，真相源手写连号段先于正文；单1 精度/臂表批已
# 用 N2601..2646+段十名/简 2647..2650、单2 加载链批未用正文 N 故段十一顺延
# 2657/2659——与 data.rs EV_SEG10=351 快照常量同批登记）
NAME11_BASE_MANIFEST = 2657
BRIEF11_BASE_MANIFEST = 2659

# v1.16.36 单3 扩池起：段十一（2 条）满员后新批连号追加为段十二（3 条：名
# 2667..=2669/简 2670..=2672，真相源手写连号段先于正文；单1 EV_SEG11=353 快照
# 预建+单2 零改动未用正文 N 故段十二顺延 2667/2670——与 data.rs EV_SEG11=353
# 快照常量同批登记）
NAME12_BASE_MANIFEST = 2667
BRIEF12_BASE_MANIFEST = 2670

# v1.16.37 单2 扩池起：段十二（3 条）满员后新批连号追加为段十三（3 条：名
# 2684..=2686/简 2687..=2689，真相源手写连号段先于正文；v1.16.36 正文收编至
# N2683（evtx 2673..2678+evf 2679..2681 共 9 站+evf 2682/2683 漩心 loot 双裸串）
# 故段十三顺延 2684/2687；另手写检定副标行 N2690「受缶一问」（special76 检定窗
# 启用，356 哑缶一声副标，SPECIAL_SUB[76]=2690 查表）——与 data.rs EV_SEG12=356
# 快照常量差写法兑现/EV_SEG13=359 快照预建同批登记）
NAME13_BASE_MANIFEST = 2684
BRIEF13_BASE_MANIFEST = 2687

# v1.16.38 单2 扩池起：段十三（3 条）满员后新批连号追加为段十四（3 条：名
# 2701..=2703/简 2704..=2706，真相源手写连号段先于正文——名/简手写必须先于
# 正文入库，若正文先行则本 manifest 基址错位；v1.16.37 正文收编至 N2700（evtx
# 2691/2692 检定双臂+evf 2693/2694+evtx 2695 伴随+evf 2696/2697+evtx 2698 双门控
# +evtx 2699/2700 检定成/败臂共 10 站）故段十四顺延 2701/2704——与 data.rs
# EV_SEG13=359 快照差写法兑现/EV_SEG14=362 快照预建同批登记）
NAME14_BASE_MANIFEST = 2701
BRIEF14_BASE_MANIFEST = 2704

# v1.16.40 单3 扩池起：段十四（3 条）满员后新批连号追加为段十五（3 条：名
# 2746..=2748/简 2749..=2751，真相源手写连号段先于正文——名/简手写必须先于
# 正文入库，若正文先行则本 manifest 基址错位；v1.16.39 臂表收编至 N2726、
# v1.16.40 长老委托文案 N2727..2745（页签/委托名/描述/达标句/发奖模板 19 行）
# 故段十五顺延 2746/2749——与 data.rs EV_SEG14=362 快照差写法兑现/EV_SEG15=365
# 快照预建同批登记）
NAME15_BASE_MANIFEST = 2746
BRIEF15_BASE_MANIFEST = 2749

dry = '--dry-run' in sys.argv
collect_tables = '--tables' in sys.argv  # 刀②/③通道开关（单刀量测用；表收编后自动幂等）
# v1.16.29 单1（wasm 源级减包）三刀开关——均 append-only 旧 N 不变号，逐刀可量测：
do_block_tmpl = '--block-tmpl' in sys.argv  # 刀A：块外（build_event 之外）format! 纯{}模板收编 evf
do_block_lit = '--block-lit' in sys.argv    # 刀C：块外 ≥40B 散件字面量 evtx 化（24-39B 档勘察实测净负，不做）
do_tables2 = '--tables2' in sys.argv        # 刀B：表通道扩容（结构体表字段级 `_n` + 整表连号基址）
# v1.16.34 单1 刀α：精度模板通道（--block-prec）——{:.0}/{:.1}/{:.2} 占位变换
# {} 入 blob，调用点实参按占位序包 crate::data::Fmt0/1/2 Display 包装。开启后
# format! 收集域与 --block-tmpl 同域（纯{} 站早已收编完，重跑幂等无新命中）。
# pack_text.py:139 旧『非纯{}自动跳过不得手改』纪律由本通道正式接替（分类失败的
# spec 仍保守留原地——通道与 blob 永不脱钩）。
do_block_prec = '--block-prec' in sys.argv
# v1.16.33 单1 NIZE 通道开关（special 副标 match 表通道 + const 串表 N 化批）——
# alloc dedup 共享行、重跑幂等（域已换出即跳过），子域可单刀量测。
# 注意用 startswith 前缀判（--nize=special 是单元素，in 列表判定必漏）
do_nize = any(narg.startswith('--nize') for narg in sys.argv)
nize_scope = 'all'
for narg in sys.argv:
    if narg.startswith('--nize='):
        nize_scope = narg.split('=', 1)[1]

# 块外扫描域：v1.16.33 单1 主刀②起 draw.rs/charpanel.rs 延后批开刀（v1.16.29
# 勘察遗留）——bare≥40B 站 evtx 零分配（&'static str blob 切片，与字面量引用
# 等价、仅多两次数组访问+OnceLock 原子读 ns 级）；fmt 纯{} 站 evf（String 分配
# 与原每帧 format! 等价）。const 表体仍由 const_item_ranges 排除（N 化归 --nize
# 通道）、cfg(test) 测试串永不入池、非纯{}（{:.2}/{:.0}）自动跳过不得手改
BLOCK_SCAN_FILES = ['game.rs', 'data.rs', 'guide.rs', 'main.rs', 'menu.rs',
                    'npc.rs', 'world.rs', 'gallery.rs', 'ui.rs', 'art.rs',
                    'cam.rs', 'pixel.rs', 'save.rs', 'sounds.rs',
                    'draw.rs', 'charpanel.rs']
SRC_DIR = ROOT / 'src'

# v1.16.29 刀B 表通道扩容清单。
# ARR_TABLES：整表连号（行主序 alloc_fresh，基址 emit 进 text_offsets.inc；表本体
# 随后由施工方换查表 fn——重跑时表已不在源码，走 .inc 基址回读，绝不二次收编）。
#   (表名, 文件, 基址键, 期望条数)
# v1.16.32 单1 第三批：副刀（DONE_NOTE/CHAIN_NAMES@game.rs）+ data.rs 尾部小表批。
# 期望条数=数组字面量条数（LIT.findall 全量对账）；表体含引号注释会虚增计数——
# 勘察已核：九表体内均无引号注释（INHERITS 体内注释含 "already borrowed" 走
# STRUCT 通道按字段匹配，不入本清单）。
ARR_TABLES = [
    ('NPC_TALES', 'data.rs', 'TXT_TALES_BASE', 12),   # [[&str;3];4] 行主序 npc*3+seg
    ('HELP_LINES', 'draw.rs', 'TXT_HELP_BASE', 17),   # [&str;17]
    ('MINOR_LINES', 'data.rs', 'TXT_MINOR_BASE', 10),  # [&str;10] 模拟杂事线
    ('COMP_TALES', 'data.rs', 'TXT_COMPT_BASE', 3),   # [&str;3] 道侣共通小传
    ('NPC_BRIEF', 'data.rs', 'TXT_NPCBRIEF_BASE', 4),  # [&str;4] 人物页一行简介
    ('DONGFU_FAC_DESC', 'data.rs', 'TXT_DFD_BASE', 3),  # [&str;3] 洞府设施说明
    ('WEATHER_FLAVOR', 'data.rs', 'TXT_WFLAV_BASE', 5),  # [&str;5] 天象流转画面句
    ('EXPED_CHOICE_LABELS', 'data.rs', 'TXT_EXPCHO_BASE', 8),  # [[&str;2];4] 行主序 k*2+j
    ('AD_CREATIVES', 'data.rs', 'TXT_ADCREA_BASE', 3),  # [&str;3] 假广告轮换
    ('DONE_NOTE', 'game.rs', 'TXT_DONE_BASE', 9),     # [&str;9] 章成金行副句
    ('CHAIN_NAMES', 'game.rs', 'TXT_CHAIN_BASE', 18),  # [&str;18] 连锁链名
]
# STRUCT_TABLES：结构体表字段级——表体内 `field: "raw"` 原位换 `field_n: N`
#（alloc_fresh 行主序，同字段各行连号）；`lore: &[...]` 类对子字段换
# `lore_ns: S, lore_len: L`（段内连号）。字段声明与 impl 访问器由施工方手改
#（脚本只动表体与真相源，与刀②/③分工同款）；期望条数按行数钉死防半收编。
#   (表名, 文件, 单值字段列表, 对子字段列表, 行数)
# v1.16.32 单1 主刀第三批：data.rs 结构体表整批 + npc.rs/game.rs 副刀结构体表。
# CHAPTERS 双条目——第一条目 num/name（9 行），第二条目 goals.text（嵌套
# ChapterGoal{ text, pid } 原位换 text_n，pid 保留；期望 18=9 章×2 目标钉死）。
# NPC_DESC 不收编（#[allow(dead_code)] 死表——未引用 const 零 rdata，收编反而净增）。
STRUCT_TABLES = [
    ('CHAPTERS', 'data.rs', ['intro', 'open', 'reward_note', 'couplet'], ['lore'], 9),
    ('ENDINGS', 'game.rs', ['name', 'alias', 'body', 'gain'], [], 4),
    ('LOCATIONS', 'data.rs', ['desc'], [], 6),
    ('HINTS', 'guide.rs', ['text'], [], 14),
    ('CHAPTERS', 'data.rs', ['num', 'name'], [], 9),
    ('CHAPTERS', 'data.rs', ['text'], [], 18),
    ('TECHS', 'data.rs', ['name', 'desc'], [], 8),
    ('RELICS', 'data.rs', ['name', 'desc'], [], 8),
    ('PILLS', 'data.rs', ['name', 'desc'], [], 6),
    ('TALENTS', 'data.rs', ['name', 'desc'], [], 33),
    ('UPGRADES', 'data.rs', ['name', 'desc'], [], 10),
    ('ACHS', 'data.rs', ['name', 'desc'], [], 30),
    ('ORIGINS', 'data.rs', ['name', 'desc'], [], 6),
    ('MAP_THEMES', 'data.rs', ['name', 'desc'], [], 5),
    ('REGION_MECH', 'data.rs', ['mech', 'desc'], [], 5),
    ('SECT_GOODS', 'data.rs', ['name', 'desc'], [], 6),
    ('EXPED_STEPS', 'data.rs', ['text'], [], 8),
    ('NPC_EVENTS', 'npc.rs', ['title', 'text'], [], 4),
    ('INHERITS', 'game.rs', ['name', 'desc'], [], 9),
]
# HINTS text 单值连号 → 附发基址（static_measure.py 指引叶宽度口径改走真相源）
# v1.16.32 单1 第三批整表连号基址：副刀 ARR（DONE_NOTE/CHAIN_NAMES）+ data.rs
# 尾部小表批（MINOR_LINES/COMP_TALES/NPC_BRIEF/DONGFU_FAC_DESC/WEATHER_FLAVOR/
# EXPED_CHOICE_LABELS/AD_CREATIVES）。WEATHERS/SLOTS/TIER_NAMES/PORTRAIT_STAGE_
# NAMES/DONGFU_FACS/NPC_NAMES/NPC_TIER_NAMES/COMPANION_NAMES/L12/L4 净负或≈0
# 不收编（勘察模型：raw 太短，访问器+delta 吃光收益；COMPANION_NAMES 另涉
# save.rs 读档匹配路径 30+ 站）
BASE_KEYS_TABLES2 = ('TXT_TALES_BASE', 'TXT_HELP_BASE', 'TXT_HINT_BASE',
                     'TXT_MINOR_BASE', 'TXT_COMPT_BASE', 'TXT_NPCBRIEF_BASE',
                     'TXT_DFD_BASE', 'TXT_WFLAV_BASE', 'TXT_EXPCHO_BASE',
                     'TXT_ADCREA_BASE', 'TXT_DONE_BASE', 'TXT_CHAIN_BASE')

# v1.16.33 单1 NIZE 通道清单（--nize）。与 ARR/STRUCT 通道不同：不走 base+i 连号，
# 每表产出 [u16;K] N 表（元素序=原数组序），N 走 alloc dedup——同值串共享既有行，
# COMP_PAGE_PREWARM_16 双档/RECALL_CHROME∩EV_NOTE_PREWARM/COMP_PAGE_PREWARM 内
# DUAL_CULT_HINT 引用由此零额外行。基址不进 .inc（N 在源码表内，重跑幂等无需回读）。
# NIZE_CONSTS：单串 const——`const NAME: &str = "…";` 整项换 `const NAME_N: u16 = N;`
#（引用位 evtx(NAME_N as usize)/访问器由施工方手改）。必须先于 NIZE_TABLES 处理
#（表内 ident 引用查此处换出的 N）。
NIZE_CONSTS = [
    ('DUAL_CULT_HINT', 'charpanel.rs', 'DUAL_CULT_HINT_N'),
    ('GUIDE_PANEL_NOTE', 'draw.rs', 'GUIDE_PANEL_NOTE_N'),
]
# NIZE_TABLES：串数组 const——`const NAME: … = [ … ];` 整项换
# `const NAME_NS: [u16; K] = [N0, …];`。元素=字符串字面量 + 白名单裸 ident 引用
#（源序解析）。期望元素数钉死防半收编（勘实测值，nize_probe.py 口径）。
NIZE_TABLES = [
    ('COMP_PAGE_PREWARM', 'charpanel.rs', 'COMP_PAGE_PREWARM_NS', 30, ('DUAL_CULT_HINT',)),
    ('COMP_PAGE_PREWARM_16', 'charpanel.rs', 'COMP_PAGE_PREWARM_16_NS', 3, ()),
    ('HIST_PREWARM_STRS', 'charpanel.rs', 'HIST_PREWARM_NS', 14, ()),
    ('PREWARM_STRS', 'gallery.rs', 'PREWARM_STRS_NS', 21, ()),
    ('EV_NOTE_PREWARM', 'gallery.rs', 'EV_NOTE_PREWARM_NS', 24, ()),
    ('RECALL_CHROME', 'gallery.rs', 'RECALL_CHROME_NS', 3, ()),
    ('GUIDE_PANEL_LINES', 'draw.rs', 'GUIDE_PANEL_LINES_NS', 5, ()),
    ('GUIDE_TABS', 'draw.rs', 'GUIDE_TABS_NS', 2, ()),
]

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

# ---- v1.16.34 单1 刀α：精度模板分类（--block-prec）----
# {:.0}/{:.1}/{:.2} 精度占位变换 {} 入 blob（blob 纪律保持纯 {}——data.rs evf
# split("{}") 机制零改动），调用点按占位序把对应实参包进 data.rs Fmt0/Fmt1/Fmt2
# Display 包装（impl 内 write!(f,"{:.N}",self.0) 与原 format!("{:.N}") 走同一
# f64 Display Formatter 路径，产物逐字节等价）。其余形态（{{}} 转义、{:>8}、
# {:#x}、{:.3}…）一律返回 None 保守留原地——『非纯{}自动跳过不得手改』纪律由
# 本通道接替后依然成立：通道收编的每一站都经过同一分类器，blob 与调用点永不
# 脱钩漂移。
PREC_RE = re.compile(r'\{:\.([012])\}')

def prec_classify(raw: str):
    """左→右扫占位。返回 (变换后 raw, kind 列表) 或 None。
    kind：None=纯 {}（实参原样）；'Fmt0'/'Fmt1'/'Fmt2'=包同名 wrapper。
    占位序==隐式位置实参序（Rust format! 语义）；扫描到的 '}'（}} 转义或孤立）
    与未识别的 '{' 形态都判 None。"""
    out = []
    kinds = []
    k = 0
    n = len(raw)
    while k < n:
        c = raw[k]
        if c == '{':
            if raw.startswith('{}', k):
                out.append('{}')
                kinds.append(None)
                k += 2
                continue
            mm = PREC_RE.match(raw, k)
            if mm:
                out.append('{}')
                kinds.append('Fmt' + mm.group(1))
                k = mm.end()
                continue
            return None
        if c == '}':
            return None
        out.append(c)
        k += 1
    return ''.join(out), kinds

def fmt_arg(a: str, kind):
    """evf 实参列表元素：纯{}实参原样 unsize；精度实参包 FmtN 构造
    （crate::data:: 全限定——不依赖各文件 glob 导入，防分叉）。
    精度实参统一 `(expr) as f64` 收敛到包装域：f32→f64 转换精确无损（同精确
    二进制值同精规格产物逐字节一致——Rust 浮点精度格式化按精确值正确舍入），
    f64→f64 恒等零开销（trivial_casts 默认 allow）；首轮 27 站实证 11 站 f32
    （charpanel785/draw1241/draw2706/game2896/3243/3281/3286×2/npc60/world1214/
    1215），已生成站由施工方补 cast 对齐规范形。"""
    if kind is None:
        return '&(%s) as &dyn core::fmt::Display' % a
    return '&(crate::data::%s((%s) as f64)) as &dyn core::fmt::Display' % (kind, a)

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
        # ---- 刀①模板通道（v1.16.34 刀α 起精度占位经 --block-prec 接替）----
        ph = raw.count('{}')
        residue = raw.replace('{}', '')
        # prec 非 None 时占位形态已由分类器判定合法（纯{} 或 {:.0/1/2} 混排）；
        # 未开旗/分类失败维持旧纪律：非纯{}跳过不得手改
        prec = prec_classify(raw) if do_block_prec else None
        if prec is None and (ph < 1 or '{' in residue or '}' in residue):
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
        # 占位数对账：精度混排时占位总数=kinds 长度（{} 与 {:.N} 都各耗一实参）
        nph = len(prec[1]) if prec is not None else ph
        if nargs != nph:
            tmpl_skips.append((m2.start(1), raw[:40], '占位%d≠实参%d' % (nph, nargs)))
            continue
        if prec is not None and prec[0] != raw:
            # 精度行：blob 存变换后 raw（纯 {} 纪律不变），alloc 同样按变换后值
            # 去重（charpanel==npc/world 多重共享行由此合并）；原文注释与旧 evf
            # 行同款取 blob 行内容（= N 解码值，防漂移钉测口径一致）
            prow, pkinds = prec
            pval = decode_raw(prow)
            assert pval is not None and encode_val(pval) == prow, \
                '精度变换 raw 往返失败: %r' % prow[:40]
            n = alloc(prow)
            tmpl_items.append((m_start, close, prow, pval.encode('utf-8'), n, args, pkinds))
        else:
            kinds0 = prec[1] if prec is not None else [None] * ph
            n = alloc(raw)
            tmpl_items.append((m_start, close, raw, val.encode('utf-8'), n, args, kinds0))
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
# v1.16.34 刀α：实参按占位序包 Fmt 包装（fmt_arg）；kinds 与 args 等长由
# 『占位≠实参』跳过纪律保证，此处再 assert 防 zip 静默截断
tmpl_edits = []
for ms2, cl2, r2, _vb2, n2, args2, kinds2 in tmpl_items:
    assert len(args2) == len(kinds2), 'tmpl kinds/args 长度失配 @N=%d' % n2
    joined2 = ', '.join(fmt_arg(a2, kd2) for a2, kd2 in zip(args2, kinds2))
    tmpl_edits.append((ms2, cl2 + 1, 'evf(%d, &[%s])/* 原文：%s */' % (n2, joined2, r2)))
all_edits = (
    [(p - 1, p + len(r) + 1, 'evtx(%d)/* 原文：%s */' % (n, r)) for p, r, _vb, n in lit_items]
    + tmpl_edits
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
out_tmpl_cnt = out_lit_cnt = out_tmpl_b = out_lit_b = out_prec_cnt = 0
# v1.16.34 刀α：--block-prec 与 tmpl/lit 同域扫描（纯{}站已收编完，重跑幂等）
if do_block_tmpl or do_block_lit or do_block_prec:
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
                if not (do_block_tmpl or do_block_prec) or b7 < MIN_B:
                    continue
                ph7 = raw7.count('{}')
                res7 = raw7.replace('{}', '')
                # v1.16.34 刀α：--block-prec 开时精度占位走 prec_classify；
                # 未开旗/分类失败维持旧『非纯{}自动跳过』纪律
                prec7 = prec_classify(raw7) if do_block_prec else None
                if prec7 is None and (ph7 < 1 or '{' in res7 or '}' in res7):
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
                nph7 = len(prec7[1]) if prec7 is not None else ph7
                if nargs7 == -1 or nargs7 != nph7:
                    continue
                # 实参逐位显式 unsize：&[&dyn Display] 元素型一统，不吃推断顺序运气
                #（v1.16.29 main.rs 自检行 i32/u32 混排实证：隐式依赖首元素定型，
                #  异型即 E0308——新收编站一律显式，codegen 等价零字节差）
                if prec7 is not None and prec7[0] != raw7:
                    # 精度行：blob 存变换后 raw（纯 {} 纪律），alloc 按变换后值
                    # 去重（charpanel==npc/world 共享行合并）；实参按占位序包 Fmt
                    prow7, pk7 = prec7
                    pval7 = decode_raw(prow7)
                    assert pval7 is not None and encode_val(pval7) == prow7, \
                        '精度变换 raw 往返失败: %r' % prow7[:40]
                    n7 = alloc(prow7)
                    assert len(args7) == len(pk7), '块外精度 kinds/args 失配'
                    edits_f.append((ms7, cl7 + 1,
                                    'evf(%d, &[%s])/* 原文：%s */'
                                    % (n7, ', '.join(fmt_arg(a7, kd7) for a7, kd7
                                                     in zip(args7, pk7)), prow7),
                                    fname2))
                    out_prec_cnt += 1
                else:
                    n7 = alloc(raw7)
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
if do_block_tmpl or do_block_lit or do_block_prec:
    print('block-out scan: %d evf templates (%d B, %d precision) + %d evtx lits >=40B (%d B)'
          % (out_tmpl_cnt, out_tmpl_b, out_prec_cnt, out_lit_cnt, out_lit_b))

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
    if not OFFS.exists():
        # v1.16.34 单1 low 清账（v1.16.33 留档：循环原在 if OFFS.exists() 体外，
        # OFFS 缺失时 txt 未绑定 NameError）——无 .inc 时直接返回空表，键读取
        # 走调用方「基址缺失」分支（首轮收集语义不变）
        return out
    txt = OFFS.read_text(encoding='utf-8')
    for key in (('TXT_NAME_BASE', 'TXT_BRIEF_BASE', 'TXT_NAME2_BASE',
                 'TXT_BRIEF2_BASE', 'TXT_NAME3_BASE', 'TXT_BRIEF3_BASE',
                 'TXT_NAME4_BASE', 'TXT_BRIEF4_BASE', 'TXT_NAME5_BASE',
                 'TXT_BRIEF5_BASE', 'TXT_NAME6_BASE', 'TXT_BRIEF6_BASE',
                 'TXT_NAME7_BASE', 'TXT_BRIEF7_BASE',
                 'TXT_NAME8_BASE', 'TXT_BRIEF8_BASE',
                 'TXT_NAME9_BASE', 'TXT_BRIEF9_BASE',
                 'TXT_NAME10_BASE', 'TXT_BRIEF10_BASE',
                 'TXT_NAME11_BASE', 'TXT_BRIEF11_BASE',
                 'TXT_NAME12_BASE', 'TXT_BRIEF12_BASE',
                 'TXT_NAME13_BASE', 'TXT_BRIEF13_BASE',
                 'TXT_NAME14_BASE', 'TXT_BRIEF14_BASE',
                 'TXT_NAME15_BASE', 'TXT_BRIEF15_BASE')
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
ARR_EXPECT = {'TXT_TALES_BASE': 12, 'TXT_HELP_BASE': 17, 'TXT_HINT_BASE': 14,
              'TXT_MINOR_BASE': 10, 'TXT_COMPT_BASE': 3, 'TXT_NPCBRIEF_BASE': 4,
              'TXT_DFD_BASE': 3, 'TXT_WFLAV_BASE': 5, 'TXT_EXPCHO_BASE': 8,
              'TXT_ADCREA_BASE': 3, 'TXT_DONE_BASE': 9, 'TXT_CHAIN_BASE': 18}
for bk3, exp3 in ARR_EXPECT.items():
    if bk3 not in new_base_emits and bk3 in inc_bases:
        assert next_n - 1 >= inc_bases[bk3] + exp3 - 1, '%s 基址段越界' % bk3
        new_base_emits[bk3] = inc_bases[bk3]

# ================= v1.16.33 单1：NIZE 通道（--nize） =================
# 两个子域均 append-only（alloc dedup 共享行，旧 N 永不变号）、重跑幂等。
# 纪律：循环变量一律带后缀（nize_* / n 前缀+数字）——模块级 i/j 是 build_event
# 定位索引与括号游标（:186-200），泄漏覆盖会把尾部拼接截断（v1.16.28 事故先例，
# 见 :512 注）；本通道绝不裸用 i/j。

def nize_const_item(text, name):
    """`(?m)^[ \\t]*const[ \\t]+NAME\\b` 起、字符串/char/注释感知扫到 depth0 ';'
    的整项区间（起, 止含分号）。表已换出（NAME_NS 等不匹配 \\b）返回 None。"""
    nm1 = re.search(r'(?m)^[ \t]*const[ \t]+' + name + r'\b', text)
    if not nm1:
        return None
    nk1 = nm1.end()
    nlen1 = len(text)
    ndep1 = 0
    while nk1 < nlen1:
        nc1 = text[nk1]
        if nc1 == '"':
            nk1 = skip_str(text, nk1, '"')
            continue
        if nc1 == "'":
            nmm1 = CHAR_LIT.match(text, nk1)
            if nmm1:
                nk1 = nmm1.end()
                continue
        if nc1 == '/' and text[nk1:nk1 + 2] == '//':
            ne1 = text.find('\n', nk1)
            if ne1 == -1:
                return None
            nk1 = ne1
            continue
        if nc1 == '/' and text[nk1:nk1 + 2] == '/*':
            ne1 = text.find('*/', nk1 + 2)
            if ne1 == -1:
                return None
            nk1 = ne1 + 2
            continue
        if nc1 in '([{':
            ndep1 += 1
        elif nc1 in ')]}':
            ndep1 -= 1
        elif nc1 == ';' and ndep1 == 0:
            return (nm1.start(), nk1 + 1)
        nk1 += 1
    return None

nize_report = []
nize_const_n = {}    # NIZE_CONSTS 换出的 const 名 -> N（NIZE_TABLES ident 引用查此表）
nize_elem_cnt = 0
nize_raw_b = 0
if do_nize:
    # ---- 子域 A：special 副标 match 表通道（game.rs eff_desc）----
    if nize_scope in ('all', 'special'):
        nsp1 = re.search(r'let s = match self\.special \{', src)
        if nsp1 is None:
            if 'const SPECIAL_SUB' in src:
                nize_report.append('special: match 已换 N 表（幂等跳过）')
            else:
                raise SystemExit('nize special: match 与 SPECIAL_SUB 均未找到——人工核对')
        else:
            nopen1 = src.index('{', nsp1.start())
            nk2 = nopen1
            ndep2 = 0
            nclose1 = None
            while nk2 < len(src):
                nc2 = src[nk2]
                if nc2 == '"':
                    nk2 = skip_str(src, nk2, '"')
                    continue
                if nc2 == "'":
                    nmm2 = CHAR_LIT.match(src, nk2)
                    if nmm2:
                        nk2 = nmm2.end()
                        continue
                if nc2 == '/' and src[nk2:nk2 + 2] == '//':
                    ne2 = src.find('\n', nk2)
                    if ne2 == -1:
                        break
                    nk2 = ne2
                    continue
                if nc2 == '/' and src[nk2:nk2 + 2] == '/*':
                    ne2 = src.find('*/', nk2 + 2)
                    if ne2 == -1:
                        break
                    nk2 = ne2 + 2
                    continue
                if nc2 == '{':
                    ndep2 += 1
                elif nc2 == '}':
                    ndep2 -= 1
                    if ndep2 == 0:
                        nclose1 = nk2
                        break
                nk2 += 1
            assert nclose1 is not None, 'special match 括号配平失败'
            nsemi1 = src.index(';', nclose1)
            nspan1 = src[nsp1.start():nsemi1 + 1]
            # 注释掩码（保位）——臂解析只在代码位；注释行原样上移保留
            nmask1 = list(nspan1)
            nk3 = 0
            while nk3 < len(nspan1):
                nc3 = nspan1[nk3]
                if nc3 == '"':
                    nk3 = skip_str(nspan1, nk3, '"')
                    continue
                if nc3 == '/' and nspan1[nk3:nk3 + 2] == '//':
                    ne3 = nspan1.find('\n', nk3)
                    if ne3 == -1:
                        ne3 = len(nspan1)
                    for nfill3 in range(nk3, ne3):
                        nmask1[nfill3] = ' '
                    nk3 = ne3
                    continue
                if nc3 == '/' and nspan1[nk3:nk3 + 2] == '/*':
                    ne3 = nspan1.find('*/', nk3 + 2)
                    if ne3 == -1:
                        break
                    for nfill3 in range(nk3, ne3 + 2):
                        nmask1[nfill3] = ' '
                    nk3 = ne3 + 2
                    continue
                nk3 += 1
            nclean1 = ''.join(nmask1)
            narms1 = {}
            for nam1 in re.finditer(r'(\d+)\s*=>\s*"((?:[^"\\]|\\.)*)"', nclean1):
                assert int(nam1.group(1)) not in narms1, \
                    'special 臂 %d 重复解析' % int(nam1.group(1))
                narms1[int(nam1.group(1))] = nam1.group(2)
            nfb1 = re.search(r'_\s*=>\s*"((?:[^"\\]|\\.)*)"', nclean1)
            assert nfb1 is not None, 'special match 无 _ 兜底臂'
            assert nclean1.count('=>') == len(narms1) + 1, \
                'special 臂数不配：数字臂 %d + _ 1 != => 总数 %d（注释残留模式位？）' \
                % (len(narms1), nclean1.count('=>'))
            nsize1 = max(max(narms1), 76) + 1   # 表覆盖 0..=76：76/预留窗位与 >76 值
            nfb_val1 = decode_raw(nfb1.group(1))
            assert nfb_val1 is not None and encode_val(nfb_val1) == nfb1.group(1), \
                'special 兜底臂往返失败'
            assert '*/' not in nfb1.group(1) and '/*' not in nfb1.group(1)
            nfb_n1 = alloc(nfb1.group(1))
            nslot1 = [None] * nsize1
            for nv1 in sorted(narms1):
                nraw4 = narms1[nv1]
                nval4 = decode_raw(nraw4)
                assert nval4 is not None and encode_val(nval4) == nraw4, \
                    'special 臂往返失败: %r' % nraw4[:20]
                assert '*/' not in nraw4 and '/*' not in nraw4
                nslot1[nv1] = alloc(nraw4)
                nize_elem_cnt += 1
                nize_raw_b += len(nval4.encode('utf-8'))
            for nsi1 in range(nsize1):
                if nslot1[nsi1] is None:
                    nslot1[nsi1] = nfb_n1    # 稀疏窗位/槽0/未登记值 -> 机缘（= _ 臂）
            # 臂间注释行原样上移（链窗位语义文档不丢）
            ncomment1 = []
            for nline1 in nspan1.split('\n'):
                nstrip1 = nline1.strip()
                if nstrip1.startswith('//'):
                    ncomment1.append('            ' + nstrip1)
            nrowtxt1 = []
            for nrs1 in range(0, nsize1, 8):
                nchunk1 = nslot1[nrs1:nrs1 + 8]
                nrowtxt1.append('                '
                                + ', '.join(str(x) for x in nchunk1) + ',')
            nrepl1 = (
                '\n'.join(ncomment1) + '\n'
                + '            // v1.16.33 单1 副标表通道（--nize）：本 match 收编 textpack——\n'
                + '            // special→N 查表 evtx 一次间接（非帧路径，draw.rs stale 缓存）；\n'
                + '            // 稀疏窗位/未登记值（>%d 与槽 0）统一落「机缘」兜底 N，与原\n' % (nsize1 - 1)
                + '            // _ 臂逐值等价（special_subtitle_fix_tests 全表钉验收）。\n'
                + '            const SPECIAL_SUB: [u16; %d] = [\n' % nsize1
                + '\n'.join(nrowtxt1) + '\n'
                + '            ];\n'
                + '            let s = evtx(SPECIAL_SUB[(self.special as usize)\n'
                + '                .min(SPECIAL_SUB.len() - 1)] as usize);'
            )
            out_edits.setdefault('game.rs', []).append((nsp1.start(), nsemi1 + 1, nrepl1))
            nize_report.append('special: %d 臂 + 兜底「机缘」，表 [u16;%d]，臂 raw %dB'
                               % (len(narms1), nsize1, nize_raw_b))
    # ---- 子域 A2（v1.16.34 单1 β+γ）：match 臂簇 N 表通道（--nize=arms）----
    # 每簇照 v1.16.33 SPECIAL_SUB 口径：臂字面量 alloc dedup → fn 内 const
    # [u16;K] N 表 + evtx 查表 + .min(len-1) 钳制（兜底臂=末槽，逐值等价）；
    # 索引表达式按簇源语义手写（表内单行注释留档映射推导）；等价性验收
    # special_sweep_gen.py 同型扩面（git HEAD 原臂字面量对照 N 表+真相源逐
    # slot 全等）。域已换出（const 名在文件中）即幂等跳过。
    # 纪律：循环变量一律带后缀——模块级 i/j 是 build_event 定位索引与括号游标。
    def nize_arm_mask(nm_span):
        """注释掩码（保位）：臂解析只在代码位；臂间注释行原样上移保留。
        字符串字面量原样保留（元素值来源）——照 special 子域 mask 同款。"""
        nm_list = list(nm_span)
        nm_k = 0
        while nm_k < len(nm_span):
            nm_c = nm_span[nm_k]
            if nm_c == '"':
                nm_k = skip_str(nm_span, nm_k, '"')
                continue
            if nm_c == "'":
                nm_m = CHAR_LIT.match(nm_span, nm_k)
                if nm_m:
                    nm_k = nm_m.end()
                    continue
            if nm_c == '/' and nm_span[nm_k:nm_k + 2] == '//':
                nm_e = nm_span.find('\n', nm_k)
                if nm_e == -1:
                    nm_e = len(nm_span)
                for nm_f in range(nm_k, nm_e):
                    nm_list[nm_f] = ' '
                nm_k = nm_e
                continue
            if nm_c == '/' and nm_span[nm_k:nm_k + 2] == '/*':
                nm_e = nm_span.find('*/', nm_k + 2)
                if nm_e == -1:
                    break
                for nm_f in range(nm_k, nm_e + 2):
                    nm_list[nm_f] = ' '
                nm_k = nm_e + 2
                continue
            nm_k += 1
        return ''.join(nm_list)

    def nize_arm_alloc(nm_raw, nm_tag):
        """臂字面量入池：decode/往返/注释界 assert + alloc dedup，返回 N。"""
        nm_val = decode_raw(nm_raw)
        assert nm_val is not None and encode_val(nm_val) == nm_raw, \
            '%s 臂往返失败: %r' % (nm_tag, nm_raw[:20])
        assert '*/' not in nm_raw and '/*' not in nm_raw, '%s 臂含注释界' % nm_tag
        return alloc(nm_raw)

    def nize_indent(nm_text, nm_pos):
        """锚所在行 行首→锚起点段（=行首缩进）。anchor 匹配起点在行中、
        span 自锚起——span.lstrip 前缀法拿不到行首缩进（v1.16.34 初版事故注）。"""
        return nm_text[nm_text.rfind('\n', 0, nm_pos) + 1:nm_pos]

    if nize_scope in ('all', 'arms'):
        ARM_NUM = r'(\d+)\s*=>\s*"((?:[^"\\]|\\.)*)"'
        ARM_RNG = r'(\d+\.\.=\d+)\s*=>\s*"((?:[^"\\]|\\.)*)"'
        ARM_FB = r'_\s*=>\s*"((?:[^"\\]|\\.)*)"'
        # --- 簇1：sim_demon 破境场景四臂（game.rs build_event 内）---
        if 'const SIM_DEMON_SCENE_NS' in src:
            nize_report.append('SIM_DEMON_SCENE_NS: 已换 N 表（幂等跳过）')
        else:
            nm_a = re.search(r'let scene = match target \{', src)
            assert nm_a, 'arms 簇1: let scene match 锚缺失'
            nm_b = src.index('{', nm_a.start())
            nm_dep = 0
            nm_k = nm_b
            nm_close = None
            while nm_k < len(src):
                nm_c = src[nm_k]
                if nm_c == '"':
                    nm_k = skip_str(src, nm_k, '"')
                    continue
                if nm_c == "'":
                    nm_m = CHAR_LIT.match(src, nm_k)
                    if nm_m:
                        nm_k = nm_m.end()
                        continue
                if nm_c == '/' and src[nm_k:nm_k + 2] == '//':
                    nm_e = src.find('\n', nm_k)
                    if nm_e == -1:
                        break
                    nm_k = nm_e
                    continue
                if nm_c == '/' and src[nm_k:nm_k + 2] == '/*':
                    nm_e = src.find('*/', nm_k + 2)
                    if nm_e == -1:
                        break
                    nm_k = nm_e + 2
                    continue
                if nm_c == '{':
                    nm_dep += 1
                elif nm_c == '}':
                    nm_dep -= 1
                    if nm_dep == 0:
                        nm_close = nm_k
                        break
                nm_k += 1
            assert nm_close is not None, 'arms 簇1: match 配平失败'
            nm_semi = src.index(';', nm_close)
            nm_span = src[nm_a.start():nm_semi + 1]
            # （行首定位已内聚进 nize_indent，span_start 登记行作废）
            nm_clean = nize_arm_mask(nm_span)
            nm_arms = {int(am.group(1)): am.group(2)
                       for am in re.finditer(ARM_NUM, nm_clean)}
            nm_fb = re.search(ARM_FB, nm_clean)
            assert nm_fb is not None, 'arms 簇1: 无 _ 兜底臂'
            assert set(nm_arms) == {11, 12, 13}, \
                'arms 簇1: 数字臂 %s != {11,12,13}' % sorted(nm_arms)
            assert nm_clean.count('=>') == len(nm_arms) + 1, \
                'arms 簇1: 臂数不配（注释残留模式位？）'
            nm_ns = [nize_arm_alloc(nm_arms[11], '簇1/11'),
                     nize_arm_alloc(nm_arms[12], '簇1/12'),
                     nize_arm_alloc(nm_arms[13], '簇1/13'),
                     nize_arm_alloc(nm_fb.group(1), '簇1/_')]
            nize_elem_cnt += 4
            nize_raw_b += sum(len(decode_raw(r).encode('utf-8'))
                              for r in [nm_arms[11], nm_arms[12], nm_arms[13],
                                        nm_fb.group(1)])
            # v1.16.34 修复注：nm_ind 走 nize_indent（锚行首→锚起点段；初版
            # span.lstrip 前缀法恒空串，替换体被压到列 0，语义等价版式尽毁）
            nm_ind = nize_indent(src, nm_a.start())
            nm_repl = (
                nm_ind
                + '// v1.16.34 单1 β刀（--nize=arms）：破境场景四臂收编 textpack——\n'
                + nm_ind
                + '// 卫语句 if target>=11 前提下 saturating_sub(11)+min 钳制，槽3=\n'
                + nm_ind
                + '// 原 _ 臂（target=14 逐值等价；usize 防御照 panic=abort 纪律）\n'
                + nm_ind + 'const SIM_DEMON_SCENE_NS: [u16; 4] = [%d, %d, %d, %d];\n'
                % tuple(nm_ns)
                + nm_ind
                + 'let scene = evtx(SIM_DEMON_SCENE_NS[(target.saturating_sub(11))\n'
                + nm_ind + '    .min(SIM_DEMON_SCENE_NS.len() - 1)] as usize);'
            )
            out_edits.setdefault('game.rs', []).append(
                (nm_a.start(), nm_semi + 1, nm_repl))
            nize_report.append('SIM_DEMON_SCENE_NS: 4 臂（target-11 槽映射），臂 raw %dB'
                               % nize_raw_b)
        # --- 簇2：退稿收束语三档（game.rs settle_coda）---
        if 'const SETTLE_CODA_NS' in src:
            nize_report.append('SETTLE_CODA_NS: 已换 N 表（幂等跳过）')
        else:
            nm_a = re.search(r'match max_realm\.min\(14\) \{', src)
            assert nm_a, 'arms 簇2: settle_coda match 锚缺失'
            nm_b = src.index('{', nm_a.start())
            nm_dep = 0
            nm_k = nm_b
            nm_close = None
            while nm_k < len(src):
                nm_c = src[nm_k]
                if nm_c == '"':
                    nm_k = skip_str(src, nm_k, '"')
                    continue
                if nm_c == "'":
                    nm_m = CHAR_LIT.match(src, nm_k)
                    if nm_m:
                        nm_k = nm_m.end()
                        continue
                if nm_c == '/' and src[nm_k:nm_k + 2] == '//':
                    nm_e = src.find('\n', nm_k)
                    if nm_e == -1:
                        break
                    nm_k = nm_e
                    continue
                if nm_c == '/' and src[nm_k:nm_k + 2] == '/*':
                    nm_e = src.find('*/', nm_k + 2)
                    if nm_e == -1:
                        break
                    nm_k = nm_e + 2
                    continue
                if nm_c == '{':
                    nm_dep += 1
                elif nm_c == '}':
                    nm_dep -= 1
                    if nm_dep == 0:
                        nm_close = nm_k
                        break
                nm_k += 1
            assert nm_close is not None, 'arms 簇2: match 配平失败'
            nm_span = src[nm_a.start():nm_close + 1]
            # （行首定位已内聚进 nize_indent，span_start 登记行作废）
            nm_clean = nize_arm_mask(nm_span)
            nm_rngs = {am.group(1): am.group(2)
                       for am in re.finditer(ARM_RNG, nm_clean)}
            nm_fb = re.search(ARM_FB, nm_clean)
            assert nm_fb is not None, 'arms 簇2: 无 _ 兜底臂'
            assert set(nm_rngs) == {'0..=4', '5..=9'}, \
                'arms 簇2: 区间臂 %s != {0..=4, 5..=9}' % sorted(nm_rngs)
            assert nm_clean.count('=>') == 3, 'arms 簇2: 臂数不配'
            nm_ns = [nize_arm_alloc(nm_rngs['0..=4'], '簇2/0..=4'),
                     nize_arm_alloc(nm_rngs['5..=9'], '簇2/5..=9'),
                     nize_arm_alloc(nm_fb.group(1), '簇2/_')]
            nize_elem_cnt += 3
            nize_raw_b += sum(len(decode_raw(r).encode('utf-8'))
                              for r in [nm_rngs['0..=4'], nm_rngs['5..=9'],
                                        nm_fb.group(1)])
            # v1.16.34 修复注：nm_ind 走 nize_indent（锚行首→锚起点段；初版
            # span.lstrip 前缀法恒空串，替换体被压到列 0，语义等价版式尽毁）
            nm_ind = nize_indent(src, nm_a.start())
            nm_repl = (
                nm_ind
                + '// v1.16.34 单1 β刀（--nize=arms）：退稿收束语三档收编 textpack——\n'
                + nm_ind
                + '// 原 0..=4/5..=9/_ 三臂，min(14)/5 降槽 .min 钳制逐值等价\n'
                + nm_ind + 'const SETTLE_CODA_NS: [u16; 3] = [%d, %d, %d];\n'
                % tuple(nm_ns)
                + nm_ind
                + 'evtx(SETTLE_CODA_NS[(max_realm.min(14) / 5)\n'
                + nm_ind + '    .min(SETTLE_CODA_NS.len() - 1)] as usize)'
            )
            out_edits.setdefault('game.rs', []).append(
                (nm_a.start(), nm_close + 1, nm_repl))
            nize_report.append('SETTLE_CODA_NS: 3 档（min(14)/5 槽映射），臂 raw %dB'
                               % sum(len(decode_raw(r).encode('utf-8'))
                                     for r in [nm_rngs['0..=4'], nm_rngs['5..=9'],
                                               nm_fb.group(1)]))
        # --- 簇3：终战副标四臂（draw.rs draw_sim Finale 分支）---
        # v1.16.35 单5 清账修复：原跳过条件 FTXT.get('draw.rs','') 恒 False——
        # FTXT 仅预载 game.rs（:815），draw.rs 惰性加载在下方 else 内，跳过查
        # 永远查不到键 → 已收编态永远进 else → 锚串（v1.16.34 收编时 match 已
        # 换 FINALE_SUB_NS 查表，grep 'match i {' 0 命中）assert 红，--nize=arms
        # 显式重跑自收编起必红（幂等跳过语义从未兑现）。改直读文件判跳过。
        if 'const FINALE_SUB_NS' in (SRC_DIR / 'draw.rs').read_text(encoding='utf-8'):
            nize_report.append('FINALE_SUB_NS: 已换 N 表（幂等跳过）')
        else:
            if 'draw.rs' not in FTXT:
                FTXT['draw.rs'] = (SRC_DIR / 'draw.rs').read_text(encoding='utf-8')
            nm_dtxt = FTXT['draw.rs']
            # v1.16.35 单3 low② 清账：锚收窄——旧 re.search(r'match i \{') 全文件
            # 过宽（draw.rs 多处 match i，首个命中未必是 draw_sim 副标臂，幸运
            # 命中属运气不属契约）。先定位 draw_sim 函数 span（fn 行到下一同级
            # fn 行）再在 span 界内 search（pos/endpos 界定，.start() 保全文件
            # 坐标，下游 nm_a.start() 两处引用零改动）。
            # v1.16.35 单5 清账注：已收编态锚串随收编消失（v1.16.34 收编把 match
            # 换成 FINALE_SUB_NS 查表，draw.rs 全文件 grep 'match i {' 实测 0 命中）
            # ——本块由上方幂等跳过保护不可达；未收编态（回滚重收编）锚在
            # draw_sim span 内命中，收窄口径在此态下生效。
            nm_fn = re.search(r'^pub fn draw_sim\(', nm_dtxt, re.M)
            assert nm_fn, 'arms 簇3: draw.rs draw_sim 函数锚缺失'
            nm_next = re.search(r'^(?:pub )?fn \w+', nm_dtxt[nm_fn.end():], re.M)
            nm_span_end = (nm_fn.end() + nm_next.start()) if nm_next else len(nm_dtxt)
            nm_a = re.compile(r'match i \{').search(
                nm_dtxt, nm_fn.end(), nm_span_end)
            assert nm_a, 'arms 簇3: draw_sim span 内 match i 锚缺失'
            nm_b = nm_dtxt.index('{', nm_a.start())
            nm_dep = 0
            nm_k = nm_b
            nm_close = None
            while nm_k < len(nm_dtxt):
                nm_c = nm_dtxt[nm_k]
                if nm_c == '"':
                    nm_k = skip_str(nm_dtxt, nm_k, '"')
                    continue
                if nm_c == "'":
                    nm_m = CHAR_LIT.match(nm_dtxt, nm_k)
                    if nm_m:
                        nm_k = nm_m.end()
                        continue
                if nm_c == '/' and nm_dtxt[nm_k:nm_k + 2] == '//':
                    nm_e = nm_dtxt.find('\n', nm_k)
                    if nm_e == -1:
                        break
                    nm_k = nm_e
                    continue
                if nm_c == '/' and nm_dtxt[nm_k:nm_k + 2] == '/*':
                    nm_e = nm_dtxt.find('*/', nm_k + 2)
                    if nm_e == -1:
                        break
                    nm_k = nm_e + 2
                    continue
                if nm_c == '{':
                    nm_dep += 1
                elif nm_c == '}':
                    nm_dep -= 1
                    if nm_dep == 0:
                        nm_close = nm_k
                        break
                nm_k += 1
            assert nm_close is not None, 'arms 簇3: match 配平失败'
            nm_span = nm_dtxt[nm_a.start():nm_close + 1]
            # （行首定位已内聚进 nize_indent，span_start 登记行作废）
            nm_clean = nize_arm_mask(nm_span)
            nm_arms = {int(am.group(1)): am.group(2)
                       for am in re.finditer(ARM_NUM, nm_clean)}
            nm_fb = re.search(ARM_FB, nm_clean)
            assert nm_fb is not None, 'arms 簇3: 无 _ 兜底臂'
            assert set(nm_arms) == {0, 1, 2}, \
                'arms 簇3: 数字臂 %s != {0,1,2}' % sorted(nm_arms)
            assert nm_clean.count('=>') == 4, 'arms 簇3: 臂数不配'
            nm_ns = [nize_arm_alloc(nm_arms[0], '簇3/0'),
                     nize_arm_alloc(nm_arms[1], '簇3/1'),
                     nize_arm_alloc(nm_arms[2], '簇3/2'),
                     nize_arm_alloc(nm_fb.group(1), '簇3/_')]
            nize_elem_cnt += 4
            nize_raw_b += sum(len(decode_raw(r).encode('utf-8'))
                              for r in [nm_arms[0], nm_arms[1], nm_arms[2],
                                        nm_fb.group(1)])
            # v1.16.34 修复注：nm_ind 走 nize_indent（锚行首→锚起点段；初版
            # span.lstrip 前缀法恒空串，替换体被压到列 0，语义等价版式尽毁）
            nm_ind = nize_indent(nm_dtxt, nm_a.start())
            nm_repl = (
                nm_ind
                + '// v1.16.34 单1 β刀（--nize=arms）：终战副标四臂收编 textpack——\n'
                + nm_ind
                + '// i.min 钳制落 _ 臂槽，逐值等价（.to_string() 调用位不变）\n'
                + nm_ind + 'const FINALE_SUB_NS: [u16; 4] = [%d, %d, %d, %d];\n'
                % tuple(nm_ns)
                + nm_ind
                + 'evtx(FINALE_SUB_NS[i.min(FINALE_SUB_NS.len() - 1)] as usize)'
            )
            out_edits.setdefault('draw.rs', []).append(
                (nm_a.start(), nm_close + 1, nm_repl))
            nize_report.append('FINALE_SUB_NS: 4 臂（i 槽映射），臂 raw %dB'
                               % sum(len(decode_raw(r).encode('utf-8'))
                                     for r in [nm_arms[0], nm_arms[1], nm_arms[2],
                                               nm_fb.group(1)]))
        # --- 簇4：境界名 15 臂（data.rs realm_title）---
        if 'const REALM_TITLE_NS' in data_src:
            nize_report.append('REALM_TITLE_NS: 已换 N 表（幂等跳过）')
        else:
            # v1.16.34 修复注：锚只含 match 行——初版锚含 fn 签名，span 整行
            # 吞掉 `pub fn realm_title…`（title_recheck 钉B v1.16.33 同型事故
            # 红线，本轮亲历后修）；match max_realm { 在 data.rs 唯一可独立锚定
            nm_a = re.search(r'match max_realm \{', data_src)
            assert nm_a, 'arms 簇4: realm_title match 锚缺失'
            nm_b = data_src.index('{', nm_a.end() - 1)
            nm_dep = 0
            nm_k = nm_b
            nm_close = None
            while nm_k < len(data_src):
                nm_c = data_src[nm_k]
                if nm_c == '"':
                    nm_k = skip_str(data_src, nm_k, '"')
                    continue
                if nm_c == "'":
                    nm_m = CHAR_LIT.match(data_src, nm_k)
                    if nm_m:
                        nm_k = nm_m.end()
                        continue
                if nm_c == '/' and data_src[nm_k:nm_k + 2] == '//':
                    nm_e = data_src.find('\n', nm_k)
                    if nm_e == -1:
                        break
                    nm_k = nm_e
                    continue
                if nm_c == '/' and data_src[nm_k:nm_k + 2] == '/*':
                    nm_e = data_src.find('*/', nm_k + 2)
                    if nm_e == -1:
                        break
                    nm_k = nm_e + 2
                    continue
                if nm_c == '{':
                    nm_dep += 1
                elif nm_c == '}':
                    nm_dep -= 1
                    if nm_dep == 0:
                        nm_close = nm_k
                        break
                nm_k += 1
            assert nm_close is not None, 'arms 簇4: match 配平失败'
            nm_span = data_src[nm_a.start():nm_close + 1]
            # （行首定位已内聚进 nize_indent，span_start 登记行作废）
            nm_clean = nize_arm_mask(nm_span)
            nm_arms = {int(am.group(1)): am.group(2)
                       for am in re.finditer(ARM_NUM, nm_clean)}
            nm_fb = re.search(ARM_FB, nm_clean)
            assert nm_fb is not None, 'arms 簇4: 无 _ 兜底臂'
            assert set(nm_arms) == set(range(14)), \
                'arms 簇4: 数字臂 %s != 0..13' % sorted(nm_arms)
            assert nm_clean.count('=>') == 15, 'arms 簇4: 臂数不配'
            nm_lit = [nm_arms[k] for k in range(14)] + [nm_fb.group(1)]
            nm_ns = [nize_arm_alloc(r, '簇4/%d' % q if q < 14 else '簇4/_')
                     for q, r in enumerate(nm_lit)]
            nize_elem_cnt += 15
            nize_raw_b += sum(len(decode_raw(r).encode('utf-8')) for r in nm_lit)
            # v1.16.34 修复注：nm_ind 走 nize_indent（锚行首→锚起点段；初版
            # span.lstrip 前缀法恒空串，替换体被压到列 0，语义等价版式尽毁）
            nm_ind = nize_indent(data_src, nm_a.start())
            nm_rows = []
            for nm_r0 in range(0, 15, 8):
                nm_rows.append(nm_ind + '    '
                               + ', '.join(str(x) for x in nm_ns[nm_r0:nm_r0 + 8])
                               + (',' if nm_r0 + 8 < 15 else ','))
            nm_repl = (
                nm_ind
                + '// v1.16.34 单1 β刀（--nize=arms）：境界名 15 臂收编 textpack（调用点\n'
                + nm_ind
                + '// 均一次性 UI 文案，evtx 零分配）——max_realm.min 钳制落 _ 臂槽逐值等价\n'
                + nm_ind + 'const REALM_TITLE_NS: [u16; 15] = [\n'
                + '\n'.join(nm_rows) + '\n'
                + nm_ind + '];\n'
                + nm_ind
                + 'evtx(REALM_TITLE_NS[max_realm.min(REALM_TITLE_NS.len() - 1)] as usize)'
            )
            out_edits.setdefault('data.rs', []).append(
                (nm_a.start(), nm_close + 1, nm_repl))
            nize_report.append('REALM_TITLE_NS: 15 臂（0..13 直映射+_ 末槽），臂 raw %dB'
                               % sum(len(decode_raw(r).encode('utf-8')) for r in nm_lit))
        # --- 簇5：双修门控单臂（charpanel.rs DualGate::ThisYear，裸字面量 evtx）---
        if 'charpanel.rs' not in FTXT:
            FTXT['charpanel.rs'] = (SRC_DIR / 'charpanel.rs').read_text(encoding='utf-8')
        nm_ctxt = FTXT['charpanel.rs']
        nm_a = re.search(r'DualGate::ThisYear => "((?:[^"\\]|\\.)*)"\.to_string\(\),',
                         nm_ctxt)
        if nm_a is None:
            nize_report.append('DualGate::ThisYear 单臂: 已换 evtx（幂等跳过）')
        else:
            nm_raw = nm_a.group(1)
            nm_n = nize_arm_alloc(nm_raw, '簇5/单臂')
            nize_elem_cnt += 1
            nize_raw_b += len(decode_raw(nm_raw).encode('utf-8'))
            nm_repl = ('DualGate::ThisYear => evtx(%d)/* 原文：%s */.to_string(),'
                       % (nm_n, nm_raw))
            out_edits.setdefault('charpanel.rs', []).append(
                (nm_a.start(), nm_a.end(), nm_repl))
            nize_report.append('DualGate::ThisYear 单臂: N=%d（%dB raw，evtx 后 .to_string() 位不变）'
                               % (nm_n, len(decode_raw(nm_raw).encode('utf-8'))))
    # ---- 子域 B：const 串表 N 化批（charpanel/gallery/draw）----
    if nize_scope in ('all', 'tables'):
        # 先单串 const（表内 ident 引用依赖其 N）
        for (nc1_name, nc1_file, nc1_nname) in NIZE_CONSTS:
            if nc1_file not in FTXT:
                FTXT[nc1_file] = (SRC_DIR / nc1_file).read_text(encoding='utf-8')
            ntext1 = FTXT[nc1_file]
            nreg3 = nize_const_item(ntext1, nc1_name)
            if nreg3 is None:
                if ('const %s' % nc1_nname) in ntext1:
                    nize_report.append('%s: 已换 N 常量（幂等跳过）' % nc1_name)
                    continue
                raise SystemExit('nize: %s 未找到且 %s 缺失——人工核对'
                                 % (nc1_name, nc1_nname))
            nitem1 = ntext1[nreg3[0]:nreg3[1]]
            nlits1 = LIT.findall(nitem1)
            assert len(nlits1) == 1, '%s 单串 const 解析 %d 条 != 1' % (nc1_name, len(nlits1))
            nraw5 = nlits1[0]
            nval5 = decode_raw(nraw5)
            assert nval5 is not None and encode_val(nval5) == nraw5, \
                '%s 往返失败: %r' % (nc1_name, nraw5[:20])
            assert '*/' not in nraw5 and '/*' not in nraw5, '%s 含注释界' % nc1_name
            nn1 = alloc(nraw5)
            nize_const_n[nc1_name] = nn1
            nize_elem_cnt += 1
            nize_raw_b += len(nval5.encode('utf-8'))
            nrepl5 = ('// v1.16.33 单1 NIZE 通道：%s 收编 textpack N=%d（引用位\n'
                      '// evtx(%s as usize)，访问器/绘制位由施工方手改）\n'
                      % (nc1_name, nn1, nc1_nname)
                      + 'const %s: u16 = %d;' % (nc1_nname, nn1))
            out_edits.setdefault(nc1_file, []).append((nreg3[0], nreg3[1], nrepl5))
            nize_report.append('%s: N=%d（%dB raw）→ %s'
                               % (nc1_name, nn1, len(nval5.encode('utf-8')), nc1_nname))
        # 再串数组 const（元素=字面量+白名单 ident，源序）
        ntok1 = re.compile(r'"((?:[^"\\]|\\.)*)"|([A-Z_][A-Z0-9_]{1,})')
        for (nt1_name, nt1_file, nt1_nname, nt1_expect, nt1_refs) in NIZE_TABLES:
            if nt1_file not in FTXT:
                FTXT[nt1_file] = (SRC_DIR / nt1_file).read_text(encoding='utf-8')
            ntext2 = FTXT[nt1_file]
            nreg4 = nize_const_item(ntext2, nt1_name)
            if nreg4 is None:
                if ('const %s' % nt1_nname) in ntext2:
                    nize_report.append('%s: 已换 N 表（幂等跳过）' % nt1_name)
                    continue
                raise SystemExit('nize: %s 未找到且 %s 缺失——人工核对'
                                 % (nt1_name, nt1_nname))
            nitem2 = ntext2[nreg4[0]:nreg4[1]]
            # 注释掩码（保位）——表体内注释行常引大写表名（PREWARM_STRS 体内
            # 「COMPANION_NAMES/NPC_NAMES/NPC_TIER_NAMES」实证），不掩会虚增
            # 元素计数；字符串字面量原样保留（元素值来源）
            nmask2 = list(nitem2)
            nk4 = 0
            while nk4 < len(nitem2):
                nc4 = nitem2[nk4]
                if nc4 == '"':
                    nk4 = skip_str(nitem2, nk4, '"')
                    continue
                if nc4 == '/' and nitem2[nk4:nk4 + 2] == '//':
                    ne4 = nitem2.find('\n', nk4)
                    if ne4 == -1:
                        ne4 = len(nitem2)
                    for nfill4 in range(nk4, ne4):
                        nmask2[nfill4] = ' '
                    nk4 = ne4
                    continue
                if nc4 == '/' and nitem2[nk4:nk4 + 2] == '/*':
                    ne4 = nitem2.find('*/', nk4 + 2)
                    if ne4 == -1:
                        break
                    for nfill4 in range(nk4, ne4 + 2):
                        nmask2[nfill4] = ' '
                    nk4 = ne4 + 2
                    continue
                nk4 += 1
            nclean3 = ''.join(nmask2)
            neq1 = nclean3.index('=')
            nelems1 = []
            for nto1 in ntok1.finditer(nclean3, neq1):
                if nto1.group(1) is not None:
                    nelems1.append(('lit', nto1.group(1)))
                else:
                    nelems1.append(('ident', nto1.group(2)))
            assert len(nelems1) == nt1_expect, \
                '%s 元素 %d != 期望 %d（防半收编，nize_probe 口径）' \
                % (nt1_name, len(nelems1), nt1_expect)
            nns1 = []
            for nkind1, nraw6 in nelems1:
                if nkind1 == 'lit':
                    nval6 = decode_raw(nraw6)
                    assert nval6 is not None and encode_val(nval6) == nraw6, \
                        '%s 往返失败: %r' % (nt1_name, nraw6[:20])
                    assert '*/' not in nraw6 and '/*' not in nraw6, '%s 含注释界' % nt1_name
                    nn2 = alloc(nraw6)
                    nize_raw_b += len(nval6.encode('utf-8'))
                else:
                    assert nraw6 in nt1_refs, \
                        '%s 裸 ident 引用 %s 不在白名单 %s' % (nt1_name, nraw6, nt1_refs)
                    assert nraw6 in nize_const_n, \
                        '%s ident %s 的 N 常量未先经 NIZE_CONSTS 换出' % (nt1_name, nraw6)
                    nn2 = nize_const_n[nraw6]
                nns1.append(nn2)
                nize_elem_cnt += 1
            nrowtxt2 = []
            for nrs2 in range(0, len(nns1), 10):
                nchunk2 = nns1[nrs2:nrs2 + 10]
                nrowtxt2.append('    ' + ', '.join(str(x) for x in nchunk2) + ',')
            nrepl6 = ('// v1.16.33 单1 NIZE 通道：%s 收编 textpack（%d 元素，同值共享\n'
                      '// N——引用位 evtx(%s[k] as usize)，访问器/循环由施工方手改）\n'
                      % (nt1_name, len(nns1), nt1_nname)
                      + 'const %s: [u16; %d] = [\n' % (nt1_nname, len(nns1))
                      + '\n'.join(nrowtxt2) + '\n];')
            out_edits.setdefault(nt1_file, []).append((nreg4[0], nreg4[1], nrepl6))
            nize_report.append('%s: %d 元素 → %s（首 N=%d，去重后 %d 行）'
                               % (nt1_name, len(nns1), nt1_nname, nns1[0],
                                  len(set(nns1))))
if do_nize:
    print('---- nize report ----')
    for nline2 in nize_report:
        print('  ' + nline2)
    print('nize: %d elements, %d B raw moved (dedup shared rows not double-counted in blob)'
          % (nize_elem_cnt, nize_raw_b))

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
# 表——每 K32=128 条一个绝对锚 TXT_ANCH（offs[k*128]），TXT_DELTA[i]=offs[i]−组首。
# v1.16.32 单1 组距 32→128（纯算术等价：off(i)=ANCH[i>>7]+DELTA[i] 逐值不变，
# data.rs txt_off 同批换 i>>7；组距上限=128×行宽 u16 域 assert 兜底）。
# 运行期 evtx 解码 = ANCH[i>>5] + DELTA[i]，O(1) 纯算术零分配（热路径等价）；
# 组内 32 条串长和 ≤64KB 恒成立（单条 <200B×32），u16 域 assert 固化。
# v1.16.32 单1：32→128 粗化（锚 rdata −4B×(N/32−N/128)；u16 delta 域按组距
# 128×行宽 assert 固化，现最大组距 ~9KB « 65535）
K32 = 128
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
for _ms, _cl, raw, vb, n, _a, _kinds in tmpl_items:
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
# v1.16.34 刀α：tmpl_items 7 元组（+kinds）；精度站单列统计
replaced_bytes += sum(len(vb) for _, _, _, vb, _, _a, _k in tmpl_items)
tmpl_bytes = sum(len(vb) for _, _, _, vb, _, _a, _k in tmpl_items)
prec_sites = sum(1 for _ms, _cl, _r, _vb, _n, _a, kinds in tmpl_items
                 if any(kd is not None for kd in kinds))
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
# v1.16.31 段六基址（首轮 emit 取 manifest 手分值；此后回读 .inc 幂等）
if name_base is not None:
    inc += emit_base('TXT_NAME6_BASE',
                     inc_bases.get('TXT_NAME6_BASE', NAME6_BASE_MANIFEST))
if brief_base is not None:
    inc += emit_base('TXT_BRIEF6_BASE',
                     inc_bases.get('TXT_BRIEF6_BASE', BRIEF6_BASE_MANIFEST))
# v1.16.31 段七基址（首轮 emit 取 manifest 手分值；此后回读 .inc 幂等）
if name_base is not None:
    inc += emit_base('TXT_NAME7_BASE',
                     inc_bases.get('TXT_NAME7_BASE', NAME7_BASE_MANIFEST))
if brief_base is not None:
    inc += emit_base('TXT_BRIEF7_BASE',
                     inc_bases.get('TXT_BRIEF7_BASE', BRIEF7_BASE_MANIFEST))
# v1.16.32 单3 段八基址（首轮 emit 取 manifest 手分值；此后回读 .inc 幂等）
if name_base is not None:
    inc += emit_base('TXT_NAME8_BASE',
                     inc_bases.get('TXT_NAME8_BASE', NAME8_BASE_MANIFEST))
if brief_base is not None:
    inc += emit_base('TXT_BRIEF8_BASE',
                     inc_bases.get('TXT_BRIEF8_BASE', BRIEF8_BASE_MANIFEST))
# v1.16.33 单3 段九基址（首轮 emit 取 manifest 手分值；此后回读 .inc 幂等）
if name_base is not None:
    inc += emit_base('TXT_NAME9_BASE',
                     inc_bases.get('TXT_NAME9_BASE', NAME9_BASE_MANIFEST))
if brief_base is not None:
    inc += emit_base('TXT_BRIEF9_BASE',
                     inc_bases.get('TXT_BRIEF9_BASE', BRIEF9_BASE_MANIFEST))
# v1.16.34 单3 段十基址（同段九款：首轮 emit manifest 手分、此后回读 .inc 幂等）
if name_base is not None:
    inc += emit_base('TXT_NAME10_BASE',
                     inc_bases.get('TXT_NAME10_BASE', NAME10_BASE_MANIFEST))
if brief_base is not None:
    inc += emit_base('TXT_BRIEF10_BASE',
                     inc_bases.get('TXT_BRIEF10_BASE', BRIEF10_BASE_MANIFEST))
# v1.16.35 单3 段十一基址（同段十款：首轮 emit manifest 手分、此后回读 .inc 幂等）
if name_base is not None:
    inc += emit_base('TXT_NAME11_BASE',
                     inc_bases.get('TXT_NAME11_BASE', NAME11_BASE_MANIFEST))
if brief_base is not None:
    inc += emit_base('TXT_BRIEF11_BASE',
                     inc_bases.get('TXT_BRIEF11_BASE', BRIEF11_BASE_MANIFEST))
# v1.16.36 单3 段十二基址（同段十一款：首轮 emit manifest 手分、此后回读 .inc 幂等）
if name_base is not None:
    inc += emit_base('TXT_NAME12_BASE',
                     inc_bases.get('TXT_NAME12_BASE', NAME12_BASE_MANIFEST))
if brief_base is not None:
    inc += emit_base('TXT_BRIEF12_BASE',
                     inc_bases.get('TXT_BRIEF12_BASE', BRIEF12_BASE_MANIFEST))
# v1.16.37 单2 段十三基址（同段十二款：首轮 emit manifest 手分、此后回读 .inc 幂等）
if name_base is not None:
    inc += emit_base('TXT_NAME13_BASE',
                     inc_bases.get('TXT_NAME13_BASE', NAME13_BASE_MANIFEST))
if brief_base is not None:
    inc += emit_base('TXT_BRIEF13_BASE',
                     inc_bases.get('TXT_BRIEF13_BASE', BRIEF13_BASE_MANIFEST))
# v1.16.38 单2 段十四基址（同段十三款：首轮 emit manifest 手分、此后回读 .inc 幂等）
if name_base is not None:
    inc += emit_base('TXT_NAME14_BASE',
                     inc_bases.get('TXT_NAME14_BASE', NAME14_BASE_MANIFEST))
if brief_base is not None:
    inc += emit_base('TXT_BRIEF14_BASE',
                     inc_bases.get('TXT_BRIEF14_BASE', BRIEF14_BASE_MANIFEST))
# v1.16.40 单3 段十五基址（同段十四款：首轮 emit manifest 手分、此后回读 .inc 幂等）
if name_base is not None:
    inc += emit_base('TXT_NAME15_BASE',
                     inc_bases.get('TXT_NAME15_BASE', NAME15_BASE_MANIFEST))
if brief_base is not None:
    inc += emit_base('TXT_BRIEF15_BASE',
                     inc_bases.get('TXT_BRIEF15_BASE', BRIEF15_BASE_MANIFEST))
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
