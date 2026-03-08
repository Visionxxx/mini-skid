use macroquad::prelude::*;
use std::f32::consts::{PI, TAU};
use crate::car::Car;
use crate::consts::*;

pub struct Track {
    pub name: &'static str,
    pub center: Vec<Vec2>,
    pub width: f32,
    pub height: Vec<f32>,
    pub start_positions: Vec<Vec2>,
    pub start_angle: f32,
    pub trees: Vec<Vec2>,
}

impl Track {
    /// Returnerer (segment-indeks, avstand, fraksjon langs segmentet)
    pub fn nearest_info(&self, p: Vec2) -> (usize, f32, f32) {
        let n = self.center.len();
        let mut best_dist = f32::MAX;
        let mut best_idx = 0;
        let mut best_t = 0.0;
        for i in 0..n {
            let a = self.center[i];
            let b = self.center[(i + 1) % n];
            let ab = b - a;
            let ap = p - a;
            let t = (ap.dot(ab) / ab.dot(ab)).clamp(0.0, 1.0);
            let closest = a + ab * t;
            let dist = p.distance(closest);
            if dist < best_dist {
                best_dist = dist;
                best_idx = i;
                best_t = t;
            }
        }
        (best_idx, best_dist, best_t)
    }

    pub fn point_on_track(&self, p: Vec2) -> bool {
        self.nearest_info(p).1 < self.width / 2.0
    }

    pub fn sector(&self, p: Vec2) -> u8 {
        let (idx, _, _) = self.nearest_info(p);
        let quarter = self.center.len() / 4;
        (idx / quarter).min(3) as u8
    }

    pub fn progress(&self, car: &Car) -> f32 {
        let (idx, _, t) = self.nearest_info(car.pos);
        car.laps as f32 * self.center.len() as f32 + idx as f32 + t
    }

    pub fn normal_at(&self, i: usize) -> Vec2 {
        let n = self.center.len();
        let prev = self.center[(i + n - 1) % n];
        let next = self.center[(i + 1) % n];
        let tangent = (next - prev).normalize();
        vec2(tangent.y, -tangent.x)
    }

    pub fn tangent_at(&self, i: usize) -> Vec2 {
        let n = self.center.len();
        let prev = self.center[(i + n - 1) % n];
        let next = self.center[(i + 1) % n];
        (next - prev).normalize()
    }

    pub fn curvature_ahead(&self, idx: usize, lookahead: usize) -> f32 {
        let n = self.center.len();
        let i1 = idx % n;
        let i2 = (idx + lookahead / 2) % n;
        let i3 = (idx + lookahead) % n;
        let d1 = (self.center[i2] - self.center[i1]).normalize();
        let d2 = (self.center[i3] - self.center[i2]).normalize();
        (d1.x * d2.y - d1.y * d2.x).abs()
    }

    pub fn height_at_pos(&self, pos: Vec2) -> f32 {
        let (idx, _, t) = self.nearest_info(pos);
        let n = self.height.len();
        let h1 = self.height[idx];
        let h2 = self.height[(idx + 1) % n];
        h1 + (h2 - h1) * t
    }

    pub fn slope_at(&self, idx: usize) -> f32 {
        let n = self.center.len();
        let j = (idx + 1) % n;
        let dist = self.center[idx].distance(self.center[j]);
        if dist > 0.01 { (self.height[j] - self.height[idx]) / dist } else { 0.0 }
    }

