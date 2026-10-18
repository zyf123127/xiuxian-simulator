// v1.4.0 山河图志：世界地图场景
// 赶路 / 宗门（拜入·任务·商店）/ 坊市入口 / 秘境远征 / 洞府闭关 / 荒骨寻仇 / 天雷峰渡劫
//
// 借用规范与 game.rs 一致：对 life / world.exp 的可变借用圈在小块内，日志音效存档在块外。
// 会话状态（sel/exp/shop）不进存档；持久化的只有 Life.loc（Lloc 键）与 Meta.sect_contrib（sc 键）。
use crate::art::Art;
use crate::data::*;
use crate::draw::{draw_bg, draw_banner, draw_motes, draw_toast, button, panel_card, version_mark};
use crate::game::*;
use crate::ui::*;
use macroquad::prelude::*;
use macroquad::rand::gen_range;

// ---------------- 会话状态（不进存档） ----------------

pub struct Exped {
    pub steps: Vec<usize>, // EXPED_STEPS 索引链
    pub cur: usize,        // 当前步（cur >= steps.len() 即完成）
    pub loot: Vec<String>, // 战利品记录
}

pub struct WorldState {
    pub sel: usize,            // 当前选中节点
    pub exp: Option<Exped>,    // 进行中的远征
    pub shop: bool,            // 宗门商店覆盖层
    pub dev: bool,             // 洞府经营覆盖层（v1.5.0）
}

impl WorldState {
    pub fn new() -> Self {
        WorldState {
            sel: NODE_START,
            exp: None,
            shop: false,
            dev: false,
        }
    }
}

// ---------------- 场景进出 ----------------

pub fn open_world(g: &mut Game) {
    if g.life.is_none() {
        return;
    }
    g.scene = Scene::World;
    g.panel = 0;
    g.world.sel = g.life.as_ref().map(|l| l.loc).unwrap_or(NODE_START);
    g.world.shop = false;
    g.world.dev = false;
    g.play(crate::sounds::Which::Page);
}

// Escape / 「返回现实」的单一出口；洞府经营覆盖层先于此关闭，远征中拒绝离场（P5 门①）
pub fn close_world(g: &mut Game) {
    if g.world.dev {
        g.world.dev = false;
        g.play(crate::sounds::Which::Page);
        return;
    }
    if g.world.exp.is_some() {
        g.toast("远征未竟，不可中途离场", C_RED);
        return;
    }
    g.scene = Scene::Reality;
    g.sync_save();
}

// ---------------- 时间与死亡 ----------------

// 推岁：与 game.rs act_cultivate 同源（l.age += 1.0）。
// 设计差异：旅途/秘境无养伤之地，不减 injured（对照 game.rs 的闭关减伤）。
// 返回 false 表示寿尽且已进入总结算，调用方必须立即 return。
fn spend_year(g: &mut Game, cause: &str) -> bool {
    let died = {
        let Some(l) = g.life.as_mut() else { return false };
        l.age += 1.0;
        l.age >= l.lifespan
    };
    if died {
        g.world.exp = None;
        g.real_die(cause);
        return false;
    }
    // v1.5.0 四时：推岁口唯一化（现实 act_cultivate + 此处），死亡当年不推时辰不产灵田
    g.advance_time();
    true
}

// ---------------- 行动 ----------------

pub fn travel(g: &mut Game, to: usize) {
    let to = to.min(LOCATIONS.len() - 1);
    let cur = g.life.as_ref().map(|l| l.loc).unwrap_or(NODE_START);
    if to == cur {
        g.toast("已在此地", C_DIM);
        return;
    }
    if !spend_year(g, "你于赶路途中旧伤复发，倒毙荒野。") {
        return;
    }
    let name = LOCATIONS[to].name;
    {
        let l = g.life.as_mut().unwrap();
        l.loc = to;
    }
    g.log(format!("你收拾行装赶路，此番抵达{}。", name), C_TEXT);
    g.play(crate::sounds::Which::Page);
    g.sync_save();
}

pub fn sect_join(g: &mut Game) {
    let (sect, realm) = {
        let l = g.life.as_ref().unwrap();
        (l.sect, l.realm)
    };
    if sect {
        g.toast("你已是青云宗弟子", C_DIM);
        return;
    }
    if realm < 1 {
        g.toast("根基尚浅，先修至炼气期再来", C_RED);
        return;
    }
    if !spend_year(g, "你于拜山途中遭劫，殒身林莽。") {
        return;
    }
    {
        let l = g.life.as_mut().unwrap();
        l.sect = true;
    }
    g.log(
        "你拜入青云宗，得授入门功法——环境加成×1.3，从此修炼事半功倍。",
        C_GREEN,
    );
    g.play(crate::sounds::Which::Ding);
    g.sync_save();
}

