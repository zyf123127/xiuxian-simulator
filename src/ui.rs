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

pub fn measure(s: &str, size: u16) -> f32 {
    let d = measure_text(s, Some(font()), size, 1.0);
    d.width
}

// 以 (x, y) 为文字左上角绘制
pub fn txt(s: &str, x: f32, y: f32, size: u16, c: Color) {
    draw_text_ex(
        s,
        x,
        y + size as f32 * 0.82,
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

// 贪心换行：中文逐字断行，优先在空格/标点处断
pub fn wrap(s: &str, size: u16, maxw: f32) -> Vec<String> {
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
    let lines = wrap(s, size, maxw);
    for (i, l) in lines.iter().enumerate() {
        txt(l, x, y + i as f32 * line_h, size, c);
    }
    lines.len() as f32 * line_h
}
