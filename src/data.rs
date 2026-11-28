// 数据表：境界、功法、法宝、丹药、天赋、轮回升级、成就、出身、事件权重
// 全部数值集中在此，便于平衡调整。

// v1.16.40 f64→f32 留档组盘点判定：不执行，永久关闭——本表/Eff 全系 f64 数据模型
// 深根（data.rs 57 + game.rs 86 处字段与算链），path_hooks_exact_multipliers 钉
// 1e-9 容差 f32 必破；重标精度契约需 150+ 处手术，而理论 ~556B 已被 v1.16.39
// 整数表/臂表实测（+153B）证伪同口径——亚 KB 收益 < 手术风险，判定不动作。
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
pub const MILESTONE: [i32; 15] = [
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
// v1.16.32 单1 主刀第三批：name/desc 收编 textpack（field_n 显式 N；真相源
// text_source.txt 行 + tables2 等价钉测）——访问器 O(1) 纯算术零分配（evtx 同款），
// 调用点 .name→.name() 编译器逐站收口（漏改即编译错，安全失败模式）。
pub struct TechDef {
    pub name_n: u16,
    pub mult: f64,
    pub pow: f64,
    pub desc_n: u16,
}

impl TechDef {
    pub fn name(&self) -> &'static str {
        evtx(self.name_n as usize)
    }
    pub fn desc(&self) -> &'static str {
        evtx(self.desc_n as usize)
    }
}
pub const TECHS: [TechDef; 8] = [
    TechDef {
        name_n: 2007,
        mult: 1.0,
        pow: 1.0,
        desc_n: 2015,
    },
    TechDef {
        name_n: 2008,
        mult: 1.3,
        pow: 1.0,
        desc_n: 2016,
    },
    TechDef {
        name_n: 2009,
        mult: 1.6,
        pow: 1.0,
        desc_n: 2017,
    },
    TechDef {
        name_n: 2010,
        mult: 2.0,
        pow: 1.2,
        desc_n: 2018,
    },
    TechDef {
        name_n: 2011,
        mult: 2.5,
        pow: 1.1,
        desc_n: 2019,
    },
    TechDef {
        name_n: 2012,
        mult: 3.1,
        pow: 1.3,
        desc_n: 2020,
    },
    TechDef {
        name_n: 2013,
        mult: 3.8,
        pow: 1.15,
        desc_n: 2021,
    },
    TechDef {
        name_n: 2014,
        mult: 4.8,
        pow: 1.4,
        desc_n: 2022,
    },
];

// ---------------- 法宝 ----------------
// v1.16.32 单1 主刀第三批：name/desc 收编 textpack（同 TechDef 注）
pub struct RelicDef {
    pub name_n: u16,
    pub pow: f64,
    pub cult: f64,
    pub desc_n: u16,
    /// v1.7.0 献祭折算的模拟点数（永久失去法宝；按 pow+cult 强弱定价）
    pub pt: i32,
}

impl RelicDef {
    pub fn name(&self) -> &'static str {
        evtx(self.name_n as usize)
    }
    pub fn desc(&self) -> &'static str {
        evtx(self.desc_n as usize)
    }
}
pub const RELICS: [RelicDef; 8] = [
    RelicDef {
        name_n: 2023,
        pow: 0.15,
        cult: 1.0,
        desc_n: 2031,
        pt: 3,
    },
    RelicDef {
        name_n: 2024,
        pow: 0.25,
        cult: 1.0,
        desc_n: 2032,
        pt: 4,
    },
    RelicDef {
        name_n: 2025,
        pow: 0.0,
        cult: 1.25,
        desc_n: 2033,
        pt: 4,
    },
    RelicDef {
        name_n: 2026,
        pow: 0.40,
        cult: 1.0,
        desc_n: 2034,
        pt: 6,
    },
    RelicDef {
        name_n: 2027,
        pow: 0.0,
        cult: 1.40,
        desc_n: 2035,
        pt: 7,
    },
    RelicDef {
        name_n: 2028,
        pow: 0.60,
        cult: 1.0,
        desc_n: 2036,
        pt: 9,
    },
    RelicDef {
        name_n: 2029,
        pow: 0.90,
        cult: 1.0,
        desc_n: 2037,
        pt: 12,
    },
    RelicDef {
        name_n: 2030,
        pow: 1.50,
        cult: 1.30,
        desc_n: 2038,
        pt: 15,
    },
];

// ---------------- v1.7.0 点数经济：死档广告 ----------------
/// 看完一次幻境广告获得的模拟点数
pub const AD_REWARD: i32 = 2;
/// 幻境广告时长（真实秒）
pub const AD_SECONDS: f32 = 5.0;
/// 轮回商阁假广告创意（按已看次数轮换）
// v1.16.32 单1 小表批：表体收编 textpack（TXT_ADCREA_BASE 起 3 条行主序连号，
// 基址见 text_offsets.inc；pack_text.py --tables2 ARR 通道产侧断言对账）——
// 查表 fn 单口 O(1) 零分配；行内容与旧字面量逐字节等价（真相源行 + tables2
// 等价钉测）。
pub fn ad_creative(k: usize) -> &'static str {
    evtx(TXT_ADCREA_BASE as usize + k.min(2))
}

// ---------------- 丹药 ----------------
// v1.16.32 单1 主刀第三批：name/desc 收编 textpack（同 TechDef 注）
pub struct PillDef {
    pub name_n: u16,
    pub desc_n: u16,
    pub edible: bool,
}

impl PillDef {
    pub fn name(&self) -> &'static str {
        evtx(self.name_n as usize)
    }
    pub fn desc(&self) -> &'static str {
        evtx(self.desc_n as usize)
    }
}
// v1.16.39 整数表第三轮①：i32→i16（值域 200..=5000 ≤ i16::MAX=32767；唯一读点
// game.rs pill_price 已是 `as f64` 扩位天然兼容；理论 −12B 以打包实测为准）
pub const PILL_PRICES: [i16; 6] = [200, 1500, 2500, 800, 5000, 4000];
pub const PILLS: [PillDef; 6] = [
    PillDef {
        name_n: 2039,
        desc_n: 2045,
        edible: true,
    },
    PillDef {
        name_n: 2040,
        desc_n: 2046,
        edible: false,
    },
    PillDef {
        name_n: 2041,
        desc_n: 2047,
        edible: false,
    },
    PillDef {
        name_n: 2042,
        desc_n: 2048,
        edible: true,
    },
    PillDef {
        name_n: 2043,
        desc_n: 2049,
        edible: true,
    },
    PillDef {
        name_n: 2044,
        desc_n: 2050,
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
    // v1.16.32 单1：Option<usize>(16B×2)→Option<u8>(2B×2) 位宽窄化——值域
    // 0..=7 恒界内（TALENTS 三行 Some 恒 < 8），读写点仅 game.rs teff() 合并
    // 拷贝（TEff→TEff 同型零转换）；33 行 TALENTS rdata 每行省 24B。语义零变化
    //（含 teff() 结果 relic/tech 现无消费面的现状行为——设计事实原样保留）。
    pub relic: Option<u8>,
    pub tech: Option<u8>,
}

pub struct TalentDef {
    pub name_n: u16,
    pub desc_n: u16,
    pub tier: u8,
    pub e: TEff,
}

impl TalentDef {
    pub fn name(&self) -> &'static str {
        evtx(self.name_n as usize)
    }
    pub fn desc(&self) -> &'static str {
        evtx(self.desc_n as usize)
    }
}

pub const TALENTS: [TalentDef; 33] = [
    TalentDef {
        name_n: 2051,
        desc_n: 2084,
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
        name_n: 2052,
        desc_n: 2085,
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
        name_n: 2053,
        desc_n: 2086,
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
        name_n: 2054,
        desc_n: 2087,
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
        name_n: 2055,
        desc_n: 2088,
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
        name_n: 2056,
        desc_n: 2089,
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
        name_n: 2057,
        desc_n: 2090,
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
        name_n: 2058,
        desc_n: 2091,
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
        name_n: 2059,
        desc_n: 2092,
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
        name_n: 2060,
        desc_n: 2093,
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
        name_n: 2061,
        desc_n: 2094,
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
        name_n: 2062,
        desc_n: 2095,
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
        name_n: 2063,
        desc_n: 2096,
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
        name_n: 2064,
        desc_n: 2097,
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
        name_n: 2065,
        desc_n: 2098,
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
        name_n: 2066,
        desc_n: 2099,
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
        name_n: 2067,
        desc_n: 2100,
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
        name_n: 2068,
        desc_n: 2101,
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
        name_n: 2069,
        desc_n: 2102,
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
        name_n: 2070,
        desc_n: 2103,
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
        name_n: 2071,
        desc_n: 2104,
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
        name_n: 2072,
        desc_n: 2105,
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
        name_n: 2073,
        desc_n: 2106,
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
        name_n: 2074,
        desc_n: 2107,
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
        name_n: 2075,
        desc_n: 2108,
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
        name_n: 2076,
        desc_n: 2109,
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
        name_n: 2077,
        desc_n: 2110,
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
        name_n: 2078,
        desc_n: 2111,
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
        name_n: 2079,
        desc_n: 2112,
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
        name_n: 2080,
        desc_n: 2113,
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
        name_n: 2081,
        desc_n: 2114,
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
        name_n: 2082,
        desc_n: 2115,
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
        name_n: 2083,
        desc_n: 2116,
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
    pub name_n: u16,
    pub desc_n: u16,
    pub max: i32,
    pub base: i64,
    pub growth: f64,
}

impl UpgDef {
    pub fn name(&self) -> &'static str {
        evtx(self.name_n as usize)
    }
    pub fn desc(&self) -> &'static str {
        evtx(self.desc_n as usize)
    }
}
pub const UPGRADES: [UpgDef; 10] = [
    UpgDef {
        name_n: 2117,
        desc_n: 2127,
        max: 6,
        base: 25,
        growth: 1.7,
    },
    UpgDef {
        name_n: 2118,
        desc_n: 2128,
        max: 6,
        base: 25,
        growth: 1.7,
    },
    UpgDef {
        name_n: 2119,
        desc_n: 2129,
        max: 6,
        base: 25,
        growth: 1.7,
    },
    UpgDef {
        name_n: 2120,
        desc_n: 2130,
        max: 6,
        base: 25,
        growth: 1.7,
    },
    UpgDef {
        name_n: 2121,
        desc_n: 2131,
        max: 1,
        base: 1200,
        growth: 1.0,
    },
    UpgDef {
        name_n: 2122,
        desc_n: 2132,
        max: 1,
        base: 800,
        growth: 1.0,
    },
    UpgDef {
        name_n: 2123,
        desc_n: 2133,
        max: 3,
        base: 60,
        growth: 2.5,
    },
    UpgDef {
        name_n: 2124,
        desc_n: 2134,
        max: 5,
        base: 40,
        growth: 2.0,
    },
    UpgDef {
        name_n: 2125,
        desc_n: 2135,
        max: 5,
        base: 50,
        growth: 2.0,
    },
    UpgDef {
        name_n: 2126,
        desc_n: 2136,
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
    pub name_n: u16,
    pub desc_n: u16,
    // v1.16.32 单1：i64→i32 位宽窄化（奖励值域 10..=200_000 恒界内；行尺寸
    // 16B→8B×30 行，rdata −240B 纯等价——消费点 game.rs 达成发放处 as i64 显式
    // 升位，算术语义零变化）
    pub reward: i32,
}

impl AchDef {
    pub fn name(&self) -> &'static str {
        evtx(self.name_n as usize)
    }
    pub fn desc(&self) -> &'static str {
        evtx(self.desc_n as usize)
    }
}
pub const ACHS: [AchDef; 30] = [
    AchDef {
        name_n: 2137,
        desc_n: 2167,
        reward: 10,
    },
    AchDef {
        name_n: 2138,
        desc_n: 2168,
        reward: 30,
    },
    AchDef {
        name_n: 2139,
        desc_n: 2169,
        reward: 80,
    },
    AchDef {
        name_n: 2140,
        desc_n: 2170,
        reward: 200,
    },
    AchDef {
        name_n: 2141,
        desc_n: 2171,
        reward: 500,
    },
    AchDef {
        name_n: 2142,
        desc_n: 2172,
        reward: 1200,
    },
    AchDef {
        name_n: 2143,
        desc_n: 2173,
        reward: 3000,
    },
    AchDef {
        name_n: 2144,
        desc_n: 2174,
        reward: 8000,
    },
    AchDef {
        name_n: 2145,
        desc_n: 2175,
        reward: 20000,
    },
    AchDef {
        name_n: 2146,
        desc_n: 2176,
        reward: 50000,
    },
    AchDef {
        name_n: 2147,
        desc_n: 2177,
        reward: 200000,
    },
    AchDef {
        name_n: 2148,
        desc_n: 2178,
        reward: 1000000,
    },
    AchDef {
        name_n: 2149,
        desc_n: 2179,
        reward: 2000,
    },
    AchDef {
        name_n: 2150,
        desc_n: 2180,
        reward: 300,
    },
    AchDef {
        name_n: 2151,
        desc_n: 2181,
        reward: 30000,
    },
    AchDef {
        name_n: 2152,
        desc_n: 2182,
        reward: 200,
    },
    AchDef {
        name_n: 2153,
        desc_n: 2183,
        reward: 5000,
    },
    AchDef {
        name_n: 2154,
        desc_n: 2184,
        reward: 300,
    },
    AchDef {
        name_n: 2155,
        desc_n: 2185,
        reward: 400,
    },
    AchDef {
        name_n: 2156,
        desc_n: 2186,
        reward: 80,
    },
    AchDef {
        name_n: 2157,
        desc_n: 2187,
        reward: 150,
    },
    AchDef {
        name_n: 2158,
        desc_n: 2188,
        reward: 500,
    },
    AchDef {
        name_n: 2159,
        desc_n: 2189,
        reward: 200,
    },
    AchDef {
        name_n: 2160,
        desc_n: 2190,
        reward: 150,
    },
    AchDef {
        name_n: 2161,
        desc_n: 2191,
        reward: 100,
    },
    AchDef {
        name_n: 2162,
        desc_n: 2192,
        reward: 200,
    },
    AchDef {
        name_n: 2163,
        desc_n: 2193,
        reward: 500,
    },
    AchDef {
        name_n: 2164,
        desc_n: 2194,
        reward: 100000,
    },
    AchDef {
        name_n: 2165,
        desc_n: 2195,
        reward: 2000,
    },
    AchDef {
        name_n: 2166,
        desc_n: 2196,
        reward: 1000,
    },
];

// ---------------- v1.16.8 章节书：主线八章（v1.16.10 增第九章·书外之章）----------------
// 章节定义静态表（照 ACHS/EVENTS 惯例）：名/引入剧情/目标清单。达成谓词不在本表——
// 全部收敛为 game.rs chapter_goal_met 单口（pid 分发，照 check_ach_all 的 (id,bool)
// 元组表先例），禁第二实现。所有谓词只读 meta 标量（不含 hist/story 等 String 字段），
// chapter_done_mask 可在存档载入帧与每帧章节视图安全调用，零分配。
// 达成持久 = meta.chap_done 位掩码（键 ch=，缺键→0 零迁移，save.rs 回环单测）。
// v1.16.32 单1 主刀第三批：goal text 收编 textpack（text_n 显式 N，pid 谓词位
// 保留原语义）——访问器同款 O(1) 零分配
pub struct ChapterGoal {
    pub text_n: u16, // 目标文案（章节书目标行）——textpack N
    pub pid: u8,            // 谓词 id（game.rs chapter_goal_met 单口 match 分发表）
}

impl ChapterGoal {
    pub fn text(&self) -> &'static str {
        evtx(self.text_n as usize)
    }
}

pub struct ChapterDef {
    // v1.16.32 单1 主刀第三批：num/name 收编 textpack（num_n/name_n 显式 N；
    // 旧「章次字面量禁运行时拼数字」纪律照旧——N 即真相源行号，非运行时拼接）
    pub num_n: u16,   // 章次字面量（"第一章"，禁运行时拼数字）
    pub name_n: u16,  // 章名（横幅/章节书行）
    pub intro_n: u16, // 引入剧情（章节书右栏，wrap_draw 单段）——v1.16.29 刀B 收编 textpack
    pub goals: &'static [ChapterGoal],
    // v1.16.9 章成之礼：达成边沿由 game.rs check_chapters 唯一发放（newly 位命中处
    // meta.dao_yun += reward，chap_done 位即领取凭证，零新存档键零迁移）。奖励行
    // 显示走 reward_note 全静态字面量（含「道韵 +N」），帧路径零分配；reward 数字
    // 与 note 文本的对应由 chapter_tests 表一致性单测钉死，防两处手改漂移。
    pub reward: u32,          // 章成之礼道韵（meta 跨世持久标量；不发灵石等现世资源）
    pub reward_note_n: u16, // 奖励行文案（章节书右栏 + 达成金行 log 复用）——刀B 收编
    // v1.16.9 章间记事：逐段解锁的章内插曲（段 i 的可见条件=本章已毕目标数 ≥ i，
    // 右栏只显示最新解锁段——纯显示推导零存档；段首自带「其一/其二」序号，全静态）。
    // v1.16.29 刀B 章间记事收编 textpack：lore_ns=本章节段首 N（行主序连号）、
    // lore_len=段数（2..=3 变长）——访问器 lore(k) 越界空串兜底（evtx 同款防御）。
    pub lore_ns: u16,
    pub lore_len: u8,
    // v1.16.20 章开场白（故事沉浸续作）：章 i 了边沿由 game.rs check_chapters 唯一
    // 追发下一章「【第X章·启】<open>」金行日志（章1 开局已有 intro、无前边沿不
    // 发；末章了后无下一章不发——首行 open 恒不演出，保非空供表契约单测钉死）。
    // 显示位=日志金行单行，预算 ≤24 字；不进章节书右栏（intro 预算不动）。
    pub open_n: u16,
    // v1.16.28 章回联句（故事沉浸续作②）：每章一句对仗联语（上下联以逗号分，
    // 单行 ≤14 字预算）。显示位两处——章节书右栏章名下（draw.rs draw_chapter_view）
    // 与结算屏「下一章」行尾（draw.rs settle 行）；字形由结算屏 real_die 章签
    // 预热口一次灌齐（game.rs prewarm 块，与 num/name/reward_note 同块）——两处
    // 显示共用同串同字号 @14，预热一处覆盖两面。纯静态表字段零存档零迁移；
    // 非空与预算由 chapter_tests::chapters_table_shape 随表钉死。
    pub couplet_n: u16,
}

// v1.16.29 刀B 访问器：intro/open/reward_note/couplet/lore 收编 textpack 连号段
//（pack_text.py --tables2 产侧原位写 N）。evtx O(1) 纯算术零分配——章节书/横幅/
// 结算屏帧路径安全（性能十戒）；行内容与旧字面量逐字节等价（真相源
// text_source.txt 行 + chapter_tests 钉测）。
impl ChapterDef {
    pub fn num(&self) -> &'static str {
        evtx(self.num_n as usize)
    }
    pub fn name(&self) -> &'static str {
        evtx(self.name_n as usize)
    }
    pub fn intro(&self) -> &'static str {
        evtx(self.intro_n as usize)
    }
    pub fn open(&self) -> &'static str {
        evtx(self.open_n as usize)
    }
    pub fn reward_note(&self) -> &'static str {
        evtx(self.reward_note_n as usize)
    }
    pub fn couplet(&self) -> &'static str {
        evtx(self.couplet_n as usize)
    }
    /// 章间记事第 k 段（越界空串兜底；段数变长 2..=3 由 lore_len 携带）
    pub fn lore(&self, k: usize) -> &'static str {
        if k < self.lore_len as usize {
            evtx(self.lore_ns as usize + k)
        } else {
            ""
        }
    }
    pub fn lore_len(&self) -> usize {
        self.lore_len as usize
    }
}

pub const CHAPTERS: [ChapterDef; 9] = [
    ChapterDef {
        num_n: 1971,
        name_n: 1980,
        intro_n: 1606,
        goals: &[
            ChapterGoal { text_n: 1989, pid: 12 },
            ChapterGoal { text_n: 1990, pid: 0 },
        ],
        reward: 50,
        reward_note_n: 1624,
        lore_ns: 1642, lore_len: 2,
        open_n: 1615,
        couplet_n: 1633,
    },
    ChapterDef {
        num_n: 1972,
        name_n: 1981,
        intro_n: 1607,
        goals: &[
            ChapterGoal { text_n: 1991, pid: 1 },
            ChapterGoal { text_n: 1992, pid: 2 },
        ],
        reward: 80,
        reward_note_n: 1625,
        lore_ns: 1644, lore_len: 2,
        open_n: 1616,
        couplet_n: 1634,
    },
    ChapterDef {
        num_n: 1973,
        name_n: 1982,
        intro_n: 1608,
        goals: &[
            ChapterGoal { text_n: 1993, pid: 3 },
            ChapterGoal { text_n: 1994, pid: 14 },
        ],
        reward: 120,
        reward_note_n: 1626,
        lore_ns: 1646, lore_len: 2,
        open_n: 1617,
        couplet_n: 1635,
    },
    ChapterDef {
        num_n: 1974,
        name_n: 1983,
        intro_n: 1609,
        goals: &[
            ChapterGoal { text_n: 1995, pid: 4 },
            ChapterGoal { text_n: 1996, pid: 5 },
        ],
        reward: 166,
        reward_note_n: 1627,
        lore_ns: 1648, lore_len: 2,
        open_n: 1618,
        couplet_n: 1636,
    },
    ChapterDef {
        num_n: 1975,
        name_n: 1984,
        intro_n: 1610,
        goals: &[
            ChapterGoal { text_n: 1997, pid: 6 },
            ChapterGoal { text_n: 1998, pid: 7 },
        ],
        reward: 200,
        reward_note_n: 1628,
        lore_ns: 1650, lore_len: 2,
        open_n: 1619,
        couplet_n: 1637,
    },
    ChapterDef {
        num_n: 1976,
        name_n: 1985,
        intro_n: 1611,
        goals: &[
            ChapterGoal { text_n: 1999, pid: 8 },
            ChapterGoal { text_n: 2000, pid: 9 },
        ],
        reward: 240,
        reward_note_n: 1629,
        lore_ns: 1652, lore_len: 2,
        open_n: 1620,
        couplet_n: 1638,
    },
    ChapterDef {
        num_n: 1977,
        name_n: 1986,
        intro_n: 1612,
        goals: &[
            ChapterGoal { text_n: 2001, pid: 10 },
            // v1.16.19 目标行去重：原「封卷已斩——讨牒 6/6 方见终战卡」与四章目标
            // 「六牒归档——讨牒 6/6，道韵+166」数字行全同——改叙事新句（pid5 不动）
            ChapterGoal { text_n: 2002, pid: 5 },
        ],
        reward: 320,
        reward_note_n: 1630,
        lore_ns: 1654, lore_len: 2,
        open_n: 1621,
        couplet_n: 1639,
    },
    ChapterDef {
        num_n: 1978,
        name_n: 1987,
        intro_n: 1613,
        goals: &[
            ChapterGoal { text_n: 2003, pid: 11 },
            ChapterGoal { text_n: 2004, pid: 13 },
        ],
        reward: 400,
        reward_note_n: 1631,
        lore_ns: 1656, lore_len: 3,
        open_n: 1622,
        couplet_n: 1640,
    },
    // v1.16.10 书外之章：四结局全览后的首个「书外」进行态目标——终局不再是悬空。
    // pid13（四卷全览）为第八章既有谓词复用（章九开局即已毕其一），新谓词 pid15
    //（历代最高修为抵执卷境）进 chapter_goal_met 单口 match；ch= 位掩码 u32 位 8，
    // 旧档载入帧静默补齐口径不变（老玩家补齐不补发，与章成之礼既定取舍一致）。
    ChapterDef {
        num_n: 1979,
        name_n: 1988,
        intro_n: 1614,
        goals: &[
            // v1.16.19 目标行去重：原句与八章第二目标「四卷全览——四种结局皆已落款」
            // 全同——改叙事新句（pid13 不动，纯字面量；达成谓词同口）
            ChapterGoal { text_n: 2005, pid: 13 },
            ChapterGoal { text_n: 2006, pid: 15 },
        ],
        reward: 500,
        reward_note_n: 1632,
        lore_ns: 1659, lore_len: 2,
        open_n: 1623,
        couplet_n: 1641,
    },
];

// ---------------- 出身 ----------------
pub struct OriginDef {
    pub name_n: u16,
    pub desc_n: u16,
}

