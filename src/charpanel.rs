// v1.5.0 角色面板 v2（底栏「角色」/ C 键，panel=5）：三页签
// 0 总览：左栏立绘与栖身之地，右区属性摘要（法宝只留计数）
// 1 法宝：左 3 装备槽（点击卸下）/ 右包内法宝（点击装备）
// 2 成就·称号：左成就 2 列带进度 / 右称号 19 格（点击佩戴，格 0=自动跟随、
//   格 16=弑神者、格 17=雨夜执印者——v1.16.10 章节称号，首章达成即授；
//   格 18=执笔人——v1.16.15 全卷称号，四结局全览即授）
// 6 道侣（v1.17.0）：道侣专页——主卡（立绘/名号/情谊条/双修加成/缝页缘/双修共进钮）
//   + 缘谱十行（同页人历世账，纯文本零立绘）；展示数据全在既有持久化字段，零新键
use crate::art::draw_cultivator;
use crate::data::*;
use crate::draw::{bar, bar_glow, button, panel_card};
use crate::game::*;
use crate::ui::*;
use macroquad::prelude::*;

pub fn draw_char_panel(g: &mut Game) {
    // v1.16.11 手札回味覆层：模态独占输入（图鉴页已遇行点按开启）。正常路径已被
    // draw_reality 早退链独占（不至本函数），此防御口照 zoom_art 的 draw_sim 早退
    // 先例兜未来调用路径——覆层期 chrome/页签一概不绘不响应，点按任意处返回
    if g.ev_recall.is_some() {
        crate::gallery::draw_event_recall_overlay(g);
        return;
    }
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, 0.6));
    panel_card(140.0, 60.0, 1000.0, 600.0);
    // v1.10.2：人物页（页签 3）改为「先内容 → 遮幅 → 后 chrome」——人物页滚动列表的
    // 上下遮幅带会盖住页签行（74..108），chrome 须最后重铺才不被遮；其余页签无溢出，
    // 维持先 chrome 后内容的旧序（立即模式命中不依赖绘制顺序，行为不变）。
    // v1.10.2 卡顿修复（次要项③）：去每帧整 Life 深克隆（含 relics/talents/equipped
    // 三个 Vec 堆分配）——各页改 &Game 只读绘制 + 收集 PanelAct，借用归还后由
    // exec_panel_act 统一执行 &mut 交互（先例 v1.10.0 draw_sim / 本版 draw_reality）。
    // ×/页签点击帧的金条高亮切换晚一帧生效（16ms，不可见），其余行为逐项等价。
    // v1.16.0：图鉴页（页签 4）同为单列滚动列表——上下遮幅带同样盖住页签行，
    // chrome 须最后重铺；与人物页共用本双绘条件。
    // v1.17.0：道侣专页（页签 6）同为滚动列表（主卡+缘谱行），并入本双绘条件
    let npc_tab = g.char_tab == 3 || g.char_tab == 4 || g.char_tab == 6;
    let mut act = draw_panel_chrome_act(g);
    if act.is_none() {
        match g.char_tab {
            1 => act = draw_relic_page(g),
            2 => act = draw_ach_title_page(g),
            3 => act = draw_npc_page(g), // v1.6.0 D：好感度页（v1.10.2 单列滚动列表）
            // v1.16.0 交游图鉴：同页人 10 卡 + 文狱四 NPC 卡（纯只读滚动，无 &mut 交互）
            4 => crate::gallery::draw_gallery_page(g),
            // v1.16.5 旧稿档案：近 5 稿结算快照（纯只读固定列表，无滚动无 &mut 交互）
            5 => draw_hist_page(g),
            // v1.17.0 道侣专页：主卡（立绘/名号/情谊/缝页/双修）+ 缘谱十行（滚动，
            // 双修钮/立绘鉴赏以 PanelAct 传出）
            6 => act = draw_comp_page(g),
            // v1.10.2：总览页纯只读，整页持 life 借用（防御 None：面板仅在 Reality 打开）
            1 => {}
            _ => {
                if let Some(l) = g.life.as_ref() {
                    draw_overview(g, l);
                }
            }
        }
        if npc_tab {
            if act.is_none() {
                act = draw_panel_chrome_act(g);
            } else {
                draw_panel_chrome_act(g); // 已有 act 仍补画 chrome（覆盖关系不变）
            }
        }
    }
    if let Some(a) = act {
        exec_panel_act(g, a);
    }
}

// v1.10.2：charpanel 各页只读绘制期间收集的 &mut 交互——借用归还后统一执行
enum PanelAct {
    Close,          // ×：关面板 + 页签/滚动会话态归零
    Tab(u8),        // 页签切换
    Gift(usize),    // 赠礼（gift_npc 单口，内部再校验灵石）
    Unequip(usize), // 卸下法宝
    Equip(usize),   // 装备法宝
    Title(i32),     // 佩戴称号
    Zoom(crate::game::ZoomArt), // v1.16.1：立绘鉴赏开启（人物页卡立绘点按）
    DualCult,       // v1.17.0：双修共进（act_dual_cult 单口，内部再全门控校验）
}

fn exec_panel_act(g: &mut Game, a: PanelAct) {
    match a {
        PanelAct::Close => {
            g.panel = 0;
            g.char_tab = 0; // v1.5.0：关面板同时归零页签（与 Esc 行为一致）
            g.npc_scroll_reset(); // v1.10.2：人物页滚动会话态同批归零（与 Esc 行为一致）
            g.play(crate::sounds::Which::Page);
        }
        PanelAct::Tab(i) => {
            // v1.17.0 P0 修（道侣专页立绘「整页空白」确定性根因）：切页签整批归零
            // 滚动会话态——npc_scroll/drag/vel/anim + ev_recall 全在单口
            // npc_scroll_reset 内（先例 ×/Esc 臂 charpanel.rs:88、main.rs:341）。
            // 三页共用 g.npc_scroll 而各页 max 不同（人物页 1698 / 专页 554 / 图鉴
            // 自有），切页签不清则人物页翻底看道侣卡（scroll=1698）→点「道侣」页签，
            // 专页主卡/缘谱 y 全为负被整卡剔除→视口只剩页签+页脚一行字的整页空白，
            // 拖拽也先钳到 554 见缘谱后半、拖到 0 才见主卡立绘（静止不钳，唯一清除
            // 口就是本臂）。ev_recall 清态（v1.16.11 防御）并入本口，语义不变。
            // 散清会留惯性/lerp 跨页泄漏，故必须走单口整批清（红线口径）。
            g.npc_scroll_reset();
            // v1.16.0：图鉴页首开帧一次性灌字形（事件帧预热先例 show_banner/指引横带；
            // 仅 PanelAct::Tab(4) 执行帧触发，C 键带陈旧 char_tab=4 直开的首帧温开可接受）
            if i == 4 {
                crate::gallery::prewarm_corpus();
                // v1.16.10 遭遇手札：静态串 + 已遇事件标题/简一（未遇不灌防图集超量）
                crate::gallery::prewarm_event_book(g);
            }
            // v1.16.5 旧稿页首开帧预热（同图鉴先例；C 键带陈旧 char_tab=5 直开的
            // 首帧温开可接受——与 Tab(4) 同一口径）
            if i == 5 {
                prewarm_hist(g);
            }
            // v1.16.15 成就·称号页首开帧预热：格 18『执笔人』为新静态名（未达成
            // 档最常见态也画灰行）——冷栅格化会触发 2048² 图集整图重传，照图鉴/
            // 旧稿先例开帧一次灌齐
            if i == 2 {
                prewarm_title_page();
            }
            // v1.17.0 道侣专页首开帧预热（同图鉴/旧稿先例）：页静态串 + 十伴名补
            // 24 档（主卡名号 24px）——漏灌则首开帧冷栅格化触发图集整图重传
            if i == 6 {
                prewarm_comp_page();
            }
            g.char_tab = i;
            g.play(crate::sounds::Which::Page);
        }
        PanelAct::Gift(i) => crate::npc::gift_npc(g, i),
        PanelAct::Unequip(r) => {
            g.play(crate::sounds::Which::Click);
            g.unequip_relic(r);
        }
        PanelAct::Equip(k) => {
            g.play(crate::sounds::Which::Ascend);
            g.equip_relic(k);
        }
        PanelAct::Title(v) => g.set_worn_title(v),
        PanelAct::Zoom(z) => {
            // v1.16.1 立绘鉴赏：开启帧灌字形 + 置覆层态（draw_reality 早退链独占）
            prewarm_zoom_text(g, &z);
            g.zoom_art = Some(z);
            g.play(crate::sounds::Which::Click);
        }
        PanelAct::DualCult => {
            // v1.17.0 双修共进：执行口 act_dual_cult 单源（门控/定价/收益/落盘全在口内；
            // 音效随执行成败由口内自定，此处零旁路）
            g.act_dual_cult();
        }
    }
}

// 标题 / × / 页签（v1.10.2 自 draw_char_panel 抽出；v1.10.2 借用化：只读绘制 +
// 收集命中，执行移交 exec_panel_act）
fn draw_panel_chrome_act(g: &Game) -> Option<PanelAct> {
    txt("角色", 170.0, 92.0, 24, C_GOLD);
    if button(1064.0, 74.0, 44.0, 36.0, "×", None, true) {
        return Some(PanelAct::Close);
    }
    // ---------- 页签（v1.6.0 D：第 4 页「人物」；v1.16.0：第 5 页「图鉴」；v1.16.5：第 6 页「旧稿」；
    // v1.17.0：第 7 页「道侣」专页——7 钮宽 116→98，五处同批：本数组/宽两处/净空注释/npc_tab/预热分支） ----------
    // 7 钮 x=300+i*98（宽 140→116→98）：末钮 888..986，与 × 钮（1064..1108）净空 78px 不重叠
    let tabs = ["总览", "法宝", "成就·称号", "人物", "图鉴", "旧稿", "道侣"];
    for (i, t) in tabs.iter().enumerate() {
        let x = 300.0 + i as f32 * 98.0;
        if button(x, 74.0, 98.0, 34.0, t, None, true) {
            return Some(PanelAct::Tab(i as u8));
        }
        if g.char_tab == i as u8 {
            draw_rectangle(x, 108.0, 98.0, 3.0, C_GOLD);
        }
    }
    None
}

// ---------- 页 3：人物（v1.6.0 D 好感度；v1.10.2 单列纵向滚动列表改版）----------
// 5 位人物（0..=3 四 NPC / 4 道侣）同一纵列滚动，不再藏进翻页：卡 960×430、间距 12、
// 步距 442（v1.16.1 立绘加高：卡 330→430，立绘盒向下延伸 310→402——玩家反馈
// 「人物画像太小，可以往下翻」），视口 x160..1120 / y116..616（高 500），内容高
// 5×430+4×12=2198，滚动偏移钳 [0, 1698]；静止对齐（scroll=i×442，末卡钳 1698）
// 一屏 = 1 整卡 + 下一卡顶部 58px（人名露出）。交互：拖动 1:1 跟手 + 松手惯性
// （vel×0.9/帧，|vel|<0.5 停）+ 桌面滚轮 ±48px/格（工程首用 mouse_wheel）+ 右缘
// 页点/底部头像条 lerp 0.2/帧跳转；静止向最近卡顶吸附。◀▶ 翻页移除，由滚轮+页点+
// 头像条替代。
// 溢出遮挡双保险（macroquad scissor 不可用：get_context 私有 vendor lib.rs:497，且
// 引擎 flush 按自身 clip 状态重设 scissor vendor quad_gl.rs:768-771）：
//   ① 遮幅带——上 104..116（卡间 12px 间隙过顶时上一卡底边最低到 y=104）、下
//     616..654（卡顶锚定 116 且 scroll 钳 ≥0，卡永不越下缘；收到 654 止，勿盖
//     panel 内框线 y655，draw.rs panel_card 内框）；
//   ② 逐元素视口裁剪——卡体/边线/立绘（纹理源等比裁剪）/按钮按视口求交，文本行
//     越界整行跳过。必要：max scroll 1698 时上一卡顶可到 y=-282，遮幅带盖不住越出
//     面板的溢出（y<60 会画到面板外暗场上）。
// 性能：整卡剔除（card_y≥616 或 card_y+430≤116 连立绘查询一并跳过，任意帧 ≤2 卡在
// 画）；LazyTex 仅可见卡与头像条取纹理，首开最多解码 npc_elder/npc_merchant 两张
// （无立绘卡零解码，LazyTex 缓存 None 不重查）。立绘 contain 收进 220×402、底边
// 对齐（实测素材宽高比 0.456..0.628 → 绘宽 183..220，较 v1.16.0 的 141..195 加宽
// 30%+ 不变形）；NPC 无 AI 图回退 v1.14.0 程序化像素立绘（art::npc_pixel_tex，
// 宁清霜/醉道人不再拿 12×12 图标冒充人像），道侣未结缘回退书卷剪影
// （art::comp_silhouette_tex），已结缘恒 comp_0..9 真立绘。AI 立绘在卡内可点按
// 全屏鉴赏（v1.16.1 zoom_art 覆层，「放大」徽标提示）。好感度/档次刻度/增益/赠礼
// 交互与数据全部保留，gift_npc 单口不动（npc.rs:269）；滚动会话态不进存档
// （×/Esc 归零见 npc_scroll_reset）。
const NPC_VIEW_Y0: f32 = 116.0; // 视口顶（页签下划线下缘）
const NPC_VIEW_Y1: f32 = 616.0; // 视口底
const NPC_CARD_H: f32 = 430.0; // 卡高（v1.16.1：330→430，立绘盒向下延伸）
const NPC_STEP: f32 = 442.0; // 卡高 + 间距 12
const NPC_SCROLL_MAX: f32 = 1698.0; // 内容 2198 − 视口 500

