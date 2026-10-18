# 直接运行 Release exe 的真机测试：截屏菜单 → 模拟点击开始新的人生 → 截屏现实界面
# 证据存 shots/manual_run_*.png（shots/ 已 gitignore）
import ctypes
import shutil
import subprocess
import time
from pathlib import Path

from PIL import ImageGrab

ctypes.windll.user32.SetProcessDPIAware()

EXE = r"target\release\xiuxian.exe"
TITLE = "凡人修仙·轮回模拟器"
OUT = Path("shots")
OUT.mkdir(exist_ok=True)

# 备份玩家存档（测试会开新人生覆盖）
save = Path("xiuxian_save.txt")
bak = Path("xiuxian_save.txt.manual_bak")
had_save = save.exists()
if had_save:
    shutil.copy2(save, bak)


def grab(tag: str) -> None:
    img = ImageGrab.grab()
    img.save(OUT / f"manual_run_{tag}.png")
    print(f"saved manual_run_{tag}.png {img.size}")


def find_hwnd() -> int:
    hwnd = ctypes.windll.user32.FindWindowW(None, TITLE)
    return hwnd


def window_rect(hwnd: int):
    r = ctypes.wintypes.RECT()
    ctypes.windll.user32.GetWindowRect(hwnd, ctypes.byref(r))
    return r


def click_at(x: int, y: int) -> None:
    ctypes.windll.user32.SetCursorPos(x, y)
    time.sleep(0.15)
    ctypes.windll.user32.mouse_event(2, 0, 0, 0, 0)  # LEFTDOWN
    time.sleep(0.08)
    ctypes.windll.user32.mouse_event(4, 0, 0, 0, 0)  # LEFTUP


import ctypes.wintypes  # noqa: E402

proc = subprocess.Popen([EXE], cwd=".")
print("launched pid=", proc.pid)
time.sleep(12)  # 等窗口创建 + 16 张立绘加载 + 音频初始化

hwnd = 0
for _ in range(20):
    hwnd = find_hwnd()
    if hwnd:
        break
    time.sleep(1)
assert hwnd, "未找到游戏窗口"
r = window_rect(hwnd)
print("window rect:", r.left, r.top, r.right, r.bottom)
grab("01_menu")

# 客户区物理尺寸 → 游戏虚拟坐标缩放（high_dpi：客户区 1280×720 逻辑 = 1920×1080 物理）
client_w = r.right - r.left - 16
scale = client_w / 1280.0
print("client scale:", scale)


def to_screen(vx: float, vy: float) -> tuple[int, int]:
    return (int(r.left + 8 + vx * scale), int(r.top + 31 + vy * scale))


# 点击「开始新的人生」（虚拟坐标 470,568）
cx, cy = to_screen(470, 568)
click_at(cx, cy)
time.sleep(3)
grab("02_reality")

# 点击底栏「角色」按钮打开面板（键盘 C 会被中文输入法截获）
cx, cy = to_screen(1032, 670)
click_at(cx, cy)
time.sleep(2)
grab("03_charpanel")

proc.terminate()
time.sleep(2)
if had_save:
    shutil.copy2(bak, save)
    bak.unlink()
print("done, save restored:", had_save)
