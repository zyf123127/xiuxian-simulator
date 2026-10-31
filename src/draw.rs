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

// v1.15.0 同点快速二击去抖：MuMu+夸克实测一击物理 tap 曾被消费两次（k1→k2，
// 点击通路的双派发残留——触摸桥 down 与 pointer/compat 路径各入队一次、
// 落入不同帧即成两个 pressed 边沿）。第二击若落在开合型按钮上会把第一击的
// 效果当帧撤销（面板开→立关），玩家体感正是"点了没反应"。本游戏无 <120ms
// 同点双击的合法操作（寻仇连点节奏 ≥200ms），同点 120ms 内第二边沿判幻影吞掉。
// 只挂在 button()（绝大多数交互的唯一入口）；charpanel/world 的裸
// is_mouse_button_pressed 判定不经过此处（无开合型按钮，不接）。
fn click_debounced(mx: f32, my: f32) -> bool {
    // 实现（v1.15.0 bisect 教训）：首版用 std Mutex+Instant+OnceLock——在 wasm 上
    // 触发 miniquad tl_event_handler "already borrowed" 恐慌风暴（一次点击=
    // 实例死亡=点哪都没反应，MuMu 夸克 100% 复现；A/B 定位到本函数）。
    // 推断：Mutex/Instant 在 wasm 引入 std 侧额外导入路径，与本壳的 RefCell
    // 事件借用相互作用。改为纯原子位打包（get_time 走 miniquad 既有 sys_now
    // 导入，零新增），语义不变：同点 120ms 内第二边沿判幻影吞掉。
    use std::sync::atomic::{AtomicU64, Ordering};
    // 打包：高 32 位=时间 ms（get_time*1000 截断），低 32 位=|x,y| 各 16 位量化
    static LAST: AtomicU64 = AtomicU64::new(0);
    let t_ms = (macroquad::time::get_time() * 1000.0) as u32 as u64;
    let qx = ((mx.max(-30000.0).min(30000.0)) + 30000.0) as u32 & 0xFFFF;
    let qy = ((my.max(-30000.0).min(30000.0)) + 30000.0) as u32 & 0xFFFF;
    let key = (t_ms << 32) | ((qx as u64) << 16) | (qy as u64);
    let prev = LAST.swap(key, Ordering::Relaxed);
    if prev != 0 {
        let (pt, pqx, pqy) = ((prev >> 32) as u32, (prev >> 16) as u16, prev as u16);
        let dt = t_ms.wrapping_sub(pt as u64) as u32;
        if dt < 120 && (qx as i32 - pqx as i32).abs() < 3 && (qy as i32 - pqy as i32).abs() < 3 {
            return false; // 幻影二击：吞
        }
    }
    true
}

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
            // v1.8.0 A6：禁用描边提亮（66,56,88→96,82,124），弱可用态一眼可辨
            Color::from_rgba(96, 82, 124, 255)
        },
    );
    if let Some(s) = sub {
        txt_c(
            label,
            x + w * 0.5,
            y + h * 0.5 - 17.0,
            20,
            // v1.8.0 A6：禁用主标 C_DIM→(184,178,196)，与可用态层级拉开
            if enabled {
                C_TEXT
            } else {
                Color::from_rgba(184, 178, 196, 255)
            },
        );
        txt_c(
            s,
            x + w * 0.5,
            y + h - 26.0, 14,
            if enabled {
                C_DIM
            } else {
                // v1.8.0 A6：禁用副标提亮（152,148,168→178,172,192）
                Color::from_rgba(178, 172, 192, 255)
            },
        );
    } else {
        txt_c(
            label,
            x + w * 0.5,
            y + h * 0.5 - 11.0,
            18,
            // v1.8.0 A6：禁用主标同上
            if enabled {
                C_TEXT
            } else {
                Color::from_rgba(184, 178, 196, 255)
            },
        );
    }
    // v1.15.0 同点快速二击去抖（见 click_debounced 注释）
    hot && enabled && is_mouse_button_pressed(MouseButton::Left) && click_debounced(mx, my)
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
        // v1.9.25 卡顿修复（根因1）：pop 字号离散化取档 {54,60,66,72}——原 (54*scale) as u16
        // 在 0.6s 动画期逐帧产生新整数字号，字形按（字符,字号）首见即栅格化 + 图集 dirty
        // 整图重上传（2048²=16MB/次）；档位有限 + show_banner 预热后动画帧零缓存 miss
        let size = ((((54.0 * scale) / 6.0).round() as u16).max(9).min(12) * 6) as u16;
        let w = measure(text, size);
        // v1.8.0 A7：横幅带收窄为文字宽+240（原全屏横带金线横穿左栏法宝区），
        // 居中绘制，装饰线端点随 bx 收拢
        let bw = (w + 240.0).min(V_W - 32.0);
        let bx = (V_W - bw) * 0.5;
        // 半透明底带提升可读性
        draw_rectangle(
            bx,
            236.0,
            bw,
            86.0,
            Color::new(0.02, 0.01, 0.05, 0.45 * a),
        );
        draw_rectangle(bx, 236.0, bw, 2.0, Color::new(c.r, c.g, c.b, 0.5 * a));
        draw_rectangle(bx, 320.0, bw, 2.0, Color::new(c.r, c.g, c.b, 0.5 * a));
        // 描边底（v1.8.0 R1 修：txt_c 为中心定位，原 `V_W*0.5 - w*0.5` 再被 txt_c 减半宽，
        // w/2 被双重应用——横幅整体左移 w/2，长标题首字被裁出左缘（既有缺陷，A7 收窄后显形）。
        // measure 与实际墨宽一致（shot_007 实测 512 ≈ w=518），直接以屏幕中心为锚）
        for (dx, dy) in [(-2.0, 0.0), (2.0, 0.0), (0.0, -2.0), (0.0, 2.0)] {
            txt_c(
                text,
                V_W * 0.5 + dx,
                250.0 + dy,
                size,
                Color::new(0.05, 0.02, 0.1, 0.9 * a),
            );
        }
        txt_c(
            text,
            V_W * 0.5,
            250.0,
            size,
            Color::new(c.r, c.g, c.b, a),
        );
        // 两侧装饰线（v1.8.0 A7：端点随 bx 收拢，不再横穿左栏）
        let ly = 250.0 + size as f32 * 0.5;
        let lw = (bw - w) * 0.5 - 20.0;
        if lw > 40.0 {
            draw_line(
                bx + 20.0,
                ly,
                bx + 20.0 + lw,
                ly,
                2.0,
                Color::new(c.r, c.g, c.b, 0.5 * a),
            );
            draw_line(
                bx + bw - 20.0,
                ly,
                bx + bw - 20.0 - lw,
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
        V_H - 20.0, 14,
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
        "凡 人 模 拟",
        V_W * 0.5,
        120.0,
        60,
        Color::new(1.0, 0.85, 0.5, glow),
    );
    txt_c("· 轮 回 模 拟 器 ·", V_W * 0.5, 190.0, 28, C_GOLD);
    txt_c(
        "天道写你的书，写了十万年，写废了十万稿——这一稿，不肯退",
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
    // v1.12.1 复审①②：panel!=0（书志/烙印/帮助罩层阅读态）时危险钮不做命中——
    // 立即模式底排 button 先于罩层绘制命中，同帧重叠矩形全触发，误点即无确认
    // begin_mortal 覆盖当前人生（resume_life 注释自证）；合页钮与它仅隔 4px
    if g.panel == 0 && button(bx, 522.0, 300.0, 54.0, "开始新的人生", None, true) {
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
            // v1.10.2 前置修复：恢复逻辑抽为 Game::resume_life（与菜单 8 秒自动进入
            // 同口——旧实现自动进入无条件 begin_mortal，有档挂菜单即被覆盖旧档人生；
            // resume_life 内含离线结算与旧档道途卡补弹）
            g.resume_life();
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
    // v1.12.1 书志：终卷四结局总览与重看（第 5 钮 bx+620=940..1080 ⊂ V_W=1280）
    if button(bx + 620.0, 592.0, 140.0, 44.0, "书志", None, true) {
        g.panel = 6;
        g.book_view = None; // Esc 关面板不清此会话态，入口处归零防重开直接落在演出页
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
    // v1.12.1 书志页：终卷四结局总览（panel 6）；达成条目点开复用演出页重读
    if g.panel == 6 {
        draw_book_record(g);
        if let Some(i) = g.book_view {
            draw_ending_page(g, i, true);
        }
    }
    version_mark();
}

// ================= 现实主界面 =================
pub fn draw_reality(g: &mut Game, art: &Art) {
    // v1.7.0：幻境广告浮层独占输入（模态期间不画不响应其余界面）
    // v1.10.2：各早退分支挪到 life 借用之前（NLL：借用存活期不接纳 &mut）
    if g.ad.is_some() {
        draw_ad_overlay(g);
        return;
    }
    // v1.10.2 离线闭关：出关结算卡独占输入（resume_life 恢复时 N≥10 弹出；
    // 收益已入账内存，收功即一次落盘；rmodal 道途卡在其后排队）
    if g.offline_card.is_some() {
        draw_bg(art);
        draw_offline_card(g);
        version_mark();
        return;
    }
    // v1.8.0 D2：现实侧好感事件卡——模态独占输入（罩层在 draw_modal_card 内）
    if g.rmodal.is_some() {
        draw_bg(art);
        draw_modal_card(g);
        version_mark();
        return;
    }
    // v1.12.1 终卷演出页：choose_ending 置位的全屏覆层独占输入（合页即回 Reality；
    // 挂早退链尾——终战卡在其前自然渲染，抉择后卡清、覆层接管）
    if g.ending_show.is_some() {
        draw_bg(art);
        draw_ending_page(g, g.ending_show.unwrap_or(0), false);
        version_mark();
        return;
    }
    // v1.10.2 卡顿修复（次要项③）：去每帧整 Life 深克隆（含 relics/talents/equipped
    // 三个 Vec 堆分配，先例 v1.10.0 draw_sim）——只读前段（顶栏/左栏渲染 + 所在行）
    // 借 life 不可变引用，借用随最后使用自然归还（NLL），后续行动栏 &mut 交互照旧
    let Some(l) = g.life.as_ref() else { return };
    draw_bg(art);
    draw_sky_life(g);
    draw_motes(g);
    render_top(g, l);
    render_left(g, l, art, true);

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
        68.0, 14,
        Color::from_rgba(162, 152, 184, 220), // v1.6.0：对比度提升（110,100,130→）
    );
    txt_r(
        &format!("所在：{}", LOCATIONS[l.loc.min(5)].name),
        1244.0,
        68.0,
        14,
        C_DIM,
    );
    // v1.10.2：顶栏三钮（烙印/帮助/主页）从 render_left 之前整体移到 life 借用
    // 最后一用（所在行）之后——立即模式命中不依赖绘制顺序（draw_sim 先例），
    // 且按钮区（y≤44）与左栏/日志区无像素重叠，z 序变化不影响画面
    if button(1086.0, 8.0, 62.0, 36.0, "烙印", None, true) {
        g.panel = 3;
        g.tab = 0;
        g.play(crate::sounds::Which::Page);
    }
    if button(1154.0, 8.0, 62.0, 36.0, "帮助", None, true) {
        g.panel = 4;
        g.play(crate::sounds::Which::Page);
    }
    // v1.9.23：返回开始界面（存档已落盘，菜单「承接前缘」可恢复）
    if button(1222.0, 8.0, 58.0, 36.0, "主页", None, true) {
        g.back_to_menu();
    }
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
    // v1.9.25 周天吐纳：8→9 钮，bw 140→127 / gap 12→10（9 钮右缘 36+9×127+8×10=1259，
    // 右边距 21px；bw=130/gap=8 右缘 1270 过贴边，弃）
    let bw = 127.0;
    let gap = 10.0;
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
    ) {
        // v1.15.0 底栏点击语义：弹层开着时点底栏 = 关弹层并同击执行（旧版
        // && !sub_open 直接吞点击——玩家忘了弹层开着时怎么点底栏都"没反应"，
        // 真机反馈"按按钮毫无反应"的确定性成因之一；现在一击必有可见反馈）
        if sub_open {
            crate::menu::close_sub(g);
        }
        g.play(crate::sounds::Which::Click);
        g.start_draft();
    }
    {
        // v1.6.0 B：修炼弹层（统一入口 open_sub 打开帧门闩，防未来回归）
        if button(x0 + (bw + gap), by, bw, bh, "修炼", Some("闭关 · 破境"), true) {
            // v1.15.0 同上：弹层开着时点修炼 = 关弹层（同帧重开同层，观感不变）
            if sub_open {
                crate::menu::close_sub(g);
            }
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
    ) {
        // v1.15.0 底栏点击语义：弹层开着时点底栏 = 关弹层并同击执行（旧版
        // && !sub_open 直接吞点击——玩家忘了弹层开着时怎么点底栏都"没反应"，
        // 真机反馈"按按钮毫无反应"的确定性成因之一；现在一击必有可见反馈）
        if sub_open {
            crate::menu::close_sub(g);
        }
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
    ) {
        // v1.15.0 底栏点击语义：弹层开着时点底栏 = 关弹层并同击执行（旧版
        // && !sub_open 直接吞点击——玩家忘了弹层开着时怎么点底栏都"没反应"，
        // 真机反馈"按按钮毫无反应"的确定性成因之一；现在一击必有可见反馈）
        if sub_open {
            crate::menu::close_sub(g);
        }
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
    ) {
        // v1.15.0 底栏点击语义：弹层开着时点底栏 = 关弹层并同击执行（旧版
        // && !sub_open 直接吞点击——玩家忘了弹层开着时怎么点底栏都"没反应"，
        // 真机反馈"按按钮毫无反应"的确定性成因之一；现在一击必有可见反馈）
        if sub_open {
            crate::menu::close_sub(g);
        }
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
    ) {
        // v1.15.0 底栏点击语义：弹层开着时点底栏 = 关弹层并同击执行（旧版
        // && !sub_open 直接吞点击——玩家忘了弹层开着时怎么点底栏都"没反应"，
        // 真机反馈"按按钮毫无反应"的确定性成因之一；现在一击必有可见反馈）
        if sub_open {
            crate::menu::close_sub(g);
        }
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
    ) {
        // v1.15.0 底栏点击语义：弹层开着时点底栏 = 关弹层并同击执行（旧版
        // && !sub_open 直接吞点击——玩家忘了弹层开着时怎么点底栏都"没反应"，
        // 真机反馈"按按钮毫无反应"的确定性成因之一；现在一击必有可见反馈）
        if sub_open {
            crate::menu::close_sub(g);
        }
        g.panel = 5;
        g.char_tab = 0; // v1.6.0 应修①②：直点「角色」归总览页，防沿用人物页签
        g.play(crate::sounds::Which::Page);
    }
    if g.can_revenge() {
        let neme = g.meta.neme.clone().unwrap();
        // 寻仇必成修复：可用性收敛到 revenge_ready 单口（重伤禁手 + 实力悬殊禁手）
        if g.revenge_ready() {
            let p = g.revenge_p();
            if button(
                x0 + (bw + gap) * 7.0,
                by,
                bw,
                bh,
                "寻仇",
                Some(&format!("{:.0}% · {}", p * 100.0, neme.name)),
                true,
            ) {
                // v1.15.0 同底栏语义：弹层开着时点寻仇 = 关弹层并执行
                if sub_open {
                    crate::menu::close_sub(g);
                }
                g.play(crate::sounds::Which::Click);
                g.act_revenge();
            }
        } else if button(
            x0 + (bw + gap) * 7.0,
            by,
            bw,
            bh,
            "寻仇",
            Some(g.revenge_disabled_text()),
            false,
        ) {
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
    // v1.9.25 周天吐纳（槽 8）：现实自动修炼开关——开启态金色下划线高亮，
    // 到点走与手点同口的 act_cultivate(true)（tick_auto 内部门控：Reality+无弹层/事件/结算）
    let auto_on = g.meta.auto == 1;
    let auto_sub = if auto_on {
        "一载/1.6s"
    } else {
        "自动闭关 · 关"
    };
    if button(
        x0 + (bw + gap) * 8.0,
        by,
        bw,
        bh,
        "周天",
        Some(auto_sub),
        true,
    ) {
        // v1.15.0 底栏点击语义：弹层开着时点底栏 = 关弹层并同击执行（旧版
        // && !sub_open 直接吞点击——玩家忘了弹层开着时怎么点底栏都"没反应"，
        // 真机反馈"按按钮毫无反应"的确定性成因之一；现在一击必有可见反馈）
        if sub_open {
            crate::menu::close_sub(g);
        }
        g.meta.auto = if auto_on { 0 } else { 1 };
        g.auto_acc = 0.0;
        g.auto_years = 0;
        g.auto_gain = 0.0;
        g.play(crate::sounds::Which::Click);
        g.sync_save();
    }
    if auto_on {
        draw_rectangle(x0 + (bw + gap) * 8.0, by + bh - 3.0, bw, 3.0, C_GOLD);
    }

    // 面板
    match g.panel {
        1 => draw_shop(g),
        2 => draw_bag(g),
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
    txt(&fmt_int(l.stones), 718.0, 15.0, 16, C_CYAN);
    txt(
        "模拟点",
        830.0,
        15.0, 16,
        Color::from_rgba(200, 160, 250, 255),
    );
    txt(
        &format!("●{}", g.meta.sim_points),
        896.0,
        15.0, 16,
        C_PURPLE,
    );
    draw_texture(&crate::art::art().icon_yun, 960.0, 15.0, WHITE);
    txt(&fmt_int(g.meta.dao_yun), 980.0, 15.0, 16, C_GOLD);
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
    // v1.14.0 立绘焕新时刻：跨相破境金环闪光——tier_flash 1.5→0，环自 0.6r 外扩渐隐
    //（会话态，旧档载入恒 0 不触发；双环内亮外淡与修士本体光环同构）
    if g.tier_flash > 0.0 {
        let k = (g.tier_flash / 1.5).min(1.0);
        let rr = 86.0 * 0.80 * (1.15 + (1.0 - k) * 0.55);
        draw_circle_lines(
            cx,
            152.0,
            rr,
            3.0,
            Color::new(1.0, 0.85, 0.5, 0.55 * k),
        );
        draw_circle_lines(
            cx,
            152.0,
            rr * 0.78,
            2.0,
            Color::new(1.0, 0.92, 0.65, 0.38 * k),
        );
    }
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
    // v1.8.0 A1：起点 312→298、步进 30→28——法宝值行收进 501..517，
    // 与宿敌卡顶 540（下方 ny）净空 23px，修左栏末行被宿敌卡叠压
    let mut y = 298.0;
    // 属性区底色（v1.9.25 过绘：α6/255 隔行 4 矩形合并为整块 1 矩形，差异肉眼不可辨）
    draw_rectangle(
        20.0,
        298.0,
        LEFT_W - 24.0,
        8.0 * 28.0,
        Color::from_rgba(255, 255, 255, 6),
    );
    let row = |label: &str, y: f32| -> f32 {
        txt(label, 30.0, y, 16, C_DIM);
        y + 28.0
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
    txt(&fmt_num(power(l)), 100.0, y - 21.0, 14, C_TEXT);
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
    txt(TECHS[l.tech].name, 100.0, y - 21.0, 14, C_GOLD);
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
            txt(&n.full(), 80.0, ny + 8.0, 14, C_TEXT);
            // v1.9.25 宿敌追缉：静态行改动态警示——宿敌随推岁成长（advance_time 6%/载），
            // 标注其当前境界提醒玩家趁早动手
            txt(
                &format!(
                    "宿敌亦在修行，寻仇宜早（已{}境）！",
                    REALMS[n.realm.min(14)].name
                ),
                36.0,
                ny + 32.0, 14,
                C_RED,
            );
        }
    }
}

// ================= 天赋抽取 =================
pub fn draw_draft(g: &mut Game, art: &Art) {
    draw_bg(art);
    draw_motes(g);
    txt_c("轮 回 镜 · 天 机 演 演", V_W * 0.5, 70.0, 28, C_GOLD);
    let max = 1 + g.meta.upg[5].min(1) as usize;
    txt_c(
        &format!("天赍 {} 选 {}——仅此一梦生效", g.draft.len(), max),
        V_W * 0.5,
        108.0, 16,
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
    // v1.12.0 留行批注：随身批注提示（开卷浮现后清空；置于落笔按钮下方空白带，避开卡片区）
    if let Some(note) = &g.meta.note_line {
        txt_c(
            &format!("朱批随身：「{}」——开卷自现，本稿气运+2", note),
            V_W * 0.5,
            604.0,
            14,
            C_GOLD,
        );
    }
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
        wrap_draw(card.desc, x + 24.0, y + 120.0, 16, cw - 48.0, 28.0, C_DIM);
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
        "落 笔",
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
    // v1.9.25 卡顿修复（根因2）：去每帧整 Life 深克隆——只读绘制段借 sim 不可变引用，
    // 速度/退出/事件弹窗等 &mut 交互统一挪到借还之后；按钮区（y≤47）与日志区（y≥68）
    // 像素无重叠，绘制次序对调不影响画面
    let Some(s) = g.sim.as_ref() else { return };
    let tribbing = s.trib.is_some();
    draw_bg(art);
    if !tribbing {
        draw_sky_life(g);
    }
    if tribbing {
        draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.10, 0.03, 0.18, 0.62));
        // 四周暗角（v1.9.25 根因3：4 矩形→预烘焙 1 纹理，角落叠加 α 精确保真）
        draw_texture_ex(
            &art.trib_vignette,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(V_W, V_H)),
                ..Default::default()
            },
        );
    }
    draw_motes(g);
    // 顶栏
    draw_rectangle(0.0, 0.0, V_W, TOP_H, Color::from_rgba(26, 12, 36, 250));
    draw_rectangle(0.0, TOP_H - 2.0, V_W, 2.0, C_GOLD_D);
    txt("【模拟】", 16.0, 15.0, 20, C_PURPLE);
    txt(
        &format!("{}岁 · {}", fmt_num(s.age), realm_full_name(&s)),
        110.0,
        15.0, 18,
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
    txt_c(&ql, 601.0, 13.0, 14, Color::from_rgba(10, 8, 20, 255));
    txt_c(&ql, 600.0, 12.0, 14, C_TEXT);
    draw_texture(&crate::art::art().icon_stone, 760.0, 16.0, WHITE);
    txt(&fmt_int(s.stones), 778.0, 15.0, 16, C_CYAN);
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
    let shown = g.sim_log.len() - start;
    // v1.9.25 过绘：隔行底色（α7/255 白，肉眼近不可辨）逐行矩形合并为整块 1 矩形
    if shown > 0 {
        draw_rectangle(
            10.0,
            area_y - 2.0,
            V_W - 20.0,
            shown as f32 * line_h,
            Color::from_rgba(255, 255, 255, 7),
        );
    }
    for (i, ll) in g.sim_log[start..].iter().enumerate() {
        txt(
            &ll.text,
            24.0,
            area_y + 10.0 + i as f32 * line_h, 16,
            ll.color,
        );
    }
    // 问笔演出
    if tribbing {
        let t = s.trib.as_ref().unwrap();
        // 多层雷云（远紫近黑）——v1.9.25 根因3：18 个 draw_circle → 9 个预烘焙云团
        // 纹理 quad（双圆 over 合成烘进图），游移公式与色彩不变
        for i in 0..9 {
            let x = 120.0 + i as f32 * 130.0 + (g.time * 26.0 + i as f32 * 31.0).sin() * 22.0;
            let y = 40.0 + (g.time * 16.0 + i as f32 * 11.0).sin() * 7.0;
            draw_texture(&art.cloud, x - 70.0, y - 70.0, WHITE);
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
            660.0, 28,
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
    // —— 只读段（s 借用）到此为止；以下交互需 &mut g ——
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
    // v1.9.23：退出刷词条——未沉入/想重来时退回选天赋（点数已退还）
    if button(1240.0, 8.0, 36.0, 36.0, "×", None, true) {
        g.quit_sim_redraft();
    }
    // 事件弹窗（v1.6.0 B：卡面加高、选项纵排带机制摘要副标签）
    // v1.8.0 D2：抽取为 draw_modal_card（模拟侧 modal 与现实侧 rmodal 共用卡面）
    draw_modal_card(g);
    draw_banner(g);
    draw_toast(g);
    version_mark();
}

// v1.8.0 D2：模态事件卡（modal=模拟侧事件 / rmodal=现实侧好感事件）共用绘制与回调分派：
// Scene::Sim → choose_option（模拟侧结算），否则 → npc::choose_roption（现实侧结算）。
pub fn draw_modal_card(g: &mut Game) {
    if g.modal.is_none() && g.rmodal.is_none() {
        return;
    }
    // v1.9.25 卡顿修复（根因2）：卡面文本/eff_desc 不再每帧克隆+重建——按卡面内容
    // 惰性缓存于 g.modal_desc（打开帧重建一次，此后逐帧只做字符串比对零分配）；
    // 模态期游戏态冻结、eff_desc 只依赖 Eff 本身（&self），无失真风险。
    // v1.10.1：缓存项含 label，按位比对 label↔label（v1.9.25 曾拿 label 比对 desc，
    // 二者按构造恒不等 → stale 恒真，缓存永不命中，模态期逐帧重建仍在）
    {
        let card = g.rmodal.as_ref().or(g.modal.as_ref()).unwrap();

        let stale = match &g.modal_desc {
            Some((t, x, d)) => {
                t != &card.title
                    || x != &card.text
                    || d.len() != card.opts.len()
                    || card
                        .opts
                        .iter()
                        .zip(d.iter())
                        .any(|((l, _), (cl, _))| l != cl)
            }
            None => true,
        };

        if stale {
            g.modal_desc = Some((
                card.title.clone(),
                card.text.clone(),
                card.opts
                    .iter()
                    .enumerate()
                    .map(|(i, (l, e))| {
                        // v1.10.2 成行择道：道途卡副标统一为改道禁示（乘区效果见卡面总述；
                        // eff_desc 无法表达 ×1.35 之类静态乘区）
                        // v1.12.1 终战卡：副标按位给抉择注脚（Eff 为空，eff_desc 无义）
                        let d = if card.kind == crate::game::CardKind::Path {
                            "择定终身，不可改道".to_string()
                        } else if card.kind == crate::game::CardKind::Finale {
                            match i {
                                0 => "永生换的 · 一字不改",
                                1 => "推荐 · 后事不详，比永生更宽",
                                2 => "描清十万稿的名字",
                                _ => "隐藏 · 未完的书自会看见",
                            }
                            .to_string()
                        } else {
                            e.eff_desc()
                        };
                        (l.clone(), d)
                    })
                    .collect(),
            ));
        }
    }

    let (title, text, descs) = {
        let (t, x, d) = g.modal_desc.as_ref().unwrap();

        (t.as_str(), x.as_str(), d.as_slice())
    };

    draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, 0.5));
    panel_card(340.0, 160.0, 600.0, 400.0);
    txt_c(title, V_W * 0.5, 188.0, 24, C_GOLD);
    draw_rectangle(380.0, 208.0, 520.0, 1.5, Color::from_rgba(96, 76, 130, 160));
    wrap_draw(text, 380.0, 224.0, 16, 520.0, 28.0, C_TEXT);
    // 命中先收集、借还后再分派（&mut）——免整卡克隆
    let mut picked: Option<usize> = None;

    // v1.12.1 终战卡四钮几何特判：默认步距 74/高 62 时 3 钮末底 526 合规、4 钮末底
    // 600 溢出 panel_card 底 560——Finale 收紧为步距 58/高 58，4 钮末底 548 合规
    let fin = g
        .rmodal
        .as_ref()
        .or(g.modal.as_ref())
        .is_some_and(|c| c.kind == crate::game::CardKind::Finale);
    let (step, bh) = if fin { (58.0, 58.0) } else { (74.0, 62.0) };

    for (i, (label, _)) in g
        .rmodal
        .as_ref()
        .or(g.modal.as_ref())
        .unwrap()
        .opts
        .iter()
        .enumerate()
    {
        if button(
            380.0,
            316.0 + i as f32 * step,
            520.0,
            bh,
            label,
            Some(descs[i].1.as_str()),
            true,
        ) {
            picked = Some(i);
        }
    }

    if let Some(i) = picked {
        g.play(crate::sounds::Which::Click);

        // v1.10.2 成行择道：道途卡抉择走 choose_path（rmodal_npc 为 None，
        // 不得落入 choose_roption——那会因 npc None 静默 return 使卡挂死）
        // v1.12.1 终战卡同理单列：choose_ending（rmodal_npc 亦为 None）
        let kind = g.rmodal.as_ref().map(|c| c.kind);
        if kind == Some(crate::game::CardKind::Path) {
            g.choose_path(i as u8 + 1);
        } else if kind == Some(crate::game::CardKind::Finale) {
            g.choose_ending(i);
        } else if g.scene == Scene::Sim {
            g.choose_option(i);
        } else {
            crate::npc::choose_roption(g, i);
        }
    }
}

// ================= 继承结算 =================
pub fn draw_inherit(g: &mut Game, art: &Art) {
    // 背景保留模拟日志渐隐
    draw_bg(art);
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::from_rgba(10, 6, 20, 235));
    draw_motes(g);
    let Some(sr) = g.sim_result.as_ref() else {
        return;
    };
    // v1.9.25 卡顿修复（根因2）：上行为借用替代原 g.sim_result.clone()——本函数对 sr
    // 只读，选项/收尾按钮的 &mut 路径上 sr 已不再被使用（NLL 借用自然终结），逐帧深克隆消除
    txt_c("草 稿 退 回", V_W * 0.5, 56.0, 28, C_GOLD);
    // 左：此生摘要
    panel_card(60.0, 84.0, 420.0, 480.0);
    txt("此 生", 84.0, 116.0, 20, C_GOLD);
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
        296.0, 16,
        C_GOLD,
    );
    // v1.12.0 留行批注：大事记点一行带走（下一稿开卷浮现后清空；定稿 §5.2 留行）。
    // 先收成 owned 再进点击循环（sr 为 &g.sim_result 借用，&mut g 须待其终结——照打稿屏 draft.clone() 先例）
    txt("—— 大事记 · 点一行留作朱批 ——", 84.0, 336.0, 14, C_DIM);
    let note_lines: Vec<String> = sr.highlights.iter().rev().take(7).cloned().collect();
    let (nmx, nmy) = crate::cam::mouse();
    let mut y = 360.0;
    for h in &note_lines {
        let noted = g.meta.note_line.as_deref() == Some(h.as_str());
        let hot = nmx >= 84.0 && nmx <= 456.0 && nmy >= y - 4.0 && nmy <= y + 20.0;
        if noted {
            txt("朱", 84.0, y, 14, C_RED);
        }
        wrap_draw(
            h,
            84.0 + if noted { 18.0 } else { 0.0 },
            y,
            14,
            372.0,
            20.0,
            if noted {
                C_RED
            } else if hot {
                C_GOLD
            } else {
                C_TEXT
            },
        );
        // 热点点击照同行继承卡先例（cam::mouse + 左键按下即拾取；重点另一行即改批）
        if hot && is_mouse_button_pressed(MouseButton::Left) {
            g.meta.note_line = Some(h.clone());
            g.toast("朱批已留——下一稿开卷自现", C_GOLD);
            g.play(crate::sounds::Which::Page);
            g.sync_save();
        }
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
        "退稿前，留一样东西。反正下一稿的你，也不知道你是谁。",
        524.0,
        116.0,
        20,
        C_PURPLE,
    );
    txt(
        &format!("还可择 {} 项 · 所选将随神魂带回现实", left),
        524.0,
        142.0,
        14,
        C_DIM,
    );
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
        txt(card.desc, 700.0, y + 12.0, 14, C_TEXT);
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
    txt_c("退 稿 结 算", V_W * 0.5, 112.0, 28, C_GOLD);
    if let Some(s) = &g.settle {
        let hh = wrap_draw(&s.cause, 250.0, 160.0, 18, 780.0, 27.0, C_TEXT);
        let mut y = 160.0 + hh + 16.0;
        draw_rectangle(250.0, y, 780.0, 1.0, Color::from_rgba(96, 76, 130, 160));
        y += 14.0;
        for (label, v) in &s.lines {
            txt(label, 250.0, y, 16, C_TEXT);
            txt_r(&format!("+{}", fmt_int(*v)), 930.0, y, 16, C_GREEN);
            y += 26.0;
        }
        y += 8.0;
        draw_rectangle(250.0, y, 780.0, 1.0, Color::from_rgba(96, 76, 130, 160));
        y += 16.0;
        txt("道韵累积", 250.0, y, 20, C_GOLD);
        txt_r(&format!("+{}", fmt_int(s.total)), 930.0, y, 20, C_GOLD);
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
        "再 打 一 稿",
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
// v1.10.2 借用化（次要项③）：签名去 l——坊市只读状态在函数头一次性值化（栈上
// 标量，life 借用随即归还），丹药存量按行瞬时短借；购买/兑换交互照旧 &mut 且
// 各分支即时 return，值化快照本帧内不失效。先例 v1.10.0 draw_sim
fn draw_shop(g: &mut Game) {
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, 0.6));
    panel_card(140.0, 60.0, 1000.0, 600.0);
    txt("修仙坊市", 170.0, 92.0, 24, C_GOLD);
    draw_texture(&crate::art::art().icon_stone, 880.0, 84.0, WHITE);
    let (stones, tech) =
        g.life
            .as_ref()
            .map_or((0i64, 0usize), |x| (x.stones, x.tech));
    txt(&fmt_int(stones), 902.0, 92.0, 20, C_CYAN);
    if button(1064.0, 74.0, 44.0, 36.0, "×", None, true) {
        g.panel = 0;
        g.buy_pending = None; // v1.8.0 B3：关坊市清求购确认态
        g.play(crate::sounds::Which::Page);
        return;
    }
    // 页签（v1.6.0 B：三页签分组——丹药阁 / 藏经阁 / 点数兑换）
    let tabs = ["丹药阁", "藏经阁", "点数兑换"];
    for (i, t) in tabs.iter().enumerate() {
        let x = 300.0 + i as f32 * 170.0;
        if button(x, 74.0, 150.0, 34.0, t, None, true) {
            g.shop_tab = i as u8;
            g.buy_pending = None; // v1.8.0 B3：切页签清求购确认态
            g.play(crate::sounds::Which::Page);
        }
        if g.shop_tab == i as u8 {
            draw_rectangle(x, 108.0, 150.0, 3.0, C_GOLD);
        }
    }
    if g.shop_tab == 2 {
        // 点数兑换（v1.8.0 A4：信息行盒加高 + 大兑换钮垂直居中 + 三行说明收编原散点提示）
        let cost = g.exchange_cost();
        draw_rectangle(160.0, 150.0, 960.0, 80.0, Color::from_rgba(44, 30, 66, 220));
        draw_rectangle_lines(160.0, 150.0, 960.0, 80.0, 1.5, Color::from_rgba(96, 76, 130, 160));
        txt("模拟点数", 176.0, 170.0, 16, C_PURPLE);
        txt(
            &format!("●{}", g.meta.sim_points),
            280.0,
            170.0, 16,
            C_PURPLE,
        );
        txt_r(
            &format!("当前价 {} 灵石 / 1 点", fmt_int(cost)),
            840.0,
            170.0, 14,
            C_CYAN,
        );
        // 大兑换钮（158..226 ⊂ 盒 150..230，上下各余 8px 垂直居中）
        if button(
            860.0,
            158.0,
            240.0,
            64.0,
            "兑换 1 点",
            Some("灵石→模拟点"),
            stones >= cost,
        ) {
            g.play(crate::sounds::Which::Coin);
            g.exchange_points();
            return;
        }
        txt(
            "轮回镜以灵石为引——点数推演一世，死后造化归身，可无限轮回。",
            176.0,
            260.0, 14,
            C_TEXT,
        );
        txt(
            "兑换价随境界水涨船高；模拟中反杀宿敌亦可白得点数。",
            176.0,
            294.0, 14,
            C_TEXT,
        );
        txt(
            &format!(
                "山穷水尽（无点无石无法宝）时，「开始模拟」将引你观看幻境，看完得 {} 点。",
                crate::data::AD_REWARD
            ),
            176.0,
            328.0, 14,
            C_TEXT,
        );
        draw_rectangle(176.0, 380.0, 928.0, 1.5, Color::from_rgba(96, 76, 130, 160));
        txt(
            "点数跨世持久（随轮回烙印存档），身死不清零——攒点推演，方是长生正途。",
            176.0,
            398.0,
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
            txt(p.desc, 320.0, y + 4.0, 14, C_TEXT);
            let stock = g.life.as_ref().map_or(0, |x| x.pills[i]);
            txt(&format!("存量 {}", stock), 320.0, y + 26.0, 14, C_DIM);
            txt_r(
                &format!("{} 灵石", fmt_int(price)),
                1010.0,
                y + 8.0,
                16,
                C_CYAN,
            );
            if button(1024.0, y - 6.0, 80.0, 40.0, "购买", None, stones >= price) {
                g.play(crate::sounds::Which::Coin);
                g.buy_pill(i);
                return;
            }
        }
        txt(
            "（成行丹/破境丹/护神丹在突破与问笔时自动生效）",
            180.0,
            636.0,
            14,
            C_DIM,
        );
    } else {
        txt(
            "藏经阁——功法一经习得，现实与模拟通用",
            180.0,
            124.0, 14,
            C_DIM,
        );
        for idx in 1..TECHS.len() {
            // v1.6.0 B：行 y=150+(idx-1)*66、盒高 58（原 156+idx*64 / 60）
            let y = 150.0 + (idx - 1) as f32 * 66.0;
            let t = &TECHS[idx];
            let price = g.tech_price(idx);
            let owned = tech >= idx;
            let cur = tech == idx;
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
                    txt_c("当前修习", 1058.0, y + 6.0, 14, C_GREEN);
                } else {
                    txt_c("已超越", 1058.0, y + 6.0, 14, C_DIM);
                }
            } else {
                txt_r(
                    &format!("{} 灵石", fmt_int(price)),
                    1030.0,
                    y + 6.0, 14,
                    // v1.8.0 A6：买不起的价格标红，与可购青色区分
                    if stones >= price { C_CYAN } else { C_RED },
                );
                // v1.8.0 B3：藏经阁「求购」二次确认（仿行囊献祭 sac_pending 范式）
                // 常态钮 80×40 @1040（不遮价格）；确认态钮 150×40 @970（盖住价格短暂可接受）
                let confirming = g.buy_pending == Some(idx);
                if confirming {
                    if button(970.0, y - 8.0, 150.0, 40.0, "确认求购?", None, stones >= price) {
                        g.buy_pending = None;
                        g.buy_tech(idx);
                        return;
                    } else if (crate::cam::mouse().0 < 964.0
                        || crate::cam::mouse().0 > 1126.0
                        || crate::cam::mouse().1 < y - 10.0
                        || crate::cam::mouse().1 > y + 42.0)
                        && is_mouse_button_pressed(MouseButton::Left)
                    {
                        g.buy_pending = None; // 点外取消
                    }
                } else if button(1040.0, y - 8.0, 80.0, 40.0, "求购", None, stones >= price) {
                    g.buy_pending = Some(idx);
                    g.play(crate::sounds::Which::Click);
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

// v1.10.2 借用化（次要项③）：签名去 l——丹药存量/功法/宗门在函数头一次性值化，
// 法宝 owned/装备态/灵纹按行瞬时短借（各交互分支即时 return，快照本帧不失效）
fn draw_bag(g: &mut Game) {
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, 0.6));
    panel_card(140.0, 70.0, 1000.0, 580.0);
    txt("行囊", 170.0, 100.0, 24, C_GOLD);
    if button(1064.0, 86.0, 44.0, 36.0, "×", None, true) {
        g.panel = 0;
        g.sac_pending = None; // v1.7.0：关行囊清献祭确认态
        g.refine_pending = None; // v1.8.0 E3：关行囊清炼化选材态
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
    let (pills, tech, sect) = g.life.as_ref().map_or(([0i32; 6], 0usize, false), |x| {
        (
            [
                x.pills[0], x.pills[1], x.pills[2], x.pills[3], x.pills[4], x.pills[5],
            ],
            x.tech,
            x.sect,
        )
    });
    if g.bag_tab == 0 {
        // 丹药（原列表移入，行距 62）
        txt("丹药（服用即生效）", 180.0, 158.0, 16, C_DIM);
        for i in 0..6 {
            let y = 188.0 + i as f32 * 62.0;
            let p = &PILLS[i];
            txt(p.name, 180.0, y + 4.0, 18, C_GOLD);
            txt(&format!("×{}", pills[i]), 300.0, y + 4.0, 16, C_TEXT);
            txt(p.desc, 360.0, y + 4.0, 14, C_TEXT);
            if button(
                1040.0,
                y - 8.0,
                84.0,
                40.0,
                "服用",
                None,
                p.edible && pills[i] > 0,
            ) {
                g.play(crate::sounds::Which::Gain);
                g.eat_pill(i);
                return;
            }
        }
    } else if g.bag_tab == 1 {
        // 法宝行式列表：装备/卸下钮 110×40 @x=1000；献祭钮 120×40 @x=866；
        // v1.8.0 E3：炼化钮 120×40 @x=736——点置选材态，再点取消
        let refining = g.refine_pending;
        if let Some(t) = refining {
            txt(
                &format!(
                    "炼化中：为【{}】选一件法宝作材料（再点「炼化」取消）",
                    RELICS[t.min(RELICS.len() - 1)].name
                ),
                180.0,
                158.0,
                16,
                C_PURPLE,
            );
        } else {
            txt("法宝（未装备仍保有 30% 灵性）", 180.0, 158.0, 16, C_DIM);
        }
        for (k, rl) in RELICS.iter().enumerate() {
            let y = 190.0 + k as f32 * 54.0;
            let owned = g.life.as_ref().is_some_and(|x| x.relics.contains(&k));
            let on = g.life.as_ref().is_some_and(|x| x.equipped.contains(&k));
            let lv = g.life.as_ref().map_or(0, |x| x.relic_lv[k.min(7)]);
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
            // v1.8.0 E3：灵纹等级徽标
            if lv > 0 {
                txt(&format!("+{}", lv), 318.0, y + 2.0, 14, C_PURPLE);
            }
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
                let can = g
                    .life
                    .as_ref()
                    .is_some_and(|x| x.equipped.len() < RELIC_EQUIP_MAX);
                if button(1000.0, y - 4.0, 110.0, 40.0, "装备", None, can) {
                    g.play(crate::sounds::Which::Ascend);
                    g.equip_relic(k);
                    return;
                }
            } else {
                txt_c("未得", 1055.0, y + 6.0, 14, C_DIM);
            }
            // v1.8.0 E3：炼化钮（owned 行）——置/清选材态（与献祭互斥）
            if owned {
                let is_target = refining == Some(k);
                let rlbl = if is_target { "取消炼化" } else { "炼化" };
                if button(736.0, y - 4.0, 120.0, 40.0, rlbl, None, true) {
                    if is_target {
                        g.refine_pending = None;
                    } else {
                        g.refine_pending = Some(k);
                        g.sac_pending = None; // 互斥：置炼化清献祭
                    }
                    g.play(crate::sounds::Which::Click);
                }
            }
            // v1.7.0：献祭换模拟点（二次确认；点别行自动取消）
            // v1.8.0 E3：选材态下，其余 owned 行此钮位变「选为材料」
            if owned {
                if let Some(t) = refining {
                    if t != k {
                        if button(866.0, y - 4.0, 120.0, 40.0, "选为材料", None, true) {
                            g.refine_pending = None;
                            g.sac_pending = None;
                            g.refine_relic(t, k);
                            return;
                        }
                    }
                } else {
                    let confirming = g.sac_pending == Some(k);
                    let lbl = if confirming {
                        "确认献祭?".to_string()
                    } else {
                        format!("献祭+{}", rl.pt)
                    };
                    if button(866.0, y - 4.0, 120.0, 40.0, &lbl, None, true) {
                        if confirming {
                            g.sac_pending = None;
                            g.refine_pending = None;
                            g.sacrifice_relic(k);
                            return;
                        }
                        g.sac_pending = Some(k);
                        g.refine_pending = None; // 互斥：置献祭清炼化
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
        }
    } else {
        // 行装：功法 / 法宝摘要 / 宗门 / 称号
        txt("功法", 180.0, 164.0, 16, C_DIM);
        txt(
            &format!("《{}》 {}", TECHS[tech].name, TECHS[tech].desc),
            250.0,
            164.0,
            16,
            C_GOLD,
        );
        txt("法宝", 180.0, 200.0, 16, C_DIM);
        if g.life.as_ref().is_none_or(|x| x.relics.is_empty()) {
            txt("尚无法宝，模拟奇遇可寻。", 250.0, 200.0, 14, C_DIM);
        } else {
            txt(
                &format!(
                    "已装 {}/{} · 包内 {}（详情与装卸见「法宝」页签）",
                    g.life
                        .as_ref()
                        .map_or(0, |x| x.equipped.len().min(RELIC_EQUIP_MAX)),
                    RELIC_EQUIP_MAX,
                    g.life.as_ref().map_or(0, |x| x.relics.len())
                ),
                250.0,
                200.0, 14,
                C_CYAN,
            );
        }
        txt("宗门", 180.0, 236.0, 16, C_DIM);
        txt(
            if sect { "青云宗弟子" } else { "无门无派" },
            250.0,
            236.0, 14,
            C_TEXT,
        );
        txt("称号", 180.0, 264.0, 16, C_DIM);
        txt(&format!("『{}』", g.worn_title_of()), 250.0, 264.0, 14, C_GOLD);
        txt(
            "—— 随身行装一览；丹药服用见「丹药」页签，法宝装卸见「法宝」页签 ——",
            180.0,
            320.0,
            14,
            C_DIM,
        );
    }
}

// ================= v1.10.2 离线闭关：出关结算卡 =================
// 独占输入（先例 ad-overlay）：明细列表 + 单「收功」钮。收益已在 resume_life 时
// 入账内存（qi/stones/age），收功即一次 sync_save（去抖兜底 pagehide 亦覆盖）。
fn draw_offline_card(g: &mut Game) {
    let Some(r) = g.offline_card.as_ref() else {
        return;
    };
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, 0.78));
    panel_card(340.0, 160.0, 600.0, 400.0);
    txt_c("离 闭 归 来", V_W * 0.5, 188.0, 24, C_GOLD);
    draw_rectangle(380.0, 208.0, 520.0, 1.5, Color::from_rgba(96, 76, 130, 160));
    txt(
        &format!("现实人间已悄然流逝 {} 载。", r.years),
        380.0,
        226.0, 16,
        C_TEXT,
    );
    txt(
        &format!("闭关吐纳（五折）：修为 +{}", fmt_num(r.qi)),
        380.0,
        260.0, 15,
        C_CYAN,
    );
    txt(
        &format!("洞府灵田岁入：灵石 +{}", fmt_int(r.stones)),
        380.0,
        288.0, 15,
        C_CYAN,
    );
    if r.life_capped {
        txt("寿元将尽——再不归返便要坐化！", 380.0, 320.0, 15, C_RED);
    }
    txt(
        "收益已入账；修为满不破境，周天吐纳可续——点击收功归于现实。",
        380.0,
        352.0, 14,
        C_DIM,
    );
    if button(490.0, 420.0, 300.0, 64.0, "收功", Some("归于现实 · 落盘"), true) {
        g.offline_card = None;
        g.play(crate::sounds::Which::Gain);
        g.log("你收功出关，将离闭所攒尽数归于己身。", C_GOLD);
        g.sync_save();
    }
}

// ================= 轮回烙印 =================
// v1.8.0 B1：十项烙印 5+5 页签化（复用 g.tab，零新字段）。
// 布局（面板 200,80,880,560）：96..132 标题｜150..190 页签｜216..548 行带（5×72）｜592/614 页脚
fn draw_upgrade_panel(g: &mut Game) {
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, 0.6));
    panel_card(200.0, 80.0, 880.0, 560.0);
    txt("轮回烙印", 230.0, 114.0, 24, C_GOLD);
    draw_texture(&crate::art::art().icon_yun, 800.0, 104.0, WHITE);
    txt(&fmt_int(g.meta.dao_yun), 824.0, 114.0, 20, C_GOLD);
    if button(996.0, 96.0, 44.0, 36.0, "×", None, true) {
        g.panel = 0;
        g.play(crate::sounds::Which::Page);
        return;
    }
    draw_rectangle(210.0, 132.0, 840.0, 1.5, Color::from_rgba(96, 76, 130, 160));
    // 页签（原面板头两行说明文字让位给页签，motto 移入页脚）
    let tabs = ["根骨·气数", "机缘·造化"];
    g.tab = g.tab.min(1);
    for (i, t) in tabs.iter().enumerate() {
        let x = 230.0 + i as f32 * 200.0;
        if button(x, 150.0, 190.0, 40.0, t, None, true) {
            g.tab = i as u8;
            g.play(crate::sounds::Which::Page);
        }
        if g.tab == i as u8 {
            draw_rectangle(x, 188.0, 190.0, 3.0, C_GOLD);
        }
    }
    // 行带：每页 5 行 ×72（原 10 行 ×42 一屏摊不下 72 大行，分页后行距翻近一倍）
    let base = if g.tab == 0 { 0usize } else { 5usize };
    for j in 0..5usize {
        let i = base + j;
        if i >= UPGRADES.len() {
            break;
        }
        let y = 216.0 + j as f32 * 72.0;
        let u = &UPGRADES[i];
        let lv = g.meta.upg[i];
        let maxed = lv >= u.max;
        let cost = upg_cost(i, lv);
        txt(u.name, 230.0, y + 12.0, 16, C_GOLD);
        txt(&format!("{}/{}", lv, u.max), 350.0, y + 14.0, 14, C_TEXT);
        txt(u.desc, 420.0, y + 14.0, 14, C_TEXT);
        if maxed {
            txt("圆满", 950.0, y + 14.0, 14, C_DIM);
        } else {
            txt_r(
                &format!("{} 道韵", fmt_int(cost)),
                930.0,
                y + 14.0,
                14,
                C_CYAN,
            );
            if button(
                946.0,
                y + 14.0,
                100.0,
                44.0,
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
    // 页脚（原头部说明收编：motto 归位 + 轮回总计）
    txt(
        "道韵：上一稿的你，留给这一稿的批注——稿退人亡，批注仍在。",
        230.0,
        592.0,
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
        614.0,
        14,
        C_CYAN,
    );
}

// ================= 帮助 =================
// v1.8.0 B2：两页签「玩法」/「操作·技巧」，行距 41→52、字号 16→17（8 行 ×52 起点 200 底 616 < 650）
fn draw_help_panel(g: &mut Game) {
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, 0.6));
    panel_card(140.0, 70.0, 1000.0, 580.0);
    txt("轮回镜·使用指南", 170.0, 104.0, 24, C_GOLD);
    if button(1064.0, 86.0, 44.0, 36.0, "×", None, true) {
        g.panel = 0;
        g.play(crate::sounds::Which::Page);
        return;
    }
    // 页签 @y=134 190×38
    let tabs = ["玩法", "操作·技巧"];
    g.help_tab = g.help_tab.min(1);
    for (i, t) in tabs.iter().enumerate() {
        let x = 170.0 + i as f32 * 200.0;
        if button(x, 134.0, 190.0, 38.0, t, None, true) {
            g.help_tab = i as u8;
            g.play(crate::sounds::Which::Page);
        }
        if g.help_tab == i as u8 {
            draw_rectangle(x, 170.0, 190.0, 3.0, C_GOLD);
        }
    }
    let lines: [&str; 15] = [
        "【轮回镜】耗 1 枚模拟点数，推演“可能的你”的一生，直至死亡。",
        "【模拟前】3 选 1 天赋（仅此一梦生效）；模拟中自动修炼、游历、闭关、突破。",
        "【模拟毕】随机给出数项“造化”，选一项带回现实：修为、功法、法宝、丹药……",
        "【现实】闭关涨修为（寿元-1年）；修为满可突破；清源洞府闭关+25%，天雷峰问笔更稳。",
        // v1.12.1 终战引导：终战门槛（neme_slain>=6 才弹终战卡）的可发现性兜底
        "【终战】现实破境至问笔、九重雷劫全扛，且斩敌链通至封卷——终战卡上落款四结局，书志可重看。",
        "【坊市】灵石购丹药，亦可兑换模拟点数（境界越高越贵）。",
        "【宿敌】血仇之敌每世模拟中皆会遭遇——实力不足则被斩，可逃可死。",
        "【斩敌】在模拟中反杀宿敌（领悟其招）或现实寻仇，皆可得灵石、点数与大笔道韵。",
        "【宿敌不灭】斩其一身，还有更强的幕后之敌接棒，直至……",
        "【现实身死】寿元耗尽则一世归尘，结算大笔道韵；轮回再起，烙印永存。",
        "【轮回烙印】道韵购买永久强化：属性下限、继承选项数、点数返还……",
        "【操作】M 键打开山河图（世界地图），C 键查看角色面板；模拟中空格暂停、数字键调速。",
        "此身虽微，万世可期。道友，请开始你的第一次模拟。",
        // v1.8.0 B2：操作·技巧页新增两行
        "【区域】山河图志五大区域各有专属机制：宗门兴望、大漠秘藏、瘴惑灵芽、冰心澄神、仙屿潮音——初至区域，图鉴有赏。",
        "【炼化】行囊·法宝页可「炼化」：献祭一件法宝为材料，为另一件提升灵纹（攻效与灵性再上一层，至多 +3 级）。",
    ];
    let base = if g.help_tab == 0 { 0usize } else { 7usize };
    let count = if g.help_tab == 0 { 7usize } else { 8usize };
    for j in 0..count {
        let i = base + j;
        let c = if j == 0 { C_GOLD } else { C_TEXT };
        wrap_draw(lines[i], 180.0, 200.0 + j as f32 * 52.0, 16, 920.0, 30.0, c);
    }
}

// ================= 书志 · 终卷四结局（v1.12.1，未竟稿 §9/§12）=================

// 演出页：四结局全屏覆层。问笔终战当帧首读（from_book=false，合页回 Reality）与
// 书志页重读（from_book=true，合页回书志）共用；正文为 ENDINGS 定稿原文摘句
// （B《留白》织机声行为定稿原文通用句，无条件追加——伏笔③的同页人具名分支按
// 护栏裁剪序②降级，若下版同页人工作流落地可再升档）。
// pub：世界地图天雷峰可触发问笔终战（world.rs 复审③b 挂同款独占分支）
pub fn draw_ending_page(g: &mut Game, idx: usize, from_book: bool) {
    let i = idx.min(3);
    let e = &crate::game::ENDINGS[i];
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::from_rgba(8, 5, 16, 242));
    panel_card(190.0, 80.0, 900.0, 560.0);
    txt_c(
        &format!("《{}》 · {}", e.name, e.alias),
        V_W * 0.5,
        130.0,
        32,
        C_GOLD,
    );
    draw_rectangle(280.0, 162.0, 720.0, 1.5, Color::from_rgba(96, 76, 130, 160));
    wrap_draw(e.body, 260.0, 186.0, 17, 760.0, 30.0, C_TEXT);
    for (j, ln) in e.gain.lines().enumerate() {
        txt_c(ln, V_W * 0.5, 412.0 + j as f32 * 26.0, 16, C_CYAN);
    }
    // 合页钮 y470..518：书志重看路径下标题屏底层按钮带（开始 522-576/底排 592-636）
    // 先于本页做命中测试——立即模式下交互矩形不得与其重叠（烙印面板顿悟钮同款布局回避）
    if button(V_W * 0.5 - 130.0, 470.0, 260.0, 48.0, "合上这一页", None, true) {
        g.play(crate::sounds::Which::Page);
        if from_book {
            g.book_view = None;
        } else {
            g.ending_show = None;
        }
    }
}

