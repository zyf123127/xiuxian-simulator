// 全部绘制：菜单、现实主界面、模拟天赋抽取、模拟播放、继承结算、总结算、各面板
use crate::art::{draw_cultivator, Art};
use crate::data::*;
use crate::game::*;
use crate::ui::*;
use macroquad::prelude::*;
use macroquad::rand::gen_range;

const TOP_H: f32 = 52.0;
const LEFT_W: f32 = 360.0;
const PANEL_B: f32 = 612.0;

pub fn button(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: &str,
    sub: Option<&str>,
    enabled: bool,
) -> bool {
    // v1.5.0：经信箱相机回虚拟坐标（桌面恒等，H5 任意屏比例正确命中）
    let (mx, my) = crate::cam::mouse();
    let hot = mx >= x && mx <= x + w && my >= y && my <= y + h;
    let base = if !enabled {
        Color::from_rgba(38, 32, 56, 225)
    } else if hot {
        Color::from_rgba(92, 66, 122, 245)
    } else {
        Color::from_rgba(56, 42, 86, 235)
    };
    draw_rectangle(x, y, w, h, base);
    draw_rectangle_lines(
        x,
        y,
        w,
        h,
        2.0,
        if enabled {
            C_GOLD_D
        } else {
            Color::from_rgba(66, 56, 88, 255)
        },
    );
    if let Some(s) = sub {
        txt_c(
            label,
            x + w * 0.5,
            y + h * 0.5 - 17.0,
            20,
            if enabled { C_TEXT } else { C_DIM },
        );
        txt_c(
            s,
            x + w * 0.5,
            y + h - 26.0,
            13,
            if enabled {
                C_DIM
            } else {
                Color::from_rgba(152, 148, 168, 255) // v1.6.0：对比度提升（110,104,120→）
            },
        );
    } else {
        txt_c(
            label,
            x + w * 0.5,
            y + h * 0.5 - 11.0,
            18,
            if enabled { C_TEXT } else { C_DIM },
        );
    }
    hot && enabled && is_mouse_button_pressed(MouseButton::Left)
}

pub fn bar(x: f32, y: f32, w: f32, h: f32, frac: f32, c: Color) {
    draw_rectangle(x, y, w, h, Color::from_rgba(14, 10, 26, 230));
    let f = frac.clamp(0.0, 1.0);
    if f > 0.003 {
        draw_rectangle(x, y, w * f, h, c);
    }
    draw_rectangle_lines(x, y, w, h, 1.5, Color::from_rgba(96, 80, 130, 255));
}

pub(crate) fn panel_card(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, C_PANEL);
    draw_rectangle_lines(x, y, w, h, 2.5, C_GOLD_D);
    draw_rectangle_lines(
        x + 5.0,
        y + 5.0,
        w - 10.0,
        h - 10.0,
        1.0,
        Color::from_rgba(96, 76, 130, 160),
    );
    // 四角金色 L 形角饰
    let cl = 16.0f32;
    let t = 3.0f32;
    for (cx, cy, dx, dy) in [
        (x, y, 1.0, 1.0),
        (x + w, y, -1.0, 1.0),
        (x, y + h, 1.0, -1.0),
        (x + w, y + h, -1.0, -1.0),
    ] {
        draw_rectangle(
            cx - if dx < 0.0 { cl } else { 0.0 },
            cy - if dy < 0.0 { t } else { 0.0 },
            cl,
            t,
            C_GOLD,
        );
        draw_rectangle(
            cx - if dx < 0.0 { t } else { 0.0 },
            cy - if dy < 0.0 { cl } else { 0.0 },
            t,
            cl,
            C_GOLD,
        );
    }
}

// 修士脚下旋转法阵（透视椭圆 + 双圈刻度反向旋转）
fn draw_magic_circle(cx: f32, cy: f32, t: f32, c: Color) {
    let squash = 0.42f32;
    let draw_ellipse_line = |ccx: f32, ccy: f32, r: f32, seg: i32, col: Color| {
        let mut prev = (ccx + r, ccy);
        for i in 1..=seg {
            let a = std::f32::consts::TAU * i as f32 / seg as f32;
            let p = (ccx + a.cos() * r, ccy + a.sin() * r * squash);
            draw_line(prev.0, prev.1, p.0, p.1, 1.5, col);
            prev = p;
        }
    };
    let a = |v: f32| Color::new(c.r, c.g, c.b, v);
    draw_ellipse_line(cx, cy, 86.0, 48, a(0.30));
    draw_ellipse_line(cx, cy, 62.0, 40, a(0.20));
    // 外圈顺时针刻度
    for i in 0..12 {
        let ang = t * 0.35 + std::f32::consts::TAU * i as f32 / 12.0;
        let (r0, r1) = (72.0, 84.0);
        draw_line(
            cx + ang.cos() * r0,
            cy + ang.sin() * r0 * squash,
            cx + ang.cos() * r1,
            cy + ang.sin() * r1 * squash,
            2.0,
            a(0.40),
        );
    }
    // 内圈逆时针菱形符文点
    for i in 0..6 {
        let ang = -t * 0.5 + std::f32::consts::TAU * i as f32 / 6.0;
        let px = cx + ang.cos() * 62.0;
        let py = cy + ang.sin() * 62.0 * squash;
        draw_rectangle(px - 2.0, py - 2.0, 4.0, 4.0, a(0.5));
    }
    // 中心微光
    draw_ellipse(cx, cy, 60.0, 26.0, 0.0, a(0.06));
}

pub fn draw_bg(art: &Art) {
    draw_texture(&art.bg, 0.0, 0.0, WHITE);
}

// 云雾带 + 仙鹤剪影（纯函数绘制，零分配）
pub fn draw_sky_life(g: &Game) {
    let t = g.time;
    // 三条云雾带
    for i in 0..3 {
        let speed = 8.0 + i as f32 * 5.0;
        let y = 180.0 + i as f32 * 150.0;
        let off = ((t * speed) % (V_W + 400.0)) - 200.0;
        let a = 0.045 + i as f32 * 0.012;
        for k in 0..3 {
            let cx = ((off + k as f32 * 460.0 + 100.0 * i as f32) % (V_W + 400.0)) - 200.0;
            draw_ellipse(
                cx,
                y + (k as f32 * 23.0) % 40.0,
                190.0,
                22.0,
                0.0,
                Color::new(0.65, 0.6, 0.85, a),
            );
        }
    }
    // 两只仙鹤剪影横穿
    for h in 0..2 {
        let period = 34.0 + h as f32 * 17.0;
        let ph = (t / period + h as f32 * 0.45).fract();
        let x = -80.0 + ph * (V_W + 160.0);
        let y = 120.0 + h as f32 * 90.0 + (t * (0.7 + h as f32 * 0.3)).sin() * 18.0;
        let s = 0.8 + h as f32 * 0.35;
        let dir = 1.0;
        let flap = ((t * 5.0 + h as f32 * 1.7).sin() * 6.0).abs();
        let c = Color::new(0.82, 0.80, 0.92, 0.55);
        // 身体
        draw_ellipse(x, y, 13.0 * s, 3.2 * s, 0.0, c);
        // 双翼（上下摆动）
        draw_triangle(
            vec2(x - 2.0 * s * dir, y - 1.0 * s),
            vec2(x - 10.0 * s * dir, y - (6.0 + flap) * s),
            vec2(x + 3.0 * s * dir, y - 1.0 * s),
            c,
        );
        draw_triangle(
            vec2(x - 2.0 * s * dir, y + 1.0 * s),
            vec2(x - 10.0 * s * dir, y + (6.0 + flap) * s),
            vec2(x + 3.0 * s * dir, y + 1.0 * s),
            c,
        );
        // 颈与尾
        draw_line(
            x + 10.0 * s * dir,
            y - 2.0 * s,
            x + 17.0 * s * dir,
            y - 5.0 * s,
            2.0 * s,
            c,
        );
        draw_line(
            x - 12.0 * s * dir,
            y,
            x - 18.0 * s * dir,
            y + 2.5 * s,
            2.0 * s,
            c,
        );
    }
    // 流星（拖尾）
    for m in &g.meteors {
        let a = (m.life / 1.1).clamp(0.0, 1.0);
        let tail = vec2(m.x - m.vx * 0.12, m.y - m.vy * 0.12);
        draw_line(
            m.x,
            m.y,
            tail.x,
            tail.y,
            2.0,
            Color::new(1.0, 1.0, 0.95, 0.7 * a),
        );
        draw_circle(m.x, m.y, 2.2, Color::new(1.0, 1.0, 0.9, 0.9 * a));
    }
}

