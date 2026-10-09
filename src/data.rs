// 数据表：境界、功法、法宝、丹药、天赋、轮回升级、成就、出身、事件权重
// 全部数值集中在此，便于平衡调整。

pub struct RealmDef {
    pub name: &'static str,
    pub layers: &'static [&'static str],
    pub life: f64,      // 进入该境界的寿命基准（年）
    pub qi: f64,        // 该境界第一层所需修为
    pub power: f64,     // 战力基准
    pub aura: (u8, u8, u8),
    pub bbreak: f32,    // 从上一大境界突破进入此境的基础成功率
    pub bdeath: f32,    // 突破失败身死概率
    pub immortal: bool, // 仙界境界（无走火入魔死劫）
}

const L12: [&str; 12] = [
    "一层", "二层", "三层", "四层", "五层", "六层", "七层", "八层", "九层", "十层", "十一层", "十二层",
];
const L4: [&str; 4] = ["初期", "中期", "后期", "大圆满"];
const L1: [&str; 1] = ["凡躯"];

pub const REALMS: [RealmDef; 15] = [
    RealmDef { name: "凡人", layers: &L1, life: 75.0, qi: 30.0, power: 5.0, aura: (170, 170, 170), bbreak: 1.0, bdeath: 0.0, immortal: false },
    RealmDef { name: "炼气", layers: &L12, life: 130.0, qi: 60.0, power: 18.0, aura: (185, 225, 255), bbreak: 1.0, bdeath: 0.0, immortal: false },
    RealmDef { name: "筑基", layers: &L4, life: 260.0, qi: 4000.0, power: 90.0, aura: (120, 255, 160), bbreak: 0.72, bdeath: 0.02, immortal: false },
    RealmDef { name: "金丹", layers: &L4, life: 550.0, qi: 16000.0, power: 420.0, aura: (255, 210, 90), bbreak: 0.55, bdeath: 0.04, immortal: false },
    RealmDef { name: "元婴", layers: &L4, life: 1100.0, qi: 65000.0, power: 2000.0, aura: (110, 170, 255), bbreak: 0.42, bdeath: 0.06, immortal: false },
    RealmDef { name: "化神", layers: &L4, life: 2200.0, qi: 260000.0, power: 9500.0, aura: (200, 120, 255), bbreak: 0.32, bdeath: 0.08, immortal: false },
    RealmDef { name: "炼虚", layers: &L4, life: 4500.0, qi: 1_050_000.0, power: 45000.0, aura: (95, 115, 255), bbreak: 0.24, bdeath: 0.10, immortal: false },
    RealmDef { name: "合体", layers: &L4, life: 9000.0, qi: 4_200_000.0, power: 210000.0, aura: (255, 110, 90), bbreak: 0.18, bdeath: 0.12, immortal: false },
    RealmDef { name: "大乘", layers: &L4, life: 18000.0, qi: 1.7e7, power: 1.0e6, aura: (255, 240, 200), bbreak: 0.13, bdeath: 0.15, immortal: false },
    RealmDef { name: "渡劫", layers: &L4, life: 36000.0, qi: 6.5e7, power: 4.5e6, aura: (170, 230, 255), bbreak: 0.10, bdeath: 0.0, immortal: false },
    RealmDef { name: "真仙", layers: &L4, life: 150000.0, qi: 2.6e8, power: 2.5e7, aura: (255, 190, 120), bbreak: 0.35, bdeath: 0.0, immortal: true },
    RealmDef { name: "金仙", layers: &L4, life: 600000.0, qi: 1.0e9, power: 1.2e8, aura: (255, 230, 140), bbreak: 0.30, bdeath: 0.0, immortal: true },
    RealmDef { name: "太乙", layers: &L4, life: 2.4e6, qi: 4.0e9, power: 6.0e8, aura: (170, 255, 220), bbreak: 0.26, bdeath: 0.0, immortal: true },
    RealmDef { name: "大罗", layers: &L4, life: 1.0e7, qi: 1.6e10, power: 3.0e9, aura: (255, 150, 220), bbreak: 0.22, bdeath: 0.0, immortal: true },
    RealmDef { name: "道祖", layers: &L4, life: 1.0e9, qi: 6.4e10, power: 1.5e10, aura: (255, 255, 255), bbreak: 0.18, bdeath: 0.0, immortal: true },
];

