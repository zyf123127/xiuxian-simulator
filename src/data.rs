// 数据表：境界、功法、法宝、丹药、天赋、轮回升级、成就、出身、事件权重
// 全部数值集中在此，便于平衡调整。

pub struct RealmDef {
    pub name: &'static str,
    pub layers: &'static [&'static str],
    pub life: f64,  // 进入该境界的寿命基准（年）
    pub qi: f64,    // 该境界第一层所需修为
    pub power: f64, // 战力基准
    pub aura: (u8, u8, u8),
    pub bbreak: f32,    // 从上一大境界突破进入此境的基础成功率
    pub bdeath: f32,    // 突破失败身死概率
    pub immortal: bool, // 仙界境界（无走火入魔死劫）
}

const L12: [&str; 12] = [
    "一阶",
    "二阶",
    "三阶",
    "四阶",
    "五阶",
    "六阶",
    "七阶",
    "八阶",
    "九阶",
    "十阶",
    "十一阶",
    "十二阶",
];
const L4: [&str; 4] = ["初期", "中期", "后期", "大圆满"];
const L1: [&str; 1] = ["凡躯"];

pub const REALMS: [RealmDef; 15] = [
    RealmDef {
        name: "白丁",
        layers: &L1,
        life: 75.0,
        qi: 30.0,
        power: 5.0,
        aura: (170, 170, 170),
        bbreak: 1.0,
        bdeath: 0.0,
        immortal: false,
    },
    RealmDef {
        name: "沁墨",
        layers: &L12,
        life: 130.0,
        qi: 60.0,
        power: 18.0,
        aura: (185, 225, 255),
        bbreak: 1.0,
        bdeath: 0.0,
        immortal: false,
    },
    RealmDef {
        name: "成行",
        layers: &L4,
        life: 260.0,
        qi: 4000.0,
        power: 90.0,
        aura: (120, 255, 160),
        bbreak: 0.72,
        bdeath: 0.02,
        immortal: false,
    },
    RealmDef {
        name: "洇章",
        layers: &L4,
        life: 550.0,
        qi: 16000.0,
        power: 420.0,
        aura: (255, 210, 90),
        bbreak: 0.55,
        bdeath: 0.04,
        immortal: false,
    },
    RealmDef {
        name: "立骨",
        layers: &L4,
        life: 1100.0,
        qi: 65000.0,
        power: 2000.0,
        aura: (110, 170, 255),
        bbreak: 0.42,
        bdeath: 0.06,
        immortal: false,
    },
    RealmDef {
        name: "生花",
        layers: &L4,
        life: 2200.0,
        qi: 260000.0,
        power: 9500.0,
        aura: (200, 120, 255),
        bbreak: 0.32,
        bdeath: 0.08,
        immortal: false,
    },
    RealmDef {
        name: "藏锋",
        layers: &L4,
        life: 4500.0,
        qi: 1_050_000.0,
        power: 45000.0,
        aura: (95, 115, 255),
        bbreak: 0.24,
        bdeath: 0.10,
        immortal: false,
    },
    RealmDef {
        name: "铁画",
        layers: &L4,
        life: 9000.0,
        qi: 4_200_000.0,
        power: 210000.0,
        aura: (255, 110, 90),
        bbreak: 0.18,
        bdeath: 0.12,
        immortal: false,
    },
    RealmDef {
        name: "传世",
        layers: &L4,
        life: 18000.0,
        qi: 1.7e7,
        power: 1.0e6,
        aura: (255, 240, 200),
        bbreak: 0.13,
        bdeath: 0.15,
        immortal: false,
    },
    RealmDef {
        name: "问笔",
        layers: &L4,
        life: 36000.0,
        qi: 6.5e7,
        power: 4.5e6,
        aura: (170, 230, 255),
        bbreak: 0.10,
        bdeath: 0.0,
        immortal: false,
    },
    RealmDef {
        name: "墨仙",
        layers: &L4,
        life: 150000.0,
        qi: 2.6e8,
        power: 2.5e7,
        aura: (255, 190, 120),
        bbreak: 0.35,
        bdeath: 0.0,
        immortal: true,
    },
    RealmDef {
        name: "砚尊",
        layers: &L4,
        life: 600000.0,
        qi: 1.0e9,
        power: 1.2e8,
        aura: (255, 230, 140),
        bbreak: 0.30,
        bdeath: 0.0,
        immortal: true,
    },
    RealmDef {
        name: "秉笔",
        layers: &L4,
        life: 2.4e6,
        qi: 4.0e9,
        power: 6.0e8,
        aura: (170, 255, 220),
        bbreak: 0.26,
        bdeath: 0.0,
        immortal: true,
    },
    RealmDef {
        name: "执卷",
        layers: &L4,
        life: 1.0e7,
        qi: 1.6e10,
        power: 3.0e9,
        aura: (255, 150, 220),
        bbreak: 0.22,
        bdeath: 0.0,
        immortal: true,
    },
    RealmDef {
        name: "天笔",
        layers: &L4,
        life: 1.0e9,
        qi: 6.4e10,
        power: 1.5e10,
        aura: (255, 255, 255),
        bbreak: 0.18,
        bdeath: 0.0,
        immortal: true,
    },
];

// 大境界内小境界修为倍率（初/中/后/大圆满）
pub const LAYER4: [f64; 4] = [1.0, 2.2, 4.8, 10.0];

// 死亡结算：达到各境界可得道韵
pub const MILESTONE: [i64; 15] = [
    0, 8, 25, 70, 180, 450, 1100, 2600, 6000, 14000, 32000, 75000, 170000, 420000, 1_100_000,
];

pub const REALM_COLORS: [(u8, u8, u8); 15] = [
    (170, 170, 170),
    (185, 225, 255),
    (120, 255, 160),
    (255, 210, 90),
    (110, 170, 255),
    (200, 120, 255),
    (95, 115, 255),
    (255, 110, 90),
    (255, 240, 200),
    (170, 230, 255),
    (255, 190, 120),
    (255, 230, 140),
    (170, 255, 220),
    (255, 150, 220),
    (255, 255, 255),
];

