// v1.16.0 道途一念：任务指引（现实主界面日志区顶部一行目标语）
//
// 设计边界（规格对账）：
// * 零存档键——指引由游戏状态纯推导，旧档零迁移；唯一会话态 guide_last（不进存档）
//   放在 Game 上，本模块无任何状态。
// * 帧路径零分配——HINTS 全为 &'static str 字面量，guide_index 是纯布尔分支树，
//   禁 format!/clone/String（境界名/胜率等数字细节由既有左栏/按钮副标展示，
//   指引行只指路）。
// * 非任务链状态机：只是一个 10 叶优先级 if 链，自上而下首中即出。
// * 终结性（构造性证明——覆盖块之下不放任何叶，无死分支；v1.17.2 链首插里程碑叶
//   后重对账）：
//   叶 0 = is_stuck（死局叶，最高优先——三空唯一出路是广告，任何里程碑不得截胡）。
//   叶 1 = 里程碑叶 total_sims==0（v1.17.2 链首插入，开局无行动引导之补——
//   主线第一章「开卷首模拟」，压经济叶与修为候破叶，开局推演优先一切养成）。
//   过叶 0 ∧ 过叶 1 ⟺ total_sims>0 ∧ ¬is_stuck；自此以下与 v1.16.0 原链逐叶
//   等价（原叶 1..8 → 现叶 2..9，谓词零改动）：Reality 恒 life=Some（draw_reality
//   早退守卫），过叶 0 ⟺ P≥1 ∨ S≥ex ∨ R≠∅；叶 4/5/6 谓词（修为百分比/洞府双资源
//   /auto∧P<1）与该补集正交、各自可构造命中；叶 7∨8∨9 = (R≠∅∧P<1∧S<ex) ∨ P≥1
//   ∨ S≥ex ⊇ 该补集（R≠∅∧P≥1→8、R≠∅∧S≥ex→9），故任何过叶 0/1 的态必在
//   7/8/9 终结——链到叶 9 必然终结。里程碑叶只置于链首（覆盖块之上），原链内部
//   不动，原终结证明对 total_sims>0 子域原样成立。裁叶说明（v1.16.0 沿革）：
//   赠礼叶/默认闭关叶结构性不可达（被 7/8/9 三分覆盖），移入 futureCandidates，
//   赠礼引流由人物页/图鉴『有心事待叙』徽标承载。
// * 判定条件全部复用既有单口谓词，禁另写第二实现：is_stuck / can_break /
//   revenge_ready / sim_cost / exchange_cost / DONGFU_UP_STONES / DONGFU_UP_CONTRIB
//   ＋ meta.total_sims（里程碑叶唯一新谓词，纯标量读）。
use crate::data::{DONGFU_UP_CONTRIB, DONGFU_UP_STONES, NODE_DONGFU};
use crate::game::{qi_need, Game};

/// 跳转动作（全部复用既有单入口，不复制任何结算逻辑；无 Gift——赠礼叶被裁，见文件头）
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GuideAct {
    None,  // 纯提示叶（寻仇是不可逆 RNG 结算、周天开关在底栏，只指路不代打）
    Cult,  // 修炼弹层：crate::menu::open_sub(Sub::Cult)（自带开帧门闩与破境按钮）
    Draft, // 开始模拟：play(Click)+start_draft（stuck 态内部自动转 start_ad，与底栏同口）
    Shop,  // 坊市·点数兑换页签：panel=1, shop_tab=2
    Bag,   // 行囊·法宝页：panel=2, bag_tab=1
    World, // 山河图：crate::world::open_world（内部已做底图预烘）
}

pub struct GuideHint {
    pub text: &'static str,
    pub btn: Option<&'static str>,
    pub act: GuideAct,
}

/// 10 叶提示表（下标即优先级；全字面量禁插值）。v1.17.2 链首插叶 1 里程碑叶
/// （开卷首模拟——主线第一章目标，压经济/养成叶），原 9 叶语义逐叶保留于 2..9。
pub const HINTS: [GuideHint; 10] = [
    GuideHint {
        text: "山穷水尽：观看幻境广告，得 2 枚模拟点",
        btn: Some("看广告"),
        act: GuideAct::Draft,
    },
    GuideHint {
        text: "主线第一章：开卷首模拟，推演你的一生",
        btn: Some("开模拟"),
        act: GuideAct::Draft,
    },
    GuideHint {
        text: "修为圆满：可冲击下一层/境",
        btn: Some("去闭关"),
        act: GuideAct::Cult,
    },
    GuideHint {
        text: "宿敌可寻仇：一战定赏（底栏「寻仇」）",
        btn: None,
        act: GuideAct::None,
    },
    GuideHint {
        text: "修为将满：闭关苦修，静候破境",
        btn: Some("去闭关"),
        act: GuideAct::Cult,
    },
    GuideHint {
        text: "洞府设施可扩建（山河图·洞府经营）",
        btn: Some("去山河"),
        act: GuideAct::World,
    },
    GuideHint {
        text: "周天吐纳未开：底栏「周天」一键自动闭关",
        btn: None,
        act: GuideAct::None,
    },
    GuideHint {
        text: "可献祭法宝折算模拟点（行囊·法宝页）",
        btn: Some("去行囊"),
        act: GuideAct::Bag,
    },
    GuideHint {
        text: "模拟点已足：开卷推演新一生",
        btn: Some("开模拟"),
        act: GuideAct::Draft,
    },
    GuideHint {
        text: "灵石充裕：坊市可兑模拟点",
        btn: Some("去坊市"),
        act: GuideAct::Shop,
    },
];