    pub fn draw(&self) {
        let bounds = self.bounds();
        let margin = 400.0;

        // Gress-bakgrunn
        draw_rectangle(
            bounds.0 - margin, bounds.1 - margin,
            bounds.2 - bounds.0 + margin * 2.0,
            bounds.3 - bounds.1 + margin * 2.0,
            Color::new(0.15, 0.38, 0.1, 1.0),
        );

        // Gress-variasjon
        let step = 60.0;
        let x_start = ((bounds.0 - margin) / step).floor() as i32;
        let x_end = ((bounds.2 + margin) / step).ceil() as i32;
        let y_start = ((bounds.1 - margin) / step).floor() as i32;
        let y_end = ((bounds.3 + margin) / step).ceil() as i32;
        for ix in x_start..x_end {
            for iy in y_start..y_end {
                let hash = ((ix * 7 + iy * 13 + 37) as u32).wrapping_mul(2654435761);
                let x = ix as f32 * step + (hash % 40) as f32;
                let y = iy as f32 * step + ((hash >> 8) % 40) as f32;
                let shade = 0.08 + ((hash >> 16) % 5) as f32 * 0.01;
                draw_circle(x, y, 2.0, Color::new(0.1, 0.28 + shade, shade, 1.0));
            }
        }

        // Traer
        for tree in &self.trees {
            draw_circle(tree.x + 4.0, tree.y + 4.0, 16.0, Color::new(0.0, 0.0, 0.0, 0.2));
            draw_circle(tree.x, tree.y, 16.0, Color::new(0.1, 0.45, 0.08, 1.0));
            draw_circle(tree.x - 4.0, tree.y - 3.0, 10.0, Color::new(0.12, 0.5, 0.1, 1.0));
            draw_circle(tree.x, tree.y, 4.0, Color::new(0.35, 0.22, 0.1, 1.0));
        }

        let n = self.center.len();
        let half = self.width / 2.0;

        // Bro-skygge paa bakken (tegnes foer asfalt)
        for i in 0..n {
            let h = self.height[i];
            if h > BRIDGE_HEIGHT {
                let j = (i + 1) % n;
                let c1 = self.center[i];
                let c2 = self.center[j];
                let n1 = self.normal_at(i);
                let n2 = self.normal_at(j);
                let sw = half + 10.0;
                let so = vec2(h * 3.0, h * 4.0);
                let p1 = c1 + n1 * sw + so;
                let p2 = c1 - n1 * sw + so;
                let p3 = c2 - n2 * sw + so;
                let p4 = c2 + n2 * sw + so;
                draw_triangle(p1, p2, p3, Color::new(0.0, 0.0, 0.0, 0.25));
                draw_triangle(p1, p3, p4, Color::new(0.0, 0.0, 0.0, 0.25));
            }
        }

        // Asfalt (bakkehøyde-basert skyggelegging)
        for i in 0..n {
            let j = (i + 1) % n;
            let c1 = self.center[i];
            let c2 = self.center[j];
            let n1 = self.normal_at(i);
            let n2 = self.normal_at(j);
            let h = (self.height[i] + self.height[j]) / 2.0;
            let shade = 0.28 + (i as f32 * 0.008).sin() * 0.03 + h * 0.04;
            let asphalt = Color::new(shade, shade, shade + 0.02, 1.0);
            let p1 = c1 + n1 * half;
            let p2 = c1 - n1 * half;
            let p3 = c2 - n2 * half;
            let p4 = c2 + n2 * half;
            draw_triangle(p1, p2, p3, asphalt);
            draw_triangle(p1, p3, p4, asphalt);
        }

        // Curbs
        for i in 0..n {
            let j = (i + 1) % n;
            let c1 = self.center[i];
            let c2 = self.center[j];
            let n1 = self.normal_at(i);
            let n2 = self.normal_at(j);
            let curb = if (i / 4) % 2 == 0 { RED } else { WHITE };
            for sign in [-1.0_f32, 1.0] {
                let p1 = c1 + n1 * half * sign;
                let p2 = c2 + n2 * half * sign;
                draw_line(p1.x, p1.y, p2.x, p2.y, 3.0, curb);
            }
            if (i / 5) % 2 == 0 {
                draw_line(c1.x, c1.y, c2.x, c2.y, 1.0, Color::new(1.0, 1.0, 1.0, 0.2));
            }
        }

        // Bakke-markorer (sjevroner paa oppoverbakker)
        let marker_step = 12;
        for i in (0..n).step_by(marker_step) {
            let slope = self.slope_at(i);
            let h = self.height[i];
            if slope.abs() > 0.004 && h < BRIDGE_HEIGHT {
                let c = self.center[i];
                let tang = self.tangent_at(i);
                let norm = self.normal_at(i);
                let dir = if slope > 0.0 { 1.0 } else { -1.0 };
                let tip = c + tang * 6.0 * dir;
                let l = c - tang * 4.0 * dir + norm * 5.0;
                let r = c - tang * 4.0 * dir - norm * 5.0;
                let alpha = (slope.abs() * 60.0).clamp(0.08, 0.3);
                draw_triangle(tip, l, r, Color::new(1.0, 1.0, 1.0, alpha));
            }
        }

        // Start/maal-linje
        let c = self.center[0];
        let norm = self.normal_at(0);
        let tang = self.tangent_at(0);
        let cs = 14.0;
        let rows = (self.width / cs) as i32;
        for r in -rows / 2..rows / 2 {
            for col in 0..3 {
                let color = if (r + col) % 2 == 0 { WHITE } else { BLACK };
                let base = c + norm * (r as f32 * cs) + tang * ((col as f32 - 1.5) * cs);
                let p1 = base;
                let p2 = base + norm * cs;
                let p3 = base + norm * cs + tang * cs;
                let p4 = base + tang * cs;
                draw_triangle(p1, p2, p3, color);
                draw_triangle(p1, p3, p4, color);
            }
        }

        // Startfelt (grid boxes)
        for (i, pos) in self.start_positions.iter().enumerate() {
            let fwd = tang;
            let side = norm;
            let hw = 16.0;
            let hh = 10.0;
            let corners = [
                *pos + fwd * hw + side * hh,
                *pos + fwd * hw - side * hh,
                *pos - fwd * hw - side * hh,
                *pos - fwd * hw + side * hh,
            ];
            let grid_color = Color::new(1.0, 1.0, 1.0, 0.3);
            draw_line(corners[0].x, corners[0].y, corners[1].x, corners[1].y, 2.0, grid_color);
            draw_line(corners[1].x, corners[1].y, corners[2].x, corners[2].y, 2.0, grid_color);
            draw_line(corners[2].x, corners[2].y, corners[3].x, corners[3].y, 2.0, grid_color);
            draw_line(corners[3].x, corners[3].y, corners[0].x, corners[0].y, 2.0, grid_color);
            let num_pos = *pos - fwd * 5.0;
            let count = i + 1;
            for d in 0..count {
                draw_circle(
                    num_pos.x + (d as f32 - count as f32 / 2.0 + 0.5) * 6.0,
                    num_pos.y,
                    2.5,
                    Color::new(1.0, 1.0, 1.0, 0.4),
                );
            }
        }

        // ── Trackside-dekor ──

        // Barrierer (annethvert segment, litt utenfor banen)
        let barrier_step = 6;
        for i in (0..n).step_by(barrier_step) {
            if self.height[i] > BRIDGE_HEIGHT { continue; }
            let j = (i + barrier_step.min(n - 1)) % n;
            let c1 = self.center[i];
            let c2 = self.center[j];
            let n1 = self.normal_at(i);
            let n2 = self.normal_at(j);
            let barrier_dist = half + 8.0;
            let barrier_color = if (i / 12) % 3 == 0 {
                Color::new(0.85, 0.15, 0.1, 0.6)
            } else {
                Color::new(0.9, 0.9, 0.9, 0.4)
            };
            for sign in [-1.0_f32, 1.0] {
                let p1 = c1 + n1 * barrier_dist * sign;
                let p2 = c2 + n2 * barrier_dist * sign;
                draw_line(p1.x, p1.y, p2.x, p2.y, 2.5, barrier_color);
            }
        }

        // Flagg/stolper ved markante punkter
        let flag_step = (n / 16).max(1);
        for i in (0..n).step_by(flag_step) {
            if self.height[i] > BRIDGE_HEIGHT { continue; }
            let c = self.center[i];
            let norm = self.normal_at(i);
            let hash = ((i * 31 + 17) as u32).wrapping_mul(2654435761);
            let sign = if hash % 2 == 0 { 1.0 } else { -1.0 };
            let pole_base = c + norm * (half + 20.0) * sign;
            // Stolpe
            draw_line(pole_base.x, pole_base.y, pole_base.x, pole_base.y - 18.0, 2.0,
                Color::new(0.5, 0.5, 0.5, 0.7));
            // Flagg (liten trekant)
            let flag_color = match hash % 4 {
                0 => Color::new(1.0, 0.2, 0.1, 0.7),
                1 => Color::new(0.1, 0.5, 1.0, 0.7),
                2 => Color::new(1.0, 0.9, 0.1, 0.7),
                _ => Color::new(0.2, 0.9, 0.2, 0.7),
            };
            let ft = pole_base + vec2(0.0, -18.0);
            let wave = (get_time() as f32 * 3.0 + i as f32 * 0.1).sin() * 3.0;
            draw_triangle(
                ft,
                ft + vec2(10.0 + wave, 3.0),
                ft + vec2(2.0 + wave * 0.5, 8.0),
                flag_color,
            );
        }

        // Tilskuere (klynger av fargede prikker naer start/maal)
        let start_c = self.center[0];
        let start_n = self.normal_at(0);
        for cluster in 0..2 {
            let cluster_sign = if cluster == 0 { 1.0 } else { -1.0 };
            let base = start_c + start_n * (half + 35.0) * cluster_sign;
            for p in 0..20 {
                let hash = ((p * 73 + cluster * 137 + 42) as u32).wrapping_mul(2654435761);
                let ox = ((hash % 50) as f32) - 25.0;
                let oy = ((hash >> 8) % 30) as f32 - 15.0;
                let head_color = Color::new(
                    0.6 + ((hash >> 12) % 4) as f32 * 0.1,
                    0.4 + ((hash >> 16) % 5) as f32 * 0.1,
                    0.3 + ((hash >> 20) % 4) as f32 * 0.15,
                    0.8,
                );
                let pos = base + vec2(ox, oy);
                // Kropp
                draw_circle(pos.x, pos.y + 2.0, 2.5, head_color);
                // Hode
                draw_circle(pos.x, pos.y, 1.8, Color::new(0.9, 0.75, 0.6, 0.8));
            }
        }
    }

