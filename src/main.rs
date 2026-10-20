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

fn window_conf() -> Conf {
    Conf {
        window_title: "凡人修仙·轮回模拟器".to_owned(),
        window_width: 1280,
        window_height: 720,
        // v1.6.0 B 清晰度：high_dpi 恒开。桌面端 miniquad 调 SetProcessDPIAware()，
        // 窗口按物理像素开窗 + 字形按 dpr 原生栅格；wasm 端 backing store = CSS×dpr
        // 1:1 物理像素，dpr>1 设备上文字按原生分辨率栅格（此前 1× 位图被非整数倍
        // 放大发虚）。图集爆量风险已由 vendor/macroquad 封顶补丁护持（Cargo.toml
        // [patch.crates-io]，atlas 按 JS 探测的 MAX_TEXTURE_SIZE 动态封顶）。
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
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
    loop {
        let dt = get_frame_time().min(1.0 / 20.0);
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
            if g.submenu.is_none() && g.rmodal.is_none() {
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
            if is_mouse_button_pressed(MouseButton::Left) {
                xx_clicks += 1;
                xx_last = Some(crate::cam::mouse());
            }
            let (mx, my) = crate::cam::mouse();
            draw_line(mx - 40.0, my, mx + 40.0, my, 3.0, Color::new(1.0, 0.1, 0.1, 0.9));
            draw_line(mx, my - 40.0, mx, my + 40.0, 3.0, Color::new(1.0, 0.1, 0.1, 0.9));
            draw_circle(mx, my, 4.0, Color::new(1.0, 0.1, 0.1, 1.0));
            draw_text(
                &format!("{:.0},{:.0}", mx, my),
                mx + 24.0,
                my - 20.0,
                26.0,
                Color::new(1.0, 0.1, 0.1, 1.0),
            );
            let last_str = match xx_last {
                Some((x, y)) => format!("{:.0},{:.0}", x, y),
                None => "-".to_string(),
            };
            // F: = 帧号。真机截图若两秒内 F 不涨 = 帧时钟停摆（冻帧：画面是
            // 最后一帧的残影，输入永远无人轮询）——配合 JS 触摸帧泵定位。
            draw_text(
                &format!("clicks:{} F:{} last:{}", xx_clicks, frame, last_str),
                830.0,
                40.0,
                26.0,
                Color::new(1.0, 0.9, 0.2, 1.0),
            );
            if let Some((x, y)) = xx_last {
                draw_circle(x, y, 9.0, Color::new(1.0, 0.9, 0.2, 0.95));
            }
        }
        set_default_camera();
        // 音效冲刷
        if let Some(w) = g.sfx_ok.take() {
            g.audio.play(w);
        }
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
            g.sync_save();
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
                l.companion = Some("南宫婉".to_string());
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
                    g.act_cultivate();
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
