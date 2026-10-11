// 入口：窗口、主循环、场景分发、GF_DEBUG 自动驾驶冒烟
mod art;
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
        // 绘制
        match g.scene {
            Scene::Menu => draw::draw_menu(&mut g, art::art()),
            Scene::Reality => draw::draw_reality(&mut g, art::art()),
            Scene::Draft => draw::draw_draft(&mut g, art::art()),
            Scene::Sim => draw::draw_sim(&mut g, art::art()),
            Scene::Inherit => draw::draw_inherit(&mut g, art::art()),
            Scene::Settle => draw::draw_settle(&mut g, art::art()),
            Scene::World => crate::world::draw_world(&mut g, art::art()),
        }
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
    if frame % 120 == 30 && *shots < 40 {
        let img = get_screen_data();
        let path = format!("shots/shot_{:03}.png", *shots);
        img.export_png(&path);
        *shots += 1;
        if *shots >= 40 {
            g.sync_save();
            std::process::exit(0);
        }
    }
    // 面板巡检：shots 24..30 依次打开 坊市(丹药)/坊市(藏经阁)/行囊/烙印/帮助/角色/烙印
    if *shots >= 24 && *shots < 31 && g.scene == Scene::Reality {
        let seq = [1usize, 1, 2, 3, 4, 5, 3];
        g.panel = seq[(*shots - 24) as usize] as u8;
        g.shop_tab = if *shots == 24 { 0 } else { 1 };
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
            // v1.4.0 镜头：31 地图全景｜32 选中落霞秘境｜33 赶路后｜34 远征第一步｜35-38 推进/链尾/返回
            if frame % 30 != 0 {
                return;
            }
            if *shots == 31 {
                crate::world::open_world(g);
            } else if *shots == 32 {
                g.world.sel = crate::data::NODE_RUINS;
            } else if *shots == 33 {
                crate::world::travel(g, crate::data::NODE_RUINS);
            } else if *shots == 34 {
                crate::world::exp_start(g);
            } else if *shots >= 35 {
                if g.world.exp.is_some() {
                    if crate::world::exp_done(g) {
                        crate::world::exp_quit(g); // 链尾归返
                    } else {
                        crate::world::exp_choose(g, 0);
                    }
                } else {
                    crate::world::close_world(g);
                }
            }
        }
    }
}
