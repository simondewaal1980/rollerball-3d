//! Cameras, the fixed right-hand HUD, the menu and all on-screen messages.

use bevy::camera::ScalingMode;
use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

use crate::audio::MusicState;
use crate::common::*;
use crate::font::{spawn_text, text_width, PixelText, TextColor};
use crate::game::{Game, Mode, Phase};
use crate::table::{TableValue, TableValueKind};

#[derive(Component)]
pub struct HudCamera;

/// Always-on camera for the sidebar composite.
#[derive(Component)]
pub struct HudCompositeCamera;

#[derive(Component)]
pub struct MessageUi;

/// The always-visible right-hand sidebar composite.
#[derive(Component)]
pub struct HudComposite;



#[derive(Component)]
pub struct MenuOption(pub usize);

/// Entries in the start menu, top to bottom. The last one leaves the game.
pub const MENU_LABELS: [&str; 4] = ["1 PLAYER", "2 PLAYERS", "MATCH PLAY", "EXIT GAME"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuChoice {
    Start { players: usize, match_play: bool },
    Quit,
    None,
}

/// What activating the start menu does. `confirm` is Enter and `back` is
/// Escape. The exit entry deliberately ignores the flipper/space start keys,
/// so bumping the table controls cannot close the game by accident.
pub fn menu_choice(index: usize, start: bool, confirm: bool, back: bool) -> MenuChoice {
    if index >= MENU_LABELS.len() {
        return MenuChoice::None;
    }
    if index == MENU_LABELS.len() - 1 {
        if confirm || back {
            MenuChoice::Quit
        } else {
            MenuChoice::None
        }
    } else if start || confirm {
        MenuChoice::Start {
            players: if index == 0 { 1 } else { 2 },
            match_play: index == 2,
        }
    } else {
        MenuChoice::None
    }
}

/// The alternating start instruction in the sidebar menu.
#[derive(Component)]
pub struct MenuPrompt;

#[derive(Component)]
pub struct PauseTitle;

#[derive(Component)]
pub struct PauseHint;

#[derive(Component)]
pub struct GameOverTitle;

#[derive(Component)]
pub struct GameOverHint;

#[derive(Component)]
pub struct ActiveMarker(pub usize);

#[derive(Component)]
pub struct GameOverDetail;

/// Small reminder of the fullscreen key in the bottom corner of the sidebar.
#[derive(Component)]
pub struct FullscreenHint;

#[derive(Component, Clone, Copy, Debug)]
pub enum HudField {
    Name1,
    Name2,
    Score1,
    Score2,
    High,
    Bonus,
    Balls,
    Floor,
    FloorName,
    ModeLabel,
    Music,
}

pub fn build_cameras(commands: &mut Commands, images: &mut Assets<Image>) {
    // The UI pass has its own fixed 3x canvas instead of following the window
    // size. Fullscreen can then change the window without rebuilding textures,
    // and the composite camera only has to move its viewport.
    let (w, h) = (WINDOW_W as u32, WINDOW_H as u32);

    let projection = Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::FixedVertical {
            viewport_height: DESIGN_H,
        },
        // The UI camera sits at z=1000 and some backing panels live slightly
        // behind z=0, so the default 1000-unit far plane would clip them.
        far: 2000.0,
        ..OrthographicProjection::default_2d()
    });

    // All 2D art (HUD, menu, messages) is rendered into its own transparent
    // texture. That lets the sidebar clear itself every frame without wiping
    // the perspective table, and it avoids ghosting when a label changes
    // length (for example "CLOSE TO THE EDGE" -> "TOP SCREEN").
    let mut ui_image = Image::new_target_texture(
        w,
        h,
        TextureFormat::Rgba8Unorm,
        Some(TextureFormat::Rgba8UnormSrgb),
    );
    ui_image.sampler = ImageSampler::nearest();
    let ui_image = images.add(ui_image);

    commands.spawn((
        Camera2d,
        HudCamera,
        Camera {
            order: 1,
            // The UI texture is opaque, so it never ghosts. Only the sidebar
            // slice is composited; the menu lives in that sidebar too.
            clear_color: ClearColorConfig::Custom(Color::srgb_u8(5, 8, 22)),
            ..default()
        },
        RenderTarget::Image(ui_image.clone().into()),
        projection.clone(),
        Transform::from_xyz(DESIGN_W * 0.5, DESIGN_H * 0.5, 1000.0),
        RenderLayers::layer(LAYER_HUD),
        Name::new("UI pass"),
    ));

    // The sidebar is always visible, including while a ball is in play.
    commands.spawn((
        Camera2d,
        HudCompositeCamera,
        Camera {
            order: 3,
            viewport: None,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        projection,
        Transform::from_xyz(DESIGN_W * 0.5, DESIGN_H * 0.5, 1000.0),
        RenderLayers::layer(LAYER_HUD_COMPOSITE),
        Name::new("HUD composite pass"),
    ));

    // Right-hand sidebar only. The texture coordinates select the HUD slice
    // of the 960x720 UI image (x = 672..960).
    commands.spawn((
        Sprite {
            image: ui_image.clone(),
            rect: Some(Rect::new(
                w as f32 * 0.7,
                0.0,
                w as f32,
                h as f32,
            )),
            custom_size: Some(Vec2::new(HUD_W, DESIGN_H)),
            ..default()
        },
        Transform::from_xyz(HUD_CENTER_X, DESIGN_H * 0.5, 0.0),
        RenderLayers::layer(LAYER_HUD_COMPOSITE),
        HudComposite,
        Name::new("HUD composite"),
    ));

}

