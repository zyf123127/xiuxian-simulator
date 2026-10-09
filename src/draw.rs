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

pub fn button(x: f32, y: f32, w: f32, h: f32, label: &str, sub: Option<&str>, enabled: bool) -> bool {
    let (mx, my) = mouse_position();
    let hot = mx >= x && mx <= x + w && my >= y && my <= y + h;
    let base = if !enabled {
        Color::from_rgba(38, 32, 56, 225)
    } else if hot {
        Color::from_rgba(92, 66, 122, 245)
    } else {
        Color::from_rgba(56, 42, 86, 235)
    };
    draw_rectangle(x, y, w, h, base);
    draw_rectangle_lines(x, y, w, h, 2.0, if enabled { C_GOLD_D } else { Color::from_rgba(66, 56, 88, 255) });
    if let Some(s) = sub {
        txt_c(label, x + w * 0.5, y + h * 0.5 - 17.0, 20, if enabled { C_TEXT } else { C_DIM });
        txt_c(s, x + w * 0.5, y + h - 26.0, 13, if enabled { C_DIM } else { Color::from_rgba(110, 104, 120, 255) });
    } else {
        txt_c(label, x + w * 0.5, y + h * 0.5 - 11.0, 18, if enabled { C_TEXT } else { C_DIM });
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

fn panel_card(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, C_PANEL);
    draw_rectangle_lines(x, y, w, h, 2.5, C_GOLD_D);
    draw_rectangle_lines(x + 5.0, y + 5.0, w - 10.0, h - 10.0, 1.0, Color::from_rgba(96, 76, 130, 160));
}

pub fn draw_bg(art: &Art) {
    draw_texture(&art.bg, 0.0, 0.0, WHITE);
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
        let c = if s.gold { Color::new(1.0, 0.85, 0.45, a) } else { Color::new(0.6, 0.9, 1.0, a) };
        draw_circle(s.x, s.y, 2.6, c);
    }
}

pub fn version_mark() {
    txt_r(&format!("v{}", env!("CARGO_PKG_VERSION")), V_W - 8.0, V_H - 20.0, 13, Color::from_rgba(120, 112, 140, 200));
}

fn draw_toast(g: &Game) {
    if let Some((text, t, c)) = &g.toast {
        let a = t.min(1.0).clamp(0.0, 1.0);
        let w = measure(text, 22) + 60.0;
        draw_rectangle(V_W * 0.5 - w * 0.5, 70.0, w, 44.0, Color::new(0.05, 0.03, 0.1, 0.85 * a));
        draw_rectangle_lines(V_W * 0.5 - w * 0.5, 70.0, w, 44.0, 1.5, Color::new(c.r, c.g, c.b, a));
        txt_c(text, V_W * 0.5, 80.0, 20, Color::new(c.r, c.g, c.b, a));
    }
}

