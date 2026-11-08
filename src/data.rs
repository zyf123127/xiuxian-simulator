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

// ---------------- v1.16.8 章节书：主线八章（v1.16.10 增第九章·书外之章）----------------
// 章节定义静态表（照 ACHS/EVENTS 惯例）：名/引入剧情/目标清单。达成谓词不在本表——
// 全部收敛为 game.rs chapter_goal_met 单口（pid 分发，照 check_ach_all 的 (id,bool)
// 元组表先例），禁第二实现。所有谓词只读 meta 标量（不含 hist/story 等 String 字段），
// chapter_done_mask 可在存档载入帧与每帧章节视图安全调用，零分配。
// 达成持久 = meta.chap_done 位掩码（键 ch=，缺键→0 零迁移，save.rs 回环单测）。
pub struct ChapterGoal {
    pub text: &'static str, // 目标文案（章节书目标行）
    pub pid: u8,            // 谓词 id（game.rs chapter_goal_met 单口 match 分发表）
}

pub struct ChapterDef {
    pub num: &'static str,   // 章次字面量（"第一章"，禁运行时拼数字）
    pub name: &'static str,  // 章名（横幅/章节书行）
    pub intro: &'static str, // 引入剧情（章节书右栏，wrap_draw 单段）
    pub goals: &'static [ChapterGoal],
    // v1.16.9 章成之礼：达成边沿由 game.rs check_chapters 唯一发放（newly 位命中处
    // meta.dao_yun += reward，chap_done 位即领取凭证，零新存档键零迁移）。奖励行
    // 显示走 reward_note 全静态字面量（含「道韵 +N」），帧路径零分配；reward 数字
    // 与 note 文本的对应由 chapter_tests 表一致性单测钉死，防两处手改漂移。
    pub reward: u32,          // 章成之礼道韵（meta 跨世持久标量；不发灵石等现世资源）
    pub reward_note: &'static str, // 奖励行文案（章节书右栏 + 达成金行 log 复用）
    // v1.16.9 章间记事：逐段解锁的章内插曲（段 i 的可见条件=本章已毕目标数 ≥ i，
    // 右栏只显示最新解锁段——纯显示推导零存档；段首自带「其一/其二」序号，全静态）。
    pub lore: &'static [&'static str],
}