// 流光进度条（亮带在填充区往复）
pub fn bar_glow(x: f32, y: f32, w: f32, h: f32, frac: f32, c: Color, t: f32) {
    bar(x, y, w, h, frac, c);
    let f = frac.clamp(0.0, 1.0);
    if f > 0.02 {
        let sweep = ((t * 0.45).fract() * 2.0 - 1.0).abs();
        let gx = x + sweep * (w * f);
        let gw = 26.0f32.min(w * f);
        if gx + gw > x && gx < x + w * f {
            draw_rectangle(gx, y + 1.0, gw, h - 2.0, Color::new(1.0, 1.0, 1.0, 0.18));
        }
    }
}

// 全屏横幅演出（突破/飞升/斩敌）
pub fn draw_banner(g: &Game) {
    if let Some((text, t, c)) = &g.banner {
        let k = (1.0 - t / 2.4).clamp(0.0, 1.0);
        let a = if k < 0.12 {
            k / 0.12
        } else if k > 0.8 {
            (1.0 - k) / 0.2
        } else {
            1.0
        };
        let scale = 1.0 + 0.35 * (1.0 - (k * 4.0).min(1.0));
        let size = (54.0 * scale) as u16;
        let w = measure(text, size);
        // 半透明底带提升可读性
        draw_rectangle(
            0.0,
            236.0,
            V_W,
            86.0,
            Color::new(0.02, 0.01, 0.05, 0.45 * a),
        );
        draw_rectangle(0.0, 236.0, V_W, 2.0, Color::new(c.r, c.g, c.b, 0.5 * a));
        draw_rectangle(0.0, 320.0, V_W, 2.0, Color::new(c.r, c.g, c.b, 0.5 * a));
        // 描边底
        for (dx, dy) in [(-2.0, 0.0), (2.0, 0.0), (0.0, -2.0), (0.0, 2.0)] {
            txt_c(
                text,
                V_W * 0.5 - w * 0.5 + dx,
                250.0 + dy,
                size,
                Color::new(0.05, 0.02, 0.1, 0.9 * a),
            );
        }
        txt_c(
            text,
            V_W * 0.5 - w * 0.5,
            250.0,
            size,
            Color::new(c.r, c.g, c.b, a),
        );
        // 两侧装饰线
        let ly = 250.0 + size as f32 * 0.5;
        let lw = (V_W - w) * 0.5 - 60.0;
        if lw > 40.0 {
            draw_line(
                60.0,
                ly,
                60.0 + lw,
                ly,
                2.0,
                Color::new(c.r, c.g, c.b, 0.5 * a),
            );
            draw_line(
                V_W - 60.0,
                ly,
                V_W - 60.0 - lw,
                ly,
                2.0,
                Color::new(c.r, c.g, c.b, 0.5 * a),
            );
        }
    }
}

// 修士灵气环绕粒子（现实左面板，纯函数无分配）
pub fn draw_aura_motes(cx: f32, cy: f32, t: f32, c: Color) {
    for i in 0..9 {
        let seed = i as f32 * 2.399;
        let ph = (t * (0.25 + 0.03 * (i % 4) as f32) + seed).fract();
        let ang = seed + t * 0.4;
        let r = 78.0 + (seed * 13.7).sin() * 16.0;
        let x = cx + ang.cos() * r;
        let y = cy + 30.0 - ph * 110.0;
        let a = 0.35 * (ph * (1.0 - ph) * 4.0).min(1.0);
        draw_circle(x, y, 1.6 + (seed % 1.4), Color::new(c.r, c.g, c.b, a));
    }
}

pub fn draw_motes(g: &Game) {
    for m in &g.motes {
        let a = 0.20 + 0.16 * (g.time * 1.7 + m.ph).sin();
        let c = if m.gold {
            Color::new(1.0, 0.85, 0.5, a)
        } else {
            Color::new(0.55, 0.85, 0.9, a)
        };
        draw_circle(m.x, m.y, m.size, c);
    }
    for s in &g.sparks {
        let a = (s.life / s.max).clamp(0.0, 1.0);
        let c = if s.gold {
            Color::new(1.0, 0.85, 0.45, a)
        } else {
            Color::new(0.6, 0.9, 1.0, a)
        };
        draw_circle(s.x, s.y, 2.6, c);
    }
}

static WATERMARK: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

pub fn version_mark() {
    // v1.5.0：水印仅调试冒烟（GF_DEBUG=1）时绘制；GF_CLEAN=1 净版截图模式同样不画。
    // H5/正式包（两个环境变量都不设）不画版本号，发布画面更干净。
    let on = *WATERMARK.get_or_init(|| {
        std::env::var("GF_DEBUG").is_ok() && std::env::var("GF_CLEAN").is_err()
    });
    if !on {
        return;
    }
    txt_r(
        &format!("v{}", env!("CARGO_PKG_VERSION")),
        V_W - 8.0,
        V_H - 20.0,
        13,
        Color::from_rgba(120, 112, 140, 200),
    );
}

pub(crate) fn draw_toast(g: &Game) {
    if let Some((text, t, c)) = &g.toast {
        let a = t.min(1.0).clamp(0.0, 1.0);
        let w = measure(text, 22) + 60.0;
        draw_rectangle(
            V_W * 0.5 - w * 0.5,
            70.0,
            w,
            44.0,
            Color::new(0.05, 0.03, 0.1, 0.85 * a),
        );
        draw_rectangle_lines(
            V_W * 0.5 - w * 0.5,
            70.0,
            w,
            44.0,
            1.5,
            Color::new(c.r, c.g, c.b, a),
        );
        txt_c(text, V_W * 0.5, 80.0, 20, Color::new(c.r, c.g, c.b, a));
    }
}

// ================= 菜单 =================
pub fn draw_menu(g: &mut Game, art: &Art) {
    draw_bg(art);
    draw_sky_life(g);
    draw_motes(g);
    let t = g.time;
    let glow = 0.75 + 0.25 * (t * 1.2).sin();
    txt_c(
        "凡 人 修 仙",
        V_W * 0.5,
        120.0,
        60,
        Color::new(1.0, 0.85, 0.5, glow),
    );
    txt_c("· 轮 回 模 拟 器 ·", V_W * 0.5, 190.0, 32, C_GOLD);
    txt_c(
        "轮回镜中演一世，造化归身证长生",
        V_W * 0.5,
        240.0,
        18,
        C_DIM,
    );
    draw_cultivator(
        art,
        V_W * 0.5,
        385.0,
        1.2,
        col_realm(g.meta.best_realm),
        t,
        true,
        portrait_tier(g.meta.best_realm),
    );
    let bx = V_W * 0.5 - 320.0;
    if button(bx, 522.0, 300.0, 54.0, "开始新的人生", None, true) {
        g.play(crate::sounds::Which::Click);
        g.begin_mortal();
    }
    if g.meta.life.is_some() {
        if button(
            bx + 320.0,
            522.0,
            300.0,
            54.0,
            "承接前缘（继续）",
            None,
            true,
        ) {
            g.play(crate::sounds::Which::Click);
            g.life = g.meta.life.clone(); // 还原现实主角（缺失则 draw_reality 黑屏）
            g.panel = 0;
            g.scene = Scene::Reality;
            g.log("轮回镜微亮，前缘未了。", C_TEXT);
        }
    }
    let mlabel = if g.meta.music {
        "音乐：开"
    } else {
        "音乐：关"
    };
    if button(bx, 592.0, 140.0, 44.0, mlabel, None, true) {
        g.meta.music = !g.meta.music;
        if g.meta.music {
            g.audio.start_bgm();
        } else {
            g.audio.stop_bgm();
        }
        crate::save::save(&g.meta);
    }
    let slabel = if g.meta.sfx {
        "音效：开"
    } else {
        "音效：关"
    };
    if button(bx + 155.0, 592.0, 140.0, 44.0, slabel, None, true) {
        g.meta.sfx = !g.meta.sfx;
        g.audio.sfx_on = g.meta.sfx;
        crate::save::save(&g.meta);
    }
    if button(bx + 310.0, 592.0, 140.0, 44.0, "轮回烙印", None, true) {
        g.panel = 3;
        g.tab = 0;
    }
    if button(bx + 465.0, 592.0, 140.0, 44.0, "帮助", None, true) {
        g.panel = 4;
    }
    let best = REALMS[g.meta.best_realm.min(14)].name;
    txt_c(
        &format!(
            "已历 {} 世 · 最高境界 {}境 · 斩杀宿敌 {} · 道韵 {}",
            g.meta.lives,
            best,
            g.meta.neme_slain,
            fmt_int(g.meta.dao_yun)
        ),
        V_W * 0.5,
        676.0,
        16,
        C_DIM,
    );
    if g.panel == 3 {
        draw_upgrade_panel(g);
    }
    if g.panel == 4 {
        draw_help_panel(g);
    }
    version_mark();
}

