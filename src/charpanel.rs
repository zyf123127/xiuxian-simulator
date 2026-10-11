// v1.5.0 角色面板 v2（底栏「角色」/ C 键，panel=5）：三页签
// 0 总览：左栏立绘与栖身之地，右区属性摘要（法宝只留计数）
// 1 法宝：左 3 装备槽（点击卸下）/ 右包内法宝（点击装备）
// 2 成就·称号：左成就 2 列带进度 / 右称号 17 格（点击佩戴，格 0=自动跟随、格 16=弑神者）
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
        g.char_tab = 0; // v1.5.0：关面板同时归零页签（与 Esc 行为一致）
        g.play(crate::sounds::Which::Page);
        return;
    }
    // ---------- 页签（v1.6.0 D：追加第 4 页「人物」） ----------
    let tabs = ["总览", "法宝", "成就·称号", "人物"];
    for (i, t) in tabs.iter().enumerate() {
        let x = 300.0 + i as f32 * 150.0;
        if button(x, 74.0, 140.0, 34.0, t, None, true) {
            g.char_tab = i as u8;
            g.play(crate::sounds::Which::Page);
        }
        if g.char_tab == i as u8 {
            draw_rectangle(x, 108.0, 140.0, 3.0, C_GOLD);
        }
    }
    match g.char_tab {
        1 => draw_relic_page(g, &l),
        2 => draw_ach_title_page(g, &l),
        3 => draw_npc_page(g, &l), // v1.6.0 D：好感度页
        _ => draw_overview(g, &l),
    }
}

// ---------- 页 3：人物（v1.6.0 D 好感度：长老 / 商人 / 道侣 三卡 + 赠礼） ----------
fn draw_npc_page(g: &mut Game, l: &Life) {
    for i in 0..2usize {
        let y = 120.0 + i as f32 * 170.0; // 卡 960×160 @y=120/290
        let fav = g.meta.npc_fav[i];
        let cost = crate::npc::gift_cost(fav);
        draw_rectangle(160.0, y, 960.0, 160.0, Color::from_rgba(30, 24, 50, 210));
        draw_rectangle_lines(160.0, y, 960.0, 160.0, 2.0, C_GOLD_D);
        draw_texture(&crate::art::art().icon_yun, 184.0, y + 18.0, WHITE);
        txt(NPC_NAMES[i], 230.0, y + 22.0, 20, C_GOLD);
        txt_c(crate::npc::tier_name(fav), 420.0, y + 24.0, 15, C_PURPLE);
        txt(
            &format!("好感 {}/{}", fav, NPC_FAV_MAX),
            500.0,
            y + 24.0,
            15,
            C_TEXT,
        );
        bar(230.0, y + 56.0, 340.0, 14.0, fav as f32 / NPC_FAV_MAX as f32, C_PURPLE);
        // 档次刻度
        for &th in NPC_FAV_TIERS.iter() {
            let tx = 230.0 + th as f32 / NPC_FAV_MAX as f32 * 340.0;
            draw_rectangle(tx - 1.0, y + 54.0, 2.0, 18.0, Color::from_rgba(96, 76, 130, 200));
        }
        wrap_draw(NPC_DESC[i], 230.0, y + 86.0, 14, 600.0, 22.0, C_DIM);
        // 增益一行
        let eff = if i == crate::npc::NPC_ELDER {
            format!("当前：宗门任务收益 ×{:.2}", crate::npc::elder_quest_mult(fav))
        } else {
            format!("当前：坊市丹价 ×{:.2}", crate::npc::merchant_disc(fav))
        };
        txt(&eff, 230.0, y + 134.0, 14, C_CYAN);
        if button(920.0, y + 60.0, 150.0, 56.0, "赠礼", Some(&format!("{} 灵石", fmt_int(cost))), l.stones >= cost)
        {
            crate::npc::gift_npc(g, i);
            return;
        }
    }
    // 第三卡：道侣
    let y = 460.0;
    let cfav = l.comp_fav;
    draw_rectangle(160.0, y, 960.0, 160.0, Color::from_rgba(42, 26, 52, 210));
    draw_rectangle_lines(160.0, y, 960.0, 160.0, 2.0, C_PURPLE);
    draw_texture(&crate::art::art().icon_pill, 184.0, y + 18.0, WHITE);
    match &l.companion {
        Some(cn) => {
            txt(cn, 230.0, y + 22.0, 20, C_PURPLE);
            txt_c(crate::npc::tier_name(cfav), 420.0, y + 24.0, 15, C_PURPLE);
            txt(
                &format!("情谊 {}/{}", cfav, NPC_FAV_MAX),
                500.0,
                y + 24.0,
                15,
                C_TEXT,
            );
            bar(230.0, y + 56.0, 340.0, 14.0, cfav as f32 / NPC_FAV_MAX as f32, C_PURPLE);
            txt(
                &format!(
                    "当前：双修加成 ×{:.2}（结缘起步+{:.0}%，只升不降）",
                    crate::npc::comp_cult_mult(cfav),
                    (crate::npc::comp_cult_mult(COMPANION_BOND_FAV) - 1.0) * 100.0
                ),
                230.0,
                y + 86.0,
                14,
                C_CYAN,
            );
            txt(
                "情谊随赠礼事件与遗泽积累；更换道侣亦不忘旧谊。",
                230.0,
                y + 112.0,
                14,
                C_DIM,
            );
        }
        None => {
            txt("道侣 · 此生未结缘", 230.0, y + 22.0, 20, C_DIM);
            txt(
                "模拟中「良缘天定」事件可选执子之手；遗泽亦可携道侣魂归现实。",
                230.0,
                y + 56.0,
                14,
                C_DIM,
            );
            txt(
                &format!(
                    "结缘起步情谊 {}（双修 +{:.0}%），道侣赠礼事件再 +{}。",
                    COMPANION_BOND_FAV,
                    (crate::npc::comp_cult_mult(COMPANION_BOND_FAV) - 1.0) * 100.0,
                    COMPANION_GIFT_FAV
                ),
                230.0,
                y + 84.0,
                14,
                C_CYAN,
            );
        }
    }
    txt(
        "好感随交往与赠礼增长，跨世持久（随轮回烙印存档）；赠礼价随交情水涨船高。",
        170.0,
        636.0,
        13,
        C_DIM,
    );
}

