//! The physical table: four stacked screens, walls, bumpers, slingshots,
//! rollover lanes, drop targets, kickback holes, the slot cabinet and all
//! decorative pixel art.

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::camera::visibility::RenderLayers;

use crate::art::Art;
use crate::common::*;
use crate::font::spawn_text;
use crate::game::Game;
use crate::physics::{Bumper, DropTarget, Slingshot};
use crate::slots::SlotReel;

// Feature positions, referenced by the physics and UI systems.
pub const KICKBACK_POS: [Vec2; 3] = [
    Vec2::new(30.0, 66.0),   // bottom screen, left
    Vec2::new(190.0, 66.0),  // bottom screen, right
    Vec2::new(184.0, 884.0), // top screen, right
];
pub const LANE_EXIT_Y: f32 = 880.0;
pub const SLOT_CABINET_POS: Vec2 = Vec2::new(112.0, 620.0);
pub const SLOT_TRIGGER_POS: Vec2 = Vec2::new(112.0, 585.0);
pub const DROP_TARGET_X: [f32; 3] = [82.0, 112.0, 142.0];
pub const DROP_TARGET_Y: f32 = 362.0;
pub const ABC_BOTTOM: [Vec2; 3] = [
    Vec2::new(80.0, 206.0),
    Vec2::new(112.0, 206.0),
    Vec2::new(144.0, 206.0),
];
pub const ABC_TOP: [Vec2; 3] = [
    Vec2::new(30.0, 900.0),
    Vec2::new(48.0, 900.0),
    Vec2::new(66.0, 900.0),
];

/// Every bumper in table order (position, colour index). The 3D layer uses the
/// same list so the raised hardware can never drift away from the collider.
pub const BUMPERS: [(Vec2, usize); 7] = [
    (Vec2::new(95.0, 900.0), 0),
    (Vec2::new(148.0, 900.0), 1),
    (Vec2::new(58.0, 698.0), 0),
    (Vec2::new(164.0, 698.0), 1),
    (Vec2::new(112.0, 425.0), 2),
    (Vec2::new(58.0, 158.0), 1),
    (Vec2::new(164.0, 158.0), 2),
];

/// Marks a physics wall so the 3D layer can raise a matching rail on it.
#[derive(Component)]
pub struct WallMarker {
    pub color: Color,
    pub length: f32,
    pub thickness: f32,
}

#[derive(Component)]
pub struct Lamp {
    pub screen: usize,
    pub letter: usize,
}

#[derive(Component)]
pub struct HoleVisual {
    pub index: usize,
}

#[derive(Component)]
pub struct Spinner;

#[derive(Component)]
pub struct PlungerVisual { pub base_y: f32 }

#[derive(Component)]
pub struct TableValue(pub TableValueKind);

