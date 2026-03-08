use macroquad::prelude::*;
use macroquad::input::{touches, TouchPhase};
use crate::car::{Car, touch_button_rects, get_touch_input};
use crate::consts::*;
use crate::particle::SkidBuffer;
use crate::track::Track;

#[derive(PartialEq)]
pub enum MenuAction {
    None,
    Start,
    TogglePlayers,
    SelectTrack(usize),
}

#[derive(PartialEq)]
pub enum InGameAction {
    None,
    Pause,
}

#[derive(PartialEq)]
pub enum PauseAction {
    None,
    Resume,
    Menu,
}

#[derive(PartialEq)]
pub enum FinishAction {
    None,
    Retry,
    Menu,
}

fn touch_just_pressed() -> Vec<Vec2> {
    touches().into_iter()
        .filter(|t| t.phase == TouchPhase::Started)
        .map(|t| t.position)
        .collect()
}

fn draw_button(x: f32, y: f32, w: f32, h: f32, label: &str, bg: Color, text_color: Color) {
    draw_rectangle(x, y, w, h, bg);
    draw_rectangle_lines(x, y, w, h, 2.0, Color::new(text_color.r, text_color.g, text_color.b, 0.5));
    let text_w = label.len() as f32 * 9.0;
    draw_text(label, x + w / 2.0 - text_w / 2.0, y + h / 2.0 + 7.0, 24.0, text_color);
}

fn point_in_rect(px: f32, py: f32, rx: f32, ry: f32, rw: f32, rh: f32) -> bool {
    px >= rx && px <= rx + rw && py >= ry && py <= ry + rh
}

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

pub fn draw_pause_button() -> InGameAction {
    let bx = screen_width() - 60.0;
    let by = 55.0;
    let bw = 52.0;
    let bh = 36.0;
    draw_rectangle(bx, by, bw, bh, Color::new(0.15, 0.15, 0.25, 0.7));
    draw_rectangle_lines(bx, by, bw, bh, 1.5, Color::new(1.0, 1.0, 1.0, 0.3));
    // Pause-ikon (to streker)
    draw_rectangle(bx + 17.0, by + 8.0, 6.0, 20.0, Color::new(1.0, 1.0, 1.0, 0.6));
    draw_rectangle(bx + 29.0, by + 8.0, 6.0, 20.0, Color::new(1.0, 1.0, 1.0, 0.6));

    let mut action = InGameAction::None;
    for p in touch_just_pressed() {
        if point_in_rect(p.x, p.y, bx, by, bw, bh) { action = InGameAction::Pause; }
    }
    if is_mouse_button_pressed(MouseButton::Left) && touch_just_pressed().is_empty() {
        let (mx, my) = mouse_position();
        if point_in_rect(mx, my, bx, by, bw, bh) { action = InGameAction::Pause; }
    }
    action
}

pub fn draw_finish_screen(cars: &[Car], positions: &[usize], race_time: f32) -> FinishAction {
    let cx = screen_width() / 2.0;
    draw_rectangle(cx - 250.0, 100.0, 500.0, 470.0, Color::new(0.0, 0.0, 0.0, 0.85));
    draw_rectangle_lines(cx - 250.0, 100.0, 500.0, 470.0, 3.0, Color::new(1.0, 0.8, 0.0, 1.0));
    draw_text("RACE FERDIG!", cx - 110.0, 150.0, 42.0, Color::new(1.0, 0.8, 0.0, 1.0));

    let mins = (race_time / 60.0) as u32;
    let secs = race_time % 60.0;
    draw_text(&format!("Tid: {:02}:{:05.2}", mins, secs), cx - 70.0, 180.0, 20.0, LIGHTGRAY);

    for (rank, &car_idx) in positions.iter().enumerate() {
        let car = &cars[car_idx];
        let y = 220.0 + rank as f32 * 50.0;
        let medal = if rank == 0 { ">> " } else { "   " };
        let color = if rank == 0 { Color::new(1.0, 0.85, 0.0, 1.0) } else { car.color };
        let size = if rank == 0 { 28.0 } else { 22.0 };
        draw_text(&format!("{}{}.  #{} {}", medal, rank + 1, car.number, car.name), cx - 180.0, y, size, color);
        if car.best_lap < f32::MAX {
            let bm = (car.best_lap / 60.0) as u32;
            let bs = car.best_lap % 60.0;
            draw_text(&format!("Beste runde: {:02}:{:05.2}", bm, bs), cx - 180.0, y + 18.0, 14.0, LIGHTGRAY);
        }
    }

    // Touch-knapper: Omkjoring og Meny
    let btn_y = 500.0;
    let btn_h = 42.0;
    let retry_x = cx - 200.0;
    let retry_w = 180.0;
    let menu_x = cx + 20.0;
    let menu_w = 180.0;
    draw_button(retry_x, btn_y, retry_w, btn_h, "OMKJORING (R)", Color::new(0.1, 0.3, 0.1, 0.9), Color::new(0.2, 1.0, 0.2, 1.0));
    draw_button(menu_x, btn_y, menu_w, btn_h, "MENY (Enter)", Color::new(0.3, 0.1, 0.1, 0.9), Color::new(1.0, 0.6, 0.3, 1.0));

    let mut action = FinishAction::None;
    let taps = touch_just_pressed();
    let mouse_tap = is_mouse_button_pressed(MouseButton::Left) && taps.is_empty();
    let all_taps: Vec<(f32, f32)> = if mouse_tap {
        let (mx, my) = mouse_position();
        vec![(mx, my)]
    } else {
        taps.iter().map(|p| (p.x, p.y)).collect()
    };
    for (px, py) in all_taps {
        if point_in_rect(px, py, retry_x, btn_y, retry_w, btn_h) { action = FinishAction::Retry; break; }
        if point_in_rect(px, py, menu_x, btn_y, menu_w, btn_h) { action = FinishAction::Menu; break; }
    }
    action
}

