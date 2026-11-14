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
    // v1.16.20 章开场白（故事沉浸续作）：章 i 了边沿由 game.rs check_chapters 唯一
    // 追发下一章「【第X章·启】<open>」金行日志（章1 开局已有 intro、无前边沿不
    // 发；末章了后无下一章不发——首行 open 恒不演出，保非空供表契约单测钉死）。
    // 显示位=日志金行单行，预算 ≤24 字；不进章节书右栏（intro 预算不动）。
    pub open: &'static str,
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
        open: "方印无字，你有十万稿可写。",
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
        open: "轮回镜起雾了——镜里那人，先你一步醒。",
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
        open: "血仇的名字又浮上册面。这一次，你先出手。",
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
        open: "文狱的灯次第亮起：六张牒，一张都不能少。",
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
        open: "图志摊开，五域的风各有各的味道。",
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
        open: "线穿针眼——有人愿意与你缝作同页了。",
    },
    ChapterDef {
        num: "第七章",
        name: "问笔终战",
        intro: "六牒在手，修为圆满——现实破境的雷劫里，执笔人在等你。九重雷劫尽数扛下，便是落笔之刻。",
        goals: &[
            ChapterGoal { text: "现实问笔——九重雷劫扛下，首登仙界", pid: 10 },
            // v1.16.19 目标行去重：原「封卷已斩——讨牒 6/6 方见终战卡」与四章目标
            // 「六牒归档——讨牒 6/6，道韵+166」数字行全同——改叙事新句（pid5 不动）
            ChapterGoal { text: "六牒在手——封卷吏候你多时", pid: 5 },
        ],
        reward: 320,
        reward_note: "章成之礼：道韵 +320",
        lore: &[
            "其一：九重雷劫，一重一问。问到第七重时，天问累了——你没累。",
            "其二：天门开处没有仙宫，只有一张摆好纸笔的案。执笔人不在，笔在等你。",
        ],
        open: "雷云在峰顶集结。笔在案上，等你落款。",
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
        open: "十万稿废纸忽然安静——它们在等你一个结尾。",
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
            // v1.16.19 目标行去重：原句与八章第二目标「四卷全览——四种结局皆已落款」
            // 全同——改叙事新句（pid13 不动，纯字面量；达成谓词同口）
            ChapterGoal { text: "四卷重温——把别人的结局读进去", pid: 13 },
            ChapterGoal { text: "执卷境——历代最高修为抵执卷，书外续行", pid: 15 },
        ],
        reward: 500,
        reward_note: "章成之礼：道韵 +500",
        lore: &[
            "其一：书成之夜，方印上多了一行小字：本卷完，续卷无期。你笑了——无期，不是无。",
            "其二：合订的人临行回头：书外的路没有稿，走一步，是一步。你提印出门，天光正好。",
        ],
        open: "书成了。你的名字，还要往上写。",
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
// v1.16.12 洞府经营联动 3 条（134 阵眼共鸣/135 田畦岁稔/136 丹房遗香）：名下对应
// 设施满级才入池（门控在 game.rs sim_event_weights，读 meta.dongfu 跨世持久态）。
// 本批重算账本：总基权 478→489（本批 +11，全常规无稀有），稀有 11 条 19 基权
// ≈3.89%（分母增大略降，守 ~4% 口径）；mr 分布 0:42/1:36/2:35/3:11/4:9/5:4
// v1.16.23 减包刀③：元组收窄 (usize,i32,usize,bool)=16B/条 → (u16,u8,u8,bool)=5B/条
// （252 条 data 段 4032B→1260B）。id 字段语义不变（下标==id 的对账防线照旧，
// game.rs test 行 id 连续断言仍在）；读点统一 as usize 转换，下游类型零变。
// 权重上限 255（现单条最大 12，tools/tables_recheck.py 静态对账同步收窄口径）。
pub const EVENTS: [(u16, u8, u8, bool); 281] = [
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
    // 稀有 11 条 19 基权 ≈3.97%（维持 ~4% 口径）；mr 分布（全行口径含链段，
    // id≤133 共 134 条）0:42/1:34/2:34/3:11/4:9/5:4——v1.16.13 复审⑩修正原注
    // 0:38（≤117 批前值误抄；总基权 478 之后 v1.16.12 再扩 3 条，现全量 489）。
    // v1.16.12 补 134..=136（阵眼共鸣/田畦岁稔/丹房遗香）后全表 137 条重算：
    // 0:42/1:36/2:35/3:11/4:9/5:4，总基权 489，稀有 11 条 19 基权 ≈3.89%
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
    (134, 4, 1, true),  // 阵眼共鸣（名下聚灵阵满级才入池，v1.16.12）
    (135, 4, 1, true),  // 田畦岁稔（名下灵田满级才入池，v1.16.12）
    (136, 3, 2, true),  // 丹房遗香（名下丹房满级才入池，v1.16.12）
    // v1.16.13 扩池 14 条（id 137..=150）：链11 雷劫侧写 3（137..=139 劫痕渡客——
    // 邻山修士渡劫的旁观线，只写劫云过境/劫灰余韵侧写，绝不触玩家自身破境裁决，
    // 与问笔卡特化 special13/14/15/16 及 sim_do_break 分域；置链臂 45/46 第五窗见
    // CHAIN1112_SPECIAL_LO/HI）/链12 书肆换届 3（140..=142，置链臂 47/48 同窗），
    // 链内段权重 0；常规 8（143..=150，mr 薄档优先——原 mr=3 仅 11/mr=4 仅 9/
    // mr=5 仅 4，其中稀有 1（145 冰窖遗鲜 w=2））。题材避开既有全部 137 条：
    // 公道秤/渡蚁桥/冰窖遗鲜/夜磨坊/货栈灵鼠/采薇客/补鞍匠/雪泥鸿迹（初选
    // 「驿壁题诗」撞 71 既有名、「星坠原」撞 56 星坠荒原、「云外钟声」撞 98
    // 矿洞钟声、「雪夜扣门」撞 31 雨夜借宿，均弃）。扩池后账本重算：总基权
    // 489→520（本批 +31），稀有 12 条 21 基权 ≈4.04%（维持 ~4% 口径）；mr 分布
    //（全行口径含链段，151 条）0:46/1:36/2:35/3:13/4:15/5:6
    (137, 3, 4, true),  // 劫云过境（链11 首段：邻山渡劫侧写）
    (138, 0, 0, true),  // 劫灰拾遗（链11 中段，仅链内）
    (139, 0, 0, true),  // 劫后茶话（链11 末段，仅链内）
    (140, 3, 4, true),  // 书肆易主（链12 首段）
    (141, 0, 0, true),  // 代笔提要（链12 中段，仅链内）
    (142, 0, 0, true),  // 肆后书案（链12 末段，仅链内）
    (143, 3, 3, true),  // 公道秤
    (144, 3, 3, true),  // 渡蚁桥
    (145, 2, 5, true),  // 冰窖遗鲜（稀有）
    (146, 4, 4, true),  // 夜磨坊
    (147, 3, 4, false), // 货栈灵鼠
    (148, 3, 5, true),  // 采薇客
    (149, 4, 4, true),  // 补鞍匠
    (150, 3, 4, false), // 雪泥鸿迹
    // v1.16.14 扩池 5 条（id 151..=155）：链13 死当归主 3（151..=153，当铺死当
    // 旧物归主线——货郎=行商、当铺=坐商，物—人—缘结构与既有十二链同构而载体
    // 全新，「归」字刻痕贯穿三段；置链臂 49/50 第六窗见 CHAIN13_SPECIAL_LO/HI，
    // 链内段权重 0）/宗门换届 2（154 新掌接印/155 旧掌传灯：入宗门 sim.sect
    //（会话）且行走东境青云（map_region%5==0，宗门兴望 quest_mult 1.20）才入池，
    // 门控见 sim_event_weights——把 quest_mult 1.20 抽象机制落实为掌门接印/传灯
    // 人物故事）。题材避开既有全部 151 条（「当/赎/掌/箫/归」字全表名零占用）。
    // 扩池后账本重算：总基权 520→529（本批 +9），稀有 12 条 21 基权 ≈3.97%
    //（维持 ~4% 口径）；mr 分布（全行口径含链段，156 条）
    // 0:48/1:38/2:35/3:14/4:15/5:6
    (151, 3, 3, true),  // 当铺死当（链13 首段）
    (152, 0, 0, true),  // 访主赎当（链13 中段，仅链内）
    (153, 0, 0, true),  // 物归旧主（链13 末段，仅链内）
    (154, 3, 1, true),  // 新掌接印（入宗门+东境青云才入池）
    (155, 3, 1, true),  // 旧掌传灯（入宗门+东境青云才入池）
    // v1.16.15 扩池 9 条（156..=164）：伴随深化 3（156 雪窖藏酿/157 同守昙花/
    // 158 雨后补瓦：道侣生活流——题材避开既有 6 伴随「论道/赠剑/危难/灯约/
    // 缝页/霜橘」与既有全表；COMPANION_EVENTS 白名单门控见 game.rs）+ 常规 6
    // （159 萤火引路/160 山洪改道/161 白鹿过涧/162 月蚀之夜/163 壶中天/164 檐铃
    // 无风：山野异象与民俗生活流——全常规零稀有、纯 dao/stones/luck/qi_pct
    // 效果、不占 special，灵石臂照 ss 缩放）。扩池后账本重算：总基权 529→556
    // （本批 +27，全常规无稀有），稀有 12 条 21 基权 ≈3.78%（分母增大略降，守
    // ~4% 口径）；mr 分布（全行口径含链段，165 条）0:48/1:42/2:39/3:15/4:15/5:6
    (156, 3, 1, true),  // 雪窖藏酿（伴随）
    (157, 3, 1, true),  // 同守昙花（伴随）
    (158, 3, 2, true),  // 雨后补瓦（伴随）
    (159, 3, 1, true),  // 萤火引路
    (160, 3, 2, true),  // 山洪改道
    (161, 3, 2, true),  // 白鹿过涧
    (162, 3, 2, true),  // 月蚀之夜
    (163, 3, 3, true),  // 壶中天
    (164, 3, 1, true),  // 檐铃无风
    // v1.16.16 扩池 5 条（165..=169）：结局回响 3（165 定稿之世/166 留白之野/
    // 167 执笔之赠：对应结局已落款（ed= bit0/1/2 持久位）才入池——落款之后世界
    // 首次有回响，门控见 game.rs sim_event_weights，照 111 读持久位零新键；合订
    // bit3 为隐藏卷余韵不另占池）+ 书外之章联动 2（168 天曹补写/169 书肆新声：
    // 第九章·书外之章已了（ch= bit8）才入池，照 74/75 章节联动先例）。题材避开
    // 既有全部 165 条（「话本/野地/描名/晒卷/无脊」意象全表零占用）。扩池后账本
    // 重算：总基权 556→571（本批 +15，全常规零稀有），稀有 12 条 21 基权 ≈3.68%
    //（分母增大略降，守 ~4% 口径）；mr 分布（全行口径含链段，170 条）
    // 0:48/1:42/2:39/3:15/4:18/5:8
    (165, 3, 4, true),  // 定稿之世（结局回响：A 定稿已落款）
    (166, 3, 4, true),  // 留白之野（结局回响：B 留白已落款）
    (167, 3, 4, true),  // 执笔之赠（结局回响：C 执笔已落款）
    (168, 3, 5, true),  // 天曹补写（书外之章联动）
    (169, 3, 5, true),  // 书肆新声（书外之章联动）
    // v1.16.16 扩池 13 条（170..=182）：老年层 +2（170 炉暖旧器/171 灯影授业，
    // 「交托回望」散件——老年层从 88/89 两条独占扩到 4 条，老年门控同批扩
    // id∈{88,89,170,171} 见 game.rs sim_event_weights）/开局层 +3（172 义渡/
    // 173 村塾借灯/174 霜桥早市，mr0 生活流——摊薄开局头部权重，白丁常态池
    // 22→27 条/118→137 权、前四张集中度 33%→28.5%（实测））/新链14 修伞记
    // 3（175 首段 w3+176/177 链内
    // w0：凡伞铺老匠只修凡伞不识法器，与遗宝墨骨伞刻意分域反差；置链臂 53/54
    // 第七窗见 CHAIN14_SPECIAL_LO/HI）/恶缘 +2（178 沼瘴蚀符/179 赝丹局——缓
    // 善缘偏甜与 mr2/3 薄档，179 w2 顺带入稀有层）/高境风味 +2（180 云阶试步/
    // 181 观澜悟潮，mr5 独占 6→8）/伴随 +1（182 霜夜补裘，COMPANION_EVENTS
    // 扩 10）。扩池后账本重算：总基权 571→607（本批 +36，全批 w≤5），稀有
    // 13 条 23 基权 ≈3.79%（守 ~4% 口径）；mr 分布（全行口径含链段，183 条）
    // 0:55/1:44/2:40/3:16/4:18/5:10
    (170, 3, 1, true),  // 炉暖旧器（老年层：余量≤10 载才入池）
    (171, 3, 0, true),  // 灯影授业（老年层：余量≤10 载才入池）
    (172, 5, 0, true),  // 义渡
    (173, 4, 0, true),  // 村塾借灯
    (174, 4, 0, true),  // 霜桥早市
    (175, 3, 0, true),  // 修伞记（链14 首段）
    (176, 0, 0, true),  // 伞骨重削（链14 中段，仅链内）
    (177, 0, 0, true),  // 还伞（链14 末段，仅链内）
    (178, 3, 2, false), // 沼瘴蚀符
    (179, 2, 3, false), // 赝丹局（稀有）
    (180, 3, 5, true),  // 云阶试步
    (181, 3, 5, true),  // 观澜悟潮
    (182, 3, 1, true),  // 霜夜补裘（伴随）
    // v1.16.17 扩池 10 条（183..=192）：mr5 补薄 +2（186 仙曹索卷 恶缘/187 星槎
    // 访客 善缘——mr5 无条件常态 7→9 条（表口径含结局/书外门控 10→12 条），
    // 缓「数百年事件年里 7 卡轮换」的重复感）/ mr3 恶缘 +1（188 灵税催帖，
    // 仙府催赋，mr3 恶缘 6→7 条）/伴随 +3（183 灯下习字 mr0——白丁
    // 有伴层首条伴随、184 同参一炉 mr2 双修题材（Eff.comp_fav 好感通道首用 +
    // COMPANION_GIFT_FAV）、185 云海携游 mr4——伴随白名单扩 13，此前 10 条
    // mr 全≥1 且≤3，白丁有伴与 mr4+ 零覆盖）/老年层 +1（189 旧履还坊，门控
    // 同批扩 id∈{88,89,170,171,189}）/章节联动 +2（190 雨夜寻印 = 第一章
    // 雨夜执印 bit0、191 文狱退牒 = 第四章文狱讨牒 bit3——一/四章首获覆盖，
    // 照 74/75 同点门控写法）/开局层 +1（192 书僮窃墨 w4——白丁常态池
    // 27→30 条/137→147 权、前四张集中度 28.5%→27.1%（无伴口径 39/144，
    // tools/ledger_recheck.py 实测复算；表口径含伴 39/147））。扩池后账本重算：
    // 总基权 607→638（本批 +31，全批 w≤4），稀有 13 条 23 基权 ≈3.61%
    // （守 ~4% 口径，纯稀释零新增）；恶缘（常规 w>0）32→34 条共 137 权
    // ≈21.5%；mr 分布（全行含链段，193 条）0:58/1:46/2:41/3:17/4:19/5:12；
    // 常态池（w>0）mr 分布 0:30 条147权/1:46 条/2:41 条/3:17 条57权/
    // 4:19 条61权/5:12 条38权
    // v1.16.18 扩池 12 条（193..=204，本批 +37 全 w≥3 稀有区零新增）：
    // mr6 长生世独占首批 3（193/194/195——realm<mr 归零天然支持 mr=6 零代码）、
    // 章节联动 2（196=bit1 二章/197=bit4 五章）、老年层 2（198/199）、
    // 恶缘 4（194/200/201/202——202 w4 摊薄 mr0 妖兽拦路独大）、伴随白名单
    // +1（203）、老年×有伴双门控 1（204）。扩池后账本（tools/ledger_recheck.py
    // 实测复算）：总基权 638→675；稀有 13 条 23 基权 ≈3.41%（纯稀释）；
    // 恶缘常规 34→38 条共 150 权 ≈22.2%；mr 分布（全行含链段，205 条）
    // 0:60/1:49/2:42/3:18/4:20/5:13/6:3（mr6 首次出现）；常态池（w>0）
    // mr 分布 0:32 条154权/1:49 条188权/2:42 条159权/3:18 条60权/
    // 4:20 条64权/5:13 条41权/6:3 条9权；mr0 开局头部 top4 集中度
    // 27.1%→25.3%（含伴口径 39/154；无伴口径 25.8%——分母被本批摊薄，
    // 头部四张仍为 0/1/3/5 四条老卡）
    (183, 3, 0, true),  // 灯下习字（伴随：白丁有伴层）
    (184, 3, 2, true),  // 同参一炉（伴随：双修题材，好感+5）
    (185, 3, 4, true),  // 云海携游（伴随：mr4 层）
    (186, 3, 5, false), // 仙曹索卷（mr5 恶缘）
    (187, 3, 5, true),  // 星槎访客（mr5 善缘）
    (188, 3, 3, false), // 灵税催帖（mr3 恶缘）
    (189, 3, 0, true),  // 旧履还坊（老年层：余量≤10 载才入池）
    (190, 3, 1, true),  // 雨夜寻印（章节联动：第一章·雨夜执印已了才入池）
    (191, 3, 1, true),  // 文狱退牒（章节联动：第四章·文狱讨牒已了才入池）
    (192, 4, 0, true),  // 书僮窃墨
    // v1.16.18 扩池 12 条（193..=204）
    (193, 3, 6, true),  // 后辈证道（mr6 长生世独占首批）
    (194, 3, 6, false), // 冒籍疑云（mr6 恶缘）
    (195, 3, 6, true),  // 万卷尘页（mr6 善缘）
    (196, 3, 1, true),  // 镜影忆战（章节联动：第二章·镜中万象已了才入池）
    (197, 3, 2, true),  // 五域风谣（章节联动：第五章·山河万象已了才入池）
    (198, 3, 0, true),  // 窗前晒卷（老年层：余量≤10 载才入池）
    (199, 3, 1, true),  // 旧邻问名（老年层：余量≤10 载才入池）
    (200, 3, 5, false), // 仙籍勾误（mr5 恶缘）
    (201, 3, 3, false), // 坊契赖账（mr3 恶缘）
    (202, 4, 0, false), // 夜叩柴门（mr0 恶缘：摊薄开局妖兽拦路独大）
    (203, 3, 4, true),  // 云海拾简（伴随白名单第 14 条，mr4 层）
    (204, 3, 1, true),  // 同席说岁（老年×有伴双门控：余量≤10 且有伴才入池）
    // v1.16.19 扩池 14 条（205..=218，本批 +36；分类：链15 三条（205 首段入池
    // w3 + 206/207 链内 w0 不计权）+ mr7 长生世风味 2 + 伴随 3 + mr6 稀有恶缘
    // 各 1 + 老年层 2 + 章节联动 1 + mr0 恶缘 1，共 14 条）：
    // 链15 观星问岁（205/206/207——醉道人携酒观星指星对账，Npc(3) 真立绘；
    // 置链臂 56/57 第八窗见 CHAIN15_SPECIAL_LO/HI，CHAIN_FIRSTS/CHAIN_NAMES/
    // gallery 链起名单三处同批登记；缘了闩 bit14）/
    // mr7 长生世 2（208 剑冢阅武 w4 善/209 界碑拓铭 w3 恶——mr 列首现 7，
    // realm<mr 归零天然支持零新代码；长生世纪律：不写寿元将尽、不触破境裁决）/
    // 伴随 3（210 雨檐听铃 mr3 补伴随 mr3 零位/211 云上放灯 mr5 高境有伴/
    // 212 相扶看雪 老年×有伴双门控第二条照 204 模板——白名单 14→17，
    // tools/ledger_recheck.py COMPANION 集合同批同步；全挂 Comp 立绘）/
    // mr6 稀有 1（213 仙池映世 w2，稀有层 mr6 首条）+ mr6 恶缘 1（214 晚钟
    // 误点，mr6 恶缘 194 独苗补一条）/老年层 2（215 灶火留谱/216 旧约转托，
    // 门控 id 集合同批扩）/章节联动 1（217 邻人扛劫 = 第七章 bit6 缺角补齐，
    // 「邻人扛劫」侧写守链11 分域纪律）/mr0 恶缘 1（218 囊带乍断 w3 摊薄
    // 开局头部）。扩池后账本（tools/ledger_recheck.py 实测复算）：总基权
    // 675→711（本批 +36）；稀有 14 条 25 基权 ≈3.52%（w2 新条拉回，向 ~4%
    // 口径回靠）；恶缘常规 38→41 条共 159 权 ≈22.4%；mr 分布（全行含链段，
    // 219 条）0:64/1:52/2:42/3:19/4:20/5:14/6:6/7:2（mr7 首次出现）；
    // 常态池（w>0）mr 分布 0:34 条160权/1:52 条197权/2:42 条159权/
    // 3:19 条63权/4:20 条64权/5:14 条44权/6:6 条17权/7:2 条7权；
    // mr0 开局头部 top4 集中度 25.3%→24.4%（含伴口径 39/160；无伴口径
    // 24.8%——头部四张仍为 0/1/3/5 四条老卡，新恶缘 w3 入不了头部）
    (205, 3, 6, true),  // 观星问岁（链15 首段）
    (206, 0, 0, true),  // 指间星图（链15 中段，仅链内）
    (207, 0, 0, true),  // 问岁之约（链15 末段，仅链内）
    (208, 4, 7, true),  // 剑冢阅武（mr7 长生世风味首条）
    (209, 3, 7, false), // 界碑拓铭（mr7 恶缘）
    (210, 3, 3, true),  // 雨檐听铃（伴随白名单第 15 条，mr3 层）
    (211, 3, 5, true),  // 云上放灯（伴随白名单第 16 条，mr5 层）
    (212, 3, 1, true),  // 相扶看雪（伴随白名单第 17 条，老年×有伴双门控）
    (213, 2, 6, true),  // 仙池映世（mr6 稀有）
    (214, 3, 6, false), // 晚钟误点（mr6 恶缘）
    (215, 3, 0, true),  // 灶火留谱（老年层：余量≤10 载才入池）
    (216, 3, 1, true),  // 旧约转托（老年层：余量≤10 载才入池）
    (217, 3, 1, true),  // 邻人扛劫（章节联动：第七章·问笔终战已了才入池）
    (218, 3, 0, false), // 囊带乍断（mr0 恶缘）
    // v1.16.20 扩池 12 条（219..=230，本批 +30；分类：链16 三条（219 首段入池
    // w3 + 220/221 链内 w0 不计权）+ mr7 最薄层 3 + 伴随 2 + 老年层 1 +
    // mr6 常态 1 + mr3 恶缘 1 + 章节联动 1，共 12 条）：
    // 链16 剑穗之约（219/220/221——宁清霜剑穗线 Npc(2) 真立绘，文狱四 NPC
    // 唯一无专属事件者补缺；置链臂 60/61 第九窗见 CHAIN16_SPECIAL_LO/HI，
    // 62=「一夜结完」窗内不置链臂（断链收获）仅 eff_desc 副标；CHAIN_FIRSTS/
    // CHAIN_NAMES/gallery 链起名单三处同批登记；缘了闩 bit15）/
    // mr7 最薄层 3（222 星砂还账 w4 善 Npc(3) 醉道人立绘——链15「明年今夜」
    // 散件呼应不依赖链态/223 玉价新例 w3 恶/224 坠星动冢 w2 稀有层 mr7 首条
    // ——长生世纪律：不写寿元将尽、不触破境裁决）/伴随 2（225 同数流星 mr6
    // 有伴首条/226 霜晨共伞 mr0 有伴第 2 条照 174 霜桥早市成对分域——白名单
    // 17→19，tools/ledger_recheck.py COMPANION 集合同批同步；全挂 Comp 立绘）/
    // 老年层 1（227 半檐躲雨，门控 id 集合同批扩；老年层不挂 chain_latch 跨世
    // 重见率最高层，边际收益递减故仅 +1）/mr6 常态 1（228 池边洗鞘 w3 善）/
    // mr3 恶缘 1（229 佃契抽成 w3，与 39 灵田佃户成对分域）/章节联动 1
    //（230 落款撞名 = 第八章·落款成书 bit7 缺角补齐；与结局回响 165-167
    // 「世界侧写」分域写亲历视角）。扩池后账本（tools/ledger_recheck.py 实测
    // 复算）：总基权 711→741（本批 +30）；稀有 15 条 27 基权 ≈3.64%（224 w2
    // 入稀有区，向 ~4% 口径回靠）；恶缘常规 41→43 条共 165 权 ≈22.3%；
    // mr 分布（全行含链段，231 条）0:68/1:53/2:42/3:20/4:21/5:14/6:8/7:5；
    // 常态池（w>0）mr 分布 0:36 条166权/1:53 条200权/2:42 条159权/
    // 3:20 条66权/4:21 条67权/5:14 条44权/6:8 条23权/7:5 条16权；
    // mr0 开局头部 top4 集中度 24.4%→23.5%（含伴口径 39/166；无伴口径
    // 24.8%→24.4%——头部四张仍为 0/1/3/5 四条老卡，新条 w3/w4 入不了头部）
    (219, 3, 4, true),  // 剑穗生结（链16 首段）
    (220, 0, 0, true),  // 剑穗缓结（链16 中段，仅链内）
    (221, 0, 0, true),  // 剑成赠穗（链16 末段，仅链内）
    (222, 4, 7, true),  // 星砂还账（mr7 善缘：醉道人散件呼应链15）
    (223, 3, 7, false), // 玉价新例（mr7 恶缘）
    (224, 2, 7, true),  // 坠星动冢（mr7 稀有首条）
    (225, 3, 6, true),  // 同数流星（伴随白名单第 18 条，mr6 层首条）
    (226, 3, 0, true),  // 霜晨共伞（伴随白名单第 19 条，mr0 层第 2 条）
    (227, 3, 0, true),  // 半檐躲雨（老年层：余量≤10 载才入池）
    (228, 3, 6, true),  // 池边洗鞘（mr6 善缘）
    (229, 3, 3, false), // 佃契抽成（mr3 恶缘）
    (230, 3, 1, true),  // 落款撞名（章节联动：第八章·落款成书已了才入池）
    // v1.16.21 扩池 9 条（231..=239）：链17 星槎再访 3 条（mr5 零链补位——首段
    // mr5 善缘 w3 呼应 187 星槎访客，中/末段 w0 仅链内；special 窗 63..=65 第十窗，
    // CHAIN_FIRSTS/CHAIN_NAMES/gallery 链起/缘了闩 bit16 四处同批登记）/稀有检定 1
    //（234 旧砚问道 w2 mr2，special66 道心检定照 special33 模板——本轮可玩性主菜，
    // 过/败双结局）/mr5 恶缘 1（235 影市黑秤 w3，3→4 条）/mr7 善缘 1（236 云海拾阶
    // w4，守长生世纪律不写寿元不触破境裁决，照 208/222 先例）/mr7 恶缘 1（237 剑赋
    // 催帖 w3，双臂「纳赋/以剑代赋」fight 臂照 4 恶修劫道模板）/mr4 恶缘 1（238
    // 行会追赔 w3，与 188 灵税催帖/229 佃契抽成题材分域）/老年×有伴双门控 1（239
    // 共守岁灯 w3 mr1，照 204 模板同 if 并臂+gallery 老年臂登记；不入 COMPANION
    // 白名单——212 死模式注释，白名单成员先命中恒出「伴随」徽标）。扩池后账本
    //（tools/ledger_recheck.py 实测复算）：总基权 741→762（本批 +21）；稀有 15 条
    // 27 权→16 条 29 权 ≈3.81%；恶缘常规 43→46 条共 174 权 ≈22.8%；mr 全行分布
    // （240 条）0:70/1:54/2:43/3:20/4:22/5:16/6:8/7:7；常态池（w>0）
    // 0:36 条166权/1:54 条203权/2:43 条161权/3:20 条66权/4:22 条70权/
    // 5:16 条50权/6:8 条23权/7:7 条23权；mr0 开局头部 top4 集中度不变（本批
    // mr0 仅 w0 链段，0/1/3/5 四老卡仍 39/166≈23.5%）
    (231, 3, 5, true),  // 星槎再访（链17 首段，mr5 零链补位）
    (232, 0, 0, true),  // 星轨留书·续（链17 中段，仅链内）
    (233, 0, 0, true),  // 槎去留星·续（链17 末段，仅链内）
    (234, 2, 2, true),  // 旧砚问道（稀有 w2：special66 道心检定）
    (235, 3, 5, false), // 影市黑秤（mr5 恶缘）
    (236, 4, 7, true),  // 云海拾阶（mr7 善缘，守长生世纪律）
    (237, 3, 7, false), // 剑赋催帖（mr7 恶缘，fight 臂）
    (238, 3, 4, false), // 行会追赔（mr4 恶缘）
    (239, 3, 1, true),  // 共守岁灯（老年×有伴双门控第 3 条）
    // v1.16.22 扩池 12 条（240..=251，本批 +32 基权；分类：伴随 2（240 药圃赌芽
    // mr3 第 2 条摊薄 210 独苗/241 踏云题字 mr7 有伴首条——白名单 19→21，
    // tools/ledger_recheck.py COMPANION 集合同批同步；全挂 Comp 立绘照 210/211
    // 反查模式）/mr6 常态 2（242 灵渠截流 w3 恶——mr6 恶缘第 3 条/243 村谷抢收
    // w3 善）/mr7 善缘 2（244 酒账千年 w4 醉道人 Npc(3) 散件照 222 模板/250 云窟
    // 归卷 w3 链18 首段——mr7 零链补位）/mr4 恶缘 1（245 滩头讹偿 w3，与 37 渡
    // 灵舟真劫/238 行会契赔分域走滩头讹索）/稀有 2（246 松风问琴 w2 mr3 稀有
    // 第 3 条，special69 道心检定照 234 模板——apply_eff 十一窗判定随批；247 雾锁
    // 仙市 w2 mr5 稀有第 2 条纯效果散件）/具名 NPC 2（248 长老巡山 w3 mr2 善
    // ——Npc(0) 玄诚长老守碑人线外首卡/249 万宝旧档 w3 mr1 善——Npc(1) 胡掌柜
    // 链1 世界外延，皆显式具名与脸同源）/链18 末段 1（251 补笔归卷·续 w0 仅链内；
    // special 窗 70..=72 第十一窗，CHAIN_FIRSTS/CHAIN_NAMES/gallery 链起/缘了闩
    // bit17 四处同批登记）。长生世纪律：mr7 三条不写寿元将尽、不触破境裁决。
    // 扩池后账本（tools/ledger_recheck.py 实测复算）：总基权 762→794（本批 +32）；
    // 稀有 16 条 29 权→18 条 33 权 ≈4.16%；恶缘常规 46→48 条共 180 权 ≈22.7%；
    // mr 全行分布（252 条）0:71/1:55/2:44/3:22/4:23/5:17/6:10/7:10；常态池（w>0）
    // 217 条：0:36 条166权/1:55 条206权/2:44 条164权/3:22 条71权/4:23 条73权/
    // 5:17 条52权/6:10 条29权/7:10 条33权；mr0 开局头部 top4 集中度不变
    //（本批无 mr0 常态新条，0/1/3/5 四老卡仍 39/166≈23.5%）
    (240, 3, 3, true),  // 药圃赌芽（伴随白名单第 20 条，mr3 层第 2 条）
    (241, 3, 7, true),  // 踏云题字（伴随白名单第 21 条，mr7 有伴首条）
    (242, 3, 6, false), // 灵渠截流（mr6 恶缘第 3 条）
    (243, 3, 6, true),  // 村谷抢收（mr6 善缘）
    (244, 4, 7, true),  // 酒账千年（mr7 善缘：醉道人散件照 222 模板）
    (245, 3, 4, false), // 滩头讹偿（mr4 恶缘第 6 条）
    (246, 2, 3, true),  // 松风问琴（稀有 w2：special69 道心检定）
    (247, 2, 5, true),  // 雾锁仙市（mr5 稀有第 2 条）
    (248, 3, 2, true),  // 长老巡山（具名 Npc(0) 玄诚长老）
    (249, 3, 1, true),  // 万宝旧档（具名 Npc(1) 胡掌柜）
    (250, 3, 7, true),  // 云窟归卷（链18 首段，mr7 零链补位）
    (251, 0, 0, true),  // 补笔归卷·续（链18 末段，仅链内）
    // v1.16.23 扩池 7 条（252..=258 尾部追加，id==下标纪律勿插中）：账本以
    // tools/ledger_recheck.py 重算为准（794+20=814；稀有 18→19 条 33→35 权
    // ≈4.30%；恶缘常规 48→50 条 180→186 权 ≈22.9%）。宁清霜散件首卡照
    // 248/249 具名+脸同源模式（正文钉名⟺Npc(n)）；mr6/7 新条守长生世纪律
    // （不写寿元将尽、不触破境裁决）；链19 不做（special73 与下一检定槽互斥
    // ——v1.16.25 注：检定槽 73 已由稀有检定 279 启用，链19 后续若做改走
    // 74..=76 新窗，互斥随之解除）
    (252, 3, 2, true),  // 枰边手谈（宁清霜散件首卡 Npc(2)，mr2 善）
    (253, 3, 3, true),  // 经阁理简（玄诚长老第二卡 Npc(0)，mr3 善）
    (254, 3, 5, false), // 旧败上门（mr5 恶 4→5 条）
    (255, 3, 6, true),  // 旧友茶叙（mr6 善守世纪律）
    (256, 3, 7, false), // 冒名敛财（mr7 恶 3→4 条守世纪律）
    (257, 3, 4, true),  // 灯市挑灯（伴随 mr4 第 3 条，白名单 21→22）
    (258, 2, 1, true),  // 枯井鸣灵（mr1 稀有 w2：57/133 后第 3 条）
    // v1.16.24 扩池 12 条（259..=270 尾部追加，id==下标纪律勿插中）：账本以
    // tools/ledger_recheck.py 重算为准。具名 NPC 五卡照 248/249/252 具名+脸
    // 同源模式（正文钉名⟺Npc(n)）；259 市头新肆为 mr0 首张 NPC 专卡，摊薄
    // 开局头部（top4 含伴 23.5%→22.7%）；260 恶缘摊薄 mr0 妖兽拦路 w11 独大；
    // 265 稀有 w2 补 mr4 薄档（108 独苗）；268 老年×有伴双门控第 4 条照
    // 204/212/239 同 if 模板不入白名单；262 伴随白名单 22→23（tools/
    // ledger_recheck.py COMPANION 集合同批同步）；270 链18 余波散件照 222
    // 呼应模板（special 恒 0，链19 不做）。mr6/7 新条守长生世纪律（不写寿元
    // 将尽、不触破境裁决）。题材复检弃撞：265 信鸟投窗与 68 纸鸢传书分域
    // （纸鸢是人题诗、信鸟是天送无主信）；269 坊契转押与 201 赖账/229 抽成
    // 三案分域（赖账收不回/抽成薄收成/转押铺易手）。
    // 扩池后账本实测复算（tools/ledger_recheck.py）：总基权 814→849（本批
    // +35，全批 w≤3）；稀有 19→20 条 35→37 权 ≈4.30%→4.36%（守 ~4% 口径）；
    // 恶缘常规 50→53 条 186→195 权 ≈22.9%→23.0%；mr 全行分布（271 条）
    // 0:73/1:59/2:45/3:24/4:27/5:20/6:11/7:12；常态池（w>0）236 条：
    // 0:38 条172权/1:59 条217权/2:45 条167权/3:24 条77权/4:27 条84权/
    // 5:20 条61权/6:11 条32权/7:12 条39权；mr0 开局头部 top4 集中度
    // （含伴）23.5%→22.7%（无伴口径 24.4%→23.5%，分母被本批两条摊薄，
    // 头部四张仍为 0/1/3/5 四老卡）
    (259, 3, 0, true),  // 市头新肆（mr0 首张 NPC 专卡：胡掌柜 Npc(1)）
    (260, 3, 0, false), // 夜渡多收（mr0 恶缘 4→5 条）
    (261, 3, 1, true),  // 醉倒门前（醉道人 Npc(3) 低境首卡，呼应小传）
    (262, 3, 1, true),  // 同研一笺（伴随白名单第 23 条，mr1 第 10 条）
    (263, 3, 4, true),  // 旧穗试锋（宁清霜 Npc(2)，链16 世界外延）
    (264, 3, 4, true),  // 盘账夜灯（胡掌柜 Npc(1)，算珠线余波）
    (265, 2, 4, true),  // 信鸟投窗（稀有 w2：mr4 与 108 分域）
    (266, 3, 5, false), // 官牙抽引（mr5 恶缘 5→6 条）
    (267, 3, 5, true),  // 名册补名（玄诚长老 Npc(0)，名册线余波）
    (268, 3, 1, true),  // 炉边说旧（老年×有伴双门控第 4 条）
    (269, 3, 3, false), // 坊契转押（mr3 恶缘）
    (270, 3, 7, true),  // 窟灯未熄（mr7 善缘：链18 余波散件，守世纪律）
    // v1.16.25 扩池 10 条（271..=280 尾部追加，id==下标纪律勿插中）：账本以
    // tools/ledger_recheck.py 重算为准。四具名 NPC 卡照 248/249/252 具名+脸
    // 同源模式（正文钉名⟺Npc(n)）；272 伴随白名单 23→24（tools/
    // ledger_recheck.py COMPANION 集合同批同步）；279 稀有 w2 special73 道心
    // 检定照 234/246 模板——本轮唯一 special 新支（检定槽 73，链19 不做互斥
    // 让位，见 game.rs special 窗注记）；276/277/278 散件守长生世纪律（不写
    // 寿元将尽、不触破境裁决）。题材复检弃撞：271 柜前压价与 249 万宝旧档
    // /201 坊契赖账/264 盘账夜灯分域（错认盗卖赃物 vs 旧档压款 vs 赊账翻脸
    // vs 年终盘账）；273 扫阶拾规与 248 巡山/253 经阁/267 名册分域（洒扫阶
    // 缝拾旧规）；274 醉引歧路与 261 醉倒门前/222 星砂还账/链15 观星分域
    // （醉话指路）；275 寄剑西窗与 252 手谈/263 旧穗试锋分域（托剑寄存）；
    // 277 故市抬租与 269 坊契转押/229 佃契抽成分域（连年抬租）；279 空谷叩
    // 问与 246 松风问琴分域（谷之回声 vs 亭之无弦琴）；280 堰口分水与 243
    // 村谷抢收/39 灵田佃户分域（修堰分水）。
    // 扩池后账本实测复算（tools/ledger_recheck.py）：总基权 849→878（本批
    // +29=9×w3+1×w2）；稀有 20→21 条 37→39 权 ≈4.36%→4.44%；恶缘常规
    // 53→56 条 195→204 权 ≈23.0%→23.2%；mr 全行分布（281 条）
    // 0:74/1:59/2:47/3:26/4:28/5:21/6:13/7:13；常态池（w>0）246 条：
    // 0:39 条175权/1:59 条217权/2:47 条173权/3:26 条82权/4:28 条87权/
    // 5:21 条64权/6:13 条38权/7:13 条42权；mr0 开局头部 top4 集中度
    //（含伴）22.7%→22.3%（无伴口径 23.5%→23.1%，分母被 280 摊薄，
    // 头部四张仍为 0/1/3/5 四老卡）
    (271, 3, 2, false), // 柜前压价（mr2 恶：胡掌柜 Npc(1)，错认盗卖对质）
    (272, 3, 2, true),  // 寒江共网（伴随白名单第 24 条，mr2 第 4 条）
    (273, 3, 3, true),  // 扫阶拾规（玄诚长老第 4 卡 Npc(0)，mr3 善）
    (274, 3, 5, false), // 醉引歧路（mr5 恶：醉道人 Npc(3)，醉话指路）
    (275, 3, 4, true),  // 寄剑西窗（宁清霜第 3 散卡 Npc(2)，mr4 善）
    (276, 3, 6, true),  // 岁星校图（mr6 善，守世纪律）
    (277, 3, 6, false), // 故市抬租（mr6 恶第 4 条）
    (278, 3, 7, true),  // 岁宴万年（mr7 善，守世纪律）
    (279, 2, 3, true),  // 空谷叩问（稀有 w2：special73 道心检定）
    (280, 3, 0, true),  // 堰口分水（mr0 善，摊薄开局头部）
];