    /// Tegn bro-dekke og rekkverk oppaa (kalles mellom bakke-biler og bro-biler)
    pub fn draw_bridge(&self) {
        let n = self.center.len();
        let half = self.width / 2.0;
        let has_bridge = self.height.iter().any(|&h| h > BRIDGE_HEIGHT);
        if !has_bridge { return; }

        // Bro-stolper
        let support_step = 12;
        for i in (0..n).step_by(support_step) {
            let h = self.height[i];
            if h > BRIDGE_HEIGHT + 0.3 {
                let c = self.center[i];
                let norm = self.normal_at(i);
                for sign in [-1.0_f32, 1.0] {
                    let base = c + norm * (half + 4.0) * sign;
                    draw_rectangle(base.x - 3.0, base.y - 3.0, 6.0, 8.0,
                        Color::new(0.35, 0.35, 0.3, 0.9));
                }
            }
        }

        // Bro-asfalt (re-tegnes oppaa)
        for i in 0..n {
            if self.height[i] <= BRIDGE_HEIGHT && self.height[(i + 1) % n] <= BRIDGE_HEIGHT {
                continue;
            }
            let j = (i + 1) % n;
            let c1 = self.center[i];
            let c2 = self.center[j];
            let n1 = self.normal_at(i);
            let n2 = self.normal_at(j);
            let h = (self.height[i] + self.height[j]) / 2.0;
            let shade = 0.35 + h * 0.03;
            let asphalt = Color::new(shade + 0.02, shade + 0.01, shade + 0.05, 1.0);
            let p1 = c1 + n1 * half;
            let p2 = c1 - n1 * half;
            let p3 = c2 - n2 * half;
            let p4 = c2 + n2 * half;
            draw_triangle(p1, p2, p3, asphalt);
            draw_triangle(p1, p3, p4, asphalt);
            // Bro-curbs
            let curb = if (i / 4) % 2 == 0 { RED } else { WHITE };
            for sign in [-1.0_f32, 1.0] {
                let lp1 = c1 + n1 * half * sign;
                let lp2 = c2 + n2 * half * sign;
                draw_line(lp1.x, lp1.y, lp2.x, lp2.y, 3.0, curb);
            }
        }

        // Bro-rekkverk
        for i in 0..n {
            if self.height[i] > BRIDGE_HEIGHT + 0.5 {
                let j = (i + 1) % n;
                let c1 = self.center[i];
                let c2 = self.center[j];
                let n1 = self.normal_at(i);
                let n2 = self.normal_at(j);
                for sign in [-1.0_f32, 1.0] {
                    let p1 = c1 + n1 * (half + 3.0) * sign;
                    let p2 = c2 + n2 * (half + 3.0) * sign;
                    draw_line(p1.x, p1.y, p2.x, p2.y, 3.0, Color::new(0.85, 0.65, 0.1, 0.9));
                }
            }
        }
    }

    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for p in &self.center {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }
        (min_x, min_y, max_x, max_y)
    }

    pub fn draw_minimap(&self, cars: &[Car]) {
        let bounds = self.bounds();
        let margin = 50.0;
        let world_w = bounds.2 - bounds.0 + margin * 2.0;
        let world_h = bounds.3 - bounds.1 + margin * 2.0;

        let mw = 150.0;
        let mh = mw * (world_h / world_w);
        let mx = screen_width() - mw - 10.0;
        let my = screen_height() - mh - 10.0;
        let sx = mw / world_w;
        let sy = mh / world_h;

        draw_rectangle(mx - 3.0, my - 3.0, mw + 6.0, mh + 6.0, Color::new(0.05, 0.05, 0.08, 1.0));

        let n = self.center.len();
        let step = (n / 80).max(1);
        for i in (0..n).step_by(step) {
            let j = (i + step) % n;
            let x1 = mx + (self.center[i].x - bounds.0 + margin) * sx;
            let y1 = my + (self.center[i].y - bounds.1 + margin) * sy;
            let x2 = mx + (self.center[j].x - bounds.0 + margin) * sx;
            let y2 = my + (self.center[j].y - bounds.1 + margin) * sy;
            let h = self.height[i];
            let bright = 0.4 + h * 0.1;
            let color = if h > BRIDGE_HEIGHT {
                Color::new(0.7, 0.6, 0.2, 0.9)
            } else {
                Color::new(bright, bright, bright, 0.8)
            };
            draw_line(x1, y1, x2, y2, 2.0, color);
        }

        for car in cars {
            let cx = mx + (car.pos.x - bounds.0 + margin) * sx;
            let cy = my + (car.pos.y - bounds.1 + margin) * sy;
            draw_circle(cx, cy, 4.0, car.color);
        }
    }
}

