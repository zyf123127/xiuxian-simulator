// 素材光栅化：打坐修士、图标、仙山夜景背景 —— 全部程序生成，零外部素材
use crate::pixel::*;
use macroquad::prelude::*;

use std::sync::Mutex;

pub struct Art {
    // v1.5.0 起 draw_cultivator 改用 portraits[tier]；portrait 保留为 0 档快捷引用
    #[allow(dead_code)]
    pub portrait: Texture2D,
    pub icon_stone: Texture2D,
    pub icon_yun: Texture2D,
    pub icon_pill: Texture2D,
    pub icon_sword: Texture2D,
    pub bg: Texture2D,
    // v1.6.0 C：map_bg 移除固定字段——改 map_bg_for(region, seed) 按主题参数化 + 缓存
    pub map_icons: [Texture2D; 6],
    pub portraits: [Texture2D; 4], // v1.5.0：立绘四档（0凡人 1道基 2紫府 3真仙），portrait = portraits[0]
    // v1.7.0 AI 立绘覆盖：assets/portraits/ 下有对应 PNG 则用之，缺失回退像素立绘
    // （portrait_0..3=境界四档；npc_ovr: 0=玄诚长老 1=胡掌柜。wasm 恒回退）
    pub portrait_ovr: [Option<Texture2D>; 4],
    pub npc_ovr: [Option<Texture2D>; 2],
}

static ART: std::sync::OnceLock<Art> = std::sync::OnceLock::new();

pub fn init_art() {
    let _ = ART.set(build_art());
}

pub fn art() -> &'static Art {
    ART.get().expect("art 未初始化")
}

// v1.7.0：尝试加载 AI 立绘覆盖（文件缺失静默回退；仅桌面）
#[cfg(not(target_arch = "wasm32"))]
fn try_load_ovr(name: &str) -> Option<Texture2D> {
    let bytes = std::fs::read(format!("assets/portraits/{}.png", name)).ok()?;
    let t = Texture2D::from_file_with_format(&bytes, None);
    t.set_filter(FilterMode::Linear);
    Some(t)
}

// v1.6.0 C 仇敌换图：地图底图缓存（region, seed) → Texture2D，容量 8 上限淘汰最旧。
// 主题 0/默认种子会命中 v1.5.0 逐值路径（像素级等价），仅换幕后重建。
static MAP_BG_CACHE: Mutex<Vec<(u32, u32, Texture2D)>> = Mutex::new(Vec::new());

pub fn map_bg_for(region: u32, seed: u32) -> Texture2D {
    let mut cache = MAP_BG_CACHE.lock().unwrap();
    if let Some((_, _, t)) = cache.iter().find(|(r, s, _)| *r == region && *s == seed) {
        return t.clone();
    }
    let t = build_map_bg(region, seed);
    if cache.len() >= 8 {
        cache.remove(0);
    }
    cache.push((region, seed, t.clone()));
    t
}

const O: [u8; 4] = [10, 8, 16, 255]; // 描边

pub fn build_art() -> Art {
    let portraits = std::array::from_fn(build_portrait);
    let icon_stone = build_icon_stone();
    let icon_yun = build_icon_yun();
    let icon_pill = build_icon_pill();
    let icon_sword = build_icon_sword();
    let bg = build_bg();
    let map_icons = std::array::from_fn(build_map_icon);
    let mut a = Art {
        portrait: portraits[0].clone(),
        icon_stone,
        icon_yun,
        icon_pill,
        icon_sword,
        bg,
        map_icons,
        portraits,
        portrait_ovr: [None, None, None, None],
        npc_ovr: [None, None],
    };
    #[cfg(not(target_arch = "wasm32"))]
    {
        for (i, o) in a.portrait_ovr.iter_mut().enumerate() {
            *o = try_load_ovr(&format!("portrait_{}", i));
        }
        a.npc_ovr[0] = try_load_ovr("npc_elder");
        a.npc_ovr[1] = try_load_ovr("npc_merchant");
    }
    a
}

