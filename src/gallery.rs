// v1.16.0 交游图鉴：角色面板第 5 页签（char_tab=4）——同页人收集 + 文狱四 NPC
//
// 结构对账（规格）：
// * 单列滚动列表完全复用人物页（draw_npc_page）语言：滚动状态机 = charpanel::
//   scroll_list_tick 单口（本页传自有 max/step，会话字段仍 g.npc_scroll 系列，跨页
//   由各自 max 钳制收敛）；视口/遮幅/逐行裁剪/整卡剔除逐值同（VIEW 116..616、
//   上遮幅 104..116、下遮幅 616..654）。
// * 收纳口径：已识 = comp_bonds[i]>0 || 当前道侣==该名（运行时纯推导，零额外键）；
//   已识显真名+真立绘（comp_N.png 懒解码）+历世次数，未识显『？？？』+书卷剪影。
// * 好感/结缘状态展示口径与既有设定一致：当前道侣卡显 Life.comp_fav 实时值；
//   历世卡显 comp_bonds 次数；NPC 卡全用既有 meta.npc_fav/npc_ev（好感挂玩家体系
//   不挂具体人名，game.rs apply_eff special17 原注释口径），不加 per-人好感键。
// * 性能红线：整卡剔除（v1.16.2 卡 280/步距 292、视口 500 → 判活区间宽 780：
//   首开 scroll=0 时 ≤2 卡在画、滚动中峰值 ≤3——第 3 张为滚动临时进视口、出视口
//   即被剔除且立绘查询随剔除跳过；LazyTex 懒解码含 None 缓存、纹理句柄缓存复用）。
//   首开最多解码 2 张可见位立绘、严禁预解码全部 10 张（v1.9.24 内存四板斧）；
//   format! 仅限动态数值行且 ≤ 可见卡数（首开 ≤2、滚动峰值 ≤3）。
//   （评审修正 gallery#6：原注释『任意帧 ≤3』失准——吸附静止位与拖动中均为峰值 4）
use crate::charpanel::{
    npc_card_chrome, npc_portrait_box, npc_row_vis, prewarm_zoom_text, scroll_list_tick,
    tale_unlocked, zoom_badge,
};
use crate::data::*;
use crate::draw::bar;
use crate::game::*;
use crate::ui::*;
use macroquad::prelude::*;

// 视口/遮幅与人物页逐值一致（charpanel.rs NPC_VIEW_Y0/Y1 同值，本页独立常量便于对账）
const GAL_VIEW_Y0: f32 = 116.0;
const GAL_VIEW_Y1: f32 = 616.0;
const GAL_CARD_H: f32 = 280.0; // 卡高（v1.16.2：200→280——真机反馈图鉴立绘太小，照人物页 330→430 同款向下延伸）
const GAL_STEP: f32 = 292.0; // 卡高 + 间距 12
const GAL_TOP_PAD: f32 = 40.0; // 内容首卡前留白
// 内容高 4208 = 顶白 40 + 10×292（同页人区）+ 40（分节行）+ 12 + 4×292−12（NPC 区，
// 末卡底 4168）+ 底白 40 → 滚动钳 [0, 3708]（4208 − 视口 500）
const GAL_SECTION_OFF: f32 = 2960.0; // 分节行顶（= TOP_PAD + 10×292）
const GAL_NPC_OFF: f32 = 3012.0; // NPC 首卡顶（= 分节行底 3000 + 12 间隙）
const GAL_SCROLL_MAX: f32 = 3708.0;

// 首开预热语料（PanelAct::Tab(4) 执行帧一次性灌字形，字号档 [14,16,20] 均既有离散档；
// 仅静态标签——动态数值行的 ASCII 数字/符号由菜单期 panel_warm_chars 保底覆盖）
const PREWARM_STRS: &[&str] = &[
    "图鉴",
    "？？？",
    "此生同页",
    "曾同页",
    "世",
    "未识 · 尘封之名",
    "情谊",
    "/100",
    "好感",
    "识人小传",
    "心结已解",
    "有心事待叙",
    "—— 文狱交游 ——",
    "同页人 已识",
    "/10",
    "交游",
    "/4",
    "—— 同页之缘起于模拟，跨世持久。",
    "她的页仍在你书间",
    "缘起模拟，同页之人。她的故事，待你与她共书。",
];

