// 素材光栅化：打坐修士、图标、仙山夜景背景 —— 全部程序生成，零外部素材
use crate::pixel::*;
use macroquad::prelude::*;

use std::sync::Mutex;

pub struct Art {
    // v1.5.0 起 draw_cultivator 改用 portraits[tier]；portrait 保留为 0 档快捷引用
    #[allow(dead_code)]
    pub portrait: Texture2D,
    pub icon_stone: Texture2D,
    pub icon_yun: Texture2D,
    pub icon_pill: Texture2D,
    // v1.16.5 起两处法宝位改 relic_icons 专属像素画，通用剑图标暂无渲染消费
    // （照 portrait 的 allow(dead_code) 先例保留资产，don't delete—调试/新点位可用）
    #[allow(dead_code)]
    pub icon_sword: Texture2D,
    // v1.16.5 法宝像素画：RELICS 每件专属 12×12 图标（下标即 RELICS 下标；
    // 启动期 eager 生成，勿调序/插入 RELICS 数组——8 件下标语义漂移风险见勘察⑤）
    pub relic_icons: [Texture2D; 8],
    pub bg: Texture2D,
    // v1.6.0 C：map_bg 移除固定字段——改 map_bg_for(region, seed) 按主题参数化 + 缓存
    pub map_icons: [Texture2D; 6],
    pub portraits: [Texture2D; 4], // v1.5.0：立绘四档（0白丁 1道基 2紫府 3墨仙），portrait = portraits[0]
    // v1.7.0 AI 立绘覆盖：assets/portraits/ 下有对应 PNG 则用之，缺失回退像素立绘
    // （portrait_0..3=境界四档；v1.8.0 npc_ovr 扩四：0玄诚长老 1胡掌柜 2宁清霜 3醉道人。wasm 恒回退）
    // v1.14.0：NPC 槽缺失时不再拿 12×12 图标冒充人像——改 name_seed 程序化像素
    // 立绘兜底（art::npc_pixel_tex，只挂 npc_tex None 分支，AI 图命中即短路）；
    // 道侣未结缘槽改书卷剪影（art::comp_silhouette_tex），已结缘恒 comp 真立绘。
    // v1.9.24 懒加载：16 张覆盖图解码后约 7MB RGBA（桌面全量版更大），启动全量
    // 解码既推高峰值内存又拖慢首帧——改为首用才解码，未出场的道侣/NPC 常驻零成本。
    pub portrait_ovr: [LazyTex; 4],
    pub npc_ovr: [LazyTex; 4],
    // v1.7.1 女道侣立绘：comp_0..9 与 COMPANION_NAMES 顺序一一对应（缺失回退丹药图标）
    pub comp_ovr: [LazyTex; 10],
    // v1.9.25 卡顿修复（根因3）：问笔暗角预烘焙——原 4 个半透明矩形逐帧叠加过绘，
    // 烘焙为 1 张 320×180 纹理拉伸全屏（角落双叠 α=1-(1-0.35)² 精确保真），1 次绘制
    pub trib_vignette: Texture2D,
    // v1.9.25 卡顿修复（根因3）：雷云团预烘焙（大 r58+小 r40 双圆 over 合成）——
    // 原 9 朵云 18 个 draw_circle → 9 个纹理 quad，游移公式与色彩不变
    pub cloud: Texture2D,
}

// v1.9.24 懒加载槽：名字在构造期定死（避免每帧 format!），首取时解码一次并
// 缓存结果（含"缺图 None"也缓存，防止每帧重查文件系统/内嵌表）。
// Texture2D 是 Arc 句柄，get() 返回克隆零拷贝。
// v1.16.9 三态化（立绘外置主刀）：wasm 侧 AI 立绘从 wasm 数据段外置到 TapTap 包
// portraits/（18 张 696,587B，wasm raw −37.6%）。wasm get() 首用置 Loading 并起
// 协程 XHR 异步拉图（先例 sounds.rs::start_async_load），Loading 期 get() 返 None
// ——消费侧语义（v1.16.9 复审修后）：事件卡几何按 card.actor 锁定（draw.rs meeting
// latched，图到位只换图不改几何），立绘盒走像素/剪影兜底（npc_pixel_tex/
// comp_silhouette_tex，gallery.rs 同款），人物页/图鉴同槽剪影→AI 一次闪变（每会话
// 每槽一次）；Err 写 Done(None)=调用方 or_else 恒像素/剪影兜底（与原缺图语义一致，
// XHR 仅认 HTTP 200、CDN 抖动即兜底）。桌面侧保持同步解码收口同一三态槽（文件
// 系统+内嵌兜底语义零变化，勘察⑦零回归面）。
enum TexSlot {
    Idle,
    #[allow(dead_code)] // 仅 wasm 协程路径构造（桌面同步直通 Done，cfg 分叉固有）
    Loading,
    Done(Option<Texture2D>),
}

pub struct LazyTex {
    name: &'static str,
    cell: Mutex<TexSlot>,
}

impl LazyTex {
    const fn new(name: &'static str) -> Self {
        Self {
            name,
            cell: Mutex::new(TexSlot::Idle),
        }
    }

    fn get(&self) -> Option<Texture2D> {
        // Done 快路径：克隆返回（无状态迁移）
        {
            let g = self.cell.lock().unwrap();
            if let TexSlot::Done(t) = &*g {
                return t.clone();
            }
        }
        self.get_miss()
    }

    // v1.16.19 拆口：get() 的未命中臂（Idle→首取 / Loading→None）供限流口复用，
    // 语义与原 get() 逐行等价（纯搬移，先例 scroll_list_tick 抽口不动机）。
    fn get_miss(&self) -> Option<Texture2D> {
        #[cfg(target_arch = "wasm32")]
        {
            // Idle → Loading + 协程异步拉图（drop 锁再 spawn，防协程写回时再入死锁；
            // 协程闭包只捕获 name（&'static str，EMBED 表定死），写回经 slot_by_name
            // 全局查槽——引用捕获过不了 start_coroutine 的 'static 约束）。
            // Loading → None（像素兜底帧）
            let mut g = self.cell.lock().unwrap();
            if matches!(&*g, TexSlot::Idle) {
                *g = TexSlot::Loading;
                drop(g);
                let name = self.name;
                let task = macroquad::experimental::coroutines::start_coroutine(async move {
                    let t = load_portrait_wasm(name).await;
                    if let Some(slot) = slot_by_name(name) {
                        if let Ok(mut g) = slot.cell.lock() {
                            *g = TexSlot::Done(t);
                        }
                    }
                });
                std::mem::forget(task); // 注册进 macroquad 全局轮询表即可（sounds.rs 先例）
            }
            None
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            // 桌面：同步解码一次收口 Done（文件系统→内嵌兜底，原语义逐字节等价）
            let mut g = self.cell.lock().unwrap();
            if matches!(&*g, TexSlot::Idle) {
                *g = TexSlot::Done(try_load_ovr(self.name));
            }
            if let TexSlot::Done(t) = &*g {
                return t.clone();
            }
            None
        }
    }

    /// v1.16.19 只读窥视（新出场点收口）：绝不触发解码/XHR——Idle/Loading 返外层
    /// None（「尚未定案」），Done 返 Some(内层)，内层 None=缺图定案（本会话本槽
    /// 恒兜底）。供缘谱行/旧稿行这类一帧多行出场点「只消费已加载纹理」与限流口
    /// 降级臂（单帧首取红线 ≤2/帧，人物页 charpanel.rs:193 纪律同源）。
    pub fn peek(&self) -> Option<Option<Texture2D>> {
        let g = self.cell.lock().unwrap();
        match &*g {
            TexSlot::Done(t) => Some(t.clone()),
            _ => None,
        }
    }
}

// v1.16.19 每帧首取预算（单帧 ≤2 槽，照人物页「任意帧 ≤2 卡在画」纪律外推到行卡
// 出场点）：缘谱行/旧稿行一帧可见可达 10 行，桌面同步解码最坏 10 槽/帧一次性
// 30-80ms 卡帧（勘察红线）——本口把「Idle→首取」限到每帧 2 次，超额降级 peek
// 只消费已加载纹理（wasm 侧 XHR 起协程异步本无此险，桌面是硬约束，两端口径统一）。
// Done(Some) 常态零预算消耗；Done(None) 缺图定案经 peek 命中后不再走 get，防缺图
// 槽每帧吃满预算饿死同页其余槽（LazyTex 三态语义零改动）。
static QUOTA_STAMP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static QUOTA_LEFT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(2);

// 预算扣减：同帧判定用 get_time()（macroquad 每帧帧首更新一次，同帧多次调用同值
// ——跨帧必然递增）；fetch_update 单口扣减，预算尽返 false。绘制帧单线程，Relaxed
// 已足（照 diag_log_once AtomicBool 同级口径）。
fn first_fetch_budget_take() -> bool {
    use std::sync::atomic::Ordering::Relaxed;
    let now = macroquad::time::get_time().to_bits();
    if QUOTA_STAMP.load(Relaxed) != now {
        QUOTA_STAMP.store(now, Relaxed);
        QUOTA_LEFT.store(2, Relaxed);
    }
    QUOTA_LEFT.fetch_update(Relaxed, Relaxed, |v| v.checked_sub(1)).is_ok()
}