pub fn setup_hud(commands: &mut Commands, images: &mut Assets<Image>) {
    let layer = LAYER_HUD;
    commands.spawn((
        Sprite::from_color(Color::srgb_u8(5, 8, 22), Vec2::new(HUD_W, DESIGN_H)),
        Transform::from_xyz(HUD_CENTER_X, DESIGN_H * 0.5, -10.0),
        RenderLayers::layer(layer),
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb_u8(45, 110, 180), Vec2::new(2.0, DESIGN_H)),
        Transform::from_xyz(PLAY_W + 1.0, DESIGN_H * 0.5, -9.0),
        RenderLayers::layer(layer),
    ));

    spawn_centered_hud(commands, images, "ROLLERBALL", 228.0, [255, 225, 100], 1.5, None);
    spawn_centered_hud(
        commands,
        images,
        "1 PLAYER",
        214.0,
        [150, 210, 255],
        1.0,
        Some(HudField::ModeLabel),
    );

    spawn_centered_hud(
        commands,
        images,
        "PLAYER ONE",
        200.0,
        [170, 215, 255],
        1.0,
        Some(HudField::Name1),
    );
    spawn_hud_left(
        commands,
        images,
        "000000",
        186.0,
        [120, 255, 190],
        1.5,
        Some(HudField::Score1),
    );
    spawn_centered_hud(
        commands,
        images,
        "PLAYER TWO",
        170.0,
        [150, 170, 210],
        1.0,
        Some(HudField::Name2),
    );
    spawn_hud_left(
        commands,
        images,
        "000000",
        156.0,
        [120, 210, 180],
        1.5,
        Some(HudField::Score2),
    );

    line(commands, 148.0, layer);

    spawn_centered_hud(
        commands,
        images,
        "HIGH SCORE",
        139.0,
        [220, 190, 255],
        1.0,
        None,
    );
    spawn_hud_left(
        commands,
        images,
        "012000",
        125.0,
        [255, 235, 150],
        1.5,
        Some(HudField::High),
    );

    line(commands, 112.0, layer);

    spawn_hud_left(commands, images, "BONUS", 104.0, [170, 220, 255], 1.0, None);
    spawn_hud_left(
        commands,
        images,
        "1000X1",
        90.0,
        [255, 210, 120],
        1.5,
        Some(HudField::Bonus),
    );
    spawn_hud_left(commands, images, "BALLS", 76.0, [170, 220, 255], 1.0, None);
    spawn_hud_left(
        commands,
        images,
        "03",
        62.0,
        [120, 255, 200],
        1.5,
        Some(HudField::Balls),
    );

    line(commands, 52.0, layer);

    spawn_hud_left(
        commands,
        images,
        "FLOOR 4",
        44.0,
        [140, 190, 240],
        1.0,
        Some(HudField::Floor),
    );
    spawn_hud_left(
        commands,
        images,
        "CLOSE TO THE EDGE",
        31.0,
        [110, 140, 190],
        0.75,
        Some(HudField::FloorName),
    );

    let m1 = spawn_text(
        commands,
        images,
        layer,
        Vec2::new(229.0, 200.0),
        2.0,
        ">",
        [255, 240, 100],
        1.0,
    );
    commands.entity(m1).insert(ActiveMarker(0));
    let m2 = spawn_text(
        commands,
        images,
        layer,
        Vec2::new(229.0, 170.0),
        2.0,
        ">",
        [255, 240, 100],
        1.0,
    );
    commands.entity(m2).insert(ActiveMarker(1));

    let music = spawn_text(
        commands,
        images,
        layer,
        Vec2::new(228.0, 10.0),
        2.0,
        "MUSIC ON",
        [90, 190, 160],
        0.7,
    );
    commands.entity(music).insert(HudField::Music);
}