impl OriginDef {
    pub fn name(&self) -> &'static str {
        evtx(self.name_n as usize)
    }
    pub fn desc(&self) -> &'static str {
        evtx(self.desc_n as usize)
    }
}
pub const ORIGINS: [OriginDef; 6] = [
    OriginDef {
        name_n: 2197,
        desc_n: 2203,
    },
    OriginDef {
        name_n: 2198,
        desc_n: 2204,
    },
    OriginDef {
        name_n: 2199,
        desc_n: 2205,
    },
    OriginDef {
        name_n: 2200,
        desc_n: 2206,
    },
    OriginDef {
        name_n: 2201,
        desc_n: 2207,
    },
    OriginDef {
        name_n: 2202,
        desc_n: 2208,
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
// v1.16.27 减包刀⑥：EVENTS 元组再收窄 (u16,u8,u8,bool)=6B/条 → (u16,u16)=4B/条
// （291 行 data 段 1746B→1164B，净 −582B）。可读 4 元组行常量改名 EVENTS_ROWS，
// 只在 const-eval 被 pack_events 消费（编译期打包），不落 wasm data 段；运行期
// EVENTS 为打包位域：低 6bit 权重（新上限 63，扩池纪律：单条 w≤63）、bit6..8
// 最低境界 mr（0..7）、bit9 善缘位。解码读点 ev_w/ev_mr/ev_good（#[inline] 位运算，
// 轮盘路径纳秒级）；行可读形态与 tools/tables_recheck.py、ledger_recheck.py 的
// 4 元组解析口径零变（表行仍是 (id,w,mr,good)）；打包无损性由 game.rs
// events_bitfield_pack_lossless 全表逐行钉测兜底（w>63/mr>7 当场红）。
pub const N_EVENTS: usize = 398; // v1.16.53 段二十六开设扩池 +3：395→398（395 special80 检定朱点回访+396 笔架山影/397 次纸充宣 散件照勘察二提名——special80 检定窗 79 后第三批裁定落地：三批一条节奏 76@v1.16.37/77@v1.16.44/78@v1.16.47/79@v1.16.50，79+3 批恰本批轮到，题材照 GameBook:180 候选 a 官制线「朱点回访」兑现（386 过臂 N2888「一行名姓从此两册同录」来年伏笔顺延，收册人终卷独占不动），396/397 mr 层照 ledger_recheck derive 实测（mr6 两批未落最小善层/mr7 每批一条第九条）；v1.16.52 段二十五开设扩池 +3：392→395（392 废稿拟声/393 洗笔澄心/394 假帖索润 照勘察二提名——三卡全散件，special80 检定窗 79 后第二批/三批：三批一条节奏 76@v1.16.37/77@v1.16.44/78@v1.16.47/79@v1.16.50，预留约 v1.16.53 段二十六批落，题材待勘察提名；v1.16.51 段二十四开设扩池 +3：389→392（389/390/391 照勘察二提名——三卡全散件，special80 检定窗 79 后第一批/三批：三批一条节奏 76@v1.16.37/77@v1.16.44/78@v1.16.47/79@v1.16.50，预留约 v1.16.53 段二十六批落，题材待勘察提名；v1.16.50 段二十三 +3：386→389（386 special79 检定+387/388 散件——78+3 批恰本批轮到，兑现 GameBook B 表余留「收册人问」）；v1.16.49 段二十二 +3：383→386（三卡全散件续未轮到留档）；v1.16.48 段二十一 +3：380→383（三卡全散件）；v1.16.47 段二十 +3：377→380（377 special78 检定+378/379 散件）
/// 名/简 textpack 段一条数（v1.16.26 末态；段二自 v1.16.27 扩池起连号追加）
const EV_SEG1: usize = 291;
/// 名/简 textpack 段二条数（v1.16.27 末态；段三自 v1.16.28 扩池起连号追加）。
/// 段界常量是 len2/len3/len4 的唯一正确写法：len2 若写 N_EVENTS−EV_SEG1，N_EVENTS
/// 增长时 len2 会静默变大、id299..308 名/简读到段二简行错位（错行非空串，
/// is_empty 防线抓不住）——batch_tables_aligned 的段三 get 首尾逐字钉为内容级防线
const EV_SEG2: usize = 299;
/// 名/简 textpack 段三条数界（v1.16.28 末态 309；段四自 v1.16.29 扩池起连号
/// 追加）。len3 一律写 EV_SEG3−EV_SEG2 段界常量差（勿写 N_EVENTS−EV_SEG2——
/// N_EVENTS 增长时 len3 静默吞掉段四行、id309..316 名/简读到段三错行非空串，
/// is_empty 防线抓不住——段四 get 首尾逐字钉为内容级防线，见
/// ev_tables_section_bounds / batch_tables_aligned v1.16.29 段）
const EV_SEG3: usize = 309;
/// 名/简 textpack 段四条数界（v1.16.30 扩池起 317；段五自 v1.16.30 批连号追加）。
/// len4 唯一正确写法=EV_SEG4−EV_SEG3 段界常量差（勿写 N_EVENTS−EV_SEG3——
/// N_EVENTS 增长时 len4 静默吞掉段五行、id317..324 名/简读到段四错行非空串，
/// is_empty 防线抓不住——段五 get 首尾逐字钉为内容级防线，见
/// ev_tables_section_bounds / batch_tables_aligned v1.16.30 段）
const EV_SEG4: usize = 317;
/// 名/简 textpack 段五条数界（v1.16.30 扩池起 317；段六自 v1.16.31 扩池起
/// 连号追加）。len5 唯一正确写法=EV_SEG5−EV_SEG4 段界常量差（勿写
/// N_EVENTS−EV_SEG4——N_EVENTS 增长时 len5 静默吞掉段六行、id325..334
/// 名/简读到段五错行非空串，is_empty 防线抓不住——段六 get 首尾逐字钉为
/// 内容级防线，见 ev_tables_section_bounds / batch_tables_aligned v1.16.31 段）
const EV_SEG5: usize = 325;
/// 名/简 textpack 段六条数界（v1.16.31 扩池 10 条起 325..335；段七自 v1.16.31
/// 新功能 3 条起连号追加——段六名/简 N1905..1914/N1915..1924 紧连封死，
/// 续批照段四→段五先例开新段：名段七自下一空 N1954 起）。len6 唯一正确
/// 写法=EV_SEG6−EV_SEG5 段界常量差（勿写 N_EVENTS−EV_SEG5——N_EVENTS
/// 增长时 len6 静默吞掉段七行、id335..337 名/简读到段六错行非空串，
/// is_empty 防线抓不住——段七 get 首尾逐字钉为内容级防线，见
/// ev_tables_section_bounds / batch_tables_aligned v1.16.31 段七）
const EV_SEG6: usize = 335;
/// 名/简 textpack 段八条数界（v1.16.32 单3 扩池 6 条起 338..344；段八自下批
/// 满员后续批开段九照段四→段五先例）。len7 唯一正确写法=EV_SEG7−EV_SEG6 段界
/// 常量差（勿写 N_EVENTS−EV_SEG6——N_EVENTS 增长时 len7 静默吞掉段八行、
/// id338..343 名/简读到段七错行非空串，is_empty 防线抓不住——段八 get 首尾
/// 逐字钉为内容级防线，见 ev_tables_section_bounds / batch_tables_aligned
/// v1.16.32 段八）
const EV_SEG7: usize = 338;
/// 名/简 textpack 段八条数界（v1.16.32 单3 末态 344=N_EVENTS 快照；下批扩池开
/// 段九时 N_EVENTS 增长，本常量冻结段八上沿——len8 唯一正确写法=EV_SEG8−EV_SEG7
/// 段界常量差，勿回写 N_EVENTS−EV_SEG7：N_EVENTS 增长时 len8 会静默吞掉段九首行、
/// id338..343 名/简读到段九错行非空串，is_empty 防线抓不住——段九 get 首尾逐字
/// 钉为内容级防线照 batch_tables_aligned 先例）。v1.16.33 单1 先行引入快照常量
///（字节效应 0，扩池段九开工前把末段差写法钉死）
const EV_SEG8: usize = 344;
/// 名/简 textpack 段九条数界（v1.16.33 单3 末态 349=N_EVENTS 快照；下批扩池开
/// 段十时 N_EVENTS 增长，本常量冻结段九上沿——len9 唯一正确写法=EV_SEG9−EV_SEG8
/// 段界常量差，勿回写 N_EVENTS−EV_SEG8：N_EVENTS 增长时 len9 会静默吞掉段十
/// 首行、id344..348 名/简读到段十错行非空串，is_empty 防线抓不住——段十 get
/// 首尾逐字钉为内容级防线照 batch_tables_aligned 先例）。v1.16.34 单1 先行引入
/// 快照常量（字节效应 0，扩池段十开工前把末段差写法钉死）——v1.16.34 单3 段十
/// 已开工，差写法兑现（len9=(EV_SEG9−EV_SEG8) 在位）
const EV_SEG9: usize = 349;
/// 名/简 textpack 段十条数界（v1.16.34 单3 末态 351=N_EVENTS 快照；下批扩池开
/// 段十一时 N_EVENTS 增长，本常量冻结段十上沿——len10 唯一正确写法=EV_SEG10−EV_SEG9
/// 段界常量差，勿回写 N_EVENTS−EV_SEG9：N_EVENTS 增长时 len10 会静默吞掉段十一
/// 首行、id349..350 名/简读到段十一错行非空串，is_empty 防线抓不住——段十一 get
/// 首尾逐字钉为内容级防线照 batch_tables_aligned 先例）。v1.16.35 单1 先行引入
/// 快照常量（字节效应 0，扩池段十一开工前把末段差写法钉死）——v1.16.35 单3 段
/// 十一已开工，差写法兑现（len10=(EV_SEG10−EV_SEG9) 在位）
const EV_SEG10: usize = 351;
/// 名/简 textpack 段十一条数界（v1.16.35 单3 末态 353=N_EVENTS 快照；下批扩池开
/// 段十二时 N_EVENTS 增长，本常量冻结段十一上沿——len11 唯一正确写法=EV_SEG11−EV_SEG10
/// 段界常量差，勿回写 N_EVENTS−EV_SEG10：N_EVENTS 增长时 len11 会静默吞掉段十二
/// 首行、id351..352 名/简读到段十二错行非空串，is_empty 防线抓不住——段十二 get
/// 首尾逐字钉为内容级防线照 batch_tables_aligned 先例，由扩池单随批补钉）。
/// v1.16.36 单1 先行引入快照常量（字节效应 0，扩池段十二开工前把末段差写法钉死）
/// ——v1.16.36 单3 段十二已开工，差写法兑现（len11=(EV_SEG11−EV_SEG10) 在位）
const EV_SEG11: usize = 353;
/// 名/简 textpack 段十二条数界（v1.16.36 单3 末态 356=N_EVENTS 快照；下批扩池开
/// 段十三时 N_EVENTS 增长，本常量冻结段十二上沿——len12 唯一正确写法=EV_SEG12−EV_SEG11
/// 段界常量差，勿回写 N_EVENTS−EV_SEG11：N_EVENTS 增长时 len12 会静默吞掉段十三
/// 首行、id353..355 名/简读到段十三错行非空串，is_empty 防线抓不住——段十三 get
/// 首尾逐字钉为内容级防线照 batch_tables_aligned 先例，由扩池单随批补钉）。
/// v1.16.36 单3 先行引入快照常量（字节效应 0，扩池段十三开工前把末段差写法钉死）——
/// v1.16.37 单2 段十三已开工，差写法兑现（len12=(EV_SEG12−EV_SEG11) 在位）
const EV_SEG12: usize = 356;
/// 名/简 textpack 段十三条数界（v1.16.37 单2 末态 359=N_EVENTS 快照；下批扩池开
/// 段十四时 N_EVENTS 增长，本常量冻结段十三上沿——len13 唯一正确写法=EV_SEG13−
/// EV_SEG12 段界常量差，勿回写 N_EVENTS−EV_SEG12：N_EVENTS 增长时 len13 会静默
/// 吞掉段十四首行、id356..358 名/简读到段十四错行非空串，is_empty 防线抓不住——
/// 段十四 get 首尾逐字钉为内容级防线照 batch_tables_aligned 先例，由扩池单随批补钉）。
/// v1.16.37 单2 先行引入快照常量（字节效应 0，扩池段十四开工前把末段差写法钉死）——
/// v1.16.38 单2 段十四已开工，差写法兑现（len13=(EV_SEG13−EV_SEG12) 在位）
const EV_SEG13: usize = 359;
/// 名/简 textpack 段十四条数界（v1.16.38 单2 末态 362=N_EVENTS 快照；下批扩池开
/// 段十五时 N_EVENTS 增长，本常量冻结段十四上沿——len14 唯一正确写法=EV_SEG14−
/// EV_SEG13 段界常量差，勿回写 N_EVENTS−EV_SEG13：N_EVENTS 增长时 len14 会静默
/// 吞掉段十五首行、id359..361 名/简读到段十五错行非空串，is_empty 防线抓不住——
/// 段十五 get 首尾逐字钉为内容级防线照 batch_tables_aligned 先例，由扩池单随批
/// 补钉）。v1.16.38 单2 先行引入快照常量（字节效应 0，扩池段十五开工前把末段差
/// 写法钉死）——v1.16.40 单3 扩池段十五已开工：len14 末段写法兑现改差
/// len14=(EV_SEG14−EV_SEG13)（EVENT_NAMES/EVENT_BRIEF 同批），本常量自此被引用；
/// EV_SEG15=365 快照常量本批预建，下批扩段十六时 len15 改差 (EV_SEG15−EV_SEG14)、
/// len16 写 N_EVENTS−EV_SEG15——dead_code 警告按 v1.16.36 EV_SEG12 先例预期
/// 持续到段十六开工，非本轮缺陷，勿消
const EV_SEG14: usize = 362;

/// 名/简 textpack 段十五条数界（v1.16.40 单3 末态 365 快照；v1.16.43 扩池开段十六
/// N_EVENTS 增长，本常量冻结段十五上沿——len15 唯一正确写法=EV_SEG15−EV_SEG14
/// 段界常量差，勿回写 N_EVENTS−EV_SEG14：N_EVENTS 增长时 len15 会静默吞掉段十六
/// 首行、id362..364 名/简读到段十六错行非空串，is_empty 防线抓不住——段十六 get
/// 首尾逐字钉为内容级防线照 batch_tables_aligned 先例，v1.16.43 扩池单已随批补钉）。
/// v1.16.40 单3 先行引入快照常量（字节效应 0，扩池段十六开工前把末段差写法钉死）
/// ——v1.16.43 本批 len15 已改差兑现（EVENT_NAMES/EVENT_BRIEF :3129/:3184 款），
/// 本常量自此被引用；dead_code 警告转移到下方 EV_SEG16（预期勿消）
const EV_SEG15: usize = 365;

/// 名/简 textpack 段十六条数界（v1.16.43 扩池末态 368=N_EVENTS 快照；下批扩池开
/// 段十七时 N_EVENTS 增长，本常量冻结段十六上沿——len16 唯一正确写法=EV_SEG16−
/// EV_SEG15 段界常量差）。v1.16.43 单3 先行引入快照常量（字节效应 0）——扩池
/// 段十七开工时改差兑现照本批 len15 先例。v1.16.44 段十七批：len16 已改差兑现
///（EVENT_NAMES/EVENT_BRIEF :3184/:3243 款），本常量自此被引用；dead_code 警告
/// 转移到下方 EV_SEG17（预期勿消）
const EV_SEG16: usize = 368;

/// 名/简 textpack 段十七条数界（v1.16.45 施工单三段十八开工批：EV_SEG17 由快照
/// 预建转兑现——len17 两功能落点改差 (EV_SEG17−EV_SEG16)，本常量冻结段十七上沿）
const EV_SEG17: usize = 371;

/// 名/简 textpack 段十八条数界（v1.16.45 施工单三扩池时以 374=N_EVENTS 快照预建；
/// v1.16.46 段十九开工批兑现——len18 两功能落点改段界常量差 (EV_SEG18−EV_SEG17)，
/// 本常量冻结段十八上沿，「预期持续到段十九开工」句至此已兑现）
const EV_SEG18: usize = 374;

/// 名/简 textpack 段十九条数界（v1.16.46 段十九扩池末态 377 快照；v1.16.47 段二
/// 十开工批兑现——len19 两功能落点改段界常量差 (EV_SEG19−EV_SEG18)，本常量冻结
/// 段十九上沿，「预期持续到段二十开工」句至此已兑现）
const EV_SEG19: usize = 377;

/// 名/简 textpack 段二十条数界（v1.16.47 段二十扩池末态 380=N_EVENTS 快照；下批
/// 扩池开段二十一时 N_EVENTS 增长，本常量冻结段二十上沿——len20 唯一正确写法=
/// EV_SEG20−EV_SEG19 段界常量差，届时改差）。本批先行引入快照常量（字节效应 0，
/// 扩池段二十一开工前把末段差写法钉死）——dead_code 警告按 v1.16.36 EV_SEG12 先例
/// **预期持续到段二十一开工**，非本轮缺陷，勿消
const EV_SEG20: usize = 380;

/// 名/简 textpack 段二十一条数界（v1.16.48 段二十一扩池末态 383=N_EVENTS 快照；
/// v1.16.49 段二十二开工批兑现——len21 两功能落点改段界常量差 (EV_SEG21−EV_SEG20)，
/// 本常量冻结段二十一上沿，「预期持续到段二十二开工」句至此已兑现）
const EV_SEG21: usize = 383;

/// 名/简 textpack 段二十二条数界（v1.16.49 段二十二扩池末态 386=N_EVENTS 快照；
/// v1.16.50 段二十三开工批兑现——len22 两功能落点改段界常量差 (EV_SEG22−EV_SEG21)，
/// 本常量冻结段二十二上沿，「预期持续到段二十三开工」句至此已兑现）
const EV_SEG22: usize = 386;

/// 名/简 textpack 段二十三条数界（v1.16.50 段二十三扩池末态 389 快照；
/// v1.16.51 段二十四开工批兑现——len23 两功能落点改段界常量差 (EV_SEG23−EV_SEG22)，
/// 本常量冻结段二十三上沿，「预期持续到段二十四开工」句至此已兑现）
const EV_SEG23: usize = 389;

/// 名/简 textpack 段二十四条数界（v1.16.51 段二十四扩池末态 392 快照；
/// v1.16.52 段二十五开工批兑现——len24 两功能落点改段界常量差 (EV_SEG24−EV_SEG23)，
/// 本常量冻结段二十四上沿，「预期持续到段二十五开工」句至此已兑现）
const EV_SEG24: usize = 392;

/// 名/简 textpack 段二十五条数界（v1.16.52 段二十五扩池末态 395 快照；
/// v1.16.53 段二十六开工批兑现——len25 两功能落点改段界常量差 (EV_SEG25−EV_SEG24)，
/// 本常量冻结段二十五上沿，「预期持续到段二十六开工」句至此已兑现）
const EV_SEG25: usize = 395;

/// 名/简 textpack 段二十六条数界（v1.16.53 段二十六扩池末态 398=N_EVENTS 快照；
/// 下批扩池开段二十七时 N_EVENTS 增长，本常量冻结段二十六上沿——len26 唯一正
/// 确写法=EV_SEG26−EV_SEG25 段界常量差，届时改差）。本批先行引入快照常量（字
/// 节效应 0，扩池段二十七开工前把末段差写法钉死）——dead_code 警告按 v1.16.36
/// EV_SEG12 先例**预期持续到段二十七开工**，非本轮缺陷，勿消
const EV_SEG26: usize = 398;

pub const EVENTS_ROWS: [(u16, u8, u8, bool); N_EVENTS] = [
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
    // 74..=76 新窗，互斥随之解除——v1.16.26 注：74 亦已由稀有检定 284 启用，
    // 链19 若做改走 75..=76）
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
    // v1.16.26 扩池 10 条（281..=290 尾部追加，id==下标纪律勿插中）：账本以
    // tools/ledger_recheck.py 重算为准。具名 NPC 2（281 胡掌柜 Npc(1) 柜后
    // 传眼——避 249/259/264/271；282 宁清霜 Npc(2) 霜泉淬锋——避 252/263/
    // 275/链16 正文）/伴随 1（283 纵马霜原——白名单 24→25，mr5 有伴第 2 条
    // 摊薄 211 独苗）/稀有检定 1（284 枯莲夜雨 w2 mr4——special74 道心检定
    // 照 279/234/246 模板，检定槽 74 启用，本批唯一 special 新支）/mr6/7 善
    // 3（285/286/287 守长生世纪律）/老年×伴双门控第 5 条 1（288 同封岁坛，
    // 照 204/212/239/268 同 if 模板不入白名单）/宿敌回响 1（289 同门销踪
    // mr2 恶——neme_slain≥1 门控照 111/118 同点零新键）/mr0 恶 1（290 误入
    // 猎套摊薄开局头部）。题材复检换字：283 并辔→纵马、284 枯荷→枯莲
    //（辔/荷缺字库 cmap 2321，就地换字零新字形，勿回旧字）
    // 扩池后账本实测复算（tools/ledger_recheck.py）：总基权 878→907（本批
    // +29=9×w3+1×w2）；稀有 21→22 条 39→41 权 ≈4.44%→4.52%（分母 w3 摊薄
    // 效应微升，本轮不再添 w≤2 即可守住）；恶缘常规 56→58 条 204→210 权
    // ≈23.2% 守口径；mr 全行分布（291 条）0:75/1:60/2:48/3:27/4:29/5:23/
    // 6:15/7:14；常态池（w>0）256 条：0:40 条178权/1:60 条220权/2:48 条176权
    // /3:27 条85权/4:29 条89权/5:23 条70权/6:15 条44权/7:14 条45权（mr3 27
    // 条仍薄于均值、mr5 23 条仍最薄，本批 mr5 主补 +2）；mr0 开局头部 top4
    // 集中度（含伴）22.3%→21.9%（无伴口径 23.1%→22.7%，分母被 290 摊薄，
    // 头部四张仍为 0/1/3/5 四老卡不动）
    (281, 3, 3, true),  // 柜后传眼
    (282, 3, 5, true),  // 霜泉淬锋
    (283, 3, 5, true),  // 纵马霜原（伴随白名单第 25 条，mr5 有伴第 2 条）
    (284, 2, 4, true),  // 枯莲夜雨（稀有 w2：special74 道心检定）
    (285, 3, 6, true),  // 桥心合石
    (286, 3, 6, true),  // 香火还名
    (287, 3, 7, true),  // 千鹤过山
    (288, 3, 1, true),  // 同封岁坛（老年×有伴双门控第 5 条）
    (289, 3, 2, false), // 同门销踪（宿敌回响，neme_slain≥1 门控）
    (290, 3, 0, false), // 误入猎套
    // v1.16.27 扩池 8 条（291..=298 尾部追加，id==下标纪律勿插中）：账本以
    // tools/ledger_recheck.py 重算为准。老年层 1（291 换窗新纸 mr1——v1.16.20
    // 后五轮未补的重见率最高层）/具名 NPC 2（292 讲堂补席 Npc(0) 玄诚 mr4
    // 首卡、296 醉题酒旗 Npc(3) 醉道人 mr4 首卡——两人 mr4 零布点补位；题材
    // 避 248 巡山/253/267/273 与 222 星砂/244 酒账/261 醉倒/274）/伴随 1
    // （293 袖里暖石 mr6——白名单 25→26，mr6 有伴第 3 条摊薄 211/225 独苗）/
    // 恶缘 3（294 假丹换真 mr3、295 符价压市 mr5、298 灵脉被截 mr6——恶缘
    // 常规 58→61 条 219 权 ≈23.5% 守口径；题材避 186/200/235/254/266 与
    // 194/214/242/277）/老年×伴双门控第 6 条 1（297 重缝冬衣 mr5——照
    // 204/212/239/268/288 同 if 模板不入白名单，徽标走老年臂）。零新 special
    // 零新链；扩池文案走 textpack 管线（臂内裸字面量 → pack_text.py 收编）。
    // 扩池后账本实测复算（tools/ledger_recheck.py）：总基权 907→931（本批
    // +24=8×w3）；稀有 22 条 41 权 ≈4.52%→4.40%（分母摊薄微降，零 w≤2 新条
    // 守住）；mr 全行分布（299 条）0:75/1:61/2:48/3:28/4:31/5:25/6:17/7:14；
    // 常态池（w>0）264 条：0:40 条178权/1:61 条223权/2:48 条176权/3:28 条
    // 88权/4:31 条95权/5:25 条76权/6:17 条50权/7:14 条45权（mr3 28 条仍薄于
    // 均值、mr5 25 条仍最薄，本批 mr5 +2）；恶缘 mr3 6→7/mr5 5→6/mr6 4→5
    (291, 3, 1, true),  // 换窗新纸（老年层，余量≤10 载门控）
    (292, 3, 4, true),  // 讲堂补席（玄诚长老 Npc(0) mr4 首卡）
    (293, 3, 6, true),  // 袖里暖石（伴随白名单第 26 条，mr6 有伴第 2 条——序数
                        // 勘误随批：mr6 有伴实为 225 首条/293 第 2 条/339 第 3 条，
                        // 原注「第 3 条」把 226 mr0 误计，三副本同批改）
    (294, 3, 3, false), // 假丹换真（mr3 恶）
    (295, 3, 5, false), // 符价压市（mr5 恶）
    (296, 3, 4, true),  // 醉题酒旗（醉道人 Npc(3) mr4 首卡）
    (297, 3, 5, true),  // 重缝冬衣（老年×有伴双门控第 6 条）
    (298, 3, 6, false), // 灵脉被截（mr6 恶）
    // v1.16.28 扩池 10 条（299..=308 尾部追加，id==下标纪律勿插中）：账本以
    // tools/ledger_recheck.py 重算为准。老年×伴双门控第 7 条 1（299 合誊旧稿
    // mr2——现 6 条 mr 全 mr1×5+mr5×1，本条 mr2 换层；照 204/212/239/268/288/
    // 297 同 if 模板不入白名单，徽标走老年臂）/伴随 1（300 晨课互校 mr7——
    // 白名单 26→27，mr7 有伴第 2 条摊薄 241 独苗）/具名 NPC 3（301 霜窗描枝
    // Npc(2) 宁清霜 mr3 首卡、302 晨钟校音 Npc(0) 玄诚 mr1 首卡、308 醉后忘句
    // Npc(3) 醉道人 mr3 首卡——正文钉名⟺Npc(n) 照 248/292 口径；题材避 252
    // 枰边/263 旧穗/275 寄剑/282 淬锋 与 248 巡山/253 经阁/267 名册/273 扫阶/
    // 292 补席 与 222 星砂/244 酒账/261 醉倒/274 引路/296 题旗）/稀有 1（303
    // 雷纹入纸 w2 mr7——稀有 22→23 条 41→43 权 ≈4.48%，mr7 稀有第 2 条摊薄
    // 224 独苗；题材避 224 坠星）/散件 4（304 代书家信 mr5——mr5 常态 25 条
    // 为最薄层补位、305 灵窖渗漏 mr3 恶、306 抄经赠邻 mr6、307 书船遇风 mr2
    // 恶）。零新 special 零新链（special 75..=76 留窗、链19 不做照 v1.16.26
    // 注互斥口径——v1.16.31 更新：75 已由检定臂 329 启用，留窗仅余 76，链19
    // 若做改走 76）；扩池文案走 textpack 管线（臂内裸字面量/format! → pack_text.py
    // 收编），真相源段三名/简连号段 1442..=1451/1452..=1461 先于正文手写。
    // 扩池后账本实测复算（tools/ledger_recheck.py）：总基权 931→960（本批
    // +29=9×w3+1×w2）；稀有 23 条 43 权 ≈4.48%（v1.16.26 4.52% 上沿内）；
    // 恶缘常规 61→63 条 219→225 权 ≈23.4% 守口径；mr 全行分布（309 条）
    // 0:75/1:62/2:50/3:31/4:31/5:26/6:18/7:16；常态池（w>0）274 条：
    // 0:40 条178权/1:62 条226权/2:50 条182权/3:31 条97权/4:31 条95权/
    // 5:26 条79权/6:18 条53权/7:16 条50权（mr3 31 条/mr5 26 条两最薄层本批
    // +3/+1 摊薄；恶缘 mr2/mr3 各 +1）
    // v1.16.29 扩池 8（老年层 2——309 雪堤喂雀/310 旧席重缝，11→13 条摊薄
    // 重见率最高层；mr5 1——311 下山买茶 26→27；具名 NPC 2——312 雪水研墨
    // 宁清霜 Npc(2) 第二卡、313 第三本账 胡掌柜 Npc(1) 第六卡，正文钉名⟺
    // Npc(n) 照 248/292/301 口径；mr3 1——314 檐下折梅；稀有 1——315 琅函倒页
    // w1 mr7，稀有 23→24 条 43→44 权 ≈4.48% 守带（w2 会得 4.57% 出带，w1
    // 先例 15/57/73）；mr4 恶 1——316 药童卷方，恶缘 9→10）。单1 减包
    // −10.27KB 达 6~8 条分级上沿；真相源段四名/简连号段 1825..=1832/1833..=1840
    // 先于正文手写（基址因单1/单2 占 N1492..1824 顺延）。
    // 扩池后账本实测复算（tools/ledger_recheck.py）：总基权 960→982（本批
    // +22=7×w3+1×w1）；稀有 24 条 44 权 ≈4.48%；恶缘常规 63→64 条 225→228 权
    // ≈23.2%；mr 全行分布（317 条）0:76/1:63/2:51/3:32/4:33/5:27/6:18/7:17；
    // 常态池（w>0）mr5 27 条 82 权/mr3 32 条 100 权两最薄层续摊（本批各 +1）
    (299, 3, 2, true),  // 合誊旧稿（老年×有伴双门控第 7 条，mr2 换层）
    (300, 3, 7, true),  // 晨课互校（伴随白名单第 27 条，mr7 有伴第 2 条）
    (301, 3, 3, true),  // 霜窗描枝（宁清霜 Npc(2) mr3 首卡）
    (302, 3, 1, true),  // 晨钟校音（玄诚长老 Npc(0) mr1 首卡）
    (303, 2, 7, true),  // 雷纹入纸（稀有 w2：mr7 稀有第 2 条）
    (304, 3, 5, true),  // 代书家信（mr5 善，最薄层补位）
    (305, 3, 3, false), // 灵窖渗漏（mr3 恶）
    (306, 3, 6, true),  // 抄经赠邻（mr6 善）
    (307, 3, 2, false), // 书船遇风（mr2 恶）
    (308, 3, 3, true),  // 醉后忘句（醉道人 Npc(3) mr3 首卡）
    // v1.16.29 扩池 8 条（单1 净减 10.27KB 达 6~8 条分级上沿；总基权 960→982，
    // 稀有 23→24 条 43→44 权 ≈4.48% 守勘察带）：老年层 +2（309/310——重见率
    // 最高层摊薄）/mr5 最薄层 1（311）/具名 NPC 2（312 宁清霜 Npc(2) 第二卡/
    // 313 胡掌柜 Npc(1) 第六卡，正文钉名⟺Npc(n) 照 248/292/301 口径）/mr3 1
    // （314）/稀有 w1 1（315——w1 先例 15/57/73）/mr4 恶 1（316，恶缘 9→10）
    (309, 3, 0, true),  // 雪堤喂雀（老年层，余量≤10 载门控）
    (310, 3, 1, true),  // 旧席重缝（老年层，余量≤10 载门控）
    (311, 3, 5, true),  // 下山买茶（mr5 善，最薄层补位）
    (312, 3, 4, true),  // 雪水研墨（宁清霜 Npc(2) 第二卡）
    (313, 3, 2, true),  // 第三本账（胡掌柜 Npc(1) 第六卡）
    (314, 3, 3, true),  // 檐下折梅（mr3 善）
    (315, 1, 7, true),  // 琅函倒页（稀有 w1：mr7 稀有第 3 条）
    (316, 3, 4, false), // 药童卷方（mr4 恶）
    // v1.16.30 扩池 8 条（单1 净减 16.88KB 达 ≥8KB 分级上沿，A1..A8 全 8 条）：
    // mr5 最薄层 3（317 崖灯照径 善/318 云路设卡 恶/319 烛下问疑 玄诚长老
    // Npc(0) 第七卡，正文钉名⟺Npc(n) 照 248/292/301 口径）/mr0 1——320 雪里
    // 拾稿 宁清霜 Npc(2) 第七卡（照 259 市头新肆 mr0 NPC 专卡先例摊薄开局
    // 头部 top4 集中度）/伴随 1——321 同封新茶（白名单 27→28，mr3 伴随 2→3，
    // Comp 立绘反查照 262/257 模式）/老年×伴双门控 1——322 树下尝果（if 表内
    // 第 8 条，mr3 换层照 299 先例，不入白名单徽标走老年臂；v1.16.31 清账：
    // 本行旧写「檐下归燕」与行注/gallery/臂三面不一致，以行注 322 树下尝果
    // 为准）/mr7 最薄层 1——
    // 323 云上迎雪 善 mr6（行值 (323,3,6,true)——本行批注原与行注同误写
    // mr7，v1.16.32 单4 清账对齐；守长生世纪律不写寿元将尽不触破境裁决，
    // 照 270/287 模板）/
    // 稀有 1——324 旧卷新墨 w1 mr6（213 独苗摊薄，w1 先例 15/57/73/315；
    // 稀有 24→25 条 44→45 权 ≈4.48% 守带）。
    // 扩池后账本实测复算（tools/ledger_recheck.py）：总基权 982→1004（本批
    // +22=7×w3+1×w1）；稀有 25 条 45 权 ≈4.48%；恶缘常规 64→65 条 228→231 权
    // ≈23.0%；mr 全行分布（325 条）0:77/1:63/2:51/3:34/4:33/5:30/6:20/7:17；
    // 常态池（w>0）mr5 27→30 条 82→91 权/mr3 32→34 条 100→106 权最薄层摊薄
    (317, 3, 5, true),  // 崖灯照径（mr5 善，最薄层补位）
    (318, 3, 5, false), // 云路设卡（mr5 恶）
    (319, 3, 5, true),  // 烛下问疑（玄诚长老 Npc(0) 第七卡，mr5）
    (320, 3, 0, true),  // 雪里拾稿（宁清霜 Npc(2) 第七卡，mr0）
    (321, 3, 3, true),  // 同封新茶（伴随白名单第 28 条，mr3 有伴第 3 条）
    (322, 3, 3, true),  // 树下尝果（老年×伴双门控第 8 条，mr3 换层）
    (323, 3, 6, true),  // 云上迎雪（mr6 善——行注原误写 mr7，v1.16.32 单3 清账照 (323,3,6,true) 行值勘误）
    (324, 1, 6, true),  // 旧卷新墨（稀有 w1：mr6 稀有第 2 条）
    // v1.16.31 扩池 10 条（325..=334 尾部追加，id==下标纪律勿插中）：账本以
    // tools/ledger_recheck.py 重算为准。具名 NPC 2（325 坛泥留字 mr7——醉道人
    // Npc(3) 第七卡照 222/244 模板守长生世纪律，mr7 全行最薄层 17 条补位；
    // 326 账底一文 mr6——胡掌柜 Npc(1) 第七卡，NPC×mr6 零覆盖补位且 mr6 层
    // 20 条次薄；正文钉名⟺Npc(n) 照 248/292/301 口径）/伴随 1（327 灯下拼卷
    // mr0——白名单 28→29，伴随层 mr0 仅 2 条最薄，照 259/320 mr0 摊薄开局
    // 头部 top4 先例）/老年×伴双门控 1（328 册间拜帖 mr4——if 表内第 9 条照
    // 204..322 模板并臂勿拆，不入白名单徽标走老年臂；现 8 条 mr1×5+mr5/mr2/
    // mr3 各 1，本条 mr4 换层）/稀有检定 1（329 檐马问风 w1 mr2 special75
    // 新检定窗——75..=76 留窗自 v1.16.26 注起开用，走新检定不触链互斥照
    // 284 口径：apply_eff 新臂+副标「受檐马一问」；稀有 25→26 条 45→46 权，
    // w1 先例 15/57/73/315/324）/散件 5（330 寒泉让汲 mr5——mr5 常态最薄层
    // 补位、331 夜系渡舟 mr3——mr3 常态次薄层补位、332 纸边攒卷 mr0——
    // 摊薄开局 top4、333 冒名香摊 mr2 恶、334 接梁旧漆 mr4）。
    // 零新链（链19 不做照 v1.16.26 注互斥口径，检定需求走 special75 不占链
    // 窗）；扩池文案走 textpack 管线（臂内裸字面量/format! → pack_text.py
    // 收编），真相源段六名/简连号段 1905..=1914/1915..=1924 先于正文手写。
    // 扩池后账本实测复算（tools/ledger_recheck.py）：总基权 1004→1032（本批
    // +28=9×w3+1×w1）；稀有 26 条 46 权 ≈4.46%；恶缘常规 65→66 条 231→234
    // 权 ≈22.7%；mr 全行分布（335 条）0:79/1:63/2:53/3:35/4:35/5:31/6:21/7:18；
    // 常态池（w>0）mr5 30→31 条 91→94 权/mr3 34→35 条 106→109 权两最薄层
    // 摊薄/mr0 top4 集中度续摊
    (325, 3, 7, true),  // 坛泥留字（醉道人 Npc(3) 第七卡，mr7 善最薄层）
    (326, 3, 6, true),  // 账底一文（胡掌柜 Npc(1) 第七卡，mr6）
    (327, 3, 0, true),  // 灯下拼卷（伴随白名单第 29 条，mr0 有伴）
    (328, 3, 4, true),  // 册间拜帖（老年×伴双门控第 9 条，mr4 换层）
    (329, 1, 2, true),  // 檐马问风（稀有 w1：mr2 检定 special75）
    (330, 3, 5, true),  // 寒泉让汲（mr5 善，最薄层补位）
    (331, 3, 3, true),  // 夜系渡舟（mr3 善，次薄层补位）
    (332, 3, 0, true),  // 纸边攒卷（mr0 善，摊薄开局头部）
    (333, 3, 2, false), // 冒名香摊（mr2 恶）
    (334, 3, 4, true),  // 接梁旧漆（mr4 善）
    // v1.16.31 新功能 3 条（335..=337 尾部追加，id==下标纪律勿插中；全部零新
    // 键零资产——读 sim 天时快照/meta.dongfu 既有字段，与场景行「地名·天时」
    // 互证；sim 天时是入镜快照（演化在 advance_time 唯一推岁口），门控读当次
    // 入池判定，措辞按「本世天时底色」禁写天气变化）：天气门控 2（335 连雨
    // 收书 w3 mr4——雨（weather==2）门，胡掌柜 Npc(1) 第八卡照 313/326 模板
    // 正文钉名⟺Npc(n)；336 迎潮一立 w3 mr6——灵潮（weather==4）门，散件不
    // 挂脸）/洞府全盛 1（337 三脉同焕 w3 mr3——阵眼/田畦/丹房三设施全满才
    // 入池，与 134-136 单设施门互补同 if 并臂；玄诚长老 Npc(0) 第八卡）。
    // 扩池后账本实测复算（tools/ledger_recheck.py）：总基权 1032→1041（本批
    // +9=3×w3）；稀有 26 条 46 权 ≈4.42%；恶缘常规 66 条 234 权 ≈22.5%；
    // mr 全行分布（338 条）0:79/1:63/2:53/3:36/4:36/5:31/6:22/7:18；
    // 常态池（w>0）mr3 35→36 条 109→112 权/mr4 35→36 条 107→110 权/mr6
    // 21→22 条 60→63 权
    (335, 3, 4, true),  // 连雨收书（雨门 weather==2，胡掌柜 Npc(1) 第八卡，mr4）
    (336, 3, 6, true),  // 迎潮一立（灵潮门 weather==4，mr6 善）
    (337, 3, 3, true),  // 三脉同焕（洞府三设施全满门，玄诚 Npc(0) 第八卡，mr3）
    // v1.16.32 单3 扩池 6（338..=343，尾部追加勿插中——id==下标对账纪律）：
    // 醉道人第八卡 mr7 w3 善（mr7 常态 18→19 条全行最薄+四 NPC 卡 8/8/8/8 补齐）
    // /伴随第 30 条 mr6（有伴第 4 条，白名单 29→30）/雾天新门 w3 宁清霜第八卡
    //（weather==3 门照 335/336 同点；五天象门控雨/灵潮/雾三面铺开）/稀有
    // mr5 w1（稀有 26→27 条 46→47 权）/老年×伴双门控 mr4 换层第 10 条/mr6 恶
    //（mr6 恶 5→6 条）；扩池后账本实测复算（tools/ledger_recheck.py）：总基权
    // 1041→1057（本批 +16=5×w3+1×w1）；稀有 27 条 47 权 ≈4.45%；恶缘常规
    // 67 条 237 权；mr 全行分布（344 条）0:79/1:63/2:53/3:36/4:38/5:32/6:24/
    // 7:19；常态池（w>0）309 条
    (338, 3, 7, true),  // 暮坛对酌（醉道人 Npc(3) 第八卡，mr7 善）
    (339, 3, 6, true),  // 岁暮并账（伴随白名单第 30 条，mr6 有伴第 3 条——序数勘误
                        // 同 293 注：226 mr0 误计入 mr6，实 225/293/339 三条）
    (340, 3, 4, true),  // 雾里听剑（雾门 weather==3，宁清霜 Npc(2) 第八卡，mr4）
    (341, 1, 5, true),  // 砚底星痕（稀有 mr5 w1，散件）
    (342, 3, 4, true),  // 灯花并盏（老年×伴双门控 mr4 换层第 10 条）
    (343, 3, 6, false), // 老炉现纹（mr6 恶，恶缘 66→67 条）
    // v1.16.33 单3 扩池批 5 条（段九，N_EVENTS 344→349）：A1..A5 按减包后余量
    // 22,755B 落 22~24KB 带取 5 条（A6 洞府门续作砍，裁剪序 A6→A5→A4 勿倒）
    (344, 3, 7, true),  // 同裱旧稿（伴随白名单第 31 条，mr7 有伴第 3 条，Comp 立绘）
    (345, 3, 7, false), // 井绳朽断（mr7 恶，恶缘 67→68 条，散件）
    (346, 3, 3, true),  // 晴日晒药（晴门 weather==0，mr3 善，散件）
    (347, 1, 2, true),  // 梦续残句（稀有 w1：mr2 稀有第 4 条照 72/234/329 层位，散件 special 恒 0）
    (348, 3, 6, true),  // 霜橘分甘（老年×伴双门控第 11 条，mr6 换层——双门控
                        // mr6/mr7 原零覆盖，Comp 立绘不入白名单）
    // v1.16.34 单3 扩池批 2 条（段十，N_EVENTS 349→351）：按单1+单2 减包后余量
    // 19,440B 落 18~20KB 带取 2 条（勘察 A1/A2；A3..A7 备选裁剪序尾向头勿倒）；
    // 扩池后账本实测复算（tools/ledger_recheck.py）：总基权 1070→1076（本批
    // +6=2×w3）；稀有 28 条 48 权 ≈4.46%（原注 4.49% 系未复算 stale，本轮
    // low 清账——48/1076=4.46%）；恶缘 68 条 240 权不变；常态池
    // （w>0）mr 分布（316 条）0:44/1:63/2:54/3:37/4:38/5:32/6:26/7:22
    (349, 3, 7, true),  // 结绳记事（老年×伴双门控第 12 条，mr7 补开——双门控
                        // 首条 mr7，Comp 立绘照 344 模板不入白名单）
    (350, 3, 6, true),  // 云隙借光（云门 weather==1 第 5 面收官——五天象门控
                        // 雨/灵潮/雾/晴/云全开，mr6 善散件）
    // v1.16.35 单3 扩池批 2 条（段十一，N_EVENTS 351→353）：按单1 减包后余量
    // 27,030B 落 ≥20KB 带取 2 条（勘察 A1/A2；A3..A7 备选裁剪序尾向头勿倒）；
    // 扩池后账本实测复算（tools/ledger_recheck.py）：总基权 1076→1080（本批
    // +4=1×w3+1×w1）；稀有 29 条 49 权 ≈4.54%（v1.16.34 批注 4.49% 系未复算
    // stale——48/1076=4.46%，本批 w1 入池后 49/1080=4.54% 如实记账）；恶缘
    // 68 条 240 权不变；常态池（w>0）mr 分布（318 条）
    // 0:44/1:63/2:54/3:37/4:38/5:33/6:27/7:22
    (351, 3, 5, true),  // 共拓崖铭（伴随白名单第 32 条，mr5 有伴第 3 条——白名单
                        // mr5 仅 211/283 两条最薄层摊薄，Comp 立绘照 283/344 模板）
    (352, 1, 6, true),  // 苔阶履痕（稀有 w1：mr6 稀有第 3 条照 213/324 层位，散件 special 恒 0）
    // v1.16.36 单3 扩池批 3 条（段十二，N_EVENTS 353→356）：按单1 预建快照
    // +单2 零改动后余量 25,233B 落 ≥25KB 带取 3 条（勘察 A1/A2/A3）；
    // 扩池后账本实测复算（tools/ledger_recheck.py）：总基权 1080→1089（本批
    // +9=3×w3）；稀有 29 条 49 权不变 ≈4.50%（49/1089 如实记账）；恶缘
    // 68→69 条 240→243 权 ≈22.3%；常态池（w>0）mr 分布（319 条）
    // 0:45/1:63/2:54/3:37/4:38/5:33/6:28/7:23
    (353, 3, 7, true),  // 岭上补松（mr7 善散件 w3——mr7 常态池 22 条 66 权全池
                        // 最薄层摊薄，题材避醉道人系 222/325/338 与链 18 余波 270）
    (354, 3, 6, false), // 器铺走水（mr6 恶散件 w3——恶缘 68→69 条；题材避
                        // 343 老炉现纹/298 灵脉被截）
    (355, 3, 0, true),  // 磨镜还光（老年×伴双门控第 13 条，mr0 零覆盖补缺——
                        // 双门控 12 条 mr1×5/mr2/mr3/mr4×2/mr5/mr6/mr7 各 1 后
                        // mr0 层空缺；不入白名单，Comp 立绘照 351 模板）
    // v1.16.37 单2 扩池批 3 条（段十三，N_EVENTS 356→359）：356 哑缶一声（稀
    // 有 w1 mr2 special76 检定）/357 同扎一篱（伴随白名单 32→33 mr4）/358 灶膛
    // 煨薯（双门控第 14 条 mr3）；扩池后账本实测复算（tools/ledger_recheck.py）：
    // 总基权 1089→1096（本批 +7=1×w1+2×w3）；稀有 29→30 条 49→50 权 ≈4.56%
    //（50/1096 守 ~4.5% 带）；恶缘 69 条 243 权不变；常态池（w>0）mr 分布
    //（324 条）0:45/1:63/2:55/3:38/4:39/5:33/6:28/7:23
    // v1.16.38 单2 勘误回填：上批账目原覆写在 v1.16.36 块头上未另立块头（git
    // HEAD~1 对照取证，本轮恢复 v1.16.36 原账并另立本块）；原计数「322 条」与
    // 逐层相加不符——系 v1.16.36 块「319 条」（逐层实 321）起连续未复算 stale，
    // 324=45+63+55+38+39+33+28+23 如实记账
    (356, 1, 2, true),  // 哑缶一声（稀有 w1 mr2 善，special76 道心检定——检定窗
                        // 75..=76 最后余窗启用收官，检定臂照 329 模板；题材避
                        // 32 青灯三问/33 墨叟一问/34 拦问纸人/66 旧砚一问/69
                        // 松风问琴/73 空谷叩问/74 枯莲夜雨/75 檐马问风：哑缶
                        // 自鸣是裂口瓦缶的自鸣一问。原拟「哑磬一声」磬=cmap
                        // 外字（全池 0 出现）改写缶——N361 击缶仅伴奏引用不撞
                        // 自鸣义，不扩字体纪律）
    (357, 3, 4, true),  // 同扎一篱（伴随白名单 32→33 条 mr4——mr4 伴随 225/240/
                        // 257 三条后六批未落最陈薄层；题材避 321 同封新茶/327
                        // 灯下拼卷/344 同裱旧稿/351 共拓崖铭/82 缝补旧稿/190
                        // 旧履还坊：扎篱是枝条来递的共作手作面。原拟「织机
                        // 共梭」梭/纺=cmap 外字改写篱）
    (358, 3, 3, true),  // 灶膛煨薯（老年×伴双门控第 14 条，mr3——双门控 13 条
                        // mr1×5/mr2/mr3/mr4×2/mr5/mr6/mr7/mr0 各 1 后 mr3 层
                        // 322 后六批未落；不入白名单，Comp 立绘照 355 模板、
                        // 徽标走老年臂 gallery 同批登记；煨/薯/灶池内字 0 撞，
                        // 215 灶火留谱是家传菜谱面不同义）
    // v1.16.38 单2 扩池批 3 条（段十四，N_EVENTS 359→362）：余量 101,187B 落
    // ≥40KB 带取 3 条全做（勘察三 3 主提名：359 伴随白名单第 34 条/360 恶缘
    // mr4/361 双门控第 15 条，2 备未启用）；扩池后账本实测复算
    //（tools/ledger_recheck.py）：总基权 1096→1105（本批 +9=3×w3）；稀有 30 条
    // 50 权不变 ≈4.52%（50/1105 守 ~4.5% 带）；恶缘 69→70 条 243→246 权；
    // 常态池（w>0）mr 分布（327 条）0:45/1:63/2:55/3:38/4:40/5:34/6:28/7:24
    (359, 3, 7, true),  // 移竹当窗（伴随白名单 33→34 条 mr7——mr7 有伴 241/300/
                        // 344 三条后四批未落最陈薄层，mr7 常态池 23→24 条同批
                        // 摊薄；Comp 立绘照 351/357 模板；题材避 241 踏云题字/
                        // 300 晨课互校/344 同裱旧稿/357 同扎一篱：移竹当窗是
                        // 庭前移栽的共作造景面，题面「竹」全池 0 卡——175 伞
                        // 骨正文/348 竹篮简语仅 incidental 引用）
    (360, 3, 4, false), // 灵仓亏空（mr4 恶散件 w3——恶缘 69→70 条 243→246 权
                        // 照 345 模板，mr4 恶 316 后八批未落最陈层；题材避
                        // 343 老炉现纹/298 灵脉被截/345 井绳朽断/354 器铺走水：
                        // 灵仓亏空是管存账面的亏空追赔面，走水是火毁意外面
                        // 不同义；散件不挂脸）
    (361, 3, 5, true),  // 熏笼共暖（老年×伴双门控第 15 条，mr5——双门控 mr5 层
                        // 297 后十批未落最陈；不入白名单，Comp 立绘照 355/358
                        // 模板、徽标走老年臂 gallery 同批登记；题材避 358 灶膛
                        // 煨薯：熏笼是香道器物暖面，灶膛煨薯是灰里煨食面不同
                        // 义；熏/笼池内仅 70/250/354 臂 incidental 引用不撞）
    // v1.16.40 单3 扩池批 3 条（段十五，N_EVENTS 362→365）：余量 96,921B 落
    // ≥40KB 带取 3 条全做（勘察四 3 主提名：362 伴随白名单第 35 条/363 双门控
    // 第 16 条/364 恶缘第 71 条，2 备未启用）；扩池后账本实测复算
    //（tools/ledger_recheck.py）：总基权 1105→1114（本批 +9=3×w3）；稀有 30 条
    // 50 权不变 50/1114≈4.49%（v1.16.33 4.485%/v1.16.34 4.46% 先例可容，勘察四
    // 风险 7 留档）；恶缘 70→71 条 246→249 权；常态池（w>0）mr 分布（330 条）
    // 0:45/1:63/2:56/3:38/4:40/5:34/6:29/7:25
    (362, 3, 6, true),  // 芦絮暖枕（伴随白名单 34→35 条 mr6——mr6 伴随 225/293/
                        // 339 三条后八批未落最陈薄层（339 v1.16.32 后未落），
                        // mr6 常态池 28→29 条同批摊薄；Comp 立绘照 351/357/359
                        // 模板；第三副本 tools/ledger_recheck.py 同批；题材避
                        // 359 移竹当窗（竹≠芦、移栽≠絮枕）/297 重缝冬衣（絮枕
                        // 非缝衣）/226 霜晨共伞/210 雨檐听铃：芦絮暖枕是滩头
                        // 芦花采絮缝枕的秋声面，题面「芦」全池 0 卡——简语原
                        // 拟「霜洲」洲=cmap 外字改「秋声」，缝字 cmap 实证在字
                        // 与 297 题材面不同义）
    (363, 3, 2, true),  // 同修暖榻（老年×伴双门控第 16 条，mr2——双门控 mr2 层
                        // 仅 299 一条（v1.16.28 后十三批未落）并列最薄且最陈；
                        // 不入白名单，Comp 立绘照 355/358 模板、徽标走老年臂
                        // gallery 同批登记；题材避 297 重缝冬衣（挪榻共修非缝
                        // 补）/358 灶膛煨薯（炉边是暖处面非灰里煨食面）/361 熏
                        // 笼共暖（暖字共用器物不同义）/299 合誊旧稿（同修摊稿
                        // 非誊合为一卷））
    (364, 3, 7, false), // 山摇地动（mr7 恶散件 w3——恶缘 70→71 条 246→249 权
                        // 照 360 模板，mr7 恶 345 后七批未落（mr7 层 5 条全池
                        // 最薄）、mr7 常态池 24→25 条同批摊薄；题材避 354 器铺
                        // 走水（火毁）/360 灵仓亏空（账面）/345 井绳朽断（物故
                        // 小件）/43 沙暴藏窟（mr2 洞窟机遇）：山摇地动是地动
                        // 毁路灾后面不同义；散件不挂脸；简语原拟「簌簌/纷纷」
                        // cmap 外字改「梁尘落案」）
    // v1.16.43 单3 扩池批 3 条（段十六，N_EVENTS 365→368）：余量 92,077B 落
    // ≥40KB 带取 3 条全做（勘察四 3 主提名：365 恶缘 mr0 最陈薄层/366 伴随白名
    // 单第 36 条 mr3/367 双门控第 17 条 mr6，2 备未启用）；扩池后账本实测复算
    //（tools/ledger_recheck.py）：总基权 1114→1123（本批 +9=3×w3）；稀有 30 条
    // 50 权不变 50/1123≈4.45%（v1.16.37 4.56%/v1.16.40 4.49% 带内）；恶缘
    // 71→72 条 249→252 权；常态池（w>0）mr 分布（333 条）
    // 0:46/1:63/2:56/3:39/4:40/5:34/6:30/7:25
    (365, 3, 0, false), // 院墙夜塌（mr0 恶散件 w3——mr0 恶 290 误入猎套（段一末
                        // id）后十四批未落全表最陈薄层；散件不挂脸照 364 口径；
                        // 题材避 63 溪桥塌半边/2 禁制半塌/277 雪隘坡道塌/364 山
                        // 摇地动/345 井绳朽断/354 走水/360 亏空：院墙夜塌是宅
                        // 墙年久失修自坠面不同义（正文禁风雨/地动成因——天气门
                        // 五面收官、364 地动刚落）；「墙塌/墙倒/坊墙」全池 0；
                        // 简语原拟「凌/棱/晶」cmap 外字改「冰柱」沿 356 磬→缶
                        // 不扩字体纪律）
    (366, 3, 3, true),  // 共扫叶径（伴随白名单 35→36 条 mr3——mr3 伴随 210/240/
                        // 321 三条后十批未落全表最陈薄层（321 v1.16.30 后未落），
                        // mr3 常态池 38→39 条同批摊薄；Comp 立绘照 351/357/359/
                        // 362 模板；第三副本 tools/ledger_recheck.py 同批；题材
                        // 避 352 苔阶履痕（阶上履痕面）/309 雪堤喂雀/314 檐下折
                        // 梅（冬日面）/272 寒江共网（共作渔面）/317 崖灯照径
                        //（径字题名唯一他卡不同义）：共扫叶径是一把小帚扫出
                        // 一径秋面，「叶径/小帚」全池 0 卡；简语原拟「共糊纸
                        // 窗」撞 291 换窗新纸、「纸鸢」撞 217 断线纸鸢已否）
    (367, 3, 6, true),  // 檐冰垂光（老年×伴双门控第 17 条，mr6——双门控 mr6 层
                        // 仅 348 一条（v1.16.33）后七批未落最陈且并列最薄；不
                        // 入白名单，Comp 立绘照 355/358 模板、徽标走老年臂
                        // gallery 同批登记；题材避 212 相扶看雪（雪面）/301 霜
                        // 窗描枝（霜窗画枝面）/348 霜橘分甘/361 熏笼共暖/363
                        // 同修暖榻/145 冰窖遗鲜（冰字题名唯一他卡藏鲜面不同
                        // 义）：檐冰垂光是檐口冰柱晨光透亮面（天时底色可述照
                        // 212 看雪先例，禁写成「天气变了」）；原拟「凌/棱/晶」
                        // cmap 外字改「冰柱」）
    // v1.16.44 扩池批 3 条（段十七，N_EVENTS 368→371）：余量带取 3 条全做（勘察
    // 四 3 主提名：368 双门控第 18 条 mr7/369 special77 检定 w1 mr2/370 伴随白名
    // 单 36→37 第 37 条 mr0，2 备未启用）；扩池后账本实测复算
    //（tools/ledger_recheck.py）：总基权 1123→1130（+7=1×w1+2×w3）；稀有 30→31
    // 条 50→51 权 51/1130≈4.51%（v1.16.37 4.56%/v1.16.40 4.49% 带内）；恶缘
    // 72 条 252 权不变；常态池（w>0）mr 分布（336 条）
    // 0:47/1:63/2:57/3:39/4:40/5:34/6:30/7:26
    (368, 3, 7, true),  // 暖阁晒背（老年×伴双门控第 18 条，mr7——双门控 mr7 层
                        // 349 后九批未落全表最陈薄层，mr7 常态池 25→26 条同批
                        // 摊薄；不入白名单，Comp 立绘照 355/358 模板、徽标走老
                        // 年臂 gallery 同批登记；题材避 212 相扶看雪（雪面）/
                        // 361 熏笼共暖（暖器面）/363 同修暖榻（榻面）/367 檐冰
                        // 垂光（檐冰面）：暖阁晒背是冬日晌午阁内轮流晒背闲话
                        // 面（天时底色可述照 212 看雪先例，禁写成「天气变了」）；
                        // 「暖阁/晒背」组合全池 0——「晒」字面撞 346 晒药/198
                        // 晒卷不同义照 365「径」字面共存先例注明；原拟「共剪
                        // 窗花」剪=cmap 外字沿 356 磬→缶不扩字体纪律改晒背）
    (369, 1, 2, true),  // 漏刻无声（special77 检定 w1 mr2 善——检定窗 75..=76
                        // 收官后下一窗，v1.16.38/40/43 三批未落检定卡照 329/356
                        // 三批一条频率轮到；稀有 30→31 条；走新检定不触链互斥
                        // 照 356 口径——apply_eff 新臂+副标「漏停一问」手写行
                        // N2783；「漏刻/滴漏/更漏/铜壶」全池 0；「无声」字面
                        // 撞 212 简语「落雪无声」不同义照 365「径」共存先例；
                        // stage 恒 None 照 329/356 None 先例）
    (370, 3, 0, true),  // 同制新历（伴随白名单 36→37 第 37 条，mr0——mr0 伴随
                        // 327 后十二批全表最陈薄层，mr0 常态池 46→47 条同批摊
                        // 薄（含伴口径）；Comp 立绘照 351/357/359/362 模板；第
                        // 三副本 tools/ledger_recheck.py 同批；题材避 183 灯下
                        // 习字/226 霜晨共伞/327 灯下拼卷/321 同封新茶：同制新
                        // 历是岁首合制历书旧页撕新页写面，「新历/历书/岁首」全
                        // 池 0——「历」字面撞来历/历年复合词不同义照 365 先例；
                        // 原拟「同订历书」订撞主线书成系统「合订」面改「制」）
    (371, 3, 7, false), // 牒案翻旧（v1.16.45 施工单三段十八：mr7 恶散件——恶缘
                        // mr7 层 6 条全恶层最薄+mr7 常态池 26 条全表次薄（364
                        // 后两批未落）；文狱职牒线深化（勘察五主1）；总基权
                        // 1130→1139 第三副本 tools/ledger_recheck.py 同批；题
                        // 避 191 文狱退牒（勘察五弃「文狱翻牒」改「翻旧」面））
    (372, 3, 6, true),  // 堂前领差（mr6 善散件——善散件 350 后九批未落、mr6 层
                        // 30 条次薄；玄诚长老第九卡 CardActor::Npc(0) 照 337
                        // 先例（勘察五主2，正文钉名⟺Npc(0) 同源纪律）；门中
                        // 差事 v1.16.40 comm 系统叙事深化，纯叙事读既有态零
                        // 新键；非道侣门控非老年非天气照红线 1 避让）
    (373, 3, 5, true),  // 残页对笔（mr5 善散件——善散件 330 后十批未落、mr5 层
                        // 34 条；书稿/故人线深化（勘察五主3）：旧稿堆认出故
                        // 人早年批注笔迹隔页对答；正文照施工单三③口径 2 段
                        // （反斜杠 n 分段）放宽卡面行数（tall 版式零新代码））
    (374, 3, 3, true),  // 三纸同檐（v1.16.46 段十九：mr3 善——mr3 层 331 后
                        // 十三批未落全表最陈薄层；勘察二主1 副伴专属卡——多道
                        // 侣系统 v1.16.45 落地后 build_event 区 comp_extra 零
                        // 引用剧情空位补面；门控走 sim_event_weights 新增
                        // comp_extra 分支（无副伴不入池，语义准——勘察二两案
                        // 权衡弃白名单：白名单口径是主伴在场，副伴位空时入池
                        // 会空谈副伴；段十八红线 1「非道侣门控避让」属段十八
                        // 槽位纪律不束本批，门控批先例 74/75/85/110/111 同款独
                        // 立 if）；Comp 立绘=副伴反查照 351/357 模板；总基权
                        // 1139→1148 第三副本 tools/ledger_recheck.py 同批；题
                        // 材避 28 同页天成/N2795「同页仍止三纸」（另缝一页入
                        // 檐誊卷面，不触结缘面））
    (375, 3, 7, false), // 同页生隙（v1.16.46 段十九：mr7 恶散件——恶缘 mr7 层
                        // 371 后未落 7 条全恶层最薄+mr7 常态池 27 条全表最薄
                        // （勘察二主2 争风面 0 占位）；无门控散件——题材写成
                        // 旁人同页道侣当街生隙拉玩家评理，无伴玩家可读；总基
                        // 权同批 1139→1148；「评理」字面撞 715 评理钱（彼收钱
                        // 旁观此被拽评理）不同义共存照 365「径」先例；与 28
                        // 同页天成成对面）
    (376, 3, 4, true),  // 空阶一蝶（v1.16.46 段十九：mr4 善散件——mr4 层 357
                        // 后九批未落次陈薄层；勘察二主3 蝶意象全池 0 占位全新
                        // 面（新字形蝶 1 个照 356 不扩字纪律注：意象本字无法
                        // 避，make_font 自动收）；special78 检定窗照勘察二 A
                        // 案建议让给下批（356/369 三批一条频率），本卡纯善散
                        // 件；总基权同批 1139→1148）
    (377, 1, 2, true),  // 晒书使巡（v1.16.47 段二十：special78 道心检定 w1 mr2
                        // 善——检定窗 77 收官后下一窗，356/369 三批一条频率轮
                        // 到；照勘察二 A 案入稀有，稀有 31→32 条 51→52 权；题
                        // 材照勘察二提名兑现 GameBook 留档「晒书使巡」官制缺口
                        // ——文枢司晒书使巡曝书会抽勘卷来历，勘毕朱点入册暗钩
                        // 收册人终卷线（只暗钩不组队，收册人仍终卷独占）；走新
                        // 检定不触链互斥照 356 口径——apply_eff 新臂+SPECIAL_SUB
                        // 槽 78 副标「卷面候勘」照 76/77 手写行先例；总基权同批
                        // 1148→1155）
    (378, 3, 6, true),  // 巷尾淘井（v1.16.47 段二十：mr6 善散件——mr6 层 372
                        // 堂前领差后一批未落；无门控散件无新机制臂；题材井泉
                        // 意象与 377 书卷面错开；总基权同批 1148→1155）
    (379, 3, 7, false), // 夜渡加价（v1.16.47 段二十：mr7 恶散件——恶 mr7 层
                        // 371/375 后每批一条节奏第三条全恶层最薄维持；石罚
                        // 3000×ss 照 371/375 mr7 案值厚先例（stone_scale 乘
                        // 区）；「加价」字面撞市价类文案不同义共存照 365「径」
                        // 先例；总基权同批 1148→1155）
    (380, 3, 5, true),  // 补行代笔（v1.16.48 段二十一：mr5 善散件——mr5 层 373
                        // 残页对笔后两批未落；题材照 GameBook B 表①留档「补行
                        // 代笔」兑现，故人批注线续 373；无门控散件无新机制臂；
                        // special79 检定窗三批一条节奏（76/77/78）未轮到留档，
                        // 照勘察二裁定全散件；总基权同批 1155→1164）
    (381, 3, 6, true),  // 页脚账浅（v1.16.48 段二十一：mr6 善散件——mr6 层 378
                        // 巷尾淘井后连批续落；题材照 GameBook B 表②留档「页脚
                        // 账浅」兑现，§3.4 方印「这不是印，是账」线散件呼应不
                        // 碰六牒主链；无门控散件无新机制臂；总基权同批 1155→1164）
    (382, 3, 7, false), // 退稿索酬（v1.16.48 段二十一：mr7 恶散件——恶 mr7 层
                        // 371/375/379 后每批一条节奏第四条全恶层最薄维持；石罚
                        // 3000×ss 照 371/375/379 mr7 案值厚先例（stone_scale 乘
                        // 区）；「索酬」字面撞市价类文案不同义共存照 365「径」
                        // 先例；总基权同批 1155→1164）
    (383, 3, 1, true),  // 描名起稿（v1.16.49 段二十二开设：mr1 善散件——mr1 层
                        // 310 后约十九批未落全表最陈薄层（勘察二 ledger derive
                        // 提名）；题材照 GameBook B 表③留档「描名起稿」兑现，
                        // §3.4 十万稿线散件；无门控散件无新机制臂；special79 检
                        // 定窗三批一条节奏续未轮到留档；总基权同批 1164→1173）
    (384, 3, 3, true),  // 檐下晒卷（v1.16.49 段二十二开设：mr3 善散件——mr3 层
                        // 374 三纸同檐后三批未落；玄诚长老第十卡
                        // CardActor::Npc(0) 照 372 先例（正文钉名⟺Npc(0) 同源
                        // 纪律），散件呼应 N1699 藏经阁半卷暗钩不触书狱主链；
                        // 「晒卷」与 377 晒书使巡共用「晒」字面不同义（门内晒
                        // 卷防潮 vs 官制巡曝抽勘）照 365「径」先例；总基权同批
                        // 1164→1173）
    (385, 3, 7, false), // 废稿盗售（v1.16.49 段二十二开设：mr7 恶散件——恶 mr7
                        // 层 371/375/379/382 后每批一条节奏第五条全恶层最薄维
                        // 持；石罚 3000×ss 照 371/375/379/382 mr7 案值厚先例
                        //（stone_scale 乘区）；「盗售」字面与 371 打点/382 索
                        // 酬不同义共存照 365「径」先例；总基权同批 1164→1173）
    (386, 1, 2, true),  // 收册人问（v1.16.50 段二十三开设：special79 道心检定
                        // w1 mr2 善——检定窗 78 收官后下一窗，三批一条节奏
                        // 76@v1.16.37/77@v1.16.44/78@v1.16.47，78+3 批恰本批
                        // 轮到，照 356/369/377 三先例全 w1 mr2 入稀有；题材照
                        // GameBook B 表余留唯一候选「收册人问」兑现——晒书使
                        // ×收册人组队问，两官合卷一问玩家，只问不留队，收册
                        // 人终卷独占不动照 data.rs:2920 口径；副标「合卷一问」
                        // N2885 手写行照 76/77/78 先例；总基权同批 1173→1180）
    (387, 3, 4, true),  // 笔冢拾毫（v1.16.50 段二十三开设：mr4 善散件——mr4
                        // 层 376 空阶一蝶后三批未落次陈薄层（勘察二提名）；
                        // 题材自拟照勘察二兜底（GameBook B 表余留仅 special79
                        // 材已兑 386，无散件候选）——笔冢封土意象与蝶/书卷稿
                        // 面错开；无门控散件无新机制臂；「拾毫」字面撞 376 空
                        // 阶一蝶（彼拾阶此拾毫）不同义共存照 365「径」先例；
                        // 总基权同批 1173→1180）
    (388, 3, 7, false), // 讹捐封口（v1.16.50 段二十三开设：mr7 恶散件——恶 mr7
                        // 层 371/375/379/382/385 后每批一条节奏第六条全恶层最
                        // 薄维持；石罚 3000×ss 照 371/375/379/382/385 mr7 案
                        // 值厚先例（stone_scale 乘区）；题材自拟照勘察二兜底
                        // （同 387 口）——假香客代捐吞钱反咬讹人，钱财勒索类
                        // 但对象是功德捐资与 371 牒案/382 退稿/385 盗售不同义
                        // 共存照 365「径」先例；总基权同批 1173→1180）
    (389, 3, 5, true),  // 闲章一方（v1.16.51 段二十四开设：mr5 善散件 w3——
                        // mr5 层 380 补行代笔后两批未落；题材=书道八宝·闲章
                        // 意象（text_source N2028/N2233 在册呼应，零触碰
                        // RELICS 主链），坊市摊得旧闲章印文漫灭自奏刀补刻；
                        // special80 检定窗 79 后第一批/三批——三批一条节奏
                        // 76@v1.16.37/77@v1.16.44/78@v1.16.47/79@v1.16.50，
                        // 预留约 v1.16.53 段二十六批落，题材待勘察提名，照勘
                        // 察二裁定本批三卡全散件；无门控散件无新机制臂；总基
                        // 权同批 1180→1189；正文照施工单三③口径 2 段（tall
                        // 版式零新代码））
    (390, 3, 6, true),  // 漏行残诀（mr6 善散件 w3——mr6 层 381 页脚账浅后
                        // 两批未落常态池最小善层；题材=功法八部《漏行书》残
                        // 卷意象（text_source N2009/N2109 在册呼应零新键），
                        // 旧书摊虫蛀残卷虫道恰补断句；special80 未到留档同
                        // 389 口径；无门控散件无新机制臂；总基权同批
                        // 1180→1189）
    (391, 3, 7, false), // 押卷取赎（mr7 恶散件 w3——恶 mr7 层 371/375/379/
                        // 382/385/388 后每批一条节奏第七条；石罚 3000×ss 照
                        // 388 同款先例（stone_scale 乘区）；题材=当铺把寄存
                        // 书卷作死当索翻倍赎金否则拆页零卖，钱财勒索类但对
                        // 象是典押赎回与 371 牒案/382 退稿/385 盗售/388 讹捐
                        // 不同义共存照 365「径」先例；special80 未到留档同
                        // 389 口径；总基权同批 1180→1189）
    (392, 3, 4, true),  // 废稿拟声（v1.16.52 段二十五开设：mr4 善散件 w3——
                        // mr4 层 376 空阶一蝶后五批未落最陈薄层；题材=GameBook
                        // §四候选 b 十万稿线废稿拟人面兑现（lore N186 论道十万
                        // 废稿/N374 废稿市描名摊/N1613「十万稿废纸哗哗作响」在
                        // 册呼应，与 383 描名起稿/385 废稿盗售错开——盗售是钱
                        // 财本卡是废稿夜诵灵异善缘）；special80 检定窗 79 后第
                        // 二批/三批——三批一条节奏 76@v1.16.37/77@v1.16.44/
                        // 78@v1.16.47/79@v1.16.50，预留约 v1.16.53 段二十六批
                        // 落，题材待勘察提名，照勘察二裁定本批三卡全散件；无门
                        // 控散件无新机制臂；总基权同批 1189→1198；正文照施工
                        // 单三③口径 2 段（tall 版式零新代码））
    (393, 3, 3, true),  // 洗笔澄心（mr3 善散件 w3——mr3 层 384 檐下晒卷后两批
                        // 未落次陈；题材=文房案头·笔洗意象零新键（text_source
                        // N634「把笔洗净」近义呼应），笔洗水浑映字影；special80
                        // 未到留档同 392 口径；无门控散件无新机制臂；总基权同
                        // 批 1189→1198）
    (394, 3, 7, false), // 假帖索润（mr7 恶散件 w3——恶 mr7 层 371/375/379/
                        // 382/385/388/391 后每批一条节奏第八条；石罚 3000×ss
                        // 照 388/391 同款先例（stone_scale 乘区）；题材=伪造仙
                        // 门法帖上门索拓帖润资，钱财勒索类但对象是法帖鉴伪与
                        // 371 牒案/382 退稿/385 盗售/388 讹捐/391 押卷不同义
                        // 共存照 365「径」先例；special80 未到留档同 392 口
                        // 径；总基权同批 1189→1198）
    (395, 1, 2, true),  // 朱点回访（v1.16.53 段二十六开设：special80 道心检定
                        // w1 mr2 善——检定窗 79 收官后下一窗，79+3 批恰本批
                        // 轮到，照 356/369/377/386 四先例全 w1 mr2 入稀有；题
                        // 材照 GameBook:180 候选 a 官制线「朱点回访」兑现——
                        // 晒书使来年复查朱点卷，386 过臂 N2888「一行名姓从此
                        // 两册同录」来年伏笔顺延；收册人终卷独占不动）
    (396, 3, 6, true),  // 笔架山影（mr6 善散件 w3——mr6 层 390 漏行残诀后两
                        // 批未落，常态池 34 条仍最小善层（mr5 37 条次之）照
                        // ledger_recheck derive；文房案头·笔架山子意象零新键）
    (397, 3, 7, false), // 次纸充宣（mr7 恶散件 w3——恶 mr7 371/375/379/382/
                        // 385/388/391/394 后每批一条节奏第九条；石罚 3000×ss
                        // 照 388/391/394 同款先例（stone_scale 乘区）；题材=
                        // 纸行以次纸充宣纸勒收宣价，钱财勒索类但对象是纸张
                        // 以次充好，与 371 牒案/382 退稿/385 盗售/388 讹捐/
                        // 391 押卷/394 假帖不同义共存照 365「径」先例）
];

const fn pack_ev(r: (u16, u8, u8, bool)) -> u16 {
    ((r.1 as u16) & 0x3F) | (((r.2 as u16) & 7) << 6) | ((r.3 as u16) << 9)
}

const fn pack_events() -> [u16; N_EVENTS] {
    let mut out = [0u16; N_EVENTS];
    let mut k = 0;
    while k < N_EVENTS {
        out[k] = pack_ev(EVENTS_ROWS[k]);
        k += 1;
    }
    out
}

/// 运行期事件表（刀⑥打包位域；行定义见上 EVENTS_ROWS——const-eval 打包不落 data 段）。
/// v1.16.28 减包刀B：裁 id 列 [u16;N]（309×2B，省 598B data 段）——id==下标由
/// EVENTS_ROWS 逐行钉测（event_ids_unique_and_names_distinct 全表 + batch_tables_aligned
/// 分段）钉死，读点一律下标取值（下标即 id）；ev_w/ev_mr/ev_good 解码面零变
pub static EVENTS: [u16; N_EVENTS] = pack_events();

/// 位域解码读点（刀⑥）：权重（低 6bit，0..=63）
#[inline]
pub fn ev_w(p: u16) -> i32 {
    (p & 0x3F) as i32
}

/// 位域解码读点（刀⑥）：最低境界 mr（bit6..8，0..=7）
#[inline]
pub fn ev_mr(p: u16) -> u8 {
    ((p >> 6) & 7) as u8
}

/// 位域解码读点（刀⑥）：善缘位（bit9）
#[inline]
pub fn ev_good(p: u16) -> bool {
    p & 0x200 != 0
}

// ---------------- v1.16.10 事件回顾图鉴：静态名/简一一表 ----------------
// v1.16.30 单1 主刀②：EVENT_NAMES[id] 即 build_event 卡标题唯一来源——构造行
// `title: EVENT_NAMES[id].into()` 查表，原 317 臂 `card.title="…".into()` 字面量
// 已删（双源归一；链内段含「·续」后缀、宿敌/老年等动态文案臂的标题同为静态表行，
// 动态只在正文）。手术前 317 臂与名段逐 id 全等 0 不符的对账证据与术后静态钉
// （臂内禁 card.title 赋值 + 名段覆盖断言）常驻 tools/title_recheck.py；
// 运行期守门（臂齐全+全 id 非空）仍在 game.rs event_names_match_card_titles。
// v1.16.27 刀②/③：两表本体收编进 textpack（下 EvTable 查表器），相等性由「静态
// 两表」改「运行期比较」——测试在 blob 装载后逐 id 对账，语义面零变。
// EVENT_BRIEF[id] 为图鉴『遭遇手札』行的展示简介（一行 ≤18 字，display-only 无
// 对账约束；未遇事件整行 ？？？ 不显简介不泄题）。
// v1.16.24 简语画面化第三批 10 条（id 55/58/135/136/139/141/143/154/200/216）：
// 记账腔改画面体（有场景有动作有实物，选项对仗照旧行尾「或」结构），每条改写
// 均与 build_event 臂正文逐条对过防跑题；净 −6 字（wasm 内嵌 &str 常量，字节
// 净减），0 新增字体字符。简语预算钉测见 game.rs event_brief_11624_batch_within_18。
// v1.16.25 第四批 10 条（10/13/56/100/165/166/209/235/242/258，清账批改写）
// 同口径零新字形，钉测 event_brief_11625_batch_within_18（勘察所列最平 10 条
// 至此清完）。
// v1.16.26 第五批 12 条（12/37/38/40/53/55/58/84/109/110 记账腔改画面体 +
// 275/278 简文对齐 nit——275 远行→闭关百日、278 赴宴载笔→杯底岁星砂）：
// ≤18 字预算照守，kw 取词自对应臂原文（53 煮茶出自臂内选项「煮茶听雪」），
// 零新字形，钉测 event_brief_11626_batch5_within_18。
// v1.16.27 第六批 8 条（269/216/135/134/277/280/223/249 记账/状态腔改画面体，
// 减包协同批 −30B raw）——两表已收编 textpack，改写走真相源行直改（N1380/
// 1327/1246/1245/1388/1391/1334/1360），≤18 字预算照守，kw 取自对应臂 evtx 行
// verbatim，零新字形，钉测 event_brief_11627_batch6_within_18。
// v1.16.28 第七批 10 条（4/8/39/68/79/133/169/194/254/289 改写即减字优先，
// 合计 −15B raw；N1115/1119/1150/1179/1190/1244/1280/1305/1365/1400 行直改
// N 永不变号）——「或」干句尾全清，kw 取自对应臂 evtx 行 verbatim，零新
// 字形，钉测 event_brief_11628_batch7_within_18。
// v1.16.27 减包刀②/③：两表 291×2 条 &str 常量收编进 textpack（pack_text.py --tables
// 连号段，基址见 text_offsets.inc：TXT_NAME_BASE=820、TXT_BRIEF_BASE=1111）；扩池批
// 名/简连号追加为段二/段三/段四（基址 TXT_NAME2_BASE=1402/TXT_BRIEF2_BASE=1410/
// TXT_NAME3_BASE=1442/TXT_BRIEF3_BASE=1452/TXT_NAME4_BASE=1825/TXT_BRIEF4_BASE=
// 1833——v1.16.29 段四 8 条，基址因单1/单2 用 N1492..1824 顺延），表本体
// 换查表器：.get(i)/[id]/len() 调用面零改（gallery.rs 手札/图鉴行渲染仍零分配——
// evtx O(1) 切片，仅每行多一次 OnceLock 原子 load，纳秒级）。行内容零变化（脚本
// 往返断言 + game.rs event_names_match_card_titles 运行期全 id 对账——双源漂移防线
// 由「静态两表」改「运行期比较」）；display-only 不进存档，零存档面。N 追加不变号
// → 基址稳定；重跑 pack_text.py 幂等（基址回读自上次产出的 .inc）。
// v1.16.32 单1：EvTable 字段 usize→u16 窄化（static 两实例 rdata 224B→56B，
// 纯数据位宽等价——段界常量差口径照旧；段长/基址现值与可预见未来均 < 65535，
// TXT_DELTA u16 同款位宽口径）
pub struct EvTable {
    base: u16,
    len: u16,
    // v1.16.27 扩池起：段一（v1.16.26 末态 291 条）满员后，新批名/简连号追加为
    // 段二（textpack N 段二基址 TXT_NAME2_BASE/TXT_BRIEF2_BASE 由 pack_text.py
    // 登记与产出，append-only 旧 N 不变号）；get/len/iter/Index 四面对多段连寻
    //（v1.16.28 起扩三段）
    base2: u16,
    len2: u16,
    // v1.16.28 扩池起：段二（v1.16.27 末态 8 条）满员后，新批连号追加为段三
    //（TXT_NAME3_BASE/TXT_BRIEF3_BASE 同款登记与产出）；四面同步扩三段连寻。
    // len2/len3 一律取段界常量差（EV_SEG2−EV_SEG1 / EV_SEG3−EV_SEG2）——
    // len2 写 N_EVENTS−EV_SEG1 会随 N_EVENTS 增长静默错位（见 EV_SEG2 注）
    base3: u16,
    len3: u16,
    // v1.16.29 扩池起：段三（v1.16.28 末态 10 条）满员后，新批连号追加为段四
    //（TXT_NAME4_BASE/TXT_BRIEF4_BASE 同款登记与产出；单1/单2 已用 N1492..1824，
    // 段四基址顺延 1825/1833）；连寻面数随段同步（v1.16.29 当时四面，v1.16.30
    // 起五面）。len4 取段界常量差
    // EV_SEG4−EV_SEG3（勿写 N_EVENTS−EV_SEG3——N_EVENTS 增长会静默吞掉段五
    // 行，见 EV_SEG4 注）
    base4: u16,
    len4: u16,
    // v1.16.30 扩池起：段四（v1.16.29 末态 8 条）满员后，新批连号追加为段五
    //（TXT_NAME5_BASE/TXT_BRIEF5_BASE 同款登记与产出；单2 改写后下一 N=1865，
    // 段五基址 1865/1873）；五面同步扩五段连寻。len5 取段界常量差
    // EV_SEG5−EV_SEG4（勿写 N_EVENTS−EV_SEG4——N_EVENTS 增长会静默吞掉段六
    // 行，见 EV_SEG5 注）
    base5: u16,
    len5: u16,
    // v1.16.31 扩池起：段五（v1.16.30 末态 8 条）满员后，新批连号追加为段六
    //（TXT_NAME6_BASE/TXT_BRIEF6_BASE 同款登记与产出；单2 后下一 N=1905，
    // 段六基址 1905/1915）；六面同步扩六段连寻。len6 取段界常量差
    // EV_SEG6−EV_SEG5（勿写 N_EVENTS−EV_SEG5——N_EVENTS 增长会静默吞掉段七
    // 行，见 EV_SEG6 注）
    base6: u16,
    len6: u16,
    // v1.16.31 新功能 3 条起：段六（10 条 N1905..1914/N1915..1924 紧连封死）
    // 满员后新批连号追加为段七（TXT_NAME7_BASE/TXT_BRIEF7_BASE 同款登记与
    // 产出；段六后下一空 N=1954，段七基址 1954/1957）；七面同步扩七段连寻。
    // len7 取段界常量差 N_EVENTS−EV_SEG6（末段写法同 len5/len6 原注：段界差
    // 是唯一正确写法，下批扩池续段八时须同步改差）
    base7: u16,
    len7: u16,
    // v1.16.32 单3 扩池起：段七（3 条）满员后新批连号追加为段八（TXT_NAME8_
    // BASE/TXT_BRIEF8_BASE=2338/2344 同款登记与产出；段七后下一空 N=2338，
    // 段八基址 2338/2344）；八面同步扩八段连寻（单1 segs 表驱动化——扩段只加
    // 一行 segs）。len8 取段界常量差 N_EVENTS−EV_SEG7（末段写法同 len5/len6/
    // len7 原注：段界差是唯一正确写法，下批扩池续段九时须同步改差）
    base8: u16,
    len8: u16,
    // v1.16.33 单3 扩池起：段八（6 条）满员后新批连号追加为段九（TXT_NAME9_
    // BASE/TXT_BRIEF9_BASE=2576/2581 同款登记与产出；段八后下一空 N=2576——
    // 单1 N 化批已用 N2368..2575 故段九顺延）；九面同步扩九段连寻（扩段只加
    // 一行 segs）。len9 取段界常量差 N_EVENTS−EV_SEG8（EV_SEG8=344 快照常量
    // 单1 已引入——新「末段写法」，下批扩池续段十时须改差，见 EV_SEG8 注；
    // v1.16.34 单3 段十已开工，差写法兑现 len9=(EV_SEG9−EV_SEG8)）
    base9: u16,
    len9: u16,
    // v1.16.34 单3 扩池起：段九（5 条）满员后新批连号追加为段十（TXT_NAME10_
    // BASE/TXT_BRIEF10_BASE=2647/2649 同款登记与产出；段九后下一空 N=2647——
    // 单1 精度/臂表批已用 N2601..2646 故段十顺延）；十面同步扩十段连寻（扩段
    // 只加一行 segs）。len10 取段界常量差 N_EVENTS−EV_SEG9（EV_SEG9=349 快照
    // 常量单1 已引入——新「末段写法」，下批扩池续段十一时须改差，见 EV_SEG9 注）
    // v1.16.35 单3 扩池起：段十（2 条）满员后新批连号追加为段十一（TXT_NAME11_
    // BASE/TXT_BRIEF11_BASE=2657/2659 同款登记与产出；段十后下一空 N=2657——
    // 单2 加载链批未用正文 N 故段十一顺延）；十一面同步扩十一段连寻（扩段
    // 只加一行 segs）。len11 取段界常量差 N_EVENTS−EV_SEG10（EV_SEG10=351 快照
    // 常量单1 已引入——新「末段写法」，下批扩池续段十二时须改差，见 EV_SEG10 注；
    // v1.16.36 单1 已预建 EV_SEG11=353 快照——届时 len11 改差 (EV_SEG11−EV_SEG10)、
    // len12 写 N_EVENTS−EV_SEG11，见 EV_SEG11 注）
    // v1.16.36 单3 扩池起：段十一（2 条）满员后新批连号追加为段十二（3 条：
    // TXT_NAME12_BASE/TXT_BRIEF12_BASE=2667/2670 同款登记与产出；段十一后下一
    // 空 N=2667——单1 快照预建+单2 零改动未用正文 N 故段十二顺延）；十二面同步
    // 扩十二段连寻（扩段只加一行 segs）。len11 由 N_EVENTS−EV_SEG10 改段界常量差
    // EV_SEG11−EV_SEG10 兑现；len12 取段界常量差 N_EVENTS−EV_SEG11（EV_SEG11=353
    // 快照常量单1 已预建本批引用——新「末段写法」，下批扩池续段十三时须改差，
    // 见 EV_SEG11/EV_SEG12 注）
    // v1.16.37 单2 扩池起：段十二（3 条）满员后新批连号追加为段十三（3 条：
    // TXT_NAME13_BASE/TXT_BRIEF13_BASE=2684/2687 同款登记与产出；段十二后下一
    // 空 N=2684——v1.16.36 正文收编至 N2683 故段十三顺延）；十三面同步扩十三段
    // 连寻（扩段只加一行 segs）。len12 由 N_EVENTS−EV_SEG11 改段界常量差
    // EV_SEG12−EV_SEG11 兑现；len13 取段界常量差 N_EVENTS−EV_SEG12（EV_SEG12=356
    // 快照常量 v1.16.36 单3 已预建本批引用——新「末段写法」，下批扩池续段十四时
    // 须改差，见 EV_SEG12/EV_SEG13 注；EV_SEG13=359 快照常量本批预建，届时
    // len13 改差 (EV_SEG13−EV_SEG12)、len14 写 N_EVENTS−EV_SEG13）
    // v1.16.38 单2 扩池起：段十三（3 条）满员后新批连号追加为段十四（3 条：
    // TXT_NAME14_BASE/TXT_BRIEF14_BASE=2701/2704 同款登记与产出；段十三后下一
    // 空 N=2701——v1.16.37 正文收编至 N2700 故段十四顺延）；十四面同步扩十四段
    // 连寻（扩段只加一行 segs）。len13 由 N_EVENTS−EV_SEG12 改段界常量差
    // EV_SEG13−EV_SEG12 兑现；len14 取段界常量差 N_EVENTS−EV_SEG13（EV_SEG13=359
    // 快照常量 v1.16.37 单2 已预建本批引用——新「末段写法」，下批扩池续段十五时
    // 须改差，见 EV_SEG13/EV_SEG14 注；EV_SEG14=362 快照常量本批预建，届时
    // len14 改差 (EV_SEG14−EV_SEG13)、len15 写 N_EVENTS−EV_SEG14）
    base10: u16,
    len10: u16,
    base11: u16,
    len11: u16,
    base12: u16,
    len12: u16,
    base13: u16,
    len13: u16,
    base14: u16,
    len14: u16,
    // v1.16.40 单3 扩段十五：base15/len15 双段登记（v1.16.43 改差兑现——
    // EV_SEG15=365 快照本批引用）
    base15: u16,
    len15: u16,
    // v1.16.43 单3 扩段十六：base16/len16 双段登记（len16 末段差写法 N_EVENTS−
    // EV_SEG15——EV_SEG16=368 快照预建，下批扩段十七时改差）
    base16: u16,
    len16: u16,
    // v1.16.45 施工单三扩段十八：base18/len18 双段登记（len18 末段差写法
    // N_EVENTS−EV_SEG17——EV_SEG18=374 快照预建，下批扩段十九须改差）
    base17: u16,
    len17: u16,
    base18: u16,
    len18: u16,
    // v1.16.46 段十九扩段十九：base19/len19 双段登记（len19 末段差写法
    // N_EVENTS−EV_SEG18——EV_SEG19=377 快照预建，v1.16.47 改差兑现）
    base19: u16,
    len19: u16,
    // v1.16.47 段二十扩段二十：base20/len20 双段登记（len20 末段差写法
    // N_EVENTS−EV_SEG19——EV_SEG20=380 快照预建，下批扩段二十一须改差）
    base20: u16,
    len20: u16,
    // v1.16.48 段二十一扩段二十一：base21/len21 双段登记（len21 末段差写法
    // N_EVENTS−EV_SEG20——EV_SEG21=383 快照预建，v1.16.49 改差兑现）
    base21: u16,
    len21: u16,
    // v1.16.49 段二十二扩段二十二：base22/len22 双段登记（len22 末段差写法
    // N_EVENTS−EV_SEG21——EV_SEG22=386 快照预建，v1.16.50 改差兑现）
    base22: u16,
    len22: u16,
    // v1.16.50 段二十三扩段二十三：base23/len23 双段登记（len23 末段差写法
    // N_EVENTS−EV_SEG22——EV_SEG23=389 快照预建，v1.16.51 段二十四开工批已改
    // 差兑现 (EV_SEG23−EV_SEG22)）
    base23: u16,
    len23: u16,
    // v1.16.51 段二十四扩段二十四：base24/len24 双段登记（v1.16.52 段二十五
    // 开工批 len24 末段差已随批改差 (EV_SEG24−EV_SEG23)，快照转实用）
    base24: u16,
    len24: u16,
    // v1.16.52 段二十五扩段二十五：base25/len25 双段登记（len25 末段差写法
    // N_EVENTS−EV_SEG24——EV_SEG25=395 快照预建，下批扩段二十六须改差）
    base25: u16,
    len25: u16,
    // v1.16.53 段二十六扩段二十六：base26/len26 双段登记（len26 末段差写法
    // N_EVENTS−EV_SEG25——EV_SEG26=398 快照预建，下批扩段二十七须改差）
    base26: u16,
    len26: u16,
}

impl EvTable {
    /// 界内查表（旧 [&str; N].get(i) 等价：界内 Some，越界 None）；七段连寻：
    /// i < len 走段一，len..len+len2 走段二，len+len2..len+len2+len3 走段三，
    /// len+len2+len3..len+len2+len3+len4 走段四，len+len2+len3+len4..len5 止走
    /// 段五，len5 之末..len6 之末走段六，len6 之末..总长走段七
    // v1.16.32 单1：七段 if 链改 segs 局部 (base,len) 表 + 循环累减——逐值等价
    //（k 跨段累减、总长越界返 None），扩段八只加一行 segs（免五面手改）。
    // ev_tables_section_bounds / batch_tables_aligned 逐段首尾钉测现成兜底。
    #[inline]
    pub fn get(&self, i: usize) -> Option<&'static str> {
        let segs = [
            (self.base, self.len),
            (self.base2, self.len2),
            (self.base3, self.len3),
            (self.base4, self.len4),
            (self.base5, self.len5),
            (self.base6, self.len6),
            (self.base7, self.len7),
            (self.base8, self.len8),
            // v1.16.33 单3 扩段九：只加一行 segs（base9/len9），累减循环自动续寻
            (self.base9, self.len9),
            // v1.16.34 单3 扩段十：只加一行 segs（base10/len10），累减循环自动续寻
            (self.base10, self.len10),
            // v1.16.35 单3 扩段十一：只加一行 segs（base11/len11），累减循环自动续寻
            (self.base11, self.len11),
            // v1.16.36 单3 扩段十二：只加一行 segs（base12/len12），累减循环自动续寻
            (self.base12, self.len12),
            // v1.16.37 单2 扩段十三：只加一行 segs（base13/len13），累减循环自动续寻
            (self.base13, self.len13),
            // v1.16.38 单2 扩段十四：只加一行 segs（base14/len14），累减循环自动续寻
            (self.base14, self.len14),
            // v1.16.40 单3 扩段十五：只加一行 segs（base15/len15），累减循环自动续寻
            (self.base15, self.len15),
            // v1.16.43 单3 扩段十六：只加一行 segs（base16/len16），累减循环自动续寻
            (self.base16, self.len16),
            // v1.16.44 扩段十七：只加一行 segs（base17/len17），累减循环自动续寻
            (self.base17, self.len17),
            // v1.16.45 施工单三扩段十八：只加一行 segs（base18/len18），累减循环自动续寻
            (self.base18, self.len18),
            // v1.16.46 段十九扩段十九：只加一行 segs（base19/len19），累减循环自动续寻
            (self.base19, self.len19),
            // v1.16.47 段二十扩段二十：只加一行 segs（base20/len20），累减循环自动续寻
            (self.base20, self.len20),
            // v1.16.48 段二十一扩段二十一：只加一行 segs（base21/len21），累减循环自动续寻
            (self.base21, self.len21),
            // v1.16.49 段二十二扩段二十二：只加一行 segs（base22/len22），累减循环自动续寻
            (self.base22, self.len22),
            // v1.16.50 段二十三扩段二十三：只加一行 segs（base23/len23），累减循环自动续寻
            (self.base23, self.len23),
            // v1.16.51 段二十四扩段二十四：只加一行 segs（base24/len24），累减循环自动续寻
            (self.base24, self.len24),
            // v1.16.52 段二十五扩段二十五：只加一行 segs（base25/len25），累减循环自动续寻
            (self.base25, self.len25),
            // v1.16.53 段二十六扩段二十六：只加一行 segs（base26/len26），累减循环自动续寻
            (self.base26, self.len26),
        ];
        let mut k = i;
        for seg_k in segs.iter() {
            if k < seg_k.1 as usize {
                return Some(evtx(seg_k.0 as usize + k));
            }
            k -= seg_k.1 as usize;
        }
        None
    }
    // v1.16.32 单1：总长和单点（is_empty/iter/Index 收敛本口，免五处重复展开）
    #[inline]
    pub fn len(&self) -> usize {
        self.len as usize
            + self.len2 as usize
            + self.len3 as usize
            + self.len4 as usize
            + self.len5 as usize
            + self.len6 as usize
            + self.len7 as usize
            + self.len8 as usize
            + self.len9 as usize
            + self.len10 as usize
            + self.len11 as usize
            // v1.16.36 单3 扩段十二：len() 总长和随段扩一行
            + self.len12 as usize
            // v1.16.37 单2 扩段十三：len() 总长和随段扩一行
            + self.len13 as usize
            // v1.16.38 单2 扩段十四：len() 总长和随段扩一行
            + self.len14 as usize
            // v1.16.40 单3 扩段十五：len() 总长和随段扩一行
            + self.len15 as usize
            // v1.16.43 单3 扩段十六：len() 总长和随段扩一行
            + self.len16 as usize
            // v1.16.44 扩段十七：len() 总长和随段扩一行
            + self.len17 as usize
            // v1.16.45 施工单三扩段十八：len() 总长和随段扩一行
            + self.len18 as usize
            // v1.16.46 段十九扩段十九：len() 总长和随段扩一行
            + self.len19 as usize
            // v1.16.47 段二十扩段二十：len() 总长和随段扩一行
            + self.len20 as usize
            // v1.16.48 段二十一扩段二十一：len() 总长和随段扩一行
            + self.len21 as usize
            // v1.16.49 段二十二扩段二十二：len() 总长和随段扩一行
            + self.len22 as usize
            // v1.16.50 段二十三扩段二十三：len() 总长和随段扩一行
            + self.len23 as usize
            // v1.16.51 段二十四扩段二十四：len() 总长和随段扩一行
            + self.len24 as usize
            // v1.16.52 段二十五扩段二十五：len() 总长和随段扩一行
            + self.len25 as usize
            // v1.16.53 段二十六扩段二十六：len() 总长和随段扩一行
            + self.len26 as usize
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// 全表迭代（旧 [&str; N].iter() 等价：Item 为 &'static str，行序即 id 序）
    pub fn iter(&self) -> impl Iterator<Item = &'static str> + '_ {
        (0..self.len()).map(move |i| self.get(i).unwrap_or(""))
    }
}

impl core::ops::Index<usize> for EvTable {
    type Output = str;
    /// 直下标：越界空串兜底（evtx 同款防御，绝不 panic——panic=abort 纪律；
    /// 直下标仅测试可达，测试恒界内）
    #[inline]
    fn index(&self, i: usize) -> &str {
        if i >= self.len() {
            return "";
        }
        self.get(i).unwrap_or("")
    }
}

pub static EVENT_NAMES: EvTable = EvTable {
    base: TXT_NAME_BASE as u16,
    len: EV_SEG1 as u16,
    base2: TXT_NAME2_BASE as u16,
    // 段界常量差（勿写 N_EVENTS−EV_SEG1——N_EVENTS=309 时静默变 18、段二简行错位）
    len2: (EV_SEG2 - EV_SEG1) as u16,
    base3: TXT_NAME3_BASE as u16,
    // 段界常量差（勿写 N_EVENTS−EV_SEG2——N_EVENTS 增长时静默吞掉段四行，见
    // EV_SEG3 注）
    len3: (EV_SEG3 - EV_SEG2) as u16,
    base4: TXT_NAME4_BASE as u16,
    // 段界常量差（勿写 N_EVENTS−EV_SEG3——N_EVENTS 增长时静默吞掉段五行，见
    // EV_SEG4 注）
    len4: (EV_SEG4 - EV_SEG3) as u16,
    base5: TXT_NAME5_BASE as u16,
    // v1.16.31 扩池起改段界常量差（原 N_EVENTS−EV_SEG4 是「末段」写法，扩段六
    // 后会随 N_EVENTS 静默吞掉段六首行——见 EV_SEG5 注）
    len5: (EV_SEG5 - EV_SEG4) as u16,
    base6: TXT_NAME6_BASE as u16,
    // v1.16.31 新功能批起改段界常量差（原 N_EVENTS−EV_SEG5 是「末段」写法，扩
    // 段七后会随 N_EVENTS 静默吞掉段七首行——见 EV_SEG6 注）
    len6: (EV_SEG6 - EV_SEG5) as u16,
    base7: TXT_NAME7_BASE as u16,
    // v1.16.32 单3 扩段八起改段界常量差（原 N_EVENTS−EV_SEG6 是「末段」写法，
    // 扩段八后会随 N_EVENTS 静默吞掉段八首行——见 EV_SEG7 注）
    len7: (EV_SEG7 - EV_SEG6) as u16,
    base8: TXT_NAME8_BASE as u16,
    // v1.16.33 单1：len8 由 N_EVENTS−EV_SEG7 改段界常量差 EV_SEG8−EV_SEG7
    //（N_EVENTS 快照常量先行引入——扩池开段九时 N_EVENTS 增长不再静默吞段八
    // 首行，见 EV_SEG8 注；单3 扩池起 N_EVENTS=349≠EV_SEG8=344，差写法兑现）
    len8: (EV_SEG8 - EV_SEG7) as u16,
    // v1.16.33 单3 扩段九：base9/len9 双段登记——len9 取段界常量差
    // N_EVENTS−EV_SEG8（新「末段写法」，下批扩段十须改差，照 len7/len8 原注；
    // v1.16.34 单3 段十已开工，下两行兑现 len9=(EV_SEG9−EV_SEG8)）
    // v1.16.34 单1：len9 由 N_EVENTS−EV_SEG8 改段界常量差 EV_SEG9−EV_SEG8
    //（EV_SEG9=349 快照常量先行引入——扩池开段十时 N_EVENTS 增长不再静默吞段九
    // 首行，见 EV_SEG9 注）
    base9: TXT_NAME9_BASE as u16,
    len9: (EV_SEG9 - EV_SEG8) as u16,
    // v1.16.34 单3 扩段十：base10/len10 双段登记——len10 取段界常量差
    // N_EVENTS−EV_SEG9（新「末段写法」，下批扩段十一须改差，照 len9 原注）
    // v1.16.35 单1：len10 由 N_EVENTS−EV_SEG9 改段界常量差 EV_SEG10−EV_SEG9
    //（EV_SEG10=351 快照常量先行引入——扩池开段十一时 N_EVENTS 增长不再静默
    // 吞段十首行，见 EV_SEG10 注）
    base10: TXT_NAME10_BASE as u16,
    len10: (EV_SEG10 - EV_SEG9) as u16,
    // v1.16.35 单3 扩段十一：base11/len11 双段登记——len11 取段界常量差
    // N_EVENTS−EV_SEG10（新「末段写法」，下批扩段十二须改差，照 len10 原注；
    // v1.16.36 单1 已预建 EV_SEG11=353 快照——扩池开段十二时兑现改差，见 EV_SEG11 注）
    // v1.16.36 单1：len11 由 N_EVENTS−EV_SEG10 改段界常量差 EV_SEG11−EV_SEG10
    //（EV_SEG11=353 快照常量先行引入——扩池开段十三时 N_EVENTS 增长不再静默
    // 吞段十一首行，见 EV_SEG11 注）
    // v1.16.36 单3 扩段十二：base12/len12 双段登记——len12 取段界常量差
    // N_EVENTS−EV_SEG11（新「末段写法」，下批扩段十三须改差，照 len11 原注；
    // EV_SEG12=356 快照常量单3 已预建——届时 len12 改差 (EV_SEG12−EV_SEG11)、
    // len13 写 N_EVENTS−EV_SEG12，见 EV_SEG12 注）
    // v1.16.37 单2：len12 由 N_EVENTS−EV_SEG11 改段界常量差 EV_SEG12−EV_SEG11
    //（EV_SEG12=356 快照常量先行引入——扩池开段十四时 N_EVENTS 增长不再静默
    // 吞段十二首行，见 EV_SEG12 注）；base13/len13 双段登记照款——len13 取
    // 段界常量差 N_EVENTS−EV_SEG12（新「末段写法」，下批扩段十四须改差）
    // v1.16.38 单2：len13 由 N_EVENTS−EV_SEG12 改段界常量差 EV_SEG13−EV_SEG12
    //（EV_SEG13=359 快照常量先行引入——扩池开段十五时 N_EVENTS 增长不再静默
    // 吞段十三首行，见 EV_SEG13 注）；base14/len14 双段登记照款——len14 取
    // 段界常量差 N_EVENTS−EV_SEG13（新「末段写法」，下批扩段十五须改差）
    base11: TXT_NAME11_BASE as u16,
    len11: (EV_SEG11 - EV_SEG10) as u16,
    base12: TXT_NAME12_BASE as u16,
    len12: (EV_SEG12 - EV_SEG11) as u16,
    base13: TXT_NAME13_BASE as u16,
    len13: (EV_SEG13 - EV_SEG12) as u16,
    // v1.16.40 单3：len14 由 N_EVENTS−EV_SEG13 改段界常量差 EV_SEG14−EV_SEG13
    //（EV_SEG14=362 快照常量先行引入——扩池开段十六时 N_EVENTS 增长不再静默
    // 吞段十四首行，见 EV_SEG14 注）；base15/len15 双段登记照款——len15 取
    // 段界常量差 N_EVENTS−EV_SEG14（新「末段写法」，下批扩段十六须改差）
    base14: TXT_NAME14_BASE as u16,
    len14: (EV_SEG14 - EV_SEG13) as u16,
    // v1.16.43 单3：len15 由 N_EVENTS−EV_SEG14 改段界常量差 EV_SEG15−EV_SEG14
    // 兑现（EV_SEG15=365 快照 v1.16.40 预建本批引用——N_EVENTS 增长不再静默吞
    // 段十五行）；base16/len16 双段登记照款
    // v1.16.44：len16 由 N_EVENTS−EV_SEG15 改段界常量差 EV_SEG16−EV_SEG15 兑现
    //（EV_SEG16=368 快照 v1.16.43 预建本批引用——N_EVENTS 增长不再静默吞段十六
    // 行）；base17/len17 双段登记照款——len17 取段界常量差 N_EVENTS−EV_SEG16
    //（新「末段写法」，下批扩段十八须改差）
    base15: TXT_NAME15_BASE as u16,
    len15: (EV_SEG15 - EV_SEG14) as u16,
    base16: TXT_NAME16_BASE as u16,
    len16: (EV_SEG16 - EV_SEG15) as u16,
    base17: TXT_NAME17_BASE as u16,
    // v1.16.45 施工单三：len17 由末段差写法改段界常量差兑现（EV_SEG17 快照转
    // 实用）+base18/len18 双段登记（len18 新末段差写法 N_EVENTS−EV_SEG17，下批
    // 扩段十九须改差）
    len17: (EV_SEG17 - EV_SEG16) as u16,
    base18: TXT_NAME18_BASE as u16,
    // v1.16.46 段十九：len18 由末段差写法改段界常量差兑现（EV_SEG18 快照转
    // 实用）+base19/len19 双段登记（len19 新末段差写法 N_EVENTS−EV_SEG18，下批
    // 扩段二十须改差）
    // v1.16.47 段二十：len19 由末段差写法改段界常量差兑现（EV_SEG19 快照转
    // 实用）+base20/len20 双段登记（len20 新末段差写法 N_EVENTS−EV_SEG19，下批
    // 扩段二十一须改差）
    // v1.16.48 段二十一：len20 由末段差写法改段界常量差兑现（EV_SEG20 快照转
    // 实用）+base21/len21 双段登记（len21 新末段差写法 N_EVENTS−EV_SEG20，下批
    // 扩段二十二须改差）
    // v1.16.49 段二十二：len21 由末段差写法改段界常量差兑现（EV_SEG21 快照转
    // 实用）+base22/len22 双段登记（len22 新末段差写法 N_EVENTS−EV_SEG21，下批
    // 扩段二十三须改差）
    // v1.16.50 段二十三：len22 由末段差写法改段界常量差兑现（EV_SEG22 快照转
    // 实用，dead_code 警告随之消失）+base23/len23 双段登记（len23 新末段差写
    // 法 N_EVENTS−EV_SEG22，下批扩段二十四须改差）
    len18: (EV_SEG18 - EV_SEG17) as u16,
    base19: TXT_NAME19_BASE as u16,
    len19: (EV_SEG19 - EV_SEG18) as u16,
    base20: TXT_NAME20_BASE as u16,
    len20: (EV_SEG20 - EV_SEG19) as u16,
    base21: TXT_NAME21_BASE as u16,
    // v1.16.49 段二十二：len21 末段差写法随批改差（N_EVENTS−EV_SEG20 →段界
    // 常量差，N_EVENTS 增长不再静默吞段二十一行；注内写法用全角减号防撞
    // v11652/v11655 零残留源码钉的半角 matches 面）
    len21: (EV_SEG21 - EV_SEG20) as u16,
    base22: TXT_NAME22_BASE as u16,
    // v1.16.50 段二十三：len22 末段差写法随批改差（N_EVENTS−EV_SEG21 →段界
    // 常量差，N_EVENTS 增长不再静默吞段二十二行；注内写法用全角减号防撞
    // v11652/v11655/v11658 零残留源码钉的半角 matches 面）
    len22: (EV_SEG22 - EV_SEG21) as u16,
    base23: TXT_NAME23_BASE as u16,
    // v1.16.51 段二十四：len23 末段差写法随批改差（N_EVENTS−EV_SEG22 →段界
    // 常量差，N_EVENTS 增长不再静默吞段二十三行；注内写法用全角减号防撞
    // v11652/v11655/v11658 零残留源码钉的半角 matches 面）
    len23: (EV_SEG23 - EV_SEG22) as u16,
    base24: TXT_NAME24_BASE as u16,
    // v1.16.52 段二十五：len24 末段差写法随批改差（N_EVENTS−EV_SEG23 →段界
    // 常量差，N_EVENTS 增长不再静默吞段二十四行；注内写法用全角减号防撞
    // v11652/v11655/v11658 零残留源码钉的半角 matches 面）
    len24: (EV_SEG24 - EV_SEG23) as u16,
    // v1.16.52 段二十五开设：base25/len25 双段登记照 base24 先例款（len25 末
    // 段差写法 v1.16.53 段二十六开工批已改差兑现——见下注；全角减号同上）
    base25: TXT_NAME25_BASE as u16,
    // v1.16.53 段二十六：len25 末段差写法随批改差（N_EVENTS−EV_SEG24 →段界
    // 常量差 (EV_SEG25−EV_SEG24)，N_EVENTS 增长不再静默吞段二十五行——同上
    // 注口径）
    len25: (EV_SEG25 - EV_SEG24) as u16,
    // v1.16.53 段二十六开设：base26/len26 双段登记照 base25 先例款（len26 新
    // 末段差 N_EVENTS−EV_SEG25，下批段二十七开工改段界常量差；全角减号同上）
    base26: TXT_NAME26_BASE as u16,
    len26: (N_EVENTS - EV_SEG25) as u16,
};


// v1.16.19 前段简随前期高频卡画面化同批改（id 0/1/3/4/5/6/12）：简仍「一行 ≤18 字」
// 预算（表注见上），由「场景+结果」记账体改画面半句——玩家开局必读的一页不再最平
pub static EVENT_BRIEF: EvTable = EvTable {
    base: TXT_BRIEF_BASE as u16,
    len: EV_SEG1 as u16,
    base2: TXT_BRIEF2_BASE as u16,
    len2: (EV_SEG2 - EV_SEG1) as u16,
    base3: TXT_BRIEF3_BASE as u16,
    // 段界常量差（同 EVENT_NAMES len3 注）
    len3: (EV_SEG3 - EV_SEG2) as u16,
    base4: TXT_BRIEF4_BASE as u16,
    // 段界常量差（同 EVENT_NAMES len4 注）
    len4: (EV_SEG4 - EV_SEG3) as u16,
    base5: TXT_BRIEF5_BASE as u16,
    // v1.16.31 扩池起改段界常量差（同 EVENT_NAMES len5 注）
    len5: (EV_SEG5 - EV_SEG4) as u16,
    base6: TXT_BRIEF6_BASE as u16,
    // v1.16.31 新功能批起改段界常量差（同 EVENT_NAMES len6 注）
    len6: (EV_SEG6 - EV_SEG5) as u16,
    base7: TXT_BRIEF7_BASE as u16,
    // v1.16.32 单3 扩段八起改段界常量差（同 EVENT_NAMES len7 注）
    len7: (EV_SEG7 - EV_SEG6) as u16,
    base8: TXT_BRIEF8_BASE as u16,
    // v1.16.33 单1：同 EVENT_NAMES len8 注（EV_SEG8 快照常量差写法）
    len8: (EV_SEG8 - EV_SEG7) as u16,
    // v1.16.33 单3 扩段九：同 EVENT_NAMES base9/len9 注（len9 末段差写法）
    // v1.16.34 单1：同 EVENT_NAMES len9 注（EV_SEG9 快照常量差写法）
    base9: TXT_BRIEF9_BASE as u16,
    len9: (EV_SEG9 - EV_SEG8) as u16,
    // v1.16.34 单3 扩段十：同 EVENT_NAMES base10/len10 注（len10 末段差写法）
    // v1.16.35 单1：同 EVENT_NAMES len10 注（EV_SEG10 快照常量差写法）
    base10: TXT_BRIEF10_BASE as u16,
    len10: (EV_SEG10 - EV_SEG9) as u16,
    // v1.16.35 单3 扩段十一：同 EVENT_NAMES base11/len11 注（len11 末段差写法）
    // v1.16.36 单1：同 EVENT_NAMES len11 注（EV_SEG11 快照差写法兑现）
    // v1.16.36 单3 扩段十二：同 EVENT_NAMES base12/len12 注（len12 末段差写法）
    // v1.16.37 单2：同 EVENT_NAMES len12 注（EV_SEG12 快照差写法兑现）+
    // base13/len13 双段登记（len13 末段差写法 N_EVENTS−EV_SEG12）
    // v1.16.38 单2：同 EVENT_NAMES len13 注（EV_SEG13 快照差写法兑现）+
    // base14/len14 双段登记（len14 末段差写法 N_EVENTS−EV_SEG13）
    base11: TXT_BRIEF11_BASE as u16,
    len11: (EV_SEG11 - EV_SEG10) as u16,
    base12: TXT_BRIEF12_BASE as u16,
    len12: (EV_SEG12 - EV_SEG11) as u16,
    base13: TXT_BRIEF13_BASE as u16,
    len13: (EV_SEG13 - EV_SEG12) as u16,
    // v1.16.40 单3：同 EVENT_NAMES len14 注（EV_SEG14 快照差写法兑现）+
    // base15/len15 双段登记（len15 末段差写法 N_EVENTS−EV_SEG14）
    base14: TXT_BRIEF14_BASE as u16,
    len14: (EV_SEG14 - EV_SEG13) as u16,
    // v1.16.43 单3：同 EVENT_NAMES len15 注（改差兑现）+ base16/len16 双段登记
    // v1.16.44：同 EVENT_NAMES len16 注（EV_SEG16 快照差写法兑现）+ base17/len17
    // 双段登记（len17 末段差写法 N_EVENTS−EV_SEG16，下批扩段十八须改差）
    base15: TXT_BRIEF15_BASE as u16,
    len15: (EV_SEG15 - EV_SEG14) as u16,
    base16: TXT_BRIEF16_BASE as u16,
    len16: (EV_SEG16 - EV_SEG15) as u16,
    base17: TXT_BRIEF17_BASE as u16,
    // v1.16.45 施工单三：len17 改差兑现+base18/len18 双段登记（同上注口径）
    len17: (EV_SEG17 - EV_SEG16) as u16,
    base18: TXT_BRIEF18_BASE as u16,
    // v1.16.46 段十九：len18 改差兑现+base19/len19 双段登记（同上注口径）
    // v1.16.47 段二十：len19 改差兑现+base20/len20 双段登记（同上注口径）
    // v1.16.48 段二十一：len20 改差兑现（EV_SEG20 快照转实用）+base21/len21 双
    // 段登记（len21 新末段差 N_EVENTS−EV_SEG20，下批扩段二十二须改差）
    // v1.16.49 段二十二：len21 改差兑现（EV_SEG21 快照转实用）+base22/len22 双
    // 段登记（len22 新末段差 N_EVENTS−EV_SEG21，下批扩段二十三须改差）
    // v1.16.50 段二十三：len22 改差兑现（EV_SEG22 快照转实用）+base23/len23 双
    // 段登记（len23 新末段差 N_EVENTS−EV_SEG22，下批扩段二十四须改差）
    len18: (EV_SEG18 - EV_SEG17) as u16,
    base19: TXT_BRIEF19_BASE as u16,
    len19: (EV_SEG19 - EV_SEG18) as u16,
    base20: TXT_BRIEF20_BASE as u16,
    len20: (EV_SEG20 - EV_SEG19) as u16,
    base21: TXT_BRIEF21_BASE as u16,
    // v1.16.49 段二十二：len21 末段差写法随批改差（同上注口径）
    len21: (EV_SEG21 - EV_SEG20) as u16,
    base22: TXT_BRIEF22_BASE as u16,
    // v1.16.50 段二十三：len22 末段差写法随批改差（同上注口径）
    len22: (EV_SEG22 - EV_SEG21) as u16,
    base23: TXT_BRIEF23_BASE as u16,
    // v1.16.51 段二十四：len23 末段差写法随批改差（同上注口径）
    len23: (EV_SEG23 - EV_SEG22) as u16,
    base24: TXT_BRIEF24_BASE as u16,
    // v1.16.52 段二十五：len24 末段差写法随批改差（同上注口径）
    len24: (EV_SEG24 - EV_SEG23) as u16,
    // v1.16.52 段二十五开设：base25/len25 双段登记（同上注口径）
    base25: TXT_BRIEF25_BASE as u16,
    // v1.16.53 段二十六：len25 末段差写法随批改差（同上注口径）
    len25: (EV_SEG25 - EV_SEG24) as u16,
    // v1.16.53 段二十六开设：base26/len26 双段登记照 base25 先例款（len26 新
    // 末段差 N_EVENTS−EV_SEG25，下批段二十七开工改段界常量差；全角减号同上）
    base26: TXT_BRIEF26_BASE as u16,
    len26: (N_EVENTS - EV_SEG25) as u16,
};


// v1.16.7 稀有层阈值：EVENTS 权重 w∈1..=RARE_W 的卡按稀有呈现（描边紫；
// w=0 是链内段「仅链内出现」的哨兵，不算稀有）。扩池改权重后描边集合随之变化，
// 改动时对照本表注释（稀有基权占比维持 ~4%）
pub const RARE_W: i32 = 2;

// ---------------- 功法价格（藏经阁，现实可购） ----------------
pub const TECH_PRICES: [i32; 8] = [0, 2000, 8000, 30000, 120000, 500000, 2000000, 10000000];

// ---------------- 称号 ----------------
// v1.16.34 单1 β刀（--nize=arms）：境界名 15 臂收编 textpack（调用点
// 均一次性 UI 文案，evtx 零分配）——max_realm.min 钳制落 _ 臂槽逐值等价
//（v1.16.34 修复：通道初版 span 误吞 fn 签名整行替换——title_recheck 钉B
// v1.16.33 同型事故红线，本轮亲历；修法=锚只含 match 行，fn 头不进 span）
pub fn realm_title(max_realm: usize) -> &'static str {
    const REALM_TITLE_NS: [u16; 15] = [
        2632, 2633, 2634, 2635, 2636, 2637, 2638, 2639,
        2640, 2641, 2642, 2643, 2644, 2645, 2646,
    ];
    evtx(REALM_TITLE_NS[max_realm.min(REALM_TITLE_NS.len() - 1)] as usize)
}

// v1.16.32 单1 小表批：表体收编 textpack（TXT_MINOR_BASE 起 10 条连号）。长度
// 常量供 gen_range 上界（原 MINOR_LINES.len() 调用点等价换名）；查表 fn 越界
// min 钳制（panic=abort 纪律）。
pub const MINOR_LINES_LEN: usize = 10;
pub fn minor_line(k: usize) -> &'static str {
    evtx(TXT_MINOR_BASE as usize + k.min(MINOR_LINES_LEN - 1))
}