// 大境界内小境界修为倍率（初/中/后/大圆满）
pub const LAYER4: [f64; 4] = [1.0, 2.2, 4.8, 10.0];

// 死亡结算：达到各境界可得道韵
pub const MILESTONE: [i64; 15] = [
    0, 8, 25, 70, 180, 450, 1100, 2600, 6000, 14000, 32000, 75000, 170000, 420000, 1_100_000,
];

pub const REALM_COLORS: [(u8, u8, u8); 15] = [
    (170, 170, 170), (185, 225, 255), (120, 255, 160), (255, 210, 90), (110, 170, 255),
    (200, 120, 255), (95, 115, 255), (255, 110, 90), (255, 240, 200), (170, 230, 255),
    (255, 190, 120), (255, 230, 140), (170, 255, 220), (255, 150, 220), (255, 255, 255),
];

pub fn root_name(r: f32) -> &'static str {
    if r >= 100.0 { "混沌灵根" } else if r >= 90.0 { "圣灵根" } else if r >= 75.0 { "天灵根" }
    else if r >= 60.0 { "地灵根" } else if r >= 40.0 { "真灵根" } else if r >= 20.0 { "伪灵根" } else { "废灵根" }
}
pub fn root_color(r: f32) -> (u8, u8, u8) {
    if r >= 100.0 { (255, 255, 255) } else if r >= 90.0 { (255, 150, 220) } else if r >= 75.0 { (170, 255, 220) }
    else if r >= 60.0 { (120, 255, 160) } else if r >= 40.0 { (255, 210, 90) } else if r >= 20.0 { (185, 225, 255) } else { (150, 150, 150) }
}

// ---------------- 功法 ----------------
pub struct TechDef { pub name: &'static str, pub mult: f64, pub pow: f64, pub desc: &'static str }
pub const TECHS: [TechDef; 8] = [
    TechDef { name: "长春功", mult: 1.0, pow: 1.0, desc: "入门吐纳之法，绵绵不绝" },
    TechDef { name: "聚灵诀", mult: 1.3, pow: 1.0, desc: "聚天地灵气为己用" },
    TechDef { name: "大衍诀", mult: 1.6, pow: 1.0, desc: "衍化万物，道法自然" },
    TechDef { name: "青元剑诀", mult: 2.0, pow: 1.2, desc: "剑气纵横，攻伐无双" },
    TechDef { name: "太玄真经", mult: 2.5, pow: 1.1, desc: "玄之又玄，众妙之门" },
    TechDef { name: "玄天七杀诀", mult: 3.1, pow: 1.3, desc: "杀伐决绝，七杀成阵" },
    TechDef { name: "太上忘情录", mult: 3.8, pow: 1.15, desc: "忘情绝爱，道心不堕" },
    TechDef { name: "混沌天帝诀", mult: 4.8, pow: 1.4, desc: "传说中的开天之法" },
];

// ---------------- 法宝 ----------------
pub struct RelicDef { pub name: &'static str, pub pow: f64, pub cult: f64, pub desc: &'static str }
pub const RELICS: [RelicDef; 8] = [
    RelicDef { name: "青竹蜂云剑", pow: 0.15, cult: 1.0, desc: "七十二口飞剑，寒光凛冽" },
    RelicDef { name: "金雷竹剑", pow: 0.25, cult: 1.0, desc: "金雷竹所炼，坚不可摧" },
    RelicDef { name: "蕴灵葫芦", pow: 0.0, cult: 1.25, desc: "蕴养灵气，修炼加速" },
    RelicDef { name: "玄天斩灵剑", pow: 0.40, cult: 1.0, desc: "一剑斩灵，锐不可当" },
    RelicDef { name: "聚灵鼎", pow: 0.0, cult: 1.40, desc: "鼎中自成灵脉" },
    RelicDef { name: "八灵尺", pow: 0.60, cult: 1.0, desc: "上古异宝，镇压八方" },
    RelicDef { name: "玄天如意刃", pow: 0.90, cult: 1.0, desc: "心念所至，刃光所至" },
    RelicDef { name: "掌天瓶", pow: 1.50, cult: 1.30, desc: "神秘小瓶，可催熟万物" },
];