// ── Hjelpefunksjoner ──

/// Myk cosinus-puls: 1.0 i sentrum, 0.0 ved kanten
fn cosine_bump(d: f32, radius: f32) -> f32 {
    if d < radius {
        0.5 + 0.5 * (PI * d / radius).cos()
    } else {
        0.0
    }
}

/// Avstand paa sirkulaer bane (0..1) med wrapping
fn circular_dist(t: f32, target: f32) -> f32 {
    let mut d = (t - target).abs();
    if d > 0.5 { d = 1.0 - d; }
    d
}

pub fn generate_trees(center: &[Vec2], width: f32, height: &[f32]) -> Vec<Vec2> {
    let mut trees = Vec::new();
    let n = center.len();
    let tree_step = (n / 40).max(1);
    for i in (0..n).step_by(tree_step) {
        // Ikke plasser traer naer broer
        if height[i] > BRIDGE_HEIGHT * 0.5 { continue; }
        let c = center[i];
        let next = center[(i + 1) % n];
        let tang = (next - c).normalize();
        let norm = vec2(tang.y, -tang.x);
        for &sign in &[-1.0_f32, 1.0] {
            let dist = width / 2.0 + 40.0 + ((i * 7) % 60) as f32;
            let offset = ((i * 13) % 30) as f32 - 15.0;
            let pos = c + norm * dist * sign + tang * offset;
            trees.push(pos);
        }
    }
    trees
}