/// 图鉴页首开帧预热（charpanel exec_panel_act Tab(4) 单点调用）
pub fn prewarm_corpus() {
    for s in PREWARM_STRS {
        prewarm_text(s, &[14, 16, 20]);
    }
    for n in COMPANION_NAMES.iter() {
        prewarm_text(n, &[14, 16, 20]);
    }
    for n in NPC_NAMES.iter() {
        prewarm_text(n, &[14, 16, 20]);
    }
    for n in NPC_TIER_NAMES.iter() {
        prewarm_text(n, &[14, 16, 20]);
    }
}

/// 图鉴页（char_tab=4）：纯只读滚动列表，无 &mut 交互（滚动经 scroll_list_tick 单口）。
/// 注意：panel_card 底板由调用方 draw_char_panel 统一绘制（同人物页，本页不重铺）。
pub fn draw_gallery_page(g: &mut Game) {
    // ---------- 滚动状态机（人物页同一单口；图鉴自有 max/step）----------
    // v1.16.1：tap 捕获——已识真图卡立绘点按进鉴赏覆层（draw_zoom_overlay）
    let tap = scroll_list_tick(g, GAL_SCROLL_MAX, GAL_STEP);
    let (mx, my) = crate::cam::mouse();
    let scroll = g.npc_scroll;
    // 当前道侣下标（已识判定的运行时纯推导半边；None = 此生未结缘）
    let cur_comp: Option<usize> = g.life.as_ref().and_then(|l| {
        l.companion
            .as_deref()
            .and_then(|cn| COMPANION_NAMES.iter().position(|&n| n == cn))
    });

    // ---------- 上半部：同页人 10 卡（整卡剔除：首开 ≤3、滚动峰值 ≤4 卡在画，立绘查询随剔除跳过）----------
    for i in 0..COMPANION_NAMES.len() {
        let y = GAL_VIEW_Y0 + GAL_TOP_PAD + i as f32 * GAL_STEP - scroll;
        if y >= GAL_VIEW_Y1 || y + GAL_CARD_H <= GAL_VIEW_Y0 {
            continue;
        }
        draw_gal_comp_card(g, i, y, cur_comp == Some(i));
        // v1.16.1 点立绘放大（已识真图才开；v1.16.2 盒加大：176..396 × y+12..y+268）
        let known = g.meta.comp_bonds[i] > 0 || cur_comp == Some(i);
        if tap
            && known
            && crate::art::art().comp_tex(i).is_some()
            && mx >= 176.0
            && mx <= 396.0
            && my >= y + 12.0
            && my <= y + 268.0
        {
            let z = ZoomArt::Comp(i);
            prewarm_zoom_text(g, &z);
            g.zoom_art = Some(z);
            g.play(crate::sounds::Which::Click);
        }
    }

    // ---------- 分节行（40px，第 10 卡后）----------
    let sec_y = GAL_VIEW_Y0 + GAL_SECTION_OFF - scroll;
    if sec_y + 40.0 > GAL_VIEW_Y0 && sec_y < GAL_VIEW_Y1 && npc_row_vis(sec_y + 14.0) {
        txt_c(
            "—— 文狱交游 ——",
            640.0,
            sec_y + 12.0, 14,
            Color::from_rgba(162, 152, 184, 220), // 同「—— 现 实 ——」表头语言
        );
    }

    // ---------- 下半部：文狱四 NPC 卡（整卡剔除同上）----------
    for i in 0..crate::npc::NPC_COUNT {
        let y = GAL_VIEW_Y0 + GAL_NPC_OFF + i as f32 * GAL_STEP - scroll;
        if y >= GAL_VIEW_Y1 || y + GAL_CARD_H <= GAL_VIEW_Y0 {
            continue;
        }
        draw_gal_npc_card(g, i, y);
        // v1.16.1 点立绘放大（AI 立绘才开；像素兜底不进鉴赏；v1.16.2 盒加大同上）
        if tap
            && crate::art::art().npc_tex(i).is_some()
            && mx >= 176.0
            && mx <= 396.0
            && my >= y + 12.0
            && my <= y + 268.0
        {
            let z = ZoomArt::Npc(i);
            prewarm_zoom_text(g, &z);
            g.zoom_art = Some(z);
            g.play(crate::sounds::Which::Click);
        }
    }

    // ---------- 遮幅（口径同人物页：全不透明面板色，盖住越缘内容）----------
    let mask = Color::from_rgba(22, 16, 38, 255);
    draw_rectangle(150.0, 104.0, 980.0, 12.0, mask);
    draw_rectangle(150.0, 616.0, 980.0, 38.0, mask);

    // ---------- 页脚（y630 固定于视口外，照人物页 411-426；纯推导计数零存档键）----------
    let known = g.meta.comp_bonds.iter().filter(|&&b| b > 0).count();
    let met = g.meta.npc_fav.iter().filter(|&&f| f >= 20).count();
    txt(
        &format!(
            "同页人 已识 {}/10 · 交游 {}/4 —— 同页之缘起于模拟，跨世持久。",
            known, met
        ),
        170.0,
        630.0, 14,
        C_DIM,
    );
}

