#!/usr/bin/env python3
# 施工自检：定位哪个 src/*.rs 的字面量正则 match 捕获了指定字符（quote 配对诊断）。
import glob, os, re, sys

TARGET = sys.argv[1] if len(sys.argv) > 1 else "\u2286"
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "src")
PAT_STR = re.compile(r'"((?:[^"\\]|\\.)*)"')
PAT_CHR = re.compile(r"'([^'\\\n]|\\.)'")

for f in sorted(glob.glob(os.path.join(SRC, "*.rs"))):
    with open(f, encoding="utf-8") as fp:
        src = fp.read()
    for m in PAT_STR.finditer(src):
        if TARGET in m.group(1):
            line_no = src.count("\n", 0, m.start()) + 1
            print(f"{f}:{line_no} STR match len={len(m.group(1))}")
            snippet = m.group(1)[:120].encode("unicode_escape").decode()
            print("  head:", snippet)
    for m in PAT_CHR.finditer(src):
        if TARGET == m.group(1):
            line_no = src.count("\n", 0, m.start()) + 1
            print(f"{f}:{line_no} CHR match: {m.group(1)!r}")