pub fn compute_start(center: &[Vec2], start_idx: usize) -> (Vec<Vec2>, f32) {
    let n = center.len();
    let tang = (center[(start_idx + 1) % n] - center[(start_idx + n - 1) % n]).normalize();
    let norm = vec2(tang.y, -tang.x);
    let start_angle = tang.y.atan2(tang.x);
    let c = center[start_idx];

    let positions = vec![
        c + norm * (-18.0) - tang * 5.0,
        c + norm * 18.0 - tang * 5.0,
        c + norm * (-18.0) - tang * 40.0,
        c + norm * 18.0 - tang * 40.0,
        c + norm * (-18.0) - tang * 75.0,
        c + norm * 18.0 - tang * 75.0,
    ];
    (positions, start_angle)
}

// ── Bane-generatorer ──

pub fn make_oval() -> Track {
    let cx = 1200.0_f32;
    let cy = 900.0_f32;
    let rx = 900.0_f32;
    let ry = 600.0_f32;
    let segments = 400;
    let center: Vec<Vec2> = (0..segments)
        .map(|i| {
            let a = (i as f32 / segments as f32) * TAU;
            vec2(cx + rx * a.cos(), cy + ry * a.sin())
        })
        .collect();
    let width = 140.0;

    // Hoydeprofil: bakke paa nedre rett + humpe paa ovre rett
    let height: Vec<f32> = (0..segments)
        .map(|i| {
            let t = i as f32 / segments as f32;
            // Stor bakke paa nedre rett (rundt t=0.25)
            let hill = 1.6 * cosine_bump(circular_dist(t, 0.25), 0.12);
            // Liten humpe (hopp-rampe) paa ovre rett (rundt t=0.75)
            let bump = 0.8 * cosine_bump(circular_dist(t, 0.75), 0.04);
            hill + bump
        })
        .collect();

    let start_idx = segments * 3 / 4;
    let (start_positions, start_angle) = compute_start(&center, start_idx);
    let trees = generate_trees(&center, width, &height);
    Track { name: "Oval Speedway", center, width, height, start_positions, start_angle, trees }
}

