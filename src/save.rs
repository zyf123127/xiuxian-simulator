// 行式文本存档（损坏字段静默回退默认值）
// v1.5.0 双端存档后端：桌面走 std::fs 文件；WASM（TapTap H5）走 localStorage——
// 由 index.html 注册的 miniquad 插件 xx_storage 提供 env 导入函数
// （xx_save_len / xx_save_read / xx_save_write，长度一律 UTF-8 字节数），
// 存档格式与解析逻辑两端完全一致（照 GetFish src/save.rs 已验证实现平移）。
use crate::data::{RELICS, RELIC_EQUIP_MAX};
use crate::game::{Life, Meta, Nemes, SimRecord};

// v1.5.0 wasm 后端不使用文件路径（键名由 JS 插件持有），此常量供桌面后端与文档引用
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub const PATH: &str = "xiuxian_save.txt";
// v1.5.0 H5 localStorage 键名（wasm 端键名由 JS 插件持有，此常量供文档引用）
#[allow(dead_code)]
pub const STORAGE_KEY: &str = "xiuxian_save";

// v1.12.0 换皮迁移：旧档宿敌名号别名表——旧档 neme= 行以旧名号落盘（唯一直接以
// 名字字符串存档的文案），读档时按同下标映射到 game::NEME_TITLES 新名。
// 旧档迁移用，非展示文案；命中失败照旧走 find 失配兜底 NEME_TITLES[0]。
const OLD_NEME_TITLES: [&str; 6] = [
    "血手人屠", "血魔老祖", "幽冥鬼帝", "噬天魔尊", "灭世魔主", "太上魔祖",
];

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
    save_text_with_ts(m, m.last_ts)
}