fn line(commands: &mut Commands, y: f32, layer: usize) {
    commands.spawn((
        Sprite::from_color(Color::srgba_u8(45, 90, 140, 180), Vec2::new(86.0, 1.0)),
        Transform::from_xyz(HUD_CENTER_X, y, -5.0),
        RenderLayers::layer(layer),
    ));
}

fn spawn_centered_hud<C: TextColor>(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    text: &str,
    top: f32,
    color: C,
    scale: f32,
    field: Option<HudField>,
) -> Entity {
    let width = text_width(text, scale);
    let x = HUD_CENTER_X - width * 0.5;
    let entity = spawn_text(
        commands,
        images,
        LAYER_HUD,
        Vec2::new(x, top),
        1.0,
        text,
        color,
        scale,
    );
    if let Some(field) = field {
        commands.entity(entity).insert(field);
    }
    entity
}

fn spawn_hud_left<C: TextColor>(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    text: &str,
    top: f32,
    color: C,
    scale: f32,
    field: Option<HudField>,
) -> Entity {
    let x = if scale > 1.5 { 236.0 } else { 230.0 };
    let entity = spawn_text(
        commands,
        images,
        LAYER_HUD,
        Vec2::new(x, top),
        1.0,
        text,
        color,
        scale,
    );
    if let Some(field) = field {
        commands.entity(entity).insert(field);
    }
    entity
}

/// Menu, pause and game-over screens plus the in-game message line.
/// Menu, pause and game-over text lives in the fixed sidebar. Keeping it out
/// of the playfield is deliberate: the table can never end up hidden behind a
/// stale title screen, and the scores stay readable the whole time.
pub fn setup_overlay(commands: &mut Commands, images: &mut Assets<Image>) {
    let layer = LAYER_HUD;

    for (i, _) in MENU_LABELS.iter().enumerate() {
        let ent = spawn_text(
            commands,
            images,
            layer,
            Vec2::new(229.0, 48.0 - i as f32 * 10.0),
            1.0,
            "",
            [190, 210, 240],
            0.85,
        );
        commands.entity(ent).insert(MenuOption(i));
    }

    let ent = spawn_text(
        commands,
        images,
        layer,
        Vec2::new(229.0, 9.0),
        1.0,
        "",
        [255, 235, 120],
        0.5,
    );
    commands.entity(ent).insert(MenuPrompt);

    let ent = spawn_text(
        commands,
        images,
        layer,
        Vec2::new(230.0, 46.0),
        1.0,
        "",
        [255, 220, 80],
        1.0,
    );
    commands.entity(ent).insert(PauseTitle);
    let ent = spawn_text(
        commands,
        images,
        layer,
        Vec2::new(228.0, 32.0),
        1.0,
        "",
        [180, 210, 240],
        0.6,
    );
    commands.entity(ent).insert(PauseHint);

    let ent = spawn_text(
        commands,
        images,
        layer,
        Vec2::new(230.0, 46.0),
        1.0,
        "",
        [255, 90, 100],
        1.0,
    );
    commands.entity(ent).insert(GameOverTitle);
    let ent = spawn_text(
        commands,
        images,
        layer,
        Vec2::new(228.0, 34.0),
        1.0,
        "",
        [255, 235, 160],
        0.55,
    );
    commands.entity(ent).insert(GameOverDetail);
    let ent = spawn_text(
        commands,
        images,
        layer,
        Vec2::new(228.0, 22.0),
        1.0,
        "",
        [180, 210, 240],
        0.55,
    );
    commands.entity(ent).insert(GameOverHint);

    // Status messages live in the sidebar, so they never cover the table.
    let ent = spawn_text(
        commands,
        images,
        LAYER_HUD,
        Vec2::new(230.0, 20.0),
        1.0,
        "",
        [255, 235, 120],
        0.5,
    );
    commands.entity(ent).insert(MessageUi);

    let ent = spawn_text(
        commands,
        images,
        LAYER_HUD,
        Vec2::new(230.0, 4.0),
        1.0,
        "",
        [120, 150, 190],
        0.42,
    );
    commands.entity(ent).insert(FullscreenHint);
}

type UiTextQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut PixelText,
        &'static mut Transform,
        Option<&'static HudField>,
        Option<&'static ActiveMarker>,
        Option<&'static MessageUi>,
        Option<&'static TableValue>,
    ),
>;

