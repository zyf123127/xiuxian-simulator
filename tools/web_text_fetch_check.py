# -*- coding: utf-8 -*-
"""web_text_fetch_check.py —— v1.16.37 textpack 外置自验件（纳库）。

textpack.bin fetch 外置大刀的加载链内存级行为断言（零落盘产物，仅临时 harness
落系统临时目录用后即删）：
  1. 预取 URL 带 ?v=（源 index + minify 后双份）；
  2. 成功装载字节回传（node 实跑提取出的预取 IIFE + xx_text_len/xx_text_read
     插件函数：state 0→-1、-1→-2、1→bytes.length、read 三级 memory 回退照抄
     xx_font 语义——源码版与 minify 部署版双份各跑一遍）；
  3. 失败两态可见+重试（fetch 拒绝/HTTP 非 ok → xxTextFail：state=-1 终态 +
     errlog [ERR] + xx_stage 文案 + loading 可点重试；wasm_exports 已就绪时只置
     状态与 errlog 不抢屏——与 xxFontFail 同款收口；Rust 侧 state(96) 两处 +
     xx_state code===96 分支静态核对）；
  4. minify 后 token 存活（xx_text/__xxText/xx_text_len/xx_text_read/
     textpack.bin?v=/[ERR] 行/失败文案×2）；
  5. 三方成对静态核对（Rust main.rs xx_text mod + data.rs cfg 包 TEXTPACK 本体 +
     init_text_from_bytes + 组包三件套 21→22 断言同批）。

用法：PYTHONIOENCODING=utf-8 python tools/web_text_fetch_check.py（仓库根执行；
      依赖 node 在 PATH，node web_mime_retry_check.js 同款依赖）。
"""
import io
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / 'tools'))
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
import minify_index as M

SRC = (ROOT / 'web' / 'index.html').read_text(encoding='utf-8')
BUNDLE = str(ROOT / 'web' / 'mq_js_bundle.js')
MAIN_RS = (ROOT / 'src' / 'main.rs').read_text(encoding='utf-8')
DATA_RS = (ROOT / 'src' / 'data.rs').read_text(encoding='utf-8')

mini = M.minify(M.inline_bundle(SRC, BUNDLE))

# ---------- [1] 预取 URL 带 ?v=（源 + minify 双份） ----------
assert SRC.count('fetch("textpack.bin?v=" + GAME_VER)') == 1, '源 index 文案预取 URL 应恰 1 处'
assert 'fetch("textpack.bin?v=" + GAME_VER)' in mini, 'minify 丢文案预取 URL（?v= 缓存穿透断链）'
print('[1] 预取 URL 带 ?v=：源 ×1 / minify ×1 存活')

# ---------- [2] 行为断言 harness（node 实跑提取出的真实函数） ----------
def extract_plugin(src: str) -> str:
    """提取 xx_text_len/xx_text_read 两函数原文（锚点→首个 };）。"""
    i = src.index('importObject.env.xx_text_len = function')
    j = src.index('};', i)
    k = src.index('importObject.env.xx_text_read = function')
    m = src.index('};', k)
    return src[i:j + 2] + '\n' + src[k:m + 2]


def extract_prefetch(src: str) -> str:
    """提取文案预取 script 块体（__xxText 定义→</script> 前，含 xxTextFail）。"""
    i = src.index('window.__xxText = { state: 0, bytes: null };')
    j = src.index('</script>', i)
    return src[i:j]


