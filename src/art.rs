// 素材光栅化：打坐修士、图标、仙山夜景背景 —— 全部程序生成，零外部素材
use crate::pixel::*;
use macroquad::prelude::*;

pub struct Art {
    pub portrait: Texture2D,
    pub icon_stone: Texture2D,
    pub icon_yun: Texture2D,
    pub icon_pill: Texture2D,
    pub icon_sword: Texture2D,
    pub bg: Texture2D,
}

static ART: std::sync::OnceLock<Art> = std::sync::OnceLock::new();

pub fn init_art() {
    let _ = ART.set(build_art());
}

pub fn art() -> &'static Art {
    ART.get().expect("art 未初始化")
}

const O: [u8; 4] = [10, 8, 16, 255]; // 描边

pub fn build_art() -> Art {
    let portrait = build_portrait();
    let icon_stone = build_icon_stone();
    let icon_yun = build_icon_yun();
    let icon_pill = build_icon_pill();
    let icon_sword = build_icon_sword();
    let bg = build_bg();
    Art {
        portrait,
        icon_stone,
        icon_yun,
        icon_pill,
        icon_sword,
        bg,
    }
}

fn build_portrait() -> Texture2D {
    let mut g = PixGrid::new(32, 36);
    let skin = rgba(255, 219, 186, 255);
    let skin_d = rgba(222, 178, 148, 255);
    let robe = rgba(88, 128, 172, 255);
    let robe_d = rgba(54, 82, 120, 255);
    let robe_l = rgba(126, 164, 205, 255);
    let hair = rgba(30, 26, 38, 255);
    let gold = rgba(222, 186, 110, 255);
    // 发髻 + 头
    g.fill_ellipse(16, 5, 2, 2, hair);
    g.fill_ellipse(16, 9, 4, 4, hair);
    g.fill_ellipse(16, 11, 3, 3, skin);
    // 眉眼（垂目入定）
    g.rect(14, 11, 1, 1, rgba(40, 34, 50, 255));
    g.rect(18, 11, 1, 1, rgba(40, 34, 50, 255));
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

// 绘制修士：aura 色随境界变化，闭关时脚下光环脉动
pub fn draw_cultivator(
    art: &Art,
    cx: f32,
    cy: f32,
    scale: f32,
    aura: Color,
    t: f32,
    meditating: bool,
) {
    let s = scale;
    // 光环
    let pulse = 0.85 + 0.15 * (t * (if meditating { 2.2 } else { 1.1 })).sin();
    let r1 = 86.0 * s * pulse;
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
    // 底座蒲团阴影
    draw_ellipse(
        cx,
        cy + 60.0 * s,
        60.0 * s,
        12.0 * s,
        0.0,
        Color::new(0.0, 0.0, 0.0, 0.35),
    );
    // 本体 32×36 放大
    let pw = 32.0 * 5.0 * s;
    let ph = 36.0 * 5.0 * s;
    let bob = if meditating {
        (t * 2.0).sin() * 3.0
    } else {
        0.0
    };
    draw_texture_ex(
        &art.portrait,
        cx - pw * 0.5,
        cy - ph * 0.52 + bob,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(pw, ph)),
            ..Default::default()
        },
    );
}
