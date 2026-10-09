// 像素画布与绘图原语：低分辨率网格 → 最近邻放大 = 统一像素风
use macroquad::prelude::*;

pub struct PixGrid {
    pub w: i32,
    pub h: i32,
    pub px: Vec<[u8; 4]>,
}

impl PixGrid {
    pub fn new(w: i32, h: i32) -> Self {
        PixGrid { w, h, px: vec![[0, 0, 0, 0]; (w * h) as usize] }
    }
    pub fn set(&mut self, x: i32, y: i32, c: [u8; 4]) {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return;
        }
        self.px[(y * self.w + x) as usize] = c;
    }
    pub fn get(&self, x: i32, y: i32) -> [u8; 4] {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return [0, 0, 0, 0];
        }
        self.px[(y * self.w + x) as usize]
    }
    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: [u8; 4]) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, c);
            }
        }
    }
    pub fn fill_circle(&mut self, cx: i32, cy: i32, r: i32, c: [u8; 4]) {
        for yy in (cy - r)..=(cy + r) {
            for xx in (cx - r)..=(cx + r) {
                let dx = (xx - cx) as f32;
                let dy = (yy - cy) as f32;
                if dx * dx + dy * dy <= (r as f32 + 0.4) * (r as f32 + 0.4) {
                    self.set(xx, yy, c);
                }
            }
        }
    }
    pub fn fill_ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, c: [u8; 4]) {
        for yy in (cy - ry)..=(cy + ry) {
            for xx in (cx - rx)..=(cx + rx) {
                let dx = (xx - cx) as f32 / (rx as f32 + 0.4);
                let dy = (yy - cy) as f32 / (ry as f32 + 0.4);
                if dx * dx + dy * dy <= 1.0 {
                    self.set(xx, yy, c);
                }
            }
        }
    }
    pub fn tri(&mut self, pts: [(i32, i32); 3], c: [u8; 4]) {
        let minx = pts[0].0.min(pts[1].0).min(pts[2].0);
        let maxx = pts[0].0.max(pts[1].0).max(pts[2].0);
        let miny = pts[0].1.min(pts[1].1).min(pts[2].1);
        let maxy = pts[0].1.max(pts[1].1).max(pts[2].1);
        let (ax, ay) = (pts[0].0 as f32, pts[0].1 as f32);
        let (bx, by) = (pts[1].0 as f32, pts[1].1 as f32);
        let (cx, cy) = (pts[2].0 as f32, pts[2].1 as f32);
        for yy in miny..=maxy {
            for xx in minx..=maxx {
                let (px, py) = (xx as f32 + 0.5, yy as f32 + 0.5);
                let d1 = (bx - ax) * (py - ay) - (by - ay) * (px - ax);
                let d2 = (cx - bx) * (py - by) - (cy - by) * (px - bx);
                let d3 = (ax - cx) * (py - cy) - (ay - cy) * (px - cx);
                let neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
                let pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
                if !(neg && pos) {
                    self.set(xx, yy, c);
                }
            }
        }
    }
    // 4邻接描边：空像素旁有实像素则描边
    pub fn outline(&mut self, c: [u8; 4]) {
        let mut out: Vec<(i32, i32)> = Vec::new();
        for y in 0..self.h {
            for x in 0..self.w {
                if self.get(x, y)[3] == 0 {
                    let n = self.get(x - 1, y)[3] > 0
                        || self.get(x + 1, y)[3] > 0
                        || self.get(x, y - 1)[3] > 0
                        || self.get(x, y + 1)[3] > 0;
                    if n {
                        out.push((x, y));
                    }
                }
            }
        }
        for (x, y) in out {
            self.set(x, y, c);
        }
    }
    pub fn to_texture(&self) -> Texture2D {
        let mut img = Image::gen_image_color(self.w as u16, self.h as u16, BLANK);
        {
            let data = img.get_image_data_mut();
            for y in 0..self.h {
                for x in 0..self.w {
                    data[(y * self.w + x) as usize] = self.px[(y * self.w + x) as usize];
                }
            }
        }
        let tex = Texture2D::from_image(&img);
        tex.set_filter(FilterMode::Nearest);
        tex
    }
}

pub const BLANK: Color = Color::new(0.0, 0.0, 0.0, 0.0);

pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> [u8; 4] {
    [r, g, b, a]
}