// ---------------- v1.16.10 事件回顾图鉴：静态名/简一一表 ----------------
// EVENT_NAMES[id] 与 build_event 各臂 card.title 逐 id 相等——双源漂移防线，相等性
// 由 game.rs event_names_match_card_titles 全 id 单测钉死（链内段含「·续」后缀；
// 宿敌/老年等动态文案臂的标题本身为静态字面量，动态只在正文）。
// EVENT_BRIEF[id] 为图鉴『遭遇手札』行的展示简介（一行 ≤18 字，display-only 无
// 对账约束；未遇事件整行 ？？？ 不显简介不泄题）。
// v1.16.24 简语画面化第三批 10 条（id 55/58/135/136/139/141/143/154/200/216）：
// 记账腔改画面体（有场景有动作有实物，选项对仗照旧行尾「或」结构），每条改写
// 均与 build_event 臂正文逐条对过防跑题；净 −6 字（wasm 内嵌 &str 常量，字节
// 净减），0 新增字体字符。简语预算钉测见 game.rs event_brief_11624_batch_within_18。
// v1.16.25 第四批 10 条（10/13/56/100/165/166/209/235/242/258，清账批改写）
// 同口径零新字形，钉测 event_brief_11625_batch_within_18（勘察所列最平 10 条
// 至此清完）。
pub const EVENT_NAMES: [&str; 281] = [
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
    "阵眼共鸣",
    "田畦岁稔",
    "丹房遗香",
    // v1.16.13 扩池 14 条（137..=150）：链11/12 段名带 ·续 照中末段惯例，
    // 与 build_event card.title 双源钉死（event_names_match_card_titles）
    "劫云过境",
    "劫灰拾遗·续",
    "劫后茶话·续",
    "书肆易主",
    "代笔提要·续",
    "肆后书案·续",
    "公道秤",
    "渡蚁桥",
    "冰窖遗鲜",
    "夜磨坊",
    "货栈灵鼠",
    "采薇客",
    "补鞍匠",
    "雪泥鸿迹",
    // v1.16.14 扩池 5 条（151..=155）：链13 段名带 ·续 照中末段惯例，
    // 与 build_event card.title 双源钉死（event_names_match_card_titles）
    "当铺死当",
    "访主赎当·续",
    "物归旧主·续",
    "新掌接印",
    "旧掌传灯",
    // v1.16.15 扩池 9 条（156..=164）：题材避开既有全表名——伴随 3 名走生活流
    // （同/雪/昙花），常规 6 名走山野异象与民俗；与 build_event card.title 双源
    // 钉死（event_names_match_card_titles）
    "雪窖藏酿",
    "同守昙花",
    "雨后补瓦",
    "萤火引路",
    "山洪改道",
    "白鹿过涧",
    "月蚀之夜",
    "壶中天",
    "檐铃无风",
    // v1.16.16 扩池 5 条（165..=169）：结局回响 3 名走「世界侧写」体（之世/之野/
    // 之赠对仗结局名），书外之章联动 2 名走「天曹/书肆」余波体；题材字全表名
    // 零占用；与 build_event card.title 双源钉死（event_names_match_card_titles）
    "定稿之世",
    "留白之野",
    "执笔之赠",
    "天曹补写",
    "书肆新声",
    // v1.16.16 扩池 13 条（170..=182）：老年「交托回望」二名走炉火/灯影意象，
    // 开局生活流三名走渡口/村塾/早市，链14 段名「修伞记/伞骨重削·续/还伞·续」
    //（EVENT_NAMES 全表原无伞字名，本批首入），恶缘二名走沼瘴/丹市，高境二名
    // 走云阶/澜潮，伴随一名走霜夜补裘；与 build_event card.title 双源钉死
    //（event_names_match_card_titles）
    "炉暖旧器",
    "灯影授业",
    "义渡",
    "村塾借灯",
    "霜桥早市",
    "修伞记",
    "伞骨重削·续",
    "还伞·续",
    "沼瘴蚀符",
    "赝丹局",
    "云阶试步",
    "观澜悟潮",
    "霜夜补裘",
    // v1.16.17 扩池 10 条（183..=192）
    "灯下习字",
    "同参一炉",
    "云海携游",
    "仙曹索卷",
    "星槎访客",
    "灵税催帖",
    "旧履还坊",
    "雨夜寻印",
    "文狱退牒",
    "书僮窃墨",
    // v1.16.18 扩池 12 条（193..=204）
    "后辈证道",
    "冒籍疑云",
    "万卷尘页",
    "镜影忆战",
    "五域风谣",
    "窗前晒卷",
    "旧邻问名",
    "仙籍勾误",
    "坊契赖账",
    "夜叩柴门",
    "云海拾简",
    "同席说岁",
    // v1.16.19 扩池 14 条（205..=218）
    "观星问岁",
    "指间星图·续",
    "问岁之约·续",
    "剑冢阅武",
    "界碑拓铭",
    "雨檐听铃",
    "云上放灯",
    "相扶看雪",
    "仙池映世",
    "晚钟误点",
    "灶火留谱",
    "旧约转托",
    "邻人扛劫",
    "囊带乍断",
    // v1.16.20 扩池 12 条（219..=230）
    "剑穗生结",
    "剑穗缓结·续",
    "剑成赠穗·续",
    "星砂还账",
    "玉价新例",
    "坠星动冢",
    "同数流星",
    "霜晨共伞",
    "半檐躲雨",
    "池边洗鞘",
    "佃契抽成",
    "落款撞名",
    // v1.16.21 扩池 9 条（名表与 build_event 各臂 card.title 双源对账由
    // event_names_match_card_titles 全 id 单测钉死）
    "星槎再访",
    "星轨留书·续",
    "槎去留星·续",
    "旧砚问道",
    "影市黑秤",
    "云海拾阶",
    "剑赋催帖",
    "行会追赔",
    "共守岁灯",
    // v1.16.22 扩池 12 名（240..=251）
    "药圃赌芽",
    "踏云题字",
    "灵渠截流",
    "村谷抢收",
    "酒账千年",
    "滩头讹偿",
    "松风问琴",
    "雾锁仙市",
    "长老巡山",
    "万宝旧档",
    "云窟归卷",
    "补笔归卷·续",
    // v1.16.23 扩池 7 条（252..=258）：名表与卡臂 card.title 双源钉死，改字须同批
    "枰边手谈",
    "经阁理简",
    "旧败上门",
    "旧友茶叙",
    "冒名敛财",
    "灯市挑灯",
    "枯井鸣灵",
    // v1.16.24 扩池 12 名（259..=270）：名表与卡臂 card.title 双源钉死，改字须同批
    "市头新肆",
    "夜渡多收",
    "醉倒门前",
    "同研一笺",
    "旧穗试锋",
    "盘账夜灯",
    "信鸟投窗",
    "官牙抽引",
    "名册补名",
    "炉边说旧",
    "坊契转押",
    "窟灯未熄",
    // v1.16.25 扩池 10 名（271..=280）：名表与卡臂 card.title 双源钉死，改字须同批
    "柜前压价",
    "寒江共网",
    "扫阶拾规",
    "醉引歧路",
    "寄剑西窗",
    "岁星校图",
    "故市抬租",
    "岁宴万年",
    "空谷叩问",
    "堰口分水",
];

