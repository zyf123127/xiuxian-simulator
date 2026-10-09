// 核心逻辑：现实主角 + 模拟器系统双轨
// 现实：修炼/突破/坊市/寻仇；模拟：消耗点数推演一生，死后选继承带回现实
// 借用规范：对 life/sim 的可变借用限定在块内，日志/音效/存档在块外调用
use crate::data::*;
use crate::save;
use macroquad::prelude::*;
use macroquad::rand::gen_range;

pub const V_W: f32 = 1280.0;
pub const V_H: f32 = 720.0;
pub const SIM_SPEEDS: [f32; 4] = [0.0, 0.16, 0.06, 0.0]; // 每年间隔秒；0 数值档=瞬间跳到结局

#[derive(PartialEq, Clone, Copy)]
pub enum Scene {
    Menu,
    Reality,
    Draft,   // 模拟前：天赋 3 选 1
    Sim,     // 模拟推演中
    Inherit, // 模拟结束：选择继承
    Settle,  // 现实身故：总结算
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
            qi_pct: 0.0, stones: 0, stones_pct: 0.0, dao: 0.0, luck: 0.0, life_y: 0.0,
            fight: 0.0, relic: None, pill: None, dy: 0, injure: false, sect: false,
            lingmai: false, death: 0.0, special: 0, msg: Vec::new(), good: true,
        }
    }
}

pub struct EventCard {
    pub title: String,
    pub text: String,
    pub opts: Vec<(String, Eff)>,
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
    pub gained: f64,             // 模拟中累计修为（继承依据）
    pub power_buff: f64,         // 事件战力永久增益（+0.15 = +15%）
}

