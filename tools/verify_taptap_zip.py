# -*- coding: utf-8 -*-
"""核验 taptap_verify.zip 结构与部署集（TapTap 官方要求根层一层 taptap_h5/）。

v1.16.9 契约变更（立绘外置 + bundle 内联）：
  - 条目 = taptap_h5/index.html + taptap_h5/xiuxian.wasm + taptap_h5/portraits/ 18 张
    （恰 20 条，名单精确比对——bundle 已内联进 index.html，mq_js_bundle.js 不再是
    独立条目；build_web.bat 的 os.walk 递归组包是唯一组包口，本脚本名单断言防漏网）
  - 逐条目 file_size>0 + z.testzip() 全档 CRC 校验
"""
import sys
import zipfile

ZIP = "taptap/taptap_verify.zip"
PORTRAITS = [
    "portrait_0", "portrait_1", "portrait_2", "portrait_3",
    "npc_elder", "npc_merchant", "npc_senior", "npc_drunk",
    "comp_0", "comp_1", "comp_2", "comp_3", "comp_4",
    "comp_5", "comp_6", "comp_7", "comp_8", "comp_9",
]
NEED = ["taptap_h5/index.html", "taptap_h5/xiuxian.wasm"] + [
    f"taptap_h5/portraits/{n}.png" for n in PORTRAITS
]

with zipfile.ZipFile(ZIP) as z:
    names = z.namelist()
    bad = [n for n in names if "\\" in n]
    print("namelist:")
    for n in names:
        print("  ", n, z.getinfo(n).file_size, "bytes")
    missing = [n for n in NEED if n not in names]
    extra = [n for n in names if n not in NEED]
    # v1.16.9：全档 CRC 校验（testzip 逐条解压验证，坏档在此即红）
    corrupt = z.testzip()
    zero = [n for n in names if z.getinfo(n).file_size == 0]
    wasm = z.read("taptap_h5/xiuxian.wasm")
    idx = z.read("taptap_h5/index.html").decode("utf-8")
assert not bad, f"条目含反斜杠分隔符: {bad}"
assert not missing, f"缺条目: {missing}"
assert not extra, f"多出名单外条目: {extra}"
assert len(names) == 20, f"条目数应为恰 20 条（2 件套+18 立绘），实为 {len(names)}: {names}"
assert corrupt is None, f"CRC 校验损坏条目: {corrupt}"
assert not zero, f"零字节条目: {zero}"
assert not names or all(n.startswith("taptap_h5/") for n in names), "根层混入 taptap_h5/ 之外条目"
assert b"xx_storage" in wasm, "wasm 内未见 xx_storage 导出符号"
for token in ('GAME_VER = "1.16.9"', "miniquad_add_plugin", "xx_storage", '"use strict"'):
    assert token in idx, f"包内 index.html 缺: {token}"
assert 'xiuxian.wasm?v=' in idx, "包内 index.html 缺 wasm 缓存穿透参数"
assert '<script src="mq_js_bundle.js' not in idx, "内联后仍残留 bundle 外链行"
print("OK: 恰 20 条目（index+wasm+portraits/18）、/ 分隔、根层仅一层 taptap_h5/、"
      "CRC 全过、条目非零字节；wasm 含 xx_storage；index.html 含 GAME_VER 1.16.9 与"
      "内联 bundle（miniquad_add_plugin/use strict）与存档插件")
sys.exit(0)
