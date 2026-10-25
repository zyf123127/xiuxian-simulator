// v1.6.0 D 好感度 NPC 关系系统：
// Meta.npc_fav[4]（跨世持久）+ Life.comp_fav（道侣，随克隆入模拟、遗泽归现实）。
// 表驱动（data.rs）：档次阈值 [0,20,40,70,95]，各增益按档取值。
// v1.8.0 D1：扩四人（+宁清霜/醉道人）；D2：好感专属事件（rmodal 现实侧模态通道）。
use crate::data::*;
use crate::game::{
    fmt_num, gain_qi, qi_need, CardKind, Eff, EventCard, Game, Scene, C_GOLD, C_RED,
};

pub const NPC_ELDER: usize = 0; // 玄诚长老（宗门任务收益）
pub const NPC_MERCHANT: usize = 1; // 胡掌柜（坊市丹价折扣）
pub const NPC_SENIOR: usize = 2; // v1.8.0 D1：宁清霜（青云宗大师姐——现实跨境破境）
pub const NPC_DRUNK: usize = 3; // v1.8.0 D1：醉道人（散修酒友——修炼效率）
pub const NPC_COUNT: usize = 4;

// 好感 → 档次（0..=4）
pub fn tier_of(fav: i32) -> usize {
    let mut t = 0;
    for (i, &th) in NPC_FAV_TIERS.iter().enumerate() {
        if fav >= th {
            t = i;
        }
    }
    t
}

pub fn tier_name(fav: i32) -> &'static str {
    NPC_TIER_NAMES[tier_of(fav)]
}

// ---- 增益曲线（data.rs 表的取值包装） ----
pub fn elder_quest_mult(fav: i32) -> f64 {
    ELDER_QUEST_MULT[tier_of(fav)]
}

pub fn merchant_disc(fav: i32) -> f64 {
    MERCHANT_DISC[tier_of(fav)]
}

pub fn comp_cult_mult(fav: i32) -> f64 {
    COMPANION_CULT_MULT[tier_of(fav)]
}

// v1.8.0 D1：大师姐「剑意点拨」——现实跨境破境成功率加成（同境/问笔不加）
pub fn senior_break_add(fav: i32) -> f32 {
    SENIOR_BREAK_ADD[tier_of(fav)]
}

// v1.8.0 D1：醉道人「酒中真言」——每载修炼效率倍率（cult_gain_per_year 挂点）
pub fn drunk_cult_mult(fav: i32) -> f64 {
    DRUNK_CULT_MULT[tier_of(fav)]
}

// 赠礼造价：随档次水涨船高（基础 500 × (1+档位)）
pub fn gift_cost(fav: i32) -> i64 {
    NPC_GIFT_BASE * (1 + tier_of(fav) as i64)
}

// ---- 好感写入 ----
// NPC 好感（clamp 上限；调用点在块外，无借用冲突）
// v1.8.0 D2：写入后检查好感专属事件触发（达标、未抉择过、非模拟、无悬空事件）
pub fn add_fav(g: &mut Game, i: usize, d: i32) {
    if i >= NPC_COUNT {
        return;
    }
    let v = g.meta.npc_fav[i].clamp(0, NPC_FAV_MAX);
    g.meta.npc_fav[i] = (v + d).clamp(0, NPC_FAV_MAX);
    maybe_fav_event(g, i);
}

// v1.8.0 D2：好感专属事件触发——置 rmodal/rmodal_npc，此处置 Some 但**不置位 npc_ev**
//（抉择时才消耗：强杀进程不丢事件，npc_ev 随 choose_roption 的 sync_save 落盘）。
// add_fav 调用点恰 3 处（宗门任务/坊市购丹/人物页赠礼）均在现实/世界场景；
// 模拟赠礼直写 comp_fav（game.rs sim 块内）不经过此口。
fn maybe_fav_event(g: &mut Game, i: usize) {
    if i >= NPC_COUNT
        || g.meta.npc_ev[i]
        || g.meta.npc_fav[i] < NPC_EVENT_FAV
        || g.scene == Scene::Sim
        || g.rmodal.is_some()
    {
        return;
    }
    if let Some(card) = npc_event_card(i) {
        g.rmodal = Some(card);
        g.rmodal_npc = Some(i);
        g.log(format!("{}似有心事欲言，且听分说……", NPC_NAMES[i]), C_GOLD);
        g.play(crate::sounds::Which::Ding);
    }
}

