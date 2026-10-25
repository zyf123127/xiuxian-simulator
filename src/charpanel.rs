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
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, 0.6));
    panel_card(140.0, 60.0, 1000.0, 600.0);
    // v1.10.2：人物页（页签 3）改为「先内容 → 遮幅 → 后 chrome」——人物页滚动列表的
    // 上下遮幅带会盖住页签行（74..108），chrome 须最后重铺才不被遮；其余页签无溢出，
    // 维持先 chrome 后内容的旧序（立即模式命中不依赖绘制顺序，行为不变）。
    // v1.10.2 卡顿修复（次要项③）：去每帧整 Life 深克隆（含 relics/talents/equipped
    // 三个 Vec 堆分配）——各页改 &Game 只读绘制 + 收集 PanelAct，借用归还后由
    // exec_panel_act 统一执行 &mut 交互（先例 v1.10.0 draw_sim / 本版 draw_reality）。
    // ×/页签点击帧的金条高亮切换晚一帧生效（16ms，不可见），其余行为逐项等价。
    let npc_tab = g.char_tab == 3;
    let mut act = draw_panel_chrome_act(g);
    if act.is_none() {
        match g.char_tab {
            1 => act = draw_relic_page(g),
            2 => act = draw_ach_title_page(g),
            3 => act = draw_npc_page(g), // v1.6.0 D：好感度页（v1.10.2 单列滚动列表）
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
    }
}

// 标题 / × / 页签（v1.10.2 自 draw_char_panel 抽出；v1.10.2 借用化：只读绘制 +
// 收集命中，执行移交 exec_panel_act）
fn draw_panel_chrome_act(g: &Game) -> Option<PanelAct> {
    txt("角色", 170.0, 92.0, 24, C_GOLD);
    if button(1064.0, 74.0, 44.0, 36.0, "×", None, true) {
        return Some(PanelAct::Close);
    }
    // ---------- 页签（v1.6.0 D：追加第 4 页「人物」） ----------
    let tabs = ["总览", "法宝", "成就·称号", "人物"];
    for (i, t) in tabs.iter().enumerate() {
        let x = 300.0 + i as f32 * 150.0;
        if button(x, 74.0, 140.0, 34.0, t, None, true) {
            return Some(PanelAct::Tab(i as u8));
        }
        if g.char_tab == i as u8 {
            draw_rectangle(x, 108.0, 140.0, 3.0, C_GOLD);
        }
    }
    None
}

// ---------- 页 3：人物（v1.6.0 D 好感度；v1.10.2 单列纵向滚动列表改版）----------
// 5 位人物（0..=3 四 NPC / 4 道侣）同一纵列滚动，不再藏进翻页：卡 960×330、间距 12、
// 步距 342，视口 x160..1120 / y116..616（高 500），内容高 5×330+4×12=1698，滚动偏移
// 钳 [0, 1198]；静止对齐（scroll=i×342，末卡钳 1198）一屏 = 1 整卡 + 下一卡顶部
// 170px（人名/档次/好感条露出）。交互：拖动 1:1 跟手 + 松手惯性（vel×0.9/帧，
// |vel|<0.5 停）+ 桌面滚轮 ±48px/格（工程首用 mouse_wheel）+ 右缘页点/底部头像条
// lerp 0.2/帧跳转；静止向最近卡顶吸附。◀▶ 翻页移除，由滚轮+页点+头像条替代。
// 溢出遮挡双保险（macroquad scissor 不可用：get_context 私有 vendor lib.rs:497，且
// 引擎 flush 按自身 clip 状态重设 scissor vendor quad_gl.rs:768-771）：
//   ① 遮幅带——上 104..116（卡间 12px 间隙过顶时上一卡底边最低到 y=104）、下
//     616..654（卡顶锚定 116 且 scroll 钳 ≥0，卡永不越下缘；收到 654 止，勿盖
//     panel 内框线 y655，draw.rs panel_card 内框）；
//   ② 逐元素视口裁剪——卡体/边线/立绘（纹理源等比裁剪）/按钮按视口求交，文本行
//     越界整行跳过。必要：max scroll 1198 时上一卡顶可到 y=-56，遮幅带盖不住越出
//     面板的溢出（y<60 会画到面板外暗场上）。
// 性能：整卡剔除（card_y≥616 或 card_y+330≤116 连立绘查询一并跳过，任意帧 ≤2 卡在
// 画）；LazyTex 仅可见卡与头像条取纹理，首开最多解码 npc_elder/npc_merchant 两张
// （无立绘卡零解码，LazyTex 缓存 None 不重查）。立绘 contain 收进 220×310、底边
// 对齐（实测素材宽高比 0.456..0.628 → 绘宽 141..195 恒 ≥140 不变形）；无立绘回退
// 12×12 像素图标 ×12=144 同达标。好感度/档次刻度/增益/赠礼交互与数据全部保留，
// gift_npc 单口不动（npc.rs:269）；滚动会话态不进存档（×/Esc 归零见 npc_scroll_reset）。
const NPC_VIEW_Y0: f32 = 116.0; // 视口顶（页签下划线下缘）
const NPC_VIEW_Y1: f32 = 616.0; // 视口底
const NPC_CARD_H: f32 = 330.0; // 卡高
const NPC_STEP: f32 = 342.0; // 卡高 + 间距 12
const NPC_SCROLL_MAX: f32 = 1198.0; // 内容 1698 − 视口 500