// ---------------- 同页人十名（原创定稿 §11） ----------------
pub const COMPANION_NAMES: [&str; 10] = [ // ←锚行：GameBook 主文档 §3.1 记本行行号，以当日 grep 实测为准（分册旧锚勿沿用）；本行上方插行即漂移，扩池/加名后须施工单五回写主文档锚（原位行尾注释不漂移）
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
    pub desc_n: u16, // v1.16.29 刀B：节点简介收编 textpack（name/acts 短串留原地）
    pub acts: &'static [&'static str],
}

impl NodeDef {
    /// 节点简介（山河图·洞府面板显示位；O(1) 零分配帧路径安全）
    pub fn desc(&self) -> &'static str {
        evtx(self.desc_n as usize)
    }
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
        desc_n: 1677,
        acts: &["拜入青云", "宗门任务·1年", "宗门商店"],
    },
    NodeDef {
        name: "临仙坊市",
        x: 640.0,
        y: 230.0,
        desc_n: 1678,
        acts: &["打开坊市"],
    },
    NodeDef {
        name: "落霞秘境",
        x: 980.0,
        y: 110.0,
        desc_n: 1679,
        acts: &["开始远征·1年"],
    },
    NodeDef {
        name: "清源洞府",
        x: 430.0,
        y: 90.0,
        desc_n: 1680,
        acts: &["闭关修炼·1年", "经营洞府"],
    },
    NodeDef {
        name: "荒骨战场",
        x: 1120.0,
        y: 330.0,
        desc_n: 1681,
        acts: &["寻仇"],
    },
    NodeDef {
        name: "天雷峰",
        x: 780.0,
        y: 390.0,
        desc_n: 1682,
        acts: &["冲击境界"],
    },
];

