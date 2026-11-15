# -*- coding: utf-8 -*-
"""单3 扩池文案预核：cmap 全量核对 + 段长预算 + 题材撞车 grep（只读）。"""
import io
import sys
from pathlib import Path

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
ROOT = Path(__file__).resolve().parent.parent
BS = chr(92)
ESC = {'n': '\n', 't': '\t', 'r': '\r', '"': '"', BS: BS, "'": "'", '0': '\0'}


def dec(raw):
    if BS not in raw:
        return raw
    out = []
    k = 0
    while k < len(raw):
        c = raw[k]
        if c != BS:
            out.append(c)
            k += 1
            continue
        k += 1
        out.append(ESC[raw[k]])
        k += 1
    return ''.join(out)


NAMES = {338: '暮坛对酌', 339: '岁暮并账', 340: '雾里听剑', 341: '砚底星痕',
         342: '灯花并盏', 343: '老炉现纹'}
BRIEFS = {338: '泥封启处，醉话入稿', 339: '岁暮并账，头凑着头',
          340: '大雾锁崖，听剑识意', 341: '砚底凿痕，磨墨添星',
          342: '灯花重糊，并作一盏', 343: '老炉细纹，岁晚难补'}
BODIES = {
    338: '醉道人拍开一坛陈年泥封，说本想留着压箱底——「好酒不等人，等来等去，等的只剩酒了。」杯沿一碰，响亮如初。',
    339: '你与{}把历年的流水账并成总账——药钱、灯油、纸笔，逐项对齐。并到末行，两个人头凑着头，谁也没说话，谁也没叹气。',
    340: '大雾锁山，宁清霜在崖边练剑，剑声穿雾，一声近一声远。她说雾天听剑最真——「看不见形，只听得意。」',
    341: '磨墨时你发觉砚底藏着凿痕，星点相连，像前主人一夜一夜用针点出的星图——那夜起，每磨一次墨，图里便多一颗你认得的星。',
    342: '你与{}把历年攒下的灯花纸摊在案上，一枚一枚对齐重糊——当年各点各的灯，如今并作一盏，纸边都发了黄。',
    343: '丹房的老炉身上现了三道细纹，火舌比从前细了。坊市匠人来看过，摇头说——补得住一年，补不住岁晚。',
}
MSGS = {
    338: ['你陪他把一坛喝到见底。他的醉话你随手记了半页——日子还长，酒也还长。道心+2',
          '你要走他的酒方。他说记它做什么——酒自己记得住。气运+1'],
    339: ['{}把总账塞进函底：往后这本账，记的都是用得上的日子。道心+1',
          '总账末页你们空着没记——账会续，日子也会。气运+1'],
    340: ['你闭眼听了半炷香。雾散时她说：你听见的那一声，是你自己剑的样子。道心+2',
          '你接了半盏雾水洗砚。她说雾水养墨——今日的字，会比平日润。气运+1'],
    341: ['你不再擦砚。几夜过去，星痕走到你写过的那一行，停住了——砚认得你的书。道心+3',
          '你把星图拓在素纸上收好。多年后重翻，拓纸上的星比砚底多出一颗。气运+1'],
    342: ['糊到最后一张，{}说灯花如今不用占卦了——好日子不必看。道心+2',
          '你们留了一枚没糊的，压在砚台下——留给明年落雪的夜。气运+1'],
    343: ['你咬牙砌了新炉。匠人收钱时说，这行做的是火的买卖，也是岁的买卖。灵石-6%',
          '你把裂炉留下温茶。火苗小了，茶反倒比从前稳——器也有暮年，各有各的用处。道心+1'],
}
OPTS = {338: ('陪他慢饮', '要走酒方'), 339: ('并账封册', '留页续记'),
        340: ('闭目听剑', '接雾洗砚'), 341: ('就痕研墨', '拓图收好'),
        342: ('糊到天明', '留一枚不糊'), 343: ('砌新炉', '扉页勘误')}

from fontTools.ttLib import TTFont
cmap = TTFont(str(ROOT / 'assets' / 'font.ttf')).getBestCmap()

allchars = set()
for k, v in NAMES.items():
    allchars.update(v)
for k, v in BRIEFS.items():
    assert len(v) <= 18, '简 %d 超 18 字: %d' % (k, len(v))
    allchars.update(v)
for k, v in BODIES.items():
    n = len(v.replace('{}', '甲'))
    assert n <= 60, '正文 %d 超 60 字: %d' % (k, n)
    allchars.update(v.replace('{}', ''))
for k, lst in MSGS.items():
    for v in lst:
        assert len(v.replace('{}', '甲')) <= 60, 'msg %d 超长' % k
        allchars.update(v.replace('{}', ''))
for lst in OPTS.values():
    for v in lst:
        allchars.update(v)

miss = sorted(c for c in allchars if ord(c) > 127 and ord(c) not in cmap)
print('文案去重非 ASCII 字符 %d 个；cmap 缺字: %s' % (len(allchars), miss if miss else '无（零新字形）'))

KWS = {338: '醉话', 339: '并账', 340: '听剑', 341: '凿痕', 342: '灯花', 343: '细纹'}
for k, kw in KWS.items():
    ok_b = kw in BRIEFS[k]
    ok_a = kw in BODIES[k] or kw in ''.join(MSGS[k]) or kw in ''.join(OPTS[k])
    print('id%d kw「%s」简内=%s 臂内=%s' % (k, kw, ok_b, ok_a))

rows = {}
for line in (ROOT / 'assets' / 'text_source.txt').read_text(encoding='utf-8').splitlines():
    n, _, raw = line.partition('\t')
    rows[int(n)] = dec(raw)
blob = '\n'.join(rows.values())
game_src = (ROOT / 'src' / 'game.rs').read_text(encoding='utf-8')
for kw in KWS.values():
    hits = [n for n, v in rows.items() if kw in v]
    print('撞车查「%s」: 真相源 %d 行 %s / game.rs %d 处'
          % (kw, len(hits), hits[:5], game_src.count(kw)))