// v1.16.19 前段简随前期高频卡画面化同批改（id 0/1/3/4/5/6/12）：简仍「一行 ≤18 字」
// 预算（表注见上），由「场景+结果」记账体改画面半句——玩家开局必读的一页不再最平
pub const EVENT_BRIEF: [&str; 281] = [
    "满坡灵草露水未干，采满一筐",
    "林风忽静，獠牙自树影里扑出",
    "洞府禁制半塌，门缝里漏出宝光",
    "积灰储物袋或半价丹，摊前拾漏",
    "横剑拦道索买路财，战或破财",
    "山亭避雨逢道友，一壶茶论道",
    "执事拦下端详半晌，问可愿入门",
    "旧执化魔袭扰识海，澄心自镇",
    "千年灵乳将成，护宝妖兽环伺",
    "泉鸣如琴的洞府，灵气浓得化不开",
    "苍老声音落进识海：「吾道后继有人」",
    "空间裂缝露秘境入口，深入或拾遗",
    "下山入世，烟火气里磨道心",
    "灵潮灌顶，吐纳跟着浪头走",
    "黑影自暗中暴起，招招夺命",
    "山崩石开，遗府门里透出宝光",
    "两妖咬作一团，温润妖丹滚落脚边",
    "蒙面人压低嗓门：洗髓丹，过村无店",
    "白眉老者点三处，旧稿字字换意",
    "雪白小兽赶不走，一路跟着你",
    "云端白须仙人含笑：要哪般？",
    "入泉三日，出水时旧伤尽褪",
    "十万锈剑古冢，中央古剑择主",
    "白骨魔窟，宝光与魔气交织",
    "焦黑古木雷光未散，拾回炼丹",
    "商队车圈成阵，管事嘶声求援",
    "鹤影过顶，掷下锦盒便没入云",
    "瞎眼卦师掐指：是福是祸一念间",
    "晒书日递来半页残稿，缝作同页",
    "大比开擂，掌门亲口：魁主任拣一卷",
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
    "道场积尘三寸，蒲团上坐痕犹温",
    "沙暴裸出藏窟，斩链或只探外洞",
    "货郎赊墨引灵，赊或不买皆有后话",
    "数年后货郎赠旧罗盘，结个善缘",
    "拍门声里站着原主，刀先出了鞘",
    "无字古碑洇出半字，拓或掩",
    "装裱老裱工示警，焚拓或见守碑人",
    "守碑人拦路三问，答错留拓",
    "江底铜匣錾「沉墨候主」，收或还",
    "老渔翁识匣来历，随往江心或辞谢",
    "匣底真迹枯润相生，参悟或转赠",
    "道侣发问：道途尽头是什么模样",
    "道侣赠剑「同舟」，佩或推辞",
    "涧道遇袭，并肩接敌或推人先退",
    "追到坑口，陨铁未冷，绢书静卧",
    "无面镜客索一赌，赌命或碎镜",
    "灵潮倒卷洞府，探漩心或引气归己",
    "蒙学书院讲经，坐听或捐笔",
    "崖壁灵蜜凝花髓，买蜜或护巢",
    "云游画师白描小像，入画或赠银",
    "雪线断裂积雪倾泻，开道或绕行",
    "地下斗擂押性命，登台或押注",
    "漕船缺护道，伴航或夜读",
    "野温泉化筋骨滞涩，泡浴或掘石",
    "一局棋下了三年，观棋或投子",
    "炉火整夜不熄，锤声比雨声密",
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
    "火里稿纸卷三卷不化，灰烬让出小字",
    "埋笔成冢夜有墨香，祭或掘",
    "河床干了三年，村民争最后一线活泉",
    "断线纸鸢带血书，循线或焚鸢",
    "砚形山叩之出泉墨，取泉或拓痕",
    "废戍楼夜半点兵声，入阵或绕行",
    "万茧悬室一茧有字，选茧或全买",
    "无人官船拦江，探舱或助推",
    "墨鲸浮水背上有城，登城或远观",
    "山下散修顶着你的名号开坛，徒众不少",
    "夺积之后宿敌倾巢寻账，据守或散财",
    "旧册无风自动，血仇名上新墨",
    "雾河渡船人只收旧渡资，讲旧事或付石",
    "灯纸上有你未写过的字迹，追问或登岸",
    "灯上尽是天曹删掉的名字，点灯或携灯",
    "荒废丹房炉火复燃，守炉童子求援",
    "炉里炼的是烧残稿，问下落或添柴",
    "炉开无丹只有一页完稿，收稿或还炉",
    // v1.16.11 扩池 16 条（118..=133）
    "血册残页钉门，应战或焚册",
    "缟素七人雪夜拦道递战书",
    "崖底风雪如幡，一剑或一册了局",
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
    "名下聚灵阵满级，阵眼应灵潮而鸣",
    "名下灵田穗沉，售谷或开仓施粥",
    "名下丹炉开鼎，遗香三日凝不散",
    // v1.16.13 扩池 14 条（137..=150）：简 ≤18 字一行预算（手札行布局，
    // event_names_match_card_titles 钉死；实测全表最长 18 字）
    "邻山劫云垂而未落，茶棚照常添水",
    "雷火焦痕拾级，访劫者或拾矿壳",
    "茶烟未散，渡劫客只谈侥幸",
    "书肆挂出易主木牌，旧抄本一摞摞装箱",
    "绸商再托提要，提笔或留白",
    "肆后添了书案，坐案或荐人",
    "公道秤挂集口，称物也称心",
    "蚁群渡溪桥断，搭叶或灵渡",
    "三百年前贡桃冻于窖底，食或还",
    "水磨坊夜自转磨月光，守或问",
    "货栈灵鼠偷丹推你脚边，纵或捉",
    "南崖有人采薇而歌，坐听或分粮",
    "旧鞍夹层掉出无火漆的信，读或买鞍",
    "雪原一行脚印有去无回，循或拂平",
    // v1.16.14 扩池 5 条（151..=155）：简 ≤18 字一行预算（手札行布局，
    // event_names_match_card_titles 钉死；实测全表最长 18 字）
    "死当架旧箫刻归字，问或离去",
    "循存根寻到药庐，垫资赎箫或止步",
    "旧箫归架药香淡，听曲或收灵茶",
    "接印大典彩头满案，搭手或观礼",
    "末讲不授功法，一句「接印容易守印难」",
    // v1.16.15 扩池 9 条（156..=164）：简 ≤18 字一行预算（手札行布局，
    // event_names_match_card_titles 钉死；实测全表最长 18 字）
    "雪夜同埋一坛酒，约功成之日",
    "昙花一现只一夜，守花或先眠",
    "漏雨屋檐两人补，递瓦或熔凝",
    "夜谷萤火成行引路，循光或让行",
    "山洪直冲村落，分洪或帮抢收",
    "白鹿踏雪过涧生青草，循迹或长揖",
    "天狗蚀月锣声四起，救月或静观",
    "壶中有山瀑声不绝，养壶或转赠",
    "檐铃无风自响报归人，擦铃或听夜",
    // v1.16.16 扩池 5 条（165..=169）：简 ≤18 字一行预算（手札行布局；实测全表最长 18 字）
    "茶馆醒木一拍：这名字一字改不得",
    "官册空着一格的野地，草高过批注",
    "废稿摊有人替磨没的名字描清",
    "天曹卷宗页脚多出一行新墨补注",
    "书肆新到无脊之书，有缘翻开哪页读哪页",
    // v1.16.16 扩池 13 条（170..=182）：简 ≤18 字一行预算（手札行布局；实测全表最长 18 字）
    "炉火煨着旧物，老匠托你捎封回信",
    "油灯将尽的老屋，白发先生授半课",
    "老船工义渡，搭手压船或添香资",
    "村塾灯油见底，打油或留灯",
    "霜桥早市人声初起，走一遭或分姜茶",
    "雨夜伞铺，老匠只修凡伞不修法器",
    "老匠验出伞骨异样，重削或照旧",
    "伞成归还，另赠新削伞骨一支",
    "瘴气如粥翻涌，护身符纸一夜蚀出斑点",
    "锦衣人低声兜售九转遗丹，丹香过火",
    "生花境云阶千级，拾级或阶下观人",
    "崖上观澜，随潮入定或记潮作谱",
    "霜夜露重，与道侣灯下补一件旧裘",
    // v1.16.17 扩池 10 条（183..=192）：简 ≤18 字一行预算（手札行布局；实测全表最长 18 字）
    "灯下与道侣同临一页字帖",
    "一图两人同参，呼吸渐渐合拍",
    "道侣携你御风云海，俯瞰山河",
    "仙曹踏云索卷，补墨或力争",
    "星槎泊云讨茶，指路或同游",
    "催税帖贴上门楣，朱印不容分说",
    "旧履寻老匠换底，半世路在脚底",
    "雨夜乱葬岗，无字方印半埋土中",
    "文书吏代文狱问一句：牒压手么",
    "小书僮偷墨抄字帖，赠或罚研墨",
    // v1.16.18 扩池 12 条（193..=204）
    "后辈摆证道台，请最老的前辈观礼",
    "仙籍上另有同名同貌者，在北域行医济世",
    "藏经阁尘页自翻，你的批注未褪",
    "轮回镜自亮，重播你的第一场胜仗",
    "行脚客收五域风谣，独缺你这一域",
    "好日头晒卷，纸页翻出旧年声音",
    "旧邻孩童问：名字是娘取的还是自己取的",
    "仙籍勾错寿数，更正一笔先收作保",
    "老契压在手里，账却没人认了",
    "三更叩门声急，门外人影带着刀",
    "与道侣踏云归途，云海浮出半句残简",
    "炉火将尽，与道侣同席数算走过的年岁",
    // v1.16.19 扩池 14 条（205..=218）：简一行 ≤18 字预算（手札行布局）照守
    "醉道人拉你观星，指星要对账",
    "酒渍星图缺一笔，补或留缺",
    "星挪一格，来年之约或收星砂",
    "万剑归冢夜夜自阅武，召你试剑",
    "阴雨洇了碑上名讳，拓碑吏催着重铭",
    "雨夜檐下，听旧年同挂的那只铃",
    "云海之上放一盏灯，看它穿几层云",
    "两位老人相扶看雪，落雪无声",
    "仙池映出你第一世的脸，坐或汲",
    "晚钟多敲一记，满城疑心自扰",
    "一辈子的火候，留不留山下灶口",
    "旧笺压箱底多年，代赴或亲往",
    "邻山有人正扛他的九重雷劫",
    "行囊带断口，灵石滚落山道",
    // v1.16.20 扩池 12 条（219..=230）：简一行 ≤18 字预算（手札行布局）照守
    "宁清霜的剑穗磨断了，求一个慢结",
    "丝线里缠着一缕师尊留的旧穗",
    "穗成之日，旧丝分一段护你",
    "醉道人携星砂来还一场旧账",
    "玉匠拦路：工钱按玉价新例",
    "坠星过顶，万剑低鸣如向天揖让",
    "一人一道流星，并肩数完整场",
    "伞面不大，让出去的半边最暖",
    "半边檐下两人一雨，谁也没说话",
    "池底洗鞘，旧痕浮出当年一击",
    "佃户捧契求减，契在减不减依你",
    "无名书上落着你的名字",
    // v1.16.21 扩池 9 条（231..=239）：简一行 ≤18 字预算照守
    "星槎再泊，故客问路可曾有名",
    "槎心星轨缺一笔，补或留缺",
    "槎去云开，半幅星轨或换星砂",
    "旧砚无墨自润，叩砚试道心",
    "黑秤匠替你称完行囊，秤星在他袖里",
    "云海退处露旧阶，拾级或坐观",
    "岁帖贴上剑鞘：岁纳剑赋，或以剑代",
    "单子出了损，三年前的手印找上门",
    "岁暮与道侣共守一盏岁灯",
    // v1.16.22 扩池 12 简（240..=251，≤18 字一行预算随批钉死）
    "两畦药苗赌先后，一夜齐芽",
    "云海当纸，与道侣各题一字",
    "新渠按契分水，旧约压在渠主案头",
    "谷雨将至，替凡人独户抢收",
    "千年酒账押着一个「醉」字",
    "滩头有人讹偿，围观者越围越多",
    "无弦琴自鸣，刻字「答者留琴」",
    "雾锁江三日，雾开处浮出街市",
    "玄诚长老巡山，半页讲义指暗耗",
    "万宝楼翻出旧档，压款整三年",
    "崖窟旧籍风蚀，抄一卷是一卷",
    "百卷上新纸，老人请你留名",
    // v1.16.23 扩池 7 条（252..=258）：简 ≤18 字一行预算（照 240..=251 批口径）
    "宁清霜手谈半局，败亦有所得",
    "经阁拓本理册，残卷半页结缘",
    "旧败者携帮手寻上门，讨要颜面",
    "百年旧友亭中茶叙，不提岁数",
    "有人冒名敛财，你留真话擦假名",
    "灯市挑得一盏走马灯，共提而归",
    "枯井夜漏月色灵气，井底压着旧府基",
    // v1.16.24 扩池 12 条简（259..=270）：简 ≤18 字一行预算（照 252..=258 批口径）
    "市口新肆开张，头秤分毫不差",
    "夜半渡口，摆渡人停船坐地起价",
    "醉道人横睡门槛，扶他起来还酒账",
    "同研一池墨，两半笺渐合拍",
    "旧剑系旧穗，借你的手试锋",
    "年终盘账，灯下拨响旧算珠",
    "青羽信鸟投窗，衔来一封无名信",
    "官牙新章墨未干，抽引一成",
    "名册空了一格，替长老补名",
    "围炉温酒，两段旧事说与炭火听",
    "坊契被中人私押，铺子易手",
    "重过高崖，旧窟盏灯还亮着",
    // v1.16.25 扩池 10 条简（271..=280）：简 ≤18 字一行预算（照 252..=258 批口径）
    "万宝楼伙计错认赃物，柜前对质洗名",
    "寒江同船收网，两桨分雪共一篓",
    "领外门洒扫，阶缝里扫出半页旧规",
    "醉话指近道，雾里绕到天明",
    "宁清霜远行，托剑挂你西窗晨昏拭锋",
    "万年星图缺一笔，轮到你校补",
    "故市牙行连年抬租，旧铺难租",
    "万年岁宴开席，赴宴并载一笔",
    "空谷传回一句问话，等你作答",
    "替凡人独户修堰分水，半日活计",
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
    "灵雾漫过膝弯，你吐纳一夜，起身时袖口凝着露水。",
    // v1.16.20 画面化 2 条（模拟日志 8%/年高频杂事线，只改字不加条不加段）：
    // 结论词「悟道/有感触」改动作与画面，书稿母题收尾
    "瀑布落在肩头，水声里听出功法的调子。",
    "月圆之夜，星子排成一行未写的稿。",
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

// v1.16.25 天象流转行画面化：天气变动帧的日志由「天象流转：{名}。」记账腔改
// 五天气各一句画面短句（有场景有实物，静态查表零 format 机制；与场景行
// 「地名 · 天时」/event_brief_11625 批同画面体纪律）。表长与 WEATHERS 等长
// 由 game.rs weather_flavor_aligned 单测钉死防漂移
pub const WEATHER_FLAVOR: [&str; 5] = [
    "晴空晒暖檐角，纸墨干得快。",
    "云影过窗，砚池凉了半分。",
    "雨连下三日，檐水滴成一线。",
    "雾锁山路，脚步放轻半寸。",
    "灵潮涨了，泉鸣跟着浪头走。",
];

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

// ---------------- v1.16.23 减包刀②：事件臂文案资产化（textpack）----------------
// build_event 臂内 >=24B 中文常量串由 tools/pack_text.py 收编进 assets/textpack.bin
// （raw-deflate，与 assets/font.ttf.deflate 同款通路），源码侧为 evtx(N)/* 原文：… */
// （原文注释回填保 diff 可审计）；真相源 assets/text_source.txt（N<TAB>源码转义形态），
// 文案编辑改 source 后重跑 pack_text.py 即可，旧 N 永不变号；块内新写的裸字面量
// 功能照常（只是未压缩），重跑脚本自动收编（append 新 N）。
// boot 一次性 inflate（main.rs 启动序列，数十 KB 量级随收编增长，miniz inflate
// <5ms）；evtx() 为
// O(1) 下标切片，帧路径零分配（性能十戒）；文案内容逐字节等价（脚本往返断言），
// 字体 cmap 与 charset_diff_check 口径不变；存档只存事件 id，零兼容影响。
pub static TEXTPACK: &[u8] = include_bytes!("../assets/textpack.bin");
// 生成件（pack_text.py 产出）：完整 pub static TXT_OFFS: [u32; 648] = [...]; 声明，
// item 位置 include! 展开（随收编追加自动重产，勿手改；v1.16.23 注 577、
// v1.16.25 注 623 均已过期清账——槽数=条数+1，随收编增长）
include!("text_offsets.inc");

static TEXT_BLOB: std::sync::OnceLock<&'static [u8]> = std::sync::OnceLock::new();

/// 解包单源（get_or_init 的初始化闭包）：成功返回 leak 后的 blob，失败/校验不
/// 过返回 EMPTY 哨兵——INIT_TRIED 钉死「只试一次」，失败后不再重试（防帧路径
/// 反复 inflate）。EMPTY 即失败态（真实 blob 恒非空：647 条非空文案串接）。
static TEXT_INIT_TRIED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn unpack_text() -> &'static [u8] {
    use std::sync::atomic::Ordering;
    static EMPTY: &[u8] = &[];
    if TEXT_INIT_TRIED.swap(true, Ordering::Relaxed) {
        return EMPTY;
    }
    match miniz_oxide::inflate::decompress_to_vec(TEXTPACK) {
        Ok(v) => {
            // from_utf8 全量校验一次：之后 evtx() 的 unchecked 切片以此为安全论证
            if core::str::from_utf8(&v).is_ok() {
                Box::leak(v.into_boxed_slice()) as &'static [u8]
            } else {
                EMPTY
            }
        }
        Err(_) => EMPTY,
    }
}