// ---- v1.8.0 D2：好感专属事件表 ----
// 四事件各两选项；Eff 字段由 apply_reff 消费（现实侧），选项副标签走 eff_desc。
pub struct NpcEventDef {
    pub title: &'static str,
    pub text: &'static str,
}
pub const NPC_EVENTS: [NpcEventDef; 4] = [
    // 0 长老「旧伞相赠」：取伞（墨骨伞；已有折灵石）/ 折现（贡献+150、灵石+3000）
    NpcEventDef {
        title: "旧伞相赠",
        text: "玄诚长老唤你入静室，取出一柄旧伞：「此伞随老夫半生，今日赠你——莫坠了它的名声。」",
    },
    // 1 掌柜「万宝楼夜宴」：灵石+8000 / 聚气丹×3
    NpcEventDef {
        title: "万宝楼夜宴",
        text: "胡掌柜在万宝楼设下夜宴，席间推杯换盏：「道友肯赏光，是万宝楼的荣光——这点心意，务必收下！」",
    },
    // 2 宁清霜「剑冢论剑」：修为+25% / 道心+4·寿元+10
    NpcEventDef {
        title: "剑冢论剑",
        text: "宁清霜约你于后山剑冢论剑。一场切磋下来，她收剑而立，眸光微亮：「你的剑，比上次快了。」",
    },
    // 3 醉道人「醉后真言」：气运+5 / 道韵+40·灵石+2000
    NpcEventDef {
        title: "醉后真言",
        text: "醉道人又醉倒在洞府门前，却忽然睁眼一笑：「小子……老夫压箱底的这点东西，就当酒钱了。」",
    },
];

// 由事件表构建现实侧 EventCard（表驱动，无随机）
pub fn npc_event_card(i: usize) -> Option<EventCard> {
    let def = NPC_EVENTS.get(i)?;
    let mut card = EventCard {
        kind: CardKind::Npc,
        title: format!("【{}】{}", NPC_NAMES[i], def.title),
        text: def.text.to_string(),
        opts: Vec::new(),
    };
    match i {
        0 => {
            let mut e1 = Eff::new();
            e1.relic = Some(3); // 墨骨伞（apply_reff：未拥有入包、已有折 15000 灵石）
            e1.msg = vec!["你郑重接过旧伞，伞骨上的旧痕犹温。".to_string()];
            card.opts.push(("受伞".to_string(), e1));
            let mut e2 = Eff::new();
            e2.contrib = 150;
            e2.stones = 3000;
            e2.msg = vec!["你婉谢宝伞，长老大笑转赠灵石，并记你一功。".to_string()];
            card.opts.push(("婉谢折现".to_string(), e2));
        }
        1 => {
            let mut e1 = Eff::new();
            e1.stones = 8000;
            e1.msg = vec!["席散人去，锦盒里是厚厚一沓灵石票。".to_string()];
            card.opts.push(("收下厚礼".to_string(), e1));
            let mut e2 = Eff::new();
            e2.pill = Some((0, 3));
            e2.msg = vec!["你只讨了一匣聚气丹——掌柜直笑你识货。".to_string()];
            card.opts.push(("只讨丹药".to_string(), e2));
        }
        2 => {
            let mut e1 = Eff::new();
            e1.qi_pct = 0.25;
            e1.msg = vec!["一番论剑，你于剑意中窥得自家路——修为大进。".to_string()];
            card.opts.push(("倾力一战".to_string(), e1));
            let mut e2 = Eff::new();
            e2.dao = 4.0;
            e2.life_y = 10.0;
            e2.msg = vec!["你拱手请教，清霜剑语如泉——道心通明，身心俱泰。".to_string()];
            card.opts.push(("拱手请教".to_string(), e2));
        }
        _ => {
            let mut e1 = Eff::new();
            e1.luck = 5.0;
            e1.msg = vec!["醉话三分真——你依言改了行功路线，隐隐气机圆转。".to_string()];
            card.opts.push(("倾耳听真言".to_string(), e1));
            let mut e2 = Eff::new();
            e2.dy = 40;
            e2.stones = 2000;
            e2.msg = vec!["你扶他回屋，他袖中滑出一枚玉简与一袋灵石：「酒钱。」".to_string()];
            card.opts.push(("扶他回屋".to_string(), e2));
        }
    }
    Some(card)
}

