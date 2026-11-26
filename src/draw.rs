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

// 演出包⑰-2 底栏九钮入画 stagger 相位纯函数（照 weather_veil_alpha 包络先例
// draw.rs:765 款——draw_reality 底栏与钉测单测共用，e=进行度 T−bar_anim）：
// ((e−k×0.03)/0.03).clamp(0,1)，9 槽错峰步进 30ms。稳态 e=T=0.27 → 相位=9−k≥1.0
// clamp 恰 1.0（k=8 时恰 (0.27−0.24)/0.03=1.0 精确命中、k<8 走 clamp 收敛——
// fade(c,1.0) 逐比特恒等，⑦ clamp 收敛红线同款）；开帧 e=0 全 0.0；窗内按槽位
// 错峰显影。
#[inline]
pub(crate) fn bar_stagger_a(e: f32, k: usize) -> f32 {
    ((e - k as f32 * 0.03) / 0.03).clamp(0.0, 1.0)
}

// v1.16.46 演出包⑲ 破境金辉氛围层三角包络（照 weather_veil_alpha draw.rs:788
// 单口先例）：k=modal_anim/0.18 复用①相位零新状态，k 自 1（开帧）→0（稳态），
// 前半程渐显后半程渐隐 env∈[0,1] 峰值恰 0.5；稳态 k==0 → 0.0（消费口 a>0 才画
// 先例）不落像素逐比特恒等。⑳ 结缘金尘错峰共用本包络（bond_mote_a 包装）——
// 单源勿双写。
#[inline]
pub(crate) fn brk_veil_a(k: f32) -> f32 {
    if k <= 0.0 {
        return 0.0;
    }
    let kk = k.clamp(0.0, 1.0);
    if kk > 0.5 {
        (1.0 - kk) / 0.5
    } else {
        kk / 0.5
    }
}

// v1.16.46 演出包⑳ 结缘金尘错峰：单粒相位 = 共用三角包络作用于 (k − idx×0.05)
// ——idx 越大越早入峰（k 自 1 递减过程按粒错峰显隐）；稳态 k==0 → 全粒 0.0；
// 全域有界 [0,1]；上升位移/α 均由本包络派生（消费口乘 art_a/card 系数）。
#[inline]
pub(crate) fn bond_mote_a(k: f32, k_idx: usize) -> f32 {
    brk_veil_a((k - k_idx as f32 * 0.05).clamp(0.0, 1.0))
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
    button_accent(x, y, w, h, label, sub, enabled, false)
}

/// 剧情演出包④：带主选项金描边变体（accent=true 描边 C_GOLD，终战推荐位/道途
/// 首钮用）；其余行为与 button 逐比特一致。同帧新增按压态：hot 且左键按住
///（触屏 hot 即手指落在钮上，等效按压）底色提亮 + 内衬高亮线，松开/移出即还原。
pub fn button_accent(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: &str,
    sub: Option<&str>,
    enabled: bool,
    accent: bool,
) -> bool {
    // v1.16.37 单3 起全部既有调用点经 α=1.0 委托零改动（照 panel_card→
    // panel_card_a 包装先例 draw.rs:163-165）
    button_a_accent(x, y, w, h, label, sub, enabled, accent, 1.0)
}

/// v1.16.37 单3 剧情演出包⑦：带透明度选项钮（事件卡选项逐条入场节奏用）。
/// alpha=1.0 时逐像素等同 button_accent；底/描边/内衬/主副标同步乘 α。命中与
/// 按压判定不乘 α——沿用 draw_modal_card「动画期照常可点，命中=所见」口径
///（卡体入场期命中几何随卡体同步，稳态复位）。30+ 既有 button_accent 调用点
/// 经上方委托零改动；α 乘算不入 wrap 缓存 key（颜色不入 key，ui.rs WrapKey 先例）。
pub fn button_a_accent(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: &str,
    sub: Option<&str>,
    enabled: bool,
    accent: bool,
    alpha: f32,
) -> bool {
    let a = alpha.clamp(0.0, 1.0);
    // v1.5.0：经信箱相机回虚拟坐标（桌面恒等，H5 任意屏比例正确命中）
    let (mx, my) = crate::cam::mouse();
    let hot = mx >= x && mx <= x + w && my >= y && my <= y + h;
    // 按压态用 is_mouse_button_down（持续态）而非 pressed（边沿）——手指还压着
    // 与「点了立即弹开」视觉可辨；仅影响底色/内衬线，命中与点击判定不变
    let press = hot && enabled && is_mouse_button_down(MouseButton::Left);
    // v1.16.45 演出包⑰-1 按压下沉：press 持续态时四个绘制口（填充/描边/主副标/
    // 内衬）整体 y+2——press=false 帧 dy=0 布尔门控恒等（稳态逐像素=旧实现，非
    // clamp 浮点巧合）；press 由 is_mouse_button_down 持续态驱动，动画期=按压
    // 持续期，松开即还原——零新状态零计时零分配零新字形（颜色不变）。命中矩形
    // :94 原值不动（按压期 2px 偏移照⑮「1.2px 偏移不影响点按域语义」先例
    // charpanel.rs:509-511 口径），与既有按压语言（底色提亮+内衬高亮线）组成完整
    // 下沉视觉。helper 外三处下划线（周天 draw.rs 底栏/模拟速度钮/charpanel 页签）
    // 不随位移——按压持续期 ≤0.3s 内 2px 瞬时错位，勘察判可接受留档。
    let dy = if press { 2.0 } else { 0.0 };
    let base = if !enabled {
        Color::from_rgba(38, 32, 56, 225)
    } else if press {
        Color::from_rgba(108, 78, 140, 245)
    } else if hot {
        Color::from_rgba(92, 66, 122, 245)
    } else {
        Color::from_rgba(56, 42, 86, 235)
    };
    draw_rectangle(x, y + dy, w, h, fade(base, a));
    draw_rectangle_lines(
        x,
        y + dy,
        w,
        h,
        2.0,
        fade(
            if !enabled {
                // v1.8.0 A6：禁用描边提亮（66,56,88→96,82,124），弱可用态一眼可辨
                // v1.16.46 按钮花样 C：禁用态 hover 弱反馈第三层（96,82,124→
                // 118,102,148）——「可望不可点」弱可辨性；非 hover 禁用帧逐像素
                // =旧实现；主/副标禁用色（v1.8.0 A6 定稿）不动只动描边一色
                if hot {
                    Color::from_rgba(118, 102, 148, 255)
                } else {
                    Color::from_rgba(96, 82, 124, 255)
                }
            } else if accent {
                C_GOLD
            } else {
                C_GOLD_D
            },
            a,
        ),
    );
    if press {
        // 内缩 2px 的高亮内衬：按压「陷下去」的视觉锚点（纯矩形零分配；随 dy 同步）
        draw_rectangle_lines(
            x + 2.0,
            y + 2.0 + dy,
            w - 4.0,
            h - 4.0,
            1.0,
            fade(Color::from_rgba(160, 128, 200, 130), a),
        );
        // v1.16.46 按钮花样 A：按压态外圈金辉——1px 外扩金圈与 2px 主框构成
        // 「按下时描边变厚变亮」层次感；α 与内衬线同乘 a（stagger/卡面入场窗同
        // 步显隐）；press=false 不落像素逐像素=旧实现；零状态零分配零新字形
        draw_rectangle_lines(
            x - 1.0,
            y - 1.0 + dy,
            w + 2.0,
            h + 2.0,
            1.0,
            fade(C_GOLD, 0.35 * a),
        );
    } else if hot {
        // v1.16.46 按钮花样 B：hot 态顶部内衬高光——hover=顶光、按压=全框+下沉
        // 两级内衬语言可辨；与 press 分支 if/else 互斥防按压帧双线叠加；非 hot
        // 帧零像素逐像素=旧实现；触屏 hot≡手指落钮（:106-108 口径）与按压态天
        // 然衔接；零状态零分配零新字形
        draw_rectangle(
            x + 2.0,
            y + 2.0,
            w - 4.0,
            1.0,
            // v1.16.48 复查①（勘察六唯一真发现）：禁用态顶光同批弱化 0.25→0.10
            //（金饰=可点语言全局自洽；enabled 钮逐比特不动、禁用非 hover 帧本就
            // 零像素；v11650_ux_pins_tests 顶光 needle 同批改勿半改）
            fade(
                Color::from_rgba(214, 200, 244, 255),
                (if enabled { 0.25 } else { 0.10 }) * a,
            ),
        );
        // v1.16.47 按钮花样延伸（勘察六花样 A）：hot 态内缩四角金角饰——照
        // panel_card 角饰语言（:296-320 四角 L 形 C_GOLD fade）但内缩 2px 画在
        // 框内（leg 10px/厚 2px/α 0.5a，小号同语）：零间距钮堆（终战四钮/charpanel
        // 页签）角饰不出框不与邻钮叠画（勘察六复查②③首受益面）；与 press 分支
        // if/else 互斥照花样 B 钉口径防按压帧双饰叠加；非 hot 帧零像素逐像素=旧
        // 实现；色 C_GOLD 勿复用花样 B 的 214,200,244（钉 needle 唯一性）；零状态
        // 零分配零新字形
        // v1.16.48 复查①（勘察六唯一真发现）：hot 判定（:131）无 enabled 门控
        // ——禁用钮 hover 帧曾照样画金角饰 α0.5 与花样 C「可望不可点弱反馈」
        // 语义相抵。弱化分档：enabled 0.5 逐比特不动、禁用 0.15（弱存在仅可辨
        // 非可用）；press 分支互斥不动；禁用非 hover 帧本就零像素稳态不变
        let jgold = fade(C_GOLD, (if enabled { 0.5 } else { 0.15 }) * a);
        let jleg = 10.0f32;
        let jt = 2.0f32;
        for (ax, ay, adx, ady) in [
            (x + 2.0, y + 2.0, 1.0, 1.0),
            (x + w - 2.0, y + 2.0, -1.0, 1.0),
            (x + 2.0, y + h - 2.0, 1.0, -1.0),
            (x + w - 2.0, y + h - 2.0, -1.0, -1.0),
        ] {
            draw_rectangle(
                ax - if adx < 0.0 { jleg } else { 0.0 },
                ay - if ady < 0.0 { jt } else { 0.0 },
                jleg,
                jt,
                jgold,
            );
            draw_rectangle(
                ax - if adx < 0.0 { jt } else { 0.0 },
                ay - if ady < 0.0 { jleg } else { 0.0 },
                jt,
                jleg,
                jgold,
            );
        }
    }
    if let Some(s) = sub {
        txt_c(
            label,
            x + w * 0.5,
            y + dy + h * 0.5 - 17.0,
            20,
            fade(
                // v1.8.0 A6：禁用主标 C_DIM→(184,178,196)，与可用态层级拉开
                if enabled {
                    C_TEXT
                } else {
                    Color::from_rgba(184, 178, 196, 255)
                },
                a,
            ),
        );
        txt_c(
            s,
            x + w * 0.5,
            y + dy + h - 26.0, 14,
            fade(
                if enabled {
                    // v1.16.48 按钮花样 C（勘察六提名三）：选项钮 hot 副标提色
                    // C_DIM→C_CYAN——hover 零状态反馈（非 hot 帧逐像素不变；颜
                    // 色不入字形缓存 key 零重栅格化零新字形；禁用臂弱反馈色不
                    // 动保持分层）
                    if hot {
                        C_CYAN
                    } else {
                        C_DIM
                    }
                } else {
                    // v1.8.0 A6：禁用副标提亮（152,148,168→178,172,192）
                    Color::from_rgba(178, 172, 192, 255)
                },
                a,
            ),
        );
    } else {
        txt_c(
            label,
            x + w * 0.5,
            y + dy + h * 0.5 - 11.0,
            18,
            fade(
                // v1.8.0 A6：禁用主标同上
                if enabled {
                    C_TEXT
                } else {
                    Color::from_rgba(184, 178, 196, 255)
                },
                a,
            ),
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
    panel_card_a(x, y, w, h, 1.0);
}

// 剧情演出包：色相不变、alpha 乘算（纯算术零分配；a=1.0 时与原色逐比特一致）
fn fade(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, c.a * a.clamp(0.0, 1.0))
}

/// 带透明度卡面（剧情演出包①：入场淡入用）。alpha=1.0 时逐像素等同 panel_card；
/// 底/描边/内衬/角饰同步乘 α，全 30+ 既有调用点经 panel_card 包装零改动。
pub(crate) fn panel_card_a(x: f32, y: f32, w: f32, h: f32, alpha: f32) {
    let a = alpha.clamp(0.0, 1.0);
    draw_rectangle(x, y, w, h, fade(C_PANEL, a));
    draw_rectangle_lines(x, y, w, h, 2.5, fade(C_GOLD_D, a));
    draw_rectangle_lines(
        x + 5.0,
        y + 5.0,
        w - 10.0,
        h - 10.0,
        1.0,
        Color::new(96.0 / 255.0, 76.0 / 255.0, 130.0 / 255.0, 160.0 / 255.0 * a),
    );
    // 四角金色 L 形角饰
    let cl = 16.0f32;
    let t = 3.0f32;
    let gold = fade(C_GOLD, a);
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
            gold,
        );
        draw_rectangle(
            cx - if dx < 0.0 { t } else { 0.0 },
            cy - if dy < 0.0 { cl } else { 0.0 },
            t,
            cl,
            gold,
        );
    }
}

// 剧情演出包⑤：标题分隔线金饰——中央菱形金点（三层递变矩形叠合，纯矩形无旋转）
// + 两侧短金线（照 draw_banner 装饰线先例）。零分配零新字形；alpha 随卡面入场淡入，
// 稳态 alpha=1.0。分隔线跨度最窄 520（普通版式），饰线总宽 128 居中恒不出界。
fn title_ornament(cx: f32, cy: f32, alpha: f32) {
    let gold = fade(C_GOLD, alpha);
    let line = fade(C_GOLD_D, alpha);
    // 菱形：宽 10 高 2 / 宽 6 高 4 / 宽 2 高 8 三层叠合
    draw_rectangle(cx - 5.0, cy - 1.0, 10.0, 2.0, gold);
    draw_rectangle(cx - 3.0, cy - 2.0, 6.0, 4.0, gold);
    draw_rectangle(cx - 1.0, cy - 4.0, 2.0, 8.0, gold);
    // 两侧短金线：16px 留白 + 48px 线长，1.5px 高与分隔线同语
    draw_rectangle(cx - 64.0, cy - 0.75, 48.0, 1.5, line);
    draw_rectangle(cx + 16.0, cy - 0.75, 48.0, 1.5, line);
}

// v1.16.7 事件正文分段绘制（复审②修正：收秒标度 t=0.18-g.modal_anim，不再把
// 归一化 p 当秒用——段 0 相位 (t-0.08)/0.10 与旧 text_a 逐像素一致，「80ms 后
// 开始、100ms 淡满」的原节奏保留，段间 40ms）。段 i 除数取 0.10-i*0.04 压缩后续
// 段淡入时长：0.18s 入场窗口物理上装不下「0.08+0.04+0.10」的段 1 全程（复审定
// 稿原式 (t-0.08-i*0.04)/0.10 在 t=0.18 稳态时段 1 alpha 恒 0.6 半透明），压缩后
// t=0.18 时全段恒 1——稳态逐像素不变红线优先，段 1 变为 120ms 后开始、60ms 淡满。
// 按 '\n' 迭代分段（split 迭代器零分配），段间 8px；单段卡无段距，返回值与
// wrap_draw 相等。段落各入 wrap 缓存（key=段文本），段数少无膨胀；动画期变色不
// 触发字形重栅格（颜色不入缓存 key，ui.rs WrapKey 先例）
fn draw_event_text(
    s: &str,
    x: f32,
    y: f32,
    size: u16,
    maxw: f32,
    line_h: f32,
    c: Color,
    t: f32,
    max_h: f32, // v1.16.9 行数上界防御（复审 low①）：可用文本区像素高；0=不限。
                // 先判后画——wrap_lines 与 wrap_draw 同源测宽，累计高度（行×行高+
                // 段距 8px）超 max_h 即截断余段，绝不压选项钮区。当前事件表静态文案
                // 均在预算内（tall ≤4 行/普通 ≤3 行），本防御为几何保险，防未来
                // 长文案/动态拼文溢出（宿敌卡 full() 嵌名长度随境界名浮动尤需此保险）
) -> f32 {
    let mut yy = y;
    let mut total = 0.0f32;
    let mut used_h = 0.0f32;
    for (i, seg) in s.split('\n').enumerate() {
        let dur = (0.10 - i as f32 * 0.04).max(0.02); // 防御：段数>2 时除数不归零
        let seg_a = ((t - 0.08 - i as f32 * 0.04) / dur).clamp(0.0, 1.0);
        let seg_lines = crate::ui::wrap_lines(seg, size, maxw);
        let seg_h = seg_lines as f32 * line_h;
        let gap = if i == 0 { 0.0 } else { 8.0 }; // 段间距（与下方 yy 步进同源）
        if max_h > 0.0 && used_h + gap + seg_h > max_h {
            break; // 预算尽，余段截断（先判后画：本段一个像素都不落）
        }
        used_h += gap + seg_h;
        let h = wrap_draw(seg, x, yy, size, maxw, line_h, fade(c, seg_a));
        yy += h + 8.0; // 段间距
        total += h + 8.0;
    }
    total - 8.0 // 末段无尾距（单段卡 total==行高*行数，与旧版式一致）
}

// v1.16.7 复审④：链段阶段徽标文本——取值域固定 5 种，match 产 &'static str
// 零分配（原每帧 format! 违反「每帧绘制路径禁止新增堆分配」红线）；5 串在
// build_event 尾 prewarm_text(&[16])
fn stage_label(seg: u8, total: u8) -> &'static str {
    match (seg, total) {
        (1, 2) => "1/2",
        (2, 2) => "2/2",
        (1, 3) => "1/3",
        (2, 3) => "2/3",
        (3, 3) => "3/3",
        _ => "",
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
    // v1.16.7 加载小刀：bg 纹理降采样 640×360（build_bg 省靴期 75% 像素写），
    // 此处 Linear 放大到全屏 1280×720——唯一消费点，原 draw_texture 按纹理原
    // 尺寸 1:1 贴屏，改 dest_size 后观感等位
    draw_texture_ex(
        &art.bg,
        0.0,
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(Vec2::new(V_W, V_H)),
            ..Default::default()
        },
    );
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
    // 演出包㉑ 宿敌斩杀·血芒四沿：neme_veil>0 期间屏幕四沿各一条 10px C_RED 窄边
    //（α=0.10×brk_veil_a 共用三角包络——单源勿双写照 ⑳ bond_mote_a 先例，峰值恰
    // 窗中点 t=0.6s）。置于横幅绘制之前=层级在横幅底下；四沿均不交横幅带 236..322
    //（上/下边 y<10/y>710，左/右边 x<10/x>1270 而横幅带 bx≥16 居中收拢——v1.8.0
    // A7 收窄口径）。稳态 neme_veil==0 → nv_a=0.0 → 早退整段不落像素逐比特恒等；
    // 独立于 banner Some（banner 单槽后写胜互吞时红边仍走完自身包络——勘察三风险
    // 栏备案可接受）。与⑲错开：彼是卡开门控金辉全屏、此是横幅时刻红辉四沿，层级/
    // 色/触发三异
    let nv_a = 0.10 * brk_veil_a((g.neme_veil / crate::game::NEME_VEIL_T).clamp(0.0, 1.0));
    if nv_a > 0.0 {
        let red = fade(C_RED, nv_a);
        draw_rectangle(0.0, 0.0, V_W, 10.0, red);
        draw_rectangle(0.0, V_H - 10.0, V_W, 10.0, red);
        draw_rectangle(0.0, 0.0, 10.0, V_H, red);
        draw_rectangle(V_W - 10.0, 0.0, 10.0, V_H, red);
    }
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
        // 剧情演出包⑪ 底带流光扫过：外带 w=60 α=0.05a + 内芯 w=44 α=0.06a 两矩形叠出
        // 削边光带（中心合成 α≈0.107a，勘察口径 ≈0.10a）。x 沿 k*1.2 sawtooth 左→右扫
        // 过、扫完回卷（2.4s 内恰回卷一次：k=0.8333 即 t 余 0.4s 时归零重来）；复用既有
        // k（k=(1−t/2.4) 收敛）与 a 包络，零新状态零分配零新字形（v1.9.25 字号档位纪律
        // :480-484 与 measure 原样未触碰）。置于描边字层之下不盖字。稳态自证：banner
        // 为 2.4s 一次性演出自灭（show_banner game.rs:2104 置 2.4 → update 计时归 None
        // game.rs:2756-2760），本段在 `if let Some(..) = &g.banner` 之内——无 banner 帧
        // （稳态）不落任何像素，与旧实现逐像素一致；动画期仅底带区域内 +2 矩形，
        // 描边/正文字仍由后绘覆盖其上。几何/α 常量抽纯函数+常量钉测消费
        //（v1.16.40 单4 补面照 weather_veil_alpha 先例——见文件尾 banner_sweep_tests）
        let sx = bx + banner_sweep_phase(k) * (bw - 60.0);
        draw_rectangle(sx, 236.0, 60.0, 86.0, Color::new(c.r, c.g, c.b, SWEEP_HALO_A * a));
        draw_rectangle(
            sx + 8.0,
            236.0,
            44.0,
            86.0,
            Color::new(c.r, c.g, c.b, SWEEP_CORE_A * a),
        );
        // 描边底（v1.8.0 R1 修：txt_c 为中心定位，原 `V_W*0.5 - w*0.5` 再被 txt_c 减半宽，
        // w/2 被双重应用——横幅整体左移 w/2，长标题首字被裁出左缘（既有缺陷，A7 收窄后显形）。
        // measure 与实际墨宽一致（shot_007 实测 512 ≈ w=518），直接以屏幕中心为锚）
        // 剧情演出包⑭ 落定震颤：描边 4 向与主字同加 banner_tremor_dx（活跃域
        // k∈[0.10,0.40] 外恒 0=x 恒等旧实现；包络函数见上方批注）
        let tdx = banner_tremor_dx(k, *t);
        if k < SPREAD_END {
            // 剧情演出包⑯ 逐字路径：字距 0.88×→1.0× 平方缓动展开（banner_spread）。
            // 每字 x=整体左缘+前缀 measure+idx×extra（前缀复刻=macroquad draw_text_ex
            // 内部纯 advance 累计同式——kern 恒 0 见 banner_spread 常量区批注；每字
            // 再经 txt floor，与整串「起点 floor+内部浮点累计」口径差 <1px，切换帧
            // k=SPREAD_END（spread 已收敛 1.0、extra 恰 0）≤1px 跳变备案可接受：
            // 2.4s 一次性演出、α 满格瞬间）。⑭ 震颤 tdx 加在整体左缘（照 :528 旧式
            // 与 dx 同级进锚点），描边 4 向与主字全程同加，相对几何不变。两段循环
            // 与旧实现层序同构（先全部描边、后全部主字——主字恒盖描边之上，相邻字
            // 覆盖语义零变化）；encode_utf8 栈缓冲零堆分配、chars() 迭代零分配，
            // x 表栈数组一次计算（描边 4 向与主字共用，measure 调用 5n→n 次）。
            // n>SPREAD_CHARS_CAP 走下方 else 旧整串路径（防御臂永不触发——文案
            // 长度上界由 banner_charset_kerning_zero 一并钉住）。
            let n = text.chars().count();
            if n <= SPREAD_CHARS_CAP {
                let extra = if n > 1 {
                    w * (banner_spread(k) - 1.0) / (n - 1) as f32
                } else {
                    0.0
                };
                let lx = V_W * 0.5 + tdx - w * 0.5; // 中心锚定左缘（txt_c 中心锚显式展开）
                let mut xs = [0.0f32; SPREAD_CHARS_CAP];
                for (idx, (bi, _)) in text.char_indices().enumerate() {
                    xs[idx] = lx + measure(&text[..bi], size) + idx as f32 * extra;
                }
                for (dx, dy) in [(-2.0, 0.0), (2.0, 0.0), (0.0, -2.0), (0.0, 2.0)] {
                    for (idx, (_, ch)) in text.char_indices().enumerate() {
                        let mut buf = [0u8; 4];
                        txt(
                            ch.encode_utf8(&mut buf),
                            xs[idx] + dx,
                            250.0 + dy,
                            size,
                            Color::new(0.05, 0.02, 0.1, 0.9 * a),
                        );
                    }
                }
                for (idx, (_, ch)) in text.char_indices().enumerate() {
                    let mut buf = [0u8; 4];
                    txt(
                        ch.encode_utf8(&mut buf),
                        xs[idx],
                        250.0,
                        size,
                        Color::new(c.r, c.g, c.b, a),
                    );
                }
            } else {
                self_draw_banner_text(text, tdx, size, a, *c);
            }
        } else {
            // 旧整串路径（k>=SPREAD_END 稳态）——逐字节原样未动，稳态逐像素红线锚
            self_draw_banner_text(text, tdx, size, a, *c);
        }
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

// 剧情演出包⑫ 天气变动过渡氛围层：advance_time weather_changed 帧置 weather_anim=0.8
//（game.rs 唯一 weather 写点与触发口同体 self.life），主循环递减，本助手在触发场景
// 消费——现实页主路径（手点/自动吐纳 act_cultivate → advance_time，game.rs:13320）与
// 世界图（赶路/远征 spend_year world.rs:92 + 洞府闭关 world.rs:909 act_cultivate）
// 各 1 行调用，共 1 个助手。
// 勘察原案 draw_sim 一址经复核判死码弃用：g.sim 天时是入镜快照（game.rs:4089 注，
// sim_year/sim_year_tail 全文无 roll，launch_sim game.rs:2641 克隆后不再演化），模拟
// 播放期现实侧推岁又被 tick_auto 场景闸（game.rs:13419 首行 scene!=Reality 即 return）
// 冻结——触发口与显示口永不重叠，且显示色取的会是冻结快照而非刚变值。
// 稳态自证：weather_anim==0（会话态默认/旧档载入/0.8s 包络走完）或表值 α=0（晴）时
// 早退不落任何像素——两场景稳态画面与旧实现逐像素一致；动画期在 bg 之上、面板/云雾
// 之下叠 1 个矩形（现实页全屏=面板后绘覆盖仅背板缝隙透光；世界图取地图画布区
// (8,60,1264,480) 与既有 WEATHER_TINT 通道同域），0.8s 自灭归零。tribbing 不设门控：
// 本层先绘，问笔暗幕等后绘覆层天然盖于其上。
pub fn draw_weather_veil(g: &Game, x: f32, y: f32, w: f32, h: f32) {
    let Some(l) = g.life.as_ref() else {
        return;
    };
    let (wr, wg, wb, wa) = crate::data::WEATHER_TINT[l.weather.min(4) as usize];
    let a = weather_veil_alpha(g.weather_anim, wa);
    if a <= 0.0 {
        return; // 照 world.rs:584 既有叠色先例：a>0 才画
    }
    draw_rectangle(
        x,
        y,
        w,
        h,
        Color::new(wr as f32 / 255.0, wg as f32 / 255.0, wb as f32 / 255.0, a),
    );
}