HARNESS = r'''
const assert = require('assert');
function stubLoading() {
  return { style: { pointerEvents: "" }, onclick: null };
}
function stubDoc() {
  const loading = stubLoading();
  const errlog = { textContent: "" };
  const els = { loading, errlog };
  return {
    loading,
    errlog,
    _els: els,
    getElementById: function (id) { return els[id] || null; },
  };
}
async function flush() { await new Promise(r => setImmediate(r)); }

(async () => {
  const prefetchSrc = %PREFETCH%;
  const pluginSrc = %PLUGIN%;

  // ---- A 成功装载：fetch ok → state=1 → len/read 字节回传 ----
  // v1.16.37 单5 升版锚：GAME_VER 桩值（与断言同值自洽——harness 不读 index.html
  // 实际版本，传参即桩；升版时与 Cargo.toml/index GAME_VER 同批走）
  {
    const win = { __xxText: null };
    const doc = stubDoc();
    const stages = [];
    let fetchedUrl = "";
    const fetch = (url) => {
      fetchedUrl = url;
      return Promise.resolve({
        ok: true,
        arrayBuffer: () => Promise.resolve(new Uint8Array([0x31, 0x36, 0x2e]).buffer),
      });
    };
    new Function("window", "document", "xx_stage", "location", "fetch", "GAME_VER",
      prefetchSrc)(win, doc, (s) => stages.push(s), undefined, fetch, "1.16.37");
    await flush();
    assert.equal(fetchedUrl, "textpack.bin?v=1.16.37", "预取 URL 未带 GAME_VER 缓存穿透");
    assert.equal(win.__xxText.state, 1, "成功路径 state 应为 1");
    assert.deepEqual(Array.from(win.__xxText.bytes), [0x31, 0x36, 0x2e], "预取字节内容不符");

    const importObject = { env: {} };
    new Function("window", "importObject", pluginSrc)(win, importObject);
    assert.equal(importObject.env.xx_text_len(), 3, "就绪态 len 应报就绪字节数");
    const mem = new ArrayBuffer(8);
    win.wasm_memory = { buffer: mem };
    importObject.env.xx_text_read(0, 3);
    assert.deepEqual(Array.from(new Uint8Array(mem, 0, 3)), [0x31, 0x36, 0x2e],
      "xx_text_read 未把字节写入 wasm 内存（wasm_memory 主路径）");
    // 三级回退：wasm_exports.memory 兜底路径
    const mem2 = new ArrayBuffer(8);
    delete win.wasm_memory;
    win.wasm_exports = { memory: { buffer: mem2 } };
    importObject.env.xx_text_read(1, 3);
    assert.deepEqual(Array.from(new Uint8Array(mem2, 1, 3)), [0x31, 0x36, 0x2e],
      "xx_text_read wasm_exports.memory 回退路径失效");
    console.log("  A 成功装载字节回传（len/read + memory 两级回退）PASS");
  }

  // ---- B 失败态一：fetch 拒绝 → xxTextFail（state=-1 终态 + errlog + 可点重试） ----
  {
    const win = { __xxText: null };
    const doc = stubDoc();
    const stages = [];
    let reloaded = 0;
    const location = { reload: () => { reloaded += 1; } };
    const fetch = () => Promise.reject(new Error("network down"));
    new Function("window", "document", "xx_stage", "location", "fetch", "GAME_VER",
      prefetchSrc)(win, doc, (s) => stages.push(s), location, fetch, "1.16.37");
    await flush();
    assert.equal(win.__xxText.state, -1, "失败终态应一次性置 -1（无重试回 0）");
    assert.ok(doc.errlog.textContent.includes("[ERR] 文案资源加载失败（textpack.bin）"),
      "errlog 未追加 [ERR] 文案资源加载失败行");
    assert.ok(stages.includes("文案未至，点击重试"), "加载层失败文案未透出");
    assert.equal(doc.loading.style.pointerEvents, "auto", "失败后 loading 未转为可点");
    doc.loading.onclick();
    assert.equal(reloaded, 1, "loading 点击未触发 location.reload（重试口失效）");
    console.log("  B 失败态一（fetch 拒绝）可见 + 可点重试 PASS");
  }

  // ---- C 失败态二：HTTP 非 ok（res.ok=false）→ 同款收口 ----
  {
    const win = { __xxText: null };
    const doc = stubDoc();
    const stages = [];
    const fetch = () => Promise.resolve({ ok: false, status: 404, arrayBuffer: () => Promise.resolve(new ArrayBuffer(0)) });
    new Function("window", "document", "xx_stage", "location", "fetch", "GAME_VER",
      prefetchSrc)(win, doc, (s) => stages.push(s), undefined, fetch, "1.16.37");
    await flush();
    assert.equal(win.__xxText.state, -1, "HTTP 非 ok 应走失败终态");
    assert.ok(stages.includes("文案未至，点击重试"), "HTTP 非 ok 失败文案未透出");
    console.log("  C 失败态二（HTTP 非 ok）可见 PASS");
  }

  // ---- D 失败时 wasm_exports 已就绪：不抢屏（仅置态+errlog） ----
  {
    const win = { __xxText: null, wasm_exports: {} };
    const doc = stubDoc();
    const stages = [];
    const fetch = () => Promise.reject(new Error("down"));
    new Function("window", "document", "xx_stage", "location", "fetch", "GAME_VER",
      prefetchSrc)(win, doc, (s) => stages.push(s), undefined, fetch, "1.16.37");
    await flush();
    assert.equal(win.__xxText.state, -1, "wasm 已就绪时失败仍应置终态");
    assert.equal(stages.length, 0, "wasm 已就绪时失败文案不应抢屏（与 xxFontFail 同款）");
    console.log("  D wasm 已就绪失败不抢屏 PASS");
  }

  // ---- E len/read 状态语义：-1 下载中 / 缺对象或 -1 → -2 终态 / 未就绪 read no-op ----
  {
    const importObject = { env: {} };
    const win = {};
    new Function("window", "importObject", pluginSrc)(win, importObject);
    assert.equal(importObject.env.xx_text_len(), -2, "对象缺失应报 -2（与 xx_font_len 同款防误吞）");
    win.__xxText = { state: 0, bytes: null };
    assert.equal(importObject.env.xx_text_len(), -1, "下载中应报 -1");
    win.__xxText = { state: -1, bytes: null };
    assert.equal(importObject.env.xx_text_len(), -2, "失败终态应报 -2");
    win.__xxText = { state: 0, bytes: null };
    const mem = new ArrayBuffer(4);
    win.wasm_memory = { buffer: mem };
    importObject.env.xx_text_read(0, 4);
    assert.ok(Array.from(new Uint8Array(mem)).every(b => b === 0), "未就绪 read 必须 no-op");
    console.log("  E len/read 状态语义（-1/-2/N + no-op read）PASS");
  }
  console.log("NODE BEHAVIOR ALL PASS");
})().catch(e => { console.error("NODE FAIL:", e && e.message); process.exit(1); });
'''


