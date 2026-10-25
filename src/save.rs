// 行式文本存档（损坏字段静默回退默认值）
// v1.5.0 双端存档后端：桌面走 std::fs 文件；WASM（TapTap H5）走 localStorage——
// 由 index.html 注册的 miniquad 插件 xx_storage 提供 env 导入函数
// （xx_save_len / xx_save_read / xx_save_write，长度一律 UTF-8 字节数），
// 存档格式与解析逻辑两端完全一致（照 GetFish src/save.rs 已验证实现平移）。
use crate::data::{RELICS, RELIC_EQUIP_MAX};
use crate::game::{Life, Meta, Nemes};

// v1.5.0 wasm 后端不使用文件路径（键名由 JS 插件持有），此常量供桌面后端与文档引用
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub const PATH: &str = "xiuxian_save.txt";
// v1.5.0 H5 localStorage 键名（wasm 端键名由 JS 插件持有，此常量供文档引用）
#[allow(dead_code)]
pub const STORAGE_KEY: &str = "xiuxian_save";

// ---- 存档后端：读取全文 / 写入全文 ----
#[cfg(target_arch = "wasm32")]
mod backend {
    // 供 mq_js_bundle 插件版本核对（init_plugins 检查 wasm_exports.xx_storage() == version）
    #[unsafe(no_mangle)]
    pub extern "C" fn xx_storage() -> i32 {
        1
    }

    #[link(wasm_import_module = "env")]
    unsafe extern "C" {
        fn xx_save_len() -> i32;
        fn xx_save_read(ptr: *mut u8, len: i32);
        fn xx_save_write(ptr: *const u8, len: i32);
    }

    pub fn read_text() -> Option<String> {
        unsafe {
            let n = xx_save_len();
            if n < 0 {
                return None;
            }
            let mut buf = vec![0u8; n as usize];
            xx_save_read(buf.as_mut_ptr(), n);
            Some(String::from_utf8_lossy(&buf).into_owned())
        }
    }

