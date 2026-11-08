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
// v1.16.10 布局重排（追加分节不挪既有段，向下延伸）：
//   同页人区  40 .. 2948（顶白 40 + 10×292−12）
//   分节行    2960 .. 3000（文狱交游）
//   NPC 区    3012 .. 4168（4×292−12）
//   分节行    4180 .. 4220（遭遇手札，v1.16.10 新增）
//   事件行    4228 起，每行 52（高 46 + 间距 6）× EVENTS.len() 行
//   尾白 40；滚动钳 = 4228 + N×52 + 40 − 500（视口），draw_gallery_page 内按
//   EVENTS.len() 现算（扩池自动跟随，硬编码会漂——4208/3708 旧账教训）
const GAL_SECTION_OFF: f32 = 2960.0; // 分节行顶（= TOP_PAD + 10×292）
const GAL_NPC_OFF: f32 = 3012.0; // NPC 首卡顶（= 分节行底 3000 + 12 间隙）
const GAL_EV_SEC_OFF: f32 = 4180.0; // 遭遇手札分节行顶（= NPC 末卡底 4168 + 12）
const GAL_EV_OFF: f32 = 4228.0; // 事件首行顶（= 分节行底 4220 + 8）
const GAL_EV_H: f32 = 46.0; // 事件行卡高（纯文本两行：标题/简一行）
const GAL_EV_STEP: f32 = 52.0; // 行高 + 间距 6

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

// 遭遇手札分节行/徽标/计数静态串（v1.16.10）：首开帧一次灌字形防滚动期栅格化
// 触发图集整图重传（照 prewarm_corpus 同款事故教训）。计数串只备 ×1..=9——
// 实际遭遇同卡 ≥10 次属长尾，走 format! 兜底（≤ 可见行数，页面性能红线同 gallery#6 口径）
const EV_NOTE_PREWARM: &[&str] = &[
    "—— 遭遇手札 ——",
    "未遇",
    "？？？",
    "稀有",
    "链起",
    "链段",
    "伴随",
    "章节",
    "老年",
    "宿敌",
    "遭遇",
    "×1",
    "×2",
    "×3",
    "×4",
    "×5",
    "×6",
    "×7",
    "×8",
    "×9",
];

/// 遭遇手札首开帧预热（exec_panel_act Tab(4)/指引 Gallery 跳转同口调用）：静态串
/// 全量 + 已遇事件的标题/简介（未遇不灌——全部 118 条灌齐有图集余量风险，且未遇
/// 行只显？？？；收录随游玩渐进，预热开销与之同长）
pub fn prewarm_event_book(g: &Game) {
    for s in EV_NOTE_PREWARM {
        prewarm_text(s, &[14]);
    }
    for (i, &c) in g.meta.ev_cnt.iter().enumerate() {
        if c > 0 {
            if let Some(n) = EVENT_NAMES.get(i) {
                prewarm_text(n, &[14]);
            }
            if let Some(b) = EVENT_BRIEF.get(i) {
                prewarm_text(b, &[14]);
            }
        }
    }
}

/// 事件分类徽标单口（纯推导零新表）：伴随/章节/老年/宿敌按门控白名单与特定 id，
/// 链段=EVENTS 权重 0（仅链内直取），链起=各链首段（弹卡即建链的那条），
/// 稀有=EVENTS 权重 w∈1..=RARE_W（与 build_event card.rare 同源），余者=遭遇。
/// 返回 (文案, 色)；id 越界落 ("遭遇", C_DIM) 防御（panic=abort 纪律）。
pub fn event_badge(id: usize) -> (&'static str, Color) {
    // 越界防御：表外 id 落「遭遇」，绝不把 w=0 哨兵误判成链段（panic=abort 纪律）
    let w = match EVENTS.get(id) {
        Some(&(_, w, _, _)) => w,
        None => return ("遭遇", C_DIM),
    };
    if COMPANION_EVENTS.contains(&id) {
        return ("伴随", C_GOLD_D);
    }
    match id {
        74 | 75 => return ("章节", C_GOLD),
        88 | 89 => return ("老年", C_GOLD_D),
        110 | 111 => return ("宿敌", C_RED),
        44 | 47 | 50 | 76 | 79 | 85 | 112 | 115 => return ("链起", C_CYAN),
        _ => {}
    }
    if w == 0 {
        return ("链段", C_CYAN);
    }
    if w <= crate::data::RARE_W {
        return ("稀有", C_PURPLE);
    }
    ("遭遇", C_DIM)
}

