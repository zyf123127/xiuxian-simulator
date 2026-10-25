// 字体加载与文本工具（中文 OFL 子集字体）
use macroquad::prelude::*;
use std::sync::OnceLock;

static FONT: OnceLock<Font> = OnceLock::new();

pub fn init_font() {
    let bytes = include_bytes!("../assets/font.ttf");
    let f = load_ttf_font_from_bytes(bytes).expect("字体加载失败");
    let _ = FONT.set(f);
}

pub fn font() -> &'static Font {
    FONT.get().expect("字体未初始化")
}

// v1.6.0 A 清晰度：字号下限。低于 14px 的中文在 150% 物理栅格下发虚/笔画粘连，
// measure/txt 两处统一夹取保证宽度测量与实际绘制一致（右对齐 txt_r 不漂移）。
pub const MIN_FONT: u16 = 14;

pub fn measure(s: &str, size: u16) -> f32 {
    let size = size.max(MIN_FONT);
    let d = measure_text(s, Some(font()), size, 1.0);
    d.width
}

// 以 (x, y) 为文字左上角绘制
// v1.6.0 B 清晰度：最终 x 与基线 y 都取整——txt_c/txt_r/wrap_draw 均经此处输出，
// measure*0.5 求中与 0.82 基线系数常产生小数坐标，亚像素采样导致笔画羽化发虚。
pub fn txt(s: &str, x: f32, y: f32, size: u16, c: Color) {
    let size = size.max(MIN_FONT);
    draw_text_ex(
        s,
        x.floor(),
        (y + size as f32 * 0.82).floor(),
        TextParams {
            font: Some(font()),
            font_size: size,
            color: c,
            ..Default::default()
        },
    );
}

pub fn txt_c(s: &str, cx: f32, y: f32, size: u16, c: Color) {
    txt(s, cx - measure(s, size) * 0.5, y, size, c);
}

pub fn txt_r(s: &str, rx: f32, y: f32, size: u16, c: Color) {
    txt(s, rx - measure(s, size), y, size, c);
}

// v1.9.25 卡顿修复（根因1）：按字号集预热字形——macroquad 字形按（字符,字号）缓存，
// 首见即栅格化入图集并置 dirty，下一个绘制帧整图集重上传（2048²=16MB/次）。
// banner 的 0.6s pop 动画若逐帧首见新字号，等于逐帧整图上传；show_banner 在弹出
// 事件帧调用本函数把全部档位字形一次栅格化，动画帧零缓存 miss。
pub fn prewarm_text(text: &str, sizes: &[u16]) {
    for ch in text.chars() {
        // v1.10.1：不再跳过空白——draw_text_ex 对空格同样走 cache_glyph（0×0 精灵
        // 也会置图集 dirty），跳过会让首条 banner 的 pop 期留 4 次/档 miss
        let cs = ch.to_string();

        for &sz in sizes {
            measure(&cs, sz);
        }
    }
}

// ---------------- v1.10.2 面板语料分帧预热（卡顿修复·帧率根因1 招①）----------------
// 面板首开的迟滞 = 大批新 (字符,字号) 字形栅格化 → 图集 dirty → 之后每帧重传
// （v1.10.2 脏矩形上传后为 KB 级/帧）。本节在菜单场景把高频面板语料按字号档
// 分帧灌进字形缓存（每帧 ~40 字形，1 秒量级完成），进游戏后各面板首开 ≈ 温开
// （实测温开 0 上传/0 longtask）。字号档与字号收敛口径一致（UI_SIZES）；
// 字符集 = 菜单/主 UI/人物面板/山河图静态文案的高频 top300 + 动态文案数字与符号
//（python 按频次生成，注释口径见 v1.10.2 提交）。ASCII 数字/符号是 format!
// 动态行（好感 85/100、×1.20）的首开 miss 大头，保底置顶。
pub const UI_SIZES: [u16; 6] = [14, 16, 18, 20, 24, 28];