// ================= 现实主界面 =================
pub fn draw_reality(g: &mut Game, art: &Art) {
    let Some(l) = g.life.clone() else { return };
    // v1.7.0：幻境广告浮层独占输入（模态期间不画不响应其余界面）
    if g.ad.is_some() {
        draw_ad_overlay(g);
        return;
    }
    draw_bg(art);
    draw_sky_life(g);
    draw_motes(g);
    render_top(g, &l);
    // 顶栏按钮
    if button(1086.0, 8.0, 62.0, 36.0, "烙印", None, true) {
        g.panel = 3;
        g.tab = 0;
        g.play(crate::sounds::Which::Page);
    }
    if button(1154.0, 8.0, 62.0, 36.0, "帮助", None, true) {
        g.panel = 4;
        g.play(crate::sounds::Which::Page);
    }
    render_left(g, &l, art, true);

    // 现实日志
    draw_rectangle(
        376.0,
        60.0,
        V_W - 384.0,
        PANEL_B - 60.0,
        Color::from_rgba(14, 10, 26, 215),
    );
    draw_rectangle_lines(
        376.0,
        60.0,
        V_W - 384.0,
        PANEL_B - 60.0,
        1.5,
        Color::from_rgba(90, 74, 120, 200),
    );
    txt_c(
        "—— 现 实 ——",
        376.0 + (V_W - 384.0) * 0.5,
        68.0,
        13,
        Color::from_rgba(162, 152, 184, 220), // v1.6.0：对比度提升（110,100,130→）
    );
    txt_r(
        &format!("所在：{}", LOCATIONS[l.loc.min(5)].name),
        1244.0,
        68.0,
        14,
        C_DIM,
    );
    let area_y = 88.0;
    let area_h = PANEL_B - 60.0 - 36.0;
    let line_h = 28.0; // v1.6.0：24→28，16px 中文行距 1.5 倍更透气（18 行预算仍够）
    let max_lines = (area_h / line_h) as usize;
    let start = g.log.len().saturating_sub(max_lines);
    for (i, ll) in g.log[start..].iter().enumerate() {
        txt(&ll.text, 390.0, area_y + i as f32 * line_h, 16, ll.color);
    }

    // 底部行动栏（v1.4.0：6→8；v1.6.0 B：修炼组收进弹层 + 全钮返回值门控防弹层期穿透。
    // 取舍声明：原槽 2「冲击破境」一键直达收进修炼弹层（数字键 2 仍一键触发），
    // 核心循环动作一步变两步——Ask B 明确要求修炼组归入同一子菜单，接受）
    let bw = 140.0;
    let gap = 12.0;
    let x0 = 36.0;
    let by = 620.0;
    let bh = 88.0;
    let sub_open = g.submenu.is_some();
    let pts_ok = g.meta.sim_points >= g.sim_cost();
    // v1.7.0：山穷水尽时按钮改引广告（点了进 start_draft → start_ad）
    let stuck = g.is_stuck();
    let sim_sub: Option<String> = if stuck {
        Some("山穷水尽 · 观看广告".to_string())
    } else {
        Some(format!("推演一生 · {} 点", g.sim_cost()))
    };
    if button(
        x0,
        by,
        bw,
        bh,
        "开始模拟",
        sim_sub.as_deref(),
        pts_ok || stuck,
    ) && !sub_open
    {
        g.play(crate::sounds::Which::Click);
        g.start_draft();
    }
    {
        // v1.6.0 B：修炼弹层（统一入口 open_sub 打开帧门闩，防未来回归）
        if button(x0 + (bw + gap), by, bw, bh, "修炼", Some("闭关 · 破境"), true) && !sub_open {
            g.play(crate::sounds::Which::Click);
            crate::menu::open_sub(g, crate::menu::Sub::Cult);
        }
    }
    if button(
        x0 + (bw + gap) * 2.0,
        by,
        bw,
        bh,
        "山河图",
        Some("世界地图 · M"),
        true,
    ) && !sub_open
    {
        g.play(crate::sounds::Which::Page);
        crate::world::open_world(g);
    }
    if button(
        x0 + (bw + gap) * 3.0,
        by,
        bw,
        bh,
        "坊市",
        Some("丹药 · 藏经 · 兑换"),
        true,
    ) && !sub_open
    {
        g.panel = 1;
        g.play(crate::sounds::Which::Page);
    }
    if button(
        x0 + (bw + gap) * 4.0,
        by,
        bw,
        bh,
        "行囊",
        Some("丹药 · 法宝 · 行装"),
        true,
    ) && !sub_open
    {
        g.panel = 2;
        g.play(crate::sounds::Which::Page);
    }
    if button(
        x0 + (bw + gap) * 5.0,
        by,
        bw,
        bh,
        "人物",
        Some("长老 · 商人 · 道侣"),
        true,
    ) && !sub_open
    {
        g.panel = 5;
        g.char_tab = 3; // v1.6.0 D：直入人物页
        g.play(crate::sounds::Which::Page);
    }
    if button(
        x0 + (bw + gap) * 6.0,
        by,
        bw,
        bh,
        "角色",
        Some("C 键查看"),
        true,
    ) && !sub_open
    {
        g.panel = 5;
        g.char_tab = 0; // v1.6.0 应修①②：直点「角色」归总览页，防沿用人物页签
        g.play(crate::sounds::Which::Page);
    }
    if g.can_revenge() {
        let neme = g.meta.neme.clone().unwrap();
        let p = g.revenge_p();
        if button(
            x0 + (bw + gap) * 7.0,
            by,
            bw,
            bh,
            "寻仇",
            Some(&format!("{:.0}% · {}", p * 100.0, neme.name)),
            true,
        ) && !sub_open
        {
            g.play(crate::sounds::Which::Click);
            g.act_revenge();
        }
    } else if button(
        x0 + (bw + gap) * 7.0,
        by,
        bw,
        bh,
        "宿敌已清",
        Some("静待新敌"),
        false,
    ) {
    }

    // 面板
    match g.panel {
        1 => draw_shop(g, &l),
        2 => draw_bag(g, &l),
        3 => draw_upgrade_panel(g),
        4 => draw_help_panel(g),
        5 => crate::charpanel::draw_char_panel(g),
        _ => {}
    }
    // v1.6.0 B：修炼弹层（底栏之后绘制，盖住中部区域）
    if g.submenu.is_some() {
        crate::menu::draw_sub(g);
    }
    draw_banner(g);
    draw_toast(g);
    version_mark();
}

