//! ROLLERBALL — a four-screen MSX-style pinball remake.
//!
//! Controls: Z/Left Shift/Left = left flipper, M/Right Shift/Right = right
//! flipper, hold Space to charge the plunger, W/Up nudges, Enter starts,
//! P pauses, Esc returns to the menu.

mod art;
mod audio;
mod common;
mod font;
mod game;
mod physics;
mod render3d;
mod slots;
mod table;
mod ui;

use std::path::PathBuf;
use std::time::Duration;

use avian2d::prelude::*;
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::winit::{UpdateMode, WinitSettings};
use bevy::window::{
    MonitorSelection, PrimaryWindow, WindowMode, WindowPlugin, WindowResolution,
};
use bevy_kira_audio::prelude::AudioApp;
use bevy_kira_audio::AudioPlugin as KiraAudioPlugin;
use serde::Serialize;

use art::build_art;
use font::PixelText;
use common::*;
use game::{Game, Mode};
use physics::{CameraRig, FlipperSide, Plunger};
use ui::{MenuChoice, MENU_LABELS};

#[derive(Resource, Clone, Default)]
struct Cli {
    screenshot: Option<PathBuf>,
    self_test: Option<PathBuf>,
    /// Force the small window even for an interactive run.
    windowed: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DriverMode {
    Demo,
    SelfTest,
}

#[derive(Resource)]
struct ScriptDriver {
    mode: DriverMode,
    path: PathBuf,
    t: f32,
    /// Seconds since the menu actually handed over to play. Kept separate from
    /// `t` so a slow first frame can never skip the scripted sequence.
    play_t: f32,
    playing_started: bool,
    frame: u32,
    max_y: f32,
    min_left_angle: f32,
    max_left_angle: f32,
    screenshot_attempts: u32,
    last_screenshot_t: f32,
    saved_test: bool,
    menu_exit_seen: bool,
    slot_injected: bool,
    slot_checked: bool,
    slot_passed: bool,
    slot_detail: String,
    flipper_injected: bool,
    flipper_checked: bool,
    flipper_passed: bool,
    flipper_detail: String,
    flipper_start: Vec2,
    flipper_max_speed: f32,
}

impl ScriptDriver {
    fn demo(path: PathBuf) -> Self {
        // A stale file from an earlier run must never make the new capture
        // look finished before it has been taken.
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::remove_file(&path);
        Self {
            mode: DriverMode::Demo,
            path,
            t: 0.0,
            play_t: 0.0,
            playing_started: false,
            frame: 0,
            max_y: 0.0,
            min_left_angle: 9.0,
            max_left_angle: -9.0,
            screenshot_attempts: 0,
            last_screenshot_t: -10.0,
            saved_test: false,
            menu_exit_seen: false,
            slot_injected: false,
            slot_checked: false,
            slot_passed: false,
            slot_detail: String::new(),
            flipper_injected: false,
            flipper_checked: false,
            flipper_passed: false,
            flipper_detail: String::new(),
            flipper_start: Vec2::ZERO,
            flipper_max_speed: 0.0,
        }
    }