// 同页人卡（i=0..=9）：已识 rgba(42,26,52,210)+C_PURPLE 边（现道侣卡语言）、
// 未识 rgba(26,22,42,190)+灰边；立绘 220×256 contain 底边对齐（bottom_off 268 = 卡高
// 280 − 内缩 12；v1.16.2 真机反馈图鉴立绘太小，照人物页向下延伸同款加大）。
// 已识显真名+真立绘+历世次数；当前道侣卡另显实时情谊条与小传。
fn draw_gal_comp_card(g: &Game, i: usize, y: f32, cur: bool) {
    let x = 160.0;
    let bonds = g.meta.comp_bonds[i]; // i 循环界内（0..COMPANION_NAMES.len()），恒界内
    let known = bonds > 0 || cur;
    let (bg, border) = if known {
        (
            Color::from_rgba(42, 26, 52, 210),
            C_PURPLE,
        )
    } else {
        (
            Color::from_rgba(26, 22, 42, 190),
            Color::from_rgba(96, 82, 124, 255), // 灰边（面板禁用描边同色）
        )
    };
    npc_card_chrome(x, y, GAL_CARD_H, bg, border);
    // 立绘：已识 comp_N.png 真图（LazyTex 懒解码，仅可见卡触发）；未识书卷剪影
    //（与人物页未结缘兜底同语言）。tex=None 仅剩防御路径（icon_pill 同像素风）
    let tex = if known {
        crate::art::art().comp_tex(i)
    } else {
        None
    };
    // 未识书卷剪影兜底（与人物页未结缘同语言）；or_else 短路——真图命中永不触发
    let portrait = tex.or_else(crate::art::comp_silhouette_tex);
    npc_portrait_box(
        portrait.as_ref(),
        &crate::art::art().icon_pill,
        x + 16.0,
        y,
        220.0,
        256.0,
        268.0,
    );
    // v1.16.1 立绘鉴赏入口徽标（已识真图才显示）；v1.16.2 盒加大 (220,268)
    if known && crate::art::art().comp_tex(i).is_some() {
        zoom_badge(x + 16.0, y, 220.0, 268.0);
    }
    let col = x + 260.0; // 立绘位 220 宽 + 24 间隙（人物页 x+260 同构）
    if npc_row_vis(y + 14.0) {
        if known {
            txt(COMPANION_NAMES[i], col, y + 14.0, 20, C_PURPLE);
        } else {
            txt("？？？", col, y + 14.0, 20, C_DIM);
        }    }
    if npc_row_vis(y + 48.0) {
        if cur {
            txt("此生同页", col, y + 48.0, 14, C_GOLD);
        } else if known {
            txt(&format!("曾同页 {} 世", bonds), col, y + 48.0, 14, C_TEXT);
        } else {
            txt("未识 · 尘封之名", col, y + 48.0, 14, C_DIM);
        }
    }
    if cur {
        // 当前道侣卡：实时情谊（Life.comp_fav 挂玩家体系不挂人名——game.rs 原口径）
        let cfav = g.life.as_ref().map_or(0, |l| l.comp_fav);
        if npc_row_vis(y + 72.0) {
            txt(
                &format!("情谊 {}/{}", cfav, NPC_FAV_MAX),
                col,
                y + 72.0, 14,
                C_TEXT,
            );
        }
        if y + 92.0 >= GAL_VIEW_Y0 && y + 106.0 <= GAL_VIEW_Y1 {
            bar(
                col,
                y + 92.0,
                560.0,
                14.0,
                cfav as f32 / NPC_FAV_MAX as f32,
                C_PURPLE,
            );
        }
        // 识人小传：tale_unlocked 同口径（首段结缘保底），≤2 行不越卡底
        let cseg = tale_unlocked(cfav).max((g.meta.companion_lives >= 1) as usize);
        if cseg > 0 && npc_row_vis(y + 116.0) {
            wrap_draw(COMP_TALES[cseg - 1], col, y + 116.0, 14, 660.0, 22.0, C_TEXT);
        }
    } else if known {
        if npc_row_vis(y + 84.0) {
            txt("她的页仍在你书间", col, y + 84.0, 14, C_DIM);
        }
    } else if npc_row_vis(y + 84.0) {
        txt(
            "缘起模拟，同页之人。她的故事，待你与她共书。",
            col,
            y + 84.0, 14,
            C_DIM,
        );
    }
}