fn render_top(g: &Game, l: &Life) {
    draw_rectangle(0.0, 0.0, V_W, TOP_H, Color::from_rgba(16, 11, 30, 245));
    draw_rectangle(0.0, TOP_H - 2.0, V_W, 2.0, C_GOLD_D);
    txt(
        &format!("第 {} 世 · {}", l.n, realm_full_name(l)),
        16.0,
        15.0,
        20,
        C_GOLD,
    );
    txt("寿元", 330.0, 10.0, 14, C_DIM);
    bar(374.0, 14.0, 180.0, 14.0, (l.age / l.lifespan) as f32, C_RED);
    txt(
        &format!("{}/{}", fmt_num(l.age), fmt_num(l.lifespan)),
        562.0,
        12.0,
        14,
        C_TEXT,
    );
    draw_texture(&crate::art::art().icon_stone, 700.0, 16.0, WHITE);
    txt(&fmt_int(l.stones), 718.0, 15.0, 17, C_CYAN);
    txt(
        "模拟点",
        830.0,
        15.0,
        17,
        Color::from_rgba(200, 160, 250, 255),
    );
    txt(
        &format!("●{}", g.meta.sim_points),
        896.0,
        15.0,
        17,
        C_PURPLE,
    );
    draw_texture(&crate::art::art().icon_yun, 960.0, 15.0, WHITE);
    txt(&fmt_int(g.meta.dao_yun), 980.0, 15.0, 17, C_GOLD);
    txt_r("烙印", 1148.0, 15.0, 16, C_GOLD_D);
    txt_r("帮助", 1216.0, 15.0, 16, C_GOLD_D);
    draw_rectangle_lines(1086.0, 8.0, 62.0, 36.0, 1.5, C_GOLD_D);
    draw_rectangle_lines(1154.0, 8.0, 62.0, 36.0, 1.5, C_GOLD_D);
}

fn render_left(g: &Game, l: &Life, art: &Art, show_neme: bool) {
    draw_rectangle(8.0, 60.0, LEFT_W, PANEL_B - 60.0, C_PANEL2);
    draw_rectangle_lines(8.0, 60.0, LEFT_W, PANEL_B - 60.0, 2.0, C_GOLD_D);
    let cx = 8.0 + LEFT_W * 0.5;
    draw_magic_circle(cx, 218.0, g.time, col_realm(l.realm));
    draw_aura_motes(cx, 152.0, g.time, col_realm(l.realm));
    draw_cultivator(
        art,
        cx,
        152.0,
        0.80,
        col_realm(l.realm),
        g.time,
        l.seclusion.is_some(),
        portrait_tier(l.realm),
    );
    txt_c(&realm_full_name(l), cx, 236.0, 20, col_realm(l.realm));
    txt_c(&format!("『{}』", g.worn_title_of()), cx, 258.0, 14, C_GOLD);
    let need = qi_need(l);
    let frac = (l.qi / need) as f32;
    bar_glow(28.0, 274.0, LEFT_W - 40.0, 20.0, frac, C_CYAN, g.time);
    let qlabel = format!(
        "修为 {}/{}（{:.0}%）",
        fmt_num(l.qi),
        fmt_num(need),
        frac * 100.0
    );
    txt_c(
        &qlabel,
        cx + 1.0,
        277.0,
        14,
        Color::from_rgba(10, 8, 20, 255),
    );
    txt_c(&qlabel, cx, 276.0, 14, C_TEXT);
    let mut y = 312.0;
    // 属性区交替行底色
    for k in 0..8 {
        if k % 2 == 0 {
            draw_rectangle(
                20.0,
                306.0 + k as f32 * 30.0,
                LEFT_W - 24.0,
                30.0,
                Color::from_rgba(255, 255, 255, 6),
            );
        }
    }
    let row = |label: &str, y: f32| -> f32 {
        txt(label, 30.0, y, 16, C_DIM);
        y + 30.0
    };
    y = row("灵根", y);
    let rc = root_color(l.root);
    bar(
        104.0,
        y - 26.0,
        168.0,
        14.0,
        l.root / 100.0,
        Color::from_rgba(rc.0, rc.1, rc.2, 255),
    );
    txt(
        &format!("{} {}", l.root as i32, root_name(l.root)),
        280.0,
        y - 21.0,
        14,
        C_TEXT,
    );
    y = row("道心", y);
    bar(104.0, y - 26.0, 168.0, 14.0, l.dao / 100.0, C_GOLD);
    txt(&format!("{:.0}", l.dao), 280.0, y - 21.0, 14, C_TEXT);
    y = row("气运", y);
    bar(104.0, y - 26.0, 168.0, 14.0, l.luck / 100.0, C_GREEN);
    txt(&format!("{:.0}", l.luck), 280.0, y - 21.0, 14, C_TEXT);
    y = row("体魄", y);
    bar(104.0, y - 26.0, 168.0, 14.0, l.body / 100.0, C_RED);
    txt(&format!("{:.0}", l.body), 280.0, y - 21.0, 14, C_TEXT);
    y = row("战力", y);
    txt(&fmt_num(power(l)), 100.0, y - 21.0, 15, C_TEXT);
    if l.injured > 0 {
        txt(
            &format!("（重伤{}年）", l.injured),
            210.0,
            y - 21.0,
            14,
            C_RED,
        );
    }
    y = row("修炼", y);
    txt(
        &format!("每载 +{}", fmt_num(cult_rate(l, &g.meta) * 4.0)),
        100.0,
        y - 21.0,
        14,
        C_TEXT,
    );
    y = row("功法", y);
    txt(TECHS[l.tech].name, 100.0, y - 21.0, 15, C_GOLD);
    y = row("法宝", y);
    // v1.5.0：只留计数摘要（详情见角色面板·法宝页）
    if l.relics.is_empty() {
        txt("无", 100.0, y - 21.0, 14, C_DIM);
    } else {
        txt(
            &format!(
                "已装 {}/{} · 包内 {}",
                l.equipped.len().min(RELIC_EQUIP_MAX),
                RELIC_EQUIP_MAX,
                l.relics.len()
            ),
            100.0,
            y - 21.0,
            14,
            C_CYAN,
        );
    }
    // 宿敌卡片（贴面板底部）
    if show_neme {
        if let Some(n) = &g.meta.neme {
            let ny = 540.0;
            draw_rectangle(
                24.0,
                ny,
                LEFT_W - 32.0,
                60.0,
                Color::from_rgba(60, 22, 30, 220),
            );
            draw_rectangle_lines(24.0, ny, LEFT_W - 32.0, 60.0, 1.5, C_RED);
            txt("宿敌", 36.0, ny + 8.0, 14, C_RED);
            txt(&n.full(), 80.0, ny + 8.0, 15, C_TEXT);
            txt("每世模拟皆会遭遇——变强斩之！", 36.0, ny + 32.0, 13, C_DIM);
        }
    }
}

