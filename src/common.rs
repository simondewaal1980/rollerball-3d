//! Shared constants, small components and input state.

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Virtual screen. The whole game is laid out on a 320x240 MSX-style canvas.
// The table occupies 224 px (70%) and the fixed HUD sidebar the other 96 px.
// ---------------------------------------------------------------------------
pub const DESIGN_W: f32 = 320.0;
pub const DESIGN_H: f32 = 240.0;
pub const HUD_W: f32 = 96.0;
pub const PLAY_W: f32 = DESIGN_W - HUD_W; // 224
pub const HUD_CENTER_X: f32 = PLAY_W + HUD_W * 0.5; // 272
pub const SCREEN_H: f32 = 240.0;
pub const TABLE_SCREENS: usize = 4;
pub const TABLE_H: f32 = SCREEN_H * TABLE_SCREENS as f32; // 960

pub const LANE_LEFT: f32 = 197.0;
pub const LANE_CENTER: f32 = 208.5;
pub const BALL_RADIUS: f32 = 5.0;
pub const BALL_START: Vec2 = Vec2::new(LANE_CENTER, 34.0);

pub const GRAVITY: f32 = 1_500.0;
pub const PHYSICS_HZ: f64 = 120.0;

// ---------------------------------------------------------------------------
// Display. The window opens at 3x the 320x240 canvas (960x720). Fullscreen is
// borderless and keeps that 4:3 canvas centred with dark bars, so the table,
// the HUD and the pixel font never stretch or shift on a widescreen monitor.
// ---------------------------------------------------------------------------
pub const WINDOW_W: f32 = 960.0;
pub const WINDOW_H: f32 = 720.0;
pub const DESIGN_ASPECT: f32 = DESIGN_W / DESIGN_H; // 4:3

#[derive(Resource, Clone, Copy, Debug)]
pub struct DisplayPrefs {
    pub fullscreen: bool,
    /// Screenshot and self-test runs must never grab the person's display.
    pub capture_run: bool,
}

impl Default for DisplayPrefs {
    fn default() -> Self {
        Self {
            fullscreen: true,
            capture_run: false,
        }
    }
}

/// The centred 4:3 rectangle inside a window, with dark bars on the long axis.
/// Returns (x, y, width, height) in physical pixels.
pub fn letterbox_rect(width: u32, height: u32) -> (u32, u32, u32, u32) {
    let w = width.max(1);
    let h = height.max(1);
    let window_aspect = w as f32 / h as f32;
    if window_aspect > DESIGN_ASPECT {
        let content_h = h;
        let content_w = (h as f32 * DESIGN_ASPECT).round() as u32;
        ((w.saturating_sub(content_w)) / 2, 0, content_w.min(w), content_h)
    } else {
        let content_w = w;
        let content_h = (w as f32 / DESIGN_ASPECT).round() as u32;
        (0, (h.saturating_sub(content_h)) / 2, content_w, content_h.min(h))
    }
}

// 0 = bottom screen ("CLOSE TO THE EDGE"), 3 = top screen.
pub const SCREEN_TITLES: [&str; 4] = ["CLOSE TO THE EDGE", "MIDDLE SCREEN", "SLOT SCREEN", "TOP SCREEN"];

// Pixel font / world text sizes.
pub const FONT_ROWS: usize = 7;

pub const LAYER_TABLE: usize = 0;
pub const LAYER_HUD: usize = 1;
/// The sidebar quad, drawn by its own always-on camera so it cannot be
/// disabled together with the menu overlay.
pub const LAYER_HUD_COMPOSITE: usize = 4;

// ---------------------------------------------------------------------------
// Input. A script (self-test / screenshot demo) can feed this exactly like a
// keyboard would, so the measured run goes through the real gameplay systems.
// ---------------------------------------------------------------------------
#[derive(Resource, Default, Debug, Clone)]
pub struct Controls {
    pub left: bool,
    pub right: bool,
    pub plunger: bool,
    pub nudge: bool,
    pub up: bool,
    pub down: bool,
    pub confirm: bool,
    pub back: bool,
    pub pause: bool,
    /// Enter, Space, a flipper key or a mouse click. Starts the game from the
    /// title screen so nobody can get stuck on the menu.
    pub start_pressed: bool,
    pub start_2p_pressed: bool,
    pub start_match_pressed: bool,
    /// Edge flags, set for one logical frame by the input source.
    pub confirm_pressed: bool,
    pub back_pressed: bool,
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub plunge_pressed: bool,
    pub plunge_released: bool,
    pub nudge_pressed: bool,
    pub pause_pressed: bool,
}

impl Controls {
    pub fn clear_edges(&mut self) {
        self.start_pressed = false;
        self.start_2p_pressed = false;
        self.start_match_pressed = false;
        self.confirm_pressed = false;
        self.back_pressed = false;
        self.up_pressed = false;
        self.down_pressed = false;
        self.plunge_pressed = false;
        self.plunge_released = false;
        self.nudge_pressed = false;
        self.pause_pressed = false;
    }
}

/// Screen band that contains a world y position (0 bottom .. 3 top).
pub fn screen_at(y: f32) -> usize {
    ((y / SCREEN_H).floor() as isize).clamp(0, TABLE_SCREENS as isize - 1) as usize
}

/// Bottom y of a screen band.
pub fn screen_bottom(screen: usize) -> f32 {
    screen as f32 * SCREEN_H
}

/// Convert a 0..3 screen number into the human readable 4..1 floor.
pub fn floor_number(screen: usize) -> usize {
    4 - screen
}

/// True when a circle overlaps an axis-aligned box.
pub fn circle_over_rect(center: Vec2, radius: f32, rect_center: Vec2, half: Vec2) -> bool {
    let d = (center - rect_center).abs() - half;
    let outside = d.max(Vec2::ZERO);
    outside.length_squared() <= radius * radius
}

#[cfg(test)]
mod display_tests {
    use super::*;

    #[test]
    fn windowed_capture_fills_the_window_exactly() {
        assert_eq!(letterbox_rect(960, 720), (0, 0, 960, 720));
    }

    #[test]
    fn widescreen_keeps_a_centred_four_by_three_picture() {
        assert_eq!(letterbox_rect(1920, 1080), (240, 0, 1440, 1080));
        assert_eq!(letterbox_rect(2560, 1080), (560, 0, 1440, 1080));
    }

    #[test]
    fn tall_screens_get_bars_on_top_and_bottom() {
        assert_eq!(letterbox_rect(1200, 1000), (0, 50, 1200, 900));
    }
}