// 纯视觉连线；赶路支持任意两地直达
// v1.16.35 单1 刀3：usize→u8 窄化（值域 0..=5，LOCATIONS 6 节点索引），
// 读点 world.rs 显式扩位；数值语义零变化（tables2_pins 窄化钉测随批）
pub const MAP_PATHS: [(u8, u8); 6] = [(0, 3), (0, 1), (1, 2), (1, 4), (4, 5), (2, 5)];

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
    pub name_n: u16, // v1.16.32 单1 主刀第三批：name/desc 收编 textpack
    pub desc_n: u16,
    pub cost: u32,
    pub kind: SectGoodKind,
}

impl SectGood {
    pub fn name(&self) -> &'static str {
        evtx(self.name_n as usize)
    }
    pub fn desc(&self) -> &'static str {
        evtx(self.desc_n as usize)
    }
}

pub const SECT_GOODS: [SectGood; 6] = [
    SectGood {
        name_n: 2229,
        desc_n: 2235,
        cost: 60,
        kind: SectGoodKind::Qi15,
    },
    SectGood {
        name_n: 2230,
        desc_n: 2236,
        cost: 90,
        kind: SectGoodKind::Dao2,
    },
    SectGood {
        name_n: 2231,
        desc_n: 2237,
        cost: 90,
        kind: SectGoodKind::Body2,
    },
    SectGood {
        name_n: 2232,
        desc_n: 2238,
        cost: 140,
        kind: SectGoodKind::Life8,
    },
    SectGood {
        name_n: 2233,
        desc_n: 2239,
        cost: 260,
        kind: SectGoodKind::Relic(5),
    },
    SectGood {
        name_n: 2234,
        desc_n: 2240,
        cost: 420,
        kind: SectGoodKind::Tech(7),
    },
];