// 计数串单口：×1..=9 静态查表零分配（与 EV_NOTE_PREWARM 对齐），≥10 返回空串由
// 调用方走 format! 兜底（≤ 可见行数；封顶 255）
fn ev_cnt_str(c: u8) -> &'static str {
    match c {
        1 => "×1",
        2 => "×2",
        3 => "×3",
        4 => "×4",
        5 => "×5",
        6 => "×6",
        7 => "×7",
        8 => "×8",
        9 => "×9",
        _ => "",
    }
}

/// 图鉴页（char_tab=4）：纯只读滚动列表，无 &mut 交互（滚动经 scroll_list_tick 单口）。
/// 注意：panel_card 底板由调用方 draw_char_panel 统一绘制（同人物页，本页不重铺）。
pub fn draw_gallery_page(g: &mut Game) {
    // ---------- 滚动状态机（人物页同一单口；图鉴自有 max/step）----------
    // v1.16.1：tap 捕获——已识真图卡立绘点按进鉴赏覆层（draw_zoom_overlay）
    // v1.16.10：滚动钳按 EVENTS.len() 现算（遭遇手札追加在列表尾，扩池自动跟随；
    // 硬编码 max 会随扩池漂移——4208/3708 旧账教训）
    // v1.16.10 复审修：snap 按滚动区域传——292 卡栅格吸附与手札 52px 行距失配，
    // 手札区停手/惯性衰减后会被 lerp 拉回 292 栅格把行卡切在视口缘。分节行进入
    // 视口（scroll > GAL_EV_SEC_OFF − 视口高）前再让出一个栅格步关吸附：既保
    // 手札区自由停留，也防「贴线静止被吸过界」的越界前跳（卡栅格区行为不变）。
    let scroll_max = GAL_EV_OFF + EVENTS.len() as f32 * GAL_EV_STEP + 40.0 - (GAL_VIEW_Y1 - GAL_VIEW_Y0);
    let snap = g.npc_scroll
        <= GAL_EV_SEC_OFF - (GAL_VIEW_Y1 - GAL_VIEW_Y0) - GAL_STEP;
    let tap = scroll_list_tick(g, scroll_max, GAL_STEP, snap);
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

    // ---------- v1.16.10 遭遇手札分节行（40px，NPC 区后）----------
    let esec_y = GAL_VIEW_Y0 + GAL_EV_SEC_OFF - scroll;
    if esec_y + 40.0 > GAL_VIEW_Y0 && esec_y < GAL_VIEW_Y1 && npc_row_vis(esec_y + 14.0) {
        txt_c(
            "—— 遭遇手札 ——",
            640.0,
            esec_y + 12.0, 14,
            Color::from_rgba(162, 152, 184, 220), // 同「—— 现 实 ——」表头语言
        );
    }

    // ---------- v1.16.10 遭遇手札事件行（整卡剔除：行 46/步 52 → 视口内峰值 ≤11 行；
    // 纯文本无立绘无懒解码，format! 仅计数 ≥10 的长尾行 ≤ 可见行数）----------
    for i in 0..EVENTS.len() {
        let y = GAL_VIEW_Y0 + GAL_EV_OFF + i as f32 * GAL_EV_STEP - scroll;
        if y >= GAL_VIEW_Y1 || y + GAL_EV_H <= GAL_VIEW_Y0 {
            continue;
        }
        draw_event_note_row(g, i, y);
    }

    // ---------- 遮幅（口径同人物页：全不透明面板色，盖住越缘内容）----------
    let mask = Color::from_rgba(22, 16, 38, 255);
    draw_rectangle(150.0, 104.0, 980.0, 12.0, mask);
    draw_rectangle(150.0, 616.0, 980.0, 38.0, mask);

    // ---------- 页脚（y630 固定于视口外，照人物页 411-426；纯推导计数零存档键）----------
    let known = g.meta.comp_bonds.iter().filter(|&&b| b > 0).count();
    let met = g.meta.npc_fav.iter().filter(|&&f| f >= 20).count();
    let seen_n = g.meta.ev_cnt.iter().filter(|&&c| c > 0).count();
    txt(
        &format!(
            "同页人 已识 {}/10 · 交游 {}/4 · 手札收录 {}/{} —— 同页之缘起于模拟，跨世持久。",
            known,
            met,
            seen_n,
            EVENTS.len()
        ),
        170.0,
        630.0, 14,
        C_DIM,
    );
}

