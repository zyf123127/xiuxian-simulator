# -*- coding: utf-8 -*-
"""打包 TapTap H5 提交包（每次发版构建完成后跑一次）。

输入三件套：
  web/index.html
  web/mq_js_bundle.js
  target/wasm32-unknown-unknown/release/xiuxian.wasm

产出：
  1. taptap/taptap_verify.zip —— TapTap 上传包：根层恰一层 taptap_h5/、
     条目名 taptap_h5/<文件名>（/ 分隔）、恰 3 条目、ZIP_DEFLATED compresslevel=9；
  2. taptap/taptap_verify/taptap_h5/ —— 同三份文件落盘刷新（供解包目检/比对）；
  3. web/xiuxian.wasm —— release wasm 复制进 web/，本地 `python -m http.server` 起目录即可调真包。

前置：release 构建已完成（本脚本只打包，不构建）。打包前自检三处版本号
（Cargo.toml 与 index.html 的 GAME_VER / mq_js_bundle.js?v=）逐处一致，不一致即失败，
防止"改了代码忘改版本"带着旧版本号上架。打包后自检与 tools/verify_taptap_zip.py 同口径。
"""
import re
import shutil
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

WEB_INDEX = ROOT / "web" / "index.html"
WEB_BUNDLE = ROOT / "web" / "mq_js_bundle.js"
WASM_BUILD = ROOT / "target" / "wasm32-unknown-unknown" / "release" / "xiuxian.wasm"

ZIP_PATH = ROOT / "taptap" / "taptap_verify.zip"
H5_DIR = ROOT / "taptap" / "taptap_verify" / "taptap_h5"
WEB_WASM = ROOT / "web" / "xiuxian.wasm"

# 条目顺序固定：index.html 在前（TapTap 容器先取入口），wasm 最后
FILES = ["index.html", "mq_js_bundle.js", "xiuxian.wasm"]
SRC = {
    "index.html": WEB_INDEX,
    "mq_js_bundle.js": WEB_BUNDLE,
    "xiuxian.wasm": WASM_BUILD,
}


def fail(msg: str) -> "None":
    print(f"FAIL: {msg}")
    sys.exit(1)


def main() -> int:
    # ---- 前置检查：三件套必须齐（缺 wasm = 先跑 release 构建）----
    missing = [str(SRC[n]) for n in FILES if not SRC[n].is_file()]
    if missing:
        fail("缺产物（本脚本不构建，先 cargo build --release --target wasm32-unknown-unknown）: "
             + ", ".join(missing))

    # ---- 版本一致性：Cargo.toml ↔ index.html GAME_VER ↔ ?v= 查询串 ----
    cargo_ver = re.search(
        r'^version\s*=\s*"([^"]+)"', (ROOT / "Cargo.toml").read_text("utf-8"), re.M
    )
    if not cargo_ver:
        fail("Cargo.toml 未找到 version 字段")
    ver = cargo_ver.group(1)
    idx = WEB_INDEX.read_text("utf-8")
    tokens = (f'GAME_VER = "{ver}"', f"mq_js_bundle.js?v={ver}")
    for tok in tokens:
        if tok not in idx:
            fail(f"版本不一致：Cargo.toml={ver}，index.html 缺 {tok}（改 web/index.html 两处后重跑）")
    print(f"版本一致: {ver}（Cargo.toml / GAME_VER / ?v= 三处同）")

    # ---- 1) 三份文件刷新到 taptap/taptap_verify/taptap_h5/ ----
    H5_DIR.mkdir(parents=True, exist_ok=True)
    for name in FILES:
        shutil.copyfile(SRC[name], H5_DIR / name)

    # ---- 2) 打包 zip：arcname 固定 "taptap_h5/<名>"，恒 / 分隔，恰 3 条目 ----
    ZIP_PATH.parent.mkdir(parents=True, exist_ok=True)
    if ZIP_PATH.exists():
        ZIP_PATH.unlink()
    with zipfile.ZipFile(ZIP_PATH, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as z:
        for name in FILES:
            z.write(SRC[name], arcname="taptap_h5/" + name)

    # ---- 3) wasm 复制为 web/xiuxian.wasm 供本地调试 ----
    shutil.copyfile(WASM_BUILD, WEB_WASM)

    # ---- 打包后自检（与 tools/verify_taptap_zip.py 同口径）----
    with zipfile.ZipFile(ZIP_PATH) as z:
        names = z.namelist()
        assert names == ["taptap_h5/" + n for n in FILES], f"条目异常: {names}"
        assert all("\\" not in n for n in names), "条目含反斜杠"
        packed_idx = z.read("taptap_h5/index.html").decode("utf-8")
        packed_wasm = z.read("taptap_h5/xiuxian.wasm")
    for tok in tokens + ("miniquad_add_plugin", "xx_storage"):
        assert tok in packed_idx, f"包内 index.html 缺: {tok}"
    assert b"xx_storage" in packed_wasm, "wasm 内未见 xx_storage 导出符号"

    kb = lambda p: f"{p.stat().st_size / 1024:.0f}KB"
    print("OK 打包完成:")
    print(f"  {ZIP_PATH.relative_to(ROOT)}  3 条目  {kb(ZIP_PATH)}")
    for name in FILES:
        print(f"    taptap_h5/{name}  {kb(SRC[name])}")
    print(f"  目录刷新: {H5_DIR.relative_to(ROOT)}")
    print(f"  本地调试: {WEB_WASM.relative_to(ROOT)}  {kb(WEB_WASM)}")
    print("  复核: python tools/verify_taptap_zip.py")
    return 0


if __name__ == "__main__":
    sys.exit(main())