// 远征步骤：kind 0战 1药 2宝 3奇
// v1.16.32 单1 主刀第三批补收：text 收编 textpack（text_n 显式 N）——访问器
// O(1) 零分配（远征卡构建路径，事件期非帧路径）
pub struct ExpedStep {
    pub text_n: u16,
    pub kind: u8,
}

impl ExpedStep {
    pub fn text(&self) -> &'static str {
        evtx(self.text_n as usize)
    }
}

pub const EXPED_STEPS: [ExpedStep; 8] = [
    ExpedStep {
        text_n: 2330,
        kind: 0,
    },
    ExpedStep {
        text_n: 2331,
        kind: 0,
    },
    ExpedStep {
        text_n: 2332,
        kind: 1,
    },
    ExpedStep {
        text_n: 2333,
        kind: 1,
    },
    ExpedStep {
        text_n: 2334,
        kind: 2,
    },
    ExpedStep {
        text_n: 2335,
        kind: 2,
    },
    ExpedStep {
        text_n: 2336,
        kind: 3,
    },
    ExpedStep {
        text_n: 2337,
        kind: 3,
    },
];

// v1.16.32 单1 小表批：表体收编 textpack（TXT_EXPCHO_BASE 起 8 条行主序连号，
// k*2+j 寻址同旧 [k][j]）；两参各自 min 钳制。
pub fn exped_choice(k: usize, j: usize) -> &'static str {
    evtx(TXT_EXPCHO_BASE as usize + k.min(3) * 2 + j.min(1))
}

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
// v1.16.32 单1 小表批：表体收编 textpack（TXT_WFLAV_BASE 起 5 条连号）；等长
// 对账单测（weather_flavor_aligned）改走 WEATHER_FLAVOR_LEN。
pub const WEATHER_FLAVOR_LEN: usize = 5;
pub fn weather_flavor(w: usize) -> &'static str {
    evtx(TXT_WFLAV_BASE as usize + w.min(WEATHER_FLAVOR_LEN - 1))
}

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
pub const PORTRAIT_TIERS: [u16; 4] = [0, 3, 6, 10]; // 各档境界下限