// v1.5.0 立绘四档：0凡人 1道基（金冠）2紫府（金冠+霜鬓+袖口金纹）3真仙（金冠+白发披肩）
fn build_portrait(tier: usize) -> Texture2D {
    let tier = tier.min(3);
    let mut g = PixGrid::new(32, 36);
    let skin = rgba(255, 219, 186, 255);
    let skin_d = rgba(222, 178, 148, 255);
    let robe = rgba(88, 128, 172, 255);
    let robe_d = rgba(54, 82, 120, 255);
    let robe_l = rgba(126, 164, 205, 255);
    let hair = rgba(30, 26, 38, 255);
    let gold = rgba(222, 186, 110, 255);
    let frost = rgba(226, 230, 238, 255);
    // 发髻 + 头（真仙档须发皆白）
    let hair_c = if tier >= 3 { frost } else { hair };
    g.fill_ellipse(16, 5, 2, 2, hair_c);
    g.fill_ellipse(16, 9, 4, 4, hair_c);
    g.fill_ellipse(16, 11, 3, 3, skin);
    // 眉眼（垂目入定）
    g.rect(14, 11, 1, 1, rgba(40, 34, 50, 255));
    g.rect(18, 11, 1, 1, rgba(40, 34, 50, 255));
    // 道基以上：束发金冠
    if tier >= 1 {
        g.rect(14, 2, 4, 2, gold);
        g.set(15, 1, rgba(246, 216, 150, 255));
        g.set(16, 1, rgba(246, 216, 150, 255));
    }
    // 紫府以上：霜鬓
    if tier >= 2 {
        g.set(12, 10, frost);
        g.set(13, 10, frost);
        g.set(19, 10, frost);
        g.set(20, 10, frost);
    }
    // 肩与道袍
    g.tri([(10, 13), (22, 13), (16, 27)], robe);
    g.rect(11, 14, 10, 5, robe);
    // 领缘
    g.tri([(16, 14), (13, 19), (16, 19)], robe_l);
    g.tri([(16, 14), (19, 19), (16, 19)], robe_d);
    // 两臂环抱
    g.rect(9, 16, 3, 6, robe_d);
    g.rect(20, 16, 3, 6, robe_d);
    g.fill_circle(12, 22, 2, skin_d);
    g.fill_circle(20, 22, 2, skin_d);
    // 紫府以上：袖口金纹
    if tier >= 2 {
        g.rect(9, 21, 3, 1, gold);
        g.rect(20, 21, 3, 1, gold);
    }
    // 结印双手
    g.fill_circle(16, 21, 2, skin);
    // 腰带
    g.rect(12, 24, 8, 1, gold);
    // 盘膝
    g.fill_ellipse(16, 29, 8, 3, robe_d);
    g.fill_ellipse(16, 30, 6, 2, robe);
    g.outline(O);
    g.to_texture()
}

fn build_icon_stone() -> Texture2D {
    let mut g = PixGrid::new(12, 12);
    let c = rgba(120, 226, 232, 255);
    let c2 = rgba(210, 250, 252, 255);
    let d = rgba(66, 150, 168, 255);
    g.tri([(6, 1), (10, 6), (6, 11)], c);
    g.tri([(6, 1), (2, 6), (6, 11)], d);
    g.rect(5, 3, 1, 2, c2);
    g.outline(O);
    g.to_texture()
}

fn build_icon_yun() -> Texture2D {
    let mut g = PixGrid::new(12, 12);
    let w = rgba(235, 230, 215, 255);
    let b = rgba(40, 36, 56, 255);
    let gd = rgba(222, 186, 110, 255);
    g.fill_circle(6, 6, 5, w);
    for y in 0..12 {
        for x in 6..12 {
            let dx = (x - 6) as f32;
            let dy = (y - 6) as f32;
            if dx * dx + dy * dy <= 25.0 {
                let s = (y - 6) as i32;
                if s >= 0 || (dx + s as f32) >= 0.0 {
                    g.set(x, y, b);
                }
            }
        }
    }
    g.fill_circle(8, 4, 1, b);
    g.fill_circle(4, 8, 1, w);
    g.outline(gd);
    g.to_texture()
}

