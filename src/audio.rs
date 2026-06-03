use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::f32::consts::TAU;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sound {
    Hit,
    Crit,
    Kill,
    Hurt,
    Gold,
    Item,
    Quaff,
    LevelUp,
    Talent,
    Descend,
    Bolt,
    Scroll,
    BossWarn,
    BossHit,
    Death,
    Trade,
    Heartbeat,
}

impl Sound {
    #[allow(dead_code)]
    pub const ALL: &'static [(&'static str, Sound)] = &[
        ("hit", Sound::Hit),
        ("crit", Sound::Crit),
        ("kill", Sound::Kill),
        ("hurt", Sound::Hurt),
        ("gold", Sound::Gold),
        ("item", Sound::Item),
        ("quaff", Sound::Quaff),
        ("levelup", Sound::LevelUp),
        ("talent", Sound::Talent),
        ("descend", Sound::Descend),
        ("bolt", Sound::Bolt),
        ("scroll", Sound::Scroll),
        ("bosswarn", Sound::BossWarn),
        ("bosshit", Sound::BossHit),
        ("death", Sound::Death),
        ("trade", Sound::Trade),
        ("heartbeat", Sound::Heartbeat),
    ];
}

const SR: u32 = 44100;

#[derive(Clone, Copy)]
enum Wave {
    Square,
    Tri,
    Sine,
    Noise,
}

struct Synth {
    noise: u32,
}

impl Synth {
    fn new() -> Self {
        Synth { noise: 0x1234_5678 }
    }

