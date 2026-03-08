use macroquad::prelude::*;
use std::f32::consts::{PI, TAU};
use crate::consts::*;
use crate::particle::{Particle, SkidMark, SkidBuffer};
use crate::track::Track;

pub struct CarInput {
    pub gas: bool,
    pub brake: bool,
    pub left: bool,
    pub right: bool,
}

// Touch-knapp-layout (skjermkoordinater)
const TOUCH_BTN: f32 = 90.0;
const TOUCH_GAP: f32 = 12.0;
const TOUCH_MARGIN: f32 = 20.0;

pub fn touch_button_rects() -> [(f32, f32, f32, f32); 4] {
    let sw = screen_width();
    let sh = screen_height();
    let y = sh - TOUCH_MARGIN - TOUCH_BTN;
    [
        (TOUCH_MARGIN, y, TOUCH_BTN, TOUCH_BTN),                          // LEFT
        (TOUCH_MARGIN + TOUCH_BTN + TOUCH_GAP, y, TOUCH_BTN, TOUCH_BTN),  // RIGHT
        (sw - TOUCH_MARGIN - 2.0 * TOUCH_BTN - TOUCH_GAP, y, TOUCH_BTN, TOUCH_BTN), // BRAKE
        (sw - TOUCH_MARGIN - TOUCH_BTN, y, TOUCH_BTN, TOUCH_BTN),         // GAS
    ]
}

