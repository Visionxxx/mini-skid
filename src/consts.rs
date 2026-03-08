// ── Skjerm ──
pub const SCREEN_W: f32 = 1024.0;
pub const SCREEN_H: f32 = 768.0;

// ── Fysikk ──
pub const MAX_SPEED: f32 = 450.0;
pub const ACCEL: f32 = 320.0;
pub const BRAKE: f32 = 380.0;
pub const REVERSE_MAX: f32 = 120.0;
pub const TURN_SPEED: f32 = 3.2;
pub const BOUNCE: f32 = 0.4;
pub const CAR_RADIUS: f32 = 12.0;

// ── Skidmerker ──
pub const MAX_SKIDS: usize = 8000;
pub const SKID_DRIFT_THRESHOLD: f32 = 0.30;

// ── Race ──
pub const TOTAL_LAPS: i32 = 5;
pub const COUNTDOWN_TIME: f32 = 3.0;
pub const MAX_PARTICLES: usize = 1200;

// ── Terreng ──
pub const GRAVITY: f32 = 14.0;
pub const JUMP_SCALE: f32 = 0.025;
pub const MAX_JUMP_VEL: f32 = 5.0;
pub const JUMP_CREST_THRESHOLD: f32 = 0.06;
pub const JUMP_MIN_SPEED: f32 = 140.0;
pub const AIR_GRIP_MULT: f32 = 0.15;
pub const AIR_STEER_MULT: f32 = 0.3;
pub const SLOPE_LOOK: usize = 6;
pub const SLOPE_FORCE: f32 = 120.0;
pub const HEIGHT_COL_GAP: f32 = 1.5;
pub const BRIDGE_HEIGHT: f32 = 1.5;
pub const JUMP_LIFT_PX: f32 = 50.0;

// ── Kamera ──
pub const CAM_SMOOTH: f32 = 4.0;
pub const CAM_ZOOM_MIN: f32 = 0.55;
pub const CAM_ZOOM_MAX: f32 = 0.85;
pub const CAM_ZOOM_SMOOTH: f32 = 1.5;
