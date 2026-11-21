# -*- coding: utf-8 -*-
"""全量字体覆盖检查：扫描 src/*.rs 渲染面用字，对照 assets/font.ttf 的 cmap。只读检查。

v1.16.39 口径修正：剔除 #[cfg(test)] 区（测试断言消息/夹具字符串不入游戏渲染面——
实证 game.rs:17944"终战基础面应 3 钮"之"础"、game.rs:24361 钉测消息之"腔"、
save.rs:1091/1115 测试夹具之"妄"全部位于 cfg(test) 内，旧口径误报 3 缺字；
字体重生成只为测试消息扩 3 字形属浪费，故收窄语料而非扩字体——沿"删字面量不删字体"
反向纪律：字体只增不减，但增须为真实渲染面）。跳过规则：遇行首 #[cfg(test)] 起，
至行首顶格 `}` 止（仓内 rustfmt 惯例：顶层项闭括号顶格；全部 cfg(test) 均为
`#[cfg(test)] mod X {` 统一形态，grep 2026-11-21 实测）。"""
import glob
import re
from fontTools.ttLib import TTFont

chars = set()
for p in glob.glob("src/*.rs"):
    with open(p, encoding="utf-8") as f:
        in_test = False  # v1.16.39：位于 #[cfg(test)] 区内则跳过语料采集
        for line in f:
            stripped = line.strip()
            if stripped.startswith("#[cfg(test)]"):
                in_test = True  # 仓内统一形态：下一行即 mod X {，闭括号顶格
                continue
            if in_test:
                if line.startswith("}"):
                    in_test = False  # 顶层项闭括号，测试区结束
                continue
            line = re.sub(r"//.*", "", line)  # 去注释（注释用字不入游戏）
            for ch in line:
                if ord(ch) > 0x2E80:  # CJK 及以上，忽略 ASCII/标点
                    chars.add(ch)

cmap = TTFont("assets/font.ttf").getBestCmap()
missing = sorted(c for c in chars if ord(c) not in cmap)
print(f"src/*.rs 非注释 CJK 用字 {len(chars)} 种；assets/font.ttf 缺 {len(missing)} 个")
if missing:
    print("missing:", "".join(missing))