// v1.16.9：协程写回查槽——三组 LazyTex 数组统一按 name 定位（name 构造期定死恒唯一）。
// 仅 wasm 协程路径消费（桌面同步收口不经过此口，cfg 收窄消 dead_code）
#[cfg(target_arch = "wasm32")]
fn slot_by_name(name: &str) -> Option<&'static LazyTex> {
    let a = art();
    a.portrait_ovr
        .iter()
        .chain(a.npc_ovr.iter())
        .chain(a.comp_ovr.iter())
        .find(|s| s.name == name)
}

// v1.16.9 wasm 立绘异步加载：miniquad fs 通道 = XHR arraybuffer 不校验 MIME（wasm
// 本体同通道已证真机可达）；?v= 查询串与 index.html GAME_VER 同源（CARGO_PKG_VERSION，
// pack_taptap.py 版本一致性断言兜底）。load_texture 不自动设过滤，Ok 后手动补 Linear
//（对齐 tex_from_bytes 口径）。
#[cfg(target_arch = "wasm32")]
async fn load_portrait_wasm(name: &'static str) -> Option<Texture2D> {
    let url = format!("portraits/{}.png?v={}", name, env!("CARGO_PKG_VERSION"));
    match load_texture(&url).await {
        Ok(t) => {
            t.set_filter(FilterMode::Linear);
            Some(t)
        }
        Err(_) => {
            // v1.16.18 修3：唯一静默失败点补真机诊断埋点——此前 Err 沉 Done(None) 会话
            // 恒书卷剪影，404/CDN/缓存无法分辨。首败一行（name+URL）后静默（会话级
            // 单次防刷屏，AtomicBool 无锁；桌面同源类型检查，桌面无调用方恒空转）。
            // 语义不变：仍 None 兜底、无重试（严禁每帧重拉，性能十戒——配套 triage：
            // 报障者导出 localStorage xiuxian_save 查 gbd= 可同步确认结缘账已落）
            diag_log_once(&format!("portrait load fail: {}", url));
            None
        }
    }
}

// v1.16.18 修3 诊断单次口（cfg 无分叉——桌面也编译，同源过 cargo check 类型检查；
// wasm 经 miniquad log-impl 宏 → console.info，桌面 → println）。本会话首败打印、
// 后续静默；纹理加载失败是事件帧级路径，format! 一次性分配不在帧路径红线内。
#[allow(dead_code)] // 桌面无调用方（load_portrait_wasm 仅 wasm），保留同源类型检查
fn diag_log_once(msg: &str) {
    static LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !LOGGED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        miniquad::info!("{}", msg);
    }
}

impl Art {
    /// 境界四档覆盖立绘（首用解码；缺图回退像素立绘由调用方兜底）
    pub fn portrait_tex(&self, tier: usize) -> Option<Texture2D> {
        self.portrait_ovr[tier.min(3)].get()
    }
    /// NPC 立绘（0玄诚长老 1胡掌柜 2宁清霜 3醉道人）
    pub fn npc_tex(&self, i: usize) -> Option<Texture2D> {
        self.npc_ovr[i.min(3)].get()
    }
    /// 女道侣立绘（comp_0..9）
    pub fn comp_tex(&self, i: usize) -> Option<Texture2D> {
        self.comp_ovr[i.min(9)].get()
    }
    /// v1.16.19 一帧多行出场点限流口（缘谱行/旧稿行）：优先 peek——Done(Some) 真图
    /// 零预算直返；Done(None) 缺图定案直返 None 不占预算；未定案槽预算内才首取
    /// （get，桌面同步解码/wasm 起协程），超额返 None 由调用方兜底绘制。显示判定
    /// 与既有 comp_tex 调用方同源（bond_row_known / r.comp< len），本口只管「何时
    /// 允许首取」不管「谁该显示」。
    pub fn comp_tex_capped(&self, i: usize) -> Option<Texture2D> {
        let s = &self.comp_ovr[i.min(9)];
        match s.peek() {
            Some(t) => t,
            None => {
                if first_fetch_budget_take() {
                    s.get_miss()
                } else {
                    None
                }
            }
        }
    }
}

static ART: std::sync::OnceLock<Art> = std::sync::OnceLock::new();

pub fn init_art() {
    let _ = ART.set(build_art());
}

pub fn art() -> &'static Art {
    ART.get().expect("art 未初始化")
}

// v1.7.0：尝试加载 AI 立绘覆盖（文件缺失静默回退；仅桌面）
#[cfg(not(target_arch = "wasm32"))]
fn try_load_ovr(name: &str) -> Option<Texture2D> {
    // v1.8.9：查找顺序 CWD/assets/portraits → exe 同级 assets/portraits → 内嵌
    //（内嵌与 TapTap 包同源，发布版单独 exe 也自带立绘）
    let rel = format!("assets/portraits/{}.png", name);
    let mut paths = vec![rel.clone()];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(dir.join(&rel).to_string_lossy().to_string());
        }
    }
    for p in paths {
        if let Ok(bytes) = std::fs::read(&p) {
            return Some(tex_from_bytes(&bytes));
        }
    }
    embedded_bytes(name).map(tex_from_bytes)
}

// v1.16.9：内嵌字节解码入口随内嵌表收窄为桌面专用（wasm 侧走 load_portrait_wasm
// 的 load_texture 通道，过滤在 Ok 臂手动补 Linear 同口径）
#[cfg(not(target_arch = "wasm32"))]
fn tex_from_bytes(bytes: &[u8]) -> Texture2D {
    let t = Texture2D::from_file_with_format(bytes, None);
    t.set_filter(FilterMode::Linear);
    t
}

// v1.8.9 内嵌立绘表：与 TapTap 包同源（assets/portraits_wasm，640px 量化版）。
// 三通道查找：桌面 = 文件夹（CWD/exe 同级，换图免重编译）→ 内嵌兜底（发布版
// 单独 exe 自带立绘）；wasm 恒用内嵌。
// v1.15.5：npc_senior（宁清霜）/ npc_drunk（醉道人）补图入列，18 张全齐。
// v1.16.9 立绘外置：整表（fn+EMBED 本体）cfg 收窄为桌面专用——wasm 侧改走
// portraits/ 外置文件（LazyTex 协程拉图），include_bytes 必须 cfg 包住表本体，
// 只 cfg 调用方字节照样进 wasm 数据段（勘察④）。
#[cfg(not(target_arch = "wasm32"))]
fn embedded_bytes(name: &str) -> Option<&'static [u8]> {
    const EMBED: &[(&str, &[u8])] = &[
        (
            "portrait_0",
            include_bytes!("../assets/portraits_wasm/portrait_0.png"),
        ),
        (
            "portrait_1",
            include_bytes!("../assets/portraits_wasm/portrait_1.png"),
        ),
        (
            "portrait_2",
            include_bytes!("../assets/portraits_wasm/portrait_2.png"),
        ),
        (
            "portrait_3",
            include_bytes!("../assets/portraits_wasm/portrait_3.png"),
        ),
        (
            "npc_elder",
            include_bytes!("../assets/portraits_wasm/npc_elder.png"),
        ),
        (
            "npc_merchant",
            include_bytes!("../assets/portraits_wasm/npc_merchant.png"),
        ),
        (
            "npc_senior",
            include_bytes!("../assets/portraits_wasm/npc_senior.png"),
        ),
        (
            "npc_drunk",
            include_bytes!("../assets/portraits_wasm/npc_drunk.png"),
        ),
        ("comp_0", include_bytes!("../assets/portraits_wasm/comp_0.png")),
        ("comp_1", include_bytes!("../assets/portraits_wasm/comp_1.png")),
        ("comp_2", include_bytes!("../assets/portraits_wasm/comp_2.png")),
        ("comp_3", include_bytes!("../assets/portraits_wasm/comp_3.png")),
        ("comp_4", include_bytes!("../assets/portraits_wasm/comp_4.png")),
        ("comp_5", include_bytes!("../assets/portraits_wasm/comp_5.png")),
        ("comp_6", include_bytes!("../assets/portraits_wasm/comp_6.png")),
        ("comp_7", include_bytes!("../assets/portraits_wasm/comp_7.png")),
        ("comp_8", include_bytes!("../assets/portraits_wasm/comp_8.png")),
        ("comp_9", include_bytes!("../assets/portraits_wasm/comp_9.png")),
    ];
    EMBED
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, b)| *b)
}

// v1.6.0 C 仇敌换图：地图底图缓存（region, seed) → Texture2D。
// v1.9.24 容量 8→3：v1.16.8 降采样后每张 632×240×4 ≈ 0.6MB，3 张上限即 ~1.8MB 常驻
//（降采样前 1264×480 单张 2.4MB——手机上纯属浪费，同屏只见一张，3 张已覆盖
// 「当前+刚离开」回切不重建）。
// 主题 0/默认种子会命中 v1.5.0 逐值路径（像素级等价），仅换幕后重建。
static MAP_BG_CACHE: Mutex<Vec<(u32, u32, Texture2D)>> = Mutex::new(Vec::new());

