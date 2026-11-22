//! Pinball physics: fixed 120 Hz simulation, a CCD ball, moving flippers,
//! plunger, bumpers, slingshots, table sensors and drain handling.

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::camera::visibility::RenderLayers;
use bevy::sprite::Anchor;

use crate::art::Art;
use crate::audio::{SoundEvent, SoundQueue};
use crate::common::*;
use crate::game::{Game, Mode, Phase};
use crate::slots::SlotMachine;
use crate::table::{
    Zone, ZoneKind, ABC_BOTTOM, ABC_TOP, DROP_TARGET_X, LANE_EXIT_Y, SLOT_CABINET_POS,
    SLOT_TRIGGER_POS,
};

pub const FLIPPER_LEN: f32 = 42.0;
pub const FLIPPER_WIDTH: f32 = 7.0;
pub const FLIPPER_HALF_WIDTH: f32 = FLIPPER_WIDTH * 0.5;
pub const FLIPPER_PIVOT_X: [f32; 2] = [62.0, 162.0];
pub const FLIPPER_Y: f32 = 38.0;
pub const FLIPPER_REST: f32 = 0.42; // radians away from horizontal
pub const FLIPPER_SWEEP: f32 = 0.84; // total travel in radians (48 degrees)
/// A real flipper snaps up hard and drops back a little more gently. These
/// are angular speeds in rad/s, not the old frame-dependent "t" step.
pub const FLIPPER_RAISE_MAX: f32 = 30.0;
pub const FLIPPER_RAISE_ACCEL: f32 = 1_500.0;
pub const FLIPPER_RETURN_MAX: f32 = 13.0;
pub const FLIPPER_RETURN_ACCEL: f32 = 320.0;
/// Braking used near either end stop so the flipper stops exactly at the
/// limits instead of overshooting and wobbling.
pub const FLIPPER_BRAKE_ACCEL: f32 = 2_400.0;
/// Minimum time between two ball impulses from the same flipper.
pub const FLIPPER_KICK_COOLDOWN: f32 = 0.060;

#[derive(Component)]
pub struct Ball;

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum FlipperSide {
    Left,
    Right,
}

#[derive(Component)]
pub struct TableCamera;

#[derive(Component)]
pub struct Flipper {
    pub side: FlipperSide,
    pub screen: usize,
    /// 0 = rest, 1 = fully raised.
    pub t: f32,
    /// World-space angular velocity in rad/s. Negative is clockwise.
    pub omega: f32,
    /// Current world-space angle of the flipper, stored separately because
    /// `Rotation` normalises angles and we need the unwrapped value.
    pub angle: f32,
    pub kick_cooldown: f32,
    pub held_last_step: bool,
    /// True while the player is holding the button for this flipper.
    pub driving: bool,
}

/// The direction the flipper sweeps when it is raised: +1 (counter-clockwise)
/// for the left flipper, -1 (clockwise) for the right one.
pub fn flipper_raise_sign(side: FlipperSide) -> f32 {
    match side {
        FlipperSide::Left => 1.0,
        FlipperSide::Right => -1.0,
    }
}

/// Rest angle in world radians.
pub fn flipper_rest_angle(side: FlipperSide) -> f32 {
    match side {
        FlipperSide::Left => -FLIPPER_REST,
        FlipperSide::Right => -std::f32::consts::PI + FLIPPER_REST,
    }
}

/// World angle for a stroke position of 0 (rest) to 1 (fully raised).
pub fn flipper_angle(side: FlipperSide, t: f32) -> f32 {
    flipper_rest_angle(side) + flipper_raise_sign(side) * FLIPPER_SWEEP * t.clamp(0.0, 1.0)
}

#[derive(Component)]
pub struct Bumper {
    pub cooldown: f32,
    pub flash: f32,
    pub index: usize,
}

#[derive(Component)]
pub struct Slingshot {
    pub side: f32,
    pub cooldown: f32,
}

#[derive(Component)]
pub struct DropTarget {
    pub index: usize,
}

#[derive(Resource, Default)]
pub struct Plunger {
    pub charge: f32,
    pub holding: bool,
}

#[derive(Resource)]
pub struct CameraRig {
    pub y: f32,
}

impl Default for CameraRig {
    fn default() -> Self {
        Self { y: 120.0 }
    }
}

#[derive(Resource, Default)]
pub struct BallWatchdog {
    pub slow_time: f32,
    pub in_lane_time: f32,
    /// Time a very slow ball has spent on or against the slot cabinet.
    pub slot_time: f32,
    /// Time a slow ball has spent resting on a passive flipper.
    pub flipper_time: f32,
}