pub fn sect_quest(g: &mut Game) {
    let sect = g.life.as_ref().map_or(false, |l| l.sect);
    if !sect {
        g.toast("需先拜入青云宗", C_RED);
        return;
    }
    if !spend_year(g, "你于宗门任务中力战而亡。") {
        return;
    }
    let (realm, stones) = {
        let l = g.life.as_ref().unwrap();
        // v1.5.0：宗门任务收益随天气起伏（青云宗列）；v1.6.0 D：长老好感加成；
        // v1.8.0 C2：区域机制「宗门兴望」加成
        (
            l.realm,
            (quest_stones(l.realm) as f64
                * weather_loc_mult(l.weather as usize, NODE_SECT)
                * crate::npc::elder_quest_mult(g.meta.npc_fav[crate::npc::NPC_ELDER])
                * region_mech(g.meta.map_region).quest_mult) as i64,
        )
    };
    // v1.5.0：贡献同样随天气起伏；v1.6.0 D：长老好感加成；v1.8.0 C2：区域机制加成
    let contrib = (quest_contrib(realm) as f64
        * weather_loc_mult(
            g.life.as_ref().map(|l| l.weather as usize).unwrap_or(0),
            NODE_SECT,
        )
        * crate::npc::elder_quest_mult(g.meta.npc_fav[crate::npc::NPC_ELDER])
        * region_mech(g.meta.map_region).quest_mult) as u32;
    {
        let l = g.life.as_mut().unwrap();
        l.stones += stones;
    }
    g.meta.sect_contrib += contrib;
    // v1.6.0 D：为长老办差，积攒交情
    crate::npc::add_fav(g, crate::npc::NPC_ELDER, ELDER_QUEST_FAV);
    g.log(
        format!(
            "宗门任务完成：贡献+{}，灵石+{}。",
            contrib,
            fmt_int(stones)
        ),
        C_GOLD,
    );
    let lucky = {
        let l = g.life.as_ref().unwrap();
        gen_range(0.0, 1.0f32) < 0.05 + l.luck / 500.0
    };
    if lucky {
        let gain = {
            let l = g.life.as_ref().unwrap();
            qi_need(l) * 0.10
        };
        {
            let l = g.life.as_mut().unwrap();
            gain_qi(l, gain);
        }
        g.log("任务途中偶得灵机，修为大涨！", C_CYAN);
    }
    g.play(crate::sounds::Which::Coin);
    g.sync_save();
}

// 宗门商店：是否可兑换（贡献门 + 品类门）
fn sect_good_ok(g: &Game, i: usize) -> bool {
    if i >= SECT_GOODS.len() {
        return false;
    }
    let good = &SECT_GOODS[i];
    if g.meta.sect_contrib < good.cost {
        return false;
    }
    let Some(l) = &g.life else { return false };
    match &good.kind {
        SectGoodKind::Relic(r) => l.realm >= 4 && !l.relics.contains(r),
        SectGoodKind::Tech(t) => l.realm >= 4 && l.tech < *t,
        _ => true,
    }
}

pub fn sect_buy(g: &mut Game, i: usize) {
    if !sect_good_ok(g, i) {
        return;
    }
    let (name, cost) = (SECT_GOODS[i].name, SECT_GOODS[i].cost);
    g.meta.sect_contrib -= cost;
    let msg = {
        let l = g.life.as_mut().unwrap();
        match &SECT_GOODS[i].kind {
            SectGoodKind::Qi15 => {
                let v = qi_need(l) * 0.15;
                gain_qi(l, v);
                format!("服下{}，修为+{}", name, fmt_num(v))
            }
            SectGoodKind::Dao2 => {
                l.dao = (l.dao + 2.0).min(100.0);
                format!("服下{}，道心+2", name)
            }
            SectGoodKind::Body2 => {
                l.body = (l.body + 2.0).min(100.0);
                format!("服下{}，体魄+2", name)
            }
            SectGoodKind::Life8 => {
                l.lifespan += 8.0;
                format!("服下{}，寿元+8载", name)
            }
            SectGoodKind::Relic(r) => {
                l.relics.push(*r);
                format!("请得镇派法宝【{}】", RELICS[*r].name)
            }
            SectGoodKind::Tech(t) => {
                l.tech = *t;
                format!("请得《{}》真传", TECHS[*t].name)
            }
        }
    };
    g.log(format!("{}（贡献-{}）。", msg, cost), C_GOLD);
    g.play(crate::sounds::Which::Coin);
    g.sync_save();
}

pub fn exp_start(g: &mut Game) {
    if g.world.exp.is_some() {
        return;
    }
    let at = g.life.as_ref().map_or(false, |l| l.loc == NODE_RUINS);
    if !at {
        g.toast("需亲临落霞秘境", C_RED);
        return;
    }
    if !spend_year(g, "你殒落于落霞秘境深处，无人收殓。") {
        return;
    }
    let len = EXPED_LEN_BASE + gen_range(0usize, 3usize);
    let mut idxs: Vec<usize> = (0..EXPED_STEPS.len()).collect();
    for k in (1..idxs.len()).rev() {
        let j = gen_range(0usize, k + 1usize);
        idxs.swap(k, j);
    }
    idxs.truncate(len);
    g.world.exp = Some(Exped {
        steps: idxs,
        cur: 0,
        loot: Vec::new(),
    });
    g.log("你踏入落霞秘境，霞光在身后合拢——远征开始了。", C_CYAN);
    g.play(crate::sounds::Which::Page);
    g.sync_save();
}

pub fn exp_done(g: &Game) -> bool {
    g.world
        .exp
        .as_ref()
        .map_or(false, |e| e.cur >= e.steps.len())
}

fn push_loot(g: &mut Game, s: String) {
    if let Some(e) = g.world.exp.as_mut() {
        e.loot.push(s);
    }
}