pub fn update_ui(game: Res<Game>, music: Res<MusicState>, mut ui: UiTextQuery) {
    let active = game.current.min(1);
    let score1 = format!("{:06}", game.players[0].score.min(999_999));
    let score2 = format!("{:06}", game.players[1].score.min(999_999));
    let bonus = format!("{}X{}", game.bonus.min(9_999), game.multiplier);
    let balls = if game.match_play {
        "MATCH".to_string()
    } else {
        format!("{:02}", game.players[active].balls_left)
    };
    let floor = format!("SCREEN {}", floor_number(game.active_screen));
    let floor_name = SCREEN_TITLES[game.active_screen].to_string();
    let mode_label = if game.match_play {
        "MATCH PLAY"
    } else if game.num_players == 2 {
        "2 PLAYERS"
    } else {
        "1 PLAYER"
    };

    let msg_text = if game.mode == Mode::Playing {
        if let Some(countdown) = &game.bonus_countdown {
            format!("BONUS {}X{}", countdown.remaining, game.multiplier)
        } else if game.message_timer > 0.0 {
            game.message.clone()
        } else if game.phase == Phase::Ready {
            "HOLD SPACE TO PLUNGE".into()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let playing = game.mode == Mode::Playing;

    for (mut text, mut transform, hud, marker, message, table) in &mut ui {
        if let Some(marker) = marker {
            // Changing the string is more reliable than a visibility toggle
            // for this tiny one-glyph sprite, and it never leaves a ghost.
            let show = game.mode != Mode::Menu && marker.0 == active;
            let value = if show { ">" } else { "" };
            if text.text != value {
                text.text = value.to_string();
            }
        }

        if let Some(field) = hud {
            let (value, color): (String, [u8; 4]) = match field {
                HudField::Name1 => (
                    "PLAYER ONE".into(),
                    if active == 0 && game.mode != Mode::Menu {
                        [255, 230, 90, 255]
                    } else {
                        [130, 170, 215, 255]
                    },
                ),
                HudField::Name2 => (
                    "PLAYER TWO".into(),
                    if active == 1 && game.mode != Mode::Menu {
                        [255, 230, 90, 255]
                    } else {
                        [110, 140, 185, 255]
                    },
                ),
                HudField::Score1 => (score1.clone(), [120, 255, 190, 255]),
                HudField::Score2 => (score2.clone(), [120, 200, 180, 255]),
                HudField::High => (
                    format!("{:06}", game.high_score.min(999_999)),
                    [255, 235, 150, 255],
                ),
                HudField::Bonus => (bonus.clone(), [255, 205, 110, 255]),
                HudField::Balls => (balls.clone(), [130, 255, 210, 255]),
                HudField::Floor => (
                    if playing { floor.clone() } else { String::new() },
                    [150, 195, 240, 255],
                ),
                HudField::FloorName => (
                    if playing {
                        floor_name.clone()
                    } else {
                        String::new()
                    },
                    [120, 155, 205, 255],
                ),
                HudField::ModeLabel => (
                    if game.mode == Mode::Menu {
                        "SELECT MODE".to_string()
                    } else {
                        mode_label.to_string()
                    },
                    [140, 200, 255, 255],
                ),
                HudField::Music => (
                    if !playing {
                        String::new()
                    } else if music.enabled {
                        "MUSIC ON".into()
                    } else {
                        "MUSIC OFF".into()
                    },
                    if music.enabled {
                        [90, 210, 165, 255]
                    } else {
                        [110, 120, 145, 255]
                    },
                ),
            };
            if text.text != value {
                text.text = value;
            }
            if text.color != color {
                text.color = color;
            }
        }

        if message.is_some() {
            if text.text != msg_text {
                text.text = msg_text.clone();
            }
            if text.scale != 0.5 {
                text.scale = 0.5;
            }
            transform.translation.x = 230.0;
            transform.translation.y = 20.0;
        }

        if let Some(value) = table {
            let new_text = match value.0 {
                TableValueKind::Score => {
                    format!("{:06}", game.players[active].score.min(999_999))
                }
                TableValueKind::Bonus => format!("{:04}", game.bonus.min(9_999)),
            };
            if text.text != new_text {
                text.text = new_text;
            }
        }
    }
}

type SidebarOverlayQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut PixelText,
        Option<&'static MenuOption>,
        Option<&'static MenuPrompt>,
        Option<&'static PauseTitle>,
        Option<&'static PauseHint>,
        Option<&'static GameOverTitle>,
        Option<&'static GameOverDetail>,
        Option<&'static GameOverHint>,
        Option<&'static FullscreenHint>,
    ),
>;

/// Mode menu, pause note and game-over result, all confined to the sidebar.
pub fn update_sidebar_overlay(
    game: Res<Game>,
    time: Res<Time>,
    display: Res<DisplayPrefs>,
    mut ui: SidebarOverlayQuery,
) {
    let menu = game.mode == Mode::Menu;
    let paused = game.mode == Mode::Paused;
    let over = game.mode == Mode::GameOver;
    let exit_selected = menu && game.menu_index == MENU_LABELS.len() - 1;
    let labels = MENU_LABELS;

    for (mut text, option, prompt, pause_title, pause_hint, over_title, detail, over_hint, fullscreen) in &mut ui {
        if let Some(option) = option {
            let (value, color) = if menu {
                let selected = option.0 == game.menu_index;
                (
                    format!(
                        "{}{}",
                        if selected { "> " } else { "  " },
                        labels[option.0.min(MENU_LABELS.len() - 1)]
                    ),
                    if selected {
                        [255, 225, 90, 255]
                    } else {
                        [150, 180, 220, 255]
                    },
                )
            } else {
                (String::new(), [150, 180, 220, 255])
            };
            if text.text != value {
                text.text = value;
            }
            if text.color != color {
                text.color = color;
            }
        }

        if prompt.is_some() {
            let value = if menu {
                if exit_selected {
                    if (time.elapsed_secs() % 1.6) < 1.2 {
                        "ENTER OR ESC TO QUIT"
                    } else {
                        "LEAVE THE GAME"
                    }
                } else if (time.elapsed_secs() % 1.6) < 1.2 {
                    "PRESS ENTER TO START"
                } else {
                    "SPACE OR CLICK TOO"
                }
            } else {
                ""
            };
            if text.text != value {
                text.text = value.to_string();
            }
            text.color = if exit_selected {
                [255, 145, 120, 255]
            } else {
                [255, 235, 120, 255]
            };
        }

        if pause_title.is_some() {
            let value = if paused { "PAUSED" } else { "" };
            if text.text != value {
                text.text = value.to_string();
            }
        }
        if pause_hint.is_some() {
            let value = if paused { "P OR ENTER" } else { "" };
            if text.text != value {
                text.text = value.to_string();
            }
        }

        if over_title.is_some() {
            let value = if over { "GAME OVER" } else { "" };
            if text.text != value {
                text.text = value.to_string();
            }
        }
        if detail.is_some() {
            let value = if !over {
                String::new()
            } else if let Some(winner) = game.game_over_winner {
                format!("PLAYER {} WINS", winner + 1)
            } else if game.match_play {
                format!(
                    "P1 {:04} P2 {:04}",
                    game.players[0].score, game.players[1].score
                )
            } else {
                format!(
                    "P1 {:06} P2 {:06}",
                    game.players[0].score, game.players[1].score
                )
            };
            if text.text != value {
                text.text = value;
            }
        }
        if over_hint.is_some() {
            let value = if over { "ENTER: MENU" } else { "" };
            if text.text != value {
                text.text = value.to_string();
            }
        }

        if fullscreen.is_some() {
            let value = if menu {
                if display.fullscreen {
                    "ESC: EXIT  F11: WINDOW"
                } else {
                    "ESC: EXIT  F11: FULL"
                }
            } else if display.fullscreen {
                "F11 WINDOWED"
            } else {
                "F11 FULLSCREEN"
            };
            if text.text != value {
                text.text = value.to_string();
            }
        }
    }
}

#[cfg(test)]
mod menu_tests {
    use super::*;

    #[test]
    fn menu_offers_an_exit_entry() {
        assert_eq!(MENU_LABELS.len(), 4);
        assert_eq!(MENU_LABELS[3], "EXIT GAME");
    }

    #[test]
    fn enter_or_escape_on_exit_quits() {
        assert_eq!(menu_choice(3, false, true, false), MenuChoice::Quit);
        assert_eq!(menu_choice(3, false, false, true), MenuChoice::Quit);
    }

    #[test]
    fn space_cannot_quit_by_accident() {
        assert_eq!(menu_choice(3, true, false, false), MenuChoice::None);
    }

    #[test]
    fn start_entries_keep_their_modes() {
        assert_eq!(
            menu_choice(0, true, false, false),
            MenuChoice::Start {
                players: 1,
                match_play: false
            }
        );
        assert_eq!(
            menu_choice(1, false, true, false),
            MenuChoice::Start {
                players: 2,
                match_play: false
            }
        );
        assert_eq!(
            menu_choice(2, false, true, false),
            MenuChoice::Start {
                players: 2,
                match_play: true
            }
        );
    }
}