#[derive(Clone, Copy, Debug)]
pub enum TableValueKind {
    Score,
    Bonus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoneKind {
    Lane { screen: usize, letter: usize },
    Arrow { screen: usize, letter: usize },
    Spinner,
    SlotTrigger,
    Kickback { index: usize },
    Chute,
    Drain,
    LaneExit,
}

#[derive(Component)]
pub struct Zone {
    pub kind: ZoneKind,
    pub radius: f32,
    pub half: Vec2,
    pub inside: bool,
}

/// Build the complete table. One call keeps startup ordering simple.
pub fn spawn_table(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    art: &Art,
    background: &Handle<Image>,
) -> Entity {
    let table = commands.spawn((Name::new("Table"), Transform::default())).id();

    // A solid base guarantees a readable table even before the photo has
    // finished loading.
    commands.spawn((
        Sprite::from_color(Color::srgb_u8(9, 13, 34), Vec2::new(PLAY_W, TABLE_H)),
        Transform::from_xyz(PLAY_W * 0.5, TABLE_H * 0.5, -20.5),
        Name::new("Playfield background base"),
    ));

    spawn_playfield_artwork(commands, background);
    spawn_starfield(commands);

    // Horizontal screen seams.
    for screen in 1..4 {
        let y = screen as f32 * SCREEN_H;
        commands.spawn((
            Sprite::from_color(Color::srgba_u8(45, 80, 130, 150), Vec2::new(PLAY_W - 8.0, 1.0)),
            Transform::from_xyz(PLAY_W * 0.5, y, -8.0),
        ));
    }

    // ------------------------------------------------------------------
    // Outer cabinet and launch lane.
    // ------------------------------------------------------------------
    wall(commands, Vec2::new(4.0, 0.0), Vec2::new(4.0, 956.0), 5.0, CYAN_DIM);
    wall(commands, Vec2::new(220.0, 0.0), Vec2::new(220.0, 956.0), 5.0, CYAN_DIM);
    wall(commands, Vec2::new(4.0, 952.0), Vec2::new(197.0, 952.0), 5.0, CYAN_DIM);
    wall(commands, Vec2::new(197.0, 10.0), Vec2::new(197.0, 900.0), 4.0, MAGENTA_DIM);
    wall(commands, Vec2::new(197.0, 900.0), Vec2::new(220.0, 900.0), 4.0, MAGENTA_DIM);
    wall(commands, Vec2::new(197.0, 12.0), Vec2::new(220.0, 12.0), 4.0, MAGENTA_DIM);

    // Plunger guide rails (visual only, the physics plunger is a plate).
    for x in [201.0, 216.0] {
        commands.spawn((
            Sprite::from_color(Color::srgb_u8(90, 120, 180), Vec2::new(1.0, 34.0)),
            Transform::from_xyz(x, 28.0, -2.0),
        ));
    }
    spawn_plunger(commands);

    // funnel walls above the flippers of every screen
    for screen in 0..4 {
        let b = screen_bottom(screen);
        // End the funnel a little short of the flipper pivot. That leaves a
        // real outlane wide enough for the ball instead of a pocket where it
        // could wedge itself against the hinge.
        wall(
            commands,
            Vec2::new(4.0, b + 57.0),
            Vec2::new(48.0, b + 42.0),
            5.0,
            CYAN,
        );
        wall(
            commands,
            Vec2::new(196.0, b + 57.0),
            Vec2::new(176.0, b + 42.0),
            5.0,
            CYAN,
        );

        // Slingshots: angled rubber bands just above the flippers.
        spawn_slingshot(commands, Vec2::new(38.0, b + 105.0), 1.0);
        spawn_slingshot(commands, Vec2::new(178.0, b + 105.0), -1.0);
    }

    // ------------------------------------------------------------------
    // Screen 1 (top): score panel, two bumpers, ABC rollovers, kickback.
    // ------------------------------------------------------------------
    let b1 = screen_bottom(3);
    spawn_panel(commands, images, Vec2::new(112.0, b1 + 130.0), Vec2::new(100.0, 44.0), "SCORE");
    add_table_value(
        commands,
        images,
        Vec2::new(86.0, b1 + 129.0),
        4.1,
        TableValueKind::Score,
    );
    spawn_bumpers(commands, art);
    spawn_abc(commands, images, 3, &ABC_TOP, "ROLLOVER");
    spawn_kickback(commands, art, images, 2);
    spawn_floor_title(commands, images, 3);

    // ------------------------------------------------------------------
    // Screen 2: slot cabinet, spinner, two bumpers, side arrows.
    // ------------------------------------------------------------------
    let b2 = screen_bottom(2);
    spawn_slot(commands, images, art);
    spawn_spinner(commands, art, images, Vec2::new(112.0, b2 + 190.0));
    spawn_spinner_label(commands, images, Vec2::new(112.0, b2 + 190.0));
    // Side arrows with A/B/C lamps.
    for (i, y) in [b2 + 120.0, b2 + 155.0, b2 + 190.0].into_iter().enumerate() {
        spawn_lamp(commands, images, Vec2::new(17.0, y), 2, i, &["A", "B", "C"][i]);
        spawn_lamp(commands, images, Vec2::new(181.0, y), 2, i, &["A", "B", "C"][i]);
        zone(commands, ZoneKind::Arrow { screen: 2, letter: i }, Vec2::new(17.0, y), 10.0);
        zone(commands, ZoneKind::Arrow { screen: 2, letter: i }, Vec2::new(181.0, y), 10.0);
        commands.spawn((
            Sprite {
                image: art.arrow_down.clone(),
                custom_size: Some(Vec2::new(11.0, 8.0)),
                ..default()
            },
            Transform::from_xyz(if i % 2 == 0 { 25.0 } else { 173.0 }, y, 2.0),
        ));
    }
    spawn_floor_title(commands, images, 2);

    // ------------------------------------------------------------------
    // Screen 3: one bumper, drop targets, bonus counter, chute.
    // ------------------------------------------------------------------
    let b3 = screen_bottom(1);
    spawn_drop_targets(commands, images);
    spawn_panel(commands, images, Vec2::new(112.0, b3 + 222.0), Vec2::new(72.0, 22.0), "BONUS");
    add_table_value(
        commands,
        images,
        Vec2::new(101.0, b3 + 218.0),
        4.1,
        TableValueKind::Bonus,
    );
    spawn_text(
        commands,
        images,
        0,
        Vec2::new(10.0, b3 + 98.0),
        3.0,
        "CHUTE",
        [140, 170, 210],
        1.0,
    );
    zone(commands, ZoneKind::Chute, Vec2::new(16.0, b3 + 82.0), 13.0);
    spawn_floor_title(commands, images, 1);

    // ------------------------------------------------------------------
    // Screen 4 (bottom): launch lane, two bumpers, ABC lanes, kickbacks.
    // ------------------------------------------------------------------
    spawn_abc(commands, images, 0, &ABC_BOTTOM, "LANES");
    spawn_kickback(commands, art, images, 0);
    spawn_kickback(commands, art, images, 1);
    spawn_floor_title(commands, images, 0);

    // Drain sensor at the very bottom, plus the launch-lane exit trigger.
    zone(commands, ZoneKind::Drain, Vec2::new(100.0, 3.0), 1.0);
    zone(commands, ZoneKind::LaneExit, Vec2::new(LANE_CENTER, LANE_EXIT_Y), 1.0);

    // Other zone sensors.
    zone(commands, ZoneKind::Spinner, Vec2::new(112.0, b2 + 190.0), 16.0);
    zone(commands, ZoneKind::SlotTrigger, SLOT_TRIGGER_POS, 14.0);

    table
}

/// The person-supplied space artwork becomes the printed background of the
/// cabinet. It is a widescreen photo on a tall playfield, so instead of
/// squashing it the picture is cut into four aspect-correct columns — one per
/// screen. The planet sits at the bottom of the table and the glowing ball
/// trails run up the top screen, which keeps the artwork recognisable while
/// the neon inserts and rails stay readable on top of it.
fn spawn_playfield_artwork(commands: &mut Commands, background: &Handle<Image>) {
    const IMAGE_W: f32 = 1672.0;
    const IMAGE_H: f32 = 941.0;
    const COLUMNS: f32 = 4.0;
    // Vertical band of the photo that holds the planet and the glowing ball
    // trails. The photo's own chrome balls sit just below this band: leaving
    // them out keeps them from being mistaken for the player's ball.
    const CROP_Y: f32 = 250.0;

    let column_w = IMAGE_W / COLUMNS;
    let crop_h = column_w * (SCREEN_H / PLAY_W);
    let crop_y = CROP_Y.clamp(0.0, (IMAGE_H - crop_h).max(0.0));

    for screen in 0..TABLE_SCREENS {
        let x0 = screen as f32 * column_w;
        let rect = Rect::new(x0, crop_y, x0 + column_w, crop_y + crop_h);
        commands.spawn((
            Sprite {
                image: background.clone(),
                rect: Some(rect),
                custom_size: Some(Vec2::new(PLAY_W, SCREEN_H)),
                ..default()
            },
            Transform::from_xyz(
                PLAY_W * 0.5,
                screen_bottom(screen) + SCREEN_H * 0.5,
                -20.0,
            ),
            Name::new("Space background"),
        ));
    }

    // A dark wash keeps the printed text, lamps and score panels readable over
    // the brighter parts of the artwork.
    commands.spawn((
        Sprite::from_color(Color::srgba_u8(5, 8, 26, 135), Vec2::new(PLAY_W, TABLE_H)),
        Transform::from_xyz(PLAY_W * 0.5, TABLE_H * 0.5, -19.0),
        Name::new("Background tint"),
    ));
}

fn spawn_starfield(commands: &mut Commands) {
    let mut state = 0x9e3779b9u32;
    for _ in 0..130 {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        let x = 8.0 + (state % 206) as f32;
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        let y = 6.0 + (state % 946) as f32;
        let bright = (state % 3) as u8;
        let color = match bright {
            0 => Color::srgba_u8(55, 80, 140, 140),
            1 => Color::srgba_u8(90, 80, 160, 120),
            _ => Color::srgba_u8(140, 160, 220, 110),
        };
        let size = if state % 11 == 0 { 2.0 } else { 1.0 };
        commands.spawn((
            Sprite::from_color(color, Vec2::splat(size)),
            Transform::from_xyz(x, y, -18.0),
        ));
    }
}

fn spawn_floor_title(commands: &mut Commands, images: &mut Assets<Image>, screen: usize) {
    let text = if screen == 0 {
        "CLOSE TO THE EDGE"
    } else {
        SCREEN_TITLES[screen]
    };
    let y = screen_bottom(screen) + 218.0;
    spawn_text(
        commands,
        images,
        0,
        Vec2::new(10.0, y),
        3.0,
        text,
        [90, 120, 165],
        1.0,
    );
}

const CYAN: Color = Color::srgb(0.25, 0.9, 1.0);
const CYAN_DIM: Color = Color::srgb(0.12, 0.55, 0.8);
const MAGENTA_DIM: Color = Color::srgb(0.6, 0.18, 0.55);

pub fn zone(commands: &mut Commands, kind: ZoneKind, pos: Vec2, radius: f32) {
    commands.spawn((
        Zone {
            kind,
            radius,
            half: Vec2::ZERO,
            inside: false,
        },
        Transform::from_xyz(pos.x, pos.y, 0.0),
    ));
}

pub fn wall(commands: &mut Commands, a: Vec2, b: Vec2, thickness: f32, color: Color) {
    let delta = b - a;
    let length = delta.length();
    if length < 0.01 {
        return;
    }
    let angle = delta.y.atan2(delta.x);
    let mid = (a + b) * 0.5;
    commands.spawn((
        RigidBody::Static,
        Collider::rectangle(length, thickness),
        Transform::from_xyz(mid.x, mid.y, 1.0)
            .with_rotation(Quat::from_rotation_z(angle)),
        Sprite::from_color(color, Vec2::new(length, thickness)),
        WallMarker {
            color,
            length,
            thickness,
        },
        Restitution::new(0.45).with_combine_rule(CoefficientCombine::Max),
        Friction::new(0.08),
        RenderLayers::layer(0),
    ));
}

fn spawn_bumpers(commands: &mut Commands, art: &Art) {
    for (index, (pos, color_index)) in BUMPERS.iter().enumerate() {
        commands.spawn((
            RigidBody::Static,
            Collider::circle(11.0),
            Bumper {
                cooldown: 0.0,
                flash: 0.0,
                index,
            },
            CollisionEventsEnabled,
            Restitution::new(0.8).with_combine_rule(CoefficientCombine::Max),
            Friction::new(0.03),
            Sprite {
                image: art.bumper(*color_index),
                custom_size: Some(Vec2::splat(27.0)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 4.0),
            RenderLayers::layer(0),
        ));
    }
}

fn spawn_slingshot(commands: &mut Commands, pos: Vec2, side: f32) {
    let angle = side * 0.62;
    commands.spawn((
        RigidBody::Static,
        Collider::rectangle(30.0, 6.0),
        Slingshot { side, cooldown: 0.0 },
        CollisionEventsEnabled,
        Restitution::new(1.05).with_combine_rule(CoefficientCombine::Max),
        Friction::new(0.03),
        Sprite::from_color(Color::srgb_u8(230, 70, 150), Vec2::new(30.0, 7.0)),
        Transform::from_xyz(pos.x, pos.y, 4.0)
            .with_rotation(Quat::from_rotation_z(angle)),
        RenderLayers::layer(0),
    ));
}

fn spawn_abc(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    screen: usize,
    positions: &[Vec2; 3],
    label: &str,
) {
    for (i, pos) in positions.iter().enumerate() {
        spawn_lamp(commands, images, *pos, screen, i, &["A", "B", "C"][i]);
        zone(
            commands,
            ZoneKind::Lane {
                screen,
                letter: i,
            },
            *pos,
            11.0,
        );
    }
    let mid = (positions[0] + positions[2]) * 0.5;
    let label_y = if screen == 0 {
        positions[0].y - 15.0
    } else {
        positions[0].y + 14.0
    };
    spawn_text(
        commands,
        images,
        0,
        Vec2::new(mid.x - 12.0, label_y),
        3.0,
        label,
        [120, 200, 255],
        1.0,
    );
}

fn spawn_lamp(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    pos: Vec2,
    screen: usize,
    letter: usize,
    text: &str,
) {
    commands.spawn((
        Sprite::from_color(Color::srgb_u8(35, 48, 70), Vec2::new(12.0, 14.0)),
        Transform::from_xyz(pos.x, pos.y, 2.0),
        Lamp { screen, letter },
    ));
    spawn_text(
        commands,
        images,
        0,
        Vec2::new(pos.x - 2.5, pos.y + 3.5),
        3.2,
        text,
        [255, 255, 255],
        1.0,
    );
}

fn spawn_panel(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    pos: Vec2,
    size: Vec2,
    label: &str,
) {
    commands.spawn((
        Sprite::from_color(Color::srgba_u8(20, 26, 55, 220), size),
        Transform::from_xyz(pos.x, pos.y, -1.0),
    ));
    // Neon border.
    let half = size * 0.5;
    for (a, b) in [
        (Vec2::new(-half.x, -half.y), Vec2::new(half.x, -half.y)),
        (Vec2::new(-half.x, half.y), Vec2::new(half.x, half.y)),
        (Vec2::new(-half.x, -half.y), Vec2::new(-half.x, half.y)),
        (Vec2::new(half.x, -half.y), Vec2::new(half.x, half.y)),
    ] {
        let mid = pos + (a + b) * 0.5;
        let d = b - a;
        commands.spawn((
            Sprite::from_color(Color::srgb_u8(80, 140, 210), Vec2::new(d.length(), 1.0)),
            Transform::from_xyz(mid.x, mid.y, -0.5)
                .with_rotation(Quat::from_rotation_z(d.y.atan2(d.x))),
        ));
    }
    spawn_text(
        commands,
        images,
        0,
        Vec2::new(pos.x - label.chars().count() as f32 * 3.0, pos.y + half.y - 11.0),
        3.0,
        label,
        [180, 220, 255],
        1.0,
    );
}

fn add_table_value(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    pos: Vec2,
    z: f32,
    kind: TableValueKind,
) {
    let text = match kind {
        TableValueKind::Score => "000000",
        TableValueKind::Bonus => "1000",
    };
    let entity = spawn_text(
        commands,
        images,
        0,
        pos,
        z,
        text,
        [255, 240, 170],
        1.0,
    );
    commands.entity(entity).insert(TableValue(kind));
}

fn spawn_kickback(
    commands: &mut Commands,
    art: &Art,
    images: &mut Assets<Image>,
    index: usize,
) {
    let pos = KICKBACK_POS[index];
    commands.spawn((
        Sprite {
            image: art.hole_off.clone(),
            custom_size: Some(Vec2::splat(24.0)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 3.0),
        HoleVisual { index },
    ));
    spawn_text(
        commands,
        images,
        0,
        Vec2::new(pos.x - 12.0, pos.y + 11.0),
        3.1,
        "KICK",
        [220, 230, 255],
        1.0,
    );
    zone(commands, ZoneKind::Kickback { index }, pos, 13.0);
}

fn spawn_drop_targets(commands: &mut Commands, images: &mut Assets<Image>) {
    for (i, x) in DROP_TARGET_X.iter().enumerate() {
        commands.spawn((
            RigidBody::Static,
            Collider::rectangle(18.0, 6.0),
            DropTarget { index: i },
            CollisionEventsEnabled,
            Restitution::new(0.35),
            Friction::new(0.1),
            Sprite::from_color(Color::srgb_u8(240, 190, 60), Vec2::new(18.0, 6.0)),
            Transform::from_xyz(*x, DROP_TARGET_Y, 4.0),
            RenderLayers::layer(0),
        ));
        spawn_text(
            commands,
            images,
            0,
            Vec2::new(x - 2.5, DROP_TARGET_Y + 8.0),
            3.2,
            ["A", "B", "C"][i],
            [255, 255, 255],
            1.0,
        );
    }
    spawn_text(
        commands,
        images,
        0,
        Vec2::new(88.0, DROP_TARGET_Y + 22.0),
        3.0,
        "DROP TARGETS",
        [140, 200, 255],
        1.0,
    );
}

fn spawn_slot(commands: &mut Commands, images: &mut Assets<Image>, art: &Art) {
    let pos = SLOT_CABINET_POS;
    // Solid cabinet the ball can bounce off.
    commands.spawn((
        RigidBody::Static,
        Collider::rectangle(88.0, 38.0),
        Restitution::new(0.6).with_combine_rule(CoefficientCombine::Max),
        Friction::new(0.05),
        Sprite::from_color(Color::srgb_u8(28, 20, 54), Vec2::new(88.0, 40.0)),
        Transform::from_xyz(pos.x, pos.y, 2.0),
        RenderLayers::layer(0),
    ));

    // A peaked cap on the cabinet. The old flat top was a shelf where a slow
    // ball could balance forever ("stuck at the slots"). Two short sloped
    // rails meet just above the centre, so every landing rolls off to one
    // side and back into play.
    let roof = Color::srgb(0.62, 0.34, 0.92);
    wall(
        commands,
        pos + Vec2::new(-44.0, 18.0),
        pos + Vec2::new(0.0, 30.0),
        3.0,
        roof,
    );
    wall(
        commands,
        pos + Vec2::new(0.0, 30.0),
        pos + Vec2::new(44.0, 18.0),
        3.0,
        roof,
    );
    // Small bright cap at the ridge, so the joint reads as part of the art.
    commands.spawn((
        Sprite::from_color(Color::srgb_u8(180, 120, 240), Vec2::new(8.0, 3.0)),
        Transform::from_xyz(pos.x, pos.y + 30.0, 2.6),
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb_u8(120, 70, 190), Vec2::new(90.0, 1.0)),
        Transform::from_xyz(pos.x, pos.y + 20.0, 2.5),
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb_u8(120, 70, 190), Vec2::new(90.0, 1.0)),
        Transform::from_xyz(pos.x, pos.y - 20.0, 2.5),
    ));
    spawn_text(
        commands,
        images,
        0,
        Vec2::new(pos.x - 21.0, pos.y + 26.0),
        3.0,
        "ROLLER SLOTS",
        [255, 210, 120],
        1.0,
    );
    for i in 0..3 {
        let x = pos.x - 28.0 + i as f32 * 28.0;
        commands.spawn((
            Sprite {
                image: art.symbols()[0].clone(),
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..default()
            },
            Transform::from_xyz(x, pos.y + 2.0, 3.0),
            SlotReel { index: i },
        ));
        commands.spawn((
            Sprite::from_color(Color::srgb_u8(10, 10, 20), Vec2::new(24.0, 24.0)),
            Transform::from_xyz(x, pos.y + 2.0, 2.8),
        ));
    }
    spawn_text(
        commands,
        images,
        0,
        Vec2::new(SLOT_TRIGGER_POS.x - 12.0, SLOT_TRIGGER_POS.y - 4.0),
        3.0,
        "SLOT",
        [255, 230, 150],
        1.0,
    );
    zone(
        commands,
        ZoneKind::SlotTrigger,
        SLOT_TRIGGER_POS,
        14.0,
    );
}

fn spawn_spinner(commands: &mut Commands, art: &Art, _images: &mut Assets<Image>, pos: Vec2) {
    commands.spawn((
        Sprite {
            image: art.spinner.clone(),
            custom_size: Some(Vec2::new(28.0, 4.0)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 4.0),
        Spinner,
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb_u8(80, 100, 150), Vec2::splat(3.0)),
        Transform::from_xyz(pos.x, pos.y, 3.5),
    ));
}

fn spawn_spinner_label(commands: &mut Commands, images: &mut Assets<Image>, pos: Vec2) {
    spawn_text(
        commands,
        images,
        0,
        Vec2::new(pos.x - 18.0, pos.y + 10.0),
        3.0,
        "SPINNER",
        [150, 220, 255],
        1.0,
    );
}

fn spawn_plunger(commands: &mut Commands) {
    for i in 0..3 {
        commands.spawn((
            Sprite::from_color(Color::srgb_u8(180, 180, 210), Vec2::new(15.0, 2.0)),
            Transform::from_xyz(LANE_CENTER, 18.0 + i as f32 * 3.0, 3.0),
            PlungerVisual { base_y: 18.0 + i as f32 * 3.0 },
        ));
    }
    commands.spawn((
        Sprite::from_color(Color::srgb_u8(255, 120, 90), Vec2::new(17.0, 5.0)),
        Transform::from_xyz(LANE_CENTER, 24.0, 3.1),
        PlungerVisual { base_y: 24.0 },
    ));
}

/// Update lamps, holes, drop targets, spinner and the plunger every frame.
pub fn animate_table(
    game: Res<Game>,
    mut lamps: Query<(&Lamp, &mut Sprite)>,
    mut holes: Query<(&HoleVisual, &mut Sprite), Without<Lamp>>,
    art: Res<Art>,
    mut targets: Query<(&DropTarget, &mut Visibility), Without<Lamp>>,
    mut spinner: Query<&mut Transform, With<Spinner>>,
    time: Res<Time>,
) {
    for (lamp, mut sprite) in &mut lamps {
        let lit = game
            .letters
            .get(lamp.screen)
            .map(|letters| letters[lamp.letter])
            .unwrap_or(false);
        sprite.color = if lit {
            Color::srgb_u8(80, 255, 170)
        } else {
            Color::srgb_u8(35, 48, 70)
        };
    }

    for (hole, mut sprite) in &mut holes {
        let armed = game.kickback.get(hole.index).copied().unwrap_or(false);
        let wanted = if armed {
            art.hole_on.clone()
        } else {
            art.hole_off.clone()
        };
        if sprite.image != wanted {
            sprite.image = wanted;
        }
    }

    for (target, mut visibility) in &mut targets {
        let down = game.targets_down[target.index];
        *visibility = if down {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }

    let angle = time.elapsed_secs() * game.spinner_speed;
    for mut transform in &mut spinner {
        transform.rotation = Quat::from_rotation_z(angle);
    }
}
