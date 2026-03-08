use macroquad::prelude::*;
use crate::car::{Car, touch_button_rects, get_touch_input};
use crate::consts::*;
use crate::particle::SkidBuffer;
use crate::track::Track;

pub fn draw_skidmarks(skids: &SkidBuffer) {
    for s in skids.iter() {
        if s.alpha > 0.01 {
            draw_circle(s.pos.x, s.pos.y, 2.0, Color::new(0.08, 0.08, 0.08, s.alpha));
        }
    }
}

pub fn draw_hud(cars: &[Car], positions: &[usize], race_time: f32, track_name: &str) {
    draw_rectangle(0.0, 0.0, screen_width(), 50.0, Color::new(0.15, 0.15, 0.22, 1.0));

    draw_text("MINI SKID", screen_width() / 2.0 - 65.0, 20.0, 26.0, Color::new(1.0, 0.8, 0.0, 1.0));
    draw_text(track_name, screen_width() / 2.0 - 45.0, 36.0, 14.0, LIGHTGRAY);

    let mins = (race_time / 60.0) as u32;
    let secs = race_time % 60.0;
    draw_text(&format!("{:02}:{:05.2}", mins, secs), screen_width() / 2.0 - 30.0, 48.0, 14.0, LIGHTGRAY);

    // Spiller 1
    let p1_pos = positions.iter().position(|&i| i == 0).unwrap_or(0) + 1;
    draw_rectangle(10.0, 5.0, 6.0, 40.0, cars[0].color);
    draw_text(&cars[0].name, 22.0, 18.0, 18.0, cars[0].color);
    let s = match p1_pos { 1 => "st", 2 => "nd", 3 => "rd", _ => "th" };
    draw_text(&format!("{}{} | Runde {}/{}", p1_pos, s, cars[0].laps.max(0), TOTAL_LAPS), 22.0, 33.0, 15.0, WHITE);
    draw_text(&format!("{:.0} km/t", cars[0].vel.length()), 22.0, 46.0, 13.0, LIGHTGRAY);
    if cars[0].is_drifting { draw_text("DRIFT!", 140.0, 46.0, 13.0, YELLOW); }

    // Spiller 2 (bare hvis human)
    if !cars[1].is_ai() {
        let p2_pos = positions.iter().position(|&i| i == 1).unwrap_or(1) + 1;
        let rx = screen_width() - 220.0;
        draw_rectangle(rx, 5.0, 6.0, 40.0, cars[1].color);
        draw_text(&cars[1].name, rx + 12.0, 18.0, 18.0, cars[1].color);
        let s = match p2_pos { 1 => "st", 2 => "nd", 3 => "rd", _ => "th" };
        draw_text(&format!("{}{} | Runde {}/{}", p2_pos, s, cars[1].laps.max(0), TOTAL_LAPS), rx + 12.0, 33.0, 15.0, WHITE);
        draw_text(&format!("{:.0} km/t", cars[1].vel.length()), rx + 12.0, 46.0, 13.0, LIGHTGRAY);
        if cars[1].is_drifting { draw_text("DRIFT!", rx + 140.0, 46.0, 13.0, YELLOW); }
    }

    // Posisjonsliste
    draw_rectangle(3.0, 55.0, 140.0, 20.0 + cars.len() as f32 * 16.0, Color::new(0.1, 0.1, 0.15, 1.0));
    draw_text("POSISJONER", 8.0, 70.0, 14.0, Color::new(1.0, 0.8, 0.0, 0.8));
    for (rank, &car_idx) in positions.iter().enumerate() {
        let car = &cars[car_idx];
        let y = 86.0 + rank as f32 * 16.0;
        draw_text(&format!("{}. #{} {}", rank + 1, car.number, car.name), 8.0, y, 13.0, car.color);
    }
}