// v1.14.0 立绘焕新时刻：四相名（跨档破境 toast『道貌焕然——【X之相】』+ 角色页常显行）
pub const PORTRAIT_STAGE_NAMES: [&str; 4] = ["白丁之相", "道基之相", "紫府之相", "墨仙之相"];

pub fn portrait_tier(realm: usize) -> usize {
    let mut t = 0;
    for (i, &th) in PORTRAIT_TIERS.iter().enumerate() {
        if realm >= th as usize {
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
// v1.16.32 单1 小表批：表体收编 textpack（TXT_DFD_BASE 起 3 条连号）
pub fn dongfu_fac_desc(i: usize) -> &'static str {
    evtx(TXT_DFD_BASE as usize + i.min(2))
}
pub const JUELING_MULT: [f64; 4] = [1.00, 1.15, 1.30, 1.50];
// v1.16.39 整数表第三轮②：i32→i16（值域 0..=5000；读点 game.rs:2319/:13608 `as f64`
// 与 world.rs evf Display 模板均天然兼容；理论 −8B 以打包实测为准）
pub const LINGTIAN_YIELD: [i16; 4] = [0, 300, 1200, 5000];
pub const DANFANG_DISC: [f64; 4] = [1.00, 0.90, 0.80, 0.70];
// 升到 L+1 级费用：灵石 DONGFU_UP_STONES[L] + 贡献 DONGFU_UP_CONTRIB[L]
pub const DONGFU_UP_STONES: [i32; 3] = [2000, 20000, 200000];
// v1.16.39 整数表第三轮③：u32→u16（值域 30..=400；读点四处与 meta.sect_contrib(u32)
// 比较已同批 `as u32` 扩位——guide.rs/world.rs×2/game.rs 测试区；理论 −6B）
pub const DONGFU_UP_CONTRIB: [u16; 3] = [30, 120, 400];

// ================= v1.6.0 山河万象：仇敌突破换地图 =================

// 地图种子默认值（旧档缺 mseed 键时的迁移缺省）
pub const MAP_SEED_DEFAULT: u32 = 20261011;

// 五区主题表：主题 0 与 v1.5.0 现版逐值一致（旧档/未换幕前像素级等价）
pub struct MapTheme {
    pub name_n: u16, // v1.16.32 单1 主刀第三批：name/desc 收编 textpack
    pub desc_n: u16,
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

impl MapTheme {
    pub fn name(&self) -> &'static str {
        evtx(self.name_n as usize)
    }
    pub fn desc(&self) -> &'static str {
        evtx(self.desc_n as usize)
    }
}

pub const MAP_THEMES: [MapTheme; 5] = [
    // 0 东境（现版逐值：渐变 (24,34,40)→(14,20,26)、苔斑 (30,52,38)/(56,46,36) 600、
    //   山脊 (46,44,58)/(38,36,48)、河 (58,96,110)/(86,130,140)、林 (30,48,36)/(26,42,32) 220、
    //   边框 (122,96,52)、deco 1.0、jitter 全 0）
    MapTheme {
        name_n: 2209,
        desc_n: 2214,
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
        name_n: 2210,
        desc_n: 2215,
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
        name_n: 2211,
        desc_n: 2216,
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
        name_n: 2212,
        desc_n: 2217,
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
        name_n: 2213,
        desc_n: 2218,
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
    pub mech_n: u16,        // 机制短名（顶栏标签）——v1.16.32 收编 textpack
    pub desc_n: u16,        // 一行说明（≤18 字，图鉴/卡片）——v1.16.32 收编 textpack
    pub quest_mult: f64,    // 宗门任务贡献与灵石
    pub exped_mult: f64,    // 远征收益
    pub exped_risk: f32,    // 远征成功率减益（正=更险）
    pub cult_mult: f64,     // 现实闭关效率
    pub event_mult: f64,    // 模拟善缘事件权重
}

impl RegionMech {
    pub fn mech(&self) -> &'static str {
        evtx(self.mech_n as usize)
    }
    pub fn desc(&self) -> &'static str {
        evtx(self.desc_n as usize)
    }
}
pub const REGION_MECH: [RegionMech; 5] = [
    // 0 东境青云：宗门兴望
    RegionMech {
        mech_n: 2219,
        desc_n: 2224,
        quest_mult: 1.20,
        exped_mult: 1.00,
        exped_risk: 0.0,
        cult_mult: 1.00,
        event_mult: 1.00,
    },
    // 1 西漠黄沙：大漠秘藏
    RegionMech {
        mech_n: 2220,
        desc_n: 2225,
        quest_mult: 1.00,
        exped_mult: 1.25,
        exped_risk: 0.0,
        cult_mult: 1.00,
        event_mult: 1.00,
    },
    // 2 南疆瘴林：瘴惑灵芽
    RegionMech {
        mech_n: 2221,
        desc_n: 2226,
        quest_mult: 1.00,
        exped_mult: 1.20,
        exped_risk: 0.08,
        cult_mult: 1.00,
        event_mult: 1.00,
    },
    // 3 北境冰原：冰心澄神
    RegionMech {
        mech_n: 2222,
        desc_n: 2227,
        quest_mult: 1.00,
        exped_mult: 1.00,
        exped_risk: 0.0,
        cult_mult: 1.15,
        event_mult: 1.00,
    },
    // 4 东海仙屿：仙屿潮音
    RegionMech {
        mech_n: 2223,
        desc_n: 2228,
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
// v1.16.35 单1 刀3：i64→i32 窄化（值域 30..=80000，容于 i32），读点
// menu.rs/game.rs 显式扩位 as i64；数值语义零变化（tables2_pins 窄化钉测随批）
pub const REGION_VISIT_REWARD: [(i32, i32); 5] = [
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
// v1.16.32 单1 副刀：表体收编 textpack（TXT_NPCBRIEF_BASE 起 4 条连号）
pub fn npc_brief(i: usize) -> &'static str {
    evtx(TXT_NPCBRIEF_BASE as usize + i.min(3))
}

// 好感档次阈值与名称（fav ≥ 阈值即入档）
// v1.16.39 整数表第三轮④：i32→i8（值域 0..=95 ≤ i8::MAX=127；读点 npc.rs tier_of
// 与 fav(i32) 比较已同批 `as i32` 扩位，charpanel/gallery 三处刻度 `as f32` 天然兼容；
// 理论 −15B）
pub const NPC_FAV_TIERS: [i8; 5] = [0, 20, 40, 70, 95];
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
// v1.16.29 刀B：NPC_TALES 表本体收编 textpack（TXT_TALES_BASE 起 12 条行主序连号：
// npc*3+seg，基址见 text_offsets.inc）——查表 fn 单口，O(1) 纯算术零分配（帧路径
// 安全）；行内容与旧字面量逐字节等价（真相源 text_source.txt 行 + 钉测）。
pub fn npc_tale(npc: usize, seg: usize) -> &'static str {
    evtx(TXT_TALES_BASE as usize + npc.min(3) * 3 + seg.min(2))
}
// COMP_TALES[seg]：道侣槽共通小传（同页人），段 1 结缘即解（companion_lives≥1 保底）。
// v1.16.32 单1 小表批：表体收编 textpack（TXT_COMPT_BASE 起 3 条连号）
pub const COMP_TALES_LEN: usize = 3;
pub fn comp_tale(seg: usize) -> &'static str {
    evtx(TXT_COMPT_BASE as usize + seg.min(2))
}

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

// ---------------- v1.16.40 长老委托：现实侧轮换小目标层 ----------------
// 恒有 1 条进行中委托，从本表轮换（轮换序数=累计复命数 comm_done+map_seed 确定性
// 派生，零 RNG 态）；达标后任务面板「交差」领奖并轮换下一条。奖励刻意不含道韵
// ——全仓唯一道韵去化点是 buy_upgrade，烙印全满≈1.0 万 vs 里程碑供给 180 万+，
// 供需已倒挂，发道韵加剧后期「点不动」（勘察二红线）；走贡献（喂洞府/宗门商店）
// +灵石（喂点数兑换）+模拟点数（喂核心轮回）三层回到既有循环。
// 文案收编真相源：title_n=名（N2728..2735）/ desc_n=描述（N2736..2743）逐条连号，
// 页签「门中差事」N2727；达标句 N2744、复命发奖模板 N2745。
// 【挂点语义一句话写死】「斩除宿敌」进度：现实寻仇胜利（revenge_win）与模拟反杀
// （sim 侧阵斩宿敌）两挂点均计——宿敌授首语义统一（fame 挂点②同款双臂口径）；
// 「扩建洞府」= 三设施任一升 1 级均计；「供奉丹火」= 现实坊市购丹（buy_pill），
// 服丹/模拟侧不计；「破境而上」= 现实跨境破境成功臂（act_break 跨境 ok 臂，
// 1566 句所在分支——v1.16.42 实测改准：同境进层 ok 臂 1564 句不挂 comm_bump 不计，
// 模拟侧破境不计；接线钉 v11642_comm_wire_tests::comm_hooks_static_pins 写死）。
pub const COMM_QUEST: u8 = 0; // 宗门任务（sect_quest 成功）
pub const COMM_PILL: u8 = 1; // 坊市购丹（buy_pill 成功）
pub const COMM_EXP: u8 = 2; // 秘境远征战获（push_loot 单口）
pub const COMM_DONGFU: u8 = 3; // 洞府设施扩建（dongfu_up 成功）
pub const COMM_NEME: u8 = 4; // 斩杀宿敌（revenge_win + 模拟反杀双挂点）
pub const COMM_BREAK: u8 = 5; // 现实破境（act_break ok 臂）
pub const COMM_GIFT: u8 = 6; // 赠礼故人（gift_npc 成功）

#[derive(Clone, Copy)]
pub struct CommDef {
    pub title_n: u16,    // 委托名（真相源 N）
    pub desc_n: u16,     // 委托描述（真相源 N）
    pub kind: u8,        // 目标类别（上方 COMM_* 常量，挂点一一对应）
    pub target: u8,      // 目标次数（≥1；进度 prog saturating 封顶）
    pub rw_contrib: u32, // 复命奖励：宗门贡献（meta.sect_contrib）
    pub rw_stones: i32,  // 复命奖励：灵石（life.stones；0=不发）
    pub rw_pts: u8,      // 复命奖励：模拟点数（meta.sim_points）
}

pub const COMMISSIONS: [CommDef; 8] = [
    CommDef {
        title_n: 2728,
        desc_n: 2736,
        kind: COMM_QUEST,
        target: 2,
        rw_contrib: 30,
        rw_stones: 400,
        rw_pts: 1,
    },
    CommDef {
        title_n: 2729,
        desc_n: 2737,
        kind: COMM_QUEST,
        target: 4,
        rw_contrib: 50,
        rw_stones: 800,
        rw_pts: 1,
    },
    CommDef {
        title_n: 2730,
        desc_n: 2738,
        kind: COMM_PILL,
        target: 3,
        rw_contrib: 20,
        rw_stones: 600,
        rw_pts: 1,
    },
    CommDef {
        title_n: 2731,
        desc_n: 2739,
        kind: COMM_EXP,
        target: 1,
        rw_contrib: 60,
        rw_stones: 1200,
        rw_pts: 1,
    },
    CommDef {
        title_n: 2732,
        desc_n: 2740,
        kind: COMM_DONGFU,
        target: 1,
        rw_contrib: 80,
        rw_stones: 0,
        rw_pts: 1,
    },
    CommDef {
        title_n: 2733,
        desc_n: 2741,
        kind: COMM_NEME,
        target: 1,
        rw_contrib: 0,
        rw_stones: 2000,
        rw_pts: 2,
    },
    CommDef {
        title_n: 2734,
        desc_n: 2742,
        kind: COMM_BREAK,
        target: 1,
        rw_contrib: 0,
        rw_stones: 1500,
        rw_pts: 2,
    },
    CommDef {
        title_n: 2735,
        desc_n: 2743,
        kind: COMM_GIFT,
        target: 2,
        rw_contrib: 40,
        rw_stones: 800,
        rw_pts: 1,
    },
];

// 长老委托独立文案 N（真相源尾段追加，N 永不变号）：
pub const COMM_DONE_N: u16 = 2744; // 达标句「差事已成，可去复命」（toast+金行 log 同句）
pub const COMM_CLAIM_N: u16 = 2745; // 复命发奖模板「复命：贡献+{}，灵石+{}，模拟点数+{}」

// 长老委托文案查表口（真相源 N 直查；越界 min 钳制照 guide_tab_label 先例——
// panic=abort 纪律）
pub fn comm_title(i: usize) -> &'static str {
    evtx(COMMISSIONS[i.min(COMMISSIONS.len() - 1)].title_n as usize)
}

pub fn comm_desc(i: usize) -> &'static str {
    evtx(COMMISSIONS[i.min(COMMISSIONS.len() - 1)].desc_n as usize)
}

/// 剧情可玩性·上品委托档位（v1.16.42 勘察二主选 A）：每 5 条复命第 5 条为上品——
/// done=累计复命数（claim 发奖取自增前值），done%5==4 即第 5/10/15…条上品。
/// 5 与表长 8 互素（LCM=40）：全部 8 条委托终会轮到上品档。纯派生零新状态零存档
/// 变更；加码只走贡献/灵石/模拟点数三层单点乘区（通胀红线：禁发道韵，上方头注
/// :3855 口径）。UI 徽标（draw_comm_view/长老卡）与发奖同谓词同拍。
pub fn comm_premium(done: u32) -> bool {
    done % 5 == 4
}

// ---------------- v1.16.23 减包刀②：事件臂文案资产化（textpack）----------------
// build_event 臂内 >=24B 中文常量串由 tools/pack_text.py 收编进 assets/textpack.bin
// （raw-deflate，与 assets/font.ttf.deflate 同款通路），源码侧为 evtx(N)/* 原文：… */
// （原文注释回填保 diff 可审计）；真相源 assets/text_source.txt（N<TAB>源码转义形态），
// 文案编辑改 source 后重跑 pack_text.py 即可，旧 N 永不变号；块内新写的裸字面量
// 功能照常（只是未压缩），重跑脚本自动收编（append 新 N）。
// v1.16.27 减包刀①/②/③：① build_event 臂内 format!("模板{}", a)（含多行）整调用
// 收编——evf(N, &[&a])（见 evf 注）；②/③ EVENT_NAMES/EVENT_BRIEF 双表 291×2 条
// 收编进连号段 820..=1110/1111..=1401，表本体换 EvTable 查表器（见两 static 注）。
// ⑥ EVENTS 位域化见 EVENTS_ROWS/EVENTS 注（打包 static，与 textpack 无关）。
// boot 一次性 inflate（main.rs 启动序列，数十 KB 量级随收编增长，miniz inflate
// <5ms）；evtx() 为
// O(1) 下标切片，帧路径零分配（性能十戒）；文案内容逐字节等价（脚本往返断言），
// 字体 cmap 与 charset_diff_check 口径不变；存档只存事件 id，零兼容影响。
// v1.16.37 textpack 外置（wasm）：include_bytes 整块 cfg(not wasm32) 包住——照
// ui.rs:17/art.rs:308 双先例，必须 cfg 静态本体本身，只 cfg unpack_text 或调用方
// 字节照样进 wasm 数据段（ui.rs:10 教训原文重申）。wasm 侧由 index.html 与 wasm
// 下载全并行 fetch textpack.bin?v=GAME_VER，经 env 导入 xx_text_len/xx_text_read
// 回传（main.rs boot 头轮询，失败 state(96) 如实报错，见 init_text_from_bytes 注），
// wasm 数据段减去整个 blob（v1.16.36 时点实测 83,065B）；miniz_oxide 解码器因
// ui.rs 字体 inflate 仍留在 wasm，勿把「codec 出 wasm」写进任何账目。桌面端内嵌
// 口径不变、构建产物零变化。
#[cfg(not(target_arch = "wasm32"))]
pub static TEXTPACK: &[u8] = include_bytes!("../assets/textpack.bin");
// 生成件（pack_text.py 产出）：pub static TXT_ANCH: [u32; A]（每 K32=128 条一个绝对
// 锚，锚数 A=ceil(行数/128)——v1.16.34 单3 末态 2,657 行→21 锚（K32=128
// 粗化后锚距；历史注「1,971 行→62 锚」为 32 距口径，已随粗化清账）+
// pub static TXT_DELTA: [u16; N]（组内 delta，off(i)=ANCH[i>>7]+DELTA[i]——
// v1.16.28 刀C 两级表，v1.16.32 单1 组距 32→128 粗化后 shift 5→7；旧 [u32;N]
// 5772B→两级 3174B）+ TXT_NAME_BASE/TXT_BRIEF_BASE/
// TXT_NAME2_BASE/TXT_BRIEF2_BASE/TXT_NAME3_BASE/TXT_BRIEF3_BASE（段一~三基址）/
// TXT_NAME4_BASE/TXT_BRIEF4_BASE（v1.16.29 段四基址，因单1/单2 占 N1492..1824
// 顺延 1825/1833）+ TXT_NAME5_BASE/TXT_BRIEF5_BASE（v1.16.30 段五 1865/1873）+
// TXT_NAME6_BASE/TXT_BRIEF6_BASE（v1.16.31 段六 1905/1915）+
// TXT_NAME7_BASE/TXT_BRIEF7_BASE（v1.16.31 新功能段七 1954/1957）+
// TXT_NAME8_BASE/TXT_BRIEF8_BASE（v1.16.32 单3 扩池段八 2338/2344——单1/单2
// 已用 N1971..2337 故段八顺延）十六常量，item 位置
// include! 展开（随收编追加
// 自动重产，勿手改；历史槽数注 577/623/648/671/1403/1443/1492 均已过期清账——
// 槽数=条数+1，随收编增长：0..=669 事件臂文案 + 670..=819 事件臂 format! 模板
//（刀①）+ 820..=1110 EVENT_NAMES 段一 + 1111..=1401 EVENT_BRIEF 段一 +
// 1402..=1409 名段二/1410..=1417 简段二（v1.16.27 扩池 8 条）+ 1418..=1441
// 扩池臂文案（v1.16.27 收编 24 站）+ 1442..=1451 名段三/1452..=1461 简段三
//（v1.16.28 扩池 10 条）+ 1462..=1491 扩池臂文案（v1.16.28 收编 30 站）+
// 1492..=1605 块外 evf 模板（v1.16.29 单1 刀A 收编 114 站）+ 1606..=1725 表
// 通道字段级（单1 刀B：CHAPTERS intro/open/reward_note/couplet/lore+
// ENDINGS 四字段+LOCATIONS desc+HINTS text+NPC_TALES+HELP_LINES 共 120 行）+
// 1726..=1824 块外散件 evtx（单1 刀C 99 站，N1825=名段四首行——v1.16.30 单2
// 清账 off-by-one；内含 1800/1801 两行 INHERITS
// const 语境回退后的 dead rows）+ 1825..=1832 名段四/1833..=1840 简段四
//（v1.16.29 扩池 8 条）+ 1841..=1864 扩池臂文案（v1.16.29 收编 24 站）+
// 1865..=1872 名段五/1873..=1880 简段五（v1.16.30 扩池 8 条，单2 改写即减字
// 后下一 N=1865）+ 1881..=1904 扩池臂文案（v1.16.30 手写 24 站——本批正文
// 随段五连号先行落真相源，pack_text 幂等 0 收编）+ 1905..=1914 名段六/
// 1915..=1924 简段六（v1.16.31 扩池 10 条，手写连号段先于正文）+ 1925..=1953
// 扩池臂文案（v1.16.31 收编 29 站 build_event 臂内=23 evtx+6 evf 恰占
// N1925..1953——apply_eff special75 slog 2 站当时未收编（块外不在 build_event
// 扫描域，v1.16.31 复审必修①发现本注原写「+ apply_eff special75 slog 2 站
// = 29 站」账目失实，同批按 --block-lit 补收编）；其中 N1145/1147/1156/1170/
// 1175/1176/1191/1401/1591 九行为 v1.16.31 第十批简语改写行直改减字 −72B，
// N 不变号）+
// 1954..=1956 名段七/1957..=1959 简段七（v1.16.31 新功能 3 条——段六名/简
// 紧连封死后续批开新段照段四→段五先例）+ 1960..=1968 新功能臂文案（3 条
// ×正文+2msg 全静态=9 evtx，零动态句零 evf）+ 1969..=1970 apply_eff special75
// slog 2 站（--block-lit 块外 ≥40B 散件通道补收编，≥40B 口径照 v1.16.29 单1
// 刀C——24..39B 档实测净负不收）+
// 1971..=2337 tables2 第三批（v1.16.32 单1 主刀+副刀+小表批 367 行：CHAPTERS
// num/name/goals.text 36 + TECHS 16/RELICS 16/PILLS 12/TALENTS 66/UPGRADES 20/
// ACHS 60/ORIGINS 12/MAP_THEMES 10/REGION_MECH 10/SECT_GOODS 12/NPC_EVENTS 8/
// INHERITS 18/EXPED_STEPS 补收 8 = 字段级 304 + MINOR_LINES/COMP_TALES/
// NPC_BRIEF/DONGFU_FAC_DESC/WEATHER_FLAVOR/EXPED_CHOICE_LABELS/AD_CREATIVES/
// DONE_NOTE/CHAIN_NAMES 整表 63；EXPED_STEPS 第二轮补收 2330..=2337 在整表批
// 2312..=2329 之后连号（首 N=2330，无空洞）；
// 全行逐字节等价：tools/tables2_verify_once.py 收编前快照 25 面对账 0 失败 +
// game.rs tables2_pins 首尾钉测）；
// K32 锚距 32→128 粗化（TXT_ANCH 73→19 锚，txt_off i>>7 逐值等价——组距上限
// u16 域由 pack_text.py delta assert 固化）；
// v1.16.32 单2 第十一批改写行直改（N 不变号）：简语 8 行 N1141/1155/1189/1199/
// 1326/1341/1364/1390（id30/44/78/88/215/230/253/279，−48B，kw 钉
// event_brief_11632_batch11_within_18——253 锚用「残卷」不用勘察拟的「拓本」，
// 后者不在新简内）+ 金句B N1588（world.rs 抵达句模板 −9B，原文注释已同步）+
// N1958 用字对齐 336 臂「迎潮而立」0B；模拟首登金句A 为 wasm 裸串（game.rs
// slog，不入真相源）−6B；
// v1.16.32 单3 扩池段八：名段八 N2338..=2343（TXT_NAME8_BASE=2338）+ 简段八
// N2344..=2349（TXT_BRIEF8_BASE=2344）真相源手写连号段先于正文 + 六臂正文
// 收编 N2350 起（EV_SEG7=338 段界登记，len7 改段界差 EV_SEG7−EV_SEG6、
// len8=N_EVENTS−EV_SEG7——下批扩池续段九时 len8 同步改段界差，见 EV_SEG7 注）；
// N 追加不变号，段基址稳定）
// v1.16.33 单1 N 化批 N2368..=2575（--nize/--block-lit/--block-tmpl 通道）：
// special 副标 66 臂+兜底「机缘」N2368..=2433（SPECIAL_SUB [u16;77] 查表，
// 「机缘」占 2368、臂 10「魔窟探宝」dedup 共享既有行 843）+ const 串表 N 化
// N2434..=2522（DUAL_CULT_HINT 2434/GUIDE_PANEL_NOTE 2435 + 八表 104 元素
// dedup 后 89 行——COMP_PAGE_PREWARM_16/RECALL_CHROME/DUAL_CULT_HINT 引用
// 同串共享）+ draw.rs/charpanel.rs 块外延后批收编 N2523..=2575（30 evf+
// 39 evtx）；单2 行直改 9 行 N1307/1359/1377/1414/1417/1595/1873/1874/1959
// （第十二批简语 8 行+金句A 采药句，N 不变号，−67B）；
// v1.16.33 单3 扩池段九：名段九 N2576..=2580（TXT_NAME9_BASE=2576）+ 简段九
// N2581..=2585（TXT_BRIEF9_BASE=2581）真相源手写连号段先于正文 + 五臂正文
// 收编 N2586..=2600（15 站=11 evtx+4 evf；EV_SEG8=344 快照常量差写法兑现，
// len8=(EV_SEG8−EV_SEG7)、len9=N_EVENTS−EV_SEG8——段十预言由 v1.16.34
// 单3 兑现（len9 改 (EV_SEG9−EV_SEG8) 段界差）；
// v1.16.34 单1 刀α/β+γ 精度模板+臂表收编 N2601..=2646（27 站精度模板
// 变换 {} 入 blob + 26 行臂字面量，DualGate::ThisYear 单臂 dedup 共享既有
// 行 N2453）+ 单2 行直改 10 行 N1175/1198/1327/1334/1380/1456/1458/1460/
// 1596/1597（第十三批简语 8 行+loot 结论句 2 行，N 不变号，−46B）；
// v1.16.34 单3 扩池段十：名段十 N2647..=2648（TXT_NAME10_BASE=2647）+ 简
// 段十 N2649..=2650（TXT_BRIEF10_BASE=2649）真相源手写连号段先于正文 +
// 两臂正文收编 N2651..=2656（6 站=3 evf+3 evtx；EV_SEG9=349 快照常量差
// 写法兑现，len9=(EV_SEG9−EV_SEG8)、len10=N_EVENTS−EV_SEG9）；
// v1.16.35 单3 扩池段十一：名段十一 N2657..=2658（TXT_NAME11_BASE=2657）+
// 简段十一 N2659..=2660（TXT_BRIEF11_BASE=2659）真相源手写连号段先于正文 +
// 两臂正文收编 N2661..=2666（6 站=2 evf+4 evtx；EV_SEG10=351 快照常量差
// 写法兑现，len10=(EV_SEG10−EV_SEG9)、len11=N_EVENTS−EV_SEG10）；末态 2,667
// 行、TXT_ANCH 21 锚、TXT_DELTA [u16;2668]；N 追加不变号，段基址稳定）
include!("text_offsets.inc");