pub fn make_figure8() -> Track {
    let cx = 1200.0_f32;
    let cy = 900.0_f32;
    let scale = 750.0_f32;
    let segments = 500;
    let center: Vec<Vec2> = (0..segments)
        .map(|i| {
            let t = (i as f32 / segments as f32) * TAU;
            let denom = 1.0 + t.sin() * t.sin();
            let x = scale * t.cos() / denom;
            let y = scale * t.sin() * t.cos() / denom;
            vec2(cx + x, cy + y * 0.85)
        })
        .collect();
    let width = 130.0;

    // Hoydeprofil: BRO ved foerste krysning (t=0.25, idx 125)
    // Andre krysningen (t=0.75, idx 375) er paa bakken under broen
    let height: Vec<f32> = (0..segments)
        .map(|i| {
            let t = i as f32 / segments as f32;
            let d = circular_dist(t, 0.25);
            // Bro med flat topp (ingen hopp paa broen)
            let bridge = if d < 0.03 {
                3.0 // flat bro-dekke
            } else if d < 0.10 {
                3.0 * cosine_bump(d - 0.03, 0.07)
            } else {
                0.0
            };
            // Bølgende terreng i loopene
            let hills = ((t * TAU * 3.0).sin() * 0.35).max(0.0);
            bridge + hills
        })
        .collect();

    let (start_positions, start_angle) = compute_start(&center, 0);
    let trees = generate_trees(&center, width, &height);
    Track { name: "Figur-8 Cross", center, width, height, start_positions, start_angle, trees }
}

pub fn make_fjord() -> Track {
    let cx = 1200.0_f32;
    let cy = 900.0_f32;
    let segments = 500;
    let center: Vec<Vec2> = (0..segments)
        .map(|i| {
            let a = (i as f32 / segments as f32) * TAU;
            let r = 700.0
                + 180.0 * (2.0 * a).sin()
                + 90.0 * (3.0 * a).cos()
                + 55.0 * (5.0 * a).sin();
            vec2(cx + r * a.cos(), cy + r * 0.7 * a.sin())
        })
        .collect();
    let width = 135.0;

    // Hoydeprofil: dramatisk fjell-terreng med hopp
    let height: Vec<f32> = (0..segments)
        .map(|i| {
            let a = (i as f32 / segments as f32) * TAU;
            let t = i as f32 / segments as f32;
            // Fjell-landskap
            let terrain = 1.2 * (a).sin()
                + 0.8 * (2.0 * a).cos()
                + 0.6 * (3.0 * a).sin()
                + 0.4 * (5.0 * a).cos();
            // Hoppe-rampe (bratt bakketopp)
            let ramp = 2.8 * cosine_bump(circular_dist(t, 0.6), 0.035);
            // Ekstra bakke
            let hill2 = 1.5 * cosine_bump(circular_dist(t, 0.3), 0.06);
            (terrain + ramp + hill2).max(0.0)
        })
        .collect();

    let (start_positions, start_angle) = compute_start(&center, 0);
    let trees = generate_trees(&center, width, &height);
    Track { name: "Fjord Circuit", center, width, height, start_positions, start_angle, trees }
}