/// A ball that has almost stopped in the slot cabinet area gets a small,
/// sideways shove after a moment. The peaked cabinet roof should already let
/// it roll away, but this is the safety net for a ball balanced exactly on
/// the ridge or wedged against the cabinet's side.
pub fn slot_escape(pos: Vec2, speed: f32) -> Option<Vec2> {
    let near_slot = pos.x > SLOT_CABINET_POS.x - 58.0
        && pos.x < SLOT_CABINET_POS.x + 58.0
        && pos.y > SLOT_CABINET_POS.y - 32.0
        && pos.y < SLOT_CABINET_POS.y + 48.0;
    if !near_slot || speed >= 40.0 {
        return None;
    }
    let offset = pos.x - SLOT_CABINET_POS.x;
    let side = if offset < -2.0 { -1.0 } else { 1.0 };
    Some(Vec2::new(side * 300.0, 260.0))
}

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

pub fn spawn_flippers(commands: &mut Commands, art: &Art) {
    for screen in 0..4 {
        let b = screen_bottom(screen);
        for (i, side) in [FlipperSide::Left, FlipperSide::Right].into_iter().enumerate() {
            let pivot_x = FLIPPER_PIVOT_X[i];
            let pivot = Vec2::new(pivot_x, b + FLIPPER_Y);
            let angle = flipper_angle(side, 0.0);
            let rotation = Quat::from_rotation_z(angle);
            // The right flipper is the same body rotated to point left. Its
            // texture is mirrored so the bright rubber edge still faces up.
            let image = match side {
                FlipperSide::Left => art.flipper.clone(),
                FlipperSide::Right => art.flipper_right.clone(),
            };
            commands.spawn((
                RigidBody::Kinematic,
                // A capsule gives the flipper a rounded tip and a smooth base,
                // so the ball never catches on an invisible square corner.
                Collider::capsule_endpoints(
                    FLIPPER_HALF_WIDTH,
                    Vec2::ZERO,
                    Vec2::new(FLIPPER_LEN, 0.0),
                ),
                Flipper {
                    side,
                    screen,
                    t: 0.0,
                    omega: 0.0,
                    angle,
                    kick_cooldown: 0.0,
                    held_last_step: false,
                    driving: false,
                },
                Rotation::radians(angle),
                Sprite {
                    image,
                    custom_size: Some(Vec2::new(48.0, 10.0)),
                    ..default()
                },
                Anchor::CENTER_LEFT,
                Transform::from_xyz(pivot.x, pivot.y, 6.0).with_rotation(rotation),
                Restitution::new(0.25).with_combine_rule(CoefficientCombine::Max),
                // Rubber has grip, but it must not overcome the 24 degree
                // rest slope: a ball left on a passive flipper has to roll
                // off towards the tip instead of hanging there.
                Friction::new(0.35).with_combine_rule(CoefficientCombine::Max),
                SleepingDisabled,
                CollisionEventsEnabled,
                RenderLayers::layer(0),
            ));

            // Round metal pivot cap.
            commands.spawn((
                Sprite {
                    image: art.pivot.clone(),
                    custom_size: Some(Vec2::splat(9.0)),
                    ..default()
                },
                Transform::from_xyz(pivot.x, pivot.y, 6.5),
            ));
        }
    }
}

pub fn spawn_ball(
    commands: &mut Commands,
    art: &Art,
) -> Entity {
    let ball = commands
        .spawn((
            Ball,
            Name::new("Pinball"),
            RigidBody::Dynamic,
            Collider::circle(BALL_RADIUS),
            Restitution::new(0.30).with_combine_rule(CoefficientCombine::Max),
            Friction::new(0.06).with_combine_rule(CoefficientCombine::Min),
            LinearDamping(0.0),
            AngularDamping(0.4),
        ))
        .id();
    commands.entity(ball).insert((
        // The ball is allowed to roll. Locking its rotation made static
        // friction able to hold it on a 24 degree flipper face forever; a
        // rolling sphere always works its way downhill instead.
        // CCD is essential: a launched ball exceeds 2000 px/s.
        SweptCcd::default().with_filter(CcdFilter::ALL),
        SpeculativeCcd::default(),
        LinearVelocity(Vec2::ZERO),
        CollisionEventsEnabled,
        Sprite {
            image: art.ball.clone(),
            custom_size: Some(Vec2::splat(BALL_RADIUS * 2.0 + 1.0)),
            ..default()
        },
        // The 3D steel sphere is the visible ball; the old flat sprite stays in
        // the texture atlas but is hidden so it cannot double up under it.
        Visibility::Hidden,
        Transform::from_xyz(BALL_START.x, 30.0, 8.0),
        RenderLayers::layer(0),
    ));
    ball
}