fn build_icon_pill() -> Texture2D {
    let mut g = PixGrid::new(12, 12);
    let c = rgba(232, 100, 92, 255);
    let c2 = rgba(250, 160, 150, 255);
    g.fill_circle(6, 7, 4, c);
    g.fill_circle(5, 6, 1, c2);
    g.rect(4, 2, 4, 1, rgba(222, 186, 110, 255));
    g.outline(O);
    g.to_texture()
}

fn build_icon_sword() -> Texture2D {
    let mut g = PixGrid::new(12, 12);
    let c = rgba(200, 208, 220, 255);
    let gd = rgba(222, 186, 110, 255);
    for i in 0..8 {
        g.set(3 + i, 9 - i, c);
        g.set(4 + i, 9 - i, rgba(160, 168, 184, 255));
    }
    g.rect(3, 9, 3, 1, gd);
    g.set(3, 10, rgba(90, 70, 40, 255));
    g.set(2, 11, rgba(90, 70, 40, 255));
    g.outline(O);
    g.to_texture()
}

// 1280×720 夜空仙山背景，启动时生成一次
fn build_bg() -> Texture2D {
    let (iw, ih) = (1280u16, 720u16);
    let w = 1280i32;
    let h = 720i32;
    let mut img = Image::gen_image_color(iw, ih, Color::from_rgba(0, 0, 0, 255));
    {
        let d = img.get_image_data_mut();
        // 纵向渐变夜空
        for y in 0..h {
            let t = y as f32 / h as f32;
            let r = (16.0 + 34.0 * t) as u8;
            let g = (11.0 + 22.0 * t) as u8;
            let b = (40.0 + 56.0 * t) as u8;
            for x in 0..w {
                d[(y * w + x) as usize] = [r, g, b, 255];
            }
        }
        // 星星（确定性伪随机）
        let mut seed = 20261008u32;
        let mut rnd = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / 16777216.0
        };
        for _ in 0..170 {
            let x = (rnd() * w as f32) as i32;
            let y = (rnd() * (h as f32 * 0.55)) as i32;
            let b = (140.0 + rnd() * 115.0) as u8;
            d[(y * w + x) as usize] = [b, b, (b as f32 * 1.05).min(255.0) as u8, 255];
            if rnd() < 0.12 {
                for (xx, yy) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                    if xx >= 0 && yy >= 0 && xx < w && yy < h {
                        d[(yy * w + xx) as usize] = [b / 2, b / 2, b / 2 + 20, 255];
                    }
                }
            }
        }
        // 月亮 + 月晕
        let (mx, my) = (1040i32, 120i32);
        for y in (my - 60)..=(my + 60) {
            for x in (mx - 60)..=(mx + 60) {
                if x < 0 || y < 0 || x >= w || y >= h {
                    continue;
                }
                let dx = (x - mx) as f32;
                let dy = (y - my) as f32;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist <= 34.0 {
                    d[(y * w + x) as usize] = [246, 238, 210, 255];
                } else if dist <= 36.0 {
                    d[(y * w + x) as usize] = [255, 244, 214, 255];
                } else if dist <= 58.0 {
                    let a = (1.0 - (dist - 36.0) / 22.0) * 46.0;
                    let idx = (y * w + x) as usize;
                    let old = d[idx];
                    d[idx] = [
                        old[0].saturating_add(255u8.min((255.0 * a / 255.0) as u8) / 4),
                        old[1].saturating_add(240u8.min((240.0 * a / 255.0) as u8) / 4),
                        old[2].saturating_add(200u8.min((200.0 * a / 255.0) as u8) / 4),
                        255,
                    ];
                }
            }
        }
        // 月面阴影
        for y in (my - 20)..=(my + 20) {
            for x in (mx - 20)..=(mx + 20) {
                let dx = (x - mx - 8) as f32;
                let dy = (y - my - 6) as f32;
                if dx * dx + dy * dy < 90.0 {
                    d[(y * w + x) as usize] = [224, 214, 188, 255];
                }
            }
        }
        // 三层远山
        let layer = |d: &mut [[u8; 4]], base_y: f32, amp: f32, col: [u8; 4], ph: f32| {
            for x in 0..w {
                let t = x as f32 / w as f32;
                let yv = base_y
                    - amp * (0.6 * (t * 9.0 + ph).sin() + 0.4 * (t * 21.0 + ph * 2.0).sin()).abs()
                    - amp * 0.3 * (t * 33.0 + ph).sin().abs();
                let top = yv as i32;
                for y in top.max(0)..h {
                    d[(y * w + x) as usize] = col;
                }
            }
        };
        layer(d, 470.0, 90.0, [42, 30, 78, 255], 1.3);
        // 楼阁剪影（画在中景山上）
        layer(d, 560.0, 70.0, [32, 22, 60, 255], 4.1);
        let (px0, py0) = (180i32, 445i32);
        for (rx, ry, rw, rh) in [
            (px0 - 14, py0 + 0, 28, 40),
            (px0 - 10, py0 - 16, 20, 18),
            (px0 - 5, py0 - 30, 10, 16),
        ] {
            for y in ry..ry + rh {
                for x in rx..rx + rw {
                    if x >= 0 && y >= 0 && x < w && y < h {
                        d[(y * w + x) as usize] = [12, 8, 26, 255];
                    }
                }
            }
        }
        // 飞檐
        for i in 0..8 {
            for (xx, yy) in [(px0 - 14 - i, py0 + i / 3), (px0 + 14 + i, py0 + i / 3)] {
                if xx >= 0 && xx < w && yy >= 0 && yy < h {
                    d[(yy * w + xx) as usize] = [12, 8, 26, 255];
                }
            }
        }
        layer(d, 640.0, 60.0, [22, 14, 42, 255], 7.7);
    }
    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Linear);
    tex
}