def run_node(src_text: str, tag: str) -> None:
    """对源码版/minify 版各跑一遍行为 harness。"""
    code = (HARNESS
            .replace('%PREFETCH%', repr(extract_prefetch(src_text)))
            .replace('%PLUGIN%', repr(extract_plugin(src_text))))
    tmp = tempfile.NamedTemporaryFile('w', suffix='.js', delete=False, encoding='utf-8')
    try:
        tmp.write(code)
        tmp.close()
        r = subprocess.run(['node', tmp.name], capture_output=True, text=True,
                           encoding='utf-8', errors='replace')
        out = (r.stdout or '') + (r.stderr or '')
        assert r.returncode == 0, f'node 行为断言红（{tag}）:\n{out}'
        assert 'NODE BEHAVIOR ALL PASS' in out, f'node 未跑完（{tag}）:\n{out}'
        for ln in out.splitlines():
            if ln.startswith('  '):
                print(ln, f'[{tag}]')
    finally:
        os.unlink(tmp.name)


run_node(SRC, '源码版')
run_node(mini, 'minify版')
print('[2] 行为断言：预取 URL 带版本 / 成功装载字节回传 / 失败两态可见+重试 —— 源码版与 minify 部署版双份全过')

# ---------- [3] minify 后 token 存活 ----------
for tok in ('xx_text', '__xxText', 'xxTextFail', 'xx_text_len', 'xx_text_read',
            'textpack.bin?v=', '[ERR] 文案资源加载失败（textpack.bin）'):
    assert tok in mini, 'minify 丢 token: %s' % tok
assert mini.count('文案未至，点击重试') == 2, 'minify 后失败文案应成对存活（xxTextFail+state96）'
assert 'xx_state' in mini and 'code === 96' in mini, 'minify 丢 xx_state 96 分支'
print('[3] minify 后 token 存活：插件/预取/errlog/失败文案×2/96 分支全存活')