pub fn draw_countdown(time_left: f32) {
    // Bruker fast font_size med font_scale for aa unngaa at font-atlaset
    // sprenger seg med dusinvis av unike storrelser per frame.
    let base_size: u16 = 80;
    if time_left <= 0.0 {
        let alpha = (1.0 + time_left).clamp(0.0, 1.0);
        if alpha > 0.0 {
            let scale = 1.0 + (1.0 - alpha) * 0.5;
            draw_text_ex("GO!", screen_width() / 2.0 - 60.0 * scale, screen_height() / 2.0, TextParams {
                font_size: base_size,
                font_scale: scale,
                color: Color::new(0.0, 1.0, 0.0, alpha),
                ..Default::default()
            });
        }
    } else {
        let num = time_left.ceil() as i32;
        let frac = time_left - time_left.floor();
        let scale = 1.0 + frac * 0.3;
        let color = match num { 3 => RED, 2 => YELLOW, _ => Color::new(0.0, 1.0, 0.0, 1.0) };
        draw_text_ex(&num.to_string(), screen_width() / 2.0 - 20.0 * scale, screen_height() / 2.0, TextParams {
            font_size: base_size,
            font_scale: scale,
            color,
            ..Default::default()
        });
    }
}

pub fn draw_finish_screen(cars: &[Car], positions: &[usize], race_time: f32) {
    draw_rectangle(screen_width() / 2.0 - 250.0, 100.0, 500.0, 450.0, Color::new(0.0, 0.0, 0.0, 0.85));
    draw_rectangle_lines(screen_width() / 2.0 - 250.0, 100.0, 500.0, 450.0, 3.0, Color::new(1.0, 0.8, 0.0, 1.0));
    draw_text("RACE FERDIG!", screen_width() / 2.0 - 110.0, 150.0, 42.0, Color::new(1.0, 0.8, 0.0, 1.0));

    let mins = (race_time / 60.0) as u32;
    let secs = race_time % 60.0;
    draw_text(&format!("Tid: {:02}:{:05.2}", mins, secs), screen_width() / 2.0 - 70.0, 180.0, 20.0, LIGHTGRAY);

    for (rank, &car_idx) in positions.iter().enumerate() {
        let car = &cars[car_idx];
        let y = 220.0 + rank as f32 * 50.0;
        let medal = if rank == 0 { ">> " } else { "   " };
        let color = if rank == 0 { Color::new(1.0, 0.85, 0.0, 1.0) } else { car.color };
        let size = if rank == 0 { 28.0 } else { 22.0 };
        draw_text(&format!("{}{}.  #{} {}", medal, rank + 1, car.number, car.name), screen_width() / 2.0 - 180.0, y, size, color);
        if car.best_lap < f32::MAX {
            let bm = (car.best_lap / 60.0) as u32;
            let bs = car.best_lap % 60.0;
            draw_text(&format!("Beste runde: {:02}:{:05.2}", bm, bs), screen_width() / 2.0 - 180.0, y + 18.0, 14.0, LIGHTGRAY);
        }
    }
    draw_text("ENTER = Meny  |  R = Omkjoring", screen_width() / 2.0 - 130.0, 510.0, 18.0, Color::new(1.0, 1.0, 1.0, 0.6));
}

pub fn draw_touch_controls() {
    // Sjekk om det er aktive touch-events — vis kun da
    use macroquad::input::{touches, TouchPhase};
    let active_touches: Vec<_> = touches().into_iter()
        .filter(|t| matches!(t.phase, TouchPhase::Started | TouchPhase::Stationary | TouchPhase::Moved))
        .collect();
    let has_touch = !active_touches.is_empty();

    let btns = touch_button_rects();
    let labels = ["<", ">", "BRK", "GAS"];
    let colors = [
        Color::new(1.0, 1.0, 1.0, 0.12),
        Color::new(1.0, 1.0, 1.0, 0.12),
        Color::new(1.0, 0.3, 0.2, 0.12),
        Color::new(0.2, 1.0, 0.3, 0.12),
    ];
    let active_colors = [
        Color::new(1.0, 1.0, 1.0, 0.35),
        Color::new(1.0, 1.0, 1.0, 0.35),
        Color::new(1.0, 0.3, 0.2, 0.35),
        Color::new(0.2, 1.0, 0.3, 0.35),
    ];

    let ti = get_touch_input();
    let pressed = [ti.left, ti.right, ti.brake, ti.gas];

    for (i, &(bx, by, bw, bh)) in btns.iter().enumerate() {
        let base_alpha = if has_touch { 1.0 } else { 0.5 };
        let col = if pressed[i] { active_colors[i] } else { colors[i] };
        let col = Color::new(col.r, col.g, col.b, col.a * base_alpha);
        draw_rectangle(bx, by, bw, bh, col);
        draw_rectangle_lines(bx, by, bw, bh, 2.0, Color::new(1.0, 1.0, 1.0, 0.15 * base_alpha));
        let lw = labels[i].len() as f32 * 8.0;
        draw_text(labels[i], bx + bw / 2.0 - lw / 2.0, by + bh / 2.0 + 8.0, 28.0,
            Color::new(1.0, 1.0, 1.0, 0.4 * base_alpha));
    }
}