// 河心曲线（图像系坐标，画河/林木规避/水面波光共用，坐标单一来源）
pub fn map_river_y(x: f32) -> f32 {
    320.0 + 40.0 * (x * 0.004).sin() + 20.0 * (x * 0.011 + 1.7).sin()
}

// 1264×480 山河图志底图：渐变夜幕 + 苔斑 + 山脊带 + 河流 + 林木 + 暗金双线边框
// v1.6.0 C 仇敌换图：按 (region, seed) 参数化——主题色/密度取 MAP_THEMES[region%5]，
// LCG 种子由调用方传入，林木避让坐标经 data::node_xy（含主题抖动）。
// 主题 0 + 默认种子 = v1.5.0 逐值路径（LCG 调用序列保持一致，像素级等价）。
fn build_map_bg(region: u32, seed_arg: u32) -> Texture2D {
    let t = crate::data::theme_of(region);
    let (iw, ih) = (1264u16, 480u16);
    let w = 1264i32;
    let h = 480i32;
    let mut img = Image::gen_image_color(iw, ih, Color::from_rgba(0, 0, 0, 255));
    {
        let d = img.get_image_data_mut();
        // 纵向渐变（主题 0：(24,34,40) → (14,20,26)）
        for y in 0..h {
            let f = y as f32 / h as f32;
            let r = (t.grad_top.0 as f32 + (t.grad_bot.0 as f32 - t.grad_top.0 as f32) * f) as u8;
            let g = (t.grad_top.1 as f32 + (t.grad_bot.1 as f32 - t.grad_top.1 as f32) * f) as u8;
            let b = (t.grad_top.2 as f32 + (t.grad_bot.2 as f32 - t.grad_top.2 as f32) * f) as u8;
            for x in 0..w {
                d[(y * w + x) as usize] = [r, g, b, 255];
            }
        }
        // 河心曲线：抽为 pub fn map_river_y（v1.5.0 与水面波光共用，坐标单一来源）
        // 确定性伪随机（照 build_bg 先例）；v1.6.0 种子参数化
        let mut seed = seed_arg;
        let mut rnd = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / 16777216.0
        };
        // 苔斑噪点（主题 0：600 个 1-2px 暗绿/暗褐）
        let moss_n = (600.0 * t.deco) as usize;
        for _ in 0..moss_n {
            let x = (rnd() * w as f32) as i32;
            let y = (rnd() * h as f32) as i32;
            let c = if rnd() < 0.6 {
                [
                    (t.moss_a.0 as f32 + rnd() * 14.0) as u8,
                    (t.moss_a.1 as f32 + rnd() * 12.0) as u8,
                    t.moss_a.2,
                    255,
                ]
            } else {
                [
                    (t.moss_b.0 as f32 + rnd() * 10.0) as u8,
                    (t.moss_b.1 as f32 + rnd() * 8.0) as u8,
                    t.moss_b.2,
                    255,
                ]
            };
            d[(y * w + x) as usize] = c;
            if rnd() < 0.4 && x + 1 < w {
                d[(y * w + x + 1) as usize] = c;
            }
        }
        // 两条横向山脊带（逐列：脊线 y = base ± amp·sin 噪声，向下填充成带）
        let band = |d: &mut [[u8; 4]], base: f32, amp: f32, ph: f32, col: [u8; 4]| {
            for x in 0..w {
                let tt = x as f32 / w as f32;
                let top = (base
                    - amp
                        * (0.6 * (tt * 9.0 + ph).sin().abs()
                            + 0.4 * (tt * 21.0 + ph * 2.0).sin().abs())) as i32;
                let bh = (46.0 + 26.0 * (tt * 13.0 + ph).sin().abs()) as i32;
                for y in top.max(0)..(top + bh).min(h) {
                    d[(y * w + x) as usize] = col;
                }
            }
        };
        band(d, 200.0, 55.0, 1.3, [t.ridge_a.0, t.ridge_a.1, t.ridge_a.2, 255]);
        band(d, 330.0, 70.0, 4.1, [t.ridge_b.0, t.ridge_b.1, t.ridge_b.2, 255]);
        // 河流：横贯折线带，主色 + 上缘高光
        for x in 0..w {
            let yc = map_river_y(x as f32);
            let hw = 11.0 + 3.0 * (x as f32 * 0.02 + 0.5).sin();
            let y0 = (yc - hw) as i32;
            let y1 = (yc + hw) as i32;
            for y in y0.max(0)..=y1.min(h - 1) {
                d[(y * w + x) as usize] = [t.river.0, t.river.1, t.river.2, 255];
            }
            let hl = y0.max(0);
            if hl <= y1.min(h - 1) {
                d[(hl * w + x) as usize] = [t.river_hl.0, t.river_hl.1, t.river_hl.2, 255];
            }
        }
        // 林木（主题 0：220 个 2×2/3×3 深绿点簇；距节点 <40px 或 |y−河心|<20 不画）
        let forest_n = (220.0 * t.deco) as usize;
        for _ in 0..forest_n {
            let x = (rnd() * w as f32) as i32;
            let y = (rnd() * h as f32) as i32;
            let mut ok = (y as f32 - map_river_y(x as f32)).abs() >= 20.0;
            if ok {
                for k in 0..crate::data::LOCATIONS.len() {
                    let (nx, ny) = crate::data::node_xy(k, region);
                    let dx = x as f32 - nx;
                    let dy = y as f32 - ny;
                    if (dx * dx + dy * dy).sqrt() < 40.0 {
                        ok = false;
                        break;
                    }
                }
            }
            if !ok {
                continue;
            }
            let c = if rnd() < 0.5 {
                [t.forest_a.0, t.forest_a.1, t.forest_a.2, 255]
            } else {
                [t.forest_b.0, t.forest_b.1, t.forest_b.2, 255]
            };
            let s = if rnd() < 0.5 { 2 } else { 3 };
            for yy in y..(y + s).min(h) {
                for xx in x..(x + s).min(w) {
                    d[(yy * w + xx) as usize] = c;
                }
            }
        }
        // 四周 8px 暗金双线边框
        for i in [0, 1, 6, 7] {
            for x in 0..w {
                d[(i * w + x) as usize] = [t.frame.0, t.frame.1, t.frame.2, 255];
                d[((h - 1 - i) * w + x) as usize] = [t.frame.0, t.frame.1, t.frame.2, 255];
            }
            for y in 0..h {
                d[(y * w + i) as usize] = [t.frame.0, t.frame.1, t.frame.2, 255];
                d[(y * w + w - 1 - i) as usize] = [t.frame.0, t.frame.1, t.frame.2, 255];
            }
        }
    }
    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Linear);
    tex
}