    pub fn write_text(txt: &str) {
        unsafe { xx_save_write(txt.as_ptr(), txt.len() as i32) }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use super::PATH;

    pub fn read_text() -> Option<String> {
        std::fs::read_to_string(PATH).ok()
    }

    pub fn write_text(txt: &str) {
        let _ = std::fs::write(PATH, txt);
    }
}

// 存档文本生成（双端共用：桌面文件与 wasm localStorage 写入的是同一段文本，
// 差异仅在 backend::write_text 的存储介质）。抽出为纯函数以便单测做写读回环。
pub fn save_text(m: &Meta) -> String {
    let mut s = String::new();
    s.push_str(&format!("lives={}\n", m.lives));
    s.push_str(&format!("dy={}\n", m.dao_yun));
    s.push_str(&format!("pts={}\n", m.sim_points));
    s.push_str(&format!("upg={:?}\n", m.upg));
    s.push_str(&format!("ach={:?}\n", m.ach));
    s.push_str(&format!("fr={:?}\n", m.first_realm));
    s.push_str(&format!("best={}\n", m.best_realm));
    s.push_str(&format!("bage={}\n", m.best_age));
    s.push_str(&format!("bpow={}\n", m.best_power));
    s.push_str(&format!("wins={}\n", m.total_wins));
    s.push_str(&format!("slain={}\n", m.neme_slain));
    s.push_str(&format!("sims={}\n", m.total_sims));
    s.push_str(&format!("sc={}\n", m.sect_contrib));
    s.push_str(&format!("wtitle={}\n", m.worn_title));
    s.push_str(&format!("df={:?}\n", m.dongfu));
    // v1.6.0 山河万象：地图种子与地区（缺键回退默认 = 主题 0，旧档像素级等价）
    s.push_str(&format!("mseed={}\n", m.map_seed));
    s.push_str(&format!("mreg={}\n", m.map_region));
    // v1.8.0 C5：五域到访记录（bool → 0/1 整数数组；旧档缺键 → 按 mreg 迁移，见 parse）
    s.push_str(&format!("mvis={:?}\n", m.region_visited.map(|b| b as i32)));
    // v1.6.0 D：NPC 好感（跨世持久）
    s.push_str(&format!("nf={:?}\n", m.npc_fav));
    // v1.8.0 D2：好感专属事件已抉择标记（bool → 0/1；旧档缺键 → 全 false）
    s.push_str(&format!("nfe={:?}\n", m.npc_ev.map(|b| b as i32)));
    // v1.7.0：累计观看幻境广告次数（旧档缺键默认 0）
    s.push_str(&format!("adw={}\n", m.ad_watched));
    // v1.9.25 周天吐纳：现实自动修炼开关（旧档缺键默认 0=关）
    s.push_str(&format!("zt={}\n", m.auto));
    s.push_str(&format!("music={} sfx={}\n", m.music as i32, m.sfx as i32));
    // v1.10.2 离线闭关：最后一次落盘 epoch 秒（缺键/0 → 首落盘不计收益仅落新基准；
    // 回拨/脏值 → 按 0 收益；离线起点=最后一次落盘时刻，保守口径）
    s.push_str(&format!("ts={}\n", m.last_ts));
    if let Some(n) = &m.neme {
        s.push_str(&format!("neme={} {} {}\n", n.title, n.name, n.realm));
    }
    if let Some(l) = &m.life {
        s.push_str(&life_text(l));
    }
    s
}

pub fn save(m: &Meta) {
    // v1.5.0 双端后端：桌面写文件 / WASM 写 localStorage（JS 侧 try/catch，配额异常不上抛）
    backend::write_text(&save_text(m));
}

// ---------------- v1.10.2 落盘去抖的 pagehide 兜底（仅 wasm）----------------
// sync_save 去抖后磁盘写延迟 ≤3s；手机 WebView 切后台被强杀时 pagehide 是唯一
// 落盘时机——JS 在 pagehide/visibilitychange(hidden) 经 wasm_exports 调下列两个
// 导出，把最近一次 sync_save 的存档快照写入 localStorage（键与 xx_storage 插件
// 一致），丢档窗口从「3s 内全部变更」压到「距最近一次 sync_save 的间隔」。
#[cfg(target_arch = "wasm32")]
static PENDING: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

#[cfg(target_arch = "wasm32")]
pub fn stash_pending(m: &Meta) {
    *PENDING.lock().unwrap() = Some(save_text(m));
}

/// JS 兜底读取：pending 快照字节长度（0 = 无待兜底快照）
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn xx_save_pending_len() -> i32 {
    PENDING.lock().unwrap().as_ref().map_or(0, |s| s.len() as i32)
}

/// JS 兜底读取：pending 快照字节指针（两次调用间 wasm 单线程串行，指针有效）
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn xx_save_pending_ptr() -> *const u8 {
    PENDING
        .lock()
        .unwrap()
        .as_ref()
        .map_or(std::ptr::null(), |s| s.as_ptr())
}

fn life_text(l: &Life) -> String {
    let mut s = String::new();
    s.push_str(&format!("Ln={}\n", l.n));
    s.push_str(&format!("Lage={}\n", l.age));
    s.push_str(&format!("Lspan={}\n", l.lifespan));
    s.push_str(&format!("Lrealm={} {}\n", l.realm, l.layer));
    s.push_str(&format!("Lqi={}\n", l.qi));
    s.push_str(&format!(
        "Lstat={} {} {} {} {}\n",
        l.root, l.dao, l.luck, l.body, l.stones
    ));
    s.push_str(&format!("Ltech={}\n", l.tech));
    s.push_str(&format!("Lrelics={:?}\n", l.relics));
    s.push_str(&format!("Lpills={:?}\n", l.pills));
    s.push_str(&format!(
        "Lflags={} {} {} {}\n",
        l.sect as i32, l.lingmai as i32, l.injured, l.washed
    ));
    s.push_str(&format!(
        "Ltrack={} {} {}\n",
        l.max_realm, l.wins, l.natural_end as i32
    ));
    s.push_str(&format!("Lbuf={}\n", l.power_buff));
    s.push_str(&format!("Lloc={}\n", l.loc));
    s.push_str(&format!("Lcfav={}\n", l.comp_fav)); // v1.6.0 D：道侣好感
    s.push_str(&format!("Lequip={:?}\n", l.equipped));
    // v1.8.0 E：法宝灵纹等级（旧档缺键 → 全 0）
    s.push_str(&format!("Lrlv={:?}\n", l.relic_lv));
    s.push_str(&format!("Lslot={}\n", l.slot));
    s.push_str(&format!("Lweather={}\n", l.weather));
    // v1.10.2 筑基择道：道途（0..=3；旧档缺键 0=未择，越界归 0，选定终身不改）
    s.push_str(&format!("Lpath={}\n", l.path));
    s
}

pub fn load() -> Meta {
    // v1.5.0 双端后端：桌面读文件 / WASM 读 localStorage（无档时 None → 全默认值开局）
    match backend::read_text() {
        Some(text) => parse_save(&text),
        None => Meta::default(),
    }
}

// 解析存档文本（抽出为纯函数以便单测迁移/清洗规则；load 负责读文件）
fn parse_save(text: &str) -> Meta {
    let mut m = Meta::default();
    let mut in_life = false;
    let mut life = Life::default();
    let mut has_lequip = false; // v1.5.0：区分「缺键」（旧档→迁移）与「显式卸光」
    let mut has_mvis = false; // v1.8.0 C5：区分 mvis 缺键（旧档→按 mreg 迁移）与显式记录
    let mut neme_title = String::new();
    let mut neme_name = String::new();
    let mut neme_realm = 0usize;
    let mut has_neme = false;
    for line in text.lines() {
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        // v1.4.0 修复：旧实现先删除空格再按逗号分割，导致空格分隔的多值行
        // （Lrealm=9 3 / Lstat=a b c d e / Lflags / Ltrack）被并成一位大数（如 "9 3"→93）。
        // v1.3 中 life 块从不落盘故从未触发；v1.4 sync_save 起真实持久化 life，按分隔符切分修复。
        let nums = |s: &str| -> Vec<f64> {
            s.split([
                ',', ' ', '[', ']', '(', ')',
            ])
            .filter_map(|x| x.parse().ok())
            .collect()
        };
        match k {
            "lives" => m.lives = v.parse().unwrap_or(0),
            "dy" => m.dao_yun = v.parse().unwrap_or(0),
            "pts" => m.sim_points = v.parse().unwrap_or(3),
            "upg" => {
                let a = nums(v);
                for i in 0..10 {
                    m.upg[i] = a.get(i).copied().unwrap_or(0.0) as i32;
                }
            }
            "ach" => {
                let a = nums(v);
                let mut fixed = vec![false; crate::data::ACHS.len()];
                for (i, &x) in a.iter().enumerate() {
                    if i < fixed.len() {
                        fixed[i] = x > 0.5;
                    }
                }
                m.ach = fixed;
            }
            "fr" => {
                let a = nums(v);
                let mut fixed = vec![false; 15];
                for (i, &x) in a.iter().enumerate() {
                    if i < fixed.len() {
                        fixed[i] = x > 0.5;
                    }
                }
                m.first_realm = fixed;
            }
            "best" => m.best_realm = v.parse().unwrap_or(0),
            "bage" => m.best_age = v.parse().unwrap_or(0.0),
            "bpow" => m.best_power = v.parse().unwrap_or(0.0),
            "wins" => m.total_wins = v.parse().unwrap_or(0),
            "slain" => m.neme_slain = v.parse().unwrap_or(0),
            "sims" => m.total_sims = v.parse().unwrap_or(0),
            "sc" => m.sect_contrib = v.parse().unwrap_or(0),
            "wtitle" => m.worn_title = v.parse().unwrap_or(-1),
            "df" => {
                let a = nums(v);
                for i in 0..3 {
                    m.dongfu[i] = (a.get(i).copied().unwrap_or(0.0) as i32).clamp(0, 3);
                }
            }
            // v1.6.0 山河万象：地图种子/地区（旧档缺键 → 默认值）
            "mseed" => {
                m.map_seed = v.trim().parse::<u64>().unwrap_or(crate::data::MAP_SEED_DEFAULT as u64) as u32;
            }
            "mreg" => {
                m.map_region = v.trim().parse::<u64>().unwrap_or(0) as u32;
            }
            // v1.8.0 C5：五域到访（0/1 整数数组；越界位丢弃，缺键由尾块迁移）
            "mvis" => {
                let a = nums(v);
                for (i, &x) in a.iter().enumerate() {
                    if i < m.region_visited.len() {
                        m.region_visited[i] = x != 0.0;
                    }
                }
                has_mvis = true;
            }
            // v1.6.0 D：NPC 好感（缺键 → 全 0；越界 clamp 0..=100；v1.8.0 扩四短数组补零）
            "nf" => {
                let a = nums(v);
                for i in 0..4 {
                    m.npc_fav[i] = (a.get(i).copied().unwrap_or(0.0) as i32)
                        .clamp(0, crate::data::NPC_FAV_MAX);
                }
            }
            // v1.8.0 D2：好感事件已抉择（0/1 整数数组；缺键 → 全 false——置位在抉择时）
            "nfe" => {
                let a = nums(v);
                for (i, &x) in a.iter().enumerate() {
                    if i < m.npc_ev.len() {
                        m.npc_ev[i] = x != 0.0;
                    }
                }
            }
            // v1.7.0：广告次数（旧档缺键默认 0）
            "adw" => m.ad_watched = v.parse().unwrap_or(0),
            // v1.9.25 周天吐纳开关（旧档缺键默认 0=关；坏值静默回退）
            // v1.10.1 归一化到 0/1：脏档 zt=2..255 曾致「自动修炼暗中运转但 UI 显示关」
            "zt" => m.auto = (v.parse::<u8>() == Ok(1)) as u8,
            // v1.10.2 离线闭关：落盘基准时刻（缺键/0 → 首落盘不计收益仅落新基准）
            "ts" => m.last_ts = v.trim().parse().unwrap_or(0),
            "music" => m.music = v.trim().split(' ').next().map_or(true, |x| x == "1"),
            "sfx" => m.sfx = v.trim().split(' ').nth(1).map_or(true, |x| x == "1"),
            "neme" => {
                let mut it = v.trim().split(' ');
                neme_title = it.next().unwrap_or("").to_string();
                neme_name = it.next().unwrap_or("").to_string();
                neme_realm = it.next().and_then(|x| x.parse().ok()).unwrap_or(3);
                has_neme = !neme_title.is_empty();
            }
            "Ln" => {
                in_life = true;
                life = Life::default();
                life.n = v.parse().unwrap_or(1);
            }
            "Lage" => life.age = v.parse().unwrap_or(16.0),
            "Lspan" => life.lifespan = v.parse().unwrap_or(75.0),
            "Lrealm" => {
                let a = nums(v);
                life.realm = a.first().copied().unwrap_or(0.0) as usize;
                life.layer = a.get(1).copied().unwrap_or(0.0) as usize;
            }
            "Lqi" => life.qi = v.parse().unwrap_or(0.0),
            "Lstat" => {
                let a = nums(v);
                life.root = a.first().copied().unwrap_or(30.0) as f32;
                life.dao = a.get(1).copied().unwrap_or(50.0) as f32;
                life.luck = a.get(2).copied().unwrap_or(40.0) as f32;
                life.body = a.get(3).copied().unwrap_or(20.0) as f32;
                life.stones = a.get(4).copied().unwrap_or(0.0) as i64;
            }
            "Ltech" => life.tech = v.parse().unwrap_or(0),
            "Lrelics" => life.relics = nums(v).iter().map(|&x| x as usize).collect(),
            "Lpills" => {
                let a = nums(v);
                for i in 0..6 {
                    life.pills[i] = a.get(i).copied().unwrap_or(0.0) as i32;
                }
            }
            "Lbuf" => life.power_buff = v.parse().unwrap_or(0.0),
            "Lloc" => life.loc = v.parse().unwrap_or(crate::data::NODE_START),
            // v1.6.0 D：道侣好感（缺键 → 0；越界 clamp）
            "Lcfav" => {
                life.comp_fav = v
                    .trim()
                    .parse::<i64>()
                    .unwrap_or(0)
                    .clamp(0, crate::data::NPC_FAV_MAX as i64) as i32;
            }
            "Lequip" => {
                life.equipped = nums(v).iter().map(|&x| x as usize).collect();
                has_lequip = true;
            }
            // v1.8.0 E：法宝灵纹等级（缺键 → 全 0；越界 clamp 0..=REFINE_MAX）
            "Lrlv" => {
                let a = nums(v);
                for i in 0..8 {
                    life.relic_lv[i] = (a.get(i).copied().unwrap_or(0.0) as i32)
                        .clamp(0, crate::data::REFINE_MAX);
                }
            }
            "Lslot" => {
                life.slot = v.trim().parse::<i64>().unwrap_or(0).clamp(0, 3) as u8;
            }
            "Lweather" => {
                life.weather = v.trim().parse::<i64>().unwrap_or(0).clamp(0, 4) as u8;
            }
            "Lpath" => {
                // v1.10.2 筑基择道：值域 0..=3，越界/坏值归 0=未择（坏档/未来版本键容错）
                life.path = match v.trim().parse::<u8>() {
                    Ok(x) if x <= 3 => x,
                    _ => 0,
                };
            }
            "Lflags" => {
                let a = nums(v);
                life.sect = a.first().copied().unwrap_or(0.0) > 0.5;
                life.lingmai = a.get(1).copied().unwrap_or(0.0) > 0.5;
                life.injured = a.get(2).copied().unwrap_or(0.0) as u16;
                life.washed = a.get(3).copied().unwrap_or(0.0) as u32;
            }
            "Ltrack" => {
                let a = nums(v);
                life.max_realm = a.first().copied().unwrap_or(0.0) as usize;
                life.wins = a.get(1).copied().unwrap_or(0.0) as u32;
                life.natural_end = a.get(2).copied().unwrap_or(0.0) > 0.5;
            }
            _ => {}
        }
    }
    if has_neme {
        let title = crate::game::NEME_TITLES
            .iter()
            .find(|t| t.to_string() == neme_title)
            .copied()
            .unwrap_or(crate::game::NEME_TITLES[0]);
        m.neme = Some(Nemes {
            title,
            name: neme_name,
            realm: neme_realm,
        });
    }
    // v1.5.0：佩戴称号值域限定 {-1, 0..=14, 15}，越界一律回退自动跟随
    if !(-1..=15).contains(&m.worn_title) {
        m.worn_title = -1;
    }
    // v1.8.0 C5：mvis 缺键（v1.7.x 旧档）迁移——仅当前区域补记为已至，不补发奖励
    //（奖励发放由运行时 visit_region 兜底；索引取模与 region_mech 同口径）
    if !has_mvis {
        let t = (m.map_region as usize) % m.region_visited.len();
        m.region_visited[t] = true;
    }
    if in_life {
        // v1.5.0 迁移与清洗（顺序不可变）：先清 relics → 迁移 equipped → 再清 equipped
        fn dedup_keep_order(v: &mut Vec<usize>) {
            let mut seen = std::collections::HashSet::new();
            v.retain(|&x| seen.insert(x));
        }
        life.relics.retain(|&r| r < RELICS.len());
        dedup_keep_order(&mut life.relics);
        if !has_lequip {
            // 旧档缺 Lequip 键：默认装备包内前 3 件（与 v1.4 全量生效行为最接近）
            life.equipped = life
                .relics
                .iter()
                .take(RELIC_EQUIP_MAX)
                .copied()
                .collect();
        }
        life.equipped.retain(|&r| life.relics.contains(&r));
        dedup_keep_order(&mut life.equipped);
        life.equipped.truncate(RELIC_EQUIP_MAX);
        m.life = Some(life);
    }
    m
}

#[cfg(test)]
mod tests {
    // v1.5.0 迁移与清洗规则回归（纯 parse_save，不落盘）
    use super::*;

