// v1.6.0 D 好感度 NPC 关系系统：
// Meta.npc_fav[2]（长老/商人，跨世持久）+ Life.comp_fav（道侣，随克隆入模拟、遗泽归现实）。
// 表驱动（data.rs）：档次阈值 [0,20,40,70,95]，各增益按档取值。
use crate::data::*;
use crate::game::{Game, C_GOLD, C_RED};

pub const NPC_ELDER: usize = 0; // 玄诚长老（宗门任务收益）
pub const NPC_MERCHANT: usize = 1; // 胡掌柜（坊市丹价折扣）

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

// 赠礼造价：随档次水涨船高（基础 500 × (1+档位)）
pub fn gift_cost(fav: i32) -> i64 {
    NPC_GIFT_BASE * (1 + tier_of(fav) as i64)
}

// ---- 好感写入 ----
// 长老/商人好感（clamp 上限；调用点在块外，无借用冲突）
pub fn add_fav(g: &mut Game, i: usize, d: i32) {
    if i >= 2 {
        return;
    }
    let v = g.meta.npc_fav[i].clamp(0, NPC_FAV_MAX);
    g.meta.npc_fav[i] = (v + d).clamp(0, NPC_FAV_MAX);
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
    if i >= 2 {
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