/// Spawn and despawn the ball as the game asks for it.
pub fn manage_ball(
    mut commands: Commands,
    mut game: ResMut<Game>,
    art: Res<Art>,
    balls: Query<Entity, With<Ball>>,
    mut plunger: ResMut<Plunger>,
    mut watchdog: ResMut<BallWatchdog>,
) {
    let has_ball = !balls.is_empty();
    if (game.mode == Mode::Menu || game.mode == Mode::GameOver) && has_ball {
        for entity in &balls {
            commands.entity(entity).despawn();
        }
        game.ball_active = false;
        game.request_ball = false;
        return;
    }

    if game.mode == Mode::Playing && game.request_ball && !has_ball {
        spawn_ball(&mut commands, &art);
        game.request_ball = false;
        game.ball_active = true;
        game.phase = Phase::Ready;
        plunger.charge = 0.0;
        plunger.holding = false;
        watchdog.slow_time = 0.0;
        watchdog.in_lane_time = 0.0;
        watchdog.slot_time = 0.0;
        watchdog.flipper_time = 0.0;
    }
}

// ---------------------------------------------------------------------------
// Player-controlled parts
// ---------------------------------------------------------------------------

pub fn control_flippers(
    fixed: Res<Time<Fixed>>,
    controls: Res<Controls>,
    game: Res<Game>,
    mut flippers: Query<(&mut Flipper, &mut Rotation, &mut Transform)>,
    mut queue: ResMut<SoundQueue>,
) {
    // FixedUpdate gets the physics delta; using the render delta here made the
    // flippers run faster or slower depending on the frame rate.
    let dt = fixed.delta_secs();
    for (mut flipper, mut rotation, mut transform) in &mut flippers {
        flipper.kick_cooldown = (flipper.kick_cooldown - dt).max(0.0);

        let wants = !game.tilted()
            && game.mode == Mode::Playing
            && game.phase != Phase::Bonus
            && flipper.screen == game.active_screen
            && match flipper.side {
                FlipperSide::Left => controls.left,
                FlipperSide::Right => controls.right,
            };

        flipper.driving = wants;
        let raise_sign = flipper_raise_sign(flipper.side);
        let target = if wants { 1.0 } else { 0.0 };

        // Work out which way the flipper should travel and how fast it may go.
        // This follows the button, not the current angle: a flipper held at
        // the top must sit still instead of chattering around the end stop.
        let raising = wants;
        let (top_speed, accel) = if raising {
            (FLIPPER_RAISE_MAX, FLIPPER_RAISE_ACCEL)
        } else {
            (FLIPPER_RETURN_MAX, FLIPPER_RETURN_ACCEL)
        };
        let travel_sign = if raising { raise_sign } else { -raise_sign };
        let wanted_omega = travel_sign * top_speed;

        // Angular acceleration keeps the stroke smooth rather than teleporting
        // the flipper between two angles in one physics step.
        let change = (wanted_omega - flipper.omega).clamp(-accel * dt, accel * dt);
        flipper.omega += change;

        // Start braking early enough to stop exactly at rest or full stroke.
        let remaining_angle = (target - flipper.t).abs() * FLIPPER_SWEEP;
        let brake_speed =
            (2.0 * FLIPPER_BRAKE_ACCEL * remaining_angle).sqrt().max(1.5);
        if flipper.omega * travel_sign > 0.0 {
            flipper.omega = flipper.omega.signum() * flipper.omega.abs().min(brake_speed);
        }

        // t is normalised travel; omega is world-space, so convert with the
        // side sign. Clamp exactly to the two end stops.
        let t_step = raise_sign * flipper.omega * dt / FLIPPER_SWEEP;
        let mut next_t = flipper.t + t_step;
        if next_t >= 1.0 {
            next_t = 1.0;
            flipper.omega = 0.0;
        } else if next_t <= 0.0 {
            next_t = 0.0;
            flipper.omega = 0.0;
        }
        flipper.t = next_t;
        flipper.angle = flipper_angle(flipper.side, flipper.t);

        // Both the physics rotation and the sprite rotation follow the exact
        // same angle, so they can never drift apart.
        *rotation = Rotation::radians(flipper.angle);
        transform.rotation = Quat::from_rotation_z(flipper.angle);

        // A crisp solenoid click when the player starts a stroke.
        if wants && !flipper.held_last_step {
            queue.push(SoundEvent::FlipperHit);
        }
        flipper.held_last_step = wants;
    }
}