    const OLD_BASE: &str = "lives=3\ndy=10\nsc=5\nLn=3\nLage=20\nLspan=75\nLrealm=1 0\n\
Lstat=30 50 40 20 100\nLrelics=[0, 4, 2, 7, 1]\nLpills=[0, 0, 0, 0, 0, 0]\nLloc=3\n";

    #[test]
    fn old_save_migrates_equipped_first3() {
        let m = parse_save(OLD_BASE); // 缺 Lequip/Lslot/Lweather/wtitle/df
        let l = m.life.unwrap();
        assert_eq!(l.equipped, vec![0, 4, 2]); // 前 3 件保序迁移
        assert_eq!(l.slot, 0);
        assert_eq!(l.weather, 0);
        assert_eq!(m.worn_title, -1);
        assert_eq!(m.dongfu, [0, 0, 0]);
    }

    #[test]
    fn explicit_empty_equip_respected() {
        let m = parse_save(&format!("{}Lequip=[]\n", OLD_BASE)); // 显式卸光
        assert!(m.life.unwrap().equipped.is_empty());
    }

    #[test]
    fn out_of_range_keys_clamped() {
        let t = format!(
            "{}Lequip=[0, 4]\nLslot=9\nLweather=9\nwtitle=99\ndf=[9, -2, 1]\n",
            OLD_BASE
        );
        let m = parse_save(&t);
        let l = m.life.unwrap();
        assert_eq!(l.slot, 3);
        assert_eq!(l.weather, 4);
        assert_eq!(m.worn_title, -1);
        assert_eq!(m.dongfu, [3, 0, 1]);
        assert_eq!(l.equipped, vec![0, 4]); // ⊆ relics 且保序
    }