// 6 地点图标：12×12 像素图（宗门/坊市/秘境/洞府/荒骨/天雷峰）
fn build_map_icon(i: usize) -> Texture2D {
    let mut g = PixGrid::new(12, 12);
    match i {
        0 => {
            // 青云宗：三层收分塔 + 翘檐（瓦青/木金）
            let wq = rgba(90, 120, 110, 255);
            let gd = rgba(222, 186, 110, 255);
            g.rect(4, 8, 4, 3, wq);
            g.rect(5, 9, 2, 2, gd); // 门
            g.rect(2, 7, 8, 1, gd); // 一层檐
            g.set(1, 6, gd); // 翘檐
            g.set(10, 6, gd);
            g.rect(4, 4, 4, 3, wq);
            g.rect(3, 3, 6, 1, gd);
            g.set(2, 2, gd);
            g.set(9, 2, gd);
            g.rect(5, 1, 2, 2, wq);
            g.set(5, 0, gd);
            g.set(6, 0, gd);
        }
        1 => {
            // 临仙坊市：屋舍 + 方孔铜钱（暖褐/铜黄）
            let br = rgba(190, 140, 80, 255);
            let cp = rgba(230, 190, 100, 255);
            g.rect(1, 6, 5, 5, br);
            g.tri([(0, 6), (6, 6), (3, 3)], rgba(150, 105, 60, 255));
            g.set(3, 8, rgba(90, 60, 40, 255)); // 门
            g.fill_circle(9, 8, 3, cp);
            g.rect(8, 7, 2, 2, rgba(40, 30, 20, 255)); // 方孔
        }
        2 => {
            // 落霞秘境：拱形石门 + 门内辉光（石灰/青白）
            let st = rgba(140, 140, 160, 255);
            let gl = rgba(180, 240, 240, 255);
            g.rect(3, 5, 6, 6, st);
            g.fill_circle(6, 5, 3, st);
            g.rect(5, 6, 2, 5, gl);
            g.set(6, 5, gl);
            g.set(5, 5, rgba(220, 252, 252, 255));
        }
        3 => {
            // 清源洞府：月洞门 + 雾纹（岩灰/雾白）
            let rk = rgba(110, 105, 125, 255);
            let mt = rgba(210, 220, 230, 255);
            g.tri([(0, 11), (6, 2), (11, 11)], rk);
            g.rect(0, 10, 12, 1, rk);
            g.fill_circle(6, 9, 3, rgba(22, 18, 34, 255)); // 洞口
            g.rect(4, 7, 4, 1, mt);
            g.set(3, 9, mt);
            g.set(9, 10, mt);
        }
        4 => {
            // 荒骨战场：交叉断剑/断骨（骨白/锈红）
            let bn = rgba(210, 205, 190, 255);
            let ru = rgba(150, 80, 70, 255);
            for k in 0..6 {
                g.set(2 + k, 9 - k, bn);
                g.set(9 - k, 9 - k, ru);
            }
            g.rect(1, 10, 2, 1, bn);
            g.rect(7, 10, 3, 1, bn);
            g.set(1, 4, ru);
        }
        _ => {
            // 天雷峰：尖峰 + 闪电折线（岩蓝灰/电白）
            let pk = rgba(90, 100, 130, 255);
            let lt = rgba(240, 240, 180, 255);
            g.tri([(1, 11), (6, 1), (11, 11)], pk);
            g.tri([(4, 4), (6, 1), (8, 4)], rgba(222, 230, 246, 255)); // 雪顶
            g.set(3, 2, lt);
            g.set(4, 3, lt);
            g.set(3, 4, lt);
            g.set(4, 5, lt);
        }
    }
    g.outline(O);
    g.to_texture()
}

