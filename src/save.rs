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

pub fn save(m: &Meta) {
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
    s.push_str(&format!("music={} sfx={}\n", m.music as i32, m.sfx as i32));
    if let Some(n) = &m.neme {
        s.push_str(&format!("neme={} {} {}\n", n.title, n.name, n.realm));
    }
    if let Some(l) = &m.life {
        s.push_str(&life_text(l));
    }
    // v1.5.0 双端后端：桌面写文件 / WASM 写 localStorage（JS 侧 try/catch，配额异常不上抛）
    backend::write_text(&s);
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
    s.push_str(&format!("Lequip={:?}\n", l.equipped));
    s.push_str(&format!("Lslot={}\n", l.slot));
    s.push_str(&format!("Lweather={}\n", l.weather));
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
            "Lequip" => {
                life.equipped = nums(v).iter().map(|&x| x as usize).collect();
                has_lequip = true;
            }
            "Lslot" => {
                life.slot = v.trim().parse::<i64>().unwrap_or(0).clamp(0, 3) as u8;
            }
            "Lweather" => {
                life.weather = v.trim().parse::<i64>().unwrap_or(0).clamp(0, 4) as u8;
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
}
