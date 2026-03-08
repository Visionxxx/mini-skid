use macroquad::prelude::*;
use macroquad::window::get_internal_gl;
use crate::car::Car;
use crate::consts::*;

pub struct GameCamera {
    pub pos: Vec2,
    pub zoom: f32,
    pub shake: f32,
}

impl GameCamera {
    pub fn new(pos: Vec2) -> Self {
        Self { pos, zoom: 0.6, shake: 0.0 }
    }

    pub fn add_shake(&mut self, amount: f32) {
        self.shake = (self.shake + amount).min(12.0);
    }

    pub fn update(&mut self, dt: f32, players: &[&Car]) {
        if players.is_empty() { return; }

        let mut center = Vec2::ZERO;
        for p in players.iter() {
            center += p.pos;
        }
        center /= players.len() as f32;

        self.pos += (center - self.pos) * CAM_SMOOTH * dt;

        if players.len() >= 2 {
            let dist = players[0].pos.distance(players[1].pos);
            let target_zoom = (800.0 / (dist + 400.0)).clamp(CAM_ZOOM_MIN, CAM_ZOOM_MAX);
            self.zoom += (target_zoom - self.zoom) * CAM_ZOOM_SMOOTH * dt;
        } else {
            let target_zoom = 0.7;
            self.zoom += (target_zoom - self.zoom) * CAM_ZOOM_SMOOTH * dt;
        }

        // Shake decay
        self.shake *= (-8.0 * dt).exp();
        if self.shake < 0.1 { self.shake = 0.0; }
    }

    fn shake_offset(&self) -> Vec2 {
        if self.shake < 0.1 { return Vec2::ZERO; }
        let t = get_time() as f32 * 60.0;
        vec2(
            (t * 7.1).sin() * self.shake,
            (t * 11.3).cos() * self.shake,
        )
    }

    /// Tegn til en off-screen render target med spillkamera.
    pub fn apply(&self, rt: &RenderTarget) {
        let offset = self.shake_offset();
        set_camera(&Camera2D {
            target: self.pos + offset,
            zoom: vec2(
                self.zoom * 2.0 / rt.texture.width(),
                self.zoom * 2.0 / rt.texture.height(),
            ),
            render_target: Some(rt.clone()),
            ..Default::default()
        });
    }
}

/// Tegn render-target til skjermen og bytt til skjermkoordinater for HUD.
pub fn flush_world_to_screen(rt: &RenderTarget) {
    set_default_camera();
    draw_texture_ex(
        &rt.texture,
        0., 0.,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(screen_width(), screen_height())),
            flip_y: true,
            ..Default::default()
        },
    );
    // Tving flush av RT-tekstur-tegningen foer HUD-tekst starter,
    // slik at font-atlasets tekstur ikke kolliderer med RT-teksturen.
    unsafe { get_internal_gl().flush(); }
}

/// Forvarm font-atlas med alle tegn og storrelser brukt i spillet.
/// Kall dette en gang ved oppstart for aa unngaa atlas-resize under gameplay.
pub fn prewarm_font_atlas() {
    let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.:!|/>=<#() -";
    for &size in &[13u16, 14, 15, 16, 18, 20, 22, 24, 26, 28, 30, 42, 60, 72, 80] {
        draw_text_ex(chars, -10000.0, -10000.0, TextParams {
            font_size: size,
            font_scale: 1.0,
            color: Color::new(0.0, 0.0, 0.0, 0.0),
            ..Default::default()
        });
    }
    unsafe { get_internal_gl().flush(); }
}
