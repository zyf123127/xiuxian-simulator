// v1.16.0 道途一念：任务指引（现实主界面日志区顶部一行目标语）
//
// 设计边界（规格对账）：
// * 零存档键——指引由游戏状态纯推导，旧档零迁移；会话态仅 guide_last 与
//   guide_chap_last（均不进存档，v1.16.16 预热游标）放在 Game 上，本模块无任何状态。
// * 帧路径零分配——HINTS 全为 &'static str 字面量，guide_index 是纯布尔分支树，
//   禁 format!/clone/String（境界名/胜率等数字细节由既有左栏/按钮副标展示，
//   指引行只指路）。
// * 非任务链状态机：只是一个 14 叶优先级 if 链，自上而下首中即出。
// * 终结性（构造性证明——覆盖块之下不放任何叶，无死分支；v1.16.16 讨牒叶/
//   到域叶插入后重对账）：
//   叶 0 = is_stuck（死局叶，最高优先——三空唯一出路是广告，任何里程碑不得截胡）。
//   叶 1 = 里程碑叶 total_sims==0（v1.16.8 链首插入，开局无行动引导之补——
//   主线第一章「开卷首模拟」，压经济叶与修为候破叶，开局推演优先一切养成）。
//   叶 2 = 终局叶（v1.16.9 插入）：已现实问笔（pid10 谓词，goal_met 单口复用）∧
//   四结局未全（endings_seen≠0b1111）——飞升后的唯一主线使命是讨牒 6/6 落款四
//   结局，压日常修为/经济叶。与叶 1 不互斥（total_sims==0 也可现实破境）但叶 1
//   在前：开局推演优先于终局收束，里程碑语义保持。
//   叶 5 = 讨牒叶（v1.16.16 插入）：当前章=第四章·文狱讨牒（chapter_current 单口
//   复用）∧ taodie<6——第四章进行期唯一主线使命是寻仇取牒凑六牒。谓词只读
//   chapter_current/taodie（与叶 3/4/6..13 的 qi/stones/points/loc/npc 域正交）；
//   非截断型：taodie 至 6 时第四章翻毕、chapter_current 前移，谓词自然落假，无新
//   死域；taodie<6 且 revenge_ready 的态被叶 4 先截，两域不重叠泄漏。
//   叶 6 = 图鉴叶（v1.16.10 插入）：∃i npc_fav[i]≥NPC_EVENT_FAV ∧ ¬npc_ev[i]
//   （同一故人好感已达标而专属事件未抉择）——第六章「故人心事」目标从徽标暗提示
//   升级为横带可达跳转，压修为候破/洞府/经济等日常叶（主线目标优先日常养成）。
//   谓词只读 npc_fav/npc_ev，与叶 8..13 的 qi/stones/points/loc 域正交；且非截断
//   型：抉择事件置 npc_ev[i] 后谓词翻假，链自然落回原域，无新死域。
//   叶 7 = 到域叶（v1.16.16 插入）：当前章=第五章·山河万象 ∧ 五域未全（region
//   计数<5，pid7 谓词补集）——第五章进行期唯一主线使命是走访新域。置于图鉴叶之
//   下：心事待叙优先于山河；谓词只读 chapter_current/region_visited（与叶 8..13
//   域正交）；非截断型：五域走满第五章翻毕、chapter_current 前移，谓词自然落假。
//   过叶 0..7 ⟺ total_sims>0 ∧ ¬is_stuck ∧ ¬(pid10 ∧ ed≠1111) ∧ ¬can_break ∧
//   ¬revenge_ready ∧ ¬(四章未了∧牒<6) ∧ ¬图鉴 ∧ ¬(五章未了∧域<5)——该态集 ⊆
//   「过叶 0/1」原域，自此以下与 v1.16.0 原链逐叶等价（原叶 5..10 → 现叶 8..13，
//   谓词零改动）：Reality 恒 life=Some（draw_reality 早退守卫），过叶 0 ⟺ P≥1 ∨
//   S≥ex ∨ R≠∅；叶 8/9/10 谓词（修为百分比/洞府双资源/auto∧P<1）与该补集正交、
//   各自可构造命中；叶 11∨12∨13 = (R≠∅∧P<1∧S<ex) ∨ P≥1 ∨ S≥ex ⊇ 该补集
//   （R≠∅∧P≥1→12、R≠∅∧S≥ex→13），故任何过叶 0..7 的态必在 11/12/13 终结——
//   链到叶 13 必然终结。里程碑叶/终局叶/讨牒叶/图鉴叶/到域叶只置于覆盖块之上，
//   原链内部不动，原终结证明对剩余子域原样成立。裁叶说明（v1.16.0 沿革）：
//   赠礼叶/默认闭关叶结构性不可达（被 11/12/13 三分覆盖），移入 futureCandidates，
//   赠礼引流由人物页/图鉴『有心事待叙』徽标承载（v1.16.10 起另有图鉴叶指路）。
// * 判定条件全部复用既有单口谓词，禁另写第二实现：is_stuck / can_break /
//   revenge_ready / sim_cost / exchange_cost / DONGFU_UP_STONES / DONGFU_UP_CONTRIB
//   ＋ meta.total_sims（里程碑叶唯一新谓词，纯标量读）＋ meta.npc_fav/npc_ev
//   （图鉴叶新谓词，读 NPC_EVENT_FAV 常量；既定域数据，非新实现）＋
//   Game::chapter_current（v1.16.16 讨牒/到域叶新谓词，章节单口复用；meta.taodie/
//   region_visited 既定域标量读，非新实现）。
use crate::data::{DONGFU_UP_CONTRIB, DONGFU_UP_STONES, NODE_DONGFU, NPC_EVENT_FAV};
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
    Book,  // v1.16.9 章节书：panel=6 + guide_tab=1（横带开面板直落章节书签；
           // 指引面板内点击只切签不关面板——draw 侧归零逻辑对 Book 例外）
    Gallery, // v1.16.10 交游图鉴：panel=5 + char_tab=4（角色面板直落图鉴页签，
             // prewarm_corpus 单口预热，与 exec_panel_act Tab(4) 同口）
}