pub fn get_touch_input() -> CarInput {
    use macroquad::input::{touches, TouchPhase};
    let btns = touch_button_rects();
    let mut input = CarInput { gas: false, brake: false, left: false, right: false };
    for touch in touches() {
        match touch.phase {
            TouchPhase::Started | TouchPhase::Stationary | TouchPhase::Moved => {
                let p = touch.position;
                for (i, &(bx, by, bw, bh)) in btns.iter().enumerate() {
                    if p.x >= bx && p.x <= bx + bw && p.y >= by && p.y <= by + bh {
                        match i {
                            0 => input.left = true,
                            1 => input.right = true,
                            2 => input.brake = true,
                            3 => input.gas = true,
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }
    input
}

#[derive(Clone)]
pub struct Car {
    pub pos: Vec2,
    pub vel: Vec2,
    pub angle: f32,
    pub laps: i32,
    pub last_sector: u8,
    pub sectors_visited: [bool; 4],
    pub color: Color,
    pub name: String,
    pub number: u8,
    pub keys: Option<(KeyCode, KeyCode, KeyCode, KeyCode)>,
    pub ai_skill: f32,
    pub ai_aggression: f32,
    pub ai_line_pref: f32,
    pub is_drifting: bool,
    pub finished: bool,
    pub finish_time: f32,
    pub best_lap: f32,
    pub lap_start_time: f32,
    // Terreng
    pub ground_height: f32,
    pub air_height: f32,
    pub vert_vel: f32,
    pub just_landed: bool,
    pub track_idx: usize, // Siste kjente segment-indeks (for kryss-hint)
}

impl Car {
    pub fn new_player(
        pos: Vec2, angle: f32, color: Color, name: &str, number: u8,
        keys: (KeyCode, KeyCode, KeyCode, KeyCode), track: &Track,
    ) -> Self {
        Self {
            pos, vel: Vec2::ZERO, angle, laps: 0,
            last_sector: track.sector(pos), sectors_visited: [false; 4],
            color, name: name.to_string(), number,
            keys: Some(keys),
            ai_skill: 0.0, ai_aggression: 0.0, ai_line_pref: 0.0,
            is_drifting: false, finished: false, finish_time: 0.0,
            best_lap: f32::MAX, lap_start_time: 0.0,
            ground_height: track.height_at_pos(pos), air_height: 0.0, vert_vel: 0.0, just_landed: false,
            track_idx: track.nearest_info(pos).0,
        }
    }

    pub fn new_ai(
        pos: Vec2, angle: f32, color: Color, name: &str, number: u8,
        skill: f32, aggression: f32, line_pref: f32, track: &Track,
    ) -> Self {
        Self {
            pos, vel: Vec2::ZERO, angle, laps: 0,
            last_sector: track.sector(pos), sectors_visited: [false; 4],
            color, name: name.to_string(), number,
            keys: None,
            ai_skill: skill, ai_aggression: aggression, ai_line_pref: line_pref,
            is_drifting: false, finished: false, finish_time: 0.0,
            best_lap: f32::MAX, lap_start_time: 0.0,
            ground_height: track.height_at_pos(pos), air_height: 0.0, vert_vel: 0.0, just_landed: false,
            track_idx: track.nearest_info(pos).0,
        }
    }

    pub fn is_ai(&self) -> bool { self.keys.is_none() }
    pub fn forward(&self) -> Vec2 { vec2(self.angle.cos(), self.angle.sin()) }
    pub fn right(&self) -> Vec2 { vec2(-self.angle.sin(), self.angle.cos()) }
    pub fn total_height(&self) -> f32 { self.ground_height + self.air_height }
    pub fn is_airborne(&self) -> bool { self.air_height > 0.05 }

    pub fn get_player_input(&self) -> CarInput {
        if let Some((up, down, left, right)) = self.keys {
            let mut input = CarInput {
                gas: is_key_down(up), brake: is_key_down(down),
                left: is_key_down(left), right: is_key_down(right),
            };
            // Spiller 1 faar ogsaa touch-input (iPad/mobil)
            if self.number == 1 {
                let ti = get_touch_input();
                input.gas |= ti.gas;
                input.brake |= ti.brake;
                input.left |= ti.left;
                input.right |= ti.right;
            }
            input
        } else {
            CarInput { gas: false, brake: false, left: false, right: false }
        }
    }

    pub fn get_ai_input(&self, track: &Track, all_cars: &[Car]) -> CarInput {
        let (my_idx, dist_from_center, _) = track.nearest_info(self.pos);
        let n = track.center.len();
        let speed = self.vel.length();

        let lookahead = (10.0 + speed * 0.07) as usize;
        let target_idx = (my_idx + lookahead) % n;
        let ahead_point = track.center[target_idx];
        let normal = track.normal_at(target_idx);

        // Personlig kjorelinje
        let time_wobble = (get_time() as f32 * 1.5 + self.number as f32 * 2.7).sin();
        let line_offset = self.ai_line_pref * 0.3 + time_wobble * 0.1;
        let offset_target = ahead_point + normal * (track.width * line_offset);

        // Forbikjoring
        let mut dodge = Vec2::ZERO;
        for other in all_cars {
            if other.number == self.number { continue; }
            let to_other = other.pos - self.pos;
            let dist = to_other.length();
            let ahead_dot = to_other.dot(self.forward());
            if ahead_dot > 0.0 && dist < 100.0 {
                let side_dot = to_other.dot(self.right());
                let dodge_dir = if side_dot > 0.0 { -1.0 } else { 1.0 };
                let dodge_strength = (1.0 - dist / 100.0) * 45.0;
                dodge += normal * dodge_dir * dodge_strength;
            }
        }

        let target = offset_target + dodge;

        // Korreksjon mot senter
        let center_point = track.center[my_idx];
        let off_center = dist_from_center / (track.width / 2.0);
        let correction = off_center.powi(2).min(1.0);
        let target = target.lerp(center_point, correction * 0.6);

        let to_target = target - self.pos;
        let desired = to_target.y.atan2(to_target.x);
        let mut diff = desired - self.angle;
        while diff > PI { diff -= TAU; }
        while diff < -PI { diff += TAU; }

        let curve = track.curvature_ahead(my_idx, lookahead * 3);
        let target_speed = MAX_SPEED * self.ai_skill * (1.0 - curve * 2.0 * self.ai_aggression).clamp(0.5, 1.0);

        let chasing = all_cars.iter().any(|o| {
            o.number != self.number && {
                let d = o.pos - self.pos;
                d.dot(self.forward()) > 0.0 && d.length() < 80.0
            }
        });
        let speed_boost = if chasing { 1.08 } else { 1.0 };
        let off_track = !track.point_on_track(self.pos);
        let steer_threshold = 0.05 + (get_time() as f32 * 2.0 + self.number as f32).sin() * 0.03;

        // Sjekk for motende/kryssende trafikk (viktig i figur-8)
        let mut crossing_threat = false;
        for other in all_cars {
            if other.number == self.number { continue; }
            let to_other = other.pos - self.pos;
            let dist_o = to_other.length();
            if dist_o > 120.0 || dist_o < 1.0 { continue; }
            let angle_between = self.forward().dot(other.forward()).clamp(-1.0, 1.0).acos();
            if angle_between > PI / 3.0 {
                let ahead = to_other.normalize().dot(self.forward());
                if ahead > 0.2 && dist_o < 80.0 {
                    crossing_threat = true;
                    break;
                }
            }
        }

        CarInput {
            gas: (speed < target_speed * speed_boost || speed < MAX_SPEED * 0.6) && !crossing_threat,
            brake: (speed > target_speed * 1.4 && curve > 0.15) || (off_track && speed > 150.0) || crossing_threat,
            left: diff > steer_threshold,
            right: diff < -steer_threshold,
        }
    }

    pub fn update(
        &mut self, dt: f32, track: &Track,
        skids: &mut SkidBuffer, particles: &mut Vec<Particle>,
        race_time: f32, can_move: bool, all_cars: &[Car],
    ) {
        self.just_landed = false;

        if self.finished {
            self.vel *= 0.98;
            self.pos += self.vel * dt;
            return;
        }

        let input = if !can_move {
            CarInput { gas: false, brake: false, left: false, right: false }
        } else if self.is_ai() {
            self.get_ai_input(track, all_cars)
        } else {
            self.get_player_input()
        };

        let airborne = self.is_airborne();
        let grip_mult = if airborne { AIR_GRIP_MULT } else { 1.0 };
        let steer_mult = if airborne { AIR_STEER_MULT } else { 1.0 };

        let fwd = self.forward();
        let spd = self.vel.dot(fwd);

        if input.gas {
            self.vel += fwd * ACCEL * grip_mult * dt;
            if !airborne && particles.len() < MAX_PARTICLES && rand::gen_range(0.0, 1.0) < 0.3 {
                let back = self.pos - fwd * 14.0;
                let side = self.right();
                particles.push(Particle {
                    pos: back + side * rand::gen_range(-4.0, 4.0),
                    vel: -fwd * rand::gen_range(20.0, 50.0) + vec2(rand::gen_range(-10.0, 10.0), rand::gen_range(-10.0, 10.0)),
                    life: 0.4, max_life: 0.4,
                    size: rand::gen_range(2.0, 4.0),
                    color: Color::new(0.5, 0.5, 0.5, 0.4),
                });
            }
        }

        if input.brake {
            if spd > 10.0 {
                self.vel -= fwd * BRAKE * grip_mult * dt;
                if !airborne && spd > 100.0 && particles.len() < MAX_PARTICLES {
                    particles.push(Particle {
                        pos: self.pos - fwd * 12.0,
                        vel: vec2(rand::gen_range(-15.0, 15.0), rand::gen_range(-15.0, 15.0)),
                        life: 0.5, max_life: 0.5,
                        size: rand::gen_range(3.0, 6.0),
                        color: Color::new(0.7, 0.7, 0.7, 0.3),
                    });
                }
            } else {
                self.vel -= fwd * ACCEL * 0.5 * grip_mult * dt;
                if self.vel.dot(self.forward()) < -REVERSE_MAX {
                    self.vel = fwd * (-REVERSE_MAX);
                }
            }
        }

        let turn_factor = (spd.abs() / MAX_SPEED).clamp(0.15, 1.0);
        if input.left { self.angle += TURN_SPEED * turn_factor * steer_mult * dt; }
        if input.right { self.angle -= TURN_SPEED * turn_factor * steer_mult * dt; }

        let fwd = self.forward();
        let side = self.right();
        let forward_speed = self.vel.dot(fwd);
        let side_speed = self.vel.dot(side);

        let drift_ratio = if self.vel.length() > 20.0 {
            side_speed.abs() / self.vel.length()
        } else { 0.0 };
        self.is_drifting = !airborne && drift_ratio > SKID_DRIFT_THRESHOLD;

        if self.is_drifting && track.point_on_track(self.pos) {
            let back = self.pos - fwd * 10.0;
            skids.push(SkidMark { pos: back + side * 6.0, alpha: 0.5 });
            skids.push(SkidMark { pos: back - side * 6.0, alpha: 0.5 });
        }

        let grip_rate = if self.is_drifting { 2.0 } else if airborne { 0.5 } else { 5.0 };
        let grip = (-grip_rate * dt).exp();
        self.vel = fwd * forward_speed + side * side_speed * grip;

        // Terreng: helningseffekter (kun paa bakken)
        // Bruk hint for aa unngaa segment-hopping ved kryss
        let (nearest_idx, _, _) = track.nearest_info_hint(self.pos, Some(self.track_idx));
        self.track_idx = nearest_idx;

        if !airborne {
            let n = track.center.len();
            let h_behind = track.height[(nearest_idx + n - SLOPE_LOOK) % n];
            let h_ahead = track.height[(nearest_idx + SLOPE_LOOK) % n];
            let slope = h_ahead - h_behind;
            let track_fwd = track.tangent_at(nearest_idx);
            let going_forward = self.vel.dot(track_fwd) > 0.0;
            let effective_slope = if going_forward { slope } else { -slope };
            let speed = self.vel.length();
            if effective_slope > 0.01 && speed > 10.0 {
                // Oppoverbakke: maks 10% fartstap per frame
                let decel = effective_slope * SLOPE_FORCE * dt;
                self.vel -= self.vel.normalize() * decel.min(speed * 0.1);
            } else if effective_slope < -0.01 && speed > 10.0 {
                // Nedoverbakke: boost (capped saa man ikke overgaar MAX_SPEED for mye)
                let accel = effective_slope.abs() * SLOPE_FORCE * 0.7 * dt;
                self.vel += self.vel.normalize() * accel;
            }
        }

        if self.vel.length() > MAX_SPEED {
            self.vel = self.vel.normalize() * MAX_SPEED;
        }

        self.pos += self.vel * dt;
        self.vel *= (-0.12 * dt).exp();

        // Oppdater track_idx etter posisjonsflytt
        let (new_idx, _, new_t) = track.nearest_info_hint(self.pos, Some(self.track_idx));
        self.track_idx = new_idx;

        // Terreng: oppdater bakkehoyde og sjekk for hopp
        let n = track.center.len();
        let h1 = track.height[new_idx];
        let h2 = track.height[(new_idx + 1) % n];
        let new_ground = h1 + (h2 - h1) * new_t;

        if airborne {
            self.vert_vel -= GRAVITY * dt;
            self.air_height += self.vert_vel * dt;
            if self.air_height <= 0.0 {
                self.air_height = 0.0;
                self.just_landed = true;
                self.vert_vel = 0.0;
                self.ground_height = new_ground;
                self.vel *= 0.92;
                if particles.len() < MAX_PARTICLES {
                    for _ in 0..6 {
                        particles.push(Particle {
                            pos: self.pos + vec2(rand::gen_range(-8.0, 8.0), rand::gen_range(-8.0, 8.0)),
                            vel: vec2(rand::gen_range(-25.0, 25.0), rand::gen_range(-25.0, 25.0)),
                            life: 0.4, max_life: 0.4,
                            size: rand::gen_range(3.0, 6.0),
                            color: Color::new(0.5, 0.45, 0.3, 0.5),
                        });
                    }
                }
            }
        } else {
            let look_j = SLOPE_LOOK;
            let behind = track.height[(new_idx + n - look_j) % n];
            let ahead = track.height[(new_idx + look_j) % n];
            let here = track.height[new_idx];
            let crest_strength = here - (behind + ahead) / 2.0;
            let speed = self.vel.length();
            if crest_strength > JUMP_CREST_THRESHOLD && speed > JUMP_MIN_SPEED {
                self.vert_vel = (crest_strength * JUMP_SCALE * speed).min(MAX_JUMP_VEL);
                self.air_height = 0.1; // Over is_airborne-terskelen saa luft-fysikken starter
            }
            self.ground_height = new_ground;
        }

        if !track.point_on_track(self.pos) {
            self.vel *= (-3.0 * dt).exp();
            if self.vel.length() > 50.0 && particles.len() < MAX_PARTICLES && rand::gen_range(0.0, 1.0) < 0.4 {
                particles.push(Particle {
                    pos: self.pos,
                    vel: vec2(rand::gen_range(-20.0, 20.0), rand::gen_range(-20.0, 20.0)),
                    life: 0.6, max_life: 0.6,
                    size: rand::gen_range(3.0, 7.0),
                    color: Color::new(0.4, 0.35, 0.2, 0.4),
                });
            }
        }

        // Drift: gnister + roykskyer
        if self.is_drifting && particles.len() < MAX_PARTICLES {
            let back = self.pos - fwd * 12.0;
            // Gnister
            for _ in 0..2 {
                particles.push(Particle {
                    pos: back + side * rand::gen_range(-8.0, 8.0),
                    vel: vec2(rand::gen_range(-30.0, 30.0), rand::gen_range(-40.0, -10.0)),
                    life: 0.15, max_life: 0.15,
                    size: rand::gen_range(1.0, 2.5),
                    color: if rand::gen_range(0.0, 1.0) > 0.5 { YELLOW } else { ORANGE },
                });
            }
            // Royksky (store, bløte sirkler som vokser)
            let drift_intensity = (self.vel.length() / MAX_SPEED).clamp(0.0, 1.0);
            for &tire_side in &[-6.0_f32, 6.0] {
                let tire_pos = back + side * tire_side;
                particles.push(Particle {
                    pos: tire_pos + vec2(rand::gen_range(-3.0, 3.0), rand::gen_range(-3.0, 3.0)),
                    vel: -self.vel * 0.05 + vec2(rand::gen_range(-8.0, 8.0), rand::gen_range(-8.0, 8.0)),
                    life: 0.6 + drift_intensity * 0.4, max_life: 0.6 + drift_intensity * 0.4,
                    size: 6.0 + drift_intensity * 8.0,
                    color: Color::new(0.7, 0.7, 0.7, 0.15 + drift_intensity * 0.1),
                });
            }
        }

        // Eksosflammer ved full gass og hoy fart
        if input.gas && self.vel.length() > MAX_SPEED * 0.7 && !airborne {
            let intensity = ((self.vel.length() - MAX_SPEED * 0.7) / (MAX_SPEED * 0.3)).clamp(0.0, 1.0);
            if rand::gen_range(0.0, 1.0) < intensity * 0.6 && particles.len() < MAX_PARTICLES {
                let exhaust = self.pos - fwd * 15.0;
                particles.push(Particle {
                    pos: exhaust + side * rand::gen_range(-2.0, 2.0),
                    vel: -fwd * rand::gen_range(30.0, 60.0) + vec2(rand::gen_range(-5.0, 5.0), rand::gen_range(-5.0, 5.0)),
                    life: 0.12, max_life: 0.12,
                    size: rand::gen_range(2.0, 4.0),
                    color: Color::new(1.0, 0.6 + rand::gen_range(0.0, 0.3), 0.1, 0.7),
                });
            }
        }

        // Myk vegg: gradvis brems og skyv mot banen
        let (nearest_idx, dist, _) = track.nearest_info_hint(self.pos, Some(self.track_idx));
        let half_w = track.width / 2.0;
        let max_offtrack = track.width * 1.5;
        if dist > half_w {
            let t_off = ((dist - half_w) / (max_offtrack - half_w)).clamp(0.0, 1.0);
            let target = track.center[nearest_idx];
            let pull = (target - self.pos).normalize();
            let drag = 1.0 + t_off * 4.0;
            self.vel *= (-drag * dt).exp();
            self.vel += pull * t_off * 200.0 * dt;
            if dist > max_offtrack {
                let overshoot = dist - max_offtrack;
                self.pos += pull * overshoot * 0.3 * dt;
            }
        }

        self.update_laps(track, race_time);
    }

    pub fn update_laps(&mut self, track: &Track, race_time: f32) {
        let s = track.sector(self.pos);
        if s != self.last_sector {
            self.sectors_visited[s as usize] = true;
            if self.last_sector == 3 && s == 0 && self.sectors_visited.iter().all(|&v| v) {
                self.laps += 1;
                let lap_time = race_time - self.lap_start_time;
                if lap_time < self.best_lap && self.lap_start_time > 0.0 {
                    self.best_lap = lap_time;
                }
                self.lap_start_time = race_time;
                if self.laps >= TOTAL_LAPS {
                    self.finished = true;
                    self.finish_time = race_time;
                }
                self.sectors_visited = [false; 4];
            }
            if self.last_sector == 0 && s == 3 {
                // Feil retning — nullstill sektorer uten a trekke fra runder
                self.sectors_visited = [false; 4];
            }
            self.last_sector = s;
        }
    }

    pub fn draw(&self) {
        let fwd = self.forward();
        let side = self.right();
        let hw = 14.0;
        let hh = 7.0;

        // Visuell loft: air_height lofter bilen oppover paa skjermen
        let ah = self.air_height;
        let lift = vec2(0.0, -ah * JUMP_LIFT_PX);
        let scale = 1.0 + ah * 0.12;

        let corners = [
            self.pos + (fwd * hw + side * hh) * scale + lift,
            self.pos + (fwd * hw - side * hh) * scale + lift,
            self.pos + (-fwd * hw - side * hh) * scale + lift,
            self.pos + (-fwd * hw + side * hh) * scale + lift,
        ];

        // Skygge: skarp og mork, rett under bilen paa bakken (som Super Skidmarks)
        // Liten offset nedover-hoyre for lys-illusjon
        let so = vec2(2.0, 3.0 + ah * 3.0);
        let shadow_alpha = if ah > 0.1 { 0.45 } else { 0.2 };
        let sc = [
            self.pos + (fwd * hw + side * hh) + so,
            self.pos + (fwd * hw - side * hh) + so,
            self.pos + (-fwd * hw - side * hh) + so,
            self.pos + (-fwd * hw + side * hh) + so,
        ];
        draw_triangle(sc[0], sc[1], sc[2], Color::new(0.0, 0.0, 0.0, shadow_alpha));
        draw_triangle(sc[0], sc[2], sc[3], Color::new(0.0, 0.0, 0.0, shadow_alpha));

        // Kropp
        draw_triangle(corners[0], corners[1], corners[2], self.color);
        draw_triangle(corners[0], corners[2], corners[3], self.color);

        // Tak
        let sw = 4.0;
        let s1 = self.pos + (fwd * (hw * 0.3) + side * sw) * scale + lift;
        let s2 = self.pos + (fwd * (hw * 0.3) - side * sw) * scale + lift;
        let s3 = self.pos + (-fwd * (hw * 0.5) - side * sw) * scale + lift;
        let s4 = self.pos + (-fwd * (hw * 0.5) + side * sw) * scale + lift;
        let dark = Color::new(self.color.r * 0.5, self.color.g * 0.5, self.color.b * 0.5, 1.0);
        draw_triangle(s1, s2, s3, dark);
        draw_triangle(s1, s3, s4, dark);

        // Frontrute
        let w1 = self.pos + (fwd * (hw * 0.35) + side * (sw + 1.0)) * scale + lift;
        let w2 = self.pos + (fwd * (hw * 0.35) - side * (sw + 1.0)) * scale + lift;
        let w3 = self.pos + (fwd * (hw * 0.5) - side * (sw + 1.0)) * scale + lift;
        let w4 = self.pos + (fwd * (hw * 0.5) + side * (sw + 1.0)) * scale + lift;
        draw_triangle(w1, w2, w3, Color::new(0.6, 0.8, 1.0, 0.7));
        draw_triangle(w1, w3, w4, Color::new(0.6, 0.8, 1.0, 0.7));

        // Hjul
        for &(fx, sx) in &[(10.0, 8.0), (10.0, -8.0), (-10.0, 8.0), (-10.0, -8.0)] {
            let wp = self.pos + (fwd * fx + side * sx) * scale + lift;
            draw_rectangle(wp.x - 3.0, wp.y - 2.0, 6.0, 4.0, Color::new(0.15, 0.15, 0.15, 1.0));
        }

        // Nummerprikk
        let np = self.pos + (-fwd * 2.0) * scale + lift;
        draw_circle(np.x, np.y, 3.0, WHITE);
        draw_circle(np.x, np.y, 2.0, Color::new(self.color.r * 0.3, self.color.g * 0.3, self.color.b * 0.3, 1.0));

        // Frontlykter (to gule prikker)
        let speed = self.vel.length();
        let headlight_alpha = 0.5 + (speed / MAX_SPEED) * 0.5;
        for &s in &[-4.5_f32, 4.5] {
            let hp = self.pos + (fwd * (hw + 1.0) + side * s) * scale + lift;
            draw_circle(hp.x, hp.y, 2.0, Color::new(1.0, 0.95, 0.6, headlight_alpha));
            // Lys-glow
            draw_circle(hp.x, hp.y, 4.0, Color::new(1.0, 0.95, 0.6, headlight_alpha * 0.15));
        }

        // Baklys (rod stripe ved bremsing eller lav fart)
        let braking = speed > 10.0 && {
            let fwd_vel = self.vel.dot(fwd);
            fwd_vel < speed * 0.5 || (self.keys.is_some() && is_key_down(self.keys.unwrap().1))
        };
        if braking {
            for &s in &[-5.0_f32, 5.0] {
                let tp = self.pos + (-fwd * (hw + 0.5) + side * s) * scale + lift;
                draw_circle(tp.x, tp.y, 2.5, Color::new(1.0, 0.1, 0.0, 0.9));
                draw_circle(tp.x, tp.y, 5.0, Color::new(1.0, 0.0, 0.0, 0.2));
            }
        }
    }

    pub fn draw_speed_lines(&self) {
        let speed = self.vel.length();
        if speed > MAX_SPEED * 0.7 {
            let intensity = ((speed - MAX_SPEED * 0.7) / (MAX_SPEED * 0.3)).clamp(0.0, 1.0);
            let fwd = self.forward();
            let side = self.right();
            for i in 0..4 {
                let offset = side * ((i as f32 - 1.5) * 8.0);
                let start = self.pos - fwd * 18.0 + offset;
                let end = start - fwd * (20.0 + intensity * 25.0);
                draw_line(start.x, start.y, end.x, end.y, 1.0, Color::new(1.0, 1.0, 1.0, intensity * 0.3));
            }
        }
    }
}

// ── Kollisjon (iterativ med tangentiell friksjon) ──
/// Returnerer total kollisjonsintensitet for screen shake
pub fn collide_cars(cars: &mut [Car], particles: &mut Vec<Particle>) -> f32 {
    let n = cars.len();
    let mut total_impact = 0.0_f32;
    for _ in 0..3 {
        for i in 0..n {
            for j in (i + 1)..n {
                let h_diff = (cars[i].total_height() - cars[j].total_height()).abs();
                if h_diff > HEIGHT_COL_GAP { continue; }
                let diff = cars[i].pos - cars[j].pos;
                let dist = diff.length();
                let min_dist = CAR_RADIUS * 2.0;
                if dist < min_dist && dist > 0.001 {
                    let normal = diff / dist;
                    let tangent = vec2(-normal.y, normal.x);
                    let overlap = min_dist - dist;
                    cars[i].pos += normal * (overlap * 0.5 + 0.5);
                    cars[j].pos -= normal * (overlap * 0.5 + 0.5);
                    let rel_vel = cars[i].vel - cars[j].vel;
                    let van = rel_vel.dot(normal);
                    if van < 0.0 {
                        let impact = van.abs();
                        total_impact = total_impact.max(impact * 0.02);
                        let impulse_n = normal * van * (1.0 + BOUNCE) * 0.5;
                        cars[i].vel -= impulse_n;
                        cars[j].vel += impulse_n;
                        let vat = rel_vel.dot(tangent);
                        let impulse_t = tangent * vat * 0.15;
                        cars[i].vel -= impulse_t;
                        cars[j].vel += impulse_t;
                        // Kollisjonsgnister
                        if impact > 30.0 {
                            let cp = (cars[i].pos + cars[j].pos) / 2.0;
                            for _ in 0..(impact * 0.1) as usize {
                                if particles.len() < MAX_PARTICLES {
                                    particles.push(Particle {
                                        pos: cp + vec2(rand::gen_range(-4.0, 4.0), rand::gen_range(-4.0, 4.0)),
                                        vel: vec2(rand::gen_range(-60.0, 60.0), rand::gen_range(-60.0, 60.0)),
                                        life: 0.3, max_life: 0.3,
                                        size: rand::gen_range(1.5, 3.0),
                                        color: if rand::gen_range(0.0, 1.0) > 0.5 { YELLOW } else { ORANGE },
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    total_impact
}
