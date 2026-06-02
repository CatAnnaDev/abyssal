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
    }
    b
}

fn ambient_loop() -> Vec<f32> {
    let mut s = Synth::new();
    let dur = 7.0f32;
    let n = (SR as f32 * dur) as usize;
    let mut b: Vec<f32> = Vec::with_capacity(n);
    let base = 55.0f32;
    for i in 0..n {
        let t = i as f32 / SR as f32;
        let lfo = (t * TAU / dur).sin() * 0.5 + 0.5;
        let drone = (base * t * TAU).sin() * 0.5
            + (base * 1.5 * t * TAU).sin() * 0.22
            + (base * 2.01 * t * TAU).sin() * 0.12;
        let shimmer = ((base * 4.0 + 0.3 * (t * 0.5).sin()) * t * TAU).sin() * 0.05 * lfo;
        let wind = s.rand() * 0.015 * lfo;
        let edge = (i.min(n - i) as f32 / (SR as f32 * 0.4)).min(1.0);
        b.push((drone * 0.5 + shimmer + wind) * 0.5 * edge);
    }
    b
}

pub struct Audio {
    _stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    ambient: Option<Sink>,
    volume: f32,
    pub muted: bool,
}

impl Audio {
    pub fn new(ambient_on: bool, master_volume: f32, ambient_volume: f32) -> Self {
        let volume = master_volume.clamp(0.0, 2.0);
        match OutputStream::try_default() {
            Ok((stream, handle)) => {
                let ambient = if ambient_on {
                    Sink::try_new(&handle).ok().map(|sink| {
                        sink.set_volume((ambient_volume * volume).clamp(0.0, 2.0));
                        sink.append(SamplesBuffer::new(1, SR, ambient_loop()).repeat_infinite());
                        sink
                    })
                } else {
                    None
                };
                Audio { _stream: Some(stream), handle: Some(handle), ambient, volume, muted: false }
            }
            Err(_) => Audio { _stream: None, handle: None, ambient: None, volume, muted: false },
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

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        if let Some(a) = &self.ambient {
            if self.muted {
                a.pause();
            } else {
                a.play();
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
        let all = [
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
        ];
        let mut montage: Vec<f32> = Vec::new();
        let gap = vec![0.0f32; (SR as f32 * 0.22) as usize];
        for (name, s) in all {
            let buf = render(s);
            write_wav(&format!("{}/{}.wav", dir, name), &buf);
            montage.extend_from_slice(&buf);
            montage.extend_from_slice(&gap);
        }
        write_wav(&format!("{}/_all_sfx.wav", dir), &montage);
        let amb = ambient_loop();
        write_wav(&format!("{}/ambient.wav", dir), &amb);
        assert!(!montage.is_empty());
    }
}