// ---------- 页 0：总览 ----------
fn draw_overview(g: &mut Game, l: &Life) {
    // ---------- 左栏：立绘 / 境界 / 修为 / 寿元 / 所在 / 宗门 / 洞府 ----------
    draw_cultivator(
        crate::art::art(),
        305.0,
        250.0,
        0.75,
        col_realm(l.realm),
        g.time,
        l.seclusion.is_some(),
        portrait_tier(l.realm),
    );
    txt(&realm_full_name(l), 170.0, 345.0, 20, col_realm(l.realm));
    txt_c(&format!("『{}』", g.worn_title_of()), 305.0, 370.0, 14, C_GOLD);
    // 破境条（修为）
    let need = qi_need(l);
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
    // v1.5.0 洞府经营一览
    txt("洞府：", 170.0, 572.0, 15, C_DIM);
    txt(
        &format!(
            "{}{} · {}{} · {}{}",
            DONGFU_FACS[0],
            g.meta.dongfu[0].clamp(0, 3),
            DONGFU_FACS[1],
            g.meta.dongfu[1].clamp(0, 3),
            DONGFU_FACS[2],
            g.meta.dongfu[2].clamp(0, 3)
        ),
        230.0,
        572.0,
        14,
        C_CYAN,
    );

    // ---------- 右区左列：四维 / 功法 / 法宝摘要 / 丹药 ----------
    let col1 = 460.0;
    let rows: [(&str, f32, (u8, u8, u8), f32); 4] = [
        ("灵根", l.root, root_color(l.root), l.root / 100.0),
        ("道心", l.dao, (240, 200, 110), l.dao / 100.0),
        ("气运", l.luck, (140, 230, 150), l.luck / 100.0),
        ("体魄", l.body, (240, 120, 110), l.body / 100.0),
    ];
    for (i, (name, v, (r, gc, b), f)) in rows.iter().enumerate() {
        // v1.5.0 终局修复：行起点 104→118，为页签栏（y=74 高 34，底边 108）让位，
        // 后续行 30px 步进与功法/法宝/丹药行相应顺延 14px
        let y = 118.0 + i as f32 * 30.0;
        txt(name, col1, y, 14, C_DIM);
        bar(col1 + 45.0, y + 2.0, 150.0, 12.0, *f, Color::from_rgba(*r, *gc, *b, 255));
        txt(&format!("{} {}", *v as i32, root_name(*v)), col1 + 205.0, y, 13, C_TEXT);
    }
    txt("功法", col1, 246.0, 14, C_DIM);
    txt(
        &format!("《{}》", TECHS[l.tech].name),
        col1 + 45.0,
        246.0,
        15,
        C_GOLD,
    );
    txt(TECHS[l.tech].desc, col1 + 45.0, 268.0, 13, C_DIM);
    txt("法宝", col1, 304.0, 14, C_DIM);
    // v1.5.0：只留计数摘要（装备详情见法宝页），宽约 120px < 295px 可用
    if l.relics.is_empty() {
        txt("无", col1 + 45.0, 304.0, 14, C_DIM);
    } else {
        txt(
            &format!(
                "已装 {}/{} · 包内 {}",
                l.equipped.len().min(RELIC_EQUIP_MAX),
                RELIC_EQUIP_MAX,
                l.relics.len()
            ),
            col1 + 45.0,
            304.0,
            14,
            C_CYAN,
        );
    }
    txt("丹药", col1, 344.0, 14, C_DIM);
    {
        let owned: Vec<String> = PILLS
            .iter()
            .enumerate()
            .filter(|(i, _)| l.pills[*i] > 0)
            .map(|(i, p)| format!("{}×{}", p.name, l.pills[i]))
            .collect();
        if owned.is_empty() {
            txt("无", col1 + 45.0, 344.0, 14, C_DIM);
        } else {
            let s = owned.join(" · ");
            wrap_draw(&s, col1 + 45.0, 344.0, 14, 300.0, 22.0, C_TEXT);
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
            fmt_num(power(l))
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

// ---------- 页 1：法宝（左 3 槽点击卸下 / 右包内法宝点击装备） ----------
fn draw_relic_page(g: &mut Game, l: &Life) {
    // 命中检测（手动，与天赋卡/继承卡先例一致；button 会重铺底色故不用）
    let (mx, my) = crate::cam::mouse();
    let clicked = is_mouse_button_pressed(MouseButton::Left);
    // 左：3 个装备槽
    let slot_ys = [180.0f32, 254.0, 358.0];
    let equipped = &l.equipped;
    txt("装备槽（点击卸下）", 180.0, 152.0, 16, C_GOLD);
    for s in 0..RELIC_EQUIP_MAX {
        let y = slot_ys[s];
        let filled = s < equipped.len();
        let r = if filled { equipped[s] } else { usize::MAX };
        draw_rectangle(
            180.0,
            y,
            380.0,
            64.0,
            if filled {
                Color::from_rgba(70, 52, 24, 210)
            } else {
                Color::from_rgba(30, 24, 50, 200)
            },
        );
        draw_rectangle_lines(
            180.0,
            y,
            380.0,
            64.0,
            2.0,
            if filled { C_GOLD_D } else { Color::from_rgba(70, 60, 96, 255) },
        );
        if filled {
            draw_texture(&crate::art::art().icon_sword, 196.0, y + 18.0, WHITE);
            txt(&format!("【{}】", RELICS[r].name), 228.0, y + 14.0, 17, C_GOLD);
            txt(RELICS[r].desc, 228.0, y + 40.0, 13, C_TEXT);
            txt_c("卸下", 510.0, y + 24.0, 14, C_RED);
        } else {
            txt(
                &format!("法宝槽 {}（空）", s + 1),
                228.0,
                y + 24.0,
                15,
                C_DIM,
            );
        }
        if filled
            && clicked
            && mx >= 180.0
            && mx <= 560.0
            && my >= y
            && my <= y + 64.0
        {
            g.play(crate::sounds::Which::Click);
            g.unequip_relic(r);
            return;
        }
    }
    txt(
        &format!(
            "已装 {}/{}；未装备的法宝保有 {:.0}% 灵性。",
            equipped.len().min(RELIC_EQUIP_MAX),
            RELIC_EQUIP_MAX,
            RELIC_UNEQUIP_FRAC * 100.0
        ),
        180.0,
        446.0,
        13,
        C_DIM,
    );

    // 右：包内法宝 8 行（点击装备）
    txt("包内法宝（点击装备）", 600.0, 152.0, 16, C_GOLD);
    for (k, rl) in RELICS.iter().enumerate() {
        let y = 180.0 + k as f32 * 55.0;
        let owned = l.relics.contains(&k);
        let on = equipped.contains(&k);
        let can = owned && !on && equipped.len() < RELIC_EQUIP_MAX;
        draw_rectangle(
            600.0,
            y,
            520.0,
            48.0,
            if on {
                Color::from_rgba(70, 52, 24, 210)
            } else if owned {
                Color::from_rgba(44, 34, 70, 220)
            } else {
                Color::from_rgba(26, 22, 42, 200)
            },
        );
        draw_rectangle_lines(
            600.0,
            y,
            520.0,
            48.0,
            1.5,
            if on {
                C_GOLD_D
            } else {
                Color::from_rgba(70, 60, 96, 255)
            },
        );
        txt(
            &format!("【{}】", rl.name),
            616.0,
            y + 6.0,
            16,
            if owned { C_GOLD } else { C_DIM },
        );
        txt(
            rl.desc,
            780.0,
            y + 8.0,
            13,
            if owned { C_TEXT } else { C_DIM },
        );
        if on {
            txt_c("已装备", 1080.0, y + 8.0, 14, C_GOLD);
        } else if !owned {
            txt_c("未得", 1080.0, y + 8.0, 14, C_DIM);
        } else if !can {
            txt_c("槽已满", 1080.0, y + 8.0, 14, C_RED);
        } else {
            txt_c("装备", 1080.0, y + 8.0, 14, C_CYAN);
        }
        if can && clicked && mx >= 600.0 && mx <= 1120.0 && my >= y && my <= y + 48.0 {
            g.play(crate::sounds::Which::Ascend);
            g.equip_relic(k);
            return;
        }
    }
}

// ---------- 页 2：成就·称号 ----------
// 称号格→值映射（格/earned/入参三处共用，勿改出 off-by-one）：
//   格 0    → 值 -1（自动跟随，恒可选）
//   格 1..=15 → 值 c-1（0..=14 境界称号，earned = best_realm >= c-1）
//   格 16   → 值 15（弑神者，earned = neme_slain >= 3）
fn title_cell(c: usize) -> (&'static str, i32, bool) {
    let g_name = |v: usize| realm_title(v);
    match c {
        0 => ("自动跟随", -1, true),
        16 => ("弑神者", 15, false),
        c => (g_name(c - 1), (c - 1) as i32, false),
    }
}

fn draw_ach_title_page(g: &mut Game, l: &Life) {
    // 左：成就 2 列 × 15 行
    txt("成就", 170.0, 118.0, 18, C_GOLD);
    for i in 0..ACHS.len() {
        let col = i / 15;
        let row = i % 15;
        let x = 170.0 + col as f32 * 290.0;
        let y = 150.0 + row as f32 * 32.0;
        let done = g.meta.ach[i];
        txt(
            &format!("{} {}", if done { "■" } else { "□" }, ACHS[i].name),
            x,
            y,
            14,
            if done { C_GOLD } else { C_DIM },
        );
        if let Some((cur, goal)) = g.ach_progress(i) {
            txt_r(&format!("{}/{}", cur, goal), x + 272.0, y, 11, C_DIM);
        }
    }

    // 右：称号 3 列 × 6 行 17 格
    txt("称号（点击佩戴）", 760.0, 118.0, 18, C_GOLD);
    let worn = g.meta.worn_title;
    let neme_slain = g.meta.neme_slain;
    let best_realm = g.meta.best_realm;
    for c in 0..17usize {
        let (name, val, _) = title_cell(c);
        let earned = match c {
            0 => true,
            16 => neme_slain >= 3,
            _ => best_realm >= (c - 1),
        };
        let col = c % 3;
        let row = c / 3;
        let x = 760.0 + col as f32 * 120.0;
        let y = 150.0 + row as f32 * 78.0;
        let is_worn = worn == val;
        draw_rectangle(
            x,
            y,
            112.0,
            64.0,
            if is_worn {
                Color::from_rgba(70, 52, 24, 235)
            } else if earned {
                Color::from_rgba(44, 34, 70, 220)
            } else {
                Color::from_rgba(26, 22, 42, 200)
            },
        );
        draw_rectangle_lines(
            x,
            y,
            112.0,
            64.0,
            if is_worn { 2.5 } else { 1.5 },
            if is_worn {
                C_GOLD
            } else if earned {
                C_GOLD_D
            } else {
                Color::from_rgba(66, 56, 88, 255)
            },
        );
        txt_c(
            name,
            x + 56.0,
            y + 14.0,
            15,
            if earned { C_TEXT } else { C_DIM },
        );
        txt_c(
            if is_worn {
                "佩戴中"
            } else if earned {
                "可佩戴"
            } else {
                "未获得"
            },
            x + 56.0,
            y + 40.0,
            12,
            if is_worn {
                C_GOLD
            } else if earned {
                C_CYAN
            } else {
                C_DIM
            },
        );
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = crate::cam::mouse();
            if mx >= x && mx <= x + 112.0 && my >= y && my <= y + 64.0 {
                g.set_worn_title(val);
                return;
            }
        }
    }
    // 页脚提示（口径：称号解锁按历代最高境界判定，与总览页「生涯」的本世口径区分）
    txt(
        &format!(
            "历代最高 {}境 · 斩杀宿敌 {} · 「自动跟随」随最强境界实时变化",
            REALMS[best_realm.min(14)].name,
            neme_slain
        ),
        170.0,
        636.0,
        13,
        C_DIM,
    );
    let _ = l; // 保留参数以便未来扩展（当前页签只读 meta）
}
