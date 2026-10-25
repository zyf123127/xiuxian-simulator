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
                13 => "立即渡劫",
                14 => "闭关备劫",
                15 => "立即破境",
                16 => "蛰伏蓄势",
                17 => "结为道侣",
                18 => "登擂夺魁",
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
    Npc,  // v1.8.0 D2 现实好感事件：choose_roption 抉择（rmodal_npc 配对）
    Path, // v1.10.2 筑基择道：choose_path 抉择（三途分流；副标由 draw_modal_card 特判）
}

pub struct EventCard {
    pub title: String,

    pub text: String,

    pub opts: Vec<(String, Eff)>,

    pub kind: CardKind,
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

    pub path: u8, // v1.10.2 筑基择道：道途 0未择/1剑修/2气修/3丹修（存档键 Lpath=，旧档缺键 0；选定终身不改，模拟侧恒 0）
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
        }
    }
}

// v1.10.2 筑基择道：道途常量与名号（三途分流，分别挂钩战力/周天吐纳/坊市丹药与灵田）
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

    pub music: bool,

    pub sfx: bool,

    pub life: Option<Life>,

    pub sect_contrib: u32, // 宗门贡献（跨世持久）

    pub worn_title: i32, // 佩戴称号：-1 自动跟随 / 0..=14 境界称号 / 15 弑神者（v1.5.0）

    pub dongfu: [i32; 3], // 洞府设施等级 0..=3：0聚灵阵 1灵田 2丹房（v1.5.0）

    pub map_seed: u32,   // v1.6.0 山河万象：地图伪随机种子（LCG 演化）
    pub map_region: u32, // v1.6.0 山河万象：当前地区索引（MAP_THEMES 下标，wrapping 递增）

    pub ad_watched: i32, // v1.7.0：累计观看幻境广告次数（也用于广告创意轮换）

    pub npc_fav: [i32; 4], // v1.6.0 D：NPC 好感（v1.8.0 扩四：0长老 1商人 2宁清霜 3醉道人；跨世持久，0..=100）

    pub npc_ev: [bool; 4], // v1.8.0 D2：NPC 好感专属事件已抉择标记（抉择即消耗，跨世持久）

    pub region_visited: [bool; 5], // v1.8.0 C5：五域到访记录（按 map_region%5 置位，跨世持久）

    pub auto: u8, // v1.9.25 周天吐纳：现实自动修炼开关 0关/1开（存档键 zt=，旧档缺键默认 0）

    pub last_ts: u64, // v1.10.2 离线闭关：最后一次落盘 epoch 秒（存档键 ts=；旧档缺键/0 → 首落盘不计收益仅落新基准）
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
}

pub fn gain_qi(l: &mut Life, amount: f64) {
    l.qi += amount;

    l.gained += amount;
}

// v1.10.2 离线闭关：出关结算卡数据（会话态不进存档；收益在 resume_life 时已入账内存）
pub struct OfflineReport {
    pub years: i64,        // 离闭载数
    pub qi: f64,           // 修为收益（五折后）
    pub stones: i64,       // 灵田岁入（静态口径）
    pub life_capped: bool, // 被寿元钳住（再不归返便要坐化）
}