pub fn map_bg_for(region: u32, seed: u32) -> Texture2D {
    let mut cache = MAP_BG_CACHE.lock().unwrap();
    if let Some((_, _, t)) = cache.iter().find(|(r, s, _)| *r == region && *s == seed) {
        return t.clone();
    }
    let t = build_map_bg(region, seed);
    if cache.len() >= 3 {
        cache.remove(0);
    }
    cache.push((region, seed, t.clone()));
    t
}

const O: [u8; 4] = [10, 8, 16, 255]; // 描边

pub fn build_art() -> Art {
    let portraits = std::array::from_fn(build_portrait);
    let icon_stone = build_icon_stone();
    let icon_yun = build_icon_yun();
    let icon_pill = build_icon_pill();
    let icon_sword = build_icon_sword();
    let relic_icons = std::array::from_fn(build_relic_icon); // v1.16.5 法宝像素画
    let bg = build_bg();
    let map_icons = std::array::from_fn(build_map_icon);
    let mut a = Art {
        portrait: portraits[0].clone(),
        icon_stone,
        icon_yun,
        icon_pill,
        icon_sword,
        relic_icons,
        bg,
        map_icons,
        portraits,
        // v1.9.24：覆盖立绘全部懒加载（桌面/手机首帧前零解码，首用各自解码一次）。
        // 名单与此前两段 eager 加载完全一致（npc_senior/npc_drunk 无文件→None 也缓存）。
        portrait_ovr: [
            LazyTex::new("portrait_0"),
            LazyTex::new("portrait_1"),
            LazyTex::new("portrait_2"),
            LazyTex::new("portrait_3"),
        ],
        npc_ovr: [
            LazyTex::new("npc_elder"),
            LazyTex::new("npc_merchant"),
            LazyTex::new("npc_senior"),
            LazyTex::new("npc_drunk"),
        ],
        comp_ovr: [
            LazyTex::new("comp_0"),
            LazyTex::new("comp_1"),
            LazyTex::new("comp_2"),
            LazyTex::new("comp_3"),
            LazyTex::new("comp_4"),
            LazyTex::new("comp_5"),
            LazyTex::new("comp_6"),
            LazyTex::new("comp_7"),
            LazyTex::new("comp_8"),
            LazyTex::new("comp_9"),
        ],
        trib_vignette: build_trib_vignette(),
        cloud: build_cloud(),
    };
    a
}

// v1.9.25 卡顿修复（根因3）：问笔四周暗角。原 4 个 α0.35 矩形（上/下 90px 带 +
// 左/右 110px 带）逐帧叠加，角落被两条带同时覆盖时 macroquad 顺序混合得
// 1-(1-0.35)²=0.5775——烘焙纹理按每像素覆盖带数精确复现该合成，视觉逐像素等价。
// 320×180（4× 降采样）+ Linear 过滤：带缘约 4px 软化，雷劫演出中不可辨。
fn build_trib_vignette() -> Texture2D {
    const SW: u16 = 320;
    const SH: u16 = 180;
    const A: f32 = 0.35;

    let mut img = Image::gen_image_color(SW, SH, Color::new(0.0, 0.0, 0.0, 0.0));

    for py in 0..SH {
        for px in 0..SW {
            // 像素中心映射回逻辑坐标（1280×720）
            let lx = px as f32 * 4.0 + 2.0;
            let ly = py as f32 * 4.0 + 2.0;

            let mut n = 0;
            if ly < 90.0 {
                n += 1;
            }
            if ly >= 630.0 {
                n += 1;
            }
            if lx < 110.0 {
                n += 1;
            }
            if lx >= 1170.0 {
                n += 1;
            }

            if n > 0 {
                let a = 1.0 - (1.0 - A).powi(n);

                img.set_pixel(
                    px as u32,
                    py as u32,
                    Color::new(0.0, 0.0, 0.02, a),
                );
            }
        }
    }

    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Linear);
    tex
}

// v1.9.25 卡顿修复（根因3）：雷云团。原图形为每朵云先画大圆 (r58, α0.45,
// 色 0.20,0.13,0.34) 再画小圆 (r40, α0.5, 色 0.13,0.08,0.24) 于 (+30,+12) 偏移——
// 烘焙按同序 over 合成，双圆硬边逐像素等价；画布 140×136，锚点 = 大圆心 (70,70)。
fn build_cloud() -> Texture2D {
    const W: u16 = 140;
    const H: u16 = 136;

    let big = (70.0f32, 70.0f32, 58.0f32, (0.20, 0.13, 0.34, 0.45));
    let small = (100.0f32, 82.0f32, 40.0f32, (0.13, 0.08, 0.24, 0.5));

    let mut img = Image::gen_image_color(W, H, Color::new(0.0, 0.0, 0.0, 0.0));

    for py in 0..H {
        for px in 0..W {
            let (x, y) = (px as f32 + 0.5, py as f32 + 0.5);

            let in_big = (x - big.0).powi(2) + (y - big.1).powi(2) <= big.2 * big.2;
            let in_small =
                (x - small.0).powi(2) + (y - small.1).powi(2) <= small.2 * small.2;

            // 同序 over 合成：先大圆后小圆
            let (mut r, mut g, mut b, mut a) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);

            if in_big {
                r = big.3 .0;
                g = big.3 .1;
                b = big.3 .2;
                a = big.3 .3;
            }

            if in_small {
                let (r2, g2, b2, a2) = (small.3 .0, small.3 .1, small.3 .2, small.3 .3);

                let oa = a2 + a * (1.0 - a2);

                if oa > 0.0 {
                    r = (r2 * a2 + r * a * (1.0 - a2)) / oa;
                    g = (g2 * a2 + g * a * (1.0 - a2)) / oa;
                    b = (b2 * a2 + b * a * (1.0 - a2)) / oa;
                }

                a = oa;
            }

            if a > 0.0 {
                img.set_pixel(
                    px as u32,
                    py as u32,
                    Color::new(
                        r.min(1.0),
                        g.min(1.0),
                        b.min(1.0),
                        a.min(1.0),
                    ),
                );
            }
        }
    }

    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Linear);
    tex
}

// v1.5.0 立绘四档：0白丁 1道基（金冠）2紫府（金冠+霜鬓+袖口金纹）3墨仙（金冠+白发披肩）
fn build_portrait(tier: usize) -> Texture2D {
    let tier = tier.min(3);
    let mut g = PixGrid::new(32, 36);
    let skin = rgba(255, 219, 186, 255);
    let skin_d = rgba(222, 178, 148, 255);
    let robe = rgba(88, 128, 172, 255);
    let robe_d = rgba(54, 82, 120, 255);
    let robe_l = rgba(126, 164, 205, 255);
    let hair = rgba(30, 26, 38, 255);
    let gold = rgba(222, 186, 110, 255);
    let frost = rgba(226, 230, 238, 255);
    // 发髻 + 头（墨仙档须发皆白）
    let hair_c = if tier >= 3 { frost } else { hair };
    g.fill_ellipse(16, 5, 2, 2, hair_c);
    g.fill_ellipse(16, 9, 4, 4, hair_c);
    g.fill_ellipse(16, 11, 3, 3, skin);
    // 眉眼（垂目入定）
    g.rect(14, 11, 1, 1, rgba(40, 34, 50, 255));
    g.rect(18, 11, 1, 1, rgba(40, 34, 50, 255));
    // 道基以上：束发金冠
    if tier >= 1 {
        g.rect(14, 2, 4, 2, gold);
        g.set(15, 1, rgba(246, 216, 150, 255));
        g.set(16, 1, rgba(246, 216, 150, 255));
    }
    // 紫府以上：霜鬓
    if tier >= 2 {
        g.set(12, 10, frost);
        g.set(13, 10, frost);
        g.set(19, 10, frost);
        g.set(20, 10, frost);
    }
    // 肩与道袍
    g.tri([(10, 13), (22, 13), (16, 27)], robe);
    g.rect(11, 14, 10, 5, robe);
    // 领缘
    g.tri([(16, 14), (13, 19), (16, 19)], robe_l);
    g.tri([(16, 14), (19, 19), (16, 19)], robe_d);
    // 两臂环抱
    g.rect(9, 16, 3, 6, robe_d);
    g.rect(20, 16, 3, 6, robe_d);
    g.fill_circle(12, 22, 2, skin_d);
    g.fill_circle(20, 22, 2, skin_d);
    // 紫府以上：袖口金纹
    if tier >= 2 {
        g.rect(9, 21, 3, 1, gold);
        g.rect(20, 21, 3, 1, gold);
    }
    // 结印双手
    g.fill_circle(16, 21, 2, skin);
    // 腰带
    g.rect(12, 24, 8, 1, gold);
    // 盘膝
    g.fill_ellipse(16, 29, 8, 3, robe_d);
    g.fill_ellipse(16, 30, 6, 2, robe);
    g.outline(O);
    g.to_texture()
}