// 远征抉择：i=0 激进（成功率-0.10、收益×1.5）；i=1 谨慎（+0.10、×0.6）
pub fn exp_choose(g: &mut Game, i: usize) {
    if exp_done(g) {
        return;
    }
    let step_idx = {
        let e = g.world.exp.as_ref().unwrap();
        e.steps[e.cur]
    };
    let kind = EXPED_STEPS[step_idx].kind as usize;
    // 先耗时后判定（途中寿尽则已入结算）
    if !spend_year(g, "你殒落于落霞秘境深处，无人收殓。") {
        return;
    }
    let (padj, mult) = if i == 0 {
        (-0.10f32, 1.5f32)
    } else {
        (0.10f32, 0.6f32)
    };
    // 成功率（P4：power(l) 解析到自由函数 pub fn power(l:&Life)；stone_scale 走 g 的方法）
    let realm = {
        let l = g.life.as_ref().unwrap();
        l.realm
    };
    // v1.5.0：秘境战获随天气起伏（落霞秘境列）
    let wx = g
        .life
        .as_ref()
        .map(|l| l.weather as usize)
        .unwrap_or(0);
    let wxm = weather_loc_mult(wx, NODE_RUINS);
    // v1.8.0 C2：区域机制（大漠秘藏/瘴惑灵芽影响远征收益与风险）
    let rm = region_mech(g.meta.map_region);
    let p = {
        let l = g.life.as_ref().unwrap();
        let base = match kind {
            0 => {
                0.55
                    + ((power(l) / (REALMS[l.realm].power * 1.2) - 1.0).clamp(-0.2, 0.2)) as f32
                    + l.body / 500.0
            }
            1 => 0.65 + l.luck / 400.0,
            2 => 0.50 + l.dao / 400.0,
            _ => 0.85 + l.dao / 600.0,
        };
        (base + padj - rm.exped_risk).clamp(0.05, 0.95)
    };
    let ok = gen_range(0.0, 1.0f32) < p;
    let cur = g.world.exp.as_ref().unwrap().cur;
    if !ok {
        {
            let l = g.life.as_mut().unwrap();
            l.injured = l.injured.saturating_add(gen_range(2i32, 5i32) as u16);
        }
        g.log(
            format!("远征第{}步受挫，仓皇而退，身受重伤！", cur + 1),
            C_RED,
        );
        g.play(crate::sounds::Which::BreakFail);
    } else {
        match kind {
            0 => {
                // 战：灵石 + 25% 掉宝（查重，已有则折算——先例 game.rs take_inherit 法宝）
                let scale = g.stone_scale(realm);
                let s = (EXPED_STONE * scale * mult as f64 * wxm * rm.exped_mult) as i64;
                {
                    let l = g.life.as_mut().unwrap();
                    l.stones += s;
                }
                let mut msg = format!("击退强敌，缴获灵石 {}", fmt_int(s));
                if gen_range(0.0, 1.0f32) < 0.25 {
                    let r = gen_range(0usize, 3usize);
                    let (has, conv) = {
                        let l = g.life.as_ref().unwrap();
                        (l.relics.contains(&r), (2000.0 * g.stone_scale(realm)) as i64)
                    };
                    if !has {
                        {
                            let l = g.life.as_mut().unwrap();
                            l.relics.push(r);
                        }
                        msg += &format!("，并寻得法宝【{}】", RELICS[r].name);
                    } else {
                        {
                            let l = g.life.as_mut().unwrap();
                            l.stones += conv;
                        }
                        msg += &format!("，余宝折算灵石 {}", fmt_int(conv));
                    }
                }
                push_loot(g, msg);
                g.play(crate::sounds::Which::Coin);
            }
            1 => {
                // 药：聚气丹+1 培元丹+1 + 修为
                let gain = {
                    let l = g.life.as_ref().unwrap();
                    qi_need(l) * EXPED_HERB_QI * mult as f64 * rm.exped_mult
                };
                {
                    let l = g.life.as_mut().unwrap();
                    l.pills[0] += 1;
                    l.pills[3] += 1;
                    gain_qi(l, gain);
                }
                push_loot(g, format!("采得灵药：聚气丹+1、培元丹+1，修为+{}", fmt_num(gain)));
                g.play(crate::sounds::Which::Gain);
            }
            2 => {
                // 宝：50% 灵石 / 50% 贡献
                if gen_range(0.0, 1.0f32) < 0.5 {
                    let scale = g.stone_scale(realm);
                    let s = (EXPED_STONE_BONE * scale * mult as f64 * wxm * rm.exped_mult) as i64;
                    {
                        let l = g.life.as_mut().unwrap();
                        l.stones += s;
                    }
                    push_loot(g, format!("破禁得宝，折算灵石 {}", fmt_int(s)));
                } else {
                    let c = (EXPED_CONTRIB as f32 * mult * rm.exped_mult as f32) as u32;
                    g.meta.sect_contrib += c;
                    push_loot(g, format!("得宗门信物，贡献+{}", c));
                }
                g.play(crate::sounds::Which::Coin);
            }
            _ => {
                // 奇：修为
                let gain = {
                    let l = g.life.as_ref().unwrap();
                    qi_need(l) * EXPED_QI_PCT * mult as f64 * rm.exped_mult
                };
                {
                    let l = g.life.as_mut().unwrap();
                    gain_qi(l, gain);
                }
                push_loot(g, format!("静悟道韵，修为+{}", fmt_num(gain)));
                g.play(crate::sounds::Which::Gain);
            }
        }
    }
    // 推进步数（圈块）
    {
        let e = g.world.exp.as_mut().unwrap();
        e.cur += 1;
    }
    g.log(
        format!("秘境远征·第{}步：{}", cur + 1, EXPED_STEPS[step_idx].text),
        C_TEXT,
    );
    g.sync_save();
}

