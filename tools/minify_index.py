# -*- coding: utf-8 -*-
"""v1.16.8 加载小刀①：web/index.html 构建期最小化（部署件瘦身，源文件不动）。
v1.16.9 加载刀B：mq_js_bundle.js 构建期内联进 index.html（去掉外链同步脚本 =
省 1 个串行 RTT，冷启 ~100-400ms），并加反引号守卫。

只做三类可证明安全的变换（行结构保持，不做任何跨行拼接）：
1. 剥 HTML 注释（<!-- -->，含跨行；仅 <script> 块外——脚本内出现 <!-- 属字符串，
   不动）。本文件 4 处注释均经 grep 实测在 script 外。
2. 剥整行 // 注释（仅 <script> 块内；行首缩进后以 // 起始才算——不动行尾注释，
   不碰字符串/正则里的 //。全文件 // 行经实测全部位于 script 内）。
3. 剥行首缩进 + 丢空行（HTML/JS 均对行首空白不敏感；缩进剥离不改任何语义——
   反引号守卫行除外，见下）。

v1.16.9 反引号守卫：<script> 段内含活反引号的行、以及处于模板字符串态中的行，
跳过 lstrip 原样保留——跨行模板字符串的行首缩进属于字符串内容，剥缩进会静默
改写字符串值（mq_js_bundle 内联后本文件存在 4 个同行闭合反引号；守卫防未来
有人写跨行模板时被本脚本无声破坏）。

v1.16.9 bundle 内联（刀B）：源 index 的 `<script src="mq_js_bundle.js?v=...">`
行在最小化前替换为 `<script>` + bundle 原文 + `</script>`（bundle 内容逐字节
原样不进最小化变换，语义零风险）。前置断言 bundle 不含 "</script>"（含则中止
构建回退外链）。顺序保持：bundle 先于 wasm 加载内核（原外链行位置不动）。

明确不做：跨行合并、变量改名、字符串内压缩——把风险留在零。
用法: python tools/minify_index.py <src> <dst> [bundle.js]
      （bundle 缺省 = 不内联，保持 v1.16.8 纯最小化行为）
退出码非 0 = 产出自检失败（token 缺失/为空/变大），build_web.bat 据此中止。
"""
import os
import re
import sys
import zlib


def inline_bundle(text: str, bundle_path: str) -> str:
    """刀B：bundle 外链行 → 内联三行（<script> / bundle 原文 / </script>）。

    bundle 内容原样插入（不 lstrip 不删注释——bundle 是第三方压缩胶水，逐字节
    保持唯一安全口径）；内联位置=原外链行，顺序语义不变。
    """
    with open(bundle_path, encoding="utf-8") as f:
        bundle = f.read().rstrip("\n")
    assert "</script>" not in bundle.lower(), (
        "mq_js_bundle.js 含 </script>，字符串内联不可行——本轮回退外链（构建中止，"
        "请人工改用外链契约）"
    )
    out = []
    inlined = False
    for ln in text.split("\n"):
        if '<script src="mq_js_bundle.js' in ln and "</script>" in ln:
            out.append("<script>")
            out.append(bundle)
            out.append("</script>")
            inlined = True
        else:
            out.append(ln)
    assert inlined, "源 index 未找到 mq_js_bundle.js 外链行（契约漂移，中止）"
    return "\n".join(out)


def minify(text: str) -> str:
    out_lines = []
    in_script = False
    in_html_comment = False
    in_template = False  # v1.16.9 反引号守卫：script 段内模板字符串态追踪
    for ln in text.split("\n"):
        if in_html_comment:
            if "-->" in ln:
                in_html_comment = False
            continue  # 注释体整行丢弃
        if "<!--" in ln and not in_script:
            if ln.strip().startswith("<!--") and ln.strip().endswith("-->"):
                continue  # 单行注释整行丢
            # 行中注释（未在本文件出现，防御）：剥掉注释段保留其余
            ln = re.sub(r"<!--.*?-->", "", ln)
            if "<!--" in ln:
                in_html_comment = True
                ln = ln[: ln.index("<!--")]
                if not ln.strip():
                    continue
        if "<script" in ln and "</script>" not in ln:
            in_script = True
        elif "</script>" in ln:
            in_script = False
        t = ln.strip()
        if not t:
            continue  # 空行丢
        if in_script and t.startswith("//"):
            continue  # 整行 JS 注释丢
        # v1.16.9 反引号守卫：script 段内——含反引号的行（开/闭模板或同行闭合）
        # 与模板态中的行一律原样保留（不剥缩进）。模板态按"本行反引号奇偶"翻转；
        # 整行 // 注释已在上方丢弃（注释内反引号不参与态追踪）。
        if in_script and ("`" in ln or in_template):
            if ln.count("`") % 2 == 1:
                in_template = not in_template
            out_lines.append(ln)
            continue
        out_lines.append(ln.lstrip())  # 行首缩进剥（行结构保持）
    return "\n".join(out_lines)


def main() -> int:
    src, dst = sys.argv[1], sys.argv[2]
    bundle = sys.argv[3] if len(sys.argv) > 3 else None
    with open(src, encoding="utf-8") as f:
        text = f.read()
    n_src_raw = len(text.encode("utf-8"))
    if bundle:
        text = inline_bundle(text, bundle)
    mini = minify(text)

    # 产出自检：部署契约 token 必须逐字存活（verify_taptap_zip.py 同款清单；
    # 版本串由 build_web.bat 前置的升版步骤保证，这里做通用存在性断言）。
    # v1.16.9 刀B：外链 token mq_js_bundle.js?v= 随内联消失，改断言 bundle
    # 特征串（首行 "use strict" + miniquad_add_plugin）逐字存活。
    for token in ("GAME_VER", '"use strict"', "miniquad_add_plugin", "xx_storage"):
        assert token in mini, f"minify 丢 token: {token}"
    assert "xiuxian.wasm?v=" in mini, "minify 丢 wasm 缓存穿透参数"
    if bundle:
        assert '<script src="mq_js_bundle.js' not in mini, "内联后仍残留外链行"
    assert mini.strip(), "minify 产出为空"
    assert len(mini.encode("utf-8")) < n_src_raw, "minify 未变小，疑似异常"

    # 每行行数守恒审计：只允许减少行（丢注释/空行），不允许拼行
    n_src, n_dst = text.count("\n"), mini.count("\n")
    assert n_dst <= n_src, f"行数增多（疑似拼行）: {n_src} -> {n_dst}"

    with open(dst, "w", encoding="utf-8", newline="\n") as f:
        f.write(mini)

    raw_a, raw_b = n_src_raw, len(mini.encode("utf-8"))
    wire_a = len(zlib.compress(text.encode("utf-8"), 9))
    wire_b = len(zlib.compress(mini.encode("utf-8"), 9))
    print(
        f"[minify] {os.path.basename(src)} -> {os.path.basename(dst)}"
        + (f" (+bundle {os.path.basename(bundle)} inlined)" if bundle else "")
        + f": raw {raw_a} -> {raw_b} ({raw_a - raw_b} saved), "
        f"deflate {wire_a} -> {wire_b} (wire -{wire_a - wire_b} bytes)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