fn build_icon_stone() -> Texture2D {
    let mut g = PixGrid::new(12, 12);
    let c = rgba(120, 226, 232, 255);
    let c2 = rgba(210, 250, 252, 255);
    let d = rgba(66, 150, 168, 255);
    g.tri([(6, 1), (10, 6), (6, 11)], c);
    g.tri([(6, 1), (2, 6), (6, 11)], d);
    g.rect(5, 3, 1, 2, c2);
    g.outline(O);
    g.to_texture()
}

fn build_icon_yun() -> Texture2D {
    let mut g = PixGrid::new(12, 12);
    let w = rgba(235, 230, 215, 255);
    let b = rgba(40, 36, 56, 255);
    let gd = rgba(222, 186, 110, 255);
    g.fill_circle(6, 6, 5, w);
    for y in 0..12 {
        for x in 6..12 {
            let dx = (x - 6) as f32;
            let dy = (y - 6) as f32;
            if dx * dx + dy * dy <= 25.0 {
                let s = (y - 6) as i32;
                if s >= 0 || (dx + s as f32) >= 0.0 {
                    g.set(x, y, b);
                }
            }
        }
    }
    g.fill_circle(8, 4, 1, b);
    g.fill_circle(4, 8, 1, w);
    g.outline(gd);
    g.to_texture()
}

fn build_icon_pill() -> Texture2D {
    let mut g = PixGrid::new(12, 12);
    let c = rgba(232, 100, 92, 255);
    let c2 = rgba(250, 160, 150, 255);
    g.fill_circle(6, 7, 4, c);
    g.fill_circle(5, 6, 1, c2);
    g.rect(4, 2, 4, 1, rgba(222, 186, 110, 255));
    g.outline(O);
    g.to_texture()
}

fn build_icon_sword() -> Texture2D {
    let mut g = PixGrid::new(12, 12);
    let c = rgba(200, 208, 220, 255);
    let gd = rgba(222, 186, 110, 255);
    for i in 0..8 {
        g.set(3 + i, 9 - i, c);
        g.set(4 + i, 9 - i, rgba(160, 168, 184, 255));
    }
    g.rect(3, 9, 3, 1, gd);
    g.set(3, 10, rgba(90, 70, 40, 255));
    g.set(2, 11, rgba(90, 70, 40, 255));
    g.outline(O);
    g.to_texture()
}

// v1.16.5 法宝像素画：RELICS 每件专属 12×12 图标（与 icon_sword 同规格，两处显示位
// 1:1 零缩放），逐件独立轮廓与主色相；纯数据 relic_pixels 与纹理化 build_relic_icon
// 分离可单测（先例 person_pixels / name_seed_fixed）。启动期 build_art 一次生成
// 8 张（约 4.5KB RGBA），行囊恒渲染 8 行——eager 模式无需预热纪律。
fn relic_pixels(idx: usize) -> PixGrid {
    let mut g = PixGrid::new(12, 12);
    match idx {
        // 0 呵霜毫：斜杆毛笔白毫锋 + 青霜一点（青白）
        0 => {
            let shaft = rgba(122, 90, 56, 255);
            let shaft_hi = rgba(158, 122, 78, 255);
            for i in 0..5 {
                g.set(9 - i, 1 + i, shaft);
                g.set(8 - i, 1 + i, shaft_hi);
            }
            let tip = rgba(238, 242, 246, 255);
            g.set(4, 6, tip);
            g.set(3, 7, tip);
            g.set(2, 8, tip);
            g.set(3, 6, rgba(214, 222, 230, 255));
            g.set(2, 7, rgba(214, 222, 230, 255));
            g.fill_circle(3, 9, 1, rgba(150, 208, 228, 255)); // 呵气成霜
        }
        // 1 裁云笺：斜置纸笺 + 裂金缘（米金）
        1 => {
            let paper = rgba(238, 228, 200, 255);
            let paper_sh = rgba(214, 200, 168, 255);
            g.rect(3, 2, 5, 8, paper);
            g.rect(3, 2, 5, 1, paper_sh);
            for (x, y) in [(8, 3), (9, 4), (8, 6), (9, 7), (8, 9)] {
                g.set(x, y, rgba(212, 178, 106, 255)); // 裁云金缘
            }
            g.rect(4, 5, 3, 1, paper_sh); // 折行
        }
        // 2 镇行钉：金帽铁身竖钉（铁灰）
        2 => {
            g.rect(4, 1, 4, 2, rgba(222, 186, 110, 255)); // 金帽
            g.rect(5, 3, 2, 6, rgba(140, 148, 160, 255)); // 钉身
            g.set(5, 9, rgba(110, 116, 128, 255));
            g.set(6, 9, rgba(110, 116, 128, 255));
            g.tri([(5, 9), (7, 9), (6, 11)], rgba(96, 102, 112, 255)); // 钉尖
        }
        // 3 墨骨伞：半开伞骨辐 + 弯柄（墨黑）
        3 => {
            g.fill_ellipse(5, 4, 5, 3, rgba(38, 34, 46, 255)); // 伞面
            g.rect(4, 1, 3, 1, rgba(38, 34, 46, 255)); // 伞尖
            for i in 1..4 {
                g.set(5 - i, 2 + i, rgba(70, 64, 80, 255)); // 伞骨
                g.set(5 + i, 2 + i, rgba(70, 64, 80, 255));
            }
            g.rect(8, 4, 1, 6, rgba(96, 70, 44, 255)); // 伞柄
            g.set(7, 10, rgba(96, 70, 44, 255)); // 柄钩
        }
        // 4 曝书炉：三足炉 + 墨香汽（铜褐）
        4 => {
            let bronze = rgba(158, 106, 62, 255);
            let bronze_hi = rgba(196, 140, 84, 255);
            g.fill_circle(5, 6, 3, bronze); // 炉身
            g.rect(2, 3, 7, 1, bronze_hi); // 炉口沿
            g.set(5, 2, rgba(200, 196, 190, 255)); // 墨香汽
            g.set(6, 1, rgba(170, 166, 160, 255));
            for x in [3, 5, 7] {
                g.set(x, 9, rgba(120, 78, 44, 255)); // 三足
            }
        }
        // 5 闲章：方印十字篆纹（朱红）
        5 => {
            g.rect(3, 2, 6, 7, rgba(186, 54, 44, 255)); // 印身
            g.rect(4, 3, 4, 5, rgba(240, 232, 220, 255)); // 印面留白
            g.rect(5, 3, 1, 5, rgba(186, 54, 44, 255)); // 篆纹·竖
            g.rect(4, 5, 4, 1, rgba(186, 54, 44, 255)); // 篆纹·横
            g.rect(4, 9, 4, 1, rgba(140, 40, 34, 255)); // 印座
        }
        // 6 洗笔江：坎卦三水纹（青碧）
        6 => {
            let w1 = rgba(64, 158, 148, 255);
            let w2 = rgba(120, 202, 190, 255);
            g.rect(1, 3, 10, 1, w1);
            g.rect(1, 6, 4, 1, w2); // 中断
            g.rect(7, 6, 4, 1, w2);
            g.rect(1, 9, 10, 1, w1);
        }
        // 7 无字簿：摊开空白簿页（纸白）
        7 => {
            let page = rgba(240, 236, 226, 255);
            let spine = rgba(176, 166, 150, 255);
            g.rect(2, 3, 4, 7, page); // 左页
            g.rect(6, 3, 4, 7, page); // 右页
            for y in 3..10 {
                g.set(5, y, spine); // 书脊
            }
            g.rect(2, 10, 8, 1, rgba(110, 84, 56, 255)); // 封底
        }
        _ => {}
    }
    g
}

fn build_relic_icon(idx: usize) -> Texture2D {
    let mut g = relic_pixels(idx);
    g.outline(O);
    g.to_texture()
}