// 视口内竖向裁剪：返回可见段 (y, h)，None = 完全在视口外
fn npc_clip_band(y: f32, h: f32) -> Option<(f32, f32)> {
    let t = y.max(NPC_VIEW_Y0);
    let b = (y + h).min(NPC_VIEW_Y1);
    if b > t { Some((t, b - t)) } else { None }
}

// 视口内文本行：越界整行跳过（字形上探 ≤2px 由上遮幅兜底、下探由下遮幅兜底）
fn npc_row_vis(y: f32) -> bool {
    y >= NPC_VIEW_Y0 && y < NPC_VIEW_Y1
}

// 卡体 + 边线（全部经视口裁剪；上下线仅整线可见时画，避免跨界 1-2px 残影）
fn npc_card_chrome(x: f32, y: f32, bg: Color, border: Color) {
    if let Some((ty, th)) = npc_clip_band(y, NPC_CARD_H) {
        draw_rectangle(x, ty, 960.0, th, bg);
        draw_rectangle(x, ty, 2.0, th, border);
        draw_rectangle(x + 958.0, ty, 2.0, th, border);
        if y >= NPC_VIEW_Y0 {
            draw_rectangle(x, y, 960.0, 2.0, border);
        }
        if y + NPC_CARD_H <= NPC_VIEW_Y1 {
            draw_rectangle(x, y + NPC_CARD_H - 2.0, 960.0, 2.0, border);
        }
    }
}

// 立绘：contain 收进 220×310、底边对齐（公式沿用旧卡闭包）；并按视口上下缘对纹理
// 源等比裁剪（source 为纹理像素，vendor texture.rs:537-545 按纹理尺寸归一）；
// 无立绘回退 12×12 图标 ×12=144（绘宽 144 ≥140，同为像素风不糊）
fn npc_portrait(tex: Option<&Texture2D>, icon: &Texture2D, x: f32, card_y: f32) {
    let (dw, dh) = match tex {
        Some(t) => {
            let ar = t.width() / t.height().max(1.0);
            let w = (310.0 * ar).min(220.0);
            (w, w / ar)
        }
        None => (144.0, 144.0),
    };
    let top = card_y + 318.0 - dh; // 底边对齐（卡底内缩 12）
    let Some((vy, vh)) = npc_clip_band(top, dh) else {
        return;
    };
    let (tw, th) = match tex {
        Some(t) => (t.width(), t.height().max(1.0)),
        None => (12.0, 12.0),
    };
    draw_texture_ex(
        tex.unwrap_or(icon),
        x + (220.0 - dw) * 0.5,
        vy,
        WHITE,
        DrawTextureParams {
            source: Some(Rect::new(0.0, (vy - top) / dh * th, tw, vh / dh * th)),
            dest_size: Some(vec2(dw, vh)),
            ..Default::default()
        },
    );
}