// ================= v1.10.2 离线闭关：出关结算 =================
// 纯函数（可测）：离线收益三件套。口径：90 秒 = 1 载；raw = min(离线秒, 8h)/90；
// 效率 5 折；N = min(raw×0.5, lifespan − age − 1)——终值恒保 age<lifespan，绝不替玩家死亡。
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
    lifespan: f64,
    age: f64,
) -> Option<(i64, f64, i64, bool)> {
    if last_ts == 0 {
        return None; // 首次落盘前无基准
    }
    let sec = (now as f64 - last_ts as f64).max(0.0); // 时钟回拨/脏值 → 按 0 收益
    let raw = sec.min(OFFLINE_CAP_SECS) / OFFLINE_SEC_PER_YEAR;
    let n_raw = (raw * OFFLINE_EFF) as i64;
    let cap = ((lifespan - age - 1.0) as i64).max(0);
    let n = n_raw.min(cap);
    if n <= 0 {
        return None;
    }
    Some((
        n,
        gain_per_year * n as f64,
        (lingtian_per_year as f64 * n as f64).round() as i64,
        n_raw > cap, // 被寿元钳住
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

    pub panel: u8, // 0无 1坊市 2行囊 3轮回烙印 4帮助 5角色

    pub char_tab: u8, // 角色面板页签 0总览 1法宝 2成就·称号（v1.5.0）

    // v1.10.2 人物页单列滚动列表会话态（不进存档，×/Esc 归零；先例 char_page）：
    // npc_scroll 滚动偏移（钳 [0, 1198]）；npc_drag 拖动锚点 (按下x, 按下y, 按下时scroll)；
    // npc_vel 松手惯性 px/帧；npc_anim 页点/头像条跳转目标（lerp 0.2/帧平滑趋近）
    pub npc_scroll: f32,
    pub npc_drag: Option<(f32, f32, f32)>,
    pub npc_vel: f32,
    pub npc_anim: Option<f32>,

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
        "凡人之躯".to_string()
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
    }

    if l.injured > 0 {
        r *= 0.5;
    }

    // v1.10.2 筑基择道：气修（path=2）每载修为 ×1.25（乘区尾接）
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

    // v1.10.2 筑基择道：剑修（path=1）战力 ×1.35（乘区尾接；模拟侧 path 恒 0 不受益）
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

    // v1.10.2 筑基择道：气修（path=2）破境率 +0.06（加在 0.05..0.95 钳制之前）
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
        let meta = save::load();

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
        self.toast = Some((text.into(), 2.6, c));
    }

    pub fn play(&mut self, w: crate::sounds::Which) {
        self.sfx_ok = Some(w);
    }

    // v1.10.2：人物页滚动会话态归零（×/Esc 关面板同批归零，先例 char_page 归零口径）
    pub fn npc_scroll_reset(&mut self) {
        self.npc_scroll = 0.0;
        self.npc_drag = None;
        self.npc_vel = 0.0;
        self.npc_anim = None;
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

        let stones = 100 * (1i64 << upg[6].min(4) as i64);

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
            "十六岁这年，你于清源洞府侧的破败道观中醒来，识海中多出一座古朴的【轮回镜】。",
            C_GOLD,
        );

        self.log(
            "镜灵低语：\"以点数演一世，死而不亡；取其造化，归汝一身。\"",
            C_CYAN,
        );

        self.log(
            format!("此外，你有一桩血仇未报——{}，此獠不除，道心难安。", nm),
            C_RED,
        );

        self.log(
            "【开始模拟】推演一生，【继承】带回归途。此身虽弱，万世可期。",
            C_TEXT,
        );

        self.check_ach_all();

        // v1.8.0 C5：新世开局即处当前区域（新档为东境青云）——补记到访
        self.visit_region(true);

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
        // v1.10.2 离线闭关：恢复即结算（收益先入账内存，落盘交收功/去抖/pagehide）
        self.offline_settle();
        // v1.10.2 筑基择道：旧档 realm≥2 且未择道 → 补弹道途卡（offline_card 若在弹，
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
        let Some(l) = self.life.as_ref() else {
            return;
        };
        let now = date_now();
        let gain = self.cult_gain_per_year();
        let lv = self.meta.dongfu[1].clamp(0, 3) as usize;
        // v1.10.2 复审③：丹修（path=3）灵田岁入 ×1.6——与唯一推岁口 advance_time 的
        // 静态项同式，同一载在线/离线口径一致（漏乘则丹修离闭收益少 60%）
        let path_pill = if l.path == PATH_PILL { 1.6 } else { 1.0 };
        let lingtian = (LINGTIAN_YIELD[lv.min(3)] as f64
            * weather_loc_mult(l.weather as usize, l.loc)
            * path_pill)
        .round() as i64;
        let Some((n, qi, stones, capped)) = offline_settle_calc(
            self.meta.last_ts,
            now,
            gain,
            lingtian,
            l.lifespan,
            l.age,
        ) else {
            self.meta.last_ts = now; // 本次不计收益，仅落新基准
            return;
        };
        {
            let l = self.life.as_mut().unwrap();
            l.qi += qi; // 与 act_cultivate 完全同式累加；修为满不破境（不调破境口）
            l.stones += stones;
            l.age += n as f64;
        }
        self.meta.last_ts = now;
        if n >= 10 {
            self.offline_card = Some(OfflineReport {
                years: n,
                qi,
                stones,
                life_capped: capped,
            });
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

    // ================= v1.10.2 筑基择道：三途分流 =================

    // 弹道途卡（rmodal 独占模态；neme 卡/事件卡不可同帧抢输入——rmodal 空时才弹）。
    // 三钮副标由 draw_modal_card 的 CardKind::Path 分支统一为「择定终身，不可改道」。
    pub fn open_path_card(&mut self) {
        if self.rmodal.is_some() {
            return;
        }
        self.rmodal = Some(EventCard {
            kind: CardKind::Path,
            title: "筑基择道 · 三途分流".into(),
            text: "筑基之上道途分三，一经择定终身不改（模拟推演不携道途）：剑修战力×1.35；气修每载修为×1.25、破境率+6%；丹修服丹效果×1.35、灵田岁入×1.6。你将何以证道？".into(),
            opts: vec![
                ("剑修 · 战力 ×1.35".into(), Eff::new()),
                ("气修 · 修为 ×1.25 破境 +6%".into(), Eff::new()),
                ("丹修 · 服丹 ×1.35 灵田 ×1.6".into(), Eff::new()),
            ],
        });
        self.rmodal_npc = None; // 道途卡不归属 NPC
    }

    // 道途抉择：选定终身、无改道入口；未到筑基（realm<2）或已择者卫语句拒绝
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
            format!("筑基关口，你择定道途——{}。自此终身不改。", name),
            C_GOLD,
        );
        self.play(crate::sounds::Which::Ascend);
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

        // v1.10.2 筑基择道：道途不进模拟——推演是另一条人生，择道只属现实主角；
        // 不剥离会把五处乘区全部带进模拟侧、破坏推演对照（单测 sim_does_not_inherit_path）
        s.path = 0;

        self.sim = Some(s);

        self.sim_start_age = self.sim.as_ref().unwrap().age;

        self.sim_log.clear();

        self.speed = 2;

        self.acc = 0.0;

        self.scene = Scene::Sim;

        let names: Vec<&str> = self.picked.iter().map(|&t| TALENTS[t].name).collect();

        self.slog(
            format!(
                "轮回镜亮起，你意识沉入镜像……此世天赋：{}。",
                names.join("、")
            ),
            C_GOLD,
        );

        self.play(crate::sounds::Which::Ding);
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
            // 瞬间推演至死

            let mut guard = 0;

            while self.sim.is_some() && guard < 20000 {
                self.sim_year();

                guard += 1;

                if self.modal.is_some() {
                    self.choose_option(0);
                }

                if self.scene != Scene::Sim {
                    break;
                }
            }

            return;
        }

        if self.speed == 0 {
            return;
        }

        self.acc += dt;

        let mut guard = 0;

        while self.acc >= interval && guard < 60 {
            self.acc -= interval;

            self.sim_year();

            guard += 1;

            if self.sim.is_none() || self.modal.is_some() {
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
                gain_qi(s, rate * 3.5);

                s.seclusion = Some(*n);

                *n -= 1;

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

                    self.slog(format!("你寻一处幽谷闭关{}载，潜心苦修。", yrs), C_DIM);
                } else {
                    gain_qi(s, rate * 2.5);
                }
            }
        }

        if demon {
            self.sim_demon();

            return;
        }

        match act {
            1 => self.sim_event(),

            2 => {
                self.slog("出关，气息渊渟岳峙。", C_DIM);
            }

            3 => self.sim_break(),

            _ => {}
        }

        // 宿敌遭遇

        if self.sim.is_some() && self.meta.neme.is_some() && gen_range(0.0, 1.0) < 0.025 {
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

                self.sim_die("寿元耗尽，你于静室中含笑坐化，一场大梦归真。");

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

                card.opts.push(("立即渡劫！".into(), e1));

                let mut e2 = Eff::new();

                e2.special = 14;

                e2.msg = vec!["你转入闭关，默运玄功迎劫（每载存活+4%，三载后自动渡劫）".into()];

                card.opts.push(("闭关准备".into(), e2));
            } else {
                let p = {
                    let s = self.sim.as_ref().unwrap();

                    big_break_p(s, target)
                };

                card.title = format!("{}境之门", rd.name);

                card.text = format!(

                    "你修为圆满，{}境之门隐现眼前。冲击成功率 {:.0}%，失败有 {:.0}% 身死之危。是孤注一掷，还是蛰伏蓄势？",

                    rd.name,

                    p * 100.0,

                    rd.bdeath * 100.0

                );

                let mut e1 = Eff::new();

                e1.special = 15;

                e1.good = false;

                card.opts.push(("立即破境！".into(), e1));

                let mut e2 = Eff::new();

                e2.special = 16;

                e2.msg = vec!["你决定蛰伏蓄势（每载成功率+8%，至多+24%后自动破境）".into()];

                card.opts.push(("继续蓄势".into(), e2));
            }

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

            self.sim_die("寿元耗尽，你于静室中含笑坐化，一场大梦归真。");

            return;
        }

        if gen_range(0.0, 1.0) < 0.08 {
            let i = gen_range(0, MINOR_LINES.len());

            self.slog(MINOR_LINES[i], C_DIM);
        }
    }

    fn sim_neme(&mut self) {
        let (my, my_realm) = {
            let s = self.sim.as_ref().unwrap();

            (power(s), s.realm)
        };

        let (nfull, np, nrealm) = {
            let n = self.meta.neme.as_ref().unwrap();

            (n.full(), n.power(), n.realm)
        };

        if my_realm + 1 < nrealm || my < np * 0.5 {
            // 打不过：六成逃，四成死

            let escape = gen_range(0.0, 1.0) < 0.6;

            {
                let s = self.sim.as_mut().unwrap();

                if escape {
                    s.injured = 8;

                    s.stones = (s.stones as f64 * 0.8) as i64;
                }
            }

            if escape {
                self.slog(
                    format!("你于荒野撞见{}！自知不敌，拼死遁走，身受重伤。", nfull),
                    C_RED,
                );
            } else {
                self.sim_die(&format!(
                    "你撞见了宿敌{}！对面一掌拍下，你连名字都来不及报。",
                    nfull
                ));
            }
        } else {
            // 决战

            let r = my / np.max(1.0);

            let p = (r * r / (1.0 + r * r)).clamp(0.05, 0.95);

            if gen_range(0.0, 1.0) < p {
                // 反杀宿敌！

                let neme = self.meta.neme.take().unwrap();

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
                self.sim_die(&format!(
                    "你于乱战中再遇宿敌{}！激战三十合，终究技逊一筹，被一剑枭首。",
                    nfull
                ));
            }
        }
    }

    fn spawn_next_neme(&mut self) {
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
            format!("宿敌虽死，其背后之势涌动——新的强敌浮出水面：{}。", nm),
            C_RED,
        );

        // v1.6.0 山河万象：斩敌换幕（现实寻仇与模拟反杀共用此口）
        self.advance_map();
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
    }

    pub fn sim_event(&mut self) {
        let (luck, realm, sect, lingmai) = {
            let s = self.sim.as_ref().unwrap();

            let lk = s.luck;

            (lk, s.realm, s.sect, s.lingmai)
        };

        let mut total = 0.0f32;

        let mut ws: Vec<f32> = Vec::new();

        for &(id, w, mr, good) in EVENTS.iter() {
            let mut ww = w as f32;

            if (id == 6 && sect) || (id == 9 && lingmai) {
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

        if total <= 0.01 {
            return;
        }

        let mut roll = gen_range(0.0, total);

        let mut chosen = 0;

        for (i, w) in ws.iter().enumerate() {
            roll -= w;

            if roll <= 0.0 {
                chosen = i;

                break;
            }
        }

        self.build_event(EVENTS[chosen].0);
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
        };

        match id {
            28 => {
                card.title = "良缘天定".into();
                let cn = COMPANION_NAMES[gen_range(0, COMPANION_NAMES.len())];
                card.text = format!(
                    "同门师妹{}于月下递来一炉亲手所炼的凝神丹：\"道途漫漫，若不弃……愿与师兄结伴同行。\"",
                    cn
                );
                let mut e1 = Eff::new();
                e1.special = 17;
                e1.msg = vec![format!(
                    "你与{}结为道侣！修炼速度+20%，心魔有抚，或得赠礼。",
                    cn
                )];
                card.opts.push(("执子之手".into(), e1));
                let mut e2 = Eff::new();
                e2.dao = 5.0;
                e2.msg = vec!["你婉拒道：\"大道未成，何以家为。\"道心+5".into()];
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

            0 => {
                let g = (80.0 * ss * gen_range(0.7, 1.4)) as i64;

                card.title = "灵草满坡".into();

                card.text = "山坳里野生的灵草开得正盛，你采下满满一筐，换了一笔灵石。".into();

                let mut e = Eff::new();

                e.stones = g;

                e.msg = vec![format!("获得灵石 {}", fmt_int(g))];

                card.opts.push(("收下".into(), e));
            }

            1 => {
                card.title = "妖兽拦路".into();

                card.text = "一头妖兽自林中扑出，獠牙滴涎，拦住去路！".into();

                let mut e = Eff::new();

                e.fight = 0.85;

                e.good = false;

                card.opts.push(("拔剑而战".into(), e));
            }

            2 => {
                card.title = "上古洞府".into();

                card.text = "荒山之中你发现一座上古洞府，禁制残破，里面似有传承遗物。".into();

                let mut e1 = Eff::new();

                e1.fight = 1.15;

                e1.special = 1;

                e1.good = false;

                card.opts.push(("探索一番".into(), e1));

                card.opts.push(("敬而远之".into(), Eff::new()));
            }

            3 => {
                card.title = "坊市淘宝".into();

                if gen_range(0.0, 1.0) < 0.5 {
                    let g = (150.0 * ss * gen_range(0.7, 1.4)) as i64;

                    card.text = "你在坊市地摊上捡漏一个储物袋，打开一看——".into();

                    let mut e = Eff::new();

                    e.stones = g;

                    e.msg = vec![format!("袋中竟有灵石 {}", fmt_int(g))];

                    card.opts.push(("大喜收下".into(), e));
                } else {
                    card.text = "老掌柜拉住你：\"小哥，这瓶丹药便宜卖你。\"".into();

                    let mut e = Eff::new();

                    e.pill = Some((0, 2));

                    e.msg = vec!["获得聚气丹 ×2".into()];

                    card.opts.push(("购下".into(), e));
                }
            }

            4 => {
                card.title = "恶修劫道".into();

                card.text = "\"此路是我开！\"一名黑衣修士拦住你，眼中贪光闪烁。".into();

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

                card.text = "路遇一位云游道友，你们于山亭中对坐论道，各有所悟。".into();

                let mut e = Eff::new();

                e.dao = 5.0;

                e.qi_pct = 0.06;

                e.msg = vec!["道心+5，修为+6%".into()];

                card.opts.push(("尽兴而归".into(), e));
            }

            6 => {
                card.title = "宗门相邀".into();

                card.text =
                    "一位宗门执事看你有仙缘：\"可愿入我门下？\"（修炼环境+30%，每年俸禄）".into();

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

                card.opts.push(("悄然退去".into(), Eff::new()));
            }

            9 => {
                card.title = "灵脉洞府".into();

                card.text = "你寻得一处天然灵脉洞府，灵气浓郁如雾。（本世修炼环境+50%）".into();

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

                card.text = "你下山行走红尘，于市井烟火中磨砺道心。".into();

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

                let mut e = Eff::new();

                e.pill = Some((4, 1));

                e.stones = -800;

                e.msg = vec!["获得洗髓丹 ×1".into()];

                let can = self.sim.as_ref().map_or(false, |s| s.stones >= 800);

                if can {
                    card.text = "蒙面商人：\"洗髓丹一枚，800 灵石，过了这村没这店。\"".into();

                    card.opts.push(("买下".into(), e));
                } else {
                    card.text =
                        "蒙面商人：\"洗髓丹一枚，童叟无欺……\"你摸了摸储物袋，钱不够。".into();
                }

                card.opts.push(("摆手离开".into(), Eff::new()));
            }

            18 => {
                card.title = "前辈指点".into();

                card.text = "一位白眉老者随手点拨了你几句，你如醍醐灌顶。".into();

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

                card.opts.push(("转身就走".into(), Eff::new()));
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

                e2.stones = -500;

                e2.luck = 8.0;

                e2.msg = vec!["你留下香火钱。气运+8".into()];

                card.opts.push(("捐些香火".into(), e2));
            }

            _ => {}
        }

        if !card.opts.is_empty() {
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

        self.apply_eff(&eff);

        self.sync_save();
    }

    fn apply_eff(&mut self, e: &Eff) {
        match e.special {
            13 => {
                // 立即渡劫（玩家抉择）
                let (target, acc) = {
                    let s = self.sim.as_ref().unwrap();
                    (10, s.pending_break.as_ref().map(|(_, a)| *a).unwrap_or(0.0))
                };
                self.sim_do_break(target, acc);
            }
            14 => {
                // 闭关准备三载后自动渡劫
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
                // 结为道侣
                let cn = COMPANION_NAMES[gen_range(0, COMPANION_NAMES.len())];
                let first = {
                    let s = self.sim.as_mut().unwrap();
                    let first = s.companion.is_none();
                    s.companion = Some(cn.to_string());
                    // v1.6.0 D 应修②：好感只升不降——sim Life 由现实克隆（launch_sim）带入
                    // comp_fav，直接赋 25 会让遗泽等来源的高好感倒退（如 1.30 档→1.20）。
                    // 换伴侣保留好感：好感挂「玩家—NPC 体系」而非具体人名。
                    s.comp_fav = s.comp_fav.max(COMPANION_BOND_FAV);
                    s.highlights
                        .push(format!("{}岁，与{}结为道侣", s.age as i32, cn));
                    first
                };
                if first {
                    self.meta.companion_lives += 1;
                    self.check_ach(22);
                    if self.meta.companion_lives >= 3 {
                        self.check_ach(26);
                    }
                }
                self.slog(
                    format!(
                        "你与{}于月下结为道侣！修炼速度+{:.0}%，心魔有抚。",
                        cn,
                        (crate::npc::comp_cult_mult(COMPANION_BOND_FAV) - 1.0) * 100.0
                    ),
                    C_GOLD,
                );
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
                            .push(format!("{}岁，古剑冢得玄天斩灵剑", s.age as i32));

                        self.slog("古剑认主——【玄天斩灵剑】出鞘！十万剑齐鸣相送！", C_GOLD);
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

        let died = {
            let Some(s) = self.sim.as_mut() else { return };

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

            e.death > 0.0 && gen_range(0.0, 1.0) < e.death
        };

        for m in &e.msg {
            self.slog(m.clone(), if e.good { C_CYAN } else { C_GREEN });
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

                true
            }

            F::Lose => {
                self.slog(format!("不敌{}，身受重伤，仓皇遁走！", name), C_RED);

                self.play(crate::sounds::Which::BreakFail);

                false
            }

            F::Dead => {
                self.sim_die(&format!("你伤重不治，陨落于{}之爪。", name));

                false
            }
        }
    }

    // ================= 模拟：渡劫 =================

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
            self.sim_die("天威之下，你形神俱灭，化作漫天流萤。渡劫失败。");
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
            if let Some(s) = self.sim.as_mut() {
                s.highlights
                    .push(format!("{}与你相伴至最后一刻，含笑而逝。", cn));
            }
        }
        let comp_fav = self.sim.as_ref().map(|s| s.comp_fav).unwrap_or(0);

        self.meta.dao_yun += dy;

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
        });

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

        self.log("大梦初醒，轮回镜归于沉寂。镜中一生，造化归身。", C_TEXT);

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
            self.real_die("寿元耗尽，你于榻上安然长逝。轮回镜微光一闪，一切归零——唯道韵永存。");

            return 0.0;
        }

        // v1.5.0 四时：死亡当年不推时辰、不产灵田（推进点=现实推岁口，唯一）
        self.advance_time();

        self.check_ach_all();

        self.sync_save();

        rate
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

    // 佩戴称号：-1 自动跟随 title_of；15 弑神者；0..=14 锁定境界称号
    pub fn worn_title_of(&self) -> String {
        match self.meta.worn_title {
            -1 => self.title_of(),
            15 => "弑神者".to_string(),
            t => realm_title(t.clamp(0, 14) as usize).to_string(),
        }
    }

    // 佩戴称号（值域即 {-1, 0..=14, 15}）：越权/未获得拒绝并 toast
    pub fn set_worn_title(&mut self, t: i32) {
        let ok = match t {
            -1 => true,
            15 => self.meta.neme_slain >= 3,
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
            // v1.10.2 筑基择道：丹修（path=3）灵田岁入 ×1.6（静态乘区，随唯一推岁口）
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
                    // 现实渡劫：九重雷劫一次性结算（轮回镜护持，失败重伤不身死）
                    let mut report: Vec<String> = Vec::new();
                    let (pw, dead_at) = {
                        let l = self.life.as_ref().unwrap();
                        let e = teff(l);
                        let mut base = (0.92 - 4.0 * 0.015)
                            + l.dao / 500.0
                            + l.luck / 600.0
                            + l.body as f32 / 800.0;
                        let mut b = e.trib_add + if l.pills[5] > 0 { 0.12 } else { 0.0 };
                        if l.pills[5] > 0 {
                            report.push("护神丹化作雷纹护体（存活+12%）。".to_string());
                        }
                        if l.loc == NODE_PEAK {
                            base += PEAK_TRIB_BONUS;
                            report.push("天雷峰渡劫台：每重雷劫存活+2%。".to_string());
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
                    } else {
                        let w = dead_at.unwrap() + 1;
                        let l = self.life.as_mut().unwrap();
                        l.injured = 10;
                        l.qi = (l.qi * 0.4).max(0.0);
                        self.log(format!(
                        "渡劫在第{}重雷劫处功亏一篑！轮回镜金光护住你的元神——身受重伤，但无性命之忧。重整旗鼓，来日再战！",
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
            // v1.8.0 D1：大师姐「剑意点拨」——仅现实跨境破境加成（同境分支、渡劫分支不加；
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
            // v1.10.2 筑基择道：丹修（path=3）服丹效果 ×1.35（单点；模拟侧 teff 路径 path 恒 0）
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

        self.life = None;

        self.modal = None;

        self.panel = 0;

        self.settle = Some(SettleInfo {
            cause: cause.to_string(),

            lines,

            total,
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

pub const NEME_TITLES: [&str; 6] = [
    "血手人屠",
    "血魔老祖",
    "幽冥鬼帝",
    "噬天魔尊",
    "灭世魔主",
    "太上魔祖",
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

pub const INHERITS: [InheritDef; 8] = [
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
];

// ---------------- 寻仇必成修复回归：下限归 0 + 重伤/悬殊禁手 ----------------

#[cfg(test)]
mod revenge_tests {
    use super::*;

    // 免音频初始化与存档 IO 的最小 Game：仅寻仇路径所需字段
    fn game_with(l: Life, neme: Option<Nemes>) -> Game {
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

    fn neme(realm: usize) -> Nemes {
        Nemes {
            title: "魔修",
            name: "试炼者".to_string(),
            realm,
        }
    }

    // ① r→0：胜率归 0（原 clamp 下限 0.05 = 弱者恒 5% 保底，连点 N 次必成之根）
    #[test]
    fn underdog_rate_floors_at_zero() {
        // 凡人（战力 5.0×1.2）对道祖宿敌（1.5e10×1.25）：比率 ~3e-10，胜率应近 0 而非 5%
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

    // 离线收益纯函数：门槛/封顶/回拨/钳寿逐项口径
    #[test]
    fn offline_settle_calc_gates() {
        // 缺基准（ts=0）不结算
        assert!(offline_settle_calc(0, 1_000_000, 10.0, 100, 75.0, 20.0).is_none());
        // 时钟回拨按 0 收益
        assert!(offline_settle_calc(2_000_000, 1_000_000, 10.0, 100, 75.0, 20.0).is_none());
        // N=0（<3 分钟）不动：179s → raw=1.99 → N=0
        assert!(offline_settle_calc(1000, 1179, 10.0, 100, 75.0, 20.0).is_none());
        // 90s=1 载、五折：1800s → raw=20 → N=10；qi=10×10=100；stones=100×10=1000
        let (n, qi, st, capped) = offline_settle_calc(1000, 2800, 10.0, 100, 75.0, 20.0).unwrap();
        assert_eq!(n, 10);
        assert!((qi - 100.0).abs() < 1e-9, "qi={}", qi);
        assert_eq!(st, 1000);
        assert!(!capped);
        // 8h 封顶：一天挂机 → raw = min(86400,28800)/90 = 320 → N=160，
        // 但寿元余量 cap=(75−20−1)=54 更紧 → N=54 且 capped
        let (n, _, _, capped) =
            offline_settle_calc(1000, 1000 + 86_400, 10.0, 100, 75.0, 20.0).unwrap();
        assert_eq!(n, 54);
        assert!(capped);
        // 8h 封顶本径（寿元充裕）：lifespan=300 → cap=279 > 160 → N=160 且未钳
        let (n, _, _, capped) =
            offline_settle_calc(1000, 1000 + 86_400, 10.0, 100, 300.0, 20.0).unwrap();
        assert_eq!(n, 160);
        assert!(!capped);
        // 寿元钳制：age=73, lifespan=75 → cap = 1 → N=1 且 capped（绝不替玩家死亡）
        let (n, _, _, capped) =
            offline_settle_calc(1000, 1000 + 86_400, 10.0, 100, 75.0, 73.0).unwrap();
        assert_eq!(n, 1);
        assert!(capped);
    }

    // ================= v1.10.2 筑基择道 =================

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
}
