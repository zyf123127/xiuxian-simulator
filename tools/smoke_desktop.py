# -*- coding: utf-8 -*-
"""桌面版确定性冒烟门控（工作流/CI 用，等价于 GF_DEBUG=1 timeout 6 exe）。

语义：以 GF_DEBUG=1 启动 release exe，6 秒后仍存活 = 通过（exit 0）；
6 秒内自行退出/崩溃 = 不通过（exit 1，附输出尾部）。shots/ 已 gitignore。
用法：python tools/smoke_desktop.py
"""
import os
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
EXE = ROOT / "target" / "release" / "xiuxian.exe"
SECONDS = 6.0


def main() -> int:
    if not EXE.is_file():
        print(f"[smoke] exe 缺失：{EXE}", file=sys.stderr)
        return 1
    env = dict(os.environ, GF_DEBUG="1")
    proc = subprocess.Popen(
        [str(EXE)],
        cwd=str(ROOT),
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    t0 = time.time()
    try:
        code = proc.wait(timeout=SECONDS)
        out = (proc.stdout.read() if proc.stdout else b"")[-2000:]
        print(f"[smoke] 进程 {SECONDS}s 内自行退出 code={code}\n{out.decode('utf-8', 'replace')}", file=sys.stderr)
        return 1
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait()
        print(f"[smoke] OK：存活 {time.time() - t0:.1f}s 被正常终止（exit 124 语义）")
        return 0


if __name__ == "__main__":
    sys.exit(main())