static TEXT_BLOB: std::sync::OnceLock<&'static [u8]> = std::sync::OnceLock::new();

/// 解包单源（get_or_init 的初始化闭包）：成功返回 leak 后的 blob，失败/校验不
/// 过返回 EMPTY 哨兵——INIT_TRIED 钉死「只试一次」，失败后不再重试（防帧路径
/// 反复 inflate）。EMPTY 即失败态（真实 blob 恒非空：2,657 条非空文案串接，
/// 随收编增长勿当硬断言——v1.16.26 注 647/v1.16.29 注 1,492/1,865/
/// v1.16.30 注 1,905/v1.16.31 注 1,954/1,969 均已过期清账）。
static TEXT_INIT_TRIED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn unpack_text() -> &'static [u8] {
    use std::sync::atomic::Ordering;
    static EMPTY: &[u8] = &[];
    if TEXT_INIT_TRIED.swap(true, Ordering::Relaxed) {
        return EMPTY;
    }
    // v1.16.37 textpack 外置拆分（作用域变更说明）：原函数体是一个 match 尾表达式，
    // 现拆为双 cfg 分支的先赋值再返回写法——桌面分支体与原 match 逐字等价（仅挪进
    // 代码块），wasm 分支为兜底桩（见块内注）；不写成「cfg 尾 match + cfg 尾块」
    // 是因为带属性的块尾表达式存在语句/尾表达式歧义，显式赋值消除解析歧义。
    let blob: &'static [u8];
    #[cfg(not(target_arch = "wasm32"))]
    {
        blob = match miniz_oxide::inflate::decompress_to_vec(TEXTPACK) {
            Ok(v) => {
                // from_utf8 全量校验一次：之后 evtx() 的 unchecked 切片以此为安全论证
                if core::str::from_utf8(&v).is_ok() {
                    Box::leak(v.into_boxed_slice()) as &'static [u8]
                } else {
                    EMPTY
                }
            }
            Err(_) => EMPTY,
        };
    }
    #[cfg(target_arch = "wasm32")]
    {
        // v1.16.37：wasm 无内嵌 blob（TEXTPACK 本体已 cfg 剪出数据段），本函数
        // 降级为惰性兜底桩——正常路径 boot 的 init_text_from_bytes 已先行
        // TEXT_BLOB.set，get_or_init 直达已装快路径根本不会调到此处；仅防漏装
        // 环境首调 evtx 时占坑成终态 EMPTY（文案缺失但不崩）。EMPTY 在 wasm 绝
        // 不可为归宿：失败收口归 main.rs boot 的 state(96)，此处绝不静默重试。
        blob = EMPTY;
    }
    blob
}

/// boot 一次性解包（main.rs 启动序列调用；幂等）。get_or_init 并发安全：
/// 并发首调阻塞等待完成，其余调用直达已装快路径。解压失败/UTF-8 校验失败
/// 时不装 blob——evtx() 全返空串（文案缺失但不崩，wasm trap 纪律）。
pub fn init_text() {
    TEXT_BLOB.get_or_init(unpack_text);
}

/// v1.16.37 textpack 外置（wasm 专用装载口，照 ui.rs init_font_from_bytes 先例）：
/// index.html xx_text 插件回传的 deflate 字节 → miniz inflate → UTF-8 全量校验 →
/// leak → TEXT_BLOB.set。与桌面 unpack_text 解压/校验/leak 逐项同口径；唯一差异
/// 是失败不落 EMPTY 哨兵而返 false——wasm 失败归宿由 main.rs boot 收口 state(96)
/// （可点重试），绝不静默落 EMPTY 继续跑（满屏空文案比黑屏更劣）。已占坑（防御：
/// evtx 惰性兜底先行占了 OnceLock）或解压/UTF-8 失败或产空一律 false，不覆写。
#[cfg(target_arch = "wasm32")]
pub fn init_text_from_bytes(deflated: &[u8]) -> bool {
    if TEXT_BLOB.get().is_some() {
        return false;
    }
    let v = match miniz_oxide::inflate::decompress_to_vec(deflated) {
        Ok(v) => v,
        Err(_) => return false,
    };
    // from_utf8 全量校验一次：之后 evtx() 的 unchecked 切片以此为安全论证；
    // 空产出等价失败态（真实 blob 恒非空——EMPTY 即失败哨兵，见 unpack_text 注）
    if core::str::from_utf8(&v).is_err() || v.is_empty() {
        return false;
    }
    TEXT_BLOB
        .set(Box::leak(v.into_boxed_slice()) as &'static [u8])
        .is_ok()
}

/// O(1) 查池：blob[off(i)..off(i+1)]。越界/未初始化一律空串兜底
/// （绝不回错文案；i 由脚本分配的编译期常量，正常路径恒界内）。
/// 惰性兜底单源：漏调 init_text 的环境（cargo test 等）首调自动解包——
/// get_or_init 的 once 语义保证并发查池者阻塞至完成（多线程测试不竞态空串）；
/// 正常路径（boot 已装）等价一次原子 load，帧路径零影响。
///
/// v1.16.28 减包刀C：off(i) = TXT_ANCH[i>>5] + TXT_DELTA[i]（v1.16.32 单1 组距
/// 32→128 粗化：off(i) = TXT_ANCH[i>>7] + TXT_DELTA[i]，逐值等价——每 K32=128 条一个
/// u32 绝对锚 + u16 组内 delta，产物侧 5772B→3174B）——O(1) 纯算术零分配，
/// 输出与旧 [u32;N] 逐值等价（pack_text.py 产侧 assert + 本文件钉测双兜底）
pub fn evtx(i: usize) -> &'static str {
    let blob = TEXT_BLOB.get_or_init(unpack_text);
    if blob.is_empty() {
        return "";
    }
    // 防御边界：i >= len-1 即空（i+1 需界内）；写 saturating_sub 令 usize::MAX
    // 这类极端值不触发加法溢出（dev profile 溢出即 panic——panic=abort 纪律）
    if i >= TXT_DELTA.len().saturating_sub(1) {
        return "";
    }
    let s = txt_off(i);
    let e = txt_off(i + 1);
    if s <= e && e <= blob.len() {
        // 安全论证：TEXT_BLOB 只在 from_utf8 全量通过后装入，切片必为合法 UTF-8
        unsafe { core::str::from_utf8_unchecked(&blob[s..e]) }
    } else {
        ""
    }
}

/// v1.16.28 刀C 两级 offset 解码单口：off(i) = ANCH[i>>7] + DELTA[i]（单1 组距
/// 32→128 粗化后 shift 7——原注 i>>5 已随粗化清账）。
/// K32=128 与 pack_text.py 产侧同源（delta 域 u16 由产侧 assert 固化——组距
/// 128 行现实行宽 ~9KB « 65535）；
/// #[inline] 供 evtx 两次取界零开销（纯算术零分配——帧路径纪律）
#[inline]
fn txt_off(i: usize) -> usize {
    TXT_ANCH[i >> 7] as usize + TXT_DELTA[i] as usize
}

/// v1.16.27 减包刀①：format! 模板收编——`evf(N, &[&a, &b])` ≡ `format!(<N 号模板>, a, b)`。
/// blob 模板只含 "{}" 占位（pack_text.py 收编纪律 assert 固化：无 {:spec}、无 {{}}
/// 转义、占位数==实参数），等价性 = format! 的 "{}" Display 语义：split("{}") 得
/// 前后缀片段，依序「片段·Display(实参)」拼装——每段实参只消费一个占位，替换产物
/// 不被二次扫描（replacen 逐位替换会把实参里的 "{}" 再吃掉，语义不等，弃用）。
/// 实参 &dyn Display 按引用传：format! 同款借用语义（不 move，同一 String 实参可
/// 跨臂复用；String/i32/&str 异型混排与 format! 一致），求值顺序一致（&[...] 依序
/// 求值一次）。事件触发期路径（非帧路径），分配量与原 format! 相当（一次 String）。
pub fn evf(n: usize, args: &[&dyn core::fmt::Display]) -> String {
    let t = evtx(n);
    let mut out = String::with_capacity(t.len() + 16 * args.len());
    let mut pieces = t.split("{}");
    if let Some(p) = pieces.next() {
        out.push_str(p);
    }
    let nargs = args.len();
    let nph = t.matches("{}").count();
    for k in 0..nph {
        if k < nargs {
            use core::fmt::Write as _;
            let _ = out.write_fmt(format_args!("{}", args[k]));
        } else {
            // 缺位兜底：以字面 "{}" 补显（可见、不静默吞位；format! 同场景为 panic，
            // 本函数不崩纪律下取最诚实输出——重建结果恒可回拼模板原文）
            out.push_str("{}");
        }
        if let Some(p) = pieces.next() {
            out.push_str(p);
        }
    }
    // 实参多于占位：多余实参不消费（format! 同场景编译期报错；此处静默忽略不崩，
    // pack_text.py 收编纪律已断言占位数==实参数，正常路径不达）
    out
}