// 视口内竖向裁剪：返回可见段 (y, h)，None = 完全在视口外
fn npc_clip_band(y: f32, h: f32) -> Option<(f32, f32)> {
    let t = y.max(NPC_VIEW_Y0);
    let b = (y + h).min(NPC_VIEW_Y1);
    if b > t { Some((t, b - t)) } else { None }
}

// 视口内文本行：越界整行跳过（字形上探 ≤2px 由上遮幅兜底、下探由下遮幅兜底）
// v1.16.0：pub(crate) 供图鉴页共用（视口同值）
pub(crate) fn npc_row_vis(y: f32) -> bool {
    y >= NPC_VIEW_Y0 && y < NPC_VIEW_Y1
}

// v1.14.0 识人小传：好感→已解锁段数（20/40/70 三档，对齐 NPC_FAV_TIERS；
// 道侣段叠加 companion_lives≥1 首段保底，见各卡调用处）——解锁纯推导零存档键
// v1.16.0：pub(crate) 供图鉴页当前道侣卡同口径取段
pub(crate) fn tale_unlocked(fav: i32) -> usize {
    (fav >= 20) as usize + (fav >= 40) as usize + (fav >= 70) as usize
}

// 卡体 + 边线（全部经视口裁剪；上下线仅整线可见时画，避免跨界 1-2px 残影）
// v1.16.0：卡高参数化 h（人物页传 NPC_CARD_H、图鉴传 200），裁剪/描线口径不变；pub(crate) 供图鉴共用
pub(crate) fn npc_card_chrome(x: f32, y: f32, h: f32, bg: Color, border: Color) {
    if let Some((ty, th)) = npc_clip_band(y, h) {
        draw_rectangle(x, ty, 960.0, th, bg);
        draw_rectangle(x, ty, 2.0, th, border);
        draw_rectangle(x + 958.0, ty, 2.0, th, border);
        if y >= NPC_VIEW_Y0 {
            draw_rectangle(x, y, 960.0, 2.0, border);
        }
        if y + h <= NPC_VIEW_Y1 {
            draw_rectangle(x, y + h - 2.0, 960.0, 2.0, border);
        }
    }
}

// 立绘：contain 收进 220×402、底边对齐（v1.16.1 卡高 430→盒向下延伸 310→402，
// 公式沿用）；并按视口上下缘对纹理源等比裁剪（source 为纹理像素，vendor
// texture.rs:537-545 按纹理尺寸归一）；v1.14.0 起兜底像（28×40 像素立绘/书卷剪影）
// 由调用方经 or_else 以 Some 传入；icon 仅剩 tex=None 防御路径（12×12 ×12=144）
// v1.16.1：薄包装改传 (220,402,418)——bottom_off 418 = 卡高 430 − 内缩 12
fn npc_portrait(tex: Option<&Texture2D>, icon: &Texture2D, x: f32, card_y: f32) {
    npc_portrait_box(tex, icon, x, card_y, 220.0, 402.0, 418.0);
}

/// v1.16.0 交游图鉴共用：box 参数化立绘（contain 收进 box_w×box_h、底边锚 card_y+bottom_off、
/// 源矩形按视口等比裁剪——与人物页同语言）。人物页 (220,310,318)，图鉴 (150,176,188)。
pub(crate) fn npc_portrait_box(
    tex: Option<&Texture2D>,
    icon: &Texture2D,
    x: f32,
    card_y: f32,
    box_w: f32,
    box_h: f32,
    bottom_off: f32,
) {
    npc_portrait_box_a(tex, icon, x, card_y, box_w, box_h, bottom_off, 1.0);
}

/// 剧情演出包③：带透明度立绘（邂逅卡入场「入画」淡入，滞后卡体 0.05s 分层显现）。
/// alpha=1.0 时 tint=WHITE，与原 npc_portrait_box 逐像素一致；人物页/图鉴既有
/// 调用点经包装零改动。
pub(crate) fn npc_portrait_box_a(
    tex: Option<&Texture2D>,
    icon: &Texture2D,
    x: f32,
    card_y: f32,
    box_w: f32,
    box_h: f32,
    bottom_off: f32,
    alpha: f32,
) {
    let a = alpha.clamp(0.0, 1.0);
    let (dw, dh) = match tex {
        Some(t) => {
            let ar = t.width() / t.height().max(1.0);
            let w = (box_h * ar).min(box_w);
            (w, w / ar)
        }
        None => (144.0f32.min(box_w), 144.0f32.min(box_h)),
    };
    let top = card_y + bottom_off - dh; // 底边对齐（bottom_off = 底边相对卡顶偏移）
    let Some((vy, vh)) = npc_clip_band(top, dh) else {
        return;
    };
    let (tw, th) = match tex {
        Some(t) => (t.width(), t.height().max(1.0)),
        None => (12.0, 12.0),
    };
    draw_texture_ex(
        tex.unwrap_or(icon),
        x + (box_w - dw) * 0.5,
        vy,
        Color::new(1.0, 1.0, 1.0, a),
        DrawTextureParams {
            source: Some(Rect::new(0.0, (vy - top) / dh * th, tw, vh / dh * th)),
            dest_size: Some(vec2(dw, vh)),
            ..Default::default()
        },
    );
}

// ---------- v1.16.1 立绘鉴赏（点立绘放大） ----------
// 入口三件套：「放大」徽标（卡内立绘盒右下角，纯视觉提示）+ 点按命中（调用方
// 按各自几何判定）+ 全屏覆层（draw_zoom_overlay，draw_reality 早退链独占）。
// AI 立绘存在才可点/显徽标（像素兜底/书卷剪影不进鉴赏——放大无意义）。

// 「放大」徽标：立绘盒右下角 58×22（badge 需 AI 立绘实际存在才显示，调用方判定；
// npc_row_vis 防滚动越缘）。人物页 (220,418) / 图鉴 (150,188)。
pub(crate) fn zoom_badge(x: f32, y: f32, box_w: f32, bottom_off: f32) {
    let bx = x + box_w - 58.0;
    let by = y + bottom_off - 26.0;
    if !npc_row_vis(by) {
        return;
    }
    draw_rectangle(bx, by, 58.0, 22.0, Color::from_rgba(14, 10, 26, 215));
    draw_rectangle_lines(bx, by, 58.0, 22.0, 1.0, C_GOLD_D);
    txt_c("放大", bx + 29.0, by + 3.0, 14, C_GOLD_D);
}

// 同页人鉴赏档次取值：当前道侣显 Life.comp_fav 实时值，历世卡显初识档（图鉴口径）
fn comp_zoom_fav(g: &Game, i: usize) -> i32 {
    let cur = g
        .life
        .as_ref()
        .and_then(|l| l.companion.as_deref())
        .and_then(|cn| COMPANION_NAMES.iter().position(|&n| n == cn));
    if cur == Some(i) {
        g.life.as_ref().map_or(0, |l| l.comp_fav)
    } else {
        0
    }
}

/// 鉴赏覆层开启帧一次性灌字形（事件帧预热先例 show_banner/指引横带；名字 20/24、
/// 档次 14/16、关闭提示 14——均既有离散档；ASCII 由菜单期保底覆盖）。人物页
/// exec_panel_act 与图鉴页直开两路共用本单口。
pub(crate) fn prewarm_zoom_text(g: &Game, z: &ZoomArt) {
    prewarm_text("点按任意处返回", &[14]);
    match *z {
        ZoomArt::Npc(i) => {
            let i = i.min(NPC_NAMES.len() - 1);
            prewarm_text(NPC_NAMES[i], &[20, 24]);
            prewarm_text(crate::npc::tier_name(g.meta.npc_fav[i]), &[14, 16]);
        }
        ZoomArt::Comp(i) => {
            let i = i.min(COMPANION_NAMES.len() - 1);
            prewarm_text(COMPANION_NAMES[i], &[20, 24]);
            prewarm_text(crate::npc::tier_name(comp_zoom_fav(g, i)), &[14, 16]);
        }
    }
}

// 鉴赏覆层（模态独占输入，照 rmodal/ending_show 早退先例）：全屏暗幕 + 立绘 contain
// 大图（高 550 收进、底边 y=650 居中，较卡内立绘放大约 1.8 倍）+ 名牌（名字 24 /
// 档次 16 居中）+ 底缘提示。任意点按关闭；开启帧由入口 prewarm_zoom_text 灌字形，
// 本函数稳态零分配（全 &'static str，无 format!/clone）。
pub fn draw_zoom_overlay(g: &mut Game) {
    let Some(z) = g.zoom_art else { return };
    let (tex, name, tier, col) = match z {
        ZoomArt::Npc(i) => {
            let i = i.min(NPC_NAMES.len() - 1);
            (
                crate::art::art().npc_tex(i),
                NPC_NAMES[i],
                crate::npc::tier_name(g.meta.npc_fav[i]),
                C_GOLD,
            )
        }
        ZoomArt::Comp(i) => {
            let i = i.min(COMPANION_NAMES.len() - 1);
            (
                crate::art::art().comp_tex(i),
                COMPANION_NAMES[i],
                crate::npc::tier_name(comp_zoom_fav(g, i)),
                C_PURPLE,
            )
        }
    };
    let Some(t) = tex else {
        g.zoom_art = None; // 防御合页：纹理缺失不该发生（入口已判 AI 图存在）
        return;
    };
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::from_rgba(8, 5, 16, 244));
    let ar = t.width() / t.height().max(1.0);
    let dw = (550.0 * ar).min(V_W - 120.0);
    let dh = dw / ar;
    draw_texture_ex(
        &t,
        (V_W - dw) * 0.5,
        650.0 - dh,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(dw, dh)),
            ..Default::default()
        },
    );
    txt_c(name, V_W * 0.5, 34.0, 24, col);
    txt_c(tier, V_W * 0.5, 72.0, 16, C_DIM);
    txt_c("点按任意处返回", V_W * 0.5, 668.0, 14, C_DIM);
    if is_mouse_button_pressed(MouseButton::Left) {
        g.zoom_art = None;
        g.play(crate::sounds::Which::Page);
    }
}

// v1.16.0：滚动状态机自 draw_npc_page 抽出为单口——图鉴页（gallery.rs）与人物页共用
// 同一状态机与会话字段（g.npc_scroll/npc_drag/npc_vel/npc_anim；两页不同帧显示，值跨页
// 由各自 max 钳制收敛；npc_scroll_reset 关面板统一归零）。纯搬移自原 205-279 行：
// 拖动/惯性/滚轮/吸附/lerp 逻辑逐行未动，仅 NPC_SCROLL_MAX/NPC_STEP 参数化为
// max/step；返回 tap（位移<8px 的点按帧标记，人物页赠礼钮唯一激活口径）。
// v1.16.10 复审修：新增 snap 关断——静止对齐吸附是「向 step 栅格 lerp 拉回」，
// 只适合与 step 同栅格的整卡区；图鉴 v1.16.10 遭遇手札区为 52px 行距（与 292 卡
// 栅格失配），吸附会把行卡切在视口缘且停手后持续抖动，该区须传 false 自由停留
// （边界行被视口缘裁切属任意列表的正常滚动形态，bug 在「拉回」不在「裁切」）。
pub(crate) fn scroll_list_tick(g: &mut Game, max: f32, step: f32, snap: bool) -> bool {
    // ---------- 滚动状态机（会话态，不进存档）----------
    let (mx, my) = crate::cam::mouse();
    let pressed = is_mouse_button_pressed(MouseButton::Left);
    let released = is_mouse_button_released(MouseButton::Left);
    // 桌面滚轮：±48px/格；打断跳转动画与惯性
    let (_, wy) = mouse_wheel();
    if wy != 0.0 {
        g.npc_anim = None;
        g.npc_vel = 0.0;
        g.npc_scroll = (g.npc_scroll - wy * 48.0).clamp(0.0, max);
    }
    // 按下且在视口内：记锚点（按下x, 按下y, 按下时scroll）。按压帧不触发任何按钮——
    // 视口内按钮一律「抬起激活 + 位移<8px 门槛」，从赠礼钮起手拖动不再误触赠礼
    // （旧 button() 按下帧即触发 draw.rs:85，本页按钮因此弃用之改手工命中）
    if pressed && mx >= 160.0 && mx <= 1120.0 && my >= NPC_VIEW_Y0 && my <= NPC_VIEW_Y1 {
        g.npc_drag = Some((mx, my, g.npc_scroll));
        g.npc_anim = None;
        g.npc_vel = 0.0;
    }
    let mut tap = false;
    if let Some((px, py, s0)) = g.npc_drag {
        let dist = ((mx - px) * (mx - px) + (my - py) * (my - py)).sqrt();
        if !released {
            // 按住：越 8px 门槛才进入拖动
            if dist >= 8.0 {
                // 1:1 跟手（自锚点起算）；速度平滑记录供松手惯性（钳 ±40 防起手越槛帧尖峰）
                let ns = (s0 - (my - py)).clamp(0.0, max);
                let dv = (ns - g.npc_scroll).clamp(-40.0, 40.0);
                g.npc_vel = g.npc_vel * 0.5 + dv * 0.5;
                g.npc_scroll = ns;
            }
        } else {
            // 抬起：结束拖动；位移<8px 视为点按（视口内按钮唯一激活口径）
            g.npc_drag = None;
            if dist < 8.0 {
                tap = true;
                g.npc_vel = 0.0;
            }
        }
    } else if g.npc_anim.is_none() {
        // 松手惯性：vel×0.9/帧，|vel|<0.5 停，撞钳制即停
        if g.npc_vel.abs() >= 0.5 {
            let raw = g.npc_scroll + g.npc_vel;
            if (g.npc_vel > 0.0 && raw >= max) || (g.npc_vel < 0.0 && raw <= 0.0) {
                g.npc_scroll = raw.clamp(0.0, max);
                g.npc_vel = 0.0;
            } else {
                g.npc_scroll = raw;
                g.npc_vel *= 0.9;
            }
        } else {
            g.npc_vel = 0.0;
            // 静止对齐：向最近卡顶吸附（lerp 0.2/帧；末卡钳 max）——任意静止态
            // =整卡+邻卡露头，页点/头像条的「当前卡」高亮恒有唯一锚。
            // snap=false 时整段跳过（自由停留，v1.16.10 图鉴手札区口径）
            if snap {
                let target = ((g.npc_scroll / step).round() * step).clamp(0.0, max);
                let d = target - g.npc_scroll;
                if d.abs() > 0.5 {
                    g.npc_scroll += d * 0.2;
                } else {
                    g.npc_scroll = target;
                }
            }
        }
    }
    // 页点/头像条跳转：lerp 0.2/帧平滑趋近，落点即目标卡顶（收敛门限 0.5px ≤2px 达标）
    if let Some(t) = g.npc_anim {
        let d = t - g.npc_scroll;
        if d.abs() <= 0.5 {
            g.npc_scroll = t;
            g.npc_anim = None;
            g.npc_vel = 0.0;
        } else {
            g.npc_scroll += d * 0.2;
        }
    }
    tap
}