# ---------- [4] Rust 侧三方成对（红线①②③④静态核对） ----------
assert '#[cfg(not(target_arch = "wasm32"))]\npub static TEXTPACK' in DATA_RS, \
    '红线①：cfg 未直接包住 TEXTPACK static 本体（只 cfg 调用方字节照样进 wasm 数据段）'
assert '#[cfg(target_arch = "wasm32")]\npub fn init_text_from_bytes' in DATA_RS, \
    'data.rs 缺 wasm 专用装载口 init_text_from_bytes'
assert 'blob = EMPTY;' in DATA_RS, 'data.rs 缺 wasm 兜底桩（unpack_text 双分支拆分）'
assert 'fn xx_text_len() -> i32;' in MAIN_RS and 'fn xx_text_read(ptr: *mut u8, len: i32);' in MAIN_RS, \
    'main.rs 缺 xx_text env 导入对声明'
assert MAIN_RS.count('xx_dbg::state(96)') == 2, 'main.rs state(96) 应恰 2 处（len<=0 与 init 失败）'
assert 'data::init_text_from_bytes(&buf)' in MAIN_RS, 'main.rs 未接 init_text_from_bytes'
assert 'mod xx_text' in MAIN_RS and 'mod xx_font' in MAIN_RS
assert MAIN_RS.index('xx_text::len()') < MAIN_RS.index('xx_font::len()'), \
    '红线②：boot 装载顺序必须 textpack 先于 font（evtx(1824) 可达 + OnceLock 占坑防御）'
assert MAIN_RS.index('data::init_text_from_bytes') < MAIN_RS.index('data::init_text()'), \
    '红线②：init_text_from_bytes 必须先于 init_text（OnceLock 已占则 set 永久失效）'
print('[4] Rust 三方成对：cfg 包本体 / 导入对 / state(96)×2 / textpack 先于 font / 先装后 init')

# ---------- [5] 组包三件套同批（自验④静态核对：全量跑在打包阶段） ----------
verify_py = (ROOT / 'tools' / 'verify_taptap_zip.py').read_text(encoding='utf-8')
pack_py = (ROOT / 'tools' / 'pack_taptap.py').read_text(encoding='utf-8')
bat = (ROOT / 'build_web.bat').read_text(encoding='utf-8')
assert 'assert len(names) == 22' in verify_py, 'verify 条目数未改恰 22'
assert '"taptap_h5/textpack.bin"' in verify_py, 'verify NEED 清单缺 textpack 条目'
assert 'assert b"xx_text_len" in wasm' in verify_py and 'assert b"xx_text_read" in wasm' in verify_py, \
    'verify 缺 wasm 内 xx_text 导入名成对断言'
assert '"xx_text"' in verify_py and "assert 'textpack.bin?v=' in idx" in verify_py, \
    'verify 缺 index 内 xx_text token / 文案缓存穿透断言'
assert '"textpack.bin"' in pack_py and '"textpack.bin": TEXTPACK_SRC' in pack_py, \
    'pack FILES/SRC 缺 textpack.bin'
assert 'arcname="taptap_h5/textpack.bin"' in pack_py, 'pack 缺显式 zip 条目'
assert 'assert b"xx_text_len" in packed_wasm' in pack_py, 'pack 缺 wasm 内导入名断言'
assert 'need=[\'index.html\',\'xiuxian.wasm\',\'font.ttf.deflate\',\'textpack.bin\']' in bat, \
    'build_web.bat need 清单缺 textpack.bin'
assert 'copy /y assets\\textpack.bin taptap\\taptap_verify\\taptap_h5\\textpack.bin' in bat \
    and 'copy /y assets\\textpack.bin web\\textpack.bin' in bat, \
    'build_web.bat 缺双口 copy（taptap_h5 部署口 + web 本地调试口）'
print('[5] 组包三件套：verify 恰 22/NEED/成对断言、pack FILES/SRC/zip 条目、bat 双口 copy+need 全同批')

print('ALL PASS')
