mod camera;
mod car;
mod consts;
mod hud;
mod particle;
mod track;

use macroquad::prelude::*;
use camera::{GameCamera, flush_world_to_screen, prewarm_font_atlas};
use car::*;
use consts::*;
use hud::*;
use particle::*;
use track::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Mini Skid".to_string(),
        window_width: SCREEN_W as i32,
        window_height: SCREEN_H as i32,
        platform: miniquad::conf::Platform {
            webgl_version: miniquad::conf::WebGLVersion::WebGL2,
            ..Default::default()
        },
        ..Default::default()
    }
}

enum GameState {
    Menu,
    Countdown(f32),
    Racing,
    Paused,
    Finished,
}

fn create_cars(track: &Track, num_players: usize) -> Vec<Car> {
    let mut cars = vec![
        Car::new_player(
            track.start_positions[0], track.start_angle,
            Color::new(0.9, 0.15, 0.1, 1.0), "SPILLER 1", 1,
            (KeyCode::W, KeyCode::S, KeyCode::A, KeyCode::D), track,
        ),
    ];
    if num_players >= 2 {
        cars.push(Car::new_player(
            track.start_positions[1], track.start_angle,
            Color::new(0.15, 0.4, 0.95, 1.0), "SPILLER 2", 2,
            (KeyCode::Up, KeyCode::Down, KeyCode::Left, KeyCode::Right), track,
        ));
    } else {
        cars.push(Car::new_ai(
            track.start_positions[1], track.start_angle,
            Color::new(0.15, 0.4, 0.95, 1.0), "FLASH", 2,
            0.96, 0.55, 0.4, track,
        ));
    }
    cars.extend([
        Car::new_ai(
            track.start_positions[2], track.start_angle,
            Color::new(0.1, 0.75, 0.2, 1.0), "TURBO", 3,
            0.98, 0.5, -0.8, track,
        ),
        Car::new_ai(
            track.start_positions[3], track.start_angle,
            Color::new(0.95, 0.85, 0.1, 1.0), "BLITZ", 4,
            0.95, 0.7, 0.9, track,
        ),
        Car::new_ai(
            track.start_positions[4], track.start_angle,
            Color::new(0.7, 0.2, 0.9, 1.0), "NITRO", 5,
            0.93, 0.3, 0.0, track,
        ),
        Car::new_ai(
            track.start_positions[5], track.start_angle,
            Color::new(0.95, 0.5, 0.1, 1.0), "RACER", 6,
            0.90, 0.6, -0.5, track,
        ),
    ]);
    cars
}

fn calc_positions(cars: &[Car], track: &Track) -> Vec<usize> {
    let mut indexed: Vec<(usize, f32)> = cars.iter().enumerate()
        .map(|(i, car)| {
            let prog = if car.finished { f32::MAX - car.finish_time } else { track.progress(car) };
            (i, prog)
        })
        .collect();
    indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    indexed.iter().map(|(i, _)| *i).collect()
}