// ================= 天赋抽取 =================
pub fn draw_draft(g: &mut Game, art: &Art) {
    draw_bg(art);
    draw_motes(g);
    txt_c("轮 回 镜 · 天 机 演 演", V_W * 0.5, 70.0, 30, C_GOLD);
    let max = 1 + g.meta.upg[5].min(1) as usize;
    txt_c(
        &format!("觉醒天赋（{} 选 {}）——仅此一梦生效", g.draft.len(), max),
        V_W * 0.5,
        108.0,
        17,
        C_DIM,
    );
    // 已选
    let names: Vec<&str> = g.picked.iter().map(|&t| TALENTS[t].name).collect();
    txt_c(
        &format!(
            "已选：{}",
            if names.is_empty() {
                "（无）".to_string()
            } else {
                names.join("、")
            }
        ),
        V_W * 0.5,
        138.0,
        16,
        C_GREEN,
    );
    // 卡片
    let n = g.draft.len();
    let cw = 280.0f32;
    let ch = 330.0f32;
    let gap = 30.0f32;
    let total_w = n as f32 * cw + (n as f32 - 1.0) * gap;
    let x0 = V_W * 0.5 - total_w * 0.5;
    let (mx, my) = crate::cam::mouse();
    let draft = g.draft.clone();
    for (i, &t) in draft.iter().enumerate() {
        let card = &TALENTS[t];
        let x = x0 + i as f32 * (cw + gap);
        let y = 170.0;
        let selected = g.picked.contains(&t);
        let float = (g.time * 1.6 + i as f32 * 0.9).sin() * 3.0;
        let y = y + float;
        let hot = mx >= x && mx <= x + cw && my >= y - float && my <= y + ch;
        let (tr, tg, tb) = TIER_COLORS[card.tier as usize];
        let bg = if selected {
            Color::from_rgba(90, 66, 40, 245)
        } else if hot {
            Color::from_rgba(56, 44, 80, 240)
        } else {
            Color::from_rgba(36, 28, 58, 235)
        };
        draw_rectangle(x, y, cw, ch, bg);
        draw_rectangle_lines(
            x,
            y,
            cw,
            ch,
            if selected { 3.0 } else { 2.0 },
            if selected {
                C_GOLD
            } else {
                Color::from_rgba(tr, tg, tb, 255)
            },
        );
        if selected {
            draw_rectangle_lines(
                x - 4.0,
                y - 4.0,
                cw + 8.0,
                ch + 8.0,
                1.0,
                Color::from_rgba(tr, tg, tb, 150),
            );
        }
        // 品级徽标
        draw_rectangle(
            x + cw * 0.5 - 34.0,
            y + 18.0,
            68.0,
            26.0,
            Color::from_rgba(tr, tg, tb, 60),
        );
        txt_c(
            TIER_NAMES[card.tier as usize],
            x + cw * 0.5,
            y + 22.0,
            16,
            Color::from_rgba(tr, tg, tb, 255),
        );
        txt_c(card.name, x + cw * 0.5, y + 84.0, 24, C_TEXT);
        wrap_draw(card.desc, x + 24.0, y + 120.0, 17, cw - 48.0, 28.0, C_DIM);
        // 小图标占位：修士
        draw_cultivator(
            art,
            x + cw * 0.5,
            y + ch - 60.0,
            0.42,
            Color::from_rgba(tr, tg, tb, 200),
            g.time + i as f32,
            true,
            portrait_tier(g.meta.best_realm),
        );
        if hot && is_mouse_button_pressed(MouseButton::Left) {
            g.toggle_pick(i);
        }
    }
    let ready = !g.picked.is_empty();
    if button(
        V_W * 0.5 - 140.0,
        540.0,
        280.0,
        52.0,
        "沉 入 轮 回 镜",
        Some("开始此世模拟"),
        ready,
    ) {
        g.play(crate::sounds::Which::Ding);
        g.launch_sim();
    }
    version_mark();
}

// ================= 模拟播放 =================
pub fn draw_sim(g: &mut Game, art: &Art) {
    let Some(s) = g.sim.clone() else { return };
    let tribbing = s.trib.is_some();
    draw_bg(art);
    if !tribbing {
        draw_sky_life(g);
    }
    if tribbing {
        draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.10, 0.03, 0.18, 0.62));
        // 四周暗角
        for (x, y, w, h) in [
            (0.0, 0.0, V_W, 90.0),
            (0.0, V_H - 90.0, V_W, 90.0),
            (0.0, 0.0, 110.0, V_H),
            (V_W - 110.0, 0.0, 110.0, V_H),
        ] {
            draw_rectangle(x, y, w, h, Color::new(0.0, 0.0, 0.02, 0.35));
        }
    }
    draw_motes(g);
    // 顶栏
    draw_rectangle(0.0, 0.0, V_W, TOP_H, Color::from_rgba(26, 12, 36, 250));
    draw_rectangle(0.0, TOP_H - 2.0, V_W, 2.0, C_GOLD_D);
    txt("【模拟】", 16.0, 15.0, 20, C_PURPLE);
    txt(
        &format!("{}岁 · {}", fmt_num(s.age), realm_full_name(&s)),
        110.0,
        15.0,
        19,
        C_TEXT,
    );
    bar(
        500.0,
        14.0,
        200.0,
        14.0,
        (s.qi / qi_need(&s)) as f32,
        C_CYAN,
    );
    let ql = format!("修为 {}/{}", fmt_num(s.qi), fmt_num(qi_need(&s)));
    txt_c(&ql, 601.0, 13.0, 13, Color::from_rgba(10, 8, 20, 255));
    txt_c(&ql, 600.0, 12.0, 13, C_TEXT);
    draw_texture(&crate::art::art().icon_stone, 760.0, 16.0, WHITE);
    txt(&fmt_int(s.stones), 778.0, 15.0, 16, C_CYAN);
    // 速度按钮
    let names = ["暂停", "缓", "快", "跳过"];
    for i in 0..4 {
        let on = g.speed == i;
        if button(
            880.0 + i as f32 * 88.0,
            8.0,
            80.0,
            36.0,
            names[i],
            None,
            true,
        ) {
            g.speed = i;
            g.acc = 0.0;
            g.play(crate::sounds::Which::Click);
        }
        if on {
            draw_rectangle(880.0 + i as f32 * 88.0, 44.0, 80.0, 3.0, C_GOLD);
        }
    }
    // 日志流
    let area_y = 76.0;
    let area_h = V_H - 120.0;
    let line_h = 25.0;
    let max_lines = (area_h / line_h) as usize;
    draw_rectangle(
        8.0,
        area_y - 8.0,
        V_W - 16.0,
        area_h + 12.0,
        Color::from_rgba(14, 10, 26, 220),
    );
    draw_rectangle_lines(
        8.0,
        area_y - 8.0,
        V_W - 16.0,
        area_h + 12.0,
        1.5,
        Color::from_rgba(120, 80, 150, 220),
    );
    let start = g.sim_log.len().saturating_sub(max_lines);
    for (i, ll) in g.sim_log[start..].iter().enumerate() {
        if i % 2 == 0 {
            draw_rectangle(
                10.0,
                area_y - 2.0 + i as f32 * line_h,
                V_W - 20.0,
                line_h,
                Color::from_rgba(255, 255, 255, 7),
            );
        }
        txt(
            &ll.text,
            24.0,
            area_y + 10.0 + i as f32 * line_h,
            17,
            ll.color,
        );
    }
    // 渡劫演出
    if tribbing {
        let t = s.trib.as_ref().unwrap();
        // 多层雷云（远紫近黑）
        for i in 0..9 {
            let x = 120.0 + i as f32 * 130.0 + (g.time * 26.0 + i as f32 * 31.0).sin() * 22.0;
            let y = 40.0 + (g.time * 16.0 + i as f32 * 11.0).sin() * 7.0;
            draw_circle(x, y, 58.0, Color::new(0.20, 0.13, 0.34, 0.45));
            draw_circle(x + 30.0, y + 12.0, 40.0, Color::new(0.13, 0.08, 0.24, 0.5));
        }
        if t.flash > 0.02 {
            let bx = V_W * 0.5 + gen_range(-160.0, 160.0);
            let mut x = bx;
            let mut y = 90.0;
            let mut branch = (bx, 90.0, 0);
            while y < 460.0 {
                let nx = x + (gen_range(0.0, 1.0) - 0.5) * 90.0;
                let ny = y + 40.0 + gen_range(0.0, 45.0);
                draw_line(x, y, nx, ny, 4.0, Color::new(0.85, 0.92, 1.0, t.flash));
                draw_line(x, y, nx, ny, 9.0, Color::new(0.5, 0.6, 1.0, t.flash * 0.35));
                // 闪电分叉
                if gen_range(0.0, 1.0) < 0.35 {
                    let fx = nx + (gen_range(0.0, 1.0) - 0.5) * 130.0;
                    draw_line(
                        nx,
                        ny,
                        fx,
                        ny + 30.0 + gen_range(0.0, 40.0),
                        2.0,
                        Color::new(0.7, 0.8, 1.0, t.flash * 0.7),
                    );
                }
                if gen_range(0.0, 1.0) < 0.2 {
                    branch = (nx, ny, 1);
                }
                x = nx;
                y = ny;
            }
            let _ = branch;
            draw_rectangle(
                0.0,
                0.0,
                V_W,
                V_H,
                Color::new(0.9, 0.95, 1.0, t.flash * 0.2),
            );
        }
        txt_c(
            &format!("第 {} 重 · 雷 劫", (t.wave + 1).min(9)),
            V_W * 0.5,
            660.0,
            30,
            C_GOLD,
        );
        for i in 0..9 {
            let x = V_W * 0.5 - 180.0 + i as f32 * 45.0;
            let c = if i < t.wave {
                C_GOLD
            } else {
                Color::from_rgba(90, 74, 120, 255)
            };
            draw_circle(x, 690.0, 8.0, c);
        }
    }
    // 事件弹窗（v1.6.0 B：卡面加高、选项纵排带机制摘要副标签）
    if g.modal.is_some() {
        let (title, text, opts) = {
            let card = g.modal.as_ref().unwrap();
            (
                card.title.clone(),
                card.text.clone(),
                card.opts.clone(), // 保留 Eff 以生成副标签
            )
        };
        draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, 0.5));
        panel_card(340.0, 160.0, 600.0, 400.0);
        txt_c(&title, V_W * 0.5, 188.0, 24, C_GOLD);
        draw_rectangle(380.0, 208.0, 520.0, 1.5, Color::from_rgba(96, 76, 130, 160));
        wrap_draw(&text, 380.0, 224.0, 17, 520.0, 28.0, C_TEXT);
        for (i, (label, eff)) in opts.iter().enumerate() {
            let desc = eff.eff_desc();
            if button(
                380.0,
                316.0 + i as f32 * 74.0,
                520.0,
                62.0,
                label,
                Some(&desc),
                true,
            ) {
                g.play(crate::sounds::Which::Click);
                g.choose_option(i);
            }
        }
    }
    draw_banner(g);
    draw_toast(g);
    version_mark();
}