// v1.16.10 遭遇手札事件行（纯只读无 &mut 交互）：已遇=标题+简介+徽标+遭遇计数
// （徽标 event_badge 单口纯推导；标题/简一静态表，未遇整行 ？？？ 不泄题）。
// 行卡描边照同页人卡语言：稀有紫 / 普通（普通稀有判定与 build_event card.rare 同源
// ——EVENTS 权重 w∈1..=RARE_W）/ 未遇灰。
fn draw_event_note_row(g: &Game, i: usize, y: f32) {
    let x = 160.0;
    let cnt = g.meta.ev_cnt[i]; // i 循环界内（0..EVENTS.len()），恒界内
    let seen = cnt > 0;
    let w = EVENTS[i].1;
    let rare = w > 0 && w <= crate::data::RARE_W;
    let (bg, border) = if seen {
        (
            Color::from_rgba(40, 32, 58, 200),
            if rare { C_PURPLE } else { C_GOLD_D },
        )
    } else {
        (
            Color::from_rgba(26, 22, 42, 190),
            Color::from_rgba(96, 82, 124, 255), // 灰边（面板禁用描边同色）
        )
    };
    npc_card_chrome(x, y, GAL_EV_H, bg, border);
    if npc_row_vis(y + 6.0) {
        if seen {
            let name = EVENT_NAMES
                .get(i)
                .copied()
                .unwrap_or("？？？"); // 防御：表行错位落 ？？？ 绝不越界
            txt(name, x + 16.0, y + 6.0, 14, if rare { C_PURPLE } else { C_TEXT });
        } else {
            txt("？？？", x + 16.0, y + 6.0, 14, C_DIM);
        }
    }
    if npc_row_vis(y + 24.0) {
        if seen {
            // 简一 display-only（EVENT_BRIEF 静态表；未遇不显示不泄题）
            let brief = EVENT_BRIEF.get(i).copied().unwrap_or("");
            if !brief.is_empty() {
                txt(brief, x + 16.0, y + 24.0, 14, C_DIM);
            }
        }
    }
    // 徽标（已遇才显——未遇不泄分类）与计数（行右）
    if seen {
        let (bt, bc) = event_badge(i);
        if npc_row_vis(y + 6.0) {
            txt(bt, 900.0, y + 6.0, 14, bc);
        }
        let cs = ev_cnt_str(cnt);
        if npc_row_vis(y + 6.0) {
            if cs.is_empty() {
                // ≥10 长尾：format! 兜底（≤ 可见行数，峰值 11）
                txt_r(&format!("×{}", cnt), 1084.0, y + 6.0, 14, C_GOLD);
            } else {
                txt_r(cs, 1084.0, y + 6.0, 14, C_GOLD);
            }
        }
    } else if npc_row_vis(y + 6.0) {
        txt("未遇", 900.0, y + 6.0, 14, C_DIM);
    }
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