pub fn root_name(r: f32) -> &'static str {
    if r >= 100.0 {
        "混沌灵根"
    } else if r >= 90.0 {
        "圣灵根"
    } else if r >= 75.0 {
        "天灵根"
    } else if r >= 60.0 {
        "地灵根"
    } else if r >= 40.0 {
        "真灵根"
    } else if r >= 20.0 {
        "伪灵根"
    } else {
        "废灵根"
    }
}
pub fn root_color(r: f32) -> (u8, u8, u8) {
    if r >= 100.0 {
        (255, 255, 255)
    } else if r >= 90.0 {
        (255, 150, 220)
    } else if r >= 75.0 {
        (170, 255, 220)
    } else if r >= 60.0 {
        (120, 255, 160)
    } else if r >= 40.0 {
        (255, 210, 90)
    } else if r >= 20.0 {
        (185, 225, 255)
    } else {
        (150, 150, 150)
    }
}

// ---------------- 功法 ----------------
pub struct TechDef {
    pub name: &'static str,
    pub mult: f64,
    pub pow: f64,
    pub desc: &'static str,
}
pub const TECHS: [TechDef; 8] = [
    TechDef {
        name: "养行篇",
        mult: 1.0,
        pow: 1.0,
        desc: "入门养行之法，字缓行匀，绵绵不绝",
    },
    TechDef {
        name: "勾留诀",
        mult: 1.3,
        pow: 1.0,
        desc: "勾留行间灵气，皆为己用",
    },
    TechDef {
        name: "漏行书",
        mult: 1.6,
        pow: 1.0,
        desc: "行间漏字，藏尽天机",
    },
    TechDef {
        name: "横行赋",
        mult: 2.0,
        pow: 1.2,
        desc: "一赋横行，笔锋攻伐无双",
    },
    TechDef {
        name: "藏锋经",
        mult: 2.5,
        pow: 1.1,
        desc: "锋藏笔中，深不可测",
    },
    TechDef {
        name: "断行七策",
        mult: 3.1,
        pow: 1.3,
        desc: "七策断行，字字杀伐",
    },
    TechDef {
        name: "无行录",
        mult: 3.8,
        pow: 1.15,
        desc: "无行无列，道心不堕",
    },
    TechDef {
        name: "天笔遗墨",
        mult: 4.8,
        pow: 1.4,
        desc: "天笔遗落的一滴墨，可开万卷",
    },
];

// ---------------- 法宝 ----------------
pub struct RelicDef {
    pub name: &'static str,
    pub pow: f64,
    pub cult: f64,
    pub desc: &'static str,
    /// v1.7.0 献祭折算的模拟点数（永久失去法宝；按 pow+cult 强弱定价）
    pub pt: i32,
}
pub const RELICS: [RelicDef; 8] = [
    RelicDef {
        name: "呵霜毫",
        pow: 0.15,
        cult: 1.0,
        desc: "笔毫呵气成霜，落笔生寒",
        pt: 3,
    },
    RelicDef {
        name: "裁云笺",
        pow: 0.25,
        cult: 1.0,
        desc: "裁云作笺，笺缘利可裂金",
        pt: 4,
    },
    RelicDef {
        name: "镇行钉",
        pow: 0.0,
        cult: 1.25,
        desc: "钉住书行，墨不散，笔愈疾",
        pt: 4,
    },
    RelicDef {
        name: "墨骨伞",
        pow: 0.40,
        cult: 1.0,
        desc: "墨骨撑开，来墨皆挡于伞外",
        pt: 6,
    },
    RelicDef {
        name: "曝书炉",
        pow: 0.0,
        cult: 1.40,
        desc: "炉上曝书，墨香入骨自养人",
        pt: 7,
    },
    RelicDef {
        name: "闲章",
        pow: 0.60,
        cult: 1.0,
        desc: "闲章盖处，一行即是一疆",
        pt: 9,
    },
    RelicDef {
        name: "洗笔江",
        pow: 0.90,
        cult: 1.0,
        desc: "一江洗笔，笔锋所指，江开路现",
        pt: 12,
    },
    RelicDef {
        name: "无字簿",
        pow: 1.50,
        cult: 1.30,
        desc: "满簿无字，落笔皆成真",
        pt: 15,
    },
];

// ---------------- v1.7.0 点数经济：死档广告 ----------------
/// 看完一次幻境广告获得的模拟点数
pub const AD_REWARD: i32 = 2;
/// 幻境广告时长（真实秒）
pub const AD_SECONDS: f32 = 5.0;
/// 轮回商阁假广告创意（按已看次数轮换）
pub const AD_CREATIVES: [&str; 3] = [
    "琼华仙丹 · 服之延寿一纪",
    "青云宗 · 今岁招收外门弟子",
    "轮回镜保养指南 · 镜灵亲自认证",
];

