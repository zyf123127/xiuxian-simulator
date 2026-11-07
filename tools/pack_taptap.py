# -*- coding: utf-8 -*-
"""打包 TapTap H5 提交包（每次发版构建完成后跑一次）。

v1.16.9 契约变更（立绘外置 + bundle 内联）：
输入：
  web/index.html          —— 源（可读，bundle 仍以外链形式存在）
  web/mq_js_bundle.js     —— 内联源（部署件内联进 index.html，不再独立成条目）
  assets/portraits_wasm/  —— 18 张 AI 立绘（640px 量化版，部署件外置为 portraits/）
  target/wasm32-unknown-unknown/release/xiuxian.wasm

产出：
  1. taptap/taptap_verify.zip —— TapTap 上传包：根层恰一层 taptap_h5/、
     条目 taptap_h5/<路径>（/ 分隔）、恰 20 条目（index+wasm+portraits/18）、
     ZIP_DEFLATED compresslevel=9；
  2. taptap/taptap_verify/taptap_h5/ —— 同部署集落盘刷新（供解包目检/比对）；
  3. web/xiuxian.wasm + web/portraits/ —— 本地 `python -m http.server` 起目录即可调真包。

前置：release 构建已完成（本脚本只打包，不构建）。打包前自检版本号
（Cargo.toml ↔ index.html GAME_VER；v1.16.9 起 index 对 bundle 是内联契约，
?v= 外链 token 不再存在于部署件，改由 minify_index.py 的 use-strict 特征断言兜底），
不一致即失败，防止"改了代码忘改版本"带着旧版本号上架。
打包后自检与 tools/verify_taptap_zip.py 同口径。
"""
import re
import shutil
import sys
import zipfile

sys.dont_write_bytecode = True  # import minify_index 不落 __pycache__（仓库卫生）
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

WEB_INDEX = ROOT / "web" / "index.html"
WEB_BUNDLE = ROOT / "web" / "mq_js_bundle.js"
PORTRAITS_SRC = ROOT / "assets" / "portraits_wasm"

# v1.16.9：wasm 源优先取 build_web.bat 的 wasm-opt -Oz 产物（xiuxian.opt.wasm）——
# 与 web/ 部署口径一致（无 binaryen 时落 raw cargo wasm，同 build_web.bat skip 分支）
_OPT_WASM = ROOT / "target" / "wasm32-unknown-unknown" / "release" / "xiuxian.opt.wasm"
_RAW_WASM = ROOT / "target" / "wasm32-unknown-unknown" / "release" / "xiuxian.wasm"
WASM_BUILD = _OPT_WASM if _OPT_WASM.is_file() else _RAW_WASM

ZIP_PATH = ROOT / "taptap" / "taptap_verify.zip"
H5_DIR = ROOT / "taptap" / "taptap_verify" / "taptap_h5"
WEB_WASM = ROOT / "web" / "xiuxian.wasm"

# 条目顺序固定：index.html 在前（TapTap 容器先取入口），wasm 第二，portraits 尾随
FILES = ["index.html", "xiuxian.wasm"]
SRC = {
    "index.html": WEB_INDEX,
    "xiuxian.wasm": WASM_BUILD,
}
PORTRAIT_COUNT = 18


def fail(msg: str) -> "None":
    print(f"FAIL: {msg}")
    sys.exit(1)


