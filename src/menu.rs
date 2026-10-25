// v1.6.0 B 嵌套子选项界面：弹层（修炼组 / 山河图前往确认）
//
// 设计要点：
// * 单层无栈：Game.submenu 持有当前弹层，Esc / 取消 / 返回三路关闭；
// * 硬伤 1 修复——open_sub 统一置位入口同时打「开帧门闩」（sub_fresh）：
//   button() 的命中判定 is_mouse_button_pressed 同帧恒真，节点点击处理与
//   draw_sub 在同一帧先后执行，若无门闩，点节点上半部会同帧触发行 1 前往。
//   置位帧（sub_fresh=true）弹层只显示不响应；下一帧主循环开头清零后正常。
// * 行按钮判定一律 `button(...) && !g.sub_fresh`。
use crate::data::*;
use crate::draw::{button, panel_card};
use crate::game::*;
use crate::ui::*;
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum Sub {
    Cult,         // 修炼组：闭关苦修 / 冲击破境 / 返回
    MapGo(usize), // 山河图：前往(赶路) / 查看详情 / 取消
    Atlas,        // v1.8.0 C3：区域图鉴（五域机制一览，纯展示）
}

// 统一置位入口：置位同时打「开帧门闩」——同帧后执行的 draw_sub 屏蔽行按钮点击
pub fn open_sub(g: &mut Game, s: Sub) {
    g.submenu = Some(s);
    g.sub_fresh = true;
    g.play(crate::sounds::Which::Page);
}

// 三路关闭（Esc/取消/返回）共用
pub fn close_sub(g: &mut Game) {
    g.submenu = None;
}

// 数字键 1..=n；submenu 为 None 时 no-op（Sim 场景 1/2/3 是调速，该场景 submenu 恒 None）
pub fn sub_hotkey(g: &mut Game, n: usize) {
    // v1.8.0 B4：好感事件卡独占期，弹层热键失效（防 Cult 弹层与 rmodal 并存穿透）
    if g.rmodal.is_some() {
        return;
    }
    let Some(sub) = g.submenu else {
        return;
    };
    match sub {
        Sub::Cult => match n {
            1 => {
                g.act_cultivate(false); // 手点：全量日志+音效
                close_sub(g);
            }
            2 => {
                if g.can_break() {
                    g.act_break();
                    close_sub(g);
                }
            }
            _ => {}
        },
        Sub::MapGo(i) => match n {
            1 => {
                crate::world::travel(g, i);
                close_sub(g);
            }
            2 | 3 => close_sub(g),
            _ => {}
        },
        Sub::Atlas => {} // v1.8.0 C3：图鉴纯展示无热键（B4 守卫在前，正常不到达）
    }
}