// ---------------- 洞府经营（v1.5.0） ----------------

// 升级设施 i（0 聚灵阵 1 灵田 2 丹房）：灵石 + 贡献双代价
pub fn dongfu_up(g: &mut Game, i: usize) {
    if i >= 3 {
        return;
    }
    let lv = g.meta.dongfu[i].clamp(0, 3);
    if lv >= 3 {
        g.toast("此设施已臻圆满", C_DIM);
        return;
    }
    let stones = DONGFU_UP_STONES[lv as usize];
    let contrib = DONGFU_UP_CONTRIB[lv as usize];
    let ok_stones = g.life.as_ref().map_or(false, |l| l.stones >= stones);
    let ok_contrib = g.meta.sect_contrib >= contrib;
    if !ok_stones || !ok_contrib {
        g.toast("灵石或贡献不足", C_RED);
        return;
    }
    {
        let l = g.life.as_mut().unwrap();
        l.stones -= stones;
    }
    g.meta.sect_contrib -= contrib;
    g.meta.dongfu[i] = lv + 1;
    g.log(
        format!(
            "洞府{}扩建至{}级。（灵石-{}，贡献-{}）",
            DONGFU_FACS[i],
            lv + 1,
            fmt_int(stones),
            contrib
        ),
        C_GOLD,
    );
    g.play(crate::sounds::Which::Ascend);
    g.sync_save();
}

// 链中收手与链尾归返共用，不耗时
pub fn exp_quit(g: &mut Game) {
    let loot = g
        .world
        .exp
        .as_ref()
        .map(|e| e.loot.clone())
        .unwrap_or_default();
    if loot.is_empty() {
        g.log("你自秘境收手归返，此行平平。", C_DIM);
    } else {
        g.log("远征归来，此行所获：", C_GOLD);
        for s in &loot {
            g.log(format!("· {}", s), C_TEXT);
        }
    }
    g.world.exp = None;
    g.sync_save();
}

// ---------------- 绘制 ----------------

