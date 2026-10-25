// 入口：窗口、主循环、场景分发、GF_DEBUG 自动驾驶冒烟
mod art;
mod cam;
mod charpanel;
mod data;
mod draw;
mod game;
mod menu;
mod npc;
mod pixel;
mod save;
mod sounds;
mod ui;
mod world;

use game::{Game, Scene};
use macroquad::prelude::*;

// v1.9.10 诊断出口（仅 wasm）：panic 消息与场景变化经 index.html 注册的
// xx_dbg 插件回传 JS——wasm 陷阱在真机上此前完全不可见（F 冻结但 errlog 无
// 任何记录，无法区分"时钟停摆"与"Rust 陷阱/互斥锁中毒"），现在一张截图即可
// 读到 panic 原文；xx_state 供 ?tap= 自动回归断言"点按钮→进游戏"。
#[cfg(target_arch = "wasm32")]
mod xx_dbg {
    #[link(wasm_import_module = "env")]
    unsafe extern "C" {
        fn xx_panic(ptr: *const u8, len: i32);
        fn xx_state(code: i32);
        // v1.9.12 菜单自检三通道（与 index.html xx_dbg 插件成对出现）：
        // xx_click：Rust 每消费一次点击上报计数——JS 看门狗据此检测
        //   "触摸到达页面但游戏引擎未消费"（输入桥断开，v1.9.10/1.9.11 真机
        //   F 冻在 ~100/101 的现象层）并弹 DOM 大旗 + 自动重载。
        // xx_js_fr / xx_js_touches：Rust 反向读取 JS 帧时钟心跳年龄与 JS 侧
        //   累计触摸数，画进菜单自检面板——一张截图同时呈现 JS 视角与 Rust
        //   视角，桥断/时钟停摆/循环死亡一眼分辨。
        fn xx_click(n: i32);
        fn xx_js_fr() -> i32;
        fn xx_js_touches() -> i32;
        fn xx_alive(frame: i32);
    }

    /// wasm 陷阱可见化：panic 原文写入 errlog（[PANIC] 前缀）。须最先安装。
    pub fn install() {
        std::panic::set_hook(Box::new(|info| {
            let msg = format!("panic: {info}");
            unsafe { xx_panic(msg.as_ptr(), msg.len() as i32) };
        }));
    }

    pub fn state(code: i32) {
        unsafe { xx_state(code) };
    }

    pub fn click(n: i32) {
        unsafe { xx_click(n) };
    }

    pub fn js_fr() -> i32 {
        unsafe { xx_js_fr() }
    }

    pub fn js_touches() -> i32 {
        unsafe { xx_js_touches() }
    }

    /// 循环心跳：每 30 帧上报一次帧号。真机实证（v1.9.10/11/12 三张截图）：
    /// 首次真触摸会杀死 async 主循环（F 冻在点击当帧），而 frame() 照常返回、
    /// JS 心跳新鲜、无任何报错——JS 侧据此在 2.5 秒内识别死亡并自动重载
    /// （用户实测重载后必好：缓存热、秒开、循环存活）。
    pub fn alive(frame: i32) {
        unsafe { xx_alive(frame) };
    }

    /// v1.9.23 调试信息开关（默认关）：红十字/黄点/诊断行/自检面板等一切
    /// 排障绘制只在 ?dbg=1（JS 调 xx_set_debug）后出现；功能逻辑（点击计数、
    /// 菜单兜底、自动进入倒计时）不受开关影响。心跳/状态/点击上报照常
    /// （JS 侧看门狗依赖它们，与可见性无关）。
    pub static DEBUG_UI: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