#[macroquad::main(window_conf)]
async fn main() {
    let tracks = vec![make_oval(), make_figure8(), make_fjord()];
    let mut selected: usize = 0;
    let mut num_players: usize = 2;
    let mut state = GameState::Menu;
    let mut cars: Vec<Car> = Vec::new();
    let mut skids = SkidBuffer::new();
    let mut particles: Vec<Particle> = Vec::with_capacity(MAX_PARTICLES);
    let mut race_time: f32 = 0.0;
    let mut positions: Vec<usize> = Vec::new();
    let mut camera = GameCamera::new(vec2(1200.0, 900.0));
    let world_rt = render_target(SCREEN_W as u32, SCREEN_H as u32);
    world_rt.texture.set_filter(FilterMode::Linear);
    prewarm_font_atlas();

    loop {
        let dt = get_frame_time().min(0.033);

        match state {
            GameState::Menu => {
                if is_key_pressed(KeyCode::Escape) { break; }
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    if selected > 0 { selected -= 1; }
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    if selected < tracks.len() - 1 { selected += 1; }
                }
                // Bytt antall spillere
                if is_key_pressed(KeyCode::Tab)
                    || is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Right)
                    || is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::D)
                {
                    num_players = if num_players == 1 { 2 } else { 1 };
                }
                // Touch: trykk paa banevalg-omraadet for aa starte
                if is_mouse_button_pressed(MouseButton::Left) {
                    let (mx, my) = mouse_position();
                    // Spillertoggle-omraadet
                    let py = 260.0 + tracks.len() as f32 * 90.0 + 15.0;
                    if my >= py && my <= py + 25.0 {
                        num_players = if num_players == 1 { 2 } else { 1 };
                    }
                    // Trykk paa en bane for aa velge og starte
                    for (i, _) in tracks.iter().enumerate() {
                        let ty = 260.0 + i as f32 * 90.0 - 30.0;
                        if my >= ty && my <= ty + 70.0
                            && mx >= screen_width() / 2.0 - 220.0
                            && mx <= screen_width() / 2.0 + 220.0
                        {
                            selected = i;
                        }
                    }
                }

                let mut start = false;
                if is_key_pressed(KeyCode::Key1) { selected = 0; start = true; }
                if is_key_pressed(KeyCode::Key2) { selected = 1; start = true; }
                if is_key_pressed(KeyCode::Key3) { selected = 2; start = true; }
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) { start = true; }

                if start {
                    let track = &tracks[selected];
                    cars = create_cars(track, num_players);
                    skids.clear();
                    particles.clear();
                    race_time = 0.0;
                    positions = calc_positions(&cars, track);
                    // Sett kamera til start
                    let mid = (cars[0].pos + cars[1].pos) / 2.0;
                    camera = GameCamera::new(mid);
                    state = GameState::Countdown(COUNTDOWN_TIME);
                }

                set_default_camera();
                draw_menu(&tracks, selected, num_players);
            }

            GameState::Countdown(ref mut time_left) => {
                let track = &tracks[selected];
                *time_left -= dt;

                if *time_left <= -1.0 {
                    state = GameState::Racing;
                    next_frame().await;
                    continue;
                }
                let tl = *time_left;

                // Kamera
                let players: Vec<&Car> = cars.iter().filter(|c| !c.is_ai()).collect();
                camera.update(dt, &players);

                let snapshot = cars.clone();
                for car in cars.iter_mut() {
                    car.update(dt, track, &mut skids, &mut particles, race_time, false, &snapshot);
                }

                // Tegn verdensrom til render target
                camera.apply(&world_rt);
                clear_background(BLACK);
                track.draw();
                draw_skidmarks(&skids);
                // Biler paa bakkenivaa
                for car in &cars {
                    if car.total_height() <= BRIDGE_HEIGHT { car.draw(); }
                }
                track.draw_bridge();
                // Biler paa broen
                for car in &cars {
                    if car.total_height() > BRIDGE_HEIGHT { car.draw(); }
                }

                // HUD i skjermrom (fast paa skjermen)
                flush_world_to_screen(&world_rt);
                draw_hud(&cars, &positions, race_time, track.name);
                track.draw_minimap(&cars);
                draw_touch_controls();
                draw_countdown(tl);
            }

            GameState::Racing => {
                let track = &tracks[selected];

                if is_key_pressed(KeyCode::Escape) {
                    state = GameState::Menu;
                    next_frame().await;
                    continue;
                }
                if is_key_pressed(KeyCode::P) {
                    state = GameState::Paused;
                    next_frame().await;
                    continue;
                }
                if is_key_pressed(KeyCode::R) {
                    cars = create_cars(track, num_players);
                    skids.clear();
                    particles.clear();
                    race_time = 0.0;
                    let mid = (cars[0].pos + cars[1].pos) / 2.0;
                    camera = GameCamera::new(mid);
                    state = GameState::Countdown(COUNTDOWN_TIME);
                    next_frame().await;
                    continue;
                }

                race_time += dt;

                // Kamera folger spillerne
                let players: Vec<&Car> = cars.iter().filter(|c| !c.is_ai()).collect();
                camera.update(dt, &players);

                let snapshot = cars.clone();
                for i in 0..cars.len() {
                    let mut car = cars[i].clone();
                    car.update(dt, track, &mut skids, &mut particles, race_time, true, &snapshot);
                    cars[i] = car;
                }
                let impact = collide_cars(&mut cars, &mut particles);
                if impact > 0.5 { camera.add_shake(impact); }

                // Landing-shake
                for car in &cars {
                    if car.just_landed && !car.is_ai() {
                        camera.add_shake(3.0);
                    }
                }

                for p in particles.iter_mut() { p.update(dt); }
                particles.retain(|p| p.life > 0.0);

                skids.fade(dt);

                positions = calc_positions(&cars, track);

                let any_finished = cars.iter().any(|c| c.finished);
                if any_finished {
                    let first_time = cars.iter().filter(|c| c.finished).map(|c| c.finish_time).fold(f32::MAX, f32::min);
                    if cars.iter().all(|c| c.finished) || race_time > first_time + 15.0 {
                        for car in cars.iter_mut() {
                            if !car.finished { car.finished = true; car.finish_time = race_time; }
                        }
                        state = GameState::Finished;
                    }
                }

                // Tegn verdensrom til render target
                camera.apply(&world_rt);
                clear_background(BLACK);
                track.draw();
                draw_skidmarks(&skids);
                for p in &particles { p.draw(); }

                let mut draw_order = positions.clone();
                draw_order.reverse();
                // Biler paa bakkenivaa
                for &idx in &draw_order {
                    if cars[idx].total_height() <= BRIDGE_HEIGHT {
                        cars[idx].draw_speed_lines();
                        cars[idx].draw();
                    }
                }
                track.draw_bridge();
                // Biler paa broen
                for &idx in &draw_order {
                    if cars[idx].total_height() > BRIDGE_HEIGHT {
                        cars[idx].draw_speed_lines();
                        cars[idx].draw();
                    }
                }

                // HUD i skjermrom (fast paa skjermen)
                flush_world_to_screen(&world_rt);
                draw_hud(&cars, &positions, race_time, track.name);
                track.draw_minimap(&cars);
                draw_touch_controls();
                draw_text("WASD / Piltaster  |  P = Pause  |  R = Reset  |  ESC = Meny", screen_width() / 2.0 - 240.0, screen_height() - 10.0, 16.0, Color::new(1.0, 1.0, 1.0, 0.4));
            }

            GameState::Paused => {
                let track = &tracks[selected];

                if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Space) {
                    state = GameState::Racing;
                    next_frame().await;
                    continue;
                }
                if is_key_pressed(KeyCode::Escape) {
                    state = GameState::Menu;
                    next_frame().await;
                    continue;
                }

                // Tegn frosset scene til render target
                camera.apply(&world_rt);
                clear_background(BLACK);
                track.draw();
                draw_skidmarks(&skids);
                for p in &particles { p.draw(); }
                for car in &cars {
                    if car.total_height() <= BRIDGE_HEIGHT { car.draw(); }
                }
                track.draw_bridge();
                for car in &cars {
                    if car.total_height() > BRIDGE_HEIGHT { car.draw(); }
                }

                flush_world_to_screen(&world_rt);
                draw_hud(&cars, &positions, race_time, track.name);
                track.draw_minimap(&cars);

                // Pause-overlegg
                draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.5));
                draw_text("PAUSE", screen_width() / 2.0 - 70.0, screen_height() / 2.0 - 10.0, 60.0, WHITE);
                draw_text("P / Space = Fortsett  |  ESC = Meny", screen_width() / 2.0 - 155.0, screen_height() / 2.0 + 25.0, 20.0, LIGHTGRAY);
            }

            GameState::Finished => {
                let track = &tracks[selected];

                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Escape) {
                    state = GameState::Menu;
                    next_frame().await;
                    continue;
                }
                if is_key_pressed(KeyCode::R) {
                    cars = create_cars(track, num_players);
                    skids.clear();
                    particles.clear();
                    race_time = 0.0;
                    let mid = (cars[0].pos + cars[1].pos) / 2.0;
                    camera = GameCamera::new(mid);
                    state = GameState::Countdown(COUNTDOWN_TIME);
                    next_frame().await;
                    continue;
                }

                for car in cars.iter_mut() {
                    car.vel *= 0.98;
                    car.pos += car.vel * dt;
                }

                let players: Vec<&Car> = cars.iter().filter(|c| !c.is_ai()).collect();
                camera.update(dt, &players);

                camera.apply(&world_rt);
                clear_background(BLACK);
                track.draw();
                draw_skidmarks(&skids);
                for car in &cars {
                    if car.total_height() <= BRIDGE_HEIGHT { car.draw(); }
                }
                track.draw_bridge();
                for car in &cars {
                    if car.total_height() > BRIDGE_HEIGHT { car.draw(); }
                }

                flush_world_to_screen(&world_rt);
                draw_hud(&cars, &positions, race_time, track.name);
                draw_finish_screen(&cars, &positions, race_time);
            }
        }

        next_frame().await;
    }
}