// 绘制修士：aura 色随境界变化，闭关时脚下光环脉动
// v1.5.0 立绘分档：tier≥2 背光+环绕光点，tier≥3 仙环；tier 选 portraits[tier.min(3)]
pub fn draw_cultivator(
    art: &Art,
    cx: f32,
    cy: f32,
    scale: f32,
    aura: Color,
    t: f32,
    meditating: bool,
    tier: usize,
) {
    let s = scale;
    let tier = tier.min(3);
    // 光环
    let pulse = 0.85 + 0.15 * (t * (if meditating { 2.2 } else { 1.1 })).sin();
    let r1 = 86.0 * s * pulse;
    // 紫府以上：背后背光
    if tier >= 2 {
        draw_circle(
            cx,
            cy + 6.0 * s,
            r1 * 1.22,
            Color::new(aura.r, aura.g, aura.b, 0.10),
        );
    }
    draw_circle(
        cx,
        cy + 6.0 * s,
        r1,
        Color::new(aura.r, aura.g, aura.b, 0.10),
    );
    draw_circle(
        cx,
        cy + 6.0 * s,
        r1 * 0.72,
        Color::new(aura.r, aura.g, aura.b, 0.14),
    );
    draw_circle_lines(
        cx,
        cy + 6.0 * s,
        r1,
        2.0,
        Color::new(aura.r, aura.g, aura.b, 0.35 * pulse),
    );
    draw_circle_lines(
        cx,
        cy + 6.0 * s,
        r1 * 0.72,
        1.5,
        Color::new(aura.r, aura.g, aura.b, 0.28 * pulse),
    );
    // 真仙档：身后金色仙环
    if tier >= 3 {
        draw_circle_lines(
            cx,
            cy + 6.0 * s,
            r1 * 1.18,
            2.5,
            Color::new(1.0, 0.85, 0.5, 0.5 * pulse),
        );
    }
    // 底座蒲团阴影
    draw_ellipse(
        cx,
        cy + 60.0 * s,
        60.0 * s,
        12.0 * s,
        0.0,
        Color::new(0.0, 0.0, 0.0, 0.35),
    );
    // 本体 32×36 放大；AI 覆盖立绘（4:5）按原框高度等比收进
    let bob = if meditating {
        (t * 2.0).sin() * 3.0
    } else {
        0.0
    };
    let (tex, pw, ph) = match art.portrait_ovr[tier].as_ref() {
        Some(t) => (t, 144.0 * s, 180.0 * s),
        None => (&art.portraits[tier], 160.0 * s, 180.0 * s),
    };
    draw_texture_ex(
        tex,
        cx - pw * 0.5,
        cy - ph * 0.52 + bob,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(pw, ph)),
            ..Default::default()
        },
    );
    // 紫府以上：绕体光点
    if tier >= 2 {
        for k in 0..5 {
            let a = t * 1.4 + k as f32 * std::f32::consts::TAU / 5.0;
            let ox = a.cos() * r1 * 1.05;
            let oy = (a.sin() * 0.45) * r1 * 1.05 - 14.0 * s;
            draw_circle(
                cx + ox,
                cy + 6.0 * s + oy,
                2.2 * s,
                Color::new(1.0, 0.92, 0.65, 0.55 + 0.4 * (t * 3.0 + k as f32).sin()),
            );
        }
    }
}
