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
use crate::draw::{bar, panel_card};
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

// v1.16.13 复审⑦ tap 纵向夹取单口（手札行/同页人卡/NPC 卡三处共用，纯函数供单测）：
// 列表循环剔除只剔整行/整卡出界，部分可见行经上下遮幅（309-310）仍可点——命中
// 条件必须再夹进视口带 [GAL_VIEW_Y0, GAL_VIEW_Y1]，防点遮幅带开覆层/鉴赏
fn tap_y_in_view(my: f32) -> bool {
    my >= GAL_VIEW_Y0 && my <= GAL_VIEW_Y1
}

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
    // v1.16.11 low 修复：页脚「手札收录 n/N」的收/录二字此前不在任何预热串
    //（COMPANION_NAMES/NPC_NAMES/NPC_TIER_NAMES 皆无）——新档首开图鉴帧冷栅格化
    "手札收录",
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
    // v1.16.16 结局回响徽标（165..=167 event_badge 单口新增词）
    "回响",
    "×1",
    "×2",
    "×3",
    "×4",
    "×5",
    "×6",
    "×7",
    "×8",
    "×9",
    // v1.16.11 手札回味覆层静态 chrome（全部 @14，prewarm_event_book 一次灌齐；
    // 覆层正文/标题字形走构建帧预热，见 draw_event_recall_overlay 注释）
    "手札回味",
    "本世未存全文，仅录简介",
    "点按任意处返回",
];

