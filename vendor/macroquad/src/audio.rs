//! Loading and playing sounds.

#![allow(dead_code)]

use crate::{file::load_file, get_context, Error};
use std::sync::Arc;

#[cfg(feature = "audio")]
use quad_snd::{AudioContext as QuadSndContext, Sound as QuadSndSound};

#[cfg(feature = "audio")]
pub use quad_snd::PlaySoundParams;

#[cfg(not(feature = "audio"))]
mod dummy_audio {
    use crate::audio::PlaySoundParams;

    pub struct AudioContext {}

    impl AudioContext {
        pub fn new() -> AudioContext {
            AudioContext {}
        }

        #[cfg(target_os = "android")]
        pub fn pause(&mut self) {}

        #[cfg(target_os = "android")]
        pub fn resume(&mut self) {}
    }

    pub struct Sound {}

    impl Sound {
        pub fn load(_ctx: &mut AudioContext, _data: &[u8]) -> Sound {
            Sound {}
        }

        pub fn play(&self, _ctx: &mut AudioContext, _params: PlaySoundParams) {
            eprintln!("warn: macroquad's \"audio\" feature disabled.");
        }

        pub fn stop(&self, _ctx: &mut AudioContext) {}

        pub fn set_volume(&self, _ctx: &mut AudioContext, _volume: f32) {}

        #[allow(dead_code)]
        pub fn is_loaded(&self) -> bool {
            true
        }

        pub fn delete(&self, _ctx: &AudioContext) {}
    }
}

#[cfg(not(feature = "audio"))]
use dummy_audio::{AudioContext as QuadSndContext, Sound as QuadSndSound};

#[cfg(not(feature = "audio"))]
pub struct PlaySoundParams {
    pub looped: bool,
    pub volume: f32,
}

pub struct AudioContext {
    native_ctx: QuadSndContext,
}

impl AudioContext {
    pub fn new() -> AudioContext {
        AudioContext {
            native_ctx: QuadSndContext::new(),
        }
    }

    #[cfg(target_os = "android")]
    pub fn pause(&mut self) {
        self.native_ctx.pause()
    }

    #[cfg(target_os = "android")]
    pub fn resume(&mut self) {
        self.native_ctx.resume()
    }
}

struct QuadSndSoundGuarded(QuadSndSound);

impl Drop for QuadSndSoundGuarded {
    fn drop(&mut self) {
        let ctx = &get_context().audio_context;
        self.0.delete(&ctx.native_ctx);
    }
}

#[derive(Clone)]
pub struct Sound(Arc<QuadSndSoundGuarded>);

impl std::fmt::Debug for Sound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sound").finish()
    }
}

/// Load audio file.
///
/// Attempts to automatically detect the format of the source of data.

pub async fn load_sound(path: &str) -> Result<Sound, Error> {
    let data = load_file(path).await?;

    load_sound_from_bytes(&data).await
}

/// Load audio data.
///
/// Attempts to automatically detect the format of the source of data.
pub async fn load_sound_from_bytes(data: &[u8]) -> Result<Sound, Error> {
    load_sound_from_bytes_bounded(data, f64::INFINITY).await
}

/// PATCH(getfish) v1.15.2 有界音频加载：wasm 上 decodeAudioData 在 suspended
/// AudioContext（X5/TapTap 容器静音页实测）可能永不完成，原实现的
/// `while !is_loaded { next_frame().await }` 会永续空转——协程永不退场，
/// 每帧白耗 2 次 FrameFuture poll，且该 sound 永不可用。max_wait 为墙钟秒
/// （get_time 走 sys_now，帧率被钳时也保真），到点即放弃返回 Err，调用方
/// 可静默降级（无音效可玩）。
pub async fn load_sound_from_bytes_bounded(data: &[u8], max_wait: f64) -> Result<Sound, Error> {
    let sound = {
        let ctx = &mut get_context().audio_context;
        QuadSndSound::load(&mut ctx.native_ctx, data)
    };

    // only on wasm the sound is not ready right away
    #[cfg(target_arch = "wasm32")]
    {
        let t0 = crate::time::get_time();

        while sound.is_loaded() == false {
            if crate::time::get_time() - t0 > max_wait {
                return Err(Error::UnknownError("audio decode timeout"));
            }

            crate::window::next_frame().await;
        }
    }

    Ok(Sound(Arc::new(QuadSndSoundGuarded(sound))))
}

pub fn play_sound_once(sound: &Sound) {
    let ctx = &mut get_context().audio_context;

    sound.0 .0.play(
        &mut ctx.native_ctx,
        PlaySoundParams {
            looped: false,
            volume: 1.0,
        },
    );
}

pub fn play_sound(sound: &Sound, params: PlaySoundParams) {
    let ctx = &mut get_context().audio_context;
    sound.0 .0.play(&mut ctx.native_ctx, params);
}

pub fn stop_sound(sound: &Sound) {
    let ctx = &mut get_context().audio_context;
    sound.0 .0.stop(&mut ctx.native_ctx);
}

pub fn set_sound_volume(sound: &Sound, volume: f32) {
    let ctx = &mut get_context().audio_context;
    sound.0 .0.set_volume(&mut ctx.native_ctx, volume);
}