// ================= 继承结算 =================
pub fn draw_inherit(g: &mut Game, art: &Art) {
    // 背景保留模拟日志渐隐
    draw_bg(art);
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::from_rgba(10, 6, 20, 235));
    draw_motes(g);
    let Some(sr) = g.sim_result.clone() else {
        return;
    };
    txt_c("一 世 落 幕 · 造 化 归 身", V_W * 0.5, 56.0, 30, C_GOLD);
    // 左：此生摘要
    panel_card(60.0, 84.0, 420.0, 480.0);
    txt("此 生", 84.0, 116.0, 22, C_GOLD);
    wrap_draw(&sr.cause, 84.0, 146.0, 16, 372.0, 25.0, C_RED);
    txt(
        &format!(
            "享年 {} 载（历 {} 年）",
            fmt_num(sr.age),
            fmt_num(sr.years as f64)
        ),
        84.0,
        240.0,
        16,
        C_TEXT,
    );
    txt(
        &format!("最高境界：{}境", REALMS[sr.max_realm.min(14)].name),
        84.0,
        268.0,
        16,
        C_TEXT,
    );
    txt(
        &format!("道韵 +{}", fmt_int(sr.total_dy)),
        84.0,
        296.0,
        17,
        C_GOLD,
    );
    txt("—— 大事记 ——", 84.0, 336.0, 14, C_DIM);
    let mut y = 360.0;
    for h in sr.highlights.iter().rev().take(7) {
        wrap_draw(h, 84.0, y, 14, 372.0, 20.0, C_TEXT);
        y += 26.0;
        if y > 540.0 {
            break;
        }
    }
    // 右：继承选项
    panel_card(500.0, 84.0, 720.0, 480.0);
    let _n = g.inherit_opts.len();
    let left = g.inherit_max().saturating_sub(g.inherit_picked);
    txt(
        &format!("镜灵低语：还可择 {} 项造化", left),
        524.0,
        116.0,
        20,
        C_PURPLE,
    );
    txt("所选将随神魂带回现实", 524.0, 142.0, 14, C_DIM);
    // v1.6.0 B：行高 62 / 步进 70（至多 5 行，卡底 564 内收齐）
    for (i, &idx) in g.inherit_opts.iter().enumerate() {
        let card = &INHERITS[idx];
        let y = 160.0 + i as f32 * 70.0;
        let (mx, my) = crate::cam::mouse();
        let hot = mx >= 520.0 && mx <= 1200.0 && my >= y && my <= y + 62.0;
        draw_rectangle(
            520.0,
            y,
            680.0,
            62.0,
            if hot {
                Color::from_rgba(80, 58, 110, 240)
            } else {
                Color::from_rgba(44, 34, 70, 235)
            },
        );
        draw_rectangle_lines(520.0, y, 680.0, 62.0, 1.5, C_GOLD_D);
        txt(&format!("【{}】", card.name), 536.0, y + 10.0, 18, C_GOLD);
        txt(card.desc, 700.0, y + 12.0, 15, C_TEXT);
        txt_c("选此项", 1160.0, y + 12.0, 14, C_CYAN);
        if hot && is_mouse_button_pressed(MouseButton::Left) {
            g.play(crate::sounds::Which::Ascend);
            g.take_inherit(i);
            return;
        }
    }
    if button(
        V_W * 0.5 - 150.0,
        596.0,
        300.0,
        50.0,
        "带 着 造 化 醒 来",
        Some("回到现实"),
        true,
    ) {
        g.play(crate::sounds::Which::Ding);
        g.finish_inherit();
    }
    version_mark();
}

// ================= 总结算 =================
pub fn draw_settle(g: &mut Game, art: &Art) {
    draw_bg(art);
    draw_motes(g);
    panel_card(210.0, 70.0, 860.0, 550.0);
    txt_c("一 世 归 尘", V_W * 0.5, 112.0, 32, C_GOLD);
    if let Some(s) = &g.settle {
        let hh = wrap_draw(&s.cause, 250.0, 160.0, 18, 780.0, 27.0, C_TEXT);
        let mut y = 160.0 + hh + 16.0;
        draw_rectangle(250.0, y, 780.0, 1.0, Color::from_rgba(96, 76, 130, 160));
        y += 14.0;
        for (label, v) in &s.lines {
            txt(label, 250.0, y, 17, C_TEXT);
            txt_r(&format!("+{}", fmt_int(*v)), 930.0, y, 17, C_GREEN);
            y += 26.0;
        }
        y += 8.0;
        draw_rectangle(250.0, y, 780.0, 1.0, Color::from_rgba(96, 76, 130, 160));
        y += 16.0;
        txt("道韵累积", 250.0, y, 20, C_GOLD);
        txt_r(&format!("+{}", fmt_int(s.total)), 930.0, y, 22, C_GOLD);
        txt_r(
            &format!("总道韵 {}", fmt_int(g.meta.dao_yun)),
            1030.0,
            y,
            16,
            C_DIM,
        );
    }
    if button(
        V_W * 0.5 - 320.0,
        556.0,
        300.0,
        50.0,
        "轮 回 再 起",
        Some("道韵烙印保留"),
        true,
    ) {
        g.play(crate::sounds::Which::Click);
        g.settle = None;
        g.begin_mortal();
    }
    if button(
        V_W * 0.5 + 20.0,
        556.0,
        300.0,
        50.0,
        "回到主菜单",
        None,
        true,
    ) {
        g.play(crate::sounds::Which::Click);
        g.settle = None;
        g.scene = Scene::Menu;
    }
    draw_motes(g);
    version_mark();
}