// 剧情演出包⑪ 流光扫过几何/α 常量（cfg(test) 钉测消费，照 weather_veil_alpha
// 先例）——draw_banner 消费；稳态自证见 draw_banner 内批注（banner 自灭后
// if-let 外不落像素）。
/// 流光相位：k∈[0,1]（k=(1−t/2.4) spawn 时 0、自灭时 1）→ sawtooth 相位 [0,1)，
/// 2.4s 内恰回卷一次（k≈0.8333 处归零重来）；流光 x = bx + 相位×(bw−60)。
// 演出包㉖ 远征启程·霞光启幡（世界图消费口，world.rs draw_world 调用）：exp_start
// 成功段单口置值 exp_depart=1.2→0，本口画霞光横线自上而下竖向扫过地图画布一次
//——位置线性下移（prog 0→1 顶→底）+共用 brk_veil_a 三角包络首尾淡入出（单源
// 勿双写）。签名照 draw_weather_veil (g,x,y,w,h) 款由调用方传地图画布域；置于
// ⑫ draw_weather_veil 口之后绘制不改动既有叠序。exp_depart<=0 或包络 α==0 早退
// 不落像素，稳态逐像素不变；exp_done/途中 exp_choose 不置值=每远征仅启程一口
//（语义=「启程」非「战果」，v11656 钉面写死）。金色取 C_GOLD（game.rs 常量，
// fade 同族）；与⑪底带流光（横带循环）/⑫天气氛围（全画布叠色）方向/区域/语义三异
pub fn draw_depart_beam(g: &Game, x: f32, y: f32, w: f32, h: f32) {
    let ed = g.exp_depart;
    if ed <= 0.0 {
        return;
    }
    let a = 0.14 * brk_veil_a((ed / crate::game::EXP_DEPART_T).clamp(0.0, 1.0));
    if a <= 0.0 {
        return;
    }
    let prog = 1.0 - ed / crate::game::EXP_DEPART_T; // 0→1 顶→底（ed 自 T 递减）
    let yy = y + prog * (h - 10.0);
    draw_rectangle(x, yy, w, 2.0, fade(C_GOLD, a));
}

// 演出包㉗ 远征收手·霞光合拢（v1.16.50）：exp_quit 收敛口单口置值 exp_return，
// 世界图消费（world.rs draw_depart_beam 调用口之后同域绘制不改动既有叠序）。
// 与㉖ 三异：方向自下而上（「霞光重新合拢」照 world.rs exp_quit 出秘境句语义）、
// 颜色 C_CYAN（与⑫天气/㉒呼吸同色系但区域/方向/语义三异照㉖论证款）、触发口
// （收手非启程）。er<=0 或包络 α==0 早退不落像素，稳态逐像素不变；exp_quit 是
// 唯一置值口（exp_start/exp_choose 均不置值=每远征仅收手一口，v11659 钉面写死）。
// 结构照 draw_depart_beam 同款（共用 brk_veil_a 包络单源勿双写）
pub fn draw_return_beam(g: &Game, x: f32, y: f32, w: f32, h: f32) {
    let er = g.exp_return;
    if er <= 0.0 {
        return;
    }
    let a = 0.14 * brk_veil_a((er / crate::game::EXP_RETURN_T).clamp(0.0, 1.0));
    if a <= 0.0 {
        return;
    }
    let prog = 1.0 - er / crate::game::EXP_RETURN_T; // 0→1 底→顶（er 自 T 递减）
    let yy = y + (1.0 - prog) * (h - 10.0);
    draw_rectangle(x, yy, w, 2.0, fade(C_CYAN, a));
}

#[inline]
fn banner_sweep_phase(k: f32) -> f32 {
    (k * 1.2) % 1.0
}

/// 外带 α 系数（×a 包络；中心与内芯叠加合成 ≈0.107a，勘察口径 ≈0.10a）
const SWEEP_HALO_A: f32 = 0.05;
/// 内芯 α 系数（×a 包络）
const SWEEP_CORE_A: f32 = 0.06;

// 剧情演出包⑭ 横幅落定震颤（零新状态，复用既有 k/t 双参数）：pop scale 于 k=0.25
//（t=1.8s）落定，落定瞬间文字本体水平震颤 ±1.2px 余韵——三角包络活跃域
// k∈[0.10,0.40]（t∈[1.44,2.16]s，恰覆盖落定点），包络外恒 0（描边/正文字 x 恒等
// 旧实现）。仅动 x 位置：字号档位（:480-484 v1.9.25 红线）/measure/颜色/字形全原样，
// 描边 4 向与主字同 dx 保持相对几何；底带矩形/流光/装饰线不动（流光动带、震颤动字，
// 绘制序不变字后绘盖流光）。对全部横幅触发点（破境/飞升/斩敌/换幕/章节/血仇等
// 10+ 口）统一生效，零新状态不设 kind 旗标（勘察风险 5 备案：统一语义通用任一
// 大事件）。稳态自证与⑪同款：banner 2.4s 自灭（show_banner 置 2.4 → update 归
// None），draw_banner 段在 if-let 之内——无 banner 帧（稳态）不落任何像素。
#[inline]
fn banner_tremor_dx(k: f32, t: f32) -> f32 {
    let env = (1.0 - ((k - 0.25) / 0.15).abs()).max(0.0);
    env * TREMOR_AMP * (t * 40.0).sin()
}

/// 震颤幅度（px）：包络峰值处最大水平位移（cfg(test) 钉测消费）
const TREMOR_AMP: f32 = 1.2;

// 剧情演出包⑯ 横幅主字字距展开（照 zoom_overlay_p 早退恒等先例）：入场淡入窗
// k∈[0,SPREAD_END] 内主字/描边逐字绘制、字距自 0.88× 平方缓动展开至 1.0×——横幅
// 大字「聚拢入画再展开落定」的力度感；k>=SPREAD_END 走旧整串绘制路径（代码分支
// 恒等——比参数收敛更强的稳态保证，k>=END 后与旧实现逐像素一致）。稳态自证照
// ⑪⑭同款：banner 2.4s 自灭，draw_banner 段在 if-let 之内——无 banner 帧（稳态）
// 不落任何像素；动画窗内 α=k/0.12<1 且字距展开中，属演出形态。
// 红线一（v1.9.25 字号档位 :480-484）：仅动 x 位置，size 全程取同一档位——逐字
// 绘制的 (字符,字号) 缓存 key 与整串一致，show_banner 预热（game.rs:2158 弹出帧
// 预热 {54,60,66,72} 全档）已覆盖，零新字形零图集重传。
// 布局复刻前提（kerning 口径）：macroquad draw_text_ex 内部布局=advance 累计+
// kern 偏移（vendor/macroquad/src/text.rs:434 total_width 累计含 kerning_offset），
// 而 measure_text=纯 advance 累计（:206）——两者一致当且仅当 kern 恒 0。本工程
// 字体 assets/font.ttf 无 kern 表（fonttools 实测表列无 kern），fontdue 仅解析
// kern 表不读 GPOS（fontdue-0.9.4 src/table/ 唯有 kern.rs）→ horizontal_kern
// map 恒 None → draw_text_ex 内 :384-386 unwrap_or(0.0) → kern 偏移恒 0——前缀
// measure 与整串内部累计逐位同式（同字体同字号同 advance 同序累计）。钉测
// banner_charset_kerning_zero 防字体重生成漂移：若 make_font.py 未来产出带 kern
// 表的字体且含非零对，钉测红提醒本假设失效须重评（备选金尘粒子方案留勘察档）。
/// 字距展开窗终点（k 阈）：与 α 淡入窗 0.12（draw_banner :473 字面量）同窗——
/// α 满格与字距展开完成同刻切回整串路径（切换帧 spread 恒 1.0、extra 恒 0）。
const SPREAD_END: f32 = 0.12;
/// 字距初始收窄量（×(1−k/SPREAD_END)² 平方缓动）：k=0 时字距系数恰 0.88。
const SPREAD_SHRINK: f32 = 0.12;

/// 字距系数：k>=SPREAD_END 恒返 1.0（早退恒等路径，整串分支判据）；窗内
/// 1.0−SPREAD_SHRINK×(1−k/SPREAD_END)²（k=0 恰 0.88 聚拢、窗内单调升、收敛 1.0）。
#[inline]
fn banner_spread(k: f32) -> f32 {
    if k >= SPREAD_END {
        return 1.0;
    }
    1.0 - SPREAD_SHRINK * (1.0 - k / SPREAD_END) * (1.0 - k / SPREAD_END)
}

/// 逐字路径 x 表栈数组容量（字符数上界）：横幅全部触发口文案实测最长 13 字符
///（「山 河 换 幕 · X」X=五域域名 ≤4——上界由 banner_charset_kerning_zero 运行时
/// 遍历 REALMS/MAP_THEMES/ENDINGS/CHAPTERS+固定短语逐一断言 ≤ 此值，表扩容文案
/// 变长时钉测红提醒调容量，防御臂永不触发）。
const SPREAD_CHARS_CAP: usize = 16;

/// 旧整串主字绘制（描边 4 向+主字，⑯ 前唯一路径）——draw_banner 双路径共用抽体：
/// 稳态分支（k>=SPREAD_END）与逐字防御臂（n>SPREAD_CHARS_CAP，永不触发）同走，
/// 五处 txt_c 逐字节未动——稳态逐像素红线锚。
fn self_draw_banner_text(text: &str, tdx: f32, size: u16, a: f32, c: Color) {
    for (dx, dy) in [(-2.0, 0.0), (2.0, 0.0), (0.0, -2.0), (0.0, 2.0)] {
        txt_c(
            text,
            V_W * 0.5 + dx + tdx,
            250.0 + dy,
            size,
            Color::new(0.05, 0.02, 0.1, 0.9 * a),
        );
    }
    txt_c(
        text,
        V_W * 0.5 + tdx,
        250.0,
        size,
        Color::new(c.r, c.g, c.b, a),
    );
}

// 剧情演出包⑫ 包络纯函数（cfg(test) 钉测消费，照 menu_danger_btn_hit 先例）：
// anim 剩余秒 0.8→0（k=anim/0.8 随时间 1→0），前半程渐显后半程渐隐 env∈[0,1]，
// 合成 α=WEATHER_TINT 表值/255×env；≤0 表示不画。
fn weather_veil_alpha(anim: f32, wa: u8) -> f32 {
    if anim <= 0.0 || wa == 0 {
        return 0.0;
    }
    let k = (anim / 0.8).clamp(0.0, 1.0);
    let env = if k > 0.5 { (1.0 - k) / 0.5 } else { k / 0.5 };
    (wa as f32 / 255.0) * env
}

// 剧情演出包⑫ 包络钉测：稳态归零/表值上限/三角对称/两段单调
#[cfg(test)]
mod weather_veil_tests {
    use super::weather_veil_alpha;

    #[test]
    fn veil_alpha_steady_peak_symmetry() {
        // 稳态两臂：无动画 / 表值 α=0（晴）——任一命中即不落像素（稳态逐像素口径）
        assert_eq!(weather_veil_alpha(0.0, 34), 0.0, "无动画不画（稳态口径）");
        assert_eq!(weather_veil_alpha(0.8, 0), 0.0, "晴表值 α=0 恒不画");
        // 峰值恰表值/255（k=0.5 → env=1）：雨 wa=34 铁例
        let peak = 34.0f32 / 255.0;
        assert!(
            (weather_veil_alpha(0.4, 34) - peak).abs() < 1e-6,
            "k=0.5 峰值应恰表值/255，实得 {}",
            weather_veil_alpha(0.4, 34)
        );
        // 开帧（k=1）env 恰 0——包络两端归零、渐显起步
        assert_eq!(weather_veil_alpha(0.8, 34), 0.0, "开帧渐显起点应为 0");
        // 收尾趋零（anim=0.04 → k=0.05 → env=0.1，不到峰值两成）
        assert!(
            weather_veil_alpha(0.04, 34) < peak * 0.2,
            "收尾渐隐应接近归零，实得 {}",
            weather_veil_alpha(0.04, 34)
        );
        // 三角对称：k 与 1−k 同 α（anim 0.16↔0.64、0.28↔0.52）
        assert!(
            (weather_veil_alpha(0.16, 34) - weather_veil_alpha(0.64, 34)).abs() < 1e-6,
            "包络应关于 k=0.5 对称"
        );
        assert!(
            (weather_veil_alpha(0.28, 34) - weather_veil_alpha(0.52, 34)).abs() < 1e-6,
            "包络应关于 k=0.5 对称"
        );
        // 两段单调：渐显段随时间升（0.7→0.6 α 增）、渐隐段随时间降（0.3→0.2 α 减）
        assert!(weather_veil_alpha(0.7, 34) < weather_veil_alpha(0.6, 34));
        assert!(weather_veil_alpha(0.3, 34) > weather_veil_alpha(0.2, 34));
    }
}

// 剧情演出包⑪ 流光钉测（v1.16.40 单4 补面，照 weather_veil_tests 同款纯函数钉
// 样式）：相位 sawtooth（端点/值域/首圈单调/恰回卷一次）+ α 常量中心合成≈0.107
//（勘察口径≈0.10a）。几何/α 为纯常量表达式；稳态（无 banner 帧）零像素由
// draw_banner 的 if-let 结构保证（横幅 2.4s 自灭，game.rs show_banner/update 口），
// 本模组钉动画期常量口径。
#[cfg(test)]
mod banner_sweep_tests {
    use super::{
        banner_spread, banner_sweep_phase, banner_tremor_dx, SPREAD_CHARS_CAP, SPREAD_END,
        SPREAD_SHRINK, SWEEP_CORE_A, SWEEP_HALO_A, TREMOR_AMP,
    };

    // 相位端点与值域：k=0 起扫左缘；k∈[0,1] 全域相位 [0,1)
    #[test]
    fn sweep_phase_domain_and_ends() {
        assert_eq!(banner_sweep_phase(0.0), 0.0, "spawn 相位 0（左缘起扫）");
        let ks = [0.0f32, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0];
        for k in ks {
            let p = banner_sweep_phase(k);
            assert!(
                (0.0..1.0).contains(&p),
                "k={} 相位 {} 越域 [0,1)",
                k,
                p
            );
        }
    }

    // 回卷恰一次：60 步采样 k∈[0,1] 相位序列恰一个下降沿；首圈（k≤5/6）严格
    // 递增（左→右扫）；k=1（自灭帧）相位=0.2（第二圈 20%）
    #[test]
    fn sweep_phase_wrap_and_monotonic() {
        let steps = 60usize;
        let mut wraps = 0usize;
        let mut prev = 0.0f32;
        for idx in 1..=steps {
            let k = idx as f32 / steps as f32;
            let p = banner_sweep_phase(k);
            assert!((0.0..1.0).contains(&p), "k={} 相位越域", k);
            if p < prev {
                wraps += 1;
            } else if k <= 5.0 / 6.0 + 1e-6 {
                assert!(p > prev, "首圈相位应严格递增：k={} p={} prev={}", k, p, prev);
            }
            prev = p;
        }
        assert_eq!(wraps, 1, "2.4s 内恰回卷一次，实得 {} 次", wraps);
        let p1 = banner_sweep_phase(1.0);
        assert!(
            (p1 - 0.2).abs() < 1e-4,
            "k=1 相位应 0.2（回卷后 20%），实得 {}",
            p1
        );
    }

    // α 常量钉：中心合成 1−(1−0.05)(1−0.06)≈0.107（勘察口径 ≈0.10a）
    #[test]
    fn sweep_alpha_constants() {
        assert_eq!(SWEEP_HALO_A, 0.05);
        assert_eq!(SWEEP_CORE_A, 0.06);
        let comp = 1.0 - (1.0 - SWEEP_HALO_A) * (1.0 - SWEEP_CORE_A);
        assert!(
            (comp - 0.107).abs() < 1e-3,
            "中心合成 α 应≈0.107，实得 {}",
            comp
        );
        assert!(
            comp > 0.10 && comp < 0.115,
            "合成 α 守勘察口径带 (0.10,0.115)"
        );
    }

    // 剧情演出包⑭ 落定震颤钉测：包络两端恰归零（活跃域 k∈[0.10,0.40] 外 dx 恒 0=
    // 描边/正文字 x 恒等旧实现）+ 全域有界 |dx|≤1.2 + 活跃域内实际起振 + 幅值常量钉。
    // 稳态自证照⑪同款由 draw_banner 的 if-let 结构保证（本模组钉动画期常量口径）；
    // 震颤仅动 x、字号/颜色/字形零变化（v1.9.25 档位红线未触碰）。
    #[test]
    fn tremor_envelope_ends_and_bounds() {
        // 包络外（k≤0.10 / k≥0.40）恒 0——采样含端点与远端，t 用该 k 对应真实剩余秒
        for k in [0.0f32, 0.05, 0.10, 0.40, 0.5, 0.75, 0.9, 1.0] {
            let t = 2.4 * (1.0 - k);
            assert_eq!(
                banner_tremor_dx(k, t),
                0.0,
                "k={} 包络外应恰 0（文字位置恒等旧实现）",
                k
            );
        }
        // 全域 60 步采样：|dx|≤1.2 有界；活跃域内应实际起振（最大 |dx|>0.5）
        let mut max_abs = 0.0f32;
        for idx in 0..=60 {
            let k = idx as f32 / 60.0;
            let t = 2.4 * (1.0 - k);
            let dx = banner_tremor_dx(k, t);
            assert!(
                dx.abs() <= TREMOR_AMP + 1e-6,
                "k={} dx={} 超幅 {}",
                k,
                dx,
                TREMOR_AMP
            );
            max_abs = max_abs.max(dx.abs());
        }
        assert!(max_abs > 0.5, "活跃域内应实际起振，最大 |dx|={}", max_abs);
        // 落定点 k=0.25 恰在包络中心（env=1 峰值域）；幅值常量口径
        let k_peak = 0.25f32;
        assert!(
            banner_tremor_dx(k_peak, 2.4 * (1.0 - k_peak)).abs()
                <= TREMOR_AMP,
            "落定点应在峰值域内有界"
        );
        assert_eq!(TREMOR_AMP, 1.2);
    }

    // 震颤时间连续性：t 递减（2.16→1.44，活跃域全程）dx 振荡但幅度受三角包络约束——
    // 包络峰值线（k=0.25 邻域）处振幅应显著大于活跃域边缘（包络形状非平顶）
    #[test]
    fn tremor_envelope_shape() {
        // 边缘 5%（k=0.11/k=0.39）包络≈0.067，中心 5% 包络≈1.0：取同相位难保证，
        // 改钉包络上界——任意 t 下 |dx(k_edge)| ≤ 0.15×TREMOR_AMP+容差
        let edge = [0.11f32, 0.39];
        for k in edge {
            for step in 0..40 {
                let t = 2.16 - step as f32 * 0.018; // 2.16→1.44 全域扫
                assert!(
                    banner_tremor_dx(k, t).abs() <= 0.12 * TREMOR_AMP + 1e-6,
                    "k={} 边缘包络（1−0.14/0.15≈0.0667）应 ≤0.12×幅值，t={} 实得 {}",
                    k,
                    t,
                    banner_tremor_dx(k, t)
                );
            }
        }
    }

    // 剧情演出包⑯ 字距展开钉测：k=0 恰 0.88 聚拢 / k>=SPREAD_END 恒 1.0（早退恒等
    // 路径，含端点 0.12——切换帧两侧同值）/窗内严格单调升 / 常量口径（SPREAD_END
    // 与 α 淡入窗 0.12 同窗、SPREAD_SHRINK 同值 0.12）。
    #[test]
    fn banner_spread_envelope() {
        assert!(
            (banner_spread(0.0) - 0.88).abs() < 1e-6,
            "k=0 应恰 0.88 聚拢，实得 {}",
            banner_spread(0.0)
        );
        // 早退恒等臂：k=SPREAD_END 端点与窗后全域恒 1.0（整串分支判据同值）
        for k in [SPREAD_END, 0.5f32, 1.0] {
            assert_eq!(
                banner_spread(k),
                1.0,
                "k={} 应走早退恒等路径（整串分支判据）",
                k
            );
        }
        // 窗内严格单调升（0.12 等 13 步）：0.88 → 1.0
        let mut prev = 0.0f32;
        for step in 0..=12usize {
            let k = SPREAD_END * step as f32 / 12.0;
            let s = banner_spread(k);
            assert!(s > prev, "窗内应严格单调升：step={} s={} prev={}", step, s, prev);
            prev = s;
        }
        assert!((prev - 1.0).abs() < 1e-6, "窗尾应收敛 1.0，实得 {}", prev);
        // 常量口径：SPREAD_END 与 α 淡入窗同窗（draw.rs draw_banner :473 字面量 0.12）
        assert_eq!(SPREAD_END, 0.12);
        assert_eq!(SPREAD_SHRINK, 0.12);
    }

    // 剧情演出包⑯ 布局复刻前提钉（两钉一体）：
    // ① kerning 全零——macroquad draw_text_ex 内部布局=advance 累计+kern 偏移
    //   （vendor/macroquad/src/text.rs:434），measure_text=纯 advance（:206），逐字
    //   前缀 measure 复刻成立当且仅当 kern 恒 0。fontdue 仅解析 kern 表不读 GPOS
    //   （fontdue-0.9.4 src/table/ 唯有 kern.rs），本字体 assets/font.ttf 无 kern 表
    //   （fonttools 实测）——此处用同一 fontdue（Cargo.toml dev-dep 与 vendor
    //   macroquad 同版 0.9）解析同一 ttf，对横幅全部触发口文案字符集两两查询恒 0。
    //   若未来 make_font.py 产出带 kern 表字体且含非零对，本钉测红提醒布局假设
    //   失效须重评（备选金尘粒子方案留勘察档）。
    // ② 文案长度上界——横幅全部触发口动态文案最长 ≤ SPREAD_CHARS_CAP（逐字路径
    //   x 表栈数组容量；超出走防御臂整串路径，钉测在此保证防御臂永不触发）。
    #[test]
    fn banner_charset_kerning_zero() {
        let font = fontdue::Font::from_bytes(
            include_bytes!("../assets/font.ttf").as_ref(),
            fontdue::FontSettings::default(),
        )
        .expect("字体解析失败");
        // 横幅全部触发口动态文案源（照 draw_banner 10+ 口并集）+ 固定短语
        let mut src = String::from("飞 升 成 仙现 实 飞 升血 仇 得 报宿敌授首！突破 · 境结 局 · 《》");
        for r in crate::data::REALMS.iter() {
            src.push_str(r.name); // 「突破 · {name}境」
        }
        for t in crate::data::MAP_THEMES.iter() {
            src.push_str(t.name()); // 「山 河 换 幕 · {name}」
        }
        for e in crate::game::ENDINGS.iter() {
            src.push_str(e.name()); // 「结 局 · 《{name}》」
        }
        for c in crate::data::CHAPTERS.iter() {
            src.push_str(c.num());
            src.push_str(c.name()); // 「{num} · {name}」
        }
        // ② 文案长度上界钉：全部触发口最长文案（含标点/空格）≤ SPREAD_CHARS_CAP
        let mut max_len = 0usize;
        for r in crate::data::REALMS.iter() {
            max_len = max_len.max(format!("突破 · {}境", r.name).chars().count());
        }
        for t in crate::data::MAP_THEMES.iter() {
            max_len = max_len.max(format!("山 河 换 幕 · {}", t.name()).chars().count());
        }
        for e in crate::game::ENDINGS.iter() {
            max_len = max_len.max(format!("结 局 · 《{}》", e.name()).chars().count());
        }
        for c in crate::data::CHAPTERS.iter() {
            max_len = max_len.max(format!("{} · {}", c.num(), c.name()).chars().count());
        }
        max_len = max_len.max("飞 升 成 仙".chars().count());
        max_len = max_len.max("现 实 飞 升".chars().count());
        max_len = max_len.max("血 仇 得 报".chars().count());
        max_len = max_len.max("宿敌授首！".chars().count());
        assert!(
            max_len <= SPREAD_CHARS_CAP,
            "横幅文案最长 {} 字符超 SPREAD_CHARS_CAP={}——防御臂将触发，须调容量",
            max_len,
            SPREAD_CHARS_CAP
        );
        // ① 字符集两两 kern 查询恒 0（None=无 kern 项 / Some(0)=零值项——对布局
        // 等价；非零值即红）。集合去重控规模（≈150 字 → <20000 查询，瞬时）
        let uniq: std::collections::BTreeSet<char> = src.chars().collect();
        let cs: Vec<char> = uniq.into_iter().collect();
        for (a_idx, a) in cs.iter().enumerate() {
            for b in cs.iter() {
                // 单向 a→b 全查（含同字对）；对称对由 b→a 轮次自然覆盖
                if a == b {
                    continue;
                }
                let k = font.horizontal_kern(*a, *b, 72.0).unwrap_or(0.0);
                assert_eq!(
                    k, 0.0,
                    "kern 非零：{:?}→{:?} = {}——⑯ 前缀 measure 布局假设失效",
                    a, b, k
                );
            }
        }
    }