def main() -> int:
    # ---- 前置检查：部署集必须齐（缺 wasm = 先跑 release 构建）----
    missing = [str(SRC[n]) for n in FILES if not SRC[n].is_file()]
    if missing:
        fail("缺产物（本脚本不构建，先 cargo build --release --target wasm32-unknown-unknown）: "
             + ", ".join(missing))
    if not WEB_BUNDLE.is_file():
        fail(f"缺内联源 {WEB_BUNDLE}（minify_index.py 刀B 内联需要）")
    portraits = sorted(p.name for p in PORTRAITS_SRC.glob("*.png"))
    if len(portraits) != PORTRAIT_COUNT:
        fail(f"{PORTRAITS_SRC} 应恰 {PORTRAIT_COUNT} 张 PNG，实为 {len(portraits)}: {portraits}")

    # ---- 版本一致性：Cargo.toml ↔ index.html GAME_VER ----
    cargo_ver = re.search(
        r'^version\s*=\s*"([^"]+)"', (ROOT / "Cargo.toml").read_text("utf-8"), re.M
    )
    if not cargo_ver:
        fail("Cargo.toml 未找到 version 字段")
    ver = cargo_ver.group(1)
    idx = WEB_INDEX.read_text("utf-8")
    tokens = (f'GAME_VER = "{ver}"',)
    for tok in tokens:
        if tok not in idx:
            fail(f"版本不一致：Cargo.toml={ver}，index.html 缺 {tok}（改 web/index.html 后重跑）")
    # v1.16.9 立绘外置：LazyTex 拉图 URL 的 ?v= 取 env!(CARGO_PKG_VERSION)，与
    # Cargo.toml 同源编译期定死，无独立 token 可查——此处断言部署集 portraits
    # 源就位（上方）+ wasm 存在，即拉图闭环成立
    print(f"版本一致: {ver}（Cargo.toml / GAME_VER 同）")

    # ---- 1) 部署集刷新到 taptap/taptap_verify/taptap_h5/ ----
    # v1.16.9 刀B：index 条目走 minify_index.py 同款内联+最小化（bundle 原样内联），
    # 与 build_web.bat 部署件逐字节同口径——pack 不再直接组源 index
    sys.path.insert(0, str(ROOT / "tools"))
    import minify_index as mi
    deploy_index = mi.minify(
        mi.inline_bundle(WEB_INDEX.read_text("utf-8"), str(WEB_BUNDLE))
    )
    H5_DIR.mkdir(parents=True, exist_ok=True)
    (H5_DIR / "index.html").write_text(deploy_index, encoding="utf-8", newline="\n")
    shutil.copyfile(SRC["xiuxian.wasm"], H5_DIR / "xiuxian.wasm")
    pdir = H5_DIR / "portraits"
    pdir.mkdir(exist_ok=True)
    for p in PORTRAITS_SRC.glob("*.png"):
        shutil.copyfile(p, pdir / p.name)
    stale = H5_DIR / "mq_js_bundle.js"
    if stale.is_file():
        stale.unlink()  # v1.16.9：内联后 bundle 不再是部署条目，清上一版遗留

    # ---- 2) 打包 zip：arcname "taptap_h5/<相对路径>"，恒 / 分隔，恰 20 条目 ----
    ZIP_PATH.parent.mkdir(parents=True, exist_ok=True)
    if ZIP_PATH.exists():
        ZIP_PATH.unlink()
    with zipfile.ZipFile(ZIP_PATH, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as z:
        z.writestr("taptap_h5/index.html", deploy_index)
        z.write(SRC["xiuxian.wasm"], arcname="taptap_h5/xiuxian.wasm")
        for p in sorted(pdir.iterdir()):
            z.write(p, arcname="taptap_h5/portraits/" + p.name)

    # ---- 3) wasm 复制为 web/xiuxian.wasm（与打包同源：opt 优先）、立绘复制为
    # web/portraits/（本地调试）----
    shutil.copyfile(WASM_BUILD, WEB_WASM)
    web_pdir = ROOT / "web" / "portraits"
    web_pdir.mkdir(exist_ok=True)
    for p in PORTRAITS_SRC.glob("*.png"):
        shutil.copyfile(p, web_pdir / p.name)

    # ---- 打包后自检（与 tools/verify_taptap_zip.py 同口径）----
    expect = ["taptap_h5/" + n for n in FILES] + [
        f"taptap_h5/portraits/{n}" for n in portraits
    ]
    with zipfile.ZipFile(ZIP_PATH) as z:
        names = z.namelist()
        assert names == expect, f"条目异常: {names}"
        assert all("\\" not in n for n in names), "条目含反斜杠"
        assert z.testzip() is None, "CRC 校验损坏"
        packed_idx = z.read("taptap_h5/index.html").decode("utf-8")
        packed_wasm = z.read("taptap_h5/xiuxian.wasm")
    # v1.16.9 刀B：部署件 index 应是内联版（bundle 特征串存活、无外链行）
    for tok in ("GAME_VER", '"use strict"', "miniquad_add_plugin", "xx_storage"):
        assert tok in packed_idx, f"包内 index.html 缺: {tok}"
    assert "xiuxian.wasm?v=" in packed_idx, "包内 index.html 缺 wasm 缓存穿透参数"
    assert '<script src="mq_js_bundle.js' not in packed_idx, "内联后仍残留 bundle 外链行"
    assert b"xx_storage" in packed_wasm, "wasm 内未见 xx_storage 导出符号"

    kb = lambda p: f"{p.stat().st_size / 1024:.0f}KB"
    print("OK 打包完成:")
    print(f"  {ZIP_PATH.relative_to(ROOT)}  {len(expect)} 条目  {kb(ZIP_PATH)}")
    for name in FILES:
        print(f"    taptap_h5/{name}  {kb(SRC[name])}")
    print(f"    taptap_h5/portraits/  {PORTRAIT_COUNT} 张  {kb(pdir)}")
    print(f"  目录刷新: {H5_DIR.relative_to(ROOT)}")
    print(f"  本地调试: {WEB_WASM.relative_to(ROOT)}  {kb(WEB_WASM)} + web/portraits/")
    print("  复核: python tools/verify_taptap_zip.py")
    return 0


if __name__ == "__main__":
    sys.exit(main())