    #[test]
    fn relics_sanitized_and_title15_kept() {
        let t = format!("{}slain=3\nwtitle=15\nLrelics=[0, 0, 99, 3]\n", OLD_BASE);
        let m = parse_save(&t);
        let l = m.life.unwrap();
        assert_eq!(l.relics, vec![0, 3]); // 越界剔除+保序去重
        assert_eq!(l.equipped, vec![0, 3]); // 缺键迁移：≤3 全装
        assert_eq!(m.worn_title, 15); // 合法值域保留
    }

    // v1.6.0 山河万象：地图种子/地区迁移规则
    #[test]
    fn map_keys_default_when_missing() {
        let m = parse_save(OLD_BASE); // v1.5.0 旧档缺 mseed/mreg
        assert_eq!(m.map_seed, crate::data::MAP_SEED_DEFAULT);
        assert_eq!(m.map_region, 0); // 主题 0 = 现版地图，像素级等价
    }

    #[test]
    fn map_keys_parse_and_overflow_safe() {
        let t = format!("{}mseed=31415926\nmreg=7\n", OLD_BASE);
        let m = parse_save(&t);
        assert_eq!(m.map_seed, 31415926);
        assert_eq!(m.map_region, 7); // 任意 u32 合法（theme_of 内部取模兜底）
        let bad = format!("{}mseed=abc\nmreg=-1\n", OLD_BASE);
        let m2 = parse_save(&bad); // 坏值静默回退默认
        assert_eq!(m2.map_seed, crate::data::MAP_SEED_DEFAULT);
        assert_eq!(m2.map_region, 0);
    }