/// v1.16.13 复审⑨：ts 可注入变体（save_text 唯一实际体）。pagehide 兜底快照
/// （stash_pending）传 date_now() 只把快照级 ts= 基准推进到当前时刻——内存
/// m.last_ts 仍由 save_flush 真落盘才推进（game.rs:1533 口径：离线起点=最后
/// 一次落盘时刻，快照注入不提前该基准）。零格式变化：键名/键序/解析不变，
/// 旧档不受影响。
pub fn save_text_with_ts(m: &Meta, ts: u64) -> String {
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
    // v1.16.13 复审⑨：取注入参（save_text 传 m.last_ts；stash_pending 传 date_now()）
    s.push_str(&format!("ts={}\n", ts));
    // v1.12.1 四结局达成位掩码（bit0=A定稿 bit1=B留白 bit2=C执笔 bit3=D合订；缺键/坏值→0）
    s.push_str(&format!("ed={}\n", m.endings_seen));
    // v1.12.1 讨牒 dp=（sync_taodie 对账补发，旧档首检补齐）与缝页 bn=/blv=（255 未定、缘 0..=3）
    s.push_str(&format!("dp={}\n", m.taodie));
    s.push_str(&format!("bn={}\n", m.feng_bn));
    s.push_str(&format!("blv={}\n", m.feng_blv));
    // v1.16.0 交游图鉴：十同页人历世结缘次数（旧档缺键 → [0;10] + 当前道侣位补 1，见 parse 尾）
    s.push_str(&format!("gbd={:?}\n", m.comp_bonds));
    // v1.16.8 章节书达成位掩码 ch=（bit i = CHAPTERS[i] 已了；缺键/坏值 → 0，载入帧
    // 按事实静默补齐零迁移）与系统初见闩 sy=（bit0=周天首开 bit1=兑换首通；缺键 → 0）。
    // 独立成行置于 h=/hq= 旧稿块之前——勿插 h=/hq= 之间（文件序约定：hq= 挂最近一条 h=）
    s.push_str(&format!("ch={}\n", m.chap_done));
    s.push_str(&format!("sy={}\n", m.sys_seen));
    // v1.16.10 事件回顾图鉴：EVENTS 全 id 遭遇计数 evc=（照 ach= 先例按表重定形，
    // 旧档缺键 → vec![0; EVENTS.len()] 零迁移；独立成行置于 h= 旧稿块之前——
    // 勿插 h=/hq= 之间，文件序约定同 ch=/sy=）
    s.push_str(&format!("evc={:?}\n", m.ev_cnt));
    // v1.16.5 旧稿档案：近 5 次模拟结算快照，每条一行 h=（前 8 段空格分隔照 Lstat=
    // 先例，cause 取第 9 段整段余量照 note= 自由文本先例——死因含空格不能全切；
    // 缺键 → 空表 = 默认值，零迁移）
    for r in &m.hist {
        s.push_str(&format!(
            "h={} {} {} {} {} {} {} {} {}\n",
            r.life_no,
            r.run_no,
            r.max_realm,
            r.age,
            r.years,
            r.dy,
            r.stones,
            r.comp,
            r.cause
        ));
        // v1.16.7 本世剧情摘要：additive 平行行，紧跟其 h=（解析按「hq 挂最近一条
        // h=」的文件序约定；缺 story 不写行——旧档旧行格式逐字节不变）
        if let Some(st) = &r.story {
            s.push_str(&format!("hq={}
", st));
        }
    }
    if let Some(n) = &m.neme {
        s.push_str(&format!("neme={} {} {}\n", n.title, n.name, n.realm));
    }
    // v1.12.0 留行批注：结算屏大事记点走的一行（下一稿开卷浮现后清空；缺键 → None 零迁移）
    if let Some(note) = &m.note_line {
        s.push_str(&format!("note={}\n", note));
    }
    if let Some(l) = &m.life {
        s.push_str(&life_text(l));
    }
    s
}

// v1.14.0 存档写频（卡顿修复 P1）：落盘前与上次写入的文本做全文比对，未变则跳过
// 本次 write_text——wasm 的 localStorage.setItem 是 WebView 主线程同步磁盘操作，
// 同秒内 sig 边界 flush 与去抖 flush 撞车时内容逐字节相同，跳过零损。
// 复审修正：原「长度+末 32 字节」弱指纹漏中段变化（ts=/nf= 在串中段，尾部是
// Lslot/Lweather 区）——gift_npc/购物等同长度中段变更会被误判同文跳写，而
// save_flush 已先清 save_dirty 无人补写，丢写窗口失去 10s 上界；改为全文相等后
// 跳写仅发生在「磁盘内容与此刻逐字节相同」时，零丢写，防抖收益不变（~1KB 比对）。
// 写失败不上抛（JS 侧 try/catch），故比对基准记在写入之后——极端组合「写失败且
// 后续同文」会持续跳过，但该失败本就丢同内容，风险面与 v1.13.0 持平。
static LAST_WRITTEN: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

pub fn save(m: &Meta) {
    let text = save_text(m);
    let mut last = LAST_WRITTEN.lock().unwrap();
    if last.as_deref() == Some(text.as_str()) {
        return;
    }
    // v1.5.0 双端后端：桌面写文件 / WASM 写 localStorage（JS 侧 try/catch，配额异常不上抛）
    backend::write_text(&text);
    *last = Some(text);
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
    // v1.16.13 复审⑨：快照 ts= 注入 date_now()（save_text_with_ts）——原 save_text
    // 取 m.last_ts（距最近真落盘一个 sync→flush 间隔的旧值），sync_save 后切后台
    // 强杀时离线收益起点回退；注入只影响快照文本，内存 m.last_ts 不动（game.rs:1533
    // 真落盘才推进的口径保持）。快照置入后由 save_flush 落盘帧清空（clear_pending），
    // 杜绝「已落盘 ts 更新、pagehide 仍用旧快照回写」的 ts 回退窗。
    *PENDING.lock().unwrap() = Some(save_text_with_ts(m, crate::game::date_now()));
}

/// v1.16.13 复审⑨：save_flush 真落盘后清兜底快照——磁盘内容已含新 ts=，pagehide
/// 无需再回写（index.html len<=0 天然跳过），消除旧快照把 ts 基准往回拨的窗口
#[cfg(target_arch = "wasm32")]
pub fn clear_pending() {
    *PENDING.lock().unwrap() = None;
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
    // v1.12.0 P0 修复：道侣落盘（此前 Life.companion 从未持久化，重启即 None）。
    // 以名字字符串存档，空串=None；历史版本从未写此键，旧档缺键 → None，零迁移。
    s.push_str(&format!(
        "Lcomp={}\n",
        l.companion.as_deref().unwrap_or("")
    ));
    s.push_str(&format!("Lequip={:?}\n", l.equipped));
    // v1.8.0 E：法宝灵纹等级（旧档缺键 → 全 0）
    s.push_str(&format!("Lrlv={:?}\n", l.relic_lv));
    s.push_str(&format!("Lslot={}\n", l.slot));
    s.push_str(&format!("Lweather={}\n", l.weather));
    // v1.10.2 成行择道：道途（0..=3；旧档缺键 0=未择，越界归 0，选定终身不改）
    s.push_str(&format!("Lpath={}\n", l.path));
    // v1.17.0 双修共进：上次双修的现实年岁（旧档缺键 0=从未双修零迁移；起始年岁
    // 16 起、恒 >0，0 无碰撞）
    s.push_str(&format!("Lsxy={}\n", l.sx_year));
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
    let mut has_gbd = false; // v1.16.0：区分 gbd 缺键（旧档→当前道侣位补 1）与显式记录
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
            // v1.12.1 终卷四结局：达成位掩码（缺键/坏值 → 0，照 zt=/ts= 先例零迁移）
            "ed" => m.endings_seen = v.trim().parse().unwrap_or(0),
            // v1.12.1 讨牒 dp=（缺键 0，脏值钳 6）/缝页 bn=/blv=（未知名 255 未定，缘钳 3）
            "dp" => m.taodie = v.trim().parse::<u8>().unwrap_or(0).min(6),
            "bn" => {
                m.feng_bn = match v.trim().parse::<u8>() {
                    Ok(x) if (x as usize) < crate::data::COMPANION_NAMES.len() => x,
                    _ => crate::data::FENG_UNSET,
                };
            }
            "blv" => m.feng_blv = v.trim().parse::<u8>().unwrap_or(0).min(3),
            // v1.16.0 交游图鉴：十同页人历世结缘次数（逐位 clamp 0..=255；短数组补零，
            // 照 nf=/nfe= 先例；越位值丢弃）
            "gbd" => {
                let a = nums(v);
                for i in 0..crate::data::COMPANION_NAMES.len() {
                    m.comp_bonds[i] = a.get(i).copied().unwrap_or(0.0).clamp(0.0, 255.0) as u8;
                }
                has_gbd = true;
            }
            // v1.16.8 章节书达成位掩码（缺键/坏值 → 0，照 ed= 先例零迁移；载入帧
            // Game::new 按事实静默补齐——位不会被洗掉只会补上）；系统初见闩（缺键 → 0）
            "ch" => m.chap_done = v.trim().parse().unwrap_or(0),
            "sy" => m.sys_seen = v.trim().parse().unwrap_or(0),
            // v1.16.10 事件遭遇计数（照 ach= 重定形先例 save.rs 上方：短数组补零、
            // 长数组截断、值 saturating 口径在 clamp 0..=255——u8 溢出回 0 会被
            // 「>0 即已收录」误判为未遇，故逐值 min(255) 顶格）
            "evc" => {
                let a = nums(v);
                let mut fixed = vec![0u8; crate::data::EVENTS.len()];
                for (i, &x) in a.iter().enumerate() {
                    if i < fixed.len() {
                        fixed[i] = (x.max(0.0) as u64).min(255) as u8;
                    }
                }
                m.ev_cnt = fixed;
            }
            // v1.16.5 旧稿档案：h=<life> <run> <realm> <age> <years> <dy> <stones> <comp> <cause…>
            // splitn(9,' ') 取余量（cause 含空格照 note= 先例不能全切；值内 '=' 安全——
            // split_once 只切首对）。数字坏值 unwrap_or 回退；realm clamp 0..=14、
            // comp 界外归 255（无伴）；段数不足整行丢弃（不臆造半条记录）
            "h" => {
                let p: Vec<&str> = v.trim().splitn(9, ' ').collect();
                if p.len() >= 9 {
                    let num = |i: usize| -> f64 {
                        p.get(i)
                            .and_then(|x| x.parse::<f64>().ok())
                            .unwrap_or(0.0)
                    };
                    let c = p.get(7).and_then(|x| x.parse::<u8>().ok()).unwrap_or(255);
                    m.hist.push(SimRecord {
                        life_no: num(0) as u32,
                        run_no: num(1) as u32,
                        max_realm: (num(2) as usize).min(14),
                        age: num(3),
                        years: num(4) as i64,
                        dy: num(5) as i64,
                        stones: num(6) as i64,
                        comp: if (c as usize) < crate::data::COMPANION_NAMES.len() {
                            c
                        } else {
                            255
                        },
                        cause: p[8].trim().to_string(),
                        story: None, // v1.16.7 hq= 平行行补挂（缺行 → None 零迁移）
                    });
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
            // v1.12.0 留行批注：note= 行（空串/缺键 → None；缺键回退照 zt= 先例）
            // v1.16.7 本世剧情摘要：hq= 平行行挂最近一条 h=（文件序约定：hq 紧跟
            // 其 h=）。hist 空（手改档错位）则整行丢弃——不影响 9 段主档；坏值/空串
            // 不置 story（保持 None 渲染跳过）
            "hq" => {
                let t = v.trim();
                if !t.is_empty() {
                    if let Some(r) = m.hist.last_mut() {
                        r.story = Some(t.to_string());
                    }
                }
            }
            "note" => {
                m.note_line = if v.trim().is_empty() {
                    None
                } else {
                    Some(v.trim().to_string())
                };
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
            // v1.12.0 P0 修复：道侣落盘回读（空串/缺键 → None；缺键回退照 Lrlv/zt 先例）
            "Lcomp" => {
                life.companion = if v.trim().is_empty() {
                    None
                } else {
                    Some(v.trim().to_string())
                };
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
                // v1.10.2 成行择道：值域 0..=3，越界/坏值归 0=未择（坏档/未来版本键容错）
                life.path = match v.trim().parse::<u8>() {
                    Ok(x) if x <= 3 => x,
                    _ => 0,
                };
            }
            // v1.17.0 双修共进：上次双修年岁（缺键 0=从未双修；坏值/负值静默回 0，
            // 照 Lpath 先例；u16 解析天然拒负——冷却闩只对比不索引，无越界面）
            "Lsxy" => life.sx_year = v.trim().parse::<u16>().unwrap_or(0),
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
        // v1.12.0 换皮迁移：旧表六名按同下标映射新名（OLD_NEME_TITLES，见文件头）；
        // 新名直读；未知名号走原 find 失配兜底 NEME_TITLES[0]（战力/寻仇/斩杀进度
        // 全按结构与 neme_slain 计数走，功能无损，仅称号显示）。
        let title = match OLD_NEME_TITLES
            .iter()
            .position(|t| *t == neme_title)
        {
            Some(i) => crate::game::NEME_TITLES[i],
            None => crate::game::NEME_TITLES
                .iter()
                .find(|t| t.to_string() == neme_title)
                .copied()
                .unwrap_or(crate::game::NEME_TITLES[0]),
        };
        m.neme = Some(Nemes {
            title,
            name: neme_name,
            realm: neme_realm,
        });
    }
    // v1.5.0：佩戴称号值域限定 {-1, 0..=14, 15}，越界一律回退自动跟随
    // （v1.16.10 扩 16=章节称号·雨夜执印者；v1.16.15 扩 17=执笔人——16/17 均
    // 保留位不占新键；v1.16.16 修复：值域漏抬 17 致四结局全收集的佩显状态读档
    // 即被钳回自动跟随。脏档降级读档越界值仍钳回 -1，无崩溃面）
    if !(-1..=17).contains(&m.worn_title) {
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
    // v1.16.0 交游图鉴：gbd 缺键（v1.15.x 旧档）迁移（照 mvis 先例 additive-only）——
    // 仅现实同页人可定位（companion 名字命中 COMPANION_NAMES）时该位记 1，保底
    // 『当前道侣不显示未识』；历史各伴次数不可考、不臆补。has_gbd==true 时零改动。
    if !has_gbd {
        let cur = m.life.as_ref().and_then(|l| {
            l.companion
                .as_deref()
                .and_then(|cn| crate::data::COMPANION_NAMES.iter().position(|&n| n == cn))
        });
        if let Some(idx) = cur {
            m.comp_bonds[idx] = 1;
        }
    }
    // v1.16.5 防御性截断：脏档塞了超量 h= 行也只留最近 5 条（hist[0]=最旧，
    // 尾部=最新；写端 sim_die 同步超量 remove(0)，此处仅防手改档）
    if m.hist.len() > crate::game::HIST_MAX {
        let extra = m.hist.len() - crate::game::HIST_MAX;
        m.hist.drain(0..extra);
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

    // ---------------- v1.16.0 交游图鉴：gbd= 键（回环/缺键/短数组/越位 clamp/迁移） ----------------

    // 回环：save_text→parse_save gbd 逐位相等（additive-only：其余键不受新键影响）
    #[test]
    fn gbd_roundtrip_bitwise() {
        let mut m = Meta::default();
        m.comp_bonds = [1, 0, 3, 255, 0, 7, 2, 0, 9, 4];
        let text = save_text(&m);
        let back = parse_save(&text);
        assert_eq!(back.comp_bonds, [1, 0, 3, 255, 0, 7, 2, 0, 9, 4]);
    }

    // 缺键默认 [0;10]（OLD_BASE 无道侣 → 无迁移）；越位值丢弃、逐位 clamp 0..=255
    #[test]
    fn gbd_missing_default_and_clamp() {
        let m = parse_save(OLD_BASE);
        assert_eq!(m.comp_bonds, [0; 10]);
        let t = format!("{}gbd=[300, -7, 3]\n", OLD_BASE);
        let m2 = parse_save(&t);
        assert_eq!(m2.comp_bonds, [255, 0, 3, 0, 0, 0, 0, 0, 0, 0]);
    }

    // 短数组补零（照 nf=/nfe= 先例）
    #[test]
    fn gbd_short_array_zero_filled() {
        let t = format!("{}gbd=[5]\n", OLD_BASE);
        let m = parse_save(&t);
        assert_eq!(m.comp_bonds[0], 5);
        assert_eq!(m.comp_bonds[1..], [0; 9]);
    }

    // 缺键迁移：旧档无 gbd + 现实同页人可定位（Lcomp=闻人未晚）→ comp_bonds[0]==1，
    // 其余位不臆补（照 mvis 迁移先例）；显式 gbd 键存在时零改动
    #[test]
    fn gbd_migration_current_companion() {
        let t = format!("{}Lcomp=闻人未晚\nLcfav=60\n", OLD_BASE);
        let m = parse_save(&t);
        assert_eq!(m.comp_bonds[0], 1);
        assert_eq!(m.comp_bonds[1..], [0; 9]);
        let t2 = format!(
            "{}Lcomp=闻人未晚\ngbd=[0, 2, 0, 0, 0, 0, 0, 0, 0, 0]\n",
            OLD_BASE
        );
        let m2 = parse_save(&t2); // has_gbd=true → 零迁移
        assert_eq!(m2.comp_bonds, [0, 2, 0, 0, 0, 0, 0, 0, 0, 0]);
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

    // v1.10.2 成行择道：Lpath= 回环 / 旧档缺键 / 越界归 0
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

    // v1.17.0 双修共进：Lsxy= 回环 / 旧档缺键 0（从未双修）/ 坏值 0（照 Lpath 先例）
    #[test]
    fn lsxy_roundtrip_missing_and_bad() {
        let mut m = Meta::default();
        let mut l = Life::default();
        l.sx_year = 42;
        m.life = Some(l);
        let text = save_text(&m);
        assert!(text.contains("Lsxy=42\n"));
        assert_eq!(parse_save(&text).life.as_ref().unwrap().sx_year, 42);

        // 旧档缺 Lsxy → 0 = 从未双修（向后兼容零迁移）
        let old: String = text
            .lines()
            .filter(|l| !l.starts_with("Lsxy="))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(parse_save(&old).life.as_ref().unwrap().sx_year, 0);

        // 坏值/负值 → 0（u16 解析天然拒负，冷却闩只对比不索引）
        let bad = format!("{}Lsxy=abc\n", OLD_BASE);
        assert_eq!(parse_save(&bad).life.as_ref().unwrap().sx_year, 0);
        let neg = format!("{}Lsxy=-3\n", OLD_BASE);
        assert_eq!(parse_save(&neg).life.as_ref().unwrap().sx_year, 0);
    }

    // v1.12.0 P0 修复：道侣落盘——Lcomp= 回环 / 空串 None / 旧档缺键 None
    #[test]
    fn lcomp_roundtrip_missing_and_empty() {
        let mut m = Meta::default();
        let mut l = Life::default();
        l.companion = Some("闻人未晚".to_string());
        l.comp_fav = 35;
        m.life = Some(l);
        let text = save_text(&m);
        assert!(text.contains("Lcomp=闻人未晚\n"));
        let l2 = parse_save(&text).life.unwrap();
        assert_eq!(l2.companion.as_deref(), Some("闻人未晚"));
        assert_eq!(l2.comp_fav, 35);

        // 空串 = None（显式无道侣态）
        let none_text = text.replace("Lcomp=闻人未晚\n", "Lcomp=\n");
        assert_eq!(parse_save(&none_text).life.unwrap().companion, None);

        // 旧档缺 Lcomp → None（历史版本从未落盘道侣，旧档行为不变）
        let old: String = text
            .lines()
            .filter(|l| !l.starts_with("Lcomp="))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(parse_save(&old).life.unwrap().companion, None);
    }

    // v1.12.0 换皮迁移：旧档宿敌名号按同下标映射新名（真档样例 neme=旧名号 萧绝尘 3）
    #[test]
    fn old_neme_title_maps_to_new() {
        let m = parse_save(&format!(
            "lives=1\nslain=0\nneme={} 萧绝尘 3\n",
            OLD_NEME_TITLES[0]
        ));
        let n = m.neme.unwrap();
        assert_eq!(n.title, crate::game::NEME_TITLES[0]);
        assert_eq!(n.name, "萧绝尘"); // 随机名串原样保留
        assert_eq!(n.realm, 3); // 档位（境界）不变

        // 六名一一对应：末档旧名 → 封卷
        let m1 = parse_save(&format!("neme={} 墨荒 14\n", OLD_NEME_TITLES[5]));
        let n1 = m1.neme.unwrap();
        assert_eq!(n1.title, "封卷");
        assert_eq!(n1.realm, 14);

        // 新名直读不再二次映射
        let m2 = parse_save("neme=撕页 厉无咎 5\n");
        let n2 = m2.neme.unwrap();
        assert_eq!(n2.title, "撕页");
        assert_eq!(n2.name, "厉无咎");
        assert_eq!(n2.realm, 5);
    }

    // v1.12.0 留行批注：note= 回环 / 空串 None / 旧档缺键 None（零迁移）
    #[test]
    fn note_line_roundtrip_missing_and_empty() {
        let mut m = Meta::default();
        m.note_line = Some("23岁，与洛书遥缝作同页".to_string());
        let text = save_text(&m);
        assert!(text.contains("note=23岁，与洛书遥缝作同页\n"));
        assert_eq!(
            parse_save(&text).note_line.as_deref(),
            Some("23岁，与洛书遥缝作同页")
        );

        // 空串 = None（解析容错；save_text 不会写出空值）
        let none_text = text.replace("note=23岁，与洛书遥缝作同页\n", "note=\n");
        assert_eq!(parse_save(&none_text).note_line, None);

        // 旧档缺 note 键 → None（历史版本从未落盘批注，向后兼容零迁移）
        let old: String = text
            .lines()
            .filter(|l| !l.starts_with("note="))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(parse_save(&old).note_line, None);
    }

    // v1.12.1 终卷四结局：ed= 回环 / 旧档缺键 0 / 坏值 0（零迁移，照 zt=/ts= 先例）
    #[test]
    fn endings_seen_roundtrip_missing_and_bad() {
        let mut m = Meta::default();
        m.endings_seen = 0b0111; // A/B/C 已达成（bit1=B《留白》，是 D《合订》的钥匙位）
        let text = save_text(&m);
        assert!(text.contains("ed=7\n"));
        assert_eq!(parse_save(&text).endings_seen, 0b0111);

        // 旧档缺 ed 键 → 0（历史版本从未落盘结局位，向后兼容零迁移）
        let old: String = text
            .lines()
            .filter(|l| !l.starts_with("ed="))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(parse_save(&old).endings_seen, 0);

        // 坏值 → 0
        let bad = format!("{}ed=abc\n", OLD_BASE);
        assert_eq!(parse_save(&bad).endings_seen, 0);
    }

    // v1.12.1 讨牒/缝页：dp=/bn=/blv= 回环 / 旧档缺键默认 / 坏值越界回退（零迁移）
    #[test]
    fn taodie_feng_roundtrip_missing_and_bad() {
        let mut m = Meta::default();
        m.taodie = 6;
        m.feng_bn = 3;
        m.feng_blv = 2;
        let text = save_text(&m);
        assert!(text.contains("dp=6\n") && text.contains("bn=3\n") && text.contains("blv=2\n"));
        let m2 = parse_save(&text);
        assert_eq!((m2.taodie, m2.feng_bn, m2.feng_blv), (6, 3, 2));

        // 旧档缺键 → dp 0 / bn 255 未定 / blv 0（向后兼容零迁移）；坏值/越界 → 回退钳制
        let old: String = text
            .lines()
            .filter(|l| !l.starts_with("dp=") && !l.starts_with("bn=") && !l.starts_with("blv="))
            .collect::<Vec<_>>()
            .join("\n");
        let m3 = parse_save(&old);
        assert_eq!((m3.taodie, m3.feng_bn, m3.feng_blv), (0, crate::data::FENG_UNSET, 0));
        let m4 = parse_save(&format!("{}dp=9\nbn=77\nblv=9\n", OLD_BASE));
        assert_eq!((m4.taodie, m4.feng_bn, m4.feng_blv), (6, crate::data::FENG_UNSET, 3));
    }

    // ---------------- v1.16.5 旧稿档案：h= 键（回环/缺键/坏值截断） ----------------

    // 回环：save_text→parse_save hist 逐条相等（含空格死因、负灵石、无伴 255）
    #[test]
    fn hist_roundtrip_bitwise() {
        let mut m = Meta::default();
        m.hist = vec![
            SimRecord {
                life_no: 3,
                run_no: 7,
                max_realm: 4,
                age: 76.5,
                years: 60,
                dy: 320,
                stones: -1200,
                comp: 255,
                cause: "九问皆妄，你于镜前枯坐而终。".into(),
                story: None,
            },
            SimRecord {
                life_no: 3,
                run_no: 8,
                max_realm: 5,
                age: 88.0,
                years: 72,
                dy: 500,
                stones: 42000,
                comp: 3,
                cause: "寿元自然耗尽".into(),
                story: None,
            },
        ];
        let text = save_text(&m);
        let back = parse_save(&text);
        assert_eq!(back.hist.len(), 2);
        assert_eq!(back.hist[0].life_no, 3);
        assert_eq!(back.hist[0].max_realm, 4);
        assert_eq!(back.hist[0].age, 76.5);
        assert_eq!(back.hist[0].stones, -1200);
        assert_eq!(back.hist[0].comp, 255);
        assert_eq!(back.hist[0].cause, "九问皆妄，你于镜前枯坐而终。");
        assert_eq!(back.hist[1].comp, 3);
        assert_eq!(back.hist[1].cause, "寿元自然耗尽");
    }

    // 缺键默认空表（旧档零迁移）；坏行（段数不足）整行丢弃、界外值钳制
    #[test]
    fn hist_missing_default_and_dirty_clamped() {
        let m = parse_save(OLD_BASE);
        assert!(m.hist.is_empty(), "旧档无 h= 键 → 空表");
        let t = format!(
            "{}h=9 9 99 1.5 2 3 4 77 只留两段\nh=1 2 3 太短\nh=2 5 6 60.0 55 300 900 2 命殒于外 一身道果散入天地\n",
            OLD_BASE
        );
        let m2 = parse_save(&t);
        // 「太短」行丢弃；两条有效记录均入账
        assert_eq!(m2.hist.len(), 2);
        assert_eq!(m2.hist[0].max_realm, 14, "realm 99 → clamp 14");
        assert_eq!(m2.hist[0].comp, 255, "comp 77 界外 → 255 无伴");
        assert_eq!(m2.hist[1].comp, 2);
        assert_eq!(m2.hist[1].cause, "命殒于外 一身道果散入天地", "cause 空格余量不截断");
    }

    // v1.16.7 剧情摘要 hq= 平行行：回环（紧跟 h= → 挂对一条）、旧行缺 hq → None、
    // hist 空错位丢弃、hq 在两条 h= 之间挂前一条（文件序约定）
    #[test]
    fn hq_roundtrip_missing_and_misplaced() {
        // 写端回环：带 story 的记录 → hq= 紧跟 h=
        let mut m = Meta::default();
        m.hist = vec![SimRecord {
            life_no: 2,
            run_no: 5,
            max_realm: 6,
            age: 90.0,
            years: 74,
            dy: 900,
            stones: 5000,
            comp: 1,
            cause: "纸寿尽了".into(),
            story: Some("遇【江底墨匣】；沉墨遗匣·缘了".into()),
        }];
        let text = save_text(&m);
        let mut hq_after_h = false;
        for (a, b) in text.lines().zip(text.lines().skip(1)) {
            if a.starts_with("h=") {
                hq_after_h = b.starts_with("hq=");
            }
        }
        assert!(hq_after_h, "hq= 必须紧跟其 h= 行");
        let back = parse_save(&text);
        assert_eq!(
            back.hist[0].story.as_deref(),
            Some("遇【江底墨匣】；沉墨遗匣·缘了")
        );
        // 旧行（无 hq=）→ None，9 段主档照常解析
        let old = format!("{}h=1 2 3 50.0 40 100 800 255 老死因
", OLD_BASE);
        let m2 = parse_save(&old);
        assert_eq!(m2.hist.len(), 1);
        assert_eq!(m2.hist[0].cause, "老死因");
        assert!(m2.hist[0].story.is_none(), "旧档旧行缺 hq → None");
        // hq 错位（hist 空）→ 整行丢弃，不 panic 不挂错
        let orphan = format!("{}hq=孤儿摘要
", OLD_BASE);
        let m3 = parse_save(&orphan);
        assert!(m3.hist.is_empty());
        // hq 在两条 h= 之间 → 挂前一条（last_mut 语义）
        let mid = format!(
            "{}h=1 1 3 50.0 40 100 800 255 甲
hq=甲的剧情
h=1 2 3 50.0 40 100 800 255 乙
",
            OLD_BASE
        );
        let m4 = parse_save(&mid);
        assert_eq!(m4.hist.len(), 2);
        assert_eq!(m4.hist[0].story.as_deref(), Some("甲的剧情"));
        assert!(m4.hist[1].story.is_none());
    }

    // 防御性截断：脏档 7 条 h= → 只留最近 5 条（尾部保留）
    #[test]
    fn hist_overflow_keeps_latest_five() {
        let mut t = OLD_BASE.to_string();
        for i in 0..7 {
            t.push_str(&format!("h=1 {} 3 50.0 40 100 800 255 死因{}\n", i, i));
        }
        let m = parse_save(&t);
        assert_eq!(m.hist.len(), 5);
        assert_eq!(m.hist[0].run_no, 2, "弹最旧，保 2..=6");
        assert_eq!(m.hist[4].run_no, 6);
        assert_eq!(m.hist[4].cause, "死因6");
    }

    // ---------------- v1.16.8 章节书：ch=/sy= 回环 / 旧档缺键 0 / 坏值 0 ----------------

    // ch=（章节达成位掩码）/ sy=（系统初见闩）回环；缺键零迁移；坏值静默回 0。
    // ch= 独立成行置于 h= 块之前（文件序约定：勿插 h=/hq= 之间）。
    // v1.16.10 补验：sy= 新三位 bit2/3/4（首入现实/首次传承/首修为圆满）与
    // ch= bit8（第九章·书外之章）同口径回环——纯 additive，旧键位不变。
    #[test]
    fn chapters_sys_seen_roundtrip_missing_and_bad() {
        let mut m = Meta::default();
        m.chap_done = 0b0001_0111; // 前五章已了（bit0..=4）
        m.sys_seen = 0b0011; // 周天已开 + 兑换已通
        let text = save_text(&m);
        assert!(text.contains("ch=23\n") && text.contains("sy=3\n"));
        // ch= 行必须先于任何 h= 行（旧稿块文件序约定）
        let ch_pos = text.find("ch=").unwrap();
        let h_pos = text.find("h=").map(|p| p as isize).unwrap_or(isize::MAX);
        assert!(ch_pos < h_pos as usize, "ch= 必须在 h= 旧稿块之前");
        let m2 = parse_save(&text);
        assert_eq!(m2.chap_done, 0b0001_0111);
        assert_eq!(m2.sys_seen, 0b0011);

        // v1.16.10 新位回环：sy bit2..=4（首入现实/首次传承/首修为圆满）+ ch bit8
        let mut m5 = Meta::default();
        m5.chap_done = 0x1FF; // 九章全了（bit0..=8）
        m5.sys_seen = 0b11100; // 三新闩全置
        let text5 = save_text(&m5);
        assert!(text5.contains("ch=511\n") && text5.contains("sy=28\n"));
        let m6 = parse_save(&text5);
        assert_eq!((m6.chap_done, m6.sys_seen), (0x1FF, 0b11100));

        // 旧档缺 ch=/sy= 键 → 0（历史版本从未落盘，向后兼容零迁移；
        // Game::new 载入帧再按事实静默补齐 chap_done）
        let old: String = text
            .lines()
            .filter(|l| !l.starts_with("ch=") && !l.starts_with("sy="))
            .collect::<Vec<_>>()
            .join("\n");
        let m3 = parse_save(&old);
        assert_eq!((m3.chap_done, m3.sys_seen), (0, 0));

        // 坏值 → 0
        let bad = format!("{}ch=abc\nsy=-\n", OLD_BASE);
        let m4 = parse_save(&bad);
        assert_eq!((m4.chap_done, m4.sys_seen), (0, 0));

        // v1.16.11 新位回环：sy bit5（首见洞府经营）纯 additive 独立追加——
        // 既有 0b11100 常量断言不动（上），新位不洗旧位
        let mut m7 = Meta::default();
        m7.sys_seen = 0b100000;
        let text7 = save_text(&m7);
        assert!(text7.contains("sy=32\n"));
        let m8 = parse_save(&text7);
        assert_eq!(m8.sys_seen, 0b100000);
    }

    // v1.16.10 章节称号：佩戴值域扩 16（雨夜执印者）——合法值保留、越界仍钳 -1
    // v1.16.16 同测续用：17（执笔人）入保留域、以 18 作越界样本
    #[test]
    fn worn_title_value16_kept_and_out_of_range_clamped() {
        let t = format!("{}wtitle=16\n", OLD_BASE);
        assert_eq!(parse_save(&t).worn_title, 16, "章节称号合法值保留");
        let t17 = format!("{}wtitle=17\n", OLD_BASE);
        assert_eq!(
            parse_save(&t17).worn_title,
            17,
            "执笔人称号合法值保留（v1.16.16 值域扩 -1..=17）"
        );
        let bad = format!("{}wtitle=18\n", OLD_BASE);
        assert_eq!(parse_save(&bad).worn_title, -1, "越界仍回退自动跟随");
    }

    // ---------------- v1.16.10 事件回顾图鉴：evc= 键（回环/缺键/重定形/越值） ----------------

    // 回环：save_text→parse_save evc 逐位相等（>0 即已收录的单一数据源，无 seen 位）
    #[test]
    fn evc_roundtrip_bitwise() {
        let mut m = Meta::default();
        let n = crate::data::EVENTS.len();
        m.ev_cnt = vec![0; n];
        m.ev_cnt[0] = 1;
        m.ev_cnt[85] = 255; // saturating 顶格
        m.ev_cnt[n - 1] = 7;
        let text = save_text(&m);
        assert!(text.contains("evc="));
        // evc= 行必须先于 h= 旧稿块（文件序约定同 ch=/sy=；按行首匹配——
        // 子串 find 会误中 "ach=" 尾部的 "h="）
        let evc_pos = text
            .lines()
            .position(|l| l.starts_with("evc="))
            .expect("evc= 行缺失");
        let h_pos = text
            .lines()
            .position(|l| l.starts_with("h="))
            .map(|p| p as isize)
            .unwrap_or(isize::MAX);
        assert!(evc_pos < h_pos as usize, "evc= 必须在 h= 旧稿块之前");
        let back = parse_save(&text);
        assert_eq!(back.ev_cnt.len(), n);
        assert_eq!((back.ev_cnt[0], back.ev_cnt[85], back.ev_cnt[n - 1]), (1, 255, 7));
    }

    // 旧档缺 evc= → 全 0（零迁移，照 ach= 重定形先例）；短数组补零、长数组截断、
    // 越值钳 0..=255（负值归 0）
    #[test]
    fn evc_missing_default_and_reshape() {
        let m = parse_save(OLD_BASE);
        assert_eq!(m.ev_cnt, vec![0; crate::data::EVENTS.len()]);
        let t = format!(
            "{}evc=[3, 300, -2]\n",
            OLD_BASE
        );
        let m2 = parse_save(&t);
        assert_eq!(m2.ev_cnt[0], 3);
        assert_eq!(m2.ev_cnt[1], 255, "越值顶格");
        assert_eq!(m2.ev_cnt[2], 0, "负值归 0");
        assert_eq!(m2.ev_cnt[3..], vec![0; crate::data::EVENTS.len() - 3][..]);
        // 长数组截断（脏档塞超长值不 panic 不越表）
        let mut long = String::from("evc=[");
        for i in 0..200 {
            if i > 0 {
                long.push_str(", ");
            }
            long.push_str(&(i % 256).to_string());
        }
        long.push_str("]\n");
        let m3 = parse_save(&format!("{}{}", OLD_BASE, long));
        assert_eq!(m3.ev_cnt.len(), crate::data::EVENTS.len());
        assert_eq!(m3.ev_cnt[117], 117);
    }
}

// ---------------- v1.16.13 复审⑨：ts 注入口回归（纯函数单测） ----------------
// 现场背景：pagehide 兜底快照原用 save_text（ts= 取 m.last_ts 旧值），sync→flush
// 间隔内强杀会令离线收益起点回退一个间隔；拆 save_text_with_ts 后 stash_pending
// 注入 date_now()，内存 m.last_ts 口径（save_flush 真落盘才推进）不动。
#[cfg(test)]
mod v11613_ts_inject_tests {
    use super::{save_text, save_text_with_ts};
    use crate::game::Meta;

    // save_text 恒等于注入 m.last_ts（原路径零行为变化，双端写入文本逐字节不变）
    #[test]
    fn save_text_is_ts_identity() {
        let mut m = Meta::default();
        m.last_ts = 1_700_000_000;
        assert_eq!(save_text(&m), save_text_with_ts(&m, m.last_ts));
    }

    // 注入只改 ts= 一行，其余行（含 h=/hq=/ch=/sy=/evc= 行序）逐字节不变——
    // 旧行解析（parse_save）零迁移不受影响
    #[test]
    fn ts_injection_changes_only_ts_line() {
        let m = Meta::default();
        let a = save_text_with_ts(&m, 111);
        let b = save_text_with_ts(&m, 222);
        let la: Vec<&str> = a.lines().collect();
        let lb: Vec<&str> = b.lines().collect();
        assert_eq!(la.len(), lb.len(), "行数一致（无新增/删行）");
        let mut diff = 0usize;
        for (x, y) in la.iter().zip(lb.iter()) {
            if x != y {
                diff += 1;
                assert!(x.starts_with("ts=") && y.starts_with("ts="), "仅 ts= 行可变");
            }
        }
        assert_eq!(diff, 1, "恰好 ts= 一行不同");
        assert!(b.contains("ts=222\n"), "注入值落行式键值");
    }
}
