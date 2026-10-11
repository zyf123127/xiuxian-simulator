// 入口：窗口、主循环、场景分发、GF_DEBUG 自动驾驶冒烟
mod art;
mod cam;
mod charpanel;
mod data;
mod draw;
mod game;
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
        high_dpi: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // v1.5.0 GF_CLEAN：净版截图模式——删档保证从头开始（必须在读档之前）；
    // 无版本水印见 draw.rs version_mark
    if std::env::var("GF_CLEAN").is_ok() {
        let _ = std::fs::remove_file(save::PATH);
    }
    ui::init_font();
    art::init_art();
    let audio = sounds::init().await;
    let mut g = Game::new(audio);
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
    loop {
        let dt = get_frame_time().min(1.0 / 20.0);
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
        if is_key_pressed(KeyCode::Escape) {
            if g.panel != 0 {
                g.panel = 0;
                g.char_tab = 0; // 与角色面板 × 行为一致：关面板同时归零页签
            } else if g.scene == Scene::World {
                crate::world::close_world(&mut g); // 远征中有卫语句拒绝离场
            }
        }
        if g.scene == Scene::Reality {
            if is_key_pressed(KeyCode::C) {
                g.panel = if g.panel == 5 { 0 } else { 5 };
            }
            if g.panel == 0 && is_key_pressed(KeyCode::M) {
                crate::world::open_world(&mut g);
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
    if frame % 120 == 30 && *shots < 42 {
        let img = get_screen_data();
        let path = format!("shots/shot_{:03}.png", *shots);
        img.export_png(&path);
        *shots += 1;
        if *shots >= 42 {
            g.sync_save();
            std::process::exit(0);
        }
    }
    // 面板巡检：shots 24..30 依次打开 坊市(丹药)/坊市(藏经阁)/行囊/烙印/角色(总览)/角色(法宝页)/角色(成就称号页)
    // （v1.5.0：帮助页让位给角色总览页；角色三页签全覆盖）
    if *shots >= 24 && *shots < 31 && g.scene == Scene::Reality {
        let seq = [1usize, 1, 2, 3, 5, 5, 5];
        g.panel = seq[(*shots - 24) as usize] as u8;
        g.shop_tab = if *shots == 24 { 0 } else { 1 };
        g.char_tab = match *shots {
            29 => 1, // 角色·法宝页
            30 => 2, // 角色·成就称号页
            _ => 0,  // 角色·总览页
        };
        // 冒烟专用：发放两件法宝，覆盖装备/行囊标记/摘要显示（正式玩法不由此入账）
        if *shots == 25 && frame % 120 == 0 {
            if let Some(l) = g.life.as_mut() {
                if !l.relics.contains(&0) {
                    l.relics.push(0);
                }
                if !l.relics.contains(&1) {
                    l.relics.push(1);
                }
            }
        }
        // 行囊页顺带演示装备法宝（equip_relic 内部自守卫：未拥有/已装/槽满皆静默或提示）
        if *shots == 26 && frame % 120 == 0 {
            g.equip_relic(0);
        }
    }
    if frame < 30 {
        return;
    }
    match g.scene {
        Scene::Menu => {
            if frame % 30 == 0 {
                g.begin_mortal();
            }
        }
        Scene::Reality => {
            // 方案锚点调整：open_world 不能放在 Scene::World 分支内（场景未进入时不可达），
            // 改由 Reality 分支在 shots==31 时开启地图（open_world 内部会清 panel）
            if *shots == 31 && frame % 30 == 0 {
                crate::world::open_world(g);
                return;
            }
            if g.panel != 0 {
                if frame % 60 == 0 && !(*shots >= 24 && *shots < 31) {
                    g.panel = 0;
                }
                return;
            }
            if frame % 45 == 0 {
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
            // 前几张截图用慢速看播放效果，之后跳过
            g.speed = if *shots < 3 { 1 } else { 3 };
        }
        Scene::Inherit => {
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
            // v1.5.0 镜头（32 起步，覆盖四时洞天）：31 地图全景（Reality 分支开启）
            // 32 选中青云宗（路线高亮+顶栏时段天气）｜33 赶路青云宗（推岁→时段/天气流转）
            // 34 拜入+宗门任务（天气收益）｜35 赶路清源洞府｜36 开洞府经营（冒烟补贴资源）
            // 37 升聚灵阵｜38 升灵田｜39 赶路落霞秘境｜40 开启远征｜41 远征推进一步（谨慎档）
            // （单发动作以 frame%120==0 门控：每格截图恰好触发一次并被同格截图捕获）
            if frame % 30 != 0 {
                return;
            }
            if *shots == 31 {
                crate::world::open_world(g); // 幂等，重复触发无害
            } else if *shots == 32 {
                g.world.sel = crate::data::NODE_SECT;
            } else if *shots == 33 && frame % 120 == 0 {
                crate::world::travel(g, crate::data::NODE_SECT);
            } else if *shots == 34 && frame % 120 == 0 {
                crate::world::sect_join(g);
                crate::world::sect_quest(g);
            } else if *shots == 35 && frame % 120 == 0 {
                crate::world::travel(g, crate::data::NODE_DONGFU);
            } else if *shots == 36 && frame % 120 == 0 {
                // 冒烟专用：补贴洞府扩建资源（正式玩法不由此入账）
                if let Some(l) = g.life.as_mut() {
                    l.stones += 200_000;
                }
                g.meta.sect_contrib += 500;
                g.world.dev = true;
            } else if *shots == 37 && frame % 120 == 0 {
                crate::world::dongfu_up(g, 0);
            } else if *shots == 38 && frame % 120 == 0 {
                crate::world::dongfu_up(g, 1);
            } else if *shots == 39 && frame % 120 == 0 {
                g.world.dev = false; // 收起经营覆盖层再赶路（真实节点点击有 !dev 门控，冒烟直调需手动复位）
                crate::world::travel(g, crate::data::NODE_RUINS);
            } else if *shots == 40 && frame % 120 == 0 {
                crate::world::exp_start(g);
            } else if *shots == 41 && frame % 120 == 0 {
                crate::world::exp_choose(g, 1); // 谨慎档：成功率+0.10，冒烟求稳
            }
        }
    }
}