    // v1.6.0 D：好感键迁移规则（缺键全 0；越界 clamp 0..=100）；v1.8.0：扩四短数组补零
    #[test]
    fn npc_fav_defaults_and_clamp() {
        let m = parse_save(OLD_BASE); // 旧档缺 nf/Lcfav
        assert_eq!(m.npc_fav, [0, 0, 0, 0]);
        assert_eq!(m.life.unwrap().comp_fav, 0);
        let t = format!("{}nf=[120, -5]\nLcfav=99\n", OLD_BASE);
        let m2 = parse_save(&t);
        assert_eq!(m2.npc_fav, [100, 0, 0, 0]); // 上溢夹 100 / 负值夹 0 / 短数组补零
        assert_eq!(m2.life.unwrap().comp_fav, 99); // 合法值保留
    }

    // v1.8.0 C5：mvis 缺键迁移——仅当前区域（mreg%5）补记，不补发
    #[test]
    fn mvis_missing_derives_current_region() {
        let m = parse_save(OLD_BASE); // 缺 mreg → 主题 0
        assert_eq!(m.region_visited, [true, false, false, false, false]);
        let t = format!("{}mreg=7\n", OLD_BASE); // 7 % 5 = 2
        let m2 = parse_save(&t);
        assert_eq!(m2.region_visited, [false, false, true, false, false]);
    }

