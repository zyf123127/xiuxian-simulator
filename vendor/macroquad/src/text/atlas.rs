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
            max_line_height: 0,
            sprites: HashMap::new(),
            filter,
            unique_id: Self::UNIQUENESS_OFFSET,
            max_dim: Self::device_max_dim(),
        }
    }

    /// PATCH(getfish): 设备允许的最大图集边长。
    /// wasm：读取 JS 探测的真实 GL MAX_TEXTURE_SIZE（index.html 的 gf_gpu 插件桥），
    /// 钳制到 [512, 4096]——上限封顶防止巨纹理分配，下限防止设备上报异常值导致死循环。
    /// 若 JS 未提供该导入（--import-undefined 桩返回 0），同样落到 512 兜底。
    /// native：固定 4096（桌面 GPU 上限普遍 ≥ 8192，足够）。
    fn device_max_dim() -> u16 {
        #[cfg(target_arch = "wasm32")]
        {
            #[link(wasm_import_module = "env")]
            unsafe extern "C" {
                fn gf_max_texture_size() -> i32;
            }
            let v = unsafe { gf_max_texture_size() };
            if v < 512 {
                512
            } else if v > 4096 {
                4096
            } else {
                v as u16
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            4096
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

            // PATCH(getfish): 整图集重上传计数（性能验收仪表）
            crate::text::ATLAS_UPLOADS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            let (texture_width, texture_height) = ctx.texture_size(self.texture);
            if texture_width != self.image.width as _ || texture_height != self.image.height as _ {
                ctx.delete_texture(self.texture);

                self.texture = ctx.new_texture_from_rgba8(
                    self.image.width,
                    self.image.height,
                    &self.image.bytes[..],
                );
                ctx.texture_set_filter(self.texture, self.filter, miniquad::MipmapFilterMode::None);
            }

            ctx.texture_update(self.texture, &self.image.bytes);
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
