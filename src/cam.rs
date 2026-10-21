// v1.5.0 H5 信箱相机：虚拟坐标恒定 1280×720（V_W/V_H），实际屏幕任意尺寸/比例。
// 渲染：主循环每帧 set_camera(&view_cam()) → 各场景按 V_W/V_H 绝对坐标绘制 → set_default_camera()。
// 输入：命中检测经 mouse() 的 screen_to_world 回到虚拟坐标。
// 桌面 1280×720 下 fit=1、dw=1280、dh=720、display_rect=(0,0,1280,720)，恒等映射，与 v1.4 行为一致。
// 参照 GetFish main.rs:401-421 已验证 build_cam。
// 注：g.shake 为死字段（仅置位/衰减、无绘制消费），相机不掺震屏；
//     未来启用震屏时在 cam.target 上加偏移即可（GetFish shake 分支先例）。
use crate::game::{V_H, V_W};
use macroquad::prelude::*;

pub fn view_cam() -> Camera2D {
    let (sw, sh) = (screen_width().max(1.0), screen_height().max(1.0));
    // 完整可见优先：世界 1280×720 全部显示，多余部分向两侧延展（清除色）
    let fit = (sw / V_W).min(sh / V_H).max(0.001);
    // 可见世界区随屏幕延展（必须为 sw/fit 与 sh/fit！写成 V_W*fit 会让 390×219 的
    // 世界区被拉满 390×844 屏 → 纵向 ~3.85× 非等比畸变，且桌面恒等门禁测不出）
    let dw = sw / fit;
    let dh = sh / fit;
    let mut cam = Camera2D::from_display_rect(Rect::new(
        V_W * 0.5 - dw * 0.5,
        V_H * 0.5 - dh * 0.5,
        dw,
        dh,
    ));
    cam.zoom.y = -cam.zoom.y; // from_display_rect 为 y-up，翻回 macroquad 默认 y-down
    cam
}

// 虚拟坐标鼠标：纯矩阵运算，每帧 ~5 次调用，无需缓存
pub fn mouse() -> (f32, f32) {
    let p = {
        // v1.9.8 触摸优先（仅 wasm）：macroquad 对 touch_event 注入的位置原样
        // 存储、无任何 dpi 参与——与 JS 注入值逐位一致，除以信箱 fit 即精确虚拟
        // 坐标。此前的鼠标通路补偿（×dpi_scale）在真机上不可靠：wasm 侧记录的
        // dpi 是启动瞬间抓取的，X5 实例化后 dpr 才涨到 3.25，native 恒 1.0 而时
        // 钟补偿用实时值 → 虚拟坐标整体 ×3.25（真机 clicks 39 全部打在 4475,139
        // 这类屏幕外坐标）。桌面 IAB 的 dpr 恒 ≈1，两种写法等价——历轮本地测试
        // 测不出的原因。无触摸时（桌面浏览器纯鼠标）退回鼠标通路并按
        // 同源 dpi_scale 乘回（与 macroquad 的除法同源，数学上精确抵消）。
        #[cfg(target_arch = "wasm32")]
        {
            let ts = touches();
            if let Some(t) = ts.last() {
                t.position
            } else {
                let s = macroquad::miniquad::window::dpi_scale();
                vec2(mouse_position().0 * s, mouse_position().1 * s)
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            vec2(mouse_position().0, mouse_position().1)
        }
    };
    let w = view_cam().screen_to_world(p);
    (w.x, w.y)
}