    // 剧情演出包⑯ 双路径结构静态钉（施工单四补面，照 game.rs comm_hooks_static_pins
    // fs 读源+计数/锚行款）：draw_banner 双路径判据/旧整串路径抽体/前缀 measure
    // 复刻消费点/震颤 tdx 加性关系/零分配面/α 淡入窗同窗六面锚——结构性防漂移：
    // ①双路径判据 `if k < SPREAD_END {` 恰 1（删分支即稳态走逐字=非恒等）；
    // ②self_draw_banner_text 调用恰 2（防御臂+稳态臂共用抽体——抽体即旧实现逐字
    // 节，少一处=某臂改写；多于 2=新增臂未经稳态评估）；③前缀 measure 复刻消费点
    // 恰 1；④震颤 tdx 加在整体左缘（⑭加性关系保持）；⑤encode_utf8 栈缓冲恰 2
    //（描边/主字两循环——新增堆分配即红）；⑥α 淡入窗字面量 `if k < 0.12 {` 恰 1
    // 在位——SPREAD_END 常量钉 0.12 与此同窗（banner_spread_envelope 已钉常量值），
    // 改 α 窗不同步 SPREAD_END 即切换帧不在 α 满格点。
    #[test]
    fn spread_dual_path_static_pins() {
        let root = env!("CARGO_MANIFEST_DIR");
        let src = std::fs::read_to_string(
            std::path::Path::new(root).join("src").join("draw.rs"),
        )
        .expect("读 src/draw.rs");
        // prod 剔 cfg(test) 区（照 comm_hooks_static_pins 款——test 区含同款字符串
        // 会污染计数）
        let mut prod = String::new();
        let mut skip = false;
        for line in src.lines() {
            if line.starts_with("#[cfg(test)]") {
                skip = true;
                continue;
            }
            if skip {
                if line == "}" {
                    skip = false;
                }
                continue;
            }
            prod.push_str(line);
            prod.push('\n');
        }
        let count = |hay: &str, needle: &str| hay.matches(needle).count();
        assert_eq!(
            count(&prod, "if k < SPREAD_END {"),
            1,
            "⑯ 双路径判据须恰 1（draw_banner 主字段）"
        );
        assert_eq!(
            count(&prod, "self_draw_banner_text(text, tdx, size, a, *c);"),
            2,
            "旧整串路径抽体调用须恰 2（逐字防御臂+稳态臂）"
        );
        assert_eq!(
            count(&prod, "lx + measure(&text[..bi], size)"),
            1,
            "前缀 measure 复刻消费点须恰 1（x 表一次计算）"
        );
        assert_eq!(
            count(&prod, "V_W * 0.5 + tdx - w * 0.5"),
            1,
            "⑭ 震颤 tdx 须加在整体左缘（中心锚定左缘锚行）"
        );
        assert_eq!(
            count(&prod, "ch.encode_utf8(&mut buf),"),
            2,
            "encode_utf8 栈缓冲须恰 2（描边/主字两循环——新增堆分配即红）"
        );
        assert_eq!(
            count(&prod, "let a = if k < 0.12 {"),
            1,
            "α 淡入窗字面量锚行须在位——与 SPREAD_END=0.12 同窗（切换帧 α 满格点）"
        );
        assert_eq!(SPREAD_END, 0.12, "SPREAD_END 常量与 α 窗同值（同窗判据）");
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

// v1.16.13 复审①：菜单底排两危险钮命中守卫单口（draw_menu 两钮与单测共用）——
// panel!=0（书志/烙印/帮助/角色阅读态）时立即模式底排 button 先于罩层绘制命中，
// 同帧重叠矩形全触发（见下方 619-621 注先例），一律不做命中；承接前缘钮另需
// 有档（need_life）。回归口：书志页全貌带 y554 行落按钮带 522..576 内，开书志
// 点该带不得静默 resume_life。
fn menu_danger_btn_hit(panel: u8, need_life: bool, has_life: bool) -> bool {
    panel == 0 && (!need_life || has_life)
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
        evtx(2523)/* 原文：天道写你的书，写了十万年，写废了十万稿——这一稿，不肯退 */,
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
    if menu_danger_btn_hit(g.panel, false, false)
        && button(bx, 522.0, 300.0, 54.0, "开始新的人生", None, true)
    {
        g.play(crate::sounds::Which::Click);
        g.begin_mortal();
    }
    // v1.16.13 复审②（同 619-621 同帧重叠先例）：本钮带 522..576 与书志页全貌带
    // y554 行重叠——panel!=0 阅读态下不做命中，防开书志点全貌带静默 resume_life
    if menu_danger_btn_hit(g.panel, true, g.meta.life.is_some()) {
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
    // v1.16.47 按钮花样延伸（勘察六花样 D）：toggle on 态内底 3px 金下划线——选中
    // 态语言补缺（照周天槽 8 内底画法同款几何 y+h−3..y+h）；关态零像素零新字形零
    // 状态。画面细节①（三处既有下划线 y 统一）不同批不做——那是既有稳态像素移位
    // 需目检支撑，本件为纯新增像素低风险
    if g.meta.music {
        draw_rectangle(bx, 592.0 + 44.0 - 3.0, 140.0, 3.0, C_GOLD);
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
    // v1.16.47 花样 D 同批（音效 toggle，几何照音乐款）
    if g.meta.sfx {
        draw_rectangle(bx + 155.0, 592.0 + 44.0 - 3.0, 140.0, 3.0, C_GOLD);
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
        // v1.16.11 书志页全量字形预热（事件帧一次，照 prewarm_guide_panel 单口语言
        // ——顺带治书志页标题/导语/？？？/ENDINGS 名+alias+gain 首开全冷的既有潜伏项）
        prewarm_book_record(g);
    }
    let best = REALMS[g.meta.best_realm.min(14)].name;
    txt_c(
        &evf(2524, &[&(g.meta.lives) as &dyn core::fmt::Display, &(best) as &dyn core::fmt::Display, &(g.meta.neme_slain) as &dyn core::fmt::Display, &(fmt_int(g.meta.dao_yun)) as &dyn core::fmt::Display])/* 原文：已历 {} 世 · 最高境界 {}境 · 斩杀宿敌 {} · 道韵 {} */,
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
    // v1.16.4：暂避收起态（modal_min）不再独占早退——场景照画（日志/面板可见），
    // 尾部 draw_modal_card 只画「待抉择」徽章；抉择仍独占（Esc 吞/C·M 避让守卫
    // 与 npc_ev 未消耗不变），赠礼等再触发被 maybe_fav_event 的 rmodal 守卫挡住
    if g.rmodal.is_some() && !g.modal_min {
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
    // v1.16.1 立绘鉴赏覆层：模态独占输入（人物页/图鉴点立绘开启；任意点按/Esc 关闭，
    // 关闭后回到所在面板）。照 rmodal/ending_show 早退先例——下层不绘不响应，天然
    // 免穿透；开启帧字形已由入口 prewarm_zoom_text 灌好
    if g.zoom_art.is_some() {
        draw_bg(art);
        crate::charpanel::draw_zoom_overlay(g);
        version_mark();
        return;
    }
    // v1.16.11 手札回味覆层：图鉴已遇行点按开启（仅 panel=5 图鉴页内可达）——模态
    // 独占输入照 zoom_art 同款早退；panel 已关的残留态不浮层（panel!=5 直接放行，
    // 残留由 Esc/C/npc_scroll_reset 三口清）
    if g.panel == 5 && g.ev_recall.is_some() {
        draw_bg(art);
        crate::gallery::draw_event_recall_overlay(g);
        version_mark();
        return;
    }
    // v1.10.2 卡顿修复（次要项③）：去每帧整 Life 深克隆（含 relics/talents/equipped
    // 三个 Vec 堆分配，先例 v1.10.0 draw_sim）——只读前段（顶栏/左栏渲染 + 所在行）
    // 借 life 不可变引用，借用随最后使用自然归还（NLL），后续行动栏 &mut 交互照旧
    let Some(l) = g.life.as_ref() else { return };
    draw_bg(art);
    // 剧情演出包⑫ 天气过渡氛围层（现实页消费口：手点/自动吐纳 act_cultivate →
    // advance_time weather_changed 触发）。矩形在 bg 之上、render_top/render_left
    // 等面板之前——面板后绘覆盖，仅背板缝隙透光，不扰任何 UI 像素；anim<=0 或晴
    // （表值 α=0）早退不落像素，稳态逐像素不变
    draw_weather_veil(g, 0.0, 0.0, V_W, V_H);
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
    // v1.16.10 新手一次性提示（sy= bit4）：修为首满指路一句（单口 sys_hint_break_ready，
    // 边沿帧才动 meta/落盘；l 借用至上一行已终结，此处 &mut 合法）
    g.sys_hint_break_ready();
    // ===== v1.16.0 道途一念：任务指引横带 (376,86,896,40) =====
    // v1.16.1 显性化（真机反馈「没看到引导」）：带高 34→40、底色提亮、目标/提示语
    // 升档 14/16→16/18，顶栏另加「任务」钮开道途指引面板（双入口兜底可发现性）。
    // 仅 panel==0 且无弹层时绘制——panel_card(140,60,1000,600) 右缘只盖到 x1140，
    // 横带右段 1272 遮不住，必须整条隐藏而非遮盖；ad/offline_card/rmodal/ending_show
    // /zoom_art 态由本函数头部早退链天然排除。提示语由 guide_index 纯推导（零存档键）。
    if g.panel == 0 && g.submenu.is_none() {
        // min 钳制：纪律 1（panic=abort）——下标访问一律带运行期上限，构造性 0..=8 之上再加保险
        let idx = crate::guide::guide_index(g).min(crate::guide::HINTS.len() as u8 - 1);
        // 事件帧预热（先例 ui.rs prewarm/show_banner「事件帧预热」）：仅提示下标变化帧
        // 一次性灌字形，字号只用既有离散档 16/18（UI_SIZES 第 2/3 档；按钮标经 button()
        // 无副标档 18px 绘制，变化帧绘制路径自栅格化一次，仍属事件级一次性，稳态零 churn）
        if g.guide_last != Some(idx) {
            let hint = &crate::guide::HINTS[idx as usize];
            prewarm_text(hint.text(), &[16, 18]);
            prewarm_text("目标", &[16]);
            if let Some(b) = hint.btn {
                prewarm_text(b, &[16, 18]);
            }
            g.guide_last = Some(idx);
        }
        // v1.16.16 主线章签（横带右段常驻小签）：chapter_current 单口复用——任何指引
        // 叶下都可见「主线在第几章」。事件帧预热照 guide_last 先例：仅当前章变化帧灌
        // 14px 两串（ch.num/ch.name 静态字面量）；布局右缘锚 1098（按钮 1112 前 14px），
        // num 起点 992 与最长提示叶（终局叶 guide.rs 27 全角+5 半角）留安全隙——
        // v1.16.17 复核：真字体 hmtx 静态度量 @18px 宽 522.4px → 右端 ~966px →
        // 隙 ~25.6px（≈26px 成立；tools/static_measure.py 可复算，缩叶改文案须同步；
        // macroquad advance 与 hmtx 同源无 kerning，运行期 measure 未另测）
        let curch = crate::game::Game::chapter_current(&g.meta);
        if g.guide_chap_last != Some(curch) {
            let chc = &crate::data::CHAPTERS[curch];
            prewarm_text(chc.num(), &[14]);
            prewarm_text(chc.name(), &[14]);
            g.guide_chap_last = Some(curch);
        }
        let hint = &crate::guide::HINTS[idx as usize];
        draw_rectangle(
            376.0,
            86.0,
            V_W - 384.0,
            40.0,
            Color::from_rgba(30, 22, 52, 225), // v1.16.1 提亮（24,17,40,205→），暗底上更醒目
        );
        draw_rectangle(376.0, 86.0, 4.0, 40.0, C_GOLD_D); // 左缘 4px 金竖条
        txt("目标", 392.0, 96.0, 16, C_GOLD_D);
        txt(hint.text(), 444.0, 92.0, 18, C_GOLD);
        // 右段章签：num+name 两静态分片直绘（禁 format!——横带稳态帧零分配红线；
        // 章名全表 4 字，992/1042 双锚定宽布局）
        let chc = &crate::data::CHAPTERS[curch];
        txt(chc.num(), 992.0, 96.0, 14, C_GOLD_D);
        txt(chc.name(), 1042.0, 96.0, 14, C_DIM);
        // 跳转按钮复用 draw::button()（继承去抖/悬停/描边语言）；纯提示叶不画
        if let Some(b) = hint.btn {
            if button(1112.0, 92.0, 144.0, 28.0, b, None, true) {
                crate::guide::exec_jump(g, hint.act);
            }
        }
    }
    // v1.16.1 顶栏四钮（任务/烙印/帮助/主页）：任务钮开道途指引面板（panel=6）——
    // 指引横带只在主界面常驻，玩家在其他页签不知道要做什么时由此兜底（真机反馈）
    if button(1018.0, 8.0, 62.0, 36.0, "任务", None, true) {
        g.panel = 6;
        g.play(crate::sounds::Which::Page);
        // 事件帧预热：面板静态文案+当前目标行+章节书全量（v1.16.9 封装单口，
        // 横带 Book 跳转共用 prewarm_guide_panel）
        prewarm_guide_panel(g);
    }
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
        // v1.16.16 帮助页新增行事件帧预热（照横带/任务钮事件帧预热先例；整页既有
        // 行为首开冷栅格化一次的既定行为，本行随新增先行灌字形防首次展开重传尖峰）
        prewarm_text(evtx(1717)/* 原文：【章节书】任务钮→章节书签：主线九章目标、进度与章间记事。 */, &[16]);
    }
    // v1.9.23：返回开始界面（存档已落盘，菜单「承接前缘」可恢复）
    if button(1222.0, 8.0, 58.0, 36.0, "主页", None, true) {
        g.back_to_menu();
    }
    // v1.16.0：日志行起点 88→124——顶部让位任务指引横带；
    // v1.16.1：124→132（横带加高 34→40）——行预算 (612-132)/28=17（原 17 不变），
    // 日志框底部仍留 ≥12px 空隙，日志 Vec 本就只显最新若干行
    let area_y = 132.0;
    let area_h = PANEL_B - 124.0;
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
    // 演出包⑰-2 底栏九钮入画 stagger：进行度 e=T−bar_anim（0→T），各钮相位
    // ((e−k×0.03)/0.03).clamp(0,1)——稳态 bar_anim==0 → e=T → 相位=9−k≥1.0 clamp
    // 恰 1.0，fade(c,1.0) 逐比特恒等（⑦ clamp 收敛红线同款 draw.rs:2736 先例）；
    // 命中不乘 α 照⑦「动画期照常可点，命中=所见」口径（button_a_accent 内部判定
    // 不走 α），0.27s 窗内透明但可点；α 走颜色不入字形缓存口零重栅格化，稳态快
    // 路径零分配。漏置触发口=该入口无动画优雅降级不破稳态（勘察风险栏 5）。
    let bar_e = crate::game::BAR_ANIM_T - g.bar_anim;
    let bar_a = |k: usize| bar_stagger_a(bar_e, k);
    let sub_open = g.submenu.is_some();
    let pts_ok = g.meta.sim_points >= g.sim_cost();
    // v1.7.0：山穷水尽时按钮改引广告（点了进 start_draft → start_ad）
    let stuck = g.is_stuck();
    let sim_sub: Option<String> = if stuck {
        Some("山穷水尽 · 观看广告".to_string())
    } else {
        Some(format!("推演一生 · {} 点", g.sim_cost()))
    };
    if button_a_accent(
        x0,
        by,
        bw,
        bh,
        "开始模拟",
        sim_sub.as_deref(),
        pts_ok || stuck,
        false,
        bar_a(0),
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
        if button_a_accent(
            x0 + (bw + gap),
            by,
            bw,
            bh,
            "修炼",
            Some("闭关 · 破境"),
            true,
            false,
            bar_a(1),
        ) {
            // v1.15.0 同上：弹层开着时点修炼 = 关弹层（同帧重开同层，观感不变）
            if sub_open {
                crate::menu::close_sub(g);
            }
            g.play(crate::sounds::Which::Click);
            crate::menu::open_sub(g, crate::menu::Sub::Cult);
        }
    }
    if button_a_accent(
        x0 + (bw + gap) * 2.0,
        by,
        bw,
        bh,
        "山河图",
        Some("世界地图 · M"),
        true,
        false,
        bar_a(2),
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
    if button_a_accent(
        x0 + (bw + gap) * 3.0,
        by,
        bw,
        bh,
        "坊市",
        Some("丹药 · 藏经 · 兑换"),
        true,
        false,
        bar_a(3),
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
    if button_a_accent(
        x0 + (bw + gap) * 4.0,
        by,
        bw,
        bh,
        "行囊",
        Some("丹药 · 法宝 · 行装"),
        true,
        false,
        bar_a(4),
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
    if button_a_accent(
        x0 + (bw + gap) * 5.0,
        by,
        bw,
        bh,
        "人物",
        Some("长老 · 商人 · 道侣"),
        true,
        false,
        bar_a(5),
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
    if button_a_accent(
        x0 + (bw + gap) * 6.0,
        by,
        bw,
        bh,
        "角色",
        Some("C 键查看"),
        true,
        false,
        bar_a(6),
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
            if button_a_accent(
                x0 + (bw + gap) * 7.0,
                by,
                bw,
                bh,
                "寻仇",
                Some(&format!("{:.0}% · {}", p * 100.0, neme.name)),
                true,
                false,
                bar_a(7),
            ) {
                // v1.15.0 同底栏语义：弹层开着时点寻仇 = 关弹层并执行
                if sub_open {
                    crate::menu::close_sub(g);
                }
                g.play(crate::sounds::Which::Click);
                g.act_revenge();
            }
        } else if button_a_accent(
            x0 + (bw + gap) * 7.0,
            by,
            bw,
            bh,
            "寻仇",
            Some(g.revenge_disabled_text()),
            false,
            false,
            bar_a(7),
        ) {
        }
    } else if button_a_accent(
        x0 + (bw + gap) * 7.0,
        by,
        bw,
        bh,
        "宿敌已清",
        Some("静待新敌"),
        false,
        false,
        bar_a(7),
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
    if button_a_accent(
        x0 + (bw + gap) * 8.0,
        by,
        bw,
        bh,
        "周天",
        Some(auto_sub),
        true,
        false,
        bar_a(8),
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
        // v1.16.8 系统初见闩（sy= bit0）：周天首开一句戏文（一次性边沿，additive 闩
        // 推导不出「从未开过」；log/toast 走既有渲染口径，讨牒 log 先例不另预热）
        if !auto_on && g.meta.sys_seen & 0b0001 == 0 {
            g.meta.sys_seen |= 0b0001;
            g.log(
                evtx(2525)/* 原文：周天既开，吐纳自转——从此闭关不必亲守，岁月自替你修行。 */,
                C_GOLD,
            );
            g.toast("周天吐纳 · 初启", C_GOLD);
        }
        g.sync_save();
    }
    if auto_on {
        // 演出包⑰-2 勿半改：外置下划线与槽 8 钮体同批乘 stagger α（显影同步）
        draw_rectangle(
            x0 + (bw + gap) * 8.0,
            by + bh - 3.0,
            bw,
            3.0,
            fade(C_GOLD, bar_a(8)),
        );
    }

    // 面板
    match g.panel {
        1 => draw_shop(g),
        2 => draw_bag(g),
        3 => draw_upgrade_panel(g),
        4 => draw_help_panel(g),
        5 => crate::charpanel::draw_char_panel(g),
        6 => draw_guide_panel(g), // v1.16.1：道途指引（顶栏「任务」钮）
        _ => {}
    }
    // v1.6.0 B：修炼弹层（底栏之后绘制，盖住中部区域）
    if g.submenu.is_some() {
        crate::menu::draw_sub(g);
    }
    // v1.16.4：暂避收起态补画「待抉择」徽章（非收起态上方早退链已独占渲染卡面）
    if g.rmodal.is_some() {
        draw_modal_card(g);
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
    // v1.16.45 bug② 根因修复：删除 v1.0.0 起遗留的第二份顶栏标签（txt_r 烙印/
    // 帮助 16px 暗金右对齐两行 + draw_rectangle_lines 1086/1154 两行 legacy 1.5px
    // 钮框）——与下方 button() 自绘 2px 金框+白 18px 居中标签同位叠画，鬼影字
    //（16/18 两档 measure 差错位凸出的「印/助」右部框形部件）透过 α235 底框只剩
    // ~7.8% 即玩家所见「字旁浅浅方框」。四行无任何逻辑消费，删除零像素差（button
    // 本体四调用保留不动；静态钉 ghost_label_removed_pins 反向钉四真钮防误删）。
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
    // 演出包㉒ 周天吐纳·气脉呼吸：qi_breath>0 期间修为条外扩 2px C_CYAN 描边
    // halo（α=0.12×brk_veil_a 共用三角包络单源勿双写）。吐纳每载重置=呼吸与周天
    // 1.6s 同频，停手/auto 停后 1.6s 内收敛 0；稳态 qi_breath==0 → qb_a=0.0 →
    // 早退不落像素逐比特恒等（render_left &Game 只读零借用冲突——勘察三落点）。
    // 与⑫错开：彼是全屏天气叠色 0.8s、此是左栏修为条局地 1.6s 呼吸，区域/周期/色三异
    let qb_a = 0.12 * brk_veil_a((g.qi_breath / crate::game::QI_BREATH_T).clamp(0.0, 1.0));
    if qb_a > 0.0 {
        draw_rectangle_lines(26.0, 272.0, LEFT_W - 36.0, 24.0, 2.0, fade(C_CYAN, qb_a));
    }
    // UX 文案沉浸（勘察六③真出戏口，v1.16.50）：修为满（qi 超蓄 frac≥1.0）时
    // 以「修为圆满」替换「修为 {}/{}（{}%）」——百分数直出 143% 类数字出戏；
    // 圆满语汇照既有事件文案「你修为圆满」款；「修/为/圆/满」四字均入 PANEL_WARM_
    // CHARS 常驻档（ui.rs 实测）零冷栅格化；frac<1.0 走原查表逐像素不变；与
    // charpanel 口同批同语义（满则圆满）防半改
    let qlabel = if frac >= 1.0 {
        "修为圆满".to_string()
    } else {
        evf(2615, &[&(fmt_num(l.qi)) as &dyn core::fmt::Display, &(fmt_num(need)) as &dyn core::fmt::Display, &(crate::data::Fmt0((frac * 100.0) as f64)) as &dyn core::fmt::Display])/* 原文：修为 {}/{}（{}%） */
    };
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
    txt(TECHS[l.tech].name(), 100.0, y - 21.0, 14, C_GOLD);
    y = row("法宝", y);
    // v1.5.0：只留计数摘要（详情见角色面板·法宝页）
    if l.relics.is_empty() {
        txt("无", 100.0, y - 21.0, 14, C_DIM);
    } else {
        txt(
            &evf(2526, &[&(l.equipped.len().min(RELIC_EQUIP_MAX)) as &dyn core::fmt::Display, &(RELIC_EQUIP_MAX) as &dyn core::fmt::Display, &(l.relics.len()) as &dyn core::fmt::Display])/* 原文：已装 {}/{} · 包内 {} */,
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
                &evf(2527, &[&(REALMS[n.realm.min(14)].name) as &dyn core::fmt::Display])/* 原文：宿敌亦在修行，寻仇宜早（已{}境）！ */,
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
        &evf(2528, &[&(g.draft.len()) as &dyn core::fmt::Display, &(max) as &dyn core::fmt::Display])/* 原文：天赍 {} 选 {}——仅此一梦生效 */,
        V_W * 0.5,
        108.0, 16,
        C_DIM,
    );
    // 已选
    let names: Vec<&str> = g.picked.iter().map(|&t| TALENTS[t].name()).collect();
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
            &evf(2529, &[&(note) as &dyn core::fmt::Display])/* 原文：朱批随身：「{}」——开卷自现，本稿气运+2 */,
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
    // 演出包㉓ 天赍落笔·镜纹荡漾：卡片区金纹矩形环脉冲（共用 brk_veil_a 三角
    // 包络；draft_pact==0 早退不落像素逐比特恒等——回镜页兜底口，主可见面在
    // draw_sim 首段金环；勘察三双挂建议）。与⑨..㉒错开：Draft 页此前零演出包
    // 触达，触发（launch_sim 落笔）/区域（卡片区环）/语义（天赍落笔）三异
    let dp_a = 0.12 * brk_veil_a((g.draft_pact / crate::game::DRAFT_PACT_T).clamp(0.0, 1.0));
    if dp_a > 0.0 {
        draw_rectangle_lines(
            x0 - 14.0,
            156.0,
            total_w + 28.0,
            ch + 28.0,
            2.0,
            fade(C_GOLD, dp_a),
        );
    }
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
        txt_c(card.name(), x + cw * 0.5, y + 84.0, 24, C_TEXT);
        wrap_draw(card.desc(), x + 24.0, y + 120.0, 16, cw - 48.0, 28.0, C_DIM);
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
        if hot && is_mouse_button_pressed(MouseButton::Left) && click_debounced(mx, my) {
            // v1.16.3 接入同点去抖：天赋卡是开合型交互却用裸 pressed——真机一击
            // 双派发（触摸桥+compat 两路 pressed 边沿落不同帧）会把「选中→取消」
            // 一拍吃掉，玩家体感正是「点了天赋就无法改变选择」（v1.15.0 同根因
            // 先例：button() 全量接去抖，charpanel 裸判定当时无开合钮不接）
            g.toggle_pick(i);
        }
    }
    let ready = !g.picked.is_empty();
    // v1.16.49 按钮花样 A（勘察六提名·墨蘸好点亮）：落笔钮 ready 门控 accent
    //——选定第一张天赋瞬间描边 C_GOLD_D→C_GOLD；未选态 accent=false 与旧实现
    // 逐比特一致（button→button_accent 委托先例照 :107-109 款，稳态零像素变更）
    if button_accent(
        V_W * 0.5 - 140.0,
        540.0,
        280.0,
        52.0,
        "落 笔",
        Some("开始此世模拟"),
        ready,
        ready,
    ) {
        g.play(crate::sounds::Which::Ding);
        g.launch_sim();
    }
    version_mark();
}

// ================= 模拟播放 =================
pub fn draw_sim(g: &mut Game, art: &Art) {
    // v1.16.4 立绘鉴赏覆层（道侣随行小像点按开启）：模态独占输入，照 draw_reality
    // 早退先例——下层不绘不响应；事件卡/模拟冻结不受影响（modal 仍在，合页即回）
    if g.zoom_art.is_some() {
        draw_bg(art);
        crate::charpanel::draw_zoom_overlay(g);
        version_mark();
        return;
    }
    // v1.9.25 卡顿修复（根因2）：去每帧整 Life 深克隆——只读绘制段借 sim 不可变引用，
    // 速度/退出/事件弹窗等 &mut 交互统一挪到借还之后；按钮区（y≤47）与日志区（y≥68）
    // 像素无重叠，绘制次序对调不影响画面
    let Some(s) = g.sim.as_ref() else { return };
    let tribbing = s.trib.is_some();
    draw_bg(art);
    // v1.16.49 ㉓遮蔽修复（勘察六 finding① 根因闭环）：全屏金环原绘于此处
    //（draw_bg 之后），被下方顶栏 (0,0,V_W,52) 与日志面板 (8,68,1264,612) 依
    // 绘制序覆盖——环仅底缘全线+两侧 76px 短须可见，与原注释「主可见面」意图
    // 不符；本批整块挪至日志面板绘制之后、问笔演出层之前（仍在 draw_modal_card
    // 之下，见下 :log 段尾）。稳态 draft_pact==0 早退逐比特不变，唯脉冲 0.9s 内
    // 环线全程可见；v11653「α 双口恰 2」钉不受挪位影响（needle 计数与位置无关），
    // 位点序防回挪钉见 v11656_veil_sim_ring_unoccluded_pin
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
        // v1.16.21 日志流左缘色标：非灰行加 2.5px 色条——金=大事记、红=受挫、
        // 琥珀=对价账、青=进益一眼可扫（灰=常事不打标）。纯矩形零分配，
        // ≤24 矩形/帧（可视行数上限）
        if ll.color != C_DIM {
            draw_rectangle(
                11.0,
                area_y + 13.0 + i as f32 * line_h,
                2.5,
                line_h - 10.0,
                ll.color,
            );
        }
        txt(
            &ll.text,
            24.0,
            area_y + 10.0 + i as f32 * line_h, 16,
            ll.color,
        );
    }
    // 演出包㉓ 天赍落笔·镜纹荡漾：落笔入画金环（launch_sim 落笔帧即切本场景，
    // 本口为主可见面；0.9s 全屏内缩金环一次性脉冲，draft_pact==0 早退逐比特恒
    // 等——勘察三双挂建议的主口）。与⑨..㉒错开：触发（落笔）/区域（全屏环）/
    // 语义（天赍落笔）三异，与⑲ brk 门控金辉（彼卡开门控常驻）全异。
    // v1.16.49 ㉓遮蔽修复：本块自 draw_bg 之后原位挪至日志面板绘制完之后——
    // 环线不再被顶栏/日志面板覆盖，脉冲 0.9s 内全环可见；问笔演出层（tribbing
    // 暗角）在本口之后绘仍在其上，与原叠序一致（落笔帧 trib 必空，两态无重叠
    // 场景；挪位理由与逐比特论证见 draw_bg 后的批注）
    let dp_a = 0.12 * brk_veil_a((g.draft_pact / crate::game::DRAFT_PACT_T).clamp(0.0, 1.0));
    if dp_a > 0.0 {
        draw_rectangle_lines(16.0, 16.0, V_W - 32.0, V_H - 32.0, 2.0, fade(C_GOLD, dp_a));
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
    // v1.16.4 道侣随行小像：结识道侣后顶栏右侧常驻小卡（头像+名+情谊）——
    // 真机反馈「结识道侣之后有关道侣的信息旁看不到人像，立绘作用没体现」。
    // 点按整卡 → zoom_art 全屏鉴赏（本函数头部早退链独占）。头像取立绘顶部
    // 35% 方形头裁剪（人物页头像条同语言）。sim 视图本就逐帧 format!（年龄/
    // 境界行），本芯片的动态情谊行不新增分配纪律负担。
    if let Some(ci) = g
        .sim
        .as_ref()
        .and_then(|x| x.companion.as_deref())
        .and_then(|cn| COMPANION_NAMES.iter().position(|&n| n == cn))
    {
        let (cx, cy) = (1000.0, 52.0);
        draw_rectangle(cx, cy, 268.0, 40.0, Color::from_rgba(26, 18, 42, 220));
        draw_rectangle_lines(cx, cy, 268.0, 40.0, 1.5, C_PURPLE);
        if let Some(t) = crate::art::art().comp_tex(ci) {
            let side = (t.height().max(1.0) * 0.35).min(t.width());
            draw_texture_ex(
                &t,
                cx + 4.0,
                cy + 4.0,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new((t.width() - side) * 0.5, 0.0, side, side)),
                    dest_size: Some(vec2(32.0, 32.0)),
                    ..Default::default()
                },
            );
        }
        txt(COMPANION_NAMES[ci], cx + 42.0, cy + 3.0, 16, C_PURPLE);
        let fav = g.sim.as_ref().map_or(0, |x| x.comp_fav);
        txt(&evf(2530, &[&(fav) as &dyn core::fmt::Display])/* 原文：情谊 {}/100 · 点看立绘 */, cx + 42.0, cy + 22.0, 14, C_DIM);
        let (mx, my) = crate::cam::mouse();
        if is_mouse_button_pressed(MouseButton::Left)
            && mx >= cx
            && mx <= cx + 268.0
            && my >= cy
            && my <= cy + 40.0
        {
            let z = crate::game::ZoomArt::Comp(ci);
            crate::charpanel::prewarm_zoom_text(g, &z);
            g.zoom_art = Some(z);
            // 剧情演出包⑬ 入画动效计时（四开启口同批同改勿半改，照 charpanel 口同款）
            g.zoom_anim = 0.22;
            g.play(crate::sounds::Which::Click);
        }
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
    // v1.16.4 暂避收起态（真机反馈「卡片一直挡着对话，想先看日志再选」）：
    // 只画顶栏下居中小徽章，点按重回卡面抉择。收起期间模拟冻结不动（update_sim
    // 对 modal.is_some() 早退——modal 仍在，选项与结算零失真），日志随便翻。
    if g.modal_min {
        if button(
            V_W * 0.5 - 120.0,
            54.0,
            240.0,
            30.0,
            "待抉择 · 展开卡面",
            None,
            true,
        ) {
            g.modal_min = false;
            g.play(crate::sounds::Which::Page);
        }
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
                            // v1.16.34 单1 β刀（--nize=arms）：终战副标四臂收编 textpack——
                            // i.min 钳制落 _ 臂槽，逐值等价（.to_string() 调用位不变）
                            const FINALE_SUB_NS: [u16; 4] = [2628, 2629, 2630, 2631];
                            evtx(FINALE_SUB_NS[i.min(FINALE_SUB_NS.len() - 1)] as usize)
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

    // 剧情演出包①：入场淡入+轻滑（modal_anim 0.18→0 剩余秒，各开启口置值、主循环
    // 递减；稳态 modal_anim==0 → k=0，下方全部相位恒等旧版式逐像素不变）。相位反推
    // 照 banner 先例（draw.rs k=1-t/T）；动画期命中几何随卡体同步偏移 dy，0.18s 后
    // 精确复位——动画期照常可点，命中=所见。淡入通道收编于剧情演出包①/③（终局页
    // gain_a 同款），新出场点默认复用 npc_portrait_box_a 的 alpha 参数——勿另起炉灶
    //（v1.16.35 单4 勘察备案：结算/回味等无出场计时状态屏不加淡入，稳态逐像素红线；
    // v1.16.41 翻转备案：鉴赏覆层原属该域，本批照⑫新增计时器先例补 zoom_anim
    // 入画动效——立绘主时刻+开启帧已有 prewarm_zoom_text 事件口，翻转理由见
    // game.rs zoom_anim 字段批注；结算/回味等其余状态屏维持不加）
    let k = (g.modal_anim / 0.18).clamp(0.0, 1.0); // 1=开启帧 → 0=稳态
    let p = 1.0 - k; // 入场进度 0→1
    let dim = 0.5 * (0.3 + 0.7 * p); // 暗幕先行：开启帧即压暗 0.15，随进度至 0.5
    let dy = 14.0 * k; // 卡体轻滑 y+14→0
    let card_a = p; // 卡体 alpha
    // ③ 立绘滞后 0.05s 分层入画；⑥ 正文滞后 0.08s 浮现（纯算术相位，零分配零新字形）
    let art_a = (((0.18 - g.modal_anim) - 0.05) / 0.13).clamp(0.0, 1.0);
    let text_a = (((0.18 - g.modal_anim) - 0.08) / 0.10).clamp(0.0, 1.0);
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, dim));
    // v1.16.3 邂逅版式：卡带 actor（模拟「同页天成」/现实好感事件）且有真立绘时
    // 改宽卡 800×520——立绘收左 contain 220×420（复用 npc_portrait_box 人物页同
    // 语言），正文/抉择钮右移。真机反馈「模拟中结识人物看不到画像，立绘作用没体现」。
    // Finale/Path/破境等无 actor 卡维持 600×400 原版式逐像素不变。
    // v1.16.9 复审修（立绘外置后）②：纹理未到/缺失走像素/剪影兜底（gallery.rs 同款
    // None 语义——NPC 像素 name_seed / 道侣书卷剪影），不再落云纹占位或旧版式；
    // AI 图到后 Some 短路换图（同槽一次闪变，每会话每槽一次）
    let portrait: Option<Texture2D> = {
        let card = g.rmodal.as_ref().or(g.modal.as_ref()).unwrap();
        match card.actor {
            Some(crate::game::CardActor::Comp(i)) => crate::art::art()
                .comp_tex(i.min(9))
                .or_else(crate::art::comp_silhouette_tex),
            Some(crate::game::CardActor::Npc(i)) => crate::art::art()
                .npc_tex(i.min(3))
                .or_else(|| crate::art::npc_pixel_tex(i.min(3))),
            None => None,
        }
    };
    // v1.16.7 稀有/链段标记采样（纯会话态随卡生灭；rare=EVENTS w≤RARE_W、
    // stage=链段进度，build_event 尾统一置位）
    let (card_rare, card_stage, card_cat, card_brk) = {
        let card = g.rmodal.as_ref().or(g.modal.as_ref()).unwrap();
        (card.rare, card.stage, card.cat, card.brk)
    };
    // v1.16.46 演出包⑳ 结缘仪式门控采样（随卡生灭会话态照 rare/stage 采样先例）：
    // opts 静态扫描 special==17（opts≤4 零分配）——构建口唯一精确记号（模拟 28
    // 臂 game.rs:4404「认下这页」/现实 rencounter 认下臂 game.rs:14180；cfg(test)
    // 夹具不算），勿放宽 actor==Comp——普通邂逅/好感卡会误入
    let card_rite = g
        .rmodal
        .as_ref()
        .or(g.modal.as_ref())
        .unwrap()
        .opts
        .iter()
        .any(|o| o.1.special == 17);
    // v1.16.46 演出包⑲ 破境金辉氛围层：暗幕上卡体下全屏金辉（brk 门控=天劫/破境
    // 两路统一置位 game.rs:3420「天劫/破境两路同点」）；brk_veil_a(k) 三角包络
    // 峰值 α=0.08，稳态 modal_anim==0 → k=0 → α=0 → 早退不落像素逐比特恒等；
    // 纯矩形零分配零新字形，颜色走 fade 通道不入字形缓存 key
    let brk_a = 0.08 * brk_veil_a(k);
    if brk_a > 0.0 {
        draw_rectangle(0.0, 0.0, V_W, V_H, fade(C_GOLD, brk_a));
    }
    // v1.16.21 场景要素行采样（&str 借用横跨绘制段——与 modal_desc 同为只读，
    // 段内唯一 &mut 是段尾 modal_min，借用纪律不破）
    let scene: Option<&str> = g
        .rmodal
        .as_ref()
        .or(g.modal.as_ref())
        .unwrap()
        .scene
        .as_deref();
    // v1.16.9 复审修（立绘外置后）①：meeting 按 card.actor 锁定版式——开卡帧即定
    // 800×520/600×400 几何。wasm 立绘外置后 XHR 拉图期 get() 返 None，若按原
    // portrait.is_some() 判定，卡先走旧版式、图到位（1~3s）切邂逅版式 → 标题/正文/
    // 按钮几何中途跳动，触屏玩家可能点在移动前的按钮位上。latched 后图后到只换图
    // 不改几何（actor 有而纹理未到时立绘盒兜底链见上）；actor=None（Finale/Path/
    // 普通事件卡）恒旧版式，语义不变
    let meeting = g
        .rmodal
        .as_ref()
        .or(g.modal.as_ref())
        .unwrap()
        .actor
        .is_some();
    // v1.16.7 复审①：tall/cy 判定上移到暂避钮命中采样之前——hide_hit 与绘制
    // 必须同源（原 hit 固定采样 (820,174+dy) 而 tall 卡按钮画在 cy+14=154，点可见
    // 钮无效、点下方空白反而触发暂避，仅 tall 版式错位 20px）。kind/行数只读
    // 采样在 modal_desc 借用前完成，与分支内原式逐值一致
    let kind_is_npc = g
        .rmodal
        .as_ref()
        .or(g.modal.as_ref())
        .is_some_and(|c| c.kind == crate::game::CardKind::Npc);
    let lines = crate::ui::wrap_lines(text, 16, 520.0);
    let segs = text.matches('\n').count() + 1;
    let tall = !meeting && kind_is_npc && lines + segs - 1 > 3; // 加高容量 ≤4 行（v1.16.46 勘四必修①改准：4*28+段距8=120px 恰 tall max_h——原 132px 注与 112px 实参自相矛盾已修正）
    let cy = if tall { 140.0 } else { 160.0 };
    // v1.16.4 暂避钮命中先采样：title/descs 只读借用横跨绘制段直到选项循环结束，
    // g.modal_min 的 &mut 移到段尾执行，免 E0502（钮面在两版式分支内照常绘出）
    let (mxp, myp) = crate::cam::mouse();
    let (hx, hy) = if meeting {
        (920.0, 122.0 + dy)
    } else {
        (820.0, cy + 14.0 + dy) // 与下方 button(820, cy+14+dy) 同源（复审①）
    };
    let hide_hit = is_mouse_button_pressed(MouseButton::Left)
        && mxp >= hx
        && mxp <= hx + 110.0
        && myp >= hy
        && myp <= hy + 26.0;
    // v1.16.38 单3 剧情演出包⑨：暂避钮两版式改挂卡体入场 α（下两处
    // button_a_accent(.., card_a)）——命中采样在 α 之前且不乘 α：开帧钮不可见
    // 但可点，与 v1.16.37 选项钮 ⑦「动画期照常可点，命中=所见」同口径
    //（draw.rs 选项钮段注；几何随 dy 同步，稳态 card_a==1 逐像素不变）
    if meeting {
        panel_card_a(240.0, 100.0 + dy, 800.0, 520.0, card_a);
        // v1.16.7 稀有/链段卡描边：叠于 panel_card_a 金描边之上（2.5 同位覆盖）——
        // 链段卡青、稀有卡紫、v1.16.21 破境抉择卡琥珀（新色位不占既有语义），
        // 链首段 50 两者兼有时链段优先；稳态随卡面全显
        let edge = if card_stage.is_some() {
            Some(C_CYAN)
        } else if card_rare {
            Some(C_PURPLE)
        } else if card_brk {
            Some(C_AMBER)
        } else {
            None
        };
        if let Some(ec) = edge {
            draw_rectangle_lines(240.0, 100.0 + dy, 800.0, 520.0, 2.5, fade(ec, card_a));
        }
        txt_c(title, V_W * 0.5, 128.0 + dy, 24, fade(C_GOLD, card_a));
        // v1.16.7 链段阶段徽标（复用品级徽标 68×26 半透明底+16px 字语言）：标题
        // 右侧 x=736..804，与暂避钮（920,122）不交
        // v1.16.38 单3 剧情演出包⑩：链段/类目徽标+场景要素行挂 0.05s 立绘滞后
        // 通道 art_a（徽标比标题迟 ~50ms 浮现，稳态 art_a==1 逐像素不变；纯只读
        // 共享 art_a 与 ⑧ 沉降无写冲突；徽标/场景行零命中几何纯视觉；本批注与
        // 下方类目/场景行及普通版式对称三处同批同改，勿半改）
        if let Some((seg, total)) = card_stage {
            let (bx, by) = (736.0, 122.0 + dy);
            draw_rectangle(bx, by, 68.0, 26.0, fade(Color::from_rgba(130, 220, 230, 60), art_a));
            draw_rectangle_lines(bx, by, 68.0, 26.0, 1.0, fade(C_CYAN, art_a));
            txt_c(
                stage_label(seg, total), // 复审④：&'static str 零分配
                bx + 34.0,
                by + 4.0,
                16,
                fade(C_CYAN, art_a),
            );
        }
        // v1.16.21 类目徽标 chip：标题左侧镜像 stage 徽标位（736..804 关于版式中轴
        // 640 → 476..544；事件标题 ≤6 字（568..712）恒不相交）。底色取徽标色 α0.24
        if let Some((cw, cc)) = card_cat {
            let (bx, by) = (476.0, 122.0 + dy);
            draw_rectangle(bx, by, 68.0, 26.0, fade(Color::new(cc.r, cc.g, cc.b, 0.24), art_a));
            draw_rectangle_lines(bx, by, 68.0, 26.0, 1.0, fade(cc, art_a));
            txt_c(cw, bx + 34.0, by + 4.0, 16, fade(cc, art_a));
        }
        draw_rectangle(
            270.0,
            148.0 + dy,
            740.0,
            1.5,
            Color::new(
                96.0 / 255.0,
                76.0 / 255.0,
                130.0 / 255.0,
                160.0 / 255.0 * card_a,
            ),
        );
        title_ornament(V_W * 0.5, 148.0 + dy, card_a);
        // v1.16.21 场景要素行：分隔线右端「青云宗 · 雨」@14 C_DIM（152..166，
        // 正文列 170 起/暂避钮底 148——双不交，零版式位移）
        if let Some(sc) = scene {
            txt_r(sc, 1010.0, 151.0 + dy, 14, fade(C_DIM, art_a));
        }
        // 立绘盒 (270,160) 220×420、底边锚 580（box_h=bottom_off=420：顶 = 580−dh ≥ 160，
        // 恒落 npc_clip_band 116..616 视口内不裁）；v1.16.9 复审修后 AI 图未到/缺失
        // 时 portrait 走像素/剪影兜底恒 Some（actor 有恒邂逅版式，见 meeting latched）
        // v1.16.37 单3 剧情演出包⑧：入画沉降——立绘在 0.05s 滞后淡入通道上叠
        // 二次缓动 y+3px×(1−art_a)²（淡入起 +3px 随进度归零，纯算术零分配）。
        // 收敛数学：稳态 modal_anim==0 → art_a==1 → 沉降恰 0，稳态逐像素不变
        //（draw.rs:1821 纪律）；动画期极值 +3px 叠加 dy 滑入（≤14px）仍在
        // npc_clip_band 116..616 窗内不触裁剪（槽底 583<616）；结算屏/回味/
        // 终局页等备案域不入（art_a 仅本邂逅卡通道，draw.rs:1825 备案原样）。
        let art_sink = 3.0 * (1.0 - art_a) * (1.0 - art_a);
        crate::charpanel::npc_portrait_box_a(
            portrait.as_ref(),
            &crate::art::art().icon_yun,
            270.0,
            160.0 + dy + art_sink,
            220.0,
            420.0,
            420.0,
            art_a,
        );
        // v1.16.46 演出包⑳ 结缘仪式金尘：special17 门控（card_rite 采样段）——
        // 8 粒金尘纯矩形错峰（bond_mote_a 包络派生 α 与上升位移，盒内 220×420
        // 全域有界）+盒缘金描边呼吸（⑤ title_ornament 三层矩形同语言）；叠加层
        // 画在 npc_portrait_box_a 之后不改其几何（v11648_bond_rite_tests 零位移
        // 钉）；α 乘 art_a 与立绘同显隐（稳态 modal_anim==0 → k=0 → 全 0 整段
        // 早退不落像素逐比特恒等）；勿动 npc_portrait_box_a 内部（npc_clip_band
        // charpanel.rs:229-235 红线）
        if card_rite && k > 0.0 && art_a > 0.0 {
            for mote_k in 0..8usize {
                let env = bond_mote_a(k, mote_k);
                if env <= 0.0 {
                    continue;
                }
                let mx = 270.0 + 26.0 + (mote_k % 4) as f32 * 56.0;
                let my = 160.0 + dy + art_sink + 96.0 + (mote_k / 4) as f32 * 130.0
                    - 24.0 * env;
                let ms = if mote_k % 3 == 0 { 3.0 } else { 2.0 };
                draw_rectangle(mx, my, ms, ms, fade(C_GOLD, 0.45 * env * art_a));
            }
            draw_rectangle_lines(
                270.0 - 3.0,
                160.0 + dy + art_sink - 3.0,
                220.0 + 6.0,
                420.0 + 6.0,
                1.5,
                fade(C_GOLD, 0.3 * brk_veil_a(k) * art_a),
            );
        }
        draw_event_text(
            text,
            530.0,
            170.0 + dy,
            16,
            480.0,
            28.0,
            C_TEXT,
            0.18 - g.modal_anim,
            160.0, // v1.16.9 文本区 170..330 预算（5 行×28+段距），超限截断不压钮区
        );
        // v1.16.38 单3 剧情演出包⑨：暂避钮挂卡体入场 α——分层入画收尾（卡体→
        // 徽标/立绘→正文→选项钮错峰→暂避钮同批全显）；accent=false 非推荐位
        // 语义不变，稳态 card_a==1 逐像素不变
        button_a_accent(920.0, 122.0 + dy, 110.0, 26.0, "暂 避", None, true, false, card_a);
    } else {
        // v1.16.7 多段正文承载：普通 600×400 文本区（224..316）仅 3 行；模拟事件卡
        //（kind=Npc）正文 >3 行走 600×440 加高变体（卡顶 140/文本 204/钮 336，
        // 3 钮末底 546 < 卡底 580 合规）——Finale/Path 恒走原几何逐像素不变
        //（终战特判 step(58,58) 不与此相交；kind/lines/segs/tall/cy 已上移至
        // 命中采样段，此处直接消费）
        panel_card_a(
            340.0,
            cy + dy,
            600.0,
            if tall { 440.0 } else { 400.0 },
            card_a,
        );
        let edge = if card_stage.is_some() {
            Some(C_CYAN)
        } else if card_rare {
            Some(C_PURPLE)
        } else if card_brk {
            Some(C_AMBER)
        } else {
            None
        };
        if let Some(ec) = edge {
            draw_rectangle_lines(340.0, cy + dy, 600.0, if tall { 440.0 } else { 400.0 }, 2.5, fade(ec, card_a));
        }
        txt_c(title, V_W * 0.5, cy + 28.0 + dy, 24, fade(C_GOLD, card_a));
        // v1.16.7 链段阶段徽标：标题右侧（x=736..804），与暂避钮（820, cy+14）不交
        // v1.16.38 单3 剧情演出包⑩：普通版式链段/类目徽标+场景行同款挂 art_a
        //（照 meeting 版式 ⑩ 注，两版式对称同改勿半改）
        if let Some((seg, total)) = card_stage {
            let (bx, by) = (736.0, cy + 13.0 + dy);
            draw_rectangle(bx, by, 68.0, 26.0, fade(Color::from_rgba(130, 220, 230, 60), art_a));
            draw_rectangle_lines(bx, by, 68.0, 26.0, 1.0, fade(C_CYAN, art_a));
            txt_c(
                stage_label(seg, total), // 复审④：&'static str 零分配
                bx + 34.0,
                by + 4.0,
                16,
                fade(C_CYAN, art_a),
            );
        }
        // v1.16.21 类目徽标 chip：标题左侧镜像 stage 徽标位（736..804 关于版式中轴
        // 640 → 476..544；事件标题 ≤6 字（568..712）恒不相交）。底色取徽标色 α0.24
        if let Some((cw, cc)) = card_cat {
            let (bx, by) = (476.0, cy + 13.0 + dy);
            draw_rectangle(bx, by, 68.0, 26.0, fade(Color::new(cc.r, cc.g, cc.b, 0.24), art_a));
            draw_rectangle_lines(bx, by, 68.0, 26.0, 1.0, fade(cc, art_a));
            txt_c(cw, bx + 34.0, by + 4.0, 16, fade(cc, art_a));
        }
        draw_rectangle(
            380.0,
            cy + 48.0 + dy,
            520.0,
            1.5,
            Color::new(
                96.0 / 255.0,
                76.0 / 255.0,
                130.0 / 255.0,
                160.0 / 255.0 * card_a,
            ),
        );
        title_ornament(V_W * 0.5, cy + 48.0 + dy, card_a);
        // v1.16.21 场景要素行：分隔线右端「青云宗 · 雨」@14 C_DIM（cy+51..cy+65，
        // 正文列 cy+64 起/暂避钮底 cy+40——双不交，零版式位移）
        if let Some(sc) = scene {
            txt_r(sc, 900.0, cy + 51.0 + dy, 14, fade(C_DIM, art_a));
        }
        if kind_is_npc {
            // 分段淡入正文（复审②：秒标度 t，稳态全段恒 1 与旧单次 wrap_draw
            // 逐像素一致）；v1.16.9 文本区像素预算上移（cy+64..316 钮区）：
            // tall 卡 112px（4 行）、普通卡 92px（3 行），超限截断不压钮区
            // v1.16.46 勘四必修①：tall max_h 112→120——两段式 2+2 行 used_h=
            // 56+8+56=120px 恰满（v1.16.45 段十八 371/373 与段十九 374 第二段
            // 在 112px 下被先判后画整段截断，勘察四静态算术+fontTools hmtx
            // advance=1.0em 实锤）；204+120=324 仍在 tall 钮区 336 上方留 12px，
            // 普通卡 92px 不动逐像素不变
            draw_event_text(
                text,
                380.0,
                cy + 64.0 + dy,
                16,
                520.0,
                28.0,
                C_TEXT,
                0.18 - g.modal_anim,
                if tall { 120.0 } else { 92.0 },
            );
        } else {
            // Finale/Path 卡维持 v1.16.6 动效原样（单次 wrap_draw，逐像素不变）
            wrap_draw(text, 380.0, 224.0 + dy, 16, 520.0, 28.0, fade(C_TEXT, text_a));
        }
        // v1.16.38 单3 剧情演出包⑨：普通版式暂避钮同款挂卡体入场 α（两版式
        // 对称同改，照 meeting 版式注）
        button_a_accent(820.0, cy + 14.0 + dy, 110.0, 26.0, "暂 避", None, true, false, card_a);
    }
    // 命中先收集、借还后再分派（&mut）——免整卡克隆
    let mut picked: Option<usize> = None;

    // v1.12.1 终战卡四钮几何特判：默认步距 74/高 62 时 3 钮末底 526 合规、4 钮末底
    // 600 溢出 panel_card 底 560——Finale 收紧为步距 58/高 58，4 钮末底 548 合规
    // （Finale/Path 无 actor 恒走旧版式，特判不与邂逅版式相交）
    let fin = g
        .rmodal
        .as_ref()
        .or(g.modal.as_ref())
        .is_some_and(|c| c.kind == crate::game::CardKind::Finale);
    let (step, bh) = if fin { (58.0, 58.0) } else { (74.0, 62.0) };
    // 剧情演出包④：主选项金描边（终战推荐位=下标 1「《留白》·撕稿」副标「推荐」、
    // 道途首钮=下标 0「剑修」——推荐/默认路径的可扫视性提升）
    let kind_acc = g.rmodal.as_ref().or(g.modal.as_ref()).map(|c| c.kind);

    for (i, (label, _)) in g
        .rmodal
        .as_ref()
        .or(g.modal.as_ref())
        .unwrap()
        .opts
        .iter()
        .enumerate()
    {
        let (bx, by, bw, bhh) = if meeting {
            // 邂逅版式：右列纵排（正文 ≥2 行至 282、钮起 330；3 钮末底 540 < 卡底 620）
            (530.0, 330.0 + dy + i as f32 * 74.0, 480.0, 62.0)
        } else {
            (380.0, 316.0 + dy + i as f32 * step, 520.0, bh)
        };
        let accent = matches!(kind_acc, Some(crate::game::CardKind::Path) if i == 0)
            || matches!(kind_acc, Some(crate::game::CardKind::Finale) if i == 1)
            // v1.16.21 破境抉择卡首钮（立即破境/立即问笔）推荐位金描边
            || (card_brk && i == 0);
        // v1.16.37 单3 剧情演出包⑦：选项钮逐条入场节奏——开启帧起 0.06s 后每钮
        // 错峰 30ms（相位式照 draw_event_text 段除数压缩先例，除数取 0.03 恒定：
        // 稳态 modal_anim==0 时值=(0.12−0.03i)/0.03=4−i，i≤3 恒 ≥1.0 收敛为
        // 1.0——稳态逐像素不变红线；终战 4 钮 i≤3 恰收敛，事件卡 opts ≤3 同款）。
        // 命中不乘 α（动画期照常可点，卡体入场期本就随卡体偏移，口径沿用）。
        let btn_a = ((((0.18 - g.modal_anim) - 0.06) - i as f32 * 0.03) / 0.03).clamp(0.0, 1.0);
        if button_a_accent(bx, by, bw, bhh, label, Some(descs[i].1.as_str()), true, accent, btn_a) {
            picked = Some(i);
        }
    }

    // v1.16.4 暂避执行：段尾统一 &mut（title/descs 借用已还）。同一帧不可能同时
    // 命中选项与暂避钮（几何不相交），先暂避不互斥
    if hide_hit {
        g.modal_min = true;
        g.play(crate::sounds::Which::Page);
    }

    if let Some(i) = picked {
        g.play(crate::sounds::Which::Click);

        // v1.10.2 成行择道：道途卡抉择走 choose_path（rmodal_npc 为 None，
        // 不得落入 choose_roption——那会因 npc None 静默 return 使卡挂死）
        // v1.12.1 终战卡同理单列：choose_ending（rmodal_npc 亦为 None）
        // v1.16.45 现实邂逅卡同理单列：choose_rencounter（rmodal_npc 亦为 None，
        // 认下臂 special17 经 apply_reff → bond_life_companion 落现实 life）
        let kind = g.rmodal.as_ref().map(|c| c.kind);
        if kind == Some(crate::game::CardKind::Path) {
            g.choose_path(i as u8 + 1);
        } else if kind == Some(crate::game::CardKind::Finale) {
            g.choose_ending(i);
        } else if kind == Some(crate::game::CardKind::Rencounter) {
            g.choose_rencounter(i);
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
    // 演出包㉔ 遗泽拾取·魂光拾遗：拾取时刻金辉脉冲（共用 brk_veil_a 三角包络；
    // inh_spark==0 早退不落像素逐比特恒等；覆层独占分支 return 早退时不消费
    // 不破稳态——递减走主循环，重新可见时包络自然衰减）。与⑨..㉒错开：Inherit
    // 页既有演出仅⑰-2 置值口与静态小像，无拾取时刻动效，触发（take_inherit
    // 八分支收敛出口）/区域（此生卡描边）/语义（魂光拾遗）三异
    let is_a = 0.12 * brk_veil_a((g.inh_spark / crate::game::INH_SPARK_T).clamp(0.0, 1.0));
    if is_a > 0.0 {
        draw_rectangle_lines(58.0, 82.0, 424.0, 484.0, 2.0, fade(C_GOLD, is_a));
    }
    // v1.16.13 终局页『本稿遭遇』列表/回味覆层早退独占（照 zoom_art/ev_recall 先例
    // ——下层不绘不响应；Scene::Inherit 无 Reality 面板跳转路径，finish_inherit 是
    // 唯一出口，跳面板会静默弃未选继承项，故落页内覆层而非跳转）
    if g.run_recall_open {
        if g.ev_recall.is_some() {
            crate::gallery::draw_event_recall_overlay(g);
        } else {
            crate::gallery::draw_run_recall_list(g);
        }
        version_mark();
        return;
    }
    let Some(sr) = g.sim_result.as_ref() else {
        return;
    };
    // v1.9.25 卡顿修复（根因2）：上行为借用替代原 g.sim_result.clone()——本函数对 sr
    // 只读，选项/收尾按钮的 &mut 路径上 sr 已不再被使用（NLL 借用自然终结），逐帧深克隆消除
    txt_c("草 稿 退 回", V_W * 0.5, 56.0, 28, C_GOLD);
    // 左：此生摘要
    panel_card(60.0, 84.0, 420.0, 480.0);
    // UX 批：拾取窗内「此 生」标题字色随㉔同源包络向亮金插值（零新状态复用 is_a；
    // t=归一化包络 [0,1]，is_a==0 → t=0 → 各分量逐比特恒等 C_GOLD 稳态不变；
    // 纯算术插值不入字形缓存零重栅格化）
    let life_t = (is_a / 0.12).clamp(0.0, 1.0);
    let c_life = Color::new(
        C_GOLD.r + (1.0 - C_GOLD.r) * 0.35 * life_t,
        C_GOLD.g + (1.0 - C_GOLD.g) * 0.35 * life_t,
        C_GOLD.b + (1.0 - C_GOLD.b) * 0.35 * life_t,
        C_GOLD.a,
    );
    txt("此 生", 84.0, 116.0, 20, c_life);
    // v1.16.20 立绘沉浸续作①（继承屏伴小像）：此生同行者在左卡右上角显影——
    // 判定与结局页谢幕像/随行小像同源（sr.comp 名账 → COMPANION_NAMES 位置，
    // SimResult 会话态零新键）。独占屏（继承）直查 comp_tex 1 槽（红线口径：
    // 模态/结算/继承独占屏直查 ≤1 槽，无预算竞争面）；无伴零绘制零位移；XHR
    // 到图前画同尺寸暗槽兜底（结局页 :3438 先例），到图只换图零位移（槽几何按
    // companion 账锁定，不读纹理存在性——meeting latched 纪律）。槽位 (424,298)
    // 与道韵行（x84..140）/大事记头行（y336 起 x84..280）/朱批行热点（y356 起）
    // 水平垂直双净空，零版式改动。
    let inh_comp = sr
        .comp
        .as_deref()
        .and_then(|cn| COMPANION_NAMES.iter().position(|&n| n == cn));
    if let Some(ci) = inh_comp {
        txt_r(COMPANION_NAMES[ci], 416.0, 306.0, 14, C_PURPLE);
        let (av_x, av_y) = (424.0, 298.0);
        draw_rectangle(av_x, av_y, 32.0, 32.0, Color::from_rgba(30, 24, 50, 160));
        draw_rectangle_lines(av_x, av_y, 32.0, 32.0, 1.0, Color::from_rgba(150, 120, 190, 255));
        if let Some(t) = crate::art::art().comp_tex(ci) {
            let side = (t.height().max(1.0) * 0.35).min(t.width());
            draw_texture_ex(
                &t,
                av_x,
                av_y,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new((t.width() - side) * 0.5, 0.0, side, side)),
                    dest_size: Some(vec2(32.0, 32.0)),
                    ..Default::default()
                },
            );
        }
    }
    wrap_draw(&sr.cause, 84.0, 146.0, 16, 372.0, 25.0, C_RED);
    txt(
        &evf(2531, &[&(fmt_num(sr.age)) as &dyn core::fmt::Display, &(fmt_num(sr.years as f64)) as &dyn core::fmt::Display])/* 原文：享年 {} 载（历 {} 年） */,
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
    // v1.16.8 主线感：本稿剧情一行（SimResult.story 与旧稿档案 hq= 同源一次拼接；
    // 会话态零新键，&str 借用分片绘制零 format!；无摘要整行隐去零位移）。
    // v1.16.13 本稿剧情行点开『本稿遭遇』列表：此处只读绘制+热态金色反馈+记
    // open_story，&mut 全部放在 note_lines collect 之后（sr 借用随 collect 终结
    // ——本段末纪律照旧；is_mouse_button_pressed 帧内稳定，同帧后置判定等价）
    let mut open_story = false;
    if let Some(st) = sr.story.as_deref() {
        let (smx, smy) = crate::cam::mouse();
        let (mx0, mx1, my0, my1) = (486.0f32, 1230.0f32, 560.0f32, 592.0f32);
        let hot = smx >= mx0 && smx <= mx1 && smy >= my0 && smy <= my1;
        txt("本稿剧情", 490.0, 570.0, 14, if hot { C_GOLD } else { C_DIM });
        txt(st, 570.0, 570.0, 14, if hot { C_GOLD } else { C_GOLD_D });
        open_story = hot && is_mouse_button_pressed(MouseButton::Left);
    }
    // v1.12.0 留行批注：大事记点一行带走（下一稿开卷浮现后清空；定稿 §5.2 留行）。
    // 先收成 owned 再进点击循环（sr 为 &g.sim_result 借用，&mut g 须待其终结——照打稿屏 draft.clone() 先例）
    txt(evtx(2532)/* 原文：—— 大事记 · 点一行留作朱批 —— */, 84.0, 336.0, 14, C_DIM);
    let note_lines: Vec<String> = sr.highlights.iter().rev().take(7).cloned().collect();
    // v1.16.13 承上：open_story 落点（sr 借用已终结，&mut 照旧；覆层下帧经函数头
    // 早退独占接管，终局页不离场——继承选项/合页钮原位保留）
    if open_story {
        crate::gallery::prewarm_run_recall(g);
        g.run_recall_open = true;
        g.run_recall_scroll = 0.0;
        g.run_recall_drag = None;
        g.ev_recall = None;
        g.play(crate::sounds::Which::Page);
    }
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
        evtx(2533)/* 原文：退稿前，留一样东西。反正下一稿的你，也不知道你是谁。 */,
        524.0,
        116.0,
        20,
        C_PURPLE,
    );
    txt(
        &evf(2534, &[&(left) as &dyn core::fmt::Display])/* 原文：还可择 {} 项 · 所选将随神魂带回现实 */,
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
        txt(&format!("【{}】", card.name()), 536.0, y + 10.0, 18, C_GOLD);
        txt(card.desc(), 700.0, y + 12.0, 14, C_TEXT);
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
    // v1.16.18 搭车⑥：Inherit 页补画 toast/banner（render_reality/Sim 先例
    // draw.rs:1160-1161/1741-1742）——治「朱批已留」toast（本页 :2214 弹出）在本页
    // 永不可见的既有缺。单槽后写胜：同帧若另有终局横幅 banner 亦后写胜（朱批 toast
    // 与终局 banner 不同帧触发，实际无冲突面；可接受口径照勘察红线先行声明）
    draw_banner(g);
    draw_toast(g);
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
        // v1.16.19 死因行下一句收束语（按境界档静态 &'static str，SettleInfo.coda
        // real_die 事件帧定值 + 预热；settle 帧 format! 豁免不含本句——本行零分配）
        txt_c(s.coda, V_W * 0.5, 160.0 + hh + 4.0, 14, C_DIM);
        let mut y = 160.0 + hh + 30.0;
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
        // v1.16.20 立绘沉浸续作②（结算屏谢幕像）：此生同行者立于卡外右缘（结局页
        // 谢幕像 :3422 同款语言——npc_portrait_box_a + None 暗槽兜底）。快照读
        // SettleInfo.comp（real_die 事件帧定格的 companion 账终值），几何不随到图
        // 切换（meeting latched 纪律）；无伴零绘制零位移。独占屏（结算）直查
        // comp_tex 1 槽（红线口径 ≤1 槽/独占屏）。置卡外右缘 (1084..1254, 176..476)：
        // 卡体右缘 1070、卡内列宽 ≤1030（死因 wrap/数值 txt_r 1030）、双钮右缘 960、
        // 下一章行 prog 右锚 1040——水平全净空，零版式改动零碰撞。
        if let Some(ci) = s.comp {
            let (px, py, pw, ph) = (1084.0, 176.0, 170.0, 300.0);
            match crate::art::art().comp_tex(ci) {
                Some(t) => {
                    crate::charpanel::npc_portrait_box_a(
                        Some(&t),
                        &crate::art::art().icon_pill,
                        px,
                        py,
                        pw,
                        ph,
                        ph,
                        1.0,
                    );
                }
                None => {
                    draw_rectangle(px, py, pw, ph, Color::from_rgba(30, 24, 50, 160));
                    draw_rectangle_lines(px, py, pw, ph, 1.0, C_PURPLE);
                }
            }
        }
    }
    // v1.16.8 主线感：下一章预告一行（章节书当前章+首未了目标，全静态分片绘制
    // 零 format!——帧路径零分配红线；主线全了走字面量收束行）
    // v1.16.16 行右补「差多少」x/y 进度（prog_txt 静态查表零分配）与章成之礼行
    // （reward_note 静态字面量）——每世身故即见「下一章差什么、了了给什么」。
    // settle 屏有 format! 豁免先例（draw.rs 道韵累积行），本行仍走静态串零分配；
    // 字形由 real_die 事件帧预热（game.rs 结算章签预热口，防首帧冷栅格化）
    {
        let cur = crate::game::Game::chapter_current(&g.meta);
        let ch = &crate::data::CHAPTERS[cur];
        txt("下一章", 250.0, 524.0, 14, C_DIM);
        txt(ch.num(), 314.0, 524.0, 14, C_GOLD_D);
        txt(ch.name(), 376.0, 522.0, 16, C_GOLD);
        let goal = ch
            .goals
            .iter()
            .find(|gl| !crate::game::Game::chapter_goal_met(&g.meta, gl.pid))
            .map(|gl| gl.text())
            .unwrap_or(evtx(1799)/* 原文：主线已了——十万稿废纸，终有一册 */);
        txt(goal, 470.0, 524.0, 14, C_TEXT);
        if ch
            .goals
            .iter()
            .any(|gl| !crate::game::Game::chapter_goal_met(&g.meta, gl.pid))
        {
            txt_r(ch.reward_note(), 980.0, 524.0, 14, C_GOLD_D);
            if let Some(gl) = ch
                .goals
                .iter()
                .find(|gl| !crate::game::Game::chapter_goal_met(&g.meta, gl.pid))
            {
                if let Some((x, n)) = crate::game::Game::chapter_goal_prog(&g.meta, gl.pid) {
                    let s = prog_txt(x, n);
                    if !s.is_empty() {
                        txt_r(s, 1040.0, 524.0, 14, C_DIM);
                    }
                }
            }
        }
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
            // UX 批：选中条左端 12px 加亮段（坊市/行囊/角色页三组页签同批同语防
            // 半改；零状态纯新增像素照花样 D 先例口径；原位叠画不改选中条本体）
            draw_rectangle(x, 108.0, 12.0, 3.0, Color::from_rgba(255, 244, 200, 255));
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
            &evf(2535, &[&(fmt_int(cost)) as &dyn core::fmt::Display])/* 原文：当前价 {} 灵石 / 1 点 */,
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
            evtx(2536)/* 原文：轮回镜以灵石为引——点数推演一世，死后造化归身，可无限轮回。 */,
            176.0,
            260.0, 14,
            C_TEXT,
        );
        txt(
            evtx(2537)/* 原文：兑换价随境界水涨船高；模拟中反杀宿敌亦可白得点数。 */,
            176.0,
            294.0, 14,
            C_TEXT,
        );
        txt(
            &evf(2538, &[&(crate::data::AD_REWARD) as &dyn core::fmt::Display])/* 原文：山穷水尽（无点无石无法宝）时，「开始模拟」将引你观看幻境，看完得 {} 点。 */,
            176.0,
            328.0, 14,
            C_TEXT,
        );
        draw_rectangle(176.0, 380.0, 928.0, 1.5, Color::from_rgba(96, 76, 130, 160));
        txt(
            evtx(2539)/* 原文：点数跨世持久（随轮回烙印存档），身死不清零——攒点推演，方是长生正途。 */,
            176.0,
            398.0,
            14,
            C_DIM,
        );
    } else if g.shop_tab == 0 {
        for i in 0..6 {
            let y = 192.0 + i as f32 * 78.0;
            // 演出包㉕ 坊市成交·金铃涟漪（v1.16.49）：购丹成交后丹药阁六行金色
            // 下划线按行错峰显影——共用 brk_veil_a 包络，相位按行 −i×0.05 照
            // bond_mote_a 折相位款（单源勿双写；i 越大相位越低越早走完脉冲=自
            // 顶向下涟漪余晖）。market_ping==0 → 全行 α=0 早退不落像素，稳态
            // 逐像素不变；行底 y+58 距下行内容（y+78−6）净空 14px 无叠画。
            // UX 伪影修复①（勘察六 v1.16.50）：末行 i=5 的线 y+58=640..641.5 落
            // 底部提示行（txt y=636 基线带≈630.8..651.0）字形带内——成交后金线
            // 横穿灰字（文字后画盖线、字间透线），故末行不画线（其余行逐像素不
            // 变；嵌套 if 保「早退门」needle 字面照 v11656 钉面）
            let mp_a = 0.12
                * brk_veil_a(
                    ((g.market_ping / crate::game::MARKET_PING_T) - i as f32 * 0.05)
                        .clamp(0.0, 1.0),
                );
            if mp_a > 0.0 {
                if i < 5 {
                    draw_rectangle(160.0, y + 58.0, 880.0, 1.5, fade(C_GOLD, mp_a));
                }
            }
            let p = &PILLS[i];
            let price = g.pill_price(i);
            draw_texture(&crate::art::art().icon_pill, 160.0, y - 6.0, WHITE);
            txt(p.name(), 184.0, y + 4.0, 18, C_GOLD);
            txt(p.desc(), 320.0, y + 4.0, 14, C_TEXT);
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
            evtx(2540)/* 原文：（成行丹/破境丹/护神丹在突破与问笔时自动生效） */,
            180.0,
            636.0,
            14,
            C_DIM,
        );
    } else {
        txt(
            evtx(2541)/* 原文：藏经阁——功法一经习得，现实与模拟通用 */,
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
            txt(&format!("《{}》", t.name()), 180.0, y + 4.0, 18, C_GOLD);
            txt(
                &evf(2616, &[&(crate::data::Fmt1(t.mult)) as &dyn core::fmt::Display, &(crate::data::Fmt1(t.pow)) as &dyn core::fmt::Display])/* 原文：修炼×{}  战力×{} */,
                420.0,
                y + 4.0,
                14,
                C_TEXT,
            );
            txt(t.desc(), 420.0, y + 26.0, 14, C_DIM);
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
    let creative = crate::data::ad_creative((g.meta.ad_watched.max(0) as usize) % 3);
    txt_c(creative, 640.0, 288.0, 24, C_TEXT);
    txt_c(evtx(2542)/* 原文：—— 山穷水尽？镜灵为你接引一段商途幻境 —— */, 640.0, 326.0, 14, C_DIM);
    // 倒计时条
    let frac = (t / secs).clamp(0.0, 1.0);
    draw_rectangle(440.0, 360.0, 400.0, 14.0, Color::new(0.0, 0.0, 0.0, 0.5));
    draw_rectangle(440.0, 360.0, 400.0 * frac, 14.0, C_CYAN);
    txt_c(
        &evf(2543, &[&(crate::data::AD_REWARD) as &dyn core::fmt::Display])/* 原文：看完可得 {} 枚模拟点数 */,
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
            &evf(2617, &[&(crate::data::Fmt0((t.ceil()) as f64)) as &dyn core::fmt::Display])/* 原文：幻境播送中… {} 秒 */,
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
            // UX 批：选中条左端 12px 加亮段（三组页签同批同语，同坊市口批注）
            draw_rectangle(x, 120.0, 12.0, 3.0, Color::from_rgba(255, 244, 200, 255));
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
            // v1.16.5 顺带补图（照法宝行节奏：图标 172、名 196，丹药 6 种共用 icon_pill）
            draw_texture(&crate::art::art().icon_pill, 172.0, y + 3.0, WHITE);
            txt(p.name(), 196.0, y + 4.0, 18, C_GOLD);
            txt(&format!("×{}", pills[i]), 300.0, y + 4.0, 16, C_TEXT);
            txt(p.desc(), 360.0, y + 4.0, 14, C_TEXT);
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
                &evf(2544, &[&(RELICS[t.min(RELICS.len() - 1)].name()) as &dyn core::fmt::Display])/* 原文：炼化中：为【{}】选一件法宝作材料（再点「炼化」取消） */,
                180.0,
                158.0,
                16,
                C_PURPLE,
            );
        } else {
            txt(evtx(2545)/* 原文：法宝（未装备仍保有 30% 灵性） */, 180.0, 158.0, 16, C_DIM);
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
            draw_texture(&crate::art::art().relic_icons[k], 172.0, y - 2.0, WHITE); // v1.16.5 专属像素画替代共用剑图标
            txt(
                &format!("【{}】", rl.name()),
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
                rl.desc(),
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
            &format!("《{}》 {}", TECHS[tech].name(), TECHS[tech].desc()),
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
                &evf(2546, &[&(g.life
                        .as_ref()
                        .map_or(0, |x| x.equipped.len().min(RELIC_EQUIP_MAX))) as &dyn core::fmt::Display, &(RELIC_EQUIP_MAX) as &dyn core::fmt::Display, &(g.life.as_ref().map_or(0, |x| x.relics.len())) as &dyn core::fmt::Display])/* 原文：已装 {}/{} · 包内 {}（详情与装卸见「法宝」页签） */,
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
            evtx(2547)/* 原文：—— 随身行装一览；丹药服用见「丹药」页签，法宝装卸见「法宝」页签 —— */,
            180.0,
            320.0,
            14,
            C_DIM,
        );
    }
}

// ================= v1.10.2 离线闭关：出关结算卡 =================
// 独占输入（先例 ad-overlay）：明细列表 + 单「收功」钮。收益已在 resume_life 时
// 入账内存（qi/stones；v1.16.12 免烧寿元，age 不动），收功即一次 sync_save
//（去抖兜底 pagehide 亦覆盖）。
fn draw_offline_card(g: &mut Game) {
    let Some(r) = g.offline_card.as_ref() else {
        return;
    };
    // 演出包㉘ 离闭归来·结算卡浮现（v1.16.50）：resume_life n>=10 臂同帧旁置
    // offline_anim→暗幕 α 淡入（相位反推照③ modal_anim 款：开启帧 k=1→p=0.3
    // 系暗幕 0.234 先行压暗、随进度升至 0.78；稳态 anim==0 → k=0 → p=1.0 →
    // α 恒等 0.78 逐像素不变）。卡面 panel_card 无 α 口仅暗幕先行——勘察六备
    // 案：不做整体位移（卡内 txt 绝对坐标全改 dy 成本高）；动画期照常可点命
    // 中所见（①款独占输入判定不动）
    let k = (g.offline_anim / crate::game::OFFLINE_ANIM_T).clamp(0.0, 1.0); // 1=开启帧 → 0=稳态
    let p = 1.0 - k; // 入场进度 0→1
    let dim = 0.78 * (0.3 + 0.7 * p);
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, dim));
    panel_card(340.0, 160.0, 600.0, 400.0);
    txt_c("离 闭 归 来", V_W * 0.5, 188.0, 24, C_GOLD);
    draw_rectangle(380.0, 208.0, 520.0, 1.5, Color::from_rgba(96, 76, 130, 160));
    // v1.16.12 免烧寿元：原文「现实人间已悄然流逝 N 载」与 age 不动自相矛盾，改口
    txt(
        &evf(2548, &[&(r.years) as &dyn core::fmt::Display])/* 原文：离闭吐纳 {} 载功行，未曾间断。 */,
        380.0,
        226.0, 16,
        C_TEXT,
    );
    // v1.16.13 复审⑧：字号 15 是图集孤立档（UI_SIZES=[14,16,18,20,24,28] 之外，
    // 每字独立栅格化+首开图集 dirty）——收敛回 16（与 2788 行同档；本卡无预热口，
    // 静态后仍随面板通用字形命中）。行距 28px 容 16px 无溢出（面板内宽 ~520）。
    txt(
        &evf(2549, &[&(fmt_num(r.qi)) as &dyn core::fmt::Display])/* 原文：闭关吐纳（五折）：修为 +{} */,
        380.0,
        260.0, 16,
        C_CYAN,
    );
    txt(
        &evf(2550, &[&(fmt_int(r.stones)) as &dyn core::fmt::Display])/* 原文：洞府灵田岁入：灵石 +{} */,
        380.0,
        288.0, 16,
        C_CYAN,
    );
    txt(
        evtx(2551)/* 原文：收益已入账；修为满不破境，周天吐纳可续——点击收功归于现实。 */,
        380.0,
        320.0, 14,
        C_DIM,
    );
    if button(490.0, 420.0, 300.0, 64.0, "收功", Some("归于现实 · 落盘"), true) {
        g.offline_card = None;
        g.play(crate::sounds::Which::Gain);
        g.log(evtx(2552)/* 原文：你收功出关，将离闭所攒尽数归于己身。 */, C_GOLD);
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
        txt(u.name(), 230.0, y + 12.0, 16, C_GOLD);
        txt(&format!("{}/{}", lv, u.max), 350.0, y + 14.0, 14, C_TEXT);
        txt(u.desc(), 420.0, y + 14.0, 14, C_TEXT);
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
        evtx(2553)/* 原文：道韵：上一稿的你，留给这一稿的批注——稿退人亡，批注仍在。 */,
        230.0,
        592.0,
        14,
        C_DIM,
    );
    txt(
        &evf(2554, &[&(g.meta.lives) as &dyn core::fmt::Display, &(g.meta.total_sims) as &dyn core::fmt::Display, &(g.meta.neme_slain) as &dyn core::fmt::Display, &(fmt_num(g.meta.best_age)) as &dyn core::fmt::Display])/* 原文：轮回总计：历世 {} · 模拟 {} 次 · 斩杀宿敌 {} · 最长享年 {} 载 */,
        230.0,
        614.0,
        14,
        C_CYAN,
    );
}

// ================= 帮助 =================
// v1.8.0 B2：两页签「玩法」/「操作·技巧」，行距 41→52、字号 16→17（8 行 ×52 起点 200 底 616 < 650）
// v1.16.26：引导页文案自 draw_help_panel 局部提升为常量（字节中性，rodata 同址）
// ——供 draw_tests::help_lines_quote_style 引号规范回归钉测
// v1.16.29 刀B：帮助页文案表收编 textpack（TXT_HELP_BASE 起 17 行连号，
// 基址见 text_offsets.inc）——查表 fn 单口，O(1) 纯算术零分配（帮助页开屏
// 帧路径安全）；行内容与旧字面量逐字节等价（真相源 text_source.txt 行 + 钉测）。
pub fn help_line(idx: usize) -> &'static str {
    crate::data::evtx(crate::data::TXT_HELP_BASE as usize + idx.min(16))
}

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
    // v1.16.26 直引号统一：弯引号 “ ” → 「」（字形均在 cmap，字节中性）；
    // 文案本体已收编 textpack（v1.16.29 刀B：help_line 查表 fn，见 fn 上方）
    // v1.16.0：玩法签 7→8 行（新增图鉴行居第 8）；v1.16.16：玩法签 8→9 行（章节书
    // 行居第 9，末行 y=200+8×52=616、行高 30 → 底 646 < 卡底 650），操作·技巧签
    // 后移至 base 9（行数不变）
    let (base, count) = if g.help_tab == 0 { (0usize, 9usize) } else { (9usize, 8usize) };
    for j in 0..count {
        let i = base + j;
        let c = if j == 0 { C_GOLD } else { C_TEXT };
        wrap_draw(help_line(i), 180.0, 200.0 + j as f32 * 52.0, 16, 920.0, 30.0, c);
    }
}

// ================= v1.16.1 道途指引面板（顶栏「任务」钮，panel=6）=================
// 真机反馈「任务引导系统没看到」：指引横带只在现实主界面常驻，玩家在人物/图鉴/
// 坊市等页签里没有指引入口。本面板给一个全局可达的显性入口：当前目标（guide_index
// 纯推导，零存档键）+ 跳转按钮（guide::exec_jump 既有单口）+ 修行之道静态五行。
// v1.16.8 章节书页签：主线章节视图（v1.16.10 九章；左列章目/右栏本章引入+目标清单+当前目标高亮
// +前情提要）——达成位 chap_done（meta additive ch=），当前目标=首未了目标行。
// 布局照 draw_help_panel 语言；开启/切签帧字形由 prewarm_chapter_view 事件帧预热。
// ⚠panel=6 在 Menu 场景是书志页（draw.rs 顶部 dispatch）——两处场景隔离，本面板
// 只在 Reality 分发（draw_reality match g.panel），会话态 guide_tab 不跨场景共写。
// v1.16.33 单1 NIZE 通道：GUIDE_PANEL_LINES 收编 textpack（5 元素，同值共享
// N——引用位 evtx(GUIDE_PANEL_LINES_NS[k] as usize)，访问器/循环由施工方手改）
const GUIDE_PANEL_LINES_NS: [u16; 5] = [
    2516, 2517, 2518, 2519, 2520,
];

/// 指引面板「修行之道」行查表口（v1.16.33 单1 NIZE 通道访问器；越界 min 钳制
/// ——panic=abort 纪律；evtx 零分配 &'static str，帧路径零分配口径不变）
#[inline]
fn guide_panel_line(k: usize) -> &'static str {
    evtx(GUIDE_PANEL_LINES_NS[k.min(GUIDE_PANEL_LINES_NS.len() - 1)] as usize)
}

/// 指引面板底注查表口（v1.16.33 单1；同款钳制）
#[inline]
fn guide_panel_note() -> &'static str {
    evtx(GUIDE_PANEL_NOTE_N as usize)
}

/// 指引面板页签查表口（v1.16.33 单1；同款钳制）
#[inline]
fn guide_tab_label(k: usize) -> &'static str {
    evtx(GUIDE_TABS_NS[k.min(GUIDE_TABS_NS.len() - 1)] as usize)
}
// v1.16.33 单1 NIZE 通道：GUIDE_PANEL_NOTE 收编 textpack N=2435（引用位
// evtx(GUIDE_PANEL_NOTE_N as usize)，访问器/绘制位由施工方手改）
const GUIDE_PANEL_NOTE_N: u16 = 2435;
// v1.16.33 单1 NIZE 通道：GUIDE_TABS 收编 textpack（2 元素，同值共享
// N——引用位 evtx(GUIDE_TABS_NS[k] as usize)，访问器/循环由施工方手改）
// v1.16.40 长老委托：扩第三签「门中差事」（N2727 真相源尾段追加，独立 N 非
// 同值共享），guide_tab 钳制/第三签分支/预热同批扩
const GUIDE_TABS_NS: [u16; 3] = [
    2521, 2522, 2727,
];

// 章节书全量字形预热（开启面板/切签事件帧一次；九章文案+状态词+奖励行+章间记事
// 全段+进度串+前情摘要，动态的只有前情摘要一行 &str 借用，稳态帧零栅格化零分配）。
// v1.16.9 改 pub：guide.rs exec_jump Book 臂横带跳转同口预热（任务钮/跳转共用单口）。
pub fn prewarm_chapter_view(g: &Game) {
    prewarm_text("章节书", &[16]);
    prewarm_text("本章目标", &[16]);
    prewarm_text("章间记事", &[14]);
    prewarm_text("前情提要", &[14]);
    for s in ["已了", "进行", "未启", "已毕", "待办"] {
        prewarm_text(s, &[14]);
    }
    // v1.16.9 目标行右进度串全量预热（prog_txt 静态查表：分母 1/2/3/4/5/6/9/10/13
    // × 分子 0..=分母——漏灌会复现首开 2048² 图集整图重传卡顿，charpanel.rs 同类
    // 事故教训。v1.16.16 分母扩 4/9/10/13：全 pid 数值化 + 结算屏/主线汇总复用）
    for n in [1u32, 2, 3, 4, 5, 6, 9, 10, 13] {
        for x in 0..=n {
            prewarm_text(prog_txt(x, n), &[14]);
        }
    }
    for ch in crate::data::CHAPTERS.iter() {
        prewarm_text(ch.num(), &[14]);
        prewarm_text(ch.name(), &[14, 16, 20]);
        prewarm_text(ch.intro(), &[16]);
        prewarm_text(ch.reward_note(), &[14]); // 章成之礼行（含「道韵 +N」数字）
        for lk in 0..ch.lore_len() {
            prewarm_text(ch.lore(lk), &[14]); // 章间记事全段预热（可见性是纯显示推导）
        }
        for gl in ch.goals.iter() {
            prewarm_text(gl.text(), &[16]);
        }
    }
    if let Some(st) = g.meta.hist.last().and_then(|r| r.story.as_deref()) {
        prewarm_text(st, &[14]);
    }
}

// 指引面板全量字形预热单口（v1.16.9 封装）：任务钮开面板与横带 Book 跳转共用。
// 静态文案+当前目标行+两签+章节书视图（guide_tab 跨次开面板可能残留 1，一次灌齐
// 防缺字——原任务钮块内联预热原样搬入）。
pub fn prewarm_guide_panel(g: &Game) {
    prewarm_text("道途指引", &[24]);
    prewarm_text("当前目标", &[16]);
    prewarm_text("修行之道", &[16]);
    for k in 0..GUIDE_PANEL_LINES_NS.len() {
        prewarm_text(guide_panel_line(k), &[14]);
    }
    prewarm_text(guide_panel_note(), &[14]);
    let idx = crate::guide::guide_index(g).min(crate::guide::HINTS.len() as u8 - 1);
    let hint = &crate::guide::HINTS[idx as usize];
    prewarm_text(hint.text(), &[20]);
    if let Some(b) = hint.btn {
        prewarm_text(b, &[18]);
    }
    for k in 0..GUIDE_TABS_NS.len() {
        prewarm_text(guide_tab_label(k), &[16]);
    }
    // v1.16.40 长老委托第三签：委托名/描述/达标句/钮文案随 comm_sel 轮换——开
    // 面板帧一次灌齐当前委托全字形（comm_current 纯推导只读）；进度串 prog_txt
    // 分母 1..=4 组合由 prewarm_chapter_view @14 预热；委托视图进度 @16 绘制
    //（v1.16.42 勘察五§5 注释改准）——字形经 PANEL_WARM_CHARS（含 0-9 与 / 全
    // 数字）× UI_SIZES 六档全量预热兜底。「进度」为 <24B 原位字面量（NIZE 阈值
    // 纪律），预热照灌防冷栅格化。v1.16.42 上品徽标串同批预热（徽标随 done 节律
    // 显隐，无条件灌齐成本 7 字形 @14 一次）
    let ck = g.meta.comm_current();
    prewarm_text(crate::data::comm_title(ck), &[20]);
    prewarm_text(crate::data::comm_desc(ck), &[14]);
    prewarm_text(crate::data::evtx(crate::data::COMM_DONE_N as usize), &[16]);
    prewarm_text("交差", &[20]);
    prewarm_text("进度", &[14]);
    prewarm_text("上品 · 报酬翻倍", &[14]);
    prewarm_text("·上品", &[14]);
    // 剧情可玩性·委托称号：第三签进度行右挂目标指引串同批预热（复命口
    // comm_claim 同款；prog_txt n/10 档由 prewarm_chapter_view @14 预热兜底）
    prewarm_text("差事十全", &[14]);
    prewarm_chapter_view(g);
}

// v1.16.9 目标进度串静态查表：帧路径零分配红线（draw_chapter_view 无 format!），
// 数字不进 format!——(分子,分母) 全组合枚举返回 &'static str，越界组合返回空串
// 隐藏（防御臂，progress 单口已 clamp 不可达）。串集由 prewarm_chapter_view 全量预热。
// v1.16.16 分母扩 4/9/10/13（全 pid 数值化：4=四结局、9=主线 X/9 汇总、
// 10=问笔、13=执卷境）。
fn prog_txt(x: u32, n: u32) -> &'static str {
    match (x, n) {
        (0, 1) => "0/1",
        (1, 1) => "1/1",
        (0, 2) => "0/2",
        (1, 2) => "1/2",
        (2, 2) => "2/2",
        (0, 3) => "0/3",
        (1, 3) => "1/3",
        (2, 3) => "2/3",
        (3, 3) => "3/3",
        (0, 4) => "0/4",
        (1, 4) => "1/4",
        (2, 4) => "2/4",
        (3, 4) => "3/4",
        (4, 4) => "4/4",
        (0, 5) => "0/5",
        (1, 5) => "1/5",
        (2, 5) => "2/5",
        (3, 5) => "3/5",
        (4, 5) => "4/5",
        (5, 5) => "5/5",
        (0, 6) => "0/6",
        (1, 6) => "1/6",
        (2, 6) => "2/6",
        (3, 6) => "3/6",
        (4, 6) => "4/6",
        (5, 6) => "5/6",
        (6, 6) => "6/6",
        (0, 9) => "0/9",
        (1, 9) => "1/9",
        (2, 9) => "2/9",
        (3, 9) => "3/9",
        (4, 9) => "4/9",
        (5, 9) => "5/9",
        (6, 9) => "6/9",
        (7, 9) => "7/9",
        (8, 9) => "8/9",
        (9, 9) => "9/9",
        (0, 10) => "0/10",
        (1, 10) => "1/10",
        (2, 10) => "2/10",
        (3, 10) => "3/10",
        (4, 10) => "4/10",
        (5, 10) => "5/10",
        (6, 10) => "6/10",
        (7, 10) => "7/10",
        (8, 10) => "8/10",
        (9, 10) => "9/10",
        (10, 10) => "10/10",
        (0, 13) => "0/13",
        (1, 13) => "1/13",
        (2, 13) => "2/13",
        (3, 13) => "3/13",
        (4, 13) => "4/13",
        (5, 13) => "5/13",
        (6, 13) => "6/13",
        (7, 13) => "7/13",
        (8, 13) => "8/13",
        (9, 13) => "9/13",
        (10, 13) => "10/13",
        (11, 13) => "11/13",
        (12, 13) => "12/13",
        (13, 13) => "13/13",
        _ => "",
    }
}