    pub fn debug_on() -> bool {
        DEBUG_UI.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// JS → Rust 调试开关入口（?dbg=1 时 JS 在 wasm 就绪后调用）
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn xx_set_debug(on: i32) {
    xx_dbg::DEBUG_UI.store(on != 0, std::sync::atomic::Ordering::Relaxed);
}

fn window_conf() -> Conf {
    Conf {
        window_title: "凡人模拟·轮回模拟器".to_owned(),
        window_width: 1280,
        window_height: 720,
        // v1.6.0 B 清晰度：high_dpi 恒开。桌面端 miniquad 调 SetProcessDPIAware()，
        // 窗口按物理像素开窗 + 字形按 dpr 原生栅格。
        // v1.9.9 wasm 回退 dpr=1 制（像素风坐标制）：wasm 开 high_dpi 后，X5 真机
        // 上坐标在 wasm 内部被乘 dpr(3.25)（注入 1036→记录 3367，v1.9.8 真机数据），
        // 且 canvas/backing/JS 三处 dpr 捕获时点不一致（启动瞬间 vs 实时）无法对账；
        // 像素风版本（v1.5.x，wasm 未开 high_dpi）在该真机上验证可玩。本版 wasm
        // 关闭 high_dpi：JS dpi_scale() 恒 1，一切 dpi 乘除退化为空操作——本地 dpr=1
        // 环境与手机从构造上同构（本地过=手机过）。
        // v1.9.10 清晰度回补：dpr=1 制下手机端按 CSS 分辨率栅格被 GPU 放大 3.25×，
        // 真机反馈"画质变模糊"。由 index.html 在 bundle 加载前把 canvas 的
        // clientWidth/clientHeight getter 覆盖为真实 CSS 尺寸 ×SS（固定 3，确定性
        // 常数而非 devicePixelRatio，不破坏同构性）——miniquad 自身的启动路径据此
        // 把 backing 开到 CSS×3（与 v1.9.8 真机跑通过的 2772x1280 同一代码路径），
        // Rust 侧零 dpi 除法、cam.rs 零改动，坐标仍在唯一坐标系（backing）里。
        high_dpi: !cfg!(target_arch = "wasm32"),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // v1.9.10 wasm 陷阱可见化：越早装越好（后续任何 panic 都会回传 errlog）
    #[cfg(target_arch = "wasm32")]
    xx_dbg::install();
    // v1.5.0 GF_CLEAN：净版截图模式——删档保证从头开始（必须在读档之前）；
    // 无版本水印见 draw.rs version_mark
    // v1.8.0：GF_DEBUG 冒烟同样删档——冒烟会直置 npc_fav 等元数据并存档，
    // 若载入上次冒烟的存档，好感 ≥70 会在跑图途中误触好感事件模态，破坏确定性
    if std::env::var("GF_CLEAN").is_ok() || std::env::var("GF_DEBUG").is_ok() {
        let _ = std::fs::remove_file(save::PATH);
    }
    ui::init_font();
    art::init_art();
    let audio = sounds::init().await;
    let mut g = Game::new(audio);
    // v1.6.0 A 清晰度：high_dpi 后窗口按物理像素开窗，125%/150% 缩放下 screen_width()
    // 首帧只有 853/1024（物理/倍率）。这里请求 1280×720 逻辑尺寸（macroquad 内部 ×dpi →
    // 客户区 1920×1080 物理像素），第 2 帧起 screen_width()==1280、cam.rs fit==1 零改动、
    // 渲染分辨率=物理像素 → 文字锐利且窗口视觉尺寸与 v1.5.0 一致（100% 缩放时为 no-op）。
    // 回退方案 (a)：删掉本块即接受 853×480 逻辑视口（16:9 精确整除，fit=0.667 无断裂）。
    // 注意：request 至下一帧 next_frame().await 后才生效（macroquad window.rs 注释原文），
    // 150% 下首帧 ≈853 属预期，不算异常。
    #[cfg(not(target_arch = "wasm32"))]
    macroquad::window::request_new_screen_size(1280.0, 720.0);
    if g.meta.music {
        g.audio.start_bgm();
    }
    g.audio.sfx_on = g.meta.sfx;
    let debug = std::env::var("GF_DEBUG").is_ok();
    if debug {
        let _ = std::fs::create_dir_all("shots");
    }
    let mut frame = 0u32;
    let mut shots = 0u32;
    // v1.9.5 真机输入诊断（仅 wasm）：点击计数 + 最近点击的虚拟坐标常驻右上角。
    // 配合 JS 侧大号十字光标：一张真机截图即可区分"事件没到 Rust"（clicks 恒 0）
    // 与"到了但按钮判定不吃"（clicks 增长且十字在按钮上）。
    #[cfg(target_arch = "wasm32")]
    let (mut xx_clicks, mut xx_last): (u32, Option<(f32, f32)>) = (0, None);
    #[cfg(target_arch = "wasm32")]
    let mut xx_clicks_handled: u32 = 0;
    // v1.9.25 卡顿修复验收仪表（仅 wasm）：字体缓存 churn 计数上一帧采样值——
    // HUD 显示「总量/本帧增量」，banner 动画期应见 G:+0 U:+0（预热+字号离散化生效）
    #[cfg(target_arch = "wasm32")]
    let (mut xx_g_prev, mut xx_u_prev): (usize, usize) = (0, 0);
    // v1.9.12 菜单自动进入倒计时（仅 wasm）：真机"点按钮无效"的终极保险——
    // 倒计时归零直接 begin_mortal，零输入依赖；循环活着就必然进入，循环死了
    // 冻结的倒计时数字本身就是诊断（配合 JS 看门狗大旗分层定位）。
    #[cfg(target_arch = "wasm32")]
    let mut xx_auto: f32 = 8.0;
    // v1.9.10 场景变化回传（wasm）：JS 侧 __xxState 断言"点按钮→进游戏"；
    // 0=Menu 1=Reality 2=Draft 3=Sim 4=Inherit 5=Settle 6=World
    #[cfg(target_arch = "wasm32")]
    let mut xx_scene_prev = Scene::World; // 与初始 Menu 不同，保证首帧上报 0
    loop {
        let dt = get_frame_time().min(1.0 / 20.0);
        #[cfg(target_arch = "wasm32")]
        if g.scene != xx_scene_prev {
            xx_scene_prev = g.scene;
            xx_dbg::state(match g.scene {
                Scene::Menu => 0,
                Scene::Reality => 1,
                Scene::Draft => 2,
                Scene::Sim => 3,
                Scene::Inherit => 4,
                Scene::Settle => 5,
                Scene::World => 6,
            });
        }
        // v1.6.0 B：开帧门闩清零（置位发生于上一帧绘制阶段、本帧输入/绘制前清零，时序闭合）
        g.sub_fresh = false;
        // v1.7.0：幻境广告倒计时按真实秒推进（与场景无关）
        g.tick_ad(dt);
        // 快捷键
        if g.scene == Scene::Sim {
            if is_key_pressed(KeyCode::Space) {
                g.speed = if g.speed == 0 { 2 } else { 0 };
            }
            if is_key_pressed(KeyCode::Key1) {
                g.speed = 1;
            }
            if is_key_pressed(KeyCode::Key2) {
                g.speed = 2;
            }
            if is_key_pressed(KeyCode::Key3) || is_key_pressed(KeyCode::Key4) {
                g.speed = 3;
            }
        }
        // v1.6.0 B：数字键场景无关转发（弹层热键；Sim 场景 submenu 恒 None → no-op）
        for (n, kc) in [(1usize, KeyCode::Key1), (2, KeyCode::Key2), (3, KeyCode::Key3)] {
            if is_key_pressed(kc) {
                crate::menu::sub_hotkey(&mut g, n);
            }
        }
        if is_key_pressed(KeyCode::Escape) {
            // v1.6.0 B：弹层优先关闭
            // v1.8.0 B4：好感事件卡独占期 Esc 被吞——事件必须抉择，不得绕开
            if g.rmodal.is_some() {
                // 吞掉：事件必须抉择
            } else if g.submenu.take().is_some() {
                g.play(crate::sounds::Which::Page);
            } else if g.panel != 0 {
                g.panel = 0;
                g.char_tab = 0; // 与角色面板 × 行为一致：关面板同时归零页签
                g.npc_scroll_reset(); // v1.10.2：人物页滚动会话态同批归零（先例 char_page 口径）
                // v1.8.0 B3/E3：关面板统一清各面板的挂起确认态（顺手修复既有缺陷：
                // Esc 关行囊后 sac_pending 残留，重开仍处确认态）
                g.sac_pending = None;
                g.buy_pending = None;
                g.refine_pending = None;
            } else if g.scene == Scene::World {
                crate::world::close_world(&mut g); // 远征中有卫语句拒绝离场
            }
        }
        if g.scene == Scene::Reality {
            // v1.6.0 B：弹层打开时 C/M 暂避让，防误触穿透
            // v1.8.0 B4：好感事件卡独占期一并避让
            // v1.10.2 复审②：离闭结算卡独占期同样避让（防卡上开面板/山河图穿透）
            // v1.12.1 复审③a：终卷演出页覆层独占期同样避让
            if g.submenu.is_none()
                && g.rmodal.is_none()
                && g.offline_card.is_none()
                && g.ending_show.is_none()
            {
                if is_key_pressed(KeyCode::C) {
                    g.panel = if g.panel == 5 { 0 } else { 5 };
                }
                if g.panel == 0 && is_key_pressed(KeyCode::M) {
                    crate::world::open_world(&mut g);
                }
            }
        }
        // 更新
        if g.scene == Scene::Sim {
            g.update_sim(dt);
        } else {
            g.time += dt;
            g.update_ambient(dt);
        }
        // v1.10.2 面板语料分帧预热（卡顿修复：面板首开 ≈ 温开）：菜单期每帧灌 ~40 字形
        if g.scene == Scene::Menu {
            ui::panel_warm_step(40);
        }
        // v1.9.25 周天吐纳：每帧 tick（内部自门控：仅 Reality 且弹层/事件/结算全关时累计）
        g.tick_auto(dt);
        // 绘制（v1.5.0：信箱相机，虚拟 1280×720；桌面恒等映射，H5 任意屏比例等比适配）
        set_camera(&crate::cam::view_cam());
        match g.scene {
            Scene::Menu => draw::draw_menu(&mut g, art::art()),
            Scene::Reality => draw::draw_reality(&mut g, art::art()),
            Scene::Draft => draw::draw_draft(&mut g, art::art()),
            Scene::Sim => draw::draw_sim(&mut g, art::art()),
            Scene::Inherit => draw::draw_inherit(&mut g, art::art()),
            Scene::Settle => draw::draw_settle(&mut g, art::art()),
            Scene::World => crate::world::draw_world(&mut g, art::art()),
        }
        // v1.9.3 触摸诊断（仅 wasm）：大号红色十字 = 游戏认为的指针位置（初始
        // 在左上角 0,0——若十字一直钉在角上，说明 Rust 从未收到指针移动）。
        // 右上角 clicks 计数 = Rust 实际收到的鼠标按下次数。
        #[cfg(target_arch = "wasm32")]
        {
            // v1.9.13 循环心跳：定期上报（JS 侧快速唤醒/看门狗/大旗的采样基础；
            // v1.9.13 真机实证：死亡后连续 frame() 叩击可唤醒循环（第 4 轮 +30 帧），
            // 上报越密，唤醒检测越快）。v1.9.16 粒度 15→8 帧：车夫 ~24fps 供血下
            // 上报间隔 ~0.33s，低帧率（触摸驱动 ~2fps）时也从 7.5s 缩到 4s——
            // 看门狗/大旗死线不再被上报粒度拖爆误判"心跳停滞"。
            if frame % 8 == 0 {
                xx_dbg::alive(frame as i32);
            }
            if is_mouse_button_pressed(MouseButton::Left) {
                xx_clicks += 1;
                xx_last = Some(crate::cam::mouse());
                xx_dbg::click(xx_clicks as i32);
            }
            // v1.9.23 调试开关：以下排障绘制默认关（?dbg=1 打开）
            if xx_dbg::debug_on() {
                let (mx, my) = crate::cam::mouse();
                draw_line(mx - 40.0, my, mx + 40.0, my, 3.0, Color::new(1.0, 0.1, 0.1, 0.9));
                draw_line(mx, my - 40.0, mx, my + 40.0, 3.0, Color::new(1.0, 0.1, 0.1, 0.9));
                draw_circle(mx, my, 4.0, Color::new(1.0, 0.1, 0.1, 1.0));
                draw_text(
                    &format!("{:.0},{:.0}", mx, my),
                    mx + 24.0,
                    my - 20.0,
                    24.0,
                    Color::new(1.0, 0.1, 0.1, 1.0),
                );
                let last_str = match xx_last {
                    Some((x, y)) => format!("{:.0},{:.0}", x, y),
                    None => "-".to_string(),
                };
                // 全链路诊断：D=wasm 缓存 dpi，S=wasm 屏幕尺寸，T=触摸数+原始位置，
                // M=mouse_position 原始值。dpr=1 制下应恒为 D:1.00 T/M=注入原值。
                let (mpx, mpy) = mouse_position();
                let ts = touches();
                let t_str = match ts.last() {
                    Some(t) => format!("{}@{:.0},{:.0}", ts.len(), t.position.x, t.position.y),
                    None => "0".to_string(),
                };
                draw_text(
                    &format!(
                        "c:{} F:{} l:{} D:{:.2} S:{:.0}x{:.0} T:{} M:{:.0},{:.0}",
                        xx_clicks,
                        frame,
                        last_str,
                        macroquad::miniquad::window::dpi_scale(),
                        screen_width(),
                        screen_height(),
                        t_str,
                        mpx,
                        mpy
                    ),
                    700.0,
                    40.0,
                    20.0,
                    Color::new(1.0, 0.9, 0.2, 1.0),
                );
                // v1.9.25 字体缓存 churn 仪表（第二行防溢出）：G=新字形栅格化 U=图集整图
                // texture_update 次数（总量/本帧增量）。验收口径：banner 弹出 0.6s 动画期
                // 逐帧增量 +0/+0（弹出前预热全档位 + pop 字号离散化后动画帧零缓存 miss）。
                // 临时计数逻辑（vendor text.rs/atlas.rs），真机验收通过后可整体移除。
                let gl = macroquad::text::GLYPH_RASTERS.load(std::sync::atomic::Ordering::Relaxed);
                let up = macroquad::text::ATLAS_UPLOADS.load(std::sync::atomic::Ordering::Relaxed);
                let (dg, du) = (gl - xx_g_prev, up - xx_u_prev);
                xx_g_prev = gl;
                xx_u_prev = up;
                draw_text(
                    &format!("G:{}/+{} U:{}/+{}", gl, dg, up, du),
                    700.0,
                    70.0,
                    20.0,
                    Color::new(0.7, 0.95, 1.0, 1.0),
                );
                if let Some((x, y)) = xx_last {
                    draw_circle(x, y, 9.0, Color::new(1.0, 0.9, 0.2, 0.95));
                }
            }
            // v1.9.11 菜单兜底（真机"进去过一次、重进后点开始无效"的保险丝）：
            // 每个被消费的点击只评估一次——若落在「开始新的人生」矩形外扩 30 内
            // 而场景仍是 Menu（button() 因任何原因没吃到），强制 begin_mortal。
            // button() 正常触发时场景已离开 Menu，天然不会二次进入。
            if xx_clicks > xx_clicks_handled {
                xx_clicks_handled = xx_clicks;
                if g.scene == Scene::Menu {
                    if let Some((x, y)) = xx_last {
                        let (bx, by, bw, bh) = (320.0f32, 522.0f32, 300.0f32, 54.0f32);
                        let m = 30.0;
                        if x >= bx - m && x <= bx + bw + m && y >= by - m && y <= by + bh + m {
                            g.play(crate::sounds::Which::Click);
                            g.begin_mortal();
                        }
                    }
                }
            }
            // v1.9.12 菜单自检面板（v1.9.23 受调试开关控制，默认关）。
            // 同屏呈现 JS 视角（时钟心跳/触摸数）与 Rust 视角（已消费点击/帧数/
            // 场景）——真机 F 冻结 + fr 新鲜的"悖论"从此肉眼可判：
            //   触达涨/已收不涨 = 输入桥断（JS 看门狗同时会弹大旗+自动重载）；
            //   帧号/倒计时冻结  = 游戏循环死（时钟行仍跳 = JS 活着）；
            //   全部正常仍菜单  = 逻辑层（兜底与倒计时强制进入）。
            if g.scene == Scene::Menu {
                if xx_dbg::debug_on() {
                    let fr = xx_dbg::js_fr();
                    let tch = xx_dbg::js_touches();
                    // 放右上角（黄色诊断行正下方），不遮标题/按钮/底部状态栏
                    draw_rectangle(
                        646.0,
                        84.0,
                        620.0,
                        96.0,
                        Color::new(0.0, 0.0, 0.0, 0.62),
                    );
                    draw_text(
                        &format!(
                            "自检 触达:{} 已收:{} 帧:{} 时钟:{}ms 场景:{}",
                            tch, xx_clicks, frame, fr, g.scene_xx_name(),
                        ),
                        662.0,
                        118.0,
                        20.0,
                        Color::new(0.5, 1.0, 0.7, 1.0),
                    );
                    draw_text(
                        &format!(
                            "{} 秒后自动进入 · 点「开始新的人生」立即开始",
                            xx_auto.max(0.0) as i32
                        ),
                        662.0,
                        152.0,
                        20.0,
                        Color::new(1.0, 0.9, 0.4, 1.0),
                    );
                }
                // 倒计时：菜单停留 8 秒直接进入（零输入依赖的终极保险）。
                // dt 累减；任何途径离开菜单后本块不再执行，天然停摆。
                // （功能逻辑，不受调试开关控制）
                xx_auto -= dt;
                if xx_auto <= 0.0 {
                    g.play(crate::sounds::Which::Click);
                    // v1.10.2 前置修复（数据丢失）：有档恢复（含离线结算）而非无条件
                    // 开新人生——旧实现挂菜单 8 秒即被 begin_mortal 覆盖旧档人生；
                    // ?tap= 自动回归与 mumu 冒烟预期同步：有档时进的是 Reality
                    g.resume_or_begin();
                }
            }
        }
        set_default_camera();
        // 音效冲刷
        if let Some(w) = g.sfx_ok.take() {
            g.audio.play(w);
        }
        // v1.10.2 存档落盘去抖（卡顿修复·次要项②）：场景/弹窗/面板开合边界立即落，
        // 否则每 ≥3s 一落（周天挂机 localStorage 写 0.6→0.2 次/s；wasm 侧另有
        // index.html pagehide/visibilitychange 兜底，native 关窗丢 ≤3s 属既定折中）
        g.save_timer += dt;
        let sig = (
            g.scene as u8,
            g.panel,
            g.rmodal.is_some(),
            g.modal.is_some(),
            g.submenu.is_some(),
            g.world.shop,
            g.world.dev,
            g.world.exp.is_some(),
        );
        if sig != g.prev_sig {
            g.save_flush();
        } else if g.save_dirty && g.save_timer >= 3.0 {
            g.save_flush();
        }
        g.prev_sig = sig;
        if debug {
            autopilot(&mut g, frame, &mut shots);
        }
        frame += 1;
        next_frame().await
    }
}

// 自动驾驶：自动玩几个轮回并截图，用于无人冒烟验收
fn autopilot(g: &mut Game, frame: u32, shots: &mut u32) {
    if frame % 120 == 30 && *shots < 63 {
        let img = get_screen_data();
        let path = format!("shots/shot_{:03}.png", *shots);
        img.export_png(&path);
        *shots += 1;
        if *shots >= 63 {
            // v1.10.2：落盘去抖后 sync_save 只置脏，退出前必须显式 flush
            g.save_flush();
            std::process::exit(0);
        }
    }
    // 面板巡检：shots 24..32 依次 坊市(丹药阁)/坊市(点数兑换)/坊市(藏经阁)/行囊(丹药)/
    // 行囊(法宝·补贴装备)/主界面+修炼弹层/人物页(好感度)/主界面
    // （v1.6.0：B 三页签坊市与行囊、Cult 弹层、D 人物页入镜；
    //   冒烟直置 submenu 不经 open_sub 门闩——冒烟无鼠标同帧点击，
    //   且门闩由主循环开头清零，不影响 shot_029 之后的帧）
    if *shots >= 24 && *shots < 32 && g.scene == Scene::Reality {
        let seq = [1usize, 1, 1, 2, 2, 0, 5, 0];
        g.panel = seq[(*shots - 24) as usize] as u8;
        g.shop_tab = match *shots {
            24 => 0,
            25 => 2,
            26 => 1,
            _ => g.shop_tab,
        };
        g.bag_tab = match *shots {
            27 => 0,
            28 => 1,
            _ => g.bag_tab,
        };
        g.char_tab = match *shots {
            30 => 3, // 角色·人物页（好感度三卡）
            _ => 0,
        };
        // 冒烟专用：人物页入镜前给现实主角注入道侣（正式玩法不由此入账；
        // 覆盖 v1.7.1 道侣 AI 立绘 comp_ovr 渲染路径——有 comp_N.png 即见其像，无则丹药图标）
        if *shots == 30 && frame % 120 == 0 {
            if let Some(l) = g.life.as_mut() {
                l.companion = Some("闻人未晚".to_string());
                l.comp_fav = 60;
            }
        }
        if *shots != 29 {
            g.submenu = None;
        }
        // 冒烟专用：发放两件法宝并演示装备（正式玩法不由此入账；equip_relic 自守卫）
        if *shots == 28 && frame % 120 == 0 {
            if let Some(l) = g.life.as_mut() {
                if !l.relics.contains(&0) {
                    l.relics.push(0);
                }
                if !l.relics.contains(&1) {
                    l.relics.push(1);
                }
            }
            g.equip_relic(0);
        }
        // 修炼弹层入镜（shot_029：闭关苦修/冲击破境/返回）
        if *shots == 29 {
            g.submenu = Some(crate::menu::Sub::Cult);
        }
        return;
    }
    if frame < 30 {
        return;
    }
    // v1.7.0 死档→广告→献祭链路（51..54 置场，对应截图 052..055；frame%120 错开截图帧）
    // v1.8.0：链路后追加扩镜 56..59（056 行囊炼化 / 057 人物页四卡 / 058 区域图鉴 / 059 帮助页签）
    if *shots >= 51 && *shots <= 54 {
        if frame % 120 == 0 {
            match *shots {
                // 强制死档场：回现实、清点数/灵石/法宝 → shot_052 底栏「山穷水尽」
                51 => {
                    g.scene = Scene::Reality;
                    g.sim = None;
                    g.draft.clear();
                    g.picked.clear();
                    g.panel = 0;
                    g.submenu = None;
                    g.meta.sim_points = 0;
                    if let Some(l) = g.life.as_mut() {
                        l.stones = 0;
                        l.relics.clear();
                        l.equipped.clear();
                    }
                }
                // shot_052 已拍 → 打开广告浮层 → shot_053
                52 => g.start_ad(),
                // shot_053 已拍 → 倒计时快进到可领取 → shot_054
                53 => g.ad = Some(0.0),
                // shot_054 已拍 → 领取，再发一件法宝献祭 → shot_055
                54 => {
                    g.collect_ad();
                    if let Some(l) = g.life.as_mut() {
                        if !l.relics.contains(&5) {
                            l.relics.push(5);
                        }
                    }
                    g.sacrifice_relic(5);
                }
                _ => {}
            }
        }
        return;
    }
    // v1.8.0 百炼乾坤扩镜（56..62 置场，对应截图 056..062；frame%120 单发，同死档链范式）
    // （R1 验收补镜：060 轮回烙印页签 / 061 藏经阁求购确认态 / 062 好感事件卡 rmodal；
    //   B4 键盘守卫链截图法不可验，保持静态代码审查口径）
    if *shots >= 56 && *shots <= 62 {
        if frame % 120 == 0 {
            match *shots {
                // 56：行囊法宝页+炼化选材态——relic_lv[0]=2 徽标入镜、row0 取消炼化 / row1 选为材料
                56 => {
                    g.scene = Scene::Reality;
                    g.sim = None;
                    g.modal = None; // 清模拟尾段可能残留的事件卡（draw_modal_card 双通道保险）
                    g.rmodal = None;
                    g.rmodal_npc = None;
                    g.panel = 2;
                    g.bag_tab = 1;
                    g.submenu = None;
                    if let Some(l) = g.life.as_mut() {
                        if !l.relics.contains(&0) {
                            l.relics.push(0);
                        }
                        if !l.relics.contains(&1) {
                            l.relics.push(1);
                        }
                        l.relic_lv[0] = 2; // 冒烟专用：灵纹等级徽标（正式玩法不由此入账）
                    }
                    g.refine_pending = Some(0);
                }
                // 57：人物页四卡（npc_fav 直置不经 add_fav——不触发 rmodal 好感事件）
                57 => {
                    g.panel = 5;
                    g.char_tab = 3;
                    g.submenu = None;
                    g.refine_pending = None;
                    g.meta.npc_fav = [72, 45, 80, 20];
                }
                // 58：山河图志+区域图鉴弹层（顶栏机制短标一并入镜）
                58 => {
                    g.panel = 0;
                    g.char_tab = 0;
                    crate::world::open_world(g);
                    crate::menu::open_sub(g, crate::menu::Sub::Atlas);
                }
                // 59：帮助面板·操作·技巧页签（清掉 58 遗留的图鉴弹层——冒烟直置不经 Esc 链）
                59 => {
                    g.submenu = None;
                    crate::world::close_world(g);
                    g.panel = 4;
                    g.help_tab = 1;
                }
                // 60（R1 补镜）：轮回烙印第 2 页签「机缘·造化」（B1 行带 5..9 入镜）
                60 => {
                    g.panel = 3;
                    g.tab = 1;
                    g.play(crate::sounds::Which::Page);
                }
                // 61（R1 补镜）：坊市藏经阁 + 求购二次确认态（B3「确认求购?」150px 钮入镜）
                61 => {
                    g.panel = 1;
                    g.shop_tab = 1;
                    g.buy_pending = Some(1); // 冒烟直置确认态（正式玩法由求购钮置位）
                }
                // 62（R1 补镜）：现实侧好感事件卡（D2 rmodal 模态通道入镜；直置不经 add_fav）
                62 => {
                    g.panel = 0;
                    g.buy_pending = None;
                    g.meta.npc_fav[crate::npc::NPC_SENIOR] = 80; // 与卡面归属一致（不触发 add_fav 链）
                    if let Some(card) = crate::npc::npc_event_card(crate::npc::NPC_SENIOR) {
                        g.rmodal = Some(card);
                        g.rmodal_npc = Some(crate::npc::NPC_SENIOR);
                    }
                }
                _ => {}
            }
        }
        return;
    }
    match g.scene {
        Scene::Menu => {
            if frame % 30 == 0 {
                g.begin_mortal();
            }
        }
        Scene::Reality => {
            // v1.6.0 镜头：32 开地图（World 分支 33 起覆盖山河弹层/洞府/远征）
            if *shots == 32 && frame % 30 == 0 {
                crate::world::open_world(g);
                return;
            }
            // v1.6.0 C：强制寻仇胜利 → spawn_next_neme→advance_map 换幕链（横幅入镜）
            if *shots == 46 && frame % 120 == 0 {
                g.debug_win_revenge();
                return;
            }
            // v1.6.0 C：换幕后重开地图（西漠），选点天雷峰终帧
            if *shots == 47 && frame % 30 == 0 {
                crate::world::open_world(g);
                g.world.sel = crate::data::NODE_PEAK;
                return;
            }
            // v1.6.0 补镜头（验收第 1 轮发现③）：48 归返现实后强开一次模拟，
            // 由 Draft/Sim 分支接力为事件弹窗与继承画面补确定性入镜（48..51）
            if *shots == 48 && frame % 30 == 0 {
                if g.meta.sim_points >= g.sim_cost() {
                    g.start_draft();
                }
                return;
            }
            if g.panel != 0 {
                if frame % 60 == 0 {
                    g.panel = 0;
                }
                return;
            }
            // 46 起留给强制斩敌换幕镜头，不再消耗现实岁月
            if frame % 45 == 0 && *shots < 44 {
                if g.meta.sim_points >= g.sim_cost() {
                    g.start_draft();
                } else if g.can_break() {
                    g.act_break();
                } else {
                    g.act_cultivate(false); // 冒烟手点口径：全量日志+音效
                }
            }
        }
        Scene::Draft => {
            let wait = if *shots < 3 { 130 } else { 30 };
            if frame % wait == 0 {
                if g.picked.is_empty() {
                    g.toggle_pick(0);
                } else {
                    g.launch_sim();
                }
            }
        }
        Scene::Sim => {
            // 前几张截图用慢速看播放效果，之后跳过；48..51 尾段强制慢速供弹层入镜
            g.speed = if *shots < 3 || (*shots >= 48 && *shots < 51) {
                1
            } else {
                3
            };
            // 尾段确定性弹层（验收第 1 轮发现③）：速 3 下弹窗同帧自选不渲染、
            // 自然事件又依赖 RNG，48 镜从未捕获新弹窗布局。这里注入真实 EventCard
            // 走真实 draw_sim 渲染路径（纵排大按钮 + eff_desc 副标）；
            // 若自然事件已先弹出（is_none 守卫）则顺带入镜。慢速下弹窗冻结画面。
            if *shots >= 49 && *shots < 51 && g.modal.is_none() && g.sim.is_some() {
                let mut card = game::EventCard {
                    kind: game::CardKind::Npc,
                    title: "仙人问道".into(),
                    text: "云端降下一位白须仙人，含笑看你：「小子，老夫可指你一条路，要哪般？」"
                        .into(),
                    opts: Vec::new(),
                };
                let mut e1 = game::Eff::new();
                e1.dao = 8.0;
                e1.qi_pct = 0.15;
                e1.msg = vec!["道心+8，修为+15%".into()];
                card.opts.push(("求道！".into(), e1));
                let mut e2 = game::Eff::new();
                e2.special = 8;
                e2.good = false;
                e2.msg = vec!["你求问武道。战力永久+12%！".into()];
                card.opts.push(("求武！".into(), e2));
                let mut e3 = game::Eff::new();
                e3.life_y = 50.0;
                e3.msg = vec!["你求问长生。寿元+50 载".into()];
                card.opts.push(("求寿！".into(), e3));
                g.modal = Some(card);
            }
            // 末窗强制落幕：保证继承画面被 shot_051 捕获（弹层开着也会被一并清掉；
            // 继承画面此后被 Inherit 分支冻结，稳定撑到截图帧）
            if *shots == 51 && frame % 120 == 0 && g.sim.is_some() {
                g.sim_die("冒烟验证：一世至此落幕，镜中造化待你选取。");
            }
        }
        Scene::Inherit => {
            // 48 起尾段冻结：sim_die 后继承画面保持静止，稳定撑到 shot_051；
            // 继承选项交互已在 0..47 流程每世覆盖，此处不再动作
            if *shots >= 48 {
                return;
            }
            if frame % 40 == 0 {
                if g.inherit_picked < g.inherit_max() && !g.inherit_opts.is_empty() {
                    g.take_inherit(0);
                } else {
                    g.finish_inherit();
                }
            }
        }
        Scene::Settle => {
            if frame % 120 == 0 {
                g.settle = None;
                g.begin_mortal();
            }
        }
        Scene::World => {
            // v1.6.0 镜头（33 起步）：33 地图全景｜34 选中荒骨战场+山河弹层（前往确认，%120 单发）
            // 35 弹层取消→travel 前往青云宗（sel 同步归位防高亮残留）｜36 拜入+宗门任务（长老好感）
            // 37 赶路清源洞府｜38 冒烟补贴+开洞府经营｜39 升聚灵阵｜40 升灵田
            // 41 收经营→赶路落霞秘境｜42 开启远征｜43 远征界面入镜｜44 谨慎推进一步
            // 45 收手归返现实｜48 归返现实（尾段补镜头）｜（46 强制斩敌换幕 / 47 西漠终帧
            // 在 Reality 分支；48..51 尾段在 Sim 分支注入弹层/强制落幕补拍弹窗与继承画面）
            // （单发动作以 frame%120==0 门控：每格截图恰好触发一次并被同格截图捕获）
            if frame % 30 != 0 {
                return;
            }
            if *shots == 34 && frame % 120 == 0 {
                g.world.sel = crate::data::NODE_BONES;
                g.submenu = Some(crate::menu::Sub::MapGo(crate::data::NODE_BONES));
            } else if *shots == 35 && frame % 120 == 0 {
                g.submenu = None;
                // travel 只改 loc 不改 sel：显式归位防高亮残留（荒骨战场→青云宗）
                g.world.sel = crate::data::NODE_SECT;
                crate::world::travel(g, crate::data::NODE_SECT);
            } else if *shots == 36 && frame % 120 == 0 {
                crate::world::sect_join(g);
                crate::world::sect_quest(g);
            } else if *shots == 37 && frame % 120 == 0 {
                g.world.sel = crate::data::NODE_DONGFU; // 同型预防：赶路后 sel 同步
                crate::world::travel(g, crate::data::NODE_DONGFU);
            } else if *shots == 38 && frame % 120 == 0 {
                // 冒烟专用：补贴洞府扩建资源（正式玩法不由此入账）
                if let Some(l) = g.life.as_mut() {
                    l.stones += 200_000;
                }
                g.meta.sect_contrib += 500;
                g.world.dev = true;
            } else if *shots == 39 && frame % 120 == 0 {
                crate::world::dongfu_up(g, 0);
            } else if *shots == 40 && frame % 120 == 0 {
                crate::world::dongfu_up(g, 1);
            } else if *shots == 41 && frame % 120 == 0 {
                g.world.dev = false; // 收起经营覆盖层再赶路（真实节点点击有 !dev 门控，冒烟直调需手动复位）
                g.world.sel = crate::data::NODE_RUINS; // 同型预防：赶路后 sel 同步
                crate::world::travel(g, crate::data::NODE_RUINS);
            } else if *shots == 42 && frame % 120 == 0 {
                crate::world::exp_start(g);
            } else if *shots == 44 && frame % 120 == 0 {
                crate::world::exp_choose(g, 1); // 谨慎档：成功率+0.10，冒烟求稳
            } else if *shots == 45 && frame % 120 == 0 {
                crate::world::exp_quit(g);
                crate::world::close_world(g);
            } else if *shots == 48 && frame % 30 == 0 {
                // 尾段补镜头（发现③）：归返现实，交由 Reality 分支强开模拟
                crate::world::close_world(g);
            }
        }
    }
}
