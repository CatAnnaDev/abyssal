use crate::entity::Color;
use crate::rng::Rng;

pub struct FloatText {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub color: Color,
    pub age: i32,
    pub ttl: i32,
}

pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub glyph: char,
    pub color: Color,
    pub ttl: i32,
}

pub struct Projectile {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub glyph: char,
    pub color: Color,
    pub ttl: i32,
}

#[derive(Default)]
pub struct Fx {
    pub floats: Vec<FloatText>,
    pub particles: Vec<Particle>,
    pub projectiles: Vec<Projectile>,
    pub shake: i32,
    pub combo: i32,
    pub combo_timer: i32,
    pub transition: i32,
    pub transition_floor: i32,
}

impl Fx {
    pub fn tick(&mut self) {
        for f in self.floats.iter_mut() {
            f.age += 1;
            f.y -= 0.45;
        }
        self.floats.retain(|f| f.age < f.ttl);

        for p in self.particles.iter_mut() {
            p.x += p.vx;
            p.y += p.vy;
            p.vy += 0.04;
            p.ttl -= 1;
        }
        self.particles.retain(|p| p.ttl > 0);

        let mut trail: Vec<Particle> = Vec::new();
        for p in self.projectiles.iter() {
            let glow = (
                ((p.color.0 as u16 + 60).min(255)) as u8,
                ((p.color.1 as u16 + 60).min(255)) as u8,
                ((p.color.2 as u16 + 60).min(255)) as u8,
            );
            trail.push(Particle { x: p.x, y: p.y, vx: 0.0, vy: 0.0, glyph: '\u{00b7}', color: p.color, ttl: 4 });
            trail.push(Particle { x: p.x - p.vx * 0.5, y: p.y - p.vy * 0.5, glyph: '\u{2218}', color: glow, vx: 0.0, vy: 0.0, ttl: 3 });
        }
        self.particles.extend(trail);

        for p in self.projectiles.iter_mut() {
            p.x += p.vx;
            p.y += p.vy;
            p.ttl -= 1;
        }
        self.projectiles.retain(|p| p.ttl > 0);

        if self.shake > 0 {
            self.shake -= 1;
        }
        if self.transition > 0 {
            self.transition -= 1;
        }
        if self.combo_timer > 0 {
            self.combo_timer -= 1;
            if self.combo_timer == 0 {
                self.combo = 0;
            }
        }
    }

    pub fn damage(&mut self, x: i32, y: i32, amount: i32, crit: bool) {
        let (text, color) = if crit {
            (format!("{}!", amount), (255, 230, 80))
        } else {
            (format!("{}", amount), (255, 120, 110))
        };
        self.floats.push(FloatText { x: x as f32, y: y as f32 - 0.4, text, color, age: 0, ttl: 7 });
    }

    pub fn damage_el(&mut self, x: i32, y: i32, amount: i32, crit: bool, base: Color) {
        let text = if crit { format!("{}!", amount) } else { format!("{}", amount) };
        let color = if crit {
            ((base.0 as u16 + 40).min(255) as u8, (base.1 as u16 + 40).min(255) as u8, (base.2 as u16 + 40).min(255) as u8)
        } else {
            base
        };
        self.floats.push(FloatText { x: x as f32, y: y as f32 - 0.4, text, color, age: 0, ttl: 7 });
    }

    pub fn label(&mut self, x: i32, y: i32, text: &str, color: Color) {
        self.floats.push(FloatText { x: x as f32, y: y as f32 - 0.4, text: text.to_string(), color, age: 0, ttl: 9 });
    }

    pub fn burst(&mut self, rng: &mut Rng, x: i32, y: i32, color: Color, count: i32, glyph: char) {
        for _ in 0..count {
            let a = rng.range(0.0, 6.2831);
            let speed = rng.range(0.1, 0.5);
            self.particles.push(Particle {
                x: x as f32,
                y: y as f32,
                vx: a.cos() * speed,
                vy: a.sin() * speed * 0.6,
                glyph,
                color,
                ttl: rng.between(4, 9),
            });
        }
    }

    pub fn projectile(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, glyph: char, color: Color) {
        let dx = (x1 - x0) as f32;
        let dy = (y1 - y0) as f32;
        let dist = (dx * dx + dy * dy).sqrt().max(1.0);
        let speed = 0.9;
        let ttl = (dist / speed).ceil() as i32 + 1;
        self.projectiles.push(Projectile {
            x: x0 as f32,
            y: y0 as f32,
            vx: dx / dist * speed,
            vy: dy / dist * speed,
            glyph,
            color,
            ttl,
        });
    }

    pub fn add_shake(&mut self, frames: i32) {
        self.shake = self.shake.max(frames);
    }

    pub fn bump_combo(&mut self) {
        self.combo += 1;
        self.combo_timer = 26;
    }

    pub fn begin_transition(&mut self, floor: i32) {
        self.transition = 16;
        self.transition_floor = floor;
    }

    pub fn shake_offset(&self) -> i32 {
        if self.shake > 0 {
            self.shake % 3
        } else {
            0
        }
    }
}