// ---------------- 丹药 ----------------
pub struct PillDef {
    pub name: &'static str,
    pub desc: &'static str,
    pub edible: bool,
}
pub const PILL_PRICES: [i64; 6] = [200, 1500, 2500, 800, 5000, 4000];
pub const PILLS: [PillDef; 6] = [
    PillDef {
        name: "聚气丹",
        desc: "服之修为大增（当前层6%）",
        edible: true,
    },
    PillDef {
        name: "成行丹",
        desc: "冲击成行境时自动服用 +25%",
        edible: false,
    },
    PillDef {
        name: "破境丹",
        desc: "突破大境界 +12%（至多叠3）",
        edible: false,
    },
    PillDef {
        name: "培元丹",
        desc: "延寿15年，疗愈重伤",
        edible: true,
    },
    PillDef {
        name: "洗髓丹",
        desc: "永久提升灵根+2",
        edible: true,
    },
    PillDef {
        name: "护神丹",
        desc: "问笔时每重雷劫+10%存活",
        edible: false,
    },
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
    pub stone: i64,      // 初始灵石+
    pub cult_mult: f64,  // 修炼倍率
    pub power_mult: f64, // 战力倍率
    pub break_add: f32,  // 大境界突破率+
    pub trib_add: f32,   // 问笔每重存活+
    pub event_mult: f32, // 奇遇(善缘)权重倍率
    pub pill_mult: f64,  // 丹药效果倍率
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
        name: "灵光初绽",
        desc: "灵根 +10",
        tier: 0,
        e: TEff {
            root: 10.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "道种入命",
        desc: "道心 +12",
        tier: 0,
        e: TEff {
            root: 0.0,
            dao: 12.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "福缘深厚",
        desc: "气运 +12",
        tier: 0,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 12.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "筋骨强健",
        desc: "体魄 +12",
        tier: 0,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 12.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "龟鹤遐龄",
        desc: "寿元 +20 载",
        tier: 0,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 20.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "殷实人家",
        desc: "初始灵石 +800",
        tier: 0,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 800,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "勤能补拙",
        desc: "修炼速度 ×1.15",
        tier: 0,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 1.15,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "虎豹雷音",
        desc: "战力 ×1.25",
        tier: 0,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 1.25,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "心如磐石",
        desc: "突破成功率 +4%",
        tier: 0,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.04,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "命硬福厚",
        desc: "问笔存活 +4%/重",
        tier: 0,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.04,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "慧眼识珠",
        desc: "善缘奇遇 ×1.25",
        tier: 0,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 1.25,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "药王亲传",
        desc: "丹药效果 ×1.4",
        tier: 0,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 1.4,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "灵苗仙种",
        desc: "灵根 +16",
        tier: 0,
        e: TEff {
            root: 16.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "含玉而生",
        desc: "初始灵石 +1500",
        tier: 0,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 1500,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "松鹤延年",
        desc: "寿元 +40 载",
        tier: 0,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 40.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "气海辽阔",
        desc: "灵根 +6，道心 +6",
        tier: 0,
        e: TEff {
            root: 6.0,
            dao: 6.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "三清道韵",
        desc: "修炼速度 ×1.4",
        tier: 1,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 1.4,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "武曲星临",
        desc: "战力 ×1.55",
        tier: 1,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 1.55,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "逆天改命",
        desc: "突破成功率 +9%",
        tier: 1,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.09,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "九天雷体",
        desc: "问笔存活 +11%/重",
        tier: 1,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.11,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "仙缘广布",
        desc: "善缘奇遇 ×1.55",
        tier: 1,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 1.55,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "丹道通神",
        desc: "丹药效果 ×2.0",
        tier: 1,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 2.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "百岁光阴",
        desc: "寿元 ×1.35",
        tier: 1,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 1.35,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "天纵奇才",
        desc: "灵根 +22，体魄 +8",
        tier: 1,
        e: TEff {
            root: 22.0,
            dao: 0.0,
            luck: 0.0,
            body: 8.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "家传法宝",
        desc: "出生自带【镇行钉】",
        tier: 1,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: Some(2),
            tech: None,
        },
    },
    TalentDef {
        name: "祖传经卷",
        desc: "出生自带《漏行书》",
        tier: 1,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: Some(2),
        },
    },
    TalentDef {
        name: "混沌道体",
        desc: "修炼 ×1.65，灵根 +15",
        tier: 2,
        e: TEff {
            root: 15.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 1.65,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "天命之子",
        desc: "气运 +40",
        tier: 2,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 40.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "雷帝转世",
        desc: "问笔存活 +20%/重",
        tier: 2,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.20,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "剑仙转世",
        desc: "战力 ×1.9",
        tier: 2,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 1.9,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "长生道体",
        desc: "寿元 ×1.7",
        tier: 2,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 1.7,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
    TalentDef {
        name: "无底砚转世",
        desc: "出生自带【无字簿】，落笔皆成真",
        tier: 2,
        e: TEff {
            root: 0.0,
            dao: 0.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.0,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: Some(7),
            tech: None,
        },
    },
    TalentDef {
        name: "菩提圣心",
        desc: "道心 +40，突破 +10%",
        tier: 2,
        e: TEff {
            root: 0.0,
            dao: 40.0,
            luck: 0.0,
            body: 0.0,
            life_add: 0.0,
            life_mult: 0.0,
            stone: 0,
            cult_mult: 0.0,
            power_mult: 0.0,
            break_add: 0.10,
            trib_add: 0.0,
            event_mult: 0.0,
            pill_mult: 0.0,
            relic: None,
            tech: None,
        },
    },
];

pub const TIER_NAMES: [&str; 3] = ["凡品", "灵品", "仙品"];
pub const TIER_COLORS: [(u8, u8, u8); 3] = [(190, 190, 190), (130, 220, 235), (255, 190, 90)];

// ---------------- 轮回烙印（道韵永久强化） ----------------
pub struct UpgDef {
    pub name: &'static str,
    pub desc: &'static str,
    pub max: i32,
    pub base: i64,
    pub growth: f64,
}
pub const UPGRADES: [UpgDef; 10] = [
    UpgDef {
        name: "灵根天成",
        desc: "灵根下限 +8/级",
        max: 6,
        base: 25,
        growth: 1.7,
    },
    UpgDef {
        name: "道心通明",
        desc: "道心下限 +8/级",
        max: 6,
        base: 25,
        growth: 1.7,
    },
    UpgDef {
        name: "气运加身",
        desc: "气运下限 +8/级",
        max: 6,
        base: 25,
        growth: 1.7,
    },
    UpgDef {
        name: "铜皮铁骨",
        desc: "体魄下限 +8/级",
        max: 6,
        base: 25,
        growth: 1.7,
    },
    UpgDef {
        name: "造化青睐",
        desc: "每次模拟可继承 +1 项",
        max: 1,
        base: 1200,
        growth: 1.0,
    },
    UpgDef {
        name: "天赋异禀",
        desc: "模拟天赋 3选1 → 4选2",
        max: 1,
        base: 800,
        growth: 1.0,
    },
    UpgDef {
        name: "转世富商",
        desc: "初始灵石 ×2/级",
        max: 3,
        base: 60,
        growth: 2.5,
    },
    UpgDef {
        name: "轮回记忆",
        desc: "修炼速度 +12%/级",
        max: 5,
        base: 40,
        growth: 2.0,
    },
    UpgDef {
        name: "龟息延年",
        desc: "寿元 +10%/级",
        max: 5,
        base: 50,
        growth: 2.0,
    },
    UpgDef {
        name: "点数亲和",
        desc: "模拟 25% 概率返还点数",
        max: 1,
        base: 1500,
        growth: 1.0,
    },
];
pub fn upg_cost(id: usize, lv: i32) -> i64 {
    let u = &UPGRADES[id];
    (u.base as f64 * u.growth.powi(lv)).ceil() as i64
}

// ---------------- 成就（首达奖励道韵） ----------------
pub struct AchDef {
    pub name: &'static str,
    pub desc: &'static str,
    pub reward: i64,
}
pub const ACHS: [AchDef; 30] = [
    AchDef {
        name: "踏上书途",
        desc: "沁墨初成",
        reward: 10,
    },
    AchDef {
        name: "自成一行",
        desc: "踏入成行境",
        reward: 30,
    },
    AchDef {
        name: "洇章成文",
        desc: "凝成洇章",
        reward: 80,
    },
    AchDef {
        name: "字字立骨",
        desc: "书中立骨",
        reward: 200,
    },
    AchDef {
        name: "妙笔生花",
        desc: "修至生花",
        reward: 500,
    },
    AchDef {
        name: "藏锋守拙",
        desc: "修至藏锋",
        reward: 1200,
    },
    AchDef {
        name: "书我合一",
        desc: "修至铁画",
        reward: 3000,
    },
    AchDef {
        name: "书成传世",
        desc: "修至传世",
        reward: 8000,
    },
    AchDef {
        name: "问笔临头",
        desc: "进入问笔境",
        reward: 20000,
    },
    AchDef {
        name: "飞升成仙",
        desc: "渡过九重雷劫",
        reward: 50000,
    },
    AchDef {
        name: "位证砚尊",
        desc: "修至砚尊",
        reward: 200000,
    },
    AchDef {
        name: "执卷至尊",
        desc: "修至执卷",
        reward: 1000000,
    },
    AchDef {
        name: "万古长青",
        desc: "活过十万岁",
        reward: 2000,
    },
    AchDef {
        name: "十世轮回",
        desc: "累计轮回十世",
        reward: 300,
    },
    AchDef {
        name: "百世轮回",
        desc: "累计轮回百世",
        reward: 30000,
    },
    AchDef {
        name: "富甲一方",
        desc: "身家百万灵石",
        reward: 200,
    },
    AchDef {
        name: "战力通天",
        desc: "战力破十亿",
        reward: 5000,
    },
    AchDef {
        name: "洗髓伐毛",
        desc: "服用洗髓丹10枚",
        reward: 300,
    },
    AchDef {
        name: "剑仙之姿",
        desc: "剑心通明+结丹",
        reward: 400,
    },
    AchDef {
        name: "寿比南山",
        desc: "活过千岁",
        reward: 80,
    },
    AchDef {
        name: "宗门泰斗",
        desc: "加入宗门并凝成洇章",
        reward: 150,
    },
    AchDef {
        name: "一世长生",
        desc: "自然坐化且曾立骨",
        reward: 500,
    },
    AchDef {
        name: "道侣结缘",
        desc: "模拟中结为道侣",
        reward: 200,
    },
    AchDef {
        name: "宗门魁首",
        desc: "宗门大比夺魁",
        reward: 150,
    },
    AchDef {
        name: "蓄势而成",
        desc: "蓄势圆满后破境",
        reward: 100,
    },
    AchDef {
        name: "镜灵赐福",
        desc: "触发镜灵赐福",
        reward: 200,
    },
    AchDef {
        name: "三世情缘",
        desc: "三世模拟中皆结道侣",
        reward: 500,
    },
    AchDef {
        name: "现实飞升",
        desc: "现实中问笔飞升",
        reward: 100000,
    },
    AchDef {
        name: "富可敌国",
        desc: "现实身家上亿灵石",
        reward: 2000,
    },
    AchDef {
        name: "千秋万代",
        desc: "现实享年五千载",
        reward: 1000,
    },
];

// ---------------- 出身 ----------------
pub struct OriginDef {
    pub name: &'static str,
    pub desc: &'static str,
}
pub const ORIGINS: [OriginDef; 6] = [
    OriginDef {
        name: "农家子弟",
        desc: "寒门出身，白手起家",
    },
    OriginDef {
        name: "商贾之家",
        desc: "家资殷实，见多识广",
    },
    OriginDef {
        name: "修仙家族",
        desc: "祖上曾出过成行修士",
    },
    OriginDef {
        name: "没落世家",
        desc: "藏书万卷，家风清正",
    },
    OriginDef {
        name: "山村猎户",
        desc: "筋骨强健，胆气过人",
    },
    OriginDef {
        name: "渔樵江渚",
        desc: "生于烟波，性近自然",
    },
];

// ---------------- 游历事件表 (id, 权重, 最低境界, 是否善缘) ----------------
pub const EVENTS: [(usize, i32, usize, bool); 31] = [
    (0, 12, 0, true),  // 灵草满坡
    (1, 11, 0, false), // 妖兽拦路
    (2, 6, 1, true),   // 上古洞府
    (3, 8, 0, true),   // 坊市淘宝
    (4, 8, 1, false),  // 恶修劫道
    (5, 8, 0, true),   // 道友论道
    (6, 5, 1, true),   // 宗门相邀
    (7, 6, 2, false),  // 心魔袭扰
    (8, 5, 2, true),   // 天材地宝
    (9, 3, 2, true),   // 灵脉洞府
    (10, 2, 3, true),  // 上古传承
    (11, 6, 2, true),  // 秘境探秘
    (12, 6, 0, true),  // 红尘历练
    (13, 5, 0, true),  // 灵气潮汐
    (14, 6, 2, false), // 邪修偷袭
    (15, 1, 4, true),  // 仙人遗府
    (16, 4, 2, true),  // 妖丹入手
    (17, 5, 1, true),  // 云游商人
    (18, 5, 1, true),  // 前辈指点
    (19, 3, 2, true),  // 灵兽认主
    (20, 4, 3, true),  // 仙人问道（三选一）
    (21, 3, 2, true),  // 灵泉洞天
    (22, 4, 2, true),  // 古剑冢
    (23, 4, 3, false), // 魔窟探宝
    (24, 3, 2, true),  // 雷击木
    (25, 5, 1, true),  // 商队护送
    (26, 2, 4, true),  // 仙人赠礼
    (27, 4, 1, true),  // 天机阁卜卦
    (28, 5, 1, true),  // 同页天成（同页人）
    (29, 4, 2, true),  // 宗门大比
    (30, 5, 2, true),  // 空白行（天曹没写的一年，每稿至多一次）
];

// ---------------- 功法价格（藏经阁，现实可购） ----------------
pub const TECH_PRICES: [i64; 8] = [0, 2000, 8000, 30000, 120000, 500000, 2000000, 10000000];

// ---------------- 称号 ----------------
pub fn realm_title(max_realm: usize) -> &'static str {
    match max_realm {
        0 => "未开笔",
        1 => "初沁墨",
        2 => "一行自立",
        3 => "章成洇然",
        4 => "骨立不倒",
        5 => "笔下生花",
        6 => "锋藏不露",
        7 => "铁画银钩",
        8 => "书传人间",
        9 => "对账之人",
        10 => "墨行纸背",
        11 => "一砚之尊",
        12 => "秉笔者",
        13 => "执卷者",
        _ => "琅函天笔",
    }
}

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

// ---------------- 同页人十名（原创定稿 §11） ----------------
pub const COMPANION_NAMES: [&str; 10] = [
    "闻人未晚",
    "洛书遥",
    "桑落",
    "阮栖梧",
    "姜稚鱼",
    "谢迟",
    "温衔雪",
    "柏舟",
    "孟拾遗",
    "应梨白",
];

// v1.12.1 同页人·三生缝页：缝页人未定标记（feng_bn 存此值=未结三生之约）；缘一双修再乘系数
pub const FENG_UNSET: u8 = 255;
pub const FENG_CULT_MULT: f64 = 1.10;

// ================= v1.4.0 山河图志 =================

// 世界地图节点定义（画布 1264×480，屏幕偏移 (8,60)）
pub struct NodeDef {
    pub name: &'static str,
    pub x: f32,
    pub y: f32,
    pub desc: &'static str,
    pub acts: &'static [&'static str],
}

pub const NODE_SECT: usize = 0; // 青云宗
pub const NODE_MARKET: usize = 1; // 临仙坊市
pub const NODE_RUINS: usize = 2; // 落霞秘境
pub const NODE_DONGFU: usize = 3; // 清源洞府
pub const NODE_BONES: usize = 4; // 荒骨战场
pub const NODE_PEAK: usize = 5; // 天雷峰
pub const NODE_START: usize = NODE_DONGFU;

pub const LOCATIONS: [NodeDef; 6] = [
    NodeDef {
        name: "青云宗",
        x: 200.0,
        y: 130.0,
        desc: "南疆第一仙宗，山门巍峨，剑气冲霄。拜入宗门可得名师指点，修炼事半功倍；完成宗门任务更可积攒贡献，换取宗门珍藏。",
        acts: &["拜入青云", "宗门任务·1年", "宗门商店"],
    },
    NodeDef {
        name: "临仙坊市",
        x: 640.0,
        y: 230.0,
        desc: "凡仙杂处的水陆大埠，三教九流云集。坊市丹药对本地修士九折优惠，南来北往的消息也最为灵通。",
        acts: &["打开坊市"],
    },
    NodeDef {
        name: "落霞秘境",
        x: 980.0,
        y: 110.0,
        desc: "上古修士遗府，每逢霞光贯日便开启一线。秘境中机缘与凶险并存——深入远征，逐层探取宝藏，见好就收方为上策。",
        acts: &["开始远征·1年"],
    },
    NodeDef {
        name: "清源洞府",
        x: 430.0,
        y: 90.0,
        desc: "一处灵气充沛的天然洞天，泉鸣如琴，石髓生烟。于此闭关修炼，每载修为+25%；亦可经营洞府——聚灵阵、灵田、丹房随四时气象日益兴旺。",
        acts: &["闭关修炼·1年", "经营洞府"],
    },
    NodeDef {
        name: "荒骨战场",
        x: 1120.0,
        y: 330.0,
        desc: "上古大战的古场，白骨露野，煞气不散。血仇之敌常于此设伏等你——是了断恩怨之地。",
        acts: &["寻仇"],
    },
    NodeDef {
        name: "天雷峰",
        x: 780.0,
        y: 390.0,
        desc: "直插云霄的孤峰，峰顶终年雷云不散。峰顶问笔台引雷入体淬炼元神——冲击大境界时，每重雷劫存活+2%。",
        acts: &["冲击境界"],
    },
];

// 纯视觉连线；赶路支持任意两地直达
pub const MAP_PATHS: [(usize, usize); 6] = [(0, 3), (0, 1), (1, 2), (1, 4), (4, 5), (2, 5)];

// 宗门经济：quest_contrib(r)=20+8r → 沁墨28 / 洇章44 / 问笔92 / 天笔132
pub fn quest_contrib(realm: usize) -> u32 {
    20 + 8 * realm as u32
}

pub fn quest_stones(realm: usize) -> i64 {
    (300.0 * (1.0 + realm as f64).powf(1.6)) as i64
}

pub enum SectGoodKind {
    Qi15,
    Dao2,
    Body2,
    Life8,
    Relic(usize),
    Tech(usize),
}

pub struct SectGood {
    pub name: &'static str,
    pub desc: &'static str,
    pub cost: u32,
    pub kind: SectGoodKind,
}

pub const SECT_GOODS: [SectGood; 6] = [
    SectGood {
        name: "凝元丹",
        desc: "宗门秘制，服之修为+当前层15%",
        cost: 60,
        kind: SectGoodKind::Qi15,
    },
    SectGood {
        name: "洗心丹",
        desc: "涤荡心尘，道心永久+2",
        cost: 90,
        kind: SectGoodKind::Dao2,
    },
    SectGood {
        name: "铁骨丹",
        desc: "淬炼筋骨，体魄永久+2",
        cost: 90,
        kind: SectGoodKind::Body2,
    },
    SectGood {
        name: "延寿丹",
        desc: "以灵物炼制，寿元+8载",
        cost: 140,
        kind: SectGoodKind::Life8,
    },
    SectGood {
        name: "闲章",
        desc: "镇派之宝拓影，战力+60%（需立骨境）",
        cost: 260,
        kind: SectGoodKind::Relic(5),
    },
    SectGood {
        name: "天笔遗墨",
        desc: "镇派无上功法（需立骨境）",
        cost: 420,
        kind: SectGoodKind::Tech(7),
    },
];

// 远征步骤：kind 0战 1药 2宝 3奇
pub struct ExpedStep {
    pub text: &'static str,
    pub kind: u8,
}

pub const EXPED_STEPS: [ExpedStep; 8] = [
    ExpedStep {
        text: "一群妖狼拦住去路，绿瞳幽幽，獠牙滴涎。",
        kind: 0,
    },
    ExpedStep {
        text: "几名邪修残党盯上了你，杀意毕露。",
        kind: 0,
    },
    ExpedStep {
        text: "谷中灵药遍地，异香扑鼻，时有兽吼隐隐。",
        kind: 1,
    },
    ExpedStep {
        text: "一座上古药园废墟，灵株在禁制中自生自灭。",
        kind: 1,
    },
    ExpedStep {
        text: "上古宝库大门半启，禁制流光如网。",
        kind: 2,
    },
    ExpedStep {
        text: "湖底沉船遗藏隐约可见，水寒刺骨。",
        kind: 2,
    },
    ExpedStep {
        text: "一位前辈坐化之地，道韵未散，石刻斑驳。",
        kind: 3,
    },
    ExpedStep {
        text: "一眼灵泉汩汩涌动，泉水中蕴含精纯灵气。",
        kind: 3,
    },
];

pub const EXPED_CHOICE_LABELS: [[&str; 2]; 4] = [
    ["正面迎击", "绕道潜行"],
    ["细致采撷", "速取而返"],
    ["强破禁制", "小心搜检"],
    ["静心感悟", "略作参详"],
];

pub const EXPED_LEN_BASE: usize = 3; // 链长 = 3 + gen_range(0, 3) → 3..=5
pub const EXPED_QI_PCT: f64 = 0.10;
pub const EXPED_HERB_QI: f64 = 0.04;
pub const EXPED_STONE: f64 = 700.0;
pub const EXPED_STONE_BONE: f64 = 1400.0;
pub const EXPED_CONTRIB: u32 = 15;
pub const MARKET_PILL_DISC: f64 = 0.9;
pub const DONGFU_CULT_MULT: f64 = 1.25;
pub const PEAK_TRIB_BONUS: f32 = 0.02; // 生效前提：现实冲墨仙走九重雷劫分支

// ================= v1.5.0 四时洞天 =================

// 时段：每推岁一载推进一格（晨→昼→昏→夜）
pub const SLOTS: [&str; 4] = ["清晨", "白昼", "黄昏", "夜晚"];

// 时段叠色（RGBA，地图整幅叠加；a=0 跳过不画）
pub const SLOT_TINT: [(u8, u8, u8, u8); 4] = [
    (255, 214, 150, 26), // 清晨：暖金薄纱
    (255, 255, 255, 0),  // 白昼：无叠色
    (255, 140, 80, 36),  // 黄昏：橘红
    (24, 34, 92, 68),    // 夜晚：深蓝
];

// 天气：每推岁一载按转移矩阵演化
pub const WEATHERS: [&str; 5] = ["晴", "云", "雨", "雾", "灵潮"];

// 天气叠色（同上）
pub const WEATHER_TINT: [(u8, u8, u8, u8); 5] = [
    (255, 255, 255, 0),  // 晴
    (200, 200, 215, 18), // 云
    (80, 120, 200, 34),  // 雨
    (190, 190, 205, 40), // 雾
    (140, 90, 220, 30),  // 灵潮
];

// 天气转移矩阵：WEATHER_TRANS[当前][下一] 概率（每行和 = 1）
pub const WEATHER_TRANS: [[f32; 5]; 5] = [
    [0.50, 0.30, 0.10, 0.07, 0.03], // 晴
    [0.35, 0.30, 0.18, 0.12, 0.05], // 云
    [0.30, 0.25, 0.30, 0.10, 0.05], // 雨
    [0.30, 0.25, 0.15, 0.25, 0.05], // 雾
    [0.45, 0.25, 0.15, 0.10, 0.05], // 灵潮
];

// 天气 × 地点 收益倍率：宗门任务/远征战获/修炼等按所在地取值
// 列序：0青云宗 1临仙坊市 2落霞秘境 3清源洞府 4荒骨战场 5天雷峰
pub const WEATHER_LOC_MULT: [[f64; 6]; 5] = [
    [1.00, 1.00, 1.00, 1.00, 1.00, 1.00], // 晴
    [1.05, 1.00, 1.00, 1.05, 1.00, 1.00], // 云
    [1.10, 1.00, 1.15, 1.10, 0.95, 0.95], // 雨：草木得润，秘境药草丰
    [1.00, 0.95, 1.20, 1.15, 1.05, 0.95], // 雾：秘境机缘隐现，洞府灵气凝聚
    [1.30, 1.10, 1.30, 1.50, 1.10, 1.20], // 灵潮：天地灵机暴涨
];

pub fn weather_loc_mult(w: usize, loc: usize) -> f64 {
    WEATHER_LOC_MULT[w.min(WEATHER_LOC_MULT.len() - 1)][loc.min(5)]
}

// 立绘分档：0白丁 1道基 2紫府 3墨仙（按最高境界取档）
pub const PORTRAIT_TIERS: [usize; 4] = [0, 3, 6, 10]; // 各档境界下限

pub fn portrait_tier(realm: usize) -> usize {
    let mut t = 0;
    for (i, &th) in PORTRAIT_TIERS.iter().enumerate() {
        if realm >= th {
            t = i;
        }
    }
    t
}

// 法宝装备（至多 3 件；未装备法宝仍发挥三成灵性）
pub const RELIC_EQUIP_MAX: usize = 3;
pub const RELIC_UNEQUIP_FRAC: f64 = 0.3;

// ================= v1.8.0 E 百炼乾坤：法宝炼化 =================
// 献祭一件法宝为材料，为目标法宝提升灵纹等级（0..=3）。
pub const REFINE_MAX: i32 = 3;
pub const REFINE_STEP: f64 = 0.08;
// 炼化后战力加成：pow × (1 + 0.08×lv)
pub fn relic_pow_at(idx: usize, lv: i32) -> f64 {
    RELICS[idx].pow * (1.0 + REFINE_STEP * lv.clamp(0, REFINE_MAX) as f64)
}
// 炼化后修炼灵性：1 + (cult-1) × (1 + 0.08×lv)——cult=1.0 的纯攻法宝不虚增修炼
pub fn relic_cult_at(idx: usize, lv: i32) -> f64 {
    1.0 + (RELICS[idx].cult - 1.0) * (1.0 + REFINE_STEP * lv.clamp(0, REFINE_MAX) as f64)
}

// 洞府经营：聚灵阵 / 灵田 / 丹房，各 0..=3 级
pub const DONGFU_FACS: [&str; 3] = ["聚灵阵", "灵田", "丹房"];
pub const DONGFU_FAC_DESC: [&str; 3] = [
    "阵纹环洞，灵气奔涌——修炼效率随阵法层级提升。",
    "洞前开垦灵田，每载自动产出灵石，收成随天气起伏。",
    "洞中丹炉不熄，坊市购丹价格随炉火层级降低。",
];
pub const JUELING_MULT: [f64; 4] = [1.00, 1.15, 1.30, 1.50];
pub const LINGTIAN_YIELD: [i64; 4] = [0, 300, 1200, 5000];
pub const DANFANG_DISC: [f64; 4] = [1.00, 0.90, 0.80, 0.70];
// 升到 L+1 级费用：灵石 DONGFU_UP_STONES[L] + 贡献 DONGFU_UP_CONTRIB[L]
pub const DONGFU_UP_STONES: [i64; 3] = [2000, 20000, 200000];
pub const DONGFU_UP_CONTRIB: [u32; 3] = [30, 120, 400];

// ================= v1.6.0 山河万象：仇敌突破换地图 =================

// 地图种子默认值（旧档缺 mseed 键时的迁移缺省）
pub const MAP_SEED_DEFAULT: u32 = 20261011;

// 五区主题表：主题 0 与 v1.5.0 现版逐值一致（旧档/未换幕前像素级等价）
pub struct MapTheme {
    pub name: &'static str,
    pub desc: &'static str,
    pub grad_top: (u8, u8, u8),  // 纵向渐变顶色
    pub grad_bot: (u8, u8, u8),  // 纵向渐变底色
    pub moss_a: (u8, u8, u8),    // 苔斑主色基准（实际值 ±随机抖动）
    pub moss_b: (u8, u8, u8),    // 苔斑次色基准
    pub ridge_a: (u8, u8, u8),   // 山脊带 1
    pub ridge_b: (u8, u8, u8),   // 山脊带 2
    pub river: (u8, u8, u8),     // 河流主色
    pub river_hl: (u8, u8, u8),  // 河流上缘高光
    pub forest_a: (u8, u8, u8),  // 林木主色
    pub forest_b: (u8, u8, u8),  // 林木次色
    pub frame: (u8, u8, u8),     // 暗金双线边框
    pub deco: f32,               // 苔斑/林木密度倍率
    pub jitter: [(f32, f32); 6], // 六节点位置抖动（主题 0 全 0 = 现版坐标）
}

pub const MAP_THEMES: [MapTheme; 5] = [
    // 0 东境（现版逐值：渐变 (24,34,40)→(14,20,26)、苔斑 (30,52,38)/(56,46,36) 600、
    //   山脊 (46,44,58)/(38,36,48)、河 (58,96,110)/(86,130,140)、林 (30,48,36)/(26,42,32) 220、
    //   边框 (122,96,52)、deco 1.0、jitter 全 0）
    MapTheme {
        name: "东境青云",
        desc: "灵气氤氲的青云故地，山河如初。",
        grad_top: (24, 34, 40),
        grad_bot: (14, 20, 26),
        moss_a: (30, 52, 38),
        moss_b: (56, 46, 36),
        ridge_a: (46, 44, 58),
        ridge_b: (38, 36, 48),
        river: (58, 96, 110),
        river_hl: (86, 130, 140),
        forest_a: (30, 48, 36),
        forest_b: (26, 42, 32),
        frame: (122, 96, 52),
        deco: 1.0,
        jitter: [(0.0, 0.0); 6],
    },
    MapTheme {
        name: "西漠黄沙",
        desc: "大漠孤烟，残卷称此地埋着上古魔宗的埋骨窟。",
        grad_top: (50, 40, 26),
        grad_bot: (30, 22, 14),
        moss_a: (70, 58, 36),
        moss_b: (88, 72, 46),
        ridge_a: (100, 80, 54),
        ridge_b: (80, 64, 44),
        river: (64, 110, 116),
        river_hl: (98, 150, 150),
        forest_a: (76, 66, 40),
        forest_b: (64, 56, 34),
        frame: (152, 112, 60),
        deco: 0.55,
        jitter: [(30.0, -20.0), (-36.0, 14.0), (20.0, 28.0), (-24.0, 26.0), (18.0, 22.0), (-28.0, 14.0)],
    },
    MapTheme {
        name: "南疆瘴林",
        desc: "瘴气弥漫的十万大山，蛊修与灵药同生于此。",
        grad_top: (18, 40, 26),
        grad_bot: (10, 24, 16),
        moss_a: (36, 66, 42),
        moss_b: (26, 54, 36),
        ridge_a: (42, 64, 50),
        ridge_b: (34, 54, 42),
        river: (46, 110, 84),
        river_hl: (78, 150, 112),
        forest_a: (18, 54, 32),
        forest_b: (14, 46, 26),
        frame: (98, 142, 74),
        deco: 1.6,
        jitter: [(-26.0, 20.0), (28.0, -18.0), (-18.0, -24.0), (22.0, -26.0), (-16.0, 18.0), (24.0, 20.0)],
    },
    MapTheme {
        name: "北境冰原",
        desc: "万里雪封的极北之地，冰河之下镇着未醒的古修。",
        grad_top: (42, 48, 60),
        grad_bot: (22, 26, 38),
        moss_a: (118, 126, 140),
        moss_b: (94, 102, 120),
        ridge_a: (88, 96, 116),
        ridge_b: (70, 78, 98),
        river: (96, 140, 170),
        river_hl: (150, 190, 215),
        forest_a: (60, 82, 86),
        forest_b: (50, 70, 76),
        frame: (150, 172, 202),
        deco: 0.8,
        jitter: [(20.0, 26.0), (-22.0, 24.0), (26.0, -20.0), (-30.0, -16.0), (20.0, -24.0), (-18.0, 26.0)],
    },
    MapTheme {
        name: "东海仙屿",
        desc: "沧溟之上的散修岛屿，潮声里有仙宫的传闻。",
        grad_top: (16, 42, 58),
        grad_bot: (8, 22, 36),
        moss_a: (42, 72, 82),
        moss_b: (30, 58, 70),
        ridge_a: (54, 86, 98),
        ridge_b: (42, 70, 82),
        river: (60, 140, 160),
        river_hl: (102, 180, 195),
        forest_a: (36, 82, 72),
        forest_b: (28, 70, 62),
        frame: (110, 162, 190),
        deco: 1.2,
        jitter: [(-24.0, -16.0), (30.0, 20.0), (-20.0, 24.0), (26.0, -22.0), (-22.0, -20.0), (18.0, 18.0)],
    },
];

pub fn theme_of(region: u32) -> &'static MapTheme {
    &MAP_THEMES[(region as usize) % MAP_THEMES.len()]
}

// ================= v1.8.0 百炼乾坤：区域专属机制 =================
// 五域各一机制：顶栏短标 + 一行说明；宗门任务/远征/现实闭关/模拟善缘四处挂钩。
pub struct RegionMech {
    pub mech: &'static str, // 机制短名（顶栏标签）
    pub desc: &'static str, // 一行说明（≤18 字，图鉴/卡片）
    pub quest_mult: f64,    // 宗门任务贡献与灵石
    pub exped_mult: f64,    // 远征收益
    pub exped_risk: f32,    // 远征成功率减益（正=更险）
    pub cult_mult: f64,     // 现实闭关效率
    pub event_mult: f64,    // 模拟善缘事件权重
}
pub const REGION_MECH: [RegionMech; 5] = [
    // 0 东境青云：宗门兴望
    RegionMech {
        mech: "宗门兴望",
        desc: "宗门兴旺，任务报偿更丰",
        quest_mult: 1.20,
        exped_mult: 1.00,
        exped_risk: 0.0,
        cult_mult: 1.00,
        event_mult: 1.00,
    },
    // 1 西漠黄沙：大漠秘藏
    RegionMech {
        mech: "大漠秘藏",
        desc: "大漠埋藏上古秘藏，远征收获更丰",
        quest_mult: 1.00,
        exped_mult: 1.25,
        exped_risk: 0.0,
        cult_mult: 1.00,
        event_mult: 1.00,
    },
    // 2 南疆瘴林：瘴惑灵芽
    RegionMech {
        mech: "瘴惑灵芽",
        desc: "瘴气惑人，远征更险而灵药更丰",
        quest_mult: 1.00,
        exped_mult: 1.20,
        exped_risk: 0.08,
        cult_mult: 1.00,
        event_mult: 1.00,
    },
    // 3 北境冰原：冰心澄神
    RegionMech {
        mech: "冰心澄神",
        desc: "冰原澄神定魄，现实闭关效率更高",
        quest_mult: 1.00,
        exped_mult: 1.00,
        exped_risk: 0.0,
        cult_mult: 1.15,
        event_mult: 1.00,
    },
    // 4 东海仙屿：仙屿潮音
    RegionMech {
        mech: "仙屿潮音",
        desc: "潮音涤心，镜像世界善缘奇遇更多",
        quest_mult: 1.00,
        exped_mult: 1.00,
        exped_risk: 0.0,
        cult_mult: 1.00,
        event_mult: 1.20,
    },
];
pub fn region_mech(region: u32) -> &'static RegionMech {
    &REGION_MECH[(region as usize) % REGION_MECH.len()]
}
// 初至奖励表（道韵, 灵石）——visit_region 按 (map_region % 5) 索引
pub const REGION_VISIT_REWARD: [(i64, i64); 5] = [
    (30, 2000),
    (60, 5000),
    (100, 12000),
    (160, 30000),
    (260, 80000),
];

