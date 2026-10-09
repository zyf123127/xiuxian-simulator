// 行式文本存档（桌面端 std::fs；损坏字段静默回退默认值）
use crate::game::{Life, Meta, Nemes};

pub const PATH: &str = "xiuxian_save.txt";

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
    s.push_str(&format!("music={} sfx={}\n", m.music as i32, m.sfx as i32));
    if let Some(n) = &m.neme {
        s.push_str(&format!("neme={} {} {}\n", n.title, n.name, n.realm));
    }
    if let Some(l) = &m.life {
        s.push_str(&life_text(l));
    }
    let _ = std::fs::write(PATH, s);
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
    s
}

pub fn load() -> Meta {
    let mut m = Meta::default();
    let Ok(text) = std::fs::read_to_string(PATH) else {
        return m;
    };
    let mut in_life = false;
    let mut life = Life::default();
    let mut neme_title = String::new();
    let mut neme_name = String::new();
    let mut neme_realm = 0usize;
    let mut has_neme = false;
    for line in text.lines() {
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let nums = |s: &str| -> Vec<f64> {
            s.replace(['[', ']', '(', ')', ' '], "")
                .split(',')
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
    if in_life {
        m.life = Some(life);
    }
    m
}