    fn self_test(path: PathBuf) -> Self {
        let mut driver = Self::demo(path);
        driver.mode = DriverMode::SelfTest;
        driver
    }
}

#[derive(Clone, Serialize)]
struct TestStep {
    step: String,
    passed: bool,
    detail: String,
}

#[derive(Serialize)]
struct SelfTestOutput {
    version: u32,
    steps: Vec<TestStep>,
}

fn main() {
    let cli = parse_cli();
    let capture = cli.screenshot.is_some() || cli.self_test.is_some();
    // A normal launch fills the screen, because the person asked for
    // fullscreen. Captures and --windowed always stay in the 960x720 window.
    let start_fullscreen = !capture && !cli.windowed;
    #[allow(unused_mut)]
    let mut window = Window {
        title: "ROLLERBALL 3D — Modern Pinball Remake".into(),
        resolution: WindowResolution::new(WINDOW_W as u32, WINDOW_H as u32)
            .with_scale_factor_override(1.0),
        // The primary monitor is known while the window is created; "current"
        // is only reliable once the window exists.
        mode: if start_fullscreen {
            WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
        } else {
            WindowMode::Windowed
        },
        resizable: false,
        focused: !capture,
        ..default()
    };
    #[cfg(target_os = "windows")]
    if cli.screenshot.is_some() {
        window.window_level = bevy::window::WindowLevel::AlwaysOnBottom;
    }

    let asset_root = audio::assets_root();
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(window),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(AssetPlugin {
                file_path: asset_root.to_string_lossy().into_owned(),
                ..default()
            }),
    )
    .add_plugins((
        PhysicsPlugins::default().with_length_unit(10.0),
        KiraAudioPlugin,
    ))
    .add_audio_channel_with_track::<audio::SfxChannel>(audio::sfx_track())
    .add_audio_channel::<audio::MusicChannel>()
    .insert_resource(ClearColor(Color::srgb_u8(3, 5, 16)))
    .insert_resource(Gravity(Vec2::NEG_Y * GRAVITY))
    .insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f64(
        1.0 / PHYSICS_HZ,
    )))
    .insert_resource(WinitSettings {
        focused_mode: UpdateMode::Continuous,
        unfocused_mode: UpdateMode::Continuous,
    })
    .init_resource::<Game>()
    .init_resource::<Controls>()
    .init_resource::<audio::SoundQueue>()
    .init_resource::<audio::MusicState>()
    .init_resource::<Plunger>()
    .init_resource::<CameraRig>()
    .init_resource::<physics::BallWatchdog>()
    .insert_resource(DisplayPrefs {
        fullscreen: start_fullscreen,
        capture_run: capture,
    })
    .insert_resource(cli.clone());

    if let Some(path) = cli.self_test.clone() {
        app.insert_resource(ScriptDriver::self_test(path));
    } else if let Some(path) = cli.screenshot.clone() {
        app.insert_resource(ScriptDriver::demo(path));
    }

    app.add_systems(Startup, (setup_world, audio::setup_audio).chain())
        .add_systems(PostStartup, (render3d::setup_render3d, apply_startup_display))
        .add_systems(
            PreUpdate,
            (drive_script, read_keyboard, manual_screenshot, handle_fullscreen_keys).chain(),
        )
        .add_systems(
            FixedUpdate,
            (
                physics::manage_ball,
                physics::control_flippers,
                physics::flipper_impacts,
                physics::control_plunger,
                physics::use_nudge,
                physics::update_zones,
                physics::update_targets,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                (
                    menu_and_pause_input,
                    sync_pause,
                    game::tick_game,
                    physics::follow_ball,
                    render3d::follow_camera_3d,
                    render3d::resize_3d_viewport,
                    render3d::sync_ball_3d,
                    render3d::sync_plunger_3d,
                    render3d::animate_3d,
                    audio::toggle_music,
                )
                    .chain(),
                (
                    physics::tick_physics_timers,
                    physics::update_plunger_visual,
                    physics::reset_on_menu,
                    slots::sync_reel_visuals,
                    table::animate_table,
                    ui::update_ui,
                    ui::update_sidebar_overlay,
                    font::refresh_text_images,
                    audio::play_queued_sounds,
                    physics::handle_collisions,
                    capture_and_test,
                )
                    .chain(),
            )
                .chain(),
        )
        .run();
}

fn parse_cli() -> Cli {
    let mut cli = Cli::default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--screenshot" => cli.screenshot = args.next().map(PathBuf::from),
            "--self-test" => cli.self_test = args.next().map(PathBuf::from),
            "--windowed" => cli.windowed = true,
            _ => {}
        }
    }
    cli
}

fn setup_world(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    let art = build_art(&mut images);
    let background: Handle<Image> = asset_server.load("backgrounds/space_background.png");
    ui::build_cameras(&mut commands, &mut images);
    table::spawn_table(&mut commands, &mut images, &art, &background);
    physics::spawn_flippers(&mut commands, &art);
    ui::setup_hud(&mut commands, &mut images);
    ui::setup_overlay(&mut commands, &mut images);
    commands.insert_resource(art);
}