/// 优先级 if 链（自上而下首中即出；P=sim_points、S=stones、R=relics、
/// ex=exchange_cost()）。每帧重算零分配零缓存，事件级才产生可见变化。
pub fn guide_index(g: &Game) -> u8 {
    let l = g.life.as_ref();
    let stones = l.map(|x| x.stones).unwrap_or(0);
    let ex = g.exchange_cost();
    let p = g.meta.sim_points;

    // 0 山穷水尽：点数/灵石/法宝三空的唯一死局叶（跳转进 start_draft→内部转 start_ad）
    if g.is_stuck() {
        return 0;
    }
    // 1 里程碑·开卷首模拟（v1.17.2 链首插入）：total_sims==0 的唯一使命是先推演一生
    //   ——主线第一章目标，压经济叶与修为候破叶（叶 2..9 全部要求 total_sims>0 才达）
    if g.meta.total_sims == 0 {
        return 1;
    }
    // 2 修为圆满：修为轴最高优先（养成叶与经济轴正交故可达）
    if g.can_break() {
        return 2;
    }
    // 3 宿敌可寻仇：带 RNG 的不可逆结算，指引只指路不代打（无按钮）
    if g.revenge_ready() {
        return 3;
    }
    if let Some(l) = l {
        // 4 修为将满（≥80% 未满）：闭关候破。can_break 已被叶 2 截走，!can_break
        //   恒真，按规格保留以自明叶面语义（顺序无关、代价一次布尔）
        if !g.can_break() && l.qi / qi_need(l) >= 0.8 {
            return 4;
        }
        // 5 洞府设施可扩建：任一设施未满级且灵石/贡献双资源足够（与 dongfu_up 同口：
        //   world.rs:468 clamp(0,3)、未满才可升；lv<3 短路保证 DONGFU_UP_*/[lv] 越界安全）
        if l.loc == NODE_DONGFU
            && (0..3).any(|i| {
                let lv = g.meta.dongfu[i].clamp(0, 3) as usize;
                lv < 3
                    && stones >= DONGFU_UP_STONES[lv]
                    && g.meta.sect_contrib >= DONGFU_UP_CONTRIB[lv]
            })
        {
            return 5;
        }
    }
    // 6 周天吐纳未开：收窄 P<1——有点数可花时不得截胡开模拟
    //   （新玩家首局 Meta::default P=3 必落叶 1 里程碑叶）
    if g.meta.auto == 0 && p < g.sim_cost() {
        return 6;
    }
    // 7 可献祭法宝折算模拟点：点数/灵石两空而法宝尚存
    if l.map_or(false, |x| !x.relics.is_empty()) && p < g.sim_cost() && stones < ex {
        return 7;
    }
    // 8 模拟点已足：开卷推演新一生
    if p >= g.sim_cost() {
        return 8;
    }
    // 9 灵石充裕：坊市可兑模拟点（兜底终结叶）。life=Some 时到达此臂必 S≥ex：
    //   ¬叶8 ⟹ P<1，¬叶7（P<1）⟹ R=∅ ∨ S≥ex；R≠∅ 时叶 7 已截走除非 S≥ex，
    //   R=∅ 时 ¬is_stuck（P<1）⟹ S≥ex——构造性证明见文件头。
    //   life=None 简并态在绘制路径不可达（draw_reality 早退守卫）。
    9
}

/// 跳转执行器：横带按钮唯一出口。全部走既有单入口，不复制任何结算逻辑；
/// 与底栏既有入口逐帧等价（坊市/行囊同款 Page 音效，draw.rs:753-754 先例）。
pub fn exec_jump(g: &mut Game, act: GuideAct) {
    match act {
        GuideAct::None => {} // 纯提示叶无按钮，不可达（防御空转）
        GuideAct::Cult => {
            crate::menu::open_sub(g, crate::menu::Sub::Cult);
        }
        GuideAct::Draft => {
            g.play(crate::sounds::Which::Click);
            g.start_draft(); // stuck 态内部自动转 start_ad，与底栏「开始模拟」完全同口
        }
        GuideAct::Shop => {
            g.panel = 1;
            g.shop_tab = 2; // 点数兑换页签（draw.rs 页签表 [丹药阁,藏经阁,点数兑换]）
            g.play(crate::sounds::Which::Page);
        }
        GuideAct::Bag => {
            g.panel = 2;
            g.bag_tab = 1; // 行囊法宝页
            g.play(crate::sounds::Which::Page);
        }
        GuideAct::World => {
            crate::world::open_world(g);
        }
    }
}