pub fn draw_pause_overlay() -> PauseAction {
    let cx = screen_width() / 2.0;
    let cy = screen_height() / 2.0;
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.5));
    draw_text("PAUSE", cx - 70.0, cy - 30.0, 60.0, WHITE);

    let btn_h = 42.0;
    let resume_w = 200.0;
    let resume_x = cx - resume_w / 2.0;
    let resume_y = cy + 10.0;
    draw_button(resume_x, resume_y, resume_w, btn_h, "FORTSETT (P)", Color::new(0.1, 0.3, 0.1, 0.9), Color::new(0.2, 1.0, 0.2, 1.0));

    let menu_w = 200.0;
    let menu_x = cx - menu_w / 2.0;
    let menu_y = resume_y + btn_h + 12.0;
    draw_button(menu_x, menu_y, menu_w, btn_h, "MENY (ESC)", Color::new(0.3, 0.1, 0.1, 0.9), Color::new(1.0, 0.6, 0.3, 1.0));

    draw_text("P / Space = Fortsett  |  ESC = Meny", cx - 155.0, menu_y + btn_h + 24.0, 16.0, Color::new(1.0, 1.0, 1.0, 0.3));

    let mut action = PauseAction::None;
    let taps = touch_just_pressed();
    let mouse_tap = is_mouse_button_pressed(MouseButton::Left) && taps.is_empty();
    let all_taps: Vec<(f32, f32)> = if mouse_tap {
        let (mx, my) = mouse_position();
        vec![(mx, my)]
    } else {
        taps.iter().map(|p| (p.x, p.y)).collect()
    };
    for (px, py) in all_taps {
        if point_in_rect(px, py, resume_x, resume_y, resume_w, btn_h) { action = PauseAction::Resume; break; }
        if point_in_rect(px, py, menu_x, menu_y, menu_w, btn_h) { action = PauseAction::Menu; break; }
    }
    action
}