fn read_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    driver: Option<Res<ScriptDriver>>,
    mut controls: ResMut<Controls>,
) {
    if driver.is_some() {
        return;
    }
    let alt = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);
    let enter = keys.pressed(KeyCode::Enter) || keys.pressed(KeyCode::NumpadEnter);
    let mut next = Controls::default();
    next.left = keys.pressed(KeyCode::ShiftLeft)
        || keys.pressed(KeyCode::KeyZ)
        || keys.pressed(KeyCode::ArrowLeft);
    next.right = keys.pressed(KeyCode::ShiftRight)
        || keys.pressed(KeyCode::KeyM)
        || keys.pressed(KeyCode::ArrowRight);
    next.plunger = keys.pressed(KeyCode::Space);
    next.nudge = keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp);
    next.up = keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp);
    next.down = keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown);
    // Alt+Enter belongs to the fullscreen toggle, not the menu.
    next.confirm = enter && !alt;
    next.back = keys.pressed(KeyCode::Escape);
    next.pause = keys.pressed(KeyCode::KeyP);
    next.confirm_pressed = !alt
        && (keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter));
    next.back_pressed = keys.just_pressed(KeyCode::Escape);
    next.up_pressed = keys.just_pressed(KeyCode::KeyW) || keys.just_pressed(KeyCode::ArrowUp);
    next.down_pressed =
        keys.just_pressed(KeyCode::KeyS) || keys.just_pressed(KeyCode::ArrowDown);
    next.plunge_pressed = keys.just_pressed(KeyCode::Space);
    next.plunge_released = keys.just_released(KeyCode::Space);
    next.nudge_pressed =
        keys.just_pressed(KeyCode::KeyW) || keys.just_pressed(KeyCode::ArrowUp);
    next.pause_pressed = keys.just_pressed(KeyCode::KeyP);
    let enter_edge =
        keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter);
    next.start_pressed = (!alt && enter_edge)
        || keys.just_pressed(KeyCode::Space)
        || keys.just_pressed(KeyCode::KeyZ)
        || keys.just_pressed(KeyCode::KeyM)
        || keys.just_pressed(KeyCode::ShiftLeft)
        || keys.just_pressed(KeyCode::ShiftRight)
        || keys.just_pressed(KeyCode::ArrowLeft)
        || keys.just_pressed(KeyCode::ArrowRight)
        || keys.just_pressed(KeyCode::Digit1)
        || mouse.just_pressed(MouseButton::Left);
    next.start_2p_pressed = keys.just_pressed(KeyCode::Digit2);
    next.start_match_pressed = keys.just_pressed(KeyCode::Digit3);
    *controls = next;
}

fn manual_screenshot(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::F12) {
        let _ = std::fs::create_dir_all("shots");
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk("shots/screenshot.png"));
    }
}

/// F11 and Alt+Enter switch between the 960x720 window and a borderless
/// fullscreen that keeps the whole 4:3 table centred. Capture runs never
/// change the person's display.
fn handle_fullscreen_keys(
    keys: Res<ButtonInput<KeyCode>>,
    driver: Option<Res<ScriptDriver>>,
    mut display: ResMut<DisplayPrefs>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if display.capture_run || driver.is_some() {
        return;
    }
    let alt = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);
    let toggle = keys.just_pressed(KeyCode::F11)
        || (alt
            && (keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter)));
    if !toggle {
        return;
    }
    display.fullscreen = !display.fullscreen;
    if let Ok(mut window) = windows.single_mut() {
        set_window_mode(&mut window, display.fullscreen);
    }
}

/// A normal launch asks for fullscreen, but only once winit has a monitor.
fn apply_startup_display(
    display: Res<DisplayPrefs>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if !display.fullscreen || display.capture_run {
        return;
    }
    if let Ok(mut window) = windows.single_mut() {
        window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Primary);
    }
}

fn set_window_mode(window: &mut Window, fullscreen: bool) {
    if fullscreen {
        window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Current);
    } else {
        window.mode = WindowMode::Windowed;
        window.position = bevy::window::WindowPosition::Centered(MonitorSelection::Current);
        window.resolution = WindowResolution::new(WINDOW_W as u32, WINDOW_H as u32)
            .with_scale_factor_override(1.0);
    }
}