// ---------------- 丹药 ----------------
pub struct PillDef { pub name: &'static str, pub desc: &'static str, pub edible: bool }
pub const PILL_PRICES: [i64; 6] = [200, 1500, 2500, 800, 5000, 4000];
pub const PILLS: [PillDef; 6] = [
    PillDef { name: "聚气丹", desc: "服之修为大增（当前层6%）", edible: true },
    PillDef { name: "筑基丹", desc: "冲击筑基时自动服用 +25%", edible: false },
    PillDef { name: "破境丹", desc: "突破大境界 +12%（至多叠3）", edible: false },
    PillDef { name: "培元丹", desc: "延寿15年，疗愈重伤", edible: true },
    PillDef { name: "洗髓丹", desc: "永久提升灵根+2", edible: true },
    PillDef { name: "护神丹", desc: "渡劫时每重雷劫+10%存活", edible: false },
];

// ---------------- 天赋（开局抽取，10 选 3） ----------------
// tier: 0 普通 1 稀有 2 传说
#[derive(Clone, Copy, Default)]
pub struct TEff {
    pub root: f32,     // 灵根+
    pub dao: f32,      // 道心+
    pub luck: f32,     // 气运+
    pub body: f32,     // 体魄+
    pub life_add: f64, // 寿元+固定年
    pub life_mult: f64,
    pub stone: i64,        // 初始灵石+
    pub cult_mult: f64,    // 修炼倍率
    pub power_mult: f64,   // 战力倍率
    pub break_add: f32,    // 大境界突破率+
    pub trib_add: f32,     // 渡劫每重存活+
    pub event_mult: f32,   // 奇遇(善缘)权重倍率
    pub pill_mult: f64,    // 丹药效果倍率
    pub relic: Option<usize>,
    pub tech: Option<usize>,
}

pub struct TalentDef {
    pub name: &'static str,
    pub desc: &'static str,
    pub tier: u8,
    pub e: TEff,
}