    // v1.8.0 E：Lrlv 缺键全 0；越界 clamp 0..=REFINE_MAX
    #[test]
    fn lrlv_missing_zero_and_clamp() {
        let m = parse_save(OLD_BASE);
        assert_eq!(m.life.unwrap().relic_lv, [0; 8]);
        let t = format!("{}Lrlv=[1, 2, 3, 9, -1]\n", OLD_BASE);
        let m2 = parse_save(&t);
        assert_eq!(m2.life.unwrap().relic_lv, [1, 2, 3, 3, 0, 0, 0, 0]);
    }

    // v1.6.0 H5 存档桥一致性：save_text → parse_save 写读回环。
    // 双端（桌面 std::fs / wasm localStorage 插件）写入的是同一段 save_text 文本、
    // 走同一个 parse_save 解析，介质差异不影响键与格式；此测试锁死新旧键的对称性。
    #[test]
    fn roundtrip_new_keys_save_text_parse() {
        let mut m = Meta::default();
        m.map_seed = 31415926;
        m.map_region = 7;
        m.npc_fav = [70, 25, 40, 10];
        m.npc_ev = [false, true, false, false];
        m.region_visited = [false, false, true, false, false];
        let mut l = Life::default();
        l.comp_fav = 35;
        l.relic_lv = [0, 2, 0, 0, 0, 0, 0, 3];
        m.life = Some(l);
        let text = save_text(&m);
        // 新键确实落盘（wasm localStorage 与桌面文件内容一致的前提）
        assert!(text.contains("mseed=31415926\n"));
        assert!(text.contains("mreg=7\n"));
        assert!(text.contains("nf=[70, 25, 40, 10]\n"));
        assert!(text.contains("nfe=[0, 1, 0, 0]\n"));
        assert!(text.contains("mvis=[0, 0, 1, 0, 0]\n"));
        assert!(text.contains("Lcfav=35\n"));
        assert!(text.contains("Lrlv=[0, 2, 0, 0, 0, 0, 0, 3]\n"));
        let m2 = parse_save(&text);
        assert_eq!(m2.map_seed, 31415926);
        assert_eq!(m2.map_region, 7);
        assert_eq!(m2.npc_fav, [70, 25, 40, 10]);
        assert_eq!(m2.npc_ev, [false, true, false, false]); // 逐位相等
        assert_eq!(m2.region_visited, [false, false, true, false, false]); // 逐位相等
        let l2 = m2.life.unwrap();
        assert_eq!(l2.comp_fav, 35);
        assert_eq!(l2.relic_lv, [0, 2, 0, 0, 0, 0, 0, 3]);
    }

