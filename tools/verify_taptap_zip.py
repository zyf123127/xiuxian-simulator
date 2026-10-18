# -*- coding: utf-8 -*-
"""核验 taptap_verify.zip 结构与三件套（TapTap 官方要求根层一层 taptap_h5/）。"""
import sys
import zipfile

ZIP = "taptap/taptap_verify.zip"
NEED = ["taptap_h5/index.html", "taptap_h5/mq_js_bundle.js", "taptap_h5/xiuxian.wasm"]

with zipfile.ZipFile(ZIP) as z:
    names = z.namelist()
    bad = [n for n in names if "\\" in n]
    print("namelist:")
    for n in names:
        print("  ", n, z.getinfo(n).file_size, "bytes")
    missing = [n for n in NEED if n not in names]
    assert not bad, f"条目含反斜杠分隔符: {bad}"
    assert not missing, f"缺条目: {missing}"
    assert len(names) == 3, f"条目数应为恰 3 条，实为 {len(names)}: {names}"
    extra = [n for n in names if not n.startswith("taptap_h5/")]
    assert not extra, f"根层多出 taptap_h5/ 之外的条目: {extra}"
    wasm = z.read("taptap_h5/xiuxian.wasm")
    idx = z.read("taptap_h5/index.html").decode("utf-8")
assert b"xx_storage" in wasm, "wasm 内未见 xx_storage 导出符号"
for token in ('GAME_VER = "1.8.6"', "mq_js_bundle.js?v=1.8.6", "miniquad_add_plugin", "xx_storage"):
    assert token in idx, f"包内 index.html 缺: {token}"
print("OK: 恰 3 条目、/ 分隔、根层仅一层 taptap_h5/；wasm 含 xx_storage；index.html 含 ?v=1.8.6 与存档插件")
sys.exit(0)