pub const PANEL_WARM_CHARS: &[char] = &[
    '，', '·', '—', '。', '：', '（', '）', '+', '1', '/', '%', '0', '×', '2', '-', '5', '3', '6', '8',
    '4', '7', '9', '『', '』', '{', '}', '灵', '修', '法', '境', '回', '点', '模', '拟', '宝', '道', '一', '可',
    '石', '已', '此', '敌', '丹', '.', '轮', '世', '无', '为', '前', '装', '【', '】', '炼', '中', '得', '献', '宗',
    '数', '化', ':', '你', '、', '实', '载', '生', '随', '门', '贡', '现', '未', '韵', '身', '宿', '药', '不', '在',
    '归', '开', '行', '选', '当', '；', '关', '元', '下', '至', '缘', '烙', '印', '远', '征', '破', '寿', '图', '镜',
    '界', '收', '死', '看', '返', '效', '历', '山', '寻', '落', '「', '」', '与', '秘', '成', '闭', '情', '于', '地',
    '人', '演', '造', '始', '的', '高', '斩', '所', '换', '仇', '天', '气', '年', '功', '大', '再', '…', '服', '备',
    '青', '云', '结', '域', '最', '坊', '市', '经', '兑', '侣', '自', '动', '机', '入', '劫', '项', '途', '件', '有',
    '洞', '府', '步', '击', '霞', '突', '区', '长', '音', '杀', '页', '水', '藏', '第', '战', '力', '重', '每', '亦',
    '事', '阁', '价', '时', '完', '存', '购', '用', '卸', '满', '级', '赠', '礼', '任', '务', '手', '获', '往', '查',
    '取', '消', '尽', '推', '河', '商', '键', '清', '根', '心', '伤', '包', '内', '神', '来', '起', '后', '涨', '作',
    '性', '子', '次', '强', '升', '就', '谊', '拜', '赶', '路', '详', '方', 'a', 's', '鉴', '初', '仙', '新', '接',
    '帮', '助', '穷', 'M', '囊', '物', '角', '色', '静', '体', '魄', '！', '赋', '雷', '暂', '享', '还', '带', '总',
    '保', '引', '幻', '跨', '久', '渡', '越', '确', '认', '材', '料', '祭', '尚', '见', '签', '弟', '派', '称', '号',
    '圆', '悟', '永', '泽', '操', '更', '之', '或', '算', '请', '率', '槽', '佩', '戴', '殒', '需', '折', '险', '苦',
    '留', '流', '承', '继', '微', '了', '乐', '主', '观', 'C', '待', '运', '醒', '仅', '梦',
];

static WARM_CURSOR: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// 每帧消费 budget 个 (字符,字号) 组合（经 measure 触发栅格化入图集，同 prewarm_text
/// 口径）。队列耗尽后为 no-op（原子快速路径）；仅在菜单场景驱动（main.rs）。
pub fn panel_warm_step(budget: usize) {
    let cur = WARM_CURSOR.load(std::sync::atomic::Ordering::Relaxed);
    let total = PANEL_WARM_CHARS.len() * UI_SIZES.len();
    if cur >= total {
        return;
    }
    let end = (cur + budget).min(total);
    for k in cur..end {
        let ch = PANEL_WARM_CHARS[k / UI_SIZES.len()];
        let sz = UI_SIZES[k % UI_SIZES.len()];
        let cs = ch.to_string();
        measure(&cs, sz);
    }
    WARM_CURSOR.store(end, std::sync::atomic::Ordering::Relaxed);
}

// ---------------- 换行记忆化（v1.9.25 根因2：wrap_draw 每帧逐字符 measure+to_string
// 是模态卡/天赋卡/结算页的稳定分配底噪；命中路径零分配，上限防膨胀整体清空） ----------------

struct WrapKey {
    text: String,
    size: u16,
    maxw_bits: u32,
}

std::thread_local! {
    static WRAP_CACHE: std::cell::RefCell<Vec<(WrapKey, Vec<String>)>> =
        std::cell::RefCell::new(Vec::new());
}

const WRAP_CACHE_CAP: usize = 256;

// 命中返回缓存槽位；未命中计算并入缓存（本帧一次分配）
fn wrap_slot(
    s: &str,
    size: u16,
    maxw: f32,
    cache: &mut Vec<(WrapKey, Vec<String>)>,
) -> usize {
    let bits = maxw.to_bits();

    if let Some(i) = cache
        .iter()
        .position(|(k, _)| k.size == size && k.maxw_bits == bits && k.text.as_str() == s)
    {
        return i;
    }

    if cache.len() >= WRAP_CACHE_CAP {
        cache.clear();
    }

    cache.push((
        WrapKey {
            text: s.to_string(),
            size,
            maxw_bits: bits,
        },
        wrap_uncached(s, size, maxw),
    ));

    cache.len() - 1
}

// 贪心换行：中文逐字断行，优先在空格/标点处断
fn wrap_uncached(s: &str, size: u16, maxw: f32) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut cur_w = 0.0;
    for ch in s.chars() {
        let cs = ch.to_string();
        let w = measure(&cs, size);
        if cur_w + w > maxw && !cur.is_empty() {
            // 尝试在行尾标点前回收
            lines.push(cur.clone());
            cur.clear();
            cur_w = 0.0;
        }
        cur.push_str(&cs);
        cur_w += w;
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

pub fn wrap_draw(s: &str, x: f32, y: f32, size: u16, maxw: f32, line_h: f32, c: Color) -> f32 {
    WRAP_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();

        let i = wrap_slot(s, size, maxw, &mut cache);

        let lines = &cache[i].1;

        for (i, l) in lines.iter().enumerate() {
            txt(l, x, y + i as f32 * line_h, size, c);
        }

        lines.len() as f32 * line_h
    })
}