    // v1.9.25 周天吐纳 zt= 键：旧档缺键默认 0=关（向后兼容）；写读回环；坏值静默回退
    #[test]
    fn zt_auto_missing_default_and_roundtrip() {
        let m = parse_save(OLD_BASE); // 旧档缺 zt
        assert_eq!(m.auto, 0);
        let mut m = Meta::default();
        m.auto = 1;
        let text = save_text(&m);
        assert!(text.contains("zt=1\n"));
        assert_eq!(parse_save(&text).auto, 1);
        let bad = format!("{}zt=abc\n", OLD_BASE);
        assert_eq!(parse_save(&bad).auto, 0);
        // v1.10.1：脏档 zt=2..255 归一化为 0（关），与 UI 门 meta.auto==1 单一口径
        let dirty = format!("{}zt=2\n", OLD_BASE);
        assert_eq!(parse_save(&dirty).auto, 0);
    }

    // v1.10.2 离线闭关：ts= 回环 / 旧档缺键 / 坏值
    #[test]
    fn offline_ts_roundtrip_missing_and_bad() {
        let mut m = Meta::default();
        m.last_ts = 1_700_000_000;
        let text = save_text(&m);
        assert!(text.contains("ts=1700000000\n"));
        assert_eq!(parse_save(&text).last_ts, 1_700_000_000);

        // 旧档缺 ts 键 → 0（首落盘不计收益仅落新基准）
        let old: String = text
            .lines()
            .filter(|l| !l.starts_with("ts="))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(parse_save(&old).last_ts, 0);

        // 坏值 → 0（时钟回拨按 0 收益由 offline_settle_calc 的 max(0.0) 兜底）
        let bad = format!("{}ts=abc\n", OLD_BASE);
        assert_eq!(parse_save(&bad).last_ts, 0);
    }

    // v1.10.2 筑基择道：Lpath= 回环 / 旧档缺键 / 越界归 0
    #[test]
    fn lpath_roundtrip_missing_and_out_of_range() {
        let mut m = Meta::default();
        let mut l = Life::default();
        l.path = 2;
        m.life = Some(l);
        let text = save_text(&m);
        assert!(text.contains("Lpath=2\n"));
        assert_eq!(parse_save(&text).life.as_ref().unwrap().path, 2);

        // 旧档缺 Lpath → 0 = 未择（向后兼容零迁移）
        let old: String = text
            .lines()
            .filter(|l| !l.starts_with("Lpath="))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(parse_save(&old).life.as_ref().unwrap().path, 0);

        // 越界归 0
        let bad: String = text
            .lines()
            .map(|l| {
                if l.starts_with("Lpath=") {
                    "Lpath=9".to_string()
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(parse_save(&bad).life.as_ref().unwrap().path, 0);
    }
}