    fn rand(&mut self) -> f32 {
        self.noise ^= self.noise << 13;
        self.noise ^= self.noise >> 17;
        self.noise ^= self.noise << 5;
        (self.noise as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    fn tone(&mut self, buf: &mut Vec<f32>, freq: f32, dur: f32, vol: f32, wave: Wave, duty: f32) {
        let n = (SR as f32 * dur) as usize;
        let atk = (n as f32 * 0.04).max(1.0);
        for i in 0..n {
            let t = i as f32 / SR as f32;
            let ph = (freq * t).fract();
            let raw = match wave {
                Wave::Square => {
                    if ph < duty {
                        1.0
                    } else {
                        -1.0
                    }
                }
                Wave::Tri => 4.0 * (ph - 0.5).abs() - 1.0,
                Wave::Sine => (ph * TAU).sin(),
                Wave::Noise => self.rand(),
            };
            let env = if (i as f32) < atk {
                i as f32 / atk
            } else {
                let p = (i as f32 - atk) / (n as f32 - atk).max(1.0);
                (1.0 - p).powf(2.2)
            };
            buf.push(raw * env * vol);
        }
    }

    fn sweep(&mut self, buf: &mut Vec<f32>, f0: f32, f1: f32, dur: f32, vol: f32, wave: Wave) {
        let n = (SR as f32 * dur) as usize;
        let mut phase = 0.0f32;
        for i in 0..n {
            let p = i as f32 / n as f32;
            let freq = f0 + (f1 - f0) * p;
            phase = (phase + freq / SR as f32).fract();
            let raw = match wave {
                Wave::Square => {
                    if phase < 0.5 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                Wave::Tri => 4.0 * (phase - 0.5).abs() - 1.0,
                Wave::Sine => (phase * TAU).sin(),
                Wave::Noise => self.rand(),
            };
            let env = (1.0 - p).powf(1.4);
            buf.push(raw * env * vol);
        }
    }

    fn silence(&self, buf: &mut Vec<f32>, dur: f32) {
        let n = (SR as f32 * dur) as usize;
        for _ in 0..n {
            buf.push(0.0);
        }
    }
}

fn render(sound: Sound) -> Vec<f32> {
    let mut s = Synth::new();
    let mut b: Vec<f32> = Vec::new();
    match sound {
        Sound::Hit => s.tone(&mut b, 220.0, 0.05, 0.32, Wave::Square, 0.5),
        Sound::Crit => {
            s.tone(&mut b, 660.0, 0.04, 0.34, Wave::Square, 0.5);
            s.tone(&mut b, 990.0, 0.06, 0.34, Wave::Square, 0.25);
        }
        Sound::Kill => {
            s.tone(&mut b, 392.0, 0.04, 0.3, Wave::Square, 0.5);
            s.tone(&mut b, 523.0, 0.04, 0.3, Wave::Square, 0.5);
            s.tone(&mut b, 784.0, 0.07, 0.3, Wave::Square, 0.5);
        }
        Sound::Hurt => {
            s.tone(&mut b, 140.0, 0.05, 0.3, Wave::Noise, 0.5);
            s.tone(&mut b, 98.0, 0.09, 0.32, Wave::Square, 0.5);
        }
        Sound::Gold => {
            s.tone(&mut b, 988.0, 0.05, 0.28, Wave::Tri, 0.5);
            s.tone(&mut b, 1319.0, 0.09, 0.28, Wave::Tri, 0.5);
        }
        Sound::Item => {
            s.tone(&mut b, 523.0, 0.05, 0.28, Wave::Tri, 0.5);
            s.tone(&mut b, 698.0, 0.05, 0.28, Wave::Tri, 0.5);
            s.tone(&mut b, 880.0, 0.08, 0.28, Wave::Tri, 0.5);
        }
        Sound::Quaff => s.sweep(&mut b, 300.0, 760.0, 0.16, 0.26, Wave::Sine),
        Sound::LevelUp => {
            for f in [523.0, 659.0, 784.0, 1047.0] {
                s.tone(&mut b, f, 0.08, 0.3, Wave::Square, 0.5);
            }
            s.tone(&mut b, 1319.0, 0.16, 0.3, Wave::Square, 0.25);
        }
        Sound::Talent => {
            for f in [440.0, 587.0, 740.0, 880.0] {
                s.tone(&mut b, f, 0.07, 0.28, Wave::Tri, 0.5);
            }
        }
        Sound::Descend => {
            s.sweep(&mut b, 440.0, 110.0, 0.28, 0.3, Wave::Square);
            s.tone(&mut b, 82.0, 0.12, 0.3, Wave::Square, 0.5);
        }
        Sound::Bolt => s.sweep(&mut b, 1200.0, 300.0, 0.14, 0.26, Wave::Square),
        Sound::Scroll => {
            s.sweep(&mut b, 600.0, 1400.0, 0.1, 0.22, Wave::Sine);
            s.tone(&mut b, 1400.0, 0.05, 0.18, Wave::Noise, 0.5);
        }
        Sound::BossWarn => {
            s.tone(&mut b, 73.0, 0.12, 0.34, Wave::Square, 0.5);
            s.tone(&mut b, 78.0, 0.16, 0.34, Wave::Square, 0.5);
        }
        Sound::BossHit => {
            s.tone(&mut b, 110.0, 0.05, 0.34, Wave::Noise, 0.5);
            s.tone(&mut b, 65.0, 0.12, 0.36, Wave::Square, 0.5);
        }
        Sound::Death => {
            for f in [330.0, 277.0, 233.0, 175.0] {
                s.tone(&mut b, f, 0.16, 0.32, Wave::Square, 0.5);
            }
            s.tone(&mut b, 110.0, 0.5, 0.32, Wave::Tri, 0.5);
        }
        Sound::Trade => {
            s.tone(&mut b, 784.0, 0.05, 0.26, Wave::Tri, 0.5);
            s.silence(&mut b, 0.02);
            s.tone(&mut b, 1047.0, 0.08, 0.26, Wave::Tri, 0.5);
        }
        Sound::Heartbeat => {
            s.tone(&mut b, 60.0, 0.1, 0.22, Wave::Sine, 0.5);
            s.silence(&mut b, 0.05);
            s.tone(&mut b, 46.0, 0.13, 0.16, Wave::Sine, 0.5);
        }
    }
    b
}

fn hz(midi: i32) -> f32 {
    440.0 * 2.0f32.powf((midi as f32 - 69.0) / 12.0)
}

fn add_voice(buf: &mut [f32], start: usize, freq: f32, dur: f32, vol: f32, wave: Wave, attack: f32, release: f32) {
    let n = (SR as f32 * dur) as usize;
    let atk = (SR as f32 * attack).max(1.0);
    let rel = (SR as f32 * release).max(1.0);
    for i in 0..n {
        let idx = start + i;
        if idx >= buf.len() {
            break;
        }
        let t = i as f32 / SR as f32;
        let ph = (freq * t).fract();
        let raw = match wave {
            Wave::Square => {
                if ph < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Wave::Tri => 4.0 * (ph - 0.5).abs() - 1.0,
            Wave::Sine => (ph * TAU).sin(),
            Wave::Noise => 0.0,
        };
        let env = if (i as f32) < atk {
            i as f32 / atk
        } else if i as f32 > n as f32 - rel {
            ((n - i) as f32 / rel).max(0.0)
        } else {
            1.0
        };
        buf[idx] += raw * env * vol;
    }
}

fn add_kick(buf: &mut [f32], start: usize, vol: f32) {
    let n = (SR as f32 * 0.2) as usize;
    let mut phase = 0.0f32;
    for i in 0..n {
        let idx = start + i;
        if idx >= buf.len() {
            break;
        }
        let p = i as f32 / n as f32;
        let freq = 50.0 + 90.0 * (1.0 - p).powf(3.0);
        phase = (phase + freq / SR as f32).fract();
        let env = (1.0 - p).powf(2.0);
        buf[idx] += (phase * TAU).sin() * env * vol;
    }
}

fn add_snare(buf: &mut [f32], start: usize, vol: f32, seed: &mut u32) {
    let n = (SR as f32 * 0.16) as usize;
    for i in 0..n {
        let idx = start + i;
        if idx >= buf.len() {
            break;
        }
        *seed ^= *seed << 13;
        *seed ^= *seed >> 17;
        *seed ^= *seed << 5;
        let noise = (*seed as f32 / u32::MAX as f32) * 2.0 - 1.0;
        let p = i as f32 / n as f32;
        let body = ((180.0 * i as f32 / SR as f32) * TAU).sin();
        let env = (1.0 - p).powf(2.5);
        buf[idx] += (noise * 0.7 + body * 0.4) * env * vol;
    }
}

fn add_hat(buf: &mut [f32], start: usize, vol: f32, seed: &mut u32) {
    let n = (SR as f32 * 0.04) as usize;
    let mut prev = 0.0f32;
    for i in 0..n {
        let idx = start + i;
        if idx >= buf.len() {
            break;
        }
        *seed ^= *seed << 13;
        *seed ^= *seed >> 17;
        *seed ^= *seed << 5;
        let noise = (*seed as f32 / u32::MAX as f32) * 2.0 - 1.0;
        let hp = noise - prev;
        prev = noise;
        let p = i as f32 / n as f32;
        let env = (1.0 - p).powf(3.0);
        buf[idx] += hp * env * vol;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MusicMode {
    Calm,
    Combat,
    Boss,
}

#[derive(Clone, Copy)]
enum Stem {
    Base,
    Combat,
    Boss,
}

struct Style {
    key: i32,
    minor: bool,
    bpm: f32,
    pad: Wave,
    lead: Wave,
    drums: u8,
}

const STYLES: [Style; 5] = [
    Style { key: 0, minor: false, bpm: 96.0, pad: Wave::Tri, lead: Wave::Square, drums: 1 },
    Style { key: -2, minor: true, bpm: 84.0, pad: Wave::Sine, lead: Wave::Tri, drums: 0 },
    Style { key: 3, minor: false, bpm: 92.0, pad: Wave::Sine, lead: Wave::Tri, drums: 1 },
    Style { key: -4, minor: true, bpm: 104.0, pad: Wave::Square, lead: Wave::Square, drums: 2 },
    Style { key: -5, minor: true, bpm: 76.0, pad: Wave::Sine, lead: Wave::Square, drums: 1 },
];

const MAJOR_CHORDS: [[i32; 4]; 8] = [
    [60, 64, 67, 71],
    [55, 59, 62, 67],
    [57, 60, 64, 67],
    [53, 57, 60, 65],
    [50, 53, 57, 60],
    [55, 59, 62, 65],
    [52, 55, 59, 62],
    [57, 60, 64, 67],
];
const MAJOR_BASS: [i32; 8] = [36, 43, 45, 41, 38, 43, 40, 45];
const MINOR_CHORDS: [[i32; 4]; 8] = [
    [57, 60, 64, 69],
    [53, 57, 60, 65],
    [60, 64, 67, 72],
    [55, 59, 62, 67],
    [50, 53, 57, 62],
    [57, 60, 64, 69],
    [52, 56, 59, 64],
    [57, 60, 64, 69],
];
const MINOR_BASS: [i32; 8] = [45, 41, 36, 43, 38, 45, 40, 45];

fn music_stem(stem: Stem, sid: i32) -> Vec<f32> {
    let st = &STYLES[(sid.max(0) as usize) % STYLES.len()];
    let beat = SR as f32 * 60.0 / st.bpm;
    let eighth = beat / 2.0;
    let bars = 8usize;
    let total = (beat * 4.0 * bars as f32) as usize;
    let mut buf = vec![0.0f32; total];
    let mut seed = match stem {
        Stem::Base => 0x2545_F491u32,
        Stem::Combat => 0x9E37_79B9,
        Stem::Boss => 0x1B56_C4E9,
    };
    let (chords_src, bass_src) = if st.minor { (&MINOR_CHORDS, &MINOR_BASS) } else { (&MAJOR_CHORDS, &MAJOR_BASS) };

    for bar in 0..bars {
        let chord: [i32; 4] = chords_src[bar].map(|n| n + st.key);
        let bassn = bass_src[bar] + st.key;
        let bar_start = (bar as f32 * 4.0 * beat) as usize;
        let bar_len = 4.0 * beat / SR as f32;
        match stem {
            Stem::Base => {
                for &m in chord.iter() {
                    add_voice(&mut buf, bar_start, hz(m), bar_len * 0.98, 0.06, st.pad, 0.06, 0.2);
                }
                add_voice(&mut buf, bar_start, hz(bassn), beat * 2.0 / SR as f32, 0.2, Wave::Sine, 0.005, 0.06);
                add_voice(&mut buf, bar_start + (beat * 2.0) as usize, hz(bassn), beat * 2.0 / SR as f32, 0.2, Wave::Sine, 0.005, 0.06);
                for slot in 0..8 {
                    if slot % 2 == 1 {
                        add_hat(&mut buf, bar_start + (slot as f32 * eighth) as usize, 0.08, &mut seed);
                    }
                }
                add_kick(&mut buf, bar_start, 0.55);
                add_kick(&mut buf, bar_start + (beat * 2.0) as usize, 0.55);
                if st.drums >= 1 {
                    add_kick(&mut buf, bar_start + (beat * 2.0 + eighth * 1.5) as usize, 0.3);
                }
                add_snare(&mut buf, bar_start + beat as usize, 0.35, &mut seed);
                add_snare(&mut buf, bar_start + (beat * 3.0) as usize, 0.35, &mut seed);
            }
            Stem::Combat => {
                let arp = [0usize, 2, 1, 3, 2, 3, 1, 2];
                for slot in 0..8 {
                    let at = bar_start + (slot as f32 * eighth) as usize;
                    let m = chord[arp[slot] % chord.len()] + 12;
                    add_voice(&mut buf, at, hz(m), eighth * 0.85 / SR as f32, 0.1, st.lead, 0.003, 0.04);
                    let hat_vol = if st.drums >= 2 { 0.13 } else if slot % 2 == 1 { 0.12 } else { 0.07 };
                    add_hat(&mut buf, at, hat_vol, &mut seed);
                }
                if (bar % 2) == 1 {
                    add_voice(&mut buf, bar_start, hz(chord[3] + 12), beat / SR as f32, 0.09, st.lead, 0.01, 0.3);
                }
                add_kick(&mut buf, bar_start + (beat * 2.0 + eighth * 1.5) as usize, 0.4);
                add_snare(&mut buf, bar_start + (beat * 3.0 + eighth) as usize, 0.18, &mut seed);
            }
            Stem::Boss => {
                let root = bassn - 12;
                add_voice(&mut buf, bar_start, hz(root), bar_len * 0.95, 0.16, Wave::Square, 0.02, 0.15);
                add_voice(&mut buf, bar_start, hz(root + 6), bar_len * 0.5, 0.05, Wave::Square, 0.02, 0.2);
                for b in 0..8 {
                    add_kick(&mut buf, bar_start + (b as f32 * eighth) as usize, 0.42);
                }
                add_snare(&mut buf, bar_start + (beat * 2.0) as usize, 0.28, &mut seed);
            }
        }
    }

    for s in buf.iter_mut() {
        *s = (*s * 0.9).tanh();
    }
    buf
}

struct StemSink {
    sink: Sink,
    cur: f32,
    target: f32,
}

pub struct Audio {
    _stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    music: Vec<StemSink>,
    music_level: f32,
    voice: i32,
    volume: f32,
    speed_cur: f32,
    intensity: f32,
    intensity_target: f32,
    pub muted: bool,
}

impl Audio {
    pub fn new(ambient_on: bool, master_volume: f32, ambient_volume: f32) -> Self {
        let volume = master_volume.clamp(0.0, 2.0);
        let music_level = ambient_volume.clamp(0.0, 2.0);
        let voice = 0i32;
        match OutputStream::try_default() {
            Ok((stream, handle)) => {
                let mut music = Vec::new();
                if ambient_on {
                    for (i, stem) in [Stem::Base, Stem::Combat, Stem::Boss].into_iter().enumerate() {
                        if let Ok(sink) = Sink::try_new(&handle) {
                            let start = if i == 0 { music_level } else { 0.0 };
                            sink.set_volume(start);
                            sink.append(SamplesBuffer::new(1, SR, music_stem(stem, voice)).repeat_infinite());
                            music.push(StemSink { sink, cur: start, target: start });
                        }
                    }
                }
                Audio { _stream: Some(stream), handle: Some(handle), music, music_level, voice, volume, speed_cur: 1.0, intensity: 0.0, intensity_target: 0.0, muted: false }
            }
            Err(_) => Audio { _stream: None, handle: None, music: Vec::new(), music_level, voice, volume, speed_cur: 1.0, intensity: 0.0, intensity_target: 0.0, muted: false },
        }
    }

    pub fn set_biome(&mut self, style: i32) {
        if self.voice == style || self.music.len() < 3 {
            return;
        }
        self.voice = style;
        let stems = [Stem::Base, Stem::Combat, Stem::Boss];
        let muted = self.muted;
        for (i, st) in self.music.iter_mut().enumerate() {
            st.sink.clear();
            st.sink.set_volume(st.cur);
            st.sink.append(SamplesBuffer::new(1, SR, music_stem(stems[i], style)).repeat_infinite());
            if muted {
                st.sink.pause();
            } else {
                st.sink.play();
            }
        }
    }

    pub fn play(&self, sound: Sound) {
        if self.muted || self.volume <= 0.0 {
            return;
        }
        if let Some(handle) = &self.handle {
            let _ = handle.play_raw(SamplesBuffer::new(1, SR, render(sound)).amplify(self.volume));
        }
    }

    pub fn set_levels(&mut self, sfx: f32, music: f32) {
        self.volume = sfx.clamp(0.0, 2.0);
        self.music_level = music.clamp(0.0, 2.0);
    }

    pub fn set_music_mode(&mut self, mode: MusicMode) {
        if self.music.len() < 3 {
            return;
        }
        let lvl = self.music_level;
        let boss = mode == MusicMode::Boss;
        self.music[0].target = if boss { 0.0 } else { lvl };
        let combat_layer = if boss {
            0.0
        } else if mode == MusicMode::Combat {
            1.0
        } else {
            (self.intensity * 0.9).min(0.6)
        };
        self.music[1].target = lvl * combat_layer;
        self.music[2].target = if boss { lvl } else { 0.0 };
    }

    pub fn set_intensity(&mut self, t: f32) {
        self.intensity_target = t.clamp(0.0, 1.0);
    }

    pub fn tick(&mut self) {
        if self.muted {
            return;
        }
        let step = (self.music_level * 0.06).max(0.01);
        for st in self.music.iter_mut() {
            if (st.cur - st.target).abs() <= step {
                st.cur = st.target;
            } else if st.cur < st.target {
                st.cur += step;
            } else {
                st.cur -= step;
            }
            st.sink.set_volume(st.cur);
        }
        let istep = 0.006;
        if (self.intensity - self.intensity_target).abs() <= istep {
            self.intensity = self.intensity_target;
        } else if self.intensity < self.intensity_target {
            self.intensity += istep;
        } else {
            self.intensity -= istep;
        }
        let speed = 1.0 + self.intensity * 0.10;
        if (speed - self.speed_cur).abs() > 0.001 {
            self.speed_cur = speed;
            for st in self.music.iter() {
                st.sink.set_speed(self.speed_cur);
            }
        }
    }

    pub fn toggle_mute(&mut self) {
        self.set_muted(!self.muted);
    }

    pub fn set_muted(&mut self, m: bool) {
        self.muted = m;
        for st in self.music.iter() {
            if self.muted {
                st.sink.pause();
            } else {
                st.sink.play();
            }
        }
    }
}

#[cfg(test)]
mod preview {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn write_wav(path: &str, samples: &[f32]) {
        let n = samples.len() as u32;
        let byte_rate = SR * 2;
        let data_len = n * 2;
        let mut f = fs::File::create(path).unwrap();
        f.write_all(b"RIFF").unwrap();
        f.write_all(&(36 + data_len).to_le_bytes()).unwrap();
        f.write_all(b"WAVEfmt ").unwrap();
        f.write_all(&16u32.to_le_bytes()).unwrap();
        f.write_all(&1u16.to_le_bytes()).unwrap();
        f.write_all(&1u16.to_le_bytes()).unwrap();
        f.write_all(&SR.to_le_bytes()).unwrap();
        f.write_all(&byte_rate.to_le_bytes()).unwrap();
        f.write_all(&2u16.to_le_bytes()).unwrap();
        f.write_all(&16u16.to_le_bytes()).unwrap();
        f.write_all(b"data").unwrap();
        f.write_all(&data_len.to_le_bytes()).unwrap();
        for s in samples {
            let v = ((s * 0.5).clamp(-1.0, 1.0) * 32767.0) as i16;
            f.write_all(&v.to_le_bytes()).unwrap();
        }
    }

    #[test]
    fn dump_wavs() {
        let dir = "/tmp/abyssal_sfx";
        let _ = fs::remove_dir_all(dir);
        fs::create_dir_all(dir).unwrap();
        let all = Sound::ALL;
        let mut montage: Vec<f32> = Vec::new();
        let gap = vec![0.0f32; (SR as f32 * 0.22) as usize];
        for &(name, s) in all {
            let buf = render(s);
            write_wav(&format!("{}/{}.wav", dir, name), &buf);
            montage.extend_from_slice(&buf);
            montage.extend_from_slice(&gap);
        }
        write_wav(&format!("{}/_all_sfx.wav", dir), &montage);

        let biomes = ["caverns", "catacombs", "frostvault", "emberdepths", "abyss"];
        for (sid, bname) in biomes.iter().enumerate() {
            let base = music_stem(Stem::Base, sid as i32);
            let combat = music_stem(Stem::Combat, sid as i32);
            let boss = music_stem(Stem::Boss, sid as i32);
            let mix = |layers: &[&Vec<f32>]| -> Vec<f32> {
                let mut out = vec![0.0f32; base.len()];
                for layer in layers {
                    for (o, v) in out.iter_mut().zip(layer.iter()) {
                        *o += v;
                    }
                }
                out.iter().map(|s| (s * 0.45).clamp(-1.0, 1.0)).collect()
            };
            let scenes = [
                ("calm", mix(&[&base])),
                ("combat", mix(&[&base, &combat])),
                ("boss", mix(&[&base, &combat, &boss])),
            ];
            for (scene, loop_buf) in &scenes {
                let mut x2 = Vec::new();
                for _ in 0..2 {
                    x2.extend_from_slice(loop_buf);
                }
                write_wav(&format!("{}/music_{}_{}.wav", dir, bname, scene), &x2);
            }
        }
        assert!(!montage.is_empty());
    }
}