// v1.16.5 人物页道侣卡立绘下标判定（纯函数便于单测）：此生 companion 优先；
// 未伴时走历世账兜底——缝页缘定之人（feng_bn，save.rs 键 bn= 持久化）优先，
// 否则历世结缘账 comp_bonds 首个已识者；全 0 返回 None → 书卷剪影路径原样
// （未结缘玩家画面零变化）。索引全部双卫（≠FENG_UNSET 且 <len，panic=abort 纪律）。
// 语义对齐图鉴 known 判定（gallery.rs「comp_bonds[i]>0 || cur」的历世半边）。
fn comp_portrait_idx(l: Option<&Life>, meta: &Meta) -> Option<usize> {
    if let Some(ci) = l.and_then(|x| {
        x.companion
            .as_deref()
            .and_then(|cn| COMPANION_NAMES.iter().position(|&n| n == cn))
    }) {
        return Some(ci);
    }
    let n = COMPANION_NAMES.len();
    let fb = meta.feng_bn;
    if fb != FENG_UNSET && (fb as usize) < n && meta.comp_bonds[fb as usize] > 0 {
        return Some(fb as usize);
    }
    (0..n).find(|&i| meta.comp_bonds[i] > 0)
}

// v1.10.2 借用化：签名去 l——滚动状态机/页点/头像条为 &mut 直写；仅卡片绘制段
// 借 life 不可变引用（块尾归还），命中赠礼以 PanelAct::Gift 跨借用传出
fn draw_npc_page(g: &mut Game) -> Option<PanelAct> {
    // ---------- 滚动状态机（单口见 scroll_list_tick；v1.16.0 抽出供图鉴页共用）----------
    let tap = scroll_list_tick(g, NPC_SCROLL_MAX, NPC_STEP, true);
    let (mx, my) = crate::cam::mouse();
    let pressed = is_mouse_button_pressed(MouseButton::Left);

    // ---------- 卡片（整卡剔除：任意帧 ≤2 卡在画，剔除含立绘查询）----------
    // v1.7.1/1.7.2：有道侣且有 AI 立绘（comp_0..9 按结缘顺序）则替代丹药图标。
    // 卡片绘制段的 life 不可变借用限定在此块内（块尾归还），命中赠礼以 Gift 传出；
    // v1.16.1 立绘点按命中以 Zoom 传出（赠礼钮在文本列 x+260 起与立绘盒无像素重叠）
    let (comp_tex, gifted, zoom_hit) = {
        let l = g.life.as_ref();
        // v1.16.5 补历世半边（原只读此生 companion）：模拟中结缘、落幕未带回现实
        // （life.companion 唯一写点=遗泽 idx8）或旧档缺 Lcomp 键时，凭 feng_bn /
        // comp_bonds 历世账兜底显真立绘；未结缘仍 None → 剪影
        let comp_idx = comp_portrait_idx(l, &g.meta);
        let comp_tex = comp_idx.and_then(|ci| crate::art::art().comp_tex(ci));
        let mut gifted = None;
        let mut zoom_hit: Option<crate::game::ZoomArt> = None;
        for i in 0..5usize {
            let card_y = NPC_VIEW_Y0 + i as f32 * NPC_STEP - g.npc_scroll;
            if card_y >= NPC_VIEW_Y1 || card_y + NPC_CARD_H <= NPC_VIEW_Y0 {
                continue;
            }
            // v1.16.1 点立绘放大：AI 立绘存在且点按落在立绘盒（176..396 × 卡顶+16..底-12）
            // ——像素兜底/书卷剪影不进鉴赏
            let art_hit = if i < 4 {
                crate::art::art().npc_tex(i).is_some()
            } else {
                comp_tex.is_some()
            };
            if art_hit
                && tap
                && zoom_hit.is_none()
                && mx >= 176.0
                && mx <= 396.0
                && my >= card_y + 16.0
                && my <= card_y + 418.0
            {
                zoom_hit = Some(if i < 4 {
                    crate::game::ZoomArt::Npc(i)
                } else {
                    // comp_tex Some ⟹ comp_idx Some（同源推导）；unwrap_or(0) 为纪律 1 防御
                    crate::game::ZoomArt::Comp(comp_idx.unwrap_or(0))
                });
            }
            let done = if i < 4 {
                draw_npc_card(g, l, i, card_y, mx, my, tap)
            } else {
                draw_comp_card(g, l, card_y, comp_tex.as_ref());
                false
            };
            if done {
                gifted = Some(i);
                break;
            }
        }
        (comp_tex, gifted, zoom_hit)
    };
    if let Some(i) = gifted {
        // 赠礼帧到此为止（先例：赠礼后 return；chrome 由调用方补画）；
        // 执行移交 exec_panel_act（滚动偏移在 Game 会话态，位置不丢）
        return Some(PanelAct::Gift(i));
    }
    if let Some(z) = zoom_hit {
        return Some(PanelAct::Zoom(z));
    }

    // ---------- 遮幅（口径见本页头注释；C_PANEL α235 半透不蔽物，用其全不透明版）----------
    let mask = Color::from_rgba(22, 16, 38, 255);
    draw_rectangle(150.0, 104.0, 980.0, 12.0, mask);
    draw_rectangle(150.0, 616.0, 980.0, 38.0, mask);

    // ---------- 发现性①：右缘页点（当前卡金色实心；点击 lerp 跳转）----------
    let cur = ((g.npc_scroll / NPC_STEP).round() as i32).clamp(0, 4) as usize;
    for i in 0..5usize {
        let cy = 366.0 + (i as f32 - 2.0) * 18.0;
        if i == cur {
            draw_circle(1128.0, cy, 4.5, C_GOLD);
        } else {
            draw_circle(1128.0, cy, 3.0, Color::from_rgba(96, 82, 124, 255));
        }
        if pressed && mx >= 1121.0 && mx <= 1136.0 && my >= cy - 9.0 && my <= cy + 9.0 {
            g.npc_anim = Some(((i as f32) * NPC_STEP).min(NPC_SCROLL_MAX));
            g.play(crate::sounds::Which::Page);
        }
    }

    // ---------- 发现性②：底部头像跳转条（名册常驻可见，「找不到人物」的直接解）----------
    // 头像取立绘顶部约 35% 高度方形头裁剪（1:1 不变形）；v1.14.0 兜底同步卡内口径：
    // NPC 缺 AI 图用像素立绘（28×40 → 头裁剪 14×14 整数倍放大不糊）、道侣未结缘用
    // 书卷剪影（已结缘 comp_0..9 恒真立绘）；当前卡金框
    for i in 0..5usize {
        let (ax, ay) = (920.0 + i as f32 * 42.0, 620.0);
        let (tex, icon) = if i < 4 {
            (
                crate::art::art()
                    .npc_tex(i)
                    .or_else(|| crate::art::npc_pixel_tex(i)),
                &crate::art::art().icon_yun,
            )
        } else {
            (
                comp_tex.clone().or_else(crate::art::comp_silhouette_tex),
                &crate::art::art().icon_pill,
            )
        };
        match tex.as_ref() {
            Some(t) => {
                let side = (t.height().max(1.0) * 0.35).min(t.width());
                draw_texture_ex(
                    t,
                    ax,
                    ay,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(
                            (t.width() - side) * 0.5,
                            0.0,
                            side,
                            side,
                        )),
                        dest_size: Some(vec2(32.0, 32.0)),
                        ..Default::default()
                    },
                );
            }
            None => {
                draw_texture_ex(
                    icon,
                    ax + 4.0,
                    ay + 4.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(24.0, 24.0)),
                        ..Default::default()
                    },
                );
            }
        }
        if i == cur {
            draw_rectangle_lines(ax - 2.0, ay - 2.0, 36.0, 36.0, 2.0, C_GOLD);
        } else {
            draw_rectangle_lines(ax, ay, 32.0, 32.0, 1.0, Color::from_rgba(96, 82, 124, 255));
        }
        if pressed && mx >= ax && mx <= ax + 32.0 && my >= ay && my <= ay + 32.0 {
            g.npc_anim = Some(((i as f32) * NPC_STEP).min(NPC_SCROLL_MAX));
            g.play(crate::sounds::Which::Page);
        }
    }

    // ---------- 发现性③：首开提示（offset≈0 时；新文案字符待脚本统一重扫字体子集）----------
    if g.npc_scroll <= 0.5 && g.npc_drag.is_none() && g.npc_anim.is_none() {
        draw_rectangle(545.0, 592.0, 190.0, 22.0, Color::from_rgba(14, 10, 26, 210));
        txt_c(
            "上下滑动查看全部 5 位",
            640.0,
            596.0,
            14,
            Color::from_rgba(200, 196, 214, 220),
        );
    }
    // v1.14.0 识人小传页脚：全图鉴解锁进度 x/15（4 NPC×3 段 + 道侣 3 段，纯推导）
    let mut tales = 0usize;
    for f in g.meta.npc_fav.iter() {
        tales += tale_unlocked(*f);
    }
    let cfav = g.life.as_ref().map_or(0, |x| x.comp_fav);
    tales += tale_unlocked(cfav).max((g.meta.companion_lives >= 1) as usize);
    txt(
        &format!(
            "好感随交往与赠礼增长，跨世持久（随轮回烙印存档）；赠礼价随交情水涨船高。 · 识人小传 {}/15",
            tales
        ),
        170.0,
        630.0, 14,
        C_DIM,
    );
    None
}