// ================= 菜单 =================
pub fn draw_menu(g: &mut Game, art: &Art) {
    draw_bg(art);
    draw_motes(g);
    let t = g.time;
    let glow = 0.75 + 0.25 * (t * 1.2).sin();
    txt_c("凡 人 修 仙", V_W * 0.5, 120.0, 60, Color::new(1.0, 0.85, 0.5, glow));
    txt_c("· 轮 回 模 拟 器 ·", V_W * 0.5, 190.0, 32, C_GOLD);
    txt_c("轮回镜中演一世，造化归身证长生", V_W * 0.5, 240.0, 18, C_DIM);
    draw_cultivator(art, V_W * 0.5, 385.0, 1.2, col_realm(g.meta.best_realm), t, true);
    let bx = V_W * 0.5 - 320.0;
    if button(bx, 522.0, 300.0, 54.0, "开始新的人生", None, true) {
        g.play(crate::sounds::Which::Click);
        g.begin_mortal();
    }
    if g.meta.life.is_some() {
        if button(bx + 320.0, 522.0, 300.0, 54.0, "承接前缘（继续）", None, true) {
            g.play(crate::sounds::Which::Click);
            g.scene = Scene::Reality;
            g.log("轮回镜微亮，前缘未了。", C_TEXT);
        }
    }
    let mlabel = if g.meta.music { "音乐：开" } else { "音乐：关" };
    if button(bx, 592.0, 140.0, 44.0, mlabel, None, true) {
        g.meta.music = !g.meta.music;
        if g.meta.music {
            g.audio.start_bgm();
        } else {
            g.audio.stop_bgm();
        }
        crate::save::save(&g.meta);
    }
    let slabel = if g.meta.sfx { "音效：开" } else { "音效：关" };
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
        &format!("已历 {} 世 · 最高境界 {}境 · 斩杀宿敌 {} · 道韵 {}", g.meta.lives, best, g.meta.neme_slain, fmt_int(g.meta.dao_yun)),
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
    draw_bg(art);
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
    draw_rectangle(376.0, 60.0, V_W - 384.0, PANEL_B - 60.0, Color::from_rgba(14, 10, 26, 215));
    draw_rectangle_lines(376.0, 60.0, V_W - 384.0, PANEL_B - 60.0, 1.5, Color::from_rgba(90, 74, 120, 200));
    txt_c("—— 现 实 ——", 376.0 + (V_W - 384.0) * 0.5, 68.0, 13, Color::from_rgba(110, 100, 130, 220));
    let area_y = 88.0;
    let area_h = PANEL_B - 60.0 - 36.0;
    let line_h = 24.0;
    let max_lines = (area_h / line_h) as usize;
    let start = g.log.len().saturating_sub(max_lines);
    for (i, ll) in g.log[start..].iter().enumerate() {
        txt(&ll.text, 390.0, area_y + i as f32 * line_h, 16, ll.color);
    }

    // 底部行动栏
    let bw = 200.0;
    let gap = 12.8;
    let x0 = 10.0;
    let by = 620.0;
    let bh = 88.0;
    let pts_ok = g.meta.sim_points >= g.sim_cost();
    if button(x0, by, bw, bh, "开始模拟", Some(&format!("推演一生 · {} 点", g.sim_cost())), pts_ok) {
        g.play(crate::sounds::Which::Click);
        g.start_draft();
    }
    if button(x0 + (bw + gap), by, bw, bh, "闭关修炼", Some("一载苦修 · 寿元-1"), true) {
        g.play(crate::sounds::Which::Click);
        g.act_cultivate();
    }
    let can = g.can_break();
    let blabel = g.break_label();
    if button(x0 + (bw + gap) * 2.0, by, bw, bh, &blabel, Some("现实突破 · 不死"), can) {
        g.play(crate::sounds::Which::Click);
        g.act_break();
    }
    if button(x0 + (bw + gap) * 3.0, by, bw, bh, "坊市", Some("丹药 · 点数兑换"), true) {
        g.panel = 1;
        g.play(crate::sounds::Which::Page);
    }
    if button(x0 + (bw + gap) * 4.0, by, bw, bh, "行囊", Some("服用丹药"), true) {
        g.panel = 2;
        g.play(crate::sounds::Which::Page);
    }
    if g.can_revenge() {
        let neme = g.meta.neme.clone().unwrap();
        let p = g.revenge_p();
        if button(x0 + (bw + gap) * 5.0, by, bw, bh, "寻仇", Some(&format!("{:.0}% · {}", p * 100.0, neme.name)), true) {
            g.play(crate::sounds::Which::Click);
            g.act_revenge();
        }
    } else if button(x0 + (bw + gap) * 5.0, by, bw, bh, "宿敌已清", Some("静待新敌"), false) {
    }

    // 面板
    match g.panel {
        1 => draw_shop(g, &l),
        2 => draw_bag(g, &l),
        3 => draw_upgrade_panel(g),
        4 => draw_help_panel(g),
        _ => {}
    }
    draw_toast(g);
    version_mark();
}