// v1.8.0 D2：现实侧好感事件抉择（rmodal 通道）。
// 抉择即消耗：先置位 npc_ev 再结算（强杀进程不丢事件）；sync_save 直落盘。
pub fn choose_roption(g: &mut Game, i: usize) {
    let card = g.rmodal.take();
    let npc = g.rmodal_npc.take();
    let (Some(card), Some(n)) = (card, npc) else {
        return;
    };
    if n >= NPC_COUNT {
        return;
    }
    let Some((_, eff)) = card.opts.into_iter().nth(i) else {
        return;
    };
    g.meta.npc_ev[n] = true;
    apply_reff(g, &eff);
    g.sync_save();
}

// 现实侧效果结算（对照模拟侧 apply_eff——全作用于 g.life / g.meta）。
// 借用纪律：life.as_mut 圈在小作用域，日志在块外。
fn apply_reff(g: &mut Game, e: &Eff) {
    let mut logs: Vec<(String, macroquad::prelude::Color)> = Vec::new();
    {
        let Some(l) = g.life.as_mut() else {
            return;
        };
        if e.qi_pct != 0.0 {
            let v = qi_need(l) * e.qi_pct as f64;
            gain_qi(l, v);
            logs.push((format!("修为+{}", fmt_num(v)), crate::game::C_CYAN));
        }
        if e.stones != 0 {
            l.stones = (l.stones + e.stones).max(0);
        }
        if e.stones_pct.abs() > 0.001 {
            l.stones = (l.stones as f64 * (1.0 + e.stones_pct as f64)).max(0.0) as i64;
        }
        if e.dao != 0.0 {
            l.dao = (l.dao + e.dao).clamp(0.0, 100.0);
        }
        if e.luck != 0.0 {
            l.luck = (l.luck + e.luck).clamp(0.0, 100.0);
        }
        if e.life_y != 0.0 {
            l.lifespan += e.life_y;
        }
        if let Some(r) = e.relic {
            if !l.relics.contains(&r) {
                l.relics.push(r);
                logs.push((
                    format!("得法宝【{}】！", RELICS[r].name),
                    crate::game::C_GOLD,
                ));
            } else {
                l.stones += 15000; // 已有同宝：折 15000 灵石
                logs.push((
                    format!("此宝你已有了，{}折价灵石 15000。", RELICS[r].name),
                    crate::game::C_CYAN,
                ));
            }
        }
        if let Some((p, n)) = e.pill {
            l.pills[p] += n;
        }
    }
    // meta 侧（借用圈外）
    if e.dy != 0 {
        g.meta.dao_yun += e.dy;
    }
    if e.contrib > 0 {
        g.meta.sect_contrib += e.contrib;
    }
    // 事件自述行 + 机制行（日志在借用块外）
    for m in &e.msg {
        logs.push((m.clone(), crate::game::C_GOLD));
    }
    for (t, c) in logs {
        g.log(t, c);
    }
    g.play(crate::sounds::Which::Ding);
}

// 道侣好感（Life.comp_fav；只升不降由各写入点自行 max 保证，此处防越界）
// 现实侧好感写入口（模拟侧赠礼/结缘在 sim 块内直写、遗泽在 take_inherit 结算，
// 故当前无调用点——按方案 4.2 定稿 API 保留）
#[allow(dead_code)]
pub fn add_comp_fav(g: &mut Game, d: i32) {
    if let Some(l) = g.life.as_mut() {
        l.comp_fav = (l.comp_fav + d).clamp(0, NPC_FAV_MAX);
    }
}

// ---- 现实赠礼：灵石换好感（人物页赠礼钮入口） ----
pub fn gift_npc(g: &mut Game, i: usize) {
    if i >= NPC_COUNT {
        return;
    }
    let cost = gift_cost(g.meta.npc_fav[i]);
    let ok = g.life.as_ref().map_or(false, |l| l.stones >= cost);
    if !ok {
        g.toast("灵石不足", C_RED);
        return;
    }
    if let Some(l) = g.life.as_mut() {
        l.stones -= cost;
    }
    add_fav(g, i, NPC_GIFT_FAV);
    g.log(
        format!(
            "你备下一份厚礼赠予{}，交情更进一分。（好感+{}）",
            NPC_NAMES[i], NPC_GIFT_FAV
        ),
        C_GOLD,
    );
    g.play(crate::sounds::Which::Coin);
    g.sync_save();
}