// NPC 大卡（i=0..=3）：好感/档次刻度/增益/简介/赠礼，取值口径与 v1.9.25 版一致，
// 行距随卡高 450→330 压缩迁移；返回 true 表示本帧命中赠礼（调用方以 Gift 传出）
fn draw_npc_card(
    g: &Game,
    l: Option<&Life>,
    i: usize,
    y: f32,
    mx: f32,
    my: f32,
    tap: bool,
) -> bool {
    let x = 160.0;
    let fav = g.meta.npc_fav[i];
    let cost = crate::npc::gift_cost(fav);
    npc_card_chrome(x, y, NPC_CARD_H, Color::from_rgba(30, 24, 50, 210), C_GOLD_D);
    // v1.14.0：AI 图缺失时程序化像素立绘兜底（or_else 短路——AI 图命中永不触发，
    // 放入 assets/portraits/npc_senior.png / npc_drunk.png 重启即自动覆盖）
    let pix = crate::art::art()
        .npc_tex(i)
        .or_else(|| crate::art::npc_pixel_tex(i));
    npc_portrait(pix.as_ref(), &crate::art::art().icon_yun, x + 16.0, y);
    // v1.16.1 立绘鉴赏入口徽标（仅 AI 立绘显示；像素兜底不进鉴赏）
    if crate::art::art().npc_tex(i).is_some() {
        zoom_badge(x + 16.0, y, 220.0, 418.0);
    }
    let col = x + 260.0;
    if npc_row_vis(y + 16.0) {
        txt(NPC_NAMES[i], col, y + 16.0, 24, C_GOLD);
    }
    if npc_row_vis(y + 54.0) {
        txt(crate::npc::tier_name(fav), col, y + 54.0, 16, C_PURPLE);
    }
    if npc_row_vis(y + 82.0) {
        txt(
            &format!("好感 {}/{}", fav, NPC_FAV_MAX),
            col,
            y + 82.0, 14,
            C_TEXT,
        );
    }
    if y + 102.0 >= NPC_VIEW_Y0 && y + 124.0 <= NPC_VIEW_Y1 {
        bar(
            col,
            y + 106.0,
            560.0,
            14.0,
            fav as f32 / NPC_FAV_MAX as f32,
            C_PURPLE,
        );
        // 档次刻度（沿 A3 样式：3px / (188,168,224,235)）
        for &t in NPC_FAV_TIERS.iter() {
            let tx = col + t as f32 / NPC_FAV_MAX as f32 * 560.0;
            draw_rectangle(tx - 1.5, y + 102.0, 3.0, 22.0, Color::from_rgba(188, 168, 224, 235));
        }
    }
    // 增益行（按 NPC 各表取值，口径不变）
    let eff = match i {
        crate::npc::NPC_ELDER => {
            format!("当前：宗门任务收益 ×{:.2}", crate::npc::elder_quest_mult(fav))
        }
        crate::npc::NPC_MERCHANT => {
            format!("当前：坊市丹价 ×{:.2}", crate::npc::merchant_disc(fav))
        }
        crate::npc::NPC_SENIOR => format!(
            "当前：跨境破境率 +{:.0}%",
            crate::npc::senior_break_add(fav) * 100.0
        ),
        _ => format!(
            "当前：每载修炼 ×{:.2}",
            crate::npc::drunk_cult_mult(fav)
        ),
    };
    if npc_row_vis(y + 136.0) {
        txt(&eff, col, y + 136.0, 14, C_CYAN);
    }
    if npc_row_vis(y + 168.0) {
        wrap_draw(NPC_BRIEF[i], col, y + 168.0, 14, 660.0, 22.0, C_DIM);
    }
    // v1.14.0 识人小传：常显最新解锁 1 段（14px ≤2 行，赠礼钮 y+258 之上不挤压）；
    // 未解锁给书道口径占位行
    let seg = tale_unlocked(fav);
    if seg > 0 {
        if npc_row_vis(y + 196.0) {
            wrap_draw(NPC_TALES[i][seg - 1], col, y + 196.0, 14, 640.0, 22.0, C_TEXT);
        }
    } else if npc_row_vis(y + 196.0) {
        txt("交情愈深，其人故事自会显于纸上。", col, y + 196.0, 14, C_DIM);
    }
    // 赠礼钮 230×60（v1.16.1 底对齐：卡高 430→y+358，随立绘盒向下延伸沉到卡底；
    // 仅整钮可见时绘出与可点）；gift_npc 内部再校验灵石（单口）
    let (bx, by) = (col, y + NPC_CARD_H - 72.0);
    if by < NPC_VIEW_Y0 || by + 60.0 > NPC_VIEW_Y1 {
        return false;
    }
    let hot = mx >= bx && mx <= bx + 230.0 && my >= by && my <= by + 60.0;
    let en = l.is_some_and(|x| x.stones >= cost);
    let base = if !en {
        Color::from_rgba(38, 32, 56, 225)
    } else if hot {
        Color::from_rgba(92, 66, 122, 245)
    } else {
        Color::from_rgba(56, 42, 86, 235)
    };
    draw_rectangle(bx, by, 230.0, 60.0, base);
    draw_rectangle_lines(
        bx,
        by,
        230.0,
        60.0,
        2.0,
        if en {
            C_GOLD_D
        } else {
            Color::from_rgba(96, 82, 124, 255)
        },
    );
    txt_c(
        "赠礼",
        bx + 115.0,
        by + 13.0,
        20,
        if en {
            C_TEXT
        } else {
            Color::from_rgba(184, 178, 196, 255)
        },
    );
    txt_c(
        &format!("{} 灵石", fmt_int(cost)),
        bx + 115.0,
        by + 34.0, 14,
        if en {
            C_DIM
        } else {
            Color::from_rgba(178, 172, 192, 255)
        },
    );
    tap && hot && en // 赠礼执行移交 exec_panel_act（gift_npc 单口、内部再校验灵石）
}

// 道侣大卡（i=4）：情谊/双修加成/结缘信息，文案与 v1.9.25 版一致；无赠礼钮
fn draw_comp_card(g: &Game, l: Option<&Life>, y: f32, comp_tex: Option<&Texture2D>) {
    let x = 160.0;
    let cfav = l.map_or(0, |x| x.comp_fav);
    npc_card_chrome(x, y, NPC_CARD_H, Color::from_rgba(42, 26, 52, 210), C_PURPLE);
    // v1.14.0：未结缘（comp_tex=None）改书卷剪影兜底；已结缘 comp_0..9 全内嵌恒真立绘
    let portrait = comp_tex
        .map(Clone::clone)
        .or_else(crate::art::comp_silhouette_tex);
    npc_portrait(portrait.as_ref(), &crate::art::art().icon_pill, x + 16.0, y);
    // v1.16.1 立绘鉴赏入口徽标（已结缘真立绘显示；书卷剪影不进鉴赏）
    if comp_tex.is_some() {
        zoom_badge(x + 16.0, y, 220.0, 418.0);
    }
    let col = x + 260.0;
    match l.and_then(|x| x.companion.as_deref()) {
        Some(cn) => {
            if npc_row_vis(y + 16.0) {
                txt(cn, col, y + 16.0, 24, C_PURPLE);
            }
            if npc_row_vis(y + 54.0) {
                txt(crate::npc::tier_name(cfav), col, y + 54.0, 16, C_PURPLE);
            }
            if npc_row_vis(y + 82.0) {
                txt(
                    &format!("情谊 {}/{}", cfav, NPC_FAV_MAX),
                    col,
                    y + 82.0, 14,
                    C_TEXT,
                );
            }
            if y + 106.0 >= NPC_VIEW_Y0 && y + 120.0 <= NPC_VIEW_Y1 {
                bar(
                    col,
                    y + 106.0,
                    560.0,
                    14.0,
                    cfav as f32 / NPC_FAV_MAX as f32,
                    C_PURPLE,
                );
            }
            if npc_row_vis(y + 136.0) {
                txt(
                    &format!(
                        "当前：双修加成 ×{:.2}（结缘起步+{:.0}%，只升不降）",
                        crate::npc::comp_cult_mult(cfav),
                        (crate::npc::comp_cult_mult(COMPANION_BOND_FAV) - 1.0) * 100.0
                    ),
                    col,
                    y + 136.0, 14,
                    C_CYAN,
                );
            }
            if npc_row_vis(y + 168.0) {
                txt(
                    "情谊随赠礼事件与遗泽积累；更换道侣亦不忘旧谊。",
                    col,
                    y + 168.0,
                    14,
                    C_DIM,
                );
            }
            if npc_row_vis(y + 194.0) {
                txt(
                    &format!("历世结缘 {} 次", g.meta.companion_lives),
                    col,
                    y + 194.0,
                    14,
                    C_DIM,
                );
            }
            // v1.14.0 识人小传：道侣槽共通段（首段结缘保底，段2/3 随情谊 40/70 解锁）
            let cseg = tale_unlocked(cfav).max((g.meta.companion_lives >= 1) as usize);
            if cseg > 0 && npc_row_vis(y + 222.0) {
                wrap_draw(COMP_TALES[cseg - 1], col, y + 222.0, 14, 640.0, 22.0, C_TEXT);
            }
        }
        None => {
            // v1.14.0 未结缘引导改版：三行全暗 → 标题紫提亮 + 暗金引导框（引用
            // 「同页天成」事件原文 game.rs:2901「同页天成」/『认下这页』，玩家可按图索骥）
            if npc_row_vis(y + 16.0) {
                txt("道侣 · 此生未结缘", col, y + 16.0, 24, C_PURPLE);
            }
            let (gx, gy, gw, gh) = (col, y + 52.0, 560.0, 88.0);
            if let Some((vy, vh)) = npc_clip_band(gy, gh) {
                draw_rectangle(gx, vy, gw, vh, Color::from_rgba(24, 17, 40, 190));
                draw_rectangle_lines(gx, vy, gw, vh, 1.5, C_GOLD_D);
            }
            if npc_row_vis(gy + 18.0) {
                txt(
                    "模拟中遇『同页天成』事件，选『认下这页』即可结缘。",
                    gx + 16.0,
                    gy + 18.0, 16,
                    C_TEXT,
                );
            }
            if npc_row_vis(gy + 50.0) {
                // v1.16.5 复审修复：历世兜底真图已同框显现，原行「结缘后…都会在此卡
                // 显现」自相矛盾（同 :912 下缘小字一处口径）——兜底态改历世承缘提示；
                // 未结缘无图态（剪影）原行照旧
                if comp_tex.is_some() {
                    txt(
                        "此绘承自历世之缘，此生再遇『同页天成』可再续。",
                        gx + 16.0,
                        gy + 50.0, 16,
                        C_TEXT,
                    );
                } else {
                    txt(
                        "结缘后，她的立绘、情谊与双修加成都会在此卡显现。",
                        gx + 16.0,
                        gy + 50.0, 16,
                        C_TEXT,
                    );
                }
            }
            if npc_row_vis(y + 156.0) {
                txt(
                    &format!(
                        "结缘起步情谊 {}（双修 +{:.0}%），道侣赠礼事件再 +{}。",
                        COMPANION_BOND_FAV,
                        (crate::npc::comp_cult_mult(COMPANION_BOND_FAV) - 1.0) * 100.0,
                        COMPANION_GIFT_FAV
                    ),
                    col,
                    y + 156.0, 14,
                    C_CYAN,
                );
            }
            // v1.14.0 识人小传：未结缘但历世有缘（companion_lives≥1 保底）亦显首段旧谊
            if g.meta.companion_lives >= 1 && npc_row_vis(y + 192.0) {
                wrap_draw(COMP_TALES[0], col, y + 192.0, 14, 640.0, 22.0, C_TEXT);
            }
            // 剪影下缘小字一行（直接回应「女角色立绘丢了」的误读；v1.16.1 随卡加高
            // 下沉：y+286/290 → y+386/390，仍盖在立绘盒底边上方）
            // v1.16.5 历世兜底显真图时不再画本行（否则「立绘自会在此显现」与真立绘同框矛盾）
            if comp_tex.is_none() && npc_row_vis(y + 390.0) {
                draw_rectangle(
                    x + 16.0,
                    y + 386.0,
                    220.0,
                    22.0,
                    Color::from_rgba(14, 10, 26, 210),
                );
                txt_c(
                    "结缘之后，立绘自会在此显现",
                    x + 126.0,
                    y + 390.0, 14,
                    C_DIM,
                );
            }
        }
    }
}

// ================= v1.17.0 页 6：道侣专页 =================
// 一页两段（滚动列表，会话态共用 g.npc_scroll 系列照图鉴先例，本页自有 max 钳制
// 收敛；snap=false——主卡 430 与缘谱行 52 两栅格失配，自由停留照图鉴手札区口径）：
//   主卡 960×430（照人物页道侣卡几何：立绘 220×402 + 右列文本）+ 间距 12
//   + 缘谱 11 行（十同页人 + 合计行；46 高 + 6 间隙步进 52，纯文本零立绘）。
// 内容高 = 430 + 12 + 11×52 = 1014，尾白 40；滚动钳 = 1014 + 40 − 500(视口) = 554。
// 性能红线：全页只允许主卡 1 次 comp_tex 查询（LazyTex 懒解码，未结缘 None → 剪影
// 兜底）；缘谱行严禁立绘（照图鉴「严禁预解码全部 10 张」红线）。立绘鉴赏 hit 照
// 人物页立绘盒口径 + tap 纵夹视口带（图鉴 tap_y_in_view 同语言）；双修钮 hit 抄
// 赠礼钮「整钮可见才可点 + 抬起位移<8px」口径。展示数据全在 gbd=/bn=/blv=/
// companion_lives/Lcomp/Lcfav 既有持久化字段，纯推导渲染零新存档键（per-伴初遇
// 世次如未来要加，照 gbd= 先例 additive，本轮明确不做）。
const COMP_CARD_H: f32 = 430.0; // 主卡高（照 NPC_CARD_H）
const COMP_GAP: f32 = 12.0; // 主卡与缘谱首行间距
const COMP_ROW_H: f32 = 46.0; // 缘谱行高（照图鉴事件行）
const COMP_ROW_STEP: f32 = 52.0; // 行步进（46 + 6）
const COMP_ROWS: usize = 11; // 十同页人 + 合计行
// 内容 430+12+11×52=1014；+40 尾白 − 500 视口 → 554
const COMP_SCROLL_MAX: f32 = COMP_CARD_H
    + COMP_GAP
    + COMP_ROWS as f32 * COMP_ROW_STEP
    + 40.0
    - (NPC_VIEW_Y1 - NPC_VIEW_Y0);

// 双修共进收益/限制说明（主卡常显行；与按钮副标同源数字——改一处须同步按钮副标）
const DUAL_CULT_HINT: &str =
    "双修共进：情谊+5 · 道心+1 · 气运+1 · 修为+2 载 · 每载一次（耗灵石随境界涨）";

// 缘谱行已识判定（纯函数供单测；gallery known 同口径：历世账 >0 || 此生同页）
fn bond_row_known(bonds: u8, cur: bool) -> bool {
    bonds > 0 || cur
}