fn render_top(g: &Game, l: &Life) {
    draw_rectangle(0.0, 0.0, V_W, TOP_H, Color::from_rgba(16, 11, 30, 245));
    draw_rectangle(0.0, TOP_H - 2.0, V_W, 2.0, C_GOLD_D);
    txt(&format!("第 {} 世 · {}", l.n, realm_full_name(l)), 16.0, 15.0, 20, C_GOLD);
    txt("寿元", 330.0, 10.0, 14, C_DIM);
    bar(374.0, 14.0, 180.0, 14.0, (l.age / l.lifespan) as f32, C_RED);
    txt(&format!("{}/{}", fmt_num(l.age), fmt_num(l.lifespan)), 562.0, 12.0, 14, C_TEXT);
    draw_texture(&crate::art::art().icon_stone, 700.0, 16.0, WHITE);
    txt(&fmt_int(l.stones), 718.0, 15.0, 17, C_CYAN);
    txt("模拟点", 830.0, 15.0, 17, Color::from_rgba(200, 160, 250, 255));
    txt(&format!("●{}", g.meta.sim_points), 896.0, 15.0, 17, C_PURPLE);
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
    draw_cultivator(art, cx, 152.0, 0.80, col_realm(l.realm), g.time, l.seclusion.is_some());
    txt_c(&realm_full_name(l), cx, 242.0, 20, col_realm(l.realm));
    let need = qi_need(l);
    let frac = (l.qi / need) as f32;
    bar(28.0, 274.0, LEFT_W - 40.0, 20.0, frac, C_CYAN);
    let qlabel = format!("修为 {}/{}（{:.0}%）", fmt_num(l.qi), fmt_num(need), frac * 100.0);
    txt_c(&qlabel, cx + 1.0, 277.0, 14, Color::from_rgba(10, 8, 20, 255));
    txt_c(&qlabel, cx, 276.0, 14, C_TEXT);
    let mut y = 312.0;
    let row = |label: &str, y: f32| -> f32 { txt(label, 30.0, y, 15, C_DIM); y + 29.0 };
    y = row("灵根", y);
    let rc = root_color(l.root);
    bar(100.0, y - 25.0, 150.0, 12.0, l.root / 100.0, Color::from_rgba(rc.0, rc.1, rc.2, 255));
    txt(&format!("{} {}", l.root as i32, root_name(l.root)), 262.0, y - 21.0, 14, C_TEXT);
    y = row("道心", y);
    bar(100.0, y - 25.0, 150.0, 12.0, l.dao / 100.0, C_GOLD);
    txt(&format!("{:.0}", l.dao), 262.0, y - 21.0, 14, C_TEXT);
    y = row("气运", y);
    bar(100.0, y - 25.0, 150.0, 12.0, l.luck / 100.0, C_GREEN);
    txt(&format!("{:.0}", l.luck), 262.0, y - 21.0, 14, C_TEXT);
    y = row("体魄", y);
    bar(100.0, y - 25.0, 150.0, 12.0, l.body / 100.0, C_RED);
    txt(&format!("{:.0}", l.body), 262.0, y - 21.0, 14, C_TEXT);
    y = row("战力", y);
    txt(&fmt_num(power(l)), 100.0, y - 21.0, 15, C_TEXT);
    if l.injured > 0 {
        txt(&format!("（重伤{}年）", l.injured), 210.0, y - 21.0, 14, C_RED);
    }
    y = row("修炼", y);
    txt(&format!("每载 +{}", fmt_num(cult_rate(l, &g.meta) * 4.0)), 100.0, y - 21.0, 14, C_TEXT);
    y = row("功法", y);
    txt(TECHS[l.tech].name, 100.0, y - 21.0, 15, C_GOLD);
    y = row("法宝", y);
    if l.relics.is_empty() {
        txt("无", 100.0, y - 21.0, 14, C_DIM);
    } else {
        let names: Vec<&str> = l.relics.iter().take(2).map(|&r| RELICS[r].name).collect();
        let mut s = names.join("·");
        if l.relics.len() > 2 {
            s.push_str(&format!(" 等{}件", l.relics.len()));
        }
        txt(&s, 100.0, y - 21.0, 14, C_CYAN);
    }
    // 宿敌卡片（贴面板底部）
    if show_neme {
        if let Some(n) = &g.meta.neme {
            let ny = 540.0;
            draw_rectangle(24.0, ny, LEFT_W - 32.0, 60.0, Color::from_rgba(60, 22, 30, 220));
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
    txt_c(&format!("已选：{}", if names.is_empty() { "（无）".to_string() } else { names.join("、") }), V_W * 0.5, 138.0, 16, C_GREEN);
    // 卡片
    let n = g.draft.len();
    let cw = 280.0f32;
    let ch = 330.0f32;
    let gap = 30.0f32;
    let total_w = n as f32 * cw + (n as f32 - 1.0) * gap;
    let x0 = V_W * 0.5 - total_w * 0.5;
    let (mx, my) = mouse_position();
    let draft = g.draft.clone();
    for (i, &t) in draft.iter().enumerate() {
        let card = &TALENTS[t];
        let x = x0 + i as f32 * (cw + gap);
        let y = 170.0;
        let selected = g.picked.contains(&t);
        let hot = mx >= x && mx <= x + cw && my >= y && my <= y + ch;
        let (tr, tg, tb) = TIER_COLORS[card.tier as usize];
        let bg = if selected {
            Color::from_rgba(90, 66, 40, 245)
        } else if hot {
            Color::from_rgba(56, 44, 80, 240)
        } else {
            Color::from_rgba(36, 28, 58, 235)
        };
        draw_rectangle(x, y, cw, ch, bg);
        draw_rectangle_lines(x, y, cw, ch, if selected { 3.0 } else { 2.0 }, if selected { C_GOLD } else { Color::from_rgba(tr, tg, tb, 255) });
        // 品级徽标
        draw_rectangle(x + cw * 0.5 - 34.0, y + 18.0, 68.0, 26.0, Color::from_rgba(tr, tg, tb, 60));
        txt_c(TIER_NAMES[card.tier as usize], x + cw * 0.5, y + 22.0, 16, Color::from_rgba(tr, tg, tb, 255));
        txt_c(card.name, x + cw * 0.5, y + 84.0, 24, C_TEXT);
        wrap_draw(card.desc, x + 24.0, y + 120.0, 17, cw - 48.0, 28.0, C_DIM);
        // 小图标占位：修士
        draw_cultivator(art, x + cw * 0.5, y + ch - 60.0, 0.42, Color::from_rgba(tr, tg, tb, 200), g.time + i as f32, true);
        if hot && is_mouse_button_pressed(MouseButton::Left) {
            g.toggle_pick(i);
        }
    }
    let ready = !g.picked.is_empty();
    if button(V_W * 0.5 - 140.0, 540.0, 280.0, 52.0, "沉 入 轮 回 镜", Some("开始此世模拟"), ready) {
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
    if tribbing {
        draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.08, 0.03, 0.15, 0.55));
    }
    draw_motes(g);
    // 顶栏
    draw_rectangle(0.0, 0.0, V_W, TOP_H, Color::from_rgba(26, 12, 36, 250));
    draw_rectangle(0.0, TOP_H - 2.0, V_W, 2.0, C_GOLD_D);
    txt("【模拟】", 16.0, 15.0, 20, C_PURPLE);
    txt(&format!("{}岁 · {}", fmt_num(s.age), realm_full_name(&s)), 110.0, 15.0, 19, C_TEXT);
    bar(500.0, 14.0, 200.0, 14.0, (s.qi / qi_need(&s)) as f32, C_CYAN);
    let ql = format!("修为 {}/{}", fmt_num(s.qi), fmt_num(qi_need(&s)));
    txt_c(&ql, 601.0, 13.0, 13, Color::from_rgba(10, 8, 20, 255));
    txt_c(&ql, 600.0, 12.0, 13, C_TEXT);
    draw_texture(&crate::art::art().icon_stone, 760.0, 16.0, WHITE);
    txt(&fmt_int(s.stones), 778.0, 15.0, 16, C_CYAN);
    // 速度按钮
    let names = ["暂停", "缓", "快", "跳过"];
    for i in 0..4 {
        let on = g.speed == i;
        if button(880.0 + i as f32 * 88.0, 8.0, 80.0, 36.0, names[i], None, true) {
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
    draw_rectangle(8.0, area_y - 8.0, V_W - 16.0, area_h + 12.0, Color::from_rgba(14, 10, 26, 220));
    draw_rectangle_lines(8.0, area_y - 8.0, V_W - 16.0, area_h + 12.0, 1.5, Color::from_rgba(120, 80, 150, 220));
    let start = g.sim_log.len().saturating_sub(max_lines);
    for (i, ll) in g.sim_log[start..].iter().enumerate() {
        txt(&ll.text, 24.0, area_y + 10.0 + i as f32 * line_h, 17, ll.color);
    }
    // 渡劫演出
    if tribbing {
        let t = s.trib.as_ref().unwrap();
        // 雷云
        for i in 0..7 {
            let x = 200.0 + i as f32 * 150.0 + (g.time * 30.0 + i as f32 * 37.0).sin() * 24.0;
            let y = 52.0 + (g.time * 18.0 + i as f32 * 13.0).sin() * 8.0;
            draw_circle(x, y, 52.0, Color::new(0.25, 0.18, 0.4, 0.5));
        }
        if t.flash > 0.02 {
            let _seed = (t.wave as i64 * 7919 + (g.time * 100.0) as i64) as u32;
            let bx = V_W * 0.5 + gen_range(-160.0, 160.0);
            let mut x = bx;
            let mut y = 90.0;
            while y < 460.0 {
                let nx = x + (gen_range(0.0, 1.0) - 0.5) * 90.0;
                let ny = y + 40.0 + gen_range(0.0, 45.0);
                draw_line(x, y, nx, ny, 4.0, Color::new(0.85, 0.92, 1.0, t.flash));
                x = nx;
                y = ny;
            }
            draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.9, 0.95, 1.0, t.flash * 0.2));
        }
        txt_c(&format!("第 {} 重 · 雷 劫", (t.wave + 1).min(9)), V_W * 0.5, 660.0, 30, C_GOLD);
        for i in 0..9 {
            let x = V_W * 0.5 - 180.0 + i as f32 * 45.0;
            let c = if i < t.wave { C_GOLD } else { Color::from_rgba(90, 74, 120, 255) };
            draw_circle(x, 690.0, 8.0, c);
        }
    }
    // 事件弹窗
    if g.modal.is_some() {
        let (title, text, labels) = {
            let card = g.modal.as_ref().unwrap();
            (
                card.title.clone(),
                card.text.clone(),
                card.opts.iter().map(|(l, _)| l.clone()).collect::<Vec<_>>(),
            )
        };
        draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, 0.5));
        panel_card(340.0, 200.0, 600.0, 300.0);
        txt_c(&title, V_W * 0.5, 228.0, 24, C_GOLD);
        draw_rectangle(380.0, 246.0, 520.0, 1.5, Color::from_rgba(96, 76, 130, 160));
        wrap_draw(&text, 380.0, 264.0, 17, 520.0, 28.0, C_TEXT);
        let n = labels.len();
        let bw2 = if n == 1 { 260.0 } else { 250.0 };
        for (i, label) in labels.iter().enumerate() {
            let bx = if n == 1 { V_W * 0.5 - bw2 * 0.5 } else { V_W * 0.5 - bw2 - 8.0 + i as f32 * (bw2 + 16.0) };
            if button(bx, 434.0, bw2, 46.0, label, None, true) {
                g.play(crate::sounds::Which::Click);
                g.choose_option(i);
            }
        }
    }
    draw_toast(g);
    version_mark();
}