impl Default for Life {
    fn default() -> Self {
        Life {
            n: 1, age: 16.0, lifespan: 75.0, realm: 0, layer: 0, qi: 0.0,
            root: 30.0, dao: 50.0, luck: 40.0, body: 20.0, stones: 100, tech: 0,
            relics: Vec::new(), pills: [0; 6], sect: false, lingmai: false, injured: 0,
            talents: Vec::new(), washed: 0, max_realm: 0, fights: 0, wins: 0,
            seclusion: None, trib: None, new_high: false, natural_end: false,
            highlights: Vec::new(), gained: 0.0, power_buff: 0.0,
        }
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
        format!("{}·{}（{}{}）", self.title, self.name, REALMS[self.realm.min(14)].name, REALMS[self.realm.min(14)].layers[1.min(REALMS[self.realm.min(14)].layers.len() - 1)])
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
    pub neme: Option<Nemes>,
    pub music: bool,
    pub sfx: bool,
    pub life: Option<Life>,
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
            neme: None,
            music: true,
            sfx: true,
            life: None,
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
}

pub fn gain_qi(l: &mut Life, amount: f64) {
    l.qi += amount;
    l.gained += amount;
}

pub struct SettleInfo {
    pub cause: String,
    pub lines: Vec<(String, i64)>,
    pub total: i64,
}

pub struct Game {
    pub scene: Scene,
    pub meta: Meta,
    pub life: Option<Life>,     // 现实主角
    pub sim: Option<Life>,      // 模拟中的"你"
    pub sim_start_age: f64,
    pub sim_log: Vec<LogLine>,
    pub sim_result: Option<SimResult>,
    pub inherit_opts: Vec<usize>, // 继承选项（INHERITS 索引）
    pub inherit_picked: u32,      // 已选造化数
    pub draft: Vec<usize>,        // 模拟天赋候选
    pub picked: Vec<usize>,
    pub log: Vec<LogLine>,
    pub modal: Option<EventCard>,
    pub panel: u8, // 0无 1坊市 2行囊 3轮回烙印 4帮助
    pub tab: u8,
    pub shop_tab: u8,
    pub speed: usize, // 模拟速度 0暂停 1慢 2快 3跳过
    pub acc: f32,
    pub motes: Vec<Mote>,
    pub meteors: Vec<Meteor>,
    pub sparks: Vec<Spark>,
    pub shake: f32,
    pub toast: Option<(String, f32, Color)>,
    pub banner: Option<(String, f32, Color)>,
    pub settle: Option<SettleInfo>,
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
        if q.fract() < 0.05 { format!("{:.0}万", q) } else { format!("{:.1}万", q) }
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
        format!("{}·{}", rd.name, rd.layers[l.layer.min(rd.layers.len() - 1)])
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
    for &r in &l.relics {
        c *= RELICS[r].cult;
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
            e.life_mult = if e.life_mult > 0.0 { e.life_mult * te.life_mult } else { te.life_mult };
        }
        e.stone += te.stone;
        if te.cult_mult > 0.0 {
            e.cult_mult = if e.cult_mult > 0.0 { e.cult_mult * te.cult_mult } else { te.cult_mult };
        }
        if te.power_mult > 0.0 {
            e.power_mult = if e.power_mult > 0.0 { e.power_mult * te.power_mult } else { te.power_mult };
        }
        e.break_add += te.break_add;
        e.trib_add += te.trib_add;
        if te.event_mult > 0.0 {
            e.event_mult = if e.event_mult > 0.0 { e.event_mult * te.event_mult } else { te.event_mult };
        }
        if te.pill_mult > 0.0 {
            e.pill_mult = if e.pill_mult > 0.0 { e.pill_mult * te.pill_mult } else { te.pill_mult };
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
    let mut r = 3.0 * root_mult(l) * TECHS[l.tech].mult * env_mult(l) * relic_cult(l)
        * (1.0 + meta.upg[7] as f64 * 0.12)
        * teff(l).cult_mult.max(1.0);
    if l.injured > 0 {
        r *= 0.5;
    }
    r
}

pub fn power(l: &Life) -> f64 {
    let rd = &REALMS[l.realm];
    let mut p = rd.power * (1.0 + l.layer as f64 * 0.25) * TECHS[l.tech].pow
        * (1.0 + l.body as f64 / 100.0)
        * teff(l).power_mult.max(1.0);
    for &r in &l.relics {
        p *= 1.0 + RELICS[r].pow;
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
            picked: Vec::new(),
            log: Vec::new(),
            modal: None,
            panel: 0,
            tab: 0,
            shop_tab: 0,
            speed: 2,
            acc: 0.0,
            motes,
            meteors: Vec::new(),
            sparks: Vec::new(),
            shake: 0.0,
            toast: None,
            banner: None,
            settle: None,
            time: 0.0,
            sfx_ok: None,
            audio,
        }
    }