/// Geometry of a ball resting on the working face of a raising flipper.
/// Returns the push direction and the speed the ball should gain. Strength
/// grows with the distance from the pivot, so a tip shot is much stronger
/// than a hit near the hinge.
fn flipper_contact(flipper: &Flipper, pivot: Vec2, ball_pos: Vec2) -> Option<(Vec2, f32)> {
    let raise_sign = flipper_raise_sign(flipper.side);
    let raise_speed = flipper.omega * raise_sign;
    // A held-up, stationary flipper is a passive surface: a ball that falls
    // onto it must not be fired into the air. Only a flipper that is actually
    // sweeping (or has just started its stroke) can launch the ball.
    let just_started = flipper.driving && flipper.t < 0.30;
    if raise_speed <= 0.5 && !just_started {
        return None;
    }
    let dir = Vec2::from_angle(flipper.angle);
    let normal = Vec2::new(-dir.y, dir.x);
    let normal = if normal.y < 0.0 { -normal } else { normal };

    let relative = ball_pos - pivot;
    // Only the top working face launches the ball; the underside just pushes
    // it away through the normal collider response.
    if relative.dot(normal) < -1.0 {
        return None;
    }

    let along = relative.dot(dir).clamp(0.0, FLIPPER_LEN);
    let closest = pivot + dir * along;
    let distance = (ball_pos - closest).length();
    let contact_distance = FLIPPER_HALF_WIDTH + BALL_RADIUS + 2.0;
    if distance > contact_distance {
        return None;
    }

    // A resting ball is struck by the full solenoid force within a few ms.
    // Only apply that minimum early in the stroke; near the end stop the
    // flipper is slowing down and must not act like a stationary cannon.
    let boost = if just_started || flipper.t < 0.75 {
        FLIPPER_RAISE_MAX * 0.55
    } else {
        0.0
    };
    let effective_omega = flipper.omega.abs().max(boost);
    let power = (effective_omega * (along + 8.0).min(FLIPPER_LEN) * 1.05 + 170.0)
        .min(1_500.0);
    Some((normal, power))
}

/// A ball can come to rest on the rubber even when the flipper is not being
/// driven: halfway down the face, balanced on the rounded tip, or wedged at
/// the hinge. Returns the downhill direction and the face normal when the ball
/// is close enough to be sitting on this flipper.
pub fn flipper_resting_contact(
    flipper: &Flipper,
    pivot: Vec2,
    ball_pos: Vec2,
) -> Option<(Vec2, Vec2)> {
    let dir = Vec2::from_angle(flipper.angle);
    let normal = Vec2::new(-dir.y, dir.x);
    let normal = if normal.y < 0.0 { -normal } else { normal };

    let relative = ball_pos - pivot;
    if relative.dot(normal) < -1.0 {
        return None;
    }
    let along = relative.dot(dir).clamp(0.0, FLIPPER_LEN);
    let closest = pivot + dir * along;
    if (ball_pos - closest).length() > FLIPPER_HALF_WIDTH + BALL_RADIUS + 3.0 {
        return None;
    }
    // Positive `dir` points at the tip. On a resting flipper that is downhill;
    // on a raised one it is uphill, so choose whichever way actually falls.
    let downhill = if dir.y <= 0.0 { dir } else { -dir };
    Some((downhill, normal))
}

/// Impulse the ball whenever a raising flipper is sweeping into it. This runs
/// every physics step, so a ball that is already resting on the flipper (and
/// therefore produces no fresh collision-start event) is still launched.
pub fn flipper_impacts(
    game: Res<Game>,
    mut flippers: Query<(&mut Flipper, &Transform)>,
    mut balls: Query<(&Transform, &mut LinearVelocity), With<Ball>>,
    mut queue: ResMut<SoundQueue>,
) {
    if !game.in_play() {
        return;
    }
    let Ok((ball_transform, mut velocity)) = balls.single_mut() else {
        return;
    };
    let ball_pos = ball_transform.translation.truncate();
    for (mut flipper, transform) in &mut flippers {
        if flipper.kick_cooldown > 0.0 {
            continue;
        }
        let pivot = transform.translation.truncate();
        let Some((direction, power)) = flipper_contact(&flipper, pivot, ball_pos) else {
            continue;
        };
        let current = velocity.0.dot(direction);
        if current < power - 4.0 {
            velocity.0 += direction * (power - current);
            flipper.kick_cooldown = FLIPPER_KICK_COOLDOWN;
            queue.push(SoundEvent::FlipperHit);
        } else {
            // Already travelling faster than the flipper can push; remember
            // the contact so it is not re-tested for every substep.
            flipper.kick_cooldown = FLIPPER_KICK_COOLDOWN;
        }
    }
}

