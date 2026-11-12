#!/usr/bin/env python3
# 施工自检：EVENT_NAMES / EVENT_BRIEF 两表逐行解析（剔除行内注释），对账
# 声明长度 == 实析条数 == EVENTS 行数，且 NAMES 去「·续」后互异。只读。
import re

src = open("src/data.rs", encoding="utf-8").read()
n_events = len(re.findall(r"\((\d+), (-?\d+), (\d+), (true|false)\)", src))

PAT_STR = re.compile(r'"((?:[^"\\]|\\.)*)"')
for name in ["EVENT_NAMES", "EVENT_BRIEF"]:
    m = re.search(
        r"pub const " + name + r": \[&str; (\d+)\] = \[(.*?)\n\];", src, re.S
    )
    decl = int(m.group(1))
    entries = PAT_STR.findall(m.group(2))
    ok = decl == len(entries) == n_events
    print(name, "decl", decl, "parsed", len(entries), "aligned:", ok)
    if name == "EVENT_NAMES":
        seen = set()
        dup = []
        for e in entries:
            base = e.replace("·续", "")
            if base in seen:
                dup.append(base)
            seen.add(base)
        print("NAMES distinct bases:", len(seen), "dup:", dup if dup else "none")