// ---------------- v1.16.34 单1 刀α：精度 Display 包装（--block-prec） ----------------
/// evf 精度模板的实参载体：blob 模板占位统一纯 {}，精度语义（{:.0}/{:.1}/{:.2}）
/// 由包装类型的 Display impl 承接——write!(f, "{:.N}", self.0) 与原 format!(
/// "…{:.N}…", v) 走同一 f64 Display Formatter 路径，产物逐字节等价（边界值
/// 0.5/0.25/0.999 钉测 evf_prec_format_equivalence 验收）。调用点由
/// pack_text.py --block-prec 通道按占位序生成 `&(FmtN(expr)) as &dyn Display`；
/// 构造是 Copy 小结构零堆分配，每实参一次 write_fmt（与原 format! 同量）。
pub struct Fmt0(pub f64);
impl core::fmt::Display for Fmt0 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:.0}", self.0)
    }
}
pub struct Fmt1(pub f64);
impl core::fmt::Display for Fmt1 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:.1}", self.0)
    }
}
pub struct Fmt2(pub f64);
impl core::fmt::Display for Fmt2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:.2}", self.0)
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
        // 全表界内：v1.16.28 刀C 两级表——重建 offs 逐条单调且末界 == blob 长
        //（off(i) = ANCH[i>>7] + DELTA[i]，与产侧 assert 双兜底——单1 粗化后
        // shift 7，原注 i>>5 已清账）
        assert!(TXT_DELTA.len() >= 2);
        let off = |i: usize| txt_off(i);
        for k in 0..TXT_DELTA.len() - 1 {
            assert!(off(k) <= off(k + 1), "offs[{}] not monotonic", k);
        }
        assert_eq!(
            off(TXT_DELTA.len() - 1),
            TEXT_BLOB.get().copied().map_or(0, |b| b.len())
        );
        // 越界防御：空串兜底（不 panic）
        assert_eq!(evtx(TXT_DELTA.len()), "");
        assert_eq!(evtx(usize::MAX), "");
    }

    // v1.16.27 刀①等价性钉测：evf 与 format! 的 "{}" Display 语义逐字节一致——
    // 在真实收编模板行上用哨兵实参对齐 format! 重建（单/双占位、String/i32 异型、
    // 占位多于实参兜底不崩且残余照拼）
    #[test]
    fn evf_format_equivalence() {
        init_text();
        // 全表扫样：首个含 1/2 个 "{}" 的模板行
        let n = TXT_DELTA.len() - 1;
        let mut got1 = None;
        let mut got2 = None;
        for k in 0..n {
            let c = evtx(k).matches("{}").count();
            if c == 1 && got1.is_none() {
                got1 = Some(k);
            }
            if c == 2 && got2.is_none() {
                got2 = Some(k);
            }
        }
        // 单占位 × String 实参
        let k1 = got1.expect("无单占位模板行（模板段缺行？）");
        let t1 = evtx(k1);
        let s = "甲乙丙".to_string();
        let p: Vec<&str> = t1.split("{}").collect();
        let expect = format!("{}{}{}", p[0], s, p[1]);
        assert_eq!(evf(k1, &[&s]), expect, "N={} 单占位与 format! 漂移", k1);
        // 双占位 × String+i32 异型实参（format! 同款逐位消费）
        let k2 = got2.expect("无双占位模板行（模板段缺行？）");
        let t2 = evtx(k2);
        let i = 7i32;
        let p2: Vec<&str> = t2.split("{}").collect();
        let expect2 = format!("{}{}{}{}{}", p2[0], s, p2[1], i, p2[2]);
        assert_eq!(evf(k2, &[&s, &i]), expect2, "N={} 双占位与 format! 漂移", k2);
        // 实参替换产物不被二次扫描：实参本身含 "{}" 时后续占位照常对位
        let tricky = "空{}占位".to_string();
        let expect3 = format!("{}{}{}{}{}", p2[0], tricky, p2[1], i, p2[2]);
        assert_eq!(evf(k2, &[&tricky, &i]), expect3, "替换产物被重扫（语义漂）");
        // 缺位兜底：占位多于实参——缺位以字面 "{}" 补显（不静默吞位，重建回模板
        // 原文；format! 同场景 panic，本函数不崩纪律）
        assert_eq!(evf(k1, &[]), t1);
    }

    // v1.16.27 刀②/③等价性钉测：EvTable 查表器段界与首末行内容——
    // 全表内容对账由 game.rs event_names_match_card_titles（运行期 vs build_event
    // 标题，全 id 扫）与 tables_recheck.py（真相源行级）双线兜底
    #[test]
    fn ev_tables_section_bounds() {
        init_text();
        // v1.16.30 扩池：len = 段一 291 + 段二 8 + 段三 10 + 段四 8 + 段五 8 = 325
        // v1.16.31 扩池：+ 段六 10 = 335；v1.16.31 新功能：+ 段七 3 = 338
        // v1.16.32 单3 扩池：+ 段八 6 = 344
        // v1.16.33 单3 扩池：+ 段九 5 = 349
        // v1.16.34 单3 扩池：+ 段十 2 = 351
        // v1.16.35 单3 扩池：+ 段十一 2 = 353
        // v1.16.36 单3 扩池：+ 段十二 3 = 356
        // v1.16.37 单2 扩池：+ 段十三 3 = 359
        // v1.16.38 单2 扩池：+ 段十四 3 = 362
        // v1.16.40 单3 扩池：+ 段十五 3 = 365
        // v1.16.43 单3 扩池：+ 段十六 3 = 368
        // v1.16.44 扩池：+ 段十七 3 = 371
        // v1.16.45 施工单三扩池：+ 段十八 3 = 374
        // v1.16.46 段十九扩池：+ 段十九 3 = 377
        // v1.16.47 段二十扩池：+ 段二十 3 = 380
        // v1.16.48 段二十一扩池：+ 段二十一 3 = 383
        // v1.16.49 段二十二扩池：+ 段二十二 3 = 386
        // v1.16.50 段二十三扩池：+ 段二十三 3 = 389
        // v1.16.51 段二十四扩池：+ 段二十四 3 = 392
        // v1.16.52 段二十五扩池：+ 段二十五 3 = 395
        // v1.16.53 段二十六扩池：+ 段二十六 3 = 398
        assert_eq!(EVENT_NAMES.len(), 398);
        assert_eq!(EVENT_BRIEF.len(), 398);
        assert_eq!(EVENT_NAMES.get(0), Some("灵草满坡"));
        assert_eq!(EVENT_NAMES.get(290), Some("误入猎套"));
        assert_eq!(EVENT_BRIEF.get(0), Some("满坡灵草露水未干，采满一筐"));
        // v1.16.31 第十批改写后随批同步（原「山道误踩索套，猎户索赔偿」−6B）
        assert_eq!(EVENT_BRIEF.get(290), Some("索套一紧，猎户索赔偿"));
        // 段二（v1.16.27 扩池 8 条，TXT_NAME2_BASE=1402/TXT_BRIEF2_BASE=1410）
        assert_eq!(EVENT_NAMES.get(291), Some("换窗新纸"));
        assert_eq!(EVENT_NAMES.get(298), Some("灵脉被截"));
        assert_eq!(EVENT_BRIEF.get(291), Some("霜日重糊窗，邻童按纸角"));
        // v1.16.33 单2 第十二批改写后随批同步（原「灵脉被新契截走，循山另引」−12B；
        // kw 拦腰截走取自 id298 臂 N1439 verbatim）
        assert_eq!(EVENT_BRIEF.get(298), Some("新贵拦腰截走灵脉"));
        // 段三（v1.16.28 扩池 10 条，TXT_NAME3_BASE=1442/TXT_BRIEF3_BASE=1452）：
        // len2 漂移唯一内容级防线——首尾逐字钉，错行（非空串）在此必红
        assert_eq!(EVENT_NAMES.get(299), Some("合誊旧稿"));
        assert_eq!(EVENT_NAMES.get(308), Some("醉后忘句"));
        assert_eq!(EVENT_BRIEF.get(299), Some("灯下合誊两册旧稿为一卷"));
        assert_eq!(EVENT_BRIEF.get(308), Some("醉道人醉后念诀醒后不认"));
        // 段四（v1.16.29 扩池 8 条，TXT_NAME4_BASE=1825/TXT_BRIEF4_BASE=1833，
        // 单1/单2 已用 N1492..1824 故段四顺延）：len3 漂移唯一内容级防线——
        // 首尾逐字钉，len3 写 N_EVENTS−EV_SEG2 静默吞段四时在此必红
        assert_eq!(EVENT_NAMES.get(309), Some("雪堤喂雀"));
        assert_eq!(EVENT_NAMES.get(316), Some("药童卷方"));
        assert_eq!(EVENT_BRIEF.get(309), Some("雪落堤上，你撒谷喂雀"));
        assert_eq!(EVENT_BRIEF.get(316), Some("药童卷走三张丹方去了邻城"));
        // 段五（v1.16.30 扩池 8 条，TXT_NAME5_BASE=1865/TXT_BRIEF5_BASE=1873，
        // 单2 改写后下一 N=1865）：len4 漂移唯一内容级防线——首尾逐字钉，
        // len4 写 N_EVENTS−EV_SEG3 静默吞段五时在此必红
        assert_eq!(EVENT_NAMES.get(317), Some("崖灯照径"));
        assert_eq!(EVENT_NAMES.get(324), Some("旧卷新墨"));
        // v1.16.33 单2 第十二批改写后随批同步（原「危崖挂灯，夜行人少绕三里」−3B；
        // kw 灯油/添满取自 id317 臂 N1881 verbatim）
        assert_eq!(EVENT_BRIEF.get(317), Some("危崖悬灯，灯油添满三夜"));
        assert_eq!(EVENT_BRIEF.get(324), Some("百年旧卷里，认出你自己的字迹"));
        // 段六（v1.16.31 扩池 10 条，TXT_NAME6_BASE=1905/TXT_BRIEF6_BASE=1915，
        // 手写名/简连号段先于正文收编）：len5 漂移唯一内容级防线——首尾逐字
        // 钉，len5 写 N_EVENTS−EV_SEG4 静默吞段六时在此必红
        assert_eq!(EVENT_NAMES.get(325), Some("坛泥留字"));
        assert_eq!(EVENT_NAMES.get(334), Some("接梁旧漆"));
        assert_eq!(EVENT_BRIEF.get(325), Some("坛泥上替醉道人留句百年后的话"));
        assert_eq!(EVENT_BRIEF.get(334), Some("接大殿朽梁心，旧漆不补"));
        // 段七（v1.16.31 新功能 3 条，TXT_NAME7_BASE=1954/TXT_BRIEF7_BASE=1957，
        // 段六名/简紧连封死后续批开新段照段四→段五先例）：len6 漂移唯一内容级
        // 防线——首尾逐字钉，len6 写 N_EVENTS−EV_SEG5 静默吞段七时在此必红
        assert_eq!(EVENT_NAMES.get(335), Some("连雨收书"));
        assert_eq!(EVENT_NAMES.get(337), Some("三脉同焕"));
        assert_eq!(EVENT_BRIEF.get(335), Some("连雨护书，雨停分文不取"));
        // v1.16.33 单2 第十二批改写后随批同步（原「三设施全满，长老登山观府」
        // −3B；kw 同焕/叩门取自 id337 臂 N1966 verbatim）——首版漏同步当场红
        assert_eq!(EVENT_BRIEF.get(337), Some("三脉同焕，长老登山叩门"));
        // 段八（v1.16.32 单3 扩池 6 条，TXT_NAME8_BASE=2338/TXT_BRIEF8_BASE=
        // 2344，手写名/简连号段先于正文收编——单1/单2 已用 N1971..2337 故段八
        // 顺延）：len7 漂移唯一内容级防线——首尾逐字钉，len7 写 N_EVENTS−
        // EV_SEG6 静默吞段八时在此必红
        assert_eq!(EVENT_NAMES.get(338), Some("暮坛对酌"));
        assert_eq!(EVENT_NAMES.get(343), Some("老炉现纹"));
        assert_eq!(EVENT_BRIEF.get(338), Some("泥封启处，醉话入稿"));
        assert_eq!(EVENT_BRIEF.get(343), Some("老炉细纹，岁晚难补"));
        // 段九（v1.16.33 单3 扩池 5 条，TXT_NAME9_BASE=2576/TXT_BRIEF9_BASE=
        // 2581，手写名/简连号段先于正文收编——单1 N 化批已用 N2368..2575 故
        // 段九顺延）：len8 漂移唯一内容级防线——首尾逐字钉，len8 写
        // N_EVENTS−EV_SEG7 静默吞段九时在此必红（EV_SEG8=344 快照兜底）
        assert_eq!(EVENT_NAMES.get(344), Some("同裱旧稿"));
        assert_eq!(EVENT_NAMES.get(348), Some("霜橘分甘"));
        assert_eq!(EVENT_BRIEF.get(344), Some("旧稿起皱，同裱成轴"));
        assert_eq!(EVENT_BRIEF.get(348), Some("霜后橘熟，分甘各半"));
        // 段十（v1.16.34 单3 扩池 2 条，TXT_NAME10_BASE=2647/TXT_BRIEF10_BASE=
        // 2649，手写名/简连号段先于正文收编——单1 精度/臂表批已用 N2601..2646
        // 故段十顺延）：len9 漂移唯一内容级防线——首尾逐字钉，len9 写
        // N_EVENTS−EV_SEG8 静默吞段十时在此必红（EV_SEG9=349 快照兜底）
        assert_eq!(EVENT_NAMES.get(349), Some("结绳记事"));
        assert_eq!(EVENT_NAMES.get(350), Some("云隙借光"));
        assert_eq!(EVENT_BRIEF.get(349), Some("年深忘事，结绳相记"));
        assert_eq!(EVENT_BRIEF.get(350), Some("云隙漏光，墨迹先干"));
        // 段十一（v1.16.35 单3 扩池 2 条，TXT_NAME11_BASE=2657/TXT_BRIEF11_BASE=
        // 2659，手写名/简连号段先于正文收编——单2 加载链批未用正文 N 故段十一
        // 顺延）：len10 漂移唯一内容级防线——首尾逐字钉，len10 写 N_EVENTS−
        // EV_SEG9 静默吞段十一时在此必红（EV_SEG10=351 快照兜底）
        assert_eq!(EVENT_NAMES.get(351), Some("共拓崖铭"));
        assert_eq!(EVENT_NAMES.get(352), Some("苔阶履痕"));
        assert_eq!(EVENT_BRIEF.get(351), Some("旧铭磨浅，同拓成幅"));
        assert_eq!(EVENT_BRIEF.get(352), Some("苔上履痕，与旧年重叠"));
        // 段十二（v1.16.36 单3 扩池 3 条，TXT_NAME12_BASE=2667/TXT_BRIEF12_BASE=
        // 2670，手写名/简连号段先于正文收编——单1 快照预建+单2 零改动未用正文 N
        // 故段十二顺延）：len11 漂移唯一内容级防线——首尾逐字钉，len11 写
        // N_EVENTS−EV_SEG10 静默吞段十二时在此必红（EV_SEG11=353 快照兜底）
        assert_eq!(EVENT_NAMES.get(353), Some("岭上补松"));
        assert_eq!(EVENT_NAMES.get(355), Some("磨镜还光"));
        assert_eq!(EVENT_BRIEF.get(353), Some("点松入土，百年成凉"));
        assert_eq!(EVENT_BRIEF.get(355), Some("旧镜磨还光，照见白头"));
        // 段十三（v1.16.37 单2 扩池 3 条，TXT_NAME13_BASE=2684/TXT_BRIEF13_BASE=
        // 2687，手写名/简连号段先于正文收编——v1.16.36 正文收编至 N2683 故段十三
        // 顺延）：len12 漂移唯一内容级防线——首尾逐字钉，len12 写 N_EVENTS−
        // EV_SEG11 静默吞段十三时在此必红（EV_SEG12=356 快照兜底）
        assert_eq!(EVENT_NAMES.get(356), Some("哑缶一声"));
        assert_eq!(EVENT_NAMES.get(358), Some("灶膛煨薯"));
        assert_eq!(EVENT_BRIEF.get(356), Some("废缶自鸣，问声来者"));
        assert_eq!(EVENT_BRIEF.get(358), Some("灰里煨薯，分两个暖"));
        // 段十四（v1.16.38 单2 扩池 3 条，TXT_NAME14_BASE=2701/TXT_BRIEF14_BASE=
        // 2704，手写名/简连号段先于正文收编——v1.16.37 正文收编至 N2700 故段十四
        // 顺延）：len13 漂移唯一内容级防线——首尾逐字钉，len13 写 N_EVENTS−
        // EV_SEG12 静默吞段十四时在此必红（EV_SEG13=359 快照兜底）
        assert_eq!(EVENT_NAMES.get(359), Some("移竹当窗"));
        assert_eq!(EVENT_NAMES.get(361), Some("熏笼共暖"));
        assert_eq!(EVENT_BRIEF.get(359), Some("竹影当窗，绿上窗纸"));
        assert_eq!(EVENT_BRIEF.get(361), Some("熏香一笼，暖到天明"));
        // 越界 None（旧 [&str; N].get 同款语义；扩池后界=365——v1.16.43 注改准，
        // 实际越界钉随段十六块下移）
        // 段十五（v1.16.40 单3 扩池 3 条，TXT_NAME15_BASE=2746/TXT_BRIEF15_BASE=
        // 2749，手写名/简连号段先于正文收编——v1.16.39 臂表收编至 N2726、
        // v1.16.40 长老委托文案 N2727..2745 故段十五顺延）：len14 漂移唯一内容
        // 级防线——首尾逐字钉，len14 写 N_EVENTS−EV_SEG13 静默吞段十五时在此
        // 必红（EV_SEG14=362 快照兜底）
        assert_eq!(EVENT_NAMES.get(362), Some("芦絮暖枕"));
        assert_eq!(EVENT_NAMES.get(364), Some("山摇地动"));
        assert_eq!(EVENT_BRIEF.get(362), Some("芦絮新枕，枕着秋声"));
        assert_eq!(EVENT_BRIEF.get(364), Some("山摇地动，梁尘落案"));
        // 段十六（v1.16.43 单3 扩池 3 条，TXT_NAME16_BASE=2761/TXT_BRIEF16_BASE=
        // 2764，手写名/简连号段先于正文收编——段十五正文收编至 N2760 故段十六
        // 顺延）：len15 漂移唯一内容级防线——首尾逐字钉，len15 写
        // N_EVENTS−EV_SEG14 静默吞段十六时在此必红（EV_SEG15=365 快照兜底；
        // v1.16.43 len15 已改差兑现，本钉即改差内容级防线）
        assert_eq!(EVENT_NAMES.get(365), Some("院墙夜塌"));
        assert_eq!(EVENT_NAMES.get(367), Some("檐冰垂光"));
        assert_eq!(EVENT_BRIEF.get(365), Some("墙塌一声，惊起满院霜"));
        assert_eq!(EVENT_BRIEF.get(367), Some("檐冰初垂，晨光一线"));
        // 段十七（v1.16.44 扩池 3 条，TXT_NAME17_BASE=2777/TXT_BRIEF17_BASE=
        // 2780，手写名/简连号段先于正文收编——v1.16.43 正文收编至 N2775、
        // v1.16.44 施工单二委托称号可佩句占 N2776 故段十七顺延 2777/2780）：
        // len16 漂移唯一内容级防线——首尾逐字钉，len16 写 N_EVENTS−EV_SEG15
        // 静默吞段十七时在此必红（EV_SEG16=368 快照兜底；v1.16.44 len16 已改
        // 差兑现，本钉即改差内容级防线）
        assert_eq!(EVENT_NAMES.get(368), Some("暖阁晒背"));
        assert_eq!(EVENT_NAMES.get(370), Some("同制新历"));
        assert_eq!(EVENT_BRIEF.get(368), Some("暖阁晒背，轮流向阳"));
        assert_eq!(EVENT_BRIEF.get(370), Some("旧页撕去，新历同写"));
        // 段十八（v1.16.45 施工单三扩池 3 条，TXT_NAME18_BASE=2801/TXT_BRIEF18_BASE=
        // 2804，手写名/简连号段先于正文收编——施工单一/二占 N2794..2800（多道侣
        // 文案 7 行）故段十八顺延 2801/2804）：len17 漂移唯一内容级防线——首尾
        // 逐字钉，len17 写 N_EVENTS−EV_SEG16 静默吞段十八时在此必红（EV_SEG17=371
        // 快照兜底；v1.16.45 len17 已改差兑现，本钉即改差内容级防线）
        assert_eq!(EVENT_NAMES.get(371), Some("牒案翻旧"));
        assert_eq!(EVENT_NAMES.get(373), Some("残页对笔"));
        assert_eq!(EVENT_BRIEF.get(371), Some("旧牒重验，你的名字在列"));
        assert_eq!(EVENT_BRIEF.get(373), Some("故人批注隔页应答，笔谈一夜"));
        // 段十九（v1.16.46 段十九扩池 3 条，TXT_NAME19_BASE=2816/TXT_BRIEF19_BASE=
        // 2819，名/简手写连号段先于正文 N2822..——勘察二提名三卡）：len18 漂移
        // 唯一内容级防线——首尾逐字钉，len18 写 N_EVENTS−EV_SEG17 静默吞段十九
        // 时在此必红（EV_SEG18=374 快照兜底；v1.16.46 len18 已改差兑现，本钉即
        // 改差内容级防线）
        assert_eq!(EVENT_NAMES.get(374), Some("三纸同檐"));
        assert_eq!(EVENT_NAMES.get(376), Some("空阶一蝶"));
        assert_eq!(EVENT_BRIEF.get(374), Some("另缝一页，三纸同檐各誊各卷"));
        assert_eq!(EVENT_BRIEF.get(376), Some("白蝶停在空阶，一帚悬而未落"));
        // 段二十（v1.16.47 段二十扩池 3 条，TXT_NAME20_BASE=2832/TXT_BRIEF20_BASE=
        // 2835，名/简手写连号段先于检定副标 N2838 照 N2783 先例——勘察二提名三
        // 卡）：len19 漂移唯一内容级防线——首尾逐字钉，len19 写 N_EVENTS−EV_SEG18
        // 静默吞段二十时在此必红（EV_SEG19=377 快照兜底；v1.16.47 len19 已改差
        // 兑现，本钉即改差内容级防线）
        assert_eq!(EVENT_NAMES.get(377), Some("晒书使巡"));
        assert_eq!(EVENT_NAMES.get(379), Some("夜渡加价"));
        assert_eq!(EVENT_BRIEF.get(377), Some("晒书使巡人间，勘一卷来历"));
        assert_eq!(EVENT_BRIEF.get(379), Some("夜渡船工坐地起价，晾你对岸"));
        // 段二十一（v1.16.48 扩池 3 条，TXT_NAME21_BASE=2849/TXT_BRIEF21_BASE=2852）
        assert_eq!(EVENT_NAMES.get(380), Some("补行代笔"));
        assert_eq!(EVENT_NAMES.get(382), Some("退稿索酬"));
        assert_eq!(EVENT_BRIEF.get(380), Some("故人批注缺半行，提笔替他补全"));
        assert_eq!(EVENT_BRIEF.get(382), Some("上家退稿还要酬钱，不给就扣卷"));
        // 段二十二（v1.16.49 扩池 3 条，TXT_NAME22_BASE=2864/TXT_BRIEF22_BASE=2867）：
        // len21 漂移唯一内容级防线——首尾逐字钉，len21 写 N_EVENTS−EV_SEG20 静默
        // 吞段二十一时在此必红（EV_SEG21=383 快照兜底；v1.16.49 len21 已改差兑现，
        // 本钉即改差内容级防线）
        assert_eq!(EVENT_NAMES.get(383), Some("描名起稿"));
        assert_eq!(EVENT_NAMES.get(385), Some("废稿盗售"));
        assert_eq!(EVENT_BRIEF.get(383), Some("划名废稿描清，照名重誊全篇"));
        assert_eq!(EVENT_BRIEF.get(385), Some("书佣偷稿盗售，买主寻上门"));
        // 段二十三（v1.16.50 扩池 3 条，TXT_NAME23_BASE=2879/TXT_BRIEF23_BASE=2882）：
        // len22 漂移唯一内容级防线——首尾逐字钉，len22 写 N_EVENTS−EV_SEG21 静默
        // 吞段二十三时在此必红（v1.16.50 len22 已改差兑现，本钉即改差内容级防线；
        // 386 检定卡名段照 377 款同 base23 段首）
        assert_eq!(EVENT_NAMES.get(386), Some("收册人问"));
        assert_eq!(EVENT_NAMES.get(388), Some("讹捐封口"));
        assert_eq!(EVENT_BRIEF.get(386), Some("晒书使携收册人，合卷一问"));
        assert_eq!(EVENT_BRIEF.get(388), Some("假香客代捐吞钱，反咬讹你"));
        // 段二十四（v1.16.51 扩池 3 条，TXT_NAME24_BASE=2896/TXT_BRIEF24_BASE=2899）：
        // len23 漂移唯一内容级防线——首尾逐字钉，len23 写 N_EVENTS−EV_SEG22 静默
        // 吞段二十四时在此必红（v1.16.51 len23 已改差兑现，本钉即改差内容级防线）
        assert_eq!(EVENT_NAMES.get(389), Some("闲章一方"));
        assert_eq!(EVENT_NAMES.get(391), Some("押卷取赎"));
        assert_eq!(EVENT_BRIEF.get(389), Some("旧章印文漫灭，奏刀补刻成趣"));
        assert_eq!(EVENT_BRIEF.get(391), Some("当铺押卷索翻倍赎金，不赎拆卖"));
        // 段二十五（v1.16.52 扩池 3 条，TXT_NAME25_BASE=2911/TXT_BRIEF25_BASE=2914）：
        // len24 漂移唯一内容级防线——首尾逐字钉，len24 写 N_EVENTS−EV_SEG23 静默
        // 吞段二十五时在此必红（v1.16.52 len24 已改差兑现，本钉即改差内容级防线）
        assert_eq!(EVENT_NAMES.get(392), Some("废稿拟声"));
        assert_eq!(EVENT_NAMES.get(394), Some("假帖索润"));
        assert_eq!(EVENT_BRIEF.get(392), Some("坊市废稿夜半作响，如有人低诵"));
        assert_eq!(EVENT_BRIEF.get(394), Some("有人持假法帖上门，索拓帖润资三百"));
        // 段二十六（v1.16.53 扩池 3 条，TXT_NAME26_BASE=2926/TXT_BRIEF26_BASE=2929）：
        // len25 漂移唯一内容级防线——首尾逐字钉，len25 写 N_EVENTS−EV_SEG24 静默
        // 吞段二十六时在此必红（v1.16.53 len25 已改差兑现，本钉即改差内容级防线）
        assert_eq!(EVENT_NAMES.get(395), Some("朱点回访"));
        assert_eq!(EVENT_NAMES.get(397), Some("次纸充宣"));
        assert_eq!(EVENT_BRIEF.get(395), Some("晒书使来年回访，复查朱点卷"));
        assert_eq!(EVENT_BRIEF.get(397), Some("纸行以次纸充宣价，迫你按契收货"));
        // 越界 None（旧 [&str; N].get 同款语义；扩池后界=398）
        assert_eq!(EVENT_NAMES.get(398), None);
        assert_eq!(EVENT_BRIEF.get(398), None);
        assert_eq!(EVENT_NAMES.get(usize::MAX), None);
    }

    // ---------------- v1.16.29 单1 三刀等价性钉测（逐字节回归） ----------------

    /// 刀A 块外 evf 模板：与原 format! 的 "{}" Display 语义逐字节一致
    ///（原字面量取自 game.rs:2225/npc.rs:336/world.rs:114 收编站「原文」注释）
    #[test]
    fn evf_block_out_format_equivalence() {
        init_text();
        assert_eq!(
            evf(1493, &[&"某吏"]),
            "此外，血仇未销——某吏。它删过你的字、批过你的行；此吏不除，书不成书。"
        );
        assert_eq!(
            evf(1587, &[&"玄诚长老", &2]),
            "你备下一份厚礼赠予玄诚长老，交情更进一分。（好感+2）"
        );
        // v1.16.32 单2 金句B：N1588 行直改后断言同步
        assert_eq!(evf(1588, &[&"青云宗"]), "行囊在肩，此番抵达青云宗。");
        // v1.16.33 单2 金句A：N1595 行直改后断言同步（world.rs:424 采药句 −13B，
        // 原文注释已同步照 evf(1574) 漂移红线；{} 实参数量不变仍单实参）
        assert_eq!(evf(1595, &[&"3"]), "采得聚气丹、培元丹，修为+3");
        // v1.16.29 单2 故事润色改写钉（N1574 真相源行直改后唯一无简钉的文本
        // 改写——此处补齐；原文注释同步见 game.rs 寻仇胜句站）
        assert_eq!(
            evf(1574, &[&"血仇之敌"]),
            "血仇得报——血仇之敌授首，账清了。"
        );
    }

    // ---------------- v1.16.34 单1 刀α：精度模板通道等价性钉测 ----------------

    /// Fmt0/1/2 包装 Display 语义 == format! 同精规格（走同一 f64 Display
    /// Formatter 路径的逐字节回归）；边界值覆盖 0.5（round-half-even 进位界）、
    /// 0.25/0.999（截断位）、2.675/99.995（二进制不可精确表示的假进位点）
    #[test]
    fn evf_prec_format_equivalence() {
        for v in [0.0, 0.25, 0.4999, 0.5, 0.501, 0.999, 1.005, 2.675, 99.995, 12345.678] {
            assert_eq!(format!("{}", Fmt0(v)), format!("{:.0}", v), "Fmt0 @{}", v);
            assert_eq!(format!("{}", Fmt1(v)), format!("{:.1}", v), "Fmt1 @{}", v);
            assert_eq!(format!("{}", Fmt2(v)), format!("{:.2}", v), "Fmt2 @{}", v);
        }
        // f32→f64 cast 站语义：f32 值转 f64 精确无损，同精规格产物逐字节一致
        //（11 站 cast 的等价性依据；0.5f32 边界同样钉死）
        let f: f32 = 0.5;
        assert_eq!(format!("{}", Fmt0(f as f64)), format!("{:.0}", f));
        assert_eq!(format!("{}", Fmt0((62.5f32 * 100.0) as f64)), format!("{:.0}", 62.5f32 * 100.0));
    }

    /// 精度收编站逐站文本钉（blob 模板纯 {} + 实参包装 → 回拼 == 原 format! 输出）；
    /// 样例覆盖三种精度档 + 纯{}/精度混排 + 多重共享行（2609 三站共享）
    #[test]
    fn evf_prec_station_equivalence() {
        init_text();
        // N2601 game.rs:2896 蛰伏句（{:.0}）
        assert_eq!(
            evf(2601, &[&(Fmt0(12.34))]),
            format!("你蛰伏蓄势，气机渐盈（破境率+{:.0}%）。", 12.34f64)
        );
        // N2609 坊市丹价（{:.2}，charpanel783/npc59/world1128 三重共享行）
        assert_eq!(
            evf(2609, &[&(Fmt2(0.8))]),
            format!("当前：坊市丹价 ×{:.2}", 0.8f64)
        );
        // N2615 修为行（{}/{}/{:.0} 混排，draw.rs:1241）
        assert_eq!(
            evf(2615, &[&"123", &"456", &(Fmt0(55.55))]),
            format!("修为 {}/{}（{:.0}%）", "123", "456", 55.55f64)
        );
        // N2604 境之门句（{}/{:.0}/{:.0} 双精度混排，game.rs:3286）
        assert_eq!(
            evf(2604, &[&"立骨", &(Fmt0(62.5)), &(Fmt0(3.7))]),
            format!(
                "你修为圆满，{}境之门隐现眼前。冲击成功率 {:.0}%，失败有 {:.0}% 身死之危。是孤注一掷，还是蛰伏蓄势？",
                "立骨", 62.5f64, 3.7f64
            )
        );
        // N2619 结缘起步句（{}/{:.0}/{} 三实参混排，charpanel.rs:970/1409 共享行）
        assert_eq!(
            evf(2619, &[&"12", &(Fmt0(50.0)), &"20"]),
            format!("结缘起步情谊 {}（双修 +{:.0}%），道侣赠礼事件再 +{}。", "12", 50.0f64, "20")
        );
        // N2616 修炼×战力（{:.1} 双精度，draw.rs:2629）
        assert_eq!(
            evf(2616, &[&(Fmt1(1.25)), &(Fmt1(0.9))]),
            format!("修炼×{:.1}  战力×{:.1}", 1.25f64, 0.9f64)
        );
    }

    // ---------------- v1.16.34 单1 β+γ 刀：match 臂簇收编等价性钉测 ----------------

    /// 四簇 N 表行内容与原臂字面量逐字节等价（N 错行/兜底漂移在此必红；全表
    /// 逐 slot 静态验收 tools/arms_sweep_check.py，git HEAD 原臂对照）
    #[test]
    fn evtx_arms_tables_equivalence() {
        init_text();
        // 簇1 sim_demon 破境场景（game.rs:3276 表，target-11 槽映射，_ 臂=槽3）
        assert_eq!(evtx(2621), "砚池深处无风自皱，墨色里现出一段石阶，阶上墨痕犹润");
        assert_eq!(evtx(2622), "案头秃笔无风自立，笔锋所指书页哗哗自翻，停在空白一页");
        assert_eq!(evtx(2623), "书山最高处那卷无字之书自己摊开，纸页里透出暖光");
        assert_eq!(evtx(2624), "笔冢之上虚悬一支巨笔，笔尖垂落至你面前，墨滴欲坠");
        // 簇2 退稿收束语三档（game.rs settle_coda，min(14)/5 槽映射）
        assert_eq!(evtx(2625), "白纸尚厚，慢慢写。");
        assert_eq!(evtx(2626), "这一稿有筋骨，下一稿更稳。");
        assert_eq!(evtx(2627), "一砚之尊，也要退稿。");
        // 簇3 终战副标（draw.rs:1801 表，i 直槽，_ 臂=槽3）
        assert_eq!(evtx(2628), "永生换的 · 一字不改");
        assert_eq!(evtx(2629), "推荐 · 后事不详，比永生更宽");
        assert_eq!(evtx(2630), "描清十万稿的名字");
        assert_eq!(evtx(2631), "隐藏 · 未完的书自会看见");
        // 簇4 境界名 15 臂（data.rs realm_title，0..13 直槽 + _ 臂=槽14）
        assert_eq!(evtx(2632), "未开笔");
        assert_eq!(evtx(2643), "一砚之尊");
        assert_eq!(evtx(2646), "琅函天笔");
    }

    /// realm_title 查表口运行期等价：旧 match 语义（0..13 直映射、≥14 与脏值
    /// 落 _ 臂「琅函天笔」）逐档回归——.min(len-1) 钳制唯一防线
    #[test]
    fn realm_title_table_equivalence() {
        assert_eq!(realm_title(0), "未开笔");
        assert_eq!(realm_title(3), "章成洇然");
        assert_eq!(realm_title(10), "墨行纸背");
        assert_eq!(realm_title(13), "执卷者");
        assert_eq!(realm_title(14), "琅函天笔");
        assert_eq!(realm_title(99), "琅函天笔");
        assert_eq!(realm_title(usize::MAX), "琅函天笔");
    }

    /// 刀B 表通道扩容：字段级 `_n`/`(ns,len)` 连号与整表基址查表，行内容与旧
    /// const 表字面量逐字节等价（N 错行在此必红；样例覆盖全部六张新表）
    #[test]
    fn tables2_textpack_equivalence() {
        init_text();
        // CHAPTERS：intro/open/reward_note/couplet 单值连号 + lore (ns,len) 对
        assert_eq!(
            CHAPTERS[0].intro(),
            "雨夜乱葬岗，你从尸堆里爬出来，手里攥着一枚方印——印身无字，只有层层叠叠被划掉的痕迹。你试着数那些划痕，数到第七层就乱了：每一层底下都压着另一层，像一册写了十万遍仍没写对的书。识海里有个累坏了的少年嗓音：你的名字是错的，天写错了。它不肯说来历，只把一笔旧账推到你面前——先推演一生，看看错在哪里。"
        );
        assert_eq!(CHAPTERS[0].open(), "方印无字，你有十万稿可写。");
        assert_eq!(CHAPTERS[0].reward_note(), "章成之礼：道韵 +50");
        assert_eq!(CHAPTERS[0].couplet(), "雨夜拾残印，灯前写错名");
        assert_eq!(
            CHAPTERS[0].lore(0),
            "其一：方印无字，划痕却比字多。识海里那声叹息说——这不是印，是账；前头十万稿，都记在它上头。"
        );
        assert_eq!(
            CHAPTERS[0].lore(1),
            "其二：你试着把印按进土里，土里渗出金纹，拼出半句话：先推演。推演不误砍柴工。"
        );
        assert_eq!(CHAPTERS[0].lore_len(), 2);
        assert_eq!(CHAPTERS[7].lore_len(), 3); // 第八章三段（变长 lore 唯一行）
        assert_eq!(CHAPTERS[0].lore(99), ""); // 越界空串兜底（evtx 同款防御）
        // ENDINGS：四字段全收编（表在 game.rs，全限定）
        assert_eq!(crate::game::ENDINGS[0].name(), "定稿");
        assert_eq!(crate::game::ENDINGS[0].alias(), "认写");
        assert_eq!(
            crate::game::ENDINGS[0].body(),
            "你赢了封卷吏。执笔人出现，递来笔：「字不错。入正文吧。从此你的每一行都是天定——天定，也就永不动删。」\n你看了很久那支笔。\n「我要是入了正文，」你问，「还能改自己吗？」\n「一字不改。永生换的。」"
        );
        assert_eq!(
            crate::game::ENDINGS[1].gain(),
            "【隐藏成就：后事不详 —— 一行之外，谁也写不了你】"
        );
        // NPC_TALES → npc_tale(npc, seg)（TXT_TALES_BASE 行主序 npc*3+seg）
        assert_eq!(
            npc_tale(0, 0),
            "他掌外门三千杂役的名册，也掌着每一页被天曹划掉的废稿。有人说他记性差——其实他只是装作没看见。看得太多，就得写进册子里。"
        );
        assert_eq!(
            npc_tale(3, 2),
            "大醉那夜他忽然清醒：「我原是天曹的笔吏，写坏了一个名字，被贬下界做衬托。」顿了顿又道：「你这卷书写下去，别管我。」言毕复醉。"
        );
        // HELP_LINES → help_line(idx)（TXT_HELP_BASE 17 行连号）
        assert_eq!(
            crate::draw::help_line(0),
            "【轮回镜】耗 1 枚模拟点数，推演「可能的你」的一生，直至死亡。"
        );
        // HINTS.text（TXT_HINT_BASE 14 行连号）
        assert_eq!(crate::guide::HINTS[0].text(), "山穷水尽：观看幻境广告，得 2 枚模拟点");
        // LOCATIONS.desc（desc_n 字段级）
        assert_eq!(
            LOCATIONS[0].desc(),
            "南疆第一仙宗，山门巍峨，剑气冲霄。拜入宗门可得名师指点，修炼事半功倍；完成宗门任务更可积攒贡献，换取宗门珍藏。"
        );
        assert_eq!(
            LOCATIONS[2].desc(),
            "上古修士遗府，每逢霞光贯日便开启一线。秘境中机缘与凶险并存——深入远征，逐层探取宝藏，见好就收方为上策。"
        );
    }

    /// 刀C 块外 ≥40B 散件 evtx 化：查池值与原字面量逐字节等价
    ///（原字面量取自 game.rs/main.rs/ui.rs 收编站「原文」注释）
    #[test]
    fn evtx_block_out_literal_equivalence() {
        init_text();
        assert_eq!(
            evtx(1726),
            "雨夜。乱葬岗。你从尸堆里爬出来，手里攥着一枚方印——印身无字，只有层层叠叠被划掉的痕迹。"
        );
        assert_eq!(evtx(1802), "等待点击（输入正常，不催促）");
        assert_eq!(evtx(1824), "字体解压失败（deflate 流损坏）");
        // v1.16.33 单1 刀2 延后批扩面：draw.rs/charpanel.rs 站样例（每文件 bare
        // 收编 ≥1 钉；原字面量取自收编站「原文」注释，与源码 grep 双源一致）
        assert_eq!(
            evtx(2523),
            "天道写你的书，写了十万年，写废了十万稿——这一稿，不肯退"
        );
        assert_eq!(evtx(2532), "—— 大事记 · 点一行留作朱批 ——");
        assert_eq!(evtx(2563), "交情愈深，其人故事自会显于纸上。");
        assert_eq!(evtx(2564), "情谊随赠礼事件与遗泽积累；更换道侣亦不忘旧谊。");
        // v1.16.33 单1 刀2 fmt→evf 通道钉：模板 Display 语义与 format! 等价
        //（占位替换产物逐字节等价；样例覆盖 draw.rs 单实参/多实参与 charpanel.rs
        // 异型混排三站）
        assert_eq!(
            evf(2524, &[&(3) as &dyn core::fmt::Display,
                        &(5) as &dyn core::fmt::Display,
                        &(2) as &dyn core::fmt::Display,
                        &("999") as &dyn core::fmt::Display]),
            "已历 3 世 · 最高境界 5境 · 斩杀宿敌 2 · 道韵 999"
        );
        assert_eq!(
            evf(2566, &[&(1200) as &dyn core::fmt::Display]),
            "耗灵石 1200 · 情谊+5 · 道心+1 气运+1 · 修为+2载"
        );
        assert_eq!(
            evf(2574, &[&(7) as &dyn core::fmt::Display,
                        &(2) as &dyn core::fmt::Display,
                        &("初沁墨") as &dyn core::fmt::Display,
                        &("88") as &dyn core::fmt::Display,
                        &("90") as &dyn core::fmt::Display]),
            "第7世·第2稿  初沁墨境  享年88载·历90年"
        );
        // v1.16.33 单1 刀1 副标表通道抽钉：SPECIAL_SUB 查表值 == 原臂字面量
        //（全表 77 slot 逐字节钉见 game.rs special_sub_table_full_sweep）
        assert_eq!(evtx(2368), "机缘");
        assert_eq!(evtx(2369), "探索上古洞府");
        assert_eq!(evtx(2433), "受檐马一问");
        // v1.16.37 单2 检定窗 76 启用（哑缶一声 356）：副标手写行 N2690
        assert_eq!(evtx(2690), "受缶一问");
    }
}