// ================= 继承结算 =================
pub fn draw_inherit(g: &mut Game, art: &Art) {
    // 背景保留模拟日志渐隐
    draw_bg(art);
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::from_rgba(10, 6, 20, 235));
    draw_motes(g);
    let Some(sr) = g.sim_result.clone() else { return };
    txt_c("一 世 落 幕 · 造 化 归 身", V_W * 0.5, 56.0, 30, C_GOLD);
    // 左：此生摘要
    panel_card(60.0, 84.0, 420.0, 480.0);
    txt("此 生", 84.0, 116.0, 22, C_GOLD);
    wrap_draw(&sr.cause, 84.0, 146.0, 16, 372.0, 25.0, C_RED);
    txt(&format!("享年 {} 载（历 {} 年）", fmt_num(sr.age), fmt_num(sr.years as f64)), 84.0, 240.0, 16, C_TEXT);
    txt(&format!("最高境界：{}境", REALMS[sr.max_realm.min(14)].name), 84.0, 268.0, 16, C_TEXT);
    txt(&format!("道韵 +{}", fmt_int(sr.total_dy)), 84.0, 296.0, 17, C_GOLD);
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
    txt(&format!("镜灵低语：还可择 {} 项造化", left), 524.0, 116.0, 20, C_PURPLE);
    txt("所选将随神魂带回现实", 524.0, 142.0, 14, C_DIM);
    for (i, &idx) in g.inherit_opts.iter().enumerate() {
        let card = &INHERITS[idx];
        let y = 160.0 + i as f32 * 62.0;
        let (mx, my) = mouse_position();
        let hot = mx >= 520.0 && mx <= 1200.0 && my >= y && my <= y + 54.0;
        draw_rectangle(520.0, y, 680.0, 54.0, if hot { Color::from_rgba(80, 58, 110, 240) } else { Color::from_rgba(44, 34, 70, 235) });
        draw_rectangle_lines(520.0, y, 680.0, 54.0, 1.5, C_GOLD_D);
        txt(&format!("【{}】", card.name), 536.0, y + 8.0, 18, C_GOLD);
        txt(card.desc, 700.0, y + 8.0, 15, C_TEXT);
        txt_c("选此项", 1160.0, y + 8.0, 14, C_CYAN);
        if hot && is_mouse_button_pressed(MouseButton::Left) {
            g.play(crate::sounds::Which::Ascend);
            g.take_inherit(i);
            return;
        }
    }
    if button(V_W * 0.5 - 150.0, 596.0, 300.0, 50.0, "带 着 造 化 醒 来", Some("回到现实"), true) {
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
        txt_r(&format!("总道韵 {}", fmt_int(g.meta.dao_yun)), 1030.0, y, 16, C_DIM);
    }
    if button(V_W * 0.5 - 320.0, 556.0, 300.0, 50.0, "轮 回 再 起", Some("道韵烙印保留"), true) {
        g.play(crate::sounds::Which::Click);
        g.settle = None;
        g.begin_mortal();
    }
    if button(V_W * 0.5 + 20.0, 556.0, 300.0, 50.0, "回到主菜单", None, true) {
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
    // 点数兑换
    let cost = g.exchange_cost();
    draw_rectangle(160.0, 108.0, 960.0, 46.0, Color::from_rgba(44, 30, 66, 220));
    txt("模拟点数", 176.0, 120.0, 17, C_PURPLE);
    txt(&format!("●{}", g.meta.sim_points), 280.0, 120.0, 17, C_PURPLE);
    txt("轮回镜以灵石为引，可无限推演", 360.0, 120.0, 14, C_DIM);
    txt_r(&format!("{} 灵石 / 1点", fmt_int(cost)), 990.0, 120.0, 15, C_CYAN);
    if button(1010.0, 114.0, 94.0, 36.0, "兑换", None, l.stones >= cost) {
        g.play(crate::sounds::Which::Coin);
        g.exchange_points();
        return;
    }
    draw_rectangle(160.0, 168.0, 960.0, 1.5, Color::from_rgba(96, 76, 130, 160));
    for i in 0..6 {
        let y = 186.0 + i as f32 * 74.0;
        let p = &PILLS[i];
        let price = g.pill_price(i);
        draw_texture(&crate::art::art().icon_pill, 160.0, y - 6.0, WHITE);
        txt(p.name, 184.0, y + 4.0, 18, C_GOLD);
        txt(p.desc, 320.0, y + 4.0, 15, C_TEXT);
        txt(&format!("存量 {}", l.pills[i]), 320.0, y + 26.0, 14, C_DIM);
        txt_r(&format!("{} 灵石", fmt_int(price)), 1010.0, y + 8.0, 16, C_CYAN);
        if button(1024.0, y - 6.0, 80.0, 40.0, "购买", None, l.stones >= price) {
            g.play(crate::sounds::Which::Coin);
            g.buy_pill(i);
            return;
        }
    }
    txt("（筑基丹/破境丹/护神丹在突破与渡劫时自动生效）", 180.0, 636.0, 14, C_DIM);
}