// 文狱 NPC 卡（i=0..=3）：像素/AI 立绘（缺 AI 图回退像素立绘，v1.14.0 语言）、
// 档次、好感条+档次刻度（照人物页 draw_npc_card）、增益行（npc::eff_line 单口）、
// 事件徽标（npc_ev 已抉择→心结已解 / 达标未抉择→有心事待叙）。
// v1.16.2：立绘盒 220×256（bottom_off 268）、文本列 x+260，与同页人卡/人物页同构。
fn draw_gal_npc_card(g: &Game, i: usize, y: f32) {
    let x = 160.0;
    let fav = g.meta.npc_fav[i]; // i 循环界内（0..NPC_COUNT），恒界内
    npc_card_chrome(x, y, GAL_CARD_H, Color::from_rgba(30, 24, 50, 210), C_GOLD_D);
    let pix = crate::art::art()
        .npc_tex(i)
        .or_else(|| crate::art::npc_pixel_tex(i));
    npc_portrait_box(
        pix.as_ref(),
        &crate::art::art().icon_yun,
        x + 16.0,
        y,
        220.0,
        256.0,
        268.0,
    );
    // v1.16.1 立绘鉴赏入口徽标（仅 AI 立绘显示；像素兜底不进鉴赏）；v1.16.2 盒加大 (220,268)
    if crate::art::art().npc_tex(i).is_some() {
        zoom_badge(x + 16.0, y, 220.0, 268.0);
    }
    let col = x + 260.0;
    if npc_row_vis(y + 14.0) {
        txt(NPC_NAMES[i], col, y + 14.0, 20, C_GOLD);
    }
    if npc_row_vis(y + 46.0) {
        txt(crate::npc::tier_name(fav), col, y + 46.0, 14, C_PURPLE);
    }
    if npc_row_vis(y + 68.0) {
        txt(
            &format!("好感 {}/{}", fav, NPC_FAV_MAX),
            col,
            y + 68.0, 14,
            C_TEXT,
        );
    }
    if y + 88.0 >= GAL_VIEW_Y0 && y + 102.0 <= GAL_VIEW_Y1 {
        bar(
            col,
            y + 88.0,
            560.0,
            14.0,
            fav as f32 / NPC_FAV_MAX as f32,
            C_PURPLE,
        );
        // 档次刻度（照人物页：3px / (188,168,224,235)）
        for &t in NPC_FAV_TIERS.iter() {
            let tx = col + t as f32 / NPC_FAV_MAX as f32 * 560.0;
            draw_rectangle(tx - 1.5, y + 84.0, 3.0, 22.0, Color::from_rgba(188, 168, 224, 235));
        }
    }
    if npc_row_vis(y + 120.0) {
        txt(&crate::npc::eff_line(i, fav), col, y + 120.0, 14, C_CYAN);
    }
    if npc_row_vis(y + 150.0) {
        // 事件徽标：抉择即消耗（choose_roption 置位 npc_ev）当拍反映为「心结已解」
        if g.meta.npc_ev[i] {
            txt("心结已解", col, y + 150.0, 14, C_GREEN);
        } else if fav >= NPC_EVENT_FAV {
            txt("有心事待叙", col, y + 150.0, 14, C_GOLD);
        }
    }
}