/// boot 一次性解包（main.rs 启动序列调用；幂等）。get_or_init 并发安全：
/// 并发首调阻塞等待完成，其余调用直达已装快路径。解压失败/UTF-8 校验失败
/// 时不装 blob——evtx() 全返空串（文案缺失但不崩，wasm trap 纪律）。
pub fn init_text() {
    TEXT_BLOB.get_or_init(unpack_text);
}

/// O(1) 查池：blob[offsets[i]..offsets[i+1]]。越界/未初始化一律空串兜底
/// （绝不回错文案；i 由脚本分配的编译期常量，正常路径恒界内）。
/// 惰性兜底单源：漏调 init_text 的环境（cargo test 等）首调自动解包——
/// get_or_init 的 once 语义保证并发查池者阻塞至完成（多线程测试不竞态空串）；
/// 正常路径（boot 已装）等价一次原子 load，帧路径零影响。
pub fn evtx(i: usize) -> &'static str {
    let blob = TEXT_BLOB.get_or_init(unpack_text);
    if blob.is_empty() {
        return "";
    }
    // 防御边界：i >= len-1 即空（i+1 需界内）；写 saturating_sub 令 usize::MAX
    // 这类极端值不触发加法溢出（dev profile 溢出即 panic——panic=abort 纪律）
    if i >= TXT_OFFS.len().saturating_sub(1) {
        return "";
    }
    let s = TXT_OFFS[i] as usize;
    let e = TXT_OFFS[i + 1] as usize;
    if s <= e && e <= blob.len() {
        // 安全论证：TEXT_BLOB 只在 from_utf8 全量通过后装入，切片必为合法 UTF-8
        unsafe { core::str::from_utf8_unchecked(&blob[s..e]) }
    } else {
        ""
    }
}