// ================= 行囊 =================
fn draw_bag(g: &mut Game, l: &Life) {
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, 0.6));
    panel_card(140.0, 70.0, 1000.0, 580.0);
    txt("行囊", 170.0, 100.0, 26, C_GOLD);
    if button(1064.0, 86.0, 44.0, 36.0, "×", None, true) {
        g.panel = 0;
        g.play(crate::sounds::Which::Page);
        return;
    }
    draw_rectangle(160.0, 126.0, 960.0, 1.5, Color::from_rgba(96, 76, 130, 160));
    txt("功法", 180.0, 152.0, 17, C_DIM);
    txt(&format!("《{}》 {}", TECHS[l.tech].name, TECHS[l.tech].desc), 250.0, 152.0, 16, C_GOLD);
    txt("法宝", 180.0, 180.0, 17, C_DIM);
    if l.relics.is_empty() {
        txt("尚无法宝，模拟奇遇可寻。", 250.0, 180.0, 15, C_DIM);
    } else {
        for (i, &r) in l.relics.iter().enumerate() {
            let col = i % 3;
            let row = i / 3;
            draw_texture(&crate::art::art().icon_sword, 236.0 + col as f32 * 320.0, 168.0 + row as f32 * 26.0, WHITE);
            txt(&format!("【{}】{}", RELICS[r].name, RELICS[r].desc), 256.0 + col as f32 * 320.0, 180.0 + row as f32 * 26.0, 13, C_CYAN);
        }
    }
    draw_rectangle(160.0, 240.0, 960.0, 1.5, Color::from_rgba(96, 76, 130, 160));
    txt("丹药", 180.0, 266.0, 17, C_DIM);
    for i in 0..6 {
        let y = 292.0 + i as f32 * 56.0;
        let p = &PILLS[i];
        txt(p.name, 180.0, y + 4.0, 18, C_GOLD);
        txt(&format!("×{}", l.pills[i]), 300.0, y + 4.0, 16, C_TEXT);
        txt(p.desc, 360.0, y + 4.0, 15, C_TEXT);
        if button(1040.0, y - 8.0, 84.0, 40.0, "服用", None, p.edible && l.pills[i] > 0) {
            g.play(crate::sounds::Which::Gain);
            g.eat_pill(i);
            return;
        }
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
    txt("道韵不灭，烙印永随——纵使身死道消，来世仍享此泽。", 230.0, 156.0, 14, C_DIM);
    for i in 0..UPGRADES.len() {
        let y = 176.0 + i as f32 * 46.0;
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
            txt_r(&format!("{} 道韵", fmt_int(cost)), 930.0, y + 6.0, 14, C_CYAN);
            if button(940.0, y - 2.0, 84.0, 36.0, "顿悟", None, g.meta.dao_yun >= cost) {
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
        "【现实】闭关修炼涨修为（寿元-1年）；修为满可突破，现实突破有镜护持，不会身死。",
        "【坊市】灵石购丹药，亦可兑换模拟点数（境界越高越贵）。",
        "【宿敌】血仇之敌每世模拟中皆会遭遇——实力不足则被斩，可逃可死。",
        "【斩敌】在模拟中反杀宿敌（领悟其招）或现实寻仇，皆可得灵石、点数与大笔道韵。",
        "【宿敌不灭】斩其一身，还有更强的幕后之敌接棒，直至……",
        "【现实身死】寿元耗尽则一世归尘，结算大笔道韵；轮回再起，烙印永存。",
        "【轮回烙印】道韵购买永久强化：属性下限、继承选项数、点数返还……",
        "【操作】模拟中：空格暂停；1/2/3/4 或点按钮调速；“跳过”瞬间演完。",
        "此身虽微，万世可期。道友，请开始你的第一次模拟。",
    ];
    for (i, s) in lines.iter().enumerate() {
        let c = if i == 0 { C_GOLD } else { C_TEXT };
        wrap_draw(s, 180.0, 136.0 + i as f32 * 41.0, 16, 920.0, 24.0, c);
    }
}