pub fn control_plunger(
    time: Res<Time>,
    controls: Res<Controls>,
    mut game: ResMut<Game>,
    mut plunger: ResMut<Plunger>,
    mut balls: Query<(&mut LinearVelocity, &mut Position), With<Ball>>,
    mut queue: ResMut<SoundQueue>,
) {
    if game.mode != Mode::Playing {
        plunger.holding = false;
        return;
    }
    if game.phase != Phase::Ready {
        if plunger.holding {
            plunger.holding = false;
            plunger.charge = 0.0;
        }
        return;
    }

    let Ok((mut velocity, mut position)) = balls.single_mut() else {
        return;
    };

    if controls.plunger {
        if !plunger.holding {
            plunger.holding = true;
            plunger.charge = 0.0;
            queue.push(SoundEvent::PlungerCharge);
        }
        plunger.charge = (plunger.charge + time.delta_secs() / 0.95).min(1.0);
        // Keep the ball parked against the plunger while charging.
        position.0 = Vec2::new(LANE_CENTER, 27.0);
        velocity.0 = Vec2::ZERO;
    } else if plunger.holding {
        let speed = 1_760.0 + 620.0 * plunger.charge;
        velocity.0 = Vec2::new(0.0, speed);
        position.0 = Vec2::new(LANE_CENTER, 30.0);
        game.phase = Phase::Play;
        game.message("BALL IN PLAY", 0.9);
        queue.push(SoundEvent::PlungerRelease);
        plunger.holding = false;
        plunger.charge = 0.0;
    }
}

pub fn use_nudge(
    mut game: ResMut<Game>,
    mut controls: ResMut<Controls>,
    mut balls: Query<&mut LinearVelocity, With<Ball>>,
    mut queue: ResMut<SoundQueue>,
) {
    if !controls.nudge_pressed || game.mode != Mode::Playing || game.phase != Phase::Play {
        return;
    }
    // Consume the edge so two fixed steps in one rendered frame cannot nudge twice.
    controls.nudge_pressed = false;
    if game.nudge_window <= 0.0 {
        game.nudge_window = crate::game::NUDGE_WINDOW;
        game.nudge_stock = crate::game::NUDGE_LIMIT;
    }
    if game.nudge_stock == 0 {
        return;
    }
    game.nudge_stock -= 1;
    if let Ok(mut velocity) = balls.single_mut() {
        let side = if game.nudge_stock % 2 == 0 { 1.0 } else { -1.0 };
        velocity.0 += Vec2::new(210.0 * side, 330.0);
    }
    queue.push(SoundEvent::FlipperHit);
    if game.nudge_stock == 0 {
        game.tilt = crate::game::TILT_SECS;
        game.message("TILT!", 2.5);
        queue.push(SoundEvent::Tilt);
    } else {
        game.message("NUDGE", 0.6);
    }
}

// ---------------------------------------------------------------------------
// Sensors
// ---------------------------------------------------------------------------

pub fn update_zones(
    mut commands: Commands,
    mut game: ResMut<Game>,
    mut zones: Query<(&mut Zone, &Transform)>,
    mut balls: Query<(Entity, &Transform, &mut Position, &mut LinearVelocity), With<Ball>>,
    mut queue: ResMut<SoundQueue>,
) {
    let Ok((ball_entity, ball_transform, mut position, mut velocity)) = balls.single_mut() else {
        return;
    };
    let ball_pos = ball_transform.translation.truncate();

    // Launch lane: the ball is passed into the top of the playfield once it
    // reaches the end of the chute. This is the one-way exit of the lane.
    if ball_pos.x > LANE_LEFT - 2.0 && ball_pos.y > LANE_EXIT_Y && game.phase == Phase::Play {
        position.0 = Vec2::new(188.0, 908.0);
        velocity.0 = Vec2::new(-310.0, 150.0);
        game.active_screen = 3;
        game.message("TOP SCREEN", 1.2);
    }

    for (mut zone, transform) in &mut zones {
        let center = transform.translation.truncate();
        let inside = if zone.radius > 1.0 {
            (ball_pos - center).length() <= zone.radius + BALL_RADIUS
        } else {
            circle_over_rect(ball_pos, BALL_RADIUS, center, zone.half)
        };

        if !inside {
            zone.inside = false;
            continue;
        }
        if zone.inside {
            continue;
        }
        zone.inside = true;

        match zone.kind {
            ZoneKind::Lane { screen, letter } => {
                game.award_letter(screen, letter, &mut queue);
            }
            ZoneKind::Arrow { screen, letter } => {
                game.award_letter(screen, letter, &mut queue);
            }
            ZoneKind::Spinner => {
                game.spinner_speed = 30.0;
                game.add_score(crate::game::SPINNER_SCORE, &mut queue);
                queue.push(SoundEvent::RolloverLane);
            }
            ZoneKind::SlotTrigger => {
                game.try_start_slot(&mut queue);
            }
            ZoneKind::Kickback { index } => {
                if index < 3 {
                    let armed = game.kickback[index];
                    if armed {
                        game.kickback[index] = false;
                        game.add_score(crate::game::KICKBACK_SCORE, &mut queue);
                        game.message("KICKBACK SAVE!", 1.2);
                    } else {
                        game.add_score(100, &mut queue);
                    }
                    let kick = match index {
                        0 => Vec2::new(480.0, 1_250.0),
                        1 => Vec2::new(-480.0, 1_250.0),
                        _ => Vec2::new(-620.0, 760.0),
                    };
                    velocity.0 = kick;
                    position.0 = center + Vec2::new(0.0, 8.0);
                    queue.push(if armed {
                        SoundEvent::Kickback
                    } else {
                        SoundEvent::BumperHit
                    });
                }
            }
            ZoneKind::Chute => {
                game.add_score(100, &mut queue);
                queue.push(SoundEvent::ChuteDrop);
                game.message("CHUTE!", 0.8);
            }
            ZoneKind::Drain => {
                if ball_pos.y < 10.0 {
                    commands.entity(ball_entity).despawn();
                    game.drain(&mut queue);
                }
            }
            ZoneKind::LaneExit => {}
        }
    }

    if game.phase == Phase::Play && ball_pos.y < 8.0 {
        commands.entity(ball_entity).despawn();
        game.drain(&mut queue);
    }
}