fn draw_guide_panel(g: &mut Game) {
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, 0.6));
    panel_card(140.0, 60.0, 1000.0, 600.0);
    txt("道途指引", 170.0, 92.0, 24, C_GOLD);
    if button(1064.0, 74.0, 44.0, 36.0, "×", None, true) {
        g.panel = 0;
        g.play(crate::sounds::Which::Page);
        return;
    }
    // 页签（照 draw_help_panel 语言；guide_tab 会话态 min 钳制，panic=abort 纪律）
    // v1.16.40 长老委托第三签「门中差事」：钳制上沿 1→2（页签循环体逐位不变）
    g.guide_tab = g.guide_tab.min(2);
    for (i, &n) in GUIDE_TABS_NS.iter().enumerate() {
        let t = evtx(n as usize);
        let x = 170.0 + i as f32 * 200.0;
        if button(x, 128.0, 190.0, 34.0, t, None, true) {
            g.guide_tab = i as u8;
            if i == 1 {
                prewarm_chapter_view(g); // 事件帧预热（切签一次）
            }
            g.play(crate::sounds::Which::Page);
        }
        if g.guide_tab == i as u8 {
            draw_rectangle(x, 162.0, 190.0, 3.0, C_GOLD);
        }
    }
    // v1.16.16 主线汇总：页签行右「主线 X/9」（chap_done 位计数，prog_txt 静态查表
    // 零分配——章节书视图禁 format! 红线；与书志全貌带「主线 X/9」口径一致，串集由
    // prewarm_chapter_view 分母 9 全量预热）。防越界：count_ones ≤ 9 上钳（章数即 9，
    // 未来扩章同 min 钳制纪律）
    txt_r(
        prog_txt(
            g.meta.chap_done.count_ones().min(crate::data::CHAPTERS.len() as u32),
            crate::data::CHAPTERS.len() as u32,
        ),
        1040.0,
        136.0,
        14,
        C_DIM,
    );
    if g.guide_tab == 2 {
        draw_comm_view(g);
        return;
    }
    if g.guide_tab == 1 {
        draw_chapter_view(g);
        return;
    }
    let idx = crate::guide::guide_index(g).min(crate::guide::HINTS.len() as u8 - 1);
    let hint = &crate::guide::HINTS[idx as usize];
    txt("当前目标", 170.0, 184.0, 16, C_GOLD_D);
    wrap_draw(hint.text(), 170.0, 214.0, 20, 900.0, 32.0, C_GOLD);
    if let Some(b) = hint.btn {
        if button(170.0, 262.0, 240.0, 54.0, b, None, true) {
            crate::guide::exec_jump(g, hint.act);
            // Shop/Bag 跳转已自置面板（坊市/行囊），其余跳转归位主界面；Book 例外：
            // 面板内点「去书志」只切章节书签，面板保持开着（横带场景 Book 自置 panel=6）
            if g.panel == 6 && hint.act != crate::guide::GuideAct::Book {
                g.panel = 0;
            }
        }
    }
    draw_rectangle(170.0, 340.0, 940.0, 1.5, Color::from_rgba(96, 76, 130, 160));
    txt("修行之道", 170.0, 356.0, 16, C_GOLD_D);
    for (i, &n) in GUIDE_PANEL_LINES_NS.iter().enumerate() {
        txt(evtx(n as usize), 170.0, 388.0 + i as f32 * 30.0, 14, C_TEXT);
    }
    txt(guide_panel_note(), 170.0, 560.0, 14, C_DIM);
}