// ================= 坊市 =================
fn draw_shop(g: &mut Game, l: &Life) {
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, 0.6));
    panel_card(140.0, 60.0, 1000.0, 600.0);
    txt("修仙坊市", 170.0, 92.0, 26, C_GOLD);
    draw_texture(&crate::art::art().icon_stone, 880.0, 84.0, WHITE);
    txt(&fmt_int(l.stones), 902.0, 92.0, 20, C_CYAN);
    if button(1064.0, 74.0, 44.0, 36.0, "×", None, true) {
        g.panel = 0;
        g.play(crate::sounds::Which::Page);
        return;
    }
    // 页签（v1.6.0 B：三页签分组——丹药阁 / 藏经阁 / 点数兑换）
    let tabs = ["丹药阁", "藏经阁", "点数兑换"];
    for (i, t) in tabs.iter().enumerate() {
        let x = 300.0 + i as f32 * 170.0;
        if button(x, 74.0, 150.0, 34.0, t, None, true) {
            g.shop_tab = i as u8;
            g.play(crate::sounds::Which::Page);
        }
        if g.shop_tab == i as u8 {
            draw_rectangle(x, 108.0, 150.0, 3.0, C_GOLD);
        }
    }
    if g.shop_tab == 2 {
        // 点数兑换（自丹药页移入独立页签）
        let cost = g.exchange_cost();
        draw_rectangle(160.0, 150.0, 960.0, 46.0, Color::from_rgba(44, 30, 66, 220));
        txt("模拟点数", 176.0, 162.0, 17, C_PURPLE);
        txt(
            &format!("●{}", g.meta.sim_points),
            280.0,
            162.0,
            17,
            C_PURPLE,
        );
        txt("轮回镜以灵石为引，可无限推演", 360.0, 162.0, 14, C_DIM);
        txt_r(
            &format!("{} 灵石 / 1点", fmt_int(cost)),
            990.0,
            162.0,
            15,
            C_CYAN,
        );
        if button(1010.0, 156.0, 94.0, 36.0, "兑换", None, l.stones >= cost) {
            g.play(crate::sounds::Which::Coin);
            g.exchange_points();
            return;
        }
        txt(
            "（境界越高兑换越贵；模拟中反杀宿敌亦可得点数）",
            176.0,
            216.0,
            14,
            C_DIM,
        );
    } else if g.shop_tab == 0 {
        for i in 0..6 {
            let y = 192.0 + i as f32 * 78.0;
            let p = &PILLS[i];
            let price = g.pill_price(i);
            draw_texture(&crate::art::art().icon_pill, 160.0, y - 6.0, WHITE);
            txt(p.name, 184.0, y + 4.0, 18, C_GOLD);
            txt(p.desc, 320.0, y + 4.0, 15, C_TEXT);
            txt(&format!("存量 {}", l.pills[i]), 320.0, y + 26.0, 14, C_DIM);
            txt_r(
                &format!("{} 灵石", fmt_int(price)),
                1010.0,
                y + 8.0,
                16,
                C_CYAN,
            );
            if button(1024.0, y - 6.0, 80.0, 40.0, "购买", None, l.stones >= price) {
                g.play(crate::sounds::Which::Coin);
                g.buy_pill(i);
                return;
            }
        }
        txt(
            "（筑基丹/破境丹/护神丹在突破与渡劫时自动生效）",
            180.0,
            636.0,
            14,
            C_DIM,
        );
    } else {
        txt(
            "藏经阁——功法一经习得，现实与模拟通用",
            180.0,
            124.0,
            15,
            C_DIM,
        );
        for idx in 1..TECHS.len() {
            // v1.6.0 B：行 y=150+(idx-1)*66、盒高 58（原 156+idx*64 / 60）
            let y = 150.0 + (idx - 1) as f32 * 66.0;
            let t = &TECHS[idx];
            let price = g.tech_price(idx);
            let owned = l.tech >= idx;
            let cur = l.tech == idx;
            draw_rectangle(
                160.0,
                y - 12.0,
                960.0,
                58.0,
                if cur {
                    Color::from_rgba(70, 52, 24, 200)
                } else {
                    Color::from_rgba(36, 28, 58, 180)
                },
            );
            txt(&format!("《{}》", t.name), 180.0, y + 4.0, 18, C_GOLD);
            txt(
                &format!("修炼×{:.1}  战力×{:.1}", t.mult, t.pow),
                420.0,
                y + 4.0,
                14,
                C_TEXT,
            );
            txt(t.desc, 420.0, y + 26.0, 14, C_DIM);
            if owned {
                if cur {
                    txt_c("当前修习", 1058.0, y + 6.0, 15, C_GREEN);
                } else {
                    txt_c("已超越", 1058.0, y + 6.0, 15, C_DIM);
                }
            } else {
                txt_r(
                    &format!("{} 灵石", fmt_int(price)),
                    1030.0,
                    y + 6.0,
                    15,
                    C_CYAN,
                );
                if button(1040.0, y - 8.0, 80.0, 40.0, "求购", None, l.stones >= price) {
                    g.play(crate::sounds::Which::Coin);
                    g.buy_tech(idx);
                    return;
                }
            }
        }
    }
}

// ================= 行囊（v1.6.0 B：三页签分组 0丹药 1法宝 2行装） =================
// v1.7.0 幻境广告浮层：死档救急的模态界面（倒计时走完才能领取，放弃无奖励）
fn draw_ad_overlay(g: &mut Game) {
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.02, 0.01, 0.05, 0.97));
    let t = g.ad.unwrap_or(0.0);
    let secs = crate::data::AD_SECONDS;
    let ready = t <= 0.0;
    panel_card(300.0, 140.0, 680.0, 440.0);
    txt_c("轮 回 商 阁", 640.0, 196.0, 28, C_GOLD);
    draw_rectangle(360.0, 216.0, 560.0, 1.5, Color::from_rgba(96, 76, 130, 160));
    // 创意区：按已看次数轮换的假广告
    let creative = crate::data::AD_CREATIVES[(g.meta.ad_watched.max(0) as usize) % 3];
    txt_c(creative, 640.0, 288.0, 24, C_TEXT);
    txt_c("—— 山穷水尽？镜灵为你接引一段商途幻境 ——", 640.0, 326.0, 14, C_DIM);
    // 倒计时条
    let frac = (t / secs).clamp(0.0, 1.0);
    draw_rectangle(440.0, 360.0, 400.0, 14.0, Color::new(0.0, 0.0, 0.0, 0.5));
    draw_rectangle(440.0, 360.0, 400.0 * frac, 14.0, C_CYAN);
    txt_c(
        &format!("看完可得 {} 枚模拟点数", crate::data::AD_REWARD),
        640.0,
        408.0,
        16,
        C_DIM,
    );
    if ready {
        if button(520.0, 440.0, 240.0, 56.0, "领取模拟点", None, true) {
            g.collect_ad();
        }
    } else {
        txt_c(
            &format!("幻境播送中… {:.0} 秒", t.ceil()),
            640.0,
            470.0,
            18,
            C_CYAN,
        );
    }
    if button(560.0, 516.0, 160.0, 36.0, "放弃", None, true) {
        g.ad = None;
        g.play(crate::sounds::Which::Page);
    }
}