pub const CHAPTERS: [ChapterDef; 9] = [
    ChapterDef {
        num: "第一章",
        name: "雨夜执印",
        intro: "雨夜乱葬岗，你从尸堆里爬出来，手里攥着一枚方印——印身无字，只有层层叠叠被划掉的痕迹。识海里有个累坏了的少年嗓音：你的名字是错的，天写错了。——先推演一生，看看错在哪里。",
        goals: &[
            ChapterGoal { text: "转生入世——此身既立，主线自此始", pid: 12 },
            ChapterGoal { text: "开卷首模拟——耗 1 枚点数推演一生", pid: 0 },
        ],
        reward: 50,
        reward_note: "章成之礼：道韵 +50",
        lore: &[
            "其一：方印无字，划痕却比字多。识海里那声叹息说——这不是印，是账；前头十万稿，都记在它上头。",
            "其二：你试着把印按进土里，土里渗出金纹，拼出半句话：先推演。推演不误砍柴工。",
        ],
    },
    ChapterDef {
        num: "第二章",
        name: "镜中万象",
        intro: "轮回镜里万象森罗：游历遇敌、闭关破境、奇遇结缘。模拟中的一战一胜，都是现实这具身体的磨刀石。",
        goals: &[
            ChapterGoal { text: "首胜——模拟中战退一次强敌", pid: 1 },
            ChapterGoal { text: "现实首破境——闭关圆满，冲击新境", pid: 2 },
        ],
        reward: 80,
        reward_note: "章成之礼：道韵 +80",
        lore: &[
            "其一：轮回镜不照脸，照的是没走的路。镜里那人赢了，你输了；镜里那人死了，你还站着。",
            "其二：镜背刻着一行小字：万象皆稿。你忽然明白，模拟不是梦——是起草。",
        ],
    },
    ChapterDef {
        num: "第三章",
        name: "血仇得报",
        intro: "删你字的血仇之敌仍按册候查。变强，寻仇，阵斩之——夺其家产，据其职牒。此吏不除，书不成书。",
        goals: &[
            ChapterGoal { text: "首斩宿敌——现实中寻仇得胜", pid: 3 },
            ChapterGoal { text: "再斩一敌——血仇之链，一环扣一环", pid: 14 },
        ],
        reward: 120,
        reward_note: "章成之礼：道韵 +120",
        lore: &[
            "其一：血仇之敌死前盯着你的方印，笑出声：「你也是被删了名字的？」你没答——收刀，取牒。",
            "其二：职牒入手温热，像墨迹未干的纸。牒上官名被划掉，补了一行小字：暂缺，待补。",
        ],
    },
    ChapterDef {
        num: "第四章",
        name: "文狱讨牒",
        intro: "每斩一吏，其职牒便归你名下。六牒齐聚之日，文狱闻你名而却步——问笔终战的门票，一张一张攒出来。",
        goals: &[
            ChapterGoal { text: "讨牒三成——职牒 3/6 归档", pid: 4 },
            ChapterGoal { text: "六牒归档——讨牒 6/6，道韵+166", pid: 5 },
        ],
        reward: 166,
        reward_note: "章成之礼：道韵 +166",
        lore: &[
            "其一：文狱里文书成精，专吃写错的名字。讨牒六张，张张带血——不是你的血，是名字的血。",
            "其二：六牒齐了那夜，方印烫了一瞬。印身划痕浅了一层：账，还了一笔。",
        ],
    },
    ChapterDef {
        num: "第五章",
        name: "山河万象",
        intro: "山河图志五大区域，各有风土与机缘：宗门兴望、大漠秘藏、瘴惑灵芽、冰心澄神、仙屿潮音。行走、换幕、初至有赏——把五个域的名字都走一遍。",
        goals: &[
            ChapterGoal { text: "三域已至——山河图志初成", pid: 6 },
            ChapterGoal { text: "五域全至——图志圆满", pid: 7 },
        ],
        reward: 200,
        reward_note: "章成之礼：道韵 +200",
        lore: &[
            "其一：山河图志摊开，五域风土扑面。你走过的地方，图志上便多一笔墨——像有人在替你续稿。",
            "其二：仙屿潮音夜里响，声里混着纸页翻动。图志合上时，你听见一句：第五章，过。",
        ],
    },
    ChapterDef {
        num: "第六章",
        name: "同页之缘",
        intro: "模拟世中有人愿与你缝作同页；现实里有长老、掌柜、宁清霜、醉道人四位故人。缘是线，把散稿订成书。",
        goals: &[
            ChapterGoal { text: "首结缘——与一人缝作同页", pid: 8 },
            ChapterGoal { text: "故人心事——好感达标，抉择一场专属事件", pid: 9 },
        ],
        reward: 240,
        reward_note: "章成之礼：道韵 +240",
        lore: &[
            "其一：缝作同页的人说：稿子写坏了不要撕，缝进去——废稿也是稿，垫在底下，书才厚。",
            "其二：故人们各有心事。心事不重，一张纸的分量；可一册书，就是一张纸一张纸摞出来的。",
        ],
    },
    ChapterDef {
        num: "第七章",
        name: "问笔终战",
        intro: "六牒在手，修为圆满——现实破境的雷劫里，执笔人在等你。九重雷劫尽数扛下，便是落笔之刻。",
        goals: &[
            ChapterGoal { text: "现实问笔——九重雷劫扛下，首登仙界", pid: 10 },
            ChapterGoal { text: "封卷已斩——讨牒 6/6 方见终战卡", pid: 5 },
        ],
        reward: 320,
        reward_note: "章成之礼：道韵 +320",
        lore: &[
            "其一：九重雷劫，一重一问。问到第七重时，天问累了——你没累。",
            "其二：天门开处没有仙宫，只有一张摆好纸笔的案。执笔人不在，笔在等你。",
        ],
    },
    ChapterDef {
        num: "第八章",
        name: "落款成书",
        intro: "终战卡上四种落款：认写、撕稿、夺笔、合订。你如何落款，书便如何成——十万稿废纸，等你一个结尾。",
        goals: &[
            ChapterGoal { text: "落款成书——达成任一结局", pid: 11 },
            ChapterGoal { text: "四卷全览——四种结局皆已落款", pid: 13 },
        ],
        reward: 400,
        reward_note: "章成之礼：道韵 +400",
        lore: &[
            "其一：执笔人递笔：你如何落款，书便如何成。十万稿废纸在身后哗哗作响，像十万人在等你。",
            "其二：落款那一刻，方印划痕尽褪，字浮出来——是你的名字。这一次，天没有写错。",
            "其三：书成之后还有书。合订的人说：下一册，从别人的名字写起。",
        ],
    },
    // v1.16.10 书外之章：四结局全览后的首个「书外」进行态目标——终局不再是悬空。
    // pid13（四卷全览）为第八章既有谓词复用（章九开局即已毕其一），新谓词 pid15
    //（历代最高修为抵执卷境）进 chapter_goal_met 单口 match；ch= 位掩码 u32 位 8，
    // 旧档载入帧静默补齐口径不变（老玩家补齐不补发，与章成之礼既定取舍一致）。
    ChapterDef {
        num: "第九章",
        name: "书外之章",
        intro: "书成了，故事却没有。合订的人说：下一册，从别人的名字写起——可你的名字，还要往上写。执卷之人，先把自己这一卷读到头。",
        goals: &[
            ChapterGoal { text: "四卷全览——四种结局皆已落款", pid: 13 },
            ChapterGoal { text: "执卷境——历代最高修为抵执卷，书外续行", pid: 15 },
        ],
        reward: 500,
        reward_note: "章成之礼：道韵 +500",
        lore: &[
            "其一：书成之夜，方印上多了一行小字：本卷完，续卷无期。你笑了——无期，不是无。",
            "其二：合订的人临行回头：书外的路没有稿，走一步，是一步。你提印出门，天光正好。",
        ],
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
pub const EVENTS: [(usize, i32, usize, bool); 134] = [
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
    // v1.16.5 扩池 13 条：低境界（mr=0）从 6 条扩到 11 条，缓「开局连见同卡」
    (31, 6, 0, true),  // 雨夜借宿
    (32, 6, 0, true),  // 古碑残字
    (33, 5, 0, false), // 雾锁迷林
    (34, 6, 0, true),  // 墨市笔坊
    (35, 5, 0, true),  // 断桥济人
    (36, 5, 1, true),  // 瘴谷寻药
    (37, 6, 1, false), // 渡灵舟
    (38, 5, 1, true),  // 残卷拍卖
    (39, 5, 1, true),  // 灵田佃户
    (40, 5, 2, false), // 酒楼斗法
    (41, 5, 2, true),  // 前人手札
    (42, 4, 2, true),  // 废墟道场
    (43, 4, 2, false), // 沙暴藏窟
    // v1.16.7 扩池 15 条（id 44..=58）：连锁剧情 9（三链×3 段；45/46/48/49/51/52
    // 为链内段，权重 0 恒不入正常轮盘，仅经 chain 直取出现）、道侣同行 3
    // （53..=55，白名单门控：sim.companion 存在才入池，见 COMPANION_EVENTS）、
    // 稀有 3（56..=58，w≤2 低境界稀有层，稀有基权占比维持 ~4%）
    (44, 4, 0, true),  // 货郎奇遇（链1 首段：万宝楼外柜）
    (45, 0, 0, true),  // 罗盘相赠（链1 分支b 段）
    (46, 0, 0, false), // 恩怨上门（链1 分支a 段）
    (47, 3, 1, true),  // 古碑残篇（链2 首段）
    (48, 0, 0, true),  // 拓印风波（链2 中段）
    (49, 0, 0, false), // 守碑人（链2 末段）
    (50, 2, 0, true),  // 江底墨匣（链3 首段）
    (51, 0, 0, true),  // 渔村释疑（链3 中段）
    (52, 0, 0, true),  // 匣底真迹（链3 末段）
    (53, 4, 1, true),  // 道侣论道（伴随）
    (54, 3, 1, true),  // 赠剑同舟（伴随）
    (55, 3, 1, false), // 危难共护（伴随）
    (56, 2, 0, true),  // 星坠荒原（稀有）
    (57, 1, 1, false), // 无面镜客（稀有）
    (58, 2, 0, false), // 灵潮倒卷（稀有）
    // v1.16.8 扩池 26 条（id 59..=84）：普通 13（59..=71，题材避开既有 59 条——
    // 书院/灵蜂/画师/雪崩/赌擂/漕运/温泉/棋枰/铸炉/蜃楼/古井/药市/驿壁，基权
    // 合计 57）、稀有 2（72..=73，w≤2）、章节联动 2（74..=75，chap_done 位门控，
    // 见 sim_event_weights）、连锁剧情 6（76..=78 灯楼谜约 3 段 / 79..=81 傩面
    // 还愿 2 段分支式；77/78/80/81 为链内段权重 0）、道侣同行 3（82..=84，
    // COMPANION_EVENTS 白名单扩 6）
    (59, 5, 0, true),  // 书院讲经
    (60, 4, 1, true),  // 灵蜂酿蜜
    (61, 4, 1, true),  // 画师留影
    (62, 4, 1, false), // 雪崩沟谷
    (63, 5, 1, false), // 赌坊押擂
    (64, 4, 2, true),  // 漕运夜航
    (65, 5, 0, true),  // 温泉濯尘
    (66, 5, 2, true),  // 棋枰斗弈
    (67, 4, 2, true),  // 铸兵守炉
    (68, 3, 3, true),  // 蜃楼残景
    (69, 5, 0, true),  // 井底传声
    (70, 5, 1, true),  // 药市问诊
    (71, 4, 0, true),  // 驿壁题诗
    (72, 2, 2, true),  // 墨渊垂钓（稀有，special33 检定）
    (73, 1, 3, false), // 纸人夜行（稀有，special34 检定）
    (74, 3, 1, false), // 血仇余响（章节联动：第三章·血仇得报已了才入池）
    (75, 3, 1, true),  // 同页灯约（章节联动：第六章·同页之缘已了且有伴才入池）
    (76, 3, 1, true),  // 灯楼谜约（链4 首段）
    (77, 0, 0, true),  // 灯下问答（链4 中段，仅链内）
    (78, 0, 0, true),  // 青灯问藏（链4 末段，仅链内；special32 检定收尾）
    (79, 3, 0, true),  // 傩面还愿（链5 首段）
    (80, 0, 0, true),  // 戏毕面热（链5 分支a 段）
    (81, 0, 0, true),  // 愿成符赠（链5 分支b 段）
    (82, 4, 1, true),  // 灯下缝页（伴随）
    (83, 3, 1, true),  // 同摘霜橘（伴随）
    (84, 3, 2, false), // 论道起隙（伴随）
    // v1.16.9 扩池 15 条（id 85..=99）：宿敌剧情化 3（85..=87 血债血偿链6——85
    // 有宿敌才入池见 sim_event_weights、86/87 链内段权重 0）、老年/将尽专属 2
    // （88..=89，寿元余量 ≤ ELDER_LEFT_YEARS 才入池，池内原零覆盖）、高境界 2
    // （90 mr=4 / 91 mr=5，补高境界普通层缺口——原 mr≥4 仅 15/26 两条稀有）、
    // 常规 8（92..=99，题材避开既有：茶棚/染坊/傀儡/煎茶/悬棺/鱼龙/盐井/矿洞/
    // 梦笔，新基权合计 52）。扩池后总基权 331→384（v1.16.10 复核修正：原批注
    // 写 383 系笔误），稀有仍 9 条 15 基权 ≈3.9%（维持 ~4% 口径）；查重对账
    // 见本批次施工单。v1.16.10 再扩池见 id 100..=117 批注（账本数字一律以重算为准）
    (85, 4, 2, true),  // 仇踪入册（链6 首段：有宿敌才入池）
    (86, 0, 0, true),  // 仇踪暗访（链6 中段，仅链内）
    (87, 0, 0, true),  // 乘虚夺积（链6 末段，仅链内）
    (88, 4, 1, true),  // 灯下了愿（将尽专属）
    (89, 3, 0, true),  // 薪火相托（将尽专属）
    (90, 5, 4, true),  // 雷池遗蜕（高境）
    (91, 4, 5, true),  // 仙门论道（高境）
    (92, 5, 0, true),  // 染坊秘色
    (93, 4, 1, true),  // 傀儡戏班
    (94, 4, 1, true),  // 雪夜煎茶
    (95, 4, 2, true),  // 断崖悬棺
    (96, 4, 1, true),  // 春汛鱼龙
    (97, 4, 1, false), // 盐井卤灾
    (98, 4, 2, false), // 矿洞钟声
    (99, 4, 2, true),  // 梦笔生花
    // v1.16.10 扩池 18 条（id 100..=117）：常规 10（100..=109，mr 薄档优先——
    // 原 mr=3 仅 5 条/mr=4 仅 3 条/mr=5 仅 1 条，题材避开既有全部 100 条：焚字/
    // 笔冢/龙祠/纸鸢/砚山/点兵/蚕茧/空舟/墨鲸/借名，新基权合计 40，其中稀有 1
    // （108 墨鲸背城 w=2））、宿敌回响 2（110..=111，入池门控见 sim_event_weights：
    // 110 读会话态 chain6_closed、111 读 meta.neme_slain≥1）、连锁剧情 6（112..=114
    // 灯花渡舟 3 段 / 115..=117 药炉童子 3 段，链内段权重 0；置链臂 special 37..=40
    // 新窗见 CHAIN78_SPECIAL_LO/HI）。扩池后总基权与稀有占比以重算为准
    // （本批 +52：384→436；稀有 10 条 17 基权 ≈3.9%，维持 ~4% 口径）
    (100, 5, 3, true),  // 焚字炉
    (101, 4, 3, true),  // 笔冢祭
    (102, 4, 3, false), // 涸河龙祠
    (103, 5, 3, true),  // 纸鸢传书
    (104, 4, 4, true),  // 砚山叩泉
    (105, 4, 4, false), // 古戍点兵
    (106, 4, 5, true),  // 天蚕茧室
    (107, 4, 5, false), // 空舟拦江
    (108, 2, 4, true),  // 墨鲸背城（稀有）
    (109, 4, 2, false), // 借名修行
    (110, 3, 2, true),  // 夺积余波（宿敌同世回响：链6 本世已缘了才入池）
    (111, 3, 1, true),  // 仇册新墨（宿敌跨世回响：斩过宿敌才入池）
    (112, 3, 2, true),  // 灯花渡舟（链7 首段）
    (113, 0, 0, true),  // 灯下旧账（链7 中段，仅链内）
    (114, 0, 0, true),  // 一渡一灯（链7 末段，仅链内）
    (115, 3, 2, true),  // 药炉童子（链8 首段）
    (116, 0, 0, true),  // 炉语问心（链8 中段，仅链内）
    (117, 0, 0, true),  // 守炉一夜（链8 末段，仅链内）
    // v1.16.11 扩池 16 条（id 118..=133）：连锁剧情 6（118..=120 循册寻仇 3 段 /
    // 121..=123 旧页回声 3 段，链内段权重 0；置链臂 special 41..=44 第四窗见
    // CHAIN910_SPECIAL_LO/HI——118 门控 meta.neme_slain≥1、121 门控 comp_bonds
    // 任意 >0，均见 sim_event_weights）、常规 10（124..=133，mr 薄档优先——
    // 原 mr=3 仅 9 条/mr=4 仅 6 条/mr=5 仅 3 条，题材避开既有全部 118 条：
    // 断弦琴/龙窑夜火/埙/枯禅遗偈/兽潮/玉贩/引魂幡/磁山/天梯/断虹，其中稀有 1
    // （133 断虹饮涧 w=2））。扩池后账本重算：总基权 436→478（本批 +42），
    // 稀有 11 条 19 基权 ≈3.97%（维持 ~4% 口径）；mr 分布 0:38/1:34/2:34/3:11/
    // 4:9/5:4
    (118, 3, 2, true),  // 血册投书（链9 首段：斩过宿敌才入池）
    (119, 0, 0, true),  // 雪夜拦道（链9 中段，仅链内）
    (120, 0, 0, true),  // 断名之局（链9 末段，仅链内）
    (121, 3, 2, true),  // 旧页回声（链10 首段：历世结缘过才入池）
    (122, 0, 0, true),  // 灯市换页（链10 中段，仅链内）
    (123, 0, 0, true),  // 换页重逢（链10 末段，仅链内）
    (124, 4, 2, true),  // 断弦琴意
    (125, 4, 3, false), // 百窑夜火
    (126, 4, 4, true),  // 埙声穿垣
    (127, 4, 4, true),  // 枯禅遗偈
    (128, 4, 2, false), // 兽潮围栅
    (129, 4, 2, false), // 玉贩困隘
    (130, 3, 4, false), // 引魂幡影
    (131, 4, 3, false), // 磁山迷辙
    (132, 3, 5, true),  // 天梯断级
    (133, 2, 1, true),  // 断虹饮涧（稀有）
];

// ---------------- v1.16.10 事件回顾图鉴：静态名/简一一表 ----------------
// EVENT_NAMES[id] 与 build_event 各臂 card.title 逐 id 相等——双源漂移防线，相等性
// 由 game.rs event_names_match_card_titles 全 id 单测钉死（链内段含「·续」后缀；
// 宿敌/老年等动态文案臂的标题本身为静态字面量，动态只在正文）。
// EVENT_BRIEF[id] 为图鉴『遭遇手札』行的展示简介（一行 ≤18 字，display-only 无
// 对账约束；未遇事件整行 ？？？ 不显简介不泄题）。
pub const EVENT_NAMES: [&str; 134] = [
    "灵草满坡",
    "妖兽拦路",
    "上古洞府",
    "坊市淘宝",
    "恶修劫道",
    "道友论道",
    "宗门相邀",
    "心魔袭扰",
    "天材地宝",
    "灵脉洞府",
    "上古传承",
    "秘境探秘",
    "红尘历练",
    "灵气潮汐",
    "邪修偷袭",
    "仙人遗府",
    "妖丹入手",
    "云游商人",
    "前辈指点",
    "灵兽认主",
    "仙人问道",
    "灵泉洞天",
    "古剑冢",
    "魔窟探宝",
    "雷击木",
    "商队护送",
    "仙人赠礼",
    "天机阁卜卦",
    "同页天成",
    "宗门大比",
    "空白行",
    "雨夜借宿",
    "古碑残字",
    "雾锁迷林",
    "墨市笔坊",
    "断桥济人",
    "瘴谷寻药",
    "渡灵舟",
    "残卷拍卖",
    "灵田佃户",
    "酒楼斗法",
    "前人手札",
    "废墟道场",
    "沙暴藏窟",
    "货郎奇遇",
    "罗盘相赠·续",
    "恩怨上门·续",
    "古碑残篇",
    "拓印风波·续",
    "守碑人·续",
    "江底墨匣",
    "渔村释疑·续",
    "匣底真迹·续",
    "道侣论道",
    "赠剑同舟",
    "危难共护",
    "星坠荒原",
    "无面镜客",
    "灵潮倒卷",
    "书院讲经",
    "灵蜂酿蜜",
    "画师留影",
    "雪崩沟谷",
    "赌坊押擂",
    "漕运夜航",
    "温泉濯尘",
    "棋枰斗弈",
    "铸兵守炉",
    "蜃楼残景",
    "井底传声",
    "药市问诊",
    "驿壁题诗",
    "墨渊垂钓",
    "纸人夜行",
    "血仇余响",
    "同页灯约",
    "灯楼谜约",
    "灯下问答·续",
    "青灯问藏·续",
    "傩面还愿",
    "戏毕面热·续",
    "愿成符赠·续",
    "灯下缝页",
    "同摘霜橘",
    "论道起隙",
    "仇踪入册",
    "仇踪暗访·续",
    "乘虚夺积·续",
    "灯下了愿",
    "薪火相托",
    "雷池遗蜕",
    "仙门论道",
    "染坊秘色",
    "傀儡戏班",
    "雪夜煎茶",
    "断崖悬棺",
    "春汛鱼龙",
    "盐井卤灾",
    "矿洞钟声",
    "梦笔生花",
    "焚字炉",
    "笔冢祭",
    "涸河龙祠",
    "纸鸢传书",
    "砚山叩泉",
    "古戍点兵",
    "天蚕茧室",
    "空舟拦江",
    "墨鲸背城",
    "借名修行",
    "夺积余波",
    "仇册新墨",
    "灯花渡舟",
    "灯下旧账·续",
    "一渡一灯·续",
    "药炉童子",
    "炉语问心·续",
    "守炉一夜·续",
    // v1.16.11 扩池 16 条（118..=133）
    "血册投书",
    "雪夜拦道·续",
    "断名之局·续",
    "旧页回声",
    "灯市换页·续",
    "换页重逢·续",
    "断弦琴意",
    "百窑夜火",
    "埙声穿垣",
    "枯禅遗偈",
    "兽潮围栅",
    "玉贩困隘",
    "引魂幡影",
    "磁山迷辙",
    "天梯断级",
    "断虹饮涧",
];

pub const EVENT_BRIEF: [&str; 134] = [
    "山坳灵草正盛，采得一筐灵石",
    "林中妖兽拦路，一战定生死",
    "禁制残破的上古洞府，探或避",
    "坊市地摊捡漏，储物袋或丹药",
    "黑衣恶修拦路索财，战或破财",
    "山亭论道，道心修为两有所得",
    "宗门执事相邀：入我门下否",
    "旧执化魔袭扰识海，澄心自镇",
    "千年灵乳将成，护宝妖兽环伺",
    "天然灵脉洞府，本世修炼加成",
    "识海苍老声音，恭聆上古大道",
    "空间裂缝露秘境入口，深入或拾遗",
    "市井烟火磨道心，破财济困",
    "灵气暴涨如潮，顺势修炼正当时",
    "黑影暴起夺命，拼死一战",
    "山崩现仙人遗府，法宝灵气兼得",
    "两妖相争两败俱伤，捡漏妖丹",
    "蒙面商人售洗髓丹，八百灵石",
    "白眉老者点拨，醍醐灌顶",
    "雪白小兽有缘，收留得气运",
    "仙人三选一：求道、求武、求寿",
    "灵泉浸泡三日，寿元修为兼得",
    "十万锈剑古冢，中央古剑择主",
    "白骨魔窟，宝光与魔气交织",
    "雷击古木炼丹材，护神丹之引",
    "商队遭妖兽围困，护送得重金",
    "乘鹤老者掷锦盒，法宝与修为",
    "天机阁卜卦，破境丹或气运",
    "晒书日递来半页残稿，缝作同页",
    "十年一度大比，魁首任拣功法",
    "天曹漏写的一年，自己落笔",
    "山雨借宿，热粥与一夜山中旧事",
    "半截古碑残字，细读或拓售",
    "白雾锁林有人唤名，循或守心",
    "笔坊兽毫可定千行，购或观摩",
    "断桥货郎翻担，济人或收脚钱",
    "药王瘴谷灵药成片，取之有度",
    "江心水妖翻浪，助战或守心",
    "无名残卷藏功法总纲，竞或旁观",
    "灵田一夜白穗，除虫或指点",
    "酒楼斗法波及无辜，镇场或掩护",
    "手札夹页藏行功捷径，依注或转售",
    "废弃道场积尘，参悟或扫塔",
    "沙暴裸出藏窟，斩链或只探外洞",
    "货郎赊墨引灵，赊或不买皆有后话",
    "数年后货郎赠旧罗盘，结个善缘",
    "原主寻墨上门，见血或十倍赔付",
    "无字古碑洇出半字，拓或掩",
    "装裱老裱工示警，焚拓或见守碑人",
    "守碑人拦路三问，答错留拓",
    "江底铜匣錾「沉墨候主」，收或还",
    "老渔翁识匣来历，随往江心或辞谢",
    "匣底真迹枯润相生，参悟或转赠",
    "道侣发问：道途尽头是什么模样",
    "道侣赠剑『同舟』，佩或推辞",
    "山涧一抓手，合力御敌或护TA先走",
    "流星坠原，绢书与陨铁之择",
    "无面镜客索一赌，赌命或碎镜",
    "灵潮倒卷漩心洞府，冒死或引气",
    "蒙学书院讲经，坐听或捐笔",
    "崖壁灵蜜凝花髓，买蜜或护巢",
    "云游画师白描小像，入画或赠银",
    "雪线断裂积雪倾泻，开道或绕行",
    "地下斗擂押性命，登台或押注",
    "漕船缺护道，伴航或夜读",
    "野温泉化筋骨滞涩，泡浴或掘石",
    "一局棋下了三年，观棋或投子",
    "守一夜炉火，通宵或助锻",
    "海上蜃楼皆前人笔意，望或临",
    "古井夜半诵经，探井或墨祭",
    "药市老叟搭腕问诊，求脉案或帮碾",
    "驿壁末行墨迹未干，题诗或抄录",
    "墨渊钓叟钓水中字，受问或静观",
    "纸人列队抬小棺，拦问或让路",
    "被斩宿敌门生寻上门，报号或避锋",
    "灯节两盏莲灯写同一字，同放或叙旧",
    "青灯候能对灯谜的人，揭谜或折返",
    "满架残卷素布遮三层，应问或叙灯事",
    "最后一问无答案：你的书写到哪了",
    "傩班唱还愿戏，戴面代跳或出钱贴班",
    "面里封着老辈一笔愿，收仪或两清",
    "傩面削符可镇夜路，佩身或转赠",
    "灯下缝补撕损旧稿，掌灯或絮话",
    "霜后灵橘最甜，就树分食或留核",
    "墨写心还是写世，先开口或各自静修",
    "行脚商人私议宿敌，暗记或避开",
    "扮佣工探营外围，设伏或罢手",
    "倾巢亲信离营，取匣或毁布防",
    "灯芯将尽了旧愿，还愿或了心结",
    "少年问本事传给谁，收徒或赠稿",
    "雷池遗蜕雷纹未散，炼体或敬退",
    "仙门真传以十万废稿为注论道",
    "染坊晾出会动的青，裁衣或讨教",
    "三具傀儡只余一具会动，修或学艺",
    "雪夜山亭煎茶老者，共饮或问道",
    "断崖七具石棺夜有翻书声，探或拜",
    "春汛银鱼跃如逆下星雨，助渔或观化",
    "盐井黑卤有物翻身，下井或封井",
    "矿洞雨夜自鸣钟，镇钟或封洞",
    "三夜同梦一纸笔，提笔或不碰",
    "焚字炉火中未烬稿显字",
    "埋笔成冢夜有墨香，祭或掘",
    "涸河龙祠香火断，争水或修祠",
    "断线纸鸢带血书，循线或焚鸢",
    "砚形山叩之出泉墨，取泉或拓痕",
    "废戍楼夜半点兵声，入阵或绕行",
    "万茧悬室一茧有字，选茧或全买",
    "无人官船拦江，探舱或助推",
    "墨鲸浮水背上有城，登城或远观",
    "散修冒名开馆收徒，揭穿或默许",
    "夺积之后宿敌倾巢寻账，据守或散财",
    "新宿敌登册，旧斩之名的余波",
    "雾河渡船人只收旧渡资，讲旧事或付石",
    "灯纸上有你未写过的字迹，追问或登岸",
    "灯上尽是天曹删掉的名字，点灯或买灯",
    "荒废丹房炉火复燃，守炉童子求援",
    "炉里炼的是烧残稿，问下落或添柴",
    "炉开无丹只有一页完稿，收稿或还炉",
    // v1.16.11 扩池 16 条（118..=133）
    "血册残页钉门，应战或焚册",
    "缟素七人雪夜拦道递战书",
    "断名崖下终局，胜或两清",
    "灯市残页字迹似你旧稿",
    "摊主只收旧稿换一页新墨",
    "页背批注与你的字迹合璧",
    "山亭断弦古琴，续弦或换钱",
    "龙窑夜半自燃，探窑或封窑",
    "断垣埙声无人自鸣，循或掩",
    "坐化僧壁间遗偈，续或拓售",
    "灵兽潮围山栅，助守或引开",
    "玉贩困雪隘，援手或收酬",
    "荒岭白幡夜招魂，破或随行",
    "磁山吸针迷辙，引路或绕行",
    "通天石梯断级，补级或跃过",
    "雨后断虹垂涧有物栖，循或守",
];

// v1.16.7 稀有层阈值：EVENTS 权重 w∈1..=RARE_W 的卡按稀有呈现（描边紫；
// w=0 是链内段「仅链内出现」的哨兵，不算稀有）。扩池改权重后描边集合随之变化，
// 改动时对照本表注释（稀有基权占比维持 ~4%）
pub const RARE_W: i32 = 2;

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

// v1.14.0 立绘焕新时刻：四相名（跨档破境 toast『道貌焕然——【X之相】』+ 角色页常显行）
pub const PORTRAIT_STAGE_NAMES: [&str; 4] = ["白丁之相", "道基之相", "紫府之相", "墨仙之相"];

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

// ================= v1.14.0 人物小传·交游图鉴 =================
// 4 NPC × 3 段 + 道侣 3 段 = 15 段；解锁纯推导（NPC: fav≥20/40/70；道侣: 首段
// companion_lives≥1 保底、段2/3 comp_fav≥40/70）——零存档键，旧档天然兼容。
// 每卡常显最新解锁 1 段（14px ≤2 行 wrap），完整 3 段收进后续展开或下版。
// NPC_TALES[npc][seg]：seg 0/1/2 分别为点头之交(20)/相谈甚欢(40)/莫逆之交(70) 档故事。
pub const NPC_TALES: [[&str; 3]; 4] = [
    [
        "他掌外门三千杂役的名册，也掌着每一页被天曹划掉的废稿。有人说他记性差——其实他只是装作没看见。看得太多，就得写进册子里。",
        "某夜你路过后山，见他独自焚稿。火光里念的不是咒，是一串名字。次日他照旧板着脸派活，仿佛那夜的灰烬从不曾存在。",
        "他终于对你说了实话：藏经阁下压着半卷没写完的书，他是最后一个读完全卷的人。「守住它。别让天曹的笔，翻到那一页。」",
    ],
    [
        "万宝楼的账有三本：一本给天曹查，一本给自己赚，还有一本谁也没见过，记的是「人情」。你在第三本上，刚刚有了名字。",
        "他收过假丹，放过水货，唯独没让你的灵石短过一文。问他缘故，他拨着算盘笑：「熟客的秤，得准——这是招牌。」",
        "坊市大集那夜他多了几杯，翻出一枚旧算珠：「家师的。他说买卖做的是人心。这话在牌匾下挂了三十年，今日转赠于你。」",
    ],
    [
        "她练剑从不用好纸——剑意太利，纸留不住。演武场的青石上全是她一个人的剑痕，深浅如一，像同一个人写了千遍的「正」字。",
        "你见过她一次出鞘。不是对人，是对一场雨：剑起雨分，落回原处一滴不乱。她说剑心通明不是无情，「是不肯写错一笔。」",
        "破境那日她立于你身后，剑未出鞘，鞘上已覆薄霜。「天曹要改你的稿，先问过我这柄剑。」后来你才知道，她的剑从不替人出第二次。",
    ],
    [
        "他讨酒从不白讨：一壶浊酒换一句醉话，十句九虚，虚里那句偏又正中你眼下的要害。坊市都笑他疯，疯子记账却比谁都清醒。",
        "有回他醉倒在你洞府门口，酒葫芦滚了出来，葫芦底贴着半张文狱封条。你没问，他醒来也没提。那壶酒，是你请的。",
        "大醉那夜他忽然清醒：「我原是天曹的笔吏，写坏了一个名字，被贬下界做衬托。」顿了顿又道：「你这卷书写下去，别管我。」言毕复醉。",
    ],
];
// COMP_TALES[seg]：道侣槽共通小传（同页人），段 1 结缘即解（companion_lives≥1 保底）。
pub const COMP_TALES: [&str; 3] = [
    "两页残稿缝作同页，墨迹便再分不开彼此。她的名字住进你的行间，你的批注她也读得见——同页之人，同命之笔。",
    "有人问同页是什么滋味。她说不清，只说像抄书时忽然多了一只手替你压着纸角：风翻不走，雨洇不透，一笔一画都有人作证。",
    "你们试过在同一页写同一句诗，落笔竟一字不差。天曹的册子对不上这样的账——同页人的字迹，天曹的笔分不出谁是谁。",
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
