// 字体加载与文本工具（中文 OFL 子集字体）
use macroquad::prelude::*;
use std::sync::OnceLock;

static FONT: OnceLock<Font> = OnceLock::new();

pub fn init_font() {
    let bytes = include_bytes!("../assets/font.ttf");
    let f = load_ttf_font_from_bytes(bytes).expect("字体加载失败");
    let _ = FONT.set(f);
}

pub fn font() -> &'static Font {
    FONT.get().expect("字体未初始化")
}

// v1.6.0 A 清晰度：字号下限。低于 14px 的中文在 150% 物理栅格下发虚/笔画粘连，
// measure/txt 两处统一夹取保证宽度测量与实际绘制一致（右对齐 txt_r 不漂移）。
pub const MIN_FONT: u16 = 14;

pub fn measure(s: &str, size: u16) -> f32 {
    let size = size.max(MIN_FONT);
    let d = measure_text(s, Some(font()), size, 1.0);
    d.width
}

// 以 (x, y) 为文字左上角绘制
// v1.6.0 B 清晰度：最终 x 与基线 y 都取整——txt_c/txt_r/wrap_draw 均经此处输出，
// measure*0.5 求中与 0.82 基线系数常产生小数坐标，亚像素采样导致笔画羽化发虚。
pub fn txt(s: &str, x: f32, y: f32, size: u16, c: Color) {
    let size = size.max(MIN_FONT);
    draw_text_ex(
        s,
        x.floor(),
        (y + size as f32 * 0.82).floor(),
        TextParams {
            font: Some(font()),
            font_size: size,
            color: c,
            ..Default::default()
        },
    );
}

pub fn txt_c(s: &str, cx: f32, y: f32, size: u16, c: Color) {
    txt(s, cx - measure(s, size) * 0.5, y, size, c);
}

pub fn txt_r(s: &str, rx: f32, y: f32, size: u16, c: Color) {
    txt(s, rx - measure(s, size), y, size, c);
}

// v1.9.25 卡顿修复（根因1）：按字号集预热字形——macroquad 字形按（字符,字号）缓存，
// 首见即栅格化入图集并置 dirty，下一个绘制帧整图集重上传（2048²=16MB/次）。
// banner 的 0.6s pop 动画若逐帧首见新字号，等于逐帧整图上传；show_banner 在弹出
// 事件帧调用本函数把全部档位字形一次栅格化，动画帧零缓存 miss。
pub fn prewarm_text(text: &str, sizes: &[u16]) {
    for ch in text.chars() {
        // v1.10.1：不再跳过空白——draw_text_ex 对空格同样走 cache_glyph（0×0 精灵
        // 也会置图集 dirty），跳过会让首条 banner 的 pop 期留 4 次/档 miss
        let cs = ch.to_string();

        for &sz in sizes {
            measure(&cs, sz);
        }
    }
}

// ---------------- 换行记忆化（v1.9.25 根因2：wrap_draw 每帧逐字符 measure+to_string
// 是模态卡/天赋卡/结算页的稳定分配底噪；命中路径零分配，上限防膨胀整体清空） ----------------

struct WrapKey {
    text: String,
    size: u16,
    maxw_bits: u32,
}

std::thread_local! {
    static WRAP_CACHE: std::cell::RefCell<Vec<(WrapKey, Vec<String>)>> =
        std::cell::RefCell::new(Vec::new());
}

const WRAP_CACHE_CAP: usize = 256;

// 命中返回缓存槽位；未命中计算并入缓存（本帧一次分配）
fn wrap_slot(
    s: &str,
    size: u16,
    maxw: f32,
    cache: &mut Vec<(WrapKey, Vec<String>)>,
) -> usize {
    let bits = maxw.to_bits();

    if let Some(i) = cache
        .iter()
        .position(|(k, _)| k.size == size && k.maxw_bits == bits && k.text.as_str() == s)
    {
        return i;
    }

    if cache.len() >= WRAP_CACHE_CAP {
        cache.clear();
    }

    cache.push((
        WrapKey {
            text: s.to_string(),
            size,
            maxw_bits: bits,
        },
        wrap_uncached(s, size, maxw),
    ));

    cache.len() - 1
}

// 贪心换行：中文逐字断行，优先在空格/标点处断
fn wrap_uncached(s: &str, size: u16, maxw: f32) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut cur_w = 0.0;
    for ch in s.chars() {
        let cs = ch.to_string();
        let w = measure(&cs, size);
        if cur_w + w > maxw && !cur.is_empty() {
            // 尝试在行尾标点前回收
            lines.push(cur.clone());
            cur.clear();
            cur_w = 0.0;
        }
        cur.push_str(&cs);
        cur_w += w;
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

pub fn wrap_draw(s: &str, x: f32, y: f32, size: u16, maxw: f32, line_h: f32, c: Color) -> f32 {
    WRAP_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();

        let i = wrap_slot(s, size, maxw, &mut cache);

        let lines = &cache[i].1;

        for (i, l) in lines.iter().enumerate() {
            txt(l, x, y + i as f32 * line_h, size, c);
        }

        lines.len() as f32 * line_h
    })
}