// v1.16.7 加载小刀：boot 唯一大像素项降采样——640×360 生成（像素写省 75%），
// 消费侧 draw_bg 传 dest_size 1280×720 由既有 set_filter(Linear) 放大（Linear
// 先例 art.rs:115/295）。bg 是渐变夜空+山影剪影非像素细节：星星坐标随域减半
// 位移、月晕环带宽度减半（勘察 risk② 预期，真机截图验收口径），远景观感不变
fn build_bg() -> Texture2D {
    let (iw, ih) = (640u16, 360u16);
    let w = 640i32;
    let h = 360i32;
    let mut img = Image::gen_image_color(iw, ih, Color::from_rgba(0, 0, 0, 255));
    {
        let d = img.get_image_data_mut();
        // 纵向渐变夜空（t=y/h 归一，随域自适应）
        for y in 0..h {
            let t = y as f32 / h as f32;
            let r = (16.0 + 34.0 * t) as u8;
            let g = (11.0 + 22.0 * t) as u8;
            let b = (40.0 + 56.0 * t) as u8;
            for x in 0..w {
                d[(y * w + x) as usize] = [r, g, b, 255];
            }
        }
        // 星星（确定性伪随机；数量保留、坐标随域减半——密度×4 夜空更繁）
        let mut seed = 20261008u32;
        let mut rnd = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / 16777216.0
        };
        for _ in 0..170 {
            let x = (rnd() * w as f32) as i32;
            let y = (rnd() * (h as f32 * 0.55)) as i32;
            let b = (140.0 + rnd() * 115.0) as u8;
            d[(y * w + x) as usize] = [b, b, (b as f32 * 1.05).min(255.0) as u8, 255];
            if rnd() < 0.12 {
                for (xx, yy) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                    if xx >= 0 && yy >= 0 && xx < w && yy < h {
                        d[(yy * w + xx) as usize] = [b / 2, b / 2, b / 2 + 20, 255];
                    }
                }
            }
        }
        // 月亮 + 月晕（圆心/半径全部 ×0.5：34/36/58 → 17/18/29）
        let (mx, my) = (520i32, 60i32);
        for y in (my - 30)..=(my + 30) {
            for x in (mx - 30)..=(mx + 30) {
                if x < 0 || y < 0 || x >= w || y >= h {
                    continue;
                }
                let dx = (x - mx) as f32;
                let dy = (y - my) as f32;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist <= 17.0 {
                    d[(y * w + x) as usize] = [246, 238, 210, 255];
                } else if dist <= 18.0 {
                    d[(y * w + x) as usize] = [255, 244, 214, 255];
                } else if dist <= 29.0 {
                    let a = (1.0 - (dist - 18.0) / 11.0) * 46.0;
                    let idx = (y * w + x) as usize;
                    let old = d[idx];
                    d[idx] = [
                        old[0].saturating_add(255u8.min((255.0 * a / 255.0) as u8) / 4),
                        old[1].saturating_add(240u8.min((240.0 * a / 255.0) as u8) / 4),
                        old[2].saturating_add(200u8.min((200.0 * a / 255.0) as u8) / 4),
                        255,
                    ];
                }
            }
        }
        // 月面阴影（偏移 (-4,-3)、r²=90/4）
        for y in (my - 10)..=(my + 10) {
            for x in (mx - 10)..=(mx + 10) {
                let dx = (x - mx - 4) as f32;
                let dy = (y - my - 3) as f32;
                if dx * dx + dy * dy < 22.5 {
                    d[(y * w + x) as usize] = [224, 214, 188, 255];
                }
            }
        }
        // 三层远山（base_y/amp ×0.5，t=x/w 归一相位不变）
        let layer = |d: &mut [[u8; 4]], base_y: f32, amp: f32, col: [u8; 4], ph: f32| {
            for x in 0..w {
                let t = x as f32 / w as f32;
                let yv = base_y
                    - amp * (0.6 * (t * 9.0 + ph).sin() + 0.4 * (t * 21.0 + ph * 2.0).sin()).abs()
                    - amp * 0.3 * (t * 33.0 + ph).sin().abs();
                let top = yv as i32;
                for y in top.max(0)..h {
                    d[(y * w + x) as usize] = col;
                }
            }
        };
        layer(d, 235.0, 45.0, [42, 30, 78, 255], 1.3);
        // 楼阁剪影（画在中景山上；基点/尺寸 ×0.5）
        layer(d, 280.0, 35.0, [32, 22, 60, 255], 4.1);
        let (px0, py0) = (90i32, 222i32);
        for (rx, ry, rw, rh) in [
            (px0 - 7, py0 + 0, 14, 20),
            (px0 - 5, py0 - 8, 10, 9),
            (px0 - 3, py0 - 15, 5, 8),
        ] {
            for y in ry..ry + rh {
                for x in rx..rx + rw {
                    if x >= 0 && y >= 0 && x < w && y < h {
                        d[(y * w + x) as usize] = [12, 8, 26, 255];
                    }
                }
            }
        }
        // 飞檐（水平延伸 8→4，垂直步进 i/3→i/2 保斜率）
        for i in 0..8 {
            for (xx, yy) in [(px0 - 7 - i, py0 + i / 2), (px0 + 7 + i, py0 + i / 2)] {
                if xx >= 0 && xx < w && yy >= 0 && yy < h {
                    d[(yy * w + xx) as usize] = [12, 8, 26, 255];
                }
            }
        }
        layer(d, 320.0, 30.0, [22, 14, 42, 255], 7.7);
    }
    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Linear);
    tex
}

// 河心曲线（图像系坐标，画河/林木规避/水面波光共用，坐标单一来源）
pub fn map_river_y(x: f32) -> f32 {
    320.0 + 40.0 * (x * 0.004).sin() + 20.0 * (x * 0.011 + 1.7).sin()
}

// 山河图志底图：渐变夜幕 + 苔斑 + 山脊带 + 河流 + 林木 + 暗金双线边框
// v1.6.0 C 仇敌换图：按 (region, seed) 参数化——主题色/密度取 MAP_THEMES[region%5]，
// LCG 种子由调用方传入，林木避让坐标经 data::node_xy（含主题抖动）。
// 主题 0 + 默认种子 = v1.5.0 逐值路径（LCG 调用序列保持一致，像素级等价）。
// v1.16.8 加载小刀②：进世界同步底图降采样 1264×480→632×240（像素写 606K→151K，
// −75%，照 v1.16.7 build_bg 先例 art.rs:589-594）。坐标纪律：消费端世界不变——
// world.rs:554 dest_size 仍传 1264×480（Linear 放大，纹理 art.rs:845 恒 Linear），
// map_river_y 保持 1264 图像系口径（world.rs:578 水面波光零改动），本函数内全部
// 特征坐标按「源 px = 图像 px/2」映射：山脊带 base/amp/带宽减半、河心曲线采样
// x*2 后 /2、林木/苔斑避让比较在图像系（x*2,y*2）、边框 8→4px、林木簇 2/3px→1/2px
// （苔斑改 1px 单点，Linear 放大后观感≈原 1-2px 软点——1px 细节减半的既定取舍，
// 真机截图验收口径）。
fn build_map_bg(region: u32, seed_arg: u32) -> Texture2D {
    let t = crate::data::theme_of(region);
    let (iw, ih) = (632u16, 240u16);
    let w = 632i32;
    let h = 240i32;
    let mut img = Image::gen_image_color(iw, ih, Color::from_rgba(0, 0, 0, 255));
    {
        let d = img.get_image_data_mut();
        // 纵向渐变（主题 0：(24,34,40) → (14,20,26)）
        for y in 0..h {
            let f = y as f32 / h as f32;
            let r = (t.grad_top.0 as f32 + (t.grad_bot.0 as f32 - t.grad_top.0 as f32) * f) as u8;
            let g = (t.grad_top.1 as f32 + (t.grad_bot.1 as f32 - t.grad_top.1 as f32) * f) as u8;
            let b = (t.grad_top.2 as f32 + (t.grad_bot.2 as f32 - t.grad_top.2 as f32) * f) as u8;
            for x in 0..w {
                d[(y * w + x) as usize] = [r, g, b, 255];
            }
        }
        // 河心曲线：抽为 pub fn map_river_y（v1.5.0 与水面波光共用，坐标单一来源）
        // 确定性伪随机（照 build_bg 先例）；v1.6.0 种子参数化
        let mut seed = seed_arg;
        let mut rnd = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / 16777216.0
        };
        // 苔斑噪点（主题 0：600 个暗绿/暗褐；源 1px 单点=图像系 ~2px 软点，
        // 原 1-2px+40% 延伸在半分辨率下改单点——密度靠数量保持）
        let moss_n = (600.0 * t.deco) as usize;
        for _ in 0..moss_n {
            let x = (rnd() * w as f32) as i32;
            let y = (rnd() * h as f32) as i32;
            let c = if rnd() < 0.6 {
                [
                    (t.moss_a.0 as f32 + rnd() * 14.0) as u8,
                    (t.moss_a.1 as f32 + rnd() * 12.0) as u8,
                    t.moss_a.2,
                    255,
                ]
            } else {
                [
                    (t.moss_b.0 as f32 + rnd() * 10.0) as u8,
                    (t.moss_b.1 as f32 + rnd() * 8.0) as u8,
                    t.moss_b.2,
                    255,
                ]
            };
            d[(y * w + x) as usize] = c;
        }
        // 两条横向山脊带（逐列：脊线 y = base ± amp·sin 噪声，向下填充成带；
        // base/amp/带高按源域减半，频率 tt 归一不变=振荡数不变）
        let band = |d: &mut [[u8; 4]], base: f32, amp: f32, ph: f32, col: [u8; 4]| {
            for x in 0..w {
                let tt = x as f32 / w as f32;
                let top = (base
                    - amp
                        * (0.6 * (tt * 9.0 + ph).sin().abs()
                            + 0.4 * (tt * 21.0 + ph * 2.0).sin().abs())) as i32;
                let bh = (23.0 + 13.0 * (tt * 13.0 + ph).sin().abs()) as i32;
                for y in top.max(0)..(top + bh).min(h) {
                    d[(y * w + x) as usize] = col;
                }
            }
        };
        band(d, 100.0, 27.5, 1.3, [t.ridge_a.0, t.ridge_a.1, t.ridge_a.2, 255]);
        band(d, 165.0, 35.0, 4.1, [t.ridge_b.0, t.ridge_b.1, t.ridge_b.2, 255]);
        // 河流：横贯折线带，主色 + 上缘高光（map_river_y 恒 1264 图像系——
        // 源坐标 x*2 采样、y/2 落点；河宽 11+3→5.5+1.5 同式随源域减半）
        for x in 0..w {
            let xm = x as f32 * 2.0;
            let yc = map_river_y(xm) / 2.0;
            let hw = 5.5 + 1.5 * (xm * 0.02 + 0.5).sin();
            let y0 = (yc - hw) as i32;
            let y1 = (yc + hw) as i32;
            for y in y0.max(0)..=y1.min(h - 1) {
                d[(y * w + x) as usize] = [t.river.0, t.river.1, t.river.2, 255];
            }
            let hl = y0.max(0);
            if hl <= y1.min(h - 1) {
                d[(hl * w + x) as usize] = [t.river_hl.0, t.river_hl.1, t.river_hl.2, 255];
            }
        }
        // 林木（主题 0：220 个 1×1/2×2 深绿点簇=图像系 2-4px；距节点 <40 图像px
        // 或 |y−河心|<20 图像px 不画——避让比较全在图像系 x*2,y*2，node_xy 零改动）
        let forest_n = (220.0 * t.deco) as usize;
        for _ in 0..forest_n {
            let x = (rnd() * w as f32) as i32;
            let y = (rnd() * h as f32) as i32;
            let xm = x as f32 * 2.0;
            let ym = y as f32 * 2.0;
            let mut ok = (ym - map_river_y(xm)).abs() >= 20.0;
            if ok {
                for k in 0..crate::data::LOCATIONS.len() {
                    let (nx, ny) = crate::data::node_xy(k, region);
                    let dx = xm - nx;
                    let dy = ym - ny;
                    if (dx * dx + dy * dy).sqrt() < 40.0 {
                        ok = false;
                        break;
                    }
                }
            }
            if !ok {
                continue;
            }
            let c = if rnd() < 0.5 {
                [t.forest_a.0, t.forest_a.1, t.forest_a.2, 255]
            } else {
                [t.forest_b.0, t.forest_b.1, t.forest_b.2, 255]
            };
            let s = if rnd() < 0.5 { 1 } else { 2 };
            for yy in y..(y + s).min(h) {
                for xx in x..(x + s).min(w) {
                    d[(yy * w + xx) as usize] = c;
                }
            }
        }
        // 四周暗金双线边框（图像系 8px → 源 4px：i∈{0,1,3,4}）
        for i in [0, 1, 3, 4] {
            for x in 0..w {
                d[(i * w + x) as usize] = [t.frame.0, t.frame.1, t.frame.2, 255];
                d[((h - 1 - i) * w + x) as usize] = [t.frame.0, t.frame.1, t.frame.2, 255];
            }
            for y in 0..h {
                d[(y * w + i) as usize] = [t.frame.0, t.frame.1, t.frame.2, 255];
                d[(y * w + w - 1 - i) as usize] = [t.frame.0, t.frame.1, t.frame.2, 255];
            }
        }
    }
    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Linear);
    tex
}

