use macroquad::prelude::*;
use crate::consts::MAX_SKIDS;

#[derive(Clone, Copy)]
pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub color: Color,
}

impl Particle {
    pub fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
        self.vel *= 0.97;
        self.life -= dt;
    }

    pub fn draw(&self) {
        if self.life <= 0.0 { return; }
        let t = (self.life / self.max_life).clamp(0.0, 1.0);
        // Royk (stor size, lav alpha) vokser; gnister (liten size, hoy alpha) krymper
        let is_smoke = self.size > 5.0 && self.color.a < 0.3;
        let s = if is_smoke {
            self.size * (0.5 + (1.0 - t) * 1.5)  // vokser fra 50% til 200%
        } else {
            self.size * (0.3 + 0.7 * t)  // krymper normalt
        };
        let alpha = if is_smoke {
            self.color.a * t * t  // fader ut raskere
        } else {
            self.color.a * t
        };
        let c = Color::new(self.color.r, self.color.g, self.color.b, alpha);
        draw_circle(self.pos.x, self.pos.y, s, c);
    }
}

#[derive(Clone, Copy)]
pub struct SkidMark {
    pub pos: Vec2,
    pub alpha: f32,
}

/// Sirkulær buffer for skidmerker — overskriver eldste ved full kapasitet
pub struct SkidBuffer {
    marks: Vec<SkidMark>,
    cursor: usize,
}

impl SkidBuffer {
    pub fn new() -> Self {
        Self { marks: Vec::with_capacity(MAX_SKIDS), cursor: 0 }
    }

    pub fn push(&mut self, mark: SkidMark) {
        if self.marks.len() < MAX_SKIDS {
            self.marks.push(mark);
        } else {
            self.marks[self.cursor] = mark;
            self.cursor = (self.cursor + 1) % MAX_SKIDS;
        }
    }

    pub fn clear(&mut self) {
        self.marks.clear();
        self.cursor = 0;
    }

    pub fn fade(&mut self, dt: f32) {
        for s in &mut self.marks {
            s.alpha -= dt * 0.015;
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, SkidMark> {
        self.marks.iter()
    }
}