pub fn draw_world(g: &mut Game, art: &Art) {
    let Some(l) = g.life.clone() else {
        g.scene = Scene::Menu;
        return;
    };
    // v1.8.0 D2：好感事件卡模态（世界场景亦独占输入，罩层在 draw_modal_card 内）
    if g.rmodal.is_some() {
        draw_bg(art);
        crate::draw::draw_modal_card(g);
        version_mark();
        return;
    }
    // 覆盖层门控（P5 四处一致的总闸；v1.5.0 增洞府经营；v1.6.0 增山河弹层）
    let gate = g.world.exp.is_none() && !g.world.shop && !g.world.dev && g.submenu.is_none();
    draw_bg(art);
    // 地图底图（画布 1264×480，屏幕偏移 (8,60)；v1.6.0 按主题/种子取缓存）
    let map_bg = crate::art::map_bg_for(g.meta.map_region, g.meta.map_seed);
    draw_texture_ex(
        &map_bg,
        8.0,
        60.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(1264.0, 480.0)),
            ..Default::default()
        },
    );
    // v1.5.0 四时：时段/天气叠色（a=0 跳过不画）
    {
        let (sr, sg, sb, sa) = SLOT_TINT[l.slot.min(3) as usize];
        if sa > 0 {
            draw_rectangle(8.0, 60.0, 1264.0, 480.0, Color::from_rgba(sr, sg, sb, sa));
        }
        let (wr, wg, wb, wa) = WEATHER_TINT[l.weather.min(4) as usize];
        if wa > 0 {
            draw_rectangle(8.0, 60.0, 1264.0, 480.0, Color::from_rgba(wr, wg, wb, wa));
        }
    }
    // v1.5.0 水面波光：河面流光 5 点（map_river_y 取图像系坐标，屏幕 x 需 -8 校正）
    for k in 0..5 {
        let sx = ((g.time * 46.0 + k as f32 * 253.0) % 1264.0) + 8.0;
        let sy = crate::art::map_river_y(sx - 8.0) + 60.0;
        let a = 0.16 + 0.12 * (g.time * 2.4 + k as f32 * 1.7).sin();
        draw_line(
            sx - 10.0,
            sy,
            sx + 10.0,
            sy,
            1.6,
            Color::new(0.75, 0.92, 0.95, a),
        );
    }
    // v1.5.0 飞鸟 3 只（+8/+60 偏移入地图画布）
    for i in 0..3 {
        let bx = ((g.time * (26.0 + i as f32 * 9.0) + i as f32 * 480.0) % 1500.0) - 120.0 + 8.0;
        let by = 130.0 + i as f32 * 46.0 + (g.time * (1.3 + i as f32 * 0.4) + i as f32 * 2.1).sin() * 24.0 + 60.0;
        let flap = (g.time * 6.0 + i as f32 * 0.9).sin() * 3.5;
        draw_line(bx - 7.0, by - flap, bx, by, 1.5, Color::new(0.85, 0.82, 0.75, 0.7));
        draw_line(bx, by, bx + 7.0, by - flap, 1.5, Color::new(0.85, 0.82, 0.75, 0.7));
    }
    // v1.5.0 洞府炊烟：清源洞府节点上方（v1.6.0 锚点随主题 node_xy 抖动）
    {
        let (dfx, dfy) = node_xy(NODE_DONGFU, g.meta.map_region);
        for k in 0..4 {
            let ph = (g.time * 0.42 + k as f32 * 0.25) % 1.0;
            let sx = dfx + 8.0 + (g.time * 0.9 + k as f32 * 1.7).sin() * 5.0 + ph * 9.0;
            let sy = dfy + 60.0 - ph * 30.0;
            let a = (1.0 - ph) * 0.32;
            draw_circle(sx, sy, 2.6 + ph * 4.2, Color::new(0.78, 0.78, 0.84, a));
        }
    }
    draw_motes(g);
    // 路径虚线（半透明金，每 18px 留空；v1.6.0 坐标经 node_xy 含主题抖动）
    for (a, b) in MAP_PATHS {
        let (x0, y0) = {
            let (ax, ay) = node_xy(a, g.meta.map_region);
            (ax + 8.0, ay + 60.0)
        };
        let (x1, y1) = {
            let (bx, by) = node_xy(b, g.meta.map_region);
            (bx + 8.0, by + 60.0)
        };
        let (dx, dy) = (x1 - x0, y1 - y0);
        let len = (dx * dx + dy * dy).sqrt();
        let n = (len / 18.0).max(1.0);
        for k in 0..n as i32 {
            let t0 = k as f32 / n;
            let t1 = t0 + 0.55 / n;
            draw_line(
                x0 + dx * t0,
                y0 + dy * t0,
                x0 + dx * t1,
                y0 + dy * t1,
                1.5,
                Color::new(1.0, 0.85, 0.5, 0.28),
            );
        }
    }
    // v1.5.0 路线高亮：与选中节点相连的边描金
    {
        let sel = g.world.sel.min(LOCATIONS.len() - 1);
        for (a, b) in MAP_PATHS {
            if a == sel || b == sel {
                let (x0, y0) = {
                    let (ax, ay) = node_xy(a, g.meta.map_region);
                    (ax + 8.0, ay + 60.0)
                };
                let (x1, y1) = {
                    let (bx, by) = node_xy(b, g.meta.map_region);
                    (bx + 8.0, by + 60.0)
                };
                draw_line(x0, y0, x1, y1, 2.5, Color::new(1.0, 0.85, 0.5, 0.55));
            }
        }
    }
    // 云雾（4 团 ×3 圆）
    for i in 0..4 {
        let cx = ((g.time * 10.0 + i as f32 * 340.0) % 1500.0) - 120.0;
        let cy = 100.0 + (i % 3) as f32 * 120.0;
        for k in 0..3 {
            draw_ellipse(
                cx + k as f32 * 42.0 - 42.0,
                cy + k as f32 * 9.0,
                88.0,
                15.0,
                0.0,
                Color::new(0.7, 0.68, 0.85, 0.05),
            );
        }
    }
    // 节点 ×6（v1.6.0：坐标经 node_xy 含主题抖动）
    let sel = g.world.sel.min(LOCATIONS.len() - 1);
    for (i, nd) in LOCATIONS.iter().enumerate() {
        let (bx, by) = node_xy(i, g.meta.map_region);
        let nx = bx + 8.0;
        let ny = by + 60.0;
        let r = 24.0 + 3.5 * (g.time * 2.0 + i as f32).sin();
        let hot = sel == i;
        draw_circle_lines(
            nx,
            ny,
            r,
            1.5,
            if hot {
                C_GOLD
            } else {
                Color::new(0.9, 0.85, 0.7, 0.45)
            },
        );
        if hot {
            draw_circle_lines(nx, ny, r + 5.0, 1.5, C_GOLD);
            draw_circle_lines(nx, ny, r + 9.0, 1.0, Color::new(1.0, 0.85, 0.5, 0.5));
        }
        draw_texture_ex(
            &art.map_icons[i],
            nx - 18.0,
            ny - 18.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(36.0, 36.0)),
                ..Default::default()
            },
        );
        txt_c(nd.name, nx, ny + 30.0, 16, if hot { C_GOLD } else { C_TEXT });
    }
    // 玩家标记：金点 + 青色 8 段旋转刻度圈
    {
        let (bx, by) = node_xy(l.loc.min(LOCATIONS.len() - 1), g.meta.map_region);
        let (px, py) = (bx + 8.0, by + 60.0);
        draw_circle(px, py, 4.5, C_GOLD);
        draw_circle_lines(px, py, 6.5, 2.0, C_CYAN);
        for k in 0..8 {
            let ang = g.time * 1.2 + std::f32::consts::TAU * k as f32 / 8.0;
            draw_line(
                px + ang.cos() * 12.0,
                py + ang.sin() * 12.0,
                px + ang.cos() * 17.0,
                py + ang.sin() * 17.0,
                2.0,
                C_CYAN,
            );
        }
        txt_c("你在此", px, py + 48.0, 13, C_CYAN);
    }
    // 点击选点（P5 门④：覆盖层打开时静默；v1.5.0 经信箱相机回虚拟坐标）
    {
        let (mx, my) = crate::cam::mouse();
        if gate && is_mouse_button_pressed(MouseButton::Left) {
            for (i, _) in LOCATIONS.iter().enumerate() {
                let (bx, by) = node_xy(i, g.meta.map_region);
                let (nx, ny) = (bx + 8.0, by + 60.0);
                let d2 = (mx - nx) * (mx - nx) + (my - ny) * (my - ny);
                if d2 < 28.0 * 28.0 {
                    if g.world.sel != i {
                        g.world.sel = i;
                        g.play(crate::sounds::Which::Click);
                        // v1.6.0 B：非原地节点直接开「前往确认」弹层（经 open_sub 打开帧门闩）
                        if i != l.loc {
                            crate::menu::open_sub(g, crate::menu::Sub::MapGo(i));
                        }
                    }
                    break;
                }
            }
        }
    }
    // ---------- 顶栏 ----------
    draw_rectangle(0.0, 0.0, V_W, 52.0, Color::from_rgba(16, 11, 30, 245));
    draw_rectangle(0.0, 50.0, V_W, 2.0, C_GOLD_D);
    txt("—— 山河图志 ——", 16.0, 15.0, 20, C_GOLD);
    // v1.6.0：顶栏追加当前地区名（时段/天气、「所在」相应右移让位）
    txt(
        &format!("· {}", theme_of(g.meta.map_region).name),
        216.0,
        17.0,
        15,
        C_GOLD_D,
    );
    // v1.8.0 C4：顶栏追加区域机制短标（时段/天气、「所在」相应右移让位）
    txt(
        region_mech(g.meta.map_region).mech,
        320.0,
        17.0,
        14,
        C_CYAN,
    );
    // v1.5.0 四时：顶栏时段·天气（v1.8.0 C4：340→430）
    txt(
        &format!(
            "{}·{}",
            SLOTS[l.slot.min(3) as usize], WEATHERS[l.weather.min(4) as usize]
        ),
        430.0,
        16.0,
        15,
        C_CYAN,
    );
    txt(
        &format!(
            "所在：{}",
            LOCATIONS[l.loc.min(LOCATIONS.len() - 1)].name
        ),
        530.0,
        15.0,
        16,
        C_TEXT,
    );
    draw_texture(&crate::art::art().icon_stone, 988.0, 16.0, WHITE);
    txt(&fmt_int(l.stones), 1006.0, 15.0, 17, C_CYAN);
    txt(
        &format!("贡献 {}", fmt_int(g.meta.sect_contrib as i64)),
        1080.0,
        15.0,
        16,
        C_GOLD,
    );
    // ---------- 主卡（左下） ----------
    panel_card(16.0, 552.0, 900.0, 156.0);
    let nd = &LOCATIONS[sel];
    txt(nd.name, 36.0, 564.0, 20, C_GOLD);
    // v1.8.0 C4：行右上机制短标·说明（区域专属机制一眼可读）
    {
        let mech = region_mech(g.meta.map_region);
        txt_r(
            &format!("{} · {}", mech.mech, mech.desc),
            900.0,
            566.0,
            14,
            C_CYAN,
        );
    }
    if sel == l.loc {
        txt("当前所在", 140.0, 564.0, 13, C_DIM);
    } else {
        // v1.8.0 A8：13→14 字号下限
        txt("尚未抵达", 140.0, 564.0, 14, C_RED);
    }
    wrap_draw(nd.desc, 36.0, 592.0, 14, 560.0, 24.0, C_DIM); // v1.6.0：行距 20→24（desc 3 行预算内）
    txt("可做", 36.0, 664.0, 14, C_DIM);
    // 动作区（P5 门②：覆盖层打开时不渲染；v1.6.0 B：异地「赶路」收进山河弹层——
    // 点节点即弹 Sub::MapGo 确认，主卡只保留原地动作）
    if gate {
        if sel == l.loc {
            match sel {                NODE_SECT => {
                    if button(
                        100.0,
                        640.0,
                        180.0,
                        56.0,
                        "拜入青云",
                        Some("寿元-1载"),
                        !l.sect && l.realm >= 1,
                    ) {
                        g.play(crate::sounds::Which::Click);
                        sect_join(g);
                    }
                    if button(
                        290.0,
                        640.0,
                        180.0,
                        56.0,
                        "宗门任务",
                        Some("1年 · 贡献+灵石"),
                        l.sect,
                    ) {
                        g.play(crate::sounds::Which::Click);
                        sect_quest(g);
                    }
                    if button(
                        480.0,
                        640.0,
                        180.0,
                        56.0,
                        "宗门商店",
                        Some(&format!("贡献 {}", fmt_int(g.meta.sect_contrib as i64))),
                        true,
                    ) {
                        g.play(crate::sounds::Which::Page);
                        g.world.shop = true;
                    }
                }
                NODE_MARKET => {
                    if button(
                        100.0,
                        640.0,
                        220.0,
                        56.0,
                        "打开坊市",
                        Some("本地丹药九折"),
                        true,
                    ) {
                        g.play(crate::sounds::Which::Page);
                        g.scene = Scene::Reality;
                        g.panel = 1;
                    }
                }
                NODE_RUINS => {
                    if button(
                        100.0,
                        640.0,
                        220.0,
                        56.0,
                        "开始远征",
                        Some("寿元-1载 · 步步凶险"),
                        g.world.exp.is_none(),
                    ) {
                        g.play(crate::sounds::Which::Click);
                        exp_start(g);
                    }
                }
                NODE_DONGFU => {
                    if button(
                        100.0,
                        640.0,
                        220.0,
                        56.0,
                        "闭关修炼",
                        Some("洞府灵气 · +25%"),
                        true,
                    ) {
                        g.play(crate::sounds::Which::Click);
                        g.act_cultivate();
                    }
                    // v1.5.0 洞府经营入口
                    if button(
                        330.0,
                        640.0,
                        220.0,
                        56.0,
                        "经营洞府",
                        Some("聚灵阵·灵田·丹房"),
                        true,
                    ) {
                        g.play(crate::sounds::Which::Page);
                        g.world.dev = true;
                    }
                }
                NODE_BONES => {
                    if g.can_revenge() {
                        let neme = g.meta.neme.clone().unwrap();
                        let p = g.revenge_p();
                        if button(
                            100.0,
                            640.0,
                            220.0,
                            56.0,
                            "寻仇",
                            Some(&format!("{:.0}% · {}", p * 100.0, neme.name)),
                            true,
                        ) {
                            g.play(crate::sounds::Which::Click);
                            g.act_revenge();
                        }
                    } else if button(100.0, 640.0, 220.0, 56.0, "宿敌已清", Some("静待新敌"), false)
                    {
                    }
                }
                _ => {
                    // 天雷峰
                    if button(
                        100.0,
                        640.0,
                        220.0,
                        56.0,
                        "冲击境界",
                        Some("现实突破 · 不死"),
                        g.can_break(),
                    ) {
                        g.play(crate::sounds::Which::Click);
                        g.act_break();
                    }
                    txt(
                        "※ 渡劫台：冲击大境界时每重雷劫存活+2%",
                        340.0,
                        664.0,
                        14, // v1.8.0 A8：13→14 字号下限
                        C_CYAN,
                    );
                }
            }
        } else {
            // v1.8.0 A5：异地主卡不再空置——一键弹「前往确认」层（与点节点同路径）
            if button(
                100.0,
                640.0,
                220.0,
                56.0,
                "前往此地",
                Some("寿元-1载 · 弹层确认"),
                true,
            ) {
                g.play(crate::sounds::Which::Page);
                crate::menu::open_sub(g, crate::menu::Sub::MapGo(sel));
            }
        }
    } else if g.world.exp.is_some() {
        // v1.5.0（建议②）：仅远征中提示误文案；洞府经营覆盖层打开时此处静默
        txt(
            &format!(
                "可做：{}——远征进行中，暂不可操作。",
                nd.acts.join("／")
            ),
            100.0,
            664.0,
            14,
            C_DIM,
        );
    }
    // ---------- 右下卡（归返） ----------
    panel_card(928.0, 552.0, 336.0, 132.0);
    // v1.8.0 C3：返回现实钮 48→44 高，为区域图鉴钮（618..662）让位
    if button(952.0, 566.0, 288.0, 44.0, "返回现实", None, gate) {
        g.play(crate::sounds::Which::Page);
        close_world(g);
    }
    // v1.8.0 C3：区域图鉴入口（gate 已含 exp.is_none()——远征中为禁用态，仍绘制）
    if button(
        952.0,
        618.0,
        288.0,
        44.0,
        "区域图鉴",
        Some("五域机制 · 初至有赏"),
        gate,
    ) {
        g.play(crate::sounds::Which::Page);
        crate::menu::open_sub(g, crate::menu::Sub::Atlas);
    }
    // v1.8.0 C3+A8：提示两行合并单行 y=666（否则被图鉴钮矩形下缘叠压），字号 13→14
    if g.world.exp.is_some() {
        txt("远征未竟，完成或收手后方可归返。", 952.0, 666.0, 14, C_RED);
    } else {
        txt("Esc 归返 · M 再临 · 点节点选定目的地。", 952.0, 666.0, 14, C_DIM);
    }
    // ---------- 商店覆盖层 ----------
    if g.world.shop {
        draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, 0.6));
        panel_card(240.0, 100.0, 800.0, 520.0);
        txt("青云宗 · 藏宝阁", 270.0, 132.0, 24, C_GOLD);
        txt(
            &format!("贡献 {}", fmt_int(g.meta.sect_contrib as i64)),
            620.0,
            132.0,
            18,
            C_GOLD,
        );
        if button(976.0, 114.0, 44.0, 36.0, "×", None, true) {
            g.world.shop = false;
            g.play(crate::sounds::Which::Page);
        }
        draw_rectangle(260.0, 152.0, 760.0, 1.5, Color::from_rgba(96, 76, 130, 160));
        for i in 0..SECT_GOODS.len() {
            let y = 170.0 + i as f32 * 74.0;
            let good = &SECT_GOODS[i];
            let ok = sect_good_ok(g, i);
            draw_rectangle(
                260.0,
                y - 8.0,
                760.0,
                62.0,
                Color::from_rgba(255, 255, 255, 5),
            );
            txt(good.name, 276.0, y + 10.0, 18, C_GOLD);
            txt(good.desc, 410.0, y + 10.0, 14, C_TEXT);
            txt_r(
                &format!("{} 贡献", good.cost),
                930.0,
                y + 10.0,
                15,
                if g.meta.sect_contrib >= good.cost {
                    C_CYAN
                } else {
                    C_RED
                },
            );
            if button(950.0, y - 2.0, 70.0, 38.0, "兑换", None, ok) {
                sect_buy(g, i);
            }
        }
    }
    // ---------- 洞府经营覆盖层（v1.5.0 四时洞天） ----------
    if g.world.dev {
        draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, 0.6));
        panel_card(240.0, 100.0, 800.0, 520.0);
        txt("洞 府 经 营", 270.0, 132.0, 24, C_GOLD);
        txt(
            &format!(
                "今岁 {} · {}",
                SLOTS[l.slot.min(3) as usize],
                WEATHERS[l.weather.min(4) as usize]
            ),
            560.0,
            136.0,
            16,
            C_CYAN,
        );
        if button(976.0, 114.0, 44.0, 36.0, "×", None, true) {
            g.world.dev = false;
            g.play(crate::sounds::Which::Page);
        }
        draw_rectangle(260.0, 152.0, 760.0, 1.5, Color::from_rgba(96, 76, 130, 160));
        txt(
            "四时流转，气象各异——设施升级需灵石与宗门贡献。",
            270.0,
            166.0,
            13,
            C_DIM,
        );
        for i in 0..3usize {
            let y = 196.0 + i as f32 * 132.0;
            let lv = g.meta.dongfu[i].clamp(0, 3) as usize;
            draw_texture(&crate::art::art().icon_yun, 270.0, y, WHITE);
            txt(DONGFU_FACS[i], 300.0, y + 2.0, 20, C_GOLD);
            txt(
                &format!("{}级", lv),
                400.0,
                y + 6.0,
                16,
                if lv >= 3 { C_GOLD } else { C_TEXT },
            );
            txt(DONGFU_FAC_DESC[i], 300.0, y + 34.0, 13, C_DIM);
            let eff = match i {
                0 => format!("当前：修炼效率 ×{:.2}", JUELING_MULT[lv]),
                1 => format!("当前：每载产灵石 {}", LINGTIAN_YIELD[lv]),
                _ => format!("当前：坊市丹价 ×{:.2}", DANFANG_DISC[lv]),
            };
            txt(&eff, 300.0, y + 58.0, 14, C_CYAN);
                if lv < 3 {
                    let (cs, cc) = (DONGFU_UP_STONES[lv], DONGFU_UP_CONTRIB[lv]);
                    let ok = l.stones >= cs && g.meta.sect_contrib >= cc;
                    txt(
                        &format!(
                            "升{}级：{} 灵石 + {} 贡献",
                            lv + 1,
                            fmt_int(cs),
                            cc
                        ),
                        540.0,
                        y + 58.0,
                        13,
                        if ok { C_TEXT } else { C_RED },
                    );
                    // v1.6.0：扩建钮 120px 宽（96px 装不下“扩建”两字 14px 下限渲染）
                    if button(896.0, y + 12.0, 120.0, 44.0, "扩建", None, ok) {
                        dongfu_up(g, i);
                    }
                } else {
                txt("圆满", 940.0, y + 58.0, 16, C_GOLD);
            }
        }
    }
    // ---------- 远征覆盖层 ----------
    if g.world.exp.is_some() {
        draw_rectangle(0.0, 0.0, V_W, V_H, Color::new(0.0, 0.0, 0.0, 0.6));
        panel_card(240.0, 140.0, 800.0, 420.0);
        txt_c("落 霞 秘 境 · 远 征", 640.0, 170.0, 24, C_GOLD);
        if exp_done(g) {
            // 链尾：只渲染 loot + 收手归返
            txt_c("远 征 完 成", 640.0, 206.0, 16, C_GREEN);
            let loot = g
                .world
                .exp
                .as_ref()
                .map(|e| e.loot.clone())
                .unwrap_or_default();
            if loot.is_empty() {
                txt("（一无所获）", 280.0, 250.0, 15, C_DIM);
            }
            for (k, s) in loot.iter().take(8).enumerate() {
                txt(&format!("· {}", s), 280.0, 250.0 + k as f32 * 26.0, 15, C_TEXT);
            }
            if button(450.0, 490.0, 180.0, 44.0, "收手归返", None, true) {
                g.play(crate::sounds::Which::Page);
                exp_quit(g);
            }
        } else {
            let (cur, len, step_text, kind) = {
                let e = g.world.exp.as_ref().unwrap();
                let st = &EXPED_STEPS[e.steps[e.cur]];
                (e.cur, e.steps.len(), st.text, st.kind as usize)
            };
            // 步进点
            for k in 0..len {
                let x = 640.0 - (len as f32 - 1.0) * 14.0 + k as f32 * 28.0;
                draw_circle(
                    x,
                    206.0,
                    5.0,
                    if k < cur {
                        C_GOLD
                    } else if k == cur {
                        C_CYAN
                    } else {
                        Color::from_rgba(90, 74, 120, 255)
                    },
                );
            }
            wrap_draw(step_text, 280.0, 240.0, 17, 720.0, 26.0, C_TEXT);
            // v1.6.0 B：双钮纵排 + 副标签（收益/风险一眼可读，替代原单行提示文案）
            let labels = EXPED_CHOICE_LABELS[kind.min(3)];
            if button(
                290.0,
                396.0,
                620.0,
                52.0,
                labels[0],
                Some("收获 ×1.5 · 风险 +10%"),
                true,
            ) {
                g.play(crate::sounds::Which::Click);
                exp_choose(g, 0);
            }
            if button(
                290.0,
                456.0,
                620.0,
                52.0,
                labels[1],
                Some("稳妥 ×0.6 · 风险 -10%"),
                true,
            ) {
                g.play(crate::sounds::Which::Click);
                exp_choose(g, 1);
            }
            if button(450.0, 514.0, 180.0, 40.0, "就此收手", None, true) {
                g.play(crate::sounds::Which::Page);
                exp_quit(g);
            }
        }
    }
    // v1.6.0 B：山河图确认弹层（点节点后出现；sub_fresh 开帧只显示不响应）
    if g.submenu.is_some() {
        crate::menu::draw_sub(g);
    }
    draw_banner(g);
    draw_toast(g);
    version_mark();
}
