use crate::{get_context, get_quad_context, math::Rect, texture::Image, Color};

use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    pub rect: Rect,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum SpriteKey {
    Texture(miniquad::TextureId),
    Id(u64),
}
pub struct Atlas {
    texture: miniquad::TextureId,
    image: Image,
    pub sprites: HashMap<SpriteKey, Sprite>,
    cursor_x: u16,
    cursor_y: u16,
    max_line_height: u16,

    pub dirty: bool,

    /// PATCH(getfish v1.10.2): 自上次 texture() 上传以来写入的精灵脏区列表。
    /// cache_sprite 每写入一个精灵 push 一条，texture() 按子矩形 texture_update_part
    /// 上传后清空——把「每 dirty 帧 16MiB 整图上传」降为「KB 级子矩形上传」。
    /// reset-in-place/倍增重建分支会整体清空（重建后走整图路径或无 sprite 引用残留）。
    dirty_rects: Vec<Rect>,

    filter: miniquad::FilterMode,

    unique_id: u64,

    /// PATCH(getfish): 该设备上本图集允许的最大边长。wasm 下经 env.gf_max_texture_size
    /// 从 JS 侧读取真实 GL MAX_TEXTURE_SIZE（index.html 探测），native 固定 4096。
    max_dim: u16,
}

impl Drop for Atlas {
    fn drop(&mut self) {
        let ctx = &mut get_context().quad_context;
        ctx.delete_texture(self.texture);
    }
}

impl Atlas {
    // pixel gap between glyphs in the atlas
    const GAP: u16 = 2;
    // well..
    const UNIQUENESS_OFFSET: u64 = 100000;

    pub fn new(ctx: &mut dyn miniquad::RenderingBackend, filter: miniquad::FilterMode) -> Atlas {
        let image = Image::gen_image_color(512, 512, Color::new(0.0, 0.0, 0.0, 0.0));
        let texture = ctx.new_texture_from_rgba8(image.width, image.height, &image.bytes);
        ctx.texture_set_filter(
            texture,
            miniquad::FilterMode::Nearest,
            miniquad::MipmapFilterMode::None,
        );

        Atlas {
            image,
            texture,
            cursor_x: 0,
            cursor_y: 0,
            dirty: false,
            dirty_rects: Vec::new(),
            max_line_height: 0,
            sprites: HashMap::new(),
            filter,
            unique_id: Self::UNIQUENESS_OFFSET,
            max_dim: Self::device_max_dim(),
        }
    }