pub fn draw_menu(tracks: &[Track], selected: usize, num_players: usize) {
    clear_background(Color::new(0.03, 0.03, 0.1, 1.0));
    let t = get_time() as f32;
    for i in 0..20 {
        for j in 0..15 {
            let x = i as f32 * 55.0 + (t * 20.0) % 55.0;
            let y = j as f32 * 55.0;
            let alpha = 0.03 + ((i + j) as f32 * 0.5 + t).sin().abs() * 0.02;
            draw_rectangle_lines(x, y, 50.0, 50.0, 1.0, Color::new(0.3, 0.3, 0.5, alpha));
        }
    }

    let glow = (t * 2.0).sin() * 0.15 + 0.85;
    draw_text("MINI SKID", screen_width() / 2.0 - 155.0, 110.0, 72.0, Color::new(glow, glow * 0.8, 0.0, 1.0));
    draw_text("Super Skidmarks!", screen_width() / 2.0 - 105.0, 148.0, 28.0, Color::new(0.8, 0.5, 0.0, 0.8));
    draw_text("Velg bane:", screen_width() / 2.0 - 50.0, 210.0, 24.0, WHITE);

    for (i, track) in tracks.iter().enumerate() {
        let y = 260.0 + i as f32 * 90.0;
        let sel = i == selected;
        if sel {
            let pulse = (t * 3.0).sin() * 0.1 + 0.9;
            draw_rectangle(screen_width() / 2.0 - 220.0, y - 30.0, 440.0, 70.0, Color::new(0.15 * pulse, 0.15 * pulse, 0.3 * pulse, 1.0));
            draw_rectangle_lines(screen_width() / 2.0 - 220.0, y - 30.0, 440.0, 70.0, 2.0, Color::new(1.0, 0.8, 0.0, pulse));
        }
        let color = if sel { Color::new(1.0, 0.9, 0.3, 1.0) } else { LIGHTGRAY };
        draw_text(&format!("{}  {}", i + 1, track.name), screen_width() / 2.0 - 120.0, y + 5.0, 30.0, color);

        // Preview
        let bounds = track.bounds();
        let pcx = screen_width() / 2.0 + 150.0;
        let pcy = y;
        let world_w = bounds.2 - bounds.0;
        let world_h = bounds.3 - bounds.1;
        let sc = 25.0 / world_w.max(world_h);
        let n = track.center.len();
        let step = (n / 50).max(1);
        let mid_x = (bounds.0 + bounds.2) / 2.0;
        let mid_y = (bounds.1 + bounds.3) / 2.0;
        for j in (0..n).step_by(step) {
            let k = (j + step) % n;
            draw_line(
                pcx + (track.center[j].x - mid_x) * sc, pcy + (track.center[j].y - mid_y) * sc,
                pcx + (track.center[k].x - mid_x) * sc, pcy + (track.center[k].y - mid_y) * sc,
                2.0, color,
            );
        }
    }

    let y = 260.0 + tracks.len() as f32 * 90.0 + 30.0;
    let player_text = if num_players == 1 {
        format!("SPILLERE: << 1 >>  (6 biler: 1 spiller + 5 AI)")
    } else {
        format!("SPILLERE: << 2 >>  (6 biler: 2 spillere + 4 AI)")
    };
    draw_text(&player_text, screen_width() / 2.0 - 200.0, y, 18.0, Color::new(0.6, 0.8, 1.0, 0.9));
    draw_text("Tab / Trykk = Bytt spillere  |  Enter = Start  |  ESC = Avslutt", screen_width() / 2.0 - 270.0, y + 30.0, 18.0, Color::new(1.0, 1.0, 1.0, 0.4));
    if num_players == 1 {
        draw_text("Spiller 1: WASD / Touch", screen_width() / 2.0 - 100.0, y + 55.0, 18.0, Color::new(1.0, 1.0, 1.0, 0.35));
    } else {
        draw_text("Spiller 1: WASD  |  Spiller 2: Piltaster", screen_width() / 2.0 - 170.0, y + 55.0, 18.0, Color::new(1.0, 1.0, 1.0, 0.35));
    }
}