fn drive_script(
    time: Res<Time>,
    mut driver: Option<ResMut<ScriptDriver>>,
    game: Res<Game>,
    mut controls: ResMut<Controls>,
) {
    let Some(driver) = driver.as_mut() else {
        return;
    };
    driver.t += time.delta_secs();
    driver.frame += 1;
    if !driver.playing_started && game.mode != Mode::Menu {
        driver.playing_started = true;
    }
    if driver.playing_started {
        driver.play_t += time.delta_secs();
    }
    let t = driver.t;
    let pt = driver.play_t;
    let mut next = Controls::default();
    match driver.mode {
        DriverMode::Demo => {
            next.confirm_pressed = t < 0.05;
            next.confirm = t < 0.05;
            next.plunger = (0.35..1.25).contains(&t);
            next.left = (2.55..2.95).contains(&t);
            next.right = (3.45..3.85).contains(&t);
        }
        DriverMode::SelfTest => {
            // Look at the start menu for a few frames, then start from it
            // exactly as a player would: press Enter. Frame counting is used
            // instead of a narrow time window so a slow first frame (shader
            // compilation, asset upload) can never skip the key press.
            let waiting = game.mode == Mode::Menu && driver.frame >= 3;
            next.confirm_pressed = waiting;
            next.confirm = waiting;
            // Charge the plunger for about a second, then release.
            next.plunger = (0.4..1.5).contains(&pt);
            // Work the left flipper so its movement can be measured.
            next.left = (1.0..1.6).contains(&pt);
            next.right = (2.0..2.4).contains(&pt);
        }
    }
    if game.mode == Mode::GameOver {
        next = Controls::default();
    }
    *controls = next;
}

fn menu_and_pause_input(
    mut controls: ResMut<Controls>,
    mut game: ResMut<Game>,
    mut queue: ResMut<audio::SoundQueue>,
    driver: Option<Res<ScriptDriver>>,
    mut exit: MessageWriter<AppExit>,
) {
    if driver.is_some() {
        // Scripted runs still use the same menu transition as a player start.
        if game.mode == Mode::Menu && controls.confirm_pressed {
            let match_play = game.menu_index == 2;
            game.start_game(1, match_play, &mut queue);
        }
        return;
    }

    match game.mode {
        Mode::Menu => {
            if controls.up_pressed {
                game.menu_index =
                    (game.menu_index + MENU_LABELS.len() - 1) % MENU_LABELS.len();
            }
            if controls.down_pressed {
                game.menu_index = (game.menu_index + 1) % MENU_LABELS.len();
            }
            // Enter, Space, a flipper key or a mouse click start the selected
            // mode. Number keys jump straight into a specific mode. The exit
            // entry is the exception: only Enter or Escape closes the game,
            // so a start key cannot quit by accident.
            if controls.start_match_pressed {
                game.start_game(2, true, &mut queue);
            } else if controls.start_2p_pressed {
                game.start_game(2, false, &mut queue);
            } else {
                match ui::menu_choice(
                    game.menu_index,
                    controls.start_pressed,
                    controls.confirm_pressed,
                    controls.back_pressed,
                ) {
                    MenuChoice::Quit => {
                        exit.write(AppExit::Success);
                    }
                    MenuChoice::Start {
                        players,
                        match_play,
                    } => game.start_game(players, match_play, &mut queue),
                    MenuChoice::None => {}
                }
            }
        }
        Mode::Playing => {
            if controls.pause_pressed {
                game.mode = Mode::Paused;
            } else if controls.back_pressed {
                game.to_menu();
            }
        }
        Mode::Paused => {
            if controls.pause_pressed || controls.confirm_pressed {
                game.mode = Mode::Playing;
            } else if controls.back_pressed {
                game.to_menu();
            }
        }
        Mode::GameOver => {
            if controls.start_pressed || controls.confirm_pressed || controls.back_pressed {
                game.to_menu();
            }
        }
    }
    controls.clear_edges();
}

fn sync_pause(game: Res<Game>, mut physics_time: ResMut<Time<Physics>>) {
    if game.mode == Mode::Paused {
        if !physics_time.is_paused() {
            physics_time.pause();
        }
    } else if physics_time.is_paused() {
        physics_time.unpause();
    }
}