// 缘谱行右缘徽标（纯函数供单测）：未识不泄标；缝页缘定者 > 旧页回声所指（缘最深
// 者，best_bond_idx 反查口径与链10 单源——std max_by_key 并列取末，勿改取首）。
// 旧页回声标须 bonds>0：「此生同页但零历世账」退化态不标（此生同页金标已由行内
// 左列表达）
fn bond_row_badge(bonds: u8, cur: bool, feng_bn: u8, best: usize, i: usize) -> Option<&'static str> {
    if !bond_row_known(bonds, cur) {
        return None;
    }
    if feng_bn != FENG_UNSET && feng_bn as usize == i {
        return Some("缘定三生");
    }
    if best == i && bonds > 0 {
        return Some("旧页回声所指");
    }
    None
}

// v1.17.0 口径弥合三文案单口（纯函数供单测；判定函数零改动——dual_cult_gate 是
// 按钮/执行/单测三处单源，此处只分流「说什么」不分流「门开不开」）：此生未伴
// （life.companion=None）而历世有缘（comp_portrait_idx 历世半边命中 comp_idx=Some）
// 时，同框真立绘与「未结缘」字样互相否认的口径分裂，按指纹法修为「历世有缘 ·
// 此生未续」；历世无缘档文案原样（未结缘玩家画面零变化，红线口径）。
// hist_bond 调用处一律取 comp_idx.is_some()（与立绘判定同源，零新判定）。
fn comp_empty_title(hist_bond: bool) -> &'static str {
    if hist_bond {
        "历世有缘 · 此生未续"
    } else {
        "道侣 · 此生未结缘"
    }
}

fn comp_nocompanion_sub(hist_bond: bool) -> &'static str {
    if hist_bond {
        "历世有缘 · 此生未续"
    } else {
        "此生未结缘"
    }
}

// 页脚引导头句三态：已伴（遗泽口径）/ 历世有缘此生未续（推演缘未承遗泽——引导改
// 「再续」，账上已有缘不再说「即可结缘」）/ 历世无缘（首缘引导原句）
fn comp_footer_head(bonded: bool, hist_bond: bool) -> &'static str {
    if bonded {
        "落幕后可承道侣遗泽：道心+8 气运+8（永久）。"
    } else if hist_bond {
        "历世有缘，此生未续——模拟中遇『同页天成』，选『认下这页』即可再续；落幕后可承道侣遗泽。"
    } else {
        "模拟中遇『同页天成』，选『认下这页』即可结缘；落幕后可承道侣遗泽。"
    }
}

// 道侣专页（char_tab=6）：滚动状态机（人物页/图鉴同一单口）→ 主卡 → 缘谱行 →
// 立绘鉴赏 hit → 遮幅 → 页脚引导。命中（鉴赏/双修）以 PanelAct 传出，执行移交
// exec_panel_act（借用纪律照人物页）。
fn draw_comp_page(g: &mut Game) -> Option<PanelAct> {
    // ---------- 滚动状态机（本页自有 max；snap=false 双栅格失配自由停留）----------
    let tap = scroll_list_tick(g, COMP_SCROLL_MAX, COMP_ROW_STEP, false);
    let (mx, my) = crate::cam::mouse();
    let scroll = g.npc_scroll;

    // ---------- 单源推导（主卡与缘谱共用；comp_portrait_idx 历世兜底口径照人物页）----------
    let l = g.life.as_ref();
    let comp_idx = comp_portrait_idx(l, &g.meta);
    // 红线：全页唯一立绘查询（LazyTex 懒解码；未结缘 → None → 剪影兜底）
    let comp_tex = comp_idx.and_then(|ci| crate::art::art().comp_tex(ci));
    let best = best_bond_idx(&g.meta.comp_bonds);

    // ---------- 主卡（双修钮命中传出）----------
    let mut act = draw_comp_main_card(
        g,
        l,
        comp_idx,
        comp_tex.as_ref(),
        NPC_VIEW_Y0 - scroll,
        mx,
        my,
        tap,
    );

    // ---------- 缘谱十一行（整行剔除；纯文本零立绘零纹理查询）----------
    let cur_comp: Option<usize> = l.and_then(|x| {
        x.companion
            .as_deref()
            .and_then(|cn| COMPANION_NAMES.iter().position(|&n| n == cn))
    });
    for i in 0..COMPANION_NAMES.len() {
        let y = NPC_VIEW_Y0 + COMP_CARD_H + COMP_GAP + i as f32 * COMP_ROW_STEP - scroll;
        if y >= NPC_VIEW_Y1 || y + COMP_ROW_H <= NPC_VIEW_Y0 {
            continue;
        }
        draw_bond_row(g, i, y, cur_comp == Some(i), best);
    }
    let ty = NPC_VIEW_Y0
        + COMP_CARD_H
        + COMP_GAP
        + COMPANION_NAMES.len() as f32 * COMP_ROW_STEP
        - scroll;
    if ty < NPC_VIEW_Y1 && ty + COMP_ROW_H > NPC_VIEW_Y0 {
        draw_bond_total_row(g, ty);
    }

    // ---------- 立绘鉴赏 hit（照人物页盒口径 176..396 × 卡顶+16..底-12 + 纵夹视口带）----------
    if act.is_none()
        && tap
        && comp_tex.is_some()
        && mx >= 176.0
        && mx <= 396.0
        && my >= (NPC_VIEW_Y0 - scroll) + 16.0
        && my <= (NPC_VIEW_Y0 - scroll) + 418.0
        && my >= NPC_VIEW_Y0
        && my <= NPC_VIEW_Y1
    {
        // comp_tex Some ⟹ comp_idx Some（同源推导）；unwrap_or(0) 为纪律 1 防御
        act = Some(PanelAct::Zoom(crate::game::ZoomArt::Comp(
            comp_idx.unwrap_or(0),
        )));
    }

    // ---------- 遮幅（口径同人物页/图鉴：全不透明面板色，盖住越缘内容）----------
    let mask = Color::from_rgba(22, 16, 38, 255);
    draw_rectangle(150.0, 104.0, 980.0, 12.0, mask);
    draw_rectangle(150.0, 616.0, 980.0, 38.0, mask);

    // ---------- 页脚引导（固定视口外一行，照人物页 y630；三态复用现成文案单次 format! 豁免）----------
    // 未结缘 → 人物页空态同句 + 遗泽口径；已结缘 → INHERITS[8] 口径；
    // 历世有缘（comp_bonds 任意 >0）→ 链10 门控提示（灯市旧页回声）。
    // v1.17.0 口径弥合：head 三态抽 comp_footer_head 单口——历世有缘档页脚不再说
    // 「即可结缘」（账上已有缘），改「即可再续」（与主卡标题同口径）
    let bonded = l.is_some_and(|x| x.companion.is_some());
    let head = comp_footer_head(bonded, comp_idx.is_some());
    let tail = if g.meta.comp_bonds.iter().any(|&b| b > 0) {
        "灯市深处，旧页回声或再现。"
    } else {
        ""
    };
    txt(&format!("{}{}", head, tail), 170.0, 630.0, 14, C_DIM);
    act
}

// 主卡：立绘（220×402 + 鉴赏徽标）/ 名号 / 档次 / 情谊条+刻度 / 双修加成 / 双修
// 收益行 / 缝页缘行 / 历世行 / 小传 / 双修共进钮（340×60，整钮可见才可点，照赠礼
// 钮口径）。已结缘返回双修命中（PanelAct::DualCult），其余 None。
fn draw_comp_main_card(
    g: &Game,
    l: Option<&Life>,
    comp_idx: Option<usize>,
    comp_tex: Option<&Texture2D>,
    y: f32,
    mx: f32,
    my: f32,
    tap: bool,
) -> Option<PanelAct> {
    let x = 160.0;
    let cfav = l.map_or(0, |x| x.comp_fav);
    npc_card_chrome(x, y, COMP_CARD_H, Color::from_rgba(42, 26, 52, 210), C_PURPLE);
    // 立绘：已结缘/历世兜底真图（comp_tex 已在页口单点查询），未结缘书卷剪影兜底
    //（or_else 短路——真图命中永不触发）
    let portrait = comp_tex
        .map(Clone::clone)
        .or_else(crate::art::comp_silhouette_tex);
    npc_portrait(portrait.as_ref(), &crate::art::art().icon_pill, x + 16.0, y);
    // 立绘鉴赏入口徽标（真图才显示；剪影不进鉴赏）
    if comp_tex.is_some() {
        zoom_badge(x + 16.0, y, 220.0, 418.0);
    }
    let col = x + 260.0;
    match l.and_then(|x| x.companion.as_deref()) {
        Some(cn) => {
            if npc_row_vis(y + 16.0) {
                txt(cn, col, y + 16.0, 24, C_PURPLE);
            }
            if npc_row_vis(y + 54.0) {
                txt(crate::npc::tier_name(cfav), col, y + 54.0, 16, C_PURPLE);
            }
            if npc_row_vis(y + 82.0) {
                txt(
                    &format!("情谊 {}/{}", cfav, NPC_FAV_MAX),
                    col,
                    y + 82.0, 14,
                    C_TEXT,
                );
            }
            if y + 102.0 >= NPC_VIEW_Y0 && y + 124.0 <= NPC_VIEW_Y1 {
                bar(
                    col,
                    y + 106.0,
                    560.0,
                    14.0,
                    cfav as f32 / NPC_FAV_MAX as f32,
                    C_PURPLE,
                );
                // 档次刻度（照人物页 :733-736 同款：3px / (188,168,224,235)）
                for &t in NPC_FAV_TIERS.iter() {
                    let tx = col + t as f32 / NPC_FAV_MAX as f32 * 560.0;
                    draw_rectangle(tx - 1.5, y + 102.0, 3.0, 22.0, Color::from_rgba(188, 168, 224, 235));
                }
            }
            if npc_row_vis(y + 136.0) {
                txt(
                    &format!(
                        "当前：双修加成 ×{:.2}（结缘起步+{:.0}%，只升不降）",
                        crate::npc::comp_cult_mult(cfav),
                        (crate::npc::comp_cult_mult(COMPANION_BOND_FAV) - 1.0) * 100.0
                    ),
                    col,
                    y + 136.0, 14,
                    C_CYAN,
                );
            }
            // 双修共进收益/限制行（与按钮副标同源数字）
            if npc_row_vis(y + 162.0) {
                txt(DUAL_CULT_HINT, col, y + 162.0, 14, C_GOLD_D);
            }
            // 缝页缘行：feng_bn==ci 显『三生缝页·缘 N』（N=blv 0..=3，ASCII 动态）
            if npc_row_vis(y + 188.0) {
                if let Some(bn) = comp_idx {
                    if g.meta.feng_bn != FENG_UNSET && g.meta.feng_bn as usize == bn {
                        txt(
                            &format!("三生缝页·缘 {}", g.meta.feng_blv.min(3)),
                            col,
                            y + 188.0, 14,
                            C_GOLD,
                        );
                    }
                }
            }
            if npc_row_vis(y + 214.0) {
                txt(
                    &format!("历世结缘 {} 次", g.meta.companion_lives),
                    col,
                    y + 214.0, 14,
                    C_DIM,
                );
            }
            // 识人小传：道侣槽共通段（首段结缘保底，段2/3 随情谊 40/70 解锁，照 :895 口径）
            let cseg = tale_unlocked(cfav).max((g.meta.companion_lives >= 1) as usize);
            if cseg > 0 && npc_row_vis(y + 240.0) {
                wrap_draw(COMP_TALES[cseg - 1], col, y + 240.0, 14, 640.0, 22.0, C_TEXT);
            }
            // 双修共进钮 340×60（底对齐 y+358 照赠礼钮；整钮可见才可点；执行口
            // act_dual_cult 内部再全门控校验——按钮仅 UI 门）
            let (bx, by) = (col, y + COMP_CARD_H - 72.0);
            if by < NPC_VIEW_Y0 || by + 60.0 > NPC_VIEW_Y1 {
                return None;
            }
            let cost = g.dual_cult_cost();
            let gate = Game::dual_cult_gate(l, cost);
            let en = gate == DualGate::Ready;
            let hot = mx >= bx && mx <= bx + 340.0 && my >= by && my <= by + 60.0;
            let base = if !en {
                Color::from_rgba(38, 32, 56, 225)
            } else if hot {
                Color::from_rgba(92, 66, 122, 245)
            } else {
                Color::from_rgba(56, 42, 86, 235)
            };
            draw_rectangle(bx, by, 340.0, 60.0, base);
            draw_rectangle_lines(
                bx,
                by,
                340.0,
                60.0,
                2.0,
                if en {
                    C_GOLD_D
                } else {
                    Color::from_rgba(96, 82, 124, 255)
                },
            );
            txt_c(
                "双修共进",
                bx + 170.0,
                by + 13.0,
                20,
                if en {
                    C_TEXT
                } else {
                    Color::from_rgba(184, 178, 196, 255)
                },
            );
            // 副标：可用态写明代价与收益；置灰态写明差什么（门控优先级 =
            // NoCompanion > ThisYear > Stones，照 gate 单源）
            let sub = match gate {
                DualGate::Ready => format!(
                    "耗灵石 {} · 情谊+5 · 道心+1 气运+1 · 修为+2载",
                    fmt_int(cost)
                ),
                DualGate::ThisYear => "本载已双修 · 推岁一载后方可再修".to_string(),
                DualGate::Stones => format!("灵石不足（需 {}）", fmt_int(cost)),
                // NoCompanion 本臂（Some(companion) 分支）按 gate 单源不可达，纯防御
                // 口径；v1.17.0 文案仍随历世账分流保持单口径（comp_nocompanion_sub
                // 单口；历世账直查 bonds>0——此处 comp_idx 是此生伴下标，不能当 hist 判据）
                DualGate::NoCompanion => comp_nocompanion_sub(
                    g.meta.comp_bonds.iter().any(|&b| b > 0)
                )
                .to_string(),
            };
            txt_c(
                &sub,
                bx + 170.0,
                by + 34.0, 14,
                if en {
                    C_DIM
                } else {
                    Color::from_rgba(178, 172, 192, 255)
                },
            );
            return (tap && hot && en).then_some(PanelAct::DualCult);
        }
        None => {
            // 未结缘空态（照 draw_comp_card 引导框语言；历世有缘者立绘兜底已同框）。
            // v1.17.0 口径弥合：标题随历世账分流——None 臂内 comp_idx=Some 即历世
            // 兜底半边命中（立绘判定单源同帧同值），不再对同框真图写「未结缘」
            if npc_row_vis(y + 16.0) {
                txt(comp_empty_title(comp_idx.is_some()), col, y + 16.0, 24, C_PURPLE);
            }
            let (gx, gy, gw, gh) = (col, y + 52.0, 560.0, 88.0);
            if let Some((vy, vh)) = npc_clip_band(gy, gh) {
                draw_rectangle(gx, vy, gw, vh, Color::from_rgba(24, 17, 40, 190));
                draw_rectangle_lines(gx, vy, gw, vh, 1.5, C_GOLD_D);
            }
            if npc_row_vis(gy + 18.0) {
                txt(
                    "模拟中遇『同页天成』事件，选『认下这页』即可结缘。",
                    gx + 16.0,
                    gy + 18.0, 16,
                    C_TEXT,
                );
            }
            if npc_row_vis(gy + 50.0) {
                if comp_tex.is_some() {
                    txt(
                        "此绘承自历世之缘，此生再遇『同页天成』可再续。",
                        gx + 16.0,
                        gy + 50.0, 16,
                        C_TEXT,
                    );
                } else {
                    txt(
                        "结缘后，她的立绘、情谊与双修加成都会在此页显现。",
                        gx + 16.0,
                        gy + 50.0, 16,
                        C_TEXT,
                    );
                }
            }
            if npc_row_vis(y + 156.0) {
                txt(
                    &format!(
                        "结缘起步情谊 {}（双修 +{:.0}%），道侣赠礼事件再 +{}。",
                        COMPANION_BOND_FAV,
                        (crate::npc::comp_cult_mult(COMPANION_BOND_FAV) - 1.0) * 100.0,
                        COMPANION_GIFT_FAV
                    ),
                    col,
                    y + 156.0, 14,
                    C_CYAN,
                );
            }
            if npc_row_vis(y + 182.0) {
                txt(DUAL_CULT_HINT, col, y + 182.0, 14, C_GOLD_D);
            }
            // 缝页缘行（历世有缘者 feng_bn 已定；此生未结缘也标出来供缘谱对读）
            if npc_row_vis(y + 208.0) && g.meta.feng_bn != FENG_UNSET {
                txt(
                    &format!("三生缝页·缘 {}", g.meta.feng_blv.min(3)),
                    col,
                    y + 208.0, 14,
                    C_GOLD,
                );
            }
            if npc_row_vis(y + 234.0) {
                txt(
                    &format!("历世结缘 {} 次", g.meta.companion_lives),
                    col,
                    y + 234.0, 14,
                    C_DIM,
                );
            }
            // 历世有缘者显首段旧谊（照 :953 口径）
            if g.meta.companion_lives >= 1 && npc_row_vis(y + 260.0) {
                wrap_draw(COMP_TALES[0], col, y + 260.0, 14, 640.0, 22.0, C_TEXT);
            }
            // 剪影下缘小字（历世兜底显真图时不画，照 :959 口径）
            if comp_tex.is_none() && npc_row_vis(y + 390.0) {
                draw_rectangle(
                    x + 16.0,
                    y + 386.0,
                    220.0,
                    22.0,
                    Color::from_rgba(14, 10, 26, 210),
                );
                txt_c(
                    "结缘之后，立绘自会在此显现",
                    x + 126.0,
                    y + 390.0, 14,
                    C_DIM,
                );
            }
        }
    }
    None
}