pub struct GuideHint {
    pub text: &'static str,
    pub btn: Option<&'static str>,
    pub act: GuideAct,
}

/// 14 叶提示表（下标即优先级；全字面量禁插值）。v1.16.8 链首插叶 1 里程碑叶
/// （开卷首模拟——主线第一章目标，压经济/养成叶）；v1.16.9 插叶 2 终局叶（问笔后
/// 落款收束期唯一使命）；v1.16.10 宿敌叶后插图鉴叶（第六章故人心事横带可达）；
/// v1.16.16 插叶 5 讨牒叶（第四章进行期主线指路）与叶 7 到域叶（第五章进行期
/// 主线指路），原叶语义逐叶保留于 3..4 与 6..13。
pub const HINTS: [GuideHint; 14] = [
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
        text: "已登仙界：讨牒 6/6 方见终战卡，落款四结局（书志页灰行指路）",
        btn: Some("去书志"),
        act: GuideAct::Book,
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
        // v1.16.16 讨牒叶：第四章·文狱讨牒进行期（详见 guide_index 叶 5 注）
        text: "主线·文狱讨牒：寻仇斩敌取牒，六牒归档见终战",
        btn: None,
        act: GuideAct::None,
    },
    GuideHint {
        text: "故人心事待叙：好感已达，人物·图鉴页抉择专属事件",
        btn: Some("去图鉴"),
        act: GuideAct::Gallery,
    },
    GuideHint {
        // v1.16.16 到域叶：第五章·山河万象进行期（详见 guide_index 叶 7 注）
        text: "主线·山河万象：走访新域，五域各有机缘",
        btn: Some("去山河"),
        act: GuideAct::World,
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
    // 1 里程碑·开卷首模拟（v1.16.8 链首插入）：total_sims==0 的唯一使命是先推演一生
    //   ——主线第一章目标，压经济叶与修为候破叶（叶 3..10 全部要求 total_sims>0 才达）
    if g.meta.total_sims == 0 {
        return 1;
    }
    // 2 终局·落款收束（v1.16.9 插入）：已现实问笔（pid10 谓词，goal_met 单口复用禁
    //   第二实现）而四结局未全——飞升后的唯一主线使命是讨牒 6/6 见终战卡、落款四结局
    //   （勘察修正：原案谓词 mask==0xFF∧ed≠1111 恒假——bit7 已含 pid13 ed==1111，
    //   两条件互斥；按其文案意图改挂 pid10 ∧ ed≠1111，见结果报告）
    if Game::chapter_goal_met(&g.meta, 10) && g.meta.endings_seen != 0b1111 {
        return 2;
    }
    // 3 修为圆满：修为轴最高优先（养成叶与经济轴正交故可达）
    if g.can_break() {
        return 3;
    }
    // 4 宿敌可寻仇：带 RNG 的不可逆结算，指引只指路不代打（无按钮）
    if g.revenge_ready() {
        return 4;
    }
    // 5 讨牒叶（v1.16.16 插入）：当前章=第四章·文狱讨牒（chapter_current 单口
    //   复用，下标 3）∧ taodie<6——第四章进行期唯一主线使命是寻仇取牒凑六牒
    //   （讨牒正路=revenge，revenge_ready 态已被叶 4 先截）。非截断：牒至 6 章节
    //   翻毕谓词自然落假（见文件头终结性证明）
    if Game::chapter_current(&g.meta) == 3 && g.meta.taodie < 6 {
        return 5;
    }
    // 6 图鉴·故人心事（v1.16.10 插入）：任一故人好感达 NPC_EVENT_FAV 且其专属
    //   事件未抉择——第六章「故人心事」目标的横带可达跳转（原为图鉴卡徽标暗提示）。
    //   只读 npc_fav/npc_ev 既定域（与叶 8..13 域正交，见文件头终结性证明）
    if g
        .meta
        .npc_fav
        .iter()
        .zip(g.meta.npc_ev.iter())
        .any(|(&f, &e)| f >= NPC_EVENT_FAV && !e)
    {
        return 6;
    }
    // 7 到域叶（v1.16.16 插入）：当前章=第五章·山河万象（下标 4）∧ 五域未全
    //   （pid7 谓词补集）——第五章进行期唯一主线使命是走访新域。置于图鉴叶之下：
    //   心事待叙优先于山河（见文件头终结性证明）；非截断：域满章节翻毕谓词落假
    if Game::chapter_current(&g.meta) == 4
        && g.meta.region_visited.iter().filter(|&&x| x).count() < 5
    {
        return 7;
    }
    if let Some(l) = l {
        // 8 修为将满（≥80% 未满）：闭关候破。can_break 已被叶 3 截走，!can_break
        //   恒真，按规格保留以自明叶面语义（顺序无关、代价一次布尔）
        if !g.can_break() && l.qi / qi_need(l) >= 0.8 {
            return 8;
        }
        // 9 洞府设施可扩建：任一设施未满级且灵石/贡献双资源足够（与 dongfu_up 同口：
        //   world.rs:468 clamp(0,3)、未满才可升；lv<3 短路保证 DONGFU_UP_*/[lv] 越界安全）
        if l.loc == NODE_DONGFU
            && (0..3).any(|i| {
                let lv = g.meta.dongfu[i].clamp(0, 3) as usize;
                lv < 3
                    && stones >= DONGFU_UP_STONES[lv]
                    && g.meta.sect_contrib >= DONGFU_UP_CONTRIB[lv]
            })
        {
            return 9;
        }
    }
    // 10 周天吐纳未开：收窄 P<1——有点数可花时不得截胡开模拟
    //   （新玩家首局 Meta::default P=3 必落叶 1 里程碑叶）
    if g.meta.auto == 0 && p < g.sim_cost() {
        return 10;
    }
    // 11 可献祭法宝折算模拟点：点数/灵石两空而法宝尚存
    if l.map_or(false, |x| !x.relics.is_empty()) && p < g.sim_cost() && stones < ex {
        return 11;
    }
    // 12 模拟点已足：开卷推演新一生
    if p >= g.sim_cost() {
        return 12;
    }
    // 13 灵石充裕：坊市可兑模拟点（兜底终结叶）。life=Some 时到达此臂必 S≥ex：
    //   ¬叶12 ⟹ P<1，¬叶11（P<1）⟹ R=∅ ∨ S≥ex；R≠∅ 时叶 11 已截走除非 S≥ex，
    //   R=∅ 时 ¬is_stuck（P<1）⟹ S≥ex——构造性证明见文件头。
    //   life=None 简并态在绘制路径不可达（draw_reality 早退守卫）。
    13
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
        GuideAct::Book => {
            // v1.16.9 章节书：横带（panel=0）开指引面板直落章节书签，面板全套静态
            // 文案事件帧预热照任务钮先例（prewarm_guide_panel 单口，含章节书签全量）；
            // 面板内（panel=6）点击只切签不关面板——draw 侧跳转归零逻辑对 Book 例外
            if g.panel != 6 {
                g.panel = 6;
                crate::draw::prewarm_guide_panel(g);
            }
            g.guide_tab = 1;
            crate::draw::prewarm_chapter_view(g);
            g.play(crate::sounds::Which::Page);
        }
        GuideAct::Gallery => {
            // v1.16.10 交游图鉴：角色面板（panel=5）直落图鉴页签（char_tab=4），
            // 首开帧字形预热走 prewarm_corpus 单口（与 exec_panel_act Tab(4) 同口；
            // 幂等——重复预热只多一次查表，事件帧级无稳态开销）
            if g.panel != 5 || g.char_tab != 4 {
                g.panel = 5;
                g.char_tab = 4;
                crate::gallery::prewarm_corpus();
            }
            // v1.16.10 遭遇手札预热同口补齐（与 exec_panel_act Tab(4) 逐字等价；
            // 幂等，未遇事件不灌）
            crate::gallery::prewarm_event_book(g);
            g.play(crate::sounds::Which::Page);
        }
    }
}