// 书志页（panel 6）：标题屏「书志」钮进入（结算屏小钮按护栏裁剪序①移除，
// 退稿后回主菜单重看）。
// 达成条目整行可点复用演出页重读；未达成灰行「？？？」+ 引导
// （终战门槛可发现性兜底：neme_slain>=6 才弹卡，玩家须能看见路标）。
fn draw_book_record(g: &mut Game) {
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, 0.6));
    panel_card(140.0, 70.0, 1000.0, 580.0);
    txt("书 志 · 终卷四结局", 170.0, 104.0, 24, C_GOLD);
    if button(1064.0, 86.0, 44.0, 36.0, "×", None, true) {
        g.panel = 0;
        g.book_view = None;
        g.play(crate::sounds::Which::Page);
        return;
    }
    txt(
        "问笔终战（现实破境问笔·九重雷劫全扛，且封卷吏已斩）之后，执笔人递笔——你如何落款？",
        170.0,
        146.0,
        14,
        C_DIM,
    );
    for (i, e) in crate::game::ENDINGS.iter().enumerate() {
        let seen = g.meta.endings_seen >> i & 1 == 1;
        // 行带 4×88 起点 170 末底 514——标题屏底层按钮带（开始 522-576/底排
        // 592-636）先于本页做命中测试，交互矩形必须错开（烙印面板同款布局回避）
        let y = 170.0 + i as f32 * 88.0;
        let (label, sub) = if seen {
            (
                format!("《{}》 · {}", e.name, e.alias),
                e.gain.lines().next().unwrap_or("").to_string(),
            )
        } else if i == 3 {
            (
                "？？？".to_string(),
                "先撕稿留白（后事不详），再来问笔——未完的书，收册人自会看见".to_string(),
            )
        } else {
            (
                "？？？".to_string(),
                "走向问笔终战，卡上自有分说".to_string(),
            )
        };
        if button(170.0, y, 940.0, 80.0, &label, Some(&sub), seen) {
            g.book_view = Some(i); // enabled=seen：未达成行点不开
        }
    }
}