#[cfg(test)]
mod text_tests {
    use super::*;

    // v1.16.23 资产化等价性样例断言（pack_text.py 往返断言之外的运行期抽查；
    // 样例 N 取自 text_source.txt 首批三行，文案改版须同步此三行）
    #[test]
    fn textpack_inflate_and_spot_check() {
        init_text();
        assert_eq!(evtx(0), "你婉拒道：「一页未成，何以同页。」道心+5");
        assert_eq!(
            evtx(1),
            "十年一度的宗门大比开擂，掌门放话：魁首可入藏经阁任拣一卷功法！"
        );
        assert_eq!(evtx(2), "你于台下观摩他人过招，亦有所得。修为+5%");
        // 全表界内：offsets 单调且末界 == blob 长
        assert!(TXT_OFFS.len() >= 2);
        for k in 0..TXT_OFFS.len() - 1 {
            assert!(TXT_OFFS[k] <= TXT_OFFS[k + 1], "offsets[{}] not monotonic", k);
        }
        assert_eq!(
            TXT_OFFS[TXT_OFFS.len() - 1] as usize,
            TEXT_BLOB.get().copied().map_or(0, |b| b.len())
        );
        // 越界防御：空串兜底（不 panic）
        assert_eq!(evtx(TXT_OFFS.len()), "");
        assert_eq!(evtx(usize::MAX), "");
    }
}