// 缘谱行（i=0..=9）：已识紫边 + 名 16px；未识灰边 + ？？？；行内左列二行态，
// 右缘徽标（缘定三生 紫 / 旧页回声所指 青）。纯文本零立绘（红线：严禁纹理查询）。
fn draw_bond_row(g: &Game, i: usize, y: f32, cur: bool, best: usize) {
    let x = 160.0;
    let bonds = g.meta.comp_bonds[i]; // i 循环界内（0..COMPANION_NAMES.len()），恒界内
    let known = bond_row_known(bonds, cur);
    let (bg, border) = if known {
        (Color::from_rgba(42, 26, 52, 200), C_PURPLE)
    } else {
        (
            Color::from_rgba(26, 22, 42, 190),
            Color::from_rgba(96, 82, 124, 255),
        )
    };
    npc_card_chrome(x, y, COMP_ROW_H, bg, border);
    if npc_row_vis(y + 6.0) {
        if known {
            txt(COMPANION_NAMES[i], x + 16.0, y + 6.0, 16, C_PURPLE);
        } else {
            txt("？？？", x + 16.0, y + 6.0, 16, C_DIM);
        }
    }
    if npc_row_vis(y + 24.0) {
        if cur {
            txt("此生同页", x + 16.0, y + 24.0, 14, C_GOLD);
        } else if known {
            txt(&format!("曾同页 {} 世", bonds), x + 16.0, y + 24.0, 14, C_TEXT);
        } else {
            txt("未识 · 尘封之名", x + 16.0, y + 24.0, 14, C_DIM);
        }
    }
    // 右缘徽标（右对齐 1084 照图鉴计数位；未识不泄标）
    if let Some(bt) = bond_row_badge(bonds, cur, g.meta.feng_bn, best, i) {
        if npc_row_vis(y + 14.0) {
            let bc = if bt == "缘定三生" { C_PURPLE } else { C_CYAN };
            txt_r(bt, 1084.0, y + 14.0, 14, bc);
        }
    }
}

// 缘谱合计行（第 11 行）：历世结缘总次数（companion_lives）× 同页累计（Σbonds，
// u8 逐位封顶 255、10 位合计 ≤2550 无溢出面）。
fn draw_bond_total_row(g: &Game, y: f32) {
    let x = 160.0;
    npc_card_chrome(x, y, COMP_ROW_H, Color::from_rgba(30, 24, 50, 200), C_GOLD_D);
    let total: u32 = g.meta.comp_bonds.iter().map(|&b| b as u32).sum();
    if npc_row_vis(y + 14.0) {
        txt(
            &format!(
                "合计 · 历世结缘 {} 次 · 同页累计 {} 世",
                g.meta.companion_lives, total
            ),
            x + 16.0,
            y + 14.0, 14,
            C_DIM,
        );
    }
}

// v1.17.0 道侣专页首开帧预热（exec_panel_act Tab(6) 单点调用，照图鉴/旧稿先例）：
// 页静态串 + 十伴名补 24 档（主卡名号 24px；14/16 照图鉴档——18 由菜单期
// panel_warm_chars 保底覆盖）+ 档次名 @16 + 小传 @14。动态数值行 ASCII 由菜单期
// panel_warm_chars 保底（先例口径）。
const COMP_PAGE_PREWARM: &[&str] = &[
    // 页签/主卡
    "道侣",
    "道侣 · 此生未结缘",
    "情谊",
    // 缘谱行态
    "？？？",
    "此生同页",
    "缘定三生",
    "旧页回声所指",
    "曾同页",
    "世",
    "未识 · 尘封之名",
    "合计 · 历世结缘",
    "次 · 同页累计",
    // 缝页/历世
    "三生缝页·缘",
    "历世结缘",
    // 双修
    "双修共进",
    DUAL_CULT_HINT,
    "耗灵石",
    "· 情谊+5 · 道心+1 气运+1 · 修为+2载",
    "本载已双修 · 推岁一载后方可再修",
    "灵石不足（需",
    "）",
    // 空态引导（复用现成文案）+ 剪影小字
    "模拟中遇『同页天成』事件，选『认下这页』即可结缘。",
    "此绘承自历世之缘，此生再遇『同页天成』可再续。",
    "结缘后，她的立绘、情谊与双修加成都会在此页显现。",
    "结缘之后，立绘自会在此显现",
    // v1.17.0 口径弥合两态（空态标题/双修副标 @24 另灌 + 页脚历世有缘句 @14）
    "历世有缘 · 此生未续",
    "历世有缘，此生未续——模拟中遇『同页天成』，选『认下这页』即可再续；落幕后可承道侣遗泽。",
    // 页脚三态（未结缘引导 / 遗泽口径 / 链10 门控提示）
    "模拟中遇『同页天成』，选『认下这页』即可结缘；落幕后可承道侣遗泽。",
    "落幕后可承道侣遗泽：道心+8 气运+8（永久）。",
    "灯市深处，旧页回声或再现。",
];

// 空态引导框 @16 档（与 COMP_PAGE_PREWARM @14 同串双档——绘制档见 draw_comp_main_card）
const COMP_PAGE_PREWARM_16: &[&str] = &[
    "模拟中遇『同页天成』事件，选『认下这页』即可结缘。",
    "此绘承自历世之缘，此生再遇『同页天成』可再续。",
    "结缘后，她的立绘、情谊与双修加成都会在此页显现。",
];

pub(crate) fn prewarm_comp_page() {
    for s in COMP_PAGE_PREWARM {
        prewarm_text(s, &[14]);
    }
    // 空态引导框两行 @16（引导框文案绘制档）；双修钮主标 @20（button 带 sub 的主标档）
    for s in COMP_PAGE_PREWARM_16 {
        prewarm_text(s, &[16]);
    }
    prewarm_text("双修共进", &[20]);
    // 主卡两条动态 format! 行的静态字集样例串（ASCII 数字与 × 由菜单期保底，
    // 先例口径）；「放大」鉴赏徽标（zoom_badge @14，本页新消费方补挂）
    prewarm_text("当前：双修加成 ×0.00（结缘起步+20%，只升不降）", &[14]);
    prewarm_text("结缘起步情谊 25（双修 +20%），道侣赠礼事件再 +5。", &[14]);
    prewarm_text("放大", &[14]);
    // 页签「道侣」@18（chrome button 无 sub 主标档；菜单期 panel_warm 保底之外再灌一道）
    prewarm_text("道侣", &[18]);
    // 主卡名号 @24 / 空态标题 @24（v1.17.0 两态）/ 鉴赏名单 @16（缘谱行名）
    prewarm_text("道侣 · 此生未结缘", &[24]);
    prewarm_text("历世有缘 · 此生未续", &[24]);
    for n in COMPANION_NAMES.iter() {
        prewarm_text(n, &[16, 24]);
    }
    // 档次名（主卡 @16）
    for n in NPC_TIER_NAMES.iter() {
        prewarm_text(n, &[16]);
    }
    // 小传（@14 wrap 两行）
    for t in COMP_TALES.iter() {
        prewarm_text(t, &[14]);
    }
}