pub const TALENTS: [TalentDef; 33] = [
    TalentDef {
        name: "灵光初绽", desc: "灵根 +10", tier: 0,
        e: TEff {
root: 10.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "道种入命", desc: "道心 +12", tier: 0,
        e: TEff {
root: 0.0, dao: 12.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "福缘深厚", desc: "气运 +12", tier: 0,
        e: TEff {
root: 0.0, dao: 0.0, luck: 12.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "筋骨强健", desc: "体魄 +12", tier: 0,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 12.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "龟鹤遐龄", desc: "寿元 +20 载", tier: 0,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 20.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "殷实人家", desc: "初始灵石 +800", tier: 0,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 800, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "勤能补拙", desc: "修炼速度 ×1.15", tier: 0,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 1.15, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "虎豹雷音", desc: "战力 ×1.25", tier: 0,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 1.25, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "心如磐石", desc: "突破成功率 +4%", tier: 0,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.04, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "命硬福厚", desc: "渡劫存活 +4%/重", tier: 0,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.04, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "慧眼识珠", desc: "善缘奇遇 ×1.25", tier: 0,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 1.25, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "药王亲传", desc: "丹药效果 ×1.4", tier: 0,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 1.4,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "灵苗仙种", desc: "灵根 +16", tier: 0,
        e: TEff {
root: 16.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "含玉而生", desc: "初始灵石 +1500", tier: 0,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 1500, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "松鹤延年", desc: "寿元 +40 载", tier: 0,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 40.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "气海辽阔", desc: "灵根 +6，道心 +6", tier: 0,
        e: TEff {
root: 6.0, dao: 6.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "三清道韵", desc: "修炼速度 ×1.4", tier: 1,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 1.4, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "武曲星临", desc: "战力 ×1.55", tier: 1,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 1.55, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "逆天改命", desc: "突破成功率 +9%", tier: 1,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.09, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "九天雷体", desc: "渡劫存活 +11%/重", tier: 1,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.11, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "仙缘广布", desc: "善缘奇遇 ×1.55", tier: 1,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 1.55, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "丹道通神", desc: "丹药效果 ×2.0", tier: 1,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 2.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "百岁光阴", desc: "寿元 ×1.35", tier: 1,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 1.35, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "天纵奇才", desc: "灵根 +22，体魄 +8", tier: 1,
        e: TEff {
root: 22.0, dao: 0.0, luck: 0.0, body: 8.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "家传法宝", desc: "出生自带【蕴灵葫芦】", tier: 1,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: Some(2),
            tech: None,
        },
    },
    TalentDef {
        name: "祖传经卷", desc: "出生自带《大衍诀》", tier: 1,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: Some(2),
        },
    },
    TalentDef {
        name: "混沌道体", desc: "修炼 ×1.65，灵根 +15", tier: 2,
        e: TEff {
root: 15.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 1.65, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "天命之子", desc: "气运 +40", tier: 2,
        e: TEff {
root: 0.0, dao: 0.0, luck: 40.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "雷帝转世", desc: "渡劫存活 +20%/重", tier: 2,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.20, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "剑仙转世", desc: "战力 ×1.9", tier: 2,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 1.9, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "长生道体", desc: "寿元 ×1.7", tier: 2,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 1.7, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "掌天瓶转世", desc: "出生自带【掌天瓶】", tier: 2,
        e: TEff {
root: 0.0, dao: 0.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.0, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: Some(7),
            tech: None,
        },
    },
    TalentDef {
        name: "菩提圣心", desc: "道心 +40，突破 +10%", tier: 2,
        e: TEff {
root: 0.0, dao: 40.0, luck: 0.0, body: 0.0, life_add: 0.0, life_mult: 0.0, stone: 0, cult_mult: 0.0, power_mult: 0.0, break_add: 0.10, trib_add: 0.0, event_mult: 0.0, pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
];

pub const TIER_NAMES: [&str; 3] = ["凡品", "灵品", "仙品"];
pub const TIER_COLORS: [(u8, u8, u8); 3] = [(190, 190, 190), (130, 220, 235), (255, 190, 90)];

// ---------------- 轮回烙印（道韵永久强化） ----------------
pub struct UpgDef { pub name: &'static str, pub desc: &'static str, pub max: i32, pub base: i64, pub growth: f64 }
pub const UPGRADES: [UpgDef; 10] = [
    UpgDef { name: "灵根天成", desc: "灵根下限 +8/级", max: 6, base: 25, growth: 1.7 },
    UpgDef { name: "道心通明", desc: "道心下限 +8/级", max: 6, base: 25, growth: 1.7 },
    UpgDef { name: "气运加身", desc: "气运下限 +8/级", max: 6, base: 25, growth: 1.7 },
    UpgDef { name: "铜皮铁骨", desc: "体魄下限 +8/级", max: 6, base: 25, growth: 1.7 },
    UpgDef { name: "造化青睐", desc: "每次模拟可继承 +1 项", max: 1, base: 1200, growth: 1.0 },
    UpgDef { name: "天赋异禀", desc: "模拟天赋 3选1 → 4选2", max: 1, base: 800, growth: 1.0 },
    UpgDef { name: "转世富商", desc: "初始灵石 ×2/级", max: 3, base: 60, growth: 2.5 },
    UpgDef { name: "轮回记忆", desc: "修炼速度 +12%/级", max: 5, base: 40, growth: 2.0 },
    UpgDef { name: "龟息延年", desc: "寿元 +10%/级", max: 5, base: 50, growth: 2.0 },
    UpgDef { name: "点数亲和", desc: "模拟 25% 概率返还点数", max: 1, base: 1500, growth: 1.0 },
];
pub fn upg_cost(id: usize, lv: i32) -> i64 {
    let u = &UPGRADES[id];
    (u.base as f64 * u.growth.powi(lv)).ceil() as i64
}

// ---------------- 成就（首达奖励道韵） ----------------
pub struct AchDef { pub name: &'static str, pub desc: &'static str, pub reward: i64 }
pub const ACHS: [AchDef; 22] = [
    AchDef { name: "踏上仙途", desc: "进入炼气期", reward: 10 },
    AchDef { name: "筑基成功", desc: "踏入筑基期", reward: 30 },
    AchDef { name: "金丹大道", desc: "结成金丹", reward: 80 },
    AchDef { name: "元婴老祖", desc: "孕育元婴", reward: 200 },
    AchDef { name: "化神通天", desc: "修至化神", reward: 500 },
    AchDef { name: "炼虚合道", desc: "修至炼虚", reward: 1200 },
    AchDef { name: "天人合一", desc: "修至合体", reward: 3000 },
    AchDef { name: "大乘至尊", desc: "修至大乘", reward: 8000 },
    AchDef { name: "劫数临头", desc: "进入渡劫期", reward: 20000 },
    AchDef { name: "飞升成仙", desc: "渡过九重雷劫", reward: 50000 },
    AchDef { name: "位证金仙", desc: "修至金仙", reward: 200000 },
    AchDef { name: "大罗至尊", desc: "修至大罗", reward: 1000000 },
    AchDef { name: "万古长青", desc: "活过十万岁", reward: 2000 },
    AchDef { name: "十世轮回", desc: "累计轮回十世", reward: 300 },
    AchDef { name: "百世轮回", desc: "累计轮回百世", reward: 30000 },
    AchDef { name: "富甲一方", desc: "身家百万灵石", reward: 200 },
    AchDef { name: "战力通天", desc: "战力破十亿", reward: 5000 },
    AchDef { name: "洗髓伐毛", desc: "服用洗髓丹10枚", reward: 300 },
    AchDef { name: "剑仙之姿", desc: "剑心通明+结丹", reward: 400 },
    AchDef { name: "寿比南山", desc: "活过千岁", reward: 80 },
    AchDef { name: "宗门泰斗", desc: "加入宗门并结成金丹", reward: 150 },
    AchDef { name: "一世长生", desc: "自然坐化且曾达元婴", reward: 500 },
];

// ---------------- 出身 ----------------
pub struct OriginDef { pub name: &'static str, pub desc: &'static str }
pub const ORIGINS: [OriginDef; 6] = [
    OriginDef { name: "农家子弟", desc: "寒门出身，白手起家" },
    OriginDef { name: "商贾之家", desc: "家资殷实，见多识广" },
    OriginDef { name: "修仙家族", desc: "祖上曾出过筑基修士" },
    OriginDef { name: "没落世家", desc: "藏书万卷，家风清正" },
    OriginDef { name: "山村猎户", desc: "筋骨强健，胆气过人" },
    OriginDef { name: "渔樵江渚", desc: "生于烟波，性近自然" },
];

// ---------------- 游历事件表 (id, 权重, 最低境界, 是否善缘) ----------------
pub const EVENTS: [(usize, i32, usize, bool); 20] = [
    (0, 12, 0, true),   // 灵草满坡
    (1, 11, 0, false),  // 妖兽拦路
    (2, 6, 1, true),    // 上古洞府
    (3, 8, 0, true),    // 坊市淘宝
    (4, 8, 1, false),   // 恶修劫道
    (5, 8, 0, true),    // 道友论道
    (6, 5, 1, true),    // 宗门相邀
    (7, 6, 2, false),   // 心魔袭扰
    (8, 5, 2, true),    // 天材地宝
    (9, 3, 2, true),    // 灵脉洞府
    (10, 2, 3, true),   // 上古传承
    (11, 6, 2, true),   // 秘境探秘
    (12, 6, 0, true),   // 红尘历练
    (13, 5, 0, true),   // 灵气潮汐
    (14, 6, 2, false),  // 邪修偷袭
    (15, 1, 4, true),   // 仙人遗府
    (16, 4, 2, true),   // 妖丹入手
    (17, 5, 1, true),   // 云游商人
    (18, 5, 1, true),   // 前辈指点
    (19, 3, 2, true),   // 灵兽认主
];

pub const MINOR_LINES: [&str; 10] = [
    "山间灵雾弥漫，你吐纳一夜，神清气爽。",
    "你在瀑布下打坐，听水声悟道。",
    "月圆之夜，你观星象有所感触。",
    "一只灵雀落在你肩头，叽喳半日方去。",
    "你抄录道经一卷，字字入心。",
    "山下雨了，你于檐下煮茶论道。",
    "你梦中见青山万重，醒来隐约有所悟。",
    "你采药归来，路上救了一只受伤的灵狐。",
    "夜里剑鸣不止，你起身舞剑一回。",
    "你在崖边远眺，云海翻涌如潮。",
];
