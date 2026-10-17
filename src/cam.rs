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
    // v1.7.0 wasm high_dpi 坐标系对齐：screen_width() 返回 backing 像素（CSS×dpr），
    // 而 macroquad mouse_position() 内部已除以 dpi_scale 返回 CSS 像素——先乘回再换算
    // （桌面端 screen_width 与 mouse_position 同为逻辑像素，恒等不乘）
    #[cfg(target_arch = "wasm32")]
    let s = macroquad::miniquad::window::dpi_scale();
    #[cfg(not(target_arch = "wasm32"))]
    let s = 1.0;
    let p = vec2(mouse_position().0 * s, mouse_position().1 * s);
    let w = view_cam().screen_to_world(p);
    (w.x, w.y)
}