// ---------- 页 0：总览 ----------
// v1.10.2 借用化：本页纯只读渲染（无 &mut 交互），整页可持 life 借用
fn draw_overview(g: &Game, l: &Life) {
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
    // v1.14.0 立绘焕新时刻：相名常显行（由境界推导，零存档键，存档往返天然兼容）
    txt(
        &format!(
            "{} · {}",
            realm_full_name(l),
            PORTRAIT_STAGE_NAMES[portrait_tier(l.realm)]
        ),
        170.0,
        345.0,
        20,
        col_realm(l.realm),
    );
    // v1.10.2 成行择道：道途标签（终身标识；未择显示占位，选定后青色常显）
    txt(
        &format!("道途：{}", path_name(l.path)),
        170.0,
        370.0, 14,
        if l.path == crate::game::PATH_NONE {
            C_DIM
        } else {
            C_CYAN
        },
    );
    txt_c(&format!("『{}』", g.worn_title_of()), 305.0, 370.0, 14, C_GOLD);
    // 破境条（修为）
    let need = qi_need(l);
    let frac = (l.qi / need) as f32;
    txt("修为", 170.0, 392.0, 14, C_DIM);
    bar_glow(220.0, 394.0, 180.0, 14.0, frac, C_CYAN, g.time);
    txt(&format!("{:.0}%", frac * 100.0), 412.0, 392.0, 14, C_TEXT);
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
        452.0, 14,
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
    txt("宗门：", 170.0, 524.0, 14, C_DIM);
    txt(
        if l.sect { "青云宗弟子" } else { "无门无派" },
        230.0,
        524.0, 14,
        C_TEXT,
    );
    txt("贡献：", 170.0, 548.0, 14, C_DIM);
    txt(
        &fmt_int(g.meta.sect_contrib as i64),
        230.0,
        548.0, 14,
        C_GOLD,
    );
    // v1.5.0 洞府经营一览
    txt("洞府：", 170.0, 572.0, 14, C_DIM);
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
    // v1.12.0 留行批注：随身批注（结算屏大事记点走的一行；开卷消费后本行隐去）
    if let Some(note) = &g.meta.note_line {
        let disp: String = if note.chars().count() > 15 {
            format!("{}…", note.chars().take(14).collect::<String>())
        } else {
            note.clone()
        };
        txt("批注：", 170.0, 596.0, 14, C_DIM);
        txt(&disp, 220.0, 596.0, 14, C_GOLD);
    }

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
        txt(&format!("{} {}", *v as i32, root_name(*v)), col1 + 205.0, y, 14, C_TEXT);
    }
    txt("功法", col1, 246.0, 14, C_DIM);
    txt(
        &format!("《{}》", TECHS[l.tech].name),
        col1 + 45.0,
        246.0, 14,
        C_GOLD,
    );
    txt(TECHS[l.tech].desc, col1 + 45.0, 268.0, 14, C_DIM);
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
                104.0 + 6.0 * 24.0, 14,
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
        344.0, 14,
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
        380.0, 14,
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
// v1.10.2 借用化：装备槽/包内状态在函数头一次性值化（栈上小数组，life 借用随即
// 归还），命中卸装/装备以 PanelAct 传出；交互口径与 v1.8.0 E3 版一致
fn draw_relic_page(g: &Game) -> Option<PanelAct> {
    // 命中检测（手动，与天赋卡/继承卡先例一致；button 会重铺底色故不用）
    let (mx, my) = crate::cam::mouse();
    let clicked = is_mouse_button_pressed(MouseButton::Left);
    // 左：3 个装备槽
    let slot_ys = [180.0f32, 254.0, 358.0];
    let Some(l) = g.life.as_ref() else { return None };
    let eq_n = l.equipped.len().min(RELIC_EQUIP_MAX);
    let equipped: [usize; RELIC_EQUIP_MAX] =
        std::array::from_fn(|i| l.equipped.get(i).copied().unwrap_or(usize::MAX));
    txt("装备槽（点击卸下）", 180.0, 152.0, 16, C_GOLD);
    for s in 0..RELIC_EQUIP_MAX {
        let y = slot_ys[s];
        let filled = s < eq_n;
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
            // v1.16.5 专属像素画替代共用剑图标（r 即 RELICS 下标，filled 臂内恒有效）
            draw_texture(&crate::art::art().relic_icons[r], 196.0, y + 18.0, WHITE);
            txt(&format!("【{}】", RELICS[r].name), 228.0, y + 14.0, 16, C_GOLD);
            // v1.8.0 E3：灵纹等级徽标
            let lv = g.life.as_ref().map(|x| x.relic_lv[r.min(7)]).unwrap_or(0);
            if lv > 0 {
                txt(&format!("+{}", lv), 352.0, y + 16.0, 14, C_PURPLE);
            }
            txt(RELICS[r].desc, 228.0, y + 40.0, 14, C_TEXT);
            txt_c("卸下", 510.0, y + 24.0, 14, C_RED);
        } else {
            txt(
                &format!("法宝槽 {}（空）", s + 1),
                228.0,
                y + 24.0, 14,
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
            return Some(PanelAct::Unequip(r)); // 执行移交 exec_panel_act
        }
    }
    txt(
        &format!(
            "已装 {}/{}；未装备的法宝保有 {:.0}% 灵性。",
            eq_n,
            RELIC_EQUIP_MAX,
            RELIC_UNEQUIP_FRAC * 100.0
        ),
        180.0,
        446.0, 14,
        C_DIM,
    );

    // 右：包内法宝 8 行（点击装备）
    txt("包内法宝（点击装备）", 600.0, 152.0, 16, C_GOLD);
    for (k, rl) in RELICS.iter().enumerate() {
        let y = 180.0 + k as f32 * 55.0;
        let owned = g.life.as_ref().is_some_and(|x| x.relics.contains(&k));
        let on = equipped.contains(&k);
        let can = owned && !on && eq_n < RELIC_EQUIP_MAX;
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
        // v1.16.5 包内列表补图标（原纯文本无图）：12×12 @616，名/lv/desc 右移 24px；
        // 点击命中区 600..1120 不动（与图标无冲突）
        draw_texture(&crate::art::art().relic_icons[k], 616.0, y + 6.0, WHITE);
        txt(
            &format!("【{}】", rl.name),
            640.0,
            y + 6.0,
            16,
            if owned { C_GOLD } else { C_DIM },
        );
        // v1.8.0 E3：灵纹等级徽标
        let lv = g.life.as_ref().map_or(0, |x| x.relic_lv[k.min(7)]);
        if lv > 0 {
            txt(&format!("+{}", lv), 758.0, y + 8.0, 14, C_PURPLE);
        }
        txt(
            rl.desc,
            804.0,
            y + 8.0, 14,
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
            return Some(PanelAct::Equip(k)); // 执行移交 exec_panel_act
        }
    }
    None
}

// ---------- 页 2：成就·称号 ----------
// 称号格→值映射（格/earned/入参三处共用，勿改出 off-by-one）：
//   格 0    → 值 -1（自动跟随，恒可选）
//   格 1..=15 → 值 c-1（0..=14 境界称号，earned = best_realm >= c-1）
//   格 16   → 值 15（弑神者，earned = neme_slain >= 3）
//   格 17   → 值 16（v1.16.10 章节称号·雨夜执印者，earned = chap_done bit0 即首章已了）
//   格 18   → 值 17（v1.16.15 全卷称号·执笔人，earned = endings_seen == 0b1111 四结局全览）
fn title_cell(c: usize) -> (&'static str, i32, bool) {
    let g_name = |v: usize| realm_title(v);
    match c {
        0 => ("自动跟随", -1, true),
        16 => ("弑神者", 15, false),
        17 => ("雨夜执印者", 16, false),
        18 => ("执笔人", 17, false),
        c => (g_name(c - 1), (c - 1) as i32, false),
    }
}

// v1.16.15 成就·称号页全量字形预热单口（照 prewarm_book_record 单口语言）：
// 首开帧（Tab(2) 执行帧）一次灌齐 19 格名+页头/页签静态串——格 18『执笔人』为
// 新静态名，未达成档（endings_seen 未满最常见态）也画灰行，漏灌则首开帧冷
// 栅格化触发 2048² 图集整图重传。动态值（境界称号名/页脚提示）源自静态表
// 且经人物页/结算屏常显已热，不入本口
pub fn prewarm_title_page() {
    prewarm_text("成就", &[18]);
    prewarm_text("称号（点击佩戴）", &[18]);
    for c in 0..19usize {
        let (name, _, _) = title_cell(c);
        prewarm_text(name, &[14]);
    }
    for s in ["自动跟随", "佩戴中", "可佩戴", "未获得"] {
        prewarm_text(s, &[14]);
    }
}

// v1.10.2 借用化：本页只读 meta（原 l 参数从未使用），佩戴命中以 PanelAct 传出
fn draw_ach_title_page(g: &Game) -> Option<PanelAct> {
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
            txt_r(&format!("{}/{}", cur, goal), x + 272.0, y, 14, C_DIM); // v1.8.0 A8：11→14 字号下限
        }
    }

    // 右：称号 3 列 × 7 行 19 格（v1.16.15 格 18 执笔人破 18 满格：步进 78→70、
    // 格高 64→60 三处常量同改——行 6 y=570..630 仍在页脚 y636 之上，行间隙 10px）
    txt("称号（点击佩戴）", 760.0, 118.0, 18, C_GOLD);
    let worn = g.meta.worn_title;
    let neme_slain = g.meta.neme_slain;
    let best_realm = g.meta.best_realm;
    let chap_done = g.meta.chap_done;
    let endings_seen = g.meta.endings_seen;
    for c in 0..19usize {
        let (name, val, _) = title_cell(c);
        let earned = match c {
            0 => true,
            16 => neme_slain >= 3,
            17 => chap_done & 1 != 0, // v1.16.10 章节称号：首章「雨夜执印」已了
            18 => endings_seen == 0b1111, // v1.16.15 全卷称号：四结局全览
            _ => best_realm >= (c - 1),
        };
        let col = c % 3;
        let row = c / 3;
        let x = 760.0 + col as f32 * 120.0;
        let y = 150.0 + row as f32 * 70.0;
        let is_worn = worn == val;
        draw_rectangle(
            x,
            y,
            112.0,
            60.0,
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
            60.0,
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
            y + 12.0, 14,
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
            y + 36.0,
            14, // v1.8.0 A8：12→14 字号下限
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
            if mx >= x && mx <= x + 112.0 && my >= y && my <= y + 60.0 {
                return Some(PanelAct::Title(val)); // 执行移交 exec_panel_act
            }
        }
    }
    // 页脚提示（称号按历代最高境界判定）；v1.12.1 讨牒 n/6 进度——看得见 6/6 才知道终战在
    let taodie = g.meta.taodie;
    txt(
        &format!(
            "历代最高 {}境 · 斩杀宿敌 {} · 讨牒 {}/6{} · 「自动跟随」随最强境界实时变化",
            REALMS[best_realm.min(14)].name,
            neme_slain,
            taodie,
            if taodie >= 6 { "（可赴问笔终战）" } else { "" },
        ),
        170.0,
        636.0, 14,
        C_DIM,
    );
    None
}

// ---------------- v1.16.5 人物页道侣立绘判定：历世账兜底（纯函数单测） ----------------

#[cfg(test)]
mod comp_portrait_tests {
    use super::*;

    // 未结缘（bonds 全 0、feng_bn 未定）→ None → 书卷剪影路径（画面零变化）
    #[test]
    fn no_bond_yields_none() {
        let meta = Meta::default();
        assert_eq!(comp_portrait_idx(Some(&Life::default()), &meta), None);
        assert_eq!(comp_portrait_idx(None, &meta), None);
    }

    // 此生 companion 优先（历世账无论先后都让位）
    #[test]
    fn current_companion_wins() {
        let mut meta = Meta::default();
        meta.comp_bonds[5] = 1;
        let mut l = Life::default();
        l.companion = Some(COMPANION_NAMES[2].to_string());
        assert_eq!(comp_portrait_idx(Some(&l), &meta), Some(2));
    }

    // 模拟结缘落幕未带回现实（life.companion=None、comp_bonds[i]>0）→ 历世兜底显真图
    #[test]
    fn historical_bond_fallback() {
        let mut meta = Meta::default();
        meta.comp_bonds[6] = 1;
        assert_eq!(comp_portrait_idx(Some(&Life::default()), &meta), Some(6));
        assert_eq!(comp_portrait_idx(None, &meta), Some(6));
    }

    // 缝页缘定之人（feng_bn）优先于按下标序的首个已识者
    #[test]
    fn feng_bn_takes_priority() {
        let mut meta = Meta::default();
        meta.comp_bonds[3] = 1;
        meta.comp_bonds[8] = 2;
        meta.feng_bn = 8;
        assert_eq!(comp_portrait_idx(Some(&Life::default()), &meta), Some(8));
    }

    // 越界纪律：feng_bn 越界（99 ≥ 10）不 panic，落回按下标序（panic=abort 防御）
    #[test]
    fn feng_bn_out_of_range_falls_through() {
        let mut meta = Meta::default();
        meta.comp_bonds[1] = 1;
        meta.feng_bn = 99;
        assert_eq!(comp_portrait_idx(None, &meta), Some(1));
    }

    // feng_bn 已定但历世账为 0（旧档 bn= 有值、gbd= 全 0）→ 不显真图，剪影兜底
    #[test]
    fn feng_bn_without_bond_stays_none() {
        let mut meta = Meta::default();
        meta.feng_bn = 4;
        assert_eq!(comp_portrait_idx(None, &meta), None);
    }
}

// ---------------- v1.16.10 章节称号：称号格映射（格/值两处共用单口防 off-by-one） ----------------

#[cfg(test)]
mod title_cell_tests {
    use super::*;

    // 格 17 → 值 16（雨夜执印者，earned=chap_done bit0 由 draw_ach_title_page 读格）
    #[test]
    fn cell17_maps_value16_rainy_seal() {
        let (name, val, _) = title_cell(17);
        assert_eq!((name, val), ("雨夜执印者", 16));
    }

    // v1.16.15 格 18 → 值 17（执笔人，earned=endings_seen==0b1111 由
    // draw_ach_title_page 读格）；19 格 3×7 网格（格 18：col=2/row=6，y=570..630
    // 仍在页脚 y636 之上——布局三常量 19/70/60 已同步，勿改回 off-by-one）
    #[test]
    fn cell18_maps_value17_scribe() {
        let (name, val, _) = title_cell(18);
        assert_eq!((name, val), ("执笔人", 17));
    }

    // 全格值域卫生：0→-1；1..=15→0..=14（境界称号）；16→15（弑神者）；
    // 17→16（章节）；18→17（全卷）；名字非空
    #[test]
    fn cells_cover_expected_values() {
        assert_eq!(title_cell(0).1, -1);
        assert_eq!(title_cell(0).0, "自动跟随");
        for c in 1..=15usize {
            assert_eq!(title_cell(c).1, c as i32 - 1, "格 {} 值漂移", c);
            assert!(!title_cell(c).0.is_empty());
        }
        assert_eq!(title_cell(16).1, 15);
        assert_eq!(title_cell(16).0, "弑神者");
        assert_eq!(title_cell(18).1, 17);
        assert_eq!(title_cell(18).0, "执笔人");
    }
}

