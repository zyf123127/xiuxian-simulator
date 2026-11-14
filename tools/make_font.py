#!/usr/bin/env python3
# 扫描 src/*.rs 中实际用到的字符 → 生成 OFL 字体子集 assets/font.ttf
# 注意：文案更新后必须重新运行本脚本，否则缺字
# 加载提速③：口径从「整个源文件」收窄为「字符串字面量 + 字符字面量」——
# 运行时会渲染的文本只可能来自这两类字面量（本工程已核实无 \u{}/\x 转义、
# 无 raw string），注释不再入集（实测注释独占 311 个非 ASCII 字符）。
# 改后必跑全量复查防豆腐块：python .mumu/fontcov_full.py（字面量口径逐字
# 符对源字体 cmap）与 python tools/check_chars.py（历史缺字回归）。
# v1.16.25 扩口径：追加扫描 assets/text_source.txt（textpack 真相源）解码值
# ——v1.16.23 起文案资产化，build_event 臂内长串收编为 evtx(N)/* 原文 */ 注释
# 后不再出现在字面量里；若此时按纯字面量口径重建，会把已收编文案的字从字体
# 里丢掉（实测 225 字 → 运行期 tofu）。运行期渲染面 = 字面量 ∪ textpack，
# 两者都必须入集；裸字面量先过字形检查再收编的跑序纪律不变。
import glob, os, re, sys
from fontTools import subset

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "src")
OUT = os.path.join(ROOT, "assets", "font.ttf")
# 源字体：fontsource NotoSansSC-700 与 unicode-range 切片(105=●,59=渟)合并而成
# （重建见 tools/build_src_font.py），覆盖主切片缺失的符号/生僻字
# （本项目曾缺 ※→①②④●渟）；同源 OFL 1.1 许可
SRC_FONT = os.path.join(ROOT, "fonts", "NotoSansSC-src.ttf")

# 字符串字面量正则与 .mumu/fontcov_full.py 同款（含转义成对）；
# 字符字面量补充 'x' / '\x' 形态（ui.rs 面板预热表即字符字面量数组）
PAT_STR = re.compile(r'"((?:[^"\\]|\\.)*)"')
PAT_CHR = re.compile(r"'([^'\\\n]|\\.)'")

chars = set()
# ASCII 可打印（拉丁字形成本极低，整档保留做防御）
for c in range(0x20, 0x7F):
    chars.add(chr(c))
# 常用中文标点与符号兜底
extras = "，。：；、！？（）《》「」·…—＋×÷％℃。【】〖〗※☆★◇◆□■▲▼"
extras += "一二三四五六七八九十百千万亿兆京零壹贰叁肆伍陆柒捌玖"
extras += "上中下前后左右天地玄黄宇宙洪荒甲乙丙丁戊己庚辛壬癸子丑寅卯辰巳午未申酉戌亥"
extras += "初终大小快慢停常疾极"
chars.update(extras)

for f in glob.glob(os.path.join(SRC, "*.rs")):
    with open(f, encoding="utf-8") as fp:
        src = fp.read()
    for m in PAT_STR.finditer(src):
        chars.update(m.group(1))
    for m in PAT_CHR.finditer(src):
        chars.update(m.group(1))

# v1.16.25 追加：textpack 真相源逐行解码（Rust 源码转义形态→真实文本，与
# tools/pack_text.py decode_raw 同款口径），解码失败的行保守跳过
_ESC = {'n': '\n', 't': '\t', 'r': '\r', '"': '"', '\\': '\\', "'": "'", '0': '\0'}


def _decode_raw(raw):
    if '\\' not in raw:
        return raw
    out = []
    k = 0
    while k < len(raw):
        c = raw[k]
        if c != '\\':
            out.append(c)
            k += 1
            continue
        k += 1
        if k >= len(raw):
            return None
        d = raw[k]
        if d == 'u':
            if raw[k + 1:k + 2] != '{':
                return None
            close = raw.index('}', k + 2)
            try:
                out.append(chr(int(raw[k + 2:close], 16)))
            except ValueError:
                return None
            k = close + 1
        elif d in _ESC:
            out.append(_ESC[d])
            k += 1
        else:
            return None
    return ''.join(out)


_SRC_TXT = os.path.join(ROOT, "assets", "text_source.txt")
if os.path.exists(_SRC_TXT):
    _n_txt = 0
    for _line in open(_SRC_TXT, encoding="utf-8").read().splitlines():
        if not _line.strip():
            continue
        _, _, _raw = _line.partition('\t')
        _val = _decode_raw(_raw)
        if _val:
            chars.update(_val)
            _n_txt += 1
    print(f"textpack rows scanned: {_n_txt}")

text = "".join(sorted(chars))
print(f"unique chars: {len(text)}")

os.makedirs(os.path.dirname(OUT), exist_ok=True)
args = [
    SRC_FONT,
    f"--text={text}",
    "--layout-features=*",
    "--no-hinting",
    "--desubroutinize",
    f"--output-file={OUT}",
]
subset.main(args)
sz = os.path.getsize(OUT)
print(f"written {OUT} ({sz/1024:.0f} KB)")

# v1.16.8 加载小刀③：产 raw-deflate 版字体（assets/font.ttf.deflate）——
# ui.rs init_font include_bytes 此文件 + miniz_oxide inflate 同步解压（boot 首帧
# 一次，几 ms 级，无启动竞态）。font.ttf 仍照常产出供 fontcov_full.py/check_chars.py
# 等工具读 cmap（容器变、内容不变）。raw deflate（zlib wbits=-15，非 gzip 容器）
# 与 miniz_oxide::inflate::decompress_to_vec 对口；mtime 不进 deflate 流，输出确定。
import zlib
defl = zlib.compressobj(9, zlib.DEFLATED, -15)
data = open(OUT, "rb").read()
blob = defl.compress(data) + defl.flush()
OUT_DEFLATE = os.path.join(ROOT, "assets", "font.ttf.deflate")
with open(OUT_DEFLATE, "wb") as f:
    f.write(blob)
# 回环自检：解压必须逐字节还原，否则宁可不发
back = zlib.decompress(blob, -15)
assert back == data, "font deflate roundtrip mismatch"
print(f"written {OUT_DEFLATE} ({len(blob)/1024:.0f} KB, ttf {len(data)} B -> deflate {len(blob)} B)")