// 6 地点图标：12×12 像素图（宗门/坊市/秘境/洞府/荒骨/天雷峰）
fn build_map_icon(i: usize) -> Texture2D {
    let mut g = PixGrid::new(12, 12);
    match i {
        0 => {
            // 青云宗：三层收分塔 + 翘檐（瓦青/木金）
            let wq = rgba(90, 120, 110, 255);
            let gd = rgba(222, 186, 110, 255);
            g.rect(4, 8, 4, 3, wq);
            g.rect(5, 9, 2, 2, gd); // 门
            g.rect(2, 7, 8, 1, gd); // 一层檐
            g.set(1, 6, gd); // 翘檐
            g.set(10, 6, gd);
            g.rect(4, 4, 4, 3, wq);
            g.rect(3, 3, 6, 1, gd);
            g.set(2, 2, gd);
            g.set(9, 2, gd);
            g.rect(5, 1, 2, 2, wq);
            g.set(5, 0, gd);
            g.set(6, 0, gd);
        }
        1 => {
            // 临仙坊市：屋舍 + 方孔铜钱（暖褐/铜黄）
            let br = rgba(190, 140, 80, 255);
            let cp = rgba(230, 190, 100, 255);
            g.rect(1, 6, 5, 5, br);
            g.tri([(0, 6), (6, 6), (3, 3)], rgba(150, 105, 60, 255));
            g.set(3, 8, rgba(90, 60, 40, 255)); // 门
            g.fill_circle(9, 8, 3, cp);
            g.rect(8, 7, 2, 2, rgba(40, 30, 20, 255)); // 方孔
        }
        2 => {
            // 落霞秘境：拱形石门 + 门内辉光（石灰/青白）
            let st = rgba(140, 140, 160, 255);
            let gl = rgba(180, 240, 240, 255);
            g.rect(3, 5, 6, 6, st);
            g.fill_circle(6, 5, 3, st);
            g.rect(5, 6, 2, 5, gl);
            g.set(6, 5, gl);
            g.set(5, 5, rgba(220, 252, 252, 255));
        }
        3 => {
            // 清源洞府：月洞门 + 雾纹（岩灰/雾白）
            let rk = rgba(110, 105, 125, 255);
            let mt = rgba(210, 220, 230, 255);
            g.tri([(0, 11), (6, 2), (11, 11)], rk);
            g.rect(0, 10, 12, 1, rk);
            g.fill_circle(6, 9, 3, rgba(22, 18, 34, 255)); // 洞口
            g.rect(4, 7, 4, 1, mt);
            g.set(3, 9, mt);
            g.set(9, 10, mt);
        }
        4 => {
            // 荒骨战场：交叉断剑/断骨（骨白/锈红）
            let bn = rgba(210, 205, 190, 255);
            let ru = rgba(150, 80, 70, 255);
            for k in 0..6 {
                g.set(2 + k, 9 - k, bn);
                g.set(9 - k, 9 - k, ru);
            }
            g.rect(1, 10, 2, 1, bn);
            g.rect(7, 10, 3, 1, bn);
            g.set(1, 4, ru);
        }
        _ => {
            // 天雷峰：尖峰 + 闪电折线（岩蓝灰/电白）
            let pk = rgba(90, 100, 130, 255);
            let lt = rgba(240, 240, 180, 255);
            g.tri([(1, 11), (6, 1), (11, 11)], pk);
            g.tri([(4, 4), (6, 1), (8, 4)], rgba(222, 230, 246, 255)); // 雪顶
            g.set(3, 2, lt);
            g.set(4, 3, lt);
            g.set(3, 4, lt);
            g.set(4, 5, lt);
        }
    }
    g.outline(O);
    g.to_texture()
}

// 绘制修士：aura 色随境界变化，闭关时脚下光环脉动
// v1.5.0 立绘分档：tier≥2 背光+环绕光点，tier≥3 仙环；tier 选 portraits[tier.min(3)]
pub fn draw_cultivator(
    art: &Art,
    cx: f32,
    cy: f32,
    scale: f32,
    aura: Color,
    t: f32,
    meditating: bool,
    tier: usize,
) {
    let s = scale;
    let tier = tier.min(3);
    // 光环
    let pulse = 0.85 + 0.15 * (t * (if meditating { 2.2 } else { 1.1 })).sin();
    let r1 = 86.0 * s * pulse;
    // 紫府以上：背后背光
    if tier >= 2 {
        draw_circle(
            cx,
            cy + 6.0 * s,
            r1 * 1.22,
            Color::new(aura.r, aura.g, aura.b, 0.10),
        );
    }
    draw_circle(
        cx,
        cy + 6.0 * s,
        r1,
        Color::new(aura.r, aura.g, aura.b, 0.10),
    );
    draw_circle(
        cx,
        cy + 6.0 * s,
        r1 * 0.72,
        Color::new(aura.r, aura.g, aura.b, 0.14),
    );
    draw_circle_lines(
        cx,
        cy + 6.0 * s,
        r1,
        2.0,
        Color::new(aura.r, aura.g, aura.b, 0.35 * pulse),
    );
    draw_circle_lines(
        cx,
        cy + 6.0 * s,
        r1 * 0.72,
        1.5,
        Color::new(aura.r, aura.g, aura.b, 0.28 * pulse),
    );
    // 墨仙档：身后金色仙环
    if tier >= 3 {
        draw_circle_lines(
            cx,
            cy + 6.0 * s,
            r1 * 1.18,
            2.5,
            Color::new(1.0, 0.85, 0.5, 0.5 * pulse),
        );
    }
    // 底座蒲团阴影
    draw_ellipse(
        cx,
        cy + 60.0 * s,
        60.0 * s,
        12.0 * s,
        0.0,
        Color::new(0.0, 0.0, 0.0, 0.35),
    );
    // 本体 32×36 放大；v1.7.2 AI 覆盖立绘为全身像（约 1:2 竖版）——按宽高比
    // contain 收进原框（宽上限防超框），底边对齐像素立绘框底（足底落于蒲团垫）
    let bob = if meditating {
        (t * 2.0).sin() * 3.0
    } else {
        0.0
    };
    // v1.9.24：覆盖立绘懒加载（首帧起第一次绘制时解码，此后缓存命中零成本）
    let ovr = art.portrait_tex(tier);
    let (tex, pw, ph) = match ovr.as_ref() {
        Some(t) => {
            let ar = t.width() / t.height().max(1.0);
            let h = 180.0 * s;
            let w = (h * ar).min(220.0 * s);
            (t, w, (w / ar).min(h))
        }
        None => (&art.portraits[tier], 160.0 * s, 180.0 * s),
    };
    draw_texture_ex(
        tex,
        cx - pw * 0.5,
        cy + 86.4 * s - ph + bob,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(pw, ph)),
            ..Default::default()
        },
    );
    // 紫府以上：绕体光点
    if tier >= 2 {
        for k in 0..5 {
            let a = t * 1.4 + k as f32 * std::f32::consts::TAU / 5.0;
            let ox = a.cos() * r1 * 1.05;
            let oy = (a.sin() * 0.45) * r1 * 1.05 - 14.0 * s;
            draw_circle(
                cx + ox,
                cy + 6.0 * s + oy,
                2.2 * s,
                Color::new(1.0, 0.92, 0.65, 0.55 + 0.4 * (t * 3.0 + k as f32).sin()),
            );
        }
    }
}