// v1.16.40 长老委托视图（第三签「门中差事」）：当前委托名/描述 + 进度（未达标
// prog_txt 静态查表——分母 1..=4 组合 prewarm_chapter_view 仅 @14 预热，本视图
// @16 绘制无专项预热（v1.16.42 勘察五§5 注释改准）——0-9 与 / 全在
// PANEL_WARM_CHARS，panel_warm_step 六档全量预热兜底，无冷栅格化缺口；达标换
// 金句 N2744）+「交差」钮（达标才可点，claim 内部再校验照 gift_npc「按钮仅 UI 门」
// 先例，复命后面板保持开显示轮换后的新委托）。全静态查表零 format!（帧路径零分配
// 红线）；委托名/描述随轮换变化，字形由 prewarm_guide_panel（开面板帧）与
// comm_claim（复命帧）双口事件级预热。v1.16.42 名行右侧上品徽标（comm_premium
// 节律显隐，字面量直绘，双口预热同批灌齐）。
fn draw_comm_view(g: &mut Game) {
    let ck = g.meta.comm_current();
    let c = crate::data::COMMISSIONS[ck];
    txt(guide_tab_label(2), 170.0, 184.0, 16, C_GOLD_D);
    txt(crate::data::comm_title(ck), 170.0, 214.0, 20, C_GOLD);
    // 剧情可玩性·上品委托（v1.16.42 勘察二主选 A）：当前委托落上品档（done%5==4）
    // 时名行右侧挂金徽标——谓词与 comm_claim 发奖同款同拍（done 在复命口尾部才
    // 自增，绘↔账恒一致）；字面量直绘不入 textpack（照「进度/交差」先例），字形
    // 由 prewarm_guide_panel/复命帧两口预热。名 4 字@20 止于 x≈250，徽标 @14
    // 7 字≈119px 至 x≈389，行内右侧无碰撞（页签带 y128..166/描述带 y262 均不交）
    if crate::data::comm_premium(g.meta.comm_done) {
        txt("上品 · 报酬翻倍", 270.0, 218.0, 14, C_GOLD);
    }
    wrap_draw(
        crate::data::comm_desc(ck),
        170.0,
        262.0,
        14,
        900.0,
        30.0,
        C_TEXT,
    );
    let done = g.meta.comm_prog >= c.target;
    if done {
        txt(
            crate::data::evtx(crate::data::COMM_DONE_N as usize),
            170.0,
            320.0, 16,
            C_GOLD,
        );
    } else {
        txt("进度", 170.0, 320.0, 14, C_DIM);
        txt(
            prog_txt(
                (g.meta.comm_prog as u32).min(c.target as u32),
                c.target as u32,
            ),
            230.0,
            320.0, 16,
            C_CYAN,
        );
    }
    // 剧情可玩性·委托称号（勘二主选）：进度行右侧同行挂「差事十全」目标指引
    // ——零版式位移照上品徽标同行右挂先例：进度态左段「进度 x/y」止于 x≈300、
    // 达标态金句 9 字 @16 止于 x≈314，挂点 x=560 起无碰撞；「差事十全」4 字 @14
    // ≈62px + prog_txt n/10 ≈45px 至 x≈688，卡内右缘余量足。n/10 走 prog_txt
    // 静态查表（分母 10 全档 prewarm_chapter_view @14 预热+PANEL_WARM_CHARS×
    // UI_SIZES 兜底）零 format! 零分配；满十单后恒 10/10 转金——称号页佩戴指路
    //（授权/佩戴走 set_worn_title 18 臂）。字形双口预热（prewarm_guide_panel/
    // comm_claim）。
    let t_earned = g.meta.comm_done >= 10;
    let t_col = if t_earned { C_GOLD } else { C_DIM };
    txt("差事十全", 560.0, 320.0, 14, t_col);
    txt(
        prog_txt((g.meta.comm_done as u32).min(10), 10),
        632.0,
        320.0, 14,
        t_col,
    );
    if button(170.0, 360.0, 240.0, 54.0, "交差", None, done) {
        g.comm_claim();
    }
}

