# -*- coding: utf-8 -*-
"""v1.16.8 加载小刀①：web/index.html 构建期最小化（部署件瘦身，源文件不动）。

只做三类可证明安全的变换（行结构保持，不做任何跨行拼接）：
1. 剥 HTML 注释（<!-- -->，含跨行；仅 <script> 块外——脚本内出现 <!-- 属字符串，
   不动）。本文件 4 处注释均经 grep 实测在 script 外。
2. 剥整行 // 注释（仅 <script> 块内；行首缩进后以 // 起始才算——不动行尾注释，
   不碰字符串/正则里的 //。全文件 // 行经实测全部位于 script 内）。
3. 剥行首缩进 + 丢空行（HTML/JS 均对行首空白不敏感；本文件无模板字符串
   backtick 实测=0、无 <pre>/<textarea>，缩进剥离不改任何语义）。

明确不做：跨行合并、变量改名、字符串内压缩——把风险留在零。
用法: python tools/minify_index.py <src> <dst>
退出码非 0 = 产出自检失败（token 缺失/为空/变大），build_web.bat 据此中止。
"""
import os
import re
import sys
import zlib


def minify(text: str) -> str:
    out_lines = []
    in_script = False
    in_html_comment = False
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
        out_lines.append(ln.lstrip())  # 行首缩进剥（行结构保持）
    return "\n".join(out_lines)


def main() -> int:
    src, dst = sys.argv[1], sys.argv[2]
    with open(src, encoding="utf-8") as f:
        text = f.read()
    mini = minify(text)

    # 产出自检：部署契约 token 必须逐字存活（verify_taptap_zip.py 同款清单；
    # 版本串由 build_web.bat 前置的升版步骤保证，这里做通用存在性断言）
    for token in ("GAME_VER", "mq_js_bundle.js?v=", "miniquad_add_plugin", "xx_storage"):
        assert token in mini, f"minify 丢 token: {token}"
    assert "xiuxian.wasm?v=" in mini, "minify 丢 wasm 缓存穿透参数"
    assert mini.strip(), "minify 产出为空"
    assert len(mini) < len(text), "minify 未变小，疑似异常"

    # 每行行数守恒审计：只允许减少行（丢注释/空行），不允许拼行
    n_src, n_dst = text.count("\n"), mini.count("\n")
    assert n_dst <= n_src, f"行数增多（疑似拼行）: {n_src} -> {n_dst}"

    with open(dst, "w", encoding="utf-8", newline="\n") as f:
        f.write(mini)

    raw_a, raw_b = len(text.encode("utf-8")), len(mini.encode("utf-8"))
    wire_a = len(zlib.compress(text.encode("utf-8"), 9))
    wire_b = len(zlib.compress(mini.encode("utf-8"), 9))
    print(
        f"[minify] {os.path.basename(src)} -> {os.path.basename(dst)}: "
        f"raw {raw_a} -> {raw_b} ({raw_a - raw_b} saved), "
        f"deflate {wire_a} -> {wire_b} (wire -{wire_a - wire_b} bytes)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