// ================= v1.14.0 程序化像素立绘兜底 =================
// 背景：宁清霜/醉道人暂无 AI 立绘（npc_senior/npc_drunk），旧兜底拿 12×12 功能图标
// （云纹/丹药）放大 144px 冒充人像，夹在真立绘卡间观感差。此处按名字种子确定性
// 生成 28×40 像素半身像：FNV-1a 名字种子 + 定参 LCG 伪随机流（先例 build_bg 星星
// art.rs:486-489 / build_map_bg 苔斑），同名必出同像、重启不变、零外部随机源。
// AI 图放入 assets/portraits/ 后 LazyTex 命中即短路，本路径永不触发（代码不假定
// 这两张图永远缺席）。纯数据构建（*_pixels 返回 PixGrid）与纹理化分离——单测只比
// 像素缓冲，不触 GPU；28×40 contain 进 220×310 立绘框 = 217×310，to_texture 恒
// Nearest（pixel.rs:112），放大 ~7.8px/格与全游戏像素风一致。

// FNV-1a 逐 char 散列（中文名按码点编码，跨平台确定）
fn name_seed(name: &str) -> u32 {
    let mut h: u32 = 0x811C_9DC5;
    for ch in name.chars() {
        h ^= ch as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

// 名字种子 → 像素半身像（28×40：发髻/脸/眉眼/肩颈/交领道袍/袖手，底边出画=半身）。
// 知名 NPC 命中锚点特征表（与 data.rs NPC_DESC 气质对齐）；未知名纯种子出像。
fn person_pixels(name: &str, female: bool) -> PixGrid {
    let mut g = PixGrid::new(28, 40);
    let senior = name == "宁清霜";
    let drunk = name == "醉道人";
    let mut s = name_seed(name);
    let mut rnd = || {
        s = s.wrapping_mul(1664525).wrapping_add(1013904223);
        (s >> 8) as f32 / 16777216.0
    };
    let skin = rgba(255, 219, 186, 255);
    let skin_d = rgba(222, 178, 148, 255);
    let ink = rgba(40, 34, 50, 255); // 眉眼墨色（build_portrait 同款）
    let frost = rgba(226, 230, 238, 255);
    let gold = rgba(222, 186, 110, 255);
    let gold_d = rgba(160, 125, 55, 255);

    // 袍色三层明暗（先例 build_portrait 88,128,172 系）：知名 NPC 锚点定色，
    // 未知名从五案书道风查表选
    let robes = [
        (rgba(88, 128, 172, 255), rgba(54, 82, 120, 255), rgba(126, 164, 205, 255)), // 玄青
        (rgba(86, 62, 110, 255), rgba(62, 44, 82, 255), rgba(112, 86, 138, 255)), // 紫棠
        (rgba(96, 108, 92, 255), rgba(66, 76, 64, 255), rgba(126, 140, 118, 255)), // 苔青
        (rgba(146, 118, 90, 255), rgba(108, 86, 64, 255), rgba(174, 146, 114, 255)), // 赭褐
        (rgba(120, 130, 152, 255), rgba(88, 96, 118, 255), rgba(152, 162, 184, 255)), // 月灰
    ];
    let (robe, robe_d, robe_l) = if senior {
        // 宁清霜：青碧道袍 + 月白领缘（大师姐，剑心通明）
        (rgba(64, 100, 96, 255), rgba(42, 68, 66, 255), rgba(216, 224, 220, 255))
    } else if drunk {
        // 醉道人：灰褐道袍（来历不明的散修酒友）
        (rgba(122, 106, 85, 255), rgba(90, 78, 60, 255), rgba(150, 132, 106, 255))
    } else {
        robes[(rnd() * 5.0) as usize % 5]
    };
    // 发色：宁清霜霜白长发；未知名男修小概率暗紫发；余者墨发
    let hair = if senior {
        frost
    } else if !female && !drunk && rnd() < 0.15 {
        rgba(60, 48, 66, 255)
    } else {
        rgba(30, 26, 38, 255)
    };
    // 腰带/饰件色：宁清霜暗金（剑穗同源）、醉道人褪色金、未知名金/青/紫三选一
    let sash = if senior {
        gold_d
    } else if drunk {
        rgba(150, 120, 70, 255)
    } else if rnd() < 0.6 {
        gold
    } else if rnd() < 0.5 {
        rgba(130, 200, 200, 255)
    } else {
        rgba(180, 140, 200, 255)
    };

    // ---- 发髻与头（女高髻/男小髻，髻位 ±1px 种子微差）----
    let bun_dx = (rnd() * 3.0) as i32 - 1;
    g.fill_ellipse(
        14 + bun_dx,
        if female { 3 } else { 2 },
        if female { 3 } else { 2 },
        2,
        hair,
    );
    g.fill_ellipse(14, 7, 5, 4, hair); // 发顶包脸
    g.fill_ellipse(14, 9, 4, 3, skin); // 脸

    // ---- 肩颈与道袍（半身，底边出画；女窄肩 sh=8 / 男宽肩 sh=6）----
    g.rect(13, 12, 2, 1, skin_d); // 颈
    let sh: i32 = if female { 8 } else { 6 };
    g.tri([(sh, 14), (28 - sh, 14), (14, 25)], robe); // 肩线收腰
    g.rect(sh + 1, 15, 26 - 2 * sh, 11, robe); // 躯干柱
    // 交领右衽（build_portrait 双色三角手法）；醉道人衣襟歪斜 +1px
    let skew = if drunk { 1 } else { 0 };
    g.tri(
        [(14 + skew, 13), (11 + skew, 20), (14 + skew, 20)],
        robe_l,
    );
    g.tri(
        [(14 + skew, 13), (17 + skew, 20), (14 + skew, 20)],
        robe_d,
    );
    // 两袖垂臂 + 宽袖口（手拢袖中）
    g.rect(sh - 2, 16, 3, 7, robe_d);
    g.rect(28 - sh - 1, 16, 3, 7, robe_d);
    g.rect(sh - 3, 22, 5, 3, robe);
    g.rect(28 - sh - 2, 22, 5, 3, robe);
    // 腰带 + 下摆（外放 1px 成袍势，中缝一道衣纹）
    g.rect(sh, 25, 28 - 2 * sh, 2, sash);
    g.rect(sh, 27, 28 - 2 * sh, 13, robe_d);
    g.rect(13, 27, 2, 13, robe);

    // ---- 女修：两缕垂发垂落肩前（霜白发在深底上格外醒目）----
    if female {
        g.rect(9, 12, 2, 8, hair);
        g.rect(17, 12, 2, 8, hair);
    }

    // ---- 眉眼与锚点特征 ----
    if senior {
        // 宁清霜：细长垂目 + 双垂落发带 + 腰侧一缕暗金剑穗
        g.rect(11, 9, 2, 1, ink);
        g.rect(15, 9, 2, 1, ink);
        g.rect(8, 5, 1, 9, frost);
        g.rect(19, 5, 1, 9, frost);
        g.rect(8, 27, 1, 3, gold_d);
        g.set(8, 30, gold);
    } else {
        g.rect(12, 9, 1, 1, ink);
        g.rect(15, 9, 1, 1, ink);
        if drunk {
            // 醉道人：眯眼上扬 + 脸颊两点酒晕 + 松散发髻两缕散发 + 身侧酒葫芦
            g.set(11, 8, ink);
            g.set(17, 8, ink);
            g.set(11, 10, rgba(196, 112, 90, 255));
            g.set(17, 10, rgba(196, 112, 90, 255));
            g.rect(8, 8, 1, 9, hair);
            g.rect(20, 9, 1, 8, hair);
            let (gourd, gourd_l, gourd_d2) =
                (rgba(150, 102, 58, 255), rgba(176, 126, 74, 255), rgba(110, 74, 42, 255));
            g.set(22, 16, gold); // 木塞（金点点缀）
            g.fill_ellipse(22, 19, 2, 2, gourd_l); // 上腹
            g.rect(21, 21, 2, 1, gourd_d2); // 束颈
            g.fill_ellipse(22, 25, 2, 3, gourd); // 下腹
        } else if !female && rnd() < 0.35 {
            g.rect(13, 11, 2, 1, hair); // 未知名男修小概率短须
            g.set(13, 12, hair);
        }
    }
    g.outline(O);
    g
}

// 未结缘道侣兜底：合拢书卷剪影（28×40 同规格）——「同页人未缝之页」意象，
// 替换旧丹药图标兜底位；结缘后 comp_0..9 真立绘恒命中，本纹理不再出现
fn comp_silhouette_pixels() -> PixGrid {
    let mut g = PixGrid::new(28, 40);
    let paper = rgba(232, 223, 200, 255); // 米白卷面
    let paper_d = rgba(205, 193, 168, 255);
    let wood = rgba(92, 72, 52, 255); // 天地杆
    let wood_d = rgba(62, 48, 36, 255);
    let band = rgba(180, 148, 88, 255); // 暗金束带
    let band_d = rgba(140, 112, 62, 255);
    let stitch = rgba(150, 132, 104, 255);
    // 天杆 + 轴头
    g.rect(5, 3, 18, 3, wood);
    g.set(4, 4, wood_d);
    g.set(23, 4, wood_d);
    // 卷面（卷起未展：卷芯与两道层叠暗线）
    g.rect(7, 6, 14, 28, paper);
    g.rect(7, 6, 2, 28, paper_d);
    g.rect(11, 6, 1, 28, paper_d);
    // 卷首缝线（未启：两排短缝）
    for x in [9, 13, 17] {
        g.rect(x, 8, 2, 1, stitch);
        g.rect(x, 10, 2, 1, stitch);
    }
    // 束带缠腰 + 结扣
    g.rect(7, 20, 14, 3, band);
    g.rect(7, 22, 14, 1, band_d);
    g.fill_circle(14, 21, 1, band_d);
    // 地杆 + 轴头
    g.rect(5, 34, 18, 3, wood);
    g.set(4, 35, wood_d);
    g.set(23, 35, wood_d);
    g.outline(O);
    g
}

// 像素立绘纹理缓存：整局只生成一次，每帧取句柄零分配（先例 LazyTex/MAP_BG_CACHE）
static NPC_PIX_TEX: Mutex<[Option<Texture2D>; 4]> = Mutex::new([None, None, None, None]);
static COMP_SIL_TEX: Mutex<Option<Texture2D>> = Mutex::new(None);

/// NPC 像素立绘兜底（i=0..=3）：仅当 npc_tex(i) 为 None（无 AI 图）时由调用方
/// or_else 进入；AI 图放入 assets/portraits/ 重启后自动覆盖，本函数不再触发。
/// 知名 NPC（宁清霜/醉道人）走锚点特征；未知名纯种子定性别出像，不假定名单。
pub fn npc_pixel_tex(i: usize) -> Option<Texture2D> {
    let i = i.min(3);
    let mut c = NPC_PIX_TEX.lock().unwrap();
    if c[i].is_none() {
        let name = crate::data::NPC_NAMES[i];
        let female = match name {
            "宁清霜" => true,
            "醉道人" => false,
            _ => name_seed(name) % 2 == 1,
        };
        c[i] = Some(build_person_pixel(name, female));
    }
    c[i].clone()
}

/// 未结缘道侣书卷剪影兜底（整局一次生成；已结缘 comp_tex 恒命中不经此路）
pub fn comp_silhouette_tex() -> Option<Texture2D> {
    let mut c = COMP_SIL_TEX.lock().unwrap();
    if c.is_none() {
        *c = Some(comp_silhouette_pixels().to_texture());
    }
    c.clone()
}

fn build_person_pixel(name: &str, female: bool) -> Texture2D {
    person_pixels(name, female).to_texture()
}

#[cfg(test)]
mod pixel_fallback_tests {
    use super::*;

    // 名字种子必须为定值（跨平台确定性口径：FNV-1a 逐 char）
    #[test]
    fn name_seed_fixed() {
        assert_eq!(name_seed("宁清霜"), 4209301141);
        assert_eq!(name_seed("醉道人"), 3873566653);
        assert_eq!(name_seed("玄诚长老"), 987919751);
        assert_eq!(name_seed("胡掌柜"), 2651355892);
    }

    // v1.16.5 法宝像素画：8 张 12×12 图标非空且两两不同（纯 PixGrid 数据，不触 GPU）
    #[test]
    fn relic_pixels_eight_distinct_nonempty() {
        let mut sigs: std::collections::HashSet<Vec<[u8; 4]>> = std::collections::HashSet::new();
        for i in 0..8 {
            let g = relic_pixels(i);
            let n = g.px.iter().filter(|p| p[3] > 0).count();
            assert!(n > 0, "relic {} 空像素图", i);
            assert!(sigs.insert(g.px.clone()), "relic {} 与前件重复", i);
        }
    }

    // 同种子两次构建逐字节相等（纯 PixGrid 数据比较，不触 GPU）
    #[test]
    fn pixel_portrait_deterministic() {
        let a = person_pixels("宁清霜", true);
        let b = person_pixels("宁清霜", true);
        assert_eq!((a.w, a.h), (28, 40));
        assert_eq!(a.px, b.px);
        let c = person_pixels("醉道人", false);
        let d = person_pixels("醉道人", false);
        assert_eq!((c.w, c.h), (28, 40));
        assert_eq!(c.px, d.px);
    }

    // 二人同屏一眼可辨（数据层：像素缓冲不同 + 各自锚点色在像）
    #[test]
    fn anchor_features_present_and_distinct() {
        let senior = person_pixels("宁清霜", true);
        let drunk = person_pixels("醉道人", false);
        assert_ne!(senior.px, drunk.px);
        let frost_n = senior.px.iter().filter(|p| **p == [226, 230, 238, 255]).count();
        assert!(frost_n > 20, "宁清霜霜白长发像素过少: {frost_n}");
        assert!(senior.px.contains(&[64, 100, 96, 255]), "青碧袍缺失");
        assert!(senior.px.contains(&[160, 125, 55, 255]), "暗金剑穗缺失");
        let flush_n = drunk.px.iter().filter(|p| **p == [196, 112, 90, 255]).count();
        assert!(flush_n >= 2, "醉道人酒晕缺失");
        assert!(drunk.px.contains(&[122, 106, 85, 255]), "灰褐袍缺失");
        assert!(drunk.px.contains(&[150, 102, 58, 255]), "酒葫芦缺失");
    }

    // 未知名（未来新 NPC）纯种子也能出像且同名同像——不 hardcode 恰好两人
    #[test]
    fn unknown_name_renders_deterministically() {
        let a = person_pixels("墨衣客", true);
        let b = person_pixels("墨衣客", true);
        assert_eq!(a.px, b.px);
        assert!(a.px.iter().any(|p| p[3] > 0), "未知名出像为空");
    }

    // 书卷剪影：定值同像 + 米白卷面/暗金束带在像
    #[test]
    fn silhouette_deterministic() {
        let a = comp_silhouette_pixels();
        let b = comp_silhouette_pixels();
        assert_eq!((a.w, a.h), (28, 40));
        assert_eq!(a.px, b.px);
        assert!(a.px.contains(&[232, 223, 200, 255]), "米白卷面缺失");
        assert!(a.px.contains(&[180, 148, 88, 255]), "暗金束带缺失");
    }

    // v1.16.23 减包刀①验收：vendor png 0.17.16（miniz_oxide 0.8.9/0.9.1 统一 +
    // flate2 显式 miniz 后端）后的真实解码链验收——image dev-dep 与 macroquad 内部
    // 依赖同一实例，decode 18 张立绘（部署集恰 18，build_web.bat 同口径断言），
    // 断言尺寸>0 且 RGBA 非全零（防「解码成功但全透明」的假绿）。
    #[test]
    fn portraits_png_decode_chain() {
        let d = std::path::Path::new("assets/portraits_wasm");
        let mut names: Vec<_> = std::fs::read_dir(d)
            .expect("portraits_wasm dir missing")
            .filter_map(|e| e.ok().map(|e| e.file_name()))
            .filter(|n| n.to_string_lossy().ends_with(".png"))
            .collect();
        names.sort();
        assert_eq!(names.len(), 18, "portraits_wasm must be exactly 18 pngs");
        for n in &names {
            let bytes = std::fs::read(d.join(n)).expect("read png");
            let img = image::load_from_memory(&bytes)
                .unwrap_or_else(|e| panic!("png decode failed {}: {}", n.to_string_lossy(), e));
            assert!(img.width() > 0 && img.height() > 0, "{} empty dims", n.to_string_lossy());
            let opaque = img.to_rgba8().pixels().filter(|p| p[3] > 0).count();
            assert!(opaque > 0, "{} fully transparent", n.to_string_lossy());
        }
    }
}