    /// PATCH(getfish): 设备允许的最大图集边长。
    /// wasm：读取 JS 探测的真实 GL MAX_TEXTURE_SIZE（index.html 的 gf_gpu 插件桥），
    /// 下限防止设备上报异常值导致死循环；上限再钳到 2048（v1.10.2 内存修复）——
    /// 收益：4096 上限的设备（手机主流）长会话图集 CPU 副本+GPU 各 -48MiB（计 -96MiB），
    /// 且每次 dirty 帧整图上传 64MiB→16MiB（-75%，与脏矩形上传叠加后为 KB 级）。
    /// 代价：2048²≈7~9K 个 (字符,字号) 组合，填满后走 reset-in-place（清 sprites +
    /// 重建 16MiB CPU image + 屏上可见字形逐帧重栅格化），预估正常游玩每小时数次
    /// （未实测），reset 帧 G/U 仪表出现尖峰属预期豁免（?dbg=1 验收口径已写明）。
    /// 若 JS 未提供该导入（--import-undefined 桩返回 0），同样落到 512 兜底。
    /// native：同步 2048（桌面本会话实测 2 分钟内图集即从 512² 爬到 2048²，
    /// 封顶一致便于双端行为对齐与验收口径统一）。
    fn device_max_dim() -> u16 {
        const CAP: u16 = 2048;
        #[cfg(target_arch = "wasm32")]
        {
            #[link(wasm_import_module = "env")]
            unsafe extern "C" {
                fn gf_max_texture_size() -> i32;
            }
            let v = unsafe { gf_max_texture_size() };
            if v < 512 {
                512
            } else if v > CAP as i32 {
                CAP
            } else {
                v as u16
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            CAP
        }
    }

    pub fn new_unique_id(&mut self) -> SpriteKey {
        self.unique_id += 1;

        SpriteKey::Id(self.unique_id)
    }

    pub fn set_filter(&mut self, filter_mode: miniquad::FilterMode) {
        let ctx = get_quad_context();
        self.filter = filter_mode;
        ctx.texture_set_filter(self.texture, filter_mode, miniquad::MipmapFilterMode::None);
    }

    pub fn get(&self, key: SpriteKey) -> Option<Sprite> {
        self.sprites.get(&key).cloned()
    }

    pub const fn width(&self) -> u16 {
        self.image.width
    }

    pub const fn height(&self) -> u16 {
        self.image.height
    }

    pub fn texture(&mut self) -> miniquad::TextureId {
        let ctx = get_quad_context();
        if self.dirty {
            self.dirty = false;

            // PATCH(getfish): 图集上传帧计数（性能验收仪表）——v1.10.2 起一次 dirty 帧
            // 只上传脏子矩形（KB 级），本计数语义仍为「发生图集上传的帧数」，
            // ?dbg=1 的 U 逐帧增量口径不变（稳态 0，reset/倍增帧豁免）。
            crate::text::ATLAS_UPLOADS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            let (texture_width, texture_height) = ctx.texture_size(self.texture);
            if texture_width != self.image.width as _ || texture_height != self.image.height as _ {
                // 图集倍增/重建：GPU 纹理尺寸不符，只能整图重建上传（旧路径），
                // 重灌期积累的脏区已被整图覆盖，作废
                ctx.delete_texture(self.texture);

                self.texture = ctx.new_texture_from_rgba8(
                    self.image.width,
                    self.image.height,
                    &self.image.bytes[..],
                );
                ctx.texture_set_filter(self.texture, self.filter, miniquad::MipmapFilterMode::None);
                self.dirty_rects.clear();
            } else {
                // PATCH(getfish v1.10.2): 脏矩形上传——只传本帧写入的子矩形
                //（miniquad texture_update_part → glTexSubImage2D）。收益：面板首开
                // 的新字形滴漏从每帧 16MiB 整图降为几十个字形 × ~1KB，GB 级首开
                // 上传风暴变 MB 级。reset-in-place 帧 GPU 残留像素无 sprite 引用
                //（sprites 已清空、新矩形由后续上传覆盖），无需整图清理。
                let stride = self.image.width as usize * 4;
                for r in &self.dirty_rects {
                    let (rx, ry, rw, rh) =
                        (r.x as usize, r.y as usize, r.w as usize, r.h as usize);
                    let mut buf = Vec::with_capacity(rw * rh * 4);
                    for row in 0..rh {
                        let start = (ry + row) * stride + rx * 4;
                        buf.extend_from_slice(&self.image.bytes[start..start + rw * 4]);
                    }
                    ctx.texture_update_part(
                        self.texture,
                        rx as i32,
                        ry as i32,
                        rw as i32,
                        rh as i32,
                        &buf,
                    );
                }
                self.dirty_rects.clear();
            }
        }

        self.texture
    }

    pub fn get_uv_rect(&self, key: SpriteKey) -> Option<Rect> {
        let ctx = get_quad_context();
        self.get(key).map(|sprite| {
            let (w, h) = ctx.texture_size(self.texture);

            Rect::new(
                sprite.rect.x / w as f32,
                sprite.rect.y / h as f32,
                sprite.rect.w / w as f32,
                sprite.rect.h / h as f32,
            )
        })
    }

    pub fn cache_sprite(&mut self, key: SpriteKey, sprite: Image) {
        let (width, height) = (sprite.width as usize, sprite.height as usize);

        let x = if self.cursor_x + (width as u16) < self.image.width {
            if height as u16 > self.max_line_height {
                self.max_line_height = height as u16;
            }
            let res = self.cursor_x + Self::GAP;
            self.cursor_x += width as u16 + Self::GAP * 2;
            res
        } else {
            self.cursor_y += self.max_line_height + Self::GAP * 2;
            self.cursor_x = width as u16 + Self::GAP;
            self.max_line_height = height as u16;
            Self::GAP
        };
        let y = self.cursor_y;

        // texture bounds exceeded
        if y + sprite.height > self.image.height || x + sprite.width > self.image.width {
            // PATCH(getfish): cap the atlas size. The original code kept
            // doubling the texture (512 -> 1024 -> ... -> 8192 -> ...). On
            // devices whose max texture size is exceeded, creating the bigger
            // texture fails, the previous texture is already deleted, and
            // every glyph afterwards samples an incomplete texture — the whole
            // UI turned into opaque black blocks. Cap the atlas at the device
            // reported max texture size (<= 4096) and reset it in place
            // instead; stale sprite keys are re-rasterized on demand by the
            // safe lookups in text.rs.
            let max_dim = self.max_dim;
            let fits_empty = sprite.width + 2 * Self::GAP <= self.image.width
                && sprite.height <= self.image.height;

            if self.image.width < max_dim {
                // still allowed to grow (may exceed MAX_DIM only for a single
                // gigantic glyph, like the original code did)
                let sprites = self.sprites.drain().collect::<Vec<_>>();
                self.cursor_x = 0;
                self.cursor_y = 0;
                self.max_line_height = 0;
                self.dirty_rects.clear(); // 重建分支：旧脏区随旧 image 一起作废

                let old_image = self.image.clone();

                // increase font texture size
                // note: if we tried to fit gigantic texture into a small atlas,
                // new_width will still be not enough. But its fine, it will
                // be regenerated on the recursion call.
                let new_width = self.image.width * 2;
                let new_height = self.image.height * 2;

                self.image =
                    Image::gen_image_color(new_width, new_height, Color::new(0.0, 0.0, 0.0, 0.0));

                // recache all previously cached symbols
                for (key, sprite) in sprites {
                    let image = old_image.sub_image(sprite.rect);
                    self.cache_sprite(key, image);
                }

                // cache the new sprite
                self.cache_sprite(key, sprite);
            } else if fits_empty {
                // atlas is at its maximum size: reset in place instead of
                // growing past what the GPU supports
                self.sprites.clear();
                self.cursor_x = 0;
                self.cursor_y = 0;
                self.max_line_height = 0;
                self.dirty_rects.clear(); // 重建分支：旧脏区随旧 image 一起作废
                self.image = Image::gen_image_color(
                    self.image.width,
                    self.image.height,
                    Color::new(0.0, 0.0, 0.0, 0.0),
                );
                self.cache_sprite(key, sprite);
            } else {
                // single glyph larger than the whole capped atlas: keep the
                // original doubling behavior as the only way to ever fit it
                let sprites = self.sprites.drain().collect::<Vec<_>>();
                self.cursor_x = 0;
                self.cursor_y = 0;
                self.max_line_height = 0;
                self.dirty_rects.clear(); // 重建分支：旧脏区随旧 image 一起作废

                let old_image = self.image.clone();

                let new_width = self.image.width * 2;
                let new_height = self.image.height * 2;

                self.image =
                    Image::gen_image_color(new_width, new_height, Color::new(0.0, 0.0, 0.0, 0.0));

                for (key, sprite) in sprites {
                    let image = old_image.sub_image(sprite.rect);
                    self.cache_sprite(key, image);
                }

                self.cache_sprite(key, sprite);
            }
        } else {
            self.dirty = true;
            self.dirty_rects.push(Rect::new(
                x as f32,
                y as f32,
                width as f32,
                height as f32,
            ));

            for j in 0..height {
                for i in 0..width {
                    self.image.set_pixel(
                        x as u32 + i as u32,
                        y as u32 + j as u32,
                        sprite.get_pixel(i as u32, j as u32),
                    );
                }
            }

            self.sprites.insert(
                key,
                Sprite {
                    rect: Rect::new(x as f32, y as f32, width as f32, height as f32),
                },
            );
        }
    }
}
