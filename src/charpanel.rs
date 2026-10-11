// v1.4.0 角色面板（底栏「角色」/ C 键，panel=5）：左栏立绘与栖身之地，右区属性总览
use crate::art::draw_cultivator;
use crate::data::*;
use crate::draw::{bar, bar_glow, button, panel_card};
use crate::game::*;
use crate::ui::*;
use macroquad::prelude::*;

pub fn draw_char_panel(g: &mut Game) {
    let Some(l) = g.life.clone() else { return };
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, 0.6));
    panel_card(140.0, 60.0, 1000.0, 600.0);
    txt("角色", 170.0, 92.0, 26, C_GOLD);
    if button(1064.0, 74.0, 44.0, 36.0, "×", None, true) {
        g.panel = 0;
        g.play(crate::sounds::Which::Page);
        return;
    }

    // ---------- 左栏：立绘 / 境界 / 修为 / 寿元 / 所在 / 宗门 ----------
    draw_cultivator(
        crate::art::art(),
        305.0,
        250.0,
        0.75,
        col_realm(l.realm),
        g.time,
        l.seclusion.is_some(),
    );
    txt(&realm_full_name(&l), 170.0, 345.0, 20, col_realm(l.realm));
    txt_c(&format!("『{}』", g.title_of()), 305.0, 370.0, 14, C_GOLD);
    // 破境条（修为）
    let need = qi_need(&l);
    let frac = (l.qi / need) as f32;
    txt("修为", 170.0, 392.0, 14, C_DIM);
    bar_glow(220.0, 394.0, 180.0, 14.0, frac, C_CYAN, g.time);
    txt(&format!("{:.0}%", frac * 100.0), 412.0, 392.0, 13, C_TEXT);
    // 寿元条
    txt("寿元", 170.0, 452.0, 14, C_DIM);
    bar(
        220.0,
        454.0,
        180.0,
        14.0,
        (l.age / l.lifespan) as f32,
        C_RED,
    );
    txt(
        &format!("{}/{} 载", fmt_num(l.age), fmt_num(l.lifespan)),
        412.0,
        452.0,
        13,
        C_TEXT,
    );
    // 所在
    txt("所在：", 170.0, 500.0, 16, C_DIM);
    txt(
        LOCATIONS[l.loc.min(LOCATIONS.len() - 1)].name,
        230.0,
        500.0,
        16,
        C_GOLD,
    );
    // 宗门 + 贡献
    txt("宗门：", 170.0, 524.0, 15, C_DIM);
    txt(
        if l.sect { "青云宗弟子" } else { "无门无派" },
        230.0,
        524.0,
        15,
        C_TEXT,
    );
    txt("贡献：", 170.0, 548.0, 15, C_DIM);
    txt(
        &fmt_int(g.meta.sect_contrib as i64),
        230.0,
        548.0,
        15,
        C_GOLD,
    );

    // ---------- 右区左列：四维 / 功法 / 法宝 / 丹药 ----------
    let col1 = 460.0;
    let rows: [(&str, f32, (u8, u8, u8), f32); 4] = [
        ("灵根", l.root, root_color(l.root), l.root / 100.0),
        ("道心", l.dao, (240, 200, 110), l.dao / 100.0),
        ("气运", l.luck, (140, 230, 150), l.luck / 100.0),
        ("体魄", l.body, (240, 120, 110), l.body / 100.0),
    ];
    for (i, (name, v, (r, gc, b), f)) in rows.iter().enumerate() {
        let y = 104.0 + i as f32 * 30.0;
        txt(name, col1, y, 14, C_DIM);
        bar(col1 + 45.0, y + 2.0, 150.0, 12.0, *f, Color::from_rgba(*r, *gc, *b, 255));
        txt(&format!("{} {}", *v as i32, root_name(*v)), col1 + 205.0, y, 13, C_TEXT);
    }
    txt("功法", col1, 232.0, 14, C_DIM);
    txt(
        &format!("《{}》", TECHS[l.tech].name),
        col1 + 45.0,
        232.0,
        15,
        C_GOLD,
    );
    txt(TECHS[l.tech].desc, col1 + 45.0, 254.0, 13, C_DIM);
    txt("法宝", col1, 290.0, 14, C_DIM);
    if l.relics.is_empty() {
        txt("无", col1 + 45.0, 290.0, 14, C_DIM);
    } else {
        let names: Vec<&str> = l.relics.iter().take(2).map(|&r| RELICS[r].name).collect();
        let mut s = names.join("·");
        if l.relics.len() > 2 {
            s.push_str(&format!(" 等{}件", l.relics.len()));
        }
        txt(&s, col1 + 45.0, 290.0, 14, C_CYAN);
    }
    txt("丹药", col1, 330.0, 14, C_DIM);
    {
        let owned: Vec<String> = PILLS
            .iter()
            .enumerate()
            .filter(|(i, _)| l.pills[*i] > 0)
            .map(|(i, p)| format!("{}×{}", p.name, l.pills[i]))
            .collect();
        if owned.is_empty() {
            txt("无", col1 + 45.0, 330.0, 14, C_DIM);
        } else {
            let s = owned.join(" · ");
            wrap_draw(&s, col1 + 45.0, 330.0, 14, 300.0, 22.0, C_TEXT);
        }
    }

    // ---------- 右区右列：天赋 / 宿敌 / 道侣 / 成就 / 生涯 ----------
    let col2 = 800.0;
    txt("天赋", col2, 104.0, 14, C_DIM);
    if l.talents.is_empty() {
        txt("（无）", col2 + 45.0, 104.0, 14, C_DIM);
    } else {
        for (i, &t) in l.talents.iter().take(6).enumerate() {
            let (tr, tg, tb) = TIER_COLORS[TALENTS[t].tier as usize];
            txt(
                TALENTS[t].name,
                col2 + 45.0,
                104.0 + i as f32 * 24.0,
                14,
                Color::from_rgba(tr, tg, tb, 255),
            );
        }
        if l.talents.len() > 6 {
            txt(
                &format!("等 {} 项", l.talents.len()),
                col2 + 45.0,
                104.0 + 6.0 * 24.0,
                13,
                C_DIM,
            );
        }
    }
    txt("宿敌", col2, 290.0, 14, C_RED);
    match &g.meta.neme {
        Some(n) => txt(
            &format!("{}（胜率 {:.0}%）", n.name, g.revenge_p() * 100.0),
            col2 + 45.0,
            290.0,
            14,
            C_TEXT,
        ),
        None => txt("无", col2 + 45.0, 290.0, 14, C_DIM),
    };
    txt("道侣", col2, 322.0, 14, C_PURPLE);
    txt(
        l.companion.as_deref().unwrap_or("此生未结缘"),
        col2 + 45.0,
        322.0,
        14,
        C_TEXT,
    );
    txt(
        &format!("历世结缘 {} 次", g.meta.companion_lives),
        col2 + 45.0,
        344.0,
        13,
        C_DIM,
    );
    txt("成就", col2, 380.0, 14, C_GOLD);
    txt(
        &format!(
            "{}/{}",
            g.meta.ach.iter().filter(|&&x| x).count(),
            g.meta.ach.len()
        ),
        col2 + 45.0,
        380.0,
        15,
        C_TEXT,
    );
    // 生涯两行
    draw_rectangle(col2, 440.0, 310.0, 1.0, Color::from_rgba(96, 76, 130, 160));
    txt(
        &format!(
            "生涯：最高 {}境 · 战力 {}",
            REALMS[l.max_realm.min(14)].name,
            fmt_num(power(&l))
        ),
        col2,
        480.0,
        14,
        C_TEXT,
    );
    txt(
        &format!(
            "斩敌 {} · 历战 {} 场 · 重伤 {} 年",
            l.wins, l.fights, l.injured
        ),
        col2,
        506.0,
        14,
        C_DIM,
    );
}
