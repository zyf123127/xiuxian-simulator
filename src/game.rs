// 核心逻辑：现实主角 + 模拟器系统双轨

// 现实：修炼/突破/坊市/寻仇；模拟：消耗点数推演一生，死后选继承带回现实

// 借用规范：对 life/sim 的可变借用限定在块内，日志/音效/存档在块外调用

// v1.5.0 守则：新增任何「模拟侧」法宝获得点，必须在同一守卫体内同步
// `s.equipped.push(...)`（否则该法宝在模拟中只按 30% 灵性计，见 RELIC_UNEQUIP_FRAC）；
// 任何克隆/构造模拟 Life 的地方，必须同步 `equipped = relics.clone()` 镜像（先例 launch_sim）。

use crate::data::*;

use crate::save;

use macroquad::prelude::*;

use macroquad::rand::gen_range;

pub const V_W: f32 = 1280.0;

pub const V_H: f32 = 720.0;

// v1.9.25 周天吐纳：现实自动修炼节拍（秒/载）。挂点 main.rs 每帧 tick_auto，
// 到点走与手点完全同口的 act_cultivate(true)（静音逐年日志/音效，防刷屏）。
pub const AUTO_CULT_SECS: f32 = 1.6;

pub const SIM_SPEEDS: [f32; 4] = [0.0, 0.16, 0.06, 0.0]; // 每年间隔秒；0 数值档=瞬间跳到结局

// v1.16.5 模拟事件防重复窗口 K：本世最近 6 次抽过的事件不再抽（事件 13%/年
// 频率下约 46 年不见同卡）；可用池小于 K（低境界）由 sim_event 退化分支清池兜底
pub const RECENT_EVENT_K: usize = 6;

// v1.16.9 老年/将尽专属事件（id 88/89）入池阈值：寿元余量 ≤10 载才入池（模拟侧
// Sim.lifespan - Sim.age）。时序自洽：sim_year 内事件在寿元检查前结算（age+=1 →
// sim_event → 宿敌 roll → 寿元检查 sim_die），「最后一载弹了愿卡、抉择后自然归真」
// 成立；modal 冻结模拟，卡面期无寿元推进。措辞纪律：门控是「当次入池判定」，弹卡
// 后寿元再变化不撤卡（无面镜客 -10/仙人赠礼 +50 可跨档），文案按「大限将至」措辞
// 而非精确年数。置 RECENT_EVENT_K 旁照同批常量惯例
pub const ELDER_LEFT_YEARS: f64 = 10.0;

// v1.16.7 连锁剧情：置链后的「保质期」预算（按事件年计）。直取成功后由下一段的
// 置链臂重置；若链态异常（直取连续失败）耗尽即静默弃链——保险丝，正常一条链至多
// 2 次推进、永不触发
pub const CHAIN_TTL: u8 = 8;

// v1.16.7 道侣同行事件白名单：sim.companion 存在才入池（与 sect/lingmai 门闩
// 同点结算，见 sim_event_weights）；链内段 45/46/48/49/51/52 与 v1.16.8 链内段
// 77/78/80/81 用 EVENTS 表权重 0 表达「仅链内出现」，无需代码特判。
// v1.16.15 道侣深化 +3：156 雪窖藏酿/157 同守昙花/158 雨后补瓦；
// v1.16.16 伴随 +1：182 霜夜补裘（有伴长世生活流第 10 味）
// v1.16.17 伴随 +3：183 灯下习字（mr0 白丁有伴层首条）/184 同参一炉（mr2 双修
// 题材）/185 云海携游（mr4）——此前 10 条 mr 全≥1 且≤3，白丁有伴与 mr4+ 零覆盖
// v1.16.18 扩池伴随 +1：203 云海拾简（mr4 层第 2 条）——白名单 13→14。
// v1.16.19 伴随 +3：210 雨檐听铃（mr3 层，此前伴随 mr3 零位）/211 云上放灯
//（mr5 高境有伴）/212 相扶看雪（老年×有伴双门控第二条，照 204 模板——白名单
// 先行无伴归零，余量≤10 载交集门在 sim_event_weights id==212 单 if 内，两道
// 门同向不冲突）——白名单 14→17。
// v1.16.20 扩池伴随 +2：225 同数流星（mr6 有伴层首条）/226 霜晨共伞（mr0 有伴
// 第 2 条，照 174 霜桥早市成对分域）——白名单 17→19。全挂 Comp 立绘照
// 210/211 反查模式（卡面主角即此生道侣）。
// 第三处硬编码副本 tools/ledger_recheck.py COMPANION 集合同批同步（漏一处
// 账本 mr0 无伴口径对不上）
pub const COMPANION_EVENTS: [usize; 19] = [
    53, 54, 55, 82, 83, 84, 156, 157, 158, 182, 183, 184, 185, 203, 210, 211, 212,
    225, 226,
];

// v1.16.7 置链臂 special 槽位段：20..=31（apply_eff 链裁决据此区分「重写链态」
// 与「本次抉择即链终局」）。v1.16.8 续编：28/29=链4（灯楼谜约）两段推进、
// 30/31=链5（傩面还愿）双分支；检定臂自 32 起（32 青灯三问/33 墨叟一问/
// 34 拦问纸人）——与置链窗口不重叠，落链上即链终局的裁决语义不变
pub const CHAIN_SPECIAL_LO: u8 = 20;
pub const CHAIN_SPECIAL_HI: u8 = 31;

// v1.16.9 链6（血债血偿）置链臂第二段窗口 35..=36：20..=31 已满、32..=34 是检定臂
// 刻意窗外（抬 CHAIN_SPECIAL_HI 会吞检定臂「落臂即清链」裁决），故链6 续编独立窗口
// ——apply_eff 链裁决改多段判定（各窗之外落链即终局，语义不变）
pub const CHAIN6_SPECIAL_LO: u8 = 35;
pub const CHAIN6_SPECIAL_HI: u8 = 36;

// v1.16.10 链7（灯花渡舟）/链8（药炉童子）置链臂窗口 37..=40：35..=36 已占、
// 32..=34 检定臂照旧窗外——apply_eff 链裁决三窗判定（第三窗，语义不变）
pub const CHAIN78_SPECIAL_LO: u8 = 37;
pub const CHAIN78_SPECIAL_HI: u8 = 40;

// v1.16.11 链9（循册寻仇——宿敌终局对决线）/链10（旧页回声——跨世情缘回访）置链臂
// 窗口 41..=44：37..=40 已占、32..=34 检定臂照旧窗外——apply_eff 链裁决四窗判定
//（第四窗，语义不变）。41/42=链9 两推进臂（118→119→120），43/44=链10（121→122→123）
pub const CHAIN910_SPECIAL_LO: u8 = 41;
pub const CHAIN910_SPECIAL_HI: u8 = 44;

// v1.16.13 链11（劫痕渡客——邻山渡劫侧写线，绝不触玩家自身破境裁决）/链12（书肆
// 换届——书肆易主代笔线）置链臂窗口 45..=48：41..=44 已占、32..=34 检定臂照旧窗外
// ——apply_eff 链裁决五窗判定（第五窗，语义不变）。45/46=链11 两推进臂
//（137→138→139），47/48=链12（140→141→142）
pub const CHAIN1112_SPECIAL_LO: u8 = 45;
pub const CHAIN1112_SPECIAL_HI: u8 = 48;

// v1.16.14 链13（死当归主——当铺死当旧物归主线）置链臂窗口 49..=52：45..=48
// 已占、32..=34 检定臂照旧窗外——apply_eff 链裁决六窗判定（第六窗，语义不变）。
// 49/50=链13 两推进臂（151→152→153）
pub const CHAIN13_SPECIAL_LO: u8 = 49;
pub const CHAIN13_SPECIAL_HI: u8 = 52;

// v1.16.16 链14（修伞记——雨夜伞铺凡伞归主线）置链臂窗口 53..=55：49..=52
// 已占、32..=34 检定臂照旧窗外——apply_eff 链裁决七窗判定（第七窗，语义不变）。
// 53/54=链14 两推进臂（175→176→177），55 留窗位备用
pub const CHAIN14_SPECIAL_LO: u8 = 53;
pub const CHAIN14_SPECIAL_HI: u8 = 55;

// v1.16.19 链15（观星问岁——醉道人携酒观星，指星对账）置链臂窗口 56..=59：
// 53..=55 已占、32..=34 检定臂照旧窗外——apply_eff 链裁决八窗判定（第八窗，
// 语义不变）。56/57=链15 两推进臂（205→206→207），58/59 留窗位备用
pub const CHAIN15_SPECIAL_LO: u8 = 56;
pub const CHAIN15_SPECIAL_HI: u8 = 59;

// v1.16.20 链16（剑穗之约——宁清霜剑穗线，文狱四 NPC 唯一无专属事件者补缺）
// 置链臂窗口 60..=62：56..=59 已占、32..=34 检定臂照旧窗外——apply_eff 链裁决
// 九窗判定（第九窗，语义不变）。60/61=链16 两推进臂（219→220→221），62=
// 「一夜结完」窗内不置链臂（eff_desc 副标「一夜结完」、apply_eff 落 _ 臂空转
// ——选它即中段断链收获，链态 (16,0,ttl) 随下次 sim_event 弃链，语义与无
// special 选项一致但卡面有分支副标可辨）
pub const CHAIN16_SPECIAL_LO: u8 = 60;
pub const CHAIN16_SPECIAL_HI: u8 = 62;

// v1.16.21 链17（星槎再访——星槎访客 187 的续线，mr5 零链空洞补位）置链臂窗口
// 63..=65：60..=62 已占、32..=34 检定臂照旧窗外——apply_eff 链裁决十窗判定
//（第十窗，语义不变）。63/64=链17 两推进臂（231→232→233），65 留窗位备用；
// v1.16.21 新检定臂 66 起窗位（旧检定臂 32..=34 语义不动），宿敌卡化臂 67/68
// 亦在窗外（落链上即终局的裁决语义天然成立）
pub const CHAIN17_SPECIAL_LO: u8 = 63;
pub const CHAIN17_SPECIAL_HI: u8 = 65;

// v1.16.16 链首段 id → 链 id 查表（缘了闩用；与 gallery event_badge「链起」
// 名单同域注记，双表各依其口——徽标表带匹配臂语义、本表带 id 映射语义）。
// 新增链须同批登记两处
pub const CHAIN_FIRSTS: [(usize, u8); 17] = [
    (44, 1),
    (47, 2),
    (50, 3),
    (76, 4),
    (79, 5),
    (85, 6),
    (112, 7),
    (115, 8),
    (118, 9),
    (121, 10),
    (137, 11),
    (140, 12),
    (151, 13),
    (175, 14),
    (205, 15),
    (219, 16),
    (231, 17), // v1.16.21 链17 星槎再访（mr5 零链补位；bit16 首用——8693 注释口径）
];

// v1.16.7 连锁剧情链名（链完成大事记「{名}·缘了」用；下标=链 id-1）。
// v1.16.8 扩五：链4 灯楼谜约 / 链5 傩面还愿；v1.16.9 扩六：链6 血债血偿
//（宿敌剧情化复仇线——真斩杀仍走 sim_neme/寻仇两口，链只述「探踪/设伏/夺积」）；
// v1.16.10 扩八：链7 灯花渡舟 / 链8 药炉童子；v1.16.11 扩十：链9 循册寻仇
//（斩宿敌之后其门下循册寻仇——与链6 成因果闭环）/ 链10 旧页回声；
// v1.16.13 扩十二：链11 劫痕渡客（邻山渡劫侧写）/ 链12 书肆换届；
// v1.16.14 扩十三：链13 死当归主（当铺死当旧物归主线）；
// v1.16.16 扩十四：链14 修伞记（雨夜伞铺老匠只修凡伞——与遗宝墨骨伞分域反差）；
// v1.16.19 扩十五：链15 观星问岁（醉道人携酒观星指星对账——mr6 高境夜话线）；
// v1.16.20 扩十六：链16 剑穗之约（宁清霜旧剑穗重结——mr4 演武场慢工线）；
// v1.16.21 扩十七：链17 星槎再访（星槎访客 187 续线——mr5 零链空洞补位）
pub const CHAIN_NAMES: [&str; 17] = [
    "货郎奇缘",
    "古碑遗篇",
    "沉墨遗匣",
    "灯楼谜约",
    "傩面还愿",
    "血债血偿",
    "灯花渡舟",
    "药炉童子",
    "循册寻仇",
    "旧页回声",
    "劫痕渡客",
    "书肆换届",
    "死当归主",
    "修伞记",
    "观星问岁",
    "剑穗之约",
    "星槎再访",
];

// v1.16.19 章成金行副句按章查表（与 CHAIN_NAMES 同域；下标=章-1，与 CHAPTERS 等长
// 对齐由 chapter_tests 等长/预算单测钉死防漂移）——原九连「主线又进一程」逐章
// 同句，改每章一句（≤16 字，呼应各章章名/章间记事意象）。金行 log 仍 format! 
// （check_chapters 事件帧既有豁免）；字形随行内串事件帧 prewarm（check_chapters 
// 内补灌，防日志区冷栅格化整图重传）。
pub const DONE_NOTE: [&str; 9] = [
    "方印落定第一笔。",
    "万象皆稿，起笔不悔。",
    "牒上一名，账上一笔。",
    "六牒归档，文狱侧目。",
    "五域行遍，图志有墨。",
    "有人与你缝作同页。",
    "九问既过，天门为你开。",
    "你的名字，写进了书脊。",
    "书外无稿，落笔即是路。",
];

// v1.16.7 旧稿剧情摘要：v1.16.19 改「首条+尾条」取样成「少年…；暮年…」双段叙事
//（原尾 2 条只有近况无开卷，一生无首尾感；单条世取首条即全条）——剥「N岁，」
// 前缀拼接 ≤32 字（纯函数可测；无前缀条目原样保留——「与你相伴至最后一刻，
// 含笑而逝」不被误剥）。预算 32 字=旧稿行剧情摘要显示位（x620 + 32×14px=448 →
// 终点 ≤1068，v1.16.19 行右头像槽 1076 起净空 8px）与章节书前情提要（wrap 610）
// 双显示位共同收口，见 charpanel.rs draw_hist_page 注。
pub fn story_from_highlights(hs: &[String]) -> Option<String> {
    let mut parts: Vec<&str> = Vec::new();
    let n = hs.len();
    if n > 0 {
        // 首条 + 尾条（n==1 只取一条，不重复拼同一条）
        let idxs: [usize; 2] = [0, n - 1];
        for &k in idxs.iter() {
            // 「N岁，」前缀剥离（前段=纯数字+「岁」）；无前缀条目原样保留——
            // 「与你相伴至最后一刻，含笑而逝」不被误剥
            let h = &hs[k];
            let body = match h.split_once('，') {
                Some((a, r)) => match a.strip_suffix('岁') {
                    Some(digits) if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) => r,
                    _ => h.as_str(),
                },
                _ => h.as_str(),
            };
            if parts.last() != Some(&body) {
                parts.push(body);
            }
        }
    }
    let joined = parts.join("；");
    let s: String = joined.chars().take(32).collect();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

#[derive(PartialEq, Clone, Copy)]

pub enum Scene {
    Menu,

    Reality,

    Draft, // 模拟前：天赋 3 选 1

    Sim, // 模拟推演中

    Inherit, // 模拟结束：选择继承

    Settle, // 现实身故：总结算

    World, // 山河图志：世界地图（v1.4.0）
}

pub struct LogLine {
    pub text: String,

    pub color: Color,
}

#[derive(Clone)]

pub struct Eff {
    pub qi_pct: f32,

    pub stones: i64,

    pub stones_pct: f32,

    pub dao: f32,

    pub luck: f32,

    pub life_y: f64,

    pub fight: f32,

    pub relic: Option<usize>,

    pub pill: Option<(usize, i32)>,

    pub dy: i64,

    pub contrib: u32, // v1.8.0 D2：宗门贡献（仅现实侧 apply_reff 消费；模拟侧 apply_eff 忽略）

    pub injure: bool,

    pub sect: bool,

    pub lingmai: bool,

    pub comp_fav: i32, // v1.16.17 伴随事件好感通道（模拟侧 apply_eff 消费；comp_fav
                       // 只升不降既定语义——负值在消费口忽略，clamp 100 照道侣赠礼 2642 同款）

    pub death: f32,

    pub special: u8, // 1洞府探索 2天材地宝 3秘境深入 7心魔检定

    pub msg: Vec<String>,

    pub good: bool,
}

impl Eff {
    pub fn new() -> Self {
        Eff {
            qi_pct: 0.0,

            stones: 0,

            stones_pct: 0.0,

            dao: 0.0,

            luck: 0.0,

            life_y: 0.0,

            fight: 0.0,

            relic: None,

            pill: None,

            dy: 0,

            contrib: 0,

            injure: false,

            sect: false,

            lingmai: false,

            comp_fav: 0,

            death: 0.0,

            special: 0,

            msg: Vec::new(),

            good: true,
        }
    }

    // v1.6.0 B：抉择按钮副标签——机制字段的一行式摘要（弹层纵排选项用）
    pub fn eff_desc(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if self.qi_pct != 0.0 {
            parts.push(format!("修为+{:.0}%", self.qi_pct * 100.0));
        }
        if self.stones != 0 {
            parts.push(format!(
                "灵石{}{}",
                if self.stones > 0 { "+" } else { "-" },
                fmt_int(self.stones.abs())
            ));
        }
        if self.stones_pct.abs() > 0.001 {
            parts.push(format!("灵石{:+.0}%", self.stones_pct * 100.0));
        }
        if self.dao != 0.0 {
            parts.push(format!("道心+{:.0}", self.dao));
        }
        if self.luck != 0.0 {
            parts.push(format!("气运+{:.0}", self.luck));
        }
        if self.life_y != 0.0 {
            parts.push(format!("寿元+{}载", fmt_num(self.life_y)));
        }
        if self.fight > 0.0 {
            parts.push(format!("交战 ×{:.2}", self.fight));
        }
        if let Some(r) = self.relic {
            parts.push(format!("法宝·{}", RELICS[r].name));
        }
        if let Some((p, n)) = self.pill {
            parts.push(format!("{}×{}", PILLS[p].name, n));
        }
        if self.dy != 0 {
            parts.push(format!("道韵+{}", fmt_int(self.dy)));
        }
        if self.contrib > 0 {
            parts.push(format!("贡献+{}", fmt_int(self.contrib as i64)));
        }
        if self.injure {
            parts.push("可能重伤".to_string());
        }
        if self.sect {
            parts.push("拜入宗门".to_string());
        }
        if self.lingmai {
            parts.push("灵脉洞府".to_string());
        }
        if self.death > 0.0 {
            parts.push(format!("身死率{:.0}%", self.death * 100.0));
        }
        if self.special != 0 {
            let s = match self.special {
                1 => "探索上古洞府",
                2 => "夺取天材地宝",
                3 => "深入秘境",
                7 => "心魔检定",
                8 => "武道传承",
                9 => "拔剑试炼",
                10 => "魔窟探宝",
                11 => "护送驰援",
                13 => "立即问笔",
                14 => "闭关备劫",
                15 => "立即破境",
                16 => "蛰伏蓄势",
                17 => "结为道侣",
                18 => "登擂夺魁",
                // v1.16.7 置链臂与新检定臂副标（选项按钮下一行式摘要）
                4 => "受碑文三问",
                5 => "得剑同舟",
                6 => "陨铁试炼",
                12 => "参悟真迹",
                20 => "续·恩怨上门",
                21 => "续·罗盘相赠",
                22 => "续·拓印风波",
                23 => "续·守碑人",
                24 => "续·渔村释疑",
                25 => "续·匣底真迹",
                26 => "镜中赌命",
                27 => "闯灵潮漩心",
                // v1.16.8 置链臂续编（链4/链5）与检定臂（32..=34）副标
                28 => "续·灯下问答",
                29 => "续·青灯问藏",
                30 => "续·代跳傩戏",
                31 => "续·贴班还愿",
                32 => "受青灯三问",
                33 => "受墨叟一问",
                34 => "拦问纸人",
                // v1.16.9 链6（血债血偿）第二段窗口副标
                35 => "续·仇踪暗访",
                36 => "续·乘虚夺积",
                // v1.16.16 链7..链13 置链臂副标（special 37..=50）：照 20..=36
                // 『续·段名』纪律——段名与 EVENT_NAMES 各链中末段「·续」名同源
                //（37/38=链7 灯花渡舟，39/40=链8 药炉童子，41/42=链9 循册寻仇，
                // 43/44=链10 旧页回声，45/46=链11 劫痕渡客，47/48=链12 书肆换届，
                // 49/50=链13 死当归主）。51/52 为链13 预留窗位（CHAIN13_SPECIAL_HI
                // 上沿），尚无事件占用，落 _ 臂「机缘」兜底
                37 => "续·灯下旧账",
                38 => "续·一渡一灯",
                39 => "续·炉语问心",
                40 => "续·守炉一夜",
                41 => "续·雪夜拦道",
                42 => "续·断名之局",
                43 => "续·灯市换页",
                44 => "续·换页重逢",
                45 => "续·劫灰拾遗",
                46 => "续·劫后茶话",
                47 => "续·代笔提要",
                48 => "续·肆后书案",
                49 => "续·访主赎当",
                50 => "续·物归旧主",
                // v1.16.16 链14（修伞记）置链臂副标：第七窗（53/54 推进臂，
                // 55 为预留窗位落 _ 臂兜底）
                53 => "续·伞骨重削",
                54 => "续·还伞",
                // v1.16.19 链15（观星问岁）置链臂副标：第八窗（56/57 推进臂，
                // 58/59 为预留窗位落 _ 臂兜底）——段名与 EVENT_NAMES 链15
                // 中末段「·续」名同源
                56 => "续·指间星图",
                57 => "续·问岁之约",
                // v1.16.20 链16（剑穗之约）置链臂副标：第九窗（60/61 推进臂——
                // 段名与 EVENT_NAMES 链16 中末段「·续」名同源；62=「一夜结完」
                // 窗内断链收获臂，照 26「镜中赌命」非『续』前缀纪律）
                60 => "续·剑穗缓结",
                61 => "续·剑成赠穗",
                62 => "一夜结完",
                // v1.16.21 链17（星槎再访）置链臂副标：第十窗（63/64 推进臂——
                // 段名与 EVENT_NAMES 链17 中末段「·续」名同源；65 预留窗位落 _
                // 臂「机缘」兜底）。新检定臂 66 照 32..=34 检定纪律、宿敌遭遇卡
                // 化臂 67/68 照 26/62 非『续』前缀纪律（复审修：漏登曾落「机缘」
                // 兜底，迎战/遁走语义错位）
                63 => "续·星轨留书",
                64 => "续·槎去留星",
                66 => "受旧砚一问",
                67 => "败则殒命",
                68 => "重伤脱身",
                _ => "机缘",
            };
            parts.push(s.to_string());
        }
        if parts.is_empty() {
            "此路平稳无事".to_string()
        } else {
            parts.join(" · ")
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum CardKind {
    Npc,    // v1.8.0 D2 现实好感事件：choose_roption 抉择（rmodal_npc 配对）
    Path,   // v1.10.2 成行择道：choose_path 抉择（三途分流；副标由 draw_modal_card 特判）
    Finale, // v1.12.1 终卷四结局：choose_ending 抉择（问笔终战独占卡；新变体是硬前置——
            // 现有三路分派 else 落 choose_roption，rmodal_npc=None 时静默挂死）
}

// v1.16.3 邂逅卡立绘：卡面主角（模拟侧「同页天成」/现实侧好感事件）——
// draw_modal_card 据此取 AI 立绘入卡（真机反馈「模拟中结识人物看不到画像，
// 立绘作用没体现」）。纹理不持有，绘制时经 art 懒取；纯会话态随卡生灭不进存档
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CardActor {
    Comp(usize), // 同页人道侣（comp_0..9）
    Npc(usize),  // 文狱四友（npc_elder/merchant/senior/drunk）
}

pub struct EventCard {
    pub title: String,

    pub text: String,

    pub opts: Vec<(String, Eff)>,

    pub kind: CardKind,

    pub actor: Option<CardActor>, // v1.16.3：卡面主角立绘（None=无人物卡，原版式）

    // v1.16.7 剧情演出·稀有/链段标记（纯会话态随卡生灭，不进存档）：
    // rare=EVENTS 权重 w∈1..=RARE_W（描边紫）；stage=链段进度 (当前段,总段)
    //（描边青+标题右侧阶段徽标，链首段 50 两者兼有时链段优先）
    pub rare: bool,

    pub stage: Option<(u8, u8)>,

    // v1.16.21 事件卡类目徽标（纯会话态随卡生灭，不进存档）：复用图鉴
    // event_badge 九类单口（伴随/章节/回响/老年/宿敌/链起/链段/稀有/遭遇），
    // build_event 尾统一置位；其余构造口（道途/终战/破境/现实好感卡）None——
    // 各有专属视觉记号不与类目 chip 叠戴
    pub cat: Option<(&'static str, Color)>,

    // v1.16.21 场景要素行（纯会话态随卡生灭）：构建帧采样 sim 所在地·天时
    // 「青云宗 · 雨」，绘于标题分隔线右端 C_DIM——零版式位移；事件帧 format!
    // 豁免（与 title/text 同路径）
    pub scene: Option<String>,

    // v1.16.21 破境抉择卡记号：draw 侧叠琥珀描边（不占稀有紫/链段青语义位）+
    // 首钮 accent（「立即破境/立即问笔」推荐位）。天劫(13/14)与破境(15/16)
    // 两路同卡构建点统一置位
    pub brk: bool,
}

#[derive(Clone)]

pub struct Trib {
    pub wave: u8,

    pub timer: f32,

    pub flash: f32,

    pub spare_used: bool,

    pub bonus: f32, // 准备 + 丹药 + 天赋汇总加成
}

#[derive(Clone)]

pub struct Life {
    pub n: u32,

    pub age: f64,

    pub lifespan: f64,

    pub realm: usize,

    pub layer: usize,

    pub qi: f64,

    pub root: f32,

    pub dao: f32,

    pub luck: f32,

    pub body: f32,

    pub stones: i64,

    pub tech: usize,

    pub relics: Vec<usize>,

    pub pills: [i32; 6],

    pub sect: bool,

    pub lingmai: bool,

    pub injured: u16,

    pub talents: Vec<usize>,

    pub washed: u32,

    pub max_realm: usize,

    pub fights: u32,

    pub wins: u32,

    pub seclusion: Option<u16>,

    pub trib: Option<Trib>,

    pub new_high: bool,

    pub natural_end: bool,

    pub highlights: Vec<String>, // 大事记

    pub gained: f64, // 模拟中累计修为（继承依据）

    pub power_buff: f64, // 事件战力永久增益（+0.15 = +15%）

    pub companion: Option<String>, // 道侣（模拟结缘；v1.6.0 遗泽可带回现实）

    pub comp_fav: i32, // v1.6.0 D：道侣好感 0..=100（结缘起步 25，只升不降）

    pub pending_break: Option<(usize, f32)>, // 模拟抉择：(目标境界, 蓄势加成)

    pub loc: usize, // 所在地（LOCATIONS 索引，山河图志）

    pub equipped: Vec<usize>, // 已装备法宝（RELICS 索引，⊆ relics，至多 RELIC_EQUIP_MAX 件；v1.5.0）

    pub relic_lv: [i32; 8], // v1.8.0 E：法宝灵纹等级 0..=REFINE_MAX（按 RELICS 下标；随世持久）

    pub slot: u8,    // 时段 0..=3（晨/昼/昏/夜，随推岁流转；v1.5.0）

    pub weather: u8, // 天气 0..=4（WEATHERS 索引，随推岁演化；v1.5.0）

    pub path: u8, // v1.10.2 成行择道：道途 0未择/1剑修/2气修/3丹修（存档键 Lpath=，旧档缺键 0；选定终身不改，模拟侧恒 0）

    pub sx_year: u16, // v1.16.17 双修共进：上次双修的现实年岁（存档键 Lsxy=，旧档缺键 0；
                      // 起始年岁 16 起、恒 >0，0 即「从未双修」零迁移）。每载一次的
                      // 冷却闩：sx_year == dual_year(age) 时本载已修，推岁后方可再修
}

impl Default for Life {
    fn default() -> Self {
        Life {
            n: 1,

            age: 16.0,

            lifespan: 75.0,

            realm: 0,

            layer: 0,

            qi: 0.0,

            root: 30.0,

            dao: 50.0,

            luck: 40.0,

            body: 20.0,

            stones: 100,

            tech: 0,

            relics: Vec::new(),

            pills: [0; 6],

            sect: false,

            lingmai: false,

            injured: 0,

            talents: Vec::new(),

            washed: 0,

            max_realm: 0,

            fights: 0,

            wins: 0,

            seclusion: None,

            trib: None,

            new_high: false,

            natural_end: false,

            highlights: Vec::new(),

            gained: 0.0,

            power_buff: 0.0,

            companion: None,

            comp_fav: 0,

            pending_break: None,

            loc: NODE_START,

            equipped: Vec::new(),

            relic_lv: [0; 8],

            slot: 0,

            weather: 0,

            path: 0,

            sx_year: 0,
        }
    }
}

// v1.10.2 成行择道：道途常量与名号（三途分流，分别挂钩战力/周天吐纳/坊市丹药与灵田）
pub const PATH_NONE: u8 = 0;
pub const PATH_SWORD: u8 = 1; // 剑修：战力 ×1.35
pub const PATH_QI: u8 = 2;    // 气修：每载修为 ×1.25、破境率 +0.06
pub const PATH_PILL: u8 = 3;  // 丹修：服丹效果 ×1.35、灵田岁入 ×1.6

pub fn path_name(p: u8) -> &'static str {
    match p {
        PATH_SWORD => "剑修",
        PATH_QI => "气修",
        PATH_PILL => "丹修",
        _ => "未择道途",
    }
}

// v1.16.17 双修共进门控态（dual_cult_gate 单源；按钮置灰文案与执行口共以此判定）
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DualGate {
    Ready,       // 可双修
    NoCompanion, // 此生未结缘
    ThisYear,    // 本载已双修（冷却闩 sx_year）
    Stones,      // 灵石不足
}

#[derive(Clone)]

pub struct Nemes {
    pub title: &'static str,

    pub name: String,

    pub realm: usize,
}

impl Nemes {
    pub fn power(&self) -> f64 {
        REALMS[self.realm.min(14)].power * 1.25
    }

    pub fn full(&self) -> String {
        format!(
            "{}·{}（{}{}）",
            self.title,
            self.name,
            REALMS[self.realm.min(14)].name,
            REALMS[self.realm.min(14)].layers[1.min(REALMS[self.realm.min(14)].layers.len() - 1)]
        )
    }
}

pub struct Meta {
    pub lives: u32,

    pub dao_yun: i64,

    pub sim_points: i32,

    pub upg: [i32; 10],

    pub ach: Vec<bool>,

    pub first_realm: Vec<bool>,

    pub best_realm: usize,

    pub best_age: f64,

    pub best_power: f64,

    pub total_wins: u32,

    pub neme_slain: u32,

    pub total_sims: u32,

    pub sims_since_bless: u32,

    pub companion_lives: u32,

    pub neme: Option<Nemes>,

    pub note_line: Option<String>, // v1.12.0 留行批注：结算屏大事记点走的一行（note= 键；开卷消费即清）

    pub music: bool,

    pub sfx: bool,

    pub life: Option<Life>,

    pub sect_contrib: u32, // 宗门贡献（跨世持久）

    pub worn_title: i32, // 佩戴称号：-1 自动跟随 / 0..=14 境界称号 / 15 弑神者 /
                         // 16 雨夜执印者（章节称号，v1.16.10）/ 17 执笔人（四结局
                         // 全收集，v1.16.15）——值域权威=set_worn_title 单口与
                         // save.rs 读档钳制回环

    pub dongfu: [i32; 3], // 洞府设施等级 0..=3：0聚灵阵 1灵田 2丹房（v1.5.0）

    pub map_seed: u32,   // v1.6.0 山河万象：地图伪随机种子（LCG 演化）
    pub map_region: u32, // v1.6.0 山河万象：当前地区索引（MAP_THEMES 下标，wrapping 递增）

    pub ad_watched: i32, // v1.7.0：累计观看幻境广告次数（也用于广告创意轮换）

    pub npc_fav: [i32; 4], // v1.6.0 D：NPC 好感（v1.8.0 扩四：0长老 1商人 2宁清霜 3醉道人；跨世持久，0..=100）

    pub npc_ev: [bool; 4], // v1.8.0 D2：NPC 好感专属事件已抉择标记（抉择即消耗，跨世持久）

    pub region_visited: [bool; 5], // v1.8.0 C5：五域到访记录（按 map_region%5 置位，跨世持久）

    pub auto: u8, // v1.9.25 周天吐纳：现实自动修炼开关 0关/1开（存档键 zt=，旧档缺键默认 0）

    pub last_ts: u64, // v1.10.2 离线闭关：最后一次落盘 epoch 秒（存档键 ts=；旧档缺键/0 → 首落盘不计收益仅落新基准）

    pub endings_seen: u8, // v1.12.1 终卷四结局达成位掩码：bit0=A定稿 bit1=B留白(推荐) bit2=C执笔 bit3=D合订(隐藏)
                          //（存档键 ed=；旧档缺键/坏值 → 0，零迁移向后兼容；D 钮显隐与书志页灰行均读此位）

    pub taodie: u8, // v1.12.1 讨牒：已请职牒 0..=6（键 dp=；sync_taodie 对账补发，旧档首检补齐）
    pub feng_bn: u8, // v1.12.1 缝页人名下标（键 bn=；FENG_UNSET=255 未定）
    pub feng_blv: u8, // v1.12.1 缝页缘 0..=3，一世同页至终 +1（键 blv=；缘一/二数值 perk，缘三纯叙事）

    pub comp_bonds: [u8; 10], // v1.16.0 交游图鉴：十同页人历世结缘次数（键 gbd=；0=未识，saturating 封顶 255；下标即 COMPANION_NAMES 下标）

    pub chap_done: u32, // v1.16.8 章节书：主线章节达成位掩码（键 ch=；bit i = CHAPTERS[i] 已了，
                        // v1.16.10 起九章用 bit0..=8；旧档缺键/坏值 → 0，载入帧按事实静默补齐
                        // 零迁移——老玩家补齐不重播横幅）

    pub sys_seen: u8, // v1.16.8 系统初见闩（键 sy=；bit0=周天首开 bit1=点数兑换首通
                      // bit2=首入现实 bit3=首次传承带回 bit4=首修为圆满（v1.16.10）
                      // bit5=首见洞府经营（v1.16.11——山河图经营线唯一无指向实缺的
                      // 初见指路，挂 open_world/travel 事件口）；
                      // 一次性戏文的边沿闩，推导不出故 additive 持久；缺键 → 0 零迁移）

    pub hist: Vec<SimRecord>, // v1.16.5 旧稿档案：近 5 次模拟结算快照（键 h= 每条一行；旧档缺键=空表零迁移，超量解析端截断）

    pub ev_cnt: Vec<u8>, // v1.16.10 事件回顾图鉴：EVENTS 全 id 遭遇计数（键 evc=；下标即
                         // EVENTS 下标，saturating 封顶 255；>0 即已收录——图鉴不另设
                         // seen 位掩码，单一数据源。旧档缺键 → vec![0; EVENTS.len()]
                         // 零迁移，照 ach= 键按表重定形先例 save.rs）
}

impl Default for Meta {
    fn default() -> Self {
        Meta {
            lives: 0,

            dao_yun: 0,

            sim_points: 3,

            upg: [0; 10],

            ach: vec![false; ACHS.len()],

            first_realm: vec![false; 15],

            best_realm: 0,

            best_age: 0.0,

            best_power: 0.0,

            total_wins: 0,

            neme_slain: 0,

            total_sims: 0,

            sims_since_bless: 0,

            companion_lives: 0,

            neme: None,

            note_line: None,

            music: true,

            sfx: true,

            life: None,

            sect_contrib: 0,

            worn_title: -1,

            dongfu: [0; 3],

            map_seed: MAP_SEED_DEFAULT,

            map_region: 0,

            ad_watched: 0,

            npc_fav: [0; 4],

            npc_ev: [false; 4],

            region_visited: [false; 5],

            auto: 0,

            last_ts: 0,

            endings_seen: 0,

            taodie: 0,
            feng_bn: crate::data::FENG_UNSET,
            feng_blv: 0,

            comp_bonds: [0; 10],

            chap_done: 0,
            sys_seen: 0,

            hist: Vec::new(),

            ev_cnt: vec![0; EVENTS.len()],
        }
    }
}

pub struct Mote {
    pub x: f32,

    pub y: f32,

    pub spd: f32,

    pub size: f32,

    pub gold: bool,

    pub ph: f32,
}

pub struct Meteor {
    pub x: f32,

    pub y: f32,

    pub vx: f32,

    pub vy: f32,

    pub life: f32,
}

pub struct Spark {
    pub x: f32,

    pub y: f32,

    pub vx: f32,

    pub vy: f32,

    pub life: f32,

    pub max: f32,

    pub gold: bool,
}

#[derive(Clone)]

pub struct SimResult {
    pub cause: String,

    pub total_dy: i64,

    pub max_realm: usize,

    pub age: f64,

    pub years: i64,

    pub qi_gain: f64,

    pub tech: usize,

    pub relic: Option<usize>,

    pub stones: i64,

    pub pills: [i32; 6],

    pub highlights: Vec<String>,

    pub comp: Option<String>, // v1.6.0 D：道侣（遗泽 idx 8 承接现实）
    pub comp_fav: i32,        // v1.6.0 D：模拟一世落幕时的道侣好感
    pub story: Option<String>, // v1.16.8 本稿剧情摘要（草稿退回页一行；会话态不进存档——
                               // 持久痕迹在 hist 同源的 SimRecord.story hq= 行，此处与
                               // sim_die 同时赋值零二次拼接）
}

// v1.16.5 旧稿档案：单次模拟结算快照（近 HIST_MAX 条滚动；meta additive 持久，
// 旧档缺键 = 空表零迁移）。comp 存 COMPANION_NAMES 下标（255=无伴，照 bn= 惯例）。
#[derive(Clone)]
pub struct SimRecord {
    pub life_no: u32,     // 第几世（sim_die 时 meta.lives）
    pub run_no: u32,      // 第几稿（sim_die 时 meta.total_sims；一世可开多稿）
    pub max_realm: usize, // 最高境界（渲染处 .min(14)，照 draw.rs:1782 先例）
    pub age: f64,         // 享年（sr.age）
    pub years: i64,       // 历经（sr.age - sim_start_age）
    pub dy: i64,          // 本稿道韵（含结缘 +50 加成后的当稿值）
    pub stones: i64,      // 终局灵石
    pub comp: u8,         // 道侣下标（255=无）
    pub cause: String,    // 结局/死因全文（代码字面量，无换行）
    // v1.16.7 本世剧情摘要（链完成/稀有遭遇等关键剧情拼接 ≤28 字；save 平行行
    // hq= 挂载，旧档缺键 → None 渲染跳过零位移，additive-only 零迁移）
    pub story: Option<String>,
}

// 旧稿档案保留条数（写满弹最旧；save.rs 解析端同此截断）
pub const HIST_MAX: usize = 5;

pub fn gain_qi(l: &mut Life, amount: f64) {
    l.qi += amount;

    l.gained += amount;
}

// v1.16.17 缘谱/链10 共用：历世结缘次数最多的同页人下标（旧页回声「反查口径」抽出
// 单源——原链10 123 臂内联块逐式平移；std max_by_key 并列取**末**（实测本工具链
// 返回并列最大者的最后一个），与链10 既有行为逐位一致，勿改取首；空表归 0）。
// 调用方自持 min(len-1) 越界钳制（panic=abort 纪律，链10 臂与道侣专页各自钳）
pub fn best_bond_idx(bonds: &[u8]) -> usize {
    bonds
        .iter()
        .enumerate()
        .max_by_key(|&(_, &b)| b)
        .map(|(i, _)| i)
        .unwrap_or(0)
}

// v1.10.2 离线闭关：出关结算卡数据（会话态不进存档；收益在 resume_life 时已入账内存）
// v1.16.12 免烧寿元：删 life_capped——离线不再推 age，钳制语义随之消亡
pub struct OfflineReport {
    pub years: i64,  // 离闭载数
    pub qi: f64,     // 修为收益（五折后）
    pub stones: i64, // 灵田岁入（静态口径）
}

// ================= v1.10.2 离线闭关：出关结算 =================
// 纯函数（可测）：离线收益三件套。口径：90 秒 = 1 载；raw = min(离线秒, 8h)/90；
// 效率 5 折；N = raw×0.5——v1.16.12 免烧寿元：不再以 lifespan − age − 1 钳制（旧钳制
// 副作用是把 age 精确钉到 lifespan−1 风烛位，承接前缘必寿元将近），收益全额发放、
// age 完全不动；8h 封顶保留为放置反刷上限。破境节奏只由在线周天吐纳 1.6s/载驱动。
// 修为 = 每载收益×N（cult_gain_per_year 同口）；灵石 = 灵田岁入×N（静态口径，不 roll 天气）。
// 返回 None = 不结算（首落盘 ts 缺 0 / 时钟回拨 / N≤0）。
pub const OFFLINE_SEC_PER_YEAR: f64 = 90.0;
pub const OFFLINE_CAP_SECS: f64 = 8.0 * 3600.0;
pub const OFFLINE_EFF: f64 = 0.5;

pub fn offline_settle_calc(
    last_ts: u64,
    now: u64,
    gain_per_year: f64,
    lingtian_per_year: i64,
) -> Option<(i64, f64, i64)> {
    if last_ts == 0 {
        return None; // 首次落盘前无基准
    }
    let sec = (now as f64 - last_ts as f64).max(0.0); // 时钟回拨/脏值 → 按 0 收益
    let raw = sec.min(OFFLINE_CAP_SECS) / OFFLINE_SEC_PER_YEAR;
    let n = (raw * OFFLINE_EFF) as i64;
    if n <= 0 {
        return None;
    }
    Some((
        n,
        gain_per_year * n as f64,
        (lingtian_per_year as f64 * n as f64).round() as i64,
    ))
}

// epoch 秒（native=SystemTime；wasm=Date.now/1e3——miniquad date::now 双端已为 epoch 秒）
pub fn date_now() -> u64 {
    macroquad::miniquad::date::now() as u64
}

pub struct SettleInfo {
    pub cause: String,

    pub lines: Vec<(String, i64)>,

    pub total: i64,

    // v1.16.19 结算页一句收束语（按身故境界档静态查表 &'static str，帧路径零分配
    // ——settle 帧 format! 豁免仅既有行，本句必须全静态，红线口径照横带/章签）。
    // 纯会话态不进存档（SettleInfo 整体同属性）；字形由 real_die 事件帧预热块灌。
    pub coda: &'static str,

    // v1.16.20 立绘沉浸续作②：结算屏谢幕像快照（此生道侣下标，None=无伴零绘制
    // 零位移）。纯会话态不进存档（SettleInfo 整体同属性，零迁移）；real_die
    // 事件帧定格 life.companion 终值——落幕后 life 已清，显示判定与 companion
    // 账同源（快照即账面终值，不读纹理存在性）。
    pub comp: Option<usize>,
}

// v1.16.19 结算收束语三档（≤20 字 &str：低境 0..=4 白丁..立骨 / 中境 5..=9 生花..
// 问笔 / 高境 10..=14 墨仙..天笔）——退稿结算页死因行下一句静收（纯显示零存档）。
pub fn settle_coda(max_realm: usize) -> &'static str {
    match max_realm.min(14) {
        0..=4 => "白纸尚厚，慢慢写。",
        5..=9 => "这一稿有筋骨，下一稿更稳。",
        _ => "一砚之尊，也要退稿。",
    }
}

// v1.16.1 立绘鉴赏：覆层状态（人物页/图鉴点立绘放大）。纯会话态不进存档；
// 纹理不持有——覆层绘制时经 art 懒取（LazyTex 缓存命中零成本），入口已判 AI 图存在。
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ZoomArt {
    Npc(usize),  // 文狱四友（0..=3）
    Comp(usize), // 同页人道侣（0..=9）
}

pub struct Game {
    pub scene: Scene,

    pub meta: Meta,

    pub life: Option<Life>, // 现实主角

    pub sim: Option<Life>, // 模拟中的"你"

    pub sim_start_age: f64,

    pub sim_log: Vec<LogLine>,

    pub sim_result: Option<SimResult>,

    pub inherit_opts: Vec<usize>, // 继承选项（INHERITS 索引）

    pub inherit_picked: u32, // 已选造化数

    pub draft: Vec<usize>, // 模拟天赋候选

    pub blessed: bool,

    pub picked: Vec<usize>,

    pub log: Vec<LogLine>,

    pub modal: Option<EventCard>,

    pub panel: u8, // 0无 1坊市 2行囊 3轮回烙印 4帮助 5角色 6书志（v1.12.1 终卷四结局重看）

    pub char_tab: u8, // 角色面板页签 0总览 1法宝 2成就·称号（v1.5.0）

    // v1.10.2 人物页单列滚动列表会话态（不进存档，×/Esc 归零；先例 char_page）：
    // npc_scroll 滚动偏移（钳 [0, 1198]）；npc_drag 拖动锚点 (按下x, 按下y, 按下时scroll)；
    // npc_vel 松手惯性 px/帧；npc_anim 页点/头像条跳转目标（lerp 0.2/帧平滑趋近）
    pub npc_scroll: f32,
    pub npc_drag: Option<(f32, f32, f32)>,
    pub npc_vel: f32,
    pub npc_anim: Option<f32>,

    // v1.14.0 立绘焕新时刻：跨相破境金环闪光剩余秒数（1.5→0 收敛；会话态不进存档，
    // 旧档载入天然为 0 不误触——只有本次会话内真实跨相破境才置值）
    pub tier_flash: f32,

    // v1.10.2 落盘去抖会话态（不进存档；卡顿修复·次要项②）：sync_save 只置脏，
    // 统一由主循环在「场景/弹窗/面板开合边界」或「距上次落盘 ≥3s」时落盘——
    // 周天吐纳挂机期 localStorage 同步写从 0.6 次/s 降到 0.2 次/s（微顿消除）。
    // wasm 另存最近快照供 pagehide/visibilitychange 强制落盘（丢档窗口 ≤3s）。
    pub save_dirty: bool,
    pub save_timer: f32,
    /// 上一帧 UI 边界信号：(scene, panel, rmodal, modal, submenu, shop, dev, exp)
    pub prev_sig: (u8, u8, bool, bool, bool, bool, bool, bool),

    // v1.9.25 周天吐纳会话态（不进存档）：计时/连续载数/累计修为/跨境满修为 toast 门闩
    pub auto_acc: f32,
    pub auto_years: u32,
    pub auto_gain: f64,
    pub auto_toast_done: bool,

    pub tab: u8,

    pub help_tab: u8, // v1.8.0 B2：帮助面板页签（0玩法 1操作·技巧；会话态不进存档）

    pub buy_pending: Option<usize>, // v1.8.0 B3：坊市藏经阁求购二次确认（TECHS 下标；不进存档）

    pub rmodal: Option<EventCard>, // v1.8.0 B4/D2：现实侧好感事件卡（模态独占输入；不进存档）

    pub rmodal_npc: Option<usize>, // v1.8.0 D2：悬空事件归属 NPC（与 rmodal 同生共死；不进存档）

    pub refine_pending: Option<usize>, // v1.8.0 E3：行囊法宝炼化选材态（目标法宝下标；不进存档；Esc 清理点已随 B4 前置建立）

    pub modal_desc: Option<(String, String, Vec<(String, String)>)>, // v1.10.1：模态卡面惰性缓存（title,text,各选项(label,desc)；不进存档，逐帧零重建）

    pub shop_tab: u8,

    pub world: crate::world::WorldState, // 山河图志会话状态（不进存档）

    pub submenu: Option<crate::menu::Sub>, // v1.6.0 B：当前弹层（None 无弹层，不进存档）
    pub bag_tab: u8,   // v1.6.0 B：行囊页签 0丹药 1法宝 2行装

    pub ad: Option<f32>,           // v1.7.0：幻境广告倒计时（Some 播放中/Some(0.0) 可领取；不进存档）
    pub sac_pending: Option<usize>, // v1.7.0：行囊献祭二次确认的法宝下标（不进存档）
    pub sub_fresh: bool, // v1.6.0 B：开帧门闩——open_sub 置位、主循环顶部清零（硬伤 1）

    pub guide_last: Option<u8>, // v1.16.0 道途一念：上次绘制的指引下标（变更帧一次性预热；纯会话态不进存档，指引由状态纯推导零存档键）

    pub guide_tab: u8, // v1.16.8 章节书：道途指引面板页签 0当前目标 1章节书（会话态不进存档，照 help_tab 口径）

    pub guide_chap_last: Option<usize>, // v1.16.16 主线章签：横带右段上次绘制的当前章下标
                                        //（变更帧一次性预热，照 guide_last 先例；纯会话态
                                        // 不进存档——当前章由 meta 纯推导零存档键）

    pub zoom_art: Option<ZoomArt>, // v1.16.1 立绘鉴赏覆层（模态独占输入；不进存档）

    pub modal_min: bool, // v1.16.4 事件卡暂避态：收起卡面先看日志，徽章点开重回抉择（不进存档）

    // 剧情演出包：卡面/终局页入场动效剩余秒（会话态不进存档，照 tier_flash
    // game.rs:815 口径——各开启口置值后向 0 收敛，0 = 无动画；旧档载入天然 0
    // 不误触；暂避收起期间计时照走，重开卡面不会二次闪动）。计时递减挂主循环
    //（modal 开启期 update_sim 冻结，照 tier_flash 的 main.rs 先例）
    pub modal_anim: f32,   // 事件/道途/终战/好感/破境卡入场：0.18→0（draw_modal_card 消费）
    pub ending_anim: f32,  // 终局演出页入场节奏：0.6→0（draw_ending_page 消费；书志重读不置位即无动画）

    pub speed: usize, // 模拟速度 0暂停 1慢 2快 3跳过

    pub acc: f32,

    pub motes: Vec<Mote>,

    pub meteors: Vec<Meteor>,

    pub sparks: Vec<Spark>,

    pub shake: f32,

    pub toast: Option<(String, f32, Color)>,

    pub banner: Option<(String, f32, Color)>,

    pub settle: Option<SettleInfo>,

    pub offline_card: Option<OfflineReport>, // v1.10.2 离线闭关：出关结算卡（独占输入；不进存档）

    pub ending_show: Option<usize>, // v1.12.1 终卷四结局：问笔终战演出页覆层（ENDINGS 下标 0..=3；不进存档，合页即清）

    pub book_view: Option<usize>, // v1.12.1 书志重看覆层：panel=6 书志页点开已达成的结局条目，复用演出页绘制（不进存档）

    pub blank_used: bool, // v1.12.0 空白行：每稿至多一次门闩（瞬态，不进存档；launch_sim 清零）

    pub recent_events: Vec<usize>, // v1.16.5 防重复排除池：本世最近 K 次抽过的模拟事件 id（瞬态，不进存档；launch_sim 清零）

    // v1.16.7 连锁剧情：Some((链id, 下段id, ttl))——下段 0=首段卡在手待分支。
    // 纯会话态不进存档；launch_sim / quit_sim_redraft 清零，跨世绝不泄漏
    pub chain: Option<(u8, u8, u8)>,

    pub chain6_closed: bool, // v1.16.10 链6 本世是否已缘了（宿敌同世回响事件 110 的
                             // 入池门控读此标志；纯会话态不进存档，launch_sim 清零
                             // ——「同世回响」顾名思义只属推演中的这一世）

    // v1.16.16 链缘了闩：bit(链id-1) 置位=该链本世已缘了，首段不再入池
    //（choose_option 缘了口置位，launch_sim 清零；chain6_closed 的全线推广——
    // 专治长寿世同链复走。纯会话态不进存档；bit 只在缘了置位，中段弃链不置，
    // 弃链后同世仍可重走）
    pub chain_latch: u32,

    // v1.16.11 手札回味：图鉴已遇行点按回看的只读详情数据源（zoom_art 同款纯会话态，
    // 不进存档、launch_sim 不清——跨世只作展示缓存无模拟语义）。ev_memo[id]=该事件
    // 最近一次构建帧的卡面正文（build_event 尾 ev_memo_set 单点捕获，与 note_event
    // 同点同口径；None=本会话尚未构建过——图鉴降级显 EVENT_BRIEF 一句）；
    // ev_recall=Some(id) 回味覆层开启中（draw_reality/draw_char_panel 早退独占，
    // Esc/C/点按三口清；写入口两口——手札图鉴行 tap（ev_cnt>0 门控）与终局页
    // 本稿遭遇行点开（行 id 出自 run_events 已遇集，cnt>0 恒成立））
    pub ev_memo: Vec<Option<String>>,
    pub ev_recall: Option<usize>,

    // v1.16.21 会话态（不进存档——铁律③会话态优先于新持久键）：闭关摘要记账
    //（入关置年数、每载累计实际修为增量；出关金行/灰行摘要用，重载归零摘要
    // 退化为原句——语义可接受不入档）
    pub secl_yrs: u8,
    pub secl_qi: f64,

    // v1.16.21 会话态：远征步次胜率副标懒缓存（modal_desc 同款——步号变更帧
    // 重建一次，稳态逐帧零分配）。(步号, 激进副标, 谨慎副标)；exp_start/exp_quit
    // 双口清（跨次远征步号重复防串）
    pub exp_p_cache: Option<(usize, String, String)>,

    // v1.16.13 终局页『本稿遭遇』（Scene::Inherit 专属会话态，不进存档）：
    // run_events=本世遭遇去重有序表（note_event 单口 push、launch_sim 清零——
    // 切勿用 ev_cnt 反推本世列表，跨世累计无边界）；后三者为列表覆层 UI 态
    //（开关/滚动偏移/触拖锚点，与图鉴页 npc_scroll 状态机互不相涉）
    pub run_events: Vec<usize>,
    pub run_recall_open: bool,
    pub run_recall_scroll: f32,
    pub run_recall_drag: Option<(f32, f32, f32)>,

    pub time: f32,

    pub sfx_ok: Option<crate::sounds::Which>,

    pub audio: crate::sounds::Audio,
}

// ---------------- 颜色常量 ----------------

pub fn col_realm(r: usize) -> Color {
    let (r, g, b) = REALM_COLORS[r.min(14)];

    Color::from_rgba(r, g, b, 255)
}

pub const C_GOLD: Color = Color::from_rgba(240, 200, 110, 255);

pub const C_GOLD_D: Color = Color::from_rgba(160, 125, 55, 255);

pub const C_TEXT: Color = Color::from_rgba(225, 220, 205, 255);

pub const C_DIM: Color = Color::from_rgba(150, 148, 140, 255);

pub const C_CYAN: Color = Color::from_rgba(130, 220, 230, 255);

pub const C_GREEN: Color = Color::from_rgba(140, 230, 150, 255);

pub const C_RED: Color = Color::from_rgba(240, 120, 110, 255);

// v1.16.21 抉择后果可见化：琥珀=非好结果句/代价侧净变动行/破境抉择卡描边——
// 不占既有语义位（C_GREEN=疗愈好事、C_RED=受挫重伤、C_GOLD=大事记金），与
// C_GOLD(240,200,110) 拉开橙度、与 C_RED(240,120,110) 拉开红度
pub const C_AMBER: Color = Color::from_rgba(243, 166, 76, 255);

pub const C_PURPLE: Color = Color::from_rgba(230, 160, 230, 255);

pub const C_PANEL: Color = Color::from_rgba(22, 16, 38, 235);

pub const C_PANEL2: Color = Color::from_rgba(32, 24, 54, 235);

pub fn fmt_num(v: f64) -> String {
    let a = v.abs();

    if a < 10000.0 {
        if a < 10.0 && a.fract() > 0.01 {
            format!("{:.1}", v)
        } else {
            format!("{}", v as i64)
        }
    } else if a < 1e8 {
        let q = v / 1e4;

        if q.fract() < 0.05 {
            format!("{:.0}万", q)
        } else {
            format!("{:.1}万", q)
        }
    } else if a < 1e12 {
        format!("{:.2}亿", v / 1e8)
    } else if a < 1e16 {
        format!("{:.2}兆", v / 1e12)
    } else {
        format!("{:.2}京", v / 1e16)
    }
}

pub fn fmt_int(v: i64) -> String {
    if v.abs() < 100000 {
        format!("{}", v)
    } else {
        fmt_num(v as f64)
    }
}

// ---------------- v1.16.21 抉择后果可见化：净变动聚合行 ----------------
// 抉择结算后把通用数值通道的实际差值并成一行「灵石-300·道心+5」（此前
// stones/dao/luck/life_y 静默无日志，玩家选完看不到这一选带来什么）。仅事件帧
// 调用（format! 事件帧豁免）；正负分色：全正 C_CYAN（进益）/全负 C_AMBER（对价）/
// 混合 C_DIM（中性账目行）。字全为既有语料字零新增。

// 符号化整数：+300 / -300
fn led_i(v: i64) -> String {
    if v >= 0 {
        format!("+{}", fmt_int(v))
    } else {
        format!("-{}", fmt_int(-v))
    }
}

// 符号化小数：+5 / -2.5（整数差值省小数位，照 eff_desc 简语口径）
fn led_f(v: f64) -> String {
    if v >= 0.0 {
        if v.fract().abs() < 0.05 {
            format!("+{:.0}", v)
        } else {
            format!("+{:.1}", v)
        }
    } else {
        let a = -v;
        if a.fract().abs() < 0.05 {
            format!("-{:.0}", a)
        } else {
            format!("-{:.1}", a)
        }
    }
}

// 净变动聚合行：传入各通道差值（非零通道逐段入行），返回 (行文, 分色)。
// 全零返回 None（无账可记不发行——特殊臂直改 sim 不经通用通道，差值恒零）。
// ds=灵石 dd=道心 dl=气运 dly=寿元(载) dq=修为 dy=道韵 dc=贡献（后两者仅现实侧
// apply_reff 非零，模拟侧传 0；修为现实侧已单独记账传 0）
pub(crate) fn ledger_line(
    ds: i64,
    dd: f64,
    dl: f64,
    dly: f64,
    dq: f64,
    dy: i64,
    dc: i64,
) -> Option<(String, Color)> {
    let mut parts: Vec<String> = Vec::new();
    let mut pos = false;
    let mut neg = false;

    if ds != 0 {
        pos |= ds > 0;
        neg |= ds < 0;
        parts.push(format!("灵石{}", led_i(ds)));
    }
    if dd != 0.0 {
        pos |= dd > 0.0;
        neg |= dd < 0.0;
        parts.push(format!("道心{}", led_f(dd)));
    }
    if dl != 0.0 {
        pos |= dl > 0.0;
        neg |= dl < 0.0;
        parts.push(format!("气运{}", led_f(dl)));
    }
    if dly != 0.0 {
        pos |= dly > 0.0;
        neg |= dly < 0.0;
        parts.push(format!("寿元{}载", led_f(dly)));
    }
    if dq > 0.0 {
        pos = true;
        parts.push(format!("修为+{}", fmt_num(dq)));
    }
    if dy != 0 {
        pos |= dy > 0;
        neg |= dy < 0;
        parts.push(format!("道韵{}", led_i(dy)));
    }
    if dc > 0 {
        pos = true;
        parts.push(format!("贡献+{}", fmt_int(dc)));
    }

    if parts.is_empty() {
        return None;
    }

    let c = if neg && !pos {
        C_AMBER
    } else if pos && !neg {
        C_CYAN
    } else {
        C_DIM
    };

    Some((parts.join("·"), c))
}

// ---------------- 公共派生 ----------------

pub fn qi_need(l: &Life) -> f64 {
    let rd = &REALMS[l.realm];

    let m = if l.realm == 1 {
        1.28f64.powi(l.layer as i32)
    } else {
        LAYER4[l.layer.min(3)]
    };

    rd.qi * m
}

pub fn realm_full_name(l: &Life) -> String {
    let rd = &REALMS[l.realm];

    if l.realm == 0 {
        "白丁之躯".to_string()
    } else {
        format!(
            "{}·{}",
            rd.name,
            rd.layers[l.layer.min(rd.layers.len() - 1)]
        )
    }
}

pub fn root_mult(l: &Life) -> f64 {
    0.5 + l.root as f64 / 50.0
}

pub fn env_mult(l: &Life) -> f64 {
    let mut e = 1.0;

    if l.sect {
        e *= 1.3;
    }

    if l.lingmai {
        e *= 1.5;
    }

    e
}

pub fn relic_cult(l: &Life) -> f64 {
    let mut c = 1.0;

    // v1.5.0 装备态：已装备法宝全额生效，未装备按 RELIC_UNEQUIP_FRAC 计三成灵性
    // v1.8.0 E：灵纹等级经 relic_cult_at 计入
    for &r in &l.relics {
        let m = if l.equipped.contains(&r) {
            relic_cult_at(r, l.relic_lv[r.min(7)])
        } else {
            1.0 + (relic_cult_at(r, l.relic_lv[r.min(7)]) - 1.0) * RELIC_UNEQUIP_FRAC
        };

        c *= m;
    }

    c
}

// 天赋效果聚合

pub fn teff(l: &Life) -> TEff {
    let mut e = TEff::default();

    for &t in &l.talents {
        let te = TALENTS[t].e;

        e.root += te.root;

        e.dao += te.dao;

        e.luck += te.luck;

        e.body += te.body;

        e.life_add += te.life_add;

        if te.life_mult > 0.0 {
            e.life_mult = if e.life_mult > 0.0 {
                e.life_mult * te.life_mult
            } else {
                te.life_mult
            };
        }

        e.stone += te.stone;

        if te.cult_mult > 0.0 {
            e.cult_mult = if e.cult_mult > 0.0 {
                e.cult_mult * te.cult_mult
            } else {
                te.cult_mult
            };
        }

        if te.power_mult > 0.0 {
            e.power_mult = if e.power_mult > 0.0 {
                e.power_mult * te.power_mult
            } else {
                te.power_mult
            };
        }

        e.break_add += te.break_add;

        e.trib_add += te.trib_add;

        if te.event_mult > 0.0 {
            e.event_mult = if e.event_mult > 0.0 {
                e.event_mult * te.event_mult
            } else {
                te.event_mult
            };
        }

        if te.pill_mult > 0.0 {
            e.pill_mult = if e.pill_mult > 0.0 {
                e.pill_mult * te.pill_mult
            } else {
                te.pill_mult
            };
        }

        if te.relic.is_some() {
            e.relic = te.relic;
        }

        if te.tech.is_some() {
            e.tech = te.tech;
        }
    }

    e
}

pub fn cult_rate(l: &Life, meta: &Meta) -> f64 {
    let mut r = 3.0
        * root_mult(l)
        * TECHS[l.tech].mult
        * env_mult(l)
        * relic_cult(l)
        * (1.0 + meta.upg[7] as f64 * 0.12)
        * teff(l).cult_mult.max(1.0);

    if l.companion.is_some() {
        // v1.6.0 D：双修倍率由道侣好感档决定（结缘起步 25 → ×1.20，生死之交 ×1.35）
        r *= crate::npc::comp_cult_mult(l.comp_fav);

        if meta.feng_blv >= 1 {
            r *= crate::data::FENG_CULT_MULT; // v1.12.1 缝页缘一：双修再乘 ×1.10（双侧共用本率）
        }
    }

    if l.injured > 0 {
        r *= 0.5;
    }

    // v1.10.2 成行择道：气修（path=2）每载修为 ×1.25（乘区尾接）
    if l.path == PATH_QI {
        r *= 1.25;
    }

    r
}

pub fn power(l: &Life) -> f64 {
    let rd = &REALMS[l.realm];

    let mut p = rd.power
        * (1.0 + l.layer as f64 * 0.25)
        * TECHS[l.tech].pow
        * (1.0 + l.body as f64 / 100.0)
        * teff(l).power_mult.max(1.0);

    // v1.5.0 装备态：已装备法宝全额生效，未装备按 RELIC_UNEQUIP_FRAC 计三成灵性
    // v1.8.0 E：灵纹等级经 relic_pow_at 计入
    for &r in &l.relics {
        p *= 1.0
            + if l.equipped.contains(&r) {
                relic_pow_at(r, l.relic_lv[r.min(7)])
            } else {
                relic_pow_at(r, l.relic_lv[r.min(7)]) * RELIC_UNEQUIP_FRAC
            };
    }

    // v1.10.2 成行择道：剑修（path=1）战力 ×1.35（乘区尾接；模拟侧 path 恒 0 不受益）
    if l.path == PATH_SWORD {
        p *= 1.35;
    }

    p *= 1.0 + l.power_buff;

    if l.injured > 0 {
        p *= 0.7;
    }

    p
}

pub fn big_break_p(l: &Life, target: usize) -> f32 {
    let rd = &REALMS[target];

    let e = teff(l);

    let mut p = rd.bbreak + l.dao / 300.0 + l.luck / 400.0 + e.break_add;

    if target == 2 && l.pills[1] > 0 {
        p += 0.25;
    }

    // v1.10.2 成行择道：气修（path=2）破境率 +0.06（加在 0.05..0.95 钳制之前）
    if l.path == PATH_QI {
        p += 0.06;
    }

    if rd.immortal {
        p.max(0.05).min(0.95)
    } else {
        p.max(0.05).min(0.95)
    }
}

impl Game {
    pub fn new(audio: crate::sounds::Audio) -> Self {
        let mut meta = save::load();

        // v1.16.8 章节书：旧档载入帧按事实静默补齐达成位（零迁移零横幅——
        // 老玩家开章节书即见已了章，此后只有新达成才走 check_chapters 边沿演出）
        meta.chap_done |= Self::chapter_done_mask(&meta);

        let mut motes = Vec::new();

        for _ in 0..42 {
            motes.push(Mote {
                x: gen_range(0.0, V_W),

                y: gen_range(0.0, V_H),

                spd: gen_range(6.0, 22.0),

                size: gen_range(1.5, 3.2),

                gold: gen_range(0.0, 1.0) < 0.3,

                ph: gen_range(0.0, 6.28),
            });
        }

        Game {
            scene: Scene::Menu,

            meta,

            life: None,

            sim: None,

            sim_start_age: 0.0,

            sim_log: Vec::new(),

            sim_result: None,

            inherit_opts: Vec::new(),

            inherit_picked: 0,

            draft: Vec::new(),

            blessed: false,

            picked: Vec::new(),

            log: Vec::new(),

            modal: None,

            panel: 0,

            char_tab: 0,

            npc_scroll: 0.0,
            npc_drag: None,
            npc_vel: 0.0,
            npc_anim: None,
            tier_flash: 0.0,

            save_dirty: false,
            save_timer: 0.0,
            prev_sig: (0, 0, false, false, false, false, false, false),

            auto_acc: 0.0,

            auto_years: 0,

            auto_gain: 0.0,

            auto_toast_done: false,

            tab: 0,

            help_tab: 0,

            buy_pending: None,

            rmodal: None,

            rmodal_npc: None,

            refine_pending: None,

            modal_desc: None,

            shop_tab: 0,

            world: crate::world::WorldState::new(),

            submenu: None,

            bag_tab: 0,

            ad: None,

            sac_pending: None,

            sub_fresh: false,

            guide_last: None,
            guide_chap_last: None,

            guide_tab: 0,

            zoom_art: None,

            modal_min: false,
            modal_anim: 0.0,
            ending_anim: 0.0,

            speed: 2,

            acc: 0.0,

            motes,

            meteors: Vec::new(),

            sparks: Vec::new(),

            shake: 0.0,

            toast: None,

            banner: None,

            settle: None,

            offline_card: None,

            ending_show: None,

            book_view: None,

            blank_used: false,

            recent_events: Vec::new(),

            chain: None,

            chain6_closed: false,
            chain_latch: 0,

            ev_memo: vec![None; EVENTS.len()],

            ev_recall: None,

            secl_yrs: 0,

            secl_qi: 0.0,

            exp_p_cache: None,

            run_events: Vec::new(),
            run_recall_open: false,
            run_recall_scroll: 0.0,
            run_recall_drag: None,

            time: 0.0,

            sfx_ok: None,

            audio,
        }
    }

    // 统一存档入口：先把现实主角快照进 meta.life 再落盘
    // （v1.4 起：保证「承接前缘」始终有档可承；draw.rs 菜单音乐/音效开关仍直写 save::save，
    //   因菜单场景 g.life 必为 None，sync_save 会把快照覆写为 None 落盘）
    pub fn sync_save(&mut self) {
        self.meta.life = self.life.clone();

        // v1.10.2 落盘去抖：meta.life 快照保持即时（承接前缘/恢复逻辑依赖），
        // 磁盘写推迟到 save_flush（主循环：场景/弹窗/面板开合边界 或 ≥3s 一落）；
        // wasm 侧同步暂存快照供 pagehide/visibilitychange 强制落盘兜底。
        self.save_dirty = true;
        #[cfg(target_arch = "wasm32")]
        save::stash_pending(&self.meta);
    }

    // v1.10.2：把去抖窗口内的变更统一落盘（幂等；无脏即 no-op）
    pub fn save_flush(&mut self) {
        if !self.save_dirty {
            return;
        }
        self.save_dirty = false;
        self.save_timer = 0.0;
        // 离线起点=最后一次落盘时刻：真实落盘才推进 ts 基准——
        // 不推进会把「在局游玩时长」按离线口径重算（多算）
        self.meta.last_ts = date_now();
        save::save(&self.meta);
        // v1.16.13 复审⑨：真落盘后清 pagehide 兜底快照（wasm）——磁盘已含推进后
        // 的 ts=，旧快照（save_text_with_ts 注入时刻的文本）继续回写只会把离线
        // 收益起点往回拨；清后 index.html 兜底 len<=0 天然跳过。桌面零路径。
        #[cfg(target_arch = "wasm32")]
        save::clear_pending();
    }

    pub fn log(&mut self, text: impl Into<String>, color: Color) {
        self.log.push(LogLine {
            text: text.into(),

            color,
        });

        if self.log.len() > 120 {
            self.log.remove(0);
        }
    }

    pub fn slog(&mut self, text: impl Into<String>, color: Color) {
        let text = text.into();

        // 自动带岁数前缀（轮回模拟器的灵魂）

        let text = self
            .sim
            .as_ref()
            .map(|s| format!("[{}岁] {}", s.age as i32, text))
            .unwrap_or(text);

        self.sim_log.push(LogLine { text, color });

        if self.sim_log.len() > 200 {
            self.sim_log.remove(0);
        }
    }

    pub fn toast(&mut self, text: impl Into<String>, c: Color) {
        // v1.16.18 搭车（show_banner 先例 game.rs show_banner：弹出帧预热渲染档字形）：
        // toast 此前零预热——动态串冷字形弹帧才栅格化 + 图集 dirty 整图重传
        // （2048²=16MB/次）。本口是全部 toast 唯一写入口，弹帧一次灌 @20 渲染档，
        // 稳态零开销（v1.9.25 根因1 同口径收敛）
        let text = text.into();
        crate::ui::prewarm_text(&text, &[20]);
        self.toast = Some((text, 2.6, c));
    }

    pub fn play(&mut self, w: crate::sounds::Which) {
        self.sfx_ok = Some(w);
    }

    // v1.16.10 事件回顾图鉴：遭遇计数唯一写入口（图鉴收录=ev_cnt[id]>0 纯推导，
    // 不另设 seen 位）。build_event 尾 opts 齐备处单点调用——轮盘与链内直取两路
    // 共用此口（链段亦收录）；id 越界卫语句直接 return（panic=abort 纪律）。
    pub fn note_event(&mut self, id: usize) {
        if id >= self.meta.ev_cnt.len() {
            return;
        }
        self.meta.ev_cnt[id] = self.meta.ev_cnt[id].saturating_add(1);
        // v1.16.13 终局页『本稿遭遇』：本世遭遇去重有序表（唯一写入口，与 ev_cnt
        // 跨世累计分域——图鉴收录口径不变，本表只述「这一世遇见过谁」）
        if !self.run_events.contains(&id) {
            self.run_events.push(id);
        }
    }

    // v1.16.11 手札回味：卡面正文捕获唯一写入口（与 note_event 同点调用——build_event
    // 尾 opts 齐备处，构建帧即存）。纯会话展示缓存：不进存档、launch_sim 不清（跨世
    // 仅作回看数据，无任何模拟语义）；id 越界卫语句直接 return（panic=abort 纪律）。
    // 同 id 重复遭遇覆盖为最新正文（动态数值随当次推演，回看取最近所见）
    pub fn ev_memo_set(&mut self, id: usize, text: &str) {
        if id >= self.ev_memo.len() {
            return;
        }
        self.ev_memo[id] = Some(text.to_string());
    }

    // v1.10.2：人物页滚动会话态归零（×/Esc 关面板同批归零，先例 char_page 归零口径）
    // v1.16.11：手札回味覆层态同批归零——本口被角色面板 ×（exec_panel_act Close）与
    // Esc 关面板臂两处共用，一并清 ev_recall（覆层独占期这两口不可达，纯防御防泄漏）
    pub fn npc_scroll_reset(&mut self) {
        self.npc_scroll = 0.0;
        self.npc_drag = None;
        self.npc_vel = 0.0;
        self.npc_anim = None;
        self.ev_recall = None;
    }

    // v1.16.13 C/M 热键避让守卫单口（main.rs Reality 臂与单测共用）：弹层/覆层任一
    // 开启即避让，防覆层期热键穿透。v1.16.13 补 ev_recall（手札回味覆层独占期
    // C 原先穿透置 panel——覆层期一步跳两层，借 zoom_art 先例同批收紧）
    pub fn key_overlay_blocking(&self) -> bool {
        self.submenu.is_some()
            || self.rmodal.is_some()
            || self.offline_card.is_some()
            || self.ending_show.is_some()
            || self.zoom_art.is_some()
            || self.ev_recall.is_some()
    }

    // v1.16.0 交游图鉴：历世结缘记账（唯一写入口）。结缘即录入——挂 apply_eff special 17
    // 『结为道侣/同页天成』臂，quit_sim_redraft 重掷不回退（这正是选结缘点而非 sim_die
    // 的原因：feng_bond_up 只覆盖正常落幕，漏 quit 路径）；换道侣不回退旧卡（次数只增）。
    // idx 越界卫语句直接 return 绝不索引越界（panic=abort 纪律）；saturating_add 封顶 255。
    pub fn gallery_bond(&mut self, idx: usize) {
        if idx >= crate::data::COMPANION_NAMES.len() {
            return;
        }
        self.meta.comp_bonds[idx] = self.meta.comp_bonds[idx].saturating_add(1);
    }

    // v1.14.0 立绘焕新时刻：跨相判定——破境后立绘档（portrait_tier）高于破境前时，
    // toast『道貌焕然——【X之相】』+ 金环闪光 1.5s（会话态 tier_flash，不进存档；
    // 同档内再破不触发，旧档载入 tier_flash 恒 0 不误触）。old_tier 须在 realm 赋值前
    // 取快照，四处破境口（现实跨境/现实问笔/模拟破境/模拟问笔）共用本口。
    pub fn stage_awaken(&mut self, old_tier: usize, new_realm: usize) {
        let nt = portrait_tier(new_realm);

        if nt > old_tier {
            self.toast(
                format!("道貌焕然——【{}】", PORTRAIT_STAGE_NAMES[nt]),
                C_GOLD,
            );

            self.tier_flash = 1.5;
        }
    }

    pub fn show_banner(&mut self, text: impl Into<String>, c: Color) {
        let text = text.into();

        // v1.9.25 卡顿修复（根因1）：弹出事件帧预热 pop 全档位字形 {54,60,66,72}——
        // 动画帧只命中缓存（0 次新字形栅格化/0 次图集整图上传），把栅格化+上传摊到事件帧一次
        crate::ui::prewarm_text(&text, &[54, 60, 66, 72]);

        self.banner = Some((text, 2.4, c));
    }

    pub fn title_of(&self) -> String {
        // 特殊称号优先

        if self.meta.neme_slain >= 3 {
            return "弑神者".to_string();
        }

        if let Some(l) = &self.life {
            if l.max_realm >= 10 && l.realm >= 10 {
                return realm_title(l.max_realm).to_string();
            }

            return realm_title(l.max_realm).to_string();
        }

        realm_title(self.meta.best_realm).to_string()
    }

    pub fn stone_scale(&self, realm: usize) -> f64 {
        (1.0 + realm as f64).powf(1.6)
    }

    // ================= 新的一轮人生（现实主角） =================

    pub fn begin_mortal(&mut self) {
        self.meta.lives += 1;

        let upg = self.meta.upg;

        let r0 = gen_range(0.0, 30.0f32);

        let root = r0.max(upg[0] as f32 * 8.0);

        let dao: f32 = 30.0 + gen_range(0.0, 10.0);

        let luck: f32 = 20.0 + gen_range(0.0, 10.0);

        let body: f32 = 10.0 + gen_range(0.0, 10.0);

        let dao = dao.max(upg[1] as f32 * 8.0);

        let luck = luck.max(upg[2] as f32 * 8.0);

        let body = body.max(upg[3] as f32 * 8.0);

        let lifespan = REALMS[0].life * (1.0 + upg[8] as f64 * 0.10);

        // v1.12.1 缝页缘二：开局灵石 ×1.3（模拟侧由现实克隆自动继承）
        let stones = 100 * (1i64 << upg[6].min(4) as i64)
            * if self.meta.feng_blv >= 2 { 13 } else { 10 }
            / 10;

        let life = Life {
            n: self.meta.lives,

            age: 16.0,

            lifespan,

            root: root.clamp(1.0, 100.0),

            dao: dao.clamp(0.0, 100.0),

            luck: luck.clamp(0.0, 100.0),

            body: body.clamp(0.0, 100.0),

            stones,

            ..Default::default()
        };

        self.meta.neme = Some(Nemes {
            title: NEME_TITLES[0],

            name: rand_name(),

            realm: 3,
        });

        self.meta.sim_points = 3;

        self.life = Some(life);

        self.world = crate::world::WorldState::new();

        self.log.clear();

        self.scene = Scene::Reality;

        let oi = gen_range(0, ORIGINS.len());

        let nm = self.meta.neme.as_ref().unwrap().full();

        self.log(
            format!("你转生于{}，{}。", ORIGINS[oi].name, ORIGINS[oi].desc),
            C_TEXT,
        );

        self.log(
            "雨夜。乱葬岗。你从尸堆里爬出来，手里攥着一枚方印——印身无字，只有层层叠叠被划掉的痕迹。",
            C_GOLD,
        );

        self.log(
            "识海里有个少年人的嗓子，累得不像少年人：\"先说好——你的名字是错的。天写错了。\"",
            C_CYAN,
        );

        self.log(
            format!("此外，血仇未销——{}。它删过你的字、批过你的行；此吏不除，书不成书。", nm),
            C_RED,
        );

        self.log(
            "【开始模拟】推演一生，【继承】带回归途。此身虽弱，万世可期。",
            C_TEXT,
        );

        self.check_ach_all();

        // v1.8.0 C5：新世开局即处当前区域（新档为东境青云）——补记到访
        self.visit_region(true);

        // v1.16.8 章节书：转生入世事实翻转点（首世=第一章转生目标边沿）
        self.check_chapters();

        // v1.16.10 新手首世一次性提示（sy= bit2）：首入现实指路一句——顶部金框是
        // 当前目标、「任务」钮随时查看（教会横带+任务钮双入口）。闩仅首世（lives==1）
        // 置位；置于本函数尾——visit_region 初至/check_chapters 的 toast 在前，单槽
        // 后写胜语义下本句最后写必显（金行 log 双通道兜底）。
        if self.meta.lives == 1 && self.meta.sys_seen & 0b0100 == 0 {
            self.meta.sys_seen |= 0b0100;
            self.log(
                "识海里那声叹息又响：低头赶路，也要抬头看目标——界面顶部金框，即你此世的道途所在。",
                C_GOLD,
            );
            self.toast("第 1 世——顶部金框是当前目标，「任务」钮随时查看", C_GOLD);
        }

        self.sync_save();
    }

    // v1.9.23：模拟未沉入/中途放弃时退出并重新刷词条（退还本次点数）。
    // 真机场景：点了「开始模拟」但因任何原因没有沉入推演（输入未消费/
    // 界面卡住），玩家需要一条确定性的退路——退出→重刷词条→再次进入。
    pub fn quit_sim_redraft(&mut self) {
        if self.scene != Scene::Sim {
            return;
        }
        self.meta.sim_points += self.sim_cost();
        self.sim = None;
        self.modal = None;
        self.chain = None; // v1.16.7 重掷清链（未竟之缘随镜面散去）
        self.log("轮回镜轻颤，此番推演未竟，点数已还。", C_CYAN);
        self.start_draft();
    }

    // v1.9.23：返回开始界面（存档已落盘；菜单里「承接前缘」可恢复）
    pub fn back_to_menu(&mut self) {
        if self.scene != Scene::Reality {
            return;
        }
        self.sync_save();
        self.scene = Scene::Menu;
        self.play(crate::sounds::Which::Page);
    }

    // ================= v1.10.2 离线闭关 / 承接前缘 =================

    // 承接前缘：恢复现实主角（自 draw.rs 菜单按钮抽出——前置修复：菜单 8 秒自动进入
    // 曾无条件 begin_mortal，有档玩家挂菜单即被覆盖旧档人生；抽出后自动进入与按钮同口）。
    // 恢复即结算离线收益（N≥10 弹结算卡 / 1≤N<10 静默入账 / N=0 不动）并补弹旧档道途卡。
    pub fn resume_life(&mut self) {
        self.life = self.meta.life.clone(); // 还原现实主角（缺失则 draw_reality 黑屏）
        self.panel = 0;
        // v1.8.0 C5：承接旧档补记当前区域到访（旧档缺 mvis 键已由 save 层迁移；
        // 此处兜底显式已至但未记的区域——已置位则为 no-op）
        self.visit_region(true);
        self.scene = Scene::Reality;
        self.log("轮回镜微亮，前缘未了。", C_TEXT);
        self.sync_taodie(); // v1.12.1 讨牒旧档首检：neme_slain>0 自动补齐职牒（零迁移）
        // v1.10.2 离线闭关：恢复即结算（收益先入账内存，落盘交收功/去抖/pagehide）
        self.offline_settle();
        // v1.10.2 成行择道：旧档 realm≥2 且未择道 → 补弹道途卡（offline_card 若在弹，
        // 其独占分支先行，收功后道途卡经 rmodal 自然接棒）
        if self
            .life
            .as_ref()
            .is_some_and(|l| l.realm >= 2 && l.path == PATH_NONE)
        {
            self.open_path_card();
        }
    }

    // 菜单 8 秒自动进入的统一入口：有档恢复（含离线结算），无档开新人生
    pub fn resume_or_begin(&mut self) {
        if self.meta.life.is_some() {
            self.resume_life();
        } else {
            self.begin_mortal();
        }
    }

    // 离线结算：以最后一次落盘（meta.last_ts）为起点算收益。严禁循环 act_cultivate
    //（其尾部逐载 sync_save 会打爆 localStorage）与 advance_time（唯一推岁口含天气
    // 转移 roll 与宿敌成长，离线一律静态口径）——qi 累加后不自动破境，交还周天吐纳
    // 既有衔接（tick_auto→act_cultivate 主循环下次自然接管）。
    pub fn offline_settle(&mut self) {
        self.offline_settle_inner(date_now());
    }

    // v1.16.12 免烧寿元：now 参数化的内部版（单测可注入时钟，整链不触真实时间）。
    // 离线只发收益（qi/stones），绝不推 age——旧实现 l.age += n 把寿元精确烧到
    // lifespan−1 风烛位（承接前缘必寿元将近的根因）；破境节奏只由在线 1.6s/载驱动。
    fn offline_settle_inner(&mut self, now: u64) {
        let Some(l) = self.life.as_ref() else {
            return;
        };
        let gain = self.cult_gain_per_year();
        let lv = self.meta.dongfu[1].clamp(0, 3) as usize;
        // v1.10.2 复审③：丹修（path=3）灵田岁入 ×1.6——与唯一推岁口 advance_time 的
        // 静态项同式，同一载在线/离线口径一致（漏乘则丹修离闭收益少 60%）
        let path_pill = if l.path == PATH_PILL { 1.6 } else { 1.0 };
        let lingtian = (LINGTIAN_YIELD[lv.min(3)] as f64
            * weather_loc_mult(l.weather as usize, l.loc)
            * path_pill)
        .round() as i64;
        let Some((n, qi, stones)) =
            offline_settle_calc(self.meta.last_ts, now, gain, lingtian)
        else {
            self.meta.last_ts = now; // 本次不计收益，仅落新基准
            return;
        };
        {
            let l = self.life.as_mut().unwrap();
            l.qi += qi; // 与 act_cultivate 完全同式累加；修为满不破境（不调破境口）
            l.stones += stones;
            // v1.16.12 免烧寿元：不再 l.age += n（收益倍率 N 仍全额发放）
        }
        self.meta.last_ts = now;
        if n >= 10 {
            self.offline_card = Some(OfflineReport { years: n, qi, stones });
        } else {
            self.log(
                format!(
                    "离闭 {} 载，修为+{}，灵田岁入灵石 {}。",
                    n,
                    fmt_num(qi),
                    fmt_int(stones)
                ),
                C_CYAN,
            );
        }
    }

    // ================= v1.10.2 成行择道：三途分流 =================

    // 弹道途卡（rmodal 独占模态；neme 卡/事件卡不可同帧抢输入——rmodal 空时才弹）。
    // 三钮副标由 draw_modal_card 的 CardKind::Path 分支统一为「择定终身，不可改道」。
    pub fn open_path_card(&mut self) {
        if self.rmodal.is_some() {
            return;
        }
        // 剧情演出包①：事件帧预热（标题 24/正文 16，同 build_event 先例）+ 入场动效
        let card = EventCard {
            kind: CardKind::Path,
            actor: None, // 道途卡无人物立绘，维持 600×400 原版式
            rare: false,
            stage: None,
            cat: None,
            scene: None,
            brk: false,
            title: "成行择道 · 三途分流".into(),
            text: "成行之上道途分三，一经择定终身不改（模拟推演不携道途）：剑修战力×1.35；气修每载修为×1.25、破境率+6%；丹修服丹效果×1.35、灵田岁入×1.6。你将何以证道？".into(),
            opts: vec![
                ("剑修 · 战力 ×1.35".into(), Eff::new()),
                ("气修 · 修为 ×1.25 破境 +6%".into(), Eff::new()),
                ("丹修 · 服丹 ×1.35 灵田 ×1.6".into(), Eff::new()),
            ],
        };
        crate::ui::prewarm_text(&card.title, &[24]);
        crate::ui::prewarm_text(&card.text, &[16]);
        self.modal_anim = 0.18;
        self.rmodal = Some(card);
        self.rmodal_npc = None; // 道途卡不归属 NPC
    }

    // 道途抉择：选定终身、无改道入口；未到成行（realm<2）或已择者卫语句拒绝
    pub fn choose_path(&mut self, p: u8) {
        self.rmodal = None;
        self.rmodal_npc = None;
        if p == 0 || p > 3 {
            return;
        }
        let Some(l) = self.life.as_mut() else {
            return;
        };
        if l.path != PATH_NONE || l.realm < 2 {
            return;
        }
        l.path = p;
        let name = path_name(p);
        self.log(
            format!("成行关口，你择定道途——{}。自此终身不改。", name),
            C_GOLD,
        );
        self.play(crate::sounds::Which::Ascend);
        self.sync_save();
    }

    // ================= v1.12.1 终卷 · 四结局（未竟稿 §9）=================

    // 弹终战抉择卡（rmodal 独占模态，复用道途卡同款机制；rmodal 被占时不弹——
    // 与 open_path_card 同口径。触发单口：act_break target==10 九重雷劫成功段
    // 且 meta.neme_slain>=6（封卷吏已斩=宿敌链 NEME_TITLES[min(lv,5)] 链顶已通）。
    // 卡面 3 选项常显（A 认写 / B 撕稿·推荐位 / C 夺笔），D《合订》仅当
    // endings_seen bit1（B 已达成）时追加为第 4 钮——§12「后事不详→合订」直译，
    // 构成「达成 B 后再轮回一世、二次问笔」的跨世钩子）。
    pub fn open_finale_card(&mut self) {
        if self.rmodal.is_some() {
            return;
        }
        let mut opts = vec![
            ("《定稿》 · 认写".to_string(), Eff::new()),
            ("《留白》 · 撕稿".to_string(), Eff::new()),
            ("《执笔》 · 夺笔".to_string(), Eff::new()),
        ];
        if self.meta.endings_seen & 0b0010 != 0 {
            opts.push(("《合订》 · 收册人（隐藏）".to_string(), Eff::new()));
        }
        // 剧情演出包①：事件帧预热（标题 24/正文 16，同 build_event 先例）+ 入场动效
        let card = EventCard {
            kind: CardKind::Finale,
            actor: None, // 终战卡无人物立绘，维持 600×400 原版式
            rare: false,
            stage: None,
            cat: None,
            scene: None,
            brk: false,
            title: "终卷 · 执笔人递笔".into(),
            text: "你赢了封卷吏。执笔人递笔：「字不错。入正文吧。」\n「我还能改自己吗？」「一字不改。永生换的。」\n别等他写，字自己跑。六牒已齐（讨牒 6/6），问笔在即。".into(),
            opts,
        };
        crate::ui::prewarm_text(&card.title, &[24]);
        crate::ui::prewarm_text(&card.text, &[16]);
        self.modal_anim = 0.18;
        self.rmodal = Some(card);
        self.rmodal_npc = None; // 终战卡不归属 NPC
    }

    // 终战抉择：i 为卡面选项下标（0认写 1撕稿 2夺笔 3合订[仅解锁时存在]）。
    // 置位掩码 → 写 log → 横幅 → 落盘 → 置演出页覆层（合页后回 Reality，雷劫
    // burst 已在前，衔接自然；模拟侧推演飞升不挂卡——推演不算数，终战只属现实层）。
    pub fn choose_ending(&mut self, i: usize) {
        self.rmodal = None;
        self.rmodal_npc = None;
        let d_unlocked = self.meta.endings_seen & 0b0010 != 0;
        let e = match i {
            0..=2 => i,
            3 if d_unlocked => 3,
            _ => return, // 未解锁的 D 位（脏点击/未来卡面）卫语句拒绝
        };
        // v1.16.15 结局之礼：位边沿幂等——首录才发奖（照 check_chapters newly=
        // done&!chap_done 同款取舍：旧档已置位静默不补发，不回溯奖励）；奖励走
        // 金行 log（同帧 check_chapters 将推章节横幅，横幅单槽后写胜会互吞，
        // log 是唯一保底通道——数字写全）。全收集帧=count_ones 3→4 天然单帧
        // 边沿：金行 log+toast 指路（不发横幅、不叠道韵——四结局之礼已足额）
        let mask = 1u8 << e;
        let first = self.meta.endings_seen & mask == 0;
        self.meta.endings_seen |= mask;
        let ed = &ENDINGS[e];
        if first {
            let dy = ENDING_DY[e];
            self.meta.dao_yun += dy;
            self.log(
                format!("【结局之礼】首录《{}》——道韵 +{}", ed.name, dy),
                C_GOLD,
            );
            if self.meta.endings_seen == 0b1111 {
                self.log(
                    "【全卷之礼】四结局集齐——称号『执笔人』可佩（角色·成就称号页）。".to_string(),
                    C_GOLD,
                );
                self.toast("称号可佩：执笔人（角色·成就称号页）", C_GOLD);
            }
        }
        self.log(format!("【结局达成】{}——{}", ed.name, ed.alias), C_GOLD);
        self.show_banner(format!("结 局 · 《{}》", ed.name), C_GOLD);
        // 剧情演出包②：终局页入场节奏 0.6s（暗幕先行→标题→正文→合页钮解禁）。
        // 演出页全部字号（32 标题/17 正文/16 收益/18 合页钮）在事件帧一次预热，
        // 动画期零新字形——32/17 不在 UI_SIZES/PANEL_WARM 覆盖内，必须此处灌
        crate::ui::prewarm_text(&format!("《{}》 · {}", ed.name, ed.alias), &[32]);
        crate::ui::prewarm_text(ed.body, &[17]);
        crate::ui::prewarm_text(ed.gain, &[16]);
        crate::ui::prewarm_text("合上这一页", &[18]);
        self.ending_anim = 0.6;
        self.ending_show = Some(e);
        self.play(crate::sounds::Which::Page);
        // v1.16.8 章节书：落款成书事实翻转点（第八章边沿；位掩码幂等，书志重读不复播）
        self.check_chapters();
        self.sync_save();
    }

    // ================= 模拟：开始 =================

    pub fn sim_cost(&self) -> i32 {
        1
    }

    pub fn start_draft(&mut self) {
        if self.life.is_none() || self.meta.sim_points < self.sim_cost() {
            // v1.7.0：山穷水尽时不空手而归，引去看广告
            if self.life.is_some() && self.is_stuck() {
                self.start_ad();
            }
            return;
        }

        // 镜灵赐福：每 10 次模拟必现仙品

        let mut blessed = false;

        if self.meta.sims_since_bless >= 9 {
            blessed = true;

            self.meta.sims_since_bless = 0;

            self.toast("镜灵赐福！此番天机必现仙品", C_GOLD);

            self.check_ach(25);

            self.play(crate::sounds::Which::Ding);
        }

        self.blessed = blessed;

        // 抽 3（升级后 4）个候选

        let count = 3 + self.meta.upg[5].min(1) as usize;

        let mut pool: Vec<usize> = (0..TALENTS.len()).collect();

        // 洗牌（权重：凡 62 灵 30 仙 8）

        let mut picks = Vec::new();

        for _ in 0..count {
            let mut total = 0.0f32;

            for &i in &pool {
                total += match TALENTS[i].tier {
                    0 => 62.0,

                    1 => 30.0,

                    _ => 8.0,
                };
            }

            let mut roll = gen_range(0.0, total);

            let mut chosen = 0;

            for (k, &i) in pool.iter().enumerate() {
                roll -= match TALENTS[i].tier {
                    0 => 62.0,

                    1 => 30.0,

                    _ => 8.0,
                };

                if roll <= 0.0 {
                    chosen = k;

                    break;
                }
            }

            picks.push(pool.remove(chosen));
        }

        if blessed && !picks.iter().any(|&t| TALENTS[t].tier == 2) {
            let legens: Vec<usize> = (0..TALENTS.len())
                .filter(|&t| TALENTS[t].tier == 2)
                .collect();

            picks[0] = legens[gen_range(0, legens.len())];
        }

        self.draft = picks;

        self.picked.clear();

        self.scene = Scene::Draft;

        self.play(crate::sounds::Which::Page);
    }

    pub fn toggle_pick(&mut self, i: usize) {
        let max = 1 + self.meta.upg[5].min(1) as usize;

        if i >= self.draft.len() {
            return;
        }

        let t = self.draft[i];

        if let Some(pos) = self.picked.iter().position(|&x| x == t) {
            self.picked.remove(pos);
        } else if self.picked.len() < max {
            self.picked.push(t);
        } else {
            // v1.16.3 满选时点新天赋 = 直接换选（挤掉最早一个）。真机反馈「点了一个
            // 天赋就无法改变选择」：旧版忽略点击，玩家必须先点掉旧选才能换新——
            // 落笔（launch_sim）前候选永远可自由切换
            self.picked.remove(0);
            self.picked.push(t);
        }

        self.play(crate::sounds::Which::Click);
    }

    pub fn launch_sim(&mut self) {
        if self.picked.is_empty() || self.meta.sim_points < self.sim_cost() {
            return;
        }

        // 25% 返还 1 点

        if self.meta.upg[9] > 0 && gen_range(0.0, 1.0) < 0.25 {
            self.meta.sim_points += 1;

            self.log("轮回镜嗡鸣——本次演演不耗点数（点数亲和）。", C_CYAN);
        }

        self.meta.sim_points -= self.sim_cost();

        self.meta.total_sims += 1;

        self.meta.sims_since_bless += 1;

        let mut s = self.life.as_ref().unwrap().clone();

        // v1.5.0 必修：模拟主角由现实 Life 克隆而来，装备态重置为全量镜像
        // （否则模拟侧未装备法宝按 RELIC_UNEQUIP_FRAC=30% 计，低于 v1.4 的全量生效）
        s.equipped = s.relics.clone();

        s.talents = self.picked.clone();

        s.highlights.clear();

        s.seclusion = None;

        s.trib = None;

        // v1.10.2 成行择道：道途不进模拟——推演是另一条人生，择道只属现实主角；
        // 不剥离会把五处乘区全部带进模拟侧、破坏推演对照（单测 sim_does_not_inherit_path）
        s.path = 0;

        // v1.12.0 留行批注：上一稿的你点走的一行，开卷浮现（take 即消耗清空）；
        // 气运+2 仅此一梦——只微调善缘权重与心魔检定两处 luck 消费
        let note_line = self.meta.note_line.take();

        if note_line.is_some() {
            s.luck = (s.luck + 2.0).min(100.0);
        }

        // v1.12.0 空白行：每稿至多一次门闩随开稿清零
        self.blank_used = false;

        // v1.16.5 防重复排除池随新稿清零（「本世最近 K 次」的口径就是一世）
        self.recent_events.clear();

        // v1.16.7 连锁剧情随新稿清零（剧情链是一世之缘，跨世绝不泄漏）
        self.chain = None;

        // v1.16.10 链6 缘了标志同批清零（同世回响事件 110 只属推演中的这一世）
        self.chain6_closed = false;
        self.chain_latch = 0; // v1.16.16 链缘了闩随世清零（同 launch_sim 清链口）

        // v1.16.13 本稿遭遇表随新稿清零（终局页『本稿遭遇』数据源只属刚落幕的
        // 这一世——链态纪律同款：跨世绝不泄漏），列表覆层 UI 态同批归零
        self.run_events.clear();
        self.run_recall_open = false;
        self.run_recall_scroll = 0.0;
        self.run_recall_drag = None;

        self.sim = Some(s);

        self.sim_start_age = self.sim.as_ref().unwrap().age;

        self.sim_log.clear();

        self.speed = 2;

        self.acc = 0.0;

        self.scene = Scene::Sim;

        let names: Vec<&str> = self.picked.iter().map(|&t| TALENTS[t].name).collect();

        // v1.12.0 留行批注：开卷先浮现上一稿的批注，再入镜像
        if let Some(note) = &note_line {
            self.slog(format!("【批注】上一稿的你：「{}」", note), C_GOLD);
        }

        self.slog(
            format!(
                "轮回镜亮起，你意识沉入镜像……此世天赋：{}。",
                names.join("、")
            ),
            C_GOLD,
        );

        self.play(crate::sounds::Which::Ding);

        // v1.16.8 章节书：开卷首模拟事实翻转点（第一章达成边沿）
        self.check_chapters();
    }

    // ================= 氛围（粒子/震动/Toast） =================

    pub fn update_ambient(&mut self, dt: f32) {
        self.time += dt;

        for m in &mut self.motes {
            m.y -= m.spd * dt;

            if m.y < -10.0 {
                m.y = V_H + 10.0;

                m.x = gen_range(0.0, V_W);
            }
        }

        self.sparks.retain_mut(|s| {
            s.life -= dt;

            s.x += s.vx * dt;

            s.y += s.vy * dt;

            s.vy += 20.0 * dt;

            s.life > 0.0
        });

        if self.shake > 0.0 {
            self.shake = (self.shake - dt * 30.0).max(0.0);
        }

        if let Some((_, t, _)) = &mut self.toast {
            *t -= dt;

            if *t <= 0.0 {
                self.toast = None;
            }
        }

        if let Some((_, t, _)) = &mut self.banner {
            *t -= dt;

            if *t <= 0.0 {
                self.banner = None;
            }
        }

        // 流星：低概率生成

        if gen_range(0.0, 1.0) < dt * 0.08 {
            let from_left = gen_range(0.0, 1.0) < 0.5;

            let x = if from_left {
                gen_range(0.0, V_W * 0.4)
            } else {
                gen_range(V_W * 0.6, V_W)
            };

            self.meteors.push(Meteor {
                x,

                y: gen_range(20.0, 200.0),

                vx: if from_left {
                    gen_range(260.0, 420.0)
                } else {
                    -gen_range(260.0, 420.0)
                },

                vy: gen_range(90.0, 160.0),

                life: 1.1,
            });
        }

        self.meteors.retain_mut(|m| {
            m.x += m.vx * dt;

            m.y += m.vy * dt;

            m.life -= dt;

            m.life > 0.0
        });
    }

    // ================= 模拟：每年推演 =================

    pub fn update_sim(&mut self, dt: f32) {
        self.update_ambient(dt);

        if self.scene != Scene::Sim || self.sim.is_none() {
            return;
        }

        let in_trib = self.sim.as_ref().map_or(false, |s| s.trib.is_some());

        if in_trib {
            self.trib_tick(if self.speed == 3 { dt * 4.0 } else { dt });

            return;
        }

        if self.modal.is_some() {
            if self.speed == 3 {
                self.choose_option(0);
            }

            return;
        }

        let interval = SIM_SPEEDS[self.speed.min(3)];

        if self.speed == 3 {
            // 瞬间推演至死。v1.14.0 分帧预算（卡顿修复 P0）：原实现单帧连跑至多
            // 20000 次 sim_year（每次 sim_year/slog/sim_event 全是 format!+String
            // +Vec 分配）全堆在点「跳过」当帧 = 数百 ms 冻屏。改为每帧至多
            // SIM_SKIP_YEARS 年、未死下帧继续——1 秒级平滑滚到结局，日志可见地
            // 滚动；20000 guard 保留为单帧保险（原语义即每帧上限，非全局）。
            // v1.15.1 真机修正：X5 上 sim_year 比 PC 慢一个量级，固定 40 年/帧
            // 仍打出 6.1s 超长帧（HUD 峰6163ms 实证）——长帧期间触摸堆积、
            // 玩家狂点。改为时间预算：桌面/快机自动跑满 40 年上限，慢机按
            // ~7ms/帧 自适应收敛（每帧至少 1 年保证推进）。get_time 走既有
            // sys_now 导入，帧路径零新增 env（防"already borrowed"教训）。
            const SIM_SKIP_YEARS: u32 = 40;

            const SIM_SKIP_BUDGET_S: f64 = 0.007;

            let t0 = macroquad::time::get_time();

            let mut n = 0;

            let mut guard = 0;

            while self.sim.is_some() && n < SIM_SKIP_YEARS && guard < 20000 {
                self.sim_year();

                n += 1;

                guard += 1;

                if self.modal.is_some() {
                    self.choose_option(0);
                }

                if self.scene != Scene::Sim {
                    break;
                }

                if n % 4 == 0 && macroquad::time::get_time() - t0 > SIM_SKIP_BUDGET_S {
                    break;
                }
            }

            return;
        }

        if self.speed == 0 {
            return;
        }

        self.acc += dt;

        let t0 = macroquad::time::get_time();

        let mut guard = 0;

        while self.acc >= interval && guard < 60 {
            self.acc -= interval;

            self.sim_year();

            guard += 1;

            if self.sim.is_none() || self.modal.is_some() {
                break;
            }

            // v1.15.1：wedge 恢复期 dt 巨大时 acc 一次性堆出几十年的追帧雪崩，
            // 同样按 ~7ms 预算截断（剩余 acc 下帧继续，不丢推进）。
            if macroquad::time::get_time() - t0 > 0.007 {
                break;
            }
        }
    }

    fn sim_year(&mut self) {
        if self.sim.is_none() {
            return;
        }

        let rate = {
            let s = self.sim.as_ref().unwrap();

            cult_rate(s, &self.meta)
        };

        // 蓄势推进（模拟抉择）
        if let Some((target, acc)) = self.sim.as_ref().unwrap().pending_break.clone() {
            let na = (acc + 0.08).min(0.24);
            {
                let s = self.sim.as_mut().unwrap();
                s.pending_break = Some((target, na));
            }
            if na >= 0.24 {
                self.slog("蓄势圆满，气机鼎盛——就在今日，破境！", C_GOLD);
                self.sim_do_break(target, na);
                self.sim_year_tail();
                return;
            }
            self.slog(
                format!("你蛰伏蓄势，气机渐盈（破境率+{:.0}%）。", na * 100.0),
                C_DIM,
            );
            self.sim_year_tail();
            return;
        }
        // 道侣赠礼
        let comp_gift =
            self.sim.as_ref().unwrap().companion.is_some() && gen_range(0.0, 1.0) < 0.08;
        if comp_gift {
            let realm = self.sim.as_ref().unwrap().realm;
            let g = (60.0 * self.stone_scale(realm)) as i64;
            let cn = {
                let s = self.sim.as_mut().unwrap();
                s.stones += g;
                // v1.6.0 D：赠礼事件积攒道侣好感
                s.comp_fav = (s.comp_fav + COMPANION_GIFT_FAV).min(NPC_FAV_MAX);
                s.companion.clone().unwrap()
            };
            self.slog(
                format!(
                    "{}为你炼制了一炉养神丹，并附赠灵石 {}。（好感+{}）",
                    cn, fmt_int(g), COMPANION_GIFT_FAV
                ),
                C_GREEN,
            );
        }
        // 自动服丹
        let pill_log = {
            let s = self.sim.as_mut().unwrap();
            let need = qi_need(s);

            let boost = teff(s).pill_mult.max(1.0);

            let mut logs = Vec::new();

            if s.pills[4] > 0 && s.root < 100.0 {
                s.pills[4] -= 1;

                s.root = (s.root + 2.0).min(100.0);

                s.washed += 1;

                logs.push(("服下洗髓丹，灵根+2。".to_string(), C_GOLD));
            }

            if s.injured > 0 && s.pills[3] > 0 {
                s.pills[3] -= 1;

                s.injured = 0;

                s.lifespan += 15.0;

                logs.push(("服下培元丹，伤势尽复，寿元+15。".to_string(), C_GREEN));
            }

            if s.pills[0] > 0 && s.qi < need {
                s.pills[0] -= 1;

                gain_qi(s, need * 0.06 * boost);

                logs.push(("服下聚气丹，修为大进。".to_string(), C_CYAN));
            }

            logs
        };

        for (t, c) in pill_log {
            self.slog(t, c);
        }

        let mut demon = false;

        let mut act: u8 = 0; // 0修炼 1游历 2闭关完成

        // v1.16.21 闭关摘要（会话态记账）：块内累计本载实际修为增量（gain_qi 受
        // 需求上限截断，快照求差才诚实），块外写回 secl_qi——消灭 3-6 载闭关
        // 「静默空心感」，出关行带上总账
        let mut secl_gain = 0.0f64;

        {
            let s = self.sim.as_mut().unwrap();

            s.age += 1.0;

            if s.injured > 0 {
                s.injured -= 1;
            }

            if s.sect {
                s.stones += (30 * (1 + s.realm as i64)).max(30);
            }

            let need = qi_need(s);

            let qi_full = s.qi >= need;

            if let Some(n) = &mut s.seclusion.clone() {
                let q0 = s.qi;

                gain_qi(s, rate * 3.5);

                secl_gain = s.qi - q0;

                // v1.16.21 复审修（既有 bug，git show HEAD 实证 v1.16.20 即如此）：
                // 原序「s.seclusion = Some(*n) 先写回、*n -= 1 后自减」把减前值写
                // 回——克隆体上的自减永不落账，seclusion 恒不归零，act=2 正常出关
                // 成死代码（闭关只能靠 5%/载 心魔败臂「被迫出关」打断，期望 ~44 载，
                // 与开卡「闭关N载」叙事严重背离，出关摘要也随之永不触发）。改为先
                // 自减再写回，闭关按宣告年数如实出关
                *n -= 1;

                s.seclusion = Some(*n);

                if gen_range(0.0, 1.0) < 0.05 {
                    demon = true;
                }

                if *n == 0 {
                    s.seclusion = None;

                    act = 2;
                }
            } else if qi_full {
                act = 3; // 尝试突破（块外处理）
            } else {
                let roll = gen_range(0.0, 1.0);

                let e = teff(s);

                let event_p = 0.13 * e.event_mult.max(1.0);

                if roll < event_p {
                    act = 1; // 游历
                } else if roll < event_p + 0.06 && s.realm >= 2 {
                    let yrs = gen_range(3, 6);

                    s.seclusion = Some(yrs);

                    self.secl_yrs = yrs as u8; // 会话态：摘要记账起点（铁律③不入档）

                    self.secl_qi = 0.0;

                    self.slog(format!("你寻一处幽谷闭关{}载，潜心苦修。", yrs), C_DIM);
                } else {
                    gain_qi(s, rate * 2.5);
                }
            }
        }

        self.secl_qi += secl_gain;

        if demon {
            self.sim_demon();

            return;
        }

        match act {
            1 => self.sim_event(),

            2 => {
                // v1.16.21 出关摘要：带上闭关年数与实际修为总账（会话态记账，
                // 重载归零则退化为原句——secl_yrs==0 防御位）
                if self.secl_yrs > 0 {
                    self.slog(
                        format!(
                            "出关，气息渊渟岳峙。闭关{}载，共得修为{}。",
                            self.secl_yrs,
                            fmt_num(self.secl_qi)
                        ),
                        C_DIM,
                    );

                    self.secl_yrs = 0;

                    self.secl_qi = 0.0;
                } else {
                    self.slog("出关，气息渊渟岳峙。", C_DIM);
                }
            }

            3 => self.sim_break(),

            _ => {}
        }

        // 宿敌遭遇

        let neme_enc = if self.meta.taodie >= 6 { 0.0125 } else { 0.025 }; // v1.12.1 讨牒 6/6 遭遇减半

        if self.sim.is_some() && self.meta.neme.is_some() && gen_range(0.0, 1.0) < neme_enc {
            self.sim_neme();
        }

        // 寿元检查

        if self.sim.is_some() {
            let (age, span) = {
                let s = self.sim.as_ref().unwrap();

                (s.age, s.lifespan)
            };

            if age >= span {
                {
                    let s = self.sim.as_mut().unwrap();

                    s.natural_end = true;
                }

                self.sim_die("纸寿尽了。这本书比你自己先老——你含笑搁笔，一场大梦归真。");

                return;
            }

            // 杂事

            if gen_range(0.0, 1.0) < 0.08 {
                let i = gen_range(0, MINOR_LINES.len());

                self.slog(MINOR_LINES[i], C_DIM);
            }
        }
    }

    fn sim_demon(&mut self) {
        let (dao, luck) = {
            let s = self.sim.as_ref().unwrap();

            (s.dao, s.luck)
        };

        let p = 0.45 + dao / 160.0 + luck / 400.0;

        let (msg, c) = {
            let s = self.sim.as_mut().unwrap();

            if gen_range(0.0, 1.0) < p {
                s.dao = (s.dao + 3.0).min(100.0);

                gain_qi(s, qi_need(s) * 0.08);

                ("心魔来袭，你心湖澄澈，魔念如雪遇阳而化。道心+3。", C_GREEN)
            } else {
                s.dao = (s.dao - 6.0).max(0.0);

                s.injured = 5;

                s.seclusion = None;

                ("心魔骤起，走火入魔！你喷出一口鲜血，被迫出关。", C_RED)
            }
        };

        self.slog(msg, c);
    }

    fn sim_break(&mut self) {
        let (same_realm, need) = {
            let s = self.sim.as_ref().unwrap();

            (s.layer + 1 < REALMS[s.realm].layers.len(), qi_need(s))
        };

        if same_realm {
            let p = {
                let s = self.sim.as_ref().unwrap();

                (0.93 + s.dao / 600.0 + s.luck / 800.0).min(0.99)
            };

            let (ok, r, ly) = {
                let s = self.sim.as_mut().unwrap();

                s.qi -= need;

                let ok = gen_range(0.0, 1.0) < p;

                if ok {
                    s.layer += 1;
                } else {
                    s.qi = (s.qi * 0.9).max(0.0);
                }

                (ok, s.realm, s.layer)
            };

            if ok {
                self.slog(
                    format!("水到渠成，你踏入{}！气息更进一分。", REALMS[r].layers[ly]),
                    C_GREEN,
                );
            } else {
                self.slog("突破受阻，真气紊乱，调息良久方平。", C_RED);
            }
        } else {
            let target = {
                let s = self.sim.as_ref().unwrap();

                s.realm + 1
            };

            if target >= REALMS.len() {
                return;
            }

            // 抉择制：弹 modal 交玩家决定（跳过模式自动选第一项）

            let rd = &REALMS[target];

            let mut card = EventCard {
                kind: CardKind::Npc,

                title: String::new(),

                text: String::new(),

                opts: Vec::new(),

                actor: None,

                rare: false,

                stage: None,

                cat: None,

                scene: None,

                brk: true, // v1.16.21 破境抉择卡：琥珀描边+首钮 accent（天劫/破境两路同点）
            };

            if target == 10 {
                let est = {
                    let s = self.sim.as_ref().unwrap();

                    let e = teff(s);

                    let base = (0.92 - 7.0 * 0.015)
                        + s.dao / 500.0
                        + s.luck / 600.0
                        + s.body as f32 / 800.0;

                    (base + e.trib_add + if s.pills[5] > 0 { 0.10 } else { 0.0 }).clamp(0.05, 0.97)
                };

                card.title = "天劫将至".into();

                card.text = format!("你修为圆满，天劫的气息已经锁定此界——九重雷劫（均波存活 {:.0}%）。硬闯，还是备上一场？", est * 100.0);

                let mut e1 = Eff::new();

                e1.special = 13;

                e1.good = false;

                card.opts.push(("立即问笔！".into(), e1));

                let mut e2 = Eff::new();

                e2.special = 14;

                e2.msg = vec!["你转入闭关，默运玄功迎劫（每载存活+4%，三载后自动问笔）".into()];

                card.opts.push(("闭关准备".into(), e2));
            } else {
                let p = {
                    let s = self.sim.as_ref().unwrap();

                    big_break_p(s, target)
                };

                // v1.16.12 仙境叙事分化（方向 A1）：target 11..=14（砚尊/秉笔/执卷/
                // 天笔）破境卡特化文案——通用「X境之门」的「身死之危」对 immortal 境
                //（bdeath=0.0，且 sim_do_break 失败仅 !rd.immortal 才 roll 死）语境
                // 不实，终局四境叙事也由此不再平线。选项复用 special 15/16
                //（apply_eff 读 pending_break/realm+1，target 无硬编码），机制零改动。
                let (title, text) = if target >= 11 {
                    let scene = match target {
                        11 => "砚池深处无风自皱，墨色里现出一段石阶，阶上墨痕犹润",
                        12 => "案头秃笔无风自立，笔锋所指书页哗哗自翻，停在空白一页",
                        13 => "书山最高处那卷无字之书自己摊开，纸页里透出暖光",
                        _ => "笔冢之上虚悬一支巨笔，笔尖垂落至你面前，墨滴欲坠",
                    };
                    (
                        format!("{}之阶", rd.name),
                        format!(
                            "你修为圆满，{}——{}之阶已现眼前。冲击成功率 {:.0}%；仙境无死劫，失败唯修为折损、身负重伤。是拾级而上，还是蛰伏蓄势？",
                            scene, rd.name, p * 100.0
                        ),
                    )
                } else {
                    (
                        format!("{}境之门", rd.name),
                        format!(
                            "你修为圆满，{}境之门隐现眼前。冲击成功率 {:.0}%，失败有 {:.0}% 身死之危。是孤注一掷，还是蛰伏蓄势？",
                            rd.name,
                            p * 100.0,
                            rd.bdeath * 100.0
                        ),
                    )
                };

                card.title = title;

                card.text = text;

                let mut e1 = Eff::new();

                e1.special = 15;

                e1.good = false;

                card.opts.push(("立即破境！".into(), e1));

                let mut e2 = Eff::new();

                e2.special = 16;

                e2.msg = vec!["你决定蛰伏蓄势（每载成功率+8%，至多+24%后自动破境）".into()];

                card.opts.push(("继续蓄势".into(), e2));
            }

            // 剧情演出包①：破境卡与普通事件卡同款入场动效（预热同 build_event 先例）
            crate::ui::prewarm_text(&card.title, &[24]);
            crate::ui::prewarm_text(&card.text, &[16]);
            self.modal_anim = 0.18;
            self.modal = Some(card);
        }
    }

    pub fn sim_do_break(&mut self, target: usize, accum: f32) {
        let need = {
            let s = self.sim.as_ref().unwrap();

            s.qi.min(qi_need(s))
        };

        let rd = &REALMS[target];

        if target == 10 {
            let s = self.sim.as_mut().unwrap();

            s.qi -= need;

            let e = teff(s);

            let mut b = e.trib_add + s.body as f32 / 800.0 + accum;

            if s.pills[5] > 0 {
                s.pills[5] -= 1;

                b += 0.10;
            }

            s.trib = Some(Trib {
                wave: 0,

                timer: 0.8,

                flash: 0.0,

                spare_used: false,

                bonus: b,
            });

            self.slog("功行圆满，天劫轰然而至！九重雷劫——", C_GOLD);

            self.play(crate::sounds::Which::Thunder);

            self.audio.swap_bgm(true);

            return;
        }

        let p = {
            let s = self.sim.as_ref().unwrap();

            (big_break_p(s, target) + accum).min(0.97)
        };

        enum BR {
            Ok,

            Fail,

            Dead,
        }

        // v1.14.0 立绘焕新：跨相判定须在 realm 赋值前取旧档快照
        let old_tier = self.sim.as_ref().map(|s| portrait_tier(s.realm));

        let r = {
            let s = self.sim.as_mut().unwrap();

            s.qi -= need;

            if s.pills[1] > 0 && target == 2 {
                s.pills[1] -= 1;
            }

            if gen_range(0.0, 1.0) < p {
                s.realm = target;

                s.layer = 0;

                s.lifespan = s.lifespan.max(rd.life);

                s.pending_break = None;

                BR::Ok
            } else {
                s.qi = (s.qi * 0.7).max(0.0);

                s.injured = 6;

                s.pending_break = None;

                if !rd.immortal && gen_range(0.0, 1.0) < rd.bdeath {
                    BR::Dead
                } else {
                    BR::Fail
                }
            }
        };

        match r {
            BR::Ok => {
                let first = {
                    let s = self.sim.as_mut().unwrap();

                    let mut f = false;

                    if target > s.max_realm {
                        s.max_realm = target;

                        f = true;
                    }

                    s.highlights
                        .push(format!("{}岁，突破【{}境】", s.age as i32, rd.name));

                    f
                };

                self.slog(
                    format!("轰！突破【{}境】！寿元大增，气冲霄汉！", rd.name),
                    C_GOLD,
                );

                if first {
                    self.slog("【首登】此身首次踏入此境！", C_GOLD);

                    self.show_banner(format!("突破 · {}境", rd.name), C_GOLD);
                }

                if accum > 0.0 {
                    self.check_ach(24);
                }

                self.burst(50, true);

                self.play(crate::sounds::Which::Ascend);

                // v1.14.0 立绘焕新时刻：跨相破境 toast + 金环闪光（置于成就 toast 之后，
                // toast 单槽后写胜——焕然词不被同帧成就播报顶掉）
                if let Some(ot) = old_tier {
                    self.stage_awaken(ot, target);
                }
            }

            BR::Fail => {
                self.slog(
                    format!("冲击{}境失败！真元反噬，身受重伤。", rd.name),
                    C_RED,
                );

                self.play(crate::sounds::Which::BreakFail);
            }

            BR::Dead => {
                self.sim_die(&format!("冲击{}境失败，走火入魔，经脉俱断而亡。", rd.name));
            }
        }
    }

    // 模拟年尾：寿元与杂事（提前 return 的路径共用）

    pub fn sim_year_tail(&mut self) {
        if self.sim.is_none() {
            return;
        }

        let (age, span) = {
            let s = self.sim.as_ref().unwrap();

            (s.age, s.lifespan)
        };

        if age >= span {
            {
                let s = self.sim.as_mut().unwrap();

                s.natural_end = true;
            }

            self.sim_die("纸寿尽了。这本书比你自己先老——你含笑搁笔，一场大梦归真。");

            return;
        }

        if gen_range(0.0, 1.0) < 0.08 {
            let i = gen_range(0, MINOR_LINES.len());

            self.slog(MINOR_LINES[i], C_DIM);
        }
    }

    fn sim_neme(&mut self) {
        // v1.16.21 宿敌遭遇卡化：原单 roll 无决策（打不过臂 40% 即死）改 EventCard
        // 二选——首项=迎战（跳过档 choose_option(0) 自动语义保「推进」向，2625/2663
        // 契约），次项=拼死遁走（确定性脱身：重伤+行囊失二成，无死）。死亡从此只属
        // 于明知胜算的抉择（卡面正文言明胜算/悬殊）。modal 占用帧本载不弹（防吞
        // 事件卡/破境卡）；链直取/K 窗/recent 口径零接触（本卡不走 build_event 管线）
        if self.modal.is_some() {
            return;
        }

        let (my, my_realm) = {
            let s = self.sim.as_ref().unwrap();

            (power(s), s.realm)
        };

        let (nfull, np, nrealm) = {
            let n = self.meta.neme.as_ref().unwrap();

            (n.full(), n.power(), n.realm)
        };

        let overmatched = my_realm + 1 < nrealm || my < np * 0.5;

        // 决战式与原 sim_neme/apply_eff 迎战臂同源（clamp 语义照 3486 勿联动改）
        let r = my / np.max(1.0);

        let p = (r * r / (1.0 + r * r)).clamp(0.05, 0.95);

        let mut card = EventCard {
            kind: CardKind::Npc,

            title: String::new(),

            text: String::new(),

            opts: Vec::new(),

            actor: None,

            rare: false,

            stage: None,

            cat: None,

            scene: None,

            brk: false,
        };

        card.title = "宿敌截道".into();

        card.text = if overmatched {
            format!(
                "荒野窄径，{}（{}境）当道而立，敌势悬殊——硬战凶多吉少，胜算约{:.0}%。迎战了断，还是弃囊遁走？",
                nfull,
                nrealm,
                p * 100.0
            )
        } else {
            format!(
                "{}（{}境）再现身前！此战胜算约{:.0}%——迎战了断，还是退避三舍？",
                nfull,
                nrealm,
                p * 100.0
            )
        };

        let mut e1 = Eff::new();

        e1.special = 67; // 迎战（统一决战式，悬殊即明码赌命）

        e1.good = false;

        card.opts.push(("迎战！".into(), e1));

        let mut e2 = Eff::new();

        e2.special = 68; // 拼死遁走（确定性脱身）

        e2.good = false;

        card.opts.push(("拼死遁走".into(), e2));

        crate::ui::prewarm_text(&card.title, &[24]);

        crate::ui::prewarm_text(&card.text, &[16]);

        self.modal_anim = 0.18;

        self.modal = Some(card);
    }

    fn spawn_next_neme(&mut self) {
        // v1.12.1 文狱讨牒：每斩一吏即请其职牒（slain 已由两入口先行 +1，此处对账补发）
        self.sync_taodie();

        let lv = self.meta.neme_slain as usize;

        let title = NEME_TITLES[(lv).min(NEME_TITLES.len() - 1)];

        let realm = (3 + lv * 2).min(14);

        self.meta.neme = Some(Nemes {
            title,

            name: rand_name(),

            realm,
        });

        let nm = self.meta.neme.as_ref().unwrap().full();

        self.log(
            format!("【文狱更换】{} 已按册候查——删你的字，从不假手他人。", nm),
            C_RED,
        );

        // v1.6.0 山河万象：斩敌换幕（现实寻仇与模拟反杀共用此口）
        self.advance_map();
    }

    // ================= v1.12.1 文狱讨牒：六吏讨伐链终局收束 =================
    // 讨牒对账单漏斗（共用 spawn_next_neme 之口）：while 补发幂等，旧档 neme_slain>0
    // 首检自动补齐零迁移；第 6 牒封卷牒=大额道韵+遭遇率减半；taodie>=6 可赴问笔终战
    fn sync_taodie(&mut self) {
        while self.meta.taodie < self.meta.neme_slain.min(6) as u8 {
            self.meta.taodie += 1;

            let title = NEME_TITLES[(self.meta.taodie - 1) as usize];

            if self.meta.taodie < 6 {
                self.log(
                    format!("文狱讨牒 {}/6：{}职牒归档——删你字的手，先一步被你合上。", self.meta.taodie, title),
                    C_CYAN,
                );
            } else {
                self.meta.dao_yun += 166;
                self.log(
                    "封卷牒归档，讨牒 6/6！六吏职牒齐聚，道韵+166——文狱闻你名而却步（遭遇减半），可赴问笔终战。",
                    C_GOLD,
                );
                // v1.16.7 大事记联动：模拟世中封卷牒归档入本世剧情（现实侧 sim=None
                // 不记——story 数据源是 sim highlights）
                if let Some(s) = self.sim.as_mut() {
                    s.highlights
                        .push(format!("{}岁，讨牒6/6·六牒归档", s.age as i32));
                }
            }
        }
        // v1.16.8 章节书：讨牒事实翻转点（第四章边沿；while 补发可一帧多牒，
        // 边沿检测按位掩码幂等，多章同了时横幅后写胜、日志逐章留痕）
        self.check_chapters();
    }

    // ================= v1.12.1 同页人·三生缝页 =================
    // 重逢判定：此番结缘的正是缝页人、且先前并非同页
    fn feng_reunion(&self, prev: Option<&str>, cn: &str) -> bool {
        self.meta.feng_bn != crate::data::FENG_UNSET
            && cn == COMPANION_NAMES[self.meta.feng_bn as usize]
            && prev != Some(cn)
    }

    // 缘深单口：一世同页至终缘+1（三生封顶；缘三纯叙事）——sim_die/real_die 共用
    fn feng_bond_up(&mut self, cn: Option<&str>) {
        if let Some(cn) = cn {
            if self.meta.feng_bn != crate::data::FENG_UNSET
                && cn == COMPANION_NAMES[self.meta.feng_bn as usize]
            {
                self.meta.feng_blv = (self.meta.feng_blv + 1).min(3);
            }
        }
    }

    // v1.6.0 仇敌突破换地图：region+1、种子 LCG 演化、人归清源洞府、横幅宣告
    fn advance_map(&mut self) {
        self.meta.map_region = self.meta.map_region.wrapping_add(1);
        self.meta.map_seed = self
            .meta
            .map_seed
            .wrapping_mul(1664525)
            .wrapping_add(1013904223);
        if let Some(l) = self.life.as_mut() {
            l.loc = NODE_DONGFU;
        }
        let t = theme_of(self.meta.map_region);
        self.log(
            format!("山河变色——你携新胜之势踏入{}。{}", t.name, t.desc),
            C_GOLD,
        );
        self.show_banner(&format!("山 河 换 幕 · {}", t.name), C_GOLD);
        // v1.8.0 C5：换幕即踏入新境——置位到访并发放初至奖励。
        //（advance_map 为现实寻仇与模拟反杀共用口：模拟反杀亦解锁新境、奖励入现实行囊，
        //  toast 弹于模拟界面——两路径语义一致，不加场景分支）
        self.visit_region(true);
    }

    // v1.8.0 C5：区域到访记录——首次踏入某主题区置位；grant=true 发放初至奖励。
    // 索引取模写死：map_region 为 wrapping u32、存档允许任意值（parse 不 clamp），
    // 与 region_mech（data.rs）同款口径。
    pub fn visit_region(&mut self, grant: bool) {
        let t = (self.meta.map_region as usize) % REGION_MECH.len();
        if self.meta.region_visited[t] {
            return;
        }
        self.meta.region_visited[t] = true;
        let tname = theme_of(self.meta.map_region).name;
        if grant {
            let (dy, stones) = REGION_VISIT_REWARD[t];
            self.meta.dao_yun += dy;
            {
                if let Some(l) = self.life.as_mut() {
                    l.stones += stones;
                }
            }
            self.log(
                format!(
                    "初至【{}】，山河图志录下你的足迹：道韵+{}，灵石+{}。",
                    tname,
                    fmt_int(dy),
                    fmt_int(stones)
                ),
                C_GOLD,
            );
            self.toast(format!("初至 {} · 图鉴有赏", tname), C_GOLD);
            self.play(crate::sounds::Which::Coin);
        } else {
            self.log(format!("初至【{}】，山河图志录下你的足迹。", tname), C_GOLD);
        }
        // v1.16.8 章节书：五域到访事实翻转点（第五章边沿）
        self.check_chapters();
    }

    pub fn sim_event(&mut self) {

        // v1.16.7 连锁剧情：链活跃 → 下段已定则跳过轮盘直取下段（链内段在 EVENTS
        // 表权重 0 恒不入正常池）。直取即消费下段（置 0，防本段卡臂异常时死循环
        // 重复出卡）；照常入 K 窗口，先 retain 去重防「同世复走同链」在窗口内叠出
        // 重复 id。ttl 是保险丝（耗尽/链态异常 → 静默弃链，落正常轮盘）
        if let Some((cid_chain, next, ttl)) = self.chain {
            if next == 0 || ttl <= 1 {
                // 首段卡在手待分支（modal 挡模拟，理论不可达）/ 保险丝耗尽：弃链
                self.chain = None;
            } else {
                let cid = next as usize;

                self.chain = Some((cid_chain, 0, ttl - 1)); // 消费下段：防本段臂异常时死循环直取

                self.recent_events.retain(|&x| x != cid);

                self.recent_events.push(cid);

                if self.recent_events.len() > RECENT_EVENT_K {
                    self.recent_events.remove(0);
                }

                self.build_event(cid);

                if self.modal.is_none() {
                    self.chain = None; // 防御：链段缺卡臂（忘写 build_event）→ 弃链防死循环
                }

                return;
            }
        }

        let (luck, realm, sect, lingmai, comp) = {
            let s = self.sim.as_ref().unwrap();

            let lk = s.luck;

            (lk, s.realm, s.sect, s.lingmai, s.companion.is_some())
        };

        let (ws, total) = self.sim_event_weights(luck, realm, sect, lingmai, comp);

        // v1.16.5 退化兜底：排除池把小池清零（低境界常见）→ 清空排除池重算，
        // 宁可重复也不静默无卡（旧逻辑 total<=0.01 直接 return）
        let (ws, total) = if total <= 0.01 {
            self.recent_events.clear();

            let (ws, total) = self.sim_event_weights(luck, realm, sect, lingmai, comp);

            if total <= 0.01 {
                return;
            }

            (ws, total)
        } else {
            (ws, total)
        };

        let mut roll = gen_range(0.0, total);

        let mut chosen = 0;

        for (i, w) in ws.iter().enumerate() {
            roll -= w;

            if roll <= 0.0 {
                chosen = i;

                break;
            }
        }

        // v1.16.5 记入防重复排除池并裁剪至 K（仅事件年调用一次，非帧路径）
        let cid = EVENTS[chosen].0;

        self.recent_events.push(cid);

        if self.recent_events.len() > RECENT_EVENT_K {
            self.recent_events.remove(0);
        }

        self.build_event(cid);
    }

    // v1.16.5 抽取权重一行一算（sim_event 退化重算复用；仅事件年调用，非帧路径）
    // v1.16.7 增 comp：道侣同行事件白名单门控（sim.companion 存在才入池）
    fn sim_event_weights(
        &self,
        luck: f32,
        realm: usize,
        sect: bool,
        lingmai: bool,
        comp: bool,
    ) -> (Vec<f32>, f32) {
        let mut total = 0.0f32;

        let mut ws: Vec<f32> = Vec::new();

        for &(id, w, mr, good) in EVENTS.iter() {
            let mut ww = w as f32;

            // v1.12.0 空白行：每稿至多一次——门闩置位后权重归零（照 sect/lingmai 先例）
            if (id == 6 && sect) || (id == 9 && lingmai) || (id == 30 && self.blank_used) {
                ww = 0.0;
            }

            // v1.16.7 道侣同行事件：无伴不入池（白名单门控，与 sect/lingmai 同点）
            if !comp && COMPANION_EVENTS.contains(&id) {
                ww = 0.0;
            }

            // v1.16.8 章节联动事件：主线章节达成（施工单1章节书系统 chap_done 位掩码）
            // 才入池——模拟侧的章节涟漪，照 sect/lingmai/blank_used 同点结算：
            // 74 血仇余响 = 第三章·血仇得报已了（bit2）；75 同页灯约 = 第六章·
            // 同页之缘已了（bit5）且有伴（宿命双门控——无伴时该题不可述）。
            // v1.16.17 章节联动 +2（一/四章首获覆盖，照 74/75 同一 if 勿拆两段）：
            // 190 雨夜寻印 = 第一章·雨夜执印已了（bit0）；191 文狱退牒 =
            // 第四章·文狱讨牒已了（bit3）
            // v1.16.18 扩池章节联动 +2（补齐 bit1/bit4——至此联动缺角仅 bit6 七章，
            // 与结局回响题材重叠按勘察口径不补；仍同一 if 勿拆）：
            // 196 镜影忆战 = 第二章·镜中万象已了（bit1）；197 五域风谣 =
            // 第五章·山河万象已了（bit4）
            // v1.16.19 章节联动 +1（217 邻人扛劫 = 第七章·问笔终战已了 bit6——
            // 联动缺角补齐；题材「邻人扛劫」侧写守链11 分域纪律：只看旁人扛劫，
            // 绝不触玩家自身破境裁决。仍同一 if 勿拆）
            // v1.16.20 章节联动 +1（230 落款撞名 = 第八章·落款成书已了 bit7——
            // 联动最后缺角补齐；与结局回响 165-167「世界侧写」分域写亲历视角。
            // 仍同一 if 勿拆）
            if (id == 74 && self.meta.chap_done & (1 << 2) == 0)
                || (id == 75 && (self.meta.chap_done & (1 << 5) == 0 || !comp))
                || (id == 190 && self.meta.chap_done & (1 << 0) == 0)
                || (id == 191 && self.meta.chap_done & (1 << 3) == 0)
                || (id == 196 && self.meta.chap_done & (1 << 1) == 0)
                || (id == 197 && self.meta.chap_done & (1 << 4) == 0)
                || (id == 217 && self.meta.chap_done & (1 << 6) == 0)
                || (id == 230 && self.meta.chap_done & (1 << 7) == 0)
            {
                ww = 0.0;
            }

            // v1.16.9 宿敌剧情化（链6 首段 85 仇踪入册）：有宿敌才入池——血仇线
            // 无宿敌不可述（照 74/75 门控同点；85 臂文案读 meta.neme 已自兜底）。
            // 只读 meta.neme 既有字段，零新增存档键
            if id == 85 && self.meta.neme.is_none() {
                ww = 0.0;
            }

            // v1.16.10 宿敌回响 2 条（照 85 门控同点，措辞按「当次入池判定」纪律）：
            // 110 夺积余波 = 链6 本世已缘了（会话态 chain6_closed，choose_option 置位
            // launch_sim 清零）——「夺积之后倾巢寻账」只属同一推演；111 仇册新墨 =
            // meta.neme_slain≥1（斩过宿敌的余波，读既有持久计数器零新键）
            if id == 110 && !self.chain6_closed {
                ww = 0.0;
            }
            if id == 111 && self.meta.neme_slain < 1 {
                ww = 0.0;
            }

            // v1.16.11 链9 首段 118 血册投书（宿敌终局对决线）：斩过宿敌才入池——
            // 「门下循册寻仇」只属斩宿敌之后（照 111 读持久计数器同点，零新键）。
            // 正文不具名：被斩者已不在册（meta.neme 是新宿敌），泛称兜底不误指
            if id == 118 && self.meta.neme_slain < 1 {
                ww = 0.0;
            }
            // v1.16.11 链10 首段 121 旧页回声（跨世情缘回访）：历世结缘过同页人才
            // 入池（comp_bonds 任意 >0，照 85/111 读既有字段零新键口径——跨世旧页
            // 无旧缘不可述）
            if id == 121 && !self.meta.comp_bonds.iter().any(|&b| b > 0) {
                ww = 0.0;
            }

            // v1.16.9 老年/将尽专属（88 灯下了愿 / 89 薪火相托）：寿元余量
            // ≤ ELDER_LEFT_YEARS 才入池——池内原零覆盖的终章题材。读 self.sim
            // 既有字段（照 blank_used/chap_done 读 self 态先例）；sim=None 帧
            // （weights 只被 sim_event 调用、sim 恒 Some）防御归零。
            // v1.16.16 老年层 +2（170 炉暖旧器/171 灯影授业，「交托回望」散件）
            // ——老年层从两条独占扩到四条，缓「每世凡人局必经层的重见率」
            // v1.16.17 老年层 +1（189 旧履还坊）——四条扩到五条
            // v1.16.18 扩池老年层 +2（198 窗前晒卷/199 旧邻问名）——五条扩到
            // 七条；老年层原 5 条不挂 chain_latch、跨世重见率最高，本批再摊薄
            // v1.16.19 老年层 +2（215 灶火留谱/216 旧约转托，「交托回望」题材
            // ——七条扩到九条，门控同批扩 id 集合）
            // v1.16.20 扩池老年层 +1（227 半檐躲雨）——九条扩到十条；老年层不挂
            // chain_latch、跨世重见率最高层，边际收益递减故仅 +1
            if (id == 88
                || id == 89
                || id == 170
                || id == 171
                || id == 189
                || id == 198
                || id == 199
                || id == 215
                || id == 216
                || id == 227)
                && !self
                    .sim
                    .as_ref()
                    .map_or(false, |s| s.lifespan - s.age <= ELDER_LEFT_YEARS)
            {
                ww = 0.0;
            }

            // v1.16.18 扩池：老年×有伴双门控首条（204 同席说岁——新门控形态，
            // 单 if 表达「余量≤10 载 AND sim.companion 存在」的交集；两条件任一
            // 不满足即归零，读 sim 既有字段与 comp 参数零新键，照 75 宿命双门控
            // 同点结算。措辞按「当次入池判定」纪律：弹卡后寿元变化不撤卡）
            // v1.16.19 双门控第二条（212 相扶看雪，照 204 模板同 if 并臂）
            // v1.16.21 双门控第三条（239 共守岁灯，同 if 并臂勿拆；不入 COMPANION
            // 白名单——徽标走老年臂，见 gallery event_badge 212 死模式注释）
            if (id == 204 || id == 212 || id == 239)
                && !(comp
                    && self
                        .sim
                        .as_ref()
                        .map_or(false, |s| s.lifespan - s.age <= ELDER_LEFT_YEARS))
            {
                ww = 0.0;
            }

            // v1.16.16 链缘了闩：本世已缘了的链，其首段不再入池（CHAIN_FIRSTS
            // 查表 id→链 id，bit 位读 chain_latch；chain6_closed 会话态先例的
            // 全线推广——长寿世同链复走封堵；中段弃链不置位，同世仍可重走）
            if let Some(&(_, cid)) = CHAIN_FIRSTS.iter().find(|&&(eid, _)| eid == id) {
                if self.chain_latch >> (cid as u32 - 1) & 1 == 1 {
                    ww = 0.0;
                }
            }

            // v1.16.12 洞府经营联动 3 条（134 阵眼共鸣/135 田畦岁稔/136 丹房遗香）：
            // 名下对应设施满级（dongfu 0..=3 级，跨世持久）才入池——经营玩法与
            // 叙事池首次联动，照 111 读持久态同点，零新键。clamp 照既有使用点
            // 口径（7612/1864/8777）；卡面措辞按跨世纪律：「你名下洞府/历年经营」
            if (id == 134 && self.meta.dongfu[0].clamp(0, 3) < 3)
                || (id == 135 && self.meta.dongfu[1].clamp(0, 3) < 3)
                || (id == 136 && self.meta.dongfu[2].clamp(0, 3) < 3)
            {
                ww = 0.0;
            }

            // v1.16.14 宗门换届 2 条（154 新掌接印/155 旧掌传灯）：入宗门（sim.sect，
            // 会话）且行走东境青云（map_region%5==0 唯 0 号域「东境青云·宗门兴望」；
            // map_region 为 wrapping u32，%5 口径照 region_mech/region_visited 同款）
            // 才入池——换届故事只属宗门与宗门地界；读既有会话/持久字段零新键。
            // sect 参数即 s.sect（sim_event 自 sim 解元组传入，宗门相邀 e.sect 置位）
            if (id == 154 || id == 155) && !(sect && self.meta.map_region % 5 == 0) {
                ww = 0.0;
            }

            // v1.16.16 结局回响 3 条（165 定稿之世/166 留白之野/167 执笔之赠）：
            // 对应结局已落款（ed= bit0/1/2 持久位）才入池——落款之后世界才有回响，
            // 照 111 读持久位同点零新键；合订（bit3）为隐藏卷余韵，不另占池。
            // 措辞按跨世纪律：三卡皆世界侧写视角，不指现世资源
            if (id == 165 && self.meta.endings_seen & 0b0001 == 0)
                || (id == 166 && self.meta.endings_seen & 0b0010 == 0)
                || (id == 167 && self.meta.endings_seen & 0b0100 == 0)
            {
                ww = 0.0;
            }

            // v1.16.16 书外之章联动 2 条（168 天曹补写/169 书肆新声）：第九章·
            // 书外之章已了（ch= bit8）才入池——书成后天曹/书肆的余波，照 74/75
            // 章节联动门控同点（只读持久掩码零新键）
            if (id == 168 || id == 169) && self.meta.chap_done & (1 << 8) == 0 {
                ww = 0.0;
            }

            // v1.16.5 防重复：本世最近 K 次抽过 → 权重归零（全灭由 sim_event 退化兜底）
            if self.recent_events.contains(&id) {
                ww = 0.0;
            }

            if good {
                ww *= (1.0 + (luck - 40.0) / 120.0).max(0.4);
                // v1.8.0 C2：区域机制「仙屿潮音」（模拟善缘事件权重；
                // 镜像世界与现世同循山河机制——同一 map_region 口径）
                ww *= region_mech(self.meta.map_region).event_mult as f32;
            } else {
                ww *= (1.0 - (luck - 40.0) / 160.0).max(0.4);
            }

            if realm < mr {
                ww = 0.0;
            }

            ws.push(ww);

            total += ww;
        }

        (ws, total)
    }

    fn build_event(&mut self, id: usize) {
        let ss = {
            let s = self.sim.as_ref().unwrap();

            self.stone_scale(s.realm)
        };

        let mut card = EventCard {
            kind: CardKind::Npc,

            title: String::new(),

            text: String::new(),

            opts: Vec::new(),

            actor: None,

            rare: false,

            stage: None,

            cat: None,

            scene: None,

            brk: false,
        };

        match id {
            28 => {
                card.title = "同页天成".into();
                let ci = gen_range(0, COMPANION_NAMES.len());
                let cn = COMPANION_NAMES[ci];
                // v1.16.3 邂逅卡立绘：卡面主角即递稿之人（卡面文本与立绘同源同名；
                // 认下后的掷名/缝页人重逢逻辑在 special 17，与卡面立绘无涉）
                card.actor = Some(CardActor::Comp(ci));
                card.text = format!(
                    "晒书日，{}于书摊间递来半页残稿：\"两页相贴，缝作同页——可愿？\"",
                    cn
                );
                let mut e1 = Eff::new();
                e1.special = 17;
                e1.msg = vec![format!(
                    "你与{}缝作同页！修炼速度+20%，心魔有抚，或得赠礼。",
                    cn
                )];
                card.opts.push(("认下这页".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 5.0;
                e2.msg = vec!["你婉拒道：\"一页未成，何以同页。\"道心+5".into()];
                card.opts.push(("专注于道".into(), e2));
            }
            29 => {
                card.title = "宗门大比".into();
                card.text = "十年一度的宗门大比开擂，掌门放话：魁首可入藏经阁任拣一卷功法！".into();
                let mut e1 = Eff::new();
                e1.fight = 1.1;
                e1.special = 18;
                e1.good = false;
                card.opts.push(("登擂比武".into(), e1));
                let mut e2 = Eff::new();
                e2.qi_pct = 0.05;
                e2.msg = vec!["你于台下观摩他人过招，亦有所得。修为+5%".into()];
                card.opts.push(("台下观摩".into(), e2));
            }

            30 => {
                // v1.12.0 空白行：天曹没写的一年——每稿至多一次（sim_event 权重归零，
                // launch_sim 清零门闩）；三选走 EventCard→choose_option→apply_eff 既有链
                self.blank_used = true;

                card.title = "空白行".into();

                card.text =
                    "翻到这一页——天曹漏写了，整页空白。你握着笔，听见自己的心跳：这一年，你自己写。"
                        .into();

                let mut e1 = Eff::new();

                e1.qi_pct = 0.20;

                e1.msg = vec!["你把这页写成修行——字字入纸，修为+20%。".into()];

                card.opts.push(("留墨 · 修为+20%".into(), e1));

                let mut e2 = Eff::new();

                e2.life_y = 6.0;

                e2.msg = vec!["你把这页留给岁月——不急，一笔一画慢慢写，寿元+6。".into()];

                card.opts.push(("留纸 · 寿元+6".into(), e2));

                let mut e3 = Eff::new();

                e3.luck = 2.0;

                e3.special = 19;

                e3.msg = vec!["你在这页写下自己的一行——笔落生根，气运+2。".into()];

                card.opts.push(("留行 · 气运+2".into(), e3));
            }

            0 => {
                let g = (80.0 * ss * gen_range(0.7, 1.4)) as i64;

                card.title = "灵草满坡".into();

                // v1.16.19 前期高频卡画面化：单段「画面+一断」体（不加减段——邂逅
                // 版式 5 行判定 draw_event_text 预算不变；下同 id 1/3/4/5/6/12）
                card.text = "转过山坳，满坡野灵草开得正艳，露水还挂在叶尖。你挽袖采满一筐，心道：这一筐，够换几块灵石了。".into();

                let mut e = Eff::new();

                e.stones = g;

                e.msg = vec![format!("获得灵石 {}", fmt_int(g))];

                card.opts.push(("收下".into(), e));
            }

            1 => {
                card.title = "妖兽拦路".into();

                card.text = "林风忽静。一头妖兽自树影里扑出，獠牙滴涎——它拦的不是路，是你腰间的储物袋。".into();

                let mut e = Eff::new();

                e.fight = 0.85;

                e.good = false;

                card.opts.push(("拔剑而战".into(), e));
            }

            2 => {
                card.title = "上古洞府".into();

                // v1.16.20 前期高频卡正文画面化第二批（id 2/9/18，照前批 7 条口径：
                // 只改字不加段，单段 ≤60 字预算由单测钉死）
                card.text = "荒山过半，一方洞府禁制塌了半边——门缝里漏出的不是光，是一段没人认领的传承。".into();

                let mut e1 = Eff::new();

                e1.fight = 1.15;

                e1.special = 1;

                e1.good = false;

                card.opts.push(("探索一番".into(), e1));

                // v1.16.15 复审口径：空选项补一行日志反馈（照 17 摆手离开同款），
                // 抉择后日志区不再无痕
                let mut e0 = Eff::new();

                e0.msg = vec!["你朝洞府遥遥一揖，转身下山。".into()];

                card.opts.push(("敬而远之".into(), e0));
            }

            3 => {
                card.title = "坊市淘宝".into();

                if gen_range(0.0, 1.0) < 0.5 {
                    let g = (150.0 * ss * gen_range(0.7, 1.4)) as i64;

                    card.text = "坊市尽头的地摊上，一只积灰储物袋怎么也卖不出去。你掂了掂，解开袋口——".into();

                    let mut e = Eff::new();

                    e.stones = g;

                    e.msg = vec![format!("袋中竟有灵石 {}", fmt_int(g))];

                    card.opts.push(("大喜收下".into(), e));
                } else {
                    // v1.16.15 复审口径：购药是花销（照 140 半价购书同款）——聚气丹
                    // 铺价 200×(1+realm)^1.2，地摊价照 ss 缩放（realm0=100，约铺价
                    // 四分之一）零扣费白得是漏洞；买不起照 17 云游商人钱不够同款
                    let g = (100.0 * ss) as i64;

                    let can = self.sim.as_ref().map_or(false, |s| s.stones >= g);

                    if can {
                        card.text = "老掌柜一把拉住你，压低嗓门：「小哥，这瓶聚气丹，铺里挂双价，你拿走算半价。」".into();

                        let mut e = Eff::new();

                        e.stones = -g;

                        e.pill = Some((0, 2));

                        e.msg = vec![format!("获得聚气丹 ×2，灵石-{}", fmt_int(g))];

                        card.opts.push(("购下".into(), e));
                    } else {
                        card.text = "「小哥，半价卖了。」老掌柜把丹瓶往你面前推。你摸了摸空瘪的储物袋，摇头把瓶子推了回去。".into();

                        let mut e0 = Eff::new();

                        e0.msg = vec!["你把丹瓶放回摊上，讪讪走开。".into()];

                        card.opts.push(("讪讪离开".into(), e0));
                    }
                }
            }

            4 => {
                card.title = "恶修劫道".into();

                card.text = "「此路是我开！」黑衣修士横剑拦道，眼里贪光像淬了毒——他打量你的行囊，比打量你的剑更久。".into();

                let mut e1 = Eff::new();

                e1.fight = 1.0;

                e1.good = false;

                card.opts.push(("战而胜之".into(), e1));

                let mut e2 = Eff::new();

                e2.stones_pct = -0.15;

                e2.good = false;

                e2.msg = vec!["你破财免灾，交出一部分灵石。".into()];

                card.opts.push(("破财免灾".into(), e2));
            }

            5 => {
                card.title = "道友论道".into();

                card.text = "山亭里避雨，遇一位云游道友。两人就着一壶粗茶，从剑谈到琴，从琴谈到天光放晴——各有所得。".into();

                let mut e = Eff::new();

                e.dao = 5.0;

                e.qi_pct = 0.06;

                e.msg = vec!["道心+5，修为+6%".into()];

                card.opts.push(("尽兴而归".into(), e));
            }

            6 => {
                card.title = "宗门相邀".into();

                // v1.16.19 画面化保留括注机制口径（入门收益提示是决策依据，原样保留）
                card.text =
                    "宗门执事拦下你，端详半晌：「这身根骨，散修可惜了——可愿入我门下？」（修炼环境+30%，每年俸禄）".into();

                let mut e1 = Eff::new();

                e1.sect = true;

                e1.msg = vec!["你正式拜入宗门！".into()];

                card.opts.push(("拜入宗门".into(), e1));

                let mut e2 = Eff::new();

                e2.luck = 2.0;

                e2.msg = vec!["你婉言谢绝，逍遥自在。气运+2".into()];

                card.opts.push(("婉言谢绝".into(), e2));
            }

            7 => {
                card.title = "心魔袭扰".into();

                card.text = "静坐之间，旧日执念化作心魔，在你识海中张牙舞爪……".into();

                let mut e = Eff::new();

                e.special = 7;

                e.good = false;

                card.opts.push(("澄心镇魔".into(), e));
            }

            8 => {
                card.title = "天材地宝".into();

                card.text = "前方灵气冲霄——一株千年灵乳正在成形！旁边盘着一头护宝妖兽。".into();

                let mut e1 = Eff::new();

                e1.fight = 1.25;

                e1.special = 2;

                e1.good = false;

                card.opts.push(("搏命夺取".into(), e1));

                let mut e0 = Eff::new();

                e0.msg = vec!["你退入林中，由它静静成形。".into()];

                card.opts.push(("悄然退去".into(), e0));
            }

            9 => {
                card.title = "灵脉洞府".into();

                // v1.16.20 正文画面化第二批：括注记账体改画面+机制注（加成数值保留）
                card.text = "山坳深处泉鸣如琴，灵气浓得化不开——迁入此府，本世修炼环境+50%".into();

                let mut e = Eff::new();

                e.lingmai = true;

                e.msg = vec!["你迁入灵脉洞府！".into()];

                card.opts.push(("迁入洞府".into(), e));
            }

            10 => {
                card.title = "上古传承".into();

                card.text = "一道苍老的声音在识海中响起：\"吾道后继有人……\"".into();

                let mut e = Eff::new();

                e.qi_pct = 0.25;

                e.dao = 6.0;

                e.msg = vec!["修为+25%，道心+6".into()];

                card.opts.push(("恭聆大道".into(), e));
            }

            11 => {
                card.title = "秘境探秘".into();

                card.text = "空间裂缝中露出一座上古秘境的入口，宝光与凶险并存。".into();

                let mut e1 = Eff::new();

                e1.special = 3;

                e1.good = false;

                card.opts.push(("深入险地".into(), e1));

                let mut e2 = Eff::new();

                let g = (60.0 * ss * gen_range(0.7, 1.4)) as i64;

                e2.stones = g;

                e2.msg = vec![format!("外围拾得灵石 {}", fmt_int(g))];

                card.opts.push(("外围拾遗".into(), e2));
            }

            12 => {
                card.title = "红尘历练".into();

                card.text = "你下山入世。市集讨价、桥头让路、替哭闹的孩童变出一朵纸花——烟火气里，道心悄悄稳了一分。".into();

                let mut e = Eff::new();

                e.dao = 5.0;

                e.stones_pct = -0.05;

                e.msg = vec!["道心+5".into()];

                card.opts.push(("扶贫济困".into(), e));
            }

            13 => {
                card.title = "灵气潮汐".into();

                card.text = "天地灵气忽然暴涨，如潮水般涌来！".into();

                let mut e = Eff::new();

                e.qi_pct = 0.18;

                e.msg = vec!["修为+18%".into()];

                card.opts.push(("顺势修炼".into(), e));
            }

            14 => {
                card.title = "邪修偷袭".into();

                card.text = "一道黑影自暗中暴起偷袭！招招夺命。".into();

                let mut e = Eff::new();

                e.fight = 1.3;

                e.good = false;

                card.opts.push(("拼死一战".into(), e));
            }

            15 => {
                card.title = "仙人遗府".into();

                card.text = "轰然一声，山体崩开，露出一座流光溢彩的仙人遗府！".into();

                let rl = gen_range(0, RELICS.len());

                let mut e = Eff::new();

                e.relic = Some(rl);

                e.qi_pct = 0.25;

                e.msg = vec![format!("获得法宝【{}】，修为+25%", RELICS[rl].name)];

                card.opts.push(("入府参悟".into(), e));
            }

            16 => {
                card.title = "妖丹入手".into();

                card.text = "你路遇两妖相争两败俱伤，捡漏一枚温润妖丹。".into();

                let mut e = Eff::new();

                e.pill = Some((0, 3));

                e.msg = vec!["得聚气丹 ×3".into()];

                card.opts.push(("收入囊中".into(), e));
            }

            17 => {
                card.title = "云游商人".into();

                // v1.16.15 复审口径：洗髓丹是永久灵根+2 的重价丹（铺底价
                // PILL_PRICES[4]=5000），800 平价不随境界与经济规模脱节——照铺价
                // 指数 (1+realm)^1.2 随境界上调（realm0 维持 800 不变，realm9≈
                // 12680，仍远低于同期铺价，是行脚商让利价）。正文带实价，can 门
                // 用同一价格防「门过了却扣成负」（apply_eff 已 max(0) 兜底）
                let realm = self.sim.as_ref().map_or(0, |s| s.realm);

                let price = (800.0 * (1.0 + realm as f64).powf(1.2)).ceil() as i64;

                let mut e = Eff::new();

                e.pill = Some((4, 1));

                e.stones = -price;

                e.msg = vec![format!("获得洗髓丹 ×1，灵石-{}", fmt_int(price))];

                let can = self.sim.as_ref().map_or(false, |s| s.stones >= price);

                if can {
                    card.text = format!(
                        "蒙面商人：\"洗髓丹一枚，{} 灵石，过了这村没这店。\"",
                        fmt_int(price)
                    );

                    card.opts.push(("买下".into(), e));
                } else {
                    card.text = format!(
                        "蒙面商人：\"洗髓丹一枚，{} 灵石，童叟无欺……\"你摸了摸储物袋，钱不够。",
                        fmt_int(price)
                    );
                }

                let mut e0 = Eff::new();

                e0.msg = vec!["你冲商人拱了拱手，转身自去。".into()];

                card.opts.push(("摆手离开".into(), e0));
            }

            18 => {
                card.title = "前辈指点".into();

                // v1.16.20 正文画面化第二批：「醍醐灌顶」结论词改动作画面（只改字不加段）
                card.text = "白眉老者只随手点了三处，你回头再看旧稿，字字都换了个意思。".into();

                let mut e = Eff::new();

                e.qi_pct = 0.12;

                e.dao = 4.0;

                e.msg = vec!["修为+12%，道心+4".into()];

                card.opts.push(("叩谢前辈".into(), e));
            }

            19 => {
                card.title = "灵兽认主".into();

                card.text = "一只雪白小兽一路跟着你，赶也赶不走，与你甚是有缘。".into();

                let mut e = Eff::new();

                e.luck = 6.0;

                let g = (50.0 * ss) as i64;

                e.stones = g;

                e.msg = vec![format!("气运+6，得灵石 {}", fmt_int(g))];

                card.opts.push(("收留".into(), e));
            }

            20 => {
                card.title = "仙人问道".into();

                card.text =
                    "云端降下一位白须仙人，含笑看你：\"小子，老夫可指你一条路，要哪般？\"".into();

                let mut e1 = Eff::new();

                e1.dao = 8.0;

                e1.qi_pct = 0.15;

                e1.msg = vec!["你求问道法。道心+8，修为+15%".into()];

                card.opts.push(("求道！".into(), e1));

                let mut e2 = Eff::new();

                e2.special = 8;

                e2.good = false;

                e2.msg = vec!["你求问武道。战力永久+12%！".into()];

                card.opts.push(("求武！".into(), e2));

                let mut e3 = Eff::new();

                e3.life_y = 50.0;

                e3.msg = vec!["你求问长生。寿元+50 载".into()];

                card.opts.push(("求寿！".into(), e3));
            }

            21 => {
                card.title = "灵泉洞天".into();

                card.text =
                    "山谷深处一眼温泉汩汩冒泡，灵气凝成白雾，你入泉浸泡三日，通体舒泰。".into();

                let mut e = Eff::new();

                e.life_y = 30.0;

                e.qi_pct = 0.10;

                e.msg = vec!["寿元+30 载，修为+10%".into()];

                card.opts.push(("入泉沐浴".into(), e));
            }

            22 => {
                card.title = "古剑冢".into();

                card.text =
                    "荒原上插着十万柄锈剑，剑鸣如龙吟。中央一柄古剑兀自震颤，似在择主。".into();

                let mut e1 = Eff::new();

                e1.fight = 1.4;

                e1.special = 9;

                e1.good = false;

                card.opts.push(("上前拔剑".into(), e1));

                let mut e2 = Eff::new();

                e2.dao = 4.0;

                e2.msg = vec!["你恭敬参拜，剑鸣渐息。道心+4".into()];

                card.opts.push(("躬身参拜".into(), e2));
            }

            23 => {
                card.title = "魔窟探宝".into();

                card.text = "山腹中一座魔修洞窟，入口白骨累累，深处宝光与魔气交织翻涌。".into();

                let mut e1 = Eff::new();

                e1.special = 10;

                e1.good = false;

                card.opts.push(("孤身深入".into(), e1));

                let mut e0 = Eff::new();

                e0.msg = vec!["你不入魔窟，原路下山。".into()];

                card.opts.push(("转身就走".into(), e0));
            }

            24 => {
                card.title = "雷击木".into();

                card.text =
                    "一株遭天雷劈中的古木倒在路边，焦黑树干里雷光未散，正是炼丹好材料。".into();

                let mut e = Eff::new();

                e.pill = Some((5, 2));

                e.msg = vec!["得雷击木，炼成护神丹 ×2".into()];

                card.opts.push(("采集".into(), e));
            }

            25 => {
                card.title = "商队护送".into();

                card.text = "一支灵石商队遭妖兽围困，管事高声求援：\"护我商队，重金相谢！\"".into();

                let mut e1 = Eff::new();

                e1.fight = 1.0;

                e1.special = 11;

                e1.good = false;

                card.opts.push(("拔刀相助".into(), e1));

                let mut e2 = Eff::new();

                e2.luck = 1.0;

                e2.msg = vec!["你婉拒离去。气运+1".into()];

                card.opts.push(("事不关己".into(), e2));
            }

            26 => {
                card.title = "仙人赠礼".into();

                card.text =
                    "一位乘鹤老者路过，随手抛下锦盒：\"与你有缘，收着吧。\"鹤影转瞬没入云中。"
                        .into();

                let rl = gen_range(0, RELICS.len());

                let mut e = Eff::new();

                e.relic = Some(rl);

                e.qi_pct = 0.30;

                e.msg = vec![format!("盒中是【{}】！修为+30%", RELICS[rl].name)];

                card.opts.push(("叩谢仙长".into(), e));
            }

            27 => {
                card.title = "天机阁卜卦".into();

                card.text =
                    "天机阁的瞎眼卦师掐指一算：\"道友印堂有光，是福是祸，在一念之间。\"".into();

                let mut e1 = Eff::new();

                e1.dao = 6.0;

                e1.pill = Some((2, 1));

                e1.msg = vec!["卦师赠你一枚破境丹，道心+6".into()];

                card.opts.push(("求上一卦".into(), e1));

                let mut e2 = Eff::new();

                // v1.16.15 复审口径：香火钱照 ss 缩放（照 38 拍卖量级）——500 平价
                // 与气运+8 的收益量级在后期失衡；realm0=180 与既有收入事件自洽
                e2.stones = -((180.0 * ss) as i64);

                e2.luck = 8.0;

                e2.msg = vec![format!(
                    "你留下香火钱。灵石-{}，气运+8",
                    fmt_int((180.0 * ss) as i64)
                )];

                card.opts.push(("捐些香火".into(), e2));
            }

            // ---------- v1.16.5 扩池 13 条（id 31..=43）----------
            // 题材覆盖：奇遇得宝/危机考验/人情往来/修行顿悟/市井奇闻/秘境探索；
            // 每条 ≥2 选项且各有得失。全部只用通用 Eff 字段（不新增 special：
            // 1/2/3/7 型臂带 return 会跳过通用结算段），赌局走普通 fight
            // （sim_fight 内建胜缴获/败重伤）—— eff_desc 副标签自动生成
            31 => {
                card.title = "雨夜借宿".into();
                card.text =
                    "山雨骤至，你叩响山腰人家的柴门。老丈披衣开门：「道友不嫌简陋，同吃一锅热粥。」"
                        .into();
                let mut e1 = Eff::new();
                e1.dao = 4.0;
                e1.luck = 1.0;
                e1.msg = vec!["柴水劈满缸，老丈讲了一夜山中旧事。道心+4，气运+1".into()];
                card.opts.push(("帮劈柴水".into(), e1));
                let g = (60.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = -g;
                e2.luck = 2.0;
                e2.msg = vec!["你留下一袋灵石作谢，此后路过必有热茶。气运+2".into()];
                card.opts.push(("留下谢仪".into(), e2));
            }
            32 => {
                card.title = "古碑残字".into();
                card.text =
                    "荒径尽头立着半截古碑，苔痕下残字若隐若现，似是一篇未写完的行功总纲。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.08;
                e1.dao = 3.0;
                e1.msg = vec!["残字入心，如春水化冰。修为+8%，道心+3".into()];
                card.opts.push(("拂苔细读".into(), e1));
                let g = (70.0 * ss * gen_range(0.7, 1.4)) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!("拓片售予坊市书坊，碑中真意却错过了。灵石+{}", fmt_int(g))];
                card.opts.push(("拓印售卖".into(), e2));
            }
            33 => {
                card.title = "雾锁迷林".into();
                card.text = "白雾灌林，方向尽失。雾深处有人低低唤你的名字，一声比一声近。".into();
                let mut e1 = Eff::new();
                e1.fight = 0.85;
                e1.good = false;
                card.opts.push(("循声而去".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 3.0;
                e2.stones_pct = -0.08;
                e2.msg = vec!["你枯坐守心，雾散时天已放晴。道心+3".into()];
                card.opts.push(("原地守心".into(), e2));
            }
            34 => {
                card.title = "墨市笔坊".into();
                card.text = "坊市新张一家笔坊，掌柜拍胸脯：「我这兽毫，一笔落纸可定千行！」案上笔杆根根乌亮。".into();
                let g = (100.0 * ss) as i64;
                let mut e1 = Eff::new();
                e1.stones = -g;
                e1.qi_pct = 0.10;
                e1.msg = vec!["新笔饱墨，行气如虹。修为+10%".into()];
                card.opts.push(("重金购笔".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.msg = vec!["你看掌柜制笔半日，悟得几分运笔之要。道心+2".into()];
                card.opts.push(("观摩制笔".into(), e2));
            }
            35 => {
                card.title = "断桥济人".into();
                card.text = "溪桥塌了半边，一位货郎守着翻倒的担子发愁。你袖袍一卷，把货担稳稳送上对岸。".into();
                let mut e1 = Eff::new();
                e1.dao = 3.0;
                e1.luck = 3.0;
                e1.msg = vec!["货郎千恩万谢，逢人便说你仁义。道心+3，气运+3".into()];
                card.opts.push(("分文不取".into(), e1));
                let g = (40.0 * ss * gen_range(0.9, 1.5)) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!("货郎硬塞了脚钱。灵石+{}", fmt_int(g))];
                card.opts.push(("收些脚钱".into(), e2));
            }
            36 => {
                card.title = "瘴谷寻药".into();
                card.text = "瘴气谷口石碑刻着「药王留谷，取之有度」，谷底灵药成片，雾气泛着幽绿。".into();
                let mut e1 = Eff::new();
                e1.fight = 1.05;
                e1.pill = Some((0, 2));
                e1.good = false;
                card.opts.push(("深入谷心".into(), e1));
                let g = (50.0 * ss * gen_range(0.7, 1.4)) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!("你拾得谷口自然枯落的药草，售得灵石 {}", fmt_int(g))];
                card.opts.push(("谷口拾遗".into(), e2));
            }
            37 => {
                card.title = "渡灵舟".into();
                card.text = "灵舟行至江心，墨色巨浪忽然立起，船家面色发白：「水妖翻浪——稳住！」".into();
                let mut e1 = Eff::new();
                e1.fight = 1.0;
                e1.good = false;
                card.opts.push(("协助退妖".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.stones_pct = -0.10;
                e2.msg = vec!["船身颠簸如筛，行囊湿了大半。道心+2".into()];
                card.opts.push(("闭舱守心".into(), e2));
            }
            38 => {
                card.title = "残卷拍卖".into();
                card.text = "拍卖行压轴一幅无名残卷，起拍百灵石——据说落款处藏着一篇功法总纲。".into();
                let g = (180.0 * ss) as i64;
                let mut e1 = Eff::new();
                e1.stones = -g;
                e1.qi_pct = 0.15;
                e1.dao = 2.0;
                e1.msg = vec!["你连夜参详，卷中总纲果然不凡。修为+15%，道心+2".into()];
                card.opts.push(("举牌竞得".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.msg = vec!["残卷被锦衣修士天价拍走。不动心，也是修行。道心+2".into()];
                card.opts.push(("袖手旁观".into(), e2));
            }
            39 => {
                card.title = "灵田佃户".into();
                card.text = "灵田佃户老周拦路作揖：「仙长，田里灵谷一夜白穗，您给瞧瞧是何缘故？」".into();
                let mut e1 = Eff::new();
                e1.dao = 4.0;
                e1.luck = 2.0;
                e1.msg = vec!["你以灵力驱尽谷虫，老周赠新米一斗。道心+4，气运+2".into()];
                card.opts.push(("亲往除虫".into(), e1));
                let g = (35.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!("你教他烟熏护田之法，老周塞来谢礼。灵石+{}", fmt_int(g))];
                card.opts.push(("指点几句".into(), e2));
            }
            40 => {
                card.title = "酒楼斗法".into();
                card.text = "坊市酒楼里两名外乡修士斗法，桌椅横飞波及无辜，掌柜朝你直使眼色。".into();
                let mut e1 = Eff::new();
                e1.fight = 1.15;
                e1.good = false;
                card.opts.push(("出手镇场".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 3.0;
                e2.stones_pct = -0.05;
                e2.msg = vec!["你护着凡人客人撤到街外，毁了一身行头。道心+3".into()];
                card.opts.push(("掩护撤离".into(), e2));
            }
            41 => {
                card.title = "前人手札".into();
                card.text = "旧书摊的线装手札里夹着几页批注，墨迹已淡，隐约指着一处行功捷径。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.20;
                e1.dao = 3.0;
                e1.msg = vec!["你按批注行功一月，水到渠成。修为+20%，道心+3".into()];
                card.opts.push(("依注苦修".into(), e1));
                let g = (150.0 * ss * gen_range(0.8, 1.5)) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!("手札被藏经楼高价收走。灵石+{}", fmt_int(g))];
                card.opts.push(("高价转售".into(), e2));
            }
            42 => {
                card.title = "废墟道场".into();
                card.text = "山雾散处露出一座废弃道场，蒲团积尘盈寸，梁上剑痕犹新。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.12;
                e1.life_y = 8.0;
                e1.msg = vec!["你入定七日，出关时步履生风。修为+12%，寿元+8".into()];
                card.opts.push(("蒲团参悟".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 5.0;
                e2.msg = vec!["你替道场扫尘七日——扫地亦是修行。道心+5".into()];
                // v1.16.15 复审口径：正文是废弃道场无塔，选项照 msg 同词改「扫尘」
                card.opts.push(("扫尘七日".into(), e2));
            }
            43 => {
                card.title = "沙暴藏窟".into();
                card.text = "沙暴过后，崖壁上裸出一处洞窟，铁链断了一半，深处隐隐透出陈年墨香。".into();
                let mut e1 = Eff::new();
                e1.fight = 1.2;
                e1.good = false;
                card.opts.push(("斩链入窟".into(), e1));
                let g = (120.0 * ss * gen_range(0.8, 1.4)) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!("外洞石案上散着前人遗落的灵石。灵石+{}", fmt_int(g))];
                card.opts.push(("只探外洞".into(), e2));
            }

            // ---------- v1.16.7 扩池 15 条（id 44..=58）----------
            // 三条连锁剧情（44..=46 货郎 / 47..=49 古碑 / 50..=52 墨匣）：首段入
            // 正常池，弹卡即置链（下段 0=待分支，抉择经 special 20..=25 推进）；
            // 链内段标题带「·续」。道侣同行 53..=55（COMPANION_EVENTS 门控）挂
            // Comp 立绘；稀有 56..=58（w≤2）高风险高回报。链内段与稀有卡无人物
            // 不挂 actor → 维持 600×400 原版式，正文 ≤3 行（约 50 字内）
            44 => {
                // 链1 首段：神秘货郎（万宝楼外柜——与胡掌柜同门，复用其脸，见
                // draw.rs 邂逅版式；npc_tex(1) 为 None 时自动落旧版式，可接受）
                card.title = "货郎奇遇".into();
                card.actor = Some(CardActor::Npc(1));
                // v1.16.7 两段文案（邂逅版式 5 行内）：叙述与叫卖分段，段间 8px
                card.text = "山道拐角，万宝楼外柜的货郎拦住你，担子里一支墨锭透着幽光，隐有灵气盘旋。\n「引灵墨，观道友有缘——先拿去用，账回头再算。」".into();
                let g = (60.0 * ss) as i64;
                let mut e1 = Eff::new();
                e1.stones = -g;
                e1.qi_pct = 0.08;
                e1.special = 20; // 置链：→46 恩怨上门
                e1.msg = vec![format!(
                    "你赊下引灵墨，当夜行功，灵气循行快了三分。修为+8%，灵石-{}（赊账一笔，必有后话）",
                    fmt_int(g)
                )];
                card.opts.push(("赊墨立契".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.special = 21; // 置链：→45 罗盘相赠
                e2.msg = vec!["货郎也不恼，笑言「缘法未到」。道心+2".into()];
                card.opts.push(("婉拒不买".into(), e2));
                self.chain = Some((1, 0, CHAIN_TTL)); // 首段建链：待分支
            }
            45 => {
                // 链1 分支b 段（仅链内）：两个选项均无置链臂 → 抉择即链终局
                card.title = "罗盘相赠·续".into();
                card.actor = Some(CardActor::Npc(1));
                card.text = "数年后山道再遇货郎。他认出你，把一枚破旧罗盘塞进你手里：「当初不买是我看走眼——赠你把玩，结个善缘。」".into();
                let mut e1 = Eff::new();
                e1.luck = 3.0;
                e1.msg = vec!["罗盘针自转不止，你行路却顺遂了许多。气运+3".into()];
                card.opts.push(("收下罗盘".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.luck = 1.0;
                e2.msg = vec!["你以热茶相敬，两人聊了一路掌故。道心+2，气运+1".into()];
                card.opts.push(("煮茶相敬".into(), e2));
            }
            46 => {
                // 链1 分支a 段（仅链内）：战或赔都终结这条孽缘
                card.title = "恩怨上门·续".into();
                card.actor = Some(CardActor::Npc(1));
                card.text = "夜半拍门声急。货郎躲在门外，脸色发白：「兄台，你赊的那支墨……原主寻上门了，说要么十倍赔付，要么见血！」".into();
                let mut e1 = Eff::new();
                e1.fight = 1.3;
                e1.good = false;
                card.opts.push(("拔剑护墨".into(), e1));
                let mut e2 = Eff::new();
                e2.stones_pct = -0.20;
                e2.good = false;
                // v1.16.20 链尾余味（故事沉浸续作）：记账体改画面句——赔付有了动作与
                // 分寸感（msg 只改字不改正文段数，:4700 版式预算不动）
                e2.msg = vec!["你数出十倍墨价放在门槛上——门里门外，从此两清。灵石-20%".into()];
                card.opts.push(("十倍赔付".into(), e2));
            }
            47 => {
                // 链2 首段：雾中古碑（文案基因承 id32 古碑残字，为另一方碑）
                card.title = "古碑残篇".into();
                // v1.16.7 两段文案共 4 行 → 首个走加高版式（600×440）的剧情卡
                card.text = "雾锁的山坳立着一方丈高古碑，苔痕不生，碑面无字——山风过处，隐有金石摩挲之声。\n你走近时，指尖竟渗出墨意，在碑上洇出半个「行」字，如有人隔世落笔。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.06;
                e1.special = 22; // 置链：→48 拓印风波
                e1.msg = vec!["血墨相引，碑文渐显，你连夜拓下前半卷。修为+6%".into()];
                card.opts.push(("以血引墨".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 3.0;
                e2.msg = vec!["你敬畏未显之文，覆土掩碑而去。道心+3".into()];
                card.opts.push(("覆土掩碑".into(), e2));
                self.chain = Some((2, 0, CHAIN_TTL));
            }
            48 => {
                // 链2 中段（仅链内）：焚拓 = 「碑随雾散」自洽收尾
                card.title = "拓印风波·续".into();
                card.text = "你携拓片到坊市装裱，老裱工却变了脸色：「此拓火气太重，三日必有争端——要么焚了它，要么我引你去见一个人。」".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.special = 23; // 置链：→49 守碑人
                e1.msg = vec!["你随他冒雨入山。道心+2".into()];
                card.opts.push(("随他见人".into(), e1));
                let g = (80.0 * ss * gen_range(0.8, 1.4)) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.luck = -1.0;
                e2.msg = vec![format!(
                    "锦衣修士当场出价购拓，你收钱焚拓。灵石+{}，气运-1",
                    fmt_int(g)
                )];
                card.opts.push(("当场焚拓".into(), e2));
            }
            49 => {
                // 链2 末段（仅链内）：special4 道心检定臂（过授功法，败负内创）
                card.title = "守碑人·续".into();
                card.actor = Some(CardActor::Npc(0));
                card.text = "雾中竟有人守碑。青衣老者负手而立：「碑文是我一生道途。拓它者，须过我三问——问错了，便把它留下。」".into();
                let mut e1 = Eff::new();
                e1.special = 4;
                e1.good = false;
                card.opts.push(("受此三问".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 1.0;
                e2.msg = vec!["你长揖而退，守碑人在雾中颔首。道心+1".into()];
                card.opts.push(("告退离山".into(), e2));
            }
            50 => {
                // 链3 首段：江底墨匣
                card.title = "江底墨匣".into();
                // v1.16.7 两段文案共 4 行 → 加高版式
                card.text = "渡舟行至江心，你失足落水。呛了几口江水，指尖却触到江底一只冰凉的铜匣，匣身纹路温润如墨玉。\n匣面錾着「沉墨候主」四字——指腹刚碰到，匣身微微发热，湍急的江水竟在你四周静了下来。".into();
                let mut e1 = Eff::new();
                e1.luck = 1.0;
                e1.special = 24; // 置链：→51 渔村释疑
                e1.msg = vec!["你把铜匣收进袖中，舟行一顺到底。气运+1".into()];
                card.opts.push(("收匣启程".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.msg = vec!["匣沉江底，水面泛起一圈墨色涟漪，久久不散。道心+2".into()];
                card.opts.push(("抛回江底".into(), e2));
                self.chain = Some((3, 0, CHAIN_TTL));
            }
            51 => {
                // 链3 中段（仅链内）：老渔翁（复用醉道人脸）
                card.title = "渔村释疑·续".into();
                card.actor = Some(CardActor::Npc(3));
                card.text = "沿江渔村有位不苟言笑的老渔翁，见匣惊起：「这是『沉墨』——墨痴三十年前沉江的遗物。他说『匣候有缘，缘至当还』。」".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.special = 25; // 置链：→52 匣底真迹
                e1.msg = vec!["你随老渔翁驾舟往江心去。道心+2".into()];
                card.opts.push(("随往江心".into(), e1));
                let g = (25.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!("老渔翁摇头叹息，赠你两尾灵鱼作谢。灵石+{}", fmt_int(g))];
                card.opts.push(("婉言辞谢".into(), e2));
            }
            52 => {
                // 链3 末段（仅链内）：special12 参悟入大事记
                card.title = "匣底真迹·续".into();
                card.text = "江心礁下，铜匣无风自开——匣底一页真迹，笔势枯润相生。你看得入神，竟忘了身在水中，灵气自四面八方涌来。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.18;
                e1.dao = 3.0;
                e1.special = 12;
                e1.msg = vec!["真迹入心，如醍醐灌顶。修为+18%，道心+3".into()];
                card.opts.push(("参悟真迹".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 4.0;
                e2.luck = 2.0;
                e2.msg = vec!["老渔翁捧页大恸，拜了三拜——那是他师兄的绝笔。道心+4，气运+2".into()];
                card.opts.push(("转赠渔翁".into(), e2));
            }
            53 => {
                // 伴随事件：道侣论道（companion 白名单门控才入池；卡面立绘反查
                // COMPANION_NAMES 下标 → Comp 版式，名必同源）
                card.title = "道侣论道".into();
                let cn = {
                    let s = self.sim.as_ref().unwrap();
                    s.companion
                        .clone()
                        .unwrap_or_else(|| COMPANION_NAMES[0].to_string())
                };
                let ci = COMPANION_NAMES
                    .iter()
                    .position(|&n| n == cn)
                    .unwrap_or(0);
                card.actor = Some(CardActor::Comp(ci));
                card.text = format!(
                    "山亭对坐，{}忽然发问：「你我同修一道，可曾想过，道途尽头是什么模样？」",
                    cn
                );
                let mut e1 = Eff::new();
                e1.qi_pct = 0.10;
                e1.dao = 3.0;
                e1.msg = vec!["你们以同一功法互证，各悟出对方的长处。修为+10%，道心+3".into()];
                card.opts.push(("同参一道".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.life_y = 2.0;
                e2.msg = vec!["一盏茶从午后喝到落雪，谁也没答，谁也不必答。气运+2，寿元+2".into()];
                card.opts.push(("煮茶听雪".into(), e2));
            }
            54 => {
                // 伴随事件：赠剑同舟（special5 战力 buff 臂）
                card.title = "赠剑同舟".into();
                let cn = {
                    let s = self.sim.as_ref().unwrap();
                    s.companion
                        .clone()
                        .unwrap_or_else(|| COMPANION_NAMES[0].to_string())
                };
                let ci = COMPANION_NAMES
                    .iter()
                    .position(|&n| n == cn)
                    .unwrap_or(0);
                card.actor = Some(CardActor::Comp(ci));
                card.text = format!(
                    "{}把一柄旧剑系上红绳递来：「早年得的，一直没舍得开锋。今日给你——剑名『同舟』。」",
                    cn
                );
                let mut e1 = Eff::new();
                e1.special = 5;
                e1.msg = vec!["你接剑入手，只觉道途有伴。战力永久+4%".into()];
                card.opts.push(("佩剑同行".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 3.0;
                e2.msg = vec![format!(
                    "你说「剑当护道，不当争斗」，{} 笑了：「也好。」道心+3",
                    cn
                )];
                card.opts.push(("推辞不受".into(), e2));
            }
            55 => {
                // 伴随事件：危难共护（无好的绕行——硬受也有回报）
                card.title = "危难共护".into();
                let cn = {
                    let s = self.sim.as_ref().unwrap();
                    s.companion
                        .clone()
                        .unwrap_or_else(|| COMPANION_NAMES[0].to_string())
                };
                let ci = COMPANION_NAMES
                    .iter()
                    .position(|&n| n == cn)
                    .unwrap_or(0);
                card.actor = Some(CardActor::Comp(ci));
                card.text = format!(
                    "山道妖兽突袭，你被扫落山涧——黑暗里一只手死死抓住你：「别松手！」是{}。",
                    cn
                );
                let mut e1 = Eff::new();
                e1.fight = 1.15;
                e1.good = false;
                card.opts.push(("合力御敌".into(), e1));
                let mut e2 = Eff::new();
                e2.stones_pct = -0.10;
                e2.luck = 2.0;
                e2.good = false;
                e2.msg = vec![
                    "你把身后让给道侣，自己硬受了这一击——行囊尽毁，却听见那追来的脚步声。灵石-10%，气运+2".into(),
                ];
                card.opts.push(("护身后人先走".into(), e2));
            }
            56 => {
                // 稀有：星坠荒原（w2；special6 五五开试炼臂）
                card.title = "星坠荒原".into();
                card.text = "夜半天穹忽亮，一颗流星坠入荒原。追到坑口，陨铁未冷，坑底静卧一卷星纹绢书。".into();
                let mut e1 = Eff::new();
                e1.special = 6;
                e1.good = false;
                card.opts.push(("赤手取铁".into(), e1));
                let mut e2 = Eff::new();
                e2.qi_pct = 0.10;
                e2.dao = 2.0;
                e2.msg = vec!["你拾绢书而去，星纹入梦。修为+10%，道心+2".into()];
                card.opts.push(("拾书而归".into(), e2));
            }
            57 => {
                // 稀有：无面镜客（w1；special26 赌命臂 / fight 碎镜）
                card.title = "无面镜客".into();
                // v1.16.7 两段文案共 4 行 → 加高版式（镜中怪谈的节奏感靠分段撑）
                card.text = "荒庙断壁前立着一面古镜，镜中人没有脸。它抬手，你也抬手；它偏头，你也偏头。\n而后它开口，用的正是你的声音：「赌一次么？赢，得我半生修为；输，留十年寿数。」".into();
                let mut e1 = Eff::new();
                e1.special = 26;
                e1.good = false;
                card.opts.push(("赌命".into(), e1));
                let mut e2 = Eff::new();
                e2.fight = 1.2;
                e2.good = false;
                card.opts.push(("一剑碎镜".into(), e2));
            }
            58 => {
                // 稀有：灵潮倒卷（w2；special27 漩心洞府臂）
                card.title = "灵潮倒卷".into();
                card.text = "地脉异动，灵潮倒卷成漩——漩心隐见一座前人闭死的洞府，灵气疯狂外泄，诱人至极，也凶险至极。".into();
                let mut e1 = Eff::new();
                e1.special = 27;
                e1.good = false;
                card.opts.push(("冒死入漩".into(), e1));
                let mut e2 = Eff::new();
                e2.qi_pct = 0.15;
                e2.msg = vec!["你在外围引外泄灵气入体，修为+15%".into()];
                card.opts.push(("外围引气".into(), e2));
            }

            // ---------- v1.16.8 扩池 26 条（id 59..=84）----------
            // 普通 13 条（59..=71）无 actor → 600 版式正文 ≤3 行（单段 ≤50 字/两段
            // 合计 ≤96 字）；稀有 2 条（72..=73，w≤2，检定臂 special33/34）；章节联动
            // 2 条（74..=75，入池门控在 sim_event_weights 读 chap_done 位——75 有伴
            // 挂 Comp 立绘）；连锁 6 条（76..=81，链4/链5，接线见 CHAIN_SPECIAL_HI
            // 与 apply_eff 置链臂 28..=31）；伴随 3 条（82..=84，白名单扩 6）。新文案
            // 一律只经本函数尾统一预热（prewarm_text title 24/text 16）
            59 => {
                // 普通：书院讲经
                card.title = "书院讲经".into();
                card.text = "山坳蒙学书院，白发先生讲《上清章句》，学童齐声诵读，书声里隐有文气流转。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.msg = vec!["你于廊下坐听半日，稚子清音涤荡心尘。道心+2".into()];
                card.opts.push(("坐听半日".into(), e1));
                let g = (30.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = -g;
                e2.luck = 2.0;
                e2.msg = vec![format!(
                    "你留下一束笔墨，先生率稚子一揖相送。灵石-{}，气运+2",
                    fmt_int(g)
                )];
                card.opts.push(("捐笔一束".into(), e2));
            }
            60 => {
                // 普通：灵蜂酿蜜
                card.title = "灵蜂酿蜜".into();
                card.text = "崖壁灵蜂采百花蕊酿蜜，蜂巢边缘凝着一点金芒——那是花髓千日一滴的精华。".into();
                let g = (40.0 * ss) as i64;
                let mut e1 = Eff::new();
                e1.stones = -g;
                e1.qi_pct = 0.06;
                e1.msg = vec![format!("蜜入喉即化，一线暖意沉入丹田。灵石-{}，修为+6%", fmt_int(g))];
                card.opts.push(("买蜜一罐".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 1.0;
                e2.luck = 1.0;
                e2.msg = vec!["你以石子驱走贪蜜的灵雀，蜂群绕你三匝而舞。道心+1，气运+1".into()];
                card.opts.push(("驱鸟护巢".into(), e2));
            }
            61 => {
                // 普通：画师留影
                card.title = "画师留影".into();
                card.text = "云游画师道人拦路，称见你「眉间有墨」，要为你白描一幅小像，分文不取。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.luck = 1.0;
                e1.msg = vec!["画成之日，画师赞「此子有书卷气」，赠你原稿。道心+2，气运+1".into()];
                card.opts.push(("慨然入画".into(), e1));
                let g = (20.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = -g;
                e2.dao = 1.0;
                e2.msg = vec![format!(
                    "你婉辞不受留银而去，画师在扉页为你题了半句偈。灵石-{}，道心+1",
                    fmt_int(g)
                )];
                card.opts.push(("婉辞赠银".into(), e2));
            }
            62 => {
                // 普通：雪崩沟谷（危机双臂皆负——受创换气运/损耗绕行）
                card.title = "雪崩沟谷".into();
                card.text = "沟谷雪线在头顶断裂，积雪裹着碎石轰然倾泻——退已不及，寒气先一步蚀骨。".into();
                let mut e1 = Eff::new();
                e1.injure = true;
                e1.luck = 2.0;
                e1.good = false;
                e1.msg = vec![
                    "你以身撞开雪墙滚落沟底——大难不死，气运反涨。重伤，气运+2".into(),
                ];
                card.opts.push(("以身开道".into(), e1));
                let mut e2 = Eff::new();
                e2.stones_pct = -0.06;
                e2.good = false;
                e2.msg = vec![
                    "你退回原路绕行远道，行囊里的干粮耗去一成。灵石-6%".into(),
                ];
                card.opts.push(("绕行远路".into(), e2));
            }
            63 => {
                // 普通：赌坊押擂（good=false；登台走 fight 臂照 id46 先例）
                card.title = "赌坊押擂".into();
                card.text = "坊市地窖里开着一处地下斗擂，庄家嘶声吆喝——押赢的抱银子，上台的赌性命。".into();
                let mut e1 = Eff::new();
                e1.fight = 1.2;
                e1.good = false;
                card.opts.push(("登台押己".into(), e1));
                let mut e2 = Eff::new();
                e2.stones_pct = -0.05;
                e2.good = false;
                e2.msg = vec![
                    "你押了庄家对头——十赌九输，庄家笑纳注银，还请你一盏酒。灵石-5%".into(),
                ];
                card.opts.push(("只押注".into(), e2));
            }
            64 => {
                // 普通：漕运夜航
                card.title = "漕运夜航".into();
                card.text = "夜航漕船满载灵米北上，船老大拱手相邀：近来水路不太平，缺一位护道的。".into();
                let g = (60.0 * ss) as i64;
                let mut e1 = Eff::new();
                e1.stones = g;
                e1.msg = vec![format!(
                    "一路风平浪静，抵岸时船老大塞来一封厚谢仪。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("伴航护船".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.msg = vec!["你倚着米袋读了一夜书，橹声欸乃如节拍。道心+2".into()];
                card.opts.push(("舱中夜读".into(), e2));
            }
            65 => {
                // 普通：温泉濯尘
                card.title = "温泉濯尘".into();
                card.text = "山间野温泉雾气氤氲，泉底铺着圆润的白石，水温恰能化开筋骨里的滞涩。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.05;
                e1.msg = vec!["泡到浑身通红，经脉滞涩尽数化开。修为+5%".into()];
                card.opts.push(("濯尘泡浴".into(), e1));
                let g = (25.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!(
                    "白石经年泡养微蕴灵气，坊市有人识货。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("掘石携归".into(), e2));
            }
            66 => {
                // 普通：棋枰斗弈
                card.title = "棋枰斗弈".into();
                card.text = "山亭里两位老叟对弈，一局棋下了三年——落子处松针积了寸许，无人肯先离席。".into();
                let mut e1 = Eff::new();
                e1.dao = 3.0;
                e1.msg = vec!["你观至中盘，忽然懂了「弃子」二字。道心+3".into()];
                card.opts.push(("观棋不语".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 1.0;
                e2.luck = 2.0;
                e2.msg = vec!["你替僵局落了一子，两叟同时抚掌大笑。道心+1，气运+2".into()];
                card.opts.push(("举手投子".into(), e2));
            }
            67 => {
                // 普通：铸兵守炉
                card.title = "铸兵守炉".into();
                card.text = "铸兵铺的老铁匠要闭关淬一口剑胚，请你替他守一夜炉火——火候差一分，前功尽弃。".into();
                let g = (50.0 * ss) as i64;
                let mut e1 = Eff::new();
                e1.stones = g;
                e1.msg = vec![format!(
                    "炉火在你注视下稳稳渡过子时，老铁匠以铁屑相谢。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("通宵守炉".into(), e1));
                let mut e2 = Eff::new();
                e2.qi_pct = 0.06;
                e2.msg = vec!["你抡锤二十下，臂酸三日，气机却顺了。修为+6%".into()];
                card.opts.push(("助锻一锤".into(), e2));
            }
            68 => {
                // 普通：蜃楼残景（mr3 高界一条）
                card.title = "蜃楼残景".into();
                card.text = "海市蜃楼现于海上，楼台街市皆前人笔意——那是仙域投在凡世的一道残影，转瞬即散。".into();
                let mut e1 = Eff::new();
                e1.dao = 4.0;
                e1.msg = vec!["残影入眼即化，你记住了飞檐上的一笔弧度。道心+4".into()];
                card.opts.push(("凝望参悟".into(), e1));
                let mut e2 = Eff::new();
                e2.qi_pct = 0.10;
                e2.msg = vec!["你以指为笔临于袖上，仙域气韵入体三分。修为+10%".into()];
                card.opts.push(("画下残景".into(), e2));
            }
            69 => {
                // 普通：井底传声
                card.title = "井底传声".into();
                card.text = "荒村古井夜半传声，井底有人断断续续诵经——村人说，井是前朝道士封的。".into();
                let mut e1 = Eff::new();
                e1.luck = 2.0;
                e1.msg = vec!["井底无物，唯壁上一行刻字被水光映亮，你记住了它。气运+2".into()];
                card.opts.push(("探井底".into(), e1));
                let g = (15.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = -g;
                e2.dao = 2.0;
                e2.msg = vec![format!(
                    "你以清水研墨倾入井中，诵声停了，井水甘冽如饴。灵石-{}，道心+2",
                    fmt_int(g)
                )];
                card.opts.push(("墨祭古井".into(), e2));
            }
            70 => {
                // 普通：药市问诊
                card.title = "药市问诊".into();
                card.text = "药市义诊的悬壶老叟搭住你的腕：「气是够的，就是走岔了道——老朽替你顺一顺？」".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.08;
                e1.msg = vec!["老叟三指一按一拨，滞气应手而散。修为+8%".into()];
                card.opts.push(("求一脉案".into(), e1));
                let g = (20.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.dao = 1.0;
                e2.msg = vec![format!(
                    "你帮药童碾了一下午药，老叟以药渣里挑出的灵芝相赠。灵石+{}，道心+1",
                    fmt_int(g)
                )];
                card.opts.push(("帮碾灵药".into(), e2));
            }
            71 => {
                // 普通：驿壁题诗
                card.title = "驿壁题诗".into();
                card.text = "官道驿站粉壁斑驳，留满前人题诗——最末一行墨迹未干：「此身虽微，万世可期。」".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.luck = 1.0;
                e1.msg = vec!["你题罢掷笔，满壁字迹似都亮了一亮。道心+2，气运+1".into()];
                card.opts.push(("题诗一首".into(), e1));
                let mut e2 = Eff::new();
                e2.qi_pct = 0.05;
                e2.msg = vec!["你抄下半壁好句，一路低吟。修为+5%".into()];
                card.opts.push(("抄录佳句".into(), e2));
            }
            72 => {
                // 稀有：墨渊垂钓（w2；special33 墨叟一问检定 / 静观悟道）
                card.title = "墨渊垂钓".into();
                card.actor = Some(CardActor::Npc(3)); // 蓑衣钓叟（复用醉道人脸）
                card.text = "夜半墨渊，蓑衣老叟垂钓，钩上无饵——钓的是水里的「字」。老叟头也不回：「来者可是爱字之人？老朽钓上一个，你认得，便归你。」".into();
                let mut e1 = Eff::new();
                e1.special = 33;
                e1.good = false;
                card.opts.push(("受墨叟一问".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 3.0;
                e2.msg = vec!["你坐到天明，老叟竿起竿落，一字未得亦一字未失。道心+3".into()];
                card.opts.push(("静观不语".into(), e2));
            }
            73 => {
                // 稀有：纸人夜行（w1 mr3；special34 拦问检定 / 避道小损）
                card.title = "纸人夜行".into();
                card.text = "深夜纸扎铺无人，纸人却列队而出，抬着一口巴掌大的小棺，步调整齐得可怕。".into();
                let mut e1 = Eff::new();
                e1.special = 34;
                e1.good = false;
                card.opts.push(("拦问纸人".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = -1.0;
                e2.good = false;
                e2.msg = vec![
                    "你贴墙让路，纸人行过时齐齐侧头看了你一眼。气运-1".into(),
                ];
                card.opts.push(("避道让行".into(), e2));
            }
            74 => {
                // 章节联动（第三章·血仇得报已了门控）：文狱余波——血仇从来传代
                card.title = "血仇余响".into();
                card.text = "被斩宿敌的门生循着旧档寻上门，声言讨一个「说法」——文狱的账，从来传代。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.luck = 1.0;
                e1.msg = vec![
                    "你报出斩其师承的名号，来人脸色数变，长揖而退。道心+2，气运+1".into(),
                ];
                card.opts.push(("报出名号".into(), e1));
                let mut e2 = Eff::new();
                e2.stones_pct = -0.05;
                e2.good = false;
                e2.msg = vec![
                    "你闭门三日，门外的脚步声来去——走了，但账没清。灵石-5%".into(),
                ];
                card.opts.push(("闭门避锋".into(), e2));
            }
            75 => {
                // 章节联动（第六章·同页之缘已了且有伴门控）：灯节重逢，挂道侣立绘
                card.title = "同页灯约".into();
                let cn = {
                    let s = self.sim.as_ref().unwrap();
                    s.companion
                        .clone()
                        .unwrap_or_else(|| COMPANION_NAMES[0].to_string())
                };
                let ci = COMPANION_NAMES
                    .iter()
                    .position(|&n| n == cn)
                    .unwrap_or(0);
                card.actor = Some(CardActor::Comp(ci));
                card.text = format!(
                    "灯节，放灯的河堤人潮如织。你与{}各执一盏莲灯，灯面上不约而同写着同一个字。",
                    cn
                );
                let mut e1 = Eff::new();
                e1.luck = 3.0;
                e1.msg = vec!["两盏灯并排漂远，河面金光缠作一股。气运+3".into()];
                card.opts.push(("同放莲灯".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.msg = vec!["你们沿堤走到灯尽，把彼此的来路说了一遍。道心+2".into()];
                card.opts.push(("灯下叙旧".into(), e2));
            }
            76 => {
                // 链4 首段：灯楼谜约（宁清霜脸作灯楼主人；首段建链待分支）
                card.title = "灯楼谜约".into();
                card.actor = Some(CardActor::Npc(2));
                // v1.16.7 两段文案（邂逅版式 5 行内）：谜面独立成段
                card.text = "上元灯会，临水灯楼悬一盏无火自明的青灯，楼主人立灯下：「青灯有主三十载，只候一个能对上灯谜的人。」\n谜面只有一行小字：「写它的人不在了，读它的人还没来。」".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.06;
                e1.special = 28; // 置链：→77 灯下问答
                e1.msg = vec!["你道：「在我。」灯焰忽然一跳。修为+6%".into()];
                card.opts.push(("揭谜应约".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.msg = vec!["有些谜不是给此刻的你出的。你转身汇入灯流。道心+2".into()];
                card.opts.push(("折返离楼".into(), e2));
                self.chain = Some((4, 0, CHAIN_TTL)); // 首段建链：待分支
            }
            77 => {
                // 链4 中段（仅链内）：灯楼主人设问（复用宁清霜脸）
                card.title = "灯下问答·续".into();
                card.actor = Some(CardActor::Npc(2));
                card.text = "灯楼主人引你上楼，满架残卷蒙尘，唯第三层遮着素布：「布下是我未写完的书。你若问得三问，便替我揭了它。」".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.special = 29; // 置链：→78 青灯问藏
                e1.msg = vec!["你在素布前坐定，第一问从灯焰里升起。道心+2".into()];
                card.opts.push(("应主人之问".into(), e1));
                let g = (30.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!(
                    "你不问布下事，只与主人聊各地灯俗，临别赠你一盏小灯。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("只叙灯事".into(), e2));
            }
            78 => {
                // 链4 末段（仅链内）：special32 青灯三问检定臂收尾
                card.title = "青灯问藏·续".into();
                card.actor = Some(CardActor::Npc(2));
                card.text = "三问已毕，青灯灯火忽然大盛。主人手按素布：「最后一问没有答案——你自己的书，写到哪里了？」".into();
                let mut e1 = Eff::new();
                e1.special = 32;
                e1.good = false;
                card.opts.push(("受青灯三问".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 1.0;
                e2.msg = vec!["你说「写到下一章」，主人笑着为你揭了素布一角。道心+1".into()];
                card.opts.push(("只借书一晚".into(), e2));
            }
            79 => {
                // 链5 首段：傩面还愿（村野戏台无立绘；双分支皆推链照链1 special20/21）
                card.title = "傩面还愿".into();
                card.text = "村中傩班唱还愿戏，班主捧出一副彩面：「欠了笔墨债的，戴它跳一场『谢墨傩』，债便两清——你面相上，有债。」".into();
                let g = (25.0 * ss) as i64;
                let mut e1 = Eff::new();
                e1.luck = 1.0;
                e1.special = 30; // 置链：→80 戏毕面热
                e1.msg = vec!["面具覆脸的一瞬，鼓点自己追上了你的脚步。气运+1".into()];
                card.opts.push(("戴面代跳".into(), e1));
                let mut e2 = Eff::new();
                e2.stones = -g;
                e2.special = 31; // 置链：→81 愿成符赠
                e2.msg = vec![format!(
                    "你说「愿钱我出，戏跳给全村」——班主眼睛亮了。灵石-{}",
                    fmt_int(g)
                )];
                card.opts.push(("出钱贴班".into(), e2));
                self.chain = Some((5, 0, CHAIN_TTL)); // 首段建链：待分支
            }
            80 => {
                // 链5 分支a 段（仅链内）：两选项均无置链臂 → 抉择即链终局
                card.title = "戏毕面热·续".into();
                card.text = "一场傩戏跳罢，面具内里温热如体温。班主朝你长揖：「面里封着老辈一笔愿——你替他还了。」".into();
                let g = (45.0 * ss) as i64;
                let mut e1 = Eff::new();
                e1.stones = g;
                e1.luck = 2.0;
                e1.msg = vec![format!(
                    "谢仪之外，你觉出体内有一股旧愿化开的暖意。灵石+{}，气运+2",
                    fmt_int(g)
                )];
                card.opts.push(("收下谢仪".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 3.0;
                e2.msg = vec!["你把谢仪推了回去：「两清便好。」鼓声送你出村。道心+3".into()];
                card.opts.push(("只愿两清".into(), e2));
            }
            81 => {
                // 链5 分支b 段（仅链内）：两选项均无置链臂 → 抉择即链终局
                card.title = "愿成符赠·续".into();
                card.text = "戏台收场，班主取下傩面一角削成木符递你：「戏钱抵了愿钱，这符镇夜路。」".into();
                let mut e1 = Eff::new();
                e1.luck = 3.0;
                e1.msg = vec!["木符入手微温，夜行的狼嚎从此远了。气运+3".into()];
                card.opts.push(("佩符于身".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.luck = 1.0;
                e2.msg = vec![
                    "你把符塞给看戏入迷的稚童——愿这种东西，传下去才灵。道心+2，气运+1".into(),
                ];
                card.opts.push(("转赠孩童".into(), e2));
            }
            82 => {
                // 伴随事件：灯下缝页（缝页人设定题材；Comp 立绘照名反查模式）
                card.title = "灯下缝页".into();
                let cn = {
                    let s = self.sim.as_ref().unwrap();
                    s.companion
                        .clone()
                        .unwrap_or_else(|| COMPANION_NAMES[0].to_string())
                };
                let ci = COMPANION_NAMES
                    .iter()
                    .position(|&n| n == cn)
                    .unwrap_or(0);
                card.actor = Some(CardActor::Comp(ci));
                card.text = format!(
                    "灯芯剥到三寸，{}就着灯光缝补一页撕损的旧稿，针脚细密——「撕开的页，总要有人缝。」",
                    cn
                );
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.luck = 1.0;
                e1.msg = vec!["你把灯芯剔亮，针脚落得又稳又密。道心+2，气运+1".into()];
                card.opts.push(("帮掌灯".into(), e1));
                let mut e2 = Eff::new();
                e2.qi_pct = 0.05;
                e2.msg = vec!["缝完那页，你们把各自的来路絮了一遍。修为+5%".into()];
                card.opts.push(("絮话至深夜".into(), e2));
            }
            83 => {
                // 伴随事件：同摘霜橘（Comp 立绘）
                card.title = "同摘霜橘".into();
                let cn = {
                    let s = self.sim.as_ref().unwrap();
                    s.companion
                        .clone()
                        .unwrap_or_else(|| COMPANION_NAMES[0].to_string())
                };
                let ci = COMPANION_NAMES
                    .iter()
                    .position(|&n| n == cn)
                    .unwrap_or(0);
                card.actor = Some(CardActor::Comp(ci));
                card.text = format!(
                    "江南霜降，{}拎着竹篮拉你上橘山——霜后的灵橘最甜，早一刻酸，晚一刻鸟啄。",
                    cn
                );
                let mut e1 = Eff::new();
                e1.qi_pct = 0.06;
                e1.msg = vec!["霜橘入腹，一身寒气散尽。修为+6%".into()];
                card.opts.push(("就树分食".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.dao = 1.0;
                e2.msg = vec!["你们把橘核埋在向阳坡上，约好下一世来看。道心+1，气运+2".into()];
                card.opts.push(("留核试种".into(), e2));
            }
            84 => {
                // 伴随事件：论道起隙（无纯坏臂——僵局两解皆有得）
                card.title = "论道起隙".into();
                let cn = {
                    let s = self.sim.as_ref().unwrap();
                    s.companion
                        .clone()
                        .unwrap_or_else(|| COMPANION_NAMES[0].to_string())
                };
                let ci = COMPANION_NAMES
                    .iter()
                    .position(|&n| n == cn)
                    .unwrap_or(0);
                card.actor = Some(CardActor::Comp(ci));
                card.text = format!(
                    "论到「墨当写心还是写世」，你与{}各执一词，谁也说不服谁——灯花爆了三次，两人都没再开口。",
                    cn
                );
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.luck = 1.0;
                e1.msg = vec![
                    "你说「写心也好写世也罢，同页就好」——灯花又爆了一声，像是应了。道心+2，气运+1".into(),
                ];
                card.opts.push(("先开口和好".into(), e1));
                let mut e2 = Eff::new();
                e2.qi_pct = 0.06;
                e2.msg = vec![
                    "你们背对背各坐一夜，醒来时功法里各自多了对方那一路的意思。修为+6%".into(),
                ];
                card.opts.push(("各自静修".into(), e2));
            }
            // ---------- v1.16.9 扩池 15 条（id 85..=99）----------
            // 链6 血债血偿（85..=87）：宿敌剧情化复仇线——85 有宿敌才入池
            //（sim_event_weights 门控读 meta.neme），动态文案走 Nemes::full()
            // 既有格式化单口勿手拼；86/87 为链内段（权重 0），宿敌链中途可能被
            // sim_neme/寻仇反杀（neme.take()），具名走 Some、None 兜底泛称。
            // ⚠结算只用 Eff 通用字段：宿敌胜率公式已有 sim_neme:2937 与现实
            // revenge_p_of:7268 两处且注释明令勿三写，本链只述「探踪/设伏/夺积」，
            // 真对决仍走既有两口
            85 => {
                // 链6 首段：仇踪入册（动态宿敌名）
                card.title = "仇踪入册".into();
                let nfull = self
                    .meta
                    .neme
                    .as_ref()
                    .map(|n| n.full())
                    .unwrap_or_else(|| "删你字的血仇之敌".to_string());
                card.text = format!(
                    "茶棚歇脚，两个行脚商人压着嗓子议论{}——「近来招兵买马，看样子要办大事。」你摩挲着方印：血仇的名字，从来不需要想起。",
                    nfull
                );
                let mut e1 = Eff::new();
                e1.dao = 1.0;
                e1.special = 35; // 置链：→86 仇踪暗访
                e1.msg = vec!["你不动声色，把行踪记入旧册——来日方长。道心+1".into()];
                card.opts.push(("循踪暗记".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 1.0;
                e2.msg = vec!["血仇宜缓不宜急，你连夜收拾行装避开风口。气运+1".into()];
                card.opts.push(("当夜就走".into(), e2));
                self.chain = Some((6, 0, CHAIN_TTL)); // 首段建链：待分支
            }
            86 => {
                // 链6 中段（仅链内）：探虚实
                card.title = "仇踪暗访·续".into();
                let nname = self
                    .meta
                    .neme
                    .as_ref()
                    .map(|n| n.name.clone())
                    .unwrap_or_else(|| "那血仇之敌".to_string());
                card.text = format!(
                    "循旧册线索，你扮作佣工混进{}的营外围——粮道、岗哨、换防的时辰，一一入画。灶下一碗冷饭的工夫，进出这条路你已走了三遍。",
                    nname
                );
                let mut e1 = Eff::new();
                e1.luck = 1.0;
                e1.special = 36; // 置链：→87 乘虚夺积
                e1.msg = vec!["你散出「携重宝夜行」的假讯，专候其亲信出营。气运+1".into()];
                card.opts.push(("布饵设伏".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 1.0;
                e2.msg = vec!["虚实已探，来日再举。你抹去痕迹悄然退走。道心+1".into()];
                card.opts.push(("就此罢手".into(), e2));
            }
            87 => {
                // 链6 末段（仅链内）：趁虚夺积——通用 Eff 结算，无第三份胜率公式
                card.title = "乘虚夺积·续".into();
                let nname = self
                    .meta
                    .neme
                    .as_ref()
                    .map(|n| n.name.clone())
                    .unwrap_or_else(|| "那血仇之敌".to_string());
                card.text = format!(
                    "假讯生效，{}的亲信果然倾巢而出。你趁夜摸入主营，帐中只剩一只描金木匣——正是其积攒多年的私库。",
                    nname
                );
                let g0 = (90.0 * ss) as i64;
                let mut e1 = Eff::new();
                e1.stones = g0;
                e1.luck = 2.0;
                e1.msg = vec![format!(
                    "匣中灵石尽入行囊，你一把火燎了空帐——等他回营，只剩满地灰烬。灵石+{}，气运+2",
                    fmt_int(g0)
                )];
                card.opts.push(("取匣而走".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 3.0;
                // v1.16.15 复审口径：删「来日对决又高一分胜算」空头承诺——
                // sim_neme 胜率只看 power 比，布防图/道心无任何机制钩子
                // v1.16.20 链尾余味：与取匣臂「满地灰烬」撞像消解——不烧，原样放回，
                // 余味落在「他永远不知道自己漏在哪一夜」的留白上
                e2.msg = vec![
                    "你不取分文，只把布防图原样放回匣中——让他永远不知道，自己漏在哪一夜。道心+3".into(),
                ];
                card.opts.push(("毁其布防".into(), e2));
            }
            // 老年/将尽专属（88..=89）：寿元余量 ≤ELDER_LEFT_YEARS 才入池（weights
            // 门控）。措辞按「大限将至」而非精确年数——门控是当次入池判定，弹卡后
            // 寿元再变化（无面镜客/仙人赠礼）不撤卡
            88 => {
                card.title = "灯下了愿".into();
                card.text = "灯芯将尽。你把这些年欠下的愿一件件数过——有些还了，有些再不还就来不及了。\n今夜把灯挑亮，了结最后一桩。".into();
                let mut e1 = Eff::new();
                e1.stones_pct = -0.10;
                e1.dao = 4.0;
                e1.msg = vec![
                    "散尽浮财，了却旧愿。灯花爆了一声，你心头忽然轻了。灵石-10%，道心+4".into(),
                ];
                card.opts.push(("重金还愿".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.luck = 2.0;
                e2.msg = vec!["不必惊动旁人，对着一盏灯把话说尽。道心+2，气运+2".into()];
                card.opts.push(("只了心结".into(), e2));
            }
            89 => {
                card.title = "薪火相托".into();
                card.text = "山下少年仰着脸问：「先生，您这一身本事，往后传给谁？」\n你望着他，像望见很多年前那个没名没姓的自己。".into();
                let mut e1 = Eff::new();
                e1.dao = 3.0;
                e1.luck = 1.0;
                e1.msg = vec![
                    "你收他做了记名弟子，倾囊相授。道统有继，此生不算白来。道心+3，气运+1".into(),
                ];
                card.opts.push(("收徒传艺".into(), e1));
                let mut e2 = Eff::new();
                // v1.16.15 复审口径：旧稿结缘，后会有报——补气运+2 消解对收徒臂
                // 的严格支配（dao 少一分但多一线善缘）
                e2.dao = 2.0;
                e2.luck = 2.0;
                e2.msg = vec!["你把一部旧稿送他：「看得懂是缘，看不懂是命。」道心+2，气运+2".into()];
                card.opts.push(("赠稿不收徒".into(), e2));
            }
            // 高境界补充（90..=91）：mr 地板 EVENTS 第三元直写即生效（weights 内
            // realm<mr→0），补高境界普通层缺口——原 mr≥4 仅 15/26 两条稀有
            90 => {
                card.title = "雷池遗蜕".into();
                card.text = "雷池深处，一具渡劫失败的前辈遗蜕盘坐如生，周身雷纹未散，池底沉着半截焦雷之髓。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.12;
                e1.msg = vec![
                    "雷髓入体，经脉如遭雷洗——痛过之后，灵气行得比从前快。修为+12%".into(),
                ];
                card.opts.push(("取雷髓炼体".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.dao = 1.0;
                e2.msg = vec![
                    "你长揖一礼，敬而退走——前辈的道，不由外人取。气运+2，道心+1".into(),
                ];
                card.opts.push(("敬而退走".into(), e2));
            }
            91 => {
                card.title = "仙门论道".into();
                card.text = "仙门真传云游至此，与你论道三日。对方执仙家礼数，你以十万废稿为注——一场论下来，谁也没说服谁。".into();
                let mut e1 = Eff::new();
                e1.dao = 4.0;
                e1.msg = vec![
                    "真传捧着你一册旧稿良久，叹道：「此稿……不在仙家谱系里。」道心+4".into(),
                ];
                card.opts.push(("以稿为注".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 3.0;
                e2.msg = vec!["不立文字，一笑而别。气运+3".into()];
                card.opts.push(("论罢即别".into(), e2));
            }
            // 常规批次（92..=99）：题材查重对账见 data.rs 批注（茶棚/染坊/傀儡/
            // 煎茶/悬棺/鱼龙/盐井/矿洞/梦笔，均不撞既有 85 条）
            92 => {
                card.title = "染坊秘色".into();
                card.text = "镇尾染坊晾出一匹「会动的青」，布上水色随日光流转，像把去年的一段江装了进去。".into();
                let mut e1 = Eff::new();
                e1.luck = 2.0;
                e1.msg = vec![
                    "青布裁作里衣，行走时衣角总带着一点洗不掉的水汽。气运+2".into(),
                ];
                card.opts.push(("买布裁衣".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.msg = vec![
                    "老板娘说染的是「去年那条江的颜色」。你记下了这手艺。道心+2".into(),
                ];
                card.opts.push(("讨教染法".into(), e2));
            }
            93 => {
                card.title = "傀儡戏班".into();
                card.text = "傀儡戏班夜宿破庙，班主醉后哀叹：祖传三具傀儡，如今只有一具还会动——另两具，是「老了」。".into();
                let mut e1 = Eff::new();
                e1.luck = 2.0;
                e1.dao = 1.0;
                e1.msg = vec![
                    "你雇匠人补全丝线。开锣那夜台下只有你一个观众，班主却哭得像个孩子。气运+2，道心+1".into(),
                ];
                card.opts.push(("出钱修傀儡".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.msg = vec!["提线的手法暗合御物之术，你看了一整夜。道心+2".into()];
                card.opts.push(("向傀儡学艺".into(), e2));
            }
            94 => {
                card.title = "雪夜煎茶".into();
                card.text = "雪夜投宿无门，山亭里一位老者正煎雪水煮茶。见你来，他不多话，只把第二只盏推了过来。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.luck = 2.0;
                e1.msg = vec![
                    "一盏茶从雪落喝到雪停，你们谁也没问对方来路。道心+2，气运+2".into(),
                ];
                card.opts.push(("共饮一盏".into(), e1));
                let mut e2 = Eff::new();
                e2.qi_pct = 0.06;
                e2.msg = vec![
                    "老者说茶要「等水醒了再下」——你忽然懂了行功的火候。修为+6%".into(),
                ];
                card.opts.push(("讨问茶道".into(), e2));
            }
            95 => {
                card.title = "断崖悬棺".into();
                card.text = "断崖半腰悬着七具石棺，绳索早已朽断。风过时，棺中传出极轻的翻书声。".into();
                let g0 = (50.0 * ss) as i64;
                let mut e1 = Eff::new();
                e1.stones = g0;
                e1.dao = 3.0;
                e1.msg = vec![format!(
                    "第三具棺里只有一册无字书。你读完，书页散作灰。棺底殉葬的灵石归了你。道心+3，灵石+{}",
                    fmt_int(g0)
                )];
                card.opts.push(("攀崖探棺".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.msg = vec!["你在崖下焚香三拜。翻书声停了一瞬，像是谁应了。气运+2".into()];
                card.opts.push(("拜而去".into(), e2));
            }
            96 => {
                card.title = "春汛鱼龙".into();
                card.text = "春汛夜，江面忽然沸腾——成群的银鱼跃出水面，鳞光映月，像一场逆着下的星雨。".into();
                let g0 = (55.0 * ss) as i64;
                let mut e1 = Eff::new();
                e1.stones = g0;
                e1.luck = 1.0;
                e1.msg = vec![format!(
                    "渔村大丰，分你一担头鱼。灵石+{}，气运+1",
                    fmt_int(g0)
                )];
                card.opts.push(("张网助渔".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 3.0;
                e2.msg = vec![
                    "鱼跃龙门只是传说，可那一夜的江水，确实在往上流。道心+3".into(),
                ];
                card.opts.push(("临流观化".into(), e2));
            }
            97 => {
                card.title = "盐井卤灾".into();
                card.text = "盐井连日出黑卤，气味刺鼻。井户说井底「有东西翻身」，夜里，井口传来咕咚咕咚的响。".into();
                let mut e1 = Eff::new();
                e1.fight = 0.9; // 通用战斗乘区（sim_fight 单口；敌按自身战力缩放）
                e1.msg = vec!["你下井一探究竟——是福是祸，下去才知道。".into()];
                card.opts.push(("下井除患".into(), e1));
                let mut e2 = Eff::new();
                e2.stones_pct = -0.08;
                e2.luck = 1.0;
                e2.good = false;
                e2.msg = vec![
                    "花一笔封井钱，另凿新眼。破财免灾。灵石-8%，气运+1".into(),
                ];
                card.opts.push(("封井另凿".into(), e2));
            }
            98 => {
                card.title = "矿洞钟声".into();
                card.text = "废弃矿洞每逢雨夜自鸣钟声，声如闷雷。挖矿人说，洞底压着「不该醒的东西」。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.08;
                e1.injure = true;
                e1.msg = vec![
                    "你在钟腹刻下镇文，钟声从此哑了——代价是三天走不利索。修为+8%，受创".into(),
                ];
                card.opts.push(("入洞镇钟".into(), e1));
                let mut e2 = Eff::new();
                e2.stones_pct = -0.10;
                e2.good = false;
                e2.msg = vec!["你买断矿脉，封洞立碑。有些东西，不值得惊动。灵石-10%".into()];
                card.opts.push(("买断封洞".into(), e2));
            }
            99 => {
                card.title = "梦笔生花".into();
                card.text = "连着三夜梦见同一支笔。第四夜醒来，它就立在案头——笔身无字，笔尖却有墨，像等了很久。".into();
                let mut e1 = Eff::new();
                e1.dao = 3.0;
                e1.qi_pct = 0.05;
                e1.msg = vec![
                    "醒来案上多了一行字，不是你写的，却认得你的笔迹。道心+3，修为+5%".into(),
                ];
                card.opts.push(("提笔落墨".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.msg = vec!["来路不明的笔，你用镇纸压住。第三夜，它自己走了。气运+2".into()];
                card.opts.push(("不碰它".into(), e2));
            }

            // ---------- v1.16.10 扩池 18 条（id 100..=117）----------
            // 常规 10（100..=109）照 92..=99 词汇：通用 Eff 字段、ss 缩放灵石、
            // 战斗只走既有 sim_fight 单口。宿敌回响 2（110..=111）照 85 门控纪律、
            // 具名走 Some 兜底泛称（neme 中途可被 take，先例 game.rs 85 臂）。连锁 6
            // （112..=117）三段式照链2/3/4/6：首段入正常池弹卡即建链待分支，中/末段
            // 权重 0 仅链内直取，推进经 special 37..=40 新窗（CHAIN78_SPECIAL_LO/HI），
            // 末段抉择即「{名}·缘了」入大事记（stage 表见函数尾）
            100 => {
                card.title = "焚字炉".into();
                card.text = "城中焚字炉前，抄经人焚烧废旧经稿。一页稿纸在火里卷了三卷而不化，灰烬让开，显出一行小字。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.08;
                e1.dao = 2.0;
                e1.msg = vec![
                    "你自灰中抢出那页——是前人未写完的行功诀。修为+8%，道心+2".into(),
                ];
                card.opts.push(("灰中抢页".into(), e1));
                let g = (45.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!(
                    "你替抄经人翻检可卖的旧纸，分得一笔辛苦钱。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("帮检旧纸".into(), e2));
            }
            101 => {
                card.title = "笔冢祭".into();
                card.text = "山阴处一座笔冢，埋的都是写秃的笔。冢前石桌搁着一盏残灯，灯下压着半页祭文笔账——写账的人不知去了哪。".into();
                let mut e1 = Eff::new();
                e1.dao = 4.0;
                e1.msg = vec![
                    "你把灯挑亮，替他续完了那半页账。冢里似乎轻轻应了一声。道心+4".into(),
                ];
                card.opts.push(("续写残账".into(), e1));
                let g = (40.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.luck = 1.0;
                e2.msg = vec![format!(
                    "冢边秃笔堆里有几杆笔杆蕴着旧灵气，识货人收。灵石+{}，气运+1",
                    fmt_int(g)
                )];
                card.opts.push(("捡拾秃笔".into(), e2));
            }
            102 => {
                card.title = "涸河龙祠".into();
                card.text = "河床干了三年，龙祠香火早断。村民为最后一眼活泉争得头破血流，祠祝拦都拦不住。".into();
                let mut e1 = Eff::new();
                e1.dao = 3.0;
                e1.luck = 2.0;
                e1.msg = vec![
                    "你以灵力探得泉眼另脉，引水分流——两村各得一半。道心+3，气运+2".into(),
                ];
                card.opts.push(("引泉均分".into(), e1));
                let g = (55.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.good = false;
                e2.msg = vec![format!(
                    "你袖手旁观，顺手收了两村「评理钱」。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("袖手评理".into(), e2));
            }
            103 => {
                card.title = "纸鸢传书".into();
                card.text = "一只断线纸鸢栽在你院里，鸢尾系着半幅血书：山那头采药人坠了涧，字迹潦草，只写了个大概方位。".into();
                let mut e1 = Eff::new();
                e1.dao = 4.0;
                e1.luck = 1.0;
                e1.msg = vec![
                    "你循书寻人，负他出涧。采药人无以为报，只说「欠你一命」。道心+4，气运+1".into(),
                ];
                card.opts.push(("循书救人".into(), e1));
                let g = (30.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.good = false;
                e2.msg = vec![format!(
                    "你把血书交给驿卒充公，换了几文酒钱。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("交驿充公".into(), e2));
            }
            104 => {
                card.title = "砚山叩泉".into();
                card.text = "一座山形如巨砚，叩击山壁的不同方位，石缝里涌出的泉水竟带着墨香——饮之，识海清明。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.12;
                e1.msg = vec![
                    "你叩了整夜，寻得涌墨的泉眼，饱饮一腹。修为+12%".into(),
                ];
                card.opts.push(("叩山取泉".into(), e1));
                let g = (60.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.dao = 1.0;
                e2.msg = vec![format!(
                    "你拓下山壁叩痕售予坊市，人称「砚山叩谱」。灵石+{}，道心+1",
                    fmt_int(g)
                )];
                card.opts.push(("拓售叩谱".into(), e2));
            }
            105 => {
                card.title = "古戍点兵".into();
                card.text = "废戍楼在月圆之夜自燃烽火，楼下空地影影绰绰列队点兵——都是千年前战死者的影子，操练至今。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.10;
                e1.injure = true;
                e1.msg = vec![
                    "你入阵随操练一夜，旧的杀阵章法入了骨——也伤了筋。修为+10%，受创".into(),
                ];
                card.opts.push(("入阵随操".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 3.0;
                e2.good = false;
                e2.msg = vec![
                    "你在楼下焚香一拜，替他们收操。影队缓缓散去，天亮了。道心+3".into(),
                ];
                card.opts.push(("焚香收操".into(), e2));
            }
            106 => {
                card.title = "天蚕茧室".into();
                card.text = "整座洞窟挂满天蚕茧，万千白茧幽幽吐光。传说万茧之中有一枚「书茧」——茧里裹着蚕读了千年的字。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.15;
                e1.dao = 3.0;
                e1.msg = vec![
                    "你赌中那枚书茧——茧丝织成一行小字，入眼即化。修为+15%，道心+3".into(),
                ];
                card.opts.push(("择茧参悟".into(), e1));
                let g = (90.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!(
                    "你替蚕主收茧整茧，工钱以外分得些头茬茧。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("帮工收茧".into(), e2));
            }
            107 => {
                card.title = "空舟拦江".into();
                card.text = "一艘官船无人自横，拦在江心。舱内桌椅齐整，茶犹温，只有一册翻到一半的无字书——读的人像是一去不回了。".into();
                let mut e1 = Eff::new();
                e1.dao = 3.0;
                e1.luck = 2.0;
                e1.msg = vec![
                    "你把书合上，供在案头，推船让出航道。行船人皆道今日江神让路。道心+3，气运+2".into(),
                ];
                card.opts.push(("供书让道".into(), e1));
                let g = (80.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.good = false;
                e2.msg = vec![format!(
                    "你收拾舱中值钱物件，报官领了半船赏钱。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("报官领赏".into(), e2));
            }
            108 => {
                // 稀有：墨鲸背城（w2）
                card.title = "墨鲸背城".into();
                card.text = "海雾散开，一头墨色巨鲸浮出水面换气——鲸背上竟驮着一座小城，城郭灯火，街市隐隐有人声。".into();
                let mut e1 = Eff::new();
                e1.dao = 5.0;
                e1.luck = 3.0;
                e1.msg = vec![
                    "鲸城住民尽是前朝避世的书生。你登城半日，借得满城书香。道心+5，气运+3".into(),
                ];
                card.opts.push(("登城借书".into(), e1));
                let g = (120.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.good = false;
                e2.msg = vec![format!(
                    "你只在鲸背边缘拾得城中抛下的旧物，售得灵石+{}——上城的路，终究没敢走。",
                    fmt_int(g)
                )];
                card.opts.push(("缘边拾遗".into(), e2));
            }
            109 => {
                card.title = "借名修行".into();
                card.text = "山下发馆收徒的散修，顶着你的名号开坛讲法——讲得似是而非，偏生徒众不少。".into();
                let mut e1 = Eff::new();
                e1.dao = 3.0;
                e1.luck = 1.0;
                e1.msg = vec![
                    "你登门只谈了一场，散修面红过耳，当众正名焚了假牒。道心+3，气运+1".into(),
                ];
                card.opts.push(("登门正名".into(), e1));
                let g = (70.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.good = false;
                e2.msg = vec![format!(
                    "你收下一笔「名号分润」，由他顶着去。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("收润默许".into(), e2));
            }
            110 => {
                // 宿敌同世回响：链6 缘了（chain6_closed）才入池；具名 Some 兜底泛称
                card.title = "夺积余波".into();
                let nname = self
                    .meta
                    .neme
                    .as_ref()
                    .map(|n| n.name.clone())
                    .unwrap_or_else(|| "那血仇之敌".to_string());
                card.text = format!(
                    "夺积之夜过后，{}果然倾巢而出寻账——营帐外火把连成一线，把你的藏身谷围成一口锅。",
                    nname
                );
                let mut e1 = Eff::new();
                e1.fight = 1.05; // 通用战斗单口 sim_fight（勿另写胜率公式）
                e1.good = false;
                card.opts.push(("据险而战".into(), e1));
                let mut e2 = Eff::new();
                e2.stones_pct = -0.12;
                e2.good = false;
                e2.msg = vec![
                    "你提前散尽劫来的浮财，账无凭据，围了三日自散。灵石-12%".into(),
                ];
                card.opts.push(("散财消账".into(), e2));
            }
            111 => {
                // 宿敌跨世回响：neme_slain≥1 才入池——新宿敌登册的余波
                card.title = "仇册新墨".into();
                let nfull = self
                    .meta
                    .neme
                    .as_ref()
                    .map(|n| n.full())
                    .unwrap_or_else(|| "删你字的血仇之敌".to_string());
                card.text = format!(
                    "夜里旧册无风自动，血仇的名字上新添了一行墨——{}.方印上被划掉的痕迹，又深了一层。",
                    nfull
                );
                let mut e1 = Eff::new();
                e1.dao = 3.0;
                e1.msg = vec![
                    "你把新名字念了一遍，念得极慢——记仇这件事，你越来越熟练了。道心+3".into(),
                ];
                card.opts.push(("默记新名".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.msg = vec![
                    "你合上旧册，就灯磨墨——来日方长，先把今日过好。气运+2".into(),
                ];
                card.opts.push(("合册安睡".into(), e2));
            }
            112 => {
                // 链7 首段：灯花渡舟（首段建链待分支）
                card.title = "灯花渡舟".into();
                card.text = "起大雾的夜，渡口只有一条不点篙的船。老船夫说：「雾河渡人不收灵石，只收旧渡资——讲一段你记得的旧事，灯花替你付账。」".into();
                let mut e1 = Eff::new();
                e1.luck = 1.0;
                e1.special = 37; // 置链：→113 灯下旧账
                e1.msg = vec!["你讲了一段。船离岸，雾自动，灯花爆了第一声。气运+1".into()];
                card.opts.push(("以旧事为资".into(), e1));
                let g = (40.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = -g;
                e2.dao = 1.0;
                // v1.16.15 复审口径：正文立规「雾河渡人不收灵石」——船夫分文不取，
                // 灵石是你自己压在渡口石上走的（损失自担），规矩不破
                e2.msg = vec![format!(
                    "你把灵石压在渡口石上：「我不讲故事。」老船夫摇头拢船自去——石上灵石，你连头也没回。灵石-{}，道心+1",
                    fmt_int(g)
                )];
                card.opts.push(("付石不语".into(), e2));
                self.chain = Some((7, 0, CHAIN_TTL)); // 首段建链：待分支
            }
            113 => {
                // 链7 中段（仅链内）
                card.title = "灯下旧账·续".into();
                card.text = "行至河心，你借灯看清了舱壁——挂满一船灯纸，每张都写着一段旧事。其中一张的字迹，你从未写过，却认得是你自己的。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.special = 38; // 置链：→114 一渡一灯
                e1.msg = vec!["你问：「这页是谁讲的？」老船夫笑而不答，只添了点灯油。道心+2".into()];
                card.opts.push(("追问灯纸".into(), e1));
                let g = (35.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = -g; // v1.16.10 复审修：买灯是花销，与卡面「灵石-{}」同号（照 112 opt2 同款）
                e2.luck = 1.0;
                e2.msg = vec![format!(
                    "到岸了，你不多问，买下一盏灯带走。灵石-{}，气运+1",
                    fmt_int(g)
                )];
                card.opts.push(("买灯登岸".into(), e2));
            }
            114 => {
                // 链7 末段（仅链内）：两选项均无置链臂 → 抉择即链终局（缘了）
                card.title = "一渡一灯·续".into();
                card.text = "老船夫终于开口：「灯上记的，都是天曹删掉的名字——删一笔，我渡一程。船到岸了，客官，你的那页，想怎么写？」".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.10;
                e1.dao = 3.0;
                e1.msg = vec![
                    "你借灯火把那页纸点亮：「接着写。」满船灯纸齐齐亮了一瞬。修为+10%，道心+3".into(),
                ];
                card.opts.push(("点灯续写".into(), e1));
                let g = (75.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.luck = 2.0;
                e2.msg = vec![format!(
                    "你收下老船夫赠的一盏灯：「写完再来讲给你。」灵石+{}，气运+2",
                    fmt_int(g)
                )];
                card.opts.push(("携灯而归".into(), e2));
            }
            115 => {
                // 链8 首段：药炉童子（首段建链待分支）
                card.title = "药炉童子".into();
                card.text = "荒废丹房的炉火忽然自己着了。守炉的童子拉住你——看不出年岁的一张小脸：「先生救命，炉要走了！它守着一味丹引，等一个肯读它的人。」".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.05;
                e1.special = 39; // 置链：→116 炉语问心
                e1.msg = vec!["你留下守炉。炉火伏低了一瞬，像行了个礼。修为+5%".into()];
                card.opts.push(("留下守炉".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 1.0;
                e2.msg = vec![
                    "你拂袖而去——来历不明的炉火，不沾。道心+1".into(),
                ];
                card.opts.push(("拂袖而去".into(), e2));
                self.chain = Some((8, 0, CHAIN_TTL)); // 首段建链：待分支
            }
            116 => {
                // 链8 中段（仅链内）
                card.title = "炉语问心·续".into();
                card.text = "守到半夜，炉火忽明忽暗，竟像在说话。童子垂首：「炉主走火那晚，把一页烧残的稿子当了丹引咽下去。这炉等这一夜，等了一百年。」".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.special = 40; // 置链：→117 守炉一夜
                e1.msg = vec!["你俯身问炉：「你要的是炉，还是那页稿？」火苗猛地蹿高三寸。道心+2".into()];
                card.opts.push(("问炉要什么".into(), e1));
                let g = (30.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!(
                    "你只管添柴控火，天亮领了童子的谢银。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("添柴到天明".into(), e2));
            }
            117 => {
                // 链8 末段（仅链内）：两选项均无置链臂 → 抉择即链终局（缘了）
                card.title = "守炉一夜·续".into();
                card.text = "天将亮时炉门自开——炉中无丹，只有一页完稿，墨迹如新。童子拜了三拜：「炉主的名字有人接了。炉可以冷了。」".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.15;
                e1.dao = 3.0;
                e1.msg = vec![
                    "你收稿参悟，百年炉火一字入心。修为+15%，道心+3".into(),
                ];
                card.opts.push(("收稿参悟".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 4.0;
                e2.luck = 2.0;
                e2.msg = vec![
                    "你把稿子折好，送回炉膛——有些稿子烧了才是写完了。道心+4，气运+2".into(),
                ];
                card.opts.push(("还稿于炉".into(), e2));
            }
            // v1.16.11 链9 循册寻仇（宿敌终局对决线，118..=120）：斩宿敌之后其门下
            // 循册寻仇——与链6 成因果闭环（链6 述「探踪/设伏/夺积」，真斩杀走
            // sim_neme/寻仇两口，本链述其后果）。被斩者已不在册（meta.neme 是新
            // 宿敌），全程泛称不具名，避免误指
            118 => {
                // 链9 首段：血册投书（首段建链待分支）
                card.title = "血册投书".into();
                card.text = "雨夜有物钉门——半页旧册残页，朱笔圈着你斩落的那位血仇之敌的名讳，末行一行新墨：「师仇未雪，门下递呈。」落款七个名字，按入门次序排着。".into();
                let mut e1 = Eff::new();
                e1.dao = 1.0;
                e1.special = 41; // 置链：→119 雪夜拦道
                e1.msg = vec!["「三日后，断名崖。」你只回了这六个字。道心+1".into()];
                card.opts.push(("应下战书".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.luck = 1.0;
                e2.msg = vec![
                    "你把残页凑上烛火：「你的账，随你师父一起了了。」灰烬里最末一个名字始终没有显形。道心+2，气运+1".into(),
                ];
                card.opts.push(("焚册不接".into(), e2));
                self.chain = Some((9, 0, CHAIN_TTL)); // 首段建链：待分支
            }
            119 => {
                // 链9 中段（仅链内）
                card.title = "雪夜拦道·续".into();
                card.text = "你没等三日。雪夜兼程走到半途，山道上一列灯笼拦路——七人皆缟素，为首的捧着那半页册子：「师叔之仇，按门规由我等分摊。先生，请了。」".into();
                let mut e1 = Eff::new();
                e1.luck = 1.0;
                e1.special = 42; // 置链：→120 断名之局
                e1.msg = vec!["你把七盏灯一一吹熄：「一个一个来，还是一起上？」雪落无声。气运+1".into()];
                card.opts.push(("逐一应名".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 1.0;
                e2.msg = vec![
                    "你转身入雪——杀过一次的人，不必向再一场血证什么。道心+1".into(),
                ];
                card.opts.push(("雪中折返".into(), e2));
            }
            120 => {
                // 链9 末段（仅链内）：两选项均无置链臂 → 抉择即链终局（缘了）
                card.title = "断名之局·续".into();
                card.text = "断名崖下，风把雪吹成一面白幡。为首者递出那半页册：「赢了你撕，输了我等收殓。」你认得他握剑的手势——与他师父，一个师门教出来的。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.10;
                e1.dao = 3.0;
                e1.msg = vec![
                    "多年记仇磨出来的那一剑，比门规快。册页你撕了，名字断在崖底。修为+10%，道心+3".into(),
                ];
                card.opts.push(("一剑断名".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 4.0;
                e2.msg = vec![
                    "你收剑，把残页递还：「账清了，册子留给你——记着怎么别走到这一步。」道心+4".into(),
                ];
                card.opts.push(("递册两清".into(), e2));
            }
            // v1.16.11 链10 旧页回声（跨世情缘回访，121..=123）：历世结缘过同页人
            // 才入池——沿书/纸/墨基因述「旧稿散页跨世回流」，与章节书/链7 的灯纸
            // 天曹口径互不相涉（回声只关乎「你写过的字」）
            121 => {
                // 链10 首段：旧页回声（首段建链待分支）
                card.title = "旧页回声".into();
                card.text = "灯市书摊，一页残稿压在《千字文》下——字迹是你自己的，你却从未写过：那是某一世里，你写给谁的半行。「客官好眼力，这页等一个认得它的人，等了很多年。」".into();
                let mut e1 = Eff::new();
                e1.luck = 1.0;
                e1.special = 43; // 置链：→122 灯市换页
                e1.msg = vec!["「从哪儿收来的？」你按住那页纸。摊主笑而不答，朝灯市深处努了努嘴。气运+1".into()];
                card.opts.push(("循墨寻摊".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 1.0;
                e2.msg = vec![
                    "你把残页折进袖中——有些字，不必问来历。道心+1".into(),
                ];
                card.opts.push(("收页入怀".into(), e2));
                self.chain = Some((10, 0, CHAIN_TTL)); // 首段建链：待分支
            }
            122 => {
                // 链10 中段（仅链内）
                card.title = "灯市换页·续".into();
                card.text = "灯市最深处果然有个不挂灯的摊子。摊主披蓑戴笠，案上摞着几百页旧稿，各朝各代的纸色都有：「只换不卖——一页旧稿，换我一页新墨。」".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.special = 44; // 置链：→123 换页重逢
                e1.msg = vec!["你把袖中残页铺开。摊主指尖一顿：「原来在这儿。」——他认得这页。道心+2".into()];
                card.opts.push(("以页换页".into(), e1));
                let g = (50.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = -g; // v1.16.11 复审口径：赎页是花销，与卡面「灵石-{}」同号（照 112/113 同款）
                e2.luck = 1.0;
                e2.msg = vec![format!(
                    "你不肯交出残页，压下一笔灵石只借看一夜。灵石-{}，气运+1",
                    fmt_int(g)
                )];
                card.opts.push(("以石借阅".into(), e2));
            }
            123 => {
                // 链10 末段（仅链内）：两选项均无置链臂 → 抉择即链终局（缘了）
                card.title = "换页重逢·续".into();
                // v1.16.16 具名升级：历世结缘次数最多的同页人（comp_bonds 反查
                // COMPANION_NAMES，零新键）——「某一世里与你同页的人」泛称升具名，
                // 情感密度落地。兜底纪律照 85 臂：门控 121 已保证 comp_bonds 任意
                // >0，此处下标仍 min 钳制防御（panic=abort 纪律）。
                // v1.16.17 反查式抽 best_bond_idx 单源（道侣专页缘谱「旧页回声所指」
                // 同式共用；std max_by_key 并列取末语义单测钉死，行为零变化）
                let best_bn = best_bond_idx(&self.meta.comp_bonds).min(COMPANION_NAMES.len() - 1);
                let cn = COMPANION_NAMES[best_bn];
                card.text = format!(
                    "摊主取出的那页新墨，背面洇着一行小字批注——笔迹你认得：{}。摊主说：「{}也在找这页。找了很多年。」",
                    cn, cn
                );
                let mut e1 = Eff::new();
                e1.dao = 3.0;
                e1.luck = 2.0;
                e1.msg = vec![
                    "你把批注与原稿合在一处，请摊主裱作一页。隔世的字迹终于挨在一起。道心+3，气运+2".into(),
                ];
                card.opts.push(("批注合璧".into(), e1));
                let mut e2 = Eff::new();
                e2.qi_pct = 0.10;
                e2.dao = 2.0;
                e2.msg = vec![
                    "你把两页各自收好——重逢不必在场，字在，人就还在。修为+10%，道心+2".into(),
                ];
                card.opts.push(("各页各藏".into(), e2));
            }
            // v1.16.11 常规补池 10 条（124..=133）：mr 薄档优先（原 mr=4 仅 6 条/
            // mr=5 仅 3 条——本批 mr4×3/mr5×1/mr3×2/mr2×3/mr1×1，v1.16.13 复审⑩
            // 修正原注 mr2×2 笔误），题材避开既有
            // 全部 118 条：断弦琴/龙窑夜火/埙/枯禅遗偈/兽潮/玉贩/引魂幡/磁山/
            // 天梯/断虹，其中稀有 1（133 断虹饮涧 w=2）。查重对账见本批次施工单
            124 => {
                card.title = "断弦琴意".into();
                card.text = "山亭里一把无主的旧琴，七弦断其三。亭柱刻着半阙未完的曲名。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.msg = vec![
                    "你取马尾续弦，就着断谱把那半阙弹完了。山风替你鼓了一回掌。道心+2".into(),
                ];
                card.opts.push(("续弦成曲".into(), e1));
                let g = (45.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!(
                    "你抱琴下山，坊市识货的琴师出了个公道价。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("抱琴换钱".into(), e2));
            }
            125 => {
                card.title = "百窑夜火".into();
                card.text = "山坳龙窑夜半无火自燃，窑工跪了一地不敢近——窑头说，这窑三年前封的那一炉，「回来了」。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.05;
                e1.luck = 1.0;
                e1.msg = vec![
                    "你贴窑听了半宿，起出一匣遇月自亮的釉灰——带回研墨，笔下的字都稳了三分。修为+5%，气运+1".into(),
                ];
                card.opts.push(("探窑起釉".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 1.0;
                e2.msg = vec![
                    "「火有主，不扰。」你帮窑工把窑门封土垒严。道心+1".into(),
                ];
                card.opts.push(("封土还窑".into(), e2));
            }
            126 => {
                card.title = "埙声穿垣".into();
                card.text = "断垣深处传出埙声，呜咽着反复同一个调子。走近了看——垣心供着一具石埙，无人而自鸣。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.msg = vec![
                    "你捧起石埙，把它没吹完的尾音吹完了。埙声停了，像卸下了什么。道心+2".into(),
                ];
                card.opts.push(("续完尾音".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 1.0;
                e2.luck = 1.0;
                e2.msg = vec![
                    "「曲有主。」你把碎垣拢回原位，替它挡了夜风。道心+1，气运+1".into(),
                ];
                card.opts.push(("拢垣掩埙".into(), e2));
            }
            127 => {
                card.title = "枯禅遗偈".into();
                card.text = "崖洞里一位坐化的老僧，壁间遗偈只写了上半——「我于人间留一问」。墨迹已枯了不知多少年。".into();
                let mut e1 = Eff::new();
                e1.dao = 3.0;
                e1.msg = vec![
                    "你以指为笔续了下半。搁手时，壁上枯了一甲子的野花无声开了。道心+3".into(),
                ];
                card.opts.push(("续偈作答".into(), e1));
                let g = (40.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.good = false;
                e2.msg = vec![format!(
                    "你把半偈拓了下来，转手卖给藏家——「卖字这件事，你越来越熟练了。」灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("拓偈易石".into(), e2));
            }
            128 => {
                card.title = "兽潮围栅".into();
                card.text = "灵兽潮涌自山脊压下，围了猎户的山栅。栅内孩童哭声一片，栅外兽瞳在暮色里连成一片星。".into();
                let mut e1 = Eff::new();
                e1.fight = 1.05; // 通用战斗单口 sim_fight（勿另写胜率公式）
                e1.good = false;
                card.opts.push(("并肩守栅".into(), e1));
                let mut e2 = Eff::new();
                e2.stones_pct = -0.08;
                e2.good = false;
                e2.msg = vec![
                    "你割了行囊里的干粮肉脯引开兽头，栅民得全——你贴身的灵石撒了一路。灵石-8%".into(),
                ];
                card.opts.push(("弃饵引潮".into(), e2));
            }
            129 => {
                card.title = "玉贩困隘".into();
                card.text = "雪隘口的坡道塌了半边，一队玉贩的车马困在崖沿。领队的朝你长揖：「先生过路，搭把手，玉随您挑。」".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.luck = 1.0;
                e1.msg = vec![
                    "你拽车上坡、垫道填辙，忙完只讨了一碗热汤。玉贩硬塞给你一枚暖玉。道心+2，气运+1".into(),
                ];
                card.opts.push(("援手不取".into(), e1));
                let g = (55.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.good = false;
                e2.msg = vec![format!(
                    "你按行价收了酬金才动手——钱货两讫，领队道谢时眼神发苦。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("收酬让道".into(), e2));
            }
            130 => {
                card.title = "引魂幡影".into();
                card.text = "荒岭上一杆白幡夜夜招魂，幡影里浮着些看不清的影子，来往客商绕着走。".into();
                let mut e1 = Eff::new();
                e1.dao = 3.0;
                e1.good = false;
                e1.msg = vec![
                    "你一剑断幡。幡下没有魂，只有半册招魂经——操幡的人借幡养念，念断了。道心+3".into(),
                ];
                card.opts.push(("断幡散念".into(), e1));
                let mut e2 = Eff::new();
                e2.qi_pct = 0.08;
                e2.msg = vec![
                    "你随幡影走了一程——它只是引你到一处避风崖洞，天明自散。修为+8%".into(),
                ];
                card.opts.push(("随幡借宿".into(), e2));
            }
            131 => {
                card.title = "磁山迷辙".into();
                card.text = "磁山吸针，车马的指北针在这里全成了摆设——一队商旅在山坳里兜了两天，辙印绕成一团乱麻。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.msg = vec![
                    "你闭目听了半刻地脉，抬手一指：「从西坳走。」商队出山那天，给你烧了三炷高香。道心+2".into(),
                ];
                card.opts.push(("循脉指路".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 1.0;
                e2.msg = vec![
                    "你弃了车马绕山而行，多走三日——慢是慢了点，稳。气运+1".into(),
                ];
                card.opts.push(("绕山缓行".into(), e2));
            }
            132 => {
                card.title = "天梯断级".into();
                card.text = "通天石梯到此断了一级，断口平整如切——梯上仙气粼然，梯下望不见底。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.12;
                e1.dao = 2.0;
                e1.msg = vec![
                    "你斩雷击木为材，补上断级。梯身嗡鸣如谢，你拾级而上如沐春风。修为+12%，道心+2".into(),
                ];
                card.opts.push(("补级而上".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.msg = vec![
                    "你退后半步，纵身跃过缺口——有些关隘，问都不必问。气运+2".into(),
                ];
                card.opts.push(("跃过缺口".into(), e2));
            }
            133 => {
                card.title = "断虹饮涧".into();
                card.text = "雨后断虹垂涧，虹脚没入一汪积水——水色映出的天，是你从未见过的那一副。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.15;
                e1.luck = 2.0;
                e1.msg = vec![
                    "你俯身照水，把那副天色看了个饱。起身时虹散了，那汪水却再没干过。修为+15%，气运+2".into(),
                ];
                card.opts.push(("循虹照水".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.msg = vec![
                    "「虹是借来的景。」你看完就走，不惊动它。道心+2".into(),
                ];
                card.opts.push(("守心不扰".into(), e2));
            }
            // v1.16.12 洞府经营联动 3 条：名下设施满级的叙事回响（门控在
            // sim_event_weights 读 meta.dongfu 跨世持久态）。措辞一律「你名下/
            // 历年经营」口径（跨世持久，非「本世」所得——照 111/110 措辞纪律）
            134 => {
                card.title = "阵眼共鸣".into();
                card.text = "你名下洞府的聚灵阵历年养到三重圆满——今晨山河灵潮过境，阵眼无风自鸣，灵机如细溪汇入你的吐纳。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.12;
                e1.luck = 1.0;
                e1.msg = vec![
                    "你循阵纹吐纳一昼夜，灵潮与阵眼相和共振，经络里暖意不散。修为+12%，气运+1".into(),
                ];
                card.opts.push(("循阵吐纳".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.msg = vec![
                    "你不取灵机，只静观阵纹明灭——历年布阵经营的功夫，今日在悟性里回响。道心+2".into(),
                ];
                card.opts.push(("静观阵纹".into(), e2));
            }
            135 => {
                let g = (120.0 * ss * gen_range(1.0, 1.6)) as i64;
                card.title = "田畦岁稔".into();
                card.text = "你名下灵田垦到三等沃腴——岁末灵稻垂穗，田埂雾气未散，山下的散修都来讨一捧新米尝鲜。".into();
                let mut e1 = Eff::new();
                e1.stones = g;
                e1.msg = vec![format!(
                    "你售了三成新谷，换来的灵石比往年厚了一倍有余。获得灵石 {}",
                    fmt_int(g)
                )];
                card.opts.push(("售谷换石".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.msg = vec![
                    "你开仓施粥，散修叩谢而去——来年山道上落下的善意，都记在这碗新米里。气运+2".into(),
                ];
                card.opts.push(("开仓施粥".into(), e2));
            }
            136 => {
                card.title = "丹房遗香".into();
                card.text = "你名下丹房历年扩到三重鼎室——开炉取丹时，砖缝里渗出经年的丹香，凝而不散，闻之神思一清。".into();
                let mut e1 = Eff::new();
                e1.pill = Some((0, 1));
                e1.qi_pct = 0.08;
                e1.msg = vec![
                    "鼎室夹层里翻出往年窖藏的一枚聚气丹，就着丹香服下，药力格外醇和。修为+8%，聚气丹+1".into(),
                ];
                card.opts.push(("取窖藏丹".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.msg = vec![
                    "你把三重鼎室细细扫了一遍，火路在指下过了一遭——历年丹火的脾性，你摸得更熟了。道心+2".into(),
                ];
                card.opts.push(("扫炉悟火".into(), e2));
            }

            // ---------- v1.16.13 扩池 14 条（id 137..=150）----------
            // 链11 雷劫侧写（137..=139 劫痕渡客）：邻山修士渡劫的旁观线——全程
            // 只写劫云过境/劫灰余韵侧写，绝不触玩家自身破境裁决（与问笔卡特化
            // special13/14/15/16 分域，见 2747-2802 注），题材避开 24/90/132 既有
            // 雷象三题。置链臂 45/46 第五窗（CHAIN1112_SPECIAL_LO/HI），末段抉择
            // 即「劫痕渡客·缘了」入大事记（stage 表见函数尾）。入池门控零新代码：
            // mr=4 由 sim_event_weights 既有 realm<mr 归零承担
            137 => {
                // 链11 首段：劫云过境（首段建链待分支）
                card.title = "劫云过境".into();
                card.text = "山口茶棚歇脚，邻山之巅劫云盘结——云底雷光垂而未落，云下有遁光不隐不避。老板娘照常添水：「第三位了。渡过去的，下山都来喝碗茶。」".into();
                let mut e1 = Eff::new();
                e1.dao = 1.0;
                e1.special = 45; // 置链：→138 劫灰拾遗
                e1.msg = vec!["你讨了壶粗茶坐下。雷声过处，满棚人声都低了——茶碗里的水纹一圈圈往外推。道心+1".into()];
                card.opts.push(("留棚观劫".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 1.0;
                e2.msg = vec![
                    "劫是别人的功课。你结了茶钱绕后山走，走出十里才听见第一声雷。气运+1".into(),
                ];
                card.opts.push(("结账绕行".into(), e2));
                self.chain = Some((11, 0, CHAIN_TTL)); // 首段建链：待分支
            }
            138 => {
                // 链11 中段（仅链内）
                card.title = "劫灰拾遗·续".into();
                card.text = "劫云散时你行至山腰回望——那道遁光顶着末重雷落了下来，山火燎出一道焦痕，雨一浇，白烟像香。次日你循焦痕拾级，雷火淬过的矿壳沿途散落，拾一枚还在嗡。".into();
                let mut e1 = Eff::new();
                e1.luck = 1.0;
                e1.special = 46; // 置链：→139 劫后茶话
                e1.msg = vec!["焦痕尽头，渡劫客衣袍半焦人却笑着，请你喝劫后酒：「替我数到七的，是不是你？」气运+1".into()];
                card.opts.push(("探访劫者".into(), e1));
                let g = (50.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!(
                    "矿壳是天地劈过的余响，坊市符师收作符材。你拾了七枚，一枚不多拾。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("只拾矿壳".into(), e2));
            }
            139 => {
                // 链11 末段（仅链内）：两选项均无置链臂 → 抉择即链终局（缘了）
                card.title = "劫后茶话·续".into();
                card.text = "渡劫客伤愈将行，临别与你就着茶棚炉火对坐。他说雷劫劈的不是人，是人身上的侥幸——劈完了，剩下的那部分才真正是自己的。老板娘在旁听了一路，给两人都续了茶。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.10;
                e1.dao = 3.0;
                e1.msg = vec![
                    "你把这句话记进随身的册子。归程一路吐纳，念头上再无侥幸可躲。修为+10%，道心+3".into(),
                ];
                card.opts.push(("记下这句话".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.luck = 2.0;
                e2.msg = vec![
                    "你送他到山口。临别他把酒囊塞给你：「劫是各人的，茶是同的。」道心+2，气运+2".into(),
                ];
                card.opts.push(("送他一程".into(), e2));
            }
            // 链12 书肆换届（140..=142）：书肆易主与代笔线——与焚字炉/笔冢祭/旧页
            // 回声的「稿」基因分域（本链只关乎「卖书的人」与「认字的人」），置链臂
            // 47/48 同第五窗
            140 => {
                // 链12 首段：书肆易主（首段建链待分支）
                card.title = "书肆易主".into();
                card.text = "城南书肆挂出「易主」木牌。老掌柜正把架上旧抄本一摞摞装箱，见你翻得仔细，摆手：「随便看。认字的人来了，书就不算白搬一趟。」".into();
                let mut e1 = Eff::new();
                e1.dao = 1.0;
                e1.special = 47; // 置链：→141 代笔提要
                e1.msg = vec!["你替他搬了一下午箱。临别他塞给你一册无皮抄本：「压箱底的，新书主不识货。」道心+1".into()];
                card.opts.push(("帮搬装箱".into(), e1));
                let g = (45.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = -g; // v1.16.11 复审口径：购书是花销，与卡面「灵石-{}」同号（照 122 赎页同款）
                e2.luck = 1.0;
                e2.msg = vec![format!(
                    "你挑中一册无名手抄。老掌柜只收半价：「识货的，就该便宜。」灵石-{}，气运+1",
                    fmt_int(g)
                )];
                card.opts.push(("半价购书".into(), e2));
                self.chain = Some((12, 0, CHAIN_TTL)); // 首段建链：待分支
            }
            141 => {
                // 链12 中段（仅链内）
                card.title = "代笔提要·续".into();
                card.text = "新书主是个绸缎商，不识修行书，却把铺面翻修得亮堂。他听伙计说你识货，请你替架上旧抄本各写一句提要——写得中肯书就卖得动，酬你一成书资。".into();
                // v1.16.15 复审口径：卡面既言「酬你一成书资」，msg 补灵石实付
                // （照 116 添柴 30*ss 同量级取 45*ss，与既有收入事件持平）
                let g141 = (45.0 * ss) as i64;
                let mut e1 = Eff::new();
                e1.luck = 1.0;
                e1.stones = g141;
                e1.special = 48; // 置链：→142 肆后书案
                e1.msg = vec![format!(
                    "你写了七日。绸缎商言而有信，书资分毫不差——还多包了一层油纸。灵石+{}，气运+1",
                    fmt_int(g141)
                )];
                card.opts.push(("应下提要".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 1.0;
                e2.msg = vec![
                    "你写完提要不肯署名。绸缎商把「无名氏」三字裱在柜台上，说是镇店的。道心+1".into(),
                ];
                card.opts.push(("提要留白".into(), e2));
            }
            142 => {
                // 链12 末段（仅链内）：两选项均无置链臂 → 抉择即链终局（缘了）
                card.title = "肆后书案·续".into();
                card.text = "书肆生意果然好了。开春时绸缎商在肆后添了张书案，笔墨齐全，说：「给像你这样的人留的——城里的字，总得有人接着认。」".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.08;
                e1.dao = 3.0;
                e1.msg = vec![
                    "你把案头当成了书斋。经你批注过的旧书，卖得比新书还贵。修为+8%，道心+3".into(),
                ];
                card.opts.push(("常来坐案".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 4.0;
                e2.msg = vec![
                    "你荐了个落魄书生来坐案，绸缎商管了他一日两餐。你只出了一句话。道心+4".into(),
                ];
                card.opts.push(("荐个坐案人".into(), e2));
            }
            // 常规 8（143..=150）照 134..=136 词汇：纯 dao/stones/msg 效果、ss 缩放
            // 灵石、不占 special（战斗不走，题材皆市井山野静态局）
            143 => {
                card.title = "公道秤".into();
                card.text = "集市东头有杆老秤，挂在「公道秤」的木牌下，说是称人心。你把储物袋搁上去，秤星自己挪了三格，停在一个不上不下的位置。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.msg = vec![
                    "你伸手拨正秤星，秤杆纹丝不动——原来称的是你自家道心。道心+2".into(),
                ];
                card.opts.push(("校秤".into(), e1));
                let g = (40.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!(
                    "你替一桩口角的两家称了灵材，各服一口气，谢钱是两家凑的。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("借秤称物".into(), e2));
            }
            144 => {
                card.title = "渡蚁桥".into();
                card.text = "山雨冲断溪上小桥，一队黑蚁衔着碎叶在水面连成一线，摇摇欲坠。队伍尽头，蚁群正往你鞋边爬来。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.luck = 1.0;
                e1.msg = vec![
                    "你摘两片阔叶替它们续上桥面。蚁群过尽，雨也停了。道心+2，气运+1".into(),
                ];
                card.opts.push(("搭叶为桥".into(), e1));
                let mut e2 = Eff::new();
                // v1.16.15 复审口径：以灵力行功亦是耗损——补 qi_pct 消解「搭叶
                // 为桥」的全面支配（第二币种：动用灵气得道心少但修为不空过）
                e2.dao = 1.0;
                e2.qi_pct = 0.04;
                e2.msg = vec![
                    "你以灵气托起整队蚁群送过对岸。它们列队向你颔首——虫豸也识礼数。修为+4%，道心+1".into(),
                ];
                card.opts.push(("灵气渡蚁".into(), e2));
            }
            145 => {
                card.title = "冰窖遗鲜".into();
                card.text = "城中冰窖清底，窖底最深处冻着一枚桃——贡品样式，签条上的年号是三百年前的。管窖的老说，冻着的仙桃不会坏，只会等。".into();
                let mut e1 = Eff::new();
                e1.dao = 5.0;
                e1.qi_pct = 0.15;
                e1.msg = vec![
                    "你在檐下等它自己化开。三百年前的甜味漫开来的时候，你懂了什么叫「物候不由人」。道心+5，修为+15%".into(),
                ];
                card.opts.push(("化而食之".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 1.0;
                e2.luck = 3.0;
                e2.msg = vec![
                    "你把桃请回窖中深处：「既等了三百年，不差再等一程。」管窖老朝你长揖。道心+1，气运+3".into(),
                ];
                card.opts.push(("请回窖底".into(), e2));
            }
            146 => {
                card.title = "夜磨坊".into();
                card.text = "河湾水磨坊半夜水轮自转，坊里无人。磨眼里的不是麦，是月光——磨出的粉落在石槽上，风一吹就没了。".into();
                let mut e1 = Eff::new();
                e1.dao = 3.0;
                e1.msg = vec![
                    "你帮着把月粉收进陶罐，天亮时罐里只有一捧清水，水底的月影却洗不掉了。道心+3".into(),
                ];
                card.opts.push(("守磨一夜".into(), e1));
                let g = (55.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!(
                    "磨坊主人收租，谢你一勺「月磨面」——掺进寻常面里蒸糕，卖相极好。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("问坊主讨面".into(), e2));
            }
            147 => {
                card.title = "货栈灵鼠".into();
                card.text = "货栈掌柜愁眉苦脸：栈里灵鼠成精，专偷丹药房那格。你去看时，那鼠正抱着枚培元丹，见你来了，把丹往你脚边一推。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.luck = 1.0;
                e1.msg = vec![
                    "你把丹还进药格，放那鼠走了。掌柜直呼晦气，你却觉得这笔账记得漂亮。道心+2，气运+1".into(),
                ];
                card.opts.push(("纵鼠归山".into(), e1));
                let g = (45.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!(
                    "你把鼠连同丹一起交还掌柜，掌柜赏了谢钱，转头又求你后天再来。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("捉鼠还丹".into(), e2));
            }
            148 => {
                card.title = "采薇客".into();
                card.text = "高秋的南崖上有人采薇而歌，歌声不悲不喜，一句一句往云里去。你循声攀上崖，见个青袍人背影，篓里薇菜浅浅一层。".into();
                let mut e1 = Eff::new();
                e1.dao = 4.0;
                e1.msg = vec![
                    "你坐下来听完那支歌。歌没有词，你却听懂了——关于怎么把日子过成道。道心+4".into(),
                ];
                card.opts.push(("坐听一歌".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 1.0;
                e2.luck = 2.0;
                e2.msg = vec![
                    "你把干粮分了他一半。青袍人回身一笑：「山下还有人记得山上人吃饭。」道心+1，气运+2".into(),
                ];
                card.opts.push(("分他行粮".into(), e2));
            }
            149 => {
                card.title = "补鞍匠".into();
                card.text = "官道旁补鞍的老匠人拆开一副旧马鞍，夹层里掉出一封没有火漆的信，信皮上的字迹被磨得只剩半行。老匠不识字，摊着手问你。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.msg = vec![
                    "你读给老匠听——是三十年前一个学徒留给师父的。老匠把信缝了回去：「话没送到，鞍就还没补完。」道心+2".into(),
                ];
                card.opts.push(("代读代寄".into(), e1));
                let g = (50.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = -g; // 购鞍是花销，与卡面「灵石-{}」同号（照 122/140 同款）
                e2.luck = 1.0;
                e2.msg = vec![format!(
                    "你出钱买下旧鞍。老匠推回一半钱：「信是你读的，鞍钱减半。」灵石-{}，气运+1",
                    fmt_int(g)
                )];
                card.opts.push(("买下旧鞍".into(), e2));
            }
            150 => {
                card.title = "雪泥鸿迹".into();
                card.text = "大雪初霁，雪原上只有一行脚印，从你门前直到林深处——去程的。没有回程的。你认得这脚印的步幅，像极了一个人负手慢行。".into();
                let mut e1 = Eff::new();
                e1.dao = 3.0;
                e1.msg = vec![
                    "你循迹走到林中空地，雪地上留着一盘没下完的棋。你替黑白各落一子，转身回家。道心+3".into(),
                ];
                card.opts.push(("循迹而行".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 1.0;
                e2.luck = 1.0;
                e2.msg = vec![
                    "你拿扫帚把那行足迹轻轻拂平——有些来客，不必被后来人惊扰。道心+1，气运+1".into(),
                ];
                card.opts.push(("拂平足迹".into(), e2));
            }

            // ---------- v1.16.14 扩池 5 条（id 151..=155）----------
            // 链13 死当归主（151..=153）：当铺死当旧物归主线——「归」字刻痕贯穿
            // 三段（151 架上旧箫→152 寻主赎当→153 物归旧主），物—人—缘结构与既有
            // 十二链同构而载体全新（货郎=行商、当铺=坐商；与链10 旧页回声分域：
            // 那条是「跨世旧稿」，本链是「本世旧物」，无跨世指涉）。置链臂 49/50
            // 第六窗（CHAIN13_SPECIAL_LO/HI），末段抉择即「死当归主·缘了」入大事
            // 记（stage 表见函数尾）
            151 => {
                // 链13 首段：当铺死当（首段建链待分支）
                card.title = "当铺死当".into();
                card.text = "镇口老当铺的死当架上，一管旧箫落了薄尘，签条一个「死」字——三年无人来赎，按例今岁拆料重铸。箫尾刻着个小小的「归」字，笔画被摩挲得发亮。".into();
                let mut e1 = Eff::new();
                e1.dao = 1.0;
                e1.special = 49; // 置链：→152 访主赎当
                e1.msg = vec!["你问下这管箫。掌柜翻出当票存根：「物主姓甚忘了，只记得年年冬至来朝铺子里望一眼。」道心+1".into()];
                card.opts.push(("问下死当".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 1.0;
                e2.msg = vec![
                    "旧物各有机缘，死当也是归宿。你出门时，掌柜正把箫收进里屋。气运+1".into(),
                ];
                card.opts.push(("不置一词".into(), e2));
                self.chain = Some((13, 0, CHAIN_TTL)); // 首段建链：待分支
            }
            152 => {
                // 链13 中段（仅链内）
                card.title = "访主赎当·续".into();
                card.text = "循当票存根寻到城郊药庐，物主是位咳疾缠身的老丹师——当年为给同门购药，把随身旧箫典进了当铺。他说物随缘去，不赎也罢，只是每年冬至会朝镇上望一眼。".into();
                let g = (30.0 * ss) as i64;
                let mut e1 = Eff::new();
                e1.stones = -g; // 代垫赎资是花销，与卡面「灵石-{}」同号（照 122/140/149 同款）
                e1.dao = 1.0;
                e1.special = 50; // 置链：→153 物归旧主
                e1.msg = vec![format!(
                    "你回当铺替他赎下旧箫送回药庐。掌柜收了当本，多句：「认物的，今年头一个。」灵石-{}，道心+1",
                    fmt_int(g)
                )];
                card.opts.push(("代为赎当".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 1.0;
                e2.msg = vec![
                    "你把存根原样放回，只把「归」字的来历讲给掌柜听。当铺的灯，那晚亮得久些。气运+1".into(),
                ];
                card.opts.push(("止步不问".into(), e2));
            }
            153 => {
                // 链13 末段（仅链内）：两选项均无置链臂 → 抉择即链终局（缘了）
                card.title = "物归旧主·续".into();
                card.text = "旧箫归架那日，老丹师的精神竟好了些。他说箫是故人手制，箫声起时，药庐里经年的药苦味都淡了。曲罢他执意送你到篱外，往你袖里塞了一撮窖了多年的灵茶。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.10;
                e1.dao = 3.0;
                e1.msg = vec![
                    "你在篱外立了半晌——箫声呜咽穿篱而出，胸中滞念竟随曲散了。修为+10%，道心+3".into(),
                ];
                card.opts.push(("篱外听曲".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.luck = 2.0;
                e2.msg = vec![
                    "你收下灵茶。他说：「物归了主，茶归有缘。」道心+2，气运+2".into(),
                ];
                card.opts.push(("收下谢礼".into(), e2));
            }
            // 宗门换届 2（154/155）照 134..=136 词汇：纯 dao/stones/msg 效果、ss
            // 缩放灵石、不占 special（不入链不入稀有，题材是宗门兴望的世情侧写）
            // ——入池门控（入宗门+东境青云）在 sim_event_weights，见该函数块注
            154 => {
                card.title = "新掌接印".into();
                card.text = "青云宗今岁接印大典——老掌门功成闭关，新掌门于观星台受印。山门外候观的散修排成长龙，你随人流上山，满山彩绸映得檐角都在发亮。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.08;
                e1.msg = vec![
                    "受印一拜，满山灵机为之一涌。你借势吐纳半日，经络里暖意不散。修为+8%".into(),
                ];
                card.opts.push(("观礼随喜".into(), e1));
                let g = (60.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!(
                    "你帮山门搭彩、递印捧诰，执事分了份观礼例钱——今岁宗门手笔比往年阔。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("帮接彩头".into(), e2));
            }
            155 => {
                card.title = "旧掌传灯".into();
                card.text = "老掌门闭关前最后一次开讲，讲的不是功法，是一句「接印容易守印难」。大殿里挤满了人，你坐在窗根下听完，出殿时暮钟正响。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.msg = vec![
                    "「印是担子，不是椅子。」这一句，你在归途上想了很久。道心+2".into(),
                ];
                card.opts.push(("殿中听讲".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 1.0;
                e2.luck = 1.0;
                e2.msg = vec![
                    "你把讲稿抄了一份，与门外候着的散修分阅——灯传下去，才算灯。道心+1，气运+1".into(),
                ];
                card.opts.push(("抄稿分阅".into(), e2));
            }

            // ---------- v1.16.15 扩池 9 条（156..=164）----------
            // 伴随深化 3（156..=158，COMPANION_EVENTS 门控照 82..=84 同款：cn 取
            // sim.companion 反查 COMPANION_NAMES 挂 Comp 立绘，缺席兜底 COMPANION_
            // NAMES[0] 不误指）+ 常规 6（159..=164，山野异象/民俗生活流，照
            // 143..=150 词汇：纯 dao/stones/luck/qi_pct 效果、不占 special、
            // 灵石臂照 ss 缩放；题材查重见 data.rs EVENTS 表 v1.16.15 批注）
            156 => {
                // 伴随事件：雪窖藏酿（Comp 立绘）
                card.title = "雪窖藏酿".into();
                let cn = {
                    let s = self.sim.as_ref().unwrap();
                    s.companion
                        .clone()
                        .unwrap_or_else(|| COMPANION_NAMES[0].to_string())
                };
                let ci = COMPANION_NAMES
                    .iter()
                    .position(|&n| n == cn)
                    .unwrap_or(0);
                card.actor = Some(CardActor::Comp(ci));
                card.text = format!(
                    "入冬第一场雪，{}抱来一坛青梅酒：「埋在檐下，功成之日同启——谁先回来，谁挖。」",
                    cn
                );
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.luck = 1.0;
                e1.msg = vec![
                    "你俩把坛子埋进雪里，插了根枯枝作记号。道心+2，气运+1".into(),
                ];
                card.opts.push(("埋坛立约".into(), e1));
                let mut e2 = Eff::new();
                e2.qi_pct = 0.05;
                e2.msg = vec![
                    "「功成哪有个准头。」你笑道，先斟了一碗——酒是酸的，雪是甜的。修为+5%".into(),
                ];
                card.opts.push(("先启一勺".into(), e2));
            }
            157 => {
                // 伴随事件：同守昙花（Comp 立绘）
                card.title = "同守昙花".into();
                let cn = {
                    let s = self.sim.as_ref().unwrap();
                    s.companion
                        .clone()
                        .unwrap_or_else(|| COMPANION_NAMES[0].to_string())
                };
                let ci = COMPANION_NAMES
                    .iter()
                    .position(|&n| n == cn)
                    .unwrap_or(0);
                card.actor = Some(CardActor::Comp(ci));
                card.text = format!(
                    "{}养了一盆昙花，掐着时辰说今夜子时开——「花开一现，错过等来年。」你俩搬了两只蒲团守在花前。",
                    cn
                );
                let mut e1 = Eff::new();
                e1.dao = 3.0;
                e1.msg = vec![
                    "子时三刻，白瓣一层层张开。有些东西慢——慢到一生只给你看一次。道心+3".into(),
                ];
                card.opts.push(("守到花开".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 1.0;
                e2.luck = 2.0;
                e2.msg = vec![format!(
                    "你守到四更睡去，醒来花已谢。{}把落花夹进你的书里：「明年它还开，你也还在。」道心+1，气运+2",
                    cn
                )];
                card.opts.push(("先睡留话".into(), e2));
            }
            158 => {
                // 伴随事件：雨后补瓦（Comp 立绘）
                card.title = "雨后补瓦".into();
                let cn = {
                    let s = self.sim.as_ref().unwrap();
                    s.companion
                        .clone()
                        .unwrap_or_else(|| COMPANION_NAMES[0].to_string())
                };
                let ci = COMPANION_NAMES
                    .iter()
                    .position(|&n| n == cn)
                    .unwrap_or(0);
                card.actor = Some(CardActor::Comp(ci));
                card.text = format!(
                    "夜雨漏了屋一角，天亮你与{}上房补瓦——一人递一人接，谁也没提这屋子本是借宿的。",
                    cn
                );
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.luck = 1.0;
                e1.msg = vec![
                    "最后一片瓦压实，日头正好爬上房脊。借宿的屋，补成了家。道心+2，气运+1".into(),
                ];
                card.opts.push(("细心补全".into(), e1));
                let mut e2 = Eff::new();
                e2.qi_pct = 0.05;
                e2.msg = vec![
                    "你以灵气把碎瓦熔凝重塑，比原先还结实三分。修为+5%".into(),
                ];
                card.opts.push(("灵气熔凝".into(), e2));
            }
            159 => {
                card.title = "萤火引路".into();
                card.text = "夏夜山道起雾迷了路，一列萤火忽然自林间亮起，蜿蜒向北——雾气竟随它们让出一条道来。".into();
                let mut e1 = Eff::new();
                e1.dao = 1.0;
                e1.luck = 1.0;
                e1.msg = vec![
                    "你随光而行，半个时辰后雾散月出，回望来路萤火已散。道心+1，气运+1".into(),
                ];
                card.opts.push(("循光出谷".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.msg = vec![
                    "你停在道口让萤先行——它们绕你三匝才去。说不清是什么谢礼，但你心里松快。气运+2".into(),
                ];
                card.opts.push(("让道萤行".into(), e2));
            }
            160 => {
                card.title = "山洪改道".into();
                card.text = "连日暴雨，山洪裹着断木直冲下游村落。你在洪头之上，身边只有一炷香的工夫。".into();
                let mut e1 = Eff::new();
                e1.dao = 3.0;
                e1.msg = vec![
                    "你以术力硬生生把洪头分进废弃故河道——浊浪过境，村中鸡犬不惊。道心+3".into(),
                ];
                card.opts.push(("分洪入故道".into(), e1));
                let g = (40.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!(
                    "术力分洪不足，你落村中帮各家抢收抢掩——水退后，村老塞给你一口袋谢钱。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("落村帮抢收".into(), e2));
            }
            161 => {
                card.title = "白鹿过涧".into();
                card.text = "雪霁。一头白鹿自涧底踏雪而过，蹄印落处竟生出一线青草，走到对面山脊才回头望你一眼。".into();
                let mut e1 = Eff::new();
                e1.dao = 1.0;
                e1.luck = 2.0;
                e1.msg = vec![
                    "你循蹄印走到山脊——鹿已不见，蹄印里那线青草却在雪里绿着。道心+1，气运+2".into(),
                ];
                card.opts.push(("循迹上山脊".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.msg = vec![
                    "你朝它消失的方向长揖一礼，转身下山——有些相遇，不必追。道心+2".into(),
                ];
                card.opts.push(("长揖目送".into(), e2));
            }
            162 => {
                card.title = "月蚀之夜".into();
                card.text = "十五夜，圆月被啃去一角。村中锣声四起——老人们说月蚀是天上犬在吞月，要敲响器救它。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.luck = 1.0;
                e1.msg = vec![
                    "你击缶和着锣声，直到月轮复圆——狗吠渐歇，月亮干干净净。道心+2，气运+1".into(),
                ];
                card.opts.push(("击缶救月".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 1.0;
                e2.qi_pct = 0.04;
                e2.msg = vec![
                    "你不上街，只在檐下静观月轮一点点吞进去又吐出来——天地自有时刻。修为+4%，道心+1".into(),
                ];
                card.opts.push(("静观复圆".into(), e2));
            }
            163 => {
                card.title = "壶中天".into();
                card.text = "集上老者售一柄陶壶：「壶里有一座山。」你冲水入壶，壶口竟传出一线飞瀑声，久久不歇。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.06;
                e1.msg = vec![
                    "你以灵泉日日养壶——瀑声渐清，静坐时如闻真山。修为+6%".into(),
                ];
                card.opts.push(("灵泉养壶".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.msg = vec![
                    "你把壶原价转赠村塾的孩童——多年后有人回来说，那孩子成了画山水的名家。气运+2".into(),
                ];
                card.opts.push(("转赠孩童".into(), e2));
            }
            164 => {
                card.title = "檐铃无风".into();
                card.text = "客栈檐角一枚铁铃，无风自响。店家说这铃响时，北面官道必有游子还乡——响一声，还一个。".into();
                let mut e1 = Eff::new();
                e1.luck = 2.0;
                e1.msg = vec![
                    "你搬梯把铃解下擦亮重新挂好——当夜铃响了三声，北面道上灯火三点。气运+2".into(),
                ];
                card.opts.push(("擦亮重挂".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.msg = vec![
                    "你枕着铃音坐到天明——每一声，都替远行的人把家喊了一遍。道心+2".into(),
                ];
                card.opts.push(("坐听一夜".into(), e2));
            }
            // v1.16.16 扩池 5 条（165..=169）：结局回响 3 + 书外之章联动 2。
            // 门控在 sim_event_weights（ed= bit0/1/2 与 ch= bit8，只读持久位零新键）；
            // 效果纯 dao/luck/qi_pct/stones（灵石臂照 ss 缩放），不占 special——
            // 落在链裁决窗之外，抉择即自然清链（本就无链，语义无害）
            165 => {
                // 结局回响·A 定稿（入正文之人的世界侧写）
                card.title = "定稿之世".into();
                card.text = "茶馆说书人开了段新话本，主角的名字你听着耳熟——那是入了正文的人。听客们交头接耳：「这名字一字改不得。」说书人一拍醒木：「天定的稿子，谁敢改。」".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.msg = vec![
                    "你听完末段，替那个名字把茶钱付了——被定稿的人生，也值得一盏热茶。道心+2".into(),
                ];
                card.opts.push(("听完这段".into(), e1));
                let g = (36.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.luck = 1.0;
                e2.msg = vec![format!(
                    "你把话本原样记进随行册——世人的嘴，也是一种稿，稿酬照付。灵石+{}，气运+1",
                    fmt_int(g)
                )];
                card.opts.push(("记下话本".into(), e2));
            }
            166 => {
                // 结局回响·B 留白（后事不详之地的世界侧写）
                card.title = "留白之野".into();
                card.text = "行脚客们多了个去处：某镇外一片无名野地。官册上那一格空着，卦摊说这叫「后事不详」——无名无姓，反倒谁都能去。野草长得很高，高过所有批注。".into();
                let mut e1 = Eff::new();
                e1.dao = 3.0;
                e1.msg = vec![
                    "你在草里坐到日暮，什么名号也没留——留白处最容得下人。道心+3".into(),
                ];
                card.opts.push(("草里坐坐".into(), e1));
                let mut e2 = Eff::new();
                e2.qi_pct = 0.08;
                e2.msg = vec![
                    "你沿着野地走了一遭，替后来的行人踩出半条小径。修为+8%".into(),
                ];
                card.opts.push(("踩出小径".into(), e2));
            }
            167 => {
                // 结局回响·C 执笔（执笔人视点的世界侧写）
                card.title = "执笔之赠".into();
                card.text = "废稿市上新添一个怪摊：不卖书，只替人描名字——把磨没了的名字一笔一笔描清楚，分文不取。你蹲看半日，摊主把笔一转，笔杆朝你：「你也描一个？」".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.luck = 2.0;
                e1.msg = vec![
                    "你接过笔，描的是自己最初的名字——笔画很稳，像描过千万遍。道心+2，气运+2".into(),
                ];
                card.opts.push(("描一个".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 1.0;
                e2.msg = vec![
                    "你摇头，只帮他把一摞废稿按名字理齐——有些名字，等得起。气运+1".into(),
                ];
                card.opts.push(("只理废稿".into(), e2));
            }
            168 => {
                // 书外之章联动（第九章·书外之章已了门控）：天曹余波
                card.title = "天曹补写".into();
                card.text = "天曹卷宗房漏了一夜雨，晒卷的小吏发现你那一册的页脚多了行小字，墨色很新，像谁趁夜补写：「此卷之外，另有续行。」小吏不敢涂，上报了事。".into();
                let mut e1 = Eff::new();
                e1.dao = 3.0;
                e1.msg = vec![
                    "你托梦去问那行字是谁写的，梦里只有翻纸声——书外的路，天曹也只配旁听。道心+3".into(),
                ];
                card.opts.push(("托梦一问".into(), e1));
                let g = (60.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!(
                    "你把「另有续行」四字揭裱收藏——书成之后还能吓着天曹，值得喝一杯。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("揭裱收藏".into(), e2));
            }
            169 => {
                // 书外之章联动（第九章·书外之章已了门控）：书肆余波
                card.title = "书肆新声".into();
                card.text = "书肆最深处新到一册无脊之书，页页相连找不到头尾。掌柜说这书不吃价、只吃缘：「有缘人翻开哪页，就从哪页读起——翻不开的，看一眼脊梁骨都摸不着。」".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.10;
                e1.dao = 1.0;
                e1.msg = vec![
                    "你一伸手，书自己翻开了——第一行是你的名字。修为+10%，道心+1".into(),
                ];
                card.opts.push(("伸手一翻".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.msg = vec![
                    "你不翻，只隔柜听人念——念书人每翻一页，你这册就厚一分。气运+2".into(),
                ];
                card.opts.push(("隔柜听念".into(), e2));
            }
            // v1.16.16 扩池 13 条（170..=182）：老年层 2（门控见 sim_event_weights
            // id∈{88,89,170,171}）/开局层 3/链14 修伞记 3（置链臂 53/54 第七窗）/
            // 恶缘 2/高境 mr5 2/伴随 1（COMPANION_EVENTS 白名单门控）。效果仍守
            // 常规批纪律：dao/luck/qi_pct/stones（灵石臂照 ss 缩放），179 用
            // fight+injure 照 62 雪崩沟谷危机臂先例
            170 => {
                // 老年层：炉暖旧器（交托回望）
                card.title = "炉暖旧器".into();
                card.text = "巷尾老匠的炉火不熄，煨着几件街坊送来的旧物——锈锁、断梳、豁口的陶壶。见你路过，他招手：「后生，搭把手，替我把这封回信捎去城南。人老了，腿脚先老。」".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.luck = 1.0;
                e1.msg = vec![
                    "你把信送到，又把回话捎了回来——老匠听完，笑了半晌。道心+2，气运+1".into(),
                ];
                card.opts.push(("捎信城南".into(), e1));
                let g = (30.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.luck = 1.0;
                e2.msg = vec![format!(
                    "你替炉子添了一整夜炭，天明时老匠把煨好的一枚铜铃塞给你：「响的不是铃，是念想。」灵石+{}，气运+1",
                    fmt_int(g)
                )];
                card.opts.push(("添火守炉".into(), e2));
            }
            171 => {
                // 老年层：灯影授业（交托回望）
                card.title = "灯影授业".into();
                card.text = "油灯将尽的老屋里，一位白发的教书先生还醒着。他望着你的印，像是认得它的气息：「我不问你从哪来。灯影里还剩半课——年少时我不敢讲，如今敢讲了。」".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.08;
                e1.dao = 1.0;
                e1.msg = vec![
                    "半课讲的是「输」。你听到灯芯结花，屋里亮了一瞬。修为+8%，道心+1".into(),
                ];
                card.opts.push(("听完半课".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.msg = vec![
                    "你默默把灯油添满，退到门外。先生看破不说破，只把那半课讲完了。道心+2".into(),
                ];
                card.opts.push(("为灯添油".into(), e2));
            }
            172 => {
                // 开局层：义渡（与 112 灯花渡舟分域——那是灵异奇遇，这是渡口日常）
                card.title = "义渡".into();
                card.text = "河面无桥，老船工撑船义渡，船头木牌写着「分文不取，随缘添香」。今日风大，他朝你招手：「后生，来帮我压一压船头。」".into();
                let mut e1 = Eff::new();
                e1.luck = 2.0;
                e1.msg = vec![
                    "一程风浪平稳，对岸等渡的人千恩万谢。老船工塞给你一支旱烟，你没抽，别在了斗笠上。气运+2".into(),
                ];
                card.opts.push(("搭手压船".into(), e1));
                let g = (20.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = -g;
                e2.good = false;
                e2.dao = 1.0;
                e2.msg = vec![format!(
                    "你往香资匣里压下一笔灵石。老船工推辞不过，朝河心拱了拱手：「替你记着这份香火。」灵石-{}，道心+1",
                    fmt_int(g)
                )];
                card.opts.push(("香资匣添香".into(), e2));
            }
            173 => {
                // 开局层：村塾借灯
                card.title = "村塾借灯".into();
                card.text = "夜宿山村，村塾的灯油见了底，老先生摸黑批课业，笔尖全凭手感。他窘迫地笑：「灯借不得，油却买得——穷教书人，只剩这张脸。」".into();
                let g = (16.0 * ss) as i64;
                let mut e1 = Eff::new();
                e1.stones = -g;
                e1.good = false;
                e1.dao = 2.0;
                e1.msg = vec![format!(
                    "你买来两斤灯油放下就走。次日清晨，塾里书声比往日都响。灵石-{}，道心+2",
                    fmt_int(g)
                )];
                card.opts.push(("出钱打油".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 1.0;
                e2.msg = vec![
                    "你把随行的小灯留在案上。老先生郑重记下你的名字，说来日塾里出了人物，第一个谢你。气运+1".into(),
                ];
                card.opts.push(("留灯而去".into(), e2));
            }
            174 => {
                // 开局层：霜桥早市
                card.title = "霜桥早市".into();
                card.text = "霜天未亮，石桥两廊早市已开：热粥、姜茶、新出炉的炊饼。摊主们跺脚呵手，见你走来，竟不约而同往桥心让了让——像给什么人留位。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.06;
                e1.msg = vec![
                    "一碗热粥下肚，你在市声里听完半座小镇醒来。修为+6%".into(),
                ];
                card.opts.push(("桥上走一遭".into(), e1));
                let g = (22.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = -g;
                e2.good = false;
                e2.luck = 2.0;
                e2.msg = vec![format!(
                    "你买了一圈姜茶挨摊分去。霜天不冷，人间值得。灵石-{}，气运+2",
                    fmt_int(g)
                )];
                card.opts.push(("姜茶绕桥".into(), e2));
            }
            175 => {
                // 链14 首段：修伞记（凡伞铺老匠——只修凡伞，不识法器；首段建链待分支）
                card.title = "修伞记".into();
                card.text = "雨夜投宿，巷口一间伞铺还亮着灯。老匠戴顶破笠，正给一把油纸伞换骨：「小店只修凡伞——金的铁的、会发光的，别拿来。我不识，也不碰。」".into();
                let mut e1 = Eff::new();
                e1.luck = 1.0;
                e1.special = 53; // 置链：→176 伞骨重削
                e1.msg = vec![
                    "你把随行的旧伞递过去。老匠验了验骨相：「这伞跟了你许多年吧？明早来取。」气运+1".into(),
                ];
                card.opts.push(("请老匠修伞".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 1.0;
                e2.msg = vec![
                    "你在檐下坐了一夜，看老匠劈骨、穿线、糊面，雨声替他打拍子。道心+1".into(),
                ];
                card.opts.push(("檐下观工".into(), e2));
                self.chain = Some((14, 0, CHAIN_TTL)); // 首段建链：待分支
            }
            176 => {
                // 链14 中段（仅链内）
                card.title = "伞骨重削·续".into();
                card.text = "翌日再来，老匠却对着拆开的伞面皱眉：「你这伞骨相古怪——三十六骨该匀不匀，像被法力催过。我照凡伞的规矩重削，你莫怪。」".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.special = 54; // 置链：→177 还伞
                e1.msg = vec![
                    "你说「照您的规矩来」。老匠一削到底——削下来的伞骨，竟是齐的。道心+2".into(),
                ];
                card.opts.push(("应他重削".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 1.0;
                e2.msg = vec![
                    "你说「将能用的将就」。老匠摇头不语，还是把最歪的那根换直了。气运+1".into(),
                ];
                card.opts.push(("劝他照旧".into(), e2));
            }
            177 => {
                // 链14 末段（仅链内）：两选项均无置链臂 → 抉择即链终局（缘了）
                card.title = "还伞·续".into();
                card.text = "伞修好了。老匠把伞递还，另塞给你一支新削的伞骨：「竹子这东西，削直了就直一辈子——人也一样。去你的路吧，雨要停了。」".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.luck = 2.0;
                e1.msg = vec![
                    "你收下伞骨。此后每逢雨夜，你都会想起这间铺子和那句话。道心+2，气运+2".into(),
                ];
                card.opts.push(("收下伞骨".into(), e1));
                let g = (26.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = -g;
                e2.good = false;
                e2.dao = 1.0;
                e2.msg = vec![format!(
                    "你把工钱压在针线筐底下。老匠追出门时，你已走进将停的雨里。灵石-{}，道心+1",
                    fmt_int(g)
                )];
                card.opts.push(("压钱离铺".into(), e2));
            }
            178 => {
                // 恶缘：沼瘴蚀符（mr2 薄档补恶）
                card.title = "沼瘴蚀符".into();
                card.text = "沼地瘴气翻涌如粥，你护身的符纸一夜之间蚀出细密斑点，蜷边发黑——像被什么东西啃着，不急，也不停。".into();
                let mut e1 = Eff::new();
                e1.stones_pct = -0.05;
                e1.good = false;
                e1.msg = vec![
                    "你当机立断弃了蚀符，抢在瘴气合拢前脱身，行囊却被蚀去一角。灵石-5%".into(),
                ];
                card.opts.push(("弃符速离".into(), e1));
                let mut e2 = Eff::new();
                e2.qi_pct = -0.08;
                e2.good = false;
                e2.dao = 2.0;
                e2.msg = vec![
                    "你分出一缕修为温养符文，硬把蚀痕一寸寸压了回去。修为-8%，道心+2".into(),
                ];
                card.opts.push(("以气养符".into(), e2));
            }
            179 => {
                // 恶缘·稀有：赝丹局（mr3 薄档补恶；w2 入稀有层）
                card.title = "赝丹局".into();
                card.text = "丹市深处，锦衣人低声兜售「九转遗丹」，要价惊人。丹香扑鼻——可香得过火，你识海里那点灵觉隐隐发腻。".into();
                let mut e1 = Eff::new();
                e1.fight = 2.0;
                e1.injure = true;
                e1.good = false;
                e1.dao = 1.0;
                e1.msg = vec![
                    "你当街点破炉温、丹纹、香源三处破绽。锦衣人脸色骤变，袖里滑出符咒——恶战难免。交战 ×2，道心+1".into(),
                ];
                card.opts.push(("当场揭穿".into(), e1));
                let g = (150.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = -g;
                e2.good = false;
                e2.dao = 1.0;
                e2.msg = vec![format!(
                    "你咬牙付钱买下，回去一验果然是赝品——只当破财免灾，记住了那股过火的香。灵石-{}，道心+1",
                    fmt_int(g)
                )];
                card.opts.push(("付钱买教训".into(), e2));
            }
            180 => {
                // 高境风味：云阶试步（mr5 独占）
                card.title = "云阶试步".into();
                card.text = "生花境深处有云阶千级，浮云所砌，每级只承一息。登山的人说：步急了阶散，步滞了云走——唯有自己的呼吸配得上它。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.10;
                e1.msg = vec![
                    "你以呼吸为尺，一级一级走上云阶。回首时，云海在脚下让出一条路。修为+10%".into(),
                ];
                card.opts.push(("拾级试步".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.msg = vec![
                    "你看一位白发行者走完全程，替他记下第九十九级的那一次停顿。道心+2".into(),
                ];
                card.opts.push(("阶下观人".into(), e2));
            }
            181 => {
                // 高境风味：观澜悟潮（mr5 独占）
                card.title = "观澜悟潮".into();
                card.text = "崖下澜潮昼夜不歇，撞石的声势却循着某种极缓的呼吸。你盘膝崖上，忽然发觉自己的心跳，正在偷偷向潮声看齐。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.08;
                e1.dao = 2.0;
                e1.msg = vec![
                    "你索性把呼吸交给潮。一坐整日，起身时衣袂都浸着潮气。修为+8%，道心+2".into(),
                ];
                card.opts.push(("随潮入定".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.msg = vec![
                    "你以指画沙，记下澜潮三段缓急——像记一支不肯散的曲子。气运+2".into(),
                ];
                card.opts.push(("记潮作谱".into(), e2));
            }
            182 => {
                // 伴随事件：霜夜补裘（Comp 立绘，照 156 模板）
                card.title = "霜夜补裘".into();
                let cn = {
                    let s = self.sim.as_ref().unwrap();
                    s.companion
                        .clone()
                        .unwrap_or_else(|| COMPANION_NAMES[0].to_string())
                };
                let ci = COMPANION_NAMES
                    .iter()
                    .position(|&n| n == cn)
                    .unwrap_or(0);
                card.actor = Some(CardActor::Comp(ci));
                card.text = format!(
                    "霜夜露重，你翻出行囊里那件旧裘，针脚还是多年前缝的。{}取过针线：「补补吧。旧物认人，人也认旧物。」",
                    cn
                );
                let mut e1 = Eff::new();
                e1.luck = 2.0;
                e1.dao = 1.0;
                e1.msg = vec![
                    "一针一线到天明，旧裘暖回了从前的样子。气运+2，道心+1".into(),
                ];
                card.opts.push(("灯下同补".into(), e1));
                let g = (40.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!(
                    "你把旧裘披回{}肩上，转身去添炭温酒——旧物认人，今夜不冷。灵石+{}",
                    cn,
                    fmt_int(g)
                )];
                card.opts.push(("披裘添炭".into(), e2));
            }

            // ================= v1.16.17 扩池 10 条（183..=192） =================
            183 => {
                // 伴随事件：灯下习字（mr0 白丁有伴层首条伴随——遗泽玩家的白丁推演
                // 此前零伴随；Comp 立绘照 82 反查模式，呼应道侣专页）
                card.title = "灯下习字".into();
                let cn = {
                    let s = self.sim.as_ref().unwrap();
                    s.companion
                        .clone()
                        .unwrap_or_else(|| COMPANION_NAMES[0].to_string())
                };
                let ci = COMPANION_NAMES
                    .iter()
                    .position(|&n| n == cn)
                    .unwrap_or(0);
                card.actor = Some(CardActor::Comp(ci));
                card.text = format!(
                    "{}在灯下临你的字帖，一横一竖描得极慢：「你的字有一笔总写歪——不是手的事，是心里那一笔没落稳。」",
                    cn
                );
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.luck = 1.0;
                e1.msg = vec![
                    "你们各临一页。写歪的那一处，两份字迹里竟都稳了。道心+2，气运+1".into(),
                ];
                card.opts.push(("受教同临".into(), e1));
                let mut e2 = Eff::new();
                e2.qi_pct = 0.05;
                e2.msg = vec![
                    "你把那页收进袖中——留着她说的那一笔，来年再看。修为+5%".into(),
                ];
                card.opts.push(("笑而不答".into(), e2));
            }
            184 => {
                // 伴随事件：同参一炉（mr2 双修题材——呼应现实侧双修共进与
                // COMPANION_CULT_MULT 好感档；Eff.comp_fav 好感通道首用，
                // +5 照 COMPANION_GIFT_FAV 量级）
                card.title = "同参一炉".into();
                let cn = {
                    let s = self.sim.as_ref().unwrap();
                    s.companion
                        .clone()
                        .unwrap_or_else(|| COMPANION_NAMES[0].to_string())
                };
                let ci = COMPANION_NAMES
                    .iter()
                    .position(|&n| n == cn)
                    .unwrap_or(0);
                card.actor = Some(CardActor::Comp(ci));
                card.text = format!(
                    "{}把两份引气图并在一案：「单修如独焰，双修如对火——你借我一分气，我还你一分神。」",
                    cn
                );
                let mut e1 = Eff::new();
                e1.comp_fav = crate::data::COMPANION_GIFT_FAV;
                e1.dao = 1.0;
                e1.msg = vec![format!(
                    "一炉火候，两息相衔。出关时神完气足。（好感+{}）道心+1",
                    crate::data::COMPANION_GIFT_FAV
                )];
                card.opts.push(("对坐同参".into(), e1));
                let mut e2 = Eff::new();
                e2.qi_pct = 0.05;
                e2.msg = vec!["同室不同功，各走各的周天，也自相安。修为+5%".into()];
                card.opts.push(("各自用功".into(), e2));
            }
            185 => {
                // 伴随事件：云海携游（mr4+ 有伴层——此前伴随 mr 全≤3，mr4+ 零覆盖）
                card.title = "云海携游".into();
                let cn = {
                    let s = self.sim.as_ref().unwrap();
                    s.companion
                        .clone()
                        .unwrap_or_else(|| COMPANION_NAMES[0].to_string())
                };
                let ci = COMPANION_NAMES
                    .iter()
                    .position(|&n| n == cn)
                    .unwrap_or(0);
                card.actor = Some(CardActor::Comp(ci));
                card.text = format!(
                    "{}御风携你入云海：「高处风大，抓稳——书上说云下众生如蚁，我想让你亲眼看看。」",
                    cn
                );
                let mut e1 = Eff::new();
                e1.luck = 2.0;
                e1.msg = vec!["云开处山河万里，你们在风里笑出声来。气运+2".into()];
                card.opts.push(("同御长风".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.msg = vec![
                    "你在云隙间认出走过的每一处——原来路也一直记着人。道心+2".into(),
                ];
                card.opts.push(("俯察来路".into(), e2));
            }
            186 => {
                // 恶缘：仙曹索卷（mr5 独占——高境墨气冲霄才有人来索；mr5 无条件
                // 常态池 7→9 条补薄）
                card.title = "仙曹索卷".into();
                card.text = "一名佩牍仙曹踏云而至，展册点名：「你名下稿卷墨气冲霄，按例当入天曹文库里架——交卷归档，或领催墨单。」".into();
                let mut e1 = Eff::new();
                e1.stones_pct = -0.08;
                e1.good = false;
                e1.msg = vec!["你以灵石润笔，买回这一稿的着落。灵石-8%".into()];
                card.opts.push(("补墨留卷".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.msg = vec![
                    "你按卷不交，报出名讳。仙曹搁笔而去——天曹账上，从此记了条硬骨头。道心+2".into(),
                ];
                card.opts.push(("据卷力争".into(), e2));
            }
            187 => {
                // 善缘：星槎访客（mr5 独占——前路无名的问路者）
                card.title = "星槎访客".into();
                card.text = "一叶星槎泊于云端，槎上客披星而下，向你讨一盏茶：「行到此处，前路无名——借高士一盏茶的工夫，问一问路。」".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.08;
                e1.dao = 1.0;
                e1.msg = vec![
                    "星客饮罢，以指为笔在你掌心画下一道星轨。修为+8%，道心+1".into(),
                ];
                card.opts.push(("奉茶指路".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.msg = vec![
                    "你们把一夜说成了一生。天明槎去，云上留一颗温热的星子。气运+2".into(),
                ];
                card.opts.push(("相伴一夜".into(), e2));
            }
            188 => {
                // 恶缘：灵税催帖（mr3 恶缘薄档补位——仙府催赋题材此前零覆盖）
                card.title = "灵税催帖".into();
                card.text = "仙府催帖贴上洞府门楣：岁修灵税逾期，按例当缴——落款朱印鲜然，不容分说。".into();
                let mut e1 = Eff::new();
                e1.stones_pct = -0.06;
                e1.msg = vec!["灵石入库，朱印划去你的名讳——今年清净了。灵石-6%".into()];
                card.opts.push(("如数缴讫".into(), e1));
                let mut e2 = Eff::new();
                e2.stones_pct = -0.10;
                e2.good = false;
                e2.msg = vec![
                    "你闭门不见。税吏坐到日暮，留下一张加层的催单走了。灵石-10%".into(),
                ];
                card.opts.push(("闭门缓缴".into(), e2));
            }
            189 => {
                // 老年层：旧履还坊（「回望交托」散件第五条；余量≤10 载门控同批扩）
                card.title = "旧履还坊".into();
                card.text = "脚上这双履陪了你大半世。鞋匠铺老匠接过去端详半晌：「底磨得偏左——半世的路，都往一边偏。」".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.msg = vec![
                    "老匠纳好新底递还：「偏过的路不必正回来，记下就行。」道心+2".into(),
                ];
                card.opts.push(("换底重纳".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 1.0;
                e2.msg = vec![
                    "你没换底，只请他把旧底擦净——路走到头了，鞋是干净的。气运+1".into(),
                ];
                card.opts.push(("原样带走".into(), e2));
            }
            190 => {
                // 章节联动（第一章·雨夜执印已了门控 bit0）：雨夜回访——原点重临
                card.title = "雨夜寻印".into();
                card.text = "又是雨夜。路过乱葬岗，新坟旁一枚无字方印半埋土中，印身划痕层层叠叠——与你怀中那枚，一模一样。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.msg = vec![
                    "两印相贴，划痕严丝合缝：原是同一枚，同一个雨夜。道心+2".into(),
                ];
                card.opts.push(("并印相认".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 1.0;
                e2.msg = vec![
                    "你把印埋了回去：名字被写错的，从来不止你一个。气运+1".into(),
                ];
                card.opts.push(("覆土掩印".into(), e2));
            }
            191 => {
                // 章节联动（第四章·文狱讨牒已了门控 bit3）：牒成的回响——文狱来问
                card.title = "文狱退牒".into();
                card.text = "一名文书吏抱着一摞职牒拦路：「讨牒六张的名讳，文狱都记着——有人托我问一句：牒多了，压手么？」".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.luck = 1.0;
                e1.msg = vec![
                    "你把职牒摊开。文书吏数罢长揖：「压手——但压得住。」道心+2，气运+1".into(),
                ];
                card.opts.push(("以牒相示".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 1.0;
                e2.msg = vec![
                    "你收牒入袖，只留一句：压不压手，问笔台上自见分晓。道心+1".into(),
                ];
                card.opts.push(("收牒不语".into(), e2));
            }
            192 => {
                // 开局层：书僮窃墨（mr0 w4——摊薄开局头部集中度）
                card.title = "书僮窃墨".into();
                card.text = "借宿的书院里，小书僮揣了你一块松烟墨，被你堵在墙角——他仰起头理直气壮：「先生罚抄的《千字文》，我差一页的墨。」".into();
                let mut e1 = Eff::new();
                e1.dao = 1.0;
                e1.msg = vec![
                    "你把墨递还给他：「抄错的字，比抄对的长记性。」道心+1".into(),
                ];
                card.opts.push(("赠墨宽宥".into(), e1));
                let mut e2 = Eff::new();
                e2.qi_pct = 0.05;
                e2.msg = vec![
                    "罚他替你研一夜的墨。天明时墨细如霜，你写的字也顺了。修为+5%".into(),
                ];
                card.opts.push(("罚研一夜墨".into(), e2));
            }
            193 => {
                // mr6 长生世独占首批·善缘：后辈证道（realm≥6 藏锋起寿 4500 载层
                // 原零独占——长生世专属叙事首条；realm<mr 归零天然支持零代码）
                card.title = "后辈证道".into();
                card.text = "一名后辈在崖下摆开证道台，请帖递上山来：请山中三位最老的前辈观礼。你掐指数了数——这山中比你老的，早就不剩三位了。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.msg = vec![
                    "观礼毕，后辈登台前只问了一句：前辈可有赠语。你说：路比名长。他怔立良久，深深一拜。道心+2".into(),
                ];
                card.opts.push(("观礼赠语".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.msg = vec![
                    "你把一卷写了半世的旧稿托人捎去——纸旧字新，后辈若接着走，这条路又长了一程。气运+2".into(),
                ];
                card.opts.push(("捎一卷旧稿".into(), e2));
            }
            194 => {
                // mr6 恶缘：冒籍疑云（长生世独占恶缘首条——名讳被冒的高境烦恼）
                card.title = "冒籍疑云".into();
                card.text = "仙籍吏登门，面有难色：册上另有一个与你同名同姓同貌的修士，近年在北域行医济世，声名鹊起——籍只有一个，得去一个。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.msg = vec![
                    "你提笔把名字让了出去：名是虚的，走过的是真的。仙籍吏搁笔长揖，册页翻过，北域那人在册上落定。道心+2".into(),
                ];
                card.opts.push(("让名而去".into(), e1));
                let mut e2 = Eff::new();
                e2.stones_pct = -0.08;
                e2.good = false;
                e2.msg = vec![
                    "你据籍力争，文书往来半载，打点耗去灵石一成——籍保住了，名讳下从此多了一行小字：存疑。灵石-8%".into(),
                ];
                card.opts.push(("据籍力争".into(), e2));
            }
            195 => {
                // mr6 善缘：万卷尘页（长生世的回望题材——万卷旧稿只有走过的人读得动）
                card.title = "万卷尘页".into();
                card.text = "藏经阁顶层，你搁了不知多少万卷的旧稿自己翻动起来——尘页上的字迹早被岁月洇开，唯独你亲手写下的批注，一笔未褪。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.qi_pct = 0.05;
                e1.msg = vec![
                    "你坐回尘页之间，把万卷旧稿从头读了一遍——读到的全是路。合卷时，尘皆不惊。修为+5%，道心+2".into(),
                ];
                card.opts.push(("坐下来重读".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.msg = vec![
                    "你抽出一页批注，夹进新到的书堆：后来人翻到时，会有人在几百年前替他点过灯。气运+2".into(),
                ];
                card.opts.push(("留一页给后来人".into(), e2));
            }
            196 => {
                // 章节联动（第二章·镜中万象已了门控 bit1）：轮回镜回放首胜
                card.title = "镜影忆战".into();
                card.text = "轮回镜无故自亮，映出的不是当下，而是你的第一场胜仗——镜中那人出手时的破绽，隔着这些年岁，如今看得分明。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.msg = vec![
                    "你把那一战拆到每一息：原来赢的不是招，是你没退。镜影散时，破绽已补全。道心+2".into(),
                ];
                card.opts.push(("以镜复盘".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 1.0;
                e2.msg = vec![
                    "你抬手按灭了镜面：打过就打过，人生不是复盘。镜暗下去之前，那道影子似乎笑了一下。气运+1".into(),
                ];
                card.opts.push(("一笑关镜".into(), e2));
            }
            197 => {
                // 章节联动（第五章·山河万象已了门控 bit4）：五域风谣收最后一域
                card.title = "五域风谣".into();
                card.text = "一个背满歌册的行脚客拦路讨歌：五域风谣已收齐四域，独缺你脚下这一域——风谣须得本地人亲口唱，可愿补一句？".into();
                let mut e1 = Eff::new();
                e1.dao = 1.0;
                e1.luck = 1.0;
                e1.msg = vec![
                    "你随口唱了半句旧调。行脚客记罢大喜：五域齐了。此后五域的酒肆里，都传唱你这半句。道心+1，气运+1".into(),
                ];
                card.opts.push(("补一句谣".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.msg = vec![
                    "你以一页旧稿换他那册四域歌本——歌里走过的路，比你自己走过的还多。气运+2".into(),
                ];
                card.opts.push(("换一册歌本".into(), e2));
            }
            198 => {
                // 老年层（余量≤10 载门控）：窗前晒卷——晒的是一辈子的字
                card.title = "窗前晒卷".into();
                card.text = "好日头。你把一辈子的卷册搬到窗前晾晒，纸页在风里哗啦啦翻动成一片——每一页都晒出了旧年的声音。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.msg = vec![
                    "你随手抽出一页，读到当年写坏的一个字，笑了半晌——字没写好，路走直了。道心+2".into(),
                ];
                card.opts.push(("挑一页重读".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 1.0;
                e2.msg = vec![
                    "你枕着晒暖的卷册睡了一觉。醒来日头正好，风还在翻页——一字未忘。气运+1".into(),
                ];
                card.opts.push(("枕卷小睡".into(), e2));
            }
            199 => {
                // 老年层（余量≤10 载门控）：旧邻问名——名字的来历
                card.title = "旧邻问名".into();
                card.text = "山下旧邻的孩童上山采药，坐在你门槛上歇脚，仰头问：前辈的名字，是爹娘取的，还是自己取的？".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.msg = vec![
                    "你从名字的来历讲起，讲了整整一个下午。孩童下山时，把你的名字也带下了山——名字这东西，原是越讲越有的。道心+2".into(),
                ];
                card.opts.push(("从头讲给他听".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 1.0;
                e2.msg = vec![
                    "你只笑了笑。孩童走后你想：来历不急，等他自己走到要改名的年纪再讲不迟。气运+1".into(),
                ];
                card.opts.push(("只笑不答".into(), e2));
            }
            200 => {
                // mr5 恶缘：仙籍勾误（勾错名讳的文书灾——恶缘薄档补位）
                card.title = "仙籍勾误".into();
                card.text = "仙籍吏持册登门，面有难色：上头勾错了名讳，把你的寿数记到了旁人账上——更正要按例核验，得先收一样东西作保。".into();
                let mut e1 = Eff::new();
                e1.stones_pct = -0.08;
                e1.good = false;
                e1.msg = vec![
                    "灵石入库作保，朱笔重新誊写你的名讳与寿数——册上干净了，手头紧了。灵石-8%".into(),
                ];
                card.opts.push(("交灵石作保".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.msg = vec![
                    "你与仙籍吏对册三日，一条条驳到朱笔折了：勾错便是勾错。吏搁笔更正，临走郑重记下了你的较真。道心+2".into(),
                ];
                card.opts.push(("对册不认".into(), e2));
            }
            201 => {
                // mr3 恶缘：坊契赖账（赊出去的旧账与新东家的翻脸）
                card.title = "坊契赖账".into();
                card.text = "坊市老掌柜寻上门来：多年前赊出去的一批灵材，买主换了东家，新东家翻脸不认——老契在你手里压着，账却没人认了。".into();
                let mut e1 = Eff::new();
                e1.stones_pct = -0.04;
                e1.good = false;
                e1.msg = vec![
                    "你携契上门对了半日旧墨，打点上下认回一笔折半的旧账——讨回的还不够打点的，图个账面干净。灵石-4%".into(),
                ];
                card.opts.push(("携契追讨".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.msg = vec![
                    "你当着中人把老契烧了：账清了，纸也该歇了。火光里想明白一件事——赊出去的时候，原本就没打算收。道心+2".into(),
                ];
                card.opts.push(("勾销旧账".into(), e2));
            }
            202 => {
                // mr0 恶缘：夜叩柴门（开局层恶缘第 4 条——摊薄妖兽拦路 w11 独大）
                card.title = "夜叩柴门".into();
                card.text = "三更，柴门被叩得山响。门外人影幢幢，自称赶夜路的行商——可行商不走夜路，更不会人人在袖里揣着刀。".into();
                let mut e1 = Eff::new();
                e1.luck = 1.0;
                e1.msg = vec![
                    "你把门顶死，吹灯坐到天明。门外人四更散去，雪地上留了一圈徘徊的脚印。气运+1".into(),
                ];
                card.opts.push(("顶门不开".into(), e1));
                let mut e2 = Eff::new();
                e2.fight = 0.9;
                e2.good = false;
                card.opts.push(("拔剑逐客".into(), e2));
            }
            203 => {
                // 伴随事件（白名单第 14 条）：云海拾简（mr4 层）——半句残简与同读的人
                card.title = "云海拾简".into();
                let cn = {
                    let s = self.sim.as_ref().unwrap();
                    s.companion
                        .clone()
                        .unwrap_or_else(|| COMPANION_NAMES[0].to_string())
                };
                let ci = COMPANION_NAMES
                    .iter()
                    .position(|&n| n == cn)
                    .unwrap_or(0);
                card.actor = Some(CardActor::Comp(ci));
                card.text = format!(
                    "与{}踏云归途，云海里浮出一枚残简——简上只有半句，像谁写了一半，被风收走了后半。",
                    cn
                );
                let mut e1 = Eff::new();
                e1.comp_fav = crate::data::COMPANION_GIFT_FAV;
                e1.dao = 1.0;
                e1.msg = vec![format!(
                    "你们各执一端，把半句补成了全句——这枚简等了几百年，等的是一双同读的人。（好感+{}）道心+1",
                    crate::data::COMPANION_GIFT_FAV
                )];
                card.opts.push(("合力补全".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.msg = vec![
                    "你把残简放回云里：写它的人还会回来找的。云海合拢，像一页轻轻翻了过去。气运+2".into(),
                ];
                card.opts.push(("放回云海".into(), e2));
            }
            204 => {
                // 老年×有伴双门控首条：同席说岁——数算共同走过的年岁（两门控交集，
                // 无伴不揭此题、余量富余时不数）
                card.title = "同席说岁".into();
                let cn = {
                    let s = self.sim.as_ref().unwrap();
                    s.companion
                        .clone()
                        .unwrap_or_else(|| COMPANION_NAMES[0].to_string())
                };
                let ci = COMPANION_NAMES
                    .iter()
                    .position(|&n| n == cn)
                    .unwrap_or(0);
                card.actor = Some(CardActor::Comp(ci));
                card.text = format!(
                    "炉火将尽，你与{}同席对坐，把共同走过的年岁一年一年数着玩——数到某些年份，两个人同时笑出了声。",
                    cn
                );
                let mut e1 = Eff::new();
                e1.comp_fav = crate::data::COMPANION_GIFT_FAV;
                e1.dao = 2.0;
                e1.msg = vec![format!(
                    "你们把年岁数到了今年。旧事重数不是翻旧账，是再谢一遍。（好感+{}）道心+2",
                    crate::data::COMPANION_GIFT_FAV
                )];
                card.opts.push(("一年一年数到底".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.msg = vec![
                    "你说火要紧，起身添炭。身后传来笑：数不完的，来年接着数——正是，来年还长着呢。气运+2".into(),
                ];
                card.opts.push(("先去添一炉火".into(), e2));
            }

            // ---------- v1.16.19 扩池 14 条（205..=218）----------
            // 链15 观星问岁 3（205 首段 mr6 w3 + 206/207 链内 w0；醉道人携酒
            // 观星指星对账，Npc(3) 真立绘；置链臂 56/57 第八窗见
            // CHAIN15_SPECIAL_LO/HI）/mr7 长生世风味 2（208 剑冢阅武 w4 善/
            // 209 界碑拓铭 w3 恶——mr 列首现 7，realm<mr 归零天然支持零新代码；
            // 措辞守长生世纪律：不写寿元将尽、不触破境裁决）/伴随 3（210 雨檐
            // 听铃 mr3 补伴随 mr3 零位、211 云上放灯 mr5 高境有伴、212 相扶看雪
            // 老年×有伴双门控第二条照 204 模板——白名单 14→17，全挂 Comp 立绘
            // 照 82/203 反查模式）/
            // mr6 稀有 1（213 仙池映世 w2——稀有层 mr6 首条）+ mr6 恶缘 1
            //（214 晚钟误点，mr6 恶缘 194 独苗补一条）/老年层 2（215 灶火留谱/
            // 216 旧约转托，「交托回望」题材，门控 id 集合同批扩）/章节联动 1
            //（217 邻人扛劫 = 第七章 bit6，缺角补齐；「邻人扛劫」侧写守链11
            // 分域纪律）/mr0 恶缘 1（218 囊带乍断 w3，摊薄开局头部）
            205 => {
                // 链15 首段：观星问岁（醉道人——醉里数星，指星对账；首段建链待分支）
                card.title = "观星问岁".into();
                card.actor = Some(CardActor::Npc(3));
                card.text = "高山夜清，醉道人拉你在崖头坐下，酒葫芦往石上一搁：「数星星去。天上每一颗都是一笔账——只是记账的那位，岁数大了，时常漏记。」".into();
                let mut e1 = Eff::new();
                e1.dao = 1.0;
                e1.special = 56; // 置链：→206 指间星图
                e1.msg = vec![
                    "你依言抬头替他数星。数到第七颗，他忽然不说话了——盯着其中一颗，看了很久。道心+1".into(),
                ];
                card.opts.push(("替他数星".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 1.0;
                e2.luck = 1.0;
                e2.msg = vec![
                    "你说「账是死的，酒是热的」。他大笑，把葫芦递你抿了一口——星没数成，夜倒是好夜。道心+1，气运+1".into(),
                ];
                card.opts.push(("只陪他喝酒".into(), e2));
                self.chain = Some((15, 0, CHAIN_TTL)); // 首段建链：待分支
            }
            206 => {
                // 链15 中段（仅链内）
                card.title = "指间星图·续".into();
                card.actor = Some(CardActor::Npc(3));
                card.text = "他以指蘸酒，在石桌上画星图——一笔一划都是旧年的天。画到末了，指头停在一处空缺：「这儿该有一颗的。是我记漏了，还是天漏了？」".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.special = 57; // 置链：→207 问岁之约
                e1.msg = vec![
                    "你接过他的指头，替那颗星补上。酒渍在月光下微微发亮，像真有一颗星落进了石桌。道心+2".into(),
                ];
                card.opts.push(("替他补上".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 1.0;
                e2.msg = vec![
                    "你说「漏了就漏着，留个缺处透气」。他愣了愣，抚掌大笑——星图到底没补。气运+1".into(),
                ];
                card.opts.push(("劝他留着缺".into(), e2));
            }
            207 => {
                // 链15 末段（仅链内）：两选项均无置链臂 → 抉择即链终局（缘了）
                card.title = "问岁之约·续".into();
                card.actor = Some(CardActor::Npc(3));
                card.text = "醉道人指着方才那颗星：「看，它挪了一格。那是我自己的名字——挪一格，就老一分。」他把星图一角撕给你：「明年今夜，来替我对一次账。」".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.luck = 2.0;
                e1.msg = vec![
                    "你把那角酒渍星图收进怀里，应下来年之约。他想了想，又补一句：「带上酒。」道心+2，气运+2".into(),
                ];
                card.opts.push(("应下来年之约".into(), e1));
                let g = (30.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!(
                    "你说「对账可以，谢礼照付」——他笑着摸出一袋旧年攒的星砂抵账。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("先收对账的谢礼".into(), e2));
            }
            208 => {
                // mr7 长生世风味·善缘：剑冢阅武（realm≥7 铁画起；不触破境裁决）
                card.title = "剑冢阅武".into();
                card.text = "万剑归冢的山谷夜夜有剑鸣，如群剑自阅武。今夜鸣声忽然齐了一瞬——冢中有一柄无名剑，朝着你的剑鞘遥遥一引，像在邀你下场走一遭。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.12;
                e1.dao = 1.0;
                e1.msg = vec![
                    "你拔剑入场，与万剑各过一招。剑鸣送你出谷时，齐齐轻颤了三声——是喝彩的意思。修为+12%，道心+1".into(),
                ];
                card.opts.push(("应召赴阅".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.msg = vec![
                    "你抱剑观礼到天明，把每一路剑势默记在心。晨光里剑鸣渐息，如群剑各自归鞘。道心+2".into(),
                ];
                card.opts.push(("观而不试".into(), e2));
            }
            209 => {
                // mr7 长生世风味·恶缘：界碑拓铭（界碑名讳洇雨，拓碑吏索重铭费）
                card.title = "界碑拓铭".into();
                card.text = "天地界碑高可摩云，碑上万灵名讳如林。一名拓碑吏踏云而来，面无表情：连日阴雨，你名下那一笔铭文洇了——按例当重铭，工费自付。".into();
                let g = (60.0 * ss) as i64;
                let mut e1 = Eff::new();
                e1.stones = -g;
                e1.good = false;
                e1.msg = vec![format!(
                    "你付了重铭的工费。新墨落碑，笔画比旧年深了几分——像谁怕它再洇一次。灵石-{}",
                    fmt_int(g)
                )];
                card.opts.push(("付工费重铭".into(), e1));
                let mut e2 = Eff::new();
                e2.qi_pct = -0.06;
                e2.dao = 2.0;
                e2.msg = vec![
                    "你说「我的名字，我自己写」。你以修为引气为墨，亲手重铭——笔画入碑三分，行气也耗了三分。修为-6%，道心+2".into(),
                ];
                card.opts.push(("亲笔重铭".into(), e2));
            }
            210 => {
                // 伴随事件（白名单第 15 条）：雨檐听铃（mr3 层，补伴随 mr3 零位）
                card.title = "雨檐听铃".into();
                let cn = {
                    let s = self.sim.as_ref().unwrap();
                    s.companion
                        .clone()
                        .unwrap_or_else(|| COMPANION_NAMES[0].to_string())
                };
                let ci = COMPANION_NAMES
                    .iter()
                    .position(|&n| n == cn)
                    .unwrap_or(0);
                card.actor = Some(CardActor::Comp(ci));
                card.text = format!(
                    "夜雨忽至，你与{}躲进檐下。檐角一只旧铃被雨点敲响——那是旧年你们亲手挂上去的，说好谁忘了归途，就循铃声回来。",
                    cn
                );
                let mut e1 = Eff::new();
                e1.comp_fav = crate::data::COMPANION_GIFT_FAV;
                e1.dao = 1.0;
                e1.msg = vec![format!(
                    "你们谁也没说话，听了一夜的铃。铃声不催人，倒像在替你们把旧话重说了一遍。（好感+{}）道心+1",
                    crate::data::COMPANION_GIFT_FAV
                )];
                card.opts.push(("听铃到雨停".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.msg = vec![
                    "你把铃翻面重新系好——旧铃换个朝向，声音竟清了一调。你说：「这样走远了也听得见。」气运+2".into(),
                ];
                card.opts.push(("替铃换个朝向".into(), e2));
            }
            211 => {
                // 伴随事件（白名单第 16 条）：云上放灯（mr5 高境有伴风味）
                card.title = "云上放灯".into();
                let cn = {
                    let s = self.sim.as_ref().unwrap();
                    s.companion
                        .clone()
                        .unwrap_or_else(|| COMPANION_NAMES[0].to_string())
                };
                let ci = COMPANION_NAMES
                    .iter()
                    .position(|&n| n == cn)
                    .unwrap_or(0);
                card.actor = Some(CardActor::Comp(ci));
                card.text = format!(
                    "云海之上风平如镜。{}提议放一盏灯——不为许愿，只为看看它究竟能穿多厚的云。灯芯是你亲手搓的，捻得很紧。",
                    cn
                );
                let mut e1 = Eff::new();
                e1.comp_fav = crate::data::COMPANION_GIFT_FAV;
                e1.luck = 1.0;
                e1.msg = vec![format!(
                    "灯穿过了三层云，光透下来的时候，云海亮出了一道细缝——像天替你们掀了掀帘。（好感+{}）气运+1",
                    crate::data::COMPANION_GIFT_FAV
                )];
                card.opts.push(("看灯穿云".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.msg = vec![
                    "灯走到半程熄了。你说「熄在半路也是到了半路」。两人把灯壳捞回来，说好明年换粗芯再放。道心+2".into(),
                ];
                card.opts.push(("捞回灯壳".into(), e2));
            }
            212 => {
                // 老年×有伴双门控第二条：相扶看雪（照 204 模板——余量≤10 载且有伴才入池）
                card.title = "相扶看雪".into();
                let cn = {
                    let s = self.sim.as_ref().unwrap();
                    s.companion
                        .clone()
                        .unwrap_or_else(|| COMPANION_NAMES[0].to_string())
                };
                let ci = COMPANION_NAMES
                    .iter()
                    .position(|&n| n == cn)
                    .unwrap_or(0);
                card.actor = Some(CardActor::Comp(ci));
                card.text = format!(
                    "落了今冬第一场雪。你与{}相扶着坐到廊下看雪，谁也不点灯——雪光够亮了。落雪无声，你们的话也渐渐少了，只剩两件旧棉衣挨在一处。",
                    cn
                );
                let mut e1 = Eff::new();
                e1.comp_fav = crate::data::COMPANION_GIFT_FAV;
                e1.dao = 2.0;
                e1.msg = vec![format!(
                    "你把肩往她那边靠了靠，她把毯往你这边挪了挪。雪落了一夜，谁也没觉得冷。（好感+{}）道心+2",
                    crate::data::COMPANION_GIFT_FAV
                )];
                card.opts.push(("同看一场雪".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.msg = vec![
                    "你在雪地里倒退着走了一圈，只为让她看一眼两个人的脚印并排印到廊前。她笑出了声。气运+2".into(),
                ];
                card.opts.push(("踏一串脚印".into(), e2));
            }
            213 => {
                // mr6 稀有·善缘：仙池映世（w2 入稀有层；池水映第一世的脸）
                card.title = "仙池映世".into();
                card.text = "高岭有一泓仙池，水色如凝墨。临水一照——池中映出的却不是你如今的脸，而是一副你认得又陌生的眉眼：那是你头一世的脸。".into();
                let mut e1 = Eff::new();
                e1.dao = 3.0;
                e1.msg = vec![
                    "你在池边枯坐一日，看那张脸从陌生看到眼熟。起身时池水轻晃，两张脸叠在一处——原来早已是一个人。道心+3".into(),
                ];
                card.opts.push(("池边枯坐一日".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 3.0;
                e2.msg = vec![
                    "你汲起一壶池水收好。水在壶中夜夜映一张不同的旧脸——像有旧相识轮流来看你。气运+3".into(),
                ];
                card.opts.push(("汲一壶池水".into(), e2));
            }
            214 => {
                // mr6 恶缘：晚钟误点（钟多敲一记，人心自扰——不触破境裁决）
                card.title = "晚钟误点".into();
                card.text = "入夜，境内古钟报晚的钟声多敲了一记。钟声不大，人心却乱了——多一记，依古例是「催岁」，一时满城修者疑心大限，道途都滞了。".into();
                let g = (40.0 * ss) as i64;
                let mut e1 = Eff::new();
                e1.stones = -g;
                e1.good = false;
                e1.luck = 1.0;
                e1.msg = vec![format!(
                    "你出钱请钟楼老吏登楼验钟——果然是钟舌松了半分。老吏修钟，众心渐安，你手头却紧了。灵石-{}，气运+1",
                    fmt_int(g)
                )];
                card.opts.push(("出钱验钟".into(), e1));
                let mut e2 = Eff::new();
                e2.qi_pct = -0.05;
                e2.dao = 1.0;
                e2.msg = vec![
                    "你不理会流言，自行功到天明——可满城疑气入夜不散，行功也滞了三分。修为-5%，道心+1".into(),
                ];
                card.opts.push(("自行不理".into(), e2));
            }
            215 => {
                // 老年层（余量≤10 载门控）：灶火留谱——把火候留给山下灶口
                card.title = "灶火留谱".into();
                card.text = "山下饭铺的灶火总不如你洞府的旺。掌勺的少年上来讨教过几回，眼睛亮——你一辈子的火候，都在几张灶谱的批注里。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.msg = vec![
                    "你把批注过的灶谱誊清了一份给他。少年捧谱下山，当晚饭铺的烟火气就变了——火候这东西，原是可以传的。道心+2".into(),
                ];
                card.opts.push(("誊谱相赠".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 1.0;
                e2.msg = vec![
                    "谱可以给，你却只口传，不落纸——「火候在手上，抄在手上的谱丢不了」。少年记住了，你也记起了年轻时的自己。气运+1".into(),
                ];
                card.opts.push(("只传口诀".into(), e2));
            }
            216 => {
                // 老年层（余量≤10 载门控）：旧约转托——年轻时未赴的旧约
                card.title = "旧约转托".into();
                card.text = "翻旧箱翻出一枚旧笺——年轻时与人约定同去看一场「彻夜不谢的灯」，后来各自西东，笺压了箱底，灯会年年还办。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.msg = vec![
                    "你修书一封，连笺一并托后辈代赴，信尾写：「替我看，替我笑，笑出声。」灯会那夜，山下果然有笑声传上来。道心+2".into(),
                ];
                card.opts.push(("托人代赴".into(), e1));
                let g = (35.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = -g;
                e2.luck = 2.0;
                e2.msg = vec![format!(
                    "你把旧笺揣进袖子，亲自下了山——灯会彻夜不谢，你看到了天明。迟到的约也是约。灵石-{}，气运+2",
                    fmt_int(g)
                )];
                card.opts.push(("亲自赴约".into(), e2));
            }
            217 => {
                // 章节联动（第七章 bit6）：邻人扛劫——只看旁人扛劫，绝不触玩家自身破境裁决
                card.title = "邻人扛劫".into();
                card.text = "邻山方向雷云盘结了一整夜。你登高望去，隔谷有人影立于劫云之下——九重雷一重一重落，那人一重一重扛，始终没有跪。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.msg = vec![
                    "你在崖头点起一盏灯，不为照路，只为让雷光里的人知道有人看着。劫云散时，对面遥遥一揖。道心+2".into(),
                ];
                card.opts.push(("遥遥举灯".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 1.0;
                e2.msg = vec![
                    "你收了灯，退回洞中静坐——有人扛自己的劫，旁人扰一句都是添乱。天明雷歇，你心里也落定了一事。气运+1".into(),
                ];
                card.opts.push(("收灯不扰".into(), e2));
            }
            218 => {
                // mr0 恶缘：囊带乍断（开局层恶缘——摊薄妖兽拦路独大）
                card.title = "囊带乍断".into();
                card.text = "山道转过一处陡坡，行囊背带「啪」地断了——囊口一歪，几块灵石顺着碎石坡骨碌碌往下滚，滚一声，你的心就跟着沉一声。".into();
                let mut e1 = Eff::new();
                e1.stones_pct = -0.04;
                e1.good = false;
                e1.msg = vec![
                    "你连滚带爬追下去，捡回大半，指缝里还是漏了几块——碎石坡不还你东西。灵石-4%".into(),
                ];
                card.opts.push(("俯身追捡".into(), e1));
                let g = (18.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = -g;
                e2.good = false;
                e2.dao = 1.0;
                e2.msg = vec![format!(
                    "你任它们滚，坐下来慢慢补带——「身外物，追不得」。补好了囊，也补明白了一件事。灵石-{}，道心+1",
                    fmt_int(g)
                )];
                card.opts.push(("补带不追".into(), e2));
            }

            // ---------- v1.16.20 扩池 12 条（219..=230）----------
            // 链16 剑穗之约 3（219 首段 mr4 w3 善缘 + 220/221 链内 w0；宁清霜
            // 剑穗线 Npc(2) 真立绘；置链臂 60/61 第九窗见 CHAIN16_SPECIAL_LO/HI，
            // 220 的「一夜结完」走 62 窗内不置链——eff_desc 有分支副标，链态随
            // 下次 sim_event 弃链；CHAIN_FIRSTS/CHAIN_NAMES/gallery 链起名单
            // 三处同批登记）/mr7 最薄层 3（222 星砂还账 w4 善 Npc(3) 立绘——
            // 链15「明年今夜」散件呼应不依赖链态；223 玉价新例 w3 恶；224 坠星
            // 动冢 w2 稀有层 mr7 首条——长生世纪律：不写寿元将尽、不触破境
            // 裁决）/伴随 2（225 同数流星 mr6 有伴首条/226 霜晨共伞 mr0 有伴
            // 第 2 条照 174 成对分域；白名单 17→19，全挂 Comp 立绘照 210 反查
            // 模式）/老年层 1（227 半檐躲雨，门控 id 集合同批扩）/mr6 常态 1
            //（228 池边洗鞘 w3 善）/mr3 恶缘 1（229 佃契抽成 w3，与 39 灵田
            // 佃户成对分域）/章节联动 1（230 落款撞名 = 第八章 bit7 缺角补齐，
            // 与结局回响「世界侧写」分域写亲历视角）
            219 => {
                // 链16 首段：剑穗生结（宁清霜——旧剑穗磨断，求一个慢结；首段建链待分支）
                card.title = "剑穗生结".into();
                card.actor = Some(CardActor::Npc(2));
                card.text = "演武场边风急。宁清霜解下旧剑——剑穗磨得只剩几缕，她却没丢，递来一束新丝：「替这穗打个结——要打得慢。」".into();
                let mut e1 = Eff::new();
                e1.dao = 1.0;
                e1.special = 60; // 置链：→220 剑穗缓结
                e1.msg = vec![
                    "你接过丝线。她的剑穗与旁人不同——结要贴着剑格走，急不得。你说「慢工我也有」。道心+1".into(),
                ];
                card.opts.push(("应下这个结".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 1.0;
                e2.luck = 1.0;
                e2.msg = vec![
                    "你说「剑是主人事，穗也是」。她收线一笑：「倒是稳字当头。」道心+1，气运+1".into(),
                ];
                card.opts.push(("婉辞".into(), e2));
                self.chain = Some((16, 0, CHAIN_TTL)); // 首段建链：待分支
            }
            220 => {
                // 链16 中段（仅链内）
                card.title = "剑穗缓结·续".into();
                card.actor = Some(CardActor::Npc(2));
                card.text = "丝线里缠着一缕旧穗——她师尊留的。这穗要缓结：一日一寸，丝才不吃力。你打了一半，她忽然说：「我等的，是有人陪我慢一回。」".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.special = 61; // 置链：→221 剑成赠穗
                e1.msg = vec![
                    "你收了急性，一日结一寸。第七日穗成，贴着剑格，风过不散。道心+2".into(),
                ];
                card.opts.push(("依她一日一寸".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 1.0;
                e2.special = 62; // 窗内不置链臂：中段断链收获（eff_desc「一夜结完」可辨）
                e2.msg = vec![
                    "你以剑气定丝，一夜结成——快是快，穗心收得紧。她看了看，笑：「这也像你的剑。」气运+1".into(),
                ];
                card.opts.push(("替她一夜结完".into(), e2));
            }
            221 => {
                // 链16 末段（仅链内）：两选项均无置链臂 → 抉择即链终局（缘了）
                card.title = "剑成赠穗·续".into();
                card.actor = Some(CardActor::Npc(2));
                card.text = "穗成之日，她把旧穗里师尊的那缕丝分出一段，系成小结递给你：「剑穗护手，也护心。这截旧丝跟了我三十年——如今换它护你。」".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.luck = 2.0;
                e1.msg = vec![
                    "你把小结系在腕上。她转身还剑入鞘，剑鸣轻轻——像道谢，又像道别旧物。道心+2，气运+2".into(),
                ];
                card.opts.push(("郑重收下".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 3.0;
                e2.msg = vec![
                    "你把小结又塞回她手里：「旧丝护旧主，更稳。」她怔了怔，收穗入怀。道心+3".into(),
                ];
                card.opts.push(("转赠还她".into(), e2));
            }
            222 => {
                // mr7 善缘：星砂还账（醉道人 Npc(3)——链15「明年今夜」散件呼应，
                // 不依赖链态；跨世新面孔，他记得的是事不是脸）
                card.title = "星砂还账".into();
                card.actor = Some(CardActor::Npc(3));
                card.text = "醉道人踏月而来，摊开手心一小撮星砂：「去年今夜有人替我数过星。这是谢礼——可惜数星的人换了一副面目，你见着他了么？」".into();
                let g = (30.0 * ss) as i64;
                let mut e1 = Eff::new();
                e1.stones = g;
                e1.luck = 1.0;
                e1.msg = vec![format!(
                    "你说「数星的人收过谢了，这份替他转收」——他把砂倒进你袖中，微微发温。灵石+{}，气运+1",
                    fmt_int(g)
                )];
                card.opts.push(("接下星砂".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.msg = vec![
                    "你说「数星的账数不完」——两人又坐到天明。他把空葫芦系回腰间：「明年的账，也记你头上。」道心+2".into(),
                ];
                card.opts.push(("陪他再数一夜".into(), e2));
            }
            223 => {
                // mr7 恶缘：玉价新例（玉价涨，工钱照新例——账目分明不容还价）
                card.title = "玉价新例".into();
                card.text = "集尾一位玉匠拦路，捧出你寄磨半载的道玉：「玉成了。只是今年玉价涨了三成——工钱照新例，也涨三成。」账目分明，不容还价。".into();
                let g = (60.0 * ss) as i64;
                let mut e1 = Eff::new();
                e1.stones = -g;
                e1.good = false;
                e1.msg = vec![format!(
                    "你如数付了新例工钱。玉入手温润生光——他拱手：「涨价的是玉，不是手艺。」灵石-{}",
                    fmt_int(g)
                )];
                card.opts.push(("付新例工钱".into(), e1));
                let mut e2 = Eff::new();
                e2.qi_pct = -0.05;
                e2.dao = 1.0;
                e2.msg = vec![
                    "你以气机压场，硬按旧例付了——他不再争，收钱转身。那玉到你手里，光泽竟暗了一分。修为-5%，道心+1".into(),
                ];
                card.opts.push(("凭修为压价".into(), e2));
            }
            224 => {
                // mr7 稀有·善缘：坠星动冢（w2 入稀有层，mr7 首条；只写万剑向天
                // 揖让的礼数，不触玩家自身破境裁决）
                card.title = "坠星动冢".into();
                card.text = "万剑归冢的山谷忽然静了——一颗坠星划过天顶，星未落，冢中万剑先齐齐低鸣一瞬，如群剑向天揖让。你行经谷口，剑鞘嗡嗡相和。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.15;
                e1.dao = 2.0;
                e1.msg = vec![
                    "你按剑驻足，把那一瞬的静记进心里——万剑让的是天，不是星。出谷时行功顺了三分。修为+15%，道心+2".into(),
                ];
                card.opts.push(("驻足观礼".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 3.0;
                e2.msg = vec![
                    "你解下剑鞘，朝天一拜——不为坠星，为万剑的礼数。谷风送你出山，一路顺遂。气运+3".into(),
                ];
                card.opts.push(("解鞘一拜".into(), e2));
            }
            225 => {
                // 伴随事件（白名单第 18 条）：同数流星（mr6 有伴层首条）
                card.title = "同数流星".into();
                let cn = {
                    let s = self.sim.as_ref().unwrap();
                    s.companion
                        .clone()
                        .unwrap_or_else(|| COMPANION_NAMES[0].to_string())
                };
                let ci = COMPANION_NAMES
                    .iter()
                    .position(|&n| n == cn)
                    .unwrap_or(0);
                card.actor = Some(CardActor::Comp(ci));
                card.text = format!(
                    "高崖夜静，{}忽然拽你往下指——流星正坠，一前一后有两道。「一人一道，谁也不用让谁。」你们并肩数完了这场流星雨。",
                    cn
                );
                let mut e1 = Eff::new();
                e1.comp_fav = crate::data::COMPANION_GIFT_FAV;
                e1.dao = 1.0;
                e1.msg = vec![format!(
                    "数到最后你们都笑了——两个人数星，快一倍，也暖一倍。（好感+{}）道心+1",
                    crate::data::COMPANION_GIFT_FAV
                )];
                card.opts.push(("并肩数星".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.msg = vec![
                    "你说「流星许愿一人一个」——你们各许各的，谁也没偷听谁的。气运+2".into(),
                ];
                card.opts.push(("各许各的".into(), e2));
            }
            226 => {
                // 伴随事件（白名单第 19 条）：霜晨共伞（mr0 有伴第 2 条，照 174
                // 霜桥早市成对分域——同是霜晨，这边写两人让伞）
                card.title = "霜晨共伞".into();
                let cn = {
                    let s = self.sim.as_ref().unwrap();
                    s.companion
                        .clone()
                        .unwrap_or_else(|| COMPANION_NAMES[0].to_string())
                };
                let ci = COMPANION_NAMES
                    .iter()
                    .position(|&n| n == cn)
                    .unwrap_or(0);
                card.actor = Some(CardActor::Comp(ci));
                card.text = format!(
                    "霜晨赶集，{}撑伞追上你——伞不大，她把大半让给你，肩头落霜也不说。半路你接过伞，她笑了：「原来你也会让。」",
                    cn
                );
                let mut e1 = Eff::new();
                e1.comp_fav = crate::data::COMPANION_GIFT_FAV;
                e1.dao = 1.0;
                e1.msg = vec![format!(
                    "一路换了两回伞，谁的肩都没湿。集口粥摊热气扑面——霜晨的暖是让出来的。（好感+{}）道心+1",
                    crate::data::COMPANION_GIFT_FAV
                )];
                card.opts.push(("轮流撑伞".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 1.0;
                e2.msg = vec![
                    "你把伞整个倾向她那边，自己湿了半边肩——她板起脸把你拽回伞下：「胡闹。」气运+1".into(),
                ];
                card.opts.push(("整伞相让".into(), e2));
            }
            227 => {
                // 老年层（余量≤10 载门控）：半檐躲雨——半边檐下两人一雨
                card.title = "半檐躲雨".into();
                card.text = "急雨没处躲，你挨进一处半檐——檐下已站了个避雨的货郎，见你年事高，默默往里让了半步。两个人，一场雨，谁也没说话。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.msg = vec![
                    "雨停时货郎扶了你一把。你说「这场雨躲得好」——檐下的慢也是日子，年轻时嫌费，如今才懂。道心+2".into(),
                ];
                card.opts.push(("共檐静候".into(), e1));
                let g = (20.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = -g;
                e2.luck = 2.0;
                e2.msg = vec![format!(
                    "你把旧伞塞给他，摆手走了。身后货郎追了两步喊了声「谢」——声撞在雨里，听着舒服。灵石-{}，气运+2",
                    fmt_int(g)
                )];
                card.opts.push(("以伞相赠".into(), e2));
            }
            228 => {
                // mr6 常态·善缘：池边洗鞘（旧痕浮出当年一击——描一遍记一遍稳）
                card.title = "池边洗鞘".into();
                card.text = "池水见底。你解下剑鞘浸在池里搓洗——鞘上一道旧痕浮出来，是多年前替人挡下的一击。水纹荡开，像那年替你心惊的人还在岸上。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.msg = vec![
                    "你没把痕洗掉，反以指尖蘸水描深一分——挡过的击就是挡过，描一遍，记一遍稳。道心+2".into(),
                ];
                card.opts.push(("重描旧痕".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 1.0;
                e2.msg = vec![
                    "你把鞘洗得干干净净收好——旧痕留鞘底，日子朝前走。起身时池面风平，气也顺。气运+1".into(),
                ];
                card.opts.push(("洗净收鞘".into(), e2));
            }
            229 => {
                // mr3 恶缘：佃契抽成（田主侧——与 39 灵田佃户佃户侧成对分域）
                card.title = "佃契抽成".into();
                card.text = "灵田佃户捧着佃契上门：秋收薄了三成，佃契上的抽成一字未改——他求你减，又说「契在，减不减都依你」。灯把两张脸照得一样黄。".into();
                let g = (50.0 * ss) as i64;
                let mut e1 = Eff::new();
                e1.stones = g;
                e1.good = false;
                e1.msg = vec![format!(
                    "你照契收了抽成。他背着空袋出门，脚步比来时沉——契没改，改的是来年谁还敢佃你的田。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("照契抽成".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 2.0;
                e2.luck = 1.0;
                e2.msg = vec![
                    "你提笔把抽成削去一半——「契是死账，人是活人」。他叩头走了；来年他的田比谁家都勤。道心+2，气运+1".into(),
                ];
                card.opts.push(("当堂让利".into(), e2));
            }
            230 => {
                // 章节联动（第八章 bit7）：落款撞名——与结局回响 165-167「世界
                // 侧写」分域，写亲历视角（落款是你的，册子不是）
                card.title = "落款撞名".into();
                card.text = "书成之后你翻检旧稿，一册无名书上落着你的名字——笔画分毫不差，落款那日你却在千里之外。管书吏查遍册底，查不出第二只手。".into();
                let mut e1 = Eff::new();
                e1.dao = 3.0;
                e1.msg = vec![
                    "你提笔在册页角补了半行小字，算与那位同名者打了照面——书成之后，连名字都有了邻人。道心+3".into(),
                ];
                card.opts.push(("收册题字".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.msg = vec![
                    "你说「名字是我的，书不是」——吏当场焚册，灰里浮出一行新字：「后会有期。」气运+2".into(),
                ];
                card.opts.push(("交吏焚册".into(), e2));
            }
            231 => {
                // 链17 首段：星槎再访（187 星槎访客续线，mr5 零链补位；首段建链待分支）
                card.title = "星槎再访".into();
                card.text = "又见星槎。槎上客认出你，隔云一揖：「上次问的路，高士如今走出名了么？说来听听。」".into();
                let mut e1 = Eff::new();
                e1.dao = 1.0;
                e1.special = 63; // 置链：→232 星轨留书·续
                e1.msg = vec![
                    "你把一路行来的地名一一说与他。他记在星轨上——「下一位问路者，就有名可寻了。」道心+1".into(),
                ];
                card.opts.push(("以路相赠".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.msg = vec![
                    "你说「路要自己走才有名字」。他大笑，槎去云开，留下一颗星子在你袖中发亮。气运+2".into(),
                ];
                card.opts.push(("只陪他坐一夜".into(), e2));
                self.chain = Some((17, 0, CHAIN_TTL)); // 首段建链：待分支
            }
            232 => {
                // 链17 中段（仅链内）
                card.title = "星轨留书·续".into();
                card.text = "他引你到槎前——槎心嵌着一幅星轨图，唯独缺一笔：「上回你说的路，我记到一半，天快亮了。」".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.special = 64; // 置链：→233 槎去留星·续
                e1.msg = vec![
                    "你以指为笔补上那一笔。星轨微微一颤，自行延向云海深处——那里有你没走过的路。道心+2".into(),
                ];
                card.opts.push(("替他补全星轨".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 1.0;
                e2.msg = vec![
                    "你说「缺处是给后来人留的」。他点头收图，槎启——你袖中星子暗了一分。气运+1".into(),
                ];
                card.opts.push(("劝他留着缺".into(), e2));
            }
            233 => {
                // 链17 末段（仅链内）：两选项均无置链臂 → 抉择即链终局（缘了）
                card.title = "槎去留星·续".into();
                card.text = "临别，星客把那幅星轨图拆成两半，一半插进你的行囊：「路无名时，就回来看它——记得替后来的人添一笔。」".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.luck = 1.0;
                e1.msg = vec![
                    "半幅星轨贴着行囊内壁，夜行时微微发亮——像有人在前路替你掌灯。道心+2，气运+1".into(),
                ];
                card.opts.push(("收下半幅星轨".into(), e1));
                let g = (30.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = g;
                e2.msg = vec![format!(
                    "你说「路我自己走」——他也不强留，抛下一袋星砂抵那半幅图的纸钱。灵石+{}",
                    fmt_int(g)
                )];
                card.opts.push(("把图还他".into(), e2));
            }
            234 => {
                // 稀有：旧砚问道（w2 mr2；special66 道心检定 / 买砚小得）
                card.title = "旧砚问道".into();
                card.text = "荒摊角落一方旧砚，砚池无墨自润。摊主不言，只以指叩砚——砚声如叩问：「你修道，为的什么？」".into();
                let mut e1 = Eff::new();
                e1.special = 66;
                e1.good = false;
                card.opts.push(("应声作答".into(), e1));
                let g = (35.0 * ss) as i64;
                let mut e2 = Eff::new();
                e2.stones = -g;
                e2.dao = 1.0;
                e2.msg = vec![format!(
                    "你不答，只把砚买下——「答案自己磨。」摊主目送你远去，若有所思。灵石-{}，道心+1",
                    fmt_int(g)
                )];
                card.opts.push(("买下旧砚".into(), e2));
            }
            235 => {
                // mr5 恶缘：影市黑秤（恶缘薄档补位——影市暗秤题材此前零覆盖）
                card.title = "影市黑秤".into();
                card.text = "影市深处，黑秤匠替你称行囊——秤成他袖手一笑：「这秤准得很，只是准的从来是我的秤。」".into();
                let g = (45.0 * ss) as i64;
                let mut e1 = Eff::new();
                e1.stones = -g;
                e1.good = false;
                e1.msg = vec![format!(
                    "差价三成。你补了——影市规矩：秤不可回改，账不可重算。灵石-{}",
                    fmt_int(g)
                )];
                card.opts.push(("补差价了事".into(), e1));
                let mut e2 = Eff::new();
                e2.stones_pct = -0.05;
                e2.luck = 1.0;
                e2.msg = vec![
                    "你掀秤而去。影市从此对你半开门——此后买进卖出都吃了暗亏，腰杆倒是直了。灵石-5%，气运+1".into(),
                ];
                card.opts.push(("掀秤而去".into(), e2));
            }
            236 => {
                // mr7 善缘：云海拾阶（w4；守长生世纪律——不写寿元不触破境裁决）
                card.title = "云海拾阶".into();
                card.text = "云海退去一瞬，崖前露出一段旧石阶，阶尽处一座无名旧观，观门虚掩，蒲团犹温。".into();
                let mut e1 = Eff::new();
                e1.qi_pct = 0.10;
                e1.dao = 1.0;
                e1.msg = vec![
                    "观中无人。你坐了半日，行功顺遂，下山时云海已平、阶也没了。修为+10%，道心+1".into(),
                ];
                card.opts.push(("拾阶叩门".into(), e1));
                let mut e2 = Eff::new();
                e2.luck = 2.0;
                e2.msg = vec![
                    "你不上去——把石阶留给下一个走得动的人。云海复涨，把旧观收了回去。气运+2".into(),
                ];
                card.opts.push(("坐观云海".into(), e2));
            }
            237 => {
                // mr7 恶缘：剑赋催帖（双臂「纳赋/以剑代赋」——fight 臂照 4 恶修劫道模板）
                card.title = "剑赋催帖".into();
                card.text = "剑冢岁帖贴上你的剑鞘：佩剑者岁纳剑赋，例不可免。帖角另有一行小字——「或以剑代之」。".into();
                let g = (55.0 * ss) as i64;
                let mut e1 = Eff::new();
                e1.stones = -g;
                e1.good = false;
                e1.msg = vec![format!(
                    "剑赋入库，帖角焚作青烟——群剑的鸣声低了三分，像一声「收讫」。灵石-{}",
                    fmt_int(g)
                )];
                card.opts.push(("如数奉纳".into(), e1));
                let mut e2 = Eff::new();
                e2.fight = 1.0;
                e2.good = false;
                card.opts.push(("以剑代赋".into(), e2));
            }
            238 => {
                // mr4 恶缘：行会追赔（与 188 灵税催帖/229 佃契抽成题材分域——商行契赔）
                card.title = "行会追赔".into();
                card.text = "商行会执事上门：你押运的一单出了损耗，按契由押运人分偿——契上按着你三年前的手印。".into();
                let g = (50.0 * ss) as i64;
                let mut e1 = Eff::new();
                e1.stones = -g;
                e1.good = false;
                e1.msg = vec![format!(
                    "照契偿清，执事拱手离去。三年前的一枚手印，今日兑成了银钱。灵石-{}",
                    fmt_int(g)
                )];
                card.opts.push(("照契如偿".into(), e1));
                let mut e2 = Eff::new();
                e2.stones_pct = -0.04;
                e2.dao = 1.0;
                e2.msg = vec![
                    "你搬出契中另一条争了半日，偿金折去三成——人也累了。灵石-4%，道心+1".into(),
                ];
                card.opts.push(("据理缓偿".into(), e2));
            }
            239 => {
                // 老年×有伴双门控第 3 条：共守岁灯（照 204/212 模板；不入 COMPANION
                // 白名单——212 死模式注释，白名单成员先命中恒出「伴随」徽标）
                card.title = "共守岁灯".into();
                card.text = "岁暮大雪，檐下你与道侣共守一盏岁灯。灯芯爆了个灯花，两个人不约而同笑出了声。".into();
                let mut e1 = Eff::new();
                e1.dao = 2.0;
                e1.luck = 1.0;
                e1.msg = vec![
                    "灯花剔落，映亮两双眼睛。旧岁在灯影里过完，新年自灯芯上开始。道心+2，气运+1".into(),
                ];
                card.opts.push(("同剔灯芯".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 1.0;
                e2.msg = vec![
                    "你把旧年的事一件件说与灯听——说到后来，灯没睡，你们也没睡。道心+1".into(),
                ];
                card.opts.push(("把旧事说与灯听".into(), e2));
            }

            _ => {}
        }

        // v1.16.7 剧情演出·稀有/链段标记（纯会话态随卡生灭，不进存档）：
        // 稀有=EVENTS 权重 w∈1..=RARE_W（w=0 是链内段「仅链内出现」的哨兵，不算）；
        // 链段进度 (当前段,总段)——链1 两段式（44→45|46），链2/链3/链4 三段式，
        // 链5 两段分支式（79→80|81）
        let w = EVENTS
            .iter()
            .find(|&&(eid, _, _, _)| eid == id)
            .map(|&(_, wt, _, _)| wt)
            .unwrap_or(0);
        card.rare = w > 0 && w <= crate::data::RARE_W;
        card.stage = match id {
            44 => Some((1, 2)),
            45 | 46 => Some((2, 2)),
            47 => Some((1, 3)),
            48 => Some((2, 3)),
            49 => Some((3, 3)),
            50 => Some((1, 3)),
            51 => Some((2, 3)),
            52 => Some((3, 3)),
            76 => Some((1, 3)),
            77 => Some((2, 3)),
            78 => Some((3, 3)),
            79 => Some((1, 2)),
            80 | 81 => Some((2, 2)),
            85 => Some((1, 3)),
            86 => Some((2, 3)),
            87 => Some((3, 3)),
            112 => Some((1, 3)),
            113 => Some((2, 3)),
            114 => Some((3, 3)),
            115 => Some((1, 3)),
            116 => Some((2, 3)),
            117 => Some((3, 3)),
            // v1.16.11 链9/链10 三段式
            118 => Some((1, 3)),
            119 => Some((2, 3)),
            120 => Some((3, 3)),
            121 => Some((1, 3)),
            122 => Some((2, 3)),
            123 => Some((3, 3)),
            // v1.16.13 链11/链12 三段式
            137 => Some((1, 3)),
            138 => Some((2, 3)),
            139 => Some((3, 3)),
            140 => Some((1, 3)),
            141 => Some((2, 3)),
            142 => Some((3, 3)),
            // v1.16.14 链13 三段式
            151 => Some((1, 3)),
            152 => Some((2, 3)),
            153 => Some((3, 3)),
            // v1.16.16 链14 三段式
            175 => Some((1, 3)),
            176 => Some((2, 3)),
            177 => Some((3, 3)),
            // v1.16.19 链15 三段式
            205 => Some((1, 3)),
            206 => Some((2, 3)),
            207 => Some((3, 3)),
            // v1.16.20 链16 三段式
            219 => Some((1, 3)),
            220 => Some((2, 3)),
            221 => Some((3, 3)),
            // v1.16.21 链17 三段式（星槎再访）
            231 => Some((1, 3)),
            232 => Some((2, 3)),
            233 => Some((3, 3)),
            _ => None,
        };

        // v1.16.21 类目徽标：复用图鉴 event_badge 九类单口（与图鉴同源零新表；
        // 越界 id 落「遭遇」防御由单口自担）
        card.cat = Some(crate::gallery::event_badge(id));

        // v1.16.21 场景要素行：构建帧采样所在地·天时（会话态随卡生灭不进存档；
        // 事件帧 format! 豁免——与 title/text 同路径）。下标 min 防御照纪律
        if let Some(s) = self.sim.as_ref() {
            card.scene = Some(format!(
                "{} · {}",
                crate::data::LOCATIONS[s.loc.min(crate::data::LOCATIONS.len() - 1)].name,
                crate::data::WEATHERS[(s.weather as usize).min(crate::data::WEATHERS.len() - 1)]
            ));
        }

        if !card.opts.is_empty() {
            // v1.16.10 事件回顾图鉴收录：轮盘/链内直取两路共经 build_event，opts
            // 齐备即记一次遭遇（与卡是否被抉择无关，照稀有记事「构建帧即记」口径）
            self.note_event(id);

            // v1.16.11 手札回味：同点捕获卡面正文（构建帧即存，纯会话展示缓存；
            // 标题不存——event_names_match_card_titles 钉死 card.title==EVENT_NAMES[id]
            // 静态双源，回看时按 id 查表零新增分配）
            self.ev_memo_set(id, &card.text);

            // v1.16.7 稀有事件遭遇记事：入本世大事记（旧稿 story 数据源；卡构建
            // 帧即记「遇」，与抉择无关）
            if card.rare {
                if let Some(s) = self.sim.as_mut() {
                    s.highlights
                        .push(format!("{}岁，遇【{}】", s.age as i32, card.title));
                }
            }

            // v1.16.5 事件卡字形预热（照 show_banner 事件帧预热先例）：卡面标题 24/
            // 正文 16 在卡构建帧一次栅格化，弹卡帧只命中缓存——新文案不在
            // PANEL_WARM_CHARS 时避免首帧栅格化触发 2048² 图集整图重传
            crate::ui::prewarm_text(&card.title, &[24]);
            crate::ui::prewarm_text(&card.text, &[16]);

            // v1.16.7 复审④：链段阶段徽标串 @16 预热（取值域固定 5 种，绘制期
            // stage_label 产 &'static str 零分配）
            if card.stage.is_some() {
                for lab in ["1/2", "2/2", "1/3", "2/3", "3/3"] {
                    crate::ui::prewarm_text(lab, &[16]);
                }
            }

            // v1.16.21 类目徽标词 @16（词域固定九类，与 chip 绘制档对齐）/ 场景行
            // @14（地名·天时组合，皆既有语料字）预热——构建帧一次灌，弹卡帧零
            // 冷栅格化
            if let Some((bw, _)) = card.cat {
                crate::ui::prewarm_text(bw, &[16]);
            }
            if let Some(sc) = &card.scene {
                crate::ui::prewarm_text(sc, &[14]);
            }

            // 剧情演出包①：开启帧置卡面入场动效（0.18→0 会话态，主循环递减）
            self.modal_anim = 0.18;
            self.modal = Some(card);
        }
    }

    pub fn choose_option(&mut self, i: usize) {
        let Some(card) = self.modal.take() else {
            return;
        };

        let Some((_, eff)) = card.opts.into_iter().nth(i) else {
            return;
        };

        // v1.16.7 链完成大事记：末段卡（stage 达总段）抉择即缘了——链 id 须在
        // apply_eff 清链前捕获（直取消费后 chain=(链id,0,ttl) 仍活跃）。中段断链
        // （非推链选项）不算完成、不入大事记
        let chain_done = card.stage.map_or(false, |(seg, total)| seg >= total);
        let chain_id = self.chain.map(|(c, _, _)| c);

        self.apply_eff(&eff);

        if chain_done {
            // v1.16.16 链缘了闩：bit(链id-1) 置位——本世已缘了的链首段不再入池
            //（CHAIN_FIRSTS 查表反门控，见 sim_event_weights；bit 位域 0..=15——
            // v1.16.19 链15=bit14、v1.16.20 链16=bit15 已占用，注释数字曾漂在
            // 0..=13 清账对齐；u32 上限充裕，链17 起再议扩型；链 id 脏值
            // saturating 防御不越位）
            if let Some(cid) = chain_id {
                let bit = (cid as u32).saturating_sub(1).min(31);
                self.chain_latch |= 1u32 << bit;
            }
            if let (Some(cid), Some(s)) = (chain_id, self.sim.as_mut()) {
                let name = CHAIN_NAMES
                    [(cid as usize).saturating_sub(1).min(CHAIN_NAMES.len() - 1)];
                s.highlights
                    .push(format!("{}岁，{}·缘了", s.age as i32, name));
                // v1.16.10 链6 缘了置会话标志：宿敌同世回响事件 110 的入池门控读此位
                // （「夺积之后倾巢寻账」只属同一推演；launch_sim 清零跨世不泄漏）
                if cid == 6 {
                    self.chain6_closed = true;
                }
            }
        }

        self.sync_save();
    }

    fn apply_eff(&mut self, e: &Eff) {
        match e.special {
            13 => {
                // 立即问笔（玩家抉择）
                let (target, acc) = {
                    let s = self.sim.as_ref().unwrap();
                    (10, s.pending_break.as_ref().map(|(_, a)| *a).unwrap_or(0.0))
                };
                self.sim_do_break(target, acc);
            }
            14 => {
                // 闭关准备三载后自动问笔
                let s = self.sim.as_mut().unwrap();
                s.pending_break = Some((10, 0.12));
                self.slog("你转入闭关，默运玄功静候天劫。", C_DIM);
            }
            15 => {
                // 立即破境（玩家抉择）
                let (target, acc) = {
                    let s = self.sim.as_ref().unwrap();
                    let t = s
                        .pending_break
                        .as_ref()
                        .map(|(t, _)| *t)
                        .unwrap_or(s.realm + 1);
                    let a = s.pending_break.as_ref().map(|(_, a)| *a).unwrap_or(0.0);
                    (t, a)
                };
                self.sim_do_break(target, acc);
            }
            16 => {
                // 蓄势
                let s = self.sim.as_mut().unwrap();
                let target = s.realm + 1;
                s.pending_break = Some((target, 0.0));
                self.check_ach(24);
            }
            17 => {
                // 结为道侣。v1.12.1 同页人·三生缝页：掷名 40% 缘定缝页人——未定名时
                // 定名初缝，已定名时强制重逢；初缝/离页重逢各补一行史笔
                let roll = gen_range(0, COMPANION_NAMES.len());
                let (idx, feng_new) = if self.meta.feng_bn == crate::data::FENG_UNSET {
                    if gen_range(0.0, 1.0) < 0.4 {
                        self.meta.feng_bn = roll as u8; // 初缝定名
                        (roll, true)
                    } else {
                        (roll, false)
                    }
                } else if gen_range(0.0, 1.0) < 0.4 {
                    (self.meta.feng_bn as usize, false) // 强制重逢
                } else {
                    (roll, false)
                };
                let cn = COMPANION_NAMES[idx];
                let prev = self.sim.as_ref().unwrap().companion.clone();
                let feng_re = !feng_new && self.feng_reunion(prev.as_deref(), cn);

                let first = {
                    let s = self.sim.as_mut().unwrap();

                    let first = s.companion.is_none();

                    s.companion = Some(cn.to_string());

                    // v1.6.0 D 应修②：好感只升不降——sim Life 由现实克隆（launch_sim）带入
                    // comp_fav，直接赋 25 会让遗泽等来源的高好感倒退（如 1.30 档→1.20）。
                    // 换伴侣保留好感：好感挂「玩家—NPC 体系」而非具体人名。
                    s.comp_fav = s.comp_fav.max(COMPANION_BOND_FAV);

                    s.highlights
                        .push(format!("{}岁，与{}缝作同页", s.age as i32, cn));
                    if feng_new {
                        s.highlights.push(format!(
                            "{}岁，与{}初缝三生之页——三生之约，缘起此生",
                            s.age as i32, cn
                        ));
                    } else if feng_re {
                        s.highlights.push(format!(
                            "{}岁，与{}同页重逢——撕开的页，又缝上了",
                            s.age as i32, cn
                        ));
                    }

                    first
                };
                if first {
                    self.meta.companion_lives += 1;
                    self.check_ach(22);
                    if self.meta.companion_lives >= 3 {
                        self.check_ach(26);
                    }
                    // v1.16.8 章节书：首结缘事实翻转点（第六章目标边沿）
                    self.check_chapters();
                }
                // v1.16.0 交游图鉴：结缘即录入（唯一写入口 gallery_bond；无论是否 first——
                // 重逢/换道侣亦记历世同页一次；quit_sim_redraft 重掷不回退本账）
                self.gallery_bond(idx);
                self.slog(
                    format!(
                        "你与{}缝作同页！修炼速度+{:.0}%，心魔有抚。",
                        cn,
                        (crate::npc::comp_cult_mult(COMPANION_BOND_FAV) - 1.0) * 100.0
                    ),
                    C_GOLD,
                );
                // v1.16.18 搭车⑤：结缘瞬间 toast（sim 侧 toast 槽 draw.rs:1741 已渲染）。
                // 与上行 slog 同帧双通道是文案分工：slog=信箱记账留痕，toast=当场提示。
                // 单槽后写胜：会顶掉同帧 check_ach(22) 成就 toast（成就 log 仍留），
                // banner 槽让给 check_chapters 章节横幅（本臂只动 toast 不动 banner）
                self.toast(format!("与{}缝作同页", cn), C_PURPLE);
            }
            18 => {
                // 宗门大比夺魁
                let won = self.sim_fight(1.1, "同门劲敌");
                if won {
                    let realm = self.sim.as_ref().unwrap().realm;
                    let g = (400.0 * self.stone_scale(realm)) as i64;
                    let (old_tech, got_tech) = {
                        let s = self.sim.as_mut().unwrap();
                        s.stones += g;
                        s.power_buff += 0.05;
                        let ot = s.tech;
                        if ot + 1 < TECHS.len() {
                            s.tech += 1;
                            (ot, true)
                        } else {
                            (ot, false)
                        }
                    };
                    if got_tech {
                        let tn = TECHS[old_tech + 1].name;
                        self.slog(
                            format!(
                                "大比夺魁！掌门亲授《{}》，另赏灵石 {}，战力永久+5%！",
                                tn,
                                fmt_int(g)
                            ),
                            C_GOLD,
                        );
                    } else {
                        self.slog(
                            format!("大比夺魁！赏灵石 {}，战力永久+5%！", fmt_int(g)),
                            C_GOLD,
                        );
                    }
                    if let Some(s) = self.sim.as_mut() {
                        s.highlights
                            .push(format!("{}岁，宗门大比夺魁", s.age as i32));
                    }
                    self.check_ach(23);
                }
            }

            19 => {
                // v1.12.0 空白行·留行：在大事记亲手写下一行。不 return——落入通用乘区
                // （气运+2 照常生效），照既有 special 臂只做旁路效果的口径
                let s = self.sim.as_mut().unwrap();

                s.highlights
                    .push(format!("{}岁，空白行上，你亲手写下一行", s.age as i32));
            }

            // ---------- v1.16.7 连锁剧情：置链臂（20..=31）与新检定臂 ----------
            // 一律不 return（照 special19 口径）：置链臂重写链态，其余臂落通用段
            // 尾的链裁决清链。检定臂（4/6/26/27/32/33/34）效果全在臂内结算，
            // 通用字段全空，落通用段无副作用。v1.16.8 续编：28/29=链4（灯楼谜约
            // 三段式 76→77→78）、30/31=链5（傩面还愿双分支 79→80|81）
            20 => {
                // 链1 货郎·分支a：赊墨立契 → 恩怨上门
                self.chain = Some((1, 46, CHAIN_TTL));
            }
            21 => {
                // 链1 货郎·分支b：婉拒不买 → 罗盘相赠
                self.chain = Some((1, 45, CHAIN_TTL));
            }
            22 => {
                // 链2 古碑：以血引墨（拓下前半卷）→ 拓印风波
                self.chain = Some((2, 48, CHAIN_TTL));
            }
            23 => {
                // 链2 拓印风波：随裱工见人 → 守碑人
                self.chain = Some((2, 49, CHAIN_TTL));
            }
            24 => {
                // 链3 墨匣：收匣启程 → 渔村释疑
                self.chain = Some((3, 51, CHAIN_TTL));
            }
            25 => {
                // 链3 渔村释疑：随老渔翁往江心 → 匣底真迹
                self.chain = Some((3, 52, CHAIN_TTL));
            }
            28 => {
                // 链4 灯楼谜约：揭谜应约 → 灯下问答
                self.chain = Some((4, 77, CHAIN_TTL));
            }
            29 => {
                // 链4 灯下问答：应主人之问 → 青灯问藏
                self.chain = Some((4, 78, CHAIN_TTL));
            }
            30 => {
                // 链5 傩面还愿·分支a：戴面代跳 → 戏毕面热
                self.chain = Some((5, 80, CHAIN_TTL));
            }
            31 => {
                // 链5 傩面还愿·分支b：出钱贴班 → 愿成符赠
                self.chain = Some((5, 81, CHAIN_TTL));
            }
            35 => {
                // 链6 血债血偿：循踪暗记 → 仇踪暗访（第二段窗口，见 CHAIN6_SPECIAL_LO）
                self.chain = Some((6, 86, CHAIN_TTL));
            }
            36 => {
                // 链6 血债血偿：布饵设伏 → 乘虚夺积
                self.chain = Some((6, 87, CHAIN_TTL));
            }
            // v1.16.10 链7（灯花渡舟）/链8（药炉童子）置链臂：第三窗（见
            // CHAIN78_SPECIAL_LO/HI），三窗之外落链即终局的裁决语义不变
            37 => {
                // 链7 灯花渡舟：以旧事为资 → 灯下旧账
                self.chain = Some((7, 113, CHAIN_TTL));
            }
            38 => {
                // 链7 灯下旧账：追问灯纸 → 一渡一灯
                self.chain = Some((7, 114, CHAIN_TTL));
            }
            39 => {
                // 链8 药炉童子：留下守炉 → 炉语问心
                self.chain = Some((8, 116, CHAIN_TTL));
            }
            40 => {
                // 链8 炉语问心：问炉要什么 → 守炉一夜
                self.chain = Some((8, 117, CHAIN_TTL));
            }
            // v1.16.11 链9（循册寻仇）/链10（旧页回声）置链臂：第四窗（见
            // CHAIN910_SPECIAL_LO/HI），四窗之外落链即终局的裁决语义不变
            41 => {
                // 链9 血册投书：应下战书 → 雪夜拦道
                self.chain = Some((9, 119, CHAIN_TTL));
            }
            42 => {
                // 链9 雪夜拦道：逐一应名 → 断名之局
                self.chain = Some((9, 120, CHAIN_TTL));
            }
            43 => {
                // 链10 旧页回声：循墨寻摊 → 灯市换页
                self.chain = Some((10, 122, CHAIN_TTL));
            }
            44 => {
                // 链10 灯市换页：以页换页 → 换页重逢
                self.chain = Some((10, 123, CHAIN_TTL));
            }
            // v1.16.13 链11（劫痕渡客）/链12（书肆换届）置链臂：第五窗（见
            // CHAIN1112_SPECIAL_LO/HI），五窗之外落链即终局的裁决语义不变
            45 => {
                // 链11 劫云过境：留棚观劫 → 劫灰拾遗
                self.chain = Some((11, 138, CHAIN_TTL));
            }
            46 => {
                // 链11 劫灰拾遗：探访劫者 → 劫后茶话
                self.chain = Some((11, 139, CHAIN_TTL));
            }
            47 => {
                // 链12 书肆易主：帮搬装箱 → 代笔提要
                self.chain = Some((12, 141, CHAIN_TTL));
            }
            48 => {
                // 链12 代笔提要：应下提要 → 肆后书案
                self.chain = Some((12, 142, CHAIN_TTL));
            }
            // v1.16.14 链13（死当归主）置链臂：第六窗（见 CHAIN13_SPECIAL_LO/HI），
            // 六窗之外落链即终局的裁决语义不变
            49 => {
                // 链13 当铺死当：问下死当 → 访主赎当
                self.chain = Some((13, 152, CHAIN_TTL));
            }
            50 => {
                // 链13 访主赎当：代为赎当 → 物归旧主
                self.chain = Some((13, 153, CHAIN_TTL));
            }
            // v1.16.16 链14（修伞记）置链臂：第七窗（见 CHAIN14_SPECIAL_LO/HI），
            // 七窗之外落链即终局的裁决语义不变
            53 => {
                // 链14 修伞记：请老匠修伞 → 伞骨重削
                self.chain = Some((14, 176, CHAIN_TTL));
            }
            54 => {
                // 链14 伞骨重削：应他重削 → 还伞
                self.chain = Some((14, 177, CHAIN_TTL));
            }
            // v1.16.19 链15（观星问岁）置链臂：第八窗（见 CHAIN15_SPECIAL_LO/HI），
            // 八窗之外落链即终局的裁决语义不变
            56 => {
                // 链15 观星问岁：替他数星 → 指间星图
                self.chain = Some((15, 206, CHAIN_TTL));
            }
            57 => {
                // 链15 指间星图：替他补笔 → 问岁之约
                self.chain = Some((15, 207, CHAIN_TTL));
            }
            // v1.16.20 链16（剑穗之约）置链臂：第九窗（见 CHAIN16_SPECIAL_LO/HI），
            // 九窗之外落链即终局的裁决语义不变。窗内 62 无置链臂（apply_eff 落
            // _ 臂空转）——选「一夜结完」链态保持 (16,0,ttl)，随下次 sim_event
            // next==0 弃链，与中段无 special 选项的断链语义等价且不置缘了闩
            60 => {
                // 链16 剑穗生结：应下这个结 → 剑穗缓结
                self.chain = Some((16, 220, CHAIN_TTL));
            }
            61 => {
                // 链16 剑穗缓结：依她一日一寸 → 剑成赠穗
                self.chain = Some((16, 221, CHAIN_TTL));
            }
            // v1.16.21 宿敌遭遇卡化（sim_neme → EventCard 管线）：两臂在九/十窗
            // 之外——链活跃时选宿敌卡选项即「落链上即终局」既定语义，裁决零特判。
            // 67=迎战（统一决战式 p=r²/(1+r²) clamp(0.05,0.95)，与 3486 原式同源
            // 勿联动改；悬殊时 p 自落 5..20%，即原打不过臂 40% 无决策死的明码化）；
            // 68=拼死遁走（确定性脱身，重伤+行囊失二成，无死——消灭无决策死亡）
            67 => {
                let (my, np) = {
                    let s = self.sim.as_ref().unwrap();

                    let np = self
                        .meta
                        .neme
                        .as_ref()
                        .map(|n| n.power())
                        .unwrap_or(f64::INFINITY);

                    (power(s), np)
                };

                let r = my / np.max(1.0);

                let p = (r * r / (1.0 + r * r)).clamp(0.05, 0.95);

                if gen_range(0.0, 1.0) < p {
                    // 反杀宿敌！（原 sim_neme 决战胜路径逐行平移）
                    let neme = self.meta.neme.take().unwrap_or(crate::game::Nemes {
                        title: "魔修",
                        name: String::new(),
                        realm: 0,
                    });

                    let reward_stones = (3000.0 * self.stone_scale(neme.realm)) as i64;

                    let reward_dy = 40 * (1 + neme.realm as i64);

                    {
                        let s = self.sim.as_mut().unwrap();

                        s.stones += reward_stones;

                        s.highlights
                            .push(format!("{}岁，阵斩宿敌{}", s.age as i32, neme.name));
                    }

                    self.meta.neme_slain += 1;

                    self.meta.dao_yun += reward_dy;

                    self.meta.sim_points += 2;

                    self.log(
                        format!(
                            "【模拟中反杀】你循战意逆斩宿敌{}！现实中的你已悟得其招，宿敌授首。",
                            neme.full()
                        ),
                        C_GOLD,
                    );

                    self.show_banner("宿敌授首！", C_GOLD);

                    self.log(
                        format!(
                            "得灵石 {}（现实），道韵+{}，模拟点数+2。",
                            fmt_int(reward_stones),
                            reward_dy
                        ),
                        C_GOLD,
                    );

                    self.spawn_next_neme();

                    self.play(crate::sounds::Which::Ascend);

                    self.burst(60, true);
                } else {
                    let nfull = self
                        .meta
                        .neme
                        .as_ref()
                        .map(|n| n.full())
                        .unwrap_or_else(|| "那魔修".to_string());

                    self.sim_die(&format!(
                        "你于乱战中再遇宿敌{}！激战三十合，终究技逊一筹，被一剑枭首。",
                        nfull
                    ));
                }
            }
            68 => {
                // 拼死遁走：确定性脱身（原打不过臂 60% 逃语义固化）；能战而走者同价
                let nfull = self
                    .meta
                    .neme
                    .as_ref()
                    .map(|n| n.full())
                    .unwrap_or_else(|| "那魔修".to_string());

                {
                    let s = self.sim.as_mut().unwrap();

                    s.injured = 8;

                    s.stones = (s.stones as f64 * 0.8) as i64;
                }

                self.slog(
                    format!(
                        "你于荒野撞见{}！自知不敌（或不愿此时了断），拼死遁走——身受重伤，行囊失了二成。",
                        nfull
                    ),
                    C_RED,
                );
            }
            // v1.16.21 链17（星槎再访）置链臂：第十窗（见 CHAIN17_SPECIAL_LO/HI），
            // 十窗之外落链即终局的裁决语义不变。窗内 65 备用无臂（落 _ 臂空转）
            63 => {
                // 链17 星槎再访：以路相赠 → 星轨留书·续
                self.chain = Some((17, 232, CHAIN_TTL));
            }
            64 => {
                // 链17 星轨留书·续：替他补全星轨 → 槎去留星·续
                self.chain = Some((17, 233, CHAIN_TTL));
            }
            66 => {
                // 旧砚问道（稀有 234）：道心检定——砚声叩问，过则砚池自出一行小字
                //（道心+3、修为+8%），败则砚声骤冷浸指（轻伤、道心-2）。检定结构照
                // special33 墨叟一问同款（dao 越高越稳）
                let dao = self.sim.as_ref().unwrap().dao;

                let p = (0.35 + dao / 160.0).min(0.9);

                if gen_range(0.0, 1.0) < p {
                    {
                        let s = self.sim.as_mut().unwrap();

                        s.dao = (s.dao + 3.0).min(100.0);

                        gain_qi(s, qi_need(s) * 0.08);
                    }

                    self.slog(
                        "你把心里那句话如实答了。砚池无墨自出一行小字，映你眉目——摊主深深一揖。道心+3，修为+8%",
                        C_GOLD,
                    );
                } else {
                    {
                        let s = self.sim.as_mut().unwrap();

                        s.injured = 3;

                        s.dao = (s.dao - 2.0).max(0.0);
                    }

                    self.slog(
                        "你答得含糊——砚声骤然一冷，寒意浸指入骨。摊主摇头把砚收起。（轻伤，道心-2）",
                        C_RED,
                    );
                }
            }
            32 => {
                // 青灯三问（链4 末段）：道心检定——过授功法一阶（满阶折灯油银），
                // 败负内创。检定结构照 special4 守碑人同款（dao 越高越稳）
                let dao = self.sim.as_ref().unwrap().dao;

                let p = (0.30 + dao / 150.0).min(0.9);

                if gen_range(0.0, 1.0) < p {
                    let realm = self.sim.as_ref().unwrap().realm;

                    let g = (320.0 * self.stone_scale(realm)) as i64;

                    let (old_tech, got) = {
                        let s = self.sim.as_mut().unwrap();

                        let ot = s.tech;

                        if ot + 1 < TECHS.len() {
                            s.tech += 1;

                            (ot, true)
                        } else {
                            s.stones += g;

                            (ot, false)
                        }
                    };

                    if got {
                        let tn = TECHS[old_tech + 1].name;

                        self.slog(
                            format!(
                                "三问皆答，素布之下正是你未写完的那一页——《{}》入你行囊！道途再进一阶。",
                                tn
                            ),
                            C_GOLD,
                        );
                    } else {
                        self.slog(
                            format!(
                                "三问皆答，青灯化萤而去——案上留银 {}，是主人三十年的灯油钱。",
                                fmt_int(g)
                            ),
                            C_GOLD,
                        );
                    }
                } else {
                    let s = self.sim.as_mut().unwrap();

                    s.injured = 4;

                    self.slog(
                        "第三问问住了你——青灯骤暗，纸页翻卷如刀。你受了一记内创。（重伤）",
                        C_RED,
                    );
                }
            }
            33 => {
                // 墨叟一问（稀有·墨渊垂钓）：道心检定——过钓得古字换灵石与气运，
                // 败渊水浸骨轻伤。结构与 special4 同款
                let dao = self.sim.as_ref().unwrap().dao;

                let p = (0.35 + dao / 160.0).min(0.9);

                if gen_range(0.0, 1.0) < p {
                    let realm = self.sim.as_ref().unwrap().realm;

                    let g = (400.0 * self.stone_scale(realm)) as i64;

                    {
                        let s = self.sim.as_mut().unwrap();

                        s.stones += g;

                        s.luck = (s.luck + 2.0).min(100.0);
                    }

                    self.slog(
                        format!(
                            "你认出那字是「缘」。墨叟大笑，古字化银 {} 入袖——渊上从此多一个传说。灵石+{}，气运+2",
                            fmt_int(g),
                            fmt_int(g)
                        ),
                        C_GOLD,
                    );
                } else {
                    let s = self.sim.as_mut().unwrap();

                    s.injured = 3;

                    self.slog(
                        "你认不得那个字——墨渊水汽骤冷，浸骨入髓。（轻伤）",
                        C_RED,
                    );
                }
            }
            34 => {
                // 拦问纸人（稀有·纸人夜行）：六四开——问出棺中秘密得遗稿金句，
                // 或被纸棺「记名」折损。检定结构照 special27 灵潮漩心同款
                let realm = self.sim.as_ref().unwrap().realm;

                let ss2 = self.stone_scale(realm);

                if gen_range(0.0, 1.0) < 0.55 {
                    let g = (500.0 * ss2) as i64;

                    {
                        let s = self.sim.as_mut().unwrap();

                        gain_qi(s, qi_need(s) * 0.12);

                        s.stones += g;
                    }

                    self.slog(
                        format!(
                            "纸人停步，小棺开了一条缝——里面是前人未写完的半句遗稿。你读罢，气机自通：修为+12%，灵石+{}。",
                            fmt_int(g)
                        ),
                        C_GOLD,
                    );
                } else {
                    let (msg, c) = {
                        let s = self.sim.as_mut().unwrap();

                        s.injured = 6;

                        s.stones = (s.stones as f64 * 0.85) as i64;

                        (
                            "纸人齐齐回头——你被记了名。当夜高热不退，行囊也丢了些。（重伤，灵石-15%）",
                            C_RED,
                        )
                    };

                    self.slog(msg, c);
                }
            }
            4 => {
                // 守碑人三问（链2 末段）：道心检定——过授功法一阶（满阶折灵石），
                // 败负内创。检定结构与 special7 心魔同款（dao 越高越稳）
                let dao = self.sim.as_ref().unwrap().dao;

                let p = (0.30 + dao / 150.0).min(0.9);

                if gen_range(0.0, 1.0) < p {
                    let realm = self.sim.as_ref().unwrap().realm;

                    let g = (300.0 * self.stone_scale(realm)) as i64;

                    let (old_tech, got) = {
                        let s = self.sim.as_mut().unwrap();

                        let ot = s.tech;

                        if ot + 1 < TECHS.len() {
                            s.tech += 1;

                            (ot, true)
                        } else {
                            s.stones += g;

                            (ot, false)
                        }
                    };

                    if got {
                        let tn = TECHS[old_tech + 1].name;

                        // v1.16.20 链尾余味（链2 守碑人过臂）：照 special32「三十年的
                        // 灯油钱」余味体补一档——授《名》后守碑人还归其碑，画面收束
                        self.slog(
                            format!("三问皆对，雾散了一线——守碑人授《{}》，转身还去守他的碑。", tn),
                            C_GOLD,
                        );
                    } else {
                        self.slog(
                            format!("三问皆对，守碑人抚掌而笑——赠灵石 {} 作谢。", fmt_int(g)),
                            C_GOLD,
                        );
                    }
                } else {
                    let s = self.sim.as_mut().unwrap();

                    s.injured = 4;

                    self.slog("三问皆错，碑文反噬——你受了一记内创。（重伤）", C_RED);
                }
            }
            5 => {
                // 同舟剑（伴随·赠剑）：战力永久+4%
                let s = self.sim.as_mut().unwrap();

                s.power_buff += 0.04;
            }
            6 => {
                // 星坠试炼（稀有·星坠荒原）：五五开——陨铁认主得法宝+大量修为，
                // 或铁羽兽反噬重伤破财（照 special10 魔窟探宝的赌点结构）
                let realm = self.sim.as_ref().unwrap().realm;

                let ss = self.stone_scale(realm);

                if gen_range(0.0, 1.0) < 0.5 {
                    let (g, relic) = {
                        let s = self.sim.as_mut().unwrap();

                        gain_qi(s, qi_need(s) * 0.30);

                        let mut relic = None;

                        if gen_range(0.0, 1.0) < 0.6 {
                            let rl = gen_range(0, RELICS.len());

                            if !s.relics.contains(&rl) {
                                s.relics.push(rl);

                                s.equipped.push(rl); // v1.5.0：模拟获得即装备（守则见文件头）

                                relic = Some(rl);
                            }
                        }

                        ((160.0 * ss) as i64, relic)
                    };

                    let mut m = format!("陨铁认主，星纹入体！修为大进，灵石+{}", fmt_int(g));

                    if let Some(rl) = relic {
                        m = format!("{}，得法宝【{}】", m, RELICS[rl].name);
                    }

                    self.slog(m, C_GOLD);
                } else {
                    let s = self.sim.as_mut().unwrap();

                    s.injured = 8;

                    s.stones = (s.stones as f64 * 0.7) as i64;

                    self.slog("铁羽兽自坑中暴起，你重伤狂遁！（重伤，灵石-30%）", C_RED);
                }
            }
            12 => {
                // 墨痴真迹（链3 末段）：参悟入大事记（修为/道心走通用字段）
                let s = self.sim.as_mut().unwrap();

                s.highlights
                    .push(format!("{}岁，江心匣底参悟墨痴真迹", s.age as i32));
            }
            26 => {
                // 镜中赌命（稀有·无面镜客）：五五开——赢半生修为+气运，输十年寿数
                if gen_range(0.0, 1.0) < 0.5 {
                    let (msg, c) = {
                        let s = self.sim.as_mut().unwrap();

                        gain_qi(s, qi_need(s) * 0.40);

                        s.luck = (s.luck + 6.0).clamp(0.0, 100.0);

                        ("镜中人一笑而散——半生修为涌入你的识海！气运+6", C_GOLD)
                    };

                    self.slog(msg, c);
                } else {
                    let (msg, c) = {
                        let s = self.sim.as_mut().unwrap();

                        s.lifespan -= 10.0; // v1.16.9 对账：与卡面「输，留十年寿数」（id57 文案）数值统一

                        s.injured = 6;

                        ("镜中人吞了你的倒影——十年寿数入镜，你受创而退！", C_RED)
                    };

                    self.slog(msg, c);
                }
            }
            27 => {
                // 灵潮漩心（稀有·灵潮倒卷）：六四开——前人坐化洞府得传承，或反噬
                let realm = self.sim.as_ref().unwrap().realm;

                let ss = self.stone_scale(realm);

                if gen_range(0.0, 1.0) < 0.6 {
                    let (old_tech, got, g) = {
                        let s = self.sim.as_mut().unwrap();

                        gain_qi(s, qi_need(s) * 0.25);

                        let ot = s.tech;

                        let mut got = false;

                        if ot + 1 < TECHS.len() {
                            s.tech += 1;

                            got = true;
                        }

                        let g = (260.0 * ss) as i64;

                        s.stones += g;

                        (ot, got, g)
                    };

                    if got {
                        let tn = TECHS[old_tech + 1].name;

                        self.slog(
                            format!(
                                "漩心洞府竟是前人坐化之地——《{}》与灵石 {} 尽归于你！",
                                tn,
                                fmt_int(g)
                            ),
                            C_GOLD,
                        );
                    } else {
                        self.slog(
                            format!("漩心洞府灵气尽枯，你扫得遗财灵石 {}。", fmt_int(g)),
                            C_GOLD,
                        );
                    }
                } else {
                    let (msg, c) = {
                        let s = self.sim.as_mut().unwrap();

                        s.injured = 9;

                        s.stones = (s.stones as f64 * 0.75) as i64;

                        ("灵潮倒卷如刃，你被抛出漩涡——重伤濒死！（重伤，灵石-25%）", C_RED)
                    };

                    self.slog(msg, c);
                }
            }

            1 => {
                let won = self.sim_fight(e.fight, "洞府守护傀儡");

                if won {
                    enum L {
                        Tech(usize),

                        Relic(usize),

                        Stones(i64),
                    }

                    let ss = self.stone_scale(self.sim.as_ref().unwrap().realm);

                    let r = {
                        let s = self.sim.as_mut().unwrap();

                        let roll = gen_range(0.0, 1.0);

                        if roll < 0.45 && s.tech + 1 < TECHS.len() {
                            s.tech += 1;

                            L::Tech(s.tech)
                        } else if roll < 0.75 {
                            let rl = gen_range(0, RELICS.len());

                            if !s.relics.contains(&rl) {
                                s.relics.push(rl);

                                s.equipped.push(rl); // v1.5.0：模拟获得即装备（守则见文件头）

                                L::Relic(rl)
                            } else {
                                let g = (200.0 * ss) as i64;

                                s.stones += g;

                                L::Stones(g)
                            }
                        } else {
                            let g = (250.0 * ss) as i64;

                            s.stones += g;

                            L::Stones(g)
                        }
                    };

                    match r {
                        L::Tech(t) => {
                            self.slog(format!("你于石壁上悟得《{}》！", TECHS[t].name), C_GOLD)
                        }

                        L::Relic(rl) => {
                            self.slog(format!("得法宝【{}】！", RELICS[rl].name), C_GOLD)
                        }

                        L::Stones(g) => self.slog(format!("得灵石 {}", fmt_int(g)), C_CYAN),
                    }
                }

                return;
            }

            2 => {
                let won = self.sim_fight(e.fight, "护宝妖兽");

                if won {
                    enum W {
                        Relic(usize),

                        Qi,
                    }

                    let ss = self.stone_scale(self.sim.as_ref().unwrap().realm);

                    let r = {
                        let s = self.sim.as_mut().unwrap();

                        let roll = gen_range(0.0, 1.0);

                        if roll < 0.5 {
                            let rl = gen_range(0, RELICS.len());

                            if !s.relics.contains(&rl) {
                                s.relics.push(rl);

                                s.equipped.push(rl); // v1.5.0：模拟获得即装备（守则见文件头）

                                W::Relic(rl)
                            } else {
                                let g = (300.0 * ss) as i64;

                                s.stones += g;

                                W::Qi
                            }
                        } else {
                            gain_qi(s, qi_need(s) * 0.30);

                            s.dao = (s.dao + 3.0).min(100.0);

                            W::Qi
                        }
                    };

                    match r {
                        W::Relic(rl) => {
                            self.slog(format!("灵乳入腹，得【{}】！", RELICS[rl].name), C_GOLD)
                        }

                        W::Qi => self.slog("千年灵乳入腹，修为大进！", C_GOLD),
                    }
                }

                return;
            }

            3 => {
                if gen_range(0.0, 1.0) < 0.6 {
                    let ss = self.stone_scale(self.sim.as_ref().unwrap().realm);

                    let (g, relic) = {
                        let s = self.sim.as_mut().unwrap();

                        let g = (500.0 * ss * gen_range(0.8, 1.6)) as i64;

                        s.stones += g;

                        gain_qi(s, qi_need(s) * 0.22);

                        let mut relic = None;

                        if gen_range(0.0, 1.0) < 0.3 {
                            let rl = gen_range(0, RELICS.len());

                            if !s.relics.contains(&rl) {
                                s.relics.push(rl);

                                s.equipped.push(rl); // v1.5.0：模拟获得即装备（守则见文件头）

                                relic = Some(rl);
                            }
                        }

                        (g, relic)
                    };

                    let mut msg = format!("秘境宝库洞开！灵石+{}", fmt_int(g));

                    if let Some(rl) = relic {
                        msg = format!("{}，得法宝【{}】", msg, RELICS[rl].name);
                    }

                    self.slog(msg, C_GOLD);
                } else {
                    let s = self.sim.as_mut().unwrap();

                    s.injured = 8;

                    s.stones = (s.stones as f64 * 0.9) as i64;

                    self.slog("秘境禁制暴起，你重伤逃出！", C_RED);
                }

                return;
            }

            8 => {
                let s = self.sim.as_mut().unwrap();

                s.power_buff += 0.12;

                self.slog("仙人指点了三招武道真意！战力永久+12%。", C_GOLD);
            }

            9 => {
                let won = self.sim_fight(1.4, "守冢剑灵");

                if won {
                    let s = self.sim.as_mut().unwrap();

                    if !s.relics.contains(&3) {
                        s.relics.push(3);

                        s.equipped.push(3); // v1.5.0：模拟获得即装备（守则见文件头）

                        s.highlights
                            .push(format!("{}岁，古剑冢得墨骨伞", s.age as i32));

                        self.slog("冢中遗物认主——【墨骨伞】撑开，遮尽十万剑气！", C_GOLD);
                    } else {
                        s.power_buff += 0.15;

                        self.slog("古剑已有所属，剑意却入了你的骨。战力永久+15%！", C_GOLD);
                    }
                }
            }

            10 => {
                if gen_range(0.0, 1.0) < 0.5 {
                    let ss = self.stone_scale(self.sim.as_ref().unwrap().realm);

                    let (g, relic) = {
                        let s = self.sim.as_mut().unwrap();

                        let g = (900.0 * ss * gen_range(0.9, 1.5)) as i64;

                        s.stones += g;

                        gain_qi(s, qi_need(s) * 0.15);

                        let mut relic = None;

                        if gen_range(0.0, 1.0) < 0.45 {
                            let rl = gen_range(0, RELICS.len());

                            if !s.relics.contains(&rl) {
                                s.relics.push(rl);

                                s.equipped.push(rl); // v1.5.0：模拟获得即装备（守则见文件头）

                                relic = Some(rl);
                            }
                        }

                        (g, relic)
                    };

                    let mut msg = format!("魔窟深处是前代魔君的库房！灵石+{}", fmt_int(g));

                    if let Some(rl) = relic {
                        msg = format!("{}，得魔功法宝【{}】", msg, RELICS[rl].name);
                    }

                    self.slog(msg, C_GOLD);
                } else {
                    let s = self.sim.as_mut().unwrap();

                    s.injured = 10;

                    s.stones = (s.stones as f64 * 0.7) as i64;

                    self.slog("魔窟里残魂反噬，你重伤狂逃！（重伤，灵石-30%）", C_RED);
                }
            }

            11 => {
                let won = self.sim_fight(1.0, "围商妖兽");

                if won {
                    let ss = self.stone_scale(self.sim.as_ref().unwrap().realm);

                    let g = (350.0 * ss * gen_range(1.0, 1.6)) as i64;

                    let s = self.sim.as_mut().unwrap();

                    s.stones += g;

                    s.luck = (s.luck + 2.0).min(100.0);

                    self.slog(
                        format!("商队安抵坊市！管事奉上酬金 {}，气运+2。", fmt_int(g)),
                        C_GREEN,
                    );
                }
            }

            12 => {
                let s = self.sim.as_mut().unwrap();

                s.dao = (s.dao + 6.0).min(100.0);

                s.pills[2] += 1;

                self.slog("卦师赠破境丹一枚：此卦大吉。", C_GREEN);
            }

            7 => {
                let (dao, luck, s_companion) = {
                    let s = self.sim.as_ref().unwrap();

                    (s.dao, s.luck, s.companion.is_some())
                };

                let mut p = 0.5 + dao / 150.0 + luck / 500.0;

                if s_companion {
                    p += 0.15;
                }

                let passed = gen_range(0.0, 1.0) < p;

                let (msg, c) = {
                    let s = self.sim.as_mut().unwrap();

                    if passed {
                        s.dao = (s.dao + 3.0).min(100.0);

                        s.qi += qi_need(s) * 0.08;

                        ("你心湖澄澈，魔念如雪遇阳而化。道心+3。", C_GREEN)
                    } else {
                        s.dao = (s.dao - 6.0).max(0.0);

                        s.injured = 5;

                        ("心魔难平，你受了内创。（道心-6）", C_RED)
                    }
                };

                self.slog(msg, c);

                return;
            }

            _ => {}
        }

        if e.fight > 0.0 {
            self.sim_fight(e.fight, "敌手");
        }

        let need = self.sim.as_ref().map(qi_need).unwrap_or(1.0);

        // v1.16.21 抉择后果可见化：通用数值通道进入前快照（灵石/道心/气运/寿元/
        // 修为），块尾求实际差值——特殊臂直改 sim 不经这些 Eff 字段，差值恒零
        // 自动不发聚合行（防重复记账）
        let (died, d_stones, d_dao, d_luck, d_life, d_qi) = {
            let Some(s) = self.sim.as_mut() else { return };

            let (b_stones, b_dao, b_luck, b_life, b_qi) =
                (s.stones, s.dao, s.luck, s.lifespan, s.qi);

            if e.qi_pct > 0.0 {
                gain_qi(s, need * e.qi_pct as f64);
            }

            if e.stones != 0 {
                s.stones = (s.stones + e.stones).max(0);
            }

            if e.stones_pct.abs() > 0.001 {
                s.stones = (s.stones as f64 * (1.0 + e.stones_pct as f64)).max(0.0) as i64;
            }

            if e.dao != 0.0 {
                s.dao = (s.dao + e.dao).clamp(0.0, 100.0);
            }

            if e.luck != 0.0 {
                s.luck = (s.luck + e.luck).clamp(0.0, 100.0);
            }

            if e.life_y != 0.0 {
                s.lifespan += e.life_y;
            }

            if e.injure {
                s.injured = 6;
            }

            if e.sect {
                s.sect = true;
            }

            if e.lingmai {
                s.lingmai = true;
            }

            if let Some(r) = e.relic {
                if !s.relics.contains(&r) {
                    s.relics.push(r);

                    s.equipped.push(r); // v1.5.0：模拟获得即装备（守则见文件头）
                }
            }

            if let Some((p, n)) = e.pill {
                s.pills[p] += n;
            }

            // v1.16.17 伴随事件好感通道：comp_fav 只升不降——负值忽略（红线：
            // 好感无降口），clamp 100 照道侣赠礼 2642 同款
            if e.comp_fav > 0 {
                s.comp_fav = (s.comp_fav + e.comp_fav).min(NPC_FAV_MAX);
            }

            (
                e.death > 0.0 && gen_range(0.0, 1.0) < e.death,
                s.stones - b_stones,
                (s.dao - b_dao) as f64,
                (s.luck - b_luck) as f64,
                s.lifespan - b_life,
                s.qi - b_qi,
            )
        };

        // v1.16.7 连锁剧情·链裁决：选卡即结算——置链臂（20..=25）已重写链态；
        // 其余任何选项（含检定臂/终结段/普通卡）落在链上即本次抉择为链的终局，
        // 立即清链（叙事上即「缘尽/碑散/匣还」），防残段复现。
        // v1.16.9 双段判定：链6（血债血偿）置链臂 35..=36 续编第二段窗口——
        // 20..=31 已满且 32..=34 是检定臂刻意窗外，抬 HI 会吞检定臂裁决语义
        // v1.16.10 三窗判定：链7/链8 置链臂 37..=40 新窗（CHAIN78_SPECIAL_LO/HI）
        // v1.16.11 四窗判定：链9/链10 置链臂 41..=44 新窗（CHAIN910_SPECIAL_LO/HI）
        // v1.16.13 五窗判定：链11/链12 置链臂 45..=48 新窗（CHAIN1112_SPECIAL_LO/HI）
        // v1.16.14 六窗判定：链13 置链臂 49..=52 新窗（CHAIN13_SPECIAL_LO/HI）
        // v1.16.16 七窗判定：链14 置链臂 53..=55 新窗（CHAIN14_SPECIAL_LO/HI）
        // v1.16.19 八窗判定：链15 置链臂 56..=59 新窗（CHAIN15_SPECIAL_LO/HI）
        // v1.16.20 九窗判定：链16 置链臂 60..=62 新窗（CHAIN16_SPECIAL_LO/HI——
        // 60/61 置链推进、62 窗内断链收获，语义不变）
        // v1.16.21 十窗判定：链17 置链臂 63..=65 新窗（CHAIN17_SPECIAL_LO/HI——
        // 63/64 置链推进、65 备用；新检定臂 66 起与宿敌卡化臂 67/68 均在窗外，
        // 落链上即终局的裁决语义不变）
        if !(CHAIN_SPECIAL_LO..=CHAIN_SPECIAL_HI).contains(&e.special)
            && !(CHAIN6_SPECIAL_LO..=CHAIN6_SPECIAL_HI).contains(&e.special)
            && !(CHAIN78_SPECIAL_LO..=CHAIN78_SPECIAL_HI).contains(&e.special)
            && !(CHAIN910_SPECIAL_LO..=CHAIN910_SPECIAL_HI).contains(&e.special)
            && !(CHAIN1112_SPECIAL_LO..=CHAIN1112_SPECIAL_HI).contains(&e.special)
            && !(CHAIN13_SPECIAL_LO..=CHAIN13_SPECIAL_HI).contains(&e.special)
            && !(CHAIN14_SPECIAL_LO..=CHAIN14_SPECIAL_HI).contains(&e.special)
            && !(CHAIN15_SPECIAL_LO..=CHAIN15_SPECIAL_HI).contains(&e.special)
            && !(CHAIN16_SPECIAL_LO..=CHAIN16_SPECIAL_HI).contains(&e.special)
            && !(CHAIN17_SPECIAL_LO..=CHAIN17_SPECIAL_HI).contains(&e.special)
        {
            self.chain = None;
        }

        // v1.16.21 抉择后果可见化：净变动聚合行（叙事行之前——现实侧 apply_reff
        // 同序，故事句恒最新；全零不发，特殊臂不经通用通道恒零）
        if let Some((led, lc)) = ledger_line(d_stones, d_dao, d_luck, d_life, d_qi, 0, 0) {
            self.slog(led, lc);
        }

        for m in &e.msg {
            // v1.16.21 结果句分色：good=青（进益）/!good=琥珀（对价警示）——
            // !good 原落 C_GREEN 与「疗愈=好事」语义位相撞，换新位 C_AMBER
            self.slog(m.clone(), if e.good { C_CYAN } else { C_AMBER });
        }

        if died {
            self.sim_die("命殒于外，一身道果散入天地。");
        }
    }

    fn sim_fight(&mut self, mult: f32, name: &str) -> bool {
        let (my, realm) = {
            let s = self.sim.as_ref().unwrap();

            (power(s), s.realm)
        };

        let enemy = my * mult as f64 * gen_range(0.75, 1.35);

        let r = my / enemy.max(1.0);

        let p = (r * r / (1.0 + r * r)).clamp(0.05, 0.95);

        let won = gen_range(0.0, 1.0) < p;

        enum F {
            Win(i64),

            Lose,

            Dead,
        }

        let res = {
            let s = self.sim.as_mut().unwrap();

            s.fights += 1;

            if won {
                s.wins += 1;

                self.meta.total_wins += 1;

                let g = (120.0 * (1.0 + realm as f64).powf(1.6) * gen_range(0.8, 1.5)) as i64;

                s.stones += g;

                F::Win(g)
            } else {
                s.injured = 6;

                s.stones = (s.stones as f64 * 0.85) as i64;

                if gen_range(0.0, 1.0) < 0.08 {
                    F::Dead
                } else {
                    F::Lose
                }
            }
        };

        match res {
            F::Win(g) => {
                self.slog(
                    format!("你战退{}！缴获灵石 {}。", name, fmt_int(g)),
                    C_GREEN,
                );

                self.play(crate::sounds::Which::BreakOk);

                // v1.16.8 章节书：首胜事实翻转点（第二章目标边沿）
                self.check_chapters();

                true
            }

            F::Lose => {
                self.slog(format!("不敌{}，身受重伤，仓皇遁走！", name), C_RED);

                self.play(crate::sounds::Which::BreakFail);

                false
            }

            F::Dead => {
                self.sim_die(&format!(
                    "你伤重不治，成了{}爪下的一滩墨渍——这一行，就此断在这里。",
                    name
                ));

                false
            }
        }
    }

    // ================= 模拟：问笔 =================

    fn trib_tick(&mut self, dt: f32) {
        let mut strike = false;

        {
            let s = self.sim.as_mut().unwrap();

            if let Some(t) = &mut s.trib {
                t.timer -= dt;

                t.flash = (t.flash - dt * 2.5).max(0.0);

                if t.timer <= 0.0 {
                    strike = true;

                    t.timer = 0.55;
                }
            }
        }

        if !strike {
            return;
        }

        let (wave, _spare_used, bonus) = {
            let s = self.sim.as_ref().unwrap();

            let t = s.trib.as_ref().unwrap();

            (t.wave, t.spare_used, t.bonus)
        };

        let p = {
            let s = self.sim.as_ref().unwrap();

            (0.92 - wave as f32 * 0.015 + s.dao / 500.0 + s.luck / 600.0 + bonus).min(0.97)
        };

        let survived = gen_range(0.0, 1.0) < p;

        self.shake = 16.0;

        if survived {
            let done = {
                let s = self.sim.as_mut().unwrap();

                if let Some(t) = &mut s.trib {
                    t.wave += 1;

                    t.flash = 1.0;

                    t.wave >= 9
                } else {
                    false
                }
            };

            self.play(crate::sounds::Which::Thunder);

            // v1.14.0 立绘焕新：跨相判定须在 realm 赋值前取旧档快照
            let old_tier = self.sim.as_ref().map(|s| portrait_tier(s.realm));

            if done {
                let first = {
                    let s = self.sim.as_mut().unwrap();

                    s.trib = None;

                    s.realm = 10;

                    s.layer = 0;

                    s.lifespan = s.lifespan.max(REALMS[10].life);

                    let mut f = false;

                    if s.max_realm < 10 {
                        s.max_realm = 10;

                        f = true;
                    }

                    s.highlights
                        .push(format!("{}岁，渡九重雷劫，飞升成仙！", s.age as i32));

                    f
                };

                self.slog("第九重雷劫散去，天门洞开！你踏虹而上——飞升成仙！", C_GOLD);

                self.show_banner("飞 升 成 仙", C_GOLD);

                if first {
                    self.slog("【首登】仙路初开！", C_GOLD);
                }

                // v1.14.0 立绘焕新时刻：跨相（墨仙之相）toast + 金环闪光
                if let Some(ot) = old_tier {
                    self.stage_awaken(ot, 10);
                }

                self.burst(120, true);

                self.play(crate::sounds::Which::Ascend);
            } else {
                let w = {
                    let s = self.sim.as_ref().unwrap();

                    s.trib.as_ref().map(|t| t.wave).unwrap_or(9)
                };

                self.slog(format!("第{}重雷劫轰落——你咬牙扛住了！", w), C_CYAN);
            }
        } else {
            self.sim_die("天道按语：此稿平平。退。——九问未过，形神俱灭，化作漫天流萤。");
        }
    }

    // ================= 模拟：死亡结算 → 继承 =================

    pub fn sim_die(&mut self, cause: &str) {
        self.audio.swap_bgm(false);

        let (max_realm, age, years, qi_gain, tech, relic, stones, pills, highlights) = {
            let s = self.sim.as_ref().unwrap();

            let relic = s.relics.first().copied();

            (
                s.max_realm,
                s.age,
                (s.age - self.sim_start_age) as i64,
                s.gained,
                s.tech,
                relic,
                s.stones,
                s.pills,
                s.highlights.clone(),
            )
        };

        let mut dy = (MILESTONE[max_realm.min(14)] as f64 / 8.0).ceil() as i64;
        let comp = self.sim.as_ref().and_then(|s| s.companion.clone());
        if let Some(cn) = &comp {
            dy += 50;
            // v1.16.20 复审修（medium）：继承屏伴小像名条 draw.rs 用 @14 档——
            // PANEL_WARM_CHARS 不含伴名字、prewarm_comp_page 只灌伴名 @16/@24，
            // 结缘首世直走到草稿退回页时该名 2-4 字形 @14 首见冷栅格化（v1.10.2
            // 脏矩形后毫秒级+KB 级上传，非卡帧红线，纯预热纪律对齐）。sim_die
            // 事件帧一次补灌（照 check_chapters 章开场白同款先例；FONT 未初始化
            // 时 prewarm_text 自守跳过，单测安全）
            crate::ui::prewarm_text(cn, &[14]);
            if let Some(s) = self.sim.as_mut() {
                s.highlights
                    .push(format!("{}与你相伴至最后一刻，含笑而逝。", cn));
            }
        }
        let comp_fav = self.sim.as_ref().map(|s| s.comp_fav).unwrap_or(0);
        self.feng_bond_up(comp.as_deref()); // v1.12.1 缝页：同页至终缘+1

        self.meta.dao_yun += dy;

        // v1.16.8 本稿剧情摘要：先算一次、草稿退回页（SimResult.story 会话态）与
        // 旧稿档案（SimRecord.story hq= 持久）同源共用，两处零二次拼接
        let story = self
            .sim
            .as_ref()
            .and_then(|s| story_from_highlights(&s.highlights));

        self.sim_result = Some(SimResult {
            cause: cause.to_string(),

            total_dy: dy,

            max_realm,

            age,

            years,

            qi_gain,

            tech,

            relic,

            stones,

            pills,

            highlights,

            comp: comp.clone(), // 原值保留给下方继承选项 idx 8 判定

            comp_fav,

            story: story.clone(),
        });

        // v1.16.5 旧稿档案：唯一写入点（sim_die 全死因漏斗；此后进继承屏，
        // finish_inherit 清 sim_result 前必须落账）。尾部插入、超量弹最旧；
        // 随本函数尾既有 sync_save 入档（save_flush 去抖），零额外落盘调用
        let ci = comp
            .as_deref()
            .and_then(|c| COMPANION_NAMES.iter().position(|&n| n == c));
        // v1.16.7 本世剧情摘要：大事记尾 1-2 条拼接 ≤28 字（v1.16.8 起本稿剧情
        // 上移与 SimResult.story 同源一次拼接——旧稿页/退回页一眼看到本世剧情核心）
        self.meta.hist.push(SimRecord {
            life_no: self.meta.lives,
            run_no: self.meta.total_sims,
            max_realm,
            age,
            years,
            dy,
            stones,
            comp: ci.map_or(255, |i| i as u8),
            cause: cause.to_string(),
            story,
        });
        if self.meta.hist.len() > HIST_MAX {
            self.meta.hist.remove(0);
        }

        self.sim = None;

        self.modal = None;

        // 生成 3（升级后 4）个继承选项

        let mut pool: Vec<usize> = (0..INHERITS.len()).collect();

        let mut opts = Vec::new();

        let n = 3 + self.meta.upg[4].min(1) as usize;
        if comp.is_some() {
            opts.push(8);
            pool.retain(|&x| x != 8);
        }

        for _ in 0..n {
            let k = gen_range(0, pool.len());

            opts.push(pool.remove(k));
        }

        self.inherit_opts = opts;

        self.inherit_picked = 0;

        self.scene = Scene::Inherit;

        self.slog(cause.to_string(), C_RED);

        self.play(crate::sounds::Which::Death);

        self.sync_save();
    }

    // ================= 继承 =================

    pub fn inherit_max(&self) -> u32 {
        1 + self.meta.upg[4].min(1) as u32
    }

    pub fn take_inherit(&mut self, i: usize) {
        if self.inherit_picked >= self.inherit_max() {
            return;
        }

        let Some(idx) = self.inherit_opts.get(i).copied() else {
            return;
        };

        let Some(sr) = self.sim_result.clone() else {
            return;
        };

        let scale = self.stone_scale(self.life.as_ref().map(|l| l.realm).unwrap_or(0));

        let Some(life) = self.life.as_mut() else {
            return;
        };

        let mut msg = String::new();

        match idx {
            0 => {
                let gain = sr.qi_gain * 0.5;

                gain_qi(life, gain);

                msg = format!("继承修为：现实修为 +{}", fmt_num(gain));
            }

            1 => {
                life.dao = (life.dao + 5.0).min(100.0);

                life.body = (life.body + 5.0).min(100.0);

                msg = "继承道果：道心 +5，体魄 +5（永久）".to_string();
            }

            2 => {
                if sr.tech > life.tech {
                    life.tech = sr.tech;

                    msg = format!("继承功法：《{}》", TECHS[sr.tech].name);
                } else {
                    let conv = qi_need(life) * 0.5;

                    gain_qi(life, conv);

                    msg = format!("功法不及其身，转化为修为 +{}", fmt_num(conv));
                }
            }

            3 => {
                if let Some(r) = sr.relic {
                    if !life.relics.contains(&r) {
                        life.relics.push(r);

                        msg = format!("继承法宝：【{}】", RELICS[r].name);
                    } else {
                        let g = (2000.0 * scale) as i64;

                        life.stones += g;

                        msg = format!("法宝已有，折算灵石 {}", fmt_int(g));
                    }
                } else {
                    life.stones += 1500;

                    msg = "未得法宝，折算灵石 1500".to_string();
                }
            }

            4 => {
                let g = sr.stones / 2;

                life.stones += g;

                msg = format!("继承财货：灵石 +{}", fmt_int(g));
            }

            5 => {
                for k in 0..6 {
                    life.pills[k] += sr.pills[k];
                }

                msg = "继承丹药：模拟中丹药尽数带回".to_string();
            }

            6 => {
                life.lifespan += 25.0;

                msg = "继承寿数：现实寿元 +25 载".to_string();
            }

            7 => {
                life.luck = (life.luck + 3.0).min(100.0);

                msg = "继承气运：现实气运 +3（永久）".to_string();
            }
            8 => {
                life.dao = (life.dao + 8.0).min(100.0);
                life.luck = (life.luck + 8.0).min(100.0);
                // v1.6.0 D：道侣遗泽单义——有伴才承接（comp_fav = max(现实,模拟) + 10）；
                // 无伴时维持原「道心+8 气运+8」文案
                let comp = sr.comp.clone();
                if let Some(cn) = &comp {
                    let fav = (sr.comp_fav.max(life.comp_fav) + INHERIT_LEGACY_FAV)
                        .min(NPC_FAV_MAX);
                    life.comp_fav = fav;
                    life.companion = Some(cn.clone());
                    msg = format!(
                        "道侣遗泽：{}随你魂归现实，情谊不减（好感+{}）。道心 +8，气运 +8。",
                        cn, INHERIT_LEGACY_FAV
                    );
                } else {
                    msg = "道侣遗泽：道心 +8，气运 +8（永久）".to_string();
                }
            }

            _ => {}
        }

        self.inherit_opts.remove(i);

        self.inherit_picked += 1;

        self.log(msg, C_GOLD);

        self.play(crate::sounds::Which::Ascend);

        self.sync_save();
    }

    pub fn finish_inherit(&mut self) {
        self.scene = Scene::Reality;

        self.sim_result = None;

        self.sim_log.clear();

        // v1.16.13：离场清终局页覆层态（run_recall_open 残留会在 Reality 吃掉
        // 一次 Esc；ev_recall 同批——覆层只属 Inherit 页，不跨场景泄漏）
        self.run_recall_open = false;
        self.ev_recall = None;

        self.log("大梦初醒，轮回镜归于沉寂。镜中一生，造化归身。", C_TEXT);

        // v1.16.21 继承未选折算（惜选张力）：收束时把未携走的造化各折一份余泽
        // 入道韵（dao_yun 既有持久键零新键；折值克制——3 选 1 至多两份，约
        // 8~22 道韵，低于章成之礼 50，不夺正选的价值）
        let fold: i64 = self
            .inherit_opts
            .iter()
            .map(|&idx| match idx {
                0 => 6,  // 修为
                1 => 8,  // 道果
                2 => 8,  // 功法
                3 => 10, // 法宝
                4 => 10, // 财货
                5 => 4,  // 丹药
                6 => 12, // 寿数
                7 => 4,  // 气运
                _ => 6,  // 道侣遗泽
            })
            .sum();

        if fold > 0 {
            self.meta.dao_yun += fold;

            self.log(
                format!("余泽入身：未携的造化化作道韵 +{}。", fold),
                C_GOLD,
            );
        }

        // v1.16.10 新手一次性提示（sy= bit3）：首次传承带回指路一句——行囊验宝、
        // 闭关自涨修为。闩语义=「从未传承过」（additive 边沿，推导不出故挂闩；
        // 老玩家升版至多一见，与新档同句）。
        if self.meta.sys_seen & 0b1000 == 0 {
            self.meta.sys_seen |= 0b1000;
            self.log(
                "镜中造化已入此身——行囊可验宝，闭关自涨修为。推演不误砍柴工。",
                C_GOLD,
            );
            self.toast("造化已入现实——行囊验宝，闭关自涨修为", C_GOLD);
        }

        self.sync_save();
    }

    // ================= 现实：行动 =================

    // v1.6.0 B：每载修炼收益完整乘链单一来源（act_cultivate 与 Cult 弹层副标签共用）
    // 乘链：cult_rate（四维/功法/环境/法宝/烙印/天赋/道侣/伤势）
    //   × 洞府闭关 DONGFU_CULT_MULT × 聚灵阵 JUELING_MULT × 天气×所在地 weather_loc_mult
    //   × 每载 4 倍（原 act_cultivate 的 rate*4.0）
    pub fn cult_gain_per_year(&self) -> f64 {
        let Some(l) = self.life.as_ref() else {
            return 0.0;
        };
        let mut r = cult_rate(l, &self.meta);
        // 山河图志：清源洞府闭关加成
        if l.loc == NODE_DONGFU {
            r *= DONGFU_CULT_MULT;
        }
        // v1.5.0 四时洞天：聚灵阵层级 + 天气×所在地收益
        r *= JUELING_MULT[self.meta.dongfu[0].clamp(0, 3) as usize];
        r *= weather_loc_mult(l.weather as usize, l.loc);
        // v1.8.0 C2：区域机制「冰心澄神」（现实闭关效率）
        r *= region_mech(self.meta.map_region).cult_mult;
        // v1.8.0 D1：醉道人「酒中真言」——每载修炼效率随好感档提升（不挂 cult_rate，模拟侧不受益）
        r *= crate::npc::drunk_cult_mult(self.meta.npc_fav[crate::npc::NPC_DRUNK]);
        r * 4.0
    }

    // auto=true 为周天吐纳路径：逐年「你闭关一载」日志与 Gain 音静音（防 1.6s 一条刷爆
    // 日志区），由 tick_auto 每 10 载合并记账；手点（修炼弹层/洞府闭关/冒烟）传 false 全量反馈。
    // 返回本载修为收益（周天累计记账用）；结算路径与手点完全同口，无第二条实现。
    pub fn act_cultivate(&mut self, auto: bool) -> f64 {
        if self.life.is_none() {
            return 0.0;
        }

        let rate = self.cult_gain_per_year();

        let (died, msg) = {
            let l = self.life.as_mut().unwrap();

            l.age += 1.0;

            if l.injured > 0 {
                l.injured -= 1;
            }

            l.qi += rate;

            let msg = format!("你闭关一载，修为精进。（+{}）", fmt_num(rate));

            let died = l.age >= l.lifespan;

            (died, msg)
        };

        if !auto {
            self.log(msg, C_CYAN);

            self.play(crate::sounds::Which::Gain);
        }

        if died {
            self.real_die("寿元耗尽，你在自家榻上安然合卷——名字终被写对了一次。唯道韵永存。");

            return 0.0;
        }

        // v1.5.0 四时：死亡当年不推时辰、不产灵田（推进点=现实推岁口，唯一）
        self.advance_time();

        self.check_ach_all();

        self.sync_save();

        rate
    }

    // ================= v1.16.17 双修共进（道侣专页钮，现实侧主动双修） =================
    // 收益/代价定稿：耗灵石 500×(1+realm) 线性——仅借「随境界涨」语义，实为线性
    // 直涨（realm=0 → 500、2 → 1500）；勿与 exchange_cost 类比（那是 1500×(1+realm)^1.3
    // 幂曲线，game.rs:10373，曲线不同，见 v1.16.18 清账②改口）、情谊+5
    // （COMPANION_GIFT_FAV 同量级）、修为 += 2×cult_gain_per_year（直加 l.qi
    // 照 act_cultivate 同口——勿动 cult_rate 乘链，否则离线/周天/弹层三路径被静默
    // 放大）、道心+1 气运+1（clamp 100 照遗泽 8921/8916 口）；不推岁（冷却挂年岁闩）。
    // 限制每载一次：Life.sx_year 记上次双修年岁（键 Lsxy=，缺键 0 零迁移；起始年岁
    // 16 起，0 即「从未双修」天然无碰撞）。comp_fav 只升不降既定语义（此为新增升口，
    // 现实侧唯一日常增长口，恰补赠礼事件外的空缺）。

    /// 双修年岁取整单口（纯函数供门控/写档/单测共用）：age 越界钳 u16（防御纪律）
    pub fn dual_year(age: f64) -> u16 {
        (age as i64).clamp(0, u16::MAX as i64) as u16
    }

    /// 双修定价单口（按钮文案与执行口共用一价）。v1.16.18 清账②改口：仅借
    /// 「随境界涨」语义——实为 500×(1+realm) 线性；勿与 exchange_cost 类比
    /// （那是 1500×(1+realm)^1.3 幂曲线 game.rs:10373，曲线不同）
    pub fn dual_cult_cost(&self) -> i64 {
        let realm = self.life.as_ref().map(|l| l.realm).unwrap_or(0);
        (500.0 * (1.0 + realm as f64)) as i64
    }

    /// 双修门控（纯函数供按钮态/执行口/单测三处单源）：
    /// NoCompanion > ThisYear > Stones 优先级（按钮置灰依次提示差什么）
    pub fn dual_cult_gate(l: Option<&Life>, cost: i64) -> DualGate {
        let Some(l) = l else {
            return DualGate::NoCompanion;
        };
        if l.companion.is_none() {
            return DualGate::NoCompanion;
        }
        if l.sx_year == Self::dual_year(l.age) {
            return DualGate::ThisYear;
        }
        if l.stones < cost {
            return DualGate::Stones;
        }
        DualGate::Ready
    }

    /// 双修共进执行（道侣专页按钮单口）：门控全再校验（按钮仅 UI 门，照 gift_npc
    /// 「内部再校验」先例）；修为直加 l.qi 不入乘链（cult_gain_per_year 单一来源纪律）
    pub fn act_dual_cult(&mut self) {
        let cost = self.dual_cult_cost();
        if matches!(Self::dual_cult_gate(self.life.as_ref(), cost), DualGate::Stones) {
            self.toast("灵石不足", C_RED);
            return;
        }
        if !matches!(
            Self::dual_cult_gate(self.life.as_ref(), cost),
            DualGate::Ready
        ) {
            return; // 未结缘/本载已修：静默拒绝（按钮置灰态本不可达，防御口）
        }
        let gain = self.cult_gain_per_year() * 2.0;
        let cn = self
            .life
            .as_ref()
            .and_then(|l| l.companion.clone())
            .unwrap_or_default();
        {
            let l = self.life.as_mut().unwrap();
            l.stones -= cost;
            l.qi += gain; // 直加照 act_cultivate 同口（不入 cult_rate 乘链）
            l.sx_year = Self::dual_year(l.age);
            l.dao = (l.dao + 1.0).min(100.0);
            l.luck = (l.luck + 1.0).min(100.0);
        }
        crate::npc::add_comp_fav(self, crate::data::COMPANION_GIFT_FAV);
        self.log(
            format!(
                "你与{}双修共进，息脉相合一夜悟道：情谊+{}，道心+1，气运+1，修为+{}。",
                cn,
                crate::data::COMPANION_GIFT_FAV,
                fmt_num(gain)
            ),
            C_GOLD,
        );
        // v1.16.18 搭车①：双修成功即时反馈（Reality toast 槽 draw.rs:1161 已渲染；
        // 专页按钮是唯一入口，Tab(6) 开页帧已预热「双修共进」@20，情谊/修为/+数字
        // 全在菜单期 panel_warm 常驻档——弹帧零冷栅格化）。与上行 log 同帧双通道
        // 仍是文案分工：log=信箱记账，toast=当场提示
        self.toast(
            format!(
                "双修共进 · 情谊+{} · 修为+{}",
                crate::data::COMPANION_GIFT_FAV,
                fmt_num(gain)
            ),
            C_GOLD,
        );
        self.play(crate::sounds::Which::Ascend);
        self.sync_save();
    }

    // ================= v1.9.25 周天吐纳：现实自动修炼（放置主循环补完） =================
    // 挂点唯一：main.rs 每帧 tick_auto(dt)。结算与手点完全同口 act_cultivate(true)，
    // 严禁复制第二条结算路径；寿尽 real_die→life None→自然停机。
    pub fn tick_auto(&mut self, dt: f32) {
        if self.scene != Scene::Reality || self.meta.auto != 1 {
            return;
        }

        // 广告/事件卡/弹层/结算期冻结（不清零，回来接着吐纳）
        // v1.10.2 复审②：离闭结算卡独占输入期间同样冻结——否则周天吐纳在卡后照跑，
        // 最坏 age 达 lifespan → real_die 玩家死在读卡画面里
        if self.ad.is_some()
            || self.rmodal.is_some()
            || self.submenu.is_some()
            || self.settle.is_some()
            || self.offline_card.is_some()
            || self.ending_show.is_some() // v1.12.1 终卷演出页覆层独占期同样冻结
        {
            return;
        }

        if self.life.is_none() {
            self.auto_acc = 0.0;

            self.auto_years = 0;

            self.auto_gain = 0.0;

            return;
        }

        self.auto_acc += dt;

        if self.auto_acc < AUTO_CULT_SECS {
            return;
        }

        self.auto_acc -= AUTO_CULT_SECS;

        // 突破衔接：修为满时同层下一层自动冲击（复用 act_break 单口不另 roll，
        // 同层成功率 0.96+dao/600 上限 0.99，失败无身死）；跨境（失败 qi×0.5+重伤）
        // 决策留给玩家——停表 + 一次性 toast，离满修为态自动复位
        if self.can_break() {
            let cross = self
                .life
                .as_ref()
                .map_or(false, |l| l.layer + 1 >= REALMS[l.realm].layers.len());

            if cross {
                if !self.auto_toast_done {
                    self.auto_toast_done = true;

                    self.toast("修为圆满，可冲击破境", C_GOLD);
                }

                self.auto_acc = 0.0; // 停表

                return;
            }

            self.act_break();

            return;
        }

        self.auto_toast_done = false;

        let rate = self.act_cultivate(true);

        // 寿尽时 act_cultivate 内部 real_die（life 已 None）：自然停机，不计账
        if self.life.is_some() {
            self.auto_years += 1;

            self.auto_gain += rate;

            if self.auto_years >= 10 {
                self.log(
                    format!(
                        "周天吐纳十载，修为精进累计+{}。",
                        fmt_num(self.auto_gain)
                    ),
                    C_CYAN,
                );

                self.play(crate::sounds::Which::Gain);

                self.auto_years = 0;

                self.auto_gain = 0.0;
            }
        }
    }

    // ================= v1.5.0 四时洞天：称号 / 法宝装备 / 洞府经营 / 成就进度 =================

    // 佩戴称号：-1 自动跟随 title_of；15 弑神者；16 章节称号（v1.16.10）；
    // 17 执笔人（v1.16.15 四结局全收集）；0..=14 锁定境界称号
    pub fn worn_title_of(&self) -> String {
        match self.meta.worn_title {
            -1 => self.title_of(),
            15 => "弑神者".to_string(),
            16 => "雨夜执印者".to_string(),
            17 => "执笔人".to_string(),
            t => realm_title(t.clamp(0, 14) as usize).to_string(),
        }
    }

    // 佩戴称号（值域即 {-1, 0..=14, 15, 16, 17}）：越权/未获得拒绝并 toast
    pub fn set_worn_title(&mut self, t: i32) {
        let ok = match t {
            -1 => true,
            15 => self.meta.neme_slain >= 3,
            // v1.16.10 章节称号：首章「雨夜执印」已了（chap_done bit0）即授权
            16 => self.meta.chap_done & 1 != 0,
            // v1.16.15 全卷称号：四结局全览（endings_seen 0b1111，结局之礼边沿
            // 帧 toast 指路）即授权——位即凭证零新键
            17 => self.meta.endings_seen == 0b1111,
            v if (0..=14).contains(&v) => self.meta.best_realm >= v as usize,
            _ => false,
        };
        if !ok {
            self.toast("尚未获得此称号", C_RED);
            return;
        }
        self.meta.worn_title = t;
        let name = self.worn_title_of();
        self.log(format!("你戴上称号『{}』。", name), C_GOLD);
        self.play(crate::sounds::Which::Click);
        self.sync_save();
    }

    // 装备法宝 r（未拥有静默拒绝；重复/槽满提示）
    pub fn equip_relic(&mut self, r: usize) {
        let Some(l) = self.life.as_ref() else {
            return;
        };
        if !l.relics.contains(&r) {
            return;
        }
        if l.equipped.contains(&r) {
            self.toast("此宝已在用", C_DIM);
            return;
        }
        if l.equipped.len() >= RELIC_EQUIP_MAX {
            self.toast("法宝槽已满，请先卸下一件", C_RED);
            return;
        }
        if let Some(l) = self.life.as_mut() {
            l.equipped.push(r);
        }
        self.log(format!("你祭起【{}】，灵光贯体。", RELICS[r].name), C_GOLD);
        self.play(crate::sounds::Which::Ascend);
        self.sync_save();
    }

    // 卸下法宝 r（不消失，仍保有 RELIC_UNEQUIP_FRAC 三成灵性）
    pub fn unequip_relic(&mut self, r: usize) {
        let had = self
            .life
            .as_ref()
            .map_or(false, |l| l.equipped.contains(&r));
        if !had {
            return;
        }
        if let Some(l) = self.life.as_mut() {
            l.equipped.retain(|&x| x != r);
        }
        self.log(format!("你收起【{}】，灵光渐敛。", RELICS[r].name), C_TEXT);
        self.play(crate::sounds::Which::Click);
        self.sync_save();
    }

    // 四时推进（每推岁一载调用一次）：时段流转、天气演化、灵田产出。
    // 调用点唯一化：现实 act_cultivate（寿尽提前 return 不调用）与世界 spend_year（寿尽同样不调用）。
    // 借用纪律：life.as_mut 圈在块内，日志在块外。
    pub fn advance_time(&mut self) {
        let mut lingtian = 0i64;
        let weather_changed;
        {
            let Some(l) = self.life.as_mut() else {
                return;
            };
            // 时段：晨→昼→昏→夜
            l.slot = (l.slot + 1) % 4;
            // 天气：按转移矩阵 roll（浮点概率，索引兜底取当前）
            let row = WEATHER_TRANS[l.weather.min(4) as usize];
            let roll = gen_range(0.0, 1.0f32);
            let mut next = l.weather.min(4) as usize;
            let mut acc = 0.0f32;
            for (k, &p) in row.iter().enumerate() {
                acc += p;
                if roll <= acc {
                    next = k;
                    break;
                }
            }
            next = next.min(4);
            weather_changed = next != l.weather.min(4) as usize;
            l.weather = next as u8;
            // 灵田：按层级产出，收成随天气与所在地起伏
            // v1.10.2 成行择道：丹修（path=3）灵田岁入 ×1.6（静态乘区，随唯一推岁口）
            let path_pill = if l.path == PATH_PILL { 1.6 } else { 1.0 };
            let lv = self.meta.dongfu[1].clamp(0, 3) as usize;
            let y = (LINGTIAN_YIELD[lv.min(3)] as f64
                * weather_loc_mult(l.weather as usize, l.loc)
                * path_pill)
            .round() as i64;
            if y > 0 {
                l.stones += y;
                lingtian = y;
            }
        }
        if lingtian > 0 {
            self.log(format!("洞府灵田岁入灵石 {}。", fmt_int(lingtian)), C_CYAN);
        }
        if weather_changed {
            let w = self
                .life
                .as_ref()
                .map(|l| l.weather.min(4) as usize)
                .unwrap_or(0);
            self.log(format!("天象流转：{}。", WEATHERS[w]), C_DIM);
        }
        // v1.9.25 宿敌追缉：宿敌随你的岁月成长。唯一推岁口（act_cultivate 与世界 spend_year
        // 都经此）覆盖闭关/赶路/任务/远征全部耗岁路径；6%/载 roll 晋入下一境，
        // 上限 min(14, 历代最高境+5)——拖得越久赏金越厚（stone_scale/道韵随境界）但
        // revenge_p 越差，与寻仇修复（下限归 0+重伤禁手）构成「养伤→变强→趁早动手」闭环。
        // 借用纪律仿灵田先例：块内收集，块外记日志。
        let mut neme_up: Option<(String, usize)> = None;
        {
            let cap = (self.meta.best_realm + 5).min(14);
            let roll = self.meta.neme.as_ref().map_or(false, |n| {
                n.realm < cap && gen_range(0.0, 1.0) < 0.06
            });
            if roll {
                if let Some(n) = self.meta.neme.as_mut() {
                    n.realm = (n.realm + 1).min(cap);
                    neme_up = Some((n.full(), n.realm));
                }
            }
        }
        if let Some((full, r)) = neme_up {
            self.log(
                format!("风声传来：{}闭关数载，已晋入{}境！", full, REALMS[r.min(14)].name),
                C_RED,
            );
        }
    }

    // 成就进度：(当前, 目标) 文案；无量化进度的成就返回 None
    pub fn ach_progress(&self, id: usize) -> Option<(String, String)> {
        let l = self.life.as_ref();
        let mr = self.meta.best_realm;
        let realm_goal = [1usize, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 13];
        Some(match id {
            0..=11 => (
                REALMS[mr.min(14)].name.to_string(),
                REALMS[realm_goal[id]].name.to_string(),
            ),
            12 => (fmt_num(l.map(|x| x.age).unwrap_or(0.0)), "十万".to_string()),
            13 => (format!("{}", self.meta.lives), "10".to_string()),
            14 => (format!("{}", self.meta.lives), "100".to_string()),
            15 => (
                fmt_num(l.map(|x| x.stones as f64).unwrap_or(0.0)),
                "100万".to_string(),
            ),
            16 => (fmt_num(l.map(power).unwrap_or(0.0)), "10亿".to_string()),
            17 => (
                format!("{}", l.map(|x| x.washed).unwrap_or(0)),
                "10".to_string(),
            ),
            18 => (format!("{}", self.meta.neme_slain), "1".to_string()),
            19 => (fmt_num(l.map(|x| x.age).unwrap_or(0.0)), "1000".to_string()),
            20 => (
                if l.map_or(false, |x| x.sect) {
                    REALMS[mr.min(14)].name.to_string()
                } else {
                    "未入宗".to_string()
                },
                REALMS[3].name.to_string(),
            ),
            21 => (
                if l.map_or(false, |x| x.natural_end) {
                    REALMS[mr.min(14)].name.to_string()
                } else {
                    "未坐化".to_string()
                },
                REALMS[4].name.to_string(),
            ),
            22 => (format!("{}", self.meta.companion_lives), "1".to_string()),
            26 => (format!("{}", self.meta.companion_lives), "3".to_string()),
            27 => (
                REALMS[l.map(|x| x.realm).unwrap_or(0).min(14)].name.to_string(),
                REALMS[10].name.to_string(),
            ),
            28 => (
                fmt_num(l.map(|x| x.stones as f64).unwrap_or(0.0)),
                "1亿".to_string(),
            ),
            29 => (fmt_num(l.map(|x| x.age).unwrap_or(0.0)), "5000".to_string()),
            _ => return None,
        })
    }

    pub fn can_break(&self) -> bool {
        if let Some(l) = &self.life {
            let max = REALMS.len() - 1;

            return l.qi >= qi_need(l) && !(l.realm == max && l.layer == 3);
        }

        false
    }

    // v1.16.10 新手一次性提示（sy= bit4）单口：修为首满指路一句——底栏「修炼」
    // 可破境。闩语义=「从未圆满过」（additive 边沿）；draw_reality 帧头调用（draw
    // 路径直写 meta 先例=draw.rs 周天首开 sy bit0），单测直接打本口。边沿帧一次
    // sync_save（去抖落盘），稳态帧零分配零落盘。
    pub fn sys_hint_break_ready(&mut self) {
        if self.can_break() && self.meta.sys_seen & 0b10000 == 0 {
            self.meta.sys_seen |= 0b10000;
            self.log(
                "修为已圆满——底栏「修炼」可破境，更上层楼。",
                C_GOLD,
            );
            self.toast("修为圆满！底栏「修炼」可破境", C_GOLD);
            self.sync_save();
        }
    }

    // v1.16.11 系统初见闩 bit5（sy= additive，照 bit4 单口语言）：首见洞府经营指路
    // ——山河图经营线唯一无指向实缺（指引叶 7 要求 loc==NODE_DONGFU 且灵石/贡献
    // 双足，未至洞府时全程无任何指向）。文案指向「经营洞府」钮（world.rs 洞府臂）。
    // 挂点=事件口非帧路径：open_world（覆盖 NODE_START==NODE_DONGFU 出生即在此与
    // advance_map 换幕归位两路）+ travel 抵达洞府（覆盖离府再归）；单口自带 loc
    // 守卫，边沿帧才动 meta/落盘，重复调用幂等。旧玩家升版至多一见（先例 bit3 注释）
    pub fn sys_hint_dongfu(&mut self) {
        if self.meta.sys_seen & 0b100000 != 0 {
            return;
        }
        if !self.life.as_ref().is_some_and(|l| l.loc == NODE_DONGFU) {
            return;
        }
        self.meta.sys_seen |= 0b100000;
        self.log(
            "山河图洞府处，「经营洞府」可扩建聚灵阵·灵田·丹房。",
            C_GOLD,
        );
        self.toast("洞府经营已开放！山河图「经营洞府」", C_GOLD);
        self.sync_save();
    }

    pub fn break_label(&self) -> String {
        if let Some(l) = &self.life {
            let rd = &REALMS[l.realm];

            if l.layer + 1 < rd.layers.len() {
                return format!("突破·{}", rd.layers[l.layer + 1]);
            }

            if l.realm + 1 < REALMS.len() {
                return format!("冲击·{}境", REALMS[l.realm + 1].name);
            }
        }

        "大道尽头".to_string()
    }

    pub fn act_break(&mut self) {
        if !self.can_break() {
            return;
        }

        // v1.14.0 立绘焕新：跨相判定须在 realm 赋值前取旧档快照（问笔/一般跨境两路共用）
        let old_tier = self.life.as_ref().map(|l| portrait_tier(l.realm));

        let (need, same_realm) = {
            let l = self.life.as_ref().unwrap();

            (qi_need(l), l.layer + 1 < REALMS[l.realm].layers.len())
        };

        if same_realm {
            let p = {
                let l = self.life.as_ref().unwrap();

                (0.96 + l.dao / 600.0).min(0.99)
            };

            let (ok, r, ly) = {
                let l = self.life.as_mut().unwrap();

                l.qi -= need;

                let ok = gen_range(0.0, 1.0) < p;

                if ok {
                    l.layer += 1;
                } else {
                    l.qi = (l.qi * 0.85).max(0.0);
                }

                (ok, l.realm, l.layer)
            };

            if ok {
                self.log(
                    format!("现实突破：你踏入{}，根基扎实无比。", REALMS[r].layers[ly]),
                    C_GREEN,
                );

                self.play(crate::sounds::Which::BreakOk);
            } else {
                self.log("突破小有波折，修为略损。", C_DIM);
            }
        } else {
            let target = {
                let l = self.life.as_ref().unwrap();

                l.realm + 1
            };

            if target >= REALMS.len() {
                return;
            }

            if target == 10 {
                    // 现实问笔：九重雷劫一次性结算（轮回镜护持，失败重伤不身死）
                    let mut report: Vec<String> = Vec::new();
                    let (pw, dead_at) = {
                        let l = self.life.as_ref().unwrap();
                        let e = teff(l);
                        let mut base = (0.92 - 4.0 * 0.015)
                            + l.dao / 500.0
                            + l.luck / 600.0
                            + l.body as f32 / 800.0;

                        // v1.12.1 缝页缘三为纯叙事（护栏裁剪：无数值 perk），史行见婚约高亮
                        let mut b = e.trib_add + if l.pills[5] > 0 { 0.12 } else { 0.0 };
                        if l.pills[5] > 0 {
                            report.push("护神丹化作雷纹护体（存活+12%）。".to_string());
                        }
                        if l.loc == NODE_PEAK {
                            base += PEAK_TRIB_BONUS;
                            report.push("天雷峰问笔台：每重雷劫存活+2%。".to_string());
                        }
                        let mut pw = 0u8;
                        let mut dead_at = None;
                        for w in 0..9u8 {
                            let pp = (base + b - w as f32 * 0.01).min(0.97);
                            if gen_range(0.0, 1.0) < pp {
                                pw += 1;
                            } else {
                                dead_at = Some(w);
                                break;
                            }
                        }
                        (pw, dead_at)
                    };
                    {
                        let l = self.life.as_mut().unwrap();
                        l.qi -= qi_need(l);
                        if l.pills[5] > 0 {
                            l.pills[5] -= 1;
                        }
                    }
                    for r in &report {
                        self.log(r.clone(), C_CYAN);
                    }
                    self.play(crate::sounds::Which::Thunder);
                    self.shake = 16.0;

                    if dead_at.is_none() {
                        let first = {
                            let l = self.life.as_mut().unwrap();
                            l.realm = 10;
                            l.layer = 0;
                            l.lifespan = l
                                .lifespan
                                .max(REALMS[10].life * (1.0 + self.meta.upg[8] as f64 * 0.10));
                            let mut f = false;
                            if l.max_realm < 10 {
                                l.max_realm = 10;
                                if !self.meta.first_realm[10] {
                                    self.meta.first_realm[10] = true;
                                    l.new_high = true;
                                    f = true;
                                }
                            }
                            f
                        };
                        self.log(
                            "九重雷劫尽数扛下！天门洞开，霞光万道——现实中，你飞升成仙！",
                            C_GOLD,
                        );
                        self.show_banner("现 实 飞 升", C_GOLD);
                        self.burst(140, true);
                        self.play(crate::sounds::Which::Ascend);
                        if first {
                            self.log("【首登】现实首登仙界！", C_GOLD);
                        }
                        self.check_ach(27);
                        self.check_ach_all();
                        // v1.16.8 章节书：首登仙界事实翻转点（第七章问笔目标边沿）
                        self.check_chapters();
                        // v1.14.0 立绘焕新时刻：跨相（墨仙之相）toast + 金环闪光
                        //（置于成就 toast 之后，防被同帧成就播报顶掉）
                        if let Some(ot) = old_tier {
                            self.stage_awaken(ot, 10);
                        }
                        // v1.12.1 终卷：封卷吏已斩（slain>=6=链顶已通）者，雷劫扛下当帧
                        // 弹终战抉择卡；未通链者维持现状飞升不弹卡。揭底只取两行压色，
                        // 避免与雷劫战报同屏剧透冲淡卡面（§8.2-8.3 纸底显形/玺灵齐声）。
                        if self.meta.neme_slain >= 6 {
                            self.log(
                                "怀里旧镇纸烫穿衣襟——纸底十万层旧字显形，每一层的名字都不一样。",
                                C_RED,
                            );
                            self.log(
                                "玺灵十万层齐声：你不是这一稿，你是十万稿摞起来的一册人。",
                                C_CYAN,
                            );
                            self.sync_taodie(); // 门前对账：保卡面「讨牒 6/6」恒真
                            self.open_finale_card();
                        }
                    } else {
                        let w = dead_at.unwrap() + 1;
                        let l = self.life.as_mut().unwrap();
                        l.injured = 10;
                        l.qi = (l.qi * 0.4).max(0.0);
                        self.log(format!(
                        "问笔至第{}重雷劫功亏一篑！轮回镜金光护住你的元神——身受重伤，但无性命之忧。重整旗鼓，来日再战！",
                        w
                    ), C_RED);
                        self.play(crate::sounds::Which::BreakFail);
                    }
                    self.sync_save();
                    return;
                }

            let p = {
                let l = self.life.as_ref().unwrap();

                big_break_p(l, target)
            };
            // v1.8.0 D1：大师姐「剑意点拨」——仅现实跨境破境加成（同境分支、问笔分支不加；
            // big_break_p 函数体不动，模拟侧不受影响）
            let p = (p + crate::npc::senior_break_add(self.meta.npc_fav[crate::npc::NPC_SENIOR]))
                .clamp(0.05, 0.95);

            let rd = &REALMS[target];

            let (ok, first) = {
                let l = self.life.as_mut().unwrap();

                l.qi -= need;

                if l.pills[1] > 0 && target == 2 {
                    l.pills[1] -= 1;
                }

                if gen_range(0.0, 1.0) < p {
                    l.realm = target;

                    l.layer = 0;

                    l.lifespan = l
                        .lifespan
                        .max(rd.life * (1.0 + self.meta.upg[8] as f64 * 0.10));

                    let mut f = false;

                    if target > l.max_realm {
                        l.max_realm = target;

                        if !self.meta.first_realm[target] {
                            self.meta.first_realm[target] = true;

                            l.new_high = true;

                            f = true;
                        }
                    }

                    (true, f)
                } else {
                    l.qi = (l.qi * 0.5).max(0.0);

                    l.injured = 3;

                    (false, false)
                }
            };

            if ok {
                self.log(format!("轰！现实突破【{}境】！寿元大增！", rd.name), C_GOLD);

                self.show_banner(format!("突破 · {}境", rd.name), C_GOLD);

                if first {
                    self.log("【首登】现实中的你首次踏入此境！", C_GOLD);
                }

                self.burst(60, true);

                self.play(crate::sounds::Which::Ascend);

                self.check_ach_all();

                // v1.16.8 章节书：现实首破境事实翻转点（第二章目标边沿）
                self.check_chapters();

                // v1.14.0 立绘焕新时刻：跨相破境 toast + 金环闪光（置于成就 toast
                // 之后——toast 单槽后写胜，焕然词不被同帧成就播报顶掉）
                if let Some(ot) = old_tier {
                    self.stage_awaken(ot, target);
                }

                // v1.12.0 P0 修复：跨境破入成行（realm 2）成功当帧弹道途卡——此前
                // open_path_card 仅 resume_life 旧档补弹，会话内首次破境后三途五处
                // 乘区全程空转；rmodal 被占时 open_path_card 自带 no-op，无需排队
                if target == 2 {
                    self.open_path_card();
                }
            } else {
                self.log(
                    format!(
                        "现实冲击{}境失败！真元反噬受轻伤（现实突破有轮回镜护持，不会身死）。",
                        rd.name
                    ),
                    C_RED,
                );

                self.play(crate::sounds::Which::BreakFail);
            }
        }

        self.sync_save();
    }

    pub fn exchange_points(&mut self) {
        let realm = self.life.as_ref().map(|l| l.realm).unwrap_or(0);

        let cost = (1500.0 * (1.0 + realm as f64).powf(1.3)) as i64;

        let can = self.life.as_ref().map_or(false, |l| l.stones >= cost);

        if !can {
            self.toast("灵石不足", C_RED);

            return;
        }

        let l = self.life.as_mut().unwrap();

        l.stones -= cost;

        self.meta.sim_points += 1;

        self.log(
            format!("以{}灵石向轮回镜献祭，得 1 枚模拟点数。", fmt_int(cost)),
            C_CYAN,
        );

        // v1.16.8 系统初见闩（sy= bit1）：点数兑换首通一句戏文（讨牒 log 先例，
        // 一次性边沿——推导不出「从未兑过」，挂 meta additive 闩，缺键 0 零迁移）
        if self.meta.sys_seen & 0b0010 == 0 {
            self.meta.sys_seen |= 0b0010;
            self.log(
                "轮回镜初饮灵石——以石换点，点数之道既通。此后灵石不屯，皆化万世之资。",
                C_GOLD,
            );
            self.toast("点数兑换 · 初通", C_GOLD);
        }

        self.play(crate::sounds::Which::Coin);

        self.sync_save();
    }

    pub fn exchange_cost(&self) -> i64 {
        let realm = self.life.as_ref().map(|l| l.realm).unwrap_or(0);

        (1500.0 * (1.0 + realm as f64).powf(1.3)) as i64
    }

    // ================= v1.7.0 点数经济：法宝献祭 / 死档 / 幻境广告 =================

    /// 山穷水尽：点数不够一次模拟、灵石不够一次兑换、又无可献祭的法宝
    pub fn is_stuck(&self) -> bool {
        if self.life.is_none() || self.meta.sim_points >= self.sim_cost() {
            return false;
        }
        match self.life.as_ref() {
            Some(l) => l.stones < self.exchange_cost() as i64 && l.relics.is_empty(),
            None => false,
        }
    }

    /// 打开幻境广告浮层（模态，独占输入直到领取或放弃）
    pub fn start_ad(&mut self) {
        if self.ad.is_some() {
            return;
        }
        self.ad = Some(crate::data::AD_SECONDS);
        self.log(
            "山穷水尽之际，镜灵为你接引一段『商途幻境』——看完可得模拟点数。",
            C_CYAN,
        );
        self.play(crate::sounds::Which::Click);
    }

    /// v1.9.12 菜单自检面板用的场景短名（中文，仅 wasm 诊断显示）
    pub fn scene_xx_name(&self) -> &'static str {
        match self.scene {
            Scene::Menu => "菜单",
            Scene::Reality => "现实",
            Scene::Draft => "选天赋",
            Scene::Sim => "模拟",
            Scene::Inherit => "继承",
            Scene::Settle => "结算",
            Scene::World => "地图",
        }
    }

    /// 每帧推进广告倒计时（真实秒；归 0 后钳住，等待领取）
    pub fn tick_ad(&mut self, dt: f32) {
        if let Some(t) = self.ad.as_mut() {
            *t = (*t - dt).max(0.0);
        }
    }

    /// 领取广告奖励（仅倒计时走完可用）
    pub fn collect_ad(&mut self) {
        if self.ad != Some(0.0) {
            return;
        }
        self.ad = None;
        self.meta.sim_points += crate::data::AD_REWARD;
        self.meta.ad_watched += 1;
        self.toast(
            &format!("领取 {} 枚模拟点数", crate::data::AD_REWARD),
            C_CYAN,
        );
        self.play(crate::sounds::Which::Coin);
        self.sync_save();
    }

    /// 献祭法宝换模拟点（永久失去；已装备的一并卸下）
    /// v1.8.0 E：被献祭件带灵纹时，灵纹随之消散（lv 清零，再得重新算）
    pub fn sacrifice_relic(&mut self, k: usize) {
        let (gain, name, refined) = {
            let Some(l) = self.life.as_mut() else { return };
            let Some(pos) = l.relics.iter().position(|&r| r == k) else {
                return;
            };
            l.relics.remove(pos);
            l.equipped.retain(|&r| r != k);
            let refined = l.relic_lv[k.min(7)];
            l.relic_lv[k.min(7)] = 0;
            (RELICS[k].pt, RELICS[k].name, refined)
        };
        self.meta.sim_points += gain;
        self.log(
            format!("献祭【{}】，轮回镜折算 {} 枚模拟点数。", name, gain),
            C_CYAN,
        );
        if refined > 0 {
            self.log("其上灵纹随之消散。", C_DIM);
        }
        self.play(crate::sounds::Which::Coin);
        self.check_ach_all();
        self.sync_save();
    }

    /// v1.8.0 E：炼化——以 material 为引，为目标法宝提升一级灵纹（至多 REFINE_MAX）。
    /// 卫语句：双方均须拥有、目标≠材料、目标未满级（满级 toast「此宝灵纹已臻圆满」）。
    /// 材料若在装备栏，消耗时自动卸下（同献祭 retain），保住 equipped ⊆ relics 不变量。
    pub fn refine_relic(&mut self, target: usize, material: usize) {
        let (t_owned, m_owned, t_lv, t_name, m_name) = match self.life.as_ref() {
            Some(l) => (
                l.relics.contains(&target),
                l.relics.contains(&material),
                l.relic_lv[target.min(7)],
                RELICS[target.min(RELICS.len() - 1)].name,
                RELICS[material.min(RELICS.len() - 1)].name,
            ),
            None => return,
        };
        if !t_owned || !m_owned || target == material {
            return;
        }
        if t_lv >= REFINE_MAX {
            self.toast("此宝灵纹已臻圆满", C_DIM);
            return;
        }
        let new_lv = {
            let Some(l) = self.life.as_mut() else { return };
            let Some(pos) = l.relics.iter().position(|&r| r == material) else {
                return;
            };
            l.relics.remove(pos);
            l.equipped.retain(|&r| r != material);
            l.relic_lv[target.min(7)] += 1;
            l.relic_lv[target.min(7)]
        };
        self.log(
            format!(
                "以【{}】为引炼化【{}】，灵纹进境 +{} 级。",
                m_name, t_name, new_lv
            ),
            C_GOLD,
        );
        self.play(crate::sounds::Which::Ascend);
        self.sync_save();
    }

    // 现实寻仇

    pub fn can_revenge(&self) -> bool {
        self.life.is_some() && self.meta.neme.is_some()
    }

    // 实力比 r = 我方战力 / 宿敌战力——胜率与禁手判定同源口径，禁双写
    fn revenge_ratio(&self) -> f64 {
        match (&self.life, &self.meta.neme) {
            (Some(l), Some(n)) => power(l) / n.power().max(1.0),
            _ => 0.0,
        }
    }

    // 比率→成功率。下限 0.05→0.0（寻仇必成修复）：原 5% 保底 + 失败不禁手，
    // 连点 N 次必成（1−0.95^N，90 次≈99%）；现实力悬殊者胜率归 0，
    // 底栏/人物页胜率显示自动跟随。上限 0.95 不变；模拟侧决战反杀 / sim_fight 事件掷 /
    // 跨境破境的 clamp(0.05,0.95) 语义各异（一次性抉择不可连刷），勿联动改。
    fn revenge_p_of(r: f64) -> f32 {
        (r * r / (1.0 + r * r)).clamp(0.0, 0.95) as f32
    }

    pub fn revenge_p(&self) -> f32 {
        Self::revenge_p_of(self.revenge_ratio())
    }

    // 寻仇可出手单一谓词（draw/world 两入口 UI 可用性 + act_revenge 卫语句同口）：
    // 未重伤（失败 injured=5，每年 −1 自然冷却；洞府闭关/疗伤丹可清）
    // 且实力比 ≥ 0.1（宿敌战力超自身 10 倍即禁手）。
    pub fn revenge_ready(&self) -> bool {
        self.can_revenge() && self.life.as_ref().unwrap().injured == 0 && self.revenge_ratio() >= 0.1
    }

    // 禁用态副标题（前置：can_revenge() 且 !revenge_ready()；与「宿敌已清 · 静待新敌」同款）
    pub fn revenge_disabled_text(&self) -> &'static str {
        if self.life.as_ref().map_or(false, |l| l.injured > 0) {
            "重伤未愈 · 静待恢复"
        } else {
            "实力悬殊 · 莫去送死"
        }
    }

    pub fn act_revenge(&mut self) {
        // 卫语句走 revenge_ready 单口：重伤/悬殊禁手态不结算，防绕过 UI 直调
        if !self.revenge_ready() {
            return;
        }

        let p = self.revenge_p();

        let neme = self.meta.neme.clone().unwrap();

        let win = gen_range(0.0, 1.0) < p;

        if win {
            // v1.6.0：胜利分支抽为 revenge_win——结构性必需：debug_win_revenge 需复用
            // 胜利分支（否则复制 45 行）；非借用修复（借用检查流敏感，
            // else 臂继续使用 neme 合法，nfull 提取不做、else 臂零改动）
            self.revenge_win(neme);
        } else {
            let l = self.life.as_mut().unwrap();

            l.injured = 5;

            l.qi *= 0.7;

            self.log(
                format!(
                    "你寻仇{}，不敌而遁，身受重伤！修炼无岁月，且先变强。",
                    neme.full()
                ),
                C_RED,
            );

            self.play(crate::sounds::Which::BreakFail);
        }

        self.sync_save();
    }

    // 寻仇胜利分支（act_revenge 抽取；debug_win_revenge 复用）：
    // 奖励结算 → 宿敌递进 → spawn_next_neme→advance_map 换幕链
    fn revenge_win(&mut self, neme: Nemes) {
        let reward_stones = (4000.0 * self.stone_scale(neme.realm)) as i64;

        let reward_dy = 50 * (1 + neme.realm as i64);

        {
            let l = self.life.as_mut().unwrap();

            l.stones += reward_stones;

            l.wins += 1;

            l.highlights
                .push(format!("{}岁，现实斩杀宿敌{}", l.age as i32, neme.name));
        }

        self.meta.dao_yun += reward_dy;

        self.meta.sim_points += 2;

        self.meta.neme_slain += 1;

        self.log(
            format!("血仇得报！你于现实中阵斩宿敌{}！", neme.full()),
            C_GOLD,
        );

        self.show_banner("血 仇 得 报", C_GOLD);

        self.log(
            format!(
                "夺其家产：灵石 {}，道韵+{}，模拟点数+2。",
                fmt_int(reward_stones),
                reward_dy
            ),
            C_GOLD,
        );

        self.spawn_next_neme();

        self.play(crate::sounds::Which::Ascend);

        self.burst(80, true);

        self.check_ach_all();

        // v1.16.8 章节书：首斩宿敌事实翻转点（第三章边沿；讨牒边沿在 sync_taodie 口）
        self.check_chapters();
    }

    // 冒烟专用（GF_DEBUG）：直接走寻仇胜利路径，覆盖 spawn_next_neme→advance_map 换图链。
    pub fn debug_win_revenge(&mut self) {
        if let Some(n) = self.meta.neme.take() {
            self.revenge_win(n);
        }
    }

    pub fn eat_pill(&mut self, id: usize) {
        let can = self.life.as_ref().map_or(false, |l| l.pills[id] > 0);

        if !can {
            return;
        }

        let boost = self.life.as_ref().map(|l| {
            let mut b = teff(l).pill_mult.max(1.0);
            // v1.10.2 成行择道：丹修（path=3）服丹效果 ×1.35（单点；模拟侧 teff 路径 path 恒 0）
            if l.path == PATH_PILL {
                b *= 1.35;
            }
            b
        })
        .unwrap_or(1.0);

        let msg = {
            let l = self.life.as_mut().unwrap();

            l.pills[id] -= 1;

            match id {
                0 => {
                    l.qi += qi_need(l) * 0.06 * boost;

                    "服下聚气丹。（修为+6%）".to_string()
                }

                3 => {
                    l.lifespan += 15.0;

                    l.injured = 0;

                    "服下培元丹，寿元+15，伤势尽复。".to_string()
                }

                4 => {
                    l.root = (l.root + 2.0).min(100.0);

                    l.washed += 1;

                    "服下洗髓丹，灵根+2！".to_string()
                }

                _ => String::new(),
            }
        };

        if id == 4 {
            let washed = self.life.as_ref().map(|l| l.washed).unwrap_or(0);

            if washed >= 10 {
                self.check_ach(17);
            }
        }

        if !msg.is_empty() {
            self.log(msg, C_GREEN);

            self.play(crate::sounds::Which::Gain);
        }

        self.sync_save();
    }

    pub fn tech_price(&self, idx: usize) -> i64 {
        let realm = self.life.as_ref().map(|l| l.realm).unwrap_or(0);

        ((TECH_PRICES[idx] as f64) * (1.0 + realm as f64).powf(1.1)).ceil() as i64
    }

    pub fn buy_tech(&mut self, idx: usize) {
        let Some(l) = self.life.as_ref() else { return };

        if idx <= l.tech {
            self.toast("已修至更高", C_DIM);

            return;
        }

        let price = self.tech_price(idx);

        if l.stones < price {
            self.toast("灵石不足", C_RED);

            return;
        }

        let name = TECHS[idx].name;

        {
            let l = self.life.as_mut().unwrap();

            l.stones -= price;

            l.tech = idx;
        }

        self.log(
            format!("你于藏经阁重金求得《{}》，闭关参悟三日而通！", name),
            C_GOLD,
        );

        self.play(crate::sounds::Which::Ascend);

        self.sync_save();
    }

    pub fn buy_pill(&mut self, id: usize) {
        let price = self.pill_price(id);

        let can = self.life.as_ref().map_or(false, |l| l.stones >= price);

        if !can {
            self.toast("灵石不足", C_RED);

            return;
        }

        let name = PILLS[id].name;

        {
            let l = self.life.as_mut().unwrap();

            l.stones -= price;

            l.pills[id] += 1;
        }

        // v1.6.0 D：购丹积攒掌柜好感
        crate::npc::add_fav(self, crate::npc::NPC_MERCHANT, MERCHANT_BUY_FAV);

        self.log(
            format!("购得{}一枚（{}灵石）。", name, fmt_int(price)),
            C_CYAN,
        );

        self.play(crate::sounds::Which::Coin);

        self.sync_save();
    }

    pub fn pill_price(&self, id: usize) -> i64 {
        let realm = self.life.as_ref().map(|l| l.realm).unwrap_or(0);

        let disc = self
            .life
            .as_ref()
            .map_or(false, |l| teff(l).pill_mult >= 2.0);

        let d = if disc { 0.8 } else { 1.0 };

        // 山河图志：身在临仙坊市，丹药九折
        let d = if self.life.as_ref().is_some_and(|l| l.loc == NODE_MARKET) {
            d * MARKET_PILL_DISC
        } else {
            d
        };

        // v1.5.0 洞府经营：丹房炉火折扣
        let d = d * DANFANG_DISC[self.meta.dongfu[2].clamp(0, 3) as usize];

        // v1.6.0 D：坊市掌柜好感折扣
        let d = d * crate::npc::merchant_disc(self.meta.npc_fav[crate::npc::NPC_MERCHANT]);

        ((PILL_PRICES[id] as f64) * (1.0 + realm as f64).powf(1.2) * d).ceil() as i64
    }

    // ================= 轮回烙印 =================

    pub fn buy_upgrade(&mut self, id: usize) {
        let lv = self.meta.upg[id];

        if lv >= UPGRADES[id].max {
            return;
        }

        let cost = upg_cost(id, lv);

        if self.meta.dao_yun < cost {
            self.toast("道韵不足", C_RED);

            return;
        }

        self.meta.dao_yun -= cost;

        self.meta.upg[id] += 1;

        let name = UPGRADES[id].name;

        let nlv = self.meta.upg[id];

        self.log(format!("轮回烙印加深：{} 升至{}级。", name, nlv), C_GOLD);

        self.play(crate::sounds::Which::Ascend);

        self.sync_save();
    }

    // ================= 成就 =================

    pub fn check_ach(&mut self, id: usize) {
        if id >= self.meta.ach.len() || self.meta.ach[id] {
            return;
        }

        self.meta.ach[id] = true;

        let reward = ACHS[id].reward;

        let name = ACHS[id].name;

        self.meta.dao_yun += reward;

        self.log(
            format!("达成成就【{}】！道韵+{}", name, fmt_int(reward)),
            C_GOLD,
        );

        self.toast(format!("成就：{}", name), C_GOLD);

        self.play(crate::sounds::Which::Ding);
    }

    pub fn check_ach_all(&mut self) {
        let Some(l) = &self.life else { return };

        let (mr, age, stones) = (l.max_realm, l.age, l.stones);

        let pw = power(l);

        let sect = l.sect;

        let washed = l.washed;

        let natural = l.natural_end;

        let lives = self.meta.lives;
        let comp_lives = self.meta.companion_lives;

        let self_neme_slain = self.meta.neme_slain;

        let checks: [(usize, bool); 30] = [
            (0, mr >= 1),
            (1, mr >= 2),
            (2, mr >= 3),
            (3, mr >= 4),
            (4, mr >= 5),
            (5, mr >= 6),
            (6, mr >= 7),
            (7, mr >= 8),
            (8, mr >= 9),
            (9, mr >= 10),
            (10, mr >= 11),
            (11, mr >= 13),
            (12, age >= 100000.0),
            (13, lives >= 10),
            (14, lives >= 100),
            (15, stones >= 1_000_000),
            (16, pw >= 1e9),
            (17, washed >= 10),
            (18, self_neme_slain >= 1),
            (19, age >= 1000.0),
            (20, sect && mr >= 3),
            (21, natural && mr >= 4),
            (22, comp_lives >= 1),
            (23, false),
            (24, false),
            (25, false),
            (26, comp_lives >= 3),
            (27, l.realm >= 10),
            (28, stones >= 100_000_000),
            (29, age >= 5000.0),
        ];

        for (id, ok) in checks {
            if ok {
                self.check_ach(id);
            }
        }
    }

    // ================= v1.16.8 章节书：主线章节达成单口（v1.16.10 增第九章·书外之章） =================
    // 谓词唯一实现（data.rs CHAPTERS 的 pid 全部走此 match 分发，禁第二实现；
    // 照 check_ach_all 的 (id,bool) 元组表先例）。只读 meta 标量、零分配——
    // 存档载入帧静默补齐与每帧章节书视图共用同一口，谓词语义不漂移。
    // 索引纪律：first_realm/region 走 get().copied().unwrap_or(false)，绝不下标裸访
    // （panic=abort；CHAPTERS 为静态表 pid 恒在 0..=15，脏 pid 落 _ 臂 false）。
    pub fn chapter_goal_met(m: &Meta, pid: u8) -> bool {
        match pid {
            0 => m.total_sims >= 1,                    // 开卷首模拟
            1 => m.total_wins >= 1,                    // 首胜（模拟战退强敌）
            2 => m.first_realm.get(1).copied().unwrap_or(false) || m.best_realm >= 1, // 现实首破境
            3 => m.neme_slain >= 1,                    // 首斩宿敌
            4 => m.taodie >= 3,                        // 讨牒三成
            5 => m.taodie >= 6,                        // 讨牒 6/6
            6 => m.region_visited.iter().filter(|&&x| x).count() >= 3, // 三域已至
            7 => m.region_visited.iter().filter(|&&x| x).count() >= 5, // 五域全至
            8 => m.companion_lives >= 1,               // 首结缘
            9 => m.npc_ev.iter().any(|&x| x),          // 好感事件已抉择
            10 => m.first_realm.get(10).copied().unwrap_or(false) || m.best_realm >= 10, // 现实首登仙界（问笔）
            11 => m.endings_seen != 0,                 // 落款成书（任一结局）
            12 => m.lives >= 1,                        // 转生入世（v1.16.8 复审修：meta.life
                                                       // 快照在菜单/身故落 None（sync_save
                                                       // 覆写口径 game.rs:1447），结算屏
                                                       // chapter_current 会推不出第一章位、
                                                       // 旧档载入补不齐——改挂单调持久的
                                                       // lives 计数（begin_mortal:1579 自增））
            13 => m.endings_seen == 0b1111,            // 四结局全览
            14 => m.neme_slain >= 2,                   // 再斩一敌
            15 => m.best_realm >= 13,                  // v1.16.10 书外之章：历代最高修为抵执卷境
            _ => false,
        }
    }

    /// 章节目标进度数值单口（v1.16.9 章节书目标行右侧 x/y 显示用）。与 chapter_goal_met
    /// 平行分发表（谓词布尔仍只以 goal_met 为准——本表只派生展示数值，met 判定不读本
    /// 表；同口纪律见上）。v1.16.16 全 pid 数值化（0..=15 全有值）：布尔型目标以 0/1
    /// 两态呈现（开卷/首胜/首结缘/故人心事/问笔/落款/转生），计数型给真实分母
    ///（结局 4、讨牒 3/6、到域 3/5、斩敌 1/2），境界型给 best_realm 渐进分母
    ///（首破境 1、问笔 10、执卷 13——pid2/10 的 first_realm 位快照可能领先
    /// best_realm（death 才回填，real_die 口），故 x=max(best, 位快照折算) 保
    /// 「x≥y ⟺ met」同口不漂）。零分配（返回标量元组），帧路径安全；单测钉死
    /// Some((x,y)) ⟹ x≤y 与 met==（x≥y）同口一致。
    pub fn chapter_goal_prog(m: &Meta, pid: u8) -> Option<(u32, u32)> {
        match pid {
            0 => Some((m.total_sims.min(1), 1)),       // 开卷首模拟
            1 => Some((m.total_wins.min(1), 1)),       // 首胜（模拟战退强敌）
            2 => Some((
                m.best_realm
                    .max(if m.first_realm.get(1).copied().unwrap_or(false) {
                        1
                    } else {
                        0
                    })
                    .min(1) as u32,
                1,
           )),                                        // 现实首破境
            3 => Some((m.neme_slain.min(1), 1)),       // 首斩宿敌
            4 => Some((m.taodie.min(3) as u32, 3)),    // 讨牒三成
            5 => Some((m.taodie.min(6) as u32, 6)),    // 讨牒 6/6
            6 => Some((
                m.region_visited.iter().filter(|&&x| x).count() as u32,
                3,
            )),                                        // 三域已至
            7 => Some((
                m.region_visited.iter().filter(|&&x| x).count() as u32,
                5,
            )),                                        // 五域全至
            8 => Some((m.companion_lives.min(1), 1)),  // 首结缘
            9 => Some((
                m.npc_ev.iter().filter(|&&x| x).count().min(1) as u32,
                1,
            )),                                        // 好感事件已抉择
            10 => Some((
                m.best_realm
                    .max(if m.first_realm.get(10).copied().unwrap_or(false) {
                        10
                    } else {
                        0
                    })
                    .min(10) as u32,
                10,
            )),                                        // 现实首登仙界（问笔）
            11 => Some((m.endings_seen.count_ones().min(1), 1)), // 落款成书
            12 => Some((m.lives.min(1), 1)),           // 转生入世
            13 => Some((m.endings_seen.count_ones().min(4), 4)), // 四结局全览
            14 => Some((m.neme_slain.min(2), 2)),      // 再斩一敌
            15 => Some((m.best_realm.min(13) as u32, 13)), // 执卷境（书外之章）
            _ => None,
        }
    }

    /// 章节达成位掩码（章 i 全目标即中 bit i）。载入帧据此静默补齐旧档
    /// （老玩家补齐不重播横幅），check_chapters 据此做边沿演出。
    pub fn chapter_done_mask(m: &Meta) -> u32 {
        let mut mask = 0u32;
        for (i, ch) in crate::data::CHAPTERS.iter().enumerate() {
            if ch.goals
                .iter()
                .all(|gl| Self::chapter_goal_met(m, gl.pid))
            {
                mask |= 1u32 << i;
            }
        }
        mask
    }

    /// 章节达成边沿演出 + 章成之礼发放（v1.16.9）：newly 位命中处 meta.dao_yun +=
    /// ch.reward——发放与演出同处边沿，chap_done 位即领取凭证，幂等零补发（载入帧
    /// 静默补齐的老玩家不补发，属既定取舍）。只发道韵（meta 跨世持久标量）：本函数
    /// 可能 life=None 帧被调（寻仇/抉择口），禁发灵石等现世资源。同帧多章全了时横幅
    /// 单槽后写胜——奖励数字只走逐章金行 log 防吞，横幅维持章名。不主动 sync_save
    /// （调用点均在既有落盘流程内，零额外落盘）。
    pub fn check_chapters(&mut self) {
        let done = Self::chapter_done_mask(&self.meta);
        let newly = done & !self.meta.chap_done;
        if newly == 0 {
            return;
        }
        self.meta.chap_done |= newly;
        for (i, ch) in crate::data::CHAPTERS.iter().enumerate() {
            if newly >> i & 1 == 1 {
                self.meta.dao_yun += ch.reward as i64;
                // v1.16.19 章成金行副句按章查表 DONE_NOTE[i]（每章一句，替代九连
                // 「主线又进一程」；reward_note 同行保留）。字形事件帧补灌（金行
                // 是动态 format! 串，日志区绘制档见 slog/16px——防冷栅格化整图重传）
                crate::ui::prewarm_text(DONE_NOTE[i.min(DONE_NOTE.len() - 1)], &[14]);
                crate::ui::prewarm_text(DONE_NOTE[i.min(DONE_NOTE.len() - 1)], &[16]);
                self.log(
                    format!(
                        "【{}·了】{}——{}。{}",
                        ch.num,
                        ch.name,
                        DONE_NOTE[i.min(DONE_NOTE.len() - 1)],
                        ch.reward_note
                    ),
                    C_GOLD,
                );
                self.show_banner(format!("{} · {}", ch.num, ch.name), C_GOLD);
                // v1.16.10 章节称号演出：首章（bit0）边沿附一句「可佩」指路——称号
                // earned 全由 chap_done 纯推导（成就·称号页格 17），此处只演出不记账。
                // toast 单槽后写胜：同帧多章全了时被后章横幅帧覆盖属既定语义（金行
                // log 已逐章兜底信息）。
                // v1.16.16 章节书初见指路（sy= bit6，占用空闲位 additive 无新键）：
                // 首章了边沿补一句「任务·章节书签」指路。承载改走金行 log 而非第二句
                // toast——toast 单槽后写胜会吞掉同边沿的称号句；log 17 行常驻可回看，
                // 与边沿横幅（单槽）互补。bit6 已置（防御位，理论不可达——chap_done
                // 位单调）不再复读。
                if i == 0 {
                    self.toast("称号可佩：雨夜执印者（角色·成就称号页）", C_GOLD);
                    if self.meta.sys_seen & 0b1000000 == 0 {
                        self.meta.sys_seen |= 0b1000000;
                        self.log(
                            "主线有册：顶栏「任务」钮可开「章节书」签——九章目标与进度尽在册。",
                            C_GOLD,
                        );
                    }
                }
                // v1.16.20 章开场白（故事沉浸续作）：章 i 了边沿追发下一章「启」句金行
                // ——同帧多章全了时「了+启」逐章成对（循环内天然成对）；末章（章9）
                // 了后无下一章不发；章1 开局已有 intro 且无前边沿（首行 open 恒不演
                // 出，保非空供表契约单测钉死）。金行走 check_chapters 事件帧 format!
                // 豁免（照上方了句）；字形随行事件帧 prewarm（日志金行绘制档 14/16
                // ——防冷栅格化整图重传，照 DONE_NOTE 同款）。
                if i + 1 < crate::data::CHAPTERS.len() {
                    let nx = &crate::data::CHAPTERS[i + 1];
                    crate::ui::prewarm_text(nx.open, &[14]);
                    crate::ui::prewarm_text(nx.open, &[16]);
                    self.log(format!("【{}·启】{}", nx.num, nx.open), C_GOLD);
                }
            }
        }
    }

    /// 当前章（首个未了章）下标：全了返回末章（章节书右栏恒有内容可看）。
    pub fn chapter_current(m: &Meta) -> usize {
        let done = Self::chapter_done_mask(m);
        (0..crate::data::CHAPTERS.len())
            .find(|&i| done >> i & 1 == 0)
            .unwrap_or(crate::data::CHAPTERS.len() - 1)
    }

    // ================= 现实死亡 → 总结算 =================

    pub fn real_die(&mut self, cause: &str) {
        let (max_realm, age, stones, wins, new_high, natural, bp) = {
            let l = self.life.as_ref().unwrap();

            (
                l.max_realm,
                l.age,
                l.stones,
                l.wins,
                l.new_high,
                l.natural_end,
                power(l),
            )
        };

        self.play(crate::sounds::Which::Death);

        self.shake = 18.0;

        let mut lines: Vec<(String, i64)> = Vec::new();

        let base = MILESTONE[max_realm.min(14)];

        if base > 0 {
            let b = if new_high { base * 2 } else { base };

            lines.push((
                format!(
                    "{}境道韵{}",
                    REALMS[max_realm.min(14)].name,
                    if new_high { "（首登×2）" } else { "" }
                ),
                b,
            ));
        }

        let age_dy = (age / 40.0) as i64;

        if age_dy > 0 {
            lines.push((format!("享年{}载", fmt_num(age)), age_dy));
        }

        let st = (stones / 2000) as i64;

        if st > 0 {
            lines.push((format!("遗留灵石{}", fmt_int(stones)), st));
        }

        let w = (wins / 2) as i64;

        if w > 0 {
            lines.push((format!("斩敌{}", wins), w));
        }

        if natural && base > 0 {
            lines.push(("寿终正寝（+10%）".to_string(), base / 10));
        }

        let total_now: i64 = lines.iter().map(|(_, v)| *v).sum();

        self.meta.dao_yun += total_now;

        let total = total_now;

        self.meta.best_realm = self.meta.best_realm.max(max_realm);

        self.meta.best_age = self.meta.best_age.max(age);

        self.meta.best_power = self.meta.best_power.max(bp);

        let feng_cn = self.life.as_ref().and_then(|l| l.companion.clone());
        self.feng_bond_up(feng_cn.as_deref()); // v1.12.1 缝页：现实同页至终（含遗泽）缘+1
        // v1.16.20 立绘沉浸续作②：结算屏谢幕像快照（life 清空前定格 companion 账
        // 终值——与结局页谢幕像/随行小像同源推导语言）
        let comp_idx = feng_cn
            .as_deref()
            .and_then(|cn| COMPANION_NAMES.iter().position(|&n| n == cn));

        self.life = None;

        self.modal = None;

        self.panel = 0;

        // v1.16.16 结算屏「下一章」行字形预热（事件帧一次，照 build_event 尾先例）：
        // 章号/章名 14/16 + 首未了目标@14 + 章成之礼行@14 + 行右进度串（数值化口径
        // 与 chapter_goal_prog 同口派生）——防 settle 首帧冷栅格化触发 2048² 图集
        // 整图重传（draw.rs:3426 事故同款教训；settle 屏旧文案本就在此帧路径外首绘，
        // 本预热只补新增字形）
        // v1.16.18 清账①：主线全了时 draw.rs 下一章行走字面量收束行「主线已了——
        // 十万稿废纸，终有一册」@14（find 未了目标为 None → unwrap_or 兜底），本块
        // 原只灌 goals 文本未灌兜底串、PANEL_WARM_CHARS 亦缺「线十万稿废纸终册」
        // 等字——补一行无条件预热（事件帧一次，纯增益；不动每帧绘制路径）
        {
            let cur = Self::chapter_current(&self.meta);
            let ch = &crate::data::CHAPTERS[cur];
            crate::ui::prewarm_text(ch.num, &[14]);
            crate::ui::prewarm_text(ch.name, &[14, 16]);
            crate::ui::prewarm_text(ch.reward_note, &[14]);
            for gl in ch.goals.iter() {
                crate::ui::prewarm_text(gl.text, &[14]);
                if let Some((x, n)) = Self::chapter_goal_prog(&self.meta, gl.pid) {
                    crate::ui::prewarm_text(&format!("{}/{}", x, n), &[14]);
                }
            }
            crate::ui::prewarm_text("主线已了——十万稿废纸，终有一册", &[14]);
            // v1.16.19 结算收束语同块补灌（按身故境界档查表静态串 @14——漏灌则
            // settle 首帧冷栅格化触发 2048² 图集整图重传，同款教训）
            crate::ui::prewarm_text(settle_coda(max_realm), &[14]);
        }

        self.settle = Some(SettleInfo {
            cause: cause.to_string(),

            lines,

            total,

            coda: settle_coda(max_realm),

            comp: comp_idx,
        });

        self.scene = Scene::Settle;

        self.sync_save();
    }

    pub fn burst(&mut self, n: usize, gold: bool) {
        let (cx, cy) = if self.life.is_some() {
            (180.0, 168.0)
        } else {
            (V_W * 0.5, V_H * 0.5)
        };

        for _ in 0..n {
            let a = gen_range(0.0, std::f32::consts::TAU);

            let sp = gen_range(30.0, 220.0);

            self.sparks.push(Spark {
                x: cx,

                y: cy,

                vx: a.cos() * sp,

                vy: a.sin() * sp - 60.0,

                life: gen_range(0.5, 1.4),

                max: 1.4,

                gold,
            });
        }
    }
}

// v1.12.1 终卷·四结局（未竟稿 §9 原文摘句，下标=endings_seen 位 0..=3）：
// 演出页/书志页共用文案表；B《留白》为定稿推荐结局（织机声行为定稿原文通用句）。
// v1.16.15 结局之礼：结局首录奖励（道韵）。位即凭证零新键（endings_seen ed=
// 已存）；标定照章成之礼 50..500 区间（data.rs CHAPTERS.reward，一章 50..九章
// 500；勿与 ACHS 成就奖励 10..3000 序列混淆）取整 200——
// 问笔终战门槛=讨牒 6/6+九重雷劫+封卷吏，单结局 200 与章成之礼量级持平；
// 全收集不发额外道韵（四结局之礼已足额），礼遇走称号『执笔人』授权
pub const ENDING_DY: [i64; 4] = [200, 200, 200, 200];

pub struct EndingDef {
    pub name: &'static str, // 结局名（横幅/书志行）
    pub alias: &'static str, // 抉择别名（认写/撕稿/夺笔/收册人）
    pub body: &'static str, // 演出页正文（\n 分段）
    pub gain: &'static str, // 达成行【结局达成/隐藏成就：…】
}

pub const ENDINGS: [EndingDef; 4] = [
    EndingDef {
        name: "定稿",
        alias: "认写",
        body: "你赢了封卷吏。执笔人出现，递来笔：「字不错。入正文吧。从此你的每一行都是天定——天定，也就永不动删。」\n你看了很久那支笔。\n「我要是入了正文，」你问，「还能改自己吗？」\n「一字不改。永生换的。」",
        gain: "【结局达成：正文第一行 · 陈砚】\n【代价：草稿权作废。你不能重来，你成了别人不能重来的理由之一】",
    },
    EndingDef {
        name: "留白",
        alias: "撕稿",
        body: "你当着执笔人的面，把自己的书撕了。\n琅函大乱——未写完的书，天道无权合卷。你的书从此只剩一页一行：\n「某年某月，青牛镇外，陈砚生。后事不详。」\n后事不详。四个字，比永生更宽。\n你回到田里。同页的织机声，从三百里外传来。",
        gain: "【隐藏成就：后事不详 —— 一行之外，谁也写不了你】",
    },
    EndingDef {
        name: "执笔",
        alias: "夺笔",
        body: "你接了那支笔。\n第一件事，把十万层废稿一层层起出来，每稿的名字，一笔一笔描清楚。\n最后一行，你写给下一个雨夜：\n「某年某月，某地，有人捡到一枚发烫的方印。印里有个声音，累得不像少年人。」",
        gain: "【隐藏成就：琅函天笔 —— 你成了别人「重来一次」的源头】",
    },
    EndingDef {
        name: "合订",
        alias: "收册人",
        body: "收册人入库时发现一册怪书：十万张废稿，自己订成了一册，书脊上无字，只有页脚一行小字，岁岁多一行——\n「未完。」",
        gain: "【隐藏成就：会自己长出新行的书】",
    },
];

pub const NEME_TITLES: [&str; 6] = [
    "断章",
    "撕页",
    "朱批",
    "删骨",
    "夺砚",
    "封卷",
];

pub fn rand_name() -> String {
    const SURNAMES: [&str; 8] = ["厉", "萧", "墨", "鬼", "血", "殷", "聂", "皇甫"];

    const GIVEN: [&str; 10] = [
        "无咎", "天殇", "屠苍", "九幽", "绝尘", "问天", "灭", "寒渊", "荒", "玄煞",
    ];

    format!(
        "{}{}",
        SURNAMES[gen_range(0, SURNAMES.len())],
        GIVEN[gen_range(0, GIVEN.len())]
    )
}

// 继承选项

pub struct InheritDef {
    pub name: &'static str,

    pub desc: &'static str,
}

    pub const INHERITS: [InheritDef; 9] = [
    InheritDef {
        name: "继承修为",

        desc: "现实修为 +模拟一生所得的五成",
    },
    InheritDef {
        name: "继承道果",

        desc: "道心 +5，体魄 +5（永久）",
    },
    InheritDef {
        name: "继承功法",

        desc: "习得模拟中修成的功法",
    },
    InheritDef {
        name: "继承法宝",

        desc: "取回模拟中获得的法宝",
    },
    InheritDef {
        name: "继承财货",

        desc: "取回模拟中灵石的一半",
    },
    InheritDef {
        name: "继承丹药",

        desc: "模拟中丹药尽数带回",
    },
    InheritDef {
        name: "继承寿数",

        desc: "现实寿元 +25 载",
    },
    InheritDef {
        name: "继承气运",

        desc: "现实气运 +3（永久）",
    },
    // v1.15.1 真根因修复：道侣继承项（索引 8）。take_inherit 的 8 =>「道侣遗泽」
    // 分支与 sim 端 opts.push(8)/pool.retain(!=8) 都按 9 项表编写，但表条目缺失
    // → 模拟死亡时有道侣 → draw INHERITS[8] 越界 → panic=abort 下主循环体逐帧
    // 重 panic（异步 future 重入同一循环体）= 永久楔死。真机实证：模拟中按跳过
    // /随机选项后 panic 风暴、"already borrowed" 刷屏、触摸全灭、按按钮毫无反应。
    InheritDef {
        name: "道侣遗泽",

        desc: "道侣随神魂归现实：道心 +8，气运 +8（永久）",
    },
];

// ---------------- 寻仇必成修复回归：下限归 0 + 重伤/悬殊禁手 ----------------

#[cfg(test)]
pub(crate) mod revenge_tests {
    use super::*;

    // 免音频初始化与存档 IO 的最小 Game：仅寻仇路径所需字段
    // v1.16.0：pub(crate) 供同文件 guide_tests 矩阵复用（构造器零分叉）
    pub(crate) fn game_with(l: Life, neme: Option<Nemes>) -> Game {
        let mut meta = Meta::default();

        meta.neme = neme;

        Game {
            scene: Scene::Menu,

            meta,

            life: Some(l),

            sim: None,

            sim_start_age: 0.0,

            sim_log: Vec::new(),

            sim_result: None,

            inherit_opts: Vec::new(),

            inherit_picked: 0,

            draft: Vec::new(),

            blessed: false,

            picked: Vec::new(),

            log: Vec::new(),

            modal: None,

            panel: 0,

            char_tab: 0,

            npc_scroll: 0.0,
            npc_drag: None,
            npc_vel: 0.0,
            npc_anim: None,
            tier_flash: 0.0,

            save_dirty: false,
            save_timer: 0.0,
            prev_sig: (0, 0, false, false, false, false, false, false),

            auto_acc: 0.0,

            auto_years: 0,

            auto_gain: 0.0,

            auto_toast_done: false,

            tab: 0,

            help_tab: 0,

            buy_pending: None,

            rmodal: None,

            rmodal_npc: None,

            refine_pending: None,

            modal_desc: None,

            shop_tab: 0,

            world: crate::world::WorldState::new(),

            submenu: None,

            bag_tab: 0,

            ad: None,

            sac_pending: None,

            sub_fresh: false,

            guide_last: None,
            guide_chap_last: None,

            guide_tab: 0,

            zoom_art: None,

            modal_min: false,
            modal_anim: 0.0,
            ending_anim: 0.0,

            speed: 0,

            acc: 0.0,

            motes: Vec::new(),

            meteors: Vec::new(),

            sparks: Vec::new(),

            shake: 0.0,

            toast: None,

            banner: None,

            settle: None,

            offline_card: None,

            ending_show: None,

            book_view: None,

            blank_used: false,

            recent_events: Vec::new(),

            chain: None,

            chain6_closed: false,
            chain_latch: 0,

            ev_memo: vec![None; EVENTS.len()],

            ev_recall: None,

            secl_yrs: 0,

            secl_qi: 0.0,

            exp_p_cache: None,

            run_events: Vec::new(),
            run_recall_open: false,
            run_recall_scroll: 0.0,
            run_recall_drag: None,

            time: 0.0,

            sfx_ok: None,

            audio: crate::sounds::Audio {
                s: None,
                music_on: false,
                sfx_on: false,
                music_vol: 0.8,
                sfx_vol: 0.8,
            },
        }
    }

    pub(crate) fn neme(realm: usize) -> Nemes {
        Nemes {
            title: "魔修",
            name: "试炼者".to_string(),
            realm,
        }
    }

    // ① r→0：胜率归 0（原 clamp 下限 0.05 = 弱者恒 5% 保底，连点 N 次必成之根）
    #[test]
    fn underdog_rate_floors_at_zero() {
        // 白丁（战力 5.0×1.2）对天笔宿敌（1.5e10×1.25）：比率 ~3e-10，胜率应近 0 而非 5%
        let g = game_with(Life::default(), Some(neme(14)));

        assert!(g.can_revenge());

        assert!(
            g.revenge_p() < 1.0e-9,
            "弱者胜率应归 0，实得 {}",
            g.revenge_p()
        );

        // 比率恰 0 的下界：clamp(0.0,…) 下公式值恰为 0.0
        assert_eq!(Game::revenge_p_of(0.0), 0.0);
    }

    // ② 对照：r=1 → 50%（公式本身未动只动下限，正常对局胜率不受影响）
    #[test]
    fn equal_power_rate_untouched() {
        let p = Game::revenge_p_of(1.0);

        assert!((0.49..=0.51).contains(&p), "r=1 应得 50%，实得 {}", p);
    }

    // ③ 重伤禁手：injured>0 时 revenge_ready()==false，卫语句令 act_revenge 空转（不结算不掉档）
    #[test]
    fn injured_blocks_revenge() {
        let mut l = Life::default();

        l.injured = 5;

        let mut g = game_with(l, Some(neme(0))); // 同境未伤本可出手（r≈0.96）

        assert!(!g.revenge_ready());

        assert_eq!(g.revenge_disabled_text(), "重伤未愈 · 静待恢复");

        g.act_revenge(); // 卫语句路径：无 IO 无结算

        assert_eq!(g.log.len(), 0);

        assert_eq!(g.life.as_ref().unwrap().injured, 5);
    }

    // ④ 实力比 ≥ 0.1 边界：悬殊禁手 vs 同境可出手
    #[test]
    fn power_ratio_gates_ready() {
        // 悬殊（宿敌战力 > 10 倍自身）：禁手
        let g = game_with(Life::default(), Some(neme(14)));

        assert!(!g.revenge_ready());

        assert_eq!(g.revenge_disabled_text(), "实力悬殊 · 莫去送死");

        // 同境（r = 6.0 / 6.25 = 0.96）：可出手，胜率约 48%
        let g2 = game_with(Life::default(), Some(neme(0)));

        assert!((g2.revenge_ratio() - 0.96).abs() < 1e-6);

        assert!(g2.revenge_ready());

        assert!((0.45..=0.5).contains(&g2.revenge_p()));
    }

    // ⑤ 周天吐纳：门控与冻结语义（开关关不累计 / 弹层期冻结不清零 / 非 Reality 不累计；
    // 全程不触发 act_cultivate——那会 sync_save 落盘，单测禁 IO）
    #[test]
    fn zt_auto_gates_and_freeze() {
        let mut g = game_with(Life::default(), None);

        g.scene = Scene::Reality;

        // 开关关（默认 0）：不累计
        assert_eq!(g.meta.auto, 0);

        g.tick_auto(AUTO_CULT_SECS * 10.0);

        assert_eq!(g.auto_acc, 0.0);

        // 开关开：正常累计
        g.meta.auto = 1;

        g.tick_auto(1.0);

        assert!((g.auto_acc - 1.0).abs() < 1e-6);

        // 弹层期冻结不清零
        g.submenu = Some(crate::menu::Sub::Cult);

        g.tick_auto(AUTO_CULT_SECS * 10.0);

        assert!((g.auto_acc - 1.0).abs() < 1e-6);

        g.submenu = None;

        // 非 Reality 场景不累计
        g.scene = Scene::World;

        g.tick_auto(AUTO_CULT_SECS * 10.0);

        assert!((g.auto_acc - 1.0).abs() < 1e-6);
    }

    // ⑥ 宿敌追缉：上限 cap=min(14,best+5) 与绝顶不越（advance_time 无落盘可循环；
    // 600 次×6% 期望 36 次成长，达 cap 概率≈1，尾风险可忽略）
    #[test]
    fn neme_growth_capped() {
        let mut g = game_with(Life::default(), Some(neme(5)));

        g.meta.best_realm = 3; // cap = min(14, 3+5) = 8

        for _ in 0..600 {
            g.advance_time();
        }

        let cap = (g.meta.best_realm + 5).min(14);

        assert_eq!(g.meta.neme.as_ref().unwrap().realm, cap);

        // 绝顶宿敌不再成长
        let mut g2 = game_with(Life::default(), Some(neme(14)));

        for _ in 0..100 {
            g2.advance_time();
        }

        assert_eq!(g2.meta.neme.as_ref().unwrap().realm, 14);
    }

    // ================= v1.10.2 离线闭关 =================

    // 离线收益纯函数：门槛/封顶/回拨逐项口径
    // v1.16.12 免烧寿元：签名不再收 lifespan/age——收益不被寿元余量钳制，
    // 8h 封顶仍是唯一上限（防放置反刷）；「绝不替玩家死亡」改由「age 不动」保证
    #[test]
    fn offline_settle_calc_gates() {
        // 缺基准（ts=0）不结算
        assert!(offline_settle_calc(0, 1_000_000, 10.0, 100).is_none());
        // 时钟回拨按 0 收益
        assert!(offline_settle_calc(2_000_000, 1_000_000, 10.0, 100).is_none());
        // N=0（<3 分钟）不动：179s → raw=1.99 → N=0
        assert!(offline_settle_calc(1000, 1179, 10.0, 100).is_none());
        // 90s=1 载、五折：1800s → raw=20 → N=10；qi=10×10=100；stones=100×10=1000
        let (n, qi, st) = offline_settle_calc(1000, 2800, 10.0, 100).unwrap();
        assert_eq!(n, 10);
        assert!((qi - 100.0).abs() < 1e-9, "qi={}", qi);
        assert_eq!(st, 1000);
        // 8h 封顶：一天挂机 → raw = min(86400,28800)/90 = 320 → N=160（五折）
        // v1.16.12：寿元余量再小也全额发放（旧断言 86400s→54 已随钳制删除）
        let (n, qi, _) = offline_settle_calc(1000, 1000 + 86_400, 10.0, 100).unwrap();
        assert_eq!(n, 160);
        assert!((qi - 1600.0).abs() < 1e-9, "qi={}", qi);
        // 超长离线（一周）仍 8h 封顶 → N=160，不随寿元余量缩水
        let (n, _, _) = offline_settle_calc(1000, 1000 + 7 * 86_400, 10.0, 100).unwrap();
        assert_eq!(n, 160);
    }

    // v1.16.12 整链：offline_settle_inner（时钟可注入）——离线发收益但绝不烧 age。
    // 旧实现 cap=lifespan−age−1 把 age 精确钉到 lifespan−1 风烛位（真机「52/75 载」
    //「154/155 载」承接前缘必寿元将近的根因）；本测试锁定任意离线时长 age/lifespan 不动。
    #[test]
    fn offline_settle_never_burns_age() {
        let mut g = game_with(Life::default(), None);
        let (age0, lifespan0) = {
            let l = g.life.as_ref().unwrap();
            (l.age, l.lifespan) // Life::default：16 / 75
        };
        // 整链：一天挂机 → N=160 满额收益入账；age/lifespan 不动（旧实现烧到 74/75）
        g.meta.last_ts = 1_000_000;
        g.offline_settle_inner(1_000_000 + 86_400);
        {
            let l = g.life.as_ref().unwrap();
            assert_eq!(l.age, age0, "离线不得烧 age");
            assert_eq!(l.lifespan, lifespan0);
            assert!(l.qi > 0.0, "修为收益须入账");
        }
        assert_eq!(g.meta.last_ts, 1_000_000 + 86_400, "离线起点基准推进");
        // N≥10 → 结算卡弹出且只带收益字段
        let card = g.offline_card.as_ref().expect("N=160 应弹结算卡");
        assert_eq!(card.years, 160);
        assert!(card.qi > 0.0);

        // 时间戳缺省（旧档无 ts=，parse 兜底 0）：不结算、仅落新基准、零收益
        g.offline_card = None;
        g.life = Some(Life::default());
        g.meta.last_ts = 0;
        g.offline_settle_inner(2_000_000);
        assert!(g.offline_card.is_none(), "ts=0 不得结算");
        assert_eq!(g.meta.last_ts, 2_000_000, "缺基准仅落新基准");
        assert_eq!(g.life.as_ref().unwrap().qi, 0.0, "缺基准零收益");
        assert_eq!(g.life.as_ref().unwrap().age, age0);

        // 时钟回拨（now < last_ts）：按 0 收益不结算，仅落新基准
        g.meta.last_ts = 3_000_000;
        g.offline_settle_inner(2_000_000);
        assert!(g.offline_card.is_none(), "回拨不得结算");
        assert_eq!(g.meta.last_ts, 2_000_000);
        assert_eq!(g.life.as_ref().unwrap().qi, 0.0);

        // 短离线（N<10）静默入账不弹卡：300s → raw=3.33 → N=1
        g.offline_settle_inner(2_000_000 + 300);
        assert!(g.offline_card.is_none(), "N=1 应静默入账");
        assert!(g.life.as_ref().unwrap().qi > 0.0, "静默径收益仍入账");
        assert_eq!(g.life.as_ref().unwrap().age, age0, "短离线同样不烧 age");
    }

    // v1.16.12 在线路径回归：周天吐纳 act_cultivate 每载仍 +1 age、寿尽仍正常身故
    //——免烧寿元仅限离线径，在线破境节奏（1.6s/载）不受影响。
    //（sync_save 在 native 仅置脏标志不落盘；advance_time/real_die 均纯内存，单测安全）
    #[test]
    fn online_cultivate_still_ages() {
        let mut g = game_with(Life::default(), None);
        let age0 = g.life.as_ref().unwrap().age; // 16
        g.act_cultivate(true);
        let age1 = g.life.as_ref().unwrap().age;
        assert!((age1 - (age0 + 1.0)).abs() < 1e-9, "在线每载 +1 age");
        assert!(g.life.as_ref().unwrap().qi > 0.0, "在线修为照常入账");
        // 寿尽线：age==lifespan → real_die → life 清空（在线死亡检查不受本次改动影响）
        {
            let l = g.life.as_mut().unwrap();
            l.age = l.lifespan - 1.0;
        }
        g.act_cultivate(true);
        assert!(g.life.is_none(), "在线寿尽仍应身故");
    }

    // ================= v1.10.2 成行择道 =================

    // 泄漏修正：launch_sim 必须剥离 path——模拟推演不携道途，五处乘区零泄漏；
    // 现实主角的道途不被触碰
    #[test]
    fn sim_does_not_inherit_path() {
        let mut l = Life::default();
        l.realm = 2;
        l.path = PATH_QI;
        let mut g = game_with(l, None);
        g.picked = vec![0]; // 至少一个天赋（launch_sim 卫语句）
        g.meta.sim_points = 5;
        g.launch_sim();
        assert_eq!(g.sim.as_ref().unwrap().path, 0, "模拟侧 path 必须为 0");
        assert_eq!(g.life.as_ref().unwrap().path, PATH_QI, "现实道途不被触碰");
    }

    // 钩子数值对照：剑修战力恰 ×1.35、气修修为恰 ×1.25、气修破境恰 +0.06
    //（丹修服丹 ×1.35 与灵田 ×1.6 的消费点含 sync_save/天气 roll，按验收口径手测）
    #[test]
    fn path_hooks_exact_multipliers() {
        let base = Life::default();
        let mut sword = base.clone();
        sword.path = PATH_SWORD;
        assert!(
            (power(&sword) / power(&base) - 1.35).abs() < 1e-9,
            "剑修战力应恰 ×1.35，实得 {}",
            power(&sword) / power(&base)
        );

        let meta = Meta::default();
        let mut qi_l = base.clone();
        qi_l.path = PATH_QI;
        let ratio = cult_rate(&qi_l, &meta) / cult_rate(&base, &meta);
        assert!((ratio - 1.25).abs() < 1e-9, "气修修为应恰 ×1.25，实得 {}", ratio);

        // +0.06 加在钳制之前：base 概率未触 0.05..0.95 界时应恰 +0.06
        let dp = big_break_p(&qi_l, 3) - big_break_p(&base, 3);
        assert!((dp - 0.06).abs() < 1e-6, "气修破境应恰 +0.06，实得 {}", dp);
    }

    // ================= v1.12.0 P0-1 修复回归 =================

    // 成行（跨境破入 realm 2）成功当帧必须弹道途卡——此前 open_path_card 仅
    // resume_life 旧档补弹（:1478），会话内首次破境后三途五处乘区全程空转。
    // act_break 成功率 p≈0.95（0.72+dao/300+luck/400 钳 0.95），循环兜住随机性；
    // 失败仅损修为/轻伤（无身死），回补修为重试。sync_save 仅置脏标记，单测无 IO。
    #[test]
    fn chengxing_break_opens_path_card() {
        let mut l = Life::default();
        l.realm = 1;
        l.layer = REALMS[1].layers.len() - 1; // 沁墨十二阶圆满（同层已尽 → 跨境）
        let mut g = game_with(l, None);

        let mut opened = false;

        for _ in 0..40 {
            if g.rmodal.is_some() {
                opened = true;

                break;
            }
            // 失败路径回补修为（qi×0.5 + injured=3 不阻断 can_break）
            let l2 = g.life.as_mut().unwrap();
            l2.qi = qi_need(l2);
            l2.injured = 0;
            g.act_break();
        }

        assert!(opened, "跨境破境成功后应弹道途卡");

        let rm = g.rmodal.as_ref().unwrap();

        assert!(
            matches!(rm.kind, CardKind::Path),
            "弹出的必须是道途卡，实得 {:?}",
            rm.title
        );

        assert_eq!(g.life.as_ref().unwrap().realm, 2, "应已破入成行");

        // 道途卡抉择后 path 落定且终身生效
        g.choose_path(PATH_SWORD);

        assert_eq!(g.life.as_ref().unwrap().path, PATH_SWORD);
    }

    // ================= v1.12.0 留行批注 =================

    // 开卷浮现上一稿的批注并消耗（take 即清），本稿气运+2 仅作用于模拟侧
    #[test]
    fn note_line_consumed_on_launch() {
        let mut l = Life::default();
        l.luck = 40.0;
        let mut g = game_with(l, None);
        g.meta.note_line = Some("23岁，与洛书遥缝作同页".to_string());
        g.picked = vec![0]; // launch_sim 卫语句
        g.meta.sim_points = 5;
        g.launch_sim();
        assert!(g.meta.note_line.is_none(), "开卷即消耗批注");
        assert!(
            g.sim_log.iter().any(|x| x.text.contains("【批注】")),
            "开卷应浮现批注行"
        );
        assert_eq!(g.sim.as_ref().unwrap().luck, 42.0, "本稿气运+2");
    }

    // ================= v1.12.0 空白行 =================

    // build_event(30)：置位每稿一次门闩并组三选卡（留墨/留纸/留行）
    #[test]
    fn blank_event_card_and_gate() {
        let mut g = game_with(Life::default(), None);
        let mut s = Life::default();
        s.realm = 2; // min_realm=2 成行境
        g.sim = Some(s);
        assert!(!g.blank_used);
        g.build_event(30);
        assert!(g.blank_used, "抽到空白行即置位门闩");
        let card = g.modal.as_ref().unwrap();
        assert_eq!(card.title, "空白行");
        assert_eq!(card.opts.len(), 3);
    }

    // 留行臂（special 19）不 return：气运+2 走通用乘区生效 + 大事记亲手写行
    #[test]
    fn blank_liuhang_applies_luck_and_highlight() {
        let mut g = game_with(Life::default(), None);
        let mut s = Life::default();
        s.realm = 2;
        s.luck = 40.0;
        s.age = 20.0;
        g.sim = Some(s);
        g.build_event(30);
        g.choose_option(2); // 留行 · 气运+2
        let s = g.sim.as_ref().unwrap();
        assert_eq!(s.luck, 42.0, "留行气运+2 应走通用乘区生效");
        assert!(
            s.highlights.iter().any(|h| h.contains("空白行")),
            "留行应在大事记写下一行"
        );
    }

    // 每稿一次：blank_used 置位后 sim_event 权重对 id 30 归零（300 抽不再现）
    #[test]
    fn blank_event_once_per_sim() {
        let mut g = game_with(Life::default(), None);
        let mut s = Life::default();
        s.realm = 5; // 放开大部分事件抬高期望
        s.luck = 100.0; // 善缘权重拉满
        g.sim = Some(s);
        g.build_event(30);
        for _ in 0..300 {
            g.modal = None;
            g.sim_event();
            if let Some(c) = &g.modal {
                assert_ne!(c.title, "空白行", "门闩置位后同一稿不得再遇空白行");
            }
        }
    }

    // 开新稿重置门闩（launch_sim 清零）
    #[test]
    fn blank_used_resets_per_sim() {
        let mut g = game_with(Life::default(), None);
        g.picked = vec![0]; // launch_sim 卫语句
        g.meta.sim_points = 5;
        g.blank_used = true;
        g.launch_sim();
        assert!(!g.blank_used, "launch_sim 必须清零空白行门闩");
    }

    // ================= v1.12.1 终卷四结局 =================

    // 终战卡：D《合订》钮仅当 endings_seen bit1（B 已达成）时出现；
    // 未解锁位 choose_ending(3) 卫语句拒绝（脏点击不置位、不置演出页）
    #[test]
    fn finale_card_d_button_gated_on_ending_b() {
        let mut g = game_with(Life::default(), None);
        g.open_finale_card();
        assert!(matches!(g.rmodal.as_ref().unwrap().kind, CardKind::Finale));
        assert_eq!(g.rmodal.as_ref().unwrap().opts.len(), 3, "未达成 B：卡面 3 钮");
        g.choose_ending(3); // 未解锁的 D 位：卫语句拒绝
        assert!(g.rmodal.is_none());
        assert_eq!(g.meta.endings_seen, 0, "未解锁位不得置位");
        assert!(g.ending_show.is_none());

        // 达成 B（bit1）后再问笔：第 4 钮出现，choose_ending(3) 置 bit3 并进演出页
        g.meta.endings_seen = 0b0010;
        g.open_finale_card();
        assert_eq!(g.rmodal.as_ref().unwrap().opts.len(), 4, "达成 B：卡面 4 钮");
        g.choose_ending(3);
        assert_eq!(g.meta.endings_seen & 0b1000, 0b1000, "D 位应置位");
        assert_eq!(g.ending_show, Some(3));
    }

    // 抉择置位/落盘链：choose_ending(i) 置位、清 rmodal、置演出页；重入幂等
    #[test]
    fn choose_ending_sets_bit_and_show() {
        let mut g = game_with(Life::default(), None);
        g.rmodal = Some(EventCard {
            kind: CardKind::Finale,
            actor: None,
            rare: false,
            stage: None,
            cat: None,
            scene: None,
            brk: false,
            title: "终卷 · 执笔人递笔".into(),
            text: "".into(),
            opts: vec![
                ("《定稿》 · 认写".into(), Eff::new()),
                ("《留白》 · 撕稿".into(), Eff::new()),
                ("《执笔》 · 夺笔".into(), Eff::new()),
            ],
        });
        g.choose_ending(1);
        assert_eq!(g.meta.endings_seen & 0b0010, 0b0010, "B《留白》位应置位");
        assert_eq!(g.ending_show, Some(1));
        assert!(g.rmodal.is_none(), "抉择后终战卡应清空");
        // 同结局重入（异常路径兜底）：位掩码幂等
        g.ending_show = None;
        g.choose_ending(1);
        assert_eq!(g.meta.endings_seen & 0b0010, 0b0010);
    }

    // ================= 剧情演出包：入场动效（会话态计时） =================

    // 卡面各开启口统一置 modal_anim=0.18（道途/终战/好感事件），会话态默认 0
    //（旧档口径：载入天然无动画）；卫语句拒绝的开启口不得置值
    #[test]
    fn card_open_sets_modal_anim() {
        let mut g = game_with(Life::default(), None);
        assert_eq!(g.modal_anim, 0.0, "会话态默认无动画（旧档载入口径）");

        g.open_path_card();
        assert!(g.rmodal.is_some());
        assert_eq!(g.modal_anim, 0.18, "道途卡开启帧置入场动效");

        g.rmodal = None;
        g.open_finale_card();
        assert!(g.rmodal.is_some());
        assert_eq!(g.modal_anim, 0.18, "终战卡开启帧置入场动效");

        // 好感事件口：好感达标 + 未抉择过 + 非模拟 + 无悬空卡 → add_fav 触发
        g.rmodal = None;
        g.meta.npc_ev = [false, false, false, false];
        g.meta.npc_fav[0] = crate::data::NPC_EVENT_FAV;
        crate::npc::add_fav(&mut g, 0, 0);
        assert!(g.rmodal.is_some(), "好感达标应触发事件卡");
        assert_eq!(g.modal_anim, 0.18, "好感事件卡开启帧置入场动效");

        // 卫语句拒绝口：rmodal 已占时 open_path_card 不置值、不动既有卡
        g.modal_anim = 0.0;
        g.open_path_card();
        assert_eq!(g.modal_anim, 0.0, "rmodal 被占时不得重启动效");
    }

    // 终局演出页：choose_ending 置 ending_anim=0.6（动画期合页钮禁点的数据基础，
    // 禁点判定在 draw 侧 can_close）；卫语句拒绝路径（未解锁 D 位）不得置值；
    // 书志重读路径不置位（from_book 在 draw 侧跳过演出）
    #[test]
    fn ending_open_sets_ending_anim() {
        let mut g = game_with(Life::default(), None);
        assert_eq!(g.ending_anim, 0.0);
        g.rmodal = Some(EventCard {
            kind: CardKind::Finale,
            actor: None,
            rare: false,
            stage: None,
            cat: None,
            scene: None,
            brk: false,
            title: "终卷 · 执笔人递笔".into(),
            text: "".into(),
            opts: vec![
                ("《定稿》 · 认写".into(), Eff::new()),
                ("《留白》 · 撕稿".into(), Eff::new()),
                ("《执笔》 · 夺笔".into(), Eff::new()),
            ],
        });
        g.choose_ending(1);
        assert_eq!(g.ending_show, Some(1));
        assert_eq!(g.ending_anim, 0.6, "终局页开启帧置入场节奏");

        // 未解锁 D 位：卫语句拒绝，不置演出页不置动效
        //（先清 endings_seen——上一步 choose_ending(1) 已置 bit1，会使 D 解锁）
        g.ending_show = None;
        g.ending_anim = 0.0;
        g.meta.endings_seen = 0;
        g.rmodal = Some(EventCard {
            kind: CardKind::Finale,
            actor: None,
            rare: false,
            stage: None,
            cat: None,
            scene: None,
            brk: false,
            title: "终卷 · 执笔人递笔".into(),
            text: "".into(),
            opts: vec![
                ("《定稿》 · 认写".into(), Eff::new()),
                ("《留白》 · 撕稿".into(), Eff::new()),
                ("《执笔》 · 夺笔".into(), Eff::new()),
                ("《合订》 · 收册人（隐藏）".into(), Eff::new()),
            ],
        });
        g.choose_ending(3);
        assert!(g.ending_show.is_none());
        assert_eq!(g.ending_anim, 0.0, "卫语句拒绝不得启动演出节奏");
    }

    // ================= v1.12.1 讨牒/缝页 =================

    // 讨牒对账：slain=6 首检 while 补发、第 6 牒 +166 道韵；幂等不重发；slain>6 钳 6
    #[test]
    fn taodie_sync_backfills_and_caps() {
        let mut g = game_with(Life::default(), None);
        g.meta.neme_slain = 6;
        // 第三章位按事实先补齐隔离（neme_slain=6 → pid3/14）：其章成之礼不入本测总账；
        // 第四章位在 taodie=0 时补不齐（谓词就是 taodie 本身），边沿必随补牒触发——
        // 断言显式计入 CHAPTERS[3].reward（v1.16.9 章成之礼，边沿发放见 check_chapters）
        g.meta.chap_done |= Game::chapter_done_mask(&g.meta);
        let dy0 = g.meta.dao_yun;
        let pay = 166 + CHAPTERS[3].reward as i64; // 封卷牒 +166、章成之礼 +166
        g.sync_taodie();
        assert_eq!((g.meta.taodie, g.meta.dao_yun), (6, dy0 + pay));
        g.sync_taodie(); // 幂等（讨牒 while 与章节位均已置）
        assert_eq!(g.meta.dao_yun, dy0 + pay);
        g.meta.neme_slain = 9;
        g.sync_taodie();
        assert_eq!(g.meta.taodie, 6); // 越链顶钳 6
    }

    // 缘二 perk：开局灵石 ×1.3（begin_mortal 单口；尾部 visit_region 初至奖 +2000
    // 一存档仅一次，与基数分立不相乘——首世 100+rv、次世 130 纯基数）
    #[test]
    fn feng_blv2_scales_starting_stones() {
        let rv = REGION_VISIT_REWARD[0].1;
        let mut g = game_with(Life::default(), None);
        g.begin_mortal();
        assert_eq!(g.life.as_ref().unwrap().stones, 100 + rv, "无缘 baseline");
        g.meta.feng_blv = 2;
        g.begin_mortal();
        assert_eq!(g.life.as_ref().unwrap().stones, 130, "缘二开局基数 ×1.3");
    }

    // 缘深单口：与缝页人同页至终缘+1（三生为限），他人/无伴离世不变——表驱动四态
    #[test]
    fn feng_bond_up_only_for_feng() {
        let cases = [
            (Some(COMPANION_NAMES[3]), 1u8, 2u8), // 缝页人同页至终：缘+1
            (None, 1, 1),                         // 无伴离世：不变
            (Some(COMPANION_NAMES[4]), 1, 1),     // 他人离世：不变
            (Some(COMPANION_NAMES[3]), 3, 3),     // 缘满三生：封顶
        ];
        for (comp, blv0, want) in cases {
            let mut l = Life::default();
            l.companion = comp.map(|c| c.to_string());
            let mut g = game_with(l, None);
            g.meta.feng_bn = 3;
            g.meta.feng_blv = blv0;
            g.real_die("试归尘");
            assert_eq!(g.meta.feng_blv, want, "comp={:?} blv0={}", comp, blv0);
        }
    }

    // 重逢判定：结缘的正是缝页人且先前并非同页；未定名恒否
    #[test]
    fn feng_reunion_predicate() {
        let mut g = game_with(Life::default(), None);
        g.meta.feng_bn = 2;
        assert!(g.feng_reunion(Some("旁人"), COMPANION_NAMES[2]));
        assert!(g.feng_reunion(None, COMPANION_NAMES[2]), "跨世初婚亦重逢");
        assert!(!g.feng_reunion(Some(COMPANION_NAMES[2]), COMPANION_NAMES[2]));
        g.meta.feng_bn = crate::data::FENG_UNSET;
        assert!(!g.feng_reunion(None, COMPANION_NAMES[2]));
    }

    // v1.16.20 立绘出场点判定同源钉死（红线「显示判定与 companion 账同源不分叉」）：
    // 结算屏谢幕像快照 SettleInfo.comp = real_die 事件帧按 companion 名账定格，与
    // 继承屏伴小像（SimResult.comp）/随行小像（draw_sim）/结局页谢幕像四处共用的
    // 「名 → COMPANION_NAMES 位置」推导同式。表驱动：无伴/有名/未知名三态。
    #[test]
    fn settle_comp_snapshot_same_source_as_companion_ledger() {
        // 名→位置推导逐名钉死（四处出场点共用表达式；None/未知名一律 None 零绘制）
        for (i, name) in COMPANION_NAMES.iter().enumerate() {
            let idx = Some(*name).and_then(|cn| COMPANION_NAMES.iter().position(|&n| n == cn));
            assert_eq!(idx, Some(i), "名账推导漂移: {}", name);
        }
        let none_idx =
            None::<&str>.and_then(|cn| COMPANION_NAMES.iter().position(|&n| n == cn));
        assert_eq!(none_idx, None, "无伴恒 None");
        let unknown = Some("不在册之人")
            .and_then(|cn| COMPANION_NAMES.iter().position(|&n| n == cn));
        assert_eq!(unknown, None, "未知名恒 None（不误触发纹理查询）");
        // 快照接线：real_die 后 SettleInfo.comp=Some(2)；无伴离世恒 None（零位移）
        let mut l = Life::default();
        l.companion = Some(COMPANION_NAMES[2].to_string());
        let mut g = game_with(l, None);
        g.real_die("试归尘");
        assert_eq!(
            g.settle.as_ref().unwrap().comp,
            Some(2),
            "结算屏谢幕像快照与 companion 账同源"
        );
        let mut g0 = game_with(Life::default(), None);
        g0.real_die("试归尘");
        assert_eq!(
            g0.settle.as_ref().unwrap().comp,
            None,
            "无伴离世谢幕像零绘制"
        );
    }
}

// ---------------- v1.16.0 道途一念：guide_index 优先级矩阵 ----------------
// 构造性终结证明的回归锚：T0..T10 逐叶钉死 + 全网格 0..=8 兜底
// （防实现时在覆盖块之下误加叶 → 死分支回归）。

#[cfg(test)]
mod guide_tests {
    use super::revenge_tests::{game_with, neme};
    use super::*;
    use crate::guide::{guide_index, HINTS};

    /// 前置谓词全走既有单口（is_stuck/can_break/revenge_ready/exchange_cost），
    /// 先 assert 前置再断言下标——谓词改语义时测试先红，指引随谓词自动跟随。

    // T0 山穷水尽：P=0 S=0 R=∅ → 0（死局叶最高优先，跳转看广告）
    #[test]
    fn t0_stuck_bottoms_out() {
        let mut l = Life::default();
        l.stones = 0; // R 恒空（default）
        let mut g = game_with(l, None);
        g.meta.sim_points = 0;
        assert!(g.is_stuck(), "前置：点数/灵石/法宝三空即死局");
        assert_eq!(guide_index(&g), 0);
    }

    // T1 破境：qi=qi_need P=0 S=2000(≥ex1500) auto=1 → 3（修为圆满叶）
    #[test]
    fn t1_can_break_top_of_cult_axis() {
        let mut l = Life::default();
        l.stones = 2000;
        l.qi = qi_need(&l);
        let mut g = game_with(l, None);
        g.meta.sim_points = 0;
        g.meta.auto = 1;
        g.meta.total_sims = 1; // v1.16.8 里程碑叶（total_sims==0）已过，验原修为叶
        assert!(!g.is_stuck(), "前置：S≥ex 不死局");
        assert!(g.can_break(), "前置：修为已满");
        assert_eq!(guide_index(&g), 3);
    }

    // T2 寻仇：弱宿敌(realm 0)未重伤 P=0 S=2000 auto=1 → 4
    #[test]
    fn t2_revenge_ready() {
        let mut l = Life::default();
        l.stones = 2000; // 不死局
        l.qi = 0.0; // 不破境不候破
        let mut g = game_with(l, Some(neme(0))); // 同境弱宿敌：r≈0.96 可出手
        g.meta.sim_points = 0;
        g.meta.auto = 1;
        g.meta.total_sims = 1; // v1.16.8 里程碑叶已过
        assert!(!g.is_stuck() && !g.can_break(), "前置：修为轴不截胡");
        assert!(g.revenge_ready(), "前置：宿敌可寻仇（未重伤且 r≥0.1）");
        assert_eq!(guide_index(&g), 4);
    }

    // T2b 图鉴叶（v1.16.10 宿敌叶后插入，v1.16.16 讨牒叶后移至 6）：任一故人好感
    // 踩线且事件未抉择 → 6（压修为候破/洞府/经济日常叶——第六章主线目标优先）；
    // 抉择后谓词翻假落回原链
    #[test]
    fn t2b_gallery_fav_event_hint() {
        let mut l = Life::default();
        l.stones = 3000; // 经济叶本可中（≥ex）
        l.qi = 0.0; // 修为轴全空，排除叶 3/8
        let mut g = game_with(l, None);
        g.meta.sim_points = 0;
        g.meta.auto = 1;
        g.meta.total_sims = 1; // v1.16.8 里程碑叶已过
        assert_eq!(guide_index(&g), 13, "前置：无心事 → 兜底经济叶");
        g.meta.npc_fav[1] = crate::data::NPC_EVENT_FAV;
        assert_eq!(guide_index(&g), 6, "故人心事待叙 → 图鉴叶（压经济叶）");
        g.meta.npc_ev[1] = true; // 抉择即消耗（npc.rs 抉择口同语义）
        assert_eq!(guide_index(&g), 13, "抉择毕谓词翻假 → 落回原链");
    }

    // T2c 章节进行叶（v1.16.16 插入）：当前章=第四章（章 0..2 已了）∧ taodie<6
    // → 5 讨牒叶；taodie 至 6 → 第四章翻毕、第五章到域叶（7）接力；五域走满 →
    // 谓词落假落回原链（灵石兜底叶 13）——两叶皆非截断的链路回归验证
    #[test]
    fn t2c_chapter_progress_taodie_region_hints() {
        let mut l = Life::default();
        l.stones = 3000; // 经济叶本可中（≥ex），兜底叶可达
        l.qi = 0.0; // 修为轴全空，排除叶 3/8
        let mut g = game_with(l, None);
        g.meta.sim_points = 0;
        g.meta.auto = 1;
        g.meta.total_sims = 1; // pid0
        g.meta.lives = 1; // pid12
        g.meta.total_wins = 1; // pid1
        g.meta.first_realm[1] = true; // pid2
        g.meta.neme_slain = 2; // pid3/pid14（第三章两目标全了）
        // 章 0..2 全了、第四章（讨牒）未了：taodie<6
        assert_eq!(Game::chapter_current(&g.meta), 3, "前置：当前章=第四章");
        assert_eq!(guide_index(&g), 5, "第四章进行期 → 讨牒叶（压日常叶）");
        g.meta.taodie = 6; // 六牒归档 → 第四章翻毕，当前章前移第五章
        assert_eq!(Game::chapter_current(&g.meta), 4, "前置：当前章=第五章");
        assert_eq!(guide_index(&g), 7, "第五章进行期 ∧ 五域未全 → 到域叶接力");
        g.meta.region_visited = [true, true, true, true, true]; // 五域全至
        assert_eq!(guide_index(&g), 13, "两章进行叶皆落假 → 落回灵石兜底叶");
    }

    // T3 候破：qi=0.9×need P=0 S=2000 auto=1 → 8
    #[test]
    fn t3_qi_near_full_seclusion() {
        let mut l = Life::default();
        l.stones = 2000;
        l.qi = 0.9 * qi_need(&l);
        let mut g = game_with(l, None);
        g.meta.sim_points = 0;
        g.meta.auto = 1;
        g.meta.total_sims = 1; // v1.16.8 里程碑叶已过
        assert!(!g.can_break(), "前置：未满不破境");
        assert!(
            g.life.as_ref().map_or(false, |x| x.qi / qi_need(x) >= 0.8),
            "前置：修为 ≥80%"
        );
        assert_eq!(guide_index(&g), 8);
    }

    // T4 扩建：loc=洞府 dongfu=[0,0,0] 贡献=30 S=2500 P=0 auto=1 qi=0 → 9
    #[test]
    fn t4_dongfu_upgradable() {
        let mut l = Life::default();
        l.loc = crate::data::NODE_DONGFU;
        l.stones = 2500;
        l.qi = 0.0;
        let mut g = game_with(l, None);
        g.meta.sim_points = 0;
        g.meta.auto = 1;
        g.meta.total_sims = 1; // v1.16.8 里程碑叶已过
        g.meta.sect_contrib = 30; // = DONGFU_UP_CONTRIB[0]，双资源踩线可达
        assert_eq!(
            g.life.as_ref().map(|x| x.loc),
            Some(crate::data::NODE_DONGFU)
        );
        assert!(
            g.life
                .as_ref()
                .map_or(false, |x| x.stones >= crate::data::DONGFU_UP_STONES[0])
                && g.meta.sect_contrib >= crate::data::DONGFU_UP_CONTRIB[0],
            "前置：灵石+贡献双足"
        );
        assert_eq!(guide_index(&g), 9);
    }

    // T5 周天：P=0 S=1600(≥ex1500<扩建2000) R=∅ auto=0 qi=0 loc=坊市 → 10
    #[test]
    fn t5_zhouytian_off_hint() {
        let mut l = Life::default();
        l.loc = crate::data::NODE_MARKET; // 不落洞府扩建叶
        l.stones = 1600;
        l.qi = 0.0;
        let mut g = game_with(l, None);
        g.meta.sim_points = 0;
        g.meta.total_sims = 1; // v1.16.8 里程碑叶已过
        assert!(!g.is_stuck(), "前置：S≥ex 不死局");
        assert!(g.meta.auto == 0 && g.meta.sim_points < g.sim_cost(), "前置：周天未开且无点可花");
        assert_eq!(guide_index(&g), 10);
    }

    // T6 献祭：R=[0] P=0 S=1000(<ex) auto=1 qi=0 loc=坊市 → 9
    #[test]
    fn t6_sacrifice_relic_hint() {
        let mut l = Life::default();
        l.loc = crate::data::NODE_MARKET;
        l.stones = 1000;
        l.relics.push(0);
        l.qi = 0.0;
        let mut g = game_with(l, None);
        g.meta.sim_points = 0;
        g.meta.auto = 1;
        g.meta.total_sims = 1; // v1.16.8 里程碑叶已过
        assert!(!g.is_stuck(), "前置：法宝尚存不死局");
        assert!(
            g.life.as_ref().map_or(false, |x| !x.relics.is_empty())
                && g.meta.sim_points < g.sim_cost()
                && g.life
                    .as_ref()
                    .map_or(false, |x| x.stones < g.exchange_cost()),
            "前置：有点可献、点灵两空"
        );
        assert_eq!(guide_index(&g), 11);
    }

    // T7a 里程碑叶（v1.16.8 链首）：Meta::default 原样（新玩家首局态 P=3 S=100
    // R=∅ auto=0 qi=0 total_sims=0）→ 1（开卷首模拟压经济/养成叶，主线第一章）
    #[test]
    fn t7a_milestone_first_sim_overrides_economy() {
        let g = game_with(Life::default(), None);
        assert_eq!(g.meta.total_sims, 0, "前置：全新档未开卷");
        assert!(
            g.meta.sim_points >= g.sim_cost(),
            "前置：首局自带 3 点可开卷（经济叶本可中）"
        );
        assert_eq!(guide_index(&g), 1, "里程碑叶压经济叶——开局只指开卷");
    }

    // T7b 开模拟：total_sims=1 后回到经济叶（Meta::default 其余原样 P=3）→ 12
    #[test]
    fn t7b_first_run_open_draft_after_milestone() {
        let mut g = game_with(Life::default(), None);
        g.meta.total_sims = 1; // 里程碑已了
        assert!(
            g.meta.sim_points >= g.sim_cost(),
            "前置：自带 3 点可开卷"
        );
        assert_eq!(guide_index(&g), 12, "周天叶收窄 P<1，有点数不得截胡");
    }

    // T8 兑点：P=0 S=3000 R=∅ auto=1 qi=0 loc=坊市 → 13（兜底叶）
    #[test]
    fn t8_stones_rich_exchange() {
        let mut l = Life::default();
        l.loc = crate::data::NODE_MARKET;
        l.stones = 3000;
        l.qi = 0.0;
        let mut g = game_with(l, None);
        g.meta.sim_points = 0;
        g.meta.auto = 1;
        g.meta.total_sims = 1; // v1.16.8 里程碑叶已过
        assert!(!g.is_stuck() && g.meta.sim_points < g.sim_cost(), "前置：无点但不死局");
        assert_eq!(guide_index(&g), 13);
    }

    // T9 边界：S 恰 =ex → 不算死局 → 13
    #[test]
    fn t9_boundary_stones_equal_exchange_cost() {
        let mut l = Life::default();
        l.stones = 1500; // = realm 0 兑换价 exchange_cost()
        l.qi = 0.0;
        let mut g = game_with(l, None);
        g.meta.sim_points = 0;
        g.meta.auto = 1;
        g.meta.total_sims = 1; // v1.16.8 里程碑叶已过
        assert_eq!(
            g.life.as_ref().map(|x| x.stones),
            Some(g.exchange_cost()),
            "前置：踩线等值"
        );
        assert!(!g.is_stuck(), "边界：S<ex 才死局，等值不算");
        assert_eq!(guide_index(&g), 13, "total_sims=1 兜底终结叶");
    }

    // T10 边界：S=ex-1 → 死局 → 0（死局叶压里程碑叶——三空时看广告是唯一出路）
    #[test]
    fn t10_boundary_one_short_is_stuck() {
        let mut l = Life::default();
        l.stones = 1499; // ex(1500)-1
        l.qi = 0.0;
        let mut g = game_with(l, None);
        g.meta.sim_points = 0;
        g.meta.auto = 1;
        g.meta.total_sims = 1; // 里程碑叶不截胡死局叶——stuck 优先级最高
        assert!(g.is_stuck(), "边界：差 1 灵石即死局");
        assert_eq!(guide_index(&g), 0);
    }

    // 全网格兜底：P×S×R×auto×qi 120 组合全部落在合法叶域内
    //（构造性保证——覆盖块之下不放任何叶，死分支在此必现形；
    //  total_sims=1 过里程碑叶，专测原链 3..11 的覆盖完备性）
    #[test]
    fn grid_always_lands_within_hints() {
        for p in [0i32, 1] {
            for s in [0i64, 1000, 1600, 2000, 3000] {
                for r in [0usize, 1] {
                    for auto in [0u8, 1] {
                        for qi_frac in [0.0f64, 0.9, 1.0] {
                            let mut l = Life::default();
                            l.loc = crate::data::NODE_MARKET;
                            l.stones = s;
                            if r == 1 {
                                l.relics.push(0);
                            }
                            l.qi = qi_frac * qi_need(&l);
                            let mut g = game_with(l, None);
                            g.meta.sim_points = p;
                            g.meta.auto = auto;
                            g.meta.total_sims = 1; // 过里程碑叶，专测原链
                            let idx = guide_index(&g) as usize;
                            assert!(
                                idx < HINTS.len(),
                                "P={} S={} R={} auto={} qi×{} → {} 越界",
                                p,
                                s,
                                r,
                                auto,
                                qi_frac,
                                idx
                            );
                        }
                    }
                }
            }
        }
    }

    // 提示表契约：14 叶全为静态字面量（编译期已保证 & 'static str），
    // 按钮叶与 GuideAct 一一对应——纯提示叶恒无按钮
    #[test]
    fn hints_table_shape() {
        assert_eq!(HINTS.len(), 14);
        for h in HINTS.iter() {
            if h.act == crate::guide::GuideAct::None {
                assert!(h.btn.is_none(), "纯提示叶不得带按钮");
            } else {
                assert!(h.btn.is_some(), "可跳转叶必须带按钮");
            }
        }
    }

    // 跳转执行器接线：三个有状态动作各自落在既有单入口的同一状态位
    //（Draft 走 start_draft 既有测试面；World 不入本测——open_world→art::map_bg_for
    //  需 GPU 上下文，无头单测不可达，World 链由既有冒烟 shot_058 实机覆盖）
    #[test]
    fn exec_jump_wires_existing_entries() {
        // Cult → 修炼弹层（open_sub 单口，含开帧门闩）
        let mut g = game_with(Life::default(), None);
        crate::guide::exec_jump(&mut g, crate::guide::GuideAct::Cult);
        assert!(matches!(g.submenu, Some(crate::menu::Sub::Cult)));
        assert!(g.sub_fresh, "open_sub 开帧门闩必须随跳转置位");
        // Shop → 坊市·点数兑换页签
        let mut g = game_with(Life::default(), None);
        crate::guide::exec_jump(&mut g, crate::guide::GuideAct::Shop);
        assert_eq!((g.panel, g.shop_tab), (1, 2));
        // Bag → 行囊·法宝页
        let mut g = game_with(Life::default(), None);
        crate::guide::exec_jump(&mut g, crate::guide::GuideAct::Bag);
        assert_eq!((g.panel, g.bag_tab), (2, 1));
        // Gallery → 角色面板·图鉴页签（v1.16.10：panel=5 + char_tab=4）
        let mut g = game_with(Life::default(), None);
        crate::guide::exec_jump(&mut g, crate::guide::GuideAct::Gallery);
        assert_eq!((g.panel, g.char_tab), (5, 4));
        // None → 空转（防御臂）
        let mut g = game_with(Life::default(), None);
        crate::guide::exec_jump(&mut g, crate::guide::GuideAct::None);
        assert_eq!(g.panel, 0);
        assert!(g.submenu.is_none());
    }
}

// ---------------- v1.16.8 章节书：goal_met / done_mask / check_chapters ----------------

#[cfg(test)]
mod chapter_tests {
    use super::revenge_tests::game_with;
    use super::*;
    use crate::data::CHAPTERS;

    // 谓词单口逐 id 钉死（0..=14 逐一翻转验证；脏 pid 落 _ 臂恒 false 不 panic）
    #[test]
    fn goal_predicates_pin_every_id() {
        let mut g = game_with(Life::default(), None);
        // 12 转生入世：v1.16.8 复审修——挂 lives 计数（单调持久，不随菜单/身故的
        // meta.life=None 快照摆动），game_with 构造器缺省 lives=0
        assert!(!Game::chapter_goal_met(&g.meta, 12), "lives=0 未入世");
        g.meta.lives = 1;
        assert!(Game::chapter_goal_met(&g.meta, 12));
        // 0 首模拟 / 14 再斩 / 3 首斩（neme_slain 单值覆盖两档）
        assert!(!Game::chapter_goal_met(&g.meta, 0));
        g.meta.total_sims = 1;
        assert!(Game::chapter_goal_met(&g.meta, 0));
        g.meta.neme_slain = 1;
        assert!(Game::chapter_goal_met(&g.meta, 3) && !Game::chapter_goal_met(&g.meta, 14));
        g.meta.neme_slain = 2;
        assert!(Game::chapter_goal_met(&g.meta, 14));
        // 1 首胜
        assert!(!Game::chapter_goal_met(&g.meta, 1));
        g.meta.total_wins = 1;
        assert!(Game::chapter_goal_met(&g.meta, 1));
        // 2 现实首破境（first_realm[1] 或 best_realm≥1 兜底）
        assert!(!Game::chapter_goal_met(&g.meta, 2));
        g.meta.first_realm[1] = true;
        assert!(Game::chapter_goal_met(&g.meta, 2));
        let mut g2 = game_with(Life::default(), None);
        g2.meta.best_realm = 1; // 旧档 fr 缺失时 best 兜底
        assert!(Game::chapter_goal_met(&g2.meta, 2));
        // 4/5 讨牒两档
        g2.meta.taodie = 3;
        assert!(Game::chapter_goal_met(&g2.meta, 4) && !Game::chapter_goal_met(&g2.meta, 5));
        g2.meta.taodie = 6;
        assert!(Game::chapter_goal_met(&g2.meta, 5));
        // 6/7 五域两档
        g2.meta.region_visited = [true, true, true, false, false];
        assert!(Game::chapter_goal_met(&g2.meta, 6) && !Game::chapter_goal_met(&g2.meta, 7));
        g2.meta.region_visited = [true; 5];
        assert!(Game::chapter_goal_met(&g2.meta, 7));
        // 8 首结缘 / 9 好感事件抉择
        g2.meta.companion_lives = 1;
        g2.meta.npc_ev[2] = true;
        assert!(Game::chapter_goal_met(&g2.meta, 8) && Game::chapter_goal_met(&g2.meta, 9));
        // 10 首登仙界（first_realm[10] 或 best_realm≥10 兜底）
        assert!(!Game::chapter_goal_met(&g2.meta, 10));
        g2.meta.first_realm[10] = true;
        assert!(Game::chapter_goal_met(&g2.meta, 10));
        let mut g3 = game_with(Life::default(), None);
        g3.meta.best_realm = 10;
        assert!(Game::chapter_goal_met(&g3.meta, 10));
        // 11/13 结局任一 / 四卷全览
        g3.meta.endings_seen = 0b0001;
        assert!(Game::chapter_goal_met(&g3.meta, 11) && !Game::chapter_goal_met(&g3.meta, 13));
        g3.meta.endings_seen = 0b1111;
        assert!(Game::chapter_goal_met(&g3.meta, 13));
        // 15 书外之章·执卷境（v1.16.10）：best_realm 阈 13（单调持久标量，跨世不摆动）
        assert!(!Game::chapter_goal_met(&g3.meta, 15), "best_realm=10 未抵执卷");
        g3.meta.best_realm = 13;
        assert!(Game::chapter_goal_met(&g3.meta, 15));
        // 脏 pid：卫语句臂 false，绝不下标裸访
        assert!(!Game::chapter_goal_met(&g3.meta, 99));
        assert!(!Game::chapter_goal_met(&g3.meta, 16));
    }

    // 章节表契约：pid 全在 0..=15 分发域内、目标数 1..=2、文案非空（静态表自检；
    // v1.16.10 九章——末章书外之章，len 9 与掩码 bit0..=8 对齐）
    #[test]
    fn chapters_table_shape() {
        assert_eq!(CHAPTERS.len(), 9);
        for (i, ch) in CHAPTERS.iter().enumerate() {
            assert!(!ch.num.is_empty() && !ch.name.is_empty() && !ch.intro.is_empty());
            // v1.16.20 章开场白随表钉死：全行非空 ≤24 字（日志金行单行预算；「了+启」
            // 逐章成对发，启句过长刷屏——含恒不演出的首行，防表扩员时漂移出空句）
            assert!(
                !ch.open.is_empty() && ch.open.chars().count() <= 24,
                "章 {} open 空句或越 24 字预算: {} ({}字)",
                i,
                ch.open,
                ch.open.chars().count()
            );
            assert!(
                ch.goals.len() >= 1 && ch.goals.len() <= 2,
                "章 {} 目标数越界",
                i
            );
            for gl in ch.goals.iter() {
                assert!(gl.pid <= 15, "章 {} pid {} 越分发域", i, gl.pid);
                assert!(!gl.text.is_empty());
            }
        }
    }

    // v1.16.20 章开场白演出：章 i 了边沿「了+启」逐章成对（本章了金行 + 下一章
    // 启金行）；幂等（newly 边沿语义，重复调用不复发）
    #[test]
    fn chapter_open_emits_paired_with_done_edge() {
        let mut g = game_with(Life::default(), None);
        g.meta.lives = 1;
        g.meta.total_sims = 1; // 仅章 1 两目标全了 → newly=bit0
        g.check_chapters();
        assert!(
            g.log.iter().any(|l| l.text.contains("【第一章·了】")),
            "了金行缺"
        );
        assert!(
            g.log
                .iter()
                .any(|l| l.text.contains("【第二章·启】") && l.text.contains(CHAPTERS[1].open)),
            "启金行缺或与表漂移: {:?}",
            g.log.iter().map(|l| &l.text).collect::<Vec<_>>()
        );
        let n_open = g.log.iter().filter(|l| l.text.contains("·启】")).count();
        g.check_chapters(); // 边沿已置，幂等不复发
        assert_eq!(
            g.log.iter().filter(|l| l.text.contains("·启】")).count(),
            n_open,
            "启句不随重复调用复发"
        );
    }

    // v1.16.20 末章了后无下一章不发启句（章9 bit8 完成后日志克制不刷屏红线）
    #[test]
    fn chapter_open_last_chapter_emits_none() {
        let mut g = game_with(Life::default(), None);
        g.meta.endings_seen = 0b1111; // pid13 四卷全览
        g.meta.best_realm = 13; // pid15 执卷境（顺带覆盖 pid10）
        let done = Game::chapter_done_mask(&g.meta);
        assert_eq!(done >> 8 & 1, 1, "前置：章 9 两目标全了");
        // 前七/八章位按事实先补齐隔离（best_realm=13 同时点亮章七 pid10、endings
        // 点亮章八 pid11/13——newly 必须只剩 bit8，否则启句成对发出属前置漏算）
        g.meta.chap_done = done & !(1u32 << 8);
        g.check_chapters();
        assert!(g.log.iter().any(|l| l.text.contains("【第九章·了】")));
        assert!(
            !g.log.iter().any(|l| l.text.contains("·启】")),
            "末章了后不得再发启句"
        );
    }

    // done_mask 推进：首模拟→章1；斩敌→章3；首胜+首破境→章2；讨牒→章4
    // （位 i = 章 i+1，掩码按事实纯推导）
    #[test]
    fn chapter_mask_progression() {
        let mut g = game_with(Life::default(), None);
        // 全新档：转生目标未挂快照 → 章 1 未了
        assert_eq!(Game::chapter_done_mask(&g.meta), 0);
        // 开局+首模拟 → 章 1 了（bit0；goal12=lives≥1 复审修后口径）
        g.meta.lives = 1;
        g.meta.total_sims = 1;
        assert_eq!(Game::chapter_done_mask(&g.meta), 0b0000_0001);
        // 章 2 需首胜∧首破境；章 3 只需 neme_slain≥1∧≥2——斩敌二即章 3 了（bit2）
        g.meta.neme_slain = 2;
        assert_eq!(Game::chapter_done_mask(&g.meta), 0b0000_0101);
        g.meta.total_wins = 1;
        g.meta.first_realm[1] = true;
        assert_eq!(Game::chapter_done_mask(&g.meta), 0b0000_0111, "章 2 补齐");
        // 讨牒 6/6 → 章 4 了（bit3）；章 7 需首登仙界仍未了
        g.meta.taodie = 6;
        assert_eq!(Game::chapter_done_mask(&g.meta), 0b0000_1111);
        // 全事齐（含 v1.16.10 书外之章 pid15：best_realm≥13）→ 0x1FF
        g.meta.region_visited = [true; 5];
        g.meta.companion_lives = 1;
        g.meta.npc_ev[0] = true;
        g.meta.first_realm[10] = true;
        g.meta.endings_seen = 0b1111;
        assert_eq!(Game::chapter_done_mask(&g.meta), 0xFF, "前八章齐，书外章未启");
        g.meta.best_realm = 13;
        assert_eq!(Game::chapter_done_mask(&g.meta), 0x1FF, "书外之章（bit8）补齐");
    }

    // 边沿演出：事实先到、位掩码后补——首调置位+金行+横幅；再调幂等零重复
    #[test]
    fn check_chapters_edge_fires_once() {
        let mut g = game_with(Life::default(), None);
        g.meta.lives = 1;
        g.meta.total_sims = 1; // 章 1 事实已齐，位未置（模拟载入帧前的窗口）
        g.check_chapters();
        assert_eq!(g.meta.chap_done, 0b0000_0001, "章 1 位已置");
        assert!(
            g.log.iter().any(|l| l.text.contains("第一章") && l.text.contains("了")),
            "金行日志逐章留痕"
        );
        assert!(g.banner.is_some(), "达成横幅已挂");
        let log_len = g.log.len();
        g.check_chapters(); // 幂等：位已置不再演
        assert_eq!(g.meta.chap_done, 0b0000_0001);
        assert_eq!(g.log.len(), log_len, "边沿只触发一次");
    }

    // 多章同帧全了：日志逐章留痕、掩码全置（横幅单槽后写胜为既定演出语义）
    #[test]
    fn check_chapters_multi_chapter_same_edge() {
        let mut g = game_with(Life::default(), None);
        g.meta.lives = 1;
        g.meta.total_sims = 1;
        g.meta.total_wins = 1;
        g.meta.first_realm[1] = true;
        g.check_chapters();
        assert_eq!(g.meta.chap_done, 0b0000_0011, "章 1/2 同帧全了");
        assert_eq!(
            g.log.iter().filter(|l| l.text.contains("·了")).count(),
            2,
            "两章各留一行金行"
        );
    }

    // 载入帧静默补齐：旧档事实已齐（chap_done=0）→ 掩码补齐后 check_chapters 零演出
    #[test]
    fn load_catchup_silent_no_replay() {
        let mut g = game_with(Life::default(), None);
        g.meta.lives = 1;
        g.meta.total_sims = 1;
        g.meta.taodie = 6;
        g.meta.neme_slain = 6;
        // Game::new 载入帧同款补齐（掩码按事实静默置位）
        g.meta.chap_done |= Game::chapter_done_mask(&g.meta);
        assert_eq!(g.meta.chap_done, 0b0000_1101, "章 1/3/4（bit0/2/3）静默补齐");
        let log_len = g.log.len();
        g.check_chapters();
        assert_eq!(g.log.len(), log_len, "补齐不重播横幅/日志");
    }

    // 当前章推进：全了钳末章（章节书右栏恒有内容可看；当前章按事实掩码推导，
    // 不读 chap_done——位掩码是事实的投影，单一数据源）
    #[test]
    fn chapter_current_advances_and_clamps() {
        let mut m = Meta::default();
        m.lives = 1;
        m.total_sims = 1;
        assert_eq!(Game::chapter_current(&m), 1, "章 1 了 → 当前章 2");
        // 全事齐（走事实而非置位）→ 钳末章（v1.16.10 起末章=书外之章）
        m.neme_slain = 2;
        m.total_wins = 1;
        m.first_realm[1] = true;
        m.taodie = 6;
        m.region_visited = [true; 5];
        m.companion_lives = 1;
        m.npc_ev[0] = true;
        m.first_realm[10] = true;
        m.endings_seen = 0b1111;
        m.best_realm = 13; // 书外之章 pid15（执卷境）
        assert_eq!(
            Game::chapter_current(&m),
            CHAPTERS.len() - 1,
            "全了钳末章"
        );
        let fresh = Meta::default();
        assert_eq!(Game::chapter_current(&fresh), 0, "全新档当前章 1");
    }

    // v1.16.9 章成之礼：边沿发放幂等——首调 dao_yun 加 reward 且金行含奖励段；
    // 再调（chap_done 位已置）零补发零重复 log。发放只动 meta.dao_yun（life=None
    // 帧可达，禁现世资源），发放与演出同处 newly 臂、chap_done 位即领取凭证。
    #[test]
    fn chapter_reward_edge_pays_once() {
        let mut g = game_with(Life::default(), None);
        g.meta.lives = 1;
        g.meta.total_sims = 1; // 章 1 事实已齐，位未置
        let dy0 = g.meta.dao_yun;
        g.check_chapters();
        let r0 = CHAPTERS[0].reward;
        assert_eq!(
            g.meta.dao_yun,
            dy0 + r0 as i64,
            "章成之礼按表发放一次"
        );
        assert!(
            g.log.iter().any(|l| l.text.contains("道韵 +") && l.text.contains("第一章")),
            "金行 log 携奖励段（横幅单槽吞字由日志兜底）"
        );
        let log_len = g.log.len();
        g.check_chapters(); // 幂等：位已置不再演不再发
        assert_eq!(g.meta.dao_yun, dy0 + r0 as i64, "补调零补发");
        assert_eq!(g.log.len(), log_len, "奖励 log 不重播");
    }

    // 多章同帧全了：dao_yun 加两章之和，逐章 log 各携奖励段（横幅单槽后写胜为既定
    // 演出语义，奖励信息不依赖横幅）
    #[test]
    fn chapter_reward_multi_same_edge_sums() {
        let mut g = game_with(Life::default(), None);
        g.meta.lives = 1;
        g.meta.total_sims = 1;
        g.meta.total_wins = 1;
        g.meta.first_realm[1] = true;
        let dy0 = g.meta.dao_yun;
        g.check_chapters();
        let expect = CHAPTERS[0].reward as i64 + CHAPTERS[1].reward as i64;
        assert_eq!(g.meta.dao_yun, dy0 + expect, "两章奖励同帧加总");
        let pays = g
            .log
            .iter()
            .filter(|l| l.text.contains("道韵 +"))
            .count();
        assert_eq!(pays, 2, "逐章金行各携奖励段");
    }

    // 奖励表静态一致性：reward 克制梯度（50..=500，v1.16.10 书外之章 +500）、
    // reward_note 携「道韵 +N」且数字与 reward 一致（两处手改漂移防线）、
    // lore 非空且每段带序号
    #[test]
    fn chapter_reward_table_consistent() {
        for ch in CHAPTERS.iter() {
            assert!(
                (50..=500).contains(&ch.reward),
                "{} 奖励越梯度 50..=500: {}",
                ch.num,
                ch.reward
            );
            let num = ch.reward.to_string();
            assert!(
                ch.reward_note.contains("道韵 +") && ch.reward_note.contains(&num),
                "{} reward_note 与 reward 数字不一致: {} vs {}",
                ch.num,
                ch.reward_note,
                ch.reward
            );
            assert!(
                !ch.lore.is_empty() && ch.lore.len() <= 3,
                "{} lore 段数 1..=3",
                ch.num
            );
            for seg in ch.lore.iter() {
                assert!(seg.contains("其"), "{} lore 段缺序号", ch.num);
            }
        }
    }

    // v1.16.19 章成金行副句表一致性（照 reward_note 一致性先例）：DONE_NOTE 与
    // CHAPTERS 等长（check_chapters 下标直查）、每句非空 ≤16 字、句句互异（防
    // 九连同句回归——本表存在动机即去重）
    #[test]
    fn done_note_table_aligns_chapters() {
        assert_eq!(DONE_NOTE.len(), CHAPTERS.len(), "副句表与章表等长");
        for (i, note) in DONE_NOTE.iter().enumerate() {
            let n = note.chars().count();
            assert!(!note.is_empty() && n <= 16, "章{} 副句越 16 字预算: {} ({}字)", i + 1, note, n);
        }
        for (a, na) in DONE_NOTE.iter().enumerate() {
            for (b, nb) in DONE_NOTE.iter().enumerate().skip(a + 1) {
                assert_ne!(na, nb, "章{} 与章{} 副句撞句: {}", a + 1, b + 1, na);
            }
        }
    }

    // v1.16.19 结算收束语三档：境界档映射钉死（0..=4 / 5..=9 / 10..=14 / 脏值钳
    // 14）+ 每档非空 ≤20 字 + 档间互异（静态 &'static str，settle 帧零分配红线）
    #[test]
    fn settle_coda_tiers_pinned() {
        let lo = settle_coda(0);
        let mid = settle_coda(5);
        let hi = settle_coda(10);
        assert_eq!(settle_coda(4), lo, "档界 0..=4 同句");
        assert_eq!(settle_coda(9), mid, "档界 5..=9 同句");
        assert_eq!(settle_coda(14), hi, "档界 10..=14 同句");
        assert_eq!(settle_coda(99), hi, "脏 realm 钳 14 不越表");
        assert_ne!(lo, mid);
        assert_ne!(mid, hi);
        assert_ne!(lo, hi);
        for s in [lo, mid, hi] {
            let n = s.chars().count();
            assert!(!s.is_empty() && n <= 20, "收束语越 20 字预算: {} ({}字)", s, n);
        }
    }

    // 进度数值单口与谓词单口同口一致：Some((x,y)) ⟹ x≤y 且 (x≥y)==goal_met；
    // v1.16.16 全 pid 数值化（0..=15 全有值），仅脏 pid（16+）恒 None
    #[test]
    fn chapter_goal_prog_matches_goal_met() {
        let mut g = game_with(Life::default(), None);
        g.meta.taodie = 2;
        g.meta.neme_slain = 1;
        g.meta.region_visited = [true, true, true, false, false];
        assert_eq!(Game::chapter_goal_prog(&g.meta, 4), Some((2, 3)));
        assert_eq!(Game::chapter_goal_prog(&g.meta, 5), Some((2, 6)));
        assert_eq!(Game::chapter_goal_prog(&g.meta, 3), Some((1, 1)));
        assert_eq!(
            Game::chapter_goal_prog(&g.meta, 14),
            Some((1, 2)),
            "neme_slain=1：pid3 已毕 pid14 半程（同源两档，8986 注释口径）"
        );
        assert_eq!(Game::chapter_goal_prog(&g.meta, 6), Some((3, 3)));
        assert_eq!(Game::chapter_goal_prog(&g.meta, 7), Some((3, 5)));
        // v1.16.16 原 None 清单（0/1/2/8/9/10/11/12/13/15）全部数值化（缺省
        // meta 下全 0 计：布尔型 (0,1)、分母型 (0,N)）
        assert_eq!(Game::chapter_goal_prog(&g.meta, 0), Some((0, 1)));
        assert_eq!(Game::chapter_goal_prog(&g.meta, 1), Some((0, 1)));
        assert_eq!(Game::chapter_goal_prog(&g.meta, 2), Some((0, 1)));
        assert_eq!(Game::chapter_goal_prog(&g.meta, 8), Some((0, 1)));
        assert_eq!(Game::chapter_goal_prog(&g.meta, 9), Some((0, 1)));
        assert_eq!(Game::chapter_goal_prog(&g.meta, 10), Some((0, 10)));
        assert_eq!(Game::chapter_goal_prog(&g.meta, 11), Some((0, 1)));
        assert_eq!(Game::chapter_goal_prog(&g.meta, 12), Some((0, 1)));
        assert_eq!(Game::chapter_goal_prog(&g.meta, 13), Some((0, 4)));
        assert_eq!(Game::chapter_goal_prog(&g.meta, 15), Some((0, 13)));
        // 位快照领先 best_realm 折算口径：fr[10]=true、best=3 → pid10 x 折满 10
        // （met 已真，x≥y 保持同口）；fr[1]=true → pid2 折满 1
        g.meta.best_realm = 3;
        g.meta.first_realm[10] = true;
        g.meta.first_realm[1] = true;
        assert_eq!(Game::chapter_goal_prog(&g.meta, 10), Some((10, 10)));
        assert_eq!(Game::chapter_goal_prog(&g.meta, 2), Some((1, 1)));
        g.meta.first_realm[10] = false;
        g.meta.first_realm[1] = false;
        assert_eq!(
            Game::chapter_goal_prog(&g.meta, 10),
            Some((3, 10)),
            "位快照撤销后回落 best_realm 渐进值"
        );
        assert_eq!(Game::chapter_goal_prog(&g.meta, 15), Some((3, 13)));
        // 全 pid 同口一致性：有值 ⟹ x≤y 且 met ⟺ x≥y；脏 pid 两口都落空臂
        for pid in 0u8..=16 {
            let met = Game::chapter_goal_met(&g.meta, pid);
            match Game::chapter_goal_prog(&g.meta, pid) {
                Some((x, y)) => {
                    assert!(x <= y);
                    assert_eq!(x >= y, met, "pid {pid} prog/met 口径漂移");
                }
                None => assert!(pid >= 16, "pid {pid} 全数值化后不该为 None"),
            }
        }
        // 推进后仍同口：neme_slain=2 → pid14 x≥y 翻真、met 翻真
        g.meta.neme_slain = 2;
        assert_eq!(Game::chapter_goal_prog(&g.meta, 14), Some((2, 2)));
        assert!(Game::chapter_goal_met(&g.meta, 14));
    }

    // v1.16.9 终局指路叶：问笔已达 ∧ 四结局未全 → 叶 2（压修为/经济叶）；四结局全 →
    // 落回原链（模拟点已足叶 9）；开局未模拟时里程碑叶 1 压终局叶（链首语义保持）
    #[test]
    fn guide_finale_leaf_routes_endgame() {
        let mut g = game_with(Life::default(), None);
        g.meta.total_sims = 1;
        g.meta.first_realm[10] = true; // 现实问笔已达成
        assert_eq!(
            crate::guide::guide_index(&g),
            2,
            "问笔后四结局未全 → 终局叶"
        );
        g.meta.endings_seen = 0b1111;
        assert_eq!(
            crate::guide::guide_index(&g),
            12,
            "四结局全了 → 落回原链（P=3 模拟点已足叶，v1.16.16 两章进行叶插入后 10→12）"
        );
        // total_sims==0 且问笔已达成：里程碑叶压终局叶（开局推演优先一切）
        let mut g = game_with(Life::default(), None);
        g.meta.first_realm[10] = true;
        assert_eq!(
            crate::guide::guide_index(&g),
            1,
            "里程碑叶压终局叶"
        );
    }

    // 终局叶按钮接线：Book 跳转 → 指引面板+章节书签（横带场景开面板；面板内场景
    // 只切签不关面板——draw 侧归零例外的前提是 exec_jump 不重复置 panel）
    #[test]
    fn exec_jump_book_opens_chapter_tab() {
        // 横带场景（panel=0）：开面板直落章节书签
        let mut g = game_with(Life::default(), None);
        crate::guide::exec_jump(&mut g, crate::guide::GuideAct::Book);
        assert_eq!((g.panel, g.guide_tab), (6, 1));
        // 面板内场景（panel=6）：只切签，面板保持
        let mut g = game_with(Life::default(), None);
        g.panel = 6;
        g.guide_tab = 0;
        crate::guide::exec_jump(&mut g, crate::guide::GuideAct::Book);
        assert_eq!((g.panel, g.guide_tab), (6, 1), "面板内只切签不关面板");
    }
}

// ---------------- v1.16.10 章节称号·雨夜执印者 + 新手一次性提示三闩 ----------------

#[cfg(test)]
mod v11610_tests {
    use super::revenge_tests::game_with;
    use super::*;

    // 章节称号授权链：首章未了 → 佩戴拒绝（worn 不变）；check_chapters 首章边沿 →
    // 「可佩」toast 演出 + 佩戴放行 + 显示口 worn_title_of 即时回读；重调幂等零重复
    #[test]
    fn chapter_title_earned_by_first_chapter() {
        let mut g = game_with(Life::default(), None);
        // 首章未了：值 16 越权拒绝（worn 保持默认 -1 自动跟随）
        g.set_worn_title(16);
        assert_eq!(g.meta.worn_title, -1, "首章未了不得佩戴章节称号");
        // 首章事实齐（转生+首模拟）、位未置 → 边沿置位 + 称号可佩演出
        g.meta.lives = 1;
        g.meta.total_sims = 1;
        g.check_chapters();
        assert_eq!(g.meta.chap_done & 1, 1, "首章位已置");
        assert!(
            g.toast
                .as_ref()
                .map_or(false, |(t, _, _)| t.contains("雨夜执印者")),
            "首章边沿演「称号可佩」toast"
        );
        // 授权后佩戴放行，显示口即时生效（三处显示位经 worn_title_of 共用本口）
        g.set_worn_title(16);
        assert_eq!(g.meta.worn_title, 16, "chap_done bit0 授权佩戴");
        assert_eq!(g.worn_title_of(), "雨夜执印者", "显示口回读章节称号");
        // 幂等：位已置重调零重复（toast 槽不变——不重演）
        let log_len = g.log.len();
        g.check_chapters();
        assert_eq!(g.log.len(), log_len, "边沿只触发一次");
        assert_eq!(g.worn_title_of(), "雨夜执印者", "幂等后佩戴不变");
    }

    // 章节称号与境界称号互不越权：值 15（弑神者）授权面不受 bit0 影响；脏值钳制
    // 由 save.rs 回环测试覆盖（-1..=16 之外钳回 -1）
    #[test]
    fn title15_authorization_unaffected() {
        let mut g = game_with(Life::default(), None);
        g.meta.lives = 1;
        g.meta.total_sims = 1;
        g.check_chapters(); // bit0 置位
        g.set_worn_title(15);
        assert_eq!(g.meta.worn_title, -1, "弑神者仍需 neme_slain≥3，bit0 不越权");
        g.meta.neme_slain = 3;
        g.set_worn_title(15);
        assert_eq!(g.meta.worn_title, 15, "弑神者原授权面不变");
    }

    // 新手一次性提示三闩：bit2 首入现实（仅首世）/ bit3 首次传承带回 / bit4 首修为
    // 圆满——各置闩一次、log+toast 双通道、重调幂等不重播
    #[test]
    fn sys_hints_latch_once() {
        // bit2：begin_mortal 首世（lives==1）置闩；次世不重播（begin_mortal 清 log，
        // 重播断言看新 log 不再含「金框」）
        let mut g = game_with(Life::default(), None);
        g.begin_mortal();
        assert_eq!(g.meta.sys_seen & 0b0100, 0b0100, "首世入现实置闩");
        assert!(g.log.iter().any(|l| l.text.contains("金框")), "金行指路");
        assert!(
            g.toast
                .as_ref()
                .map_or(false, |(t, _, _)| t.contains("第 1 世")),
            "首入现实 toast"
        );
        g.begin_mortal(); // 次世：闩已置
        assert!(!g.log.iter().any(|l| l.text.contains("金框")), "次世不重播");

        // bit3：首次传承带回置闩；重调幂等（finish_inherit 本体每调必打「大梦初醒」
        // 行，幂等断言只数闩的提示行）
        let mut g2 = game_with(Life::default(), None);
        g2.begin_mortal();
        g2.finish_inherit();
        assert_eq!(g2.meta.sys_seen & 0b1000, 0b1000, "首次传承置闩");
        assert!(
            g2.toast
                .as_ref()
                .map_or(false, |(t, _, _)| t.contains("造化已入现实")),
            "传承带回 toast"
        );
        let n1 = g2
            .log
            .iter()
            .filter(|l| l.text.contains("造化已入此身"))
            .count();
        g2.finish_inherit();
        let n2 = g2
            .log
            .iter()
            .filter(|l| l.text.contains("造化已入此身"))
            .count();
        assert_eq!((n1, n2), (1, 1), "bit3 幂等不重播");

        // bit4：修为首满置闩（sys_hint_break_ready 单口=draw_reality 帧头同口）
        let mut g3 = game_with(Life::default(), None);
        g3.begin_mortal(); // 顺带验 bit2 已闩不干扰
        assert_eq!(g3.meta.sys_seen & 0b10000, 0, "修为未满不置闩");
        {
            let l = g3.life.as_mut().unwrap();
            l.qi = qi_need(l);
        }
        g3.sys_hint_break_ready();
        assert_eq!(g3.meta.sys_seen & 0b10000, 0b10000, "修为首满置闩");
        assert!(
            g3.toast
                .as_ref()
                .map_or(false, |(t, _, _)| t.contains("修炼")),
            "修为圆满 toast"
        );
        let log_len = g3.log.len();
        g3.sys_hint_break_ready();
        assert_eq!(g3.log.len(), log_len, "bit4 幂等不重播");
    }

    // 书外之章边沿：第九章目标（pid13 四卷全览 + pid15 执卷境）齐 → bit8 置位 +
    // 奖励边沿发放；仅 pid13 齐（第八章末态）→ 章九进行态（当前章钳章九）
    #[test]
    fn book_out_chapter_edge_completes() {
        let mut g = game_with(Life::default(), None);
        // 前八章全齐的事实集（掩码 bit0..=7），位未置
        g.meta.lives = 1;
        g.meta.total_sims = 1;
        g.meta.total_wins = 1;
        g.meta.first_realm[1] = true;
        g.meta.neme_slain = 2;
        g.meta.taodie = 6;
        g.meta.region_visited = [true; 5];
        g.meta.companion_lives = 1;
        g.meta.npc_ev[0] = true;
        g.meta.first_realm[10] = true;
        g.meta.endings_seen = 0b1111;
        assert_eq!(Game::chapter_done_mask(&g.meta), 0xFF);
        assert_eq!(Game::chapter_current(&g.meta), 8, "第八章末态 → 当前章九");
        // pid15 未达：章九不完成
        g.check_chapters();
        assert_eq!(g.meta.chap_done, 0xFF, "书外之章未了");
        // best_realm 抵执卷 → 章九边沿：bit8 置位 + 奖励发放
        let dy0 = g.meta.dao_yun;
        g.meta.best_realm = 13;
        g.check_chapters();
        assert_eq!(g.meta.chap_done, 0x1FF, "书外之章（bit8）边沿置位");
        assert_eq!(
            g.meta.dao_yun,
            dy0 + crate::data::CHAPTERS[8].reward as i64,
            "章成之礼按表发放"
        );
        assert!(
            g.log
                .iter()
                .any(|l| l.text.contains("第九章") && l.text.contains("了")),
            "金行逐章留痕"
        );
    }
}

// ---------------- v1.16.0 交游图鉴：gallery_bond 卫语句与封顶 ----------------

#[cfg(test)]
mod gallery_tests {
    use super::revenge_tests::game_with;
    use super::*;

    // idx 越界（99 ≥ 10）不 panic 不写入（panic=abort 纪律：卫语句直接 return）
    #[test]
    fn gallery_bond_out_of_range_noop() {
        let mut g = game_with(Life::default(), None);
        g.gallery_bond(99);
        assert_eq!(g.meta.comp_bonds, [0; 10]);
    }

    // 正常记账：idx=3 计 1；重复结缘累加；他位不动
    #[test]
    fn gallery_bond_counts_and_accumulates() {
        let mut g = game_with(Life::default(), None);
        g.gallery_bond(3);
        assert_eq!(g.meta.comp_bonds[3], 1);
        g.gallery_bond(3);
        assert_eq!(g.meta.comp_bonds[3], 2);
        assert_eq!(g.meta.comp_bonds[0], 0);
    }

    // saturating 封顶 255（历世次数极限不回绕）
    #[test]
    fn gallery_bond_saturates_at_255() {
        let mut g = game_with(Life::default(), None);
        g.meta.comp_bonds[7] = 255;
        g.gallery_bond(7);
        assert_eq!(g.meta.comp_bonds[7], 255);
    }
}

// ---------------- v1.16.0 交游图鉴：结缘链路（special 17 → gallery_bond 当拍记账） ----------------

#[cfg(test)]
mod gallery_bond_chain_tests {
    use super::revenge_tests::game_with;
    use super::*;

    // 照模拟事件 28『同页天成』的「认下这页」选项构造（special=17，game.rs:2981-2993 同款）
    fn bond_card() -> EventCard {
        let mut e = Eff::new();
        e.special = 17;
        EventCard {
            kind: CardKind::Npc,
            actor: None,
            rare: false,
            stage: None,
            cat: None,
            scene: None,
            brk: false,
            title: "同页天成".into(),
            text: "两页相贴，缝作同页——可愿？".into(),
            opts: vec![("认下这页".into(), e)],
        }
    }

    // 结缘即记账：choose_option 当拍 comp_bonds 恰一位 +1（掷名随机落位，账在 meta）
    #[test]
    fn special17_records_bond_on_pick() {
        let mut g = game_with(Life::default(), None);
        g.sim = Some(Life::default());
        g.modal = Some(bond_card());
        g.choose_option(0);
        assert_eq!(g.meta.comp_bonds.iter().sum::<u8>(), 1);
        assert!(g.sim.as_ref().unwrap().companion.is_some());
    }

    // 重掷不回退：quit_sim_redraft（点数已还、sim 清空）后历世账不回退——
    // 这正是挂结缘点而非 sim_die 的原因（feng_bond_up 漏 quit 路径）
    #[test]
    fn bond_survives_quit_sim_redraft() {
        let mut g = game_with(Life::default(), None);
        g.scene = Scene::Sim;
        g.sim = Some(Life::default());
        g.modal = Some(bond_card());
        g.choose_option(0);
        let total = g.meta.comp_bonds.iter().sum::<u8>();
        assert!(total >= 1);
        g.quit_sim_redraft();
        assert!(g.sim.is_none());
        assert_eq!(g.meta.comp_bonds.iter().sum::<u8>(), total, "重掷不回退历世账");
    }
}

// ================= v1.16.5 模拟事件扩池 + 防重复排除池 =================

#[cfg(test)]
mod recent_event_tests {
    use super::revenge_tests::game_with;
    use super::*;

    fn sim_game(realm: usize, luck: f32) -> Game {
        let mut g = game_with(Life::default(), None);
        let mut s = Life::default();
        s.realm = realm;
        s.luck = luck;
        g.sim = Some(s);
        g
    }

    // K 窗口不变式：连抽 300 次，recent_events 恒 ≤K 且窗口内无重复 id
    #[test]
    fn no_repeat_within_k_window() {
        let mut g = sim_game(5, 100.0);
        for _ in 0..300 {
            g.modal = None;
            g.sim_event();
            assert!(g.modal.is_some(), "realm5 大池不该退化到无卡可出");
            assert!(g.recent_events.len() <= RECENT_EVENT_K, "窗口恒 ≤K");
            let mut seen = g.recent_events.clone();
            seen.sort_unstable();
            seen.dedup();
            assert_eq!(seen.len(), g.recent_events.len(), "K 窗口内不得重复");
        }
    }

    // 排除后小池全灭 → 退化：清空排除池重抽（而非旧逻辑静默无卡）
    #[test]
    fn degenerate_pool_clears_and_draws() {
        let mut g = sim_game(0, 40.0);
        g.recent_events = (0..EVENTS.len()).collect(); // 最坏情形：全池被排除
        g.modal = None;
        g.sim_event();
        assert!(g.modal.is_some(), "全排除退化后仍须出卡");
        assert_eq!(g.recent_events.len(), 1, "退化清空后仅记录本次");
    }

    // 本世口径：launch_sim 必须清空排除池（照 blank_used_resets_per_sim 先例）
    #[test]
    fn recent_events_reset_per_sim() {
        let mut g = game_with(Life::default(), None);
        g.picked = vec![0]; // launch_sim 卫语句
        g.meta.sim_points = 5;
        g.recent_events.push(3);
        g.launch_sim();
        assert!(g.recent_events.is_empty(), "launch_sim 必须清空排除池");
    }

    // 新事件卡齐全：id 31..=43 每条 ≥2 且 ≤3 选项、标题正文非空，选第一项不 panic
    #[test]
    fn new_event_cards_well_formed() {
        let mut g = game_with(Life::default(), None);
        for id in 31..=43 {
            // 每轮重新置稿：fight 类选项 8% 战死（sim_fight→sim_die 置 sim=None），
            // 不重置则下一轮 build_event 的 sim.unwrap() 必炸
            let mut s = Life::default();
            s.realm = 5;
            s.luck = 40.0;
            g.sim = Some(s);
            g.modal = None;
            g.build_event(id);
            let card = g
                .modal
                .as_ref()
                .unwrap_or_else(|| panic!("事件 {} 无卡臂（白抽）", id));
            assert!(matches!(card.kind, CardKind::Npc));
            assert!(!card.title.is_empty());
            assert!(!card.text.is_empty());
            assert!(card.opts.len() >= 2, "事件 {} 至少两个选项", id);
            assert!(card.opts.len() <= 3, "卡面几何至多三选项（draw.rs:1706）");
            g.choose_option(0); // apply_eff 全链不 panic（sync_save 仅置脏不落盘）
        }
    }
}

// ================= v1.16.7 连锁剧情 + 扩池 15 条（id 44..=58） =================

#[cfg(test)]
mod chain_event_tests {
    use super::revenge_tests::game_with;
    use super::*;

    fn sim_game(realm: usize, luck: f32) -> Game {
        let mut g = game_with(Life::default(), None);
        let mut s = Life::default();
        s.realm = realm;
        s.luck = luck;
        g.sim = Some(s);
        g
    }

    // 链1 全程：首段 44 置链待分支 → 赊墨（special20）置下段 → 直取 46 →
    // 抉择（无置链臂）即链终局
    #[test]
    fn chain1_advances_and_closes() {
        let mut g = sim_game(5, 100.0);
        g.modal = None;
        g.build_event(44);
        assert_eq!(g.chain, Some((1, 0, CHAIN_TTL)), "首段臂置链待分支");
        g.choose_option(0); // 赊墨立契
        assert_eq!(g.chain, Some((1, 46, CHAIN_TTL)), "置链臂写下段");
        g.modal = None;
        g.sim_event(); // 链活跃 → 直取 46，跳过轮盘
        assert_eq!(g.modal.as_ref().unwrap().title, "恩怨上门·续");
        g.choose_option(1); // 十倍赔付（无 special）
        assert!(g.chain.is_none(), "终结段抉择即清链");
    }

    // 链1 分支b：婉拒不买（special21）→ 直取 45
    #[test]
    fn chain1_branch_b() {
        let mut g = sim_game(5, 100.0);
        g.modal = None;
        g.build_event(44);
        g.choose_option(1);
        assert_eq!(g.chain, Some((1, 45, CHAIN_TTL)));
        g.modal = None;
        g.sim_event();
        assert_eq!(g.modal.as_ref().unwrap().title, "罗盘相赠·续");
        g.choose_option(0);
        assert!(g.chain.is_none());
    }

    // 链2 全程：47 → 48 → 49，special4 检定臂落通用链裁决 → 链终局
    #[test]
    fn chain2_advances_to_keeper() {
        let mut g = sim_game(5, 100.0);
        g.modal = None;
        g.build_event(47);
        assert_eq!(g.chain, Some((2, 0, CHAIN_TTL)));
        g.choose_option(0); // 以血引墨（special22）
        assert_eq!(g.chain, Some((2, 48, CHAIN_TTL)));
        g.modal = None;
        g.sim_event();
        assert_eq!(g.modal.as_ref().unwrap().title, "拓印风波·续");
        g.choose_option(0); // 随他见人（special23）
        assert_eq!(g.chain, Some((2, 49, CHAIN_TTL)));
        g.modal = None;
        g.sim_event();
        assert_eq!(g.modal.as_ref().unwrap().title, "守碑人·续");
        g.choose_option(0); // 受此三问（special4，非置链臂）
        assert!(g.chain.is_none(), "检定臂也是链终局");
    }

    // 断链自洽：47 选覆土掩碑（非推链）→ 立即清链，此后 sim_event 不再直取
    #[test]
    fn chain2_broken_option_closes_chain() {
        let mut g = sim_game(5, 100.0);
        g.modal = None;
        g.build_event(47);
        g.choose_option(1); // 覆土掩碑
        assert!(g.chain.is_none(), "非推链选项即链终局");
        g.modal = None;
        g.recent_events = vec![44, 47, 50]; // 排除首段，杜绝弃链后偶发重建链干扰断言
        g.sim_event(); // 不应抽出链内段（权重 0）
        let card = g.modal.as_ref().unwrap();
        assert_ne!(card.title, "拓印风波·续");
        assert_ne!(card.title, "守碑人·续");
        assert!(g.chain.is_none());
    }

    // 链3 全程：50 → 51 → 52，special12 参悟入大事记
    #[test]
    fn chain3_full_run_and_highlight() {
        let mut g = sim_game(5, 100.0);
        g.modal = None;
        g.build_event(50);
        assert_eq!(g.chain, Some((3, 0, CHAIN_TTL)));
        g.choose_option(0);
        g.modal = None;
        g.sim_event();
        assert_eq!(g.modal.as_ref().unwrap().title, "渔村释疑·续");
        g.choose_option(0);
        g.modal = None;
        g.sim_event();
        assert_eq!(g.modal.as_ref().unwrap().title, "匣底真迹·续");
        let hl0 = g.sim.as_ref().unwrap().highlights.len();
        g.choose_option(0); // 参悟真迹（special12）
        assert!(g.chain.is_none());
        assert!(
            g.sim.as_ref().unwrap().highlights.len() > hl0,
            "参悟入大事记"
        );
    }

    // ttl 保险丝：预算耗尽（ttl<=1）→ 静默弃链，落正常轮盘照常出卡
    #[test]
    fn chain_ttl_expires_silently() {
        let mut g = sim_game(5, 100.0);
        g.chain = Some((1, 46, 1));
        g.modal = None;
        // 排除全部链首段（CHAIN_FIRSTS 表派生），杜绝弃链后偶发重建链干扰断言——
        // 旧清单只硬编码链 1/2/3 首段，轮盘掷中任一未排除首段（链4..链16 共
        // 13 段在 realm5 池内）即 build_event 建链令 chain 断言偶发红（v1.16.20
        // 扩池 219 入池后命中面增大、门禁轮实测触发）；contains 判定不依赖
        // RECENT_EVENT_K 截断，超 K 长度清单对本测试语义无副作用
        g.recent_events = CHAIN_FIRSTS.iter().map(|&(eid, _)| eid).collect();
        g.sim_event();
        assert!(g.chain.is_none(), "预算耗尽弃链");
        assert!(g.modal.is_some(), "弃链后落正常轮盘");
    }

    // 直取即消费下段（置 0，链id 保留）：本段卡臂异常时不会死循环重复出卡
    #[test]
    fn chain_advance_consumes_next() {
        let mut g = sim_game(5, 100.0);
        g.chain = Some((1, 46, CHAIN_TTL));
        g.modal = None;
        g.sim_event();
        assert_eq!(g.chain, Some((1, 0, CHAIN_TTL - 1)), "下段消费为 0");
        assert_eq!(g.modal.as_ref().unwrap().title, "恩怨上门·续");
    }

    // 缺卡臂防御：下段 id 无臂（白抽）→ 弃链防死循环
    #[test]
    fn chain_missing_arm_breaks_safely() {
        let mut g = sim_game(5, 100.0);
        // v1.16.18 门禁修复：幽灵样本历史两次失效——v1.16.9 用 99（当时表外）、
        // 扩池后 99 有真实卡臂改用 199，v1.16.18 扩池 193..=204 后 199（旧邻问名）
        // 又成真实卡臂致本测假红。改为表尾+7 表达式自维护：恒表外恒无 build_event
        // 臂（三表同长契约决定臂只随表行走），扩池免疫不再顺延
        let ghost = EVENTS.len() + 7; // 212（u8 界内 255）
        g.chain = Some((9, ghost as u8, CHAIN_TTL));
        g.modal = None;
        g.sim_event();
        assert!(g.chain.is_none(), "缺臂链段弃链");
        assert!(g.modal.is_none(), "白抽不出卡（opts 空守卫）");
    }

    // 跨世不泄：launch_sim / quit_sim_redraft 清链
    #[test]
    fn chain_cleared_by_launch_and_quit() {
        let mut g = game_with(Life::default(), None);
        g.picked = vec![0]; // launch_sim 卫语句
        g.meta.sim_points = 5;
        g.chain = Some((1, 46, CHAIN_TTL));
        g.launch_sim();
        assert!(g.chain.is_none(), "launch_sim 必须清链");

        let mut g2 = game_with(Life::default(), None);
        g2.scene = Scene::Sim;
        g2.sim = Some(Life::default());
        g2.chain = Some((2, 48, CHAIN_TTL));
        g2.quit_sim_redraft();
        assert!(g2.chain.is_none(), "重掷必须清链");
    }

    // 链与防重复池：直取照常入 K 窗口，先去重（同世复走同链不在窗口叠重复 id）
    #[test]
    fn chain_advance_respects_recent_window() {
        let mut g = sim_game(5, 100.0);
        g.recent_events = vec![46, 10, 11, 12, 13, 14]; // 46 已在窗口
        g.chain = Some((1, 46, CHAIN_TTL));
        g.modal = None;
        g.sim_event();
        assert_eq!(
            g.recent_events.iter().filter(|&&x| x == 46).count(),
            1,
            "窗口内 46 恰一份"
        );
        assert!(g.recent_events.len() <= RECENT_EVENT_K);
        assert!(g.recent_events.last() == Some(&46), "去重后重入窗口尾");
    }

    // 伴随事件入池条件：无伴权重 0，有伴 >0（realm 达标时）
    // v1.16.19 门禁修：212 相扶看雪为老年×有伴双门控（白名单先行无伴归零，
    // 余量≤10 载交集在 weights id==212 单 if）——正臂对该条需同开老年态才入池
    #[test]
    fn companion_events_gated_by_companion() {
        let mut g = sim_game(5, 100.0); // 无伴
        let (ws, total) = g.sim_event_weights(100.0, 5, false, false, false);
        for &id in COMPANION_EVENTS.iter() {
            let i = EVENTS.iter().position(|&(e, _, _, _)| e == id).unwrap();
            assert_eq!(ws[i], 0.0, "无伴时事件 {} 不入池", id);
        }
        let _ = total;
        {
            let s = g.sim.as_mut().unwrap();
            s.lifespan = s.age + 5.0; // 老年态（仅 212 双门控消费，他条不受累）
        }
        // v1.16.20 门禁修：225 同数流星 mr6 层——正臂统一抬 realm 7（realm≥mr
        // 全白名单成立；负臂无伴归零与 realm 无涉保持 5）
        let (ws2, _) = g.sim_event_weights(100.0, 7, false, false, true);
        for &id in COMPANION_EVENTS.iter() {
            let i = EVENTS.iter().position(|&(e, _, _, _)| e == id).unwrap();
            assert!(ws2[i] > 0.0, "有伴时事件 {} 入池", id);
        }
    }

    // 链内段恒不入正常池（EVENTS 表权重 0，任何参数下）
    #[test]
    fn chain_only_ids_never_in_pool() {
        let g = sim_game(5, 100.0);
        let (ws, _) = g.sim_event_weights(100.0, 5, false, false, true);
        for id in [45, 46, 48, 49, 51, 52] {
            let i = EVENTS.iter().position(|&(e, _, _, _)| e == id).unwrap();
            assert_eq!(ws[i], 0.0, "链内段 {} 权重恒 0", id);
        }
    }

    // 稀有层：新稀有事件低权重（≤2）且入池
    #[test]
    fn rare_events_low_weight() {
        let g = sim_game(5, 100.0);
        let (ws, _) = g.sim_event_weights(100.0, 5, false, false, true);
        for id in [56, 57, 58] {
            let i = EVENTS.iter().position(|&(e, _, _, _)| e == id).unwrap();
            assert!(ws[i] > 0.0, "稀有事件 {} 应入池", id);
            assert!(EVENTS[i].1 <= 2, "稀有层权重 ≤2");
        }
    }

    // 新批次良构：id 44..=58 每条 ≥2 且 ≤3 选项、标题正文非空，选第一项不 panic
    #[test]
    fn new_batch_cards_well_formed() {
        let mut g = game_with(Life::default(), None);
        for id in 44..=58 {
            // 每轮重新置稿：fight 类选项 8% 战死（sim_fight→sim_die 置 sim=None），
            // 不重置则下一轮 build_event 的 sim.unwrap() 必炸；伴随臂需道侣名
            let mut s = Life::default();
            s.realm = 5;
            s.luck = 40.0;
            s.companion = Some(COMPANION_NAMES[1].to_string());
            g.sim = Some(s);
            g.modal = None;
            g.build_event(id);
            let card = g
                .modal
                .as_ref()
                .unwrap_or_else(|| panic!("事件 {} 无卡臂（白抽）", id));
            assert!(matches!(card.kind, CardKind::Npc));
            assert!(!card.title.is_empty());
            assert!(!card.text.is_empty());
            assert!(card.opts.len() >= 2, "事件 {} 至少两个选项", id);
            assert!(card.opts.len() <= 3, "卡面几何至多三选项（draw.rs:1706）");
            g.choose_option(0); // apply_eff 全链不 panic（含置链臂/检定臂）
        }
    }

    // 客串立绘：货郎链挂 Npc(1)（万宝楼外柜/胡掌柜同门），伴随卡反查道侣名挂 Comp
    #[test]
    fn new_cards_actor_assignment() {
        let mut g = sim_game(5, 100.0);
        g.modal = None;
        g.build_event(44);
        assert!(matches!(
            g.modal.as_ref().unwrap().actor,
            Some(CardActor::Npc(1))
        ));
        let mut s2 = Life::default();
        s2.realm = 5;
        s2.companion = Some(COMPANION_NAMES[3].to_string());
        g.sim = Some(s2);
        g.modal = None;
        g.build_event(54);
        assert!(matches!(
            g.modal.as_ref().unwrap().actor,
            Some(CardActor::Comp(3))
        ));
    }

    // v1.16.7 story 拼接纯函数：v1.16.19 改「首条+尾条」取样（少年…；暮年…双段
    // 成句）、剥「N岁，」前缀、无前缀条目不误剥、32 字截断（旧稿行头像槽右移后
    // 双显示位收口预算，见 story_from_highlights 注）
    #[test]
    fn story_from_highlights_strips_prefix_and_caps() {
        let hs = vec![
            "31岁，遇【星坠荒原】".to_string(),
            "35岁，货郎奇缘·缘了".to_string(),
        ];
        assert_eq!(
            story_from_highlights(&hs).as_deref(),
            Some("遇【星坠荒原】；货郎奇缘·缘了")
        );
        // v1.16.19 首条+尾条取样：3 条世取首尾（中段近况让位开卷一笔）
        let hs3 = vec![
            "17岁，初入青云宗".to_string(),
            "52岁，斩妖得宝".to_string(),
            "98岁，寿终正寝".to_string(),
        ];
        assert_eq!(
            story_from_highlights(&hs3).as_deref(),
            Some("初入青云宗；寿终正寝")
        );
        // 单条世不重复拼同一条
        let hs1 = vec!["40岁，星坠荒原".to_string()];
        assert_eq!(story_from_highlights(&hs1).as_deref(), Some("星坠荒原"));
        // 无「N岁，」前缀条目原样保留（「与你相伴至最后一刻，含笑而逝」不被误剥）
        let hs2 = vec![
            "与你相伴至最后一刻，含笑而逝".to_string(),
        ];
        assert_eq!(
            story_from_highlights(&hs2).as_deref(),
            Some("与你相伴至最后一刻，含笑而逝")
        );
        // 超 32 字截断
        let long: String = "墨".repeat(40);
        let s = story_from_highlights(&[long]).unwrap();
        assert_eq!(s.chars().count(), 32);
        // 空 → None
        assert!(story_from_highlights(&[]).is_none());
    }

    // 链完成大事记：链3 末段抉择 →「沉墨遗匣·缘了」入本世大事记（story 数据源）
    #[test]
    fn chain3_completion_records_highlight() {
        let mut g = sim_game(5, 100.0);
        g.modal = None;
        g.build_event(50);
        g.choose_option(0);
        g.modal = None;
        g.sim_event();
        g.choose_option(0);
        g.modal = None;
        g.sim_event();
        let card = g.modal.as_ref().unwrap();
        assert_eq!(card.stage, Some((3, 3)), "末段 stage 达总段");
        g.choose_option(0);
        let hl = g.sim.as_ref().unwrap().highlights.join("|");
        assert!(hl.contains("沉墨遗匣·缘了"), "链完成记事缺失: {}", hl);
    }

    // 稀有遭遇记事：build_event 对 rare 卡 push「N岁，遇【名】」；非稀有卡不记
    #[test]
    fn rare_encounter_records_highlight() {
        let mut g = sim_game(5, 100.0);
        g.modal = None;
        g.build_event(56);
        assert!(g.modal.as_ref().unwrap().rare, "56 w=2 应判稀有");
        let hl = g.sim.as_ref().unwrap().highlights.join("|");
        assert!(hl.contains("遇【星坠荒原】"), "稀有遭遇记事缺失: {}", hl);
        let n = g.sim.as_ref().unwrap().highlights.len();
        g.modal = None;
        g.build_event(0); // w=12 普通卡
        assert!(!g.modal.as_ref().unwrap().rare);
        assert_eq!(g.sim.as_ref().unwrap().highlights.len(), n, "普通卡不记");
    }

    // rare/stage 置位口径：链首段 50 两者兼有；链内段 w=0 不算稀有；链1 两段式。
    // v1.16.8 扩：链4 三段（76/77/78）链5 双分支（79/80/81）、新稀有 72/73（w≤2）
    #[test]
    fn rare_and_stage_flags() {
        let mut g = sim_game(5, 100.0);
        for (id, rare, stage) in [
            (44usize, false, Some((1u8, 2u8))),
            (45, false, Some((2, 2))),
            (46, false, Some((2, 2))),
            (47, false, Some((1, 3))),
            (48, false, Some((2, 3))),
            (49, false, Some((3, 3))),
            (50, true, Some((1, 3))),
            (51, false, Some((2, 3))),
            (52, false, Some((3, 3))),
            (56, true, None),
            (57, true, None),
            (58, true, None),
            (72, true, None),
            (73, true, None),
            (76, false, Some((1, 3))),
            (77, false, Some((2, 3))),
            (78, false, Some((3, 3))),
            (79, false, Some((1, 2))),
            (80, false, Some((2, 2))),
            (81, false, Some((2, 2))),
            (85, false, Some((1, 3))),
            (86, false, Some((2, 3))),
            (87, false, Some((3, 3))),
            (112, false, Some((1, 3))),
            (113, false, Some((2, 3))),
            (114, false, Some((3, 3))),
            (115, false, Some((1, 3))),
            (116, false, Some((2, 3))),
            (117, false, Some((3, 3))),
            // v1.16.11 链9/链10 三段式
            (118, false, Some((1, 3))),
            (119, false, Some((2, 3))),
            (120, false, Some((3, 3))),
            (121, false, Some((1, 3))),
            (122, false, Some((2, 3))),
            (123, false, Some((3, 3))),
            // v1.16.16 链14 三段式 + 新稀有 179 赝丹局（w2）
            (175, false, Some((1, 3))),
            (176, false, Some((2, 3))),
            (177, false, Some((3, 3))),
            (179, true, None),
            (108, true, None),
            (59, false, None),
            (84, false, None),
            (92, false, None),
            (0, false, None),
            // v1.16.17 扩池抽检：散件全 (非稀有, 无段标)——w3/w4 均不入稀有区
            (183, false, None),
            (186, false, None),
            (187, false, None),
            (192, false, None),
            // v1.16.19 抽检：链15 三段式 + 213 w2 入稀有区 + mr7 散件非稀有
            (205, false, Some((1, 3))),
            (206, false, Some((2, 3))),
            (207, false, Some((3, 3))),
            (213, true, None),
            (208, false, None),
            (217, false, None),
            // v1.16.20 扩池抽检：链16 三段式 + 224 w2 入稀有区 + 散件非稀有
            (219, false, Some((1, 3))),
            (220, false, Some((2, 3))),
            (221, false, Some((3, 3))),
            (224, true, None),
            (222, false, None),
            (228, false, None),
            (230, false, None),
        ] {
            g.modal = None;
            g.build_event(id);
            let card = g.modal.as_ref().unwrap();
            assert_eq!(card.rare, rare, "id {} rare 口径", id);
            assert_eq!(card.stage, stage, "id {} stage 口径", id);
        }
    }

    // skip 模式断链防御：链首段两选项必都是推链项（option0 直推勘察约定）——
    // 44 的 opt0/opt1 均带置链臂
    #[test]
    fn chain1_first_seg_options_all_advance() {
        let mut g = sim_game(5, 100.0);
        g.modal = None;
        g.build_event(44);
        let card = g.modal.as_ref().unwrap();
        let s0 = card.opts[0].1.special;
        let s1 = card.opts[1].1.special;
        assert!(
            (CHAIN_SPECIAL_LO..=CHAIN_SPECIAL_HI).contains(&s0)
                && (CHAIN_SPECIAL_LO..=CHAIN_SPECIAL_HI).contains(&s1),
            "链1 首段两个选项都必须置链（skip 模式自动选 option0）"
        );
    }
}

// ================= v1.16.10 扩池 18 条（100..=117）+ 事件回顾图鉴 + 链7/8 =================

#[cfg(test)]
mod v11610_event_tests {
    use super::revenge_tests::game_with;
    use super::*;

    fn sim_game(realm: usize, luck: f32) -> Game {
        let mut g = game_with(Life::default(), None);
        let mut s = Life::default();
        s.realm = realm;
        s.luck = luck;
        g.sim = Some(s);
        g
    }

    // 新批次良构：id 100..=117 每条 ≥2 ≤3 选项、标题正文非空、choose_option(0)
    // 全链不 panic（照 new_event_cards_well_formed 骨架；fight 臂 8% 战死每轮重置稿）
    #[test]
    fn new_batch_cards_well_formed() {
        for id in 100..=117 {
            let mut g = sim_game(5, 100.0);
            g.modal = None;
            g.build_event(id);
            let card = g
                .modal
                .as_ref()
                .unwrap_or_else(|| panic!("事件 {} 无卡臂（白抽）", id));
            assert!(matches!(card.kind, CardKind::Npc));
            assert!(!card.title.is_empty());
            assert!(!card.text.is_empty());
            assert!(card.opts.len() >= 2, "事件 {} 至少两个选项", id);
            assert!(card.opts.len() <= 3, "卡面几何至多三选项",);
            g.choose_option(0); // apply_eff 全链不 panic（sync_save 仅置脏不落盘）
        }
    }

    // 标题双源对账（v1.16.10 图鉴铺设前提）：EVENT_NAMES[id] ⟺ build_event
    // card.title 逐 id 相等；EVENT_BRIEF 非空且 ≤22 字一行（手札行布局预算）
    #[test]
    fn event_names_match_card_titles() {
        assert_eq!(EVENT_NAMES.len(), EVENTS.len(), "名表与事件表同长");
        assert_eq!(EVENT_BRIEF.len(), EVENTS.len(), "简表与事件表同长");
        for &(id, _, _, _) in EVENTS.iter() {
            let mut g = sim_game(5, 100.0);
            g.modal = None;
            g.build_event(id);
            let card = g
                .modal
                .as_ref()
                .unwrap_or_else(|| panic!("事件 {} 无卡臂", id));
            assert_eq!(
                card.title, EVENT_NAMES[id],
                "事件 {} 标题双源漂移: card「{}」≠ 表「{}」",
                id, card.title, EVENT_NAMES[id]
            );
            let b = EVENT_BRIEF[id];
            assert!(
                !b.is_empty() && b.chars().count() <= 22,
                "事件 {} 简介越一行预算: {} ({}字)",
                id,
                b,
                b.chars().count()
            );
        }
    }

    // v1.16.19 前期高频卡简一批收口：id 0/1/3/4/5/6/12 画面化同批改的简一律
    // 非空 ≤18 字（全表实测量级，比既有 ≤22 单测更紧——开局必读行不许回涨）
    // v1.16.20 扩批：第二批 15 条（id 2/9/15/17/21/25/26/27/29/42/46/67/120/184/188
    // ——勘察实跑「最平 13 条硬记账腔」去重取样）同预算一并钉死，防回涨
    #[test]
    fn event_brief_front_batch_within_18() {
        for id in [
            0usize, 1, 3, 4, 5, 6, 12, 2, 9, 15, 17, 21, 25, 26, 27, 29, 42, 46, 67, 120, 184,
            188,
        ] {
            let b = EVENT_BRIEF[id];
            let n = b.chars().count();
            assert!(
                !b.is_empty() && n <= 18,
                "事件 {} 前段简越 18 字预算: {} ({}字)",
                id,
                b,
                n
            );
        }
    }

    // v1.16.19 前期高频卡正文画面化不加减段：id 0/1/3/4/5/6/12 卡面正文不含
    // 「\n」分段（邂逅版式 5 行判定预算按单段计，加段会翻行数——润色只改字
    // 不改段纪律的回归钉）。id 3 坊市淘宝按灵石两分支，富/穷两态各过一遍
    #[test]
    fn event_text_front_batch_single_segment() {
        // v1.16.20 扩批：id 2/9/18 第二批画面化三段同预算钉死（id 9 括注体并入
        // 破折号单段；id 2/18 静态臂，双灵石态无分支面）
        for id in [0usize, 1, 3, 4, 5, 6, 12, 2, 9, 18] {
            for stones in [0i64, 999_999_999] {
                let mut g = sim_game(5, 100.0);
                if let Some(s) = g.sim.as_mut() {
                    s.stones = stones;
                }
                g.modal = None;
                g.build_event(id);
                let card = g
                    .modal
                    .as_ref()
                    .unwrap_or_else(|| panic!("事件 {} 无卡臂", id));
                assert!(
                    !card.text.contains('\n'),
                    "事件 {} 正文不得加段（单段画面+一断体）: {}",
                    id,
                    card.text
                );
                assert!(
                    card.text.chars().count() <= 60,
                    "事件 {} 正文越 60 字预算: {}",
                    id,
                    card.text
                );
            }
        }
    }

    // id 唯一 + 名互异（v1.16.14 勘察建议纯测试，零运行时成本）：EVENTS 首列 id
    // 逐条等于下标（防表行复制粘贴重号/build_event match 臂错位）；EVENT_NAMES
    // 去「·续」后缀后互异（防两事件图鉴同名难分——全链段分支名现状本就互异）
    #[test]
    fn event_ids_unique_and_names_distinct() {
        for (i, row) in EVENTS.iter().enumerate() {
            assert_eq!(row.0, i, "EVENTS[{}] 行 id={} 与下标不符（重号/错位）", i, row.0);
        }
        for (a, na) in EVENT_NAMES.iter().enumerate() {
            let ca = na.strip_suffix("·续").unwrap_or(na);
            for (b, nb) in EVENT_NAMES.iter().enumerate().skip(a + 1) {
                let cb = nb.strip_suffix("·续").unwrap_or(nb);
                assert_ne!(ca, cb, "事件名撞车：{}(id{}) vs {}(id{})", na, a, nb, b);
            }
        }
    }

    // 遭遇计数单口：build_event 构建帧收录（轮盘/链内两路共经此口）、重复累加、
    // 越界卫语句不 panic 不写入；收录判定 ev_cnt[id]>0 纯推导（无 seen 位双源）
    #[test]
    fn notebook_counts_on_build() {
        let mut g = sim_game(5, 100.0);
        assert_eq!(g.meta.ev_cnt[112], 0);
        g.modal = None;
        g.build_event(112);
        assert_eq!(g.meta.ev_cnt[112], 1, "构建帧即收录");
        g.modal = None;
        g.build_event(112);
        assert_eq!(g.meta.ev_cnt[112], 2, "重复遭遇累加");
        assert!(g.meta.ev_cnt[112] > 0, "收录=计数>0");
        // 越界卫语句（panic=abort 纪律）
        let n = g.meta.ev_cnt.len();
        g.note_event(999);
        assert_eq!(g.meta.ev_cnt.len(), n);
        // 计数顶格 saturating
        g.meta.ev_cnt[0] = 255;
        g.note_event(0);
        assert_eq!(g.meta.ev_cnt[0], 255, "封顶 255 不回绕（回 0 会误判未遇）");
    }

    // 链7 全程：112 首段建链待分支 → special37 → 直取 113 → special38 → 直取 114
    // （stage 3/3）→ 抉择即缘了「灯花渡舟·缘了」且链清
    #[test]
    fn chain7_advances_and_closes() {
        let mut g = sim_game(5, 100.0);
        g.modal = None;
        g.build_event(112);
        assert!(matches!(g.chain, Some((7, 0, _))), "首段建链待分支");
        g.choose_option(0); // special37
        assert!(matches!(g.chain, Some((7, 113, _))), "opt0 推链至 113");
        g.modal = None;
        g.sim_event(); // 直取中段
        assert_eq!(g.modal.as_ref().unwrap().title, "灯下旧账·续");
        g.choose_option(0); // special38
        assert!(matches!(g.chain, Some((7, 114, _))), "opt0 推链至 114");
        g.modal = None;
        g.sim_event(); // 直取末段
        let card = g.modal.as_ref().unwrap();
        assert_eq!(card.title, "一渡一灯·续");
        assert_eq!(card.stage, Some((3, 3)), "末段 stage 达总段");
        g.choose_option(0);
        assert!(g.chain.is_none(), "末段抉择落通用裁决清链");
        let hl = g.sim.as_ref().unwrap().highlights.join("|");
        assert!(hl.contains("灯花渡舟·缘了"), "链7 缘了记事缺失: {}", hl);
    }

    // 链8 全程：115 → special39 → 116 → special40 → 117（3/3）→「药炉童子·缘了」
    #[test]
    fn chain8_advances_and_closes() {
        let mut g = sim_game(5, 100.0);
        g.modal = None;
        g.build_event(115);
        assert!(matches!(g.chain, Some((8, 0, _))));
        g.choose_option(0); // special39
        g.modal = None;
        g.sim_event();
        assert_eq!(g.modal.as_ref().unwrap().title, "炉语问心·续");
        g.choose_option(0); // special40
        g.modal = None;
        g.sim_event();
        assert_eq!(g.modal.as_ref().unwrap().title, "守炉一夜·续");
        g.choose_option(0);
        let hl = g.sim.as_ref().unwrap().highlights.join("|");
        assert!(hl.contains("药炉童子·缘了"), "链8 缘了记事缺失: {}", hl);
    }

    // 三窗裁决回归：37..=40 置链窗不落清链；窗外检定臂照旧落「落臂即清链」；
    // 非 special 选项落通用裁决清链（三窗语义不互吞）
    #[test]
    fn chain78_window_keeps_chain_and_outside_clears() {
        let mut g = sim_game(5, 100.0);
        g.chain = Some((7, 113, CHAIN_TTL));
        let e37 = {
            let mut e = Eff::new();
            e.special = 37;
            e
        };
        g.apply_eff(&e37);
        assert!(matches!(g.chain, Some((7, 113, _))), "37 置链窗不清链");
        let e34 = {
            let mut e = Eff::new();
            e.special = 34;
            e
        };
        g.apply_eff(&e34);
        assert!(g.chain.is_none(), "窗外检定臂照旧清链");
        let e40 = {
            let mut e = Eff::new();
            e.special = 40;
            e
        };
        g.chain = Some((8, 116, CHAIN_TTL));
        g.apply_eff(&e40);
        assert!(matches!(g.chain, Some((8, 117, _))), "40 重写链态");
        let e0 = Eff::new();
        g.apply_eff(&e0);
        assert!(g.chain.is_none(), "非置链选项落通用裁决清链");
    }

    // 宿敌回响门控：110=chain6_closed（会话态，launch_sim 清零跨世不泄漏）、
    // 111=meta.neme_slain≥1（持久计数器）
    #[test]
    fn enemy_echo_events_gated() {
        let mut g = sim_game(5, 100.0);
        let i110 = EVENTS.iter().position(|&(e, _, _, _)| e == 110).unwrap();
        let i111 = EVENTS.iter().position(|&(e, _, _, _)| e == 111).unwrap();
        let (ws, _) = g.sim_event_weights(100.0, 5, false, false, false);
        assert_eq!(ws[i110], 0.0, "链6 未了 110 不入池");
        assert_eq!(ws[i111], 0.0, "未斩宿敌 111 不入池");
        g.chain6_closed = true;
        let (ws, _) = g.sim_event_weights(100.0, 5, false, false, false);
        assert!(ws[i110] > 0.0, "链6 本世缘了 110 入池");
        assert_eq!(ws[i111], 0.0, "slain=0 111 仍不入池");
        g.meta.neme_slain = 1;
        let (ws, _) = g.sim_event_weights(100.0, 5, false, false, false);
        assert!(ws[i111] > 0.0, "斩过宿敌 111 入池");
        // 会话态纪律：launch_sim 清 chain6_closed（110 只属同一推演）与链缘了闩
        let mut g2 = game_with(Life::default(), None);
        g2.meta.sim_points = 5;
        g2.picked = vec![0];
        g2.chain6_closed = true;
        g2.chain_latch = 0b1111;
        g2.launch_sim();
        assert!(!g2.chain6_closed, "开新稿必须清同世回响标志");
        assert_eq!(g2.chain_latch, 0, "开新稿必须清链缘了闩（v1.16.16）");
    }

    // 回响卡文案兜底：neme=None 帧构建不 panic 且落泛称（宿敌中途可被 take 先例）
    #[test]
    fn echo_cards_none_neme_fallback() {
        let mut g = sim_game(5, 100.0);
        g.meta.neme = None;
        g.modal = None;
        g.build_event(110);
        assert!(g.modal.as_ref().unwrap().text.contains("血仇之敌"));
        g.modal = None;
        g.build_event(111);
        assert!(g.modal.as_ref().unwrap().text.contains("血仇之敌"));
    }

    // 分类徽标单口（图鉴手札行）：伴随/章节/老年/宿敌/链起/链段/稀有/遭遇 八类
    #[test]
    fn event_badges_derive() {
        let b = crate::gallery::event_badge;
        assert_eq!(b(53).0, "伴随");
        assert_eq!(b(74).0, "章节");
        assert_eq!(b(88).0, "老年");
        assert_eq!(b(110).0, "宿敌");
        assert_eq!(b(112).0, "链起");
        assert_eq!(b(113).0, "链段");
        assert_eq!(b(56).0, "稀有");
        assert_eq!(b(108).0, "稀有");
        assert_eq!(b(92).0, "遭遇");
        // 越界防御：落「遭遇」不 panic
        assert_eq!(b(999).0, "遭遇");
    }
}

// ================= v1.16.12 洞府联动 3 条（134..=136）+ 仙境破境分化 =================

#[cfg(test)]
mod v11612_event_tests {
    use super::revenge_tests::game_with;
    use super::*;

    fn sim_game(realm: usize, luck: f32) -> Game {
        let mut g = game_with(Life::default(), None);
        let mut s = Life::default();
        s.realm = realm;
        s.luck = luck;
        g.sim = Some(s);
        g
    }

    // 新批次良构：id 134..=136 每条 ≥2 ≤3 选项、标题正文非空、choose_option(0)
    // 全链不 panic（照 v11610 骨架；stones/pill/qi_pct 臂均纯收益无战死）
    #[test]
    fn dongfu_batch_cards_well_formed() {
        for id in 134..=136 {
            let mut g = sim_game(5, 40.0);
            g.meta.dongfu = [3, 3, 3]; // 臂构建不依赖门控，满级防呆
            g.modal = None;
            g.build_event(id);
            let card = g
                .modal
                .as_ref()
                .unwrap_or_else(|| panic!("事件 {} 无卡臂", id));
            assert!(!card.title.is_empty());
            assert!(!card.text.is_empty());
            assert!(card.opts.len() >= 2, "事件 {} 至少两个选项", id);
            assert!(card.opts.len() <= 3, "事件 {} 卡面至多三选项", id);
            g.choose_option(0); // apply_eff 全链不 panic（sync_save 仅置脏不落盘）
        }
    }

    // 入池门控：名下对应设施满级（meta.dongfu[i]==3，跨世持久）才入池——
    // 134 读 dongfu[0] 聚灵阵 / 135 读 dongfu[1] 灵田 / 136 读 dongfu[2] 丹房，
    // 各自独立，互不牵连（照 110/111 门控单测口径）
    #[test]
    fn dongfu_batch_gated_by_full_facility() {
        let mut g = sim_game(2, 40.0);
        g.recent_events.clear();
        // 三设施皆未满：三条全不入池
        g.meta.dongfu = [0, 0, 0];
        let (ws, _) = g.sim_event_weights(40.0, 2, false, false, false);
        assert_eq!(ws[134], 0.0, "聚灵阵未满 134 不入池");
        assert_eq!(ws[135], 0.0, "灵田未满 135 不入池");
        assert_eq!(ws[136], 0.0, "丹房未满 136 不入池");
        // 仅聚灵阵满级：134 独入，135/136 仍 0
        g.meta.dongfu = [3, 0, 0];
        let (ws, _) = g.sim_event_weights(40.0, 2, false, false, false);
        assert!(ws[134] > 0.0, "聚灵阵满级 134 入池");
        assert_eq!(ws[135], 0.0);
        assert_eq!(ws[136], 0.0);
        // 全满：三条全入池（realm=2 覆盖 mr 1/1/2）
        g.meta.dongfu = [3, 3, 3];
        let (ws, _) = g.sim_event_weights(40.0, 2, false, false, false);
        assert!(ws[134] > 0.0 && ws[135] > 0.0 && ws[136] > 0.0, "全满全入池");
    }

    // 仙境破境卡分化（方向 A1）：target 11..=14 标题「X之阶」+ 场景特化正文，
    // 不再出现「身死之危」（immortal 境 bdeath=0.0 无死劫，sim_do_break 失败仅
    // !rd.immortal 才 roll 死）；选项仍 special 15/16 复用（apply_eff target 无硬编码）
    #[test]
    fn immortal_break_card_narrative() {
        // 通用境回归：realm 5 生花 → target 6 藏锋，仍「X境之门」带身死之危文案
        let mut g = sim_game(5, 100.0);
        g.sim.as_mut().unwrap().layer = 3; // L4 大圆满（同境末层）→ 跨境卡
        g.modal = None;
        g.sim_break();
        let card = g.modal.as_ref().expect("通用破境卡");
        assert_eq!(card.title, "藏锋境之门");
        assert!(card.text.contains("身死之危"));
        assert_eq!(card.opts[0].1.special, 15);
        assert_eq!(card.opts[1].1.special, 16);

        // 仙境四境：11 砚尊 / 12 秉笔 / 13 执卷 / 14 天笔 → 「X之阶」无死劫文案
        for (realm, expect_title) in [
            (10usize, "砚尊之阶"),
            (11, "秉笔之阶"),
            (12, "执卷之阶"),
            (13, "天笔之阶"),
        ] {
            let mut g = sim_game(realm, 100.0);
            g.sim.as_mut().unwrap().layer = 3; // L4 大圆满（同境末层）→ 跨境卡
            g.modal = None;
            g.sim_break();
            let card = g.modal.as_ref().unwrap_or_else(|| panic!("{} 无卡", expect_title));
            assert_eq!(card.title, expect_title);
            assert!(!card.text.contains("身死之危"), "{} 不得再宣死劫", expect_title);
            assert!(card.text.contains("仙境无死劫"));
            assert_eq!(card.opts[0].1.special, 15, "{} 复用 special15", expect_title);
            assert_eq!(card.opts[1].1.special, 16, "{} 复用 special16", expect_title);
        }
    }
}

// ================= v1.16.11 扩池 16 条（118..=133）+ 手札回味 =================

#[cfg(test)]
mod v11611_event_tests {
    use super::revenge_tests::game_with;
    use super::*;

    fn sim_game(realm: usize, luck: f32) -> Game {
        let mut g = game_with(Life::default(), None);
        let mut s = Life::default();
        s.realm = realm;
        s.luck = luck;
        g.sim = Some(s);
        g
    }

    // 新批次良构：id 118..=133 每条 ≥2 ≤3 选项、标题正文非空、choose_option(0)
    // 全链不 panic（照 100..=117 模板；128 fight 臂 8% 战死——每轮重置稿）
    #[test]
    fn new_batch_cards_well_formed() {
        for id in 118..=133 {
            let mut s = Life::default();
            s.realm = 5;
            s.luck = 40.0;
            let mut g = game_with(Life::default(), None);
            g.sim = Some(s);
            g.modal = None;
            g.build_event(id);
            let card = g
                .modal
                .as_ref()
                .unwrap_or_else(|| panic!("事件 {} 无卡臂（白抽）", id));
            assert!(matches!(card.kind, CardKind::Npc));
            assert!(!card.title.is_empty());
            assert!(!card.text.is_empty());
            assert!(card.opts.len() >= 2, "事件 {} 至少两个选项", id);
            assert!(card.opts.len() <= 3, "卡面几何至多三选项");
            g.choose_option(0); // apply_eff 全链不 panic（含置链臂/fight 臂）
        }
    }

    // 链9 全程：118 首段建链待分支 → special41 → 直取 119 → special42 → 直取 120
    // （stage 3/3）→ 抉择即缘了「循册寻仇·缘了」且链清
    #[test]
    fn chain9_advances_and_closes() {
        let mut g = sim_game(5, 100.0);
        g.modal = None;
        g.build_event(118);
        assert!(matches!(g.chain, Some((9, 0, _))), "首段建链待分支");
        g.choose_option(0); // special41
        assert!(matches!(g.chain, Some((9, 119, _))), "opt0 推链至 119");
        g.modal = None;
        g.sim_event(); // 直取中段
        assert_eq!(g.modal.as_ref().unwrap().title, "雪夜拦道·续");
        g.choose_option(0); // special42
        assert!(matches!(g.chain, Some((9, 120, _))), "opt0 推链至 120");
        g.modal = None;
        g.sim_event(); // 直取末段
        let card = g.modal.as_ref().unwrap();
        assert_eq!(card.title, "断名之局·续");
        assert_eq!(card.stage, Some((3, 3)), "末段 stage 达总段");
        g.choose_option(0);
        assert!(g.chain.is_none(), "末段抉择落通用裁决清链");
        let hl = g.sim.as_ref().unwrap().highlights.join("|");
        assert!(hl.contains("循册寻仇·缘了"), "链9 缘了记事缺失: {}", hl);
    }

    // 链10 全程：121 → special43 → 122 → special44 → 123（3/3）→「旧页回声·缘了」
    #[test]
    fn chain10_advances_and_closes() {
        let mut g = sim_game(5, 100.0);
        g.modal = None;
        g.build_event(121);
        assert!(matches!(g.chain, Some((10, 0, _))));
        g.choose_option(0); // special43
        g.modal = None;
        g.sim_event();
        assert_eq!(g.modal.as_ref().unwrap().title, "灯市换页·续");
        g.choose_option(0); // special44
        g.modal = None;
        g.sim_event();
        assert_eq!(g.modal.as_ref().unwrap().title, "换页重逢·续");
        g.choose_option(0);
        let hl = g.sim.as_ref().unwrap().highlights.join("|");
        assert!(hl.contains("旧页回声·缘了"), "链10 缘了记事缺失: {}", hl);
    }

    // 四窗裁决回归：41..=44 置链窗不落清链；窗外检定臂照旧落「落臂即清链」；
    // 非 special 选项落通用裁决清链（四窗语义不互吞，照 37..=40 回归模板）
    #[test]
    fn chain910_window_keeps_chain_and_outside_clears() {
        let mut g = sim_game(5, 100.0);
        g.chain = Some((9, 119, CHAIN_TTL));
        let e41 = {
            let mut e = Eff::new();
            e.special = 41;
            e
        };
        g.apply_eff(&e41);
        assert!(matches!(g.chain, Some((9, 119, _))), "41 置链窗不清链");
        let e34 = {
            let mut e = Eff::new();
            e.special = 34;
            e
        };
        g.apply_eff(&e34);
        assert!(g.chain.is_none(), "窗外检定臂照旧清链");
        let e44 = {
            let mut e = Eff::new();
            e.special = 44;
            e
        };
        g.chain = Some((10, 122, CHAIN_TTL));
        g.apply_eff(&e44);
        assert!(matches!(g.chain, Some((10, 123, _))), "44 重写链态");
        let e0 = Eff::new();
        g.apply_eff(&e0);
        assert!(g.chain.is_none(), "非置链选项落通用裁决清链");
    }

    // 链9/链10 首段门控：118=meta.neme_slain≥1（持久计数器）、
    // 121=comp_bonds 任意 >0（历世结缘过，零新键）
    #[test]
    fn chain910_first_segs_gated() {
        let mut g = sim_game(5, 100.0);
        let i118 = EVENTS.iter().position(|&(e, _, _, _)| e == 118).unwrap();
        let i121 = EVENTS.iter().position(|&(e, _, _, _)| e == 121).unwrap();
        let (ws, _) = g.sim_event_weights(100.0, 5, false, false, false);
        assert_eq!(ws[i118], 0.0, "未斩宿敌 118 不入池");
        assert_eq!(ws[i121], 0.0, "历世无结缘 121 不入池");
        g.meta.neme_slain = 1;
        let (ws, _) = g.sim_event_weights(100.0, 5, false, false, false);
        assert!(ws[i118] > 0.0, "斩过宿敌 118 入池");
        assert_eq!(ws[i121], 0.0, "slain 不喂 121（门控独立）");
        g.meta.comp_bonds[3] = 1;
        let (ws, _) = g.sim_event_weights(100.0, 5, false, false, false);
        assert!(ws[i121] > 0.0, "comp_bonds 任意 >0 121 入池");
    }

    // 手札回味展示数据：构建帧即捕获正文（memo==卡面正文）、越界卫语句不 panic
    // 不写入、ev_recall 清态三口（npc_scroll_reset/C 臂同款置 None 语义）
    #[test]
    fn recall_memo_captured_on_build() {
        let mut g = sim_game(5, 100.0);
        assert!(g.ev_recall.is_none(), "缺省覆层关");
        g.modal = None;
        g.build_event(112);
        let text = g.modal.as_ref().unwrap().text.clone();
        assert_eq!(
            g.ev_memo[112].as_deref(),
            Some(text.as_str()),
            "构建帧即捕获"
        );
        // 重复遭遇覆盖为最新正文
        g.modal = None;
        g.build_event(116);
        let t116 = g.modal.as_ref().unwrap().text.clone();
        assert_eq!(g.ev_memo[116].as_deref(), Some(t116.as_str()));
        assert_eq!(
            g.ev_memo[112].as_deref(),
            Some(text.as_str()),
            "同 id 独立槽位"
        );
        // 越界卫语句（panic=abort 纪律）：不 panic 不写入（槽位数不变）
        let n = g.ev_memo.len();
        g.ev_memo_set(999, "x");
        assert_eq!(g.ev_memo.len(), n);
        // 未遇槽位 None → 图鉴降级路径数据源存在（EVENT_BRIEF 静态表兜底）
        g.modal = None;
        g.build_event(133);
        assert!(g.ev_memo[133].is_some(), "133 构建帧捕获");
        assert!(
            !EVENT_BRIEF[124].is_empty(),
            "降级简介静态表非空"
        );
        // 清态：npc_scroll_reset 并口（×/Esc 关面板臂共用）
        g.ev_recall = Some(112);
        g.npc_scroll_reset();
        assert!(g.ev_recall.is_none(), "关面板并口清覆层态");
    }

    // v1.16.11 徽标扩类：118/121 链起、119/122 链段、124 遭遇、133 稀有
    #[test]
    fn event_badges_11611_derive() {
        let b = crate::gallery::event_badge;
        assert_eq!(b(118).0, "链起");
        assert_eq!(b(121).0, "链起");
        assert_eq!(b(119).0, "链段");
        assert_eq!(b(122).0, "链段");
        assert_eq!(b(124).0, "遭遇");
        assert_eq!(b(133).0, "稀有");
        // 越界防御：扩表后仍落「遭遇」不 panic
        assert_eq!(b(134).0, "遭遇");
    }
}

// ================= v1.16.11 书志全貌推导 + 洞府经营初见闩（施工单2） =================

#[cfg(test)]
mod v11611_book_hint_tests {
    use super::revenge_tests::game_with;
    use super::*;

    // 书志页「此书全貌」汇总带数据源（与 draw_book_record 逐式同构的纯推导，
    // 零新键）：章节 done 计数/chapter_current 现进行章、手札 ev_cnt>0 计数、
    // 结局 endings_seen 位计数；全了走「九章已毕」分支
    #[test]
    fn book_summary_derivations() {
        let mut m = Meta::default();
        // 全新档：0/9，当前章=第一章「雨夜执印」
        assert_eq!(m.chap_done.count_ones(), 0);
        assert_eq!(Game::chapter_current(&m), 0);
        assert_eq!(crate::data::CHAPTERS[0].name, "雨夜执印");
        assert_eq!(ENDINGS.len(), 4);
        assert_eq!(m.endings_seen.count_ones(), 0);
        // 现进行章走事实推导（chapter_current→chapter_done_mask，照既有
        // chapter_current_advances_and_clamps 口径）：章 1 两目标已毕 → 当前章 2
        //（下标 1）；账本 done 计数（chap_done 位）独立于现进行推导
        m.lives = 1;
        m.total_sims = 1;
        assert_eq!(Game::chapter_current(&m), 1);
        assert_eq!(
            crate::data::CHAPTERS[Game::chapter_current(&m)].name,
            crate::data::CHAPTERS[1].name,
            "全貌带现进行章名取当前章"
        );
        // 结局/手札计数：bit 口径 + ev_cnt>0 口径（0 值不计）
        m.endings_seen = 0b0111;
        assert_eq!(m.endings_seen.count_ones(), 3);
        m.ev_cnt = vec![0, 2, 255, 0];
        assert_eq!(m.ev_cnt.iter().filter(|&&c| c > 0).count(), 2);
        // 九章全了（走事实置齐）→ done≥len →「九章已毕」分支（不再报现进行章）
        m.neme_slain = 2;
        m.total_wins = 1;
        m.first_realm[1] = true;
        m.taodie = 6;
        m.region_visited = [true; 5];
        m.companion_lives = 1;
        m.npc_ev[0] = true;
        m.first_realm[10] = true;
        m.endings_seen = 0b1111;
        m.best_realm = 13;
        let done_mask = Game::chapter_done_mask(&m);
        assert_eq!(done_mask, 0x1FF, "全事齐九章全了");
        // 账本闩随 check_chapters 边沿落位（此处同式模拟）→ 全貌带走「九章已毕」分支
        m.chap_done |= done_mask;
        assert!(m.chap_done.count_ones() as usize >= crate::data::CHAPTERS.len());
    }

    // 洞府经营初见闩（sy= bit5）：出生即在洞府（NODE_START==NODE_DONGFU）→ 首见
    // 置闩+toast；位闩幂等不重播；离府不置闩、回府再调置闩（travel 口语义）
    #[test]
    fn dongfu_hint_latches_once_and_gates_by_loc() {
        let mut g = game_with(Life::default(), None);
        assert_eq!(
            g.life.as_ref().unwrap().loc,
            NODE_DONGFU,
            "出生即在洞府（NODE_START 别名）"
        );
        assert_eq!(g.meta.sys_seen & 0b100000, 0, "闩初态未置");
        g.sys_hint_dongfu();
        assert_eq!(g.meta.sys_seen & 0b100000, 0b100000, "洞府首见置闩");
        assert!(
            g.toast
                .as_ref()
                .map_or(false, |(t, _, _)| t.contains("经营洞府")),
            "toast 指向经营洞府"
        );
        let n1 = g
            .log
            .iter()
            .filter(|l| l.text.contains("经营洞府"))
            .count();
        g.sys_hint_dongfu();
        let n2 = g
            .log
            .iter()
            .filter(|l| l.text.contains("经营洞府"))
            .count();
        assert_eq!((n1, n2), (1, 1), "位闩幂等不重播");
        // 离府不置闩；回府再调置闩（travel 抵达口语义）
        let mut g2 = game_with(Life::default(), None);
        g2.life.as_mut().unwrap().loc = NODE_MARKET;
        g2.sys_hint_dongfu();
        assert_eq!(g2.meta.sys_seen & 0b100000, 0, "非洞府不置闩");
        g2.life.as_mut().unwrap().loc = NODE_DONGFU;
        g2.sys_hint_dongfu();
        assert_eq!(g2.meta.sys_seen & 0b100000, 0b100000, "回府置闩");
        // life None 防御（sim 侧/菜单期极端帧）不 panic 不置闩
        let mut g3 = game_with(Life::default(), None);
        g3.life = None;
        g3.sys_hint_dongfu();
        assert_eq!(g3.meta.sys_seen & 0b100000, 0, "无 life 不置闩");
    }
}

// ================= v1.16.9 扩池 15 条（id 85..=99） =================

#[cfg(test)]
mod event_batch_v1169_tests {
    use super::revenge_tests::{game_with, neme};
    use super::*;

    fn sim_game(realm: usize, luck: f32) -> Game {
        let mut g = game_with(Life::default(), None);
        let mut s = Life::default();
        s.realm = realm;
        s.luck = luck;
        g.sim = Some(s);
        g
    }

    // 新批次良构：id 85..=99 每条 ≥2 且 ≤3 选项、标题正文非空，选第一项不 panic
    //（含宿敌卡具名路径 85..=87、fight 臂 97 opt0 可能战死——每轮重置 sim 照
    //  59..=84 模板；build_event 直建无门控，门控语义另有专测）
    #[test]
    fn batch_cards_well_formed() {
        let mut g = game_with(Life::default(), None);
        g.meta.neme = Some(neme(5));
        for id in 85..=99 {
            let mut s = Life::default();
            s.realm = 5;
            s.luck = 40.0;
            g.sim = Some(s);
            g.modal = None;
            g.build_event(id);
            let card = g
                .modal
                .as_ref()
                .unwrap_or_else(|| panic!("事件 {} 无卡臂（白抽）", id));
            assert!(matches!(card.kind, CardKind::Npc));
            assert!(!card.title.is_empty());
            assert!(!card.text.is_empty());
            assert!(card.opts.len() >= 2, "事件 {} 至少两个选项", id);
            assert!(card.opts.len() <= 3, "卡面几何至多三选项（draw.rs:1706）");
            g.choose_option(0); // apply_eff 全链不 panic（含置链臂/fight/injure 臂）
        }
    }

    // 宿敌卡兜底路径：meta.neme=None（链中宿敌被 sim_neme/寻仇反杀后重建卡）
    // 85 建链、86/87 具名落泛称——不 panic 不空文案；置链臂不受宿敌缺失影响
    #[test]
    fn neme_chain_cards_fallback_without_neme() {
        let mut g = game_with(Life::default(), None);
        assert!(g.meta.neme.is_none(), "前置：game_with 缺省无宿敌");
        // 85 opt0（special35）置链
        g.sim = Some(Life::default());
        g.modal = None;
        g.build_event(85);
        let card = g.modal.as_ref().unwrap();
        assert!(!card.text.is_empty() && !card.text.contains("（）"));
        g.choose_option(0);
        assert_eq!(g.chain, Some((6, 86, CHAIN_TTL)), "置链不受宿敌缺失影响");
        // 86 opt0（special36）置链
        g.modal = None;
        g.sim_event();
        let card = g.modal.as_ref().unwrap();
        assert!(!card.text.is_empty() && !card.text.contains("（）"));
        g.choose_option(0);
        assert_eq!(g.chain, Some((6, 87, CHAIN_TTL)));
        // 87 opt0 落通用臂 → 清链
        g.modal = None;
        g.sim_event();
        let card = g.modal.as_ref().unwrap();
        assert!(!card.text.is_empty() && !card.text.contains("（）"));
        g.choose_option(0);
        assert!(g.chain.is_none());
    }

    // 85 宿敌门控：无宿敌不入池；有宿敌 realm≥mr(2) 入池；realm 地板独立生效
    #[test]
    fn neme_chain_first_seg_gated_by_neme() {
        let mut g = sim_game(5, 100.0);
        let i85 = EVENTS.iter().position(|&(e, _, _, _)| e == 85).unwrap();
        let (ws, _) = g.sim_event_weights(100.0, 5, false, false, false);
        assert_eq!(ws[i85], 0.0, "无宿敌 85 不入池");
        g.meta.neme = Some(neme(5));
        let (ws, _) = g.sim_event_weights(100.0, 5, false, false, false);
        assert!(ws[i85] > 0.0, "有宿敌 realm5 入池");
        let (ws, _) = g.sim_event_weights(100.0, 1, false, false, false);
        assert_eq!(ws[i85], 0.0, "realm1 < mr2：85 仍不入池（地板独立）");
    }

    // 88/89 将尽门控：寿元余量 ≤ELDER_LEFT_YEARS 才入池（Life::default 余量
    // 59 载不入；拨到余量 5 载入池）——池内原零覆盖题材的门控口径钉死
    #[test]
    fn elder_events_gated_by_lifespan_left() {
        let mut g = sim_game(2, 100.0);
        let i88 = EVENTS.iter().position(|&(e, _, _, _)| e == 88).unwrap();
        let i89 = EVENTS.iter().position(|&(e, _, _, _)| e == 89).unwrap();
        let (ws, _) = g.sim_event_weights(100.0, 2, false, false, false);
        assert_eq!(ws[i88], 0.0, "余量 59 载 88 不入池");
        assert_eq!(ws[i89], 0.0, "余量 59 载 89 不入池");
        let mut s = Life::default();
        s.realm = 2;
        s.luck = 100.0;
        s.lifespan = s.age + 5.0; // 将尽：余量 5 载
        g.sim = Some(s);
        let (ws, _) = g.sim_event_weights(100.0, 2, false, false, false);
        assert!(ws[i88] > 0.0, "余量 5 载 88 入池");
        assert!(ws[i89] > 0.0, "余量 5 载 89 入池");
    }

    // 链6 全程：85 首段建链（待分支）→ special35 → 86 直取 → special36 → 87 直取
    // → 通用臂清链 + 大事记「血债血偿·缘了」（CHAIN_NAMES[6] 尾名；双段窗口判定
    // 置链侧回归——35/36 不落「落臂即清链」）
    #[test]
    fn chain6_full_run_and_highlight() {
        let mut g = sim_game(5, 100.0);
        g.meta.neme = Some(neme(5));
        g.modal = None;
        g.build_event(85);
        assert_eq!(g.chain, Some((6, 0, CHAIN_TTL)), "首段建链待分支");
        g.choose_option(0); // 循踪暗记（special35）
        assert_eq!(g.chain, Some((6, 86, CHAIN_TTL)), "第二段窗口置链");
        g.modal = None;
        g.sim_event(); // 链活跃 → 直取 86
        assert_eq!(g.modal.as_ref().unwrap().title, "仇踪暗访·续");
        g.choose_option(0); // 布饵设伏（special36）
        assert_eq!(g.chain, Some((6, 87, CHAIN_TTL)));
        g.modal = None;
        g.sim_event(); // 直取 87
        assert_eq!(g.modal.as_ref().unwrap().title, "乘虚夺积·续");
        g.choose_option(0); // 取匣而走（无 special）→ 链终局
        assert!(g.chain.is_none(), "落通用臂即链终局（双段判定 else 侧）");
        let hl = g.sim.as_ref().unwrap().highlights.join("|");
        assert!(hl.contains("血债血偿·缘了"), "链6 完成记事缺失: {}", hl);
    }

    // 链6 中段弃链：86 opt1（就此罢手，无置链臂）→ 立即清链，87 永不再入正常池
    #[test]
    fn chain6_broken_option_closes_chain() {
        let mut g = sim_game(5, 100.0);
        g.meta.neme = Some(neme(5));
        g.modal = None;
        g.build_event(85);
        g.choose_option(0); // special35
        g.modal = None;
        g.sim_event(); // 直取 86
        g.choose_option(1); // 就此罢手
        assert!(g.chain.is_none(), "非推链选项即链终局");
        g.modal = None;
        // 排除链6 首中段与全部链首事件（v1.16.10 增 112/115——链首弹卡即建链，
        // 会污染「此刻链应空」的断言），防弃链后偶发重建干扰断言
        g.recent_events = vec![85, 86, 44, 47, 50, 76, 79, 112, 115];
        g.sim_event(); // 不应抽出链内段 87（权重 0）
        assert_ne!(g.modal.as_ref().unwrap().title, "乘虚夺积·续");
        assert!(g.chain.is_none());
    }

    // 链6 首段 opt0 必置链（skip 模式自动选 option0 直推约定）；opt1 弃链走
    // chain2/chain4 broken 先例（覆土掩碑/折返离楼同款），不设双置链要求
    #[test]
    fn chain6_first_seg_primary_option_advances() {
        let mut g = sim_game(5, 100.0);
        g.meta.neme = Some(neme(5));
        g.modal = None;
        g.build_event(85);
        let card = g.modal.as_ref().unwrap();
        let s0 = card.opts[0].1.special;
        assert!(
            (CHAIN6_SPECIAL_LO..=CHAIN6_SPECIAL_HI).contains(&s0),
            "链6 首段 opt0 必经第二段窗口置链"
        );
    }

    // 高境界/常规新普通条目入池：90..=99 realm5 全部入池（97/98 恶事件 luck=100
    // 乘区后仍 >0）；86/87 链内段除外（另测）；85/88/89 有专属门控（另测）
    #[test]
    fn new_normal_events_in_pool_at_realm5() {
        let g = sim_game(5, 100.0);
        let (ws, _) = g.sim_event_weights(100.0, 5, false, false, true);
        for id in 90..=99 {
            let i = EVENTS.iter().position(|&(e, _, _, _)| e == id).unwrap();
            assert!(ws[i] > 0.0, "普通事件 {} realm5 应入池", id);
        }
    }

    // v1.16.9 对账修复：无面镜客（id57）输臂寿元 -10.0 与卡面「输，留十年寿数」
    // 文案统一（原实现 -12.0 与文案差 2 年）。检定 p=0.5 五五开——循环重掷至出现
    // 输分支（injured=6 标记），断言寿元恰减 10
    #[test]
    fn mirror_guest_loss_deducts_ten_years() {
        let mut g = sim_game(5, 100.0);
        let mut seen_loss = false;
        for _ in 0..400 {
            let mut s = Life::default();
            s.realm = 5;
            s.luck = 100.0;
            g.sim = Some(s);
            g.modal = None;
            g.build_event(57);
            g.choose_option(0); // 赌命（special26）
            let s = g.sim.as_ref().unwrap();
            if s.injured == 6 {
                assert!(
                    (s.lifespan - 65.0).abs() < 1e-9,
                    "输臂寿元恰减 10（默认 75）：{}",
                    s.lifespan
                );
                seen_loss = true;
                break;
            }
        }
        assert!(seen_loss, "400 次检定未见输分支（p=0.5 下概率≈8e-121，不可能）");
    }
}

// ================= v1.16.8 扩池 26 条（id 59..=84） =================

#[cfg(test)]
mod event_batch_v1173_tests {
    use super::revenge_tests::game_with;
    use super::*;

    fn sim_game(realm: usize, luck: f32) -> Game {
        let mut g = game_with(Life::default(), None);
        let mut s = Life::default();
        s.realm = realm;
        s.luck = luck;
        g.sim = Some(s);
        g
    }

    // 新批次良构：id 59..=84 每条 ≥2 且 ≤3 选项、标题正文非空，选第一项不 panic
    //（含链内段 77/78/80/81 直建、检定臂 32/33/34 抉择全链、伴随臂需道侣名）
    #[test]
    fn batch_cards_well_formed() {
        let mut g = game_with(Life::default(), None);
        for id in 59..=84 {
            // 每轮重新置稿：fight 类选项 8% 战死（sim_fight→sim_die 置 sim=None），
            // 不重置则下一轮 build_event 的 sim.unwrap() 必炸
            let mut s = Life::default();
            s.realm = 5;
            s.luck = 40.0;
            s.companion = Some(COMPANION_NAMES[2].to_string());
            g.sim = Some(s);
            g.modal = None;
            g.build_event(id);
            let card = g
                .modal
                .as_ref()
                .unwrap_or_else(|| panic!("事件 {} 无卡臂（白抽）", id));
            assert!(matches!(card.kind, CardKind::Npc));
            assert!(!card.title.is_empty());
            assert!(!card.text.is_empty());
            assert!(card.opts.len() >= 2, "事件 {} 至少两个选项", id);
            assert!(card.opts.len() <= 3, "卡面几何至多三选项（draw.rs:1706）");
            g.choose_option(0); // apply_eff 全链不 panic（含置链臂/检定臂）
        }
    }

    // 链4 全程：76 首段建链 → special28 → 77 → special29 → 78 → special32 检定臂
    // 落通用链裁决 → 链终局（照 chain2_advances_to_keeper 口径）
    #[test]
    fn chain4_advances_to_lantern_trial() {
        let mut g = sim_game(5, 100.0);
        g.modal = None;
        g.build_event(76);
        assert_eq!(g.chain, Some((4, 0, CHAIN_TTL)), "首段建链待分支");
        g.choose_option(0); // 揭谜应约（special28）
        assert_eq!(g.chain, Some((4, 77, CHAIN_TTL)));
        g.modal = None;
        g.sim_event();
        assert_eq!(g.modal.as_ref().unwrap().title, "灯下问答·续");
        g.choose_option(0); // 应主人之问（special29）
        assert_eq!(g.chain, Some((4, 78, CHAIN_TTL)));
        g.modal = None;
        g.sim_event();
        assert_eq!(g.modal.as_ref().unwrap().title, "青灯问藏·续");
        g.choose_option(0); // 受青灯三问（special32，非置链臂）
        assert!(g.chain.is_none(), "检定臂也是链终局");
    }

    // 链4 分支b：折返离楼（无置链臂）→ 立即清链，77/78 永不再入正常池
    #[test]
    fn chain4_broken_option_closes_chain() {
        let mut g = sim_game(5, 100.0);
        g.modal = None;
        g.build_event(76);
        g.choose_option(1); // 折返离楼
        assert!(g.chain.is_none(), "非推链选项即链终局");
    }

    // 链5 双分支：79 → special30→80 / special31→81，两分支段抉择均即链终局
    #[test]
    fn chain5_both_branches_run_and_close() {
        // 分支a：戴面代跳
        let mut g = sim_game(5, 100.0);
        g.modal = None;
        g.build_event(79);
        assert_eq!(g.chain, Some((5, 0, CHAIN_TTL)));
        g.choose_option(0); // special30
        assert_eq!(g.chain, Some((5, 80, CHAIN_TTL)));
        g.modal = None;
        g.sim_event();
        assert_eq!(g.modal.as_ref().unwrap().title, "戏毕面热·续");
        g.choose_option(0);
        assert!(g.chain.is_none());
        // 分支b：出钱贴班
        let mut g2 = sim_game(5, 100.0);
        g2.modal = None;
        g2.build_event(79);
        g2.choose_option(1); // special31
        assert_eq!(g2.chain, Some((5, 81, CHAIN_TTL)));
        g2.modal = None;
        g2.sim_event();
        assert_eq!(g2.modal.as_ref().unwrap().title, "愿成符赠·续");
        g2.choose_option(0);
        assert!(g2.chain.is_none());
    }

    // 链4/链5 完成大事记：扩表后 CHAIN_NAMES 正确挂「灯楼谜约·缘了」（不再被
    // min 钳制错挂链3 名——勘察风险①的对账测试）
    #[test]
    fn chain45_completion_uses_new_names() {
        let mut g = sim_game(5, 100.0);
        g.modal = None;
        g.build_event(79);
        g.choose_option(0);
        g.modal = None;
        g.sim_event(); // 直取 80
        let card = g.modal.as_ref().unwrap();
        assert_eq!(card.stage, Some((2, 2)), "链5 末段达总段");
        g.choose_option(0);
        let hl = g.sim.as_ref().unwrap().highlights.join("|");
        assert!(hl.contains("傩面还愿·缘了"), "链5 完成记事缺失: {}", hl);
        assert!(!hl.contains("沉墨遗匣·缘了"), "不得错挂链3 名");
    }

    // 新链内段恒不入正常池（EVENTS 表权重 0，任何参数下）
    #[test]
    fn new_chain_only_ids_never_in_pool() {
        let g = sim_game(5, 100.0);
        let (ws, _) = g.sim_event_weights(100.0, 5, false, false, true);
        for id in [77, 78, 80, 81, 86, 87] {
            let i = EVENTS.iter().position(|&(e, _, _, _)| e == id).unwrap();
            assert_eq!(ws[i], 0.0, "链内段 {} 权重恒 0", id);
        }
    }

    // 新稀有 72/73：低权重（≤2）入池；w=2 的链4/链5 首段不设稀有双标记（w≥3）
    #[test]
    fn new_rare_events_low_weight() {
        let g = sim_game(5, 100.0);
        let (ws, _) = g.sim_event_weights(100.0, 5, false, false, true);
        for id in [72, 73] {
            let i = EVENTS.iter().position(|&(e, _, _, _)| e == id).unwrap();
            assert!(ws[i] > 0.0, "稀有事件 {} 应入池", id);
            assert!(EVENTS[i].1 <= 2, "稀有层权重 ≤2");
        }
        let mut g2 = sim_game(5, 100.0);
        g2.modal = None;
        g2.build_event(76);
        assert!(
            !g2.modal.as_ref().unwrap().rare && g2.modal.as_ref().unwrap().stage.is_some(),
            "链4 首段 w=3：有段标无稀有（勘察风险③避免双标记）"
        );
    }

    // 章节联动门控：74 需第三章·血仇得报已了（chap_done bit2）、75 需第六章·
    // 同页之缘已了（bit5）且有伴——双门控各臂逐一验证
    #[test]
    fn chapter_linked_events_gated_by_chap_done() {
        let mut g = sim_game(5, 100.0);
        let i74 = EVENTS.iter().position(|&(e, _, _, _)| e == 74).unwrap();
        let i75 = EVENTS.iter().position(|&(e, _, _, _)| e == 75).unwrap();
        // 未达成：两卡皆不入池
        let (ws, _) = g.sim_event_weights(100.0, 5, false, false, false);
        assert_eq!(ws[i74], 0.0, "第三章未了 74 不入池");
        assert_eq!(ws[i75], 0.0, "第六章未了 75 不入池");
        // 第三章已了：74 入池；75 仍被第六章门挡
        g.meta.chap_done |= 1 << 2;
        let (ws, _) = g.sim_event_weights(100.0, 5, false, false, false);
        assert!(ws[i74] > 0.0, "第三章已了 74 入池");
        assert_eq!(ws[i75], 0.0, "第六章未了 75 仍不入池");
        // 第六章已了但无伴：75 仍不入池（宿命双门控）
        g.meta.chap_done |= 1 << 5;
        let (ws, _) = g.sim_event_weights(100.0, 5, false, false, false);
        assert_eq!(ws[i75], 0.0, "第六章已了无伴 75 不入池");
        // 第六章已了且有伴：75 入池
        let (ws, _) = g.sim_event_weights(100.0, 5, false, false, true);
        assert!(ws[i75] > 0.0, "第六章已了有伴 75 入池");
    }

    // 章节联动入池走 chap_done 位——位由施工单1章节系统写入，此处直接置位即可，
    // 与 guide/章节书同源（不重复实现章节谓词）
    #[test]
    fn chapter_linked_gates_use_meta_bits_not_local_flags() {
        // 门控读 self.meta.chap_done（跨世持久），非会话态门闩——重掷/跨世后
        // 位仍在，事件仍可达（与「志」型一次性门闩的语义区别）
        let mut g = sim_game(5, 100.0);
        g.meta.chap_done = 0;
        g.meta.chap_done |= 1 << 2;
        let i74 = EVENTS.iter().position(|&(e, _, _, _)| e == 74).unwrap();
        let (ws, _) = g.sim_event_weights(100.0, 5, false, false, false);
        assert!(ws[i74] > 0.0, "位在即入池，与会话态无关");
    }

    // 客串立绘：链4 灯楼主人挂 Npc(2)（宁清霜脸），75/82..=84 反查道侣名挂 Comp
    #[test]
    fn new_cards_actor_assignment() {
        let mut g = sim_game(5, 100.0);
        g.modal = None;
        g.build_event(76);
        assert!(matches!(
            g.modal.as_ref().unwrap().actor,
            Some(CardActor::Npc(2))
        ));
        let mut s2 = Life::default();
        s2.realm = 5;
        s2.companion = Some(COMPANION_NAMES[5].to_string());
        g.sim = Some(s2);
        g.modal = None;
        g.build_event(83);
        assert!(matches!(
            g.modal.as_ref().unwrap().actor,
            Some(CardActor::Comp(5))
        ));
        g.modal = None;
        g.build_event(75);
        assert!(matches!(
            g.modal.as_ref().unwrap().actor,
            Some(CardActor::Comp(5))
        ));
    }

    // 新普通事件入池：mr 达标即入池（权重/mr 预算抽验——59..=71 全部 realm5 可见）
    #[test]
    fn new_normal_events_in_pool_at_realm5() {
        let g = sim_game(5, 100.0);
        let (ws, _) = g.sim_event_weights(100.0, 5, false, false, true);
        for id in 59..=72 {
            let i = EVENTS.iter().position(|&(e, _, _, _)| e == id).unwrap();
            assert!(ws[i] > 0.0, "普通事件 {} realm5 应入池", id);
        }
        // EVENTS 总数断言（v1.16.9 扩池后 100 条；v1.16.10 扩池 18 → 118 条；
        // v1.16.11 扩池 16 → 134 条；v1.16.12 洞府联动 3 → 137 条；v1.16.13 扩池
        // 14 → 151 条；v1.16.14 扩池 5 → 156 条；v1.16.15 扩池 9 → 165 条；
        // v1.16.16 两批：结局回响/书外之章 5 → 170 条、老年/开局/链14/恶缘/
        // 高境/伴随 13 → 183 条；v1.16.17 批：扩池 10 → 193 条；v1.16.18 批：
        // 扩池 12 → 205 条；v1.16.19 扩池 14（链15/mr7/伴随3/mr6 稀恶/老年2/章节/mr0 恶）
        // → 219 条；v1.16.20 扩池 12（链16/mr7 3/伴随2/老年1/mr6/mr3 恶/章节）
        // → 231 条，
        // v1.16.21 扩池 9（链17 星槎再访 3/special66 稀有检定/mr5 恶/
        // mr7 善恶 2/mr4 恶/老年×伴 1）→ 240 条，稀有基权合计口径见 data.rs
        // 各批注释——账本数字一律以重算为准）
        assert_eq!(EVENTS.len(), 240);
    }
}

// ---------------- v1.16.13 复审⑤：C/M 热键避让守卫单口回归（key_overlay_blocking） ----------------
// 现场背景：原守卫链（main.rs Reality 臂）漏 ev_recall——手札回味覆层独占期按 C
// 一步跳两层（先清覆层再切 panel），借 zoom_art 先例补登记；守卫收敛为 Game 单口
// 供 main.rs 与本测试共用，任一覆层/弹层新增漏登记由本测试拦截。
#[cfg(test)]
mod v11613_key_overlay_blocking_tests {
    use super::revenge_tests::game_with;
    use super::*;

    // 缺省局（菜单/无任何覆层）：守卫放行——C 切人物页、M 开山河图正常路径零损失
    #[test]
    fn default_game_allows_hotkeys() {
        let g = game_with(Life::default(), None);
        assert!(!g.key_overlay_blocking(), "无覆层时守卫必须放行");
    }

    // 六类独占态逐一登记：任一开启即避让（submenu/rmodal/offline_card/ending_show/
    // zoom_art/ev_recall）。ev_recall 为 v1.16.13 新增——原穿透回归口。
    #[test]
    fn each_overlay_blocks_hotkeys() {
        let mut g = game_with(Life::default(), None);

        g.submenu = Some(crate::menu::Sub::Atlas);
        assert!(g.key_overlay_blocking(), "弹层开启须避让");
        g.submenu = None;

        g.rmodal = Some(EventCard {
            kind: CardKind::Finale,
            actor: None,
            rare: false,
            stage: None,
            cat: None,
            scene: None,
            brk: false,
            title: "t".into(),
            text: "".into(),
            opts: vec![],
        });
        assert!(g.key_overlay_blocking(), "事件卡独占须避让");
        g.rmodal = None;

        g.offline_card = Some(OfflineReport {
            years: 1,
            qi: 0.0,
            stones: 0,
        });
        assert!(g.key_overlay_blocking(), "离闭结算卡独占须避让");
        g.offline_card = None;

        g.ending_show = Some(0);
        assert!(g.key_overlay_blocking(), "终卷演出页覆层须避让");
        g.ending_show = None;

        g.zoom_art = Some(ZoomArt::Comp(0));
        assert!(g.key_overlay_blocking(), "立绘鉴赏覆层须避让");
        g.zoom_art = None;

        // v1.16.13 复审⑤回归口：手札回味覆层开启时 C 不再穿透切面板
        g.ev_recall = Some(0);
        assert!(g.key_overlay_blocking(), "手札回味覆层须避让（原穿透 bug）");
        g.ev_recall = None;
        assert!(!g.key_overlay_blocking(), "逐项清后复归放行");
    }

    // 纵深防御语义保持：覆层清态口（npc_scroll_reset 并 ×/Esc 两口）仍清 ev_recall
    #[test]
    fn reset_portals_still_clear_recall() {
        let mut g = game_with(Life::default(), None);
        g.ev_recall = Some(3);
        g.npc_scroll_reset();
        assert!(g.ev_recall.is_none(), "npc_scroll_reset 并口清覆层态（原语义不动）");
    }
}

// ================= v1.16.13 扩池 14 条（137..=150：链11 雷劫侧写/链12 书肆换届/常规 8） =================

#[cfg(test)]
mod v11613_event_tests {
    use super::revenge_tests::game_with;
    use super::*;

    fn sim_game(realm: usize, luck: f32) -> Game {
        let mut g = game_with(Life::default(), None);
        let mut s = Life::default();
        s.realm = realm;
        s.luck = luck;
        g.sim = Some(s);
        g
    }

    // 新批次良构：id 137..=150 每条 ≥2 ≤3 选项、标题/正文非空、choose_option(0)
    // 全链不 panic（照 v11612 骨架；纯 dao/stones/msg 臂无战死）
    #[test]
    fn batch_cards_well_formed() {
        for id in 137..=150 {
            let mut g = sim_game(5, 40.0);
            g.modal = None;
            g.build_event(id);
            let card = g
                .modal
                .as_ref()
                .unwrap_or_else(|| panic!("事件 {} 无卡臂", id));
            assert!(!card.title.is_empty());
            assert!(!card.text.is_empty());
            assert!(card.opts.len() >= 2, "事件 {} 至少两个选项", id);
            assert!(card.opts.len() <= 3, "事件 {} 卡面至多三选项", id);
            g.choose_option(0); // apply_eff 全链不 panic（sync_save 仅置脏不落盘）
        }
    }

    // 链11 劫痕渡客推进：首段建链→置链臂 45/46 推进→末段缘了入大事记；
    // 中段断链（非推链选项）不入大事记（照链9/10 推进口径）
    #[test]
    fn chain11_progression_and_completion() {
        let mut g = sim_game(5, 40.0);
        // 首段：弹卡即建链待分支
        g.build_event(137);
        assert!(g.modal.is_some(), "137 首段有卡臂");
        assert_eq!(g.chain, Some((11, 0, CHAIN_TTL)), "首段建链：待分支");
        g.choose_option(0); // e1 special=45 置链
        assert_eq!(g.chain, Some((11, 138, CHAIN_TTL)), "置链臂 45 → 138");
        // 中段直取语境：build_event(138) 后推链选项续进
        g.build_event(138);
        g.choose_option(0); // e1 special=46 置链
        assert_eq!(g.chain, Some((11, 139, CHAIN_TTL)), "置链臂 46 → 139");
        // 末段：抉择即缘了——链 id 须在 apply_eff 清链前捕获（choose_option 口径），
        // 直取消费后 chain=(11,0,ttl) 仍活跃，此处同式模拟
        g.chain = Some((11, 0, CHAIN_TTL));
        g.build_event(139);
        g.choose_option(0);
        assert!(g.chain.is_none(), "末段抉择落通用段 → 链裁决清链");
        assert!(
            g.sim
                .as_ref()
                .unwrap()
                .highlights
                .iter()
                .any(|h| h.contains("劫痕渡客·缘了")),
            "末段缘了入大事记"
        );
        // 中段断链：138 走 e2（无置链臂）→ 清链且无缘了
        let mut g2 = sim_game(5, 40.0);
        g2.chain = Some((11, 138, CHAIN_TTL));
        g2.build_event(138);
        g2.choose_option(1);
        assert!(g2.chain.is_none(), "中段断链清链");
        assert!(
            !g2.sim
                .as_ref()
                .unwrap()
                .highlights
                .iter()
                .any(|h| h.contains("缘了")),
            "中段断链不入大事记"
        );
    }

    // 链12 书肆换届推进：同构第二链（置链臂 47/48，140→141→142）
    #[test]
    fn chain12_progression_and_completion() {
        let mut g = sim_game(5, 40.0);
        g.build_event(140);
        assert_eq!(g.chain, Some((12, 0, CHAIN_TTL)), "首段建链：待分支");
        g.choose_option(0); // e1 special=47 置链
        assert_eq!(g.chain, Some((12, 141, CHAIN_TTL)), "置链臂 47 → 141");
        g.build_event(141);
        g.choose_option(0); // e1 special=48 置链
        assert_eq!(g.chain, Some((12, 142, CHAIN_TTL)), "置链臂 48 → 142");
        g.chain = Some((12, 0, CHAIN_TTL));
        g.build_event(142);
        g.choose_option(0);
        assert!(g.chain.is_none(), "末段抉择清链");
        assert!(
            g.sim
                .as_ref()
                .unwrap()
                .highlights
                .iter()
                .any(|h| h.contains("书肆换届·缘了")),
            "末段缘了入大事记"
        );
    }

    // 入池门控：链首段与常规条 mr=4/3/5——realm<mr 归零（sim_event_weights 既有
    // 乘区承担，零新门控代码）；realm 达地板时权重为正
    #[test]
    fn batch_gated_by_realm_floor() {
        let mut g = sim_game(3, 40.0);
        g.recent_events.clear();
        let (ws, _) = g.sim_event_weights(40.0, 3, false, false, false);
        assert_eq!(ws[137], 0.0, "realm3 < mr4：链11 首段不入池");
        assert_eq!(ws[140], 0.0, "realm3 < mr4：链12 首段不入池");
        assert!(ws[143] > 0.0, "realm3 达 mr3 地板：143 入池");
        let (ws4, _) = g.sim_event_weights(40.0, 4, false, false, false);
        assert!(ws4[137] > 0.0, "realm4 达 mr4 地板：首段入池");
        assert!(ws4[143] > 0.0);
        let (ws5, _) = g.sim_event_weights(40.0, 5, false, false, false);
        assert!(ws5[145] > 0.0, "realm5 达稀有 mr5：145 入池");
        assert!(ws5[148] > 0.0);
    }

    // 终局页『本稿遭遇』：note_event 单口收录去重有序（构建帧即记，与 ev_cnt 分域）；
    // launch_sim 清零跨世不泄漏
    #[test]
    fn run_events_collected_and_cleared() {
        let mut g = sim_game(5, 40.0);
        g.build_event(143);
        g.build_event(143); // 同 id 重复遭遇：表内去重
        g.build_event(137);
        assert_eq!(g.run_events, vec![143, 137], "去重有序：首遇序");
        assert_eq!(g.meta.ev_cnt[143], 2, "ev_cnt 跨世累计口径不变");
        assert_eq!(g.meta.ev_cnt[137], 1);
        // launch_sim 卫语句过闸（照 sim_does_not_inherit_path 先例：picked+点数）
        g.picked = vec![0];
        g.meta.sim_points = 5;
        g.launch_sim();
        assert!(g.run_events.is_empty(), "launch_sim 清零跨世不泄漏");
        assert!(!g.run_recall_open, "覆层 UI 态同批归零");
    }
}

// ---------------- v1.16.14 扩池 5 条（151..=155）：链13 死当归主 + 宗门换届 2 ----------------

#[cfg(test)]
mod v11614_event_tests {
    use super::revenge_tests::game_with;
    use super::*;

    fn sim_game(realm: usize, luck: f32) -> Game {
        let mut g = game_with(Life::default(), None);
        let mut s = Life::default();
        s.realm = realm;
        s.luck = luck;
        g.sim = Some(s);
        g
    }

    // 新批次良构：id 151..=155 每条 ≥2 ≤3 选项、标题/正文非空、choose_option(0)
    // 全链不 panic（照 v11613 骨架；链臂/纯 dao/stones/msg 臂无战死）
    #[test]
    fn batch_cards_well_formed() {
        for id in 151..=155 {
            let mut g = sim_game(5, 40.0);
            g.modal = None;
            g.build_event(id);
            let card = g
                .modal
                .as_ref()
                .unwrap_or_else(|| panic!("事件 {} 无卡臂", id));
            assert!(!card.title.is_empty());
            assert!(!card.text.is_empty());
            assert!(card.opts.len() >= 2, "事件 {} 至少两个选项", id);
            assert!(card.opts.len() <= 3, "事件 {} 卡面至多三选项", id);
            g.choose_option(0); // apply_eff 全链不 panic（sync_save 仅置脏不落盘）
        }
    }

    // 链13 死当归主推进：首段建链→置链臂 49/50 推进→末段缘了入大事记（照链11/12
    // 口径）；中段断链不入大事记；检定臂 32..=34 照旧窗外——第六窗判定不吞
    // 「检定臂落链即终局」裁决语义（照 game.rs 58-60 教训注）
    #[test]
    fn chain13_progression_and_completion() {
        let mut g = sim_game(5, 40.0);
        // 首段：弹卡即建链待分支
        g.build_event(151);
        assert!(g.modal.is_some(), "151 首段有卡臂");
        assert_eq!(g.chain, Some((13, 0, CHAIN_TTL)), "首段建链：待分支");
        g.choose_option(0); // e1 special=49 置链
        assert_eq!(g.chain, Some((13, 152, CHAIN_TTL)), "置链臂 49 → 152");
        // 中段推链选项续进
        g.build_event(152);
        g.choose_option(0); // e1 special=50 置链
        assert_eq!(g.chain, Some((13, 153, CHAIN_TTL)), "置链臂 50 → 153");
        // 末段：抉择即缘了——直取消费后 chain=(13,0,ttl) 仍活跃，此处同式模拟
        g.chain = Some((13, 0, CHAIN_TTL));
        g.build_event(153);
        g.choose_option(0);
        assert!(g.chain.is_none(), "末段抉择落通用段 → 链裁决清链");
        assert!(
            g.sim
                .as_ref()
                .unwrap()
                .highlights
                .iter()
                .any(|h| h.contains("死当归主·缘了")),
            "末段缘了入大事记"
        );
        // 中段断链：152 走 e2（无置链臂）→ 清链且无缘了
        let mut g2 = sim_game(5, 40.0);
        g2.chain = Some((13, 152, CHAIN_TTL));
        g2.build_event(152);
        g2.choose_option(1);
        assert!(g2.chain.is_none(), "中段断链清链");
        assert!(
            !g2.sim
                .as_ref()
                .unwrap()
                .highlights
                .iter()
                .any(|h| h.contains("缘了")),
            "中段断链不入大事记"
        );
        // 检定臂照旧窗外：special=32（青灯三问）落链上即终局——第六窗不吞检定裁决
        let mut g3 = sim_game(5, 40.0);
        g3.chain = Some((13, 152, CHAIN_TTL));
        let mut e = Eff::new();
        e.special = 32;
        g3.apply_eff(&e);
        assert!(g3.chain.is_none(), "检定臂 32..=34 窗外落链即清链（六窗语义不变）");
    }

    // 入池门控：链13 首段 mr=3 由既有 realm<mr 归零承担；链内段 152/153 权重 0
    // 恒不入正常池（照 chain_only_ids_never_in_pool 口径）
    #[test]
    fn batch13_gated_by_realm_floor() {
        let mut g = sim_game(2, 40.0);
        g.recent_events.clear();
        let (ws, _) = g.sim_event_weights(40.0, 2, false, false, false);
        assert_eq!(ws[151], 0.0, "realm2 < mr3：链13 首段不入池");
        let mut g5 = sim_game(5, 40.0);
        g5.recent_events.clear();
        let (ws5, _) = g5.sim_event_weights(40.0, 5, false, false, false);
        assert!(ws5[151] > 0.0, "realm5 达 mr3 地板：链13 首段入池");
        assert_eq!(ws5[152], 0.0, "链内段权重 0 恒不入正常池");
        assert_eq!(ws5[153], 0.0, "链内段权重 0 恒不入正常池");
    }

    // 宗门换届双门：入宗门（sim.sect 会话）+ 行走东境青云（map_region%5==0，wrapping
    // u32 折回口径照 region_mech 同款）双条件齐备才入池，缺一权重归零
    #[test]
    fn sect_events_gated_by_sect_and_region() {
        // 入宗门 + 东境青云（=0）→ 入池
        let mut g = sim_game(5, 40.0);
        g.sim.as_mut().unwrap().sect = true;
        g.meta.map_region = 0;
        g.recent_events.clear();
        let (ws, _) = g.sim_event_weights(40.0, 5, true, false, false);
        assert!(ws[154] > 0.0, "入宗门+东境青云：接印入池");
        assert!(ws[155] > 0.0, "入宗门+东境青云：传灯入池");
        // 缺宗门（sect=false，region 达标）→ 不入池
        let mut g2 = sim_game(5, 40.0);
        g2.meta.map_region = 0;
        g2.recent_events.clear();
        let (ws2, _) = g2.sim_event_weights(40.0, 5, false, false, false);
        assert_eq!(ws2[154], 0.0, "无宗门：换届不入池");
        assert_eq!(ws2[155], 0.0);
        // 入宗门 + 他域（=1 西漠）→ 不入池
        let mut g3 = sim_game(5, 40.0);
        g3.sim.as_mut().unwrap().sect = true;
        g3.meta.map_region = 1;
        g3.recent_events.clear();
        let (ws3, _) = g3.sim_event_weights(40.0, 5, true, false, false);
        assert_eq!(ws3[154], 0.0, "入宗门但行走西漠：换届不入池");
        assert_eq!(ws3[155], 0.0);
        // wrapping u32 折回：map_region=5 %5==0 归 0 号域（照 region_mech %5 同款）
        let mut g4 = sim_game(5, 40.0);
        g4.sim.as_mut().unwrap().sect = true;
        g4.meta.map_region = 5;
        g4.recent_events.clear();
        let (ws4, _) = g4.sim_event_weights(40.0, 5, true, false, false);
        assert!(ws4[154] > 0.0, "map_region=5 折回东境青云：入池");
    }

    // 徽标自动跟随：151 链起/152·153 链段/154·155 遭遇（event_badge 单口按
    // EVENTS 权重与链起表推导，扩表后新 id 落位即对；badges_derive 同步扩）
    #[test]
    fn batch_badges_derive() {
        let b = crate::gallery::event_badge;
        assert_eq!(b(151).0, "链起");
        assert_eq!(b(152).0, "链段");
        assert_eq!(b(153).0, "链段");
        assert_eq!(b(154).0, "遭遇");
        assert_eq!(b(155).0, "遭遇");
        // 越界防御随表扩仍成立
        assert_eq!(b(999).0, "遭遇");
    }
}

// ================= v1.16.15 结局之礼 + 扩池 9 条（156..=164） =================
#[cfg(test)]
mod v11615_reward_and_pool_tests {
    use super::revenge_tests::game_with;
    use super::*;

    fn sim_game(realm: usize, luck: f32) -> Game {
        let mut g = game_with(Life::default(), None);
        let mut s = Life::default();
        s.realm = realm;
        s.luck = luck;
        g.sim = Some(s);
        g
    }

    // 结局之礼：首录发奖（道韵 +200 金行）+ 同结局重达幂等不重发（位边沿）
    #[test]
    fn ending_reward_first_grant_then_idempotent() {
        let mut g = game_with(Life::default(), None);
        let dy0 = g.meta.dao_yun;
        g.choose_ending(0); // 首录 A《定稿》
        assert_eq!(g.meta.endings_seen, 0b0001, "A 位应置位");
        assert_eq!(g.meta.dao_yun, dy0 + crate::game::ENDING_DY[0], "首录应发结局之礼");
        assert!(
            g.log.iter().any(|x| x.text.contains("【结局之礼】")),
            "奖励须走金行 log（横幅单槽后写胜会吞）"
        );
        // 同结局重达：位已置 → 边沿不触发，道韵不再加
        let dy1 = g.meta.dao_yun;
        g.choose_ending(0);
        assert_eq!(g.meta.dao_yun, dy1, "重达同结局不得重发奖励");
    }

    // 旧档取舍：endings_seen 已置位（跨版本老档）静默不补发——不回溯奖励
    #[test]
    fn ending_reward_old_save_no_retro() {
        let mut g = game_with(Life::default(), None);
        g.meta.endings_seen = 0b0011; // 老档：A/B 已录
        let dy0 = g.meta.dao_yun;
        g.choose_ending(0); // 重达 A
        g.choose_ending(1); // 重达 B
        assert_eq!(g.meta.dao_yun, dy0, "旧档已置位静默不补发（check_chapters 同款取舍）");
    }

    // 全卷之礼：第 4 个首录帧恰发一次（count_ones 3→4 天然单帧边沿）；
    // 称号『执笔人』(值 17) 授权=endings_seen==0b1111，未满拒绝佩戴
    #[test]
    fn ending_full_collection_fires_once_and_grants_title17() {
        let mut g = game_with(Life::default(), None);
        for i in 0..3usize {
            g.choose_ending(i);
            assert!(
                !g.log.iter().any(|x| x.text.contains("【全卷之礼】")),
                "第 {} 次首录不应触发全卷之礼",
                i + 1
            );
        }
        g.choose_ending(3); // 第 4 位：全收集帧
        assert_eq!(g.meta.endings_seen, 0b1111);
        let hits = g
            .log
            .iter()
            .filter(|x| x.text.contains("【全卷之礼】"))
            .count();
        assert_eq!(hits, 1, "全卷之礼恰发一次");
        assert!(g.toast.is_some(), "全收集帧应有可佩指路 toast");
        // 称号授权：满位可佩、未满拒绝
        g.set_worn_title(17);
        assert_eq!(g.meta.worn_title, 17);
        assert_eq!(g.worn_title_of(), "执笔人");
        let mut g2 = game_with(Life::default(), None);
        g2.meta.endings_seen = 0b1110; // 差一位
        g2.meta.worn_title = 5;
        g2.set_worn_title(17);
        assert_eq!(g2.meta.worn_title, 5, "四结局未全览：佩戴拒绝");
    }

    // 扩池 9 条良构：id 156..=164 每条 ≥2 ≤3 选项、标题/正文非空、
    // choose_option(0) 全链不 panic（伴随臂 companion 兜底 COMPANION_NAMES[0]）
    #[test]
    fn pool_batch_cards_well_formed() {
        for id in 156..=164 {
            let mut g = sim_game(5, 40.0);
            g.sim.as_mut().unwrap().companion = Some(COMPANION_NAMES[1].to_string());
            g.modal = None;
            g.build_event(id);
            let card = g
                .modal
                .as_ref()
                .unwrap_or_else(|| panic!("事件 {} 无卡臂", id));
            assert_eq!(card.title, EVENT_NAMES[id], "标题双源");
            assert!(!card.text.is_empty());
            assert!(card.opts.len() >= 2, "事件 {} 至少两个选项", id);
            g.choose_option(0); // apply_eff 全链不 panic（sync_save 仅置脏不落盘）
        }
    }

    // 伴随门控：白名单 9 条（含新 156..=158）无伴不入池、有伴入池（照 154/155
    // 门控测试同款 sim_event_weights 直测）
    #[test]
    fn pool_companion_gate() {
        for &id in COMPANION_EVENTS.iter() {
            let mut g0 = sim_game(5, 40.0);
            g0.recent_events.clear();
            let (ws0, _) = g0.sim_event_weights(40.0, 5, false, false, false);
            assert_eq!(ws0[id], 0.0, "无伴：伴随事件 {} 不入池", id);
            let mut g1 = sim_game(5, 40.0);
            // v1.16.19 门禁修：212 相扶看雪老年×有伴双门控——正臂同开老年态
            if id == 212 {
                let s = g1.sim.as_mut().unwrap();
                s.lifespan = s.age + 5.0;
            }
            g1.recent_events.clear();
            // v1.16.20 门禁修：225 同数流星 mr6 层——正臂 realm 抬 7（realm≥mr
            // 全白名单成立；无伴负臂 realm 5 恒归零不变）
            let (ws1, _) = g1.sim_event_weights(40.0, 7, false, false, true);
            assert!(ws1[id] > 0.0, "有伴：伴随事件 {} 应入池", id);
        }
    }

    // 账本钉死：总基权 711、稀有 14 条 25 基权（≈3.52%）——权重手滑入稀有区
    // （w∈1..=RARE_W）或批注漂移由本测试拦截（账本数字一律以重算为准；
    // v1.16.16 批 +36：老年 2/开局 3/链14 3/恶缘 2/高境 2/伴随 1，新增全 w≤5；
    // v1.16.17 批 +31：mr5 2/mr3 1/伴随 3/老年 1/章节 2/开局 1，全批 w≤4
    // 稀有区零新增——稀有占比 3.79%→3.60% 纯稀释；
    // v1.16.18 扩池批 +37：mr6 3（193/194/195，其中 194 兼恶缘）/章节 2/
    // 老年 2/恶缘另 3（200/201/202）/伴随 1/双门控 1，共 12 条，
    // 全批 w≥3 稀有区零新增——稀有占比 3.60%→3.41% 纯稀释；
    // v1.16.19 扩池批 +36：链15 3（首段 w3+链内 w0×2 不计权）/mr7 2/伴随 3/
    // mr6 稀有 1（213 w2 入稀有区，3.41%→3.52% 拉回）恶缘 1/老年 2/章节 1/
    // mr0 恶缘 1，共 14 条；
    // v1.16.20 扩池批 +30：链16 3（219 首段 w3+220/221 链内 w0×2 不计权）/
    // mr7 3（222 w4 善/223 w3 恶/224 w2 稀有——稀有 14→15 条 25→27 权
    // 3.52%→3.64% 拉回）/伴随 2/老年 1/mr6 常态 1/mr3 恶缘 1/章节 1，
    // 共 12 条）
    // v1.16.21 扩池批 +21：链17 3（231 首段 w3+232/233 链内 w0×2 不计权）/
    // 稀有检定 1（234 w2——稀有 15→16 条 27→29 权 3.64%→3.81%）/mr5 2（231 善/
    // 235 恶）/mr7 2（236 w4 善/237 w3 恶）/mr4 恶 1/老年×伴 1（239 w3），
    // 共 9 条
    #[test]
    fn pool_ledger_recheck() {
        let total: i32 = EVENTS.iter().map(|&(_, w, _, _)| w).sum();
        assert_eq!(total, 762, "总基权漂移——照 data.rs v1.16.21 扩池批注重算");
        let rare: Vec<(usize, i32)> = EVENTS
            .iter()
            .filter(|&&(_, w, _, _)| w > 0 && w <= crate::data::RARE_W)
            .map(|&(id, w, _, _)| (id, w))
            .collect();
        assert_eq!(rare.len(), 16, "稀有条数漂移");
        let rare_w: i32 = rare.iter().map(|&(_, w)| w).sum();
        assert_eq!(rare_w, 29, "稀有基权漂移");
    }

    // 链14 全程：175 首段建链待分支 → special53 → 直取 176 → special54 → 直取
    // 177（stage 3/3）→ 抉择即缘了「修伞记·缘了」且链清；并验缘了闩：bit13 置位
    // 后首段 175 在权重表归零（长寿世同链复走封堵）、非链首段不受累
    #[test]
    fn chain14_advances_closes_and_latches() {
        let mut g = sim_game(5, 100.0);
        g.modal = None;
        g.build_event(175);
        assert!(matches!(g.chain, Some((14, 0, _))), "首段建链待分支");
        g.choose_option(0); // special53
        assert!(matches!(g.chain, Some((14, 176, _))), "opt0 推链至 176");
        g.modal = None;
        g.sim_event(); // 直取中段
        assert_eq!(g.modal.as_ref().unwrap().title, "伞骨重削·续");
        g.choose_option(0); // special54
        assert!(matches!(g.chain, Some((14, 177, _))), "opt0 推链至 177");
        g.modal = None;
        g.sim_event(); // 直取末段
        let card = g.modal.as_ref().unwrap();
        assert_eq!(card.title, "还伞·续");
        assert_eq!(card.stage, Some((3, 3)), "末段 stage 达总段");
        g.choose_option(0);
        assert!(g.chain.is_none(), "末段抉择落通用裁决清链");
        let hl = g.sim.as_ref().unwrap().highlights.join("|");
        assert!(hl.contains("修伞记·缘了"), "链14 缘了记事缺失: {}", hl);
        // 缘了闩：bit13 置位、launch 前的权重表内 175 归零；未缘了的 172 不受累
        assert_eq!(g.chain_latch >> 13 & 1, 1, "缘了闩 bit13 已置");
        let (ws, _) = g.sim_event_weights(40.0, 5, false, false, false);
        assert_eq!(ws[175], 0.0, "已缘了链首段不再入池");
        assert!(ws[172] > 0.0, "未缘了事件不受闩累");
        // 非缘了口不置闩：中段弃链（176 opt1 不推链）后链清但闩不动
        let mut g2 = sim_game(5, 100.0);
        g2.modal = None;
        g2.build_event(175);
        g2.choose_option(0); // special53 → 176
        g2.modal = None;
        g2.sim_event();
        g2.choose_option(1); // 劝他照旧（不推链 → 通用裁决清链）
        assert!(g2.chain.is_none(), "中段弃链清链");
        assert_eq!(g2.chain_latch, 0, "弃链不置闩——同世仍可重走");
    }

    // 老年层扩层回归（v1.16.16）：170/171 与 88/89 同门控（余量≤10 载入池），
    // 伴随 182 与白名单同门控（有伴入池/无伴归零）
    #[test]
    fn elder_and_companion_expansion_gates() {
        // 老年层：sim lifespan-age=5 ≤ 10 → 入池；余量 40 → 归零
        let mut g1 = sim_game(2, 40.0);
        {
            let s = g1.sim.as_mut().unwrap();
            s.age = s.lifespan - 5.0;
        }
        let (ws, _) = g1.sim_event_weights(40.0, 2, false, false, false);
        assert!(ws[170] > 0.0 && ws[171] > 0.0, "老年余量≤10：新二条入池");
        let g2 = sim_game(2, 40.0);
        let (ws2, _) = g2.sim_event_weights(40.0, 2, false, false, false);
        assert_eq!(ws2[170], 0.0, "非老年：炉暖旧器不入池");
        assert_eq!(ws2[171], 0.0, "非老年：灯影授业不入池");
        // 伴随 182：无伴归零、有伴入池（照 COMPANION_EVENTS 白名单既有回归式）
        let g3 = sim_game(2, 40.0);
        let (ws3, _) = g3.sim_event_weights(40.0, 2, false, false, false);
        assert_eq!(ws3[182], 0.0, "无伴：伴随事件 182 不入池");
        let g4 = sim_game(2, 40.0);
        let (ws4, _) = g4.sim_event_weights(40.0, 2, false, false, true);
        assert!(ws4[182] > 0.0, "有伴：伴随事件 182 入池");
    }
}

// ================= v1.16.19 扩池 14 条（205..=218）+ 链15 观星问岁 =================

#[cfg(test)]
mod v11619_event_tests {
    use super::revenge_tests::game_with;
    use super::*;

    fn sim_game(realm: usize, luck: f32) -> Game {
        let mut g = game_with(Life::default(), None);
        let mut s = Life::default();
        s.realm = realm;
        s.luck = luck;
        g.sim = Some(s);
        g
    }

    fn ev_idx(id: usize) -> usize {
        EVENTS
            .iter()
            .position(|&(e, _, _, _)| e == id)
            .unwrap_or_else(|| panic!("事件 {} 不在 EVENTS 表", id))
    }

    // 新批次良构（散件 11 条；链 205/206/207 由 chain15 专测）：标题双源/正文
    // 非空/两到三选项/散件 special 恒 0（本批不占 special 窗，51/52/55/58/59
    // 预留窗位勿动的纪律照旧）/抉择 apply_eff 全链不 panic
    #[test]
    fn batch2_cards_well_formed() {
        for id in [208usize, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218] {
            let mut g = sim_game(7, 40.0);
            {
                let s = g.sim.as_mut().unwrap();
                s.companion = Some(COMPANION_NAMES[1].to_string());
                s.lifespan = s.age + 5.0; // 老年余量≤10 载（build 只管出卡，门在 weights；此置备防御）
            }
            g.meta.chap_done = 0x1FF;
            g.modal = None;
            g.build_event(id);
            let card = g
                .modal
                .as_ref()
                .unwrap_or_else(|| panic!("事件 {} 无卡臂", id));
            assert_eq!(card.title, EVENT_NAMES[id], "标题双源 id {}", id);
            assert!(!card.text.is_empty());
            assert!(card.opts.len() >= 2, "事件 {} 至少两选项", id);
            assert!(card.opts.len() <= 3, "事件 {} 至多三选项", id);
            for (label, e) in &card.opts {
                assert_eq!(e.special, 0, "事件 {} 选项「{}」special 非零", id, label);
            }
            g.choose_option(0); // apply_eff 全链不 panic（sync_save 仅置脏不落盘）
        }
    }

    // 链15 三段全走通：205(opt0) → 206(opt0) → 207(opt0) → 缘了（大事记行 +
    // 闩 bit14 + TTL 不耗尽）+ 闩后 205 反门控归零（chain_latch 查表口径）
    #[test]
    fn chain15_full_walk_and_latch() {
        let mut g = sim_game(6, 100.0);
        g.modal = None;
        g.build_event(205);
        let card = g.modal.as_ref().unwrap();
        assert_eq!(card.stage, Some((1, 3)), "205 首段 stage");
        assert!(
            matches!(card.actor, Some(CardActor::Npc(3))),
            "205 挂醉道人立绘"
        );
        g.choose_option(0); // special 56 → 链 (15, 206)
        assert_eq!(g.chain, Some((15, 206, CHAIN_TTL)), "第八窗置链臂推进");
        g.modal = None;
        g.sim_event(); // 链活跃直取下段
        assert_eq!(g.modal.as_ref().unwrap().title, EVENT_NAMES[206]);
        g.choose_option(0); // special 57 → 链 (15, 207)
        assert_eq!(g.chain, Some((15, 207, CHAIN_TTL)));
        g.modal = None;
        g.sim_event();
        assert_eq!(g.modal.as_ref().unwrap().title, EVENT_NAMES[207]);
        g.choose_option(0); // 末段（stage (3,3)）：缘了
        let hl = g.sim.as_ref().unwrap().highlights.join("|");
        assert!(hl.contains("观星问岁·缘了"), "链15 缘了记事缺失: {}", hl);
        assert_ne!(
            g.chain_latch & (1 << 14),
            0,
            "缘了闩 bit14 置位（CHAIN_FIRSTS (205,15) 口径）"
        );
        // 闩后 205 不再入池（weights 反门控，照 chain_latch 全线推广口径）
        let mut g2 = sim_game(6, 40.0);
        g2.chain_latch |= 1 << 14;
        g2.recent_events.clear();
        let (ws, _) = g2.sim_event_weights(40.0, 6, false, false, false);
        assert_eq!(ws[ev_idx(205)], 0.0, "缘了后 205 首段不入池");
    }

    // 门控四类：mr7 realm 门（208/209）/ 章节 bit6（217）/ 老年余量（215/216）/
    // 双门控 212（无伴、有伴余量富余皆归零）与伴随白名单 210/211（无伴归零）
    #[test]
    fn batch2_gates() {
        // mr7：realm6 归零 / realm7 入池（luck40 乘区 1.0）
        let mut g6 = sim_game(6, 40.0);
        g6.recent_events.clear();
        let (w6, _) = g6.sim_event_weights(40.0, 6, false, false, false);
        assert_eq!(w6[ev_idx(208)], 0.0, "realm<7：208 归零");
        assert_eq!(w6[ev_idx(209)], 0.0, "realm<7：209 归零");
        let mut g7 = sim_game(7, 40.0);
        g7.recent_events.clear();
        let (w7, _) = g7.sim_event_weights(40.0, 7, false, false, false);
        assert!(w7[ev_idx(208)] > 0.0, "realm7：208 入池");
        assert!(w7[ev_idx(209)] > 0.0, "realm7：209 入池");
        // 章节 217：bit6 未了归零 / 已了入池（同一 if 勿拆的联合门，逐臂验）
        let mut ga = sim_game(5, 40.0);
        ga.recent_events.clear();
        let (wa, _) = ga.sim_event_weights(40.0, 5, false, false, false);
        assert_eq!(wa[ev_idx(217)], 0.0, "七章未了：217 归零");
        let mut gb = sim_game(5, 40.0);
        gb.meta.chap_done |= 1 << 6;
        gb.recent_events.clear();
        let (wb, _) = gb.sim_event_weights(40.0, 5, false, false, false);
        assert!(wb[ev_idx(217)] > 0.0, "七章已了：217 入池");
        // 老年 215/216：余量富余归零 / 余量≤10 载入池
        let mut gc = sim_game(5, 40.0);
        gc.recent_events.clear();
        let (wcc, _) = gc.sim_event_weights(40.0, 5, false, false, false);
        assert_eq!(wcc[ev_idx(215)], 0.0, "余量富余：215 归零");
        assert_eq!(wcc[ev_idx(216)], 0.0, "余量富余：216 归零");
        let mut gd = sim_game(5, 40.0);
        gd.sim.as_mut().unwrap().lifespan = gd.sim.as_ref().unwrap().age + 5.0;
        gd.recent_events.clear();
        let (wd, _) = gd.sim_event_weights(40.0, 5, false, false, false);
        assert!(wd[ev_idx(215)] > 0.0, "余量≤10：215 入池");
        assert!(wd[ev_idx(216)] > 0.0, "余量≤10：216 入池");
        // 212 双门控三态：无伴归零 / 有伴但余量富余归零 / 有伴且余量≤10 入池
        let mut ge = sim_game(5, 40.0);
        ge.recent_events.clear();
        let (we, _) = ge.sim_event_weights(40.0, 5, false, false, false);
        assert_eq!(we[ev_idx(212)], 0.0, "无伴：212 归零");
        let mut gf = sim_game(5, 40.0);
        gf.recent_events.clear();
        let (wf, _) = gf.sim_event_weights(40.0, 5, false, false, true);
        assert_eq!(wf[ev_idx(212)], 0.0, "有伴但余量富余：212 归零");
        let mut gg = sim_game(5, 40.0);
        gg.sim.as_mut().unwrap().lifespan = gg.sim.as_ref().unwrap().age + 5.0;
        gg.recent_events.clear();
        let (wg, _) = gg.sim_event_weights(40.0, 5, false, false, true);
        assert!(wg[ev_idx(212)] > 0.0, "有伴且余量≤10：212 入池");
        // 伴随白名单 210/211：无伴归零、有伴入池
        for id in [210usize, 211] {
            let mut g0 = sim_game(5, 40.0);
            g0.recent_events.clear();
            let (w0, _) = g0.sim_event_weights(40.0, 5, false, false, false);
            assert_eq!(w0[ev_idx(id)], 0.0, "无伴：{} 归零", id);
            let mut g1 = sim_game(5, 40.0);
            g1.recent_events.clear();
            let (w1, _) = g1.sim_event_weights(40.0, 5, false, false, true);
            assert!(w1[ev_idx(id)] > 0.0, "有伴：{} 入池", id);
        }
    }

    // 立绘与稀有：213 w2 入稀有层（build_event card.rare）；伴随三条挂 Comp
    // 立绘且锁定当前道侣下标（照 82/203 反查模式）；205/206 挂 Npc(3) 醉道人像
    #[test]
    fn batch2_rare_and_actors() {
        let mut g = sim_game(6, 40.0);
        g.modal = None;
        g.build_event(213);
        assert!(g.modal.as_ref().unwrap().rare, "213 w2 应判稀有");
        for id in [210usize, 211, 212] {
            let mut gc = sim_game(5, 40.0);
            gc.sim.as_mut().unwrap().lifespan = gc.sim.as_ref().unwrap().age + 5.0;
            gc.sim.as_mut().unwrap().companion = Some(COMPANION_NAMES[5].to_string());
            gc.modal = None;
            gc.build_event(id);
            assert!(
                matches!(
                    gc.modal.as_ref().unwrap().actor,
                    Some(CardActor::Comp(5))
                ),
                "{} 应挂 Comp(5) 立绘（反查锁定）",
                id
            );
        }
        for id in [205usize, 206] {
            let mut gd = sim_game(6, 40.0);
            gd.modal = None;
            gd.build_event(id);
            assert!(
                matches!(
                    gd.modal.as_ref().unwrap().actor,
                    Some(CardActor::Npc(3))
                ),
                "{} 应挂 Npc(3) 醉道人立绘",
                id
            );
        }
    }

    // 简一行预算：本批 14 条 ≤18 字（比全表 ≤22 单测更紧，开局新行不许回涨）
    #[test]
    fn batch2_briefs_within_18() {
        for id in 205..=218usize {
            let b = EVENT_BRIEF[id];
            let n = b.chars().count();
            assert!(
                !b.is_empty() && n <= 18,
                "事件 {} 简越 18 字预算: {} ({}字)",
                id,
                b,
                n
            );
        }
    }
}

// ---------------- v1.16.17 双修共进：门控/定价/收益/冷却闩（现实侧主动双修） ----------------

#[cfg(test)]
mod dual_cult_tests {
    use super::revenge_tests::game_with;
    use super::*;

    fn bonded_life() -> Life {
        let mut l = Life::default();
        l.companion = Some(COMPANION_NAMES[0].to_string());
        l.comp_fav = 25;
        l
    }

    // 门控四态与优先级：未结缘 > 本载已修 > 灵石不足 > 就绪
    //（NoCompanion 含 life=None——面板仅在 Reality 打开，防御口径）
    #[test]
    fn gate_priorities() {
        assert_eq!(Game::dual_cult_gate(None, 100), DualGate::NoCompanion);
        assert_eq!(
            Game::dual_cult_gate(Some(&Life::default()), 100),
            DualGate::NoCompanion
        );
        // 本载已修优先于灵石不足（sx_year == 当前年岁、stones=0 仍判 ThisYear）
        let mut l = bonded_life();
        l.age = 30.0;
        l.sx_year = 30;
        l.stones = 0;
        assert_eq!(Game::dual_cult_gate(Some(&l), 500), DualGate::ThisYear);
        // 灵石不足
        let mut l = bonded_life();
        l.stones = 499;
        assert_eq!(Game::dual_cult_gate(Some(&l), 500), DualGate::Stones);
        // 就绪
        let mut l = bonded_life();
        l.stones = 500;
        assert_eq!(Game::dual_cult_gate(Some(&l), 500), DualGate::Ready);
    }

    // 定价：500×(1+realm) 线性（realm=0 → 500、2 → 1500）。v1.16.18 清账②改口：
    // 勿再类比 exchange_cost（1500×(1+realm)^1.3 幂曲线，曲线不同）——本测断言
    // 本就线性，改口零改动
    #[test]
    fn cost_scales_with_realm() {
        let g = game_with(Life::default(), None);
        assert_eq!(g.dual_cult_cost(), 500);
        let mut l = Life::default();
        l.realm = 2;
        let g2 = game_with(l, None);
        assert_eq!(g2.dual_cult_cost(), 1500);
    }

    // 年岁闩取整单口：负值/极端大值钳 0..=u16::MAX（panic=abort 纪律）
    #[test]
    fn dual_year_clamped() {
        assert_eq!(Game::dual_year(0.0), 0);
        assert_eq!(Game::dual_year(33.0), 33);
        assert_eq!(Game::dual_year(33.9), 33, "浮点截断向下");
        assert_eq!(Game::dual_year(-5.0), 0);
        assert_eq!(Game::dual_year(1.0e9), u16::MAX);
    }

    // 收益结算：灵石扣定价、修为+2载（直加不进乘链）、情谊+5、道心/气运+1、闩记本载
    #[test]
    fn act_dual_cult_grants_and_costs() {
        let mut l = bonded_life();
        l.stones = 5000;
        l.qi = 0.0;
        l.dao = 50.0;
        l.luck = 40.0;
        l.age = 33.0;
        let mut g = game_with(l, None);
        let gain = g.cult_gain_per_year() * 2.0;
        let cost = g.dual_cult_cost();
        g.act_dual_cult();
        let nl = g.life.as_ref().unwrap();
        assert_eq!(nl.stones, 5000 - cost, "灵石扣定价");
        assert!((nl.qi - gain).abs() < 1e-9, "修为 += 2×每载收益");
        assert_eq!(nl.comp_fav, 30, "情谊+5");
        assert!((nl.dao - 51.0).abs() < 1e-6, "道心+1");
        assert!((nl.luck - 41.0).abs() < 1e-6, "气运+1");
        assert_eq!(nl.sx_year, 33, "闩记本载年岁");
        // 修为直加不入乘链：cult_gain_per_year 基准不变（乘链单一来源纪律）
        assert!(
            (g.cult_gain_per_year() * 2.0 - gain).abs() < 1e-9,
            "双修不得放大每载收益基准"
        );
    }

    // 每载一次：同年内二连点零损耗（灵石/情谊/四维/闩全不动）
    #[test]
    fn act_dual_cult_twice_same_year_noop() {
        let mut l = bonded_life();
        l.stones = 5000;
        l.age = 20.0;
        let mut g = game_with(l, None);
        g.act_dual_cult();
        let s1 = g.life.as_ref().unwrap().stones;
        let f1 = g.life.as_ref().unwrap().comp_fav;
        g.act_dual_cult();
        let nl = g.life.as_ref().unwrap();
        assert_eq!(nl.stones, s1, "同年二连点不再扣灵石");
        assert_eq!(nl.comp_fav, f1, "同年二连点不再涨情谊");
        assert_eq!(nl.sx_year, 20);
    }

    // 推岁一载后解锁再修（act_cultivate 现实推岁口唯一，闩随年岁自然过期）
    #[test]
    fn next_year_unlocks_again() {
        let mut l = bonded_life();
        l.stones = 5000;
        l.age = 20.0;
        let mut g = game_with(l, None);
        g.act_dual_cult();
        assert_eq!(
            Game::dual_cult_gate(g.life.as_ref(), g.dual_cult_cost()),
            DualGate::ThisYear
        );
        g.act_cultivate(false);
        assert_eq!(g.life.as_ref().unwrap().age, 21.0);
        assert_eq!(
            Game::dual_cult_gate(g.life.as_ref(), g.dual_cult_cost()),
            DualGate::Ready,
            "推岁一载后冷却过期"
        );
    }

    // 四维与情谊封顶：道心/气运 100 顶、情谊 100 顶（clamp 口照遗泽/好感既有语义）
    #[test]
    fn dao_luck_fav_clamped_at_100() {
        let mut l = bonded_life();
        l.stones = 5000;
        l.dao = 100.0;
        l.luck = 99.5;
        l.comp_fav = 98;
        let mut g = game_with(l, None);
        g.act_dual_cult();
        let nl = g.life.as_ref().unwrap();
        assert!((nl.dao - 100.0).abs() < 1e-6);
        assert!((nl.luck - 100.0).abs() < 1e-6);
        assert_eq!(nl.comp_fav, 100);
    }

    // 未结缘/无档执行口静默拒绝（按钮仅 UI 门，执行口单源全再校验）
    #[test]
    fn act_dual_cult_without_companion_noop() {
        let mut g = game_with(Life::default(), None);
        g.act_dual_cult();
        let nl = g.life.as_ref().unwrap();
        assert_eq!(nl.sx_year, 0, "未结缘不置闩");
        assert_eq!(nl.stones, Life::default().stones);
    }
}

// ================= v1.16.17 扩池 10 条（183..=192）：门控/立绘/好感通道/账本 =================

#[cfg(test)]
mod v11617_pool_tests {
    use super::revenge_tests::game_with;
    use super::*;

    fn sim_game(realm: usize, luck: f32) -> Game {
        let mut g = game_with(Life::default(), None);
        let mut s = Life::default();
        s.realm = realm;
        s.luck = luck;
        g.sim = Some(s);
        g
    }

    // 三表同长 + 新增段 id 连续（全局同长/唯一测试之外，钉新增段不漂移）
    #[test]
    fn batch_tables_aligned() {
        // v1.16.21 扩池 9 条（231..=239）同批随批注口径 231→240
        assert_eq!(EVENTS.len(), 240);
        assert_eq!(EVENT_NAMES.len(), 240);
        assert_eq!(EVENT_BRIEF.len(), 240);
        for id in 183..=204usize {
            assert_eq!(EVENTS[id].0, id, "新增段 id 连续");
            assert!(!EVENT_NAMES[id].is_empty());
            assert!(!EVENT_BRIEF[id].is_empty());
        }
        // v1.16.19 批次段同口径补钉
        for id in 205..=218usize {
            assert_eq!(EVENTS[id].0, id, "v1.16.19 段 id 连续");
            assert!(!EVENT_NAMES[id].is_empty());
            assert!(!EVENT_BRIEF[id].is_empty());
        }
        // v1.16.20 批次段同口径补钉
        for id in 219..=230usize {
            assert_eq!(EVENTS[id].0, id, "v1.16.20 段 id 连续");
            assert!(!EVENT_NAMES[id].is_empty());
            assert!(!EVENT_BRIEF[id].is_empty());
        }
        // v1.16.21 批次段同口径补钉
        for id in 231..=239usize {
            assert_eq!(EVENTS[id].0, id, "v1.16.21 段 id 连续");
            assert!(!EVENT_NAMES[id].is_empty());
            assert!(!EVENT_BRIEF[id].is_empty());
        }
    }

    // 卡面良构：新 10 条标题双源/正文非空/两选项/抉择 apply_eff 全链不 panic
    //（章节门控两臂以 chap_done 全开进卡；伴随三条以有伴进卡）
    #[test]
    fn batch_cards_well_formed() {
        for id in 183..=204 {
            let mut g = sim_game(6, 40.0);
            g.sim.as_mut().unwrap().companion = Some(COMPANION_NAMES[1].to_string());
            g.meta.chap_done = 0x1FF; // 九章全开，章节联动两臂可入卡
            g.modal = None;
            g.build_event(id);
            let card = g
                .modal
                .as_ref()
                .unwrap_or_else(|| panic!("事件 {} 无卡臂", id));
            assert_eq!(card.title, EVENT_NAMES[id], "标题双源 id {}", id);
            assert!(!card.text.is_empty());
            assert!(card.opts.len() >= 2, "事件 {} 至少两选项", id);
            assert!(card.opts.len() <= 3, "事件 {} 至多三选项", id);
            // 本批纪律：普通条目 special 恒 0（勿动 51/52 链13 预留窗位与 55 备用）
            for (label, e) in &card.opts {
                assert_eq!(e.special, 0, "事件 {} 选项「{}」special 非零", id, label);
            }
            g.choose_option(0); // apply_eff 全链不 panic（sync_save 仅置脏不落盘）
        }
    }

    // 伴随层补位：183/184/185 白名单门控（无伴归零/有伴入池）；白丁有伴
    //（realm0）183 入池——白丁层伴随从 0→1 条；185 mr4 门槛 realm3 归零
    #[test]
    fn companion_batch_gates_and_tiers() {
        for id in [183usize, 184, 185] {
            let i = EVENTS.iter().position(|&(e, _, _, _)| e == id).unwrap();
            let mut g0 = sim_game(5, 40.0);
            g0.recent_events.clear();
            let (ws0, _) = g0.sim_event_weights(40.0, 5, false, false, false);
            assert_eq!(ws0[i], 0.0, "无伴：伴随事件 {} 不入池", id);
            let mut g1 = sim_game(5, 40.0);
            g1.recent_events.clear();
            let (ws1, _) = g1.sim_event_weights(40.0, 5, false, false, true);
            assert!(ws1[i] > 0.0, "有伴：伴随事件 {} 应入池", id);
        }
        // 白丁有伴（mr0）
        let mut g = sim_game(0, 40.0);
        g.recent_events.clear();
        let (ws, _) = g.sim_event_weights(40.0, 0, false, false, true);
        assert!(ws[183] > 0.0, "白丁有伴：183 应入池");
        // mr4 门槛
        let mut g2 = sim_game(3, 40.0);
        g2.recent_events.clear();
        let (ws2, _) = g2.sim_event_weights(40.0, 3, false, false, true);
        assert_eq!(ws2[185], 0.0, "realm<4：185 归零");
    }

    // 立绘：183/184/185 挂 Comp 立绘且锁定当前道侣下标（照 82 反查模式）
    #[test]
    fn companion_batch_actor_comp() {
        for id in [183usize, 184, 185] {
            let mut g = sim_game(5, 40.0);
            g.sim.as_mut().unwrap().companion = Some(COMPANION_NAMES[3].to_string());
            g.modal = None;
            g.build_event(id);
            let card = g.modal.as_ref().unwrap();
            assert!(
                matches!(card.actor, Some(CardActor::Comp(3))),
                "事件 {} 立绘应锁定道侣 3",
                id
            );
        }
    }

    // 好感通道首用：184 opt0 同参 → comp_fav +5（COMPANION_GIFT_FAV）；满值
    // clamp 100；非伴随事件（192）好感不动（Eff.comp_fav 默认零不误伤）；
    // 只升不降红线：通道消费口负值忽略
    #[test]
    fn event184_fav_channel() {
        let mut g = sim_game(5, 40.0);
        {
            let s = g.sim.as_mut().unwrap();
            s.companion = Some(COMPANION_NAMES[0].to_string());
            s.comp_fav = 25;
        }
        g.modal = None;
        g.build_event(184);
        g.choose_option(0);
        assert_eq!(g.sim.as_ref().unwrap().comp_fav, 30, "好感+5");

        let mut g2 = sim_game(5, 40.0);
        {
            let s = g2.sim.as_mut().unwrap();
            s.companion = Some(COMPANION_NAMES[0].to_string());
            s.comp_fav = 100;
        }
        g2.modal = None;
        g2.build_event(184);
        g2.choose_option(0);
        assert_eq!(g2.sim.as_ref().unwrap().comp_fav, 100, "clamp 100");

        let mut g3 = sim_game(5, 40.0);
        let f3 = g3.sim.as_ref().unwrap().comp_fav;
        g3.modal = None;
        g3.build_event(192);
        g3.choose_option(0);
        assert_eq!(g3.sim.as_ref().unwrap().comp_fav, f3, "非好感通道事件不动好感");
    }

    // 章节门控：190 需 bit0（第一章）、191 需 bit3（第四章）——未达归零/已达
    // 入池（照 74/75 直测同款；同一 if 双门控勿拆两段的写法由本测钉住）。
    // v1.16.18 扩池 +2：196 需 bit1（第二章）、197 需 bit4（第五章）
    #[test]
    fn chapter_batch_gates() {
        for (id, bit) in [(190usize, 0u32), (191usize, 3u32), (196usize, 1u32), (197usize, 4u32)] {
            let i = EVENTS.iter().position(|&(e, _, _, _)| e == id).unwrap();
            let mut g0 = sim_game(5, 40.0);
            g0.recent_events.clear();
            let (ws0, _) = g0.sim_event_weights(40.0, 5, false, false, false);
            assert_eq!(ws0[i], 0.0, "章节未了：{} 不入池", id);
            let mut g1 = sim_game(5, 40.0);
            g1.recent_events.clear();
            g1.meta.chap_done |= 1 << bit;
            let (ws1, _) = g1.sim_event_weights(40.0, 5, false, false, false);
            assert!(ws1[i] > 0.0, "章节已了：{} 应入池", id);
        }
    }

    // 老年门控：189 与 88/89/170/171 同门控（余量≤10 载入池；余量富余归零）
    #[test]
    fn elder_batch_gate_189() {
        let mut g1 = sim_game(2, 40.0);
        {
            let s = g1.sim.as_mut().unwrap();
            s.age = s.lifespan - 5.0;
        }
        let (ws1, _) = g1.sim_event_weights(40.0, 2, false, false, false);
        assert!(ws1[189] > 0.0, "老年余量≤10：189 入池");
        let g2 = sim_game(2, 40.0);
        let (ws2, _) = g2.sim_event_weights(40.0, 2, false, false, false);
        assert_eq!(ws2[189], 0.0, "余量富余：189 归零");
    }

    // mr5 补薄：186/187 realm5 入池、realm4 归零——mr5 无条件常态池 7→9 条
    #[test]
    fn mr5_batch_in_pool_at_realm5() {
        let mut g = sim_game(5, 40.0);
        g.recent_events.clear();
        let (ws, _) = g.sim_event_weights(40.0, 5, false, false, false);
        assert!(ws[186] > 0.0, "mr5 事件 186 realm5 应入池");
        assert!(ws[187] > 0.0, "mr5 事件 187 realm5 应入池");
        let g4 = sim_game(4, 40.0);
        let (ws4, _) = g4.sim_event_weights(40.0, 4, false, false, false);
        assert_eq!(ws4[186], 0.0, "realm<5：186 归零");
        assert_eq!(ws4[187], 0.0, "realm<5：187 归零");
    }

    // v1.16.18 扩池 mr6 门控直测：193/194/195 realm5 归零、realm6 入池——
    // EVENTS 表 mr 列首次出现 6（长生世独占层，realm<mr 归零零代码天然支持）
    #[test]
    fn mr6_batch_in_pool_at_realm6() {
        let mut g = sim_game(6, 40.0);
        g.recent_events.clear();
        let (ws, _) = g.sim_event_weights(40.0, 6, false, false, false);
        assert!(ws[193] > 0.0, "mr6 事件 193 realm6 应入池");
        assert!(ws[194] > 0.0, "mr6 事件 194 realm6 应入池");
        assert!(ws[195] > 0.0, "mr6 事件 195 realm6 应入池");
        let g5 = sim_game(5, 40.0);
        let (ws5, _) = g5.sim_event_weights(40.0, 5, false, false, false);
        assert_eq!(ws5[193], 0.0, "realm<6：193 归零");
        assert_eq!(ws5[194], 0.0, "realm<6：194 归零");
        assert_eq!(ws5[195], 0.0, "realm<6：195 归零");
    }

    // v1.16.18 扩池老年门控：198/199 与 88/89/170/171/189 同门控（余量≤10 载
    // 入池；余量富余归零——照 elder_batch_gate_189 直测同款）
    #[test]
    fn elder_batch_gate_198_199() {
        let mut g1 = sim_game(2, 40.0);
        {
            let s = g1.sim.as_mut().unwrap();
            s.age = s.lifespan - 5.0;
        }
        let (ws1, _) = g1.sim_event_weights(40.0, 2, false, false, false);
        assert!(ws1[198] > 0.0, "老年余量≤10：198 入池");
        assert!(ws1[199] > 0.0, "老年余量≤10：199 入池");
        let g2 = sim_game(2, 40.0);
        let (ws2, _) = g2.sim_event_weights(40.0, 2, false, false, false);
        assert_eq!(ws2[198], 0.0, "余量富余：198 归零");
        assert_eq!(ws2[199], 0.0, "余量富余：199 归零");
    }

    // v1.16.18 扩池双门控直测：204 同席说岁 = 老年（余量≤10）AND 有伴——
    // 新门控形态（单 if 交集），四象限全测：两条件齐备入池、任一缺即归零
    #[test]
    fn dual_gate_204() {
        // 老年+有伴：入池
        let mut g1 = sim_game(2, 40.0);
        {
            let s = g1.sim.as_mut().unwrap();
            s.age = s.lifespan - 5.0;
            s.companion = Some(COMPANION_NAMES[1].to_string());
        }
        let (ws1, _) = g1.sim_event_weights(40.0, 2, false, false, true);
        assert!(ws1[204] > 0.0, "老年+有伴：204 入池");
        // 老年+无伴：归零
        let mut g2 = sim_game(2, 40.0);
        {
            let s = g2.sim.as_mut().unwrap();
            s.age = s.lifespan - 5.0;
        }
        let (ws2, _) = g2.sim_event_weights(40.0, 2, false, false, false);
        assert_eq!(ws2[204], 0.0, "老年无伴：204 归零");
        // 非老年+有伴：归零（余量富余——若单走伴白名单则此处会误入池）
        let mut g3 = sim_game(2, 40.0);
        g3.sim.as_mut().unwrap().companion = Some(COMPANION_NAMES[1].to_string());
        let (ws3, _) = g3.sim_event_weights(40.0, 2, false, false, true);
        assert_eq!(ws3[204], 0.0, "余量富余有伴：204 归零");
        // 非老年+无伴：归零
        let g4 = sim_game(2, 40.0);
        let (ws4, _) = g4.sim_event_weights(40.0, 2, false, false, false);
        assert_eq!(ws4[204], 0.0, "余量富余无伴：204 归零");
    }
}

// ================= v1.16.20 扩池 12 条（219..=230）+ 链16 剑穗之约 =================

#[cfg(test)]
mod v11620_pool_batch_tests {
    use super::revenge_tests::game_with;
    use super::*;

    fn sim_game(realm: usize, luck: f32) -> Game {
        let mut g = game_with(Life::default(), None);
        let mut s = Life::default();
        s.realm = realm;
        s.luck = luck;
        g.sim = Some(s);
        g
    }

    // 新批次良构：id 219..=230 每条 ≥2 ≤3 选项、标题正文非空、choose_option(0)
    // 全链不 panic（照 v11610 骨架；stones/qi_pct 臂均无战死面；伴随臂道侣名
    // 缺省回退 COMPANION_NAMES[0] 不炸）
    #[test]
    fn v1170_batch_cards_well_formed() {
        let mut g = game_with(Life::default(), None);
        for id in 219..=230 {
            let mut s = Life::default();
            s.realm = 7; // mr7 三条也能构建（门控在 weights，build 不看 mr）
            s.luck = 40.0;
            s.companion = Some(COMPANION_NAMES[1].to_string());
            g.sim = Some(s);
            g.modal = None;
            g.build_event(id);
            let card = g
                .modal
                .as_ref()
                .unwrap_or_else(|| panic!("事件 {} 无卡臂（白抽）", id));
            assert!(matches!(card.kind, CardKind::Npc));
            assert!(!card.title.is_empty());
            assert!(!card.text.is_empty());
            assert!(card.opts.len() >= 2, "事件 {} 至少两个选项", id);
            assert!(card.opts.len() <= 3, "卡面几何至多三选项");
            assert_eq!(
                card.title,
                EVENT_NAMES[id],
                "事件 {} 标题双源漂移",
                id
            );
            let b = EVENT_BRIEF[id];
            assert!(
                !b.is_empty() && b.chars().count() <= 18,
                "事件 {} 简越 18 字预算: {} ({}字)",
                id,
                b,
                b.chars().count()
            );
            assert!(
                card.text.chars().count() <= 60 && !card.text.contains('\n'),
                "事件 {} 正文越 60 字或含分段: {}",
                id,
                card.text
            );
            g.choose_option(0); // apply_eff 全链不 panic
        }
    }

    // 链16 全程：219 首段建链待分支 → special60 → 直取 220 → special61 → 直取
    // 221（stage 3/3）→ 抉择即缘了「剑穗之约·缘了」且链清；缘了闩 bit15 置位后
    // 首段 219 在权重表归零、非链首段不受累
    #[test]
    fn chain16_advances_closes_and_latches() {
        let mut g = sim_game(5, 100.0);
        g.modal = None;
        g.build_event(219);
        assert!(matches!(g.chain, Some((16, 0, _))), "首段建链待分支");
        assert_eq!(g.modal.as_ref().unwrap().title, "剑穗生结");
        g.choose_option(0); // special60
        assert!(matches!(g.chain, Some((16, 220, _))), "opt0 推链至 220");
        g.modal = None;
        g.sim_event(); // 直取中段
        assert_eq!(g.modal.as_ref().unwrap().title, "剑穗缓结·续");
        g.choose_option(0); // special61
        assert!(matches!(g.chain, Some((16, 221, _))), "opt0 推链至 221");
        g.modal = None;
        g.sim_event(); // 直取末段
        let card = g.modal.as_ref().unwrap();
        assert_eq!(card.title, "剑成赠穗·续");
        assert_eq!(card.stage, Some((3, 3)), "末段 stage 达总段");
        g.choose_option(0);
        assert!(g.chain.is_none(), "末段抉择落通用裁决清链");
        let hl = g.sim.as_ref().unwrap().highlights.join("|");
        assert!(hl.contains("剑穗之约·缘了"), "链16 缘了记事缺失: {}", hl);
        // 缘了闩：bit15 置位（u32 位域充裕）；权重表内 219 归零、未缘了 205 不受累
        // （对照臂 realm 7——205 为 mr6 层，realm 5 下本就归零测不出闩效应）
        assert_eq!(g.chain_latch >> 15 & 1, 1, "缘了闩 bit15 已置");
        let (ws, _) = g.sim_event_weights(40.0, 7, false, false, false);
        assert_eq!(ws[219], 0.0, "已缘了链首段不再入池");
        assert!(ws[205] > 0.0, "未缘了事件不受闩累");
    }

    // 链16 中段分支b：220 opt1「一夜结完」（special62 第九窗内、无置链臂）——
    // 链不立即清（窗内）随下次 sim_event next==0 弃链；断链不置缘了闩（同世可重走）
    #[test]
    fn chain16_mid_branch_b_abandons_without_latch() {
        let mut g = sim_game(5, 100.0);
        g.modal = None;
        g.build_event(219);
        g.choose_option(0); // special60 → 220
        g.modal = None;
        g.sim_event();
        assert_eq!(g.modal.as_ref().unwrap().title, "剑穗缓结·续");
        g.choose_option(1); // special62（窗内，apply_eff _ 臂空转不置链）
        assert!(
            matches!(g.chain, Some((16, 0, _))),
            "窗内断链：链态保留待下帧弃链"
        );
        g.modal = None;
        g.sim_event(); // next==0 → 弃链落正常轮盘
        assert!(g.chain.is_none(), "下次 sim_event 弃链");
        assert_eq!(g.chain_latch, 0, "中段断链不置缘了闩——同世仍可重走");
        // eff_desc 副标：60/61 续段名、62 断链臂名（eff_desc 静态表对账）
        let eff = Eff {
            special: 60,
            ..Eff::new()
        };
        assert!(eff.eff_desc().contains("剑穗缓结"));
        let eff = Eff {
            special: 61,
            ..Eff::new()
        };
        assert!(eff.eff_desc().contains("剑成赠穗"));
        let eff = Eff {
            special: 62,
            ..Eff::new()
        };
        assert!(eff.eff_desc().contains("一夜结完"));
    }

    // 新批门控：227 老年层（余量≤10 入池/余量富余归零）、230 章节联动（bit7
    // 未了归零/已了入池）、224 稀有层 mr7（realm 达标入池且 w≤RARE_W）、
    // 219 链16 首段建链卡面（w3 善缘常态入池）
    #[test]
    fn v1170_batch_gates() {
        // 老年层 227：余量 5 ≤10 入池；余量 40 归零
        let mut g1 = sim_game(2, 40.0);
        {
            let s = g1.sim.as_mut().unwrap();
            s.lifespan = s.age + 5.0;
        }
        let (ws1, _) = g1.sim_event_weights(40.0, 2, false, false, false);
        assert!(ws1[227] > 0.0, "老年态 227 入池");
        let mut g2 = sim_game(2, 40.0);
        let (ws2, _) = g2.sim_event_weights(40.0, 2, false, false, false);
        assert_eq!(ws2[227], 0.0, "余量富余 227 归零");
        // 章节联动 230：bit7 未了归零；置位入池
        let mut g3 = sim_game(5, 40.0);
        let (ws3, _) = g3.sim_event_weights(40.0, 5, false, false, false);
        assert_eq!(ws3[230], 0.0, "八章未了 230 归零");
        g3.meta.chap_done |= 1 << 7;
        let (ws3b, _) = g3.sim_event_weights(40.0, 5, false, false, false);
        assert!(ws3b[230] > 0.0, "八章已了 230 入池");
        // 稀有层 224：mr7 达标入池 + w≤RARE_W
        let g4 = sim_game(7, 40.0);
        let (ws4, _) = g4.sim_event_weights(40.0, 7, false, false, false);
        assert!(ws4[224] > 0.0, "mr7 达标 224 入池");
        assert!(EVENTS[224].1 <= crate::data::RARE_W, "224 权重 ≤RARE_W");
        let g5 = sim_game(6, 40.0);
        let (ws5, _) = g5.sim_event_weights(40.0, 6, false, false, false);
        assert_eq!(ws5[224], 0.0, "realm<mr 归零（224 mr7 独占）");
        // 链16 首段 219：mr4 达标入池（w3 善缘常态）
        let g6 = sim_game(5, 40.0);
        let (ws6, _) = g6.sim_event_weights(40.0, 5, false, false, false);
        assert!(ws6[219] > 0.0, "mr4 达标 219 入池");
    }

    // 新批徽标：219 链起 / 220/221 链段 / 225/226 伴随（白名单）/ 227 老年 /
    // 230 章节 / 224 稀有（w2）/ 222/223/228/229 遭遇（八类单口全覆盖抽检）
    #[test]
    fn v1170_batch_badges_derive() {
        let b = crate::gallery::event_badge;
        assert_eq!(b(219).0, "链起");
        assert_eq!(b(220).0, "链段");
        assert_eq!(b(221).0, "链段");
        assert_eq!(b(225).0, "伴随");
        assert_eq!(b(226).0, "伴随");
        assert_eq!(b(227).0, "老年");
        assert_eq!(b(230).0, "章节");
        assert_eq!(b(224).0, "稀有");
        assert_eq!(b(222).0, "遭遇");
        assert_eq!(b(223).0, "遭遇");
        assert_eq!(b(228).0, "遭遇");
        assert_eq!(b(229).0, "遭遇");
        // 越界防御随表扩仍成立
        assert_eq!(b(999).0, "遭遇");
    }
}

// ================= v1.16.21 剧情画面·抉择后果可见化（回归三件套） =================

#[cfg(test)]
mod scene_viz_tests {
    use super::revenge_tests::game_with;
    use super::*;

    // 回归①（新画面元素显示与兜底）：build_event 统一挂类目徽标+场景行——徽标词
    // 必在九类词域（与图鉴 event_badge 单口同源）、场景行=「地名 · 天时」采样 sim
    // 现值；非池卡（道途卡）两字段恒 None 不与类目 chip 叠戴
    #[test]
    fn build_event_attaches_cat_and_scene() {
        let mut g = game_with(Life::default(), None);
        let mut s = Life::default();
        s.realm = 5;
        s.loc = crate::data::NODE_MARKET; // 临仙坊市
        s.weather = 2; // 雨
        g.sim = Some(s);
        g.modal = None;
        g.build_event(28); // 同页天成（meeting 版式同样带徽标/场景行）
        let card = g.modal.as_ref().expect("事件 28 无卡臂");
        let (w, _) = card.cat.expect("build_event 必挂类目徽标");
        assert!(
            [
                "伴随", "章节", "回响", "老年", "宿敌", "链起", "链段", "稀有", "遭遇",
            ]
            .contains(&w),
            "徽标词越九类词域: {}",
            w
        );
        assert_eq!(
            card.scene.as_deref(),
            Some("临仙坊市 · 雨"),
            "场景行应采样 sim 所在地·天时"
        );

        // 兜底：道途卡（非池卡）cat/scene/brk 全空，维持原版式语义
        g.rmodal = None;
        g.open_path_card();
        let p = g.rmodal.as_ref().expect("道途卡未弹");
        assert!(p.cat.is_none() && p.scene.is_none() && !p.brk);
    }

    // 回归②（抉择反馈正确）：apply_eff 通用通道出净变动聚合行（正/负/混合分色），
    // 结果句 good=青 / !good=琥珀（不再落疗愈绿）；聚合行在叙事句之前（故事句恒
    // 最新，与现实侧 apply_reff 同序）
    #[test]
    fn apply_eff_ledger_and_result_colors() {
        let mut g = game_with(Life::default(), None);
        g.sim = Some(Life::default());
        g.sim.as_mut().unwrap().stones = 1000;

        // 混合正负：灵石-300·道心+5 → 中性灰；good 叙事句 → 青
        let mut e = Eff::new();
        e.stones = -300;
        e.dao = 5.0;
        e.good = true;
        e.msg = vec!["试练叙事句".into()];
        g.apply_eff(&e);
        let n = g.sim_log.len();
        assert!(
            g.sim_log[n - 2].text.contains("灵石-300"),
            "聚合行应报灵石-300: {}",
            g.sim_log[n - 2].text
        );
        assert!(
            g.sim_log[n - 2].text.contains("道心+5"),
            "聚合行应报道心+5"
        );
        assert_eq!(g.sim_log[n - 2].color, C_DIM, "混合正负=中性灰");
        assert_eq!(g.sim_log[n - 1].color, C_CYAN, "good 结果句应青");

        // 全负：净变动行=琥珀（对价）；!good 叙事句=琥珀（新语义位）
        g.sim.as_mut().unwrap().stones = 1000;
        let mut e2 = Eff::new();
        e2.stones = -300;
        e2.good = false;
        e2.msg = vec!["代价叙事句".into()];
        g.apply_eff(&e2);
        let n = g.sim_log.len();
        assert_eq!(g.sim_log[n - 2].color, C_AMBER, "全负净变动=琥珀");
        assert_eq!(g.sim_log[n - 1].color, C_AMBER, "!good 结果句应琥珀");
        assert!(g.sim_log[n - 2].text.contains("灵石-300"));
    }

    // 回归③（文案预算）：场景行「地名 · 天时」全组合 ≤12 字（14px 单行右端预算）；
    // 净变动行通道全零不发行（无账可记不刷屏）；段格式=ASCII +/-（与 eff_desc 同语
    // 零新字），整数差值省小数位
    #[test]
    fn scene_and_ledger_budgets() {
        for l in crate::data::LOCATIONS.iter() {
            for w in crate::data::WEATHERS.iter() {
                let s = format!("{} · {}", l.name, w);
                assert!(s.chars().count() <= 12, "场景行越预算: {}", s);
            }
        }
        assert!(
            ledger_line(0, 0.0, 0.0, 0.0, 0.0, 0, 0).is_none(),
            "通道全零不发行"
        );
        let (t, c) = ledger_line(-300, 5.0, 0.0, 0.0, 0.0, 0, 0).unwrap();
        assert_eq!(t, "灵石-300·道心+5");
        assert_eq!(c, C_DIM, "混合正负=中性灰");
        let (t2, c2) = ledger_line(0, 0.0, 2.5, 0.0, 0.0, 0, 0).unwrap();
        assert_eq!(t2, "气运+2.5");
        assert_eq!(c2, C_CYAN, "全正=进益青");
        let (t3, c3) = ledger_line(0, 0.0, 0.0, 0.0, 0.0, -40, 150).unwrap();
        assert_eq!(t3, "道韵-40·贡献+150");
        assert_eq!(c3, C_DIM, "现实侧混合=中性灰");
    }
}

// ================= v1.16.21 可玩性深化 + 扩池批 9 条（231..=239） =================

#[cfg(test)]
mod playability_pool_tests {
    use super::revenge_tests::{game_with, neme};
    use super::*;

    fn sim_game(realm: usize, luck: f32) -> Game {
        let mut g = game_with(Life::default(), None);
        let mut s = Life::default();
        s.realm = realm;
        s.luck = luck;
        g.sim = Some(s);
        g
    }

    // 新批次良构：231..=239 每条 ≥2 且 ≤3 选项、标题=EVENT_NAMES 双源、正文单段
    // ≤60 字、简 ≤18 字；选第一项不 panic（237 opt0=奉纳非 fight 臂，安全）
    #[test]
    fn batch239_cards_well_formed() {
        for id in 231..=239 {
            let mut g = sim_game(5, 40.0);
            g.modal = None;
            g.build_event(id);
            let card = g
                .modal
                .as_ref()
                .unwrap_or_else(|| panic!("事件 {} 无卡臂（白抽）", id));
            assert_eq!(card.title, EVENT_NAMES[id], "事件 {} 标题双源漂移", id);
            assert!(
                !card.text.contains('\n'),
                "事件 {} 正文不得加段（单段预算）",
                id
            );
            assert!(
                card.text.chars().count() <= 60,
                "事件 {} 正文越 60 字: {}",
                id,
                card.text
            );
            let b = EVENT_BRIEF[id];
            assert!(
                !b.is_empty() && b.chars().count() <= 18,
                "事件 {} 简越 18 字: {} ({}字)",
                id,
                b,
                b.chars().count()
            );
            assert!(card.opts.len() >= 2, "事件 {} 至少两选项", id);
            assert!(card.opts.len() <= 3, "事件 {} 至多三选项", id);
            g.choose_option(0); // apply_eff 全链不 panic（sync_save 仅置脏不落盘）
        }
    }

    // 链17 全程：首段 231 置链待分支 → 以路相赠（special63）置下段 → sim_event
    // 直取 232 → 补全星轨（special64）置末段 → 直取 233 → 抉择（无置链臂）即链
    // 终局：清链 + 缘了闩 bit16 置位（bit0-15 既有链不动）
    #[test]
    fn chain17_advances_and_closes() {
        let mut g = sim_game(5, 40.0);
        g.modal = None;
        g.build_event(231);
        assert_eq!(g.chain, Some((17, 0, CHAIN_TTL)), "首段臂置链待分支");
        assert_eq!(g.modal.as_ref().unwrap().title, "星槎再访");
        assert_eq!(g.chain_latch & (1 << 16), 0, "缘了前闩未置");
        g.choose_option(0); // 以路相赠
        assert_eq!(g.chain, Some((17, 232, CHAIN_TTL)), "special63 置链写下段");
        g.modal = None;
        g.sim_event(); // 链活跃 → 直取 232（K 窗 retain 口径照链1 测试先例）
        assert_eq!(g.modal.as_ref().unwrap().title, "星轨留书·续");
        g.choose_option(0); // 补全星轨
        assert_eq!(g.chain, Some((17, 233, CHAIN_TTL)));
        g.modal = None;
        g.sim_event(); // 直取 233
        assert_eq!(g.modal.as_ref().unwrap().title, "槎去留星·续");
        g.choose_option(0); // 收下半幅星轨（无置链臂）
        assert!(g.chain.is_none(), "末段抉择即链终局");
        assert_eq!(g.chain_latch & (1 << 16), 1 << 16, "缘了闩 bit16 置位");
    }

    // special66 旧砚检定：两结局 apply_eff 全链不 panic、必出一结局文案、sim
    // 存活（66 臂无死亡口）；窗外落臂即清链语义天然成立（链 None 无副作用）
    #[test]
    fn special66_both_outcomes_safe() {
        for dao in [100.0f32, 0.0] {
            let mut g = sim_game(5, 40.0);
            g.sim.as_mut().unwrap().dao = dao;
            g.modal = None;
            g.build_event(234);
            g.choose_option(0); // 应声作答（special66 检定）
            let hit = g.sim_log.iter().any(|x| x.text.contains("道心+3"));
            let miss = g.sim_log.iter().any(|x| x.text.contains("轻伤"));
            assert!(hit || miss, "dao={} 检定 66 必出一结局", dao);
            assert!(g.sim.is_some(), "检定 66 无死亡口，sim 应存活");
        }
    }

    // 宿敌遭遇卡化：sim_neme 弹 EventCard 二选，首项=迎战（跳过档 choose_option(0)
    // 契约 2625/2663）；卡面言明胜算/悬殊；遁走（opt1）恒存活——40% 无决策死亡消灭
    #[test]
    fn neme_card_shape_and_flee_survives() {
        // 同境可战局
        let mut g = game_with(Life::default(), Some(neme(0)));
        g.sim = Some(Life::default());
        g.sim_neme();
        let card = g.modal.as_ref().expect("宿敌遭遇应弹卡");
        assert_eq!(card.title, "宿敌截道");
        assert_eq!(card.opts.len(), 2);
        assert_eq!(card.opts[0].0, "迎战！", "跳过档首项=迎战（保自动语义）");
        assert_eq!(card.opts[0].1.special, 67);
        assert_eq!(card.opts[1].1.special, 68);
        assert!(card.text.contains("胜算约"), "可战局卡面应言明胜算");
        g.choose_option(1); // 拼死遁走
        assert!(g.sim.is_some(), "遁走恒存活（无决策死亡消灭）");
        assert_eq!(g.sim.as_ref().unwrap().injured, 8);
        assert!(g.meta.neme.is_some(), "遁走不销宿敌");

        // 悬殊局（mr14 宿敌 vs 白丁）：正文含悬殊警示，遁走亦恒存活
        let mut g2 = game_with(Life::default(), Some(neme(14)));
        g2.sim = Some(Life::default());
        g2.sim_neme();
        let c2 = g2.modal.as_ref().unwrap();
        assert!(c2.text.contains("敌势悬殊"), "悬殊局正文应警示");
        g2.choose_option(1);
        assert!(g2.sim.is_some(), "悬殊局遁走亦恒存活");
    }

    // 继承未选折算：finish_inherit 把未选项各折一份余泽入道韵（3 选 1 留 2 份；
    // dao_yun 既有键零新键），余泽行入 log
    #[test]
    fn finish_inherit_folds_unpicked() {
        let mut g = game_with(Life::default(), None);
        g.inherit_opts = vec![1, 3, 4]; // 道果/法宝/财货 → 8+10+10
        let dy0 = g.meta.dao_yun;
        g.finish_inherit();
        assert_eq!(g.meta.dao_yun - dy0, 28, "未选三份各折一份余泽");
        assert!(g.log.iter().any(|x| x.text.contains("余泽入身")));
    }

    // 远征胜率单口：纯函数与卡面/结算同源——padj ±0.10 差距恒 0.20、界内单调、
    // clamp 上下界语义未动
    #[test]
    fn exp_p_pure_and_consistent() {
        let mut l = Life::default();
        l.realm = 2;
        l.body = 40.0;
        l.luck = 40.0;
        l.dao = 40.0;
        let p0 = crate::world::exp_p(&l, 0, -0.10, 0.0);
        let p1 = crate::world::exp_p(&l, 0, 0.10, 0.0);
        assert!((p1 - p0 - 0.20).abs() < 1e-4, "±10% 应恰 0.20 差");
        for kind in 0..4usize {
            for padj in [-0.10f32, 0.10] {
                let p = crate::world::exp_p(&l, kind, padj, 0.05);
                assert!(
                    (0.05..=0.95).contains(&p),
                    "kind{} padj{} 越界: {}",
                    kind,
                    padj,
                    p
                );
            }
        }
    }

    // 闭关出关摘要：会话态记账——闭关载累计实际修为增量（qi 0 起步必为正）。
    // 出路面收敛为两种并各自断言真实机制结果（复审修：原版用 contains("出关")
    // 做守卫，但 sim_demon 败臂「被迫出关」同样含「出关」且 45% 概率断关不走
    // act=2——原测试对随机检查器脆弱）：正常出关行必带「闭关2载，共得修为X」
    // 总账且账清；心魔断关则不走 act=2、会话账保留（下次入关才重置，机制如此）。
    // realm=1 起稿灭重闭关分支（闭关起始要求 realm>=2），出路面不发散
    #[test]
    fn seclusion_summary_session_only() {
        let mut g = sim_game(1, 40.0);
        {
            let s = g.sim.as_mut().unwrap();
            s.seclusion = Some(2);
            s.qi = 0.0;
        }
        g.secl_yrs = 2; // 模拟本会话入关（会话态不入档）
        for _ in 0..12 {
            g.sim_year();
            assert!(g.sim.is_some(), "心魔两臂/事件面均无死亡口，sim 不应殒");
            // 正常出关（末载无心魔，95%）：总账随行、账清
            if g
                .sim_log
                .iter()
                .any(|x| x.text.contains("出关，气息渊渟岳峙"))
            {
                assert!(
                    g.sim_log
                        .iter()
                        .any(|x| x.text.contains("闭关2载，共得修为")),
                    "正常出关行应带总账"
                );
                assert_eq!(g.secl_yrs, 0, "出关后账清");
                return;
            }
            // 非优雅出路面（末载/中途 5% 心魔：败臂被迫出关、化念臂吞 act=2）：
            // seclusion 已空、本载不走总账行、会话账保留（下次入关才重置——机制如此）
            if g.sim.as_ref().unwrap().seclusion.is_none() {
                assert_eq!(g.secl_yrs, 2, "非正常出关不清会话账");
                assert!(g.secl_qi > 0.0, "非正常出关保留已累计账");
                return;
            }
        }
        panic!("12 载内既未正常出关也未断关——机制面异常");
    }
}

// v1.16.21 复审修：新 special 63/64/66/67/68 副标入表回归——宿敌截道卡两钮/
// 检定66/链17 置链臂不再落「机缘」兜底（副标=选项后果摘要，兜底语义错位）
#[cfg(test)]
mod special_subtitle_fix_tests {
    use super::*;

    #[test]
    fn new_specials_subtitled_not_fallback() {
        let mut e = Eff::new();
        for (sp, want) in [
            (63u8, "续·星轨留书"),
            (64, "续·槎去留星"),
            (66, "受旧砚一问"),
            (67, "败则殒命"),
            (68, "重伤脱身"),
        ] {
            e.special = sp;
            assert_eq!(e.eff_desc(), want, "special {} 副标漂移", sp);
        }
        // 未登记 special 恒兜底「机缘」（65 预留窗位语义不变）
        e.special = 65;
        assert_eq!(e.eff_desc(), "机缘");
    }
}