fn draw_bag(g: &mut Game, l: &Life) {
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, 0.6));
    panel_card(140.0, 70.0, 1000.0, 580.0);
    txt("行囊", 170.0, 100.0, 26, C_GOLD);
    if button(1064.0, 86.0, 44.0, 36.0, "×", None, true) {
        g.panel = 0;
        g.sac_pending = None; // v1.7.0：关行囊清献祭确认态
        g.play(crate::sounds::Which::Page);
        return;
    }
    // 页签
    let tabs = ["丹药", "法宝", "行装"];
    for (i, t) in tabs.iter().enumerate() {
        let x = 300.0 + i as f32 * 150.0;
        if button(x, 86.0, 140.0, 34.0, t, None, true) {
            g.bag_tab = i as u8;
            g.play(crate::sounds::Which::Page);
        }
        if g.bag_tab == i as u8 {
            draw_rectangle(x, 120.0, 140.0, 3.0, C_GOLD);
        }
    }
    draw_rectangle(160.0, 134.0, 960.0, 1.5, Color::from_rgba(96, 76, 130, 160));
    if g.bag_tab == 0 {
        // 丹药（原列表移入，行距 62）
        txt("丹药（服用即生效）", 180.0, 158.0, 16, C_DIM);
        for i in 0..6 {
            let y = 188.0 + i as f32 * 62.0;
            let p = &PILLS[i];
            txt(p.name, 180.0, y + 4.0, 18, C_GOLD);
            txt(&format!("×{}", l.pills[i]), 300.0, y + 4.0, 16, C_TEXT);
            txt(p.desc, 360.0, y + 4.0, 15, C_TEXT);
            if button(
                1040.0,
                y - 8.0,
                84.0,
                40.0,
                "服用",
                None,
                p.edible && l.pills[i] > 0,
            ) {
                g.play(crate::sounds::Which::Gain);
                g.eat_pill(i);
                return;
            }
        }
    } else if g.bag_tab == 1 {
        // 法宝行式列表：装备/卸下钮 110×40 @x=1000
        txt("法宝（未装备仍保有 30% 灵性）", 180.0, 158.0, 16, C_DIM);
        for (k, rl) in RELICS.iter().enumerate() {
            let y = 190.0 + k as f32 * 54.0;
            let owned = l.relics.contains(&k);
            let on = l.equipped.contains(&k);
            draw_rectangle(
                160.0,
                y - 8.0,
                960.0,
                46.0,
                if on {
                    Color::from_rgba(70, 52, 24, 200)
                } else if owned {
                    Color::from_rgba(44, 34, 70, 200)
                } else {
                    Color::from_rgba(26, 22, 42, 160)
                },
            );
            draw_texture(&crate::art::art().icon_sword, 172.0, y - 2.0, WHITE);
            txt(
                &format!("【{}】", rl.name),
                196.0,
                y + 2.0,
                16,
                if owned { C_GOLD } else { C_DIM },
            );
            txt(
                rl.desc,
                360.0,
                y + 4.0,
                14,
                if owned { C_TEXT } else { C_DIM },
            );
            if on {
                if button(1000.0, y - 4.0, 110.0, 40.0, "卸下", None, true) {
                    g.play(crate::sounds::Which::Click);
                    g.unequip_relic(k);
                    return;
                }
            } else if owned {
                let can = l.equipped.len() < RELIC_EQUIP_MAX;
                if button(1000.0, y - 4.0, 110.0, 40.0, "装备", None, can) {
                    g.play(crate::sounds::Which::Ascend);
                    g.equip_relic(k);
                    return;
                }
            } else {
                txt_c("未得", 1055.0, y + 6.0, 14, C_DIM);
            }
            // v1.7.0：献祭换模拟点（二次确认；点别行自动取消）
            if owned {
                let confirming = g.sac_pending == Some(k);
                let lbl = if confirming {
                    "确认献祭?".to_string()
                } else {
                    format!("献祭+{}", rl.pt)
                };
                if button(866.0, y - 4.0, 120.0, 40.0, &lbl, None, true) {
                    if confirming {
                        g.sac_pending = None;
                        g.sacrifice_relic(k);
                        return;
                    }
                    g.sac_pending = Some(k);
                    g.play(crate::sounds::Which::Click);
                } else if confirming
                    && (crate::cam::mouse().0 < 860.0
                        || crate::cam::mouse().0 > 990.0
                        || crate::cam::mouse().1 < y - 10.0
                        || crate::cam::mouse().1 > y + 42.0)
                    && is_mouse_button_pressed(MouseButton::Left)
                {
                    g.sac_pending = None;
                }
            }
        }
    } else {
        // 行装：功法 / 法宝摘要 / 宗门 / 称号
        txt("功法", 180.0, 164.0, 17, C_DIM);
        txt(
            &format!("《{}》 {}", TECHS[l.tech].name, TECHS[l.tech].desc),
            250.0,
            164.0,
            16,
            C_GOLD,
        );
        txt("法宝", 180.0, 200.0, 17, C_DIM);
        if l.relics.is_empty() {
            txt("尚无法宝，模拟奇遇可寻。", 250.0, 200.0, 15, C_DIM);
        } else {
            txt(
                &format!(
                    "已装 {}/{} · 包内 {}（详情与装卸见「法宝」页签）",
                    l.equipped.len().min(RELIC_EQUIP_MAX),
                    RELIC_EQUIP_MAX,
                    l.relics.len()
                ),
                250.0,
                200.0,
                15,
                C_CYAN,
            );
        }
        txt("宗门", 180.0, 236.0, 17, C_DIM);
        txt(
            if l.sect { "青云宗弟子" } else { "无门无派" },
            250.0,
            236.0,
            15,
            C_TEXT,
        );
        txt("称号", 180.0, 264.0, 17, C_DIM);
        txt(&format!("『{}』", g.worn_title_of()), 250.0, 264.0, 15, C_GOLD);
        txt(
            "—— 随身行装一览；丹药服用见「丹药」页签，法宝装卸见「法宝」页签 ——",
            180.0,
            320.0,
            14,
            C_DIM,
        );
    }
}

// ================= 轮回烙印 =================
fn draw_upgrade_panel(g: &mut Game) {
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, 0.6));
    panel_card(200.0, 80.0, 880.0, 560.0);
    txt("轮回烙印", 230.0, 114.0, 26, C_GOLD);
    draw_texture(&crate::art::art().icon_yun, 800.0, 104.0, WHITE);
    txt(&fmt_int(g.meta.dao_yun), 824.0, 114.0, 20, C_GOLD);
    if button(996.0, 96.0, 44.0, 36.0, "×", None, true) {
        g.panel = 0;
        g.play(crate::sounds::Which::Page);
        return;
    }
    draw_rectangle(210.0, 132.0, 840.0, 1.5, Color::from_rgba(96, 76, 130, 160));
    txt(
        "道韵不灭，烙印永随——纵使身死道消，来世仍享此泽。",
        230.0,
        156.0,
        14,
        C_DIM,
    );
    txt(
        &format!(
            "轮回总计：历世 {} · 模拟 {} 次 · 斩杀宿敌 {} · 最长享年 {} 载",
            g.meta.lives,
            g.meta.total_sims,
            g.meta.neme_slain,
            fmt_num(g.meta.best_age)
        ),
        230.0,
        176.0,
        14,
        C_CYAN,
    );
    for i in 0..UPGRADES.len() {
        let y = 202.0 + i as f32 * 42.0;
        let u = &UPGRADES[i];
        let lv = g.meta.upg[i];
        let maxed = lv >= u.max;
        let cost = upg_cost(i, lv);
        txt(u.name, 230.0, y + 6.0, 17, C_GOLD);
        txt(&format!("{}/{}", lv, u.max), 350.0, y + 6.0, 15, C_TEXT);
        txt(u.desc, 420.0, y + 6.0, 15, C_TEXT);
        if maxed {
            txt("圆满", 950.0, y + 6.0, 15, C_DIM);
        } else {
            txt_r(
                &format!("{} 道韵", fmt_int(cost)),
                930.0,
                y + 6.0,
                14,
                C_CYAN,
            );
            if button(
                940.0,
                y - 2.0,
                84.0,
                36.0,
                "顿悟",
                None,
                g.meta.dao_yun >= cost,
            ) {
                g.play(crate::sounds::Which::Ascend);
                g.buy_upgrade(i);
                return;
            }
        }
    }
}

// ================= 帮助 =================
fn draw_help_panel(g: &mut Game) {
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, 0.6));
    panel_card(140.0, 70.0, 1000.0, 580.0);
    txt("轮回镜·使用指南", 170.0, 104.0, 26, C_GOLD);
    if button(1064.0, 86.0, 44.0, 36.0, "×", None, true) {
        g.panel = 0;
        g.play(crate::sounds::Which::Page);
        return;
    }
    let lines: [&str; 12] = [
        "【轮回镜】耗 1 枚模拟点数，推演“可能的你”的一生，直至死亡。",
        "【模拟前】3 选 1 天赋（仅此一梦生效）；模拟中自动修炼、游历、闭关、突破。",
        "【模拟毕】随机给出数项“造化”，选一项带回现实：修为、功法、法宝、丹药……",
        "【现实】闭关涨修为（寿元-1年）；修为满可突破；清源洞府闭关+25%，天雷峰渡劫更稳。",
        "【坊市】灵石购丹药，亦可兑换模拟点数（境界越高越贵）。",
        "【宿敌】血仇之敌每世模拟中皆会遭遇——实力不足则被斩，可逃可死。",
        "【斩敌】在模拟中反杀宿敌（领悟其招）或现实寻仇，皆可得灵石、点数与大笔道韵。",
        "【宿敌不灭】斩其一身，还有更强的幕后之敌接棒，直至……",
        "【现实身死】寿元耗尽则一世归尘，结算大笔道韵；轮回再起，烙印永存。",
        "【轮回烙印】道韵购买永久强化：属性下限、继承选项数、点数返还……",
        "【操作】M 键打开山河图（世界地图），C 键查看角色面板；模拟中空格暂停、数字键调速。",
        "此身虽微，万世可期。道友，请开始你的第一次模拟。",
    ];
    for (i, s) in lines.iter().enumerate() {
        let c = if i == 0 { C_GOLD } else { C_TEXT };
        wrap_draw(s, 180.0, 136.0 + i as f32 * 41.0, 16, 920.0, 24.0, c);
    }
}