// 节点屏幕系坐标（画布 1264×480）：LOCATIONS 基准 + 主题抖动，clamp 画布安全边
// （地图绘制/点击命中/林木规避三处共用，坐标单一来源）
pub fn node_xy(i: usize, region: u32) -> (f32, f32) {
    let j = theme_of(region).jitter[i.min(5)];
    let nd = &LOCATIONS[i.min(LOCATIONS.len() - 1)];
    (
        (nd.x + j.0).clamp(60.0, 1204.0),
        (nd.y + j.1).clamp(60.0, 420.0),
    )
}

// ================= v1.6.0 D 好感度 NPC 关系系统 =================

// NPC 名与卡面描述（v1.8.0 D1 扩四人：0长老 1商人 2宁清霜 3醉道人；
// 与 npc.rs NPC_ELDER/NPC_MERCHANT/NPC_SENIOR/NPC_DRUNK 对应）
pub const NPC_NAMES: [&str; 4] = ["玄诚长老", "胡掌柜", "宁清霜", "醉道人"];
// 卡面长描述（人物页 v1.8.0 D1 版面改用一行 NPC_BRIEF；此表保留为图鉴/未来扩展数据）
#[allow(dead_code)]
pub const NPC_DESC: [&str; 4] = [
    "青云宗执事长老，掌外门杂役与任务分派。交情深厚者，所托任务常有厚报。",
    "临仙坊市万宝楼的胡姓掌柜，眼光毒辣。熟客购丹，向来给足面子。",
    "青云宗大师姐，剑心通明。交情深厚者，破境之际愿以剑意相护。",
    "来历不明的散修酒友，醉话里常藏真机。酒逢知己，修炼事半功倍。",
];
// 人物页卡面一行简介（单行 ≤26 字；v1.8.0 D1 新增）
pub const NPC_BRIEF: [&str; 4] = [
    "宗门任务的靠山，好感愈深报偿愈丰。",
    "万宝楼掌柜，购丹价格随交情打折。",
    "大师姐剑意护持，跨境破境更稳。",
    "酒友真言入耳，每载修为水涨船高。",
];

