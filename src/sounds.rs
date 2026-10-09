// 程序化音频：内存合成 WAV，古筝风五声音阶 BGM + 音效
use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};

pub const SR: u32 = 22050;

#[derive(Clone, Copy, PartialEq)]
pub enum Which {
    Click,
    Gain,
    Ding,
    BreakOk,
    BreakFail,
    Death,
    Thunder,
    Ascend,
    Coin,
    Page,
}

pub struct Sounds {
    pub bgm: Sound,
    pub battle: Sound,
    pub click: Sound,
    pub gain: Sound,
    pub ding: Sound,
    pub break_ok: Sound,
    pub break_fail: Sound,
    pub death: Sound,
    pub thunder: Sound,
    pub ascend: Sound,
    pub coin: Sound,
    pub page: Sound,
}

pub struct Audio {
    pub s: Option<Sounds>,
    pub music_on: bool,
    pub sfx_on: bool,
    pub music_vol: f32,
    pub sfx_vol: f32,
}

// ---------------- WAV 合成 ----------------
fn wav_bytes(samples: &[f32]) -> Vec<u8> {
    let n = samples.len() as u32;
    let data_len = n * 2;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&SR.to_le_bytes());
    out.extend_from_slice(&(SR * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

fn mix_at(dst: &mut Vec<f32>, src: &[f32], at: f32) {
    let start = (at * SR as f32) as usize;
    for (i, v) in src.iter().enumerate() {
        let idx = start + i;
        if idx < dst.len() {
            dst[idx] += v;
        }
    }
}

// 衰减正弦 + 二次谐波（拨弦感）
fn pluck(freq: f32, dur: f32, vol: f32) -> Vec<f32> {
    let n = (dur * SR as f32) as usize;
    let mut out = vec![0.0f32; n];
    for i in 0..n {
        let t = i as f32 / SR as f32;
        let env = (-t * 5.0).exp();
        out[i] = vol
            * env
            * (0.72 * (2.0 * std::f32::consts::PI * freq * t).sin()
                + 0.22 * (2.0 * std::f32::consts::PI * freq * 2.0 * t).sin()
                + 0.08 * (2.0 * std::f32::consts::PI * freq * 3.0 * t).sin());
    }
    out
}

fn sweep(f0: f32, f1: f32, dur: f32, vol: f32) -> Vec<f32> {
    let n = (dur * SR as f32) as usize;
    let mut out = vec![0.0f32; n];
    let mut ph = 0.0f32;
    for i in 0..n {
        let t = i as f32 / n as f32;
        let f = f0 + (f1 - f0) * t;
        ph += 2.0 * std::f32::consts::PI * f / SR as f32;
        let env = (1.0 - t).powf(1.6);
        out[i] = vol * env * ph.sin();
    }
    out
}

fn noise_burst(dur: f32, vol: f32, lp: usize) -> Vec<f32> {
    let n = (dur * SR as f32) as usize;
    let mut raw = vec![0.0f32; n];
    let mut seed = 7u32;
    for i in 0..n {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        raw[i] = (seed >> 9) as f32 / 4194304.0 - 1.0;
    }
    // 简易低通（滑动平均）
    let mut out = vec![0.0f32; n];
    let mut acc = 0.0f32;
    for i in 0..n {
        acc += raw[i];
        if i >= lp {
            acc -= raw[i - lp];
        }
        let t = i as f32 / n as f32;
        out[i] = (acc / lp.min(i + 1) as f32) * vol * (1.0 - t).powf(1.3);
    }
    out
}

// ---------------- BGM：五声音阶慢拨，16 秒循环 ----------------
fn build_bgm() -> Vec<u8> {
    let dur = 16.0;
    let mut buf = vec![0.0f32; (dur * SR as f32) as usize];
    // A 商/羽调式五声
    let a3 = 220.0;
    let notes = [a3, 261.63, 293.66, 329.63, 392.0, 440.0, 523.25];
    // 旋律表（索引，9=休止）
    let mel: [usize; 26] = [
        5, 4, 2, 3, 4, 9, 5, 4, 2, 0, 1, 9, 4, 5, 6, 5, 4, 9, 2, 3, 4, 2, 1, 0, 9, 9,
    ];
    let step = 0.62;
    let mut t = 0.15;
    for &mi in mel.iter() {
        if mi != 9 {
            let f = notes[mi];
            mix_at(&mut buf, &pluck(f, 1.9, 0.16), t);
            if mi <= 3 {
                mix_at(&mut buf, &pluck(f * 2.0, 1.4, 0.05), t + 0.31);
            }
            if mi >= 4 {
                mix_at(&mut buf, &pluck(f * 0.5, 2.2, 0.06), t);
            }
        }
        t += step;
        if t > dur - 1.5 {
            break;
        }
    }
    // 低音持续音
    for i in 0..buf.len() {
        let tt = i as f32 / SR as f32;
        buf[i] += 0.028
            * (2.0 * std::f32::consts::PI * a3 * 0.5 * tt).sin()
            * (0.75 + 0.25 * (2.0 * std::f32::consts::PI * 0.125 * tt).sin());
    }
    // 首尾淡入淡出避免爆音
    let fade = (0.4 * SR as f32) as usize;
    let n = buf.len();
    for i in 0..fade {
        let k = i as f32 / fade as f32;
        buf[i] *= k;
        buf[n - 1 - i] *= k;
    }
    wav_bytes(&buf)
}

// 战斗 BGM：低音鼓点 + 急促五声音阶，8 秒循环
fn build_battle_bgm() -> Vec<u8> {
    let dur = 8.0;
    let mut buf = vec![0.0f32; (dur * SR as f32) as usize];
    let notes = [220.0f32, 261.63, 293.66, 329.63, 392.0, 440.0];
    let mel: [usize; 14] = [5, 3, 4, 2, 4, 1, 5, 3, 4, 0, 3, 2, 4, 5];
    let step = 0.28;
    let mut t = 0.05;
    for &mi in mel.iter() {
        mix_at(&mut buf, &pluck(notes[mi], 0.5, 0.15), t);
        t += step;
        if t > dur - 0.6 {
            break;
        }
    }
    // 鼓点（低频正弦快速衰减，每 0.5s）
    let mut bt = 0.0f32;
    while bt < dur - 0.2 {
        let n = (0.16 * SR as f32) as usize;
        for i in 0..n {
            let tt = i as f32 / SR as f32;
            let env = (-tt * 26.0).exp();
            let idx = (bt * SR as f32) as usize + i;
            if idx < buf.len() {
                buf[idx] += 0.30 * env * (2.0 * std::f32::consts::PI * 72.0 * tt).sin();
            }
        }
        bt += 0.5;
    }
    // 低音持续（A2 与降 B2 交替，紧张感）
    for i in 0..buf.len() {
        let tt = i as f32 / SR as f32;
        let f = if ((tt / 2.0) as i32) % 2 == 0 {
            110.0
        } else {
            116.5
        };
        buf[i] += 0.05 * (2.0 * std::f32::consts::PI * f * tt).sin();
    }
    let fade = (0.25 * SR as f32) as usize;
    let n = buf.len();
    for i in 0..fade {
        let k = i as f32 / fade as f32;
        buf[i] *= k;
        buf[n - 1 - i] *= k;
    }
    wav_bytes(&buf)
}

fn build_sfx() -> Vec<Vec<u8>> {
    let mut click = vec![0.0f32; (0.12 * SR as f32) as usize];
    mix_at(&mut click, &pluck(880.0, 0.12, 0.30), 0.0);
    let mut gain = vec![0.0f32; (0.25 * SR as f32) as usize];
    mix_at(&mut gain, &pluck(660.0, 0.25, 0.26), 0.0);
    mix_at(&mut gain, &pluck(990.0, 0.18, 0.10), 0.04);
    let mut ding = vec![0.0f32; (0.6 * SR as f32) as usize];
    mix_at(&mut ding, &pluck(1318.5, 0.55, 0.24), 0.0);
    mix_at(&mut ding, &pluck(1975.5, 0.4, 0.12), 0.05);
    let mut ok = vec![0.0f32; (1.2 * SR as f32) as usize];
    let arp = [440.0, 523.25, 659.25, 880.0];
    for (i, f) in arp.iter().enumerate() {
        mix_at(&mut ok, &pluck(*f, 0.8, 0.22), i as f32 * 0.11);
    }
    mix_at(&mut ok, &pluck(1760.0, 0.7, 0.08), 0.5);
    let mut fail = vec![0.0f32; (0.7 * SR as f32) as usize];
    mix_at(&mut fail, &sweep(220.0, 70.0, 0.6, 0.30), 0.0);
    mix_at(&mut fail, &noise_burst(0.2, 0.12, 90), 0.0);
    let mut death = vec![0.0f32; (2.2 * SR as f32) as usize];
    mix_at(&mut death, &sweep(160.0, 52.0, 1.8, 0.30), 0.0);
    mix_at(&mut death, &pluck(108.0, 2.0, 0.22), 0.0);
    mix_at(&mut death, &pluck(163.0, 1.6, 0.10), 0.02);
    mix_at(&mut death, &noise_burst(0.5, 0.10, 60), 0.0);
    let mut thunder = vec![0.0f32; (1.0 * SR as f32) as usize];
    mix_at(&mut thunder, &noise_burst(0.9, 0.55, 40), 0.0);
    mix_at(&mut thunder, &sweep(90.0, 38.0, 0.8, 0.25), 0.05);
    let mut ascend = vec![0.0f32; (2.4 * SR as f32) as usize];
    let up = [440.0, 523.25, 659.25, 784.0, 880.0, 1046.5, 1318.5, 1568.0];
    for (i, f) in up.iter().enumerate() {
        mix_at(&mut ascend, &pluck(*f, 1.3, 0.16), i as f32 * 0.13);
    }
    mix_at(&mut ascend, &pluck(2093.0, 1.6, 0.07), 1.2);
    let mut coin = vec![0.0f32; (0.25 * SR as f32) as usize];
    mix_at(&mut coin, &pluck(1568.0, 0.1, 0.22), 0.0);
    mix_at(&mut coin, &pluck(2093.0, 0.2, 0.20), 0.07);
    let mut page = vec![0.0f32; (0.15 * SR as f32) as usize];
    mix_at(&mut page, &noise_burst(0.12, 0.10, 30), 0.0);
    vec![
        wav_bytes(&click),
        wav_bytes(&gain),
        wav_bytes(&ding),
        wav_bytes(&ok),
        wav_bytes(&fail),
        wav_bytes(&death),
        wav_bytes(&thunder),
        wav_bytes(&ascend),
        wav_bytes(&coin),
        wav_bytes(&page),
    ]
}

pub async fn init() -> Audio {
    let v = build_sfx();
    let sounds = Sounds {
        bgm: load_sound_from_bytes(&build_bgm()).await.unwrap(),
        battle: load_sound_from_bytes(&build_battle_bgm()).await.unwrap(),
        click: load_sound_from_bytes(&v[0]).await.unwrap(),
        gain: load_sound_from_bytes(&v[1]).await.unwrap(),
        ding: load_sound_from_bytes(&v[2]).await.unwrap(),
        break_ok: load_sound_from_bytes(&v[3]).await.unwrap(),
        break_fail: load_sound_from_bytes(&v[4]).await.unwrap(),
        death: load_sound_from_bytes(&v[5]).await.unwrap(),
        thunder: load_sound_from_bytes(&v[6]).await.unwrap(),
        ascend: load_sound_from_bytes(&v[7]).await.unwrap(),
        coin: load_sound_from_bytes(&v[8]).await.unwrap(),
        page: load_sound_from_bytes(&v[9]).await.unwrap(),
    };
    Audio {
        s: Some(sounds),
        music_on: true,
        sfx_on: true,
        music_vol: 0.5,
        sfx_vol: 0.9,
    }
}

impl Audio {
    pub fn start_bgm(&self) {
        if let Some(s) = &self.s {
            play_sound(
                &s.bgm,
                PlaySoundParams {
                    looped: true,
                    volume: self.music_vol,
                },
            );
        }
    }
    pub fn stop_bgm(&self) {
        if let Some(s) = &self.s {
            stop_sound(&s.bgm);
            stop_sound(&s.battle);
        }
    }
    // 切换常态/战斗 BGM
    pub fn swap_bgm(&self, battle: bool) {
        if !self.music_on {
            return;
        }
        if let Some(s) = &self.s {
            stop_sound(&s.bgm);
            stop_sound(&s.battle);
            if battle {
                play_sound(
                    &s.battle,
                    PlaySoundParams {
                        looped: true,
                        volume: self.music_vol,
                    },
                );
            } else {
                play_sound(
                    &s.bgm,
                    PlaySoundParams {
                        looped: true,
                        volume: self.music_vol,
                    },
                );
            }
        }
    }
    pub fn play(&self, w: Which) {
        if !self.sfx_on {
            return;
        }
        if let Some(s) = &self.s {
            let snd = match w {
                Which::Click => &s.click,
                Which::Gain => &s.gain,
                Which::Ding => &s.ding,
                Which::BreakOk => &s.break_ok,
                Which::BreakFail => &s.break_fail,
                Which::Death => &s.death,
                Which::Thunder => &s.thunder,
                Which::Ascend => &s.ascend,
                Which::Coin => &s.coin,
                Which::Page => &s.page,
            };
            play_sound(
                snd,
                PlaySoundParams {
                    looped: false,
                    volume: self.sfx_vol,
                },
            );
        }
    }
}