/// 遭遇手札首开帧预热（exec_panel_act Tab(4)/指引 Gallery 跳转同口调用）：静态串
/// 全量 + 已遇事件的标题/简介（未遇不灌——全池灌齐有图集余量风险，且未遇
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
        // v1.16.16 章节徽标扩 168/169（书外之章联动）；结局回响 3 条另立「回响」徽标
        // v1.16.17 章节徽标扩 190/191（一章雨夜执印/四章文狱讨牒联动）
        // v1.16.19 章节徽标补漏 + 扩新：196/197（二章/五章联动，此前漏登记落
        // 「遭遇」徽标的 display 漂移，本批一并修）/217（七章邻人扛劫）
        // v1.16.20 章节徽标扩 230（八章落款撞名，bit7 联动缺角补齐）
        74 | 75 | 168 | 169 | 190 | 191 | 196 | 197 | 217 | 230 => return ("章节", C_GOLD),
        165 | 166 | 167 => return ("回响", C_PURPLE),
        // v1.16.16 老年层扩 170/171（炉暖旧器/灯影授业，门控见 sim_event_weights）
        // v1.16.17 老年层扩 189（旧履还坊）
        // v1.16.19 老年徽标补漏 + 扩新：198/199（窗前晒卷/旧邻问名，此前漏登记）
        // 与 204（同席说岁，此前落「遭遇」徽标）一并修；215/216 本批新条。
        // v1.16.20 老年徽标扩 227（半檐躲雨）。
        // v1.16.21 老年徽标扩 239（共守岁灯，老年×有伴双门控第 3 条——不入
        // COMPANION 白名单故老年臂先命中出「老年」徽标）。
        // 212 相扶看雪同为老年×有伴双门控，但在 COMPANION_EVENTS 白名单内，
        // 上方 COMPANION 臂先命中恒出「伴随」徽标——不入本臂（死模式勿挂）
        88 | 89 | 170 | 171 | 189 | 198 | 199 | 204 | 215 | 216 | 227 | 239 => {
            return ("老年", C_GOLD_D)
        }
        110 | 111 => return ("宿敌", C_RED),
        // v1.16.11 链起扩 118/121（链9 循册寻仇/链10 旧页回声首段）
        // v1.16.13 链起扩 137/140（链11 劫痕渡客/链12 书肆换届首段）
        // v1.16.14 链起扩 151（链13 死当归主首段）
        // v1.16.16 链起扩 175（链14 修伞记首段）
        // v1.16.19 链起扩 205（链15 观星问岁首段）
        // v1.16.20 链起扩 219（链16 剑穗之约首段）
        // v1.16.21 链起扩 231（链17 星槎再访首段——mr5 零链补位）
        44 | 47 | 50 | 76 | 79 | 85 | 112 | 115 | 118 | 121 | 137 | 140 | 151 | 175 | 205
        | 219 | 231 => {
            return ("链起", C_CYAN)
        }
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
            && tap_y_in_view(my) // v1.16.13 复审⑦：遮幅带内点按不开鉴赏（纵夹）
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
            && tap_y_in_view(my) // v1.16.13 复审⑦：遮幅带内点按不开鉴赏（纵夹）
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
    // v1.16.11 手札回味：已遇行点按（位移<8px 门槛由 scroll_list_tick 的 tap 语义
    // 保证）开启只读详情覆层——开启帧灌该行动态字形（事件帧预热先例），覆层本体
    // draw_event_recall_overlay 由 draw_reality 早退链独占渲染
    for i in 0..EVENTS.len() {
        let y = GAL_VIEW_Y0 + GAL_EV_OFF + i as f32 * GAL_EV_STEP - scroll;
        if y >= GAL_VIEW_Y1 || y + GAL_EV_H <= GAL_VIEW_Y0 {
            continue;
        }
        draw_event_note_row(g, i, y);
        if tap
            && g.meta.ev_cnt[i] > 0
            && mx >= 160.0
            && mx <= 1120.0
            && my >= y
            && my <= y + GAL_EV_H
            && tap_y_in_view(my) // v1.16.13 复审⑦：部分可见行遮幅带内点按不开回味覆层（纵夹）
        {
            g.ev_recall = Some(i);
            prewarm_recall_overlay(g, i);
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

// v1.16.11 手札回味覆层开启帧一次性灌字形（照 prewarm_zoom_text 先例）：静态 chrome
// （手札回味/降级标签/点按任意处返回）已并入 EV_NOTE_PREWARM 由 prewarm_event_book
// 在 Tab(4) 开页帧灌齐；此处只补该行动态内容——memo 命中路径的标题/正文已在卡构建
// 帧灌过（build_event 尾 prewarm_text title@24/text@16，同串同字号），只补降级路径
// （memo None）的名/简，一两行字形事件帧开销可忽略
fn prewarm_recall_overlay(g: &Game, i: usize) {
    if !g.ev_memo.get(i).map_or(false, |m| m.is_some()) {
        if let Some(n) = EVENT_NAMES.get(i) {
            prewarm_text(n, &[24, 14]);
        }
        if let Some(b) = EVENT_BRIEF.get(i) {
            prewarm_text(b, &[14]);
        }
    }
}

// v1.16.14 回味覆层静态 chrome 单源（全 @14）：覆层头标/降级标签/返回提示三串，
// 绘制位（draw_event_recall_overlay 三处按下标取串）与两条开启口的预热同源——
// Tab(4) 手札页口并入 EV_NOTE_PREWARM（对账测试钉死不漂移），终局页本稿遭遇口
// 由 prewarm_run_recall 循环本表自灌（该口不经 prewarm_event_book）
const RECALL_CHROME: [&str; 3] = ["手札回味", "本世未存全文，仅录简介", "点按任意处返回"];

// v1.16.11 手札回味覆层：图鉴已遇行点按开启的只读详情（纯展示不结算）。模态独占
// 输入照 zoom_art 早退先例（draw_reality 早退链独占渲染，本函数由其调用）；版式
// 轻量复刻 draw_modal_card 普通卡（panel_card 底板/标题 24 金居中/装饰线/链段青
// 稀有紫描边/正文 16 wrap 520），无选项无暂避钮。任意点按关闭；Esc/C 走 main.rs
// 独立清态口。
// 数据源 ev_memo[id]（本会话最近一次卡面正文构建帧捕获）：None 降级显 EVENT_BRIEF
// 一句（旧会话/旧档遭遇本会话未重现的行，不虚构全文）；标题恒取 EVENT_NAMES[id]
// 静态表（event_names_match_card_titles 钉死与卡面标题双源一致）。
// 性能：稳态零堆分配（全 &'static str，计数 ≥10 长尾走 format! 与手札行同款豁免）；
// 字形命中构建帧/开页帧/开启帧三层预热，覆层期零图集重传。
pub fn draw_event_recall_overlay(g: &mut Game) {
    let Some(id) = g.ev_recall else { return };
    // min/clamp 防御（panic=abort 纪律）：id 越界绝不索引
    let id = id.min(EVENTS.len().saturating_sub(1));
    let memo = g.ev_memo.get(id).and_then(|m| m.as_deref());
    let cnt = g.meta.ev_cnt.get(id).copied().unwrap_or(0);
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::from_rgba(8, 5, 16, 244));
    panel_card(340.0, 100.0, 600.0, 500.0);
    // 卡描边照 draw_modal_card 语言（2.5 同位覆盖）：链段青/稀有紫/普通卡不叠边
    let w = EVENTS[id].1;
    let rare = w > 0 && w <= crate::data::RARE_W;
    let (badge_t, badge_c) = event_badge(id);
    let is_chain = badge_t == "链起" || badge_t == "链段";
    let edge = if is_chain {
        Some(C_CYAN)
    } else if rare {
        Some(C_PURPLE)
    } else {
        None
    };
    if let Some(ec) = edge {
        draw_rectangle_lines(340.0, 100.0, 600.0, 500.0, 2.5, ec);
    }
    txt(RECALL_CHROME[0], 380.0, 112.0, 14, C_DIM);
    let name = EVENT_NAMES.get(id).copied().unwrap_or("？？？"); // 防御同手札行口径
    txt_c(name, V_W * 0.5, 128.0, 24, C_GOLD);
    draw_rectangle(
        380.0,
        164.0,
        520.0,
        1.5,
        Color::from_rgba(96, 76, 130, 160), // 照 draw_modal_card 标题装饰线同色
    );
    // 徽标/计数行：分类徽标居左（event_badge 单口纯推导），遭遇计数居右
    //（×1..=9 静态查表，≥10 format! 兜底——覆层期单行豁免；v1.16.13 复审⑥：
    // 前缀「遭遇」只随数值大小出现属口径漂移，与手札行统一为纯「×N」右对齐。
    // cnt==0 运行时不可达——两口开启侧均有保证：手札行 ev_cnt[i]>0 门控+
    // tap_y_in_view 纵夹；本稿遭遇行 id 出自 run_events 已遇集，cnt>0 恒成立）
    txt(badge_t, 380.0, 174.0, 14, badge_c);
    let cs = ev_cnt_str(cnt);
    if cs.is_empty() {
        txt_r(&format!("×{}", cnt), 900.0, 174.0, 14, C_GOLD);
    } else {
        txt_r(cs, 900.0, 174.0, 14, C_GOLD);
    }
    // 正文：memo 命中显全文 @16（区 196..560 容 13 行，全部事件卡面 ≤5 行预算
    // 皆裕）；降级显简介 @14（静态表，不虚构全文）
    match memo {
        Some(t) => {
            wrap_draw(t, 380.0, 196.0, 16, 520.0, 28.0, C_TEXT);
        }
        None => {
            txt(RECALL_CHROME[1], 380.0, 196.0, 14, C_DIM);
            if let Some(b) = EVENT_BRIEF.get(id) {
                if !b.is_empty() {
                    wrap_draw(b, 380.0, 224.0, 14, 520.0, 22.0, C_TEXT);
                }
            }
        }
    }
    txt_c(RECALL_CHROME[2], V_W * 0.5, 566.0, 14, C_DIM);
    if is_mouse_button_pressed(MouseButton::Left) {
        g.ev_recall = None;
        g.play(crate::sounds::Which::Page);
    }
}

// v1.16.13 终局页『本稿遭遇』开启帧一次性灌字形（照 prewarm_recall_overlay 先例）：
// chrome + 徽标串（取值域固定小集）+ 本稿各遇行名（EVENT_NAMES 静态表查行）
pub fn prewarm_run_recall(g: &Game) {
    prewarm_text("本 稿 遭 遇", &[24]);
    // v1.16.13 复审修：头部计数行静态部分「本稿共录/则」此前不在任何预热串
    //（本/稿@14 由空态串覆盖，共/录/则全库独此一行）——动态数字为 ASCII 整档
    // 保留，中文字形在此灌齐
    prewarm_text("本稿共录 则 · 点行回味全文", &[14]);
    prewarm_text("点行外任意处返回", &[14]);
    prewarm_text("本稿风平浪静，未遇事件", &[14]);
    // v1.16.14 复审修：本稿遭遇口点行开启的回味覆层不经 prewarm_event_book（那是
    // Tab(4) 手札页口）——chrome 三串在此自灌，否则未开过手札页的会话（开 sim→
    // 终局→本稿剧情→点行）首开覆层三串冷栅格化
    for s in RECALL_CHROME {
        prewarm_text(s, &[14]);
    }
    for id in &g.run_events {
        let (bt, _) = event_badge(*id);
        prewarm_text(bt, &[14]);
        if let Some(nm) = EVENT_NAMES.get(*id) {
            prewarm_text(nm, &[14]);
        }
    }
}

// v1.16.14 本稿遭遇列表内容带/行带常量：自局部提升为模块级，供抬起命中单口
// run_recall_row_hit 与其单测引用（数值=版式原注 190..556/行带 26，不挪不改）
const RUN_RECALL_Y0: f32 = 190.0;
const RUN_RECALL_Y1: f32 = 556.0;
const RUN_RECALL_ROW_H: f32 = 26.0;

// v1.16.14 复审修：抬起命中单口（纯函数供单测，照 tap_y_in_view 先例）——按下
// 锚点已限内容带（draw_run_recall_list 按下分支），但 <6px 位移可把抬起滑出带外：
// 上/下缘落遮幅带（167..190 / 556..596 只遮绘制不遮命中）时行号反推指向被盖住的
// 不可见行，抬起须再夹内容带（手札页 tap_y_in_view 同款纵夹）
fn run_recall_row_hit(mx: f32, my: f32, idx: usize, n: usize) -> bool {
    mx >= 380.0 && mx <= 900.0 && my >= RUN_RECALL_Y0 && my <= RUN_RECALL_Y1 && idx < n
}

// v1.16.13 终局页『本稿遭遇』列表覆层：Scene::Inherit 页「本稿剧情」行点开的只读
// 列表（draw_inherit 早退独占渲染，本函数由其调用）。数据源 g.run_events（本世遭遇
// 去重有序表，note_event 单口 push、launch_sim 清零——切勿用 ev_cnt 反推本世列表，
// 跨世累计无边界）。点行开 draw_event_recall_overlay 回味全文（ev_recall 复用，
// Esc 先关回味再关列表、main.rs 分域）；点行外任意处返回（终局页不离场——继承
// 选项/合页钮原位保留）。
// 版式照 draw_event_recall_overlay（panel_card 340,100,600,500/标题 24 金居中/装饰
// 线）；滚动自持字段 run_recall_scroll/drag（滚轮 ±48px/格照 scroll_list_tick 同参 +
// 触拖 1:1，无惯性无吸附——覆层列表短程滚动不需要状态机全套），与图鉴页 npc_scroll
// 互不相涉。性能：行渲染全静态查表零分配（EVENT_NAMES/event_badge &'static str），
// 唯头部计数一行 format!（模态静态页豁免，同 draw_inherit 本页既有行口径）。
pub fn draw_run_recall_list(g: &mut Game) {
    let n = g.run_events.len();
    // 内容视口 190..556（面板 340,100,600,500 内，标题/装饰线之下），行带 26
    let (content_y0, content_y1, row_h) = (RUN_RECALL_Y0, RUN_RECALL_Y1, RUN_RECALL_ROW_H);
    let max = ((n as f32) * row_h - (content_y1 - content_y0)).max(0.0);
    let (mx, my) = crate::cam::mouse();
    let pressed = is_mouse_button_pressed(MouseButton::Left);
    let released = is_mouse_button_released(MouseButton::Left);
    // 滚轮（桌面）
    let (_, wy) = mouse_wheel();
    if wy != 0.0 {
        g.run_recall_scroll = (g.run_recall_scroll - wy * 48.0).clamp(0.0, max);
    }
    // 输入分域：内容带内按下记锚点（拖动/点行）；带外（面板 chrome/页眉页脚/面板外）
    // 点按即返回——照 recall 覆层「任意点按关闭」先例。关闭不提前 return（提前退会
    // 跳过本帧全部绘制产生一帧空屏）：不置锚，chrome/行照常绘完，下帧函数头早退
    // 自然落到终局页
    if pressed {
        if mx >= 340.0 && mx <= 940.0 && my >= content_y0 && my <= content_y1 {
            g.run_recall_drag = Some((mx, my, g.run_recall_scroll));
        } else {
            g.run_recall_open = false;
            g.play(crate::sounds::Which::Page);
        }
    }
    if let Some((px, py, s0)) = g.run_recall_drag {
        let dist = ((mx - px) * (mx - px) + (my - py) * (my - py)).sqrt();
        if !released {
            if dist >= 6.0 {
                // 1:1 跟手（自锚点起算），无惯性（短列表自由停留即停）
                g.run_recall_scroll = (s0 - (my - py)).clamp(0.0, max);
            }
        } else {
            g.run_recall_drag = None;
            if dist < 6.0 {
                // 点行：开回味覆层（行号由抬起点反推，min 防御绝不高估）；命中
                // 单口内含 my 纵夹——带外抬起（遮幅带）不开不可见行
                let idx = ((my - content_y0 + g.run_recall_scroll) / row_h).max(0.0) as usize;
                if run_recall_row_hit(mx, my, idx, n) {
                    let id = g.run_events[idx];
                    g.ev_recall = Some(id);
                    prewarm_recall_overlay(g, id);
                    g.play(crate::sounds::Which::Click);
                }
            }
        }
    }
    // chrome（照 draw_event_recall_overlay 版式）
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::from_rgba(8, 5, 16, 244));
    panel_card(340.0, 100.0, 600.0, 500.0);
    txt_c("本 稿 遭 遇", V_W * 0.5, 128.0, 24, C_GOLD);
    draw_rectangle(
        380.0,
        164.0,
        520.0,
        1.5,
        Color::from_rgba(96, 76, 130, 160), // 照装饰线同色
    );
    if n == 0 {
        txt("本稿风平浪静，未遇事件", 380.0, 200.0, 14, C_DIM);
        return;
    }
    // 行渲染：滚动画布 + 上下遮幅（照手札页遮幅口径——全不透明面板色盖越缘行，
    // 命中侧已按内容带分域故遮幅只遮绘制），行带 26 内徽标居左+名居左
    let scroll = g.run_recall_scroll;
    let first = (scroll / row_h).floor() as usize;
    let last = ((scroll + (content_y1 - content_y0)) / row_h).ceil() as usize;
    for k in first..last.min(n) {
        let id = g.run_events[k];
        let y = content_y0 + k as f32 * row_h - scroll;
        let (bt, bc) = event_badge(id);
        txt(bt, 380.0, y, 14, bc);
        if let Some(nm) = EVENT_NAMES.get(id) {
            txt(nm, 452.0, y, 14, C_TEXT);
        }
    }
    // 遮幅（口径同手札页 309-310：全不透明面板色，盖住越缘内容）——上幅让开
    // 装饰线（164..165.5），下幅让开卡底缘（596..600 为 panel_card 边线）
    let mask = Color::from_rgba(22, 16, 38, 255);
    draw_rectangle(340.0, 167.0, 600.0, content_y0 - 167.0, mask);
    draw_rectangle(340.0, content_y1, 600.0, 596.0 - content_y1, mask);
    // v1.16.13 复审修：头部计数行（174..188 落上遮幅 167..190 带内）须在遮幅之后
    // 绘制——原版式照抄 draw_event_recall_overlay y174 徽标行，但回味覆层无滚动
    // 遮幅、本列表新加了遮幅，先绘即被 alpha 255 整行盖住（n>0 恒不可见）；照
    // footer 同法后置，行越缘像素先被遮幅吃掉再叠 chrome，无透字
    txt(
        &format!("本稿共录 {} 则 · 点行回味全文", n),
        380.0,
        174.0,
        14,
        C_DIM,
    );
    txt_c("点行外任意处返回", V_W * 0.5, 566.0, 14, C_DIM);
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

// ---------------- v1.16.13 复审⑥⑦：计数口径与 tap 纵夹（纯函数单测） ----------------
#[cfg(test)]
mod gallery_v11613_tests {
    use super::*;

    // ev_cnt_str 口径：×1..=9 静态查表；0 与 ≥10 返回空串走调用方 format! 兜底。
    // 0 运行时不可达（两口开启侧均有保证——手札行 ev_cnt[i]>0 门控+tap_y_in_view
    // 纵夹；本稿遭遇行 id 出自 run_events 已遇集），仅 ≥10 落 format!（覆层与
    // 手札行统一为纯「×N」右对齐）。
    #[test]
    fn ev_cnt_str_table_and_fallback() {
        for c in 1u8..=9 {
            assert_eq!(ev_cnt_str(c), format!("×{}", c), "×{} 须静态查表命中", c);
        }
        assert_eq!(ev_cnt_str(0), "", "0 不可达口径：空串走兜底");
        assert_eq!(ev_cnt_str(10), "", "≥10 长尾走 format! 兜底");
        assert_eq!(ev_cnt_str(255), "", "封顶 255 同走兜底");
    }

    // tap 纵夹边界：视口带 [116, 616] 闭区间内可点，上下缘外一 px 即不可点
    #[test]
    fn tap_y_clamped_to_viewport_band() {
        assert!(!tap_y_in_view(GAL_VIEW_Y0 - 1.0), "上缘外 1px 不可点");
        assert!(tap_y_in_view(GAL_VIEW_Y0), "上缘可点（闭区间）");
        assert!(tap_y_in_view((GAL_VIEW_Y0 + GAL_VIEW_Y1) * 0.5));
        assert!(tap_y_in_view(GAL_VIEW_Y1), "下缘可点（闭区间）");
        assert!(!tap_y_in_view(GAL_VIEW_Y1 + 1.0), "下缘外 1px 不可点");
    }

    // 回归场景：部分可见行（整行出界剔除剔除不到）遮幅带内点按——行内命中式
    // （my∈[y, y+GAL_EV_H]）为真但纵夹为假，覆层不得开启（遮幅 309-310 只遮绘制
    // 不遮命中，纵夹是命中侧的补口）
    #[test]
    fn partially_visible_row_mask_band_not_tappable() {
        // 首行滚出视口上半：行顶 y = VIEW_Y0 − 30，遮幅带 my=VIEW_Y0−5 落行内
        let y = GAL_VIEW_Y0 - 30.0;
        let my = GAL_VIEW_Y0 - 5.0;
        assert!(my >= y && my <= y + GAL_EV_H, "行内命中式为真（未夹前可误触）");
        assert!(!tap_y_in_view(my), "纵夹后遮幅带点按不开覆层");
        // 行内可见区仍可点：行顶滚入后 my 在视口带内
        let my_in = GAL_VIEW_Y0 + 10.0;
        assert!(my_in >= y && my_in <= y + GAL_EV_H && tap_y_in_view(my_in));
    }

    // 末行滚出视口下半的对称场景（尾行 y+GAL_EV_H 越下缘）
    #[test]
    fn partially_visible_row_below_band_not_tappable() {
        let y = GAL_VIEW_Y1 - GAL_EV_H + 30.0; // 行底 = VIEW_Y1 + 30
        let my = GAL_VIEW_Y1 + 5.0;
        assert!(my >= y && my <= y + GAL_EV_H, "行内命中式为真");
        assert!(!tap_y_in_view(my), "下遮幅带点按不开覆层");
    }
}

// ---------------- v1.16.14 复审：回味 chrome 预热对账 + 本稿遭遇抬起纵夹（纯函数单测） ----------------
#[cfg(test)]
mod gallery_v11614_tests {
    use super::*;

    // chrome 单源形状钉死：draw_event_recall_overlay 三绘制位按下标取串
    //（0=头标/1=降级标签/2=返回提示），数组被重排/增删会静默错位——逐位钉值
    #[test]
    fn recall_chrome_shape_pinned() {
        assert_eq!(RECALL_CHROME.len(), 3);
        assert_eq!(RECALL_CHROME[0], "手札回味");
        assert_eq!(RECALL_CHROME[1], "本世未存全文，仅录简介");
        assert_eq!(RECALL_CHROME[2], "点按任意处返回");
    }

    // Tab(4) 手札页开启口对账：EV_NOTE_PREWARM 必须覆盖 chrome 全集——预热清单
    // 与 chrome 是两份字面量，任一侧改串/删串即红（防该口首开覆层冷栅格化）。
    // 终局页本稿遭遇口由 prewarm_run_recall 循环 RECALL_CHROME 自灌（与绘制位
    // 同一单源，覆盖属结构保证，不存在可对账的旁路清单）
    #[test]
    fn recall_chrome_covered_by_event_book_prewarm() {
        for s in RECALL_CHROME {
            assert!(
                EV_NOTE_PREWARM.contains(&s),
                "chrome「{}」不在 EV_NOTE_PREWARM——Tab(4) 口首开覆层冷栅格化",
                s
            );
        }
    }

    // 抬起命中带边界：内容带 [RUN_RECALL_Y0, RUN_RECALL_Y1] 闭区间内可点，缘外
    // 一概不开（v1.16.14 复审修——<6px 位移可把抬起滑出按下锚点的带判定）；mx
    // 带与 idx<n 两既有维度一并钉住不回归
    #[test]
    fn run_recall_row_hit_band_boundaries() {
        let (mx, n) = (500.0, 20usize);
        assert!(!run_recall_row_hit(mx, RUN_RECALL_Y0 - 1.0, 0, n), "上缘外 1px 不可点");
        assert!(run_recall_row_hit(mx, RUN_RECALL_Y0, 0, n), "上缘可点（闭区间）");
        assert!(run_recall_row_hit(mx, (RUN_RECALL_Y0 + RUN_RECALL_Y1) * 0.5, 5, n));
        assert!(run_recall_row_hit(mx, RUN_RECALL_Y1, 5, n), "下缘可点（闭区间）");
        assert!(!run_recall_row_hit(mx, RUN_RECALL_Y1 + 1.0, 5, n), "下缘外 1px 不可点");
        assert!(!run_recall_row_hit(379.9, 300.0, 0, n), "mx 左缘外不可点");
        assert!(!run_recall_row_hit(900.1, 300.0, 0, n), "mx 右缘外不可点");
        assert!(!run_recall_row_hit(mx, 300.0, n, n), "idx 越表不可点");
    }

    // 上遮幅带误开回归（修前实况）：半行滚动 scroll=13，行0 起绘 y=Y0−13=177——
    // 仍在绘制循环内（first=0，非整行出界剔除）但整行文字落上遮幅 167..Y0 带内
    // 不可见；按下锚 my=Y0（带内合法）抬起 <6px 滑到 184.5，行号反推恰指行0。
    // 修前命中式可开不可见行，修后缘外不开
    #[test]
    fn run_recall_top_mask_band_not_tappable() {
        let scroll = 13.0f32;
        let release_y = RUN_RECALL_Y0 - 5.5; // 按下锚 Y0、抬起位移 5.5<6（tap 语义）
        let idx = ((release_y - RUN_RECALL_Y0 + scroll) / RUN_RECALL_ROW_H).max(0.0) as usize;
        assert_eq!(idx, 0, "行号反推指向上遮幅带内的行0");
        let first = (scroll / RUN_RECALL_ROW_H).floor() as usize;
        let last = ((scroll + (RUN_RECALL_Y1 - RUN_RECALL_Y0)) / RUN_RECALL_ROW_H).ceil() as usize;
        assert!(first <= idx && idx < last.min(20), "行0 在绘制循环内（非未绘不可见）");
        let draw_y = RUN_RECALL_Y0 + idx as f32 * RUN_RECALL_ROW_H - scroll;
        assert!(
            draw_y >= 167.0 && draw_y + 14.0 * 0.82 <= RUN_RECALL_Y0,
            "行0 文字整体被上遮幅 167..Y0 盖住（不可见）"
        );
        assert!(!run_recall_row_hit(500.0, release_y, idx, 20), "上遮幅带抬起不开不可见行");
    }

    // 下遮幅带对称场景：scroll=0，行14 起绘 y=Y0+14×26=554，下遮幅 556..596 盖掉
    // 行主体（可见仅 554..556 共 2px 残条）；抬起滑出下缘到 561.5，行号反推恰指
    // 行14。修前可开近整行被盖的行，修后缘外不开（可见残条 554..556 仍可点，与
    // 手札页 tap_y_in_view 闭区间口径一致）
    #[test]
    fn run_recall_bottom_mask_band_not_tappable() {
        let scroll = 0.0f32;
        let release_y = RUN_RECALL_Y1 + 5.5;
        let idx = ((release_y - RUN_RECALL_Y0 + scroll) / RUN_RECALL_ROW_H).max(0.0) as usize;
        assert_eq!(idx, 14, "行号反推指向下遮幅带内的行14");
        let draw_y = RUN_RECALL_Y0 + idx as f32 * RUN_RECALL_ROW_H - scroll;
        let visible_h = (RUN_RECALL_Y1 - draw_y).max(0.0);
        assert!(visible_h < RUN_RECALL_ROW_H, "行14 部分被下遮幅盖住（可见 {}px）", visible_h);
        assert!(!run_recall_row_hit(500.0, release_y, idx, 20), "下遮幅带抬起不开被盖住的行");
    }

    // 带内合法点行不回归：上/下缘内 2px 的抬起伏动照常命中（收窄只切带外，不切
    // 任何带内合法路径；拖拽 dist>=6 与滚轮路径不经此单口，不受影响）
    #[test]
    fn run_recall_in_band_tap_still_hits() {
        assert!(run_recall_row_hit(500.0, RUN_RECALL_Y0 + 2.0, 3, 20));
        assert!(run_recall_row_hit(500.0, RUN_RECALL_Y1 - 2.0, 3, 20));
    }
}