// ---------- 页 5：旧稿（v1.16.5 近 5 次模拟结算快照；纯只读固定列表）----------
// meta.hist（save.rs h= 键，sim_die 唯一写入点）逐条一行卡：头行（世/稿/境界/享年）
// + 数值行（道韵/灵石/道侣）+ 死因 wrap；最新一条金描边区分新旧（iter().rev() 先例
// draw.rs:1797）。行数恒 ≤5（HIST_MAX），无滚动/会话态/遮幅/整卡剔除；panel_card
// 底板由调用方 draw_char_panel 统一绘制。动态行 format! 口径照勘察（draw_inherit
// 每帧 ≥8 处先例）：可见 5 卡 × 2 行 + 固定页脚 1 次。
const HIST_PREWARM_STRS: &[&str] = &[
    "旧稿",
    "第",
    "世·第",
    "稿",
    "境",
    "享年",
    "载·历",
    "年",
    "道韵+",
    "灵石",
    "侣：",
    "无",
    "旧稿录 · 只记近 5 稿，最新在最上。",
    "尚无旧稿——模拟一世落幕后方有记录。",
];

/// 旧稿页首开帧预热（exec_panel_act Tab(5) 单点调用，照 gallery::prewarm_corpus 先例）：
/// 静态语料 + 境界名 + 十道侣名 + 各稿死因（≤5 条）——防冷字形首帧栅格化触发图集重传
pub(crate) fn prewarm_hist(g: &Game) {
    for s in HIST_PREWARM_STRS {
        prewarm_text(s, &[14, 16]);
    }
    for r in REALMS.iter() {
        prewarm_text(r.name, &[14, 16]);
    }
    for n in COMPANION_NAMES.iter() {
        prewarm_text(n, &[14]);
    }
    for r in &g.meta.hist {
        prewarm_text(&r.cause, &[14]);
        // v1.16.7 剧情摘要 @14 同口径预热（漏挂会复现旧稿页首开 2048² 图集整图
        // 重传——ui.rs 首帧栅格化先例）
        if let Some(st) = &r.story {
            prewarm_text(st, &[14]);
        }
    }
}

fn draw_hist_page(g: &Game) {
    if g.meta.hist.is_empty() {
        txt("尚无旧稿——模拟一世落幕后方有记录。", 180.0, 160.0, 16, C_DIM);
        return;
    }
    // 最新在顶（rev 首条即最新，照继承屏大事记 rev().take(7) 先例）
    for (i, r) in g.meta.hist.iter().rev().enumerate() {
        let y = 126.0 + i as f32 * 98.0;
        let latest = i == 0;
        draw_rectangle(
            160.0,
            y,
            960.0,
            92.0,
            if latest {
                Color::from_rgba(58, 44, 92, 235)
            } else {
                Color::from_rgba(44, 34, 70, 235)
            },
        );
        draw_rectangle_lines(
            160.0,
            y,
            960.0,
            92.0,
            1.5,
            if latest {
                C_GOLD_D
            } else {
                Color::from_rgba(70, 60, 96, 255)
            },
        );
        // 头行：第N世·第M稿 境界 享年/历经（max_realm .min(14) 照 draw.rs:1782 先例）
        txt(
            &format!(
                "第{}世·第{}稿  {}境  享年{}载·历{}年",
                r.life_no,
                r.run_no,
                REALMS[r.max_realm.min(14)].name,
                fmt_num(r.age),
                fmt_num(r.years as f64)
            ),
            176.0,
            y + 10.0, 16,
            C_TEXT,
        );
        // 数值行两段：道韵/灵石（道韵>0 金）+ 道侣（紫）
        txt(
            &format!("道韵+{}  灵石{}", fmt_int(r.dy), fmt_int(r.stones)),
            176.0,
            y + 36.0, 14,
            if r.dy > 0 { C_GOLD } else { C_TEXT },
        );
        let comp_name = if (r.comp as usize) < COMPANION_NAMES.len() {
            COMPANION_NAMES[r.comp as usize]
        } else {
            "无"
        };
        txt(
            &format!("侣：{}", comp_name),
            480.0,
            y + 36.0, 14,
            if r.comp != 255 { C_PURPLE } else { C_DIM },
        );
        // v1.16.7 剧情摘要行：数值行右侧 x620..1120（本为空档）单行 14px 紫；
        // None（旧档缺 hq= 行）完全跳过零位移。≤28 字 ×14px ≈392px 不越卡右缘；
        // prewarm_hist 同循环补挂防首帧栅格化整图重传
        if let Some(st) = &r.story {
            txt(st, 620.0, y + 36.0, 14, C_PURPLE);
        }
        wrap_draw(&r.cause, 176.0, y + 58.0, 14, 920.0, 20.0, C_DIM);
    }
    // 页脚（y630 固定视口外，照 gallery.rs:162-170 每帧 1 次 format! 口径）
    txt(
        &format!(
            "旧稿 {} 条 · 只记近 {} 稿，最新在最上。",
            g.meta.hist.len(),
            crate::game::HIST_MAX
        ),
        170.0,
        630.0, 14,
        C_DIM,
    );
}

// ---------------- v1.17.0 道侣专页：缘谱行态（纯函数单测，照 comp_portrait_tests 先例） ----------------

#[cfg(test)]
mod comp_page_tests {
    use super::*;

    // 已识判定：历世账 >0 或此生同页（gallery known 同口径）；未识两半皆假
    #[test]
    fn known_is_bonds_or_current() {
        assert!(bond_row_known(1, false));
        assert!(bond_row_known(255, false), "saturating 封顶 255 仍已识");
        assert!(bond_row_known(0, true));
        assert!(!bond_row_known(0, false));
    }

    // 未识行绝不泄标（缘定/旧页回声名单均不漏——未识态隐私口径照图鉴 ？？？）
    #[test]
    fn unknown_row_has_no_badge() {
        let meta = Meta::default();
        let best = best_bond_idx(&meta.comp_bonds);
        for i in 0..COMPANION_NAMES.len() {
            assert_eq!(bond_row_badge(0, false, meta.feng_bn, best, i), None);
        }
        // feng_bn 已定但该行未识（旧档 bn= 有值、gbd= 全 0）→ 不标
        assert_eq!(bond_row_badge(0, false, 4, 9, 4), None);
    }

    // 徽标优先级：缝页缘定 > 旧页回声所指；旧页标须 bonds>0（cur 零账退化态不标）
    #[test]
    fn badge_priority_feng_then_best() {
        assert_eq!(bond_row_badge(2, false, 2, 2, 2), Some("缘定三生"), "缝页标胜旧页标");
        assert_eq!(bond_row_badge(4, false, 3, 1, 1), Some("旧页回声所指"));
        assert_eq!(bond_row_badge(0, true, FENG_UNSET, 0, 0), None, "cur 零账退化态不标");
        assert_eq!(bond_row_badge(1, false, FENG_UNSET, 5, 4), None, "非最深者不标");
        // cur 行若为缝页人亦显缘定标（此生同页金标由行内左列表达，两标不互斥）
        assert_eq!(bond_row_badge(3, true, 6, 0, 6), Some("缘定三生"));
    }

    // 链10 best_bn 反查单源：charpanel 缘谱与链10 共用 best_bond_idx（game.rs 单口），
    // 此处钉 std max_by_key 并列取**末**语义不漂移（实测本工具链返回并列最大者的
    // 最后一个——与链10 123 臂既有行为逐位一致，勿「修」成取首）
    #[test]
    fn best_bond_idx_shared_helper() {
        let mut meta = Meta::default();
        meta.comp_bonds[4] = 2;
        meta.comp_bonds[7] = 2;
        meta.comp_bonds[9] = 1;
        assert_eq!(best_bond_idx(&meta.comp_bonds), 7, "并列取末（链10 既有语义）");
    }

    // 预热对账：页脚/主卡/缘谱全部静态串必须入 COMP_PAGE_PREWARM（Tab(6) 开页帧
    // 一次灌齐——漏灌即首开帧冷栅格化 2048² 图集整图重传，照 recall_chrome 对账先例）
    #[test]
    fn page_static_strings_covered_by_prewarm() {
        for s in [
            "道侣",
            "双修共进",
            DUAL_CULT_HINT,
            "此生同页",
            "缘定三生",
            "旧页回声所指",
            "三生缝页·缘",
            // v1.17.0 口径弥合两态（标题/双修副标 + 页脚历世有缘句）
            "历世有缘 · 此生未续",
            comp_footer_head(false, true),
        ] {
            assert!(
                COMP_PAGE_PREWARM.contains(&s),
                "静态串「{}」不在 COMP_PAGE_PREWARM——Tab(6) 口首开冷栅格化",
                s
            );
        }
    }

    // v1.17.0 P0 回归：切页签必须整批归零滚动会话态（人物页 max 1698 翻底
    // scroll=1698 → 点「道侣」页签，专页 max 554 下主卡/缘谱 y 全负整卡剔除→
    // 整页视口空白的确定性显示 bug；单口 npc_scroll_reset 与 ×/Esc 臂同批）。
    // 取 Tab(1)（法宝页无预热分支——Tab(6) 首开帧灌字形需 GPU 图集，测试环境
    // 不可达；归零在臂内无条件，任意页签同路径覆盖）
    #[test]
    fn tab_switch_resets_scroll_session() {
        let mut g = crate::game::revenge_tests::game_with(Life::default(), None);
        g.npc_scroll = 1698.0;
        g.npc_vel = 7.5;
        g.npc_anim = Some(442.0);
        g.npc_drag = Some((600.0, 300.0, 1698.0));
        g.ev_recall = Some(3);
        exec_panel_act(&mut g, crate::charpanel::PanelAct::Tab(1));
        assert_eq!(g.char_tab, 1);
        assert_eq!(g.npc_scroll, 0.0, "切页签滚动偏移必须归零");
        assert_eq!(g.npc_vel, 0.0, "惯性必须同批清零（跨页泄漏）");
        assert!(g.npc_anim.is_none(), "跳转 lerp 必须同批清");
        assert!(g.npc_drag.is_none(), "拖动锚点必须同批清");
        assert!(g.ev_recall.is_none(), "v1.16.11 覆层清态并入本口后语义不变");
    }

    // v1.17.0 口径弥合文案分流：历世有缘档不再写「未结缘」（同框真图互认），历世
    // 无缘档原样（未结缘玩家画面零变化）；双修副标/页脚头句同口径
    #[test]
    fn empty_state_copy_split_follows_bond_account() {
        // 标题两态
        assert_eq!(comp_empty_title(false), "道侣 · 此生未结缘");
        assert_eq!(comp_empty_title(true), "历世有缘 · 此生未续");
        // 双修 NoCompanion 副标两态
        assert_eq!(comp_nocompanion_sub(false), "此生未结缘");
        assert_eq!(comp_nocompanion_sub(true), "历世有缘 · 此生未续");
        // 页脚头句三态（已伴遗泽口径 / 历世有缘「再续」引导 / 历世无缘首缘引导）
        assert!(comp_footer_head(true, true).contains("落幕后可承道侣遗泽"));
        assert!(
            comp_footer_head(false, true).contains("即可再续"),
            "历世有缘档不再说「即可结缘」"
        );
        assert!(comp_footer_head(false, false).contains("即可结缘"));
    }

    // 空态兜底不误触发：文案分流判据与立绘判定同源（comp_idx.is_some）——
    // 全 0 账（含旧档 bn= 有值而 gbd= 全 0）必须落「未结缘」态，绝不误写「历世有缘」
    #[test]
    fn empty_title_tracks_portrait_fallback() {
        let mut meta = Meta::default();
        let hist = comp_portrait_idx(None, &meta).is_some();
        assert!(!hist);
        assert_eq!(comp_empty_title(hist), "道侣 · 此生未结缘");
        // 旧档 bn= 有值但 gbd= 全 0：comp_portrait_idx 恒 None（既有判定），文案不误触
        meta.feng_bn = 4;
        assert_eq!(comp_portrait_idx(None, &meta), None);
        assert_eq!(comp_empty_title(comp_portrait_idx(None, &meta).is_some()), "道侣 · 此生未结缘");
        // 历世账 >0 → 兜底命中 → 文案转「历世有缘」（与立绘同帧同值，口径互认）
        meta.comp_bonds[2] = 1;
        assert_eq!(comp_portrait_idx(None, &meta), Some(2));
        assert_eq!(comp_empty_title(true), "历世有缘 · 此生未续");
    }

    // v1.17.0 名字→立绘下标全表映射：伴名无重复（s.companion 字串 position 反查
    // 唯一口径）；十名逐一对应 comp_0..9 且 web/portraits/（wasm 外置包）与
    // assets/portraits_wasm/（桌面内嵌 EMBED 表）同名 png 双通道齐备——缺一字即红
    #[test]
    fn companion_names_map_to_portrait_files() {
        let root = env!("CARGO_MANIFEST_DIR");
        let mut seen = std::collections::HashSet::new();
        assert_eq!(COMPANION_NAMES.len(), 10, "同页人十名定稿");
        for (i, n) in COMPANION_NAMES.iter().enumerate() {
            assert!(seen.insert(*n), "伴名重复：{}", n);
            assert_eq!(
                COMPANION_NAMES.iter().position(|&x| x == *n),
                Some(i),
                "名字→下标 position 反查必须恒等（comp_portrait_idx 唯一映射口径）"
            );
            let web = format!("{}/web/portraits/comp_{}.png", root, i);
            let emb = format!("{}/assets/portraits_wasm/comp_{}.png", root, i);
            assert!(std::path::Path::new(&web).is_file(), "缺外置立绘 {}", web);
            assert!(std::path::Path::new(&emb).is_file(), "缺内嵌立绘 {}", emb);
        }
    }
}
