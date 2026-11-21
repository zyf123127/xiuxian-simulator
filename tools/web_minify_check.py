# -*- coding: utf-8 -*-
"""web_minify_check.py —— v1.16.35 单2 加载链自验件纳库（复审 low 顺手收编）。
minify/pack 打包语义内存断言（零落盘，照勘察配方 7）：复刻 tools/minify_index.py
main() 的全部断言 + 单2 新增面存活（MIME 两态兜底/阶段文案）+ 看门狗文案未动 +
依赖序/?v= 机制 + 字形核对。升版/加载链改动后重跑即全量回归。
用法：PYTHONIOENCODING=utf-8 python tools/web_minify_check.py（仓库根执行）"""
import sys, os, io, re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / 'tools'))
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
import minify_index as M

SRC = str(ROOT / 'web' / 'index.html')
BUNDLE = str(ROOT / 'web' / 'mq_js_bundle.js')

src = open(SRC, encoding='utf-8').read()
n_raw = len(src.encode('utf-8'))
inl = M.inline_bundle(src, BUNDLE)
mini = M.minify(inl)

# ---- main() 全断言内存复刻 ----
for token in ('GAME_VER', '"use strict"', 'miniquad_add_plugin', 'xx_storage', 'xx_font'):
    assert token in mini, 'minify 丢 token: %s' % token
assert 'xiuxian.wasm?v=' in mini, '丢 wasm 缓存穿透参数'
assert 'font.ttf.deflate?v=' in mini, '丢字体缓存穿透参数'
assert '<script src="mq_js_bundle.js' not in mini, '内联后仍残留外链行'
assert mini.strip(), 'minify 产出为空'
assert len(mini.encode('utf-8')) < n_raw, 'minify 未变小'
n_src_l, n_dst_l = inl.count('\n'), mini.count('\n')
assert n_dst_l <= n_src_l, '行数增多（疑似拼行）: %d -> %d' % (n_src_l, n_dst_l)
print('[1] main() 断言全过（内存）: raw %d -> %d (-%d), 行 %d -> %d' % (
    n_raw, len(mini.encode('utf-8')), n_raw - len(mini.encode('utf-8')), n_src_l, n_dst_l))

# ---- 项1/项2 新面存活 ----
for s in ['__xxWasmUrl', 'res.arrayBuffer', 'refetch', 'reuse',
          '摄取仙机 ', '灵机汇聚，推演轮回…', '炼化仙机…', '凝神开卷，布置洞府…',
          '前尘入卷，山河重展…', '万事俱备…', '推演开始…', '字体未至，点击重试',
          '仙机已至，炼化中…', '已纳仙机 ', '旧地重游，片刻即至…', '凡人模拟 · 灵机汇聚…']:
    assert s in mini, 'minify 丢新面: %s' % s
assert mini.count('字体未至，点击重试') == 2, '字体失败文案成对存活数≠2'
n_url = src.count('window.__xxWasmUrl')
assert n_url == 3, '__xxWasmUrl 应为 3 处（HTML 注释说明+捕获+消费），实得 %d' % n_url
print('[2] 项1 两态兜底 + 项2 文案新面全部存活（字体失败文案成对×2，__xxWasmUrl 捕获+消费×2）')

# ---- 看门狗文案逐字未动 ----
for s in ['游戏包较大，首次加载约需 10~40 秒，请稍候…（下次进入会快很多）',
          '仍在编译游戏引擎（低端设备会慢一些），请继续等待…',
          '加载较慢，可点击重试',
          '加载受阻，点击此处重试']:
    assert s in src and s in mini, '看门狗文案被误改: %s' % s
print('[3] 12s/30s/45s/D1D2 看门狗文案逐字未动')

# ---- 结构面（红线2）：link / inline script / 行首 regex 存活 ----
n_link = len(re.findall(r'<link', mini))
n_script = mini.count('<script>')
pats = re.findall(r'/\[\?&\][a-z0-9_=]+/', mini)
assert n_link >= 1 and n_script >= 6 and pats, '结构面丢失: link=%d script=%d regex=%s' % (
    n_link, n_script, pats)
print('[4] link×%d / <script>×%d / 行首 regex 存活 %s' % (n_link, n_script, pats))

# ---- 依赖序（红线3）：script 依赖顺序未被重排 ----
src_lines = src.split('\n')
order = [
    ('GAME_VER 定义', lambda l: 'var GAME_VER =' in l),
    ('字体预取', lambda l: 'fetch("font.ttf.deflate?v=" + GAME_VER)' in l),
    ('wasm 头部预取', lambda l: 'window.__xxWasmP = fetch(' in l),
    ('compileStreaming 包装', lambda l: 'var _cs = WebAssembly.compileStreaming;' in l),
    ('bundle 外链行', lambda l: '<script src="mq_js_bundle.js' in l),
    ('xx_storage 注册', lambda l: 'name: "xx_storage"' in l),
    ('xx_font 注册', lambda l: 'name: "xx_font"' in l),
    ('fetch 包装器', lambda l: 'window.fetch = function' in l),
    ('load() 调用', lambda l: 'load("xiuxian.wasm?v=" + GAME_VER)' in l),
]
idx = []
for name, pred in order:
    hit = -1
    for i, ln in enumerate(src_lines):
        if pred(ln):
            hit = i
            break
    assert hit >= 0, '锚缺失: %s' % name
    idx.append(hit)
assert idx == sorted(idx), '依赖序被破坏: %s' % list(zip([n for n, _ in order], idx))
print('[5] 依赖序完好: ' + ' -> '.join('%s@%d' % (n, i) for (n, _), i in zip(order, idx)))

# ---- GAME_VER / ?v= 机制（红线4）----
assert src.count('var GAME_VER') == 1, 'GAME_VER 定义数异常'
assert 'mq_js_bundle.js?v=' in src, 'bundle ?v= 外链行丢失'
vv = re.findall(r'\?v=" \+ GAME_VER|\?v=[0-9.]+"', src)
assert len(vv) == 4, '?v= 站点数 %d（应为 4：字体/wasm 预取/bundle/load）' % len(vv)
print('[6] GAME_VER 唯一源 ×1，?v= 站点 ×4（字体预取/wasm 预取/bundle 外链/load）')

# ---- 字形核对（铁律④参考口径：HTML 层走系统字体，无子集约束；核对仅供参考）----
newc = set('灵机汇聚推演轮回摄取仙旧地重游片刻即至炼化前尘入卷山河重展万事俱备凝神开布置洞府纳已至字体未')
have = set((ROOT / 'assets' / 'text_source.txt').read_text(encoding='utf-8'))
missing = sorted(c for c in newc if c not in have)
print('[7] 新文案独立汉字 %d 个；不在游戏字体集(text_source)者: %s（HTML 层系统字体渲染，不构成字形约束）'
      % (len(newc), ''.join(missing) if missing else '无'))
print('ALL PASS')