pub fn draw_touch_controls() {
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

pub fn draw_menu(tracks: &[Track], selected: usize, num_players: usize) -> MenuAction {
    clear_background(Color::new(0.03, 0.03, 0.1, 1.0));
    let t = get_time() as f32;
    let sw = screen_width();
    let cx = sw / 2.0;
    for i in 0..20 {
        for j in 0..15 {
            let x = i as f32 * 55.0 + (t * 20.0) % 55.0;
            let y = j as f32 * 55.0;
            let alpha = 0.03 + ((i + j) as f32 * 0.5 + t).sin().abs() * 0.02;
            draw_rectangle_lines(x, y, 50.0, 50.0, 1.0, Color::new(0.3, 0.3, 0.5, alpha));
        }
    }

    let glow = (t * 2.0).sin() * 0.15 + 0.85;
    draw_text("MINI SKID", cx - 155.0, 110.0, 72.0, Color::new(glow, glow * 0.8, 0.0, 1.0));
    draw_text("Super Skidmarks!", cx - 105.0, 148.0, 28.0, Color::new(0.8, 0.5, 0.0, 0.8));
    draw_text("Velg bane:", cx - 50.0, 210.0, 24.0, WHITE);

    let card_w = 440.0;
    let card_h = 70.0;

    for (i, track) in tracks.iter().enumerate() {
        let y = 260.0 + i as f32 * 90.0;
        let sel = i == selected;
        let card_x = cx - card_w / 2.0;
        let card_y = y - 30.0;
        if sel {
            let pulse = (t * 3.0).sin() * 0.1 + 0.9;
            draw_rectangle(card_x, card_y, card_w, card_h, Color::new(0.15 * pulse, 0.15 * pulse, 0.3 * pulse, 1.0));
            draw_rectangle_lines(card_x, card_y, card_w, card_h, 2.0, Color::new(1.0, 0.8, 0.0, pulse));
        } else {
            draw_rectangle(card_x, card_y, card_w, card_h, Color::new(0.08, 0.08, 0.15, 0.8));
            draw_rectangle_lines(card_x, card_y, card_w, card_h, 1.0, Color::new(0.4, 0.4, 0.6, 0.4));
        }
        let color = if sel { Color::new(1.0, 0.9, 0.3, 1.0) } else { LIGHTGRAY };
        draw_text(&format!("{}  {}", i + 1, track.name), cx - 120.0, y + 5.0, 30.0, color);

        // Preview
        let bounds = track.bounds();
        let pcx = cx + 150.0;
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

    // Spillertoggle-knapp
    let toggle_y = 260.0 + tracks.len() as f32 * 90.0 + 10.0;
    let toggle_w = 300.0;
    let toggle_h = 44.0;
    let toggle_x = cx - toggle_w / 2.0;
    draw_rectangle(toggle_x, toggle_y, toggle_w, toggle_h, Color::new(0.1, 0.15, 0.3, 0.9));
    draw_rectangle_lines(toggle_x, toggle_y, toggle_w, toggle_h, 2.0, Color::new(0.4, 0.6, 1.0, 0.6));
    let player_label = if num_players == 1 { "1 SPILLER  (5 AI)" } else { "2 SPILLERE  (4 AI)" };
    let plw = player_label.len() as f32 * 7.5;
    draw_text(player_label, cx - plw / 2.0, toggle_y + 28.0, 22.0, Color::new(0.6, 0.8, 1.0, 1.0));

    // START-knapp
    let start_y = toggle_y + toggle_h + 16.0;
    let start_w = 240.0;
    let start_h = 54.0;
    let start_x = cx - start_w / 2.0;
    let pulse = (t * 4.0).sin() * 0.1 + 0.9;
    draw_rectangle(start_x, start_y, start_w, start_h, Color::new(0.1 * pulse, 0.5 * pulse, 0.1 * pulse, 0.95));
    draw_rectangle_lines(start_x, start_y, start_w, start_h, 3.0, Color::new(0.2, 1.0, 0.2, pulse));
    draw_text("START", cx - 50.0, start_y + 36.0, 38.0, Color::new(0.2, 1.0, 0.2, 1.0));

    // Hjelpetekst
    let help_y = start_y + start_h + 16.0;
    if num_players == 1 {
        draw_text("Spiller 1: WASD / Touch", cx - 100.0, help_y, 16.0, Color::new(1.0, 1.0, 1.0, 0.35));
    } else {
        draw_text("Spiller 1: WASD  |  Spiller 2: Piltaster", cx - 170.0, help_y, 16.0, Color::new(1.0, 1.0, 1.0, 0.35));
    }
    draw_text("Enter = Start  |  Tab = Bytt spillere  |  ESC = Avslutt", cx - 230.0, help_y + 22.0, 14.0, Color::new(1.0, 1.0, 1.0, 0.25));

    // Touch-handling
    let mut action = MenuAction::None;
    let taps = touch_just_pressed();
    let mouse_tap = is_mouse_button_pressed(MouseButton::Left);
    let all_taps: Vec<(f32, f32)> = if mouse_tap && taps.is_empty() {
        let (mx, my) = mouse_position();
        vec![(mx, my)]
    } else {
        taps.iter().map(|p| (p.x, p.y)).collect()
    };

    for (px, py) in all_taps {
        // START-knapp
        if point_in_rect(px, py, start_x, start_y, start_w, start_h) {
            action = MenuAction::Start;
            break;
        }
        // Spillertoggle
        if point_in_rect(px, py, toggle_x, toggle_y, toggle_w, toggle_h) {
            action = MenuAction::TogglePlayers;
            break;
        }
        // Banekort
        for (i, _) in tracks.iter().enumerate() {
            let card_y = 260.0 + i as f32 * 90.0 - 30.0;
            if point_in_rect(px, py, cx - card_w / 2.0, card_y, card_w, card_h) {
                action = MenuAction::SelectTrack(i);
                break;
            }
        }
    }

    action
}