// v1.16.8 章节书视图：左列章目目录（v1.16.10 起九行，状态词右上）+ 右栏本章引入/目标清单/前情提要。
// 全静态 &str 分片绘制零 format!（帧路径零分配红线）；当前目标=首未了目标行金字高亮；
// 达成位 meta.chap_done 由 check_chapters 边沿维护，此处纯读取。
// v1.16.9 右栏加密度：目标行右 x/y 进度（prog_txt 静态查表）+ 章成之礼行（reward_note
// 全静态字面量，与 reward 数字对应由表单测钉死）+ 章间记事（lore 逐段解锁，段 i 可见
// ⟺ 已毕目标数 ≥ i，显示最新解锁段——纯显示推导零存档）。行高预算：intro ≤4 行
// 222..334 / 目标 382..460 / 礼行 448 / 记事 508..548 / 提要 588..648，均在卡内。
fn draw_chapter_view(g: &mut Game) {
    let cur = crate::game::Game::chapter_current(&g.meta);
    // 左列：章目 9 行（y188 起步进 46 → 末行 556，均在卡内且避开标题屏/底栏按钮带）
    for (i, ch) in crate::data::CHAPTERS.iter().enumerate() {
        let done = g.meta.chap_done >> i & 1 == 1;
        let y = 188.0 + i as f32 * 46.0;
        let (st, c) = if done {
            ("已了", C_GOLD_D)
        } else if i == cur {
            ("进行", C_GOLD)
        } else {
            ("未启", C_DIM)
        };
        txt(ch.num(), 178.0, y, 14, c);
        txt(ch.name(), 238.0, y - 2.0, 16, c);
        txt_r(st, 442.0, y, 14, c);
        if i == cur {
            draw_rectangle(170.0, y + 20.0, 286.0, 2.0, C_GOLD);
        }
    }
    // 右栏：本章引入 + 目标清单（当前目标=首未了行金字，行右 x/y 进度）+ 章成之礼
    // + 章间记事 + 前情提要
    let ch = &crate::data::CHAPTERS[cur];
    txt(ch.num(), 490.0, 188.0, 14, C_GOLD_D);
    txt(ch.name(), 552.0, 184.0, 20, C_GOLD);
    // v1.16.28 章回联句：章名行右侧同行尾挂（@14 dim——章名 4 字 20px 至 ~640，
    // 联句 ≤14 字约 170px 收在卡内右缘前，同行零版式位移零碰撞；结算屏「下一章」
    // 行已被 goal/礼行/进度三段占满不加挤，单点显示维持宁缺毋滥）。字形经结算屏
    // real_die 章签预热口灌齐（同串同字号，两页共用缓存）
    txt(ch.couplet(), 648.0, 190.0, 14, C_DIM);
    wrap_draw(ch.intro(), 490.0, 222.0, 16, 600.0, 28.0, C_TEXT);
    draw_rectangle(490.0, 340.0, 610.0, 1.5, Color::from_rgba(96, 76, 130, 160));
    txt("本章目标", 490.0, 352.0, 16, C_GOLD_D);
    let mut cur_goal_hit = false; // 首未了目标行标记（金字高亮一次）
    for (j, gl) in ch.goals.iter().enumerate() {
        let met = crate::game::Game::chapter_goal_met(&g.meta, gl.pid);
        let y = 382.0 + j as f32 * 30.0;
        let (st, c) = if met {
            ("已毕", C_DIM)
        } else if !cur_goal_hit {
            cur_goal_hit = true;
            ("进行", C_GOLD)
        } else {
            ("待办", C_TEXT)
        };
        txt(st, 490.0, y + 2.0, 14, c);
        wrap_draw(gl.text(), 540.0, y, 16, 500.0, 24.0, c);
        // 行右 x/y 进度（数值化目标才有值；静态查表串零分配，空串防御隐藏）
        if let Some((x, n)) = crate::game::Game::chapter_goal_prog(&g.meta, gl.pid) {
            let s = prog_txt(x, n);
            if !s.is_empty() {
                txt_r(s, 1096.0, y + 2.0, 14, c);
            }
        }
    }
    // 章成之礼行：达成奖励预告（全静态字面量，发放幂等性在 game.rs check_chapters）
    txt(ch.reward_note(), 490.0, 448.0, 14, C_GOLD_D);
    draw_rectangle(490.0, 474.0, 610.0, 1.5, Color::from_rgba(96, 76, 130, 160));
    // 章间记事：lore 段序号「其一/其二」自带静态文本，随进度翻段
    if ch.lore_len() > 0 {
        let met_n = ch
            .goals
            .iter()
            .filter(|gl| crate::game::Game::chapter_goal_met(&g.meta, gl.pid))
            .count();
        let seg = ch.lore(met_n.min(ch.lore_len() - 1));
        txt("章间记事", 490.0, 486.0, 14, C_DIM);
        wrap_draw(seg, 490.0, 508.0, 14, 610.0, 20.0, C_TEXT);
    }
    // 前情提要：旧稿档案最近一稿剧情摘要（&str 借用零分配；无稿/无摘要整块隐去）
    if let Some(st) = g.meta.hist.last().and_then(|r| r.story.as_deref()) {
        draw_rectangle(490.0, 556.0, 610.0, 1.5, Color::from_rgba(96, 76, 130, 110));
        txt("前情提要", 490.0, 566.0, 14, C_DIM);
        wrap_draw(st, 490.0, 588.0, 14, 610.0, 20.0, C_TEXT);
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
    // 剧情演出包②：入场节奏 0.6s（choose_ending 事件帧置 ending_anim，主循环递减；
    // 字号 32/17/16/18 已在该事件帧一次预热，动画期零新字形零图集重传）。相位链：
    // 暗幕先行 → 卡体 0.10s 起浮现 → 标题 0.25s 起淡入 → 正文 0.40s 起 → 收益行
    // 0.55s 起 → 合页钮动画结束后解禁（禁点期 enabled=false 灰显，不吞 Esc/暂避外输入）。
    // 书志重读（from_book）与稳态（ending_anim==0）k=0 → 全相位=1，逐像素等同旧版式
    let k = if from_book {
        0.0
    } else {
        (g.ending_anim / 0.6).clamp(0.0, 1.0)
    };
    let p = 1.0 - k; // 入场进度 0→1
    let pe = p * p * (3.0 - 2.0 * p); // smoothstep 缓动（纯算术）
    let dim_a = (242.0 * (0.35 + 0.65 * pe)) as u8; // 暗幕先行：开启帧 85，随进度至 242
    let card_a = ((p - 0.10) / 0.30).clamp(0.0, 1.0);
    let title_a = ((p - 0.25) / 0.30).clamp(0.0, 1.0);
    let body_a = ((p - 0.40) / 0.30).clamp(0.0, 1.0);
    let gain_a = ((p - 0.55) / 0.30).clamp(0.0, 1.0);
    draw_rectangle(0.0, 0.0, V_W, V_H, Color::from_rgba(8, 5, 16, dim_a));
    panel_card_a(190.0, 80.0, 900.0, 560.0, card_a);
    txt_c(
        &format!("《{}》 · {}", e.name(), e.alias()),
        V_W * 0.5,
        130.0,
        32,
        fade(C_GOLD, title_a),
    );
    draw_rectangle(
        280.0,
        162.0,
        720.0,
        1.5,
        Color::new(
            96.0 / 255.0,
            76.0 / 255.0,
            130.0 / 255.0,
            160.0 / 255.0 * card_a,
        ),
    );
    title_ornament(V_W * 0.5, 162.0, card_a);
    // v1.16.19 道侣谢幕像（立绘沉浸扩展②）：此生同行者（life.companion 推导——
    // 人物页 comp_portrait_idx 此生半边同语言，from_book 重看路径同式生效）在
    // 900 宽卡右缘收 ~160×300 立绘（npc_portrait_box_a 底边锚 478，alpha 挂
    // gain_a 相位与收益行同拍淡入）。道侣槽已被随行小像/道侣专页首取（≈零新增
    // 解码；即便冷槽也 ≤1 查询/帧，远低于首取红线）。版式纪律：正文换行宽随
    // 「有伴」收 760→560——该判定读 companion 账（覆层存续期不变量），不读纹理
    // 存在性，故 XHR 到图只换图不改几何（meeting latched draw.rs:1867 同款）；
    // 无图（wasm Loading 期/缺图定案）画同尺寸暗槽占位，到图切换零位移。
    let comp_idx = g
        .life
        .as_ref()
        .and_then(|l| l.companion.as_deref())
        .and_then(|cn| COMPANION_NAMES.iter().position(|&n| n == cn));
    let body_w = if comp_idx.is_some() { 560.0 } else { 760.0 };
    wrap_draw(e.body(), 260.0, 186.0, 17, body_w, 30.0, fade(C_TEXT, body_a));
    if let Some(ci) = comp_idx {
        let (px, py, pw, ph) = (906.0, 182.0, 170.0, 296.0); // 卡右缘（卡体 190..1090，右缩 14）
        let alpha = gain_a.clamp(0.0, 1.0);
        match crate::art::art().comp_tex(ci) {
            Some(t) => {
                crate::charpanel::npc_portrait_box_a(
                    Some(&t),
                    &crate::art::art().icon_pill,
                    px,
                    py,
                    pw,
                    ph,
                    ph,
                    alpha,
                );
            }
            None => {
                draw_rectangle(px, py, pw, ph, fade(Color::from_rgba(30, 24, 50, 160), alpha));
                draw_rectangle_lines(px, py, pw, ph, 1.0, fade(C_PURPLE, alpha));
            }
        }
    }
    for (j, ln) in e.gain().lines().enumerate() {
        txt_c(ln, V_W * 0.5, 412.0 + j as f32 * 26.0, 16, fade(C_CYAN, gain_a));
    }
    // 合页钮 y470..518：书志重看路径下标题屏底层按钮带（开始 522-576/底排 592-636）
    // 先于本页做命中测试——立即模式下交互矩形不得与其重叠（烙印面板顿悟钮同款布局回避）
    // 剧情演出包②：动画期禁点（enabled=false 灰显——视觉上预告「先读完这一页」）
    let can_close = from_book || g.ending_anim <= 0.0;
    if button(
        V_W * 0.5 - 130.0,
        470.0,
        260.0,
        48.0,
        "合上这一页",
        None,
        can_close,
    ) {
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
        evtx(2555)/* 原文：问笔终战（现实破境问笔·九重雷劫全扛，且封卷吏已斩）之后，执笔人递笔——你如何落款？ */,
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
                format!("《{}》 · {}", e.name(), e.alias()),
                e.gain().lines().next().unwrap_or("").to_string(),
            )
        } else if i == 3 {
            (
                "？？？".to_string(),
                evtx(2556)/* 原文：先撕稿留白（后事不详），再来问笔——未完的书，收册人自会看见 */.to_string(),
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
    // ===== v1.16.11 此书全貌：非交互汇总带（卡内 514..650 空白）=====
    // 四数据源全纯推导零新键：chap_done 位计数+chapter_current（章节）、
    // worn_title_of（称号）、ev_cnt>0 计数（手札）、endings_seen 位计数（结局）。
    // 布局纪律同上：只绘不响应（无 button 命中矩形），底层按钮带 522..636 先命中
    // 不受影响；format! 属本页既有行循环同款（模态静态页，非稳态滚动帧路径）
    draw_rectangle(170.0, 524.0, 940.0, 1.5, Color::from_rgba(96, 76, 130, 160));
    txt("此书全貌", 170.0, 532.0, 14, C_GOLD);
    let n_ch = crate::data::CHAPTERS.len();
    let done_ch = g.meta.chap_done.count_ones() as usize;
    let cur = Game::chapter_current(&g.meta);
    let seen_n = g.meta.ev_cnt.iter().filter(|&&c| c > 0).count();
    let end_n = g.meta.endings_seen.count_ones() as usize;
    if done_ch >= n_ch {
        txt(
            &evf(2557, &[&(done_ch) as &dyn core::fmt::Display, &(n_ch) as &dyn core::fmt::Display])/* 原文：主线 {}/{} · 九章已毕 */,
            170.0, 554.0, 14, C_TEXT,
        );
    } else {
        txt(
            &evf(2558, &[&(done_ch) as &dyn core::fmt::Display, &(n_ch) as &dyn core::fmt::Display, &(crate::data::CHAPTERS[cur.min(n_ch - 1)].name()) as &dyn core::fmt::Display])/* 原文：主线 {}/{} · 现进行：{} */,
            170.0, 554.0, 14, C_TEXT,
        );
    }
    // v1.16.15 全卷标注：四结局集齐时追加执笔人佩显状态（纯推导零新键，与
    // choose_ending 全卷之礼 toast 成对交付；并进本行零新增行零布局风险）
    let end_note = if end_n >= crate::game::ENDINGS.len() {
        if g.meta.worn_title == 17 {
            evtx(2559)/* 原文： · 四结局集齐·『执笔人』已佩 */
        } else {
            evtx(2560)/* 原文： · 四结局集齐·『执笔人』可佩 */
        }
    } else {
        ""
    };
    txt(
        &evf(2561, &[&(g.worn_title_of()) as &dyn core::fmt::Display, &(seen_n) as &dyn core::fmt::Display, &(crate::data::EVENTS.len()) as &dyn core::fmt::Display, &(end_n) as &dyn core::fmt::Display, &(crate::game::ENDINGS.len()) as &dyn core::fmt::Display, &(end_note) as &dyn core::fmt::Display])/* 原文：称号『{}』 · 手札 {}/{} · 结局 {}/{}{} */,
        170.0, 578.0, 14, C_TEXT,
    );
}

// v1.16.11 书志页全量字形预热单口（照 prewarm_guide_panel 单口语言）：书志钮开启帧
// 一次灌齐（事件帧幂等，严禁入稳态帧）——顺带治书志页既有潜伏项（标题/导语/？？？/
// ENDINGS 名+alias+gain 首行/未达成两行引导语首开全冷）。全貌带动态值只有章节名/
// 称号两处 &str 源自静态表，其余 ASCII 数字由菜单期 panel_warm_chars 保底覆盖
pub fn prewarm_book_record(g: &Game) {
    prewarm_text("书 志 · 终卷四结局", &[24]);
    prewarm_text(
        evtx(2555)/* 原文：问笔终战（现实破境问笔·九重雷劫全扛，且封卷吏已斩）之后，执笔人递笔——你如何落款？ */,
        &[14],
    );
    // ？？？双档预热（复审修）：14 供图鉴/手札行复用口径，20 供本页未达成行
    // button 主标（draw.rs button sub 存在时 label 恒 20）——「？」不在
    // PANEL_WARM_CHARS、全库无其它 @20 预热，漏 20 档则新档首开书志页
    // （结局未达成最常见态）3 个 20px 字形冷栅格化触发 2048² 图集整图重传
    prewarm_text("？？？", &[14, 20]);
    prewarm_text("走向问笔终战，卡上自有分说", &[14]);
    prewarm_text(evtx(2556)/* 原文：先撕稿留白（后事不详），再来问笔——未完的书，收册人自会看见 */, &[14]);
    for e in crate::game::ENDINGS.iter() {
        // 行首括号/间隔符与名/alias 一次灌齐（事件帧 format! 豁免，照 build_event
        // 卡面预热同口径）
        prewarm_text(&format!("《{}》 · {}", e.name(), e.alias()), &[20]);
        prewarm_text(e.gain().lines().next().unwrap_or(""), &[14]);
    }
    // 全貌带静态标签与动态源（九章章名 + 当前佩戴称号串，两源皆静态表成员）；
    // v1.16.15 全卷标注两态静态段灌齐（四结局集齐后此行才可能出现）
    prewarm_text("此书全貌", &[14]);
    for s in [
        "主线",
        "现进行：",
        "九章已毕",
        "称号『』",
        "手札",
        "结局",
        evtx(2560)/* 原文： · 四结局集齐·『执笔人』可佩 */,
        evtx(2559)/* 原文： · 四结局集齐·『执笔人』已佩 */,
    ] {
        prewarm_text(s, &[14]);
    }
    for ch in crate::data::CHAPTERS.iter() {
        prewarm_text(ch.name(), &[14]);
    }
    prewarm_text(&g.worn_title_of(), &[14]);
}

// ---------------- v1.16.13 复审①②：菜单危险钮守卫回归（纯函数单测） ----------------
// 现场背景：书志页全貌带 y554 行（draw_book_record）与底排按钮带 522..576 重叠，
// 立即模式底排 button 先于罩层绘制命中（619-621 注）——panel 守卫缺失时开书志
// 点该带即静默 begin_mortal/resume_life 覆盖当前人生。
#[cfg(test)]
mod menu_danger_btn_tests {
    use super::menu_danger_btn_hit;

    // panel==0：开始新的人生恒可命中（无档新档也要能开新局）
    #[test]
    fn begin_btn_on_menu_panel() {
        assert!(menu_danger_btn_hit(0, false, false));
        assert!(menu_danger_btn_hit(0, false, true));
    }

    // panel==0 且有档：承接前缘可命中；无档不可命中（原 `life.is_some()` 语义保持）
    #[test]
    fn resume_btn_on_menu_panel_gated_by_life() {
        assert!(menu_danger_btn_hit(0, true, true));
        assert!(!menu_danger_btn_hit(0, true, false));
    }

    // 阅读态面板（1坊市 2行囊 3烙印 4帮助 5角色 6书志）任一开启：两钮一律不做命中
    #[test]
    fn any_reading_panel_blocks_both_btns() {
        for panel in 1u8..=6 {
            assert!(
                !menu_danger_btn_hit(panel, false, false),
                "panel={} 开始新的人生须避让",
                panel
            );
            assert!(
                !menu_danger_btn_hit(panel, true, true),
                "panel={} 承接前缘须避让（有档也不命中）",
                panel
            );
        }
    }
}

// ---------------- v1.16.26 直引号规范回归：引导页文案 ----------------
#[cfg(test)]
mod draw_tests {
    // 引导页文案须走「」直引号（v1.16.26 补漏：原 3145/3147 弯引号清账，文案已
    // 提升 HELP_LINES 常量）——全 17 行禁弯引号，修过的两行钉死含「」
    #[test]
    fn help_lines_quote_style() {
        // v1.16.29 刀B：HELP_LINES 表体收编 textpack → help_line(idx) 查表（17 行）
        for (i, _l) in (0usize..17).enumerate() {
            let l = crate::draw::help_line(i);
            assert!(
                !l.contains('“') && !l.contains('”'),
                "HELP_LINES[{}] 弯引号残留: {}",
                i,
                l
            );
        }
        assert!(
            crate::draw::help_line(0).contains("「可能的你」"),
            "轮回镜行引号漂移"
        );
        assert!(
            crate::draw::help_line(2).contains("「造化」"),
            "模拟毕行引号漂移"
        );
    }

    // v1.16.33 单1 刀3 NIZE 通道查表口逐位钉（内容=原表字面量；N 错行/共享错串
    // /越界钳制错位在此必红；行值取自真相源 tools/row_reader.py 口径）
    #[test]
    fn nize_guide_tables_pinned() {
        assert_eq!(super::guide_tab_label(0), "当前目标");
        assert_eq!(super::guide_tab_label(1), "章节书");
        // v1.16.40 长老委托第三签（真相源尾段追加 N2727，独立 N 非同值共享）
        assert_eq!(super::guide_tab_label(2), "门中差事");
        assert_eq!(
            super::guide_panel_line(0),
            "· 闭关修炼长修为，修为圆满可冲击破境，境境登高。"
        );
        assert_eq!(
            super::guide_panel_line(4),
            "· 与长老、掌柜、宁清霜、醉道人赠礼交好，增益长久生效。"
        );
        assert_eq!(
            super::guide_panel_note(),
            "主界面顶部金框横带常驻显示当前目标（现实界面）。"
        );
        // 越界 min 钳制（panic=abort 纪律防御臂）——v1.16.40 扩第三签后末签=2，
        // 钳制上沿随签数走（断言本意=越界恒等于末签）
        assert_eq!(super::guide_tab_label(usize::MAX), super::guide_tab_label(2));
    }
}

// ================= v1.16.46 演出包⑲ 破境金辉氛围层（brk_veil）钉测 =================
// 模组名注：v11645/v11646/v11647 前缀均已被占用（v11645_comp_multi_tests、
// v11646_bar_anim/pool_tests、施工单二/三 v11647_seg19_tests/v11647_loadspeed_tests
// ——charpanel v11646_button_fx_tests 亦占 v11646），本批钉测模块名顺延 v11648，
// 内容版本以本头注 v1.16.46 为准（照 charpanel.rs v11646_button_fx_tests 逐模块
// 头注款）。
// 钉面：①三角包络两端归零/对称/61 步全域有界/峰值 0.5 稳态逐比特（照
// weather_veil_tests 款）②消费口静态钉（峰值 α 0.08 单口+早退门）③brk 构建口
// 静态钉（game.rs brk:true 天劫/破境两路同点恰 1 + draw.rs card_brk 采样恰 1）。
#[cfg(test)]
mod v11648_brk_veil_tests {
    use super::brk_veil_a;

    fn prod_of(fname: &str) -> String {
        let root = env!("CARGO_MANIFEST_DIR");
        let src = std::fs::read_to_string(std::path::Path::new(root).join("src").join(fname))
            .unwrap_or_else(|e| panic!("读 src/{} 失败：{}", fname, e));
        // 行首 #[cfg(test)] 起至行首顶格 } 止滤测试区（照 press_sink prod 款）
        let mut out = String::new();
        let mut skip = false;
        for line in src.lines() {
            if line.starts_with("#[cfg(test)]") {
                skip = true;
                continue;
            }
            if skip {
                if line == "}" {
                    skip = false;
                }
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        out
    }

    // ── 包络：两端归零逐比特/对称/61 步全域有界/峰值恰 0.5 ──
    #[test]
    fn brk_veil_envelope() {
        assert_eq!(brk_veil_a(0.0).to_bits(), 0f32.to_bits(), "稳态 k=0 归零逐比特");
        assert_eq!(brk_veil_a(-0.3).to_bits(), 0f32.to_bits(), "负 k 归零");
        assert_eq!(brk_veil_a(1.0).to_bits(), 0f32.to_bits(), "开帧 k=1 包络归零（三角远端）");
        assert_eq!(brk_veil_a(0.5).to_bits(), 1f32.to_bits(), "峰值恰 k=0.5");
        // 对称：brk_veil_a(0.5−d)≈brk_veil_a(0.5+d)（f32 表示误差容差 1e-6——
        // 0.4f32 与 1.0−0.6f32 差 1 ULP 属表示层非逻辑层，照 weather_veil_tests
        // 近似口径；端点/峰值仍逐比特）
        for d_k in [0.05f32, 0.1, 0.2, 0.35, 0.45] {
            let (lv, rv) = (brk_veil_a(0.5 - d_k), brk_veil_a(0.5 + d_k));
            assert!(
                (lv - rv).abs() < 1e-6,
                "三角对称 d={}: {} vs {}",
                d_k,
                lv,
                rv
            );
        }
        // 61 步全域扫描：有界 [0,1] + 两段单调
        let mut prev = 0.0f32;
        let mut rising = true;
        for st_k in 0..=60usize {
            let v = brk_veil_a(st_k as f32 / 60.0);
            assert!((0.0..=1.0).contains(&v), "包络越界 k={}", st_k as f32 / 60.0);
            if rising && v < prev {
                rising = false;
            }
            assert!(
                if rising { v >= prev } else { v <= prev },
                "两段单调破坏 k={}",
                st_k as f32 / 60.0
            );
            prev = v;
        }
    }

    // ── 静态钉：消费口 + brk 门控构建口 ──
    #[test]
    fn brk_veil_static_pins() {
        let draw_p = prod_of("draw.rs");
        let count = |needle: &str| draw_p.matches(needle).count();
        assert_eq!(count("let brk_a = 0.08 * brk_veil_a(k);"), 1, "⑲ 峰值 α 0.08 单口");
        assert_eq!(count("if brk_a > 0.0 {"), 1, "⑲ 稳态早退门（a>0 才画先例）");
        assert_eq!(
            count("let (card_rare, card_stage, card_cat, card_brk) = {"),
            1,
            "card_brk 采样口恰 1（插层置于采样之后层级=暗幕上卡体下）"
        );
        let game_p = prod_of("game.rs");
        assert_eq!(
            game_p.matches("brk: true").count(),
            1,
            "brk 构建口恰 1（game.rs 天劫/破境两路同点——新增破境类卡须并入本口并扩钉）"
        );
    }
}

// ================= v1.16.46 演出包⑳ 结缘仪式金尘（bond_motes）钉测 =================
// 模组名注：同上 v11648 顺延缘由。钉面：①bond_mote_a 错峰包络稳态逐比特/全域
// 有界/错峰序②special17 门控三面静态钉（game.rs 构建口恰 2+npc.rs 消费口恰 1+
// draw.rs 采样与早退门）③立绘盒几何零位移钉（金尘为盒外叠层不改
// npc_portrait_box_a 实参——npc_clip_band 红线）。
#[cfg(test)]
mod v11648_bond_rite_tests {
    use super::bond_mote_a;

    fn prod_of(fname: &str) -> String {
        let root = env!("CARGO_MANIFEST_DIR");
        let src = std::fs::read_to_string(std::path::Path::new(root).join("src").join(fname))
            .unwrap_or_else(|e| panic!("读 src/{} 失败：{}", fname, e));
        let mut out = String::new();
        let mut skip = false;
        for line in src.lines() {
            if line.starts_with("#[cfg(test)]") {
                skip = true;
                continue;
            }
            if skip {
                if line == "}" {
                    skip = false;
                }
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        out
    }

    // ── 错峰包络：稳态全零逐比特/有界/错峰序（idx 大者先入峰）──
    #[test]
    fn bond_mote_envelope() {
        // 稳态 k=0：全粒归零逐比特（整段早退前提）
        for idx in 0..8usize {
            assert_eq!(
                bond_mote_a(0.0, idx).to_bits(),
                0f32.to_bits(),
                "稳态金尘 idx={} 归零逐比特",
                idx
            );
        }
        // 61 步 × 8 粒全域扫描有界 [0,1]
        for st_k in 0..=60usize {
            for idx in 0..8usize {
                let v = bond_mote_a(st_k as f32 / 60.0, idx);
                assert!(
                    (0.0..=1.0).contains(&v),
                    "金尘包络越界 k={} idx={}",
                    st_k as f32 / 60.0,
                    idx
                );
            }
        }
        // 错峰序：k=0.85 时 idx7 已入峰而 idx0 未及（idx 大者早入峰）；k=0.5 时
        // idx0 恰峰值
        assert!(
            bond_mote_a(0.85, 7) > bond_mote_a(0.85, 0),
            "错峰序破坏：idx 大者应先入峰"
        );
        assert_eq!(bond_mote_a(0.5, 0).to_bits(), 1f32.to_bits(), "idx0 峰值恰 k=0.5");
    }

    // ── 静态钉：special17 门控三面 + 立绘盒零位移 ──
    #[test]
    fn bond_rite_static_pins() {
        let draw_p = prod_of("draw.rs");
        let count = |needle: &str| draw_p.matches(needle).count();
        assert_eq!(
            count("let card_rite = g"),
            1,
            "⑳ 门控采样口恰 1（opts 静态扫描 special==17）"
        );
        assert_eq!(
            count("if card_rite && k > 0.0 && art_a > 0.0 {"),
            1,
            "⑳ 稳态早退门（k=0/art_a=0 整段不落像素）"
        );
        assert_eq!(count("bond_mote_a(k, mote_k)"), 1, "⑳ 金尘错峰消费恰 1");
        assert_eq!(
            count("160.0 + dy + art_sink,\n            220.0,\n            420.0,\n            420.0,\n            art_a,"),
            1,
            "立绘盒几何零位移（金尘为盒外叠层不改 npc_portrait_box_a 实参）"
        );
        let game_p = prod_of("game.rs");
        assert_eq!(
            game_p.matches("e1.special = 17;").count(),
            2,
            "special17 构建口恰 2（模拟 28 臂+现实 rencounter 认下臂——新增结缘口须并入本钉）"
        );
        let npc_p = prod_of("npc.rs");
        assert_eq!(
            npc_p.matches("e.special == 17").count(),
            1,
            "special17 消费口恰 1（apply_reff 前置独占臂）"
        );
    }
}

// ================= v1.16.47 演出包㉑ 宿敌斩杀·血芒四沿钉测 =================
// 模组名注：v11645 前缀已被 v11645_comp_multi_tests 占用、v11646 前缀已被
// v11646_bar_anim/v11646_pool/v11646_button_fx 三模组占用、v11647 前缀已被
// v11647_seg19/v11647_loadspeed 两模组占用、v11648 前缀已被 v11648_brk_veil/
// v11648_bond_rite（draw.rs 两）/v11648_button_jewel（charpanel.rs）/v11648_comp_ui
// 四模组占用、v11649 前缀已被 v11649_seg20_tests（game.rs 施工单二）占用，本批
// 钉测模块名顺延 v11650，内容版本以本头注 v1.16.47 为准（照 v11647_seg19_tests
// 逐模块头注款）。
// 钉面：①置值口恰 2（revenge_win 现实寻仇胜+sim_neme_win_settle 模拟反杀胜）
// ②消费早退门/峰值系数单口（draw_banner 顶层独立段）③save.rs 零引用（会话态
// 不进存档红线）④包络复用单源（prod 不新增第二包络函数——照 ⑳ bond_mote_a
// 单源勿双写纪律；共用 brk_veil_a 三角由 v11648_brk_veil_tests 既有钉看守不重写；
// 本头注措辞刻意避开 prod 扫描 needle 字样，头注行位于 #[cfg(test)] 之前不入
// prod 过滤面）
#[cfg(test)]
mod v11650_neme_veil_tests {
    fn prod_of(fname: &str) -> String {
        let root = env!("CARGO_MANIFEST_DIR");
        let src = std::fs::read_to_string(std::path::Path::new(root).join("src").join(fname))
            .unwrap_or_else(|e| panic!("读 src/{} 失败：{}", fname, e));
        let mut out = String::new();
        let mut skip = false;
        for line in src.lines() {
            if line.starts_with("#[cfg(test)]") {
                skip = true;
                continue;
            }
            if skip {
                if line == "}" {
                    skip = false;
                }
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        out
    }

    #[test]
    fn neme_veil_static_pins() {
        let game_p = prod_of("game.rs");
        assert_eq!(
            game_p
                .matches("self.neme_veil = crate::game::NEME_VEIL_T;")
                .count(),
            2,
            "㉑ 置值口恰 2（现实寻仇胜+模拟反杀胜——新增斩杀口须并入本钉）"
        );
        let draw_p = prod_of("draw.rs");
        assert_eq!(
            draw_p.matches("let nv_a = 0.10 * brk_veil_a").count(),
            1,
            "㉑ 峰值系数单口（红边 α 0.10×共用包络恰 1）"
        );
        assert_eq!(
            draw_p.matches("if nv_a > 0.0 {").count(),
            1,
            "㉑ 消费早退门恰 1（稳态 neme_veil==0 整段不落像素）"
        );
        assert_eq!(
            draw_p
                .matches("draw_rectangle(0.0, V_H - 10.0, V_W, 10.0, red);")
                .count(),
            1,
            "㉑ 下沿窄边在位（四沿 10px，量级之内；本消息全用既有字集防钉测字符进运行期字体）"
        );
        assert!(
            !draw_p.contains("fn neme_veil_a"),
            "㉑ 不得新增第二包络函数（复用 brk_veil_a 单源勿双写）"
        );
        let save_p = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("save.rs"),
        )
        .unwrap_or_else(|e| panic!("读 src/save.rs 失败：{}", e));
        assert!(
            !save_p.contains("neme_veil"),
            "㉑ save.rs 零引用（会话态不进存档红线）"
        );
        let main_p = prod_of("main.rs");
        assert_eq!(
            main_p
                .matches("g.neme_veil = (g.neme_veil - dt).max(0.0);")
                .count(),
            1,
            "㉑ 主循环递减口恰 1（照 weather_anim main.rs 先例）"
        );
    }
}

// ================= v1.16.47 演出包㉒ 周天吐纳·气脉呼吸钉测 =================
// 模组名注：同上 v11650 顺延缘由（占用链见 v11650_neme_veil_tests 头注）。
// 钉面：①置值口恰 1 且在 act_cultivate died 早退之后（成功段=死亡年无呼吸；
// 勿挪 tick_auto 外层防结算/双修误触）②递减口恰 1（main.rs）③消费 α 单口+早退
// 门+halo 几何恰 1（render_left 修为条）④save.rs 零引用⑤包络复用单源
#[cfg(test)]
mod v11650_qi_breath_tests {
    fn prod_of(fname: &str) -> String {
        let root = env!("CARGO_MANIFEST_DIR");
        let src = std::fs::read_to_string(std::path::Path::new(root).join("src").join(fname))
            .unwrap_or_else(|e| panic!("读 src/{} 失败：{}", fname, e));
        let mut out = String::new();
        let mut skip = false;
        for line in src.lines() {
            if line.starts_with("#[cfg(test)]") {
                skip = true;
                continue;
            }
            if skip {
                if line == "}" {
                    skip = false;
                }
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        out
    }

    #[test]
    fn qi_breath_static_pins() {
        let game_p = prod_of("game.rs");
        assert_eq!(
            game_p
                .matches("self.qi_breath = crate::game::QI_BREATH_T;")
                .count(),
            1,
            "㉒ 置值口恰 1（act_cultivate 成功段单口——勿挪 tick_auto 外层）"
        );
        // 置值口须在 died 早退之后（成功段）：置值位点 > died 早退块位点
        let set_at = game_p
            .find("self.qi_breath = crate::game::QI_BREATH_T;")
            .expect("㉒ 置值口缺失");
        let died_at = game_p.find("if died {").expect("㉒ died 早退块缺失");
        assert!(
            set_at > died_at,
            "㉒ 置值口须在 died 早退之后（成功段=死亡年无呼吸）"
        );
        let draw_p = prod_of("draw.rs");
        assert_eq!(
            draw_p.matches("let qb_a = 0.12 * brk_veil_a").count(),
            1,
            "㉒ 消费 α 单口（halo 0.12×共用包络恰 1）"
        );
        assert_eq!(
            draw_p.matches("if qb_a > 0.0 {").count(),
            1,
            "㉒ 消费早退门恰 1（稳态 qi_breath==0 整段不落像素）"
        );
        assert_eq!(
            draw_p
                .matches(
                    "draw_rectangle_lines(26.0, 272.0, LEFT_W - 36.0, 24.0, 2.0, fade(C_CYAN, qb_a));"
                )
                .count(),
            1,
            "㉒ 修为条外扩 2px halo 几何在位（条 28,274,320,20 外扩为 26,272,324,24）"
        );
        assert!(
            !draw_p.contains("fn qi_breath_a"),
            "㉒ 不得新增第二包络函数（复用 brk_veil_a 单源勿双写）"
        );
        let save_p = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("save.rs"),
        )
        .unwrap_or_else(|e| panic!("读 src/save.rs 失败：{}", e));
        assert!(
            !save_p.contains("qi_breath"),
            "㉒ save.rs 零引用（会话态不进存档红线）"
        );
        let main_p = prod_of("main.rs");
        assert_eq!(
            main_p
                .matches("g.qi_breath = (g.qi_breath - dt).max(0.0);")
                .count(),
            1,
            "㉒ 主循环递减口恰 1（照 weather_anim main.rs 先例）"
        );
    }
}

// ================= v1.16.47 UX 沉浸感（多道侣两件+花样 A/D）钉测 =================
// 模组名注：同上 v11650 顺延缘由（占用链见 v11650_neme_veil_tests 头注）。
// 钉面：①多道侣提名一 cur 行左缘竖条+满编顺位行标恰 1 ②多道侣提名二副伴行
// 分段上色两段在位（伴名 C_PURPLE）③花样 A hot 态内缩四角金角饰（else-if 互斥
// 链内、C_GOLD 色 needle 唯一勿复用花样 B 的 214,200,244）④花样 D 两 toggle on
// 态内底金下划线恰 2（本头注措辞刻意避开 prod 扫描 needle 字样——头注行位于
// #[cfg(test)] 之前不入 prod 过滤面）
#[cfg(test)]
mod v11650_ux_pins_tests {
    fn prod_of(fname: &str) -> String {
        let root = env!("CARGO_MANIFEST_DIR");
        let src = std::fs::read_to_string(std::path::Path::new(root).join("src").join(fname))
            .unwrap_or_else(|e| panic!("读 src/{} 失败：{}", fname, e));
        let mut out = String::new();
        let mut skip = false;
        for line in src.lines() {
            if line.starts_with("#[cfg(test)]") {
                skip = true;
                continue;
            }
            if skip {
                if line == "}" {
                    skip = false;
                }
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        out
    }

    #[test]
    fn ux_immersion_static_pins() {
        let char_p = prod_of("charpanel.rs");
        assert_eq!(
            char_p.matches("· 页渐疏").count(),
            1,
            "多道侣一：满编顺位行标恰 1（缘谱行 tag 接排）"
        );
        assert_eq!(
            char_p
                .matches("draw_rectangle(x + 2.0, y + 1.0, 3.0, COMP_ROW_H - 2.0, lead);")
                .count(),
            1,
            "多道侣一：cur 行左缘 3px 竖条恰 1（多伴态门控内）"
        );
        assert_eq!(
            char_p.matches("let seg_pre = \"另有同页：\";").count(),
            1,
            "多道侣二：副伴行分段前缀在位（N2798 拆段首段）"
        );
        assert_eq!(
            char_p
                .matches("txt(&extra_names, sx, y + 240.0, 14, C_PURPLE);")
                .count(),
            1,
            "多道侣二：伴名 C_PURPLE 段恰 1（照缘谱行名色语言）"
        );
        let draw_p = prod_of("draw.rs");
        // v1.16.48 复查① 同批改 needle：jgold α 按 enabled 分档（0.5/0.15）——
        // 禁用态 hover 金角饰弱化（金饰=可点语言自洽），恰 1 口径不变
        assert_eq!(
            draw_p
                .matches("let jgold = fade(C_GOLD, (if enabled { 0.5 } else { 0.15 }) * a);")
                .count(),
            1,
            "花样 A+复查①：hot 态角饰金 α 分档单口恰 1（C_GOLD 勿复用花样 B 色值）"
        );
        // v1.16.48 复查① 同批：禁用态顶光弱化 0.25→0.10（needle 由旧单值改分档
        // 形态唯一短锚——多行 fade 结构以单行分档表达式为锚防跨行失配）
        assert_eq!(
            draw_p.matches("(if enabled { 0.25 } else { 0.10 }) * a").count(),
            1,
            "花样 B+复查①：顶光分档形态恰 1（禁用 hover 弱化同批）"
        );
        assert_eq!(
            draw_p.matches("for (ax, ay, adx, ady) in [").count(),
            1,
            "花样 A：四角 L 形元组循环恰 1（照 panel_card 角饰语言小号款）"
        );
        assert_eq!(
            draw_p
                .matches("draw_rectangle(bx, 592.0 + 44.0 - 3.0, 140.0, 3.0, C_GOLD);")
                .count(),
            1,
            "花样 D：音乐 toggle on 态下划线恰 1（内底 y+h−3 照周天款）"
        );
        assert_eq!(
            draw_p
                .matches("draw_rectangle(bx + 155.0, 592.0 + 44.0 - 3.0, 140.0, 3.0, C_GOLD);")
                .count(),
            1,
            "花样 D：音效 toggle on 态下划线恰 1"
        );
    }
}

// ================= v1.16.48 演出包㉓㉔+UX 细节钉测 =================
// 模组名注：v11645..v11651 前缀均已被占用（占用链全账见上方 v11650_neme_veil_
// tests 头注与 game.rs 段二十一区头注——v11650 三模组在 draw.rs、
// v11651_charcount_pins_tests 在 game.rs）；v11652 已被 v11652_seg21_tests
//（game.rs，同轮施工单二段二十一扩池批）占用——本批钉测模块名顺延 v11653，内
// 容版本以本头注为准（占用链纪律：每模组头注可独立回溯全链；本头注措辞刻意避
// 开 prod 扫描 needle 字样照 ㉑ 头注款）。
// 钉面：㉓ 置值恰 1+位点序（launch_sim 两早退之后）/常量恰 1/字段恰 1/递减恰 1/
// α 双口恰 2（draw_draft 卡环+draw_sim 金环）/早退门恰 2/save.rs 零引用/无第二
// 包络；㉔ 置值恰 1+位点序（四早退末位 life 缺之后、出口 remove 之前）/常量恰 1/
// 字段恰 1/递减恰 1/α 口恰 1/早退门恰 1/save.rs 零引用/无第二包络——七钉照
// v11650 两模组款
#[cfg(test)]
mod v11653_draft_pact_tests {
    fn prod_of(fname: &str) -> String {
        let root = env!("CARGO_MANIFEST_DIR");
        let src = std::fs::read_to_string(std::path::Path::new(root).join("src").join(fname))
            .unwrap_or_else(|e| panic!("读 src/{} 失败：{}", fname, e));
        let mut out = String::new();
        let mut skip = false;
        for line in src.lines() {
            if line.starts_with("#[cfg(test)]") {
                skip = true;
                continue;
            }
            if skip {
                if line == "}" {
                    skip = false;
                }
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        out
    }

    #[test]
    fn draft_pact_static_pins() {
        let game_p = prod_of("game.rs");
        assert_eq!(
            game_p.matches("pub const DRAFT_PACT_T: f32 = 0.9;").count(),
            1,
            "㉓ 常量恰 1（照 NEME_VEIL/QI_BREATH 款带窗长头注）"
        );
        assert_eq!(game_p.matches("pub draft_pact: f32,").count(), 1, "㉓ 字段恰 1");
        let set_needle = "self.draft_pact = DRAFT_PACT_T;";
        assert_eq!(game_p.matches(set_needle).count(), 1, "㉓ 置值口恰 1（launch_sim 成功段单口）");
        let early_at = game_p
            .find("if self.picked.is_empty() || self.meta.sim_points < self.sim_cost() {")
            .expect("㉓ launch_sim 头部早退在位");
        let set_at = game_p.find(set_needle).expect("㉓ 置值行在位");
        assert!(
            set_at > early_at,
            "㉓ 位点序：置值须在 picked 空/点数不足两早退之后（照㉒ set>died 款防挪外层）"
        );
        assert_eq!(
            game_p.matches("fn draft_pact_a").count(),
            0,
            "㉓ 无第二包络（共用 brk_veil_a 单源勿双写）"
        );
        let save_p = prod_of("save.rs");
        assert_eq!(save_p.matches("draft_pact").count(), 0, "㉓ 会话态不进存档（save.rs 零引用）");
        let main_p = prod_of("main.rs");
        assert_eq!(
            main_p.matches("if g.draft_pact > 0.0 {").count(),
            1,
            "㉓ 递减早退门恰 1"
        );
        assert_eq!(
            main_p.matches("g.draft_pact = (g.draft_pact - dt).max(0.0);").count(),
            1,
            "㉓ 递减行恰 1（主循环照 weather_anim 先例）"
        );
        let draw_p = prod_of("draw.rs");
        assert_eq!(
            draw_p
                .matches("0.12 * brk_veil_a((g.draft_pact / crate::game::DRAFT_PACT_T)")
                .count(),
            2,
            "㉓ α 双口恰 2（draw_draft 卡环+draw_sim 金环——新增消费口须并入本钉）"
        );
        assert_eq!(draw_p.matches("if dp_a > 0.0 {").count(), 2, "㉓ 早退门恰 2（稳态零像素）");
    }
}

#[cfg(test)]
mod v11653_inh_spark_tests {
    fn prod_of(fname: &str) -> String {
        let root = env!("CARGO_MANIFEST_DIR");
        let src = std::fs::read_to_string(std::path::Path::new(root).join("src").join(fname))
            .unwrap_or_else(|e| panic!("读 src/{} 失败：{}", fname, e));
        let mut out = String::new();
        let mut skip = false;
        for line in src.lines() {
            if line.starts_with("#[cfg(test)]") {
                skip = true;
                continue;
            }
            if skip {
                if line == "}" {
                    skip = false;
                }
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        out
    }

    #[test]
    fn inh_spark_static_pins() {
        let game_p = prod_of("game.rs");
        assert_eq!(
            game_p.matches("pub const INH_SPARK_T: f32 = 1.0;").count(),
            1,
            "㉔ 常量恰 1"
        );
        assert_eq!(game_p.matches("pub inh_spark: f32,").count(), 1, "㉔ 字段恰 1");
        let set_needle = "self.inh_spark = INH_SPARK_T;";
        assert_eq!(game_p.matches(set_needle).count(), 1, "㉔ 置值口恰 1（八分支收敛唯一出口单口）");
        let early_at = game_p
            .find("let Some(life) = self.life.as_mut() else {")
            .expect("㉔ take_inherit 四早退末位（life 缺）在位");
        let exit_at = game_p.find("self.inherit_opts.remove(i);").expect("㉔ 出口 remove 行在位");
        let set_at = game_p.find(set_needle).expect("㉔ 置值行在位");
        assert!(
            set_at > early_at && set_at < exit_at,
            "㉔ 位点序：置值须在四早退（picked 满/opts 缺/sim_result 缺/life 缺）之后、出口 remove 之前（防挪进单分支）"
        );
        assert_eq!(
            game_p.matches("fn inh_spark_a").count(),
            0,
            "㉔ 无第二包络（共用 brk_veil_a 单源勿双写）"
        );
        let save_p = prod_of("save.rs");
        assert_eq!(save_p.matches("inh_spark").count(), 0, "㉔ 会话态不进存档（save.rs 零引用）");
        let main_p = prod_of("main.rs");
        assert_eq!(main_p.matches("if g.inh_spark > 0.0 {").count(), 1, "㉔ 递减早退门恰 1");
        assert_eq!(
            main_p.matches("g.inh_spark = (g.inh_spark - dt).max(0.0);").count(),
            1,
            "㉔ 递减行恰 1"
        );
        let draw_p = prod_of("draw.rs");
        assert_eq!(
            draw_p
                .matches("0.12 * brk_veil_a((g.inh_spark / crate::game::INH_SPARK_T)")
                .count(),
            1,
            "㉔ α 口恰 1（draw_inherit 此生卡描边——新增消费口须并入本钉）"
        );
        assert_eq!(draw_p.matches("if is_a > 0.0 {").count(), 1, "㉔ 早退门恰 1（稳态零像素）");
    }
}

// ================= v1.16.49 演出包㉕㉖+UX 沉浸感批钉测 =================
// 模组名注：v11652 已被 v11652_seg21_tests（game.rs，段二十一扩池批）占用、
// v11653 已被 v11653_draft_pact_tests/v11653_inh_spark_tests（draw.rs，演出包
// ㉓㉔批）占用、v11654 已被 v11654_len_batch_tests（game.rs，剧情加长批）占用、
// v11655 已被 v11655_seg22_tests（game.rs，段二十二扩池批）占用——勘察三提名
// 时 v11655 尚空闲、施工单二先行落位，本批钉测模块名顺延 v11656，内容版本以本
// 头注 v1.16.49 为准（占用链纪律照 draw.rs:5286-5291 款；措辞避开消费面扫描
// 子串，照 ㉑ 头注款防自指）。
// 钉面：①㉕ 坊市成交·金铃涟漪契约链（常量/字段/置值恰 1+位点序照㉓款/save.rs
// 零引用/递减恰 1/消费口恰 1+早退门恰 1）②㉖ 远征启程·霞光启幡契约链（同款
// +exp_done/途中步不置值钉面写死）③㉓遮蔽修复位点序（勘察六 finding① 根因闭
// 环：sim 口须在日志面板底之后——防回挪；v11653「α 双口恰 2」照旧不受挪位影
// 响）④UX 沉浸感三件静态钉（遗泽承伴轮替文案闭环/道侣页 cur 行同页世接排/落
// 笔钮 ready 门控 accent）
#[cfg(test)]
mod v11656_market_depart_tests {
    fn prod_of(fname: &str) -> String {
        let root = env!("CARGO_MANIFEST_DIR");
        let src = std::fs::read_to_string(std::path::Path::new(root).join("src").join(fname))
            .unwrap_or_else(|e| panic!("读 src/{} 失败：{}", fname, e));
        // 行首 #[cfg(test)] 起至行首顶格 } 止滤测试区（照 v11653 prod 款——
        // 不滤则本模组自身 needle 定义行入 matches 面致计数自指，初版已踩：
        // ring needle 数出 4（生产 2+v11653 def 1+本模组 def 1）
        let mut out = String::new();
        let mut skip = false;
        for line in src.lines() {
            if line.starts_with("#[cfg(test)]") {
                skip = true;
                continue;
            }
            if skip {
                if line == "}" {
                    skip = false;
                }
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        out
    }

    // ── ㉕ 坊市成交·金铃涟漪契约链（照 v11653_draft_pact 款七钉） ──
    #[test]
    fn market_ping_static_pins() {
        let game_p = prod_of("game.rs");
        assert_eq!(
            game_p.matches("pub const MARKET_PING_T: f32 = 0.9;").count(),
            1,
            "㉕ 常量恰 1（照 ㉓ DRAFT_PACT_T 款带窗长头注）"
        );
        assert_eq!(game_p.matches("pub market_ping: f32,").count(), 1, "㉕ 字段恰 1");
        let set_needle = "self.market_ping = crate::game::MARKET_PING_T;";
        assert_eq!(
            game_p.matches(set_needle).count(),
            1,
            "㉕ 置值口恰 1（buy_pill 成功段单口）"
        );
        let early_at = game_p
            .find("crate::npc::NPC_MERCHANT, MERCHANT_BUY_FAV")
            .expect("㉕ buy_pill 成交好感行在位（购丹特有锚，全文件恰 1）");
        let set_at = game_p.find(set_needle).expect("㉕ 置值行在位");
        assert!(
            set_at > early_at,
            "㉕ 位点序：置值须在成交处理链（灵石不足早退之后的成交段）之后（照 ㉓ set>early 款防挪外层；早期锚若 !can 串非购丹独占改用本特有锚）"
        );
        assert_eq!(
            game_p.matches("pub fn market_ping_a").count(),
            0,
            "㉕ 无第二包络（共用 brk_veil_a 单源勿双写）"
        );
        let save_p = prod_of("save.rs");
        assert_eq!(
            save_p.matches("market_ping").count(),
            0,
            "㉕ 会话态不进存档（save.rs 零引用）"
        );
        let main_p = prod_of("main.rs");
        assert_eq!(
            main_p.matches("if g.market_ping > 0.0 {").count(),
            1,
            "㉕ 递减早退门恰 1"
        );
        assert_eq!(
            main_p.matches("g.market_ping = (g.market_ping - dt).max(0.0);").count(),
            1,
            "㉕ 递减行恰 1（主循环照 qi_breath 先例）"
        );
        let draw_p = prod_of("draw.rs");
        assert_eq!(
            draw_p
                .matches("g.market_ping / crate::game::MARKET_PING_T")
                .count(),
            1,
            "㉕ 消费口恰 1（丹药阁六行循环内单口——扩藏经阁/点数页须三置值口并并入本钉）"
        );
        assert_eq!(draw_p.matches("if mp_a > 0.0 {").count(), 1, "㉕ 早退门恰 1（稳态零像素）");
    }

    // ── ㉖ 远征启程·霞光启幡契约链（置值口在 world.rs，prod 任意文件名照 v11653 款） ──
    #[test]
    fn exp_depart_static_pins() {
        let world_p = prod_of("world.rs");
        let set_needle = "g.exp_depart = crate::game::EXP_DEPART_T;";
        assert_eq!(world_p.matches(set_needle).count(), 1, "㉖ 置值口恰 1（exp_start 成功段单口）");
        let exped_at = world_p
            .find("g.world.exp = Some(Exped {")
            .expect("㉖ exp=Some 启程行在位");
        let set_at = world_p.find(set_needle).expect("㉖ 置值行在位");
        assert!(
            set_at > exped_at,
            "㉖ 位点序：置值须在三早退后的 exp=Some 之后（防挪早退前）"
        );
        // 语义写死：exp_done/途中 exp_choose 均不置值（每远征仅启程一口）——
        // 全文件置值口恰 1 已由上行断言覆盖，此注为后人扩口红线留档
        let game_p = prod_of("game.rs");
        assert_eq!(
            game_p.matches("pub const EXP_DEPART_T: f32 = 1.2;").count(),
            1,
            "㉖ 常量恰 1"
        );
        assert_eq!(game_p.matches("pub exp_depart: f32,").count(), 1, "㉖ 字段恰 1");
        let save_p = prod_of("save.rs");
        assert_eq!(save_p.matches("exp_depart").count(), 0, "㉖ 会话态不进存档（save.rs 零引用）");
        let main_p = prod_of("main.rs");
        assert_eq!(
            main_p.matches("if g.exp_depart > 0.0 {").count(),
            1,
            "㉖ 递减早退门恰 1"
        );
        assert_eq!(
            main_p.matches("g.exp_depart = (g.exp_depart - dt).max(0.0);").count(),
            1,
            "㉖ 递减行恰 1"
        );
        let draw_p = prod_of("draw.rs");
        assert_eq!(
            draw_p.matches("ed / crate::game::EXP_DEPART_T").count(),
            2,
            "㉖ 消费口恰 2（draw_depart_beam 内 α 与 prog 双算同源同一口——新增消费口须并入本钉）"
        );
        assert_eq!(draw_p.matches("if ed <= 0.0 {").count(), 1, "㉖ 早退门恰 1（稳态零像素）");
        assert_eq!(
            world_p
                .matches("crate::draw::draw_depart_beam(g, 8.0, 60.0, 1264.0, 480.0);")
                .count(),
            1,
            "㉖ 世界图调用恰 1（置于⑫ draw_weather_veil 口之后不动既有叠序）"
        );
    }

    // ── ㉓遮蔽修复位点序（勘察六 finding①：sim 口须在日志面板底之后，防回挪） ──
    #[test]
    fn veil_sim_ring_unoccluded_pin() {
        let draw_p = prod_of("draw.rs");
        let ring_needle = "0.12 * brk_veil_a((g.draft_pact / crate::game::DRAFT_PACT_T)";
        assert_eq!(
            draw_p.matches(ring_needle).count(),
            2,
            "㉓ α 双口恰 2（打稿卡环+sim 金环——v11653 同款计数，挪位不改面）"
        );
        let log_needle = "Color::from_rgba(14, 10, 26, 220)";
        assert_eq!(
            draw_p.matches(log_needle).count(),
            1,
            "㉓ 日志面板底色行恰 1（位点序锚）"
        );
        let log_at = draw_p.find(log_needle).expect("㉓ 日志底行在位");
        let ring_sim_at = draw_p.rfind(ring_needle).expect("㉓ sim 口在位");
        assert!(
            ring_sim_at > log_at,
            "㉓ 遮蔽修复位点序：sim 金环口须在日志面板绘制之后（勘察六 finding① 防回挪——回挪即环被顶栏/日志面板覆盖复位旧伪影）"
        );
    }

    // ── UX 沉浸感三件静态钉（多道侣两提名+落笔 accent） ──
    #[test]
    fn ux_immersive_pins() {
        let game_p = prod_of("game.rs");
        assert_eq!(
            game_p.matches("let (_, evicted) = life.bond_slot(cn);").count(),
            1,
            "多道侣提名一：遗泽臂轮替取回恰 1（sim/现实两臂既有款不动）"
        );
        assert_eq!(
            game_p
                .matches("evf(2795, &[&(old.as_str()) as &dyn core::fmt::Display, &(cn) as &dyn core::fmt::Display])")
                .count(),
            3,
            "多道侣提名一：轮替同串恰 3（sim 28 臂/现实邂逅臂/遗泽臂——文案三通道闭环）"
        );
        let char_p = prod_of("charpanel.rs");
        assert_eq!(
            char_p.matches("情谊 {} · 同页 {} 世").count(),
            1,
            "多道侣提名二：cur 行同页世接排恰 1（multi 态限定，复用既有字集零新字形）"
        );
        let draw_p = prod_of("draw.rs");
        assert_eq!(
            draw_p.matches("Some(\"开始此世模拟\"),\n        ready,\n        ready,").count(),
            1,
            "按钮花样 A：落笔钮 ready 门控 accent 恰 1（未选态逐比特一致照 button 委托先例）"
        );
    }
}

// ================= v1.16.50 演出包㉗㉘+UX 沉浸感批钉测 =================
// 模组名注：v11645..v11657 全链占用（占用链见 draw.rs v11656 头注+game.rs
// v11657 头注）；v11658 原系勘察三提名时空闲，但段二十三扩池批（施工单二）
// 先行落位 v11658_seg23_tests（game.rs）——本批钉测模块名顺延 v11659，内容版
// 本以本头注 v1.16.50 为准（占用链纪律照 v11656 头注款；措辞避开消费面扫描
// 子串防自指）。
// 钉面：①㉗ 远征收手·霞光合拢契约链（常量/字段/置值恰 1+位点序 set<exp=None
// 照㉔收敛出口款/save.rs 零引用/递减两恰 1/消费口恰 2+早退门恰 1/世界图调用
// 恰 1+exp_start 区零置值旁证——勿碰 exp_depart 照 v11656 语义写死红线）
// ②㉘ 离闭归来·结算卡浮现契约链（常量/字段/置值恰 1+位点序 set>offline_card
// =Some/save.rs 零引用/递减两恰 1/消费口恰 1+暗幕先行式恰 1）③㉖ 覆盖层增强
// B 案钉（面板域二次调用恰 1——v11656「世界图调用恰 1」全参精确串不受影响照
// 旧）④㉕ 涟漪末行伪影修复钉（勘察六 finding① 根因闭环：末行门恰 1+早退门字
// 面照旧旁证）⑤UX 四件静态钉（步进点 cur 脉动/世界图 hot 外圈呼吸/页签三组
// 加亮段/拾取卡标题插值）⑥修为圆满两口文案钉（draw 左栏整句+charpanel 破境条
// 两字，同批同语义防半改）
#[cfg(test)]
mod v11659_ux_immersion_tests {
    fn prod_of(fname: &str) -> String {
        let root = env!("CARGO_MANIFEST_DIR");
        let src = std::fs::read_to_string(std::path::Path::new(root).join("src").join(fname))
            .unwrap_or_else(|e| panic!("读 src/{} 失败：{}", fname, e));
        // 行首 #[cfg(test)] 起至行首顶格 } 止滤测试区（照 v11656 prod 款防自指）
        let mut out = String::new();
        let mut skip = false;
        for line in src.lines() {
            if line.starts_with("#[cfg(test)]") {
                skip = true;
                continue;
            }
            if skip {
                if line == "}" {
                    skip = false;
                }
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        out
    }

    // ── ㉗ 远征收手·霞光合拢契约链（照 v11656 exp_depart 款七钉，方向/色反） ──
    #[test]
    fn exp_return_static_pins() {
        let game_p = prod_of("game.rs");
        assert_eq!(
            game_p.matches("pub const EXP_RETURN_T: f32 = 1.0;").count(),
            1,
            "㉗ 常量恰 1（照 ㉖ EXP_DEPART_T 款带窗长头注）"
        );
        assert_eq!(game_p.matches("pub exp_return: f32,").count(), 1, "㉗ 字段恰 1");
        let world_p = prod_of("world.rs");
        let set_needle = "g.exp_return = crate::game::EXP_RETURN_T;";
        assert_eq!(
            world_p.matches(set_needle).count(),
            1,
            "㉗ 置值口恰 1（exp_quit 收敛口单口——链中收手与链尾归返共用）"
        );
        let set_at = world_p.find(set_needle).expect("㉗ 置值行在位");
        // exp_quit 特有锚（多行形态——「g.world.exp = None;」另有 real_die 死亡
        // 清场一处 :90 区，须锚 exp_quit 收敛口特有的 cache 清行连缀形态）
        let none_at = world_p
            .find("g.world.exp = None;\n    g.exp_p_cache = None;")
            .expect("㉗ exp_quit 收敛清场行对在位");
        assert!(
            set_at < none_at,
            "㉗ 位点序：置值须在收敛清场 g.world.exp=None 之前（照㉔收敛出口款防挪清场后）"
        );
        // 语义写死旁证：exp_start 区零置值（exp_return 在 world.rs 仅置值口一处）
        assert_eq!(
            world_p.matches("g.exp_return").count(),
            1,
            "㉗ world.rs 仅置值口一处引用（exp_start/exp_choose 均不置值=每远征仅收手一口）"
        );
        let save_p = prod_of("save.rs");
        assert_eq!(
            save_p.matches("exp_return").count(),
            0,
            "㉗ 会话态不进存档（save.rs 零引用）"
        );
        let main_p = prod_of("main.rs");
        assert_eq!(
            main_p.matches("if g.exp_return > 0.0 {").count(),
            1,
            "㉗ 递减早退门恰 1"
        );
        assert_eq!(
            main_p.matches("g.exp_return = (g.exp_return - dt).max(0.0);").count(),
            1,
            "㉗ 递减行恰 1"
        );
        let draw_p = prod_of("draw.rs");
        assert_eq!(
            draw_p.matches("er / crate::game::EXP_RETURN_T").count(),
            2,
            "㉗ 消费口恰 2（draw_return_beam 内 α 与 prog 双算同源同一口）"
        );
        assert_eq!(draw_p.matches("if er <= 0.0 {").count(), 1, "㉗ 早退门恰 1（稳态零像素）");
        assert_eq!(
            world_p
                .matches("crate::draw::draw_return_beam(g, 8.0, 60.0, 1264.0, 480.0);")
                .count(),
            1,
            "㉗ 世界图调用恰 1（置于㉖ draw_depart_beam 口之后不动既有叠序）"
        );
        // 反向红线旁证：㉗ 勿碰 exp_depart——置值口恰 1 由 v11656 续守，此处旁证
        assert_eq!(
            world_p.matches("g.exp_depart = crate::game::EXP_DEPART_T;").count(),
            1,
            "㉗ 交叉红线：exp_depart 置值仍恰 1（exp_start 启程口未被本批改动）"
        );
    }

    // ── ㉘ 离闭归来·结算卡浮现契约链（置值口在 game.rs resume_life n>=10 臂） ──
    #[test]
    fn offline_anim_static_pins() {
        let game_p = prod_of("game.rs");
        assert_eq!(
            game_p.matches("pub const OFFLINE_ANIM_T: f32 = 0.22;").count(),
            1,
            "㉘ 常量恰 1（照 zoom_anim 0.22 款带窗长头注）"
        );
        assert_eq!(game_p.matches("pub offline_anim: f32,").count(), 1, "㉘ 字段恰 1");
        let set_needle = "self.offline_anim = crate::game::OFFLINE_ANIM_T;";
        assert_eq!(
            game_p.matches(set_needle).count(),
            1,
            "㉘ 置值口恰 1（resume_life n>=10 臂单口——else 短载分支不置值）"
        );
        let card_at = game_p
            .find("self.offline_card = Some(OfflineReport { years: n, qi, stones });")
            .expect("㉘ offline_card=Some 行在位（resume_life 特有锚）");
        let set_at = game_p.find(set_needle).expect("㉘ 置值行在位");
        assert!(
            set_at > card_at,
            "㉘ 位点序：置值须在 offline_card=Some 之后（卡片赋值同帧旁置防挪 else 臂）"
        );
        let save_p = prod_of("save.rs");
        assert_eq!(
            save_p.matches("offline_anim").count(),
            0,
            "㉘ 会话态不进存档（save.rs 零引用）"
        );
        let main_p = prod_of("main.rs");
        assert_eq!(
            main_p.matches("if g.offline_anim > 0.0 {").count(),
            1,
            "㉘ 递减早退门恰 1"
        );
        assert_eq!(
            main_p.matches("g.offline_anim = (g.offline_anim - dt).max(0.0);").count(),
            1,
            "㉘ 递减行恰 1"
        );
        let draw_p = prod_of("draw.rs");
        assert_eq!(
            draw_p
                .matches("g.offline_anim / crate::game::OFFLINE_ANIM_T")
                .count(),
            1,
            "㉘ 消费口恰 1（draw_offline_card 暗幕相位反推单口）"
        );
        assert_eq!(
            draw_p.matches("let dim = 0.78 * (0.3 + 0.7 * p);").count(),
            1,
            "㉘ 暗幕先行式恰 1（照③ modal_anim 款——稳态 p=1.0 恒等 0.78）"
        );
    }

    // ── ㉖ 覆盖层增强 B 案 + ㉕ 涟漪末行伪影修复（勘察六 finding①② 根因闭环） ──
    #[test]
    fn overlay_beam_and_ripple_endline_pins() {
        let world_p = prod_of("world.rs");
        assert_eq!(
            world_p
                .matches("crate::draw::draw_depart_beam(g, 240.0, 140.0, 800.0, 420.0);")
                .count(),
            1,
            "㉖ B 案覆盖层面板域二次调用恰 1（霞光扫过远征卷轴——v11656 世界图全参 needle 不受影响照旧）"
        );
        let draw_p = prod_of("draw.rs");
        assert_eq!(
            draw_p.matches("if i < 5 {").count(),
            1,
            "㉕ 涟漪末行门恰 1（i=5 线落底部提示行字形带内不画线——勘察六 finding①）"
        );
        assert_eq!(
            draw_p.matches("if mp_a > 0.0 {").count(),
            1,
            "㉕ 早退门字面照旧（嵌套 if 保 v11656 钉面 needle 零漂移）"
        );
    }

    // ── UX 四件静态钉（步进点脉动/hot 外圈呼吸/页签三组加亮段/拾取卡标题插值） ──
    #[test]
    fn ux_four_static_pins() {
        let world_p = prod_of("world.rs");
        assert_eq!(
            world_p.matches("5.0 + 0.8 * (g.time * 2.6).sin()").count(),
            1,
            "UX① 远征步进点 cur 微脉动恰 1（照世界图节点圈常驻脉动先例）"
        );
        assert_eq!(
            world_p.matches("0.35 + 0.15 * (g.time * 2.4).sin()").count(),
            1,
            "UX② 世界图 hot 外圈 α 呼吸恰 1（内圈静态满金作选中锚）"
        );
        let draw_p = prod_of("draw.rs");
        assert_eq!(
            draw_p.matches("Color::from_rgba(255, 244, 200, 255)").count(),
            2,
            "UX③ 页签加亮段 draw.rs 恰 2（坊市/行囊——三组同批同语防半改）"
        );
        let char_p = prod_of("charpanel.rs");
        assert_eq!(
            char_p.matches("Color::from_rgba(255, 244, 200, 255)").count(),
            1,
            "UX③ 页签加亮段 charpanel 恰 1（角色页）"
        );
        assert_eq!(
            draw_p.matches("let life_t = (is_a / 0.12).clamp(0.0, 1.0);").count(),
            1,
            "UX④ 拾取卡标题插值恰 1（复用㉔ is_a 零新状态，稳态恒等 C_GOLD）"
        );
    }

    // ── 修为圆满两口文案钉（勘察六③真出戏口——143% 直出收敛为圆满语汇） ──
    #[test]
    fn qi_full_wording_pins() {
        let draw_p = prod_of("draw.rs");
        assert_eq!(
            draw_p.matches("\"修为圆满\".to_string()").count(),
            1,
            "UX⑤ draw 左栏口整句恰 1（frac>=1.0 分支替换 evf 2615）"
        );
        assert_eq!(
            draw_p.matches("if frac >= 1.0 {").count(),
            1,
            "UX⑤ draw 满判定门恰 1（frac<1.0 走原查表逐像素不变）"
        );
        let char_p = prod_of("charpanel.rs");
        assert_eq!(
            char_p.matches("txt(\"圆满\", 412.0, 392.0, 14, C_TEXT);").count(),
            1,
            "UX⑤ charpanel 破境条口两字恰 1（左标签「修为」已有故两字足义同位等宽）"
        );
        assert_eq!(
            char_p.matches("if frac >= 1.0 {").count(),
            1,
            "UX⑤ charpanel 满判定门恰 1（两口同批同语义防半改）"
        );
    }
}