// 好感档次阈值与名称（fav ≥ 阈值即入档）
pub const NPC_FAV_TIERS: [i32; 5] = [0, 20, 40, 70, 95];
pub const NPC_TIER_NAMES: [&str; 5] = [
    "素不相识",
    "点头之交",
    "相谈甚欢",
    "莫逆之交",
    "生死之交",
];

// 各档增益曲线（下标 = tier_of(fav)）
pub const ELDER_QUEST_MULT: [f64; 5] = [1.00, 1.05, 1.10, 1.18, 1.25]; // 宗门任务收益
pub const MERCHANT_DISC: [f64; 5] = [1.00, 0.97, 0.94, 0.90, 0.85]; // 坊市丹价乘数
pub const COMPANION_CULT_MULT: [f64; 5] = [1.00, 1.20, 1.25, 1.30, 1.35]; // 道侣双修倍率
// v1.8.0 D1 新 NPC 增益曲线
pub const SENIOR_BREAK_ADD: [f32; 5] = [0.00, 0.01, 0.02, 0.03, 0.05]; // 大师姐：现实跨境破境加成
pub const DRUNK_CULT_MULT: [f64; 5] = [1.00, 1.03, 1.06, 1.10, 1.15]; // 酒友：每载修炼效率

// v1.8.0 D2：好感专属事件触发阈值（第三档「相谈甚欢」）
pub const NPC_EVENT_FAV: i32 = 70;

pub const NPC_FAV_MAX: i32 = 100;
pub const NPC_GIFT_BASE: i64 = 500; // 赠礼基础价（×(1+档位)）
pub const NPC_GIFT_FAV: i32 = 2; // 赠礼好感
pub const ELDER_QUEST_FAV: i32 = 3; // 宗门任务好感
pub const MERCHANT_BUY_FAV: i32 = 1; // 坊市购丹好感
pub const COMPANION_BOND_FAV: i32 = 25; // 结缘起步好感
pub const COMPANION_GIFT_FAV: i32 = 5; // 道侣赠礼事件好感
pub const INHERIT_LEGACY_FAV: i32 = 10; // 遗泽承接好感加成