// 弹层绘制（在场景绘制尾部调用：draw_reality / draw_world）
// 规格：panel_card(380,160,520,300)；标题 22px 金 居中 y=186；分隔线 y=206；
// 行 button(410, 220+i*78, 460, 66, label, Some(说明·消耗/收益), enabled)；
// Cult 末尾追加 返回 button(490,380,300,42)；MapGo 第三行即「取消」。
pub fn draw_sub(g: &mut Game) {
    let Some(sub) = g.submenu else {
        return;
    };
    // v1.8.0 C3：区域图鉴自成分支（纯展示，不走通用行模板）
    // 注：前置 if-let-return 不豁免其后 match 的穷尽性检查（rustc E0004 实证），
    // 故 rows match 仍须补 `Sub::Atlas => unreachable!` 臂——运行时先命中上方分流，不可达。
    if let Sub::Atlas = sub {
        draw_atlas(g);
        return;
    }
    // 行数据先行构建（&g 只读借用在此结束），动作编码：
    // 0 闭关苦修 1 冲击破境 2 前往赶路 3 查看详情(仅关) 4 取消/返回(仅关)
    let (title, rows, has_back) = match sub {
        Sub::Cult => {
            let gain = fmt_num(g.cult_gain_per_year());
            let blabel = g.break_label();
            let can = g.can_break();
            (
                "修 炼".to_string(),
                vec![
                    (
                        "闭关苦修".to_string(),
                        format!("一载苦修 · 修为+{}", gain),
                        true,
                        0u8,
                    ),
                    (blabel, "现实突破 · 不死".to_string(), can, 1),
                ],
                true,
            )
        }
        Sub::MapGo(i) => {
            let name = LOCATIONS[i.min(LOCATIONS.len() - 1)].name;
            (
                format!("前 往 · {}", name),
                vec![
                    ("前往·赶路".to_string(), "寿元 -1 载".to_string(), true, 2),
                    (
                        "查看详情".to_string(),
                        "详情已示于下方卡片".to_string(),
                        true,
                        3,
                    ),
                    ("取 消".to_string(), "留在此地".to_string(), true, 4),
                ],
                false,
            )
        }
        // v1.8.0 C3：已于函数头分流，编译期穷尽性占位（运行时不可达）
        Sub::Atlas => unreachable!("draw_atlas 已于函数头分流"),
    };
    let mut act: Option<u8> = None;
    {
        let responsive = !g.sub_fresh; // 开帧门闩：置位帧只显示不响应
        panel_card(380.0, 160.0, 520.0, 300.0);
        txt_c(&title, 640.0, 186.0, 22, C_GOLD);
        draw_rectangle(410.0, 206.0, 460.0, 1.5, Color::from_rgba(96, 76, 130, 160));
        for (i, (label, sub, enabled, code)) in rows.iter().enumerate() {
            if button(410.0, 220.0 + i as f32 * 78.0, 460.0, 66.0, label, Some(sub), *enabled)
                && responsive
            {
                act = Some(*code);
            }
        }
        if has_back && button(490.0, 380.0, 300.0, 42.0, "返 回", None, true) && responsive {
            act = Some(4);
        }
    }
    match act {
        Some(0) => {
            g.act_cultivate(false); // 手点：全量日志+音效
            close_sub(g);
        }
        Some(1) => {
            g.act_break();
            close_sub(g);
        }
        Some(2) => {
            if let Sub::MapGo(i) = sub {
                crate::world::travel(g, i);
            }
            close_sub(g);
        }
        Some(_) => close_sub(g),
        None => {}
    }
}

// v1.8.0 C3：区域图鉴弹层（纯展示）——五域主题、专属机制、到访状态与初至奖励。
// panel_card(340,100,600,520)：标题/已解锁 y=130｜分隔线 y=160｜5 行 y=170+i*78（行高 70）｜
// 返回钮 (490,566,300,42)——唯一需开帧门闩的钮。
pub fn draw_atlas(g: &mut Game) {
    let responsive = !g.sub_fresh;
    panel_card(340.0, 100.0, 600.0, 520.0);
    let unlocked = g.meta.region_visited.iter().filter(|&&v| v).count();
    txt_c("区 域 图 鉴", 640.0, 130.0, 24, C_GOLD);
    txt_r(
        &format!("已解锁 {}/5", unlocked),
        920.0,
        132.0,
        15,
        C_CYAN,
    );
    draw_rectangle(370.0, 160.0, 540.0, 1.5, Color::from_rgba(96, 76, 130, 160));
    for i in 0..5usize {
        let y = 170.0 + i as f32 * 78.0;
        if i % 2 == 0 {
            draw_rectangle(370.0, y, 540.0, 70.0, Color::from_rgba(255, 255, 255, 6));
        }
        let visited = g.meta.region_visited[i]; // i 本身 0..5，直接索引安全
        txt(MAP_THEMES[i].name, 386.0, y + 10.0, 17, C_GOLD);
        txt_r(
            if visited { "已至" } else { "未至" },
            894.0,
            y + 12.0,
            14,
            if visited { C_GREEN } else { C_DIM },
        );
        txt(REGION_MECH[i].desc, 386.0, y + 34.0, 14, C_TEXT);
        let (dy, st) = REGION_VISIT_REWARD[i % REGION_VISIT_REWARD.len()];
        txt(
            &format!("初至：道韵+{} · 灵石+{}", fmt_int(dy), fmt_int(st)),
            386.0,
            y + 52.0,
            14,
            C_CYAN,
        );
    }
    if button(490.0, 566.0, 300.0, 42.0, "返 回", None, true) && responsive {
        close_sub(g);
    }
}