    pub fn log(&mut self, text: impl Into<String>, color: Color) {
        self.log.push(LogLine { text: text.into(), color });
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

    pub fn show_banner(&mut self, text: impl Into<String>, c: Color) {
        self.banner = Some((text.into(), 2.4, c));
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
        self.log.clear();
        self.scene = Scene::Reality;
        let oi = gen_range(0, ORIGINS.len());
        let nm = self.meta.neme.as_ref().unwrap().full();
        self.log(format!("你转生于{}，{}。", ORIGINS[oi].name, ORIGINS[oi].desc), C_TEXT);
        self.log("十六岁这年，你于一间破败道观中醒来，识海中多出一座古朴的【轮回镜】。", C_GOLD);
        self.log("镜灵低语：\"以点数演一世，死而不亡；取其造化，归汝一身。\"", C_CYAN);
        self.log(format!("此外，你有一桩血仇未报——{}，此獠不除，道心难安。", nm), C_RED);
        self.log("【开始模拟】推演一生，【继承】带回归途。此身虽弱，万世可期。", C_TEXT);
        self.check_ach_all();
        save::save(&self.meta);
    }

    // ================= 模拟：开始 =================
    pub fn sim_cost(&self) -> i32 {
        1
    }

    pub fn start_draft(&mut self) {
        if self.life.is_none() || self.meta.sim_points < self.sim_cost() {
            return;
        }
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
        let mut s = self.life.as_ref().unwrap().clone();
        s.talents = self.picked.clone();
        s.highlights.clear();
        s.seclusion = None;
        s.trib = None;
        self.sim = Some(s);
        self.sim_start_age = self.sim.as_ref().unwrap().age;
        self.sim_log.clear();
        self.speed = 2;
        self.acc = 0.0;
        self.scene = Scene::Sim;
        let names: Vec<&str> = self.picked.iter().map(|&t| TALENTS[t].name).collect();
        self.slog(format!("轮回镜亮起，你意识沉入镜像……此世天赋：{}。", names.join("、")), C_GOLD);
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
            let x = if from_left { gen_range(0.0, V_W * 0.4) } else { gen_range(V_W * 0.6, V_W) };
            self.meteors.push(Meteor {
                x,
                y: gen_range(20.0, 200.0),
                vx: if from_left { gen_range(260.0, 420.0) } else { -gen_range(260.0, 420.0) },
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
                self.slog(format!("水到渠成，你踏入{}！气息更进一分。", REALMS[r].layers[ly]), C_GREEN);
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
            if target == 10 {
                // 渡劫：模拟中自动服丹硬闯
                let est = {
                    let s = self.sim.as_ref().unwrap();
                    let e = teff(s);
                    let mut b = e.trib_add + s.body as f32 / 800.0;
                    if s.pills[5] > 0 {
                        b += 0.10;
                    }
                    let base = (0.92 - 7.0 * 0.015) + s.dao / 500.0 + s.luck / 600.0;
                    (base + b).clamp(0.05, 0.97)
                };
                {
                    let s = self.sim.as_mut().unwrap();
                    s.qi -= need;
                    let e = teff(s);
                    let mut b = e.trib_add + s.body as f32 / 800.0;
                    if s.pills[5] > 0 {
                        s.pills[5] -= 1;
                        b += 0.10;
                    }
                    s.trib = Some(Trib { wave: 0, timer: 0.8, flash: 0.0, spare_used: false, bonus: b });
                }
                self.slog(format!("功行圆满，天劫轰然而至！九重雷劫（均波存活 {:.0}%）——", est * 100.0), C_GOLD);
                self.play(crate::sounds::Which::Thunder);
                self.audio.swap_bgm(true);
                return;
            }
            let p = {
                let s = self.sim.as_ref().unwrap();
                big_break_p(s, target)
            };
            let rd = &REALMS[target];
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
                    BR::Ok
                } else {
                    s.qi = (s.qi * 0.7).max(0.0);
                    s.injured = 6;
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
                        s.highlights.push(format!("{}岁，突破【{}境】", s.age as i32, rd.name));
                        f
                    };
                    self.slog(format!("轰！突破【{}境】！寿元大增，气冲霄汉！", rd.name), C_GOLD);
                    if first {
                        self.slog("【首登】此身首次踏入此境！", C_GOLD);
                        self.show_banner(format!("突破 · {}境", rd.name), C_GOLD);
                    }
                    self.burst(50, true);
                    self.play(crate::sounds::Which::Ascend);
                }
                BR::Fail => {
                    self.slog(format!("冲击{}境失败！真元反噬，身受重伤。", rd.name), C_RED);
                    self.play(crate::sounds::Which::BreakFail);
                }
                BR::Dead => {
                    self.sim_die(&format!("冲击{}境失败，走火入魔，经脉俱断而亡。", rd.name));
                }
            }
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
                self.slog(format!("你于荒野撞见{}！自知不敌，拼死遁走，身受重伤。", nfull), C_RED);
            } else {
                self.sim_die(&format!("你撞见了宿敌{}！对面一掌拍下，你连名字都来不及报。", nfull));
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
                    s.highlights.push(format!("{}岁，阵斩宿敌{}", s.age as i32, neme.name));
                }
                self.meta.neme_slain += 1;
                self.meta.dao_yun += reward_dy;
                self.meta.sim_points += 2;
                self.log(format!("【模拟中反杀】你循战意逆斩宿敌{}！现实中的你已悟得其招，宿敌授首。", neme.full()), C_GOLD);
                self.show_banner("宿敌授首！", C_GOLD);
                self.log(format!("得灵石 {}（现实），道韵+{}，模拟点数+2。", fmt_int(reward_stones), reward_dy), C_GOLD);
                self.spawn_next_neme();
                self.play(crate::sounds::Which::Ascend);
                self.burst(60, true);
            } else {
                self.sim_die(&format!("你于乱战中再遇宿敌{}！激战三十合，终究技逊一筹，被一剑枭首。", nfull));
            }
        }
    }

    fn spawn_next_neme(&mut self) {
        let lv = self.meta.neme_slain as usize;
        let title = NEME_TITLES[(lv).min(NEME_TITLES.len() - 1)];
        let realm = (3 + lv * 2).min(14);
        self.meta.neme = Some(Nemes { title, name: rand_name(), realm });
        let nm = self.meta.neme.as_ref().unwrap().full();
        self.log(format!("宿敌虽死，其背后之势涌动——新的强敌浮出水面：{}。", nm), C_RED);
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
        let mut card = EventCard { title: String::new(), text: String::new(), opts: Vec::new() };
        match id {
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
                card.text = "一位宗门执事看你有仙缘：\"可愿入我门下？\"（修炼环境+30%，每年俸禄）".into();
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
                    card.text = "蒙面商人：\"洗髓丹一枚，童叟无欺……\"你摸了摸储物袋，钱不够。".into();
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
                card.text = "云端降下一位白须仙人，含笑看你：\"小子，老夫可指你一条路，要哪般？\"".into();
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
                card.text = "山谷深处一眼温泉汩汩冒泡，灵气凝成白雾，你入泉浸泡三日，通体舒泰。".into();
                let mut e = Eff::new();
                e.life_y = 30.0;
                e.qi_pct = 0.10;
                e.msg = vec!["寿元+30 载，修为+10%".into()];
                card.opts.push(("入泉沐浴".into(), e));
            }
            22 => {
                card.title = "古剑冢".into();
                card.text = "荒原上插着十万柄锈剑，剑鸣如龙吟。中央一柄古剑兀自震颤，似在择主。".into();
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
                card.text = "一株遭天雷劈中的古木倒在路边，焦黑树干里雷光未散，正是炼丹好材料。".into();
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
                card.text = "一位乘鹤老者路过，随手抛下锦盒：\"与你有缘，收着吧。\"鹤影转瞬没入云中。".into();
                let rl = gen_range(0, RELICS.len());
                let mut e = Eff::new();
                e.relic = Some(rl);
                e.qi_pct = 0.30;
                e.msg = vec![format!("盒中是【{}】！修为+30%", RELICS[rl].name)];
                card.opts.push(("叩谢仙长".into(), e));
            }
            27 => {
                card.title = "天机阁卜卦".into();
                card.text = "天机阁的瞎眼卦师掐指一算：\"道友印堂有光，是福是祸，在一念之间。\"".into();
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
        let Some(card) = self.modal.take() else { return };
        let Some((_, eff)) = card.opts.into_iter().nth(i) else { return };
        self.apply_eff(&eff);
        save::save(&self.meta);
    }

    fn apply_eff(&mut self, e: &Eff) {
        match e.special {
            1 => {
                let won = self.sim_fight(e.fight, "洞府守护傀儡");
                if won {
                    enum L { Tech(usize), Relic(usize), Stones(i64) }
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
                        L::Tech(t) => self.slog(format!("你于石壁上悟得《{}》！", TECHS[t].name), C_GOLD),
                        L::Relic(rl) => self.slog(format!("得法宝【{}】！", RELICS[rl].name), C_GOLD),
                        L::Stones(g) => self.slog(format!("得灵石 {}", fmt_int(g)), C_CYAN),
                    }
                }
                return;
            }
            2 => {
                let won = self.sim_fight(e.fight, "护宝妖兽");
                if won {
                    enum W { Relic(usize), Qi }
                    let ss = self.stone_scale(self.sim.as_ref().unwrap().realm);
                    let r = {
                        let s = self.sim.as_mut().unwrap();
                        let roll = gen_range(0.0, 1.0);
                        if roll < 0.5 {
                            let rl = gen_range(0, RELICS.len());
                            if !s.relics.contains(&rl) {
                                s.relics.push(rl);
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
                        W::Relic(rl) => self.slog(format!("灵乳入腹，得【{}】！", RELICS[rl].name), C_GOLD),
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
                        s.highlights.push(format!("{}岁，古剑冢得玄天斩灵剑", s.age as i32));
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
                    self.slog(format!("商队安抵坊市！管事奉上酬金 {}，气运+2。", fmt_int(g)), C_GREEN);
                }
            }
            12 => {
                let s = self.sim.as_mut().unwrap();
                s.dao = (s.dao + 6.0).min(100.0);
                s.pills[2] += 1;
                self.slog("卦师赠破境丹一枚：此卦大吉。", C_GREEN);
            }
            7 => {
                let (dao, luck) = {
                    let s = self.sim.as_ref().unwrap();
                    (s.dao, s.luck)
                };
                let p = 0.5 + dao / 150.0 + luck / 500.0;
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
        enum F { Win(i64), Lose, Dead }
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
                self.slog(format!("你战退{}！缴获灵石 {}。", name, fmt_int(g)), C_GREEN);
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
                    s.highlights.push(format!("{}岁，渡九重雷劫，飞升成仙！", s.age as i32));
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
            (s.max_realm, s.age, (s.age - self.sim_start_age) as i64, s.gained, s.tech, relic, s.stones, s.pills, s.highlights.clone())
        };
        let dy = (MILESTONE[max_realm.min(14)] as f64 / 8.0).ceil() as i64;
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
        });
        self.sim = None;
        self.modal = None;
        // 生成 3（升级后 4）个继承选项
        let mut pool: Vec<usize> = (0..INHERITS.len()).collect();
        let mut opts = Vec::new();
        let n = 3 + self.meta.upg[4].min(1) as usize;
        for _ in 0..n {
            let k = gen_range(0, pool.len());
            opts.push(pool.remove(k));
        }
        self.inherit_opts = opts;
        self.inherit_picked = 0;
        self.scene = Scene::Inherit;
        self.slog(cause.to_string(), C_RED);
        self.play(crate::sounds::Which::Death);
        save::save(&self.meta);
    }

    // ================= 继承 =================
    pub fn inherit_max(&self) -> u32 {
        1 + self.meta.upg[4].min(1) as u32
    }

    pub fn take_inherit(&mut self, i: usize) {
        if self.inherit_picked >= self.inherit_max() {
            return;
        }
        let Some(idx) = self.inherit_opts.get(i).copied() else { return };
        let Some(sr) = self.sim_result.clone() else { return };
        let scale = self.stone_scale(self.life.as_ref().map(|l| l.realm).unwrap_or(0));
        let Some(life) = self.life.as_mut() else { return };
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
            _ => {}
        }
        self.inherit_opts.remove(i);
        self.inherit_picked += 1;
        self.log(msg, C_GOLD);
        self.play(crate::sounds::Which::Ascend);
        save::save(&self.meta);
    }

    pub fn finish_inherit(&mut self) {
        self.scene = Scene::Reality;
        self.sim_result = None;
        self.sim_log.clear();
        self.log("大梦初醒，轮回镜归于沉寂。镜中一生，造化归身。", C_TEXT);
        save::save(&self.meta);
    }

    // ================= 现实：行动 =================
    pub fn act_cultivate(&mut self) {
        if self.life.is_none() {
            return;
        }
        let rate = {
            let l = self.life.as_ref().unwrap();
            cult_rate(l, &self.meta)
        };
        let (died, msg) = {
            let l = self.life.as_mut().unwrap();
            l.age += 1.0;
            if l.injured > 0 {
                l.injured -= 1;
            }
            l.qi += rate * 4.0;
            let msg = format!("你闭关一载，修为精进。（+{}）", fmt_num(rate * 4.0));
            let died = l.age >= l.lifespan;
            (died, msg)
        };
        self.log(msg, C_CYAN);
        self.play(crate::sounds::Which::Gain);
        if died {
            self.real_die("寿元耗尽，你于榻上安然长逝。轮回镜微光一闪，一切归零——唯道韵永存。");
            return;
        }
        self.check_ach_all();
        save::save(&self.meta);
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
                self.log(format!("现实突破：你踏入{}，根基扎实无比。", REALMS[r].layers[ly]), C_GREEN);
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
            let p = {
                let l = self.life.as_ref().unwrap();
                big_break_p(l, target)
            };
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
                    l.lifespan = l.lifespan.max(rd.life * (1.0 + self.meta.upg[8] as f64 * 0.10));
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
                self.log(format!("现实冲击{}境失败！真元反噬受轻伤（现实突破有轮回镜护持，不会身死）。", rd.name), C_RED);
                self.play(crate::sounds::Which::BreakFail);
            }
        }
        save::save(&self.meta);
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
        self.log(format!("以{}灵石向轮回镜献祭，得 1 枚模拟点数。", fmt_int(cost)), C_CYAN);
        self.play(crate::sounds::Which::Coin);
        save::save(&self.meta);
    }

    pub fn exchange_cost(&self) -> i64 {
        let realm = self.life.as_ref().map(|l| l.realm).unwrap_or(0);
        (1500.0 * (1.0 + realm as f64).powf(1.3)) as i64
    }

    // 现实寻仇
    pub fn can_revenge(&self) -> bool {
        self.life.is_some() && self.meta.neme.is_some()
    }

    pub fn revenge_p(&self) -> f32 {
        if let (Some(l), Some(n)) = (&self.life, &self.meta.neme) {
            let my = power(l);
            let r = my / n.power().max(1.0);
            let p = (r * r / (1.0 + r * r)).clamp(0.05, 0.95);
            return p as f32;
        }
        0.0
    }

    pub fn act_revenge(&mut self) {
        if !self.can_revenge() {
            return;
        }
        let p = self.revenge_p();
        let neme = self.meta.neme.clone().unwrap();
        let win = gen_range(0.0, 1.0) < p;
        if win {
            let reward_stones = (4000.0 * self.stone_scale(neme.realm)) as i64;
            let reward_dy = 50 * (1 + neme.realm as i64);
            {
                let l = self.life.as_mut().unwrap();
                l.stones += reward_stones;
                l.wins += 1;
                l.highlights.push(format!("{}岁，现实斩杀宿敌{}", l.age as i32, neme.name));
            }
            self.meta.dao_yun += reward_dy;
            self.meta.sim_points += 2;
            self.meta.neme_slain += 1;
            self.log(format!("血仇得报！你于现实中阵斩宿敌{}！", neme.full()), C_GOLD);
            self.show_banner("血 仇 得 报", C_GOLD);
            self.log(format!("夺其家产：灵石 {}，道韵+{}，模拟点数+2。", fmt_int(reward_stones), reward_dy), C_GOLD);
            self.spawn_next_neme();
            self.play(crate::sounds::Which::Ascend);
            self.burst(80, true);
            self.check_ach_all();
        } else {
            let l = self.life.as_mut().unwrap();
            l.injured = 5;
            l.qi *= 0.7;
            self.log(format!("你寻仇{}，不敌而遁，身受重伤！修炼无岁月，且先变强。", neme.full()), C_RED);
            self.play(crate::sounds::Which::BreakFail);
        }
        save::save(&self.meta);
    }

    pub fn eat_pill(&mut self, id: usize) {
        let can = self.life.as_ref().map_or(false, |l| l.pills[id] > 0);
        if !can {
            return;
        }
        let boost = self.life.as_ref().map(|l| teff(l).pill_mult.max(1.0)).unwrap_or(1.0);
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
        save::save(&self.meta);
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
        self.log(format!("你于藏经阁重金求得《{}》，闭关参悟三日而通！", name), C_GOLD);
        self.play(crate::sounds::Which::Ascend);
        save::save(&self.meta);
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
        self.log(format!("购得{}一枚（{}灵石）。", name, fmt_int(price)), C_CYAN);
        self.play(crate::sounds::Which::Coin);
        save::save(&self.meta);
    }

    pub fn pill_price(&self, id: usize) -> i64 {
        let realm = self.life.as_ref().map(|l| l.realm).unwrap_or(0);
        let disc = self.life.as_ref().map_or(false, |l| teff(l).pill_mult >= 2.0);
        let d = if disc { 0.8 } else { 1.0 };
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
        save::save(&self.meta);
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
        self.log(format!("达成成就【{}】！道韵+{}", name, fmt_int(reward)), C_GOLD);
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
        let self_neme_slain = self.meta.neme_slain;
        let checks: [(usize, bool); 22] = [
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
            (l.max_realm, l.age, l.stones, l.wins, l.new_high, l.natural_end, power(l))
        };
        self.play(crate::sounds::Which::Death);
        self.shake = 18.0;
        let mut lines: Vec<(String, i64)> = Vec::new();
        let base = MILESTONE[max_realm.min(14)];
        if base > 0 {
            let b = if new_high { base * 2 } else { base };
            lines.push((format!("{}境道韵{}", REALMS[max_realm.min(14)].name, if new_high { "（首登×2）" } else { "" }), b));
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
        self.settle = Some(SettleInfo { cause: cause.to_string(), lines, total });
        self.scene = Scene::Settle;
        save::save(&self.meta);
    }

    pub fn burst(&mut self, n: usize, gold: bool) {
        let (cx, cy) = if self.life.is_some() { (180.0, 168.0) } else { (V_W * 0.5, V_H * 0.5) };
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


pub const NEME_TITLES: [&str; 6] = ["血手人屠", "血魔老祖", "幽冥鬼帝", "噬天魔尊", "灭世魔主", "太上魔祖"];

pub fn rand_name() -> String {
    const SURNAMES: [&str; 8] = ["厉", "萧", "墨", "鬼", "血", "殷", "聂", "皇甫"];
    const GIVEN: [&str; 10] = ["无咎", "天殇", "屠苍", "九幽", "绝尘", "问天", "灭", "寒渊", "荒", "玄煞"];
    format!("{}{}", SURNAMES[gen_range(0, SURNAMES.len())], GIVEN[gen_range(0, GIVEN.len())])
}

// 继承选项
pub struct InheritDef {
    pub name: &'static str,
    pub desc: &'static str,
}
pub const INHERITS: [InheritDef; 8] = [
    InheritDef { name: "继承修为", desc: "现实修为 +模拟一生所得的五成" },
    InheritDef { name: "继承道果", desc: "道心 +5，体魄 +5（永久）" },
    InheritDef { name: "继承功法", desc: "习得模拟中修成的功法" },
    InheritDef { name: "继承法宝", desc: "取回模拟中获得的法宝" },
    InheritDef { name: "继承财货", desc: "取回模拟中灵石的一半" },
    InheritDef { name: "继承丹药", desc: "模拟中丹药尽数带回" },
    InheritDef { name: "继承寿数", desc: "现实寿元 +25 载" },
    InheritDef { name: "继承气运", desc: "现实气运 +3（永久）" },
];