// v1.10.2 借用化：签名去 l——滚动状态机/页点/头像条为 &mut 直写；仅卡片绘制段
// 借 life 不可变引用（块尾归还），命中赠礼以 PanelAct::Gift 跨借用传出
fn draw_npc_page(g: &mut Game) -> Option<PanelAct> {
    // ---------- 滚动状态机（会话态，不进存档）----------
    let (mx, my) = crate::cam::mouse();
    let pressed = is_mouse_button_pressed(MouseButton::Left);
    let released = is_mouse_button_released(MouseButton::Left);
    // 桌面滚轮：±48px/格；打断跳转动画与惯性
    let (_, wy) = mouse_wheel();
    if wy != 0.0 {
        g.npc_anim = None;
        g.npc_vel = 0.0;
        g.npc_scroll = (g.npc_scroll - wy * 48.0).clamp(0.0, NPC_SCROLL_MAX);
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
                let ns = (s0 - (my - py)).clamp(0.0, NPC_SCROLL_MAX);
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
            if (g.npc_vel > 0.0 && raw >= NPC_SCROLL_MAX) || (g.npc_vel < 0.0 && raw <= 0.0) {
                g.npc_scroll = raw.clamp(0.0, NPC_SCROLL_MAX);
                g.npc_vel = 0.0;
            } else {
                g.npc_scroll = raw;
                g.npc_vel *= 0.9;
            }
        } else {
            g.npc_vel = 0.0;
            // 静止对齐：向最近卡顶吸附（lerp 0.2/帧；末卡钳 1198）——任意静止态
            // =整卡+邻卡露头，页点/头像条的「当前卡」高亮恒有唯一锚
            let target =
                ((g.npc_scroll / NPC_STEP).round() * NPC_STEP).clamp(0.0, NPC_SCROLL_MAX);
            let d = target - g.npc_scroll;
            if d.abs() > 0.5 {
                g.npc_scroll += d * 0.2;
            } else {
                g.npc_scroll = target;
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

    // ---------- 卡片（整卡剔除：任意帧 ≤2 卡在画，剔除含立绘查询）----------
    // v1.7.1/1.7.2：有道侣且有 AI 立绘（comp_0..9 按结缘顺序）则替代丹药图标。
    // 卡片绘制段的 life 不可变借用限定在此块内（块尾归还），命中赠礼以 Gift 传出
    let (comp_tex, gifted) = {
        let l = g.life.as_ref();
        let comp_idx = l.and_then(|x| {
            x.companion
                .as_deref()
                .and_then(|cn| COMPANION_NAMES.iter().position(|&n| n == cn))
        });
        let comp_tex = comp_idx.and_then(|ci| crate::art::art().comp_tex(ci));
        let mut gifted = None;
        for i in 0..5usize {
            let card_y = NPC_VIEW_Y0 + i as f32 * NPC_STEP - g.npc_scroll;
            if card_y >= NPC_VIEW_Y1 || card_y + NPC_CARD_H <= NPC_VIEW_Y0 {
                continue;
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
        (comp_tex, gifted)
    };

    // ---------- 遮幅（口径见本页头注释；C_PANEL α235 半透不蔽物，用其全不透明版）----------
    let mask = Color::from_rgba(22, 16, 38, 255);
    draw_rectangle(150.0, 104.0, 980.0, 12.0, mask);
    draw_rectangle(150.0, 616.0, 980.0, 38.0, mask);
    if let Some(i) = gifted {
        // 赠礼帧到此为止（先例：赠礼后 return；chrome 由调用方补画）；
        // 执行移交 exec_panel_act（滚动偏移在 Game 会话态，位置不丢）
        return Some(PanelAct::Gift(i));
    }

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
    // 头像取立绘顶部约 35% 高度方形头裁剪（1:1 不变形）；无立绘回退像素图标 24×24
    // （12px ×2 整数倍不糊）；当前卡金框
    for i in 0..5usize {
        let (ax, ay) = (920.0 + i as f32 * 42.0, 620.0);
        let (tex, icon) = if i < 4 {
            (crate::art::art().npc_tex(i), &crate::art::art().icon_yun)
        } else {
            (comp_tex.clone(), &crate::art::art().icon_pill)
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
    txt(
        "好感随交往与赠礼增长，跨世持久（随轮回烙印存档）；赠礼价随交情水涨船高。",
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
    npc_card_chrome(x, y, Color::from_rgba(30, 24, 50, 210), C_GOLD_D);
    npc_portrait(
        crate::art::art().npc_tex(i).as_ref(),
        &crate::art::art().icon_yun,
        x + 16.0,
        y,
    );
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
    // 赠礼钮 230×60 底对齐（仅整钮可见时绘出与可点）；gift_npc 内部再校验灵石（单口）
    let (bx, by) = (col, y + 258.0);
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
    npc_card_chrome(x, y, Color::from_rgba(42, 26, 52, 210), C_PURPLE);
    npc_portrait(comp_tex, &crate::art::art().icon_pill, x + 16.0, y);
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
        }
        None => {
            if npc_row_vis(y + 16.0) {
                txt("道侣 · 此生未结缘", col, y + 16.0, 24, C_DIM);
            }
            if npc_row_vis(y + 52.0) {
                txt(
                    "模拟中「同页天成」事件可选执子之手；遗泽亦可携道侣魂归现实。",
                    col,
                    y + 52.0, 14,
                    C_DIM,
                );
            }
            if npc_row_vis(y + 80.0) {
                txt(
                    &format!(
                        "结缘起步情谊 {}（双修 +{:.0}%），道侣赠礼事件再 +{}。",
                        COMPANION_BOND_FAV,
                        (crate::npc::comp_cult_mult(COMPANION_BOND_FAV) - 1.0) * 100.0,
                        COMPANION_GIFT_FAV
                    ),
                    col,
                    y + 80.0, 14,
                    C_CYAN,
                );
            }
        }
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
    txt(&realm_full_name(l), 170.0, 345.0, 20, col_realm(l.realm));
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
            draw_texture(&crate::art::art().icon_sword, 196.0, y + 18.0, WHITE);
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
        txt(
            &format!("【{}】", rl.name),
            616.0,
            y + 6.0,
            16,
            if owned { C_GOLD } else { C_DIM },
        );
        // v1.8.0 E3：灵纹等级徽标
        let lv = g.life.as_ref().map_or(0, |x| x.relic_lv[k.min(7)]);
        if lv > 0 {
            txt(&format!("+{}", lv), 734.0, y + 8.0, 14, C_PURPLE);
        }
        txt(
            rl.desc,
            780.0,
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
fn title_cell(c: usize) -> (&'static str, i32, bool) {
    let g_name = |v: usize| realm_title(v);
    match c {
        0 => ("自动跟随", -1, true),
        16 => ("弑神者", 15, false),
        c => (g_name(c - 1), (c - 1) as i32, false),
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
            y + 14.0, 14,
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
            if mx >= x && mx <= x + 112.0 && my >= y && my <= y + 64.0 {
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