pub fn update_targets(
    mut commands: Commands,
    game: Res<Game>,
    mut targets: Query<(Entity, &DropTarget, Option<&ColliderDisabled>)>,
) {
    for (entity, target, disabled) in &mut targets {
        let down = game.targets_down[target.index.min(2)];
        match (down, disabled.is_some()) {
            (true, false) => {
                commands.entity(entity).insert(ColliderDisabled);
            }
            (false, true) => {
                commands.entity(entity).remove::<ColliderDisabled>();
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Collision scoring
// ---------------------------------------------------------------------------

pub fn handle_collisions(
    mut game: ResMut<Game>,
    mut bumpers: Query<(&Transform, &mut Bumper)>,
    mut slings: Query<&mut Slingshot>,
    targets: Query<&DropTarget>,
    mut flippers: Query<(&mut Flipper, &Transform)>,
    mut balls: Query<(&Transform, &mut LinearVelocity), With<Ball>>,
    mut reader: MessageReader<CollisionStart>,
    mut queue: ResMut<SoundQueue>,
) {
    if game.mode != Mode::Playing {
        return;
    }
    for event in reader.read() {
        let Ok((ball_transform, mut velocity)) = balls.single_mut() else {
            continue;
        };
        let ball_pos = ball_transform.translation.truncate();
        // Find which of the two entities is a gameplay object. Events are for
        // colliders; our components live on the same entities.
        for entity in [event.collider1, event.collider2] {
            if let Ok((bumper_transform, mut bumper)) = bumpers.get_mut(entity) {
                if bumper.cooldown <= 0.0 {
                    bumper.cooldown = 0.10;
                    bumper.flash = 0.14;
                    game.add_score(crate::game::BUMPER_SCORE, &mut queue);
                    queue.push(SoundEvent::BumperHit);
                    let center = bumper_transform.translation.truncate();
                    let dir = (ball_pos - center).normalize_or_zero();
                    velocity.0 += dir * 360.0 + Vec2::Y * 120.0;
                }
            }
            if let Ok(mut sling) = slings.get_mut(entity) {
                if sling.cooldown <= 0.0 {
                    sling.cooldown = 0.12;
                    game.add_score(crate::game::SLINGSHOT_SCORE, &mut queue);
                    queue.push(SoundEvent::BumperHit);
                    velocity.0 += Vec2::new(-sling.side * 280.0, 380.0);
                }
            }
            if let Ok(target) = targets.get(entity) {
                game.drop_target_hit(target.index, &mut queue);
            }
            if let Ok((mut flipper, flipper_transform)) = flippers.get_mut(entity) {
                // Backup for very fast balls: CCD reports the contact even when
                // the flipper swept past between two proximity checks.
                if flipper.kick_cooldown <= 0.0 && game.phase == Phase::Play {
                    let pivot = flipper_transform.translation.truncate();
                    if let Some((direction, power)) =
                        flipper_contact(&flipper, pivot, ball_pos)
                    {
                        let current = velocity.0.dot(direction);
                        if current < power - 4.0 {
                            velocity.0 += direction * (power - current);
                            queue.push(SoundEvent::FlipperHit);
                        }
                        flipper.kick_cooldown = FLIPPER_KICK_COOLDOWN;
                    }
                }
            }
        }
    }
}

/// Slowly wear off the visual cooldowns.
pub fn tick_physics_timers(
    time: Res<Time>,
    mut bumpers: Query<&mut Bumper>,
    mut slings: Query<&mut Slingshot>,
) {
    let dt = time.delta_secs();
    for mut bumper in &mut bumpers {
        bumper.cooldown = (bumper.cooldown - dt).max(0.0);
        bumper.flash = (bumper.flash - dt).max(0.0);
    }
    for mut sling in &mut slings {
        sling.cooldown = (sling.cooldown - dt).max(0.0);
    }
}

// ---------------------------------------------------------------------------
// Camera follow and ball safety
// ---------------------------------------------------------------------------

pub fn follow_ball(
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut rig: ResMut<CameraRig>,
    balls: Query<&Transform, With<Ball>>,
    flippers: Query<(&Flipper, &Transform)>,
    mut plunger: ResMut<Plunger>,
    mut watchdog: ResMut<BallWatchdog>,
    mut balls_vel: Query<(&Transform, &mut LinearVelocity), With<Ball>>,
) {
    if let Ok(ball) = balls.single() {
        let y = ball.translation.y;
        game.active_screen = screen_at(y);
        let target = y.clamp(120.0, TABLE_H - 120.0);
        rig.y += (target - rig.y) * (1.0 - (-11.0 * time.delta_secs()).exp());
        // At the very top and bottom the camera cannot scroll further, so snap
        // to the clamped position. This keeps a fast ball fully on screen.
        if y > TABLE_H - 120.0 || y < 120.0 {
            rig.y = target;
        }
    }

    // If the ball settles in the launch lane during play (e.g. the player let
    // go too early), give it a gentle automatic relaunch instead of leaving it
    // stuck forever.
    if game.phase == Phase::Play {
        if let Ok((transform, mut velocity)) = balls_vel.single_mut() {
            let pos = transform.translation.truncate();
            if pos.x > LANE_LEFT && pos.y < 120.0 && velocity.0.length() < 40.0 {
                watchdog.in_lane_time += time.delta_secs();
                if watchdog.in_lane_time > 1.0 {
                    velocity.0 = Vec2::new(0.0, 1_900.0);
                    watchdog.in_lane_time = 0.0;
                }
            } else {
                watchdog.in_lane_time = 0.0;
            }

            let speed = velocity.0.length();
            if speed < 18.0 && game.in_play() {
                watchdog.slow_time += time.delta_secs();
                if watchdog.slow_time > 6.0 {
                    velocity.0 += Vec2::new(120.0, 260.0);
                    watchdog.slow_time = 0.0;
                }
            } else {
                watchdog.slow_time = 0.0;
            }

            // The slot cabinet has its own, quicker escape. A ball can come to
            // rest against it while the reels are still spinning; waiting six
            // seconds for the general watchdog feels like a lost ball.
            if let Some(shove) = slot_escape(pos, speed) {
                watchdog.slot_time += time.delta_secs();
                if watchdog.slot_time > 1.1 {
                    velocity.0 += shove;
                    watchdog.slot_time = 0.0;
                }
            } else {
                watchdog.slot_time = 0.0;
            }

            // A slow ball sitting on a passive flipper (on the rubber, on the
            // rounded tip or wedged at the hinge) must not hang there. Give it
            // a small downhill shove after 0.75 s. A deliberately held-up
            // flipper is skipped, so cradling a ball still works.
            let mut resting = None;
            if speed < 55.0 {
                for (flipper, transform) in &flippers {
                    if flipper.driving || flipper.omega.abs() > 0.5 {
                        continue;
                    }
                    let pivot = transform.translation.truncate();
                    if let Some((downhill, normal)) =
                        flipper_resting_contact(flipper, pivot, pos)
                    {
                        resting = Some((downhill, normal));
                        break;
                    }
                }
            }
            if let Some((downhill, normal)) = resting {
                watchdog.flipper_time += time.delta_secs();
                if watchdog.flipper_time > 0.75 {
                    velocity.0 += downhill * 220.0 + normal * 55.0;
                    watchdog.flipper_time = 0.0;
                }
            } else {
                watchdog.flipper_time = 0.0;
            }
        }
    }

    if plunger.charge > 0.0 && game.phase != Phase::Ready {
        plunger.charge = 0.0;
    }
}

pub fn update_plunger_visual(
    plunger: Res<Plunger>,
    mut parts: Query<(&mut Transform, &crate::table::PlungerVisual)>,
) {
    for (mut transform, part) in &mut parts {
        transform.translation.y = part.base_y - plunger.charge * 7.0;
    }
}

pub fn reset_on_menu(game: Res<Game>, mut rig: ResMut<CameraRig>) {
    if game.mode == Mode::Menu {
        rig.y = 120.0;
    }
}

#[allow(dead_code)]
pub fn _zone_positions() -> [Vec2; 3] {
    [ABC_BOTTOM[0], ABC_TOP[0], SLOT_TRIGGER_POS]
}

pub fn _unused(x: &mut SlotMachine) {
    let _ = x;
}

pub fn _drop_target_x() -> [f32; 3] {
    DROP_TARGET_X
}

#[cfg(test)]
mod flipper_tests {
    use super::*;

    fn flipper(side: FlipperSide, t: f32, omega: f32) -> Flipper {
        Flipper {
            side,
            screen: 0,
            t,
            omega,
            angle: flipper_angle(side, t),
            kick_cooldown: 0.0,
            held_last_step: true,
            driving: omega.abs() > 0.0,
        }
    }

    fn ball_on_face(flipper: &Flipper, pivot: Vec2, from_tip: f32) -> Vec2 {
        let dir = Vec2::from_angle(flipper.angle);
        let mut normal = Vec2::new(-dir.y, dir.x);
        if normal.y < 0.0 {
            normal = -normal;
        }
        pivot + dir * (FLIPPER_LEN - from_tip) + normal * (FLIPPER_HALF_WIDTH + BALL_RADIUS)
    }

    #[test]
    fn raising_left_flipper_launches_from_tip() {
        let pivot = Vec2::new(FLIPPER_PIVOT_X[0], FLIPPER_Y);
        let f = flipper(FlipperSide::Left, 0.0, 16.5);
        let ball = ball_on_face(&f, pivot, 0.0);
        let (direction, power) = flipper_contact(&f, pivot, ball).expect("tip contact");
        assert!(direction.x > 0.0 && direction.y > 0.0);
        assert!((800.0..=1_000.0).contains(&power), "power was {power}");
    }

    #[test]
    fn raising_right_flipper_launches_towards_the_centre() {
        let pivot = Vec2::new(FLIPPER_PIVOT_X[1], FLIPPER_Y);
        let f = flipper(FlipperSide::Right, 0.0, -16.5);
        let ball = ball_on_face(&f, pivot, 0.0);
        let (direction, _) = flipper_contact(&f, pivot, ball).expect("tip contact");
        assert!(direction.x < 0.0 && direction.y > 0.0);
    }

    #[test]
    fn held_up_flipper_is_a_passive_surface() {
        let pivot = Vec2::new(FLIPPER_PIVOT_X[0], FLIPPER_Y);
        let f = flipper(FlipperSide::Left, 1.0, 0.0);
        let ball = ball_on_face(&f, pivot, 0.0);
        assert!(flipper_contact(&f, pivot, ball).is_none());
    }

    #[test]
    fn resting_ball_on_a_flipper_face_has_a_downhill_escape() {
        let pivot = Vec2::new(FLIPPER_PIVOT_X[0], FLIPPER_Y);
        let f = flipper(FlipperSide::Left, 0.0, 0.0);
        let ball = ball_on_face(&f, pivot, 12.0);
        let (downhill, normal) = flipper_resting_contact(&f, pivot, ball)
            .expect("the ball is sitting on the rubber");
        assert!(downhill.x > 0.0 && downhill.y < 0.0, "left flipper feeds right/down");
        assert!(normal.y > 0.0);
    }

    #[test]
    fn ball_balanced_on_the_rounded_tip_is_detected() {
        let pivot = Vec2::new(FLIPPER_PIVOT_X[0], FLIPPER_Y);
        let f = flipper(FlipperSide::Left, 0.0, 0.0);
        let tip = ball_on_face(&f, pivot, 0.0);
        assert!(flipper_resting_contact(&f, pivot, tip).is_some());
    }

    #[test]
    fn ball_far_from_the_flipper_is_ignored() {
        let pivot = Vec2::new(FLIPPER_PIVOT_X[0], FLIPPER_Y);
        let f = flipper(FlipperSide::Left, 0.0, 16.5);
        let ball = ball_on_face(&f, pivot, 0.0) + Vec2::new(0.0, 30.0);
        assert!(flipper_contact(&f, pivot, ball).is_none());
    }

    #[test]
    fn slow_ball_on_the_slot_cabinet_is_shoved_off() {
        let on_ridge = SLOT_CABINET_POS + Vec2::new(0.0, 20.0);
        let shove = slot_escape(on_ridge, 2.0).expect("a resting ball must be freed");
        assert!(shove.x.abs() > 200.0 && shove.y > 100.0);

        let left_slope = SLOT_CABINET_POS + Vec2::new(-30.0, 20.0);
        let shove = slot_escape(left_slope, 2.0).expect("left slope ball must be freed");
        assert!(shove.x < 0.0, "left-side ball should be pushed left");
    }

    #[test]
    fn fast_ball_crossing_the_slot_area_is_left_alone() {
        let on_ridge = SLOT_CABINET_POS + Vec2::new(0.0, 20.0);
        assert!(slot_escape(on_ridge, 400.0).is_none());
        let far_away = Vec2::new(30.0, 400.0);
        assert!(slot_escape(far_away, 2.0).is_none());
    }
}