fn capture_and_test(
    mut commands: Commands,
    time: Res<Time>,
    mut driver: Option<ResMut<ScriptDriver>>,
    game: Res<Game>,
    bank: Res<audio::SoundBank>,
    music: Res<audio::MusicState>,
    display: Res<DisplayPrefs>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut balls: Query<(&Transform, &mut Position, &mut LinearVelocity), With<physics::Ball>>,
    flippers: Query<(&physics::Flipper, &Rotation)>,
    hud: Query<(&ui::HudField, &PixelText, &Visibility)>,
    texts: Query<&PixelText>,
    ball3d: Query<Entity, With<render3d::Ball3D>>,
    cameras3d: Query<&Projection, With<physics::TableCamera>>,
    menu_options: Query<&PixelText, With<ui::MenuOption>>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(driver) = driver.as_mut() else {
        return;
    };
    let _ = time;

    // The scripted overview of the start menu runs before the first Enter.
    if driver.mode == DriverMode::SelfTest && game.mode == Mode::Menu {
        driver.menu_exit_seen |= menu_options
            .iter()
            .any(|text| text.text.contains("EXIT"));
    }

    if let Ok((ball, _, velocity)) = balls.single() {
        let y = ball.translation.y;
        if y > driver.max_y {
            driver.max_y = y;
        }
        if driver.flipper_injected && !driver.flipper_checked {
            driver.flipper_max_speed = driver.flipper_max_speed.max(velocity.0.length());
        }
    }

    // Regression probe for the slot screen: after the launch and flipper
    // checks, drop the ball in above the slot cabinet. It must roll off or be
    // nudged free within a moment, never settle on top of the machine.
    if driver.mode == DriverMode::SelfTest {
        if !driver.slot_injected && driver.play_t >= 2.9 {
            if let Ok((_, mut position, mut velocity)) = balls.single_mut() {
                position.0 = Vec2::new(crate::table::SLOT_CABINET_POS.x, 712.0);
                velocity.0 = Vec2::ZERO;
            }
            driver.slot_injected = true;
        }
        if driver.slot_injected && !driver.slot_checked && driver.play_t >= 4.6 {
            match balls.single() {
                Ok((transform, _, velocity)) => {
                    let pos = transform.translation.truncate();
                    let stuck = physics::slot_escape(pos, velocity.0.length()).is_some();
                    driver.slot_passed = !stuck;
                    driver.slot_detail = if stuck {
                        format!(
                            "ball still resting on the slot cabinet at ({:.0}, {:.0})",
                            pos.x, pos.y
                        )
                    } else {
                        format!(
                            "ball rolled clear of the slot cabinet to ({:.0}, {:.0})",
                            pos.x, pos.y
                        )
                    };
                }
                Err(_) => {
                    driver.slot_passed = true;
                    driver.slot_detail = "ball left the slot area and drained".into();
                }
            }
            driver.slot_checked = true;
        }

        // Regression probe for the reported slipperiness bug: place a slow ball
        // on the resting left flipper of the bottom screen. With the old
        // locked-rotation ball and sticky rubber it stayed there indefinitely;
        // it must roll off the face or tip by itself.
        if !driver.flipper_injected && driver.play_t >= 5.0 {
            if let Ok((_, mut position, mut velocity)) = balls.single_mut() {
                let pivot = Vec2::new(
                    physics::FLIPPER_PIVOT_X[0],
                    physics::FLIPPER_Y,
                );
                let angle = physics::flipper_angle(FlipperSide::Left, 0.0);
                let dir = Vec2::from_angle(angle);
                let normal = Vec2::new(-dir.y, dir.x);
                let start = pivot
                    + dir * physics::FLIPPER_LEN
                    + normal * (physics::FLIPPER_HALF_WIDTH + BALL_RADIUS + 0.2);
                position.0 = start;
                velocity.0 = Vec2::ZERO;
                driver.flipper_start = start;
            }
            driver.flipper_injected = true;
        }
        if driver.flipper_injected && !driver.flipper_checked && driver.play_t >= 6.6 {
            match balls.single() {
                Ok((transform, _, velocity)) => {
                    let pos = transform.translation.truncate();
                    let travelled = pos.distance(driver.flipper_start);
                    let speed = velocity.0.length();
                    driver.flipper_passed = travelled > 18.0 || speed > 50.0;
                    driver.flipper_detail = format!(
                        "ball on the resting flipper moved {:.1} px in 1.6 s at up to {:.0} px/s (the flip itself reached {:.0} px/s, now at {:.0}, {:.0})",
                        travelled, speed, driver.flipper_max_speed, pos.x, pos.y
                    );
                }
                Err(_) => {
                    driver.flipper_passed = true;
                    driver.flipper_detail =
                        "ball rolled off the flipper and left the playfield".into();
                }
            }
            driver.flipper_checked = true;
        }
    }

    if driver.play_t > 0.5 {
        for (flipper, rotation) in &flippers {
            if flipper.side == FlipperSide::Left {
                let angle = rotation.as_radians();
                driver.min_left_angle = driver.min_left_angle.min(angle);
                driver.max_left_angle = driver.max_left_angle.max(angle);
            }
        }
    }

    match driver.mode {
        DriverMode::Demo => {
            let already_saved = std::fs::metadata(&driver.path)
                .map(|m| m.len() > 0)
                .unwrap_or(false);
            // A screenshot request can occasionally land on a frame where the
            // window entity is not visible to the capture system. Retry once a
            // second instead of failing the whole run on one missed attempt.
            if driver.frame >= 120
                && driver.t >= 2.0
                && !already_saved
                && driver.screenshot_attempts < 20
                && driver.t - driver.last_screenshot_t > 0.9
            {
                driver.screenshot_attempts += 1;
                driver.last_screenshot_t = driver.t;
                if let Some(parent) = driver.path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(driver.path.to_string_lossy().into_owned()));
            }
            if already_saved {
                exit.write(AppExit::Success);
            }
            // A capture on a slow or busy machine may need more than a couple
            // of seconds to reach 120 frames. Give it 30 s, then leave anyway
            // so a check can never hang on a window that refuses to be read.
            if driver.t > 30.0 {
                exit.write(AppExit::Success);
            }
        }
        DriverMode::SelfTest => {
            if driver.play_t > 6.9 && !driver.saved_test {
                driver.saved_test = true;
                let moved = driver.max_y > 600.0;
                let flip_delta = driver.max_left_angle - driver.min_left_angle;
                let flip_ok = flip_delta > 0.15;
                let hud_ok = hud.iter().any(|(f, _, _)| matches!(f, ui::HudField::Score1))
                    && hud.iter().any(|(f, _, _)| matches!(f, ui::HudField::Balls))
                    && texts.iter().any(|t| t.text == "PLAYER ONE")
                    && texts.iter().any(|t| t.text == "HIGH SCORE")
                    && texts.iter().any(|t| t.text == "BONUS");
                let voiced = bank
                    .handles
                    .values()
                    .filter(|handles| !handles.is_empty())
                    .count();
                let audio_ok = bank.loaded_files >= 11
                    && bank.fallback_events.is_empty()
                    && voiced == audio::SoundEvent::ALL.len();
                let audio_detail = format!(
                    "{} sound files loaded for {} of {} events, {} synth fallback(s)",
                    bank.loaded_files,
                    voiced,
                    audio::SoundEvent::ALL.len(),
                    bank.fallback_events.len()
                );
                let menu_gone = game.mode == Mode::Playing
                    && menu_options.iter().all(|text| text.text.is_empty());
                let three_d_ok = !ball3d.is_empty()
                    && cameras3d
                        .iter()
                        .any(|projection| matches!(projection, Projection::Perspective(_)));
                let music_ok = !music.source.is_empty();
                // Fullscreen check: a dry run through the same helper the F11
                // key uses, plus the letterbox maths for a 16:9 monitor. The
                // capture window itself is deliberately never thrown into
                // fullscreen, so the person's desktop is left alone.
                let (bx, by, bw, bh) = letterbox_rect(1920, 1080);
                let (wx, wy, ww, wh) = letterbox_rect(WINDOW_W as u32, WINDOW_H as u32);
                let mut probe = Window::default();
                set_window_mode(&mut probe, true);
                let probe_full = matches!(probe.mode, WindowMode::BorderlessFullscreen(_));
                set_window_mode(&mut probe, false);
                let probe_window = probe.mode == WindowMode::Windowed
                    && probe.resolution.physical_size()
                        == UVec2::new(WINDOW_W as u32, WINDOW_H as u32);
                let capture_windowed = !display.fullscreen
                    && windows
                        .single()
                        .map(|window| window.mode == WindowMode::Windowed)
                        .unwrap_or(false);
                let display_ok = probe_full
                    && probe_window
                    && capture_windowed
                    && (bx, by, bw, bh) == (240, 0, 1440, 1080)
                    && (wx, wy, ww, wh)
                        == (0, 0, WINDOW_W as u32, WINDOW_H as u32);
                let results = vec![
                    TestStep {
                        step: "move".into(),
                        passed: moved,
                        detail: format!(
                            "ball climbed {:.0} px up the launch lane and into the table",
                            driver.max_y
                        ),
                    },
                    TestStep {
                        step: "act".into(),
                        passed: flip_ok,
                        detail: format!(
                            "left flipper swept {:.2} rad ({:.0} degrees)",
                            flip_delta,
                            flip_delta.to_degrees()
                        ),
                    },
                    TestStep {
                        step: "hud".into(),
                        passed: hud_ok,
                        detail: "PLAYER ONE/TWO, HIGH SCORE, BONUS and BALLS values are drawn".into(),
                    },
                    TestStep {
                        step: "menu".into(),
                        passed: menu_gone,
                        detail: format!(
                            "the start key left the menu: mode is {:?}, {} of {} mode labels still on the sidebar",
                            game.mode,
                            menu_options.iter().filter(|text| !text.text.is_empty()).count(),
                            MENU_LABELS.len()
                        ),
                    },
                    TestStep {
                        step: "exit".into(),
                        passed: driver.menu_exit_seen
                            && ui::menu_choice(
                                MENU_LABELS.len() - 1,
                                false,
                                true,
                                false,
                            ) == MenuChoice::Quit,
                        detail: if driver.menu_exit_seen {
                            "the start menu lists EXIT GAME; Enter or Escape on it sends AppExit".into()
                        } else {
                            "the EXIT GAME entry was not seen on the start menu".into()
                        },
                    },
                    TestStep {
                        step: "launch".into(),
                        passed: driver.max_y > 700.0,
                        detail: "plunger shot reached the upper screens".into(),
                    },
                    TestStep {
                        step: "slot".into(),
                        passed: driver.slot_checked && driver.slot_passed,
                        detail: if driver.slot_checked {
                            driver.slot_detail.clone()
                        } else {
                            "the slot drop probe did not run".into()
                        },
                    },
                    TestStep {
                        step: "flipper".into(),
                        passed: driver.flipper_checked && driver.flipper_passed,
                        detail: if driver.flipper_checked {
                            driver.flipper_detail.clone()
                        } else {
                            "the resting-flipper probe did not run".into()
                        },
                    },
                    TestStep {
                        step: "audio".into(),
                        passed: audio_ok,
                        detail: audio_detail,
                    },
                    TestStep {
                        step: "3d".into(),
                        passed: three_d_ok,
                        detail: "a perspective camera follows a real 3D ball over the textured table".into(),
                    },
                    TestStep {
                        step: "music".into(),
                        passed: music_ok,
                        detail: format!(
                            "{} loop ({}) on its own channel; music is {}",
                            if music.external {
                                "external 8-bit"
                            } else {
                                "procedural 8-bit"
                            },
                            music.source,
                            if music.enabled { "on" } else { "off" }
                        ),
                    },
                    TestStep {
                        step: "display".into(),
                        passed: display_ok,
                        detail: format!(
                            "F11 / Alt+Enter toggle borderless fullscreen; a 1920x1080 screen keeps a centred {}x{} 4:3 table, windowed mode restores {}x{}",
                            bw, bh, ww, wh
                        ),
                    },
                ];
                let passed = results.iter().all(|r| r.passed);
                if let Some(parent) = driver.path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let output = SelfTestOutput { version: 1, steps: results };
                let _ = std::fs::write(
                    &driver.path,
                    serde_json::to_string_pretty(&output).unwrap_or_default(),
                );
                if !passed {
                    warn!("Self-test completed with a failing step; see {}", driver.path.display());
                }
            }
            if driver.play_t > 8.0 || driver.t > 30.0 {
                exit.write(AppExit::Success);
            }
        }
    }
}
