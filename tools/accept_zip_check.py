# -*- coding: utf-8 -*-
"""终局验收：taptap_verify.zip 结构 + 包内 index.html 与 web/index.html 一致性 + 插件/缓存参数核验"""
import zipfile, hashlib, io, re, sys

ZIP = r"E:\Rust_Game\Daoism_Game\taptap\taptap_verify.zip"
WEB = r"E:\Rust_Game\Daoism_Game\web\index.html"

z = zipfile.ZipFile(ZIP)
names = z.namelist()
print("== namelist (%d entries) ==" % len(names))
for n in names:
    info = z.getinfo(n)
    print("  %-50s %8d bytes" % (n, info.file_size))

# 根层结构断言：根层只允许 taptap_h5/ 目录
roots = sorted(set(n.split("/")[0] + ("/" if "/" in n else "") for n in names))
print("== root entries ==", roots)

tri = ["taptap_h5/index.html", "taptap_h5/mq_js_bundle.js", "taptap_h5/xiuxian.wasm"]
for t in tri:
    print("TRI %-30s %s" % (t, "OK" if t in names else "MISSING"))

# 包内 index.html vs web/index.html
in_zip = z.read("taptap_h5/index.html")
web = open(WEB, "rb").read()
h1 = hashlib.md5(in_zip).hexdigest()
h2 = hashlib.md5(web).hexdigest()
print("== index.html md5 ==")
print("  zip :", h1, len(in_zip), "bytes")
print("  web :", h2, len(web), "bytes")
print("  identical:", h1 == h2)

text = in_zip.decode("utf-8")

# 缓存穿透参数
print("== cache-busting ?v= ==")
for m in re.finditer(r'[A-Za-z0-9_.]+\?v=([^"\'\s]+)', text):
    print("  ", m.group(0))

# 存档插件三函数
print("== storage plugin functions ==")
for fn in ["xx_save_len", "xx_save_read", "xx_save_write", "miniquad_add_plugin", "xx_storage"]:
    print("  %-22s %s" % (fn, "FOUND" if fn in text else "MISSING"))

# UTF-8 字节协议（TextEncoder/TextDecoder）
for kw in ["TextEncoder", "TextDecoder", "localStorage", "xiuxian_save"]:
    print("  %-22s %s" % (kw, "FOUND" if kw in text else "MISSING"))

# GAME_VER
m = re.search(r'GAME_VER\s*=\s*"([^"]+)"', text)
print("== GAME_VER ==", m.group(1) if m else "NOT FOUND")

# bundle 与 wasm 是否也在包里逐一比对 md5
import os
for rel, disk in [("taptap_h5/mq_js_bundle.js", r"E:\Rust_Game\Daoism_Game\web\mq_js_bundle.js"),
                  ("taptap_h5/xiuxian.wasm", r"E:\Rust_Game\Daoism_Game\web\xiuxian.wasm")]:
    a = hashlib.md5(z.read(rel)).hexdigest()
    b = hashlib.md5(open(disk, "rb").read()).hexdigest()
    print("MD5 %-26s zip=%s disk=%s same=%s" % (os.path.basename(disk), a[:12], b[:12], a == b))
z.close()
print("DONE")
