//! The 3D presentation layer.
//!
//! The pinball rules and the 120 Hz Avian simulation stay exactly the same.
//! This module gives that simulation a modern body: the printed table art is
//! rendered once into a texture and laid on a slab, then rails, bumpers,
//! flippers, the slot cabinet, the plunger and a polished steel ball are built
//! as real 3D hardware above it. A perspective camera follows the ball up and
//! down the four screens.

use std::f32::consts::FRAC_PI_2;

use bevy::camera::visibility::RenderLayers;
use bevy::camera::RenderTarget;
use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::common::*;
use crate::game::Game;
use crate::physics::{Ball, Bumper, CameraRig, DropTarget, Flipper, Plunger, TableCamera};
use crate::table::{
    HoleVisual, Lamp, Spinner, WallMarker, BUMPERS, KICKBACK_POS, SLOT_CABINET_POS,
    SLOT_TRIGGER_POS,
};

// ---------------------------------------------------------------------------
// Scene constants. These are the presentation numbers listed in MECHANICS.md.
// ---------------------------------------------------------------------------
pub const CAMERA_FOV_DEG: f32 = 42.0;
pub const CAMERA_HEIGHT: f32 = 315.0;
pub const CAMERA_BACK_OFFSET: f32 = 250.0;
pub const CAMERA_LOOK_AHEAD: f32 = 0.0;
pub const TABLE_TEXTURE_WIDTH: u32 = 448;
pub const TABLE_TEXTURE_HEIGHT: u32 = 1920;
pub const SLAB_DEPTH: f32 = 12.0;
pub const RAIL_HEIGHT: f32 = 17.0;
pub const BALL_HEIGHT: f32 = 5.0;

#[derive(Component)]
pub struct Ball3D;

#[derive(Component)]
pub struct BallShadow3D;

#[derive(Component)]
pub struct Table3D;

#[derive(Component)]
pub struct Plunger3D {
    pub base_y: f32,
}

#[derive(Component)]
pub struct Spinner3D;

#[derive(Resource, Default)]
pub struct BumperGlow {
    pub rings: Vec<Handle<StandardMaterial>>,
    pub base: Vec<LinearRgba>,
}

#[derive(Resource, Default)]
pub struct KickbackGlow {
    pub rings: Vec<Handle<StandardMaterial>>,
}

/// Spawn the render-to-texture camera, the 3D camera, the table slab and all
/// the raised hardware. Runs after the 2D table has been spawned so it can
/// inspect the real physics entities.
pub fn setup_render3d(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    flippers: Query<(Entity, &Flipper)>,
    bumpers: Query<(Entity, &Bumper)>,
    holes: Query<(Entity, &HoleVisual)>,
    targets: Query<(Entity, &DropTarget)>,
    spinner: Query<Entity, With<Spinner>>,
    walls: Query<(&Transform, &WallMarker)>,
    lamps: Query<(&Transform, &Lamp)>,
) {
    // ------------------------------------------------------------------
    // The printed playfield. Rendering the existing table art into a texture
    // keeps every original label, lamp and dynamic score display readable
    // while the 3D world supplies the perspective and the depth.
    // ------------------------------------------------------------------
    let mut table_image = Image::new_target_texture(
        TABLE_TEXTURE_WIDTH,
        TABLE_TEXTURE_HEIGHT,
        TextureFormat::Rgba8Unorm,
        Some(TextureFormat::Rgba8UnormSrgb),
    );
    table_image.sampler = ImageSampler::nearest();
    let table_image = images.add(table_image);

    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            clear_color: ClearColorConfig::Custom(Color::srgb_u8(3, 5, 16)),
            ..default()
        },
        RenderTarget::Image(table_image.clone().into()),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: TABLE_H,
            },
            // The art pass sits at z=1000; the default 1000-unit far plane
            // would cut off every decal at z<0 (the printed background, screen
            // seams, panels and the space artwork). Extend it so the whole
            // layered table is captured in the texture.
            far: 2000.0,
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(PLAY_W * 0.5, TABLE_H * 0.5, 1000.0),
        RenderLayers::layer(LAYER_TABLE),
        Name::new("Table art pass"),
    ));

    // ------------------------------------------------------------------
    // Materials: dark anodised metal, chrome, rubber, neon.
    // ------------------------------------------------------------------
    let slab_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.018, 0.024, 0.055),
        metallic: 0.35,
        perceptual_roughness: 0.52,
        ..default()
    });
    let frame_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.055, 0.075, 0.13),
        metallic: 0.9,
        perceptual_roughness: 0.26,
        reflectance: 0.35,
        ..default()
    });
    let chrome_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.72, 0.78, 0.9),
        metallic: 0.95,
        perceptual_roughness: 0.16,
        reflectance: 0.55,
        ..default()
    });
    let rubber_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.34, 0.06),
        metallic: 0.15,
        perceptual_roughness: 0.48,
        emissive: LinearRgba::rgb(0.02, 0.005, 0.0),
        ..default()
    });
    let rubber_top_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.78, 0.18),
        metallic: 0.1,
        perceptual_roughness: 0.35,
        emissive: LinearRgba::rgb(0.12, 0.05, 0.0),
        ..default()
    });
    let cyan_glow = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.9, 1.0),
        emissive: LinearRgba::rgb(0.0, 2.4, 3.4),
        metallic: 0.15,
        perceptual_roughness: 0.3,
        ..default()
    });
    let magenta_glow = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.25, 0.75),
        emissive: LinearRgba::rgb(2.6, 0.05, 1.5),
        metallic: 0.15,
        perceptual_roughness: 0.3,
        ..default()
    });
    let gold_glow = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.72, 0.15),
        emissive: LinearRgba::rgb(2.2, 1.15, 0.05),
        metallic: 0.4,
        perceptual_roughness: 0.3,
        ..default()
    });
    let slot_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.16, 0.07, 0.3),
        metallic: 0.75,
        perceptual_roughness: 0.28,
        ..default()
    });
    let table_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(table_image),
        unlit: true,
        perceptual_roughness: 0.9,
        ..default()
    });

    // ------------------------------------------------------------------
    // Table slab, surface, cabinet rails and neon trim.
    // ------------------------------------------------------------------
    let backdrop_mat = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(backdrop_texture())),
        unlit: true,
        ..default()
    });
    commands.spawn((
        Name::new("Neon hall backdrop"),
        Mesh3d(meshes.add(Rectangle::new(920.0, 1500.0))),
        MeshMaterial3d(backdrop_mat),
        Transform::from_xyz(PLAY_W * 0.5, TABLE_H * 0.5, -150.0),
    ));

    commands.spawn((
        Name::new("3D table slab"),
        Mesh3d(meshes.add(Cuboid::new(PLAY_W + 14.0, TABLE_H + 14.0, SLAB_DEPTH))),
        MeshMaterial3d(slab_mat),
        Transform::from_xyz(PLAY_W * 0.5, TABLE_H * 0.5, -SLAB_DEPTH * 0.5),
        Table3D,
    ));

    commands.spawn((
        Name::new("Printed playfield"),
        Mesh3d(meshes.add(table_quad(PLAY_W, TABLE_H))),
        MeshMaterial3d(table_mat),
        Transform::from_xyz(PLAY_W * 0.5, TABLE_H * 0.5, 0.05),
    ));

    // Cabinet rails. Inner faces line up with the physics walls at x=4/220.
    for (x, mat) in [(-1.5, cyan_glow.clone()), (PLAY_W + 1.5, magenta_glow.clone())] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(9.0, TABLE_H + 10.0, RAIL_HEIGHT))),
            MeshMaterial3d(frame_mat.clone()),
            Transform::from_xyz(x, TABLE_H * 0.5, RAIL_HEIGHT * 0.5),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.6, TABLE_H, 1.2))),
            MeshMaterial3d(mat),
            Transform::from_xyz(
                if x < 0.0 { x + 4.8 } else { x - 4.8 },
                TABLE_H * 0.5,
                RAIL_HEIGHT - 0.3,
            ),
        ));
    }
    for y in [-1.5, TABLE_H + 1.5] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(PLAY_W + 12.0, 9.0, RAIL_HEIGHT))),
            MeshMaterial3d(frame_mat.clone()),
            Transform::from_xyz(PLAY_W * 0.5, y, RAIL_HEIGHT * 0.5),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(PLAY_W, 1.6, 1.2))),
            MeshMaterial3d(cyan_glow.clone()),
            Transform::from_xyz(PLAY_W * 0.5, y, RAIL_HEIGHT - 0.3),
        ));
    }

    // Corner pillars, like the chrome posts of a real cabinet.
    for (x, y) in [
        (-1.5, -1.5),
        (PLAY_W + 1.5, -1.5),
        (-1.5, TABLE_H + 1.5),
        (PLAY_W + 1.5, TABLE_H + 1.5),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(7.0, RAIL_HEIGHT + 14.0))),
            MeshMaterial3d(chrome_mat.clone()),
            Transform::from_xyz(x, y, (RAIL_HEIGHT + 14.0) * 0.5 - 2.0),
        ));
    }

    // A tiny raised lip around each of the four screens, so the seams read in
    // three dimensions as well as in the printed art.
    for screen in 1..4 {
        let y = screen as f32 * SCREEN_H;
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(PLAY_W - 6.0, 1.2, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.55, 0.9),
                emissive: LinearRgba::rgb(0.05, 0.4, 0.9),
                ..default()
            })),
            Transform::from_xyz(PLAY_W * 0.5, y, 0.6),
        ));
    }

    // ------------------------------------------------------------------
    // Physics walls become raised rails. Their 2D colours are preserved.
    // ------------------------------------------------------------------
    for (transform, wall) in &walls {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(
                wall.length.max(0.5),
                wall.thickness.max(0.5),
                RAIL_HEIGHT * 0.62,
            ))),
            MeshMaterial3d(rail_material(&mut materials, wall.color)),
            Transform {
                translation: Vec3::new(
                    transform.translation.x,
                    transform.translation.y,
                    RAIL_HEIGHT * 0.31,
                ),
                rotation: transform.rotation,
                ..default()
            },
        ));
    }

    // Launch-lane divider gets a dedicated chrome rail and a magenta cap.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(5.0, 900.0, 13.0))),
        MeshMaterial3d(frame_mat.clone()),
        Transform::from_xyz(197.0, 456.0, 6.5),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 888.0, 1.4))),
        MeshMaterial3d(magenta_glow.clone()),
        Transform::from_xyz(197.0, 456.0, 13.0),
    ));

    // ------------------------------------------------------------------
    // Bumpers: a dark barrel, a glowing crown and a polished centre.
    // ------------------------------------------------------------------
    let bumper_colors = [
        (Color::srgb(0.2, 0.95, 1.0), LinearRgba::rgb(0.0, 2.2, 3.2)),
        (Color::srgb(1.0, 0.25, 0.75), LinearRgba::rgb(2.8, 0.08, 1.5)),
        (Color::srgb(1.0, 0.78, 0.2), LinearRgba::rgb(2.4, 1.5, 0.04)),
    ];
    let mut glow = BumperGlow::default();
    let barrel = meshes.add(Cylinder::new(12.2, 8.0));
    let crown = meshes.add(Cylinder::new(13.0, 1.8));
    let cap = meshes.add(Cylinder::new(8.2, 2.4));
    let dot = meshes.add(Sphere::new(2.2).mesh().uv(16, 10));
    let barrel_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.06, 0.08, 0.14),
        metallic: 0.8,
        perceptual_roughness: 0.25,
        ..default()
    });
    for (pos, color_index) in BUMPERS.iter() {
        let (base, emissive) = bumper_colors[color_index % 3];
        let ring = materials.add(StandardMaterial {
            base_color: base,
            emissive,
            metallic: 0.2,
            perceptual_roughness: 0.25,
            ..default()
        });
        glow.rings.push(ring.clone());
        glow.base.push(emissive);
        commands.spawn((
            Mesh3d(barrel.clone()),
            MeshMaterial3d(barrel_mat.clone()),
            Transform::from_xyz(pos.x, pos.y, 4.0),
        ));
        commands.spawn((
            Mesh3d(crown.clone()),
            MeshMaterial3d(ring.clone()),
            Transform::from_xyz(pos.x, pos.y, 8.9),
        ));
        commands.spawn((
            Mesh3d(cap.clone()),
            MeshMaterial3d(chrome_mat.clone()),
            Transform::from_xyz(pos.x, pos.y, 10.2),
        ));
        commands.spawn((
            Mesh3d(dot.clone()),
            MeshMaterial3d(ring),
            Transform::from_xyz(pos.x, pos.y, 11.7),
        ));
    }
    commands.insert_resource(glow);

    // Keep the bumper sprites dark/transparent behind the 3D hardware.
    for (entity, _bumper) in &bumpers {
        commands.entity(entity).insert(Visibility::Hidden);
    }

    // ------------------------------------------------------------------
    // Flippers: a tapered rubber bat with a bright top edge.
    // ------------------------------------------------------------------
    let bat = meshes.add(Capsule3d::new(4.4, 33.2));
    let bat_top = meshes.add(Capsule3d::new(1.9, 34.0));
    for (entity, _flipper) in &flippers {
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Mesh3d(bat.clone()),
                MeshMaterial3d(rubber_mat.clone()),
                Transform::from_xyz(21.0, 0.0, 2.2)
                    .with_rotation(Quat::from_rotation_z(-FRAC_PI_2)),
            ));
            parent.spawn((
                Mesh3d(bat_top.clone()),
                MeshMaterial3d(rubber_top_mat.clone()),
                Transform::from_xyz(21.0, 1.6, 3.5)
                    .with_rotation(Quat::from_rotation_z(-FRAC_PI_2)),
            ));
        });
    }

    // Pivot caps.
    for (pivot_x, _) in [(crate::physics::FLIPPER_PIVOT_X[0], 0), (crate::physics::FLIPPER_PIVOT_X[1], 1)] {
        for screen in 0..4 {
            commands.spawn((
                Mesh3d(meshes.add(Cylinder::new(4.2, 2.2))),
                MeshMaterial3d(chrome_mat.clone()),
                Transform::from_xyz(pivot_x, crate::common::screen_bottom(screen) + crate::physics::FLIPPER_Y, 9.0),
            ));
        }
    }

    // ------------------------------------------------------------------
    // Ball.
    // ------------------------------------------------------------------
    let ball_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.88, 0.91, 0.98),
        metallic: 0.72,
        perceptual_roughness: 0.12,
        reflectance: 0.9,
        ..default()
    });
    commands.spawn((
        Name::new("3D pinball"),
        Mesh3d(meshes.add(Sphere::new(BALL_RADIUS).mesh().uv(28, 18))),
        MeshMaterial3d(ball_mat),
        Transform::from_xyz(BALL_START.x, BALL_START.y, BALL_HEIGHT),
        Ball3D,
    ));

    let shadow_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(images.add(ball_shadow_texture())),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    commands.spawn((
        Name::new("Ball shadow"),
        Mesh3d(meshes.add(Rectangle::new(19.0, 19.0))),
        MeshMaterial3d(shadow_mat),
        Transform::from_xyz(BALL_START.x, BALL_START.y, 0.08),
        BallShadow3D,
    ));

    // ------------------------------------------------------------------
    // Slot cabinet on screen 2.
    // ------------------------------------------------------------------
    let pos = SLOT_CABINET_POS;
    // Frame bars leave the printed reels visible through the middle.
    for (dx, dy, w, h) in [
        (0.0, 21.0, 94.0, 6.0),
        (0.0, -21.0, 94.0, 6.0),
        (-46.0, 0.0, 6.0, 42.0),
        (46.0, 0.0, 6.0, 42.0),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(w, h, 10.0))),
            MeshMaterial3d(slot_mat.clone()),
            Transform::from_xyz(pos.x + dx, pos.y + dy, 5.0),
        ));
    }
    for x in [pos.x - 46.0, pos.x + 46.0] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.6, 44.0, 1.4))),
            MeshMaterial3d(magenta_glow.clone()),
            Transform::from_xyz(x, pos.y, 10.4),
        ));
    }
    let reel_ring = meshes.add(Torus::new(0.82, 1.18));
    for index in 0..3 {
        let transform = Transform::from_xyz(
            pos.x - 28.0 + index as f32 * 28.0,
            pos.y + 2.0,
            1.8,
        )
        .with_rotation(Quat::from_rotation_x(FRAC_PI_2));
        commands.spawn((
            Mesh3d(reel_ring.clone()),
            MeshMaterial3d(gold_glow.clone()),
            transform,
        ));
    }
    // A small "SLOT" beacon above the trigger.
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(3.0, 1.4))),
        MeshMaterial3d(gold_glow.clone()),
        Transform::from_xyz(SLOT_TRIGGER_POS.x, SLOT_TRIGGER_POS.y, 1.6),
    ));

    // ------------------------------------------------------------------
    // Kickback holes: glowing rings that turn green when armed.
    // ------------------------------------------------------------------
    let mut kick_glow = KickbackGlow::default();
    let hole_ring = meshes.add(Torus::new(0.78, 1.22));
    for index in 0..3 {
        let pos = KICKBACK_POS[index];
        let mat = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.18, 0.25),
            emissive: LinearRgba::rgb(2.6, 0.02, 0.05),
            metallic: 0.2,
            perceptual_roughness: 0.25,
            ..default()
        });
        kick_glow.rings.push(mat.clone());
        commands.spawn((
            Mesh3d(hole_ring.clone()),
            MeshMaterial3d(mat),
            Transform::from_xyz(pos.x, pos.y, 1.7)
                .with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
        ));
    }
    commands.insert_resource(kick_glow);

    let _ = &holes;
    for (entity, _hole) in &holes {
        commands.entity(entity).insert(Visibility::Hidden);
    }

    // ------------------------------------------------------------------
    // Drop targets are small raised bars that inherit the 2D hidden state.
    // ------------------------------------------------------------------
    let target_mesh = meshes.add(Cuboid::new(18.0, 6.0, 7.0));
    let target_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.78, 0.2),
        emissive: LinearRgba::rgb(1.4, 0.7, 0.02),
        metallic: 0.3,
        perceptual_roughness: 0.35,
        ..default()
    });
    for (entity, _target) in &targets {
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Mesh3d(target_mesh.clone()),
                MeshMaterial3d(target_mat.clone()),
                Transform::from_xyz(0.0, 0.0, 1.0),
            ));
        });
    }

    // ------------------------------------------------------------------
    // Spinner: a real vertical paddle that turns with the 2D spinner.
    // ------------------------------------------------------------------
    for entity in &spinner {
        let paddle = meshes.add(Cuboid::new(28.0, 2.6, 10.0));
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Mesh3d(paddle),
                MeshMaterial3d(chrome_mat.clone()),
                Transform::from_xyz(0.0, 0.0, 2.0),
                Spinner3D,
            ));
            parent.spawn((
                Mesh3d(meshes.add(Cylinder::new(2.4, 4.0))),
                MeshMaterial3d(gold_glow.clone()),
                Transform::from_xyz(0.0, 0.0, 4.0),
            ));
        });
    }

    // ------------------------------------------------------------------
    // Plunger: chrome shaft, rubber knob and a compressed spring.
    // ------------------------------------------------------------------
    let plunger = commands
        .spawn((
            Plunger3D { base_y: 24.0 },
            Transform::from_xyz(LANE_CENTER, 24.0, 4.0),
            Visibility::default(),
        ))
        .id();
    commands.entity(plunger).with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(Cylinder::new(2.0, 30.0))),
            MeshMaterial3d(chrome_mat.clone()),
            Transform::from_xyz(0.0, -13.0, 0.0),
        ));
        parent.spawn((
            Mesh3d(meshes.add(Sphere::new(4.2).mesh().uv(16, 10))),
            MeshMaterial3d(magenta_glow.clone()),
            Transform::from_xyz(0.0, 0.0, 2.2),
        ));
        for i in 0..4 {
            parent.spawn((
                Mesh3d(meshes.add(Torus::new(0.9, 1.25))),
                MeshMaterial3d(frame_mat.clone()),
                Transform::from_xyz(0.0, -4.0 - i as f32 * 4.0, 0.0)
                    .with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
            ));
        }
    });

    // ------------------------------------------------------------------
    // Small lamp glints above the table's A/B/C inserts.
    // ------------------------------------------------------------------
    let glint = meshes.add(Sphere::new(1.4).mesh().uv(12, 8));
    let glint_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 1.0, 0.8),
        emissive: LinearRgba::rgb(0.2, 1.6, 0.9),
        ..default()
    });
    for (transform, lamp) in &lamps {
        if lamp.screen == 0 || lamp.screen == 3 {
            commands.spawn((
                Mesh3d(glint.clone()),
                MeshMaterial3d(glint_mat.clone()),
                Transform::from_xyz(transform.translation.x, transform.translation.y, 2.0),
            ));
        }
    }

    // ------------------------------------------------------------------
    // Camera and lighting.
    // ------------------------------------------------------------------
    commands.spawn((
        Camera3d::default(),
        TableCamera,
        Camera {
            order: 2,
            clear_color: ClearColorConfig::Custom(Color::srgb_u8(1, 2, 8)),
            ..default()
        },
        Projection::Perspective(PerspectiveProjection {
            fov: CAMERA_FOV_DEG.to_radians(),
            near: 1.0,
            far: 1500.0,
            ..default()
        }),
        Transform::from_xyz(PLAY_W * 0.5, -55.0, CAMERA_HEIGHT)
            .looking_at(Vec3::new(PLAY_W * 0.5, 215.0, 0.0), Vec3::Y),
        Bloom {
            intensity: 0.12,
            ..default()
        },
        Name::new("3D table camera"),
    ));

    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.55, 0.65, 0.95),
        brightness: 320.0,
        ..default()
    });
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.82, 0.9, 1.0),
            illuminance: 9_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -0.85,
            -0.45,
            0.0,
        )),
    ));
    for (y, color) in [
        (120.0, Color::srgb(0.05, 0.7, 1.0)),
        (360.0, Color::srgb(1.0, 0.15, 0.65)),
        (600.0, Color::srgb(0.1, 0.8, 0.9)),
        (840.0, Color::srgb(0.9, 0.2, 0.75)),
    ] {
        commands.spawn((
            PointLight {
                color,
                intensity: 2_200_000.0,
                range: 260.0,
                radius: 18.0,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::from_xyz(PLAY_W * 0.5, y, 145.0),
        ));
    }

    let _ = (targets, plunger);
}

fn rail_material(materials: &mut Assets<StandardMaterial>, color: Color) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: color,
        emissive: LinearRgba::rgb(color.to_linear().red * 0.35, color.to_linear().green * 0.35, color.to_linear().blue * 0.35),
        metallic: 0.55,
        perceptual_roughness: 0.32,
        ..default()
    })
}

/// A dark arcade-hall backdrop: a deep blue-to-violet gradient with a sparse
/// starfield. It is deliberately dim so the neon table hardware stays the
/// brightest thing on screen.
fn backdrop_texture() -> Image {
    const W: u32 = 512;
    const H: u32 = 512;
    let mut data = vec![0u8; (W * H * 4) as usize];
    let mut state = 0x1f12_9e37u32;
    let mut stars = Vec::new();
    for _ in 0..180 {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        let x = (state % W) as i32;
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        let y = (state % H) as i32;
        let bright = (state % 3) as u8;
        stars.push((x, y, bright));
    }
    for y in 0..H {
        let f = y as f32 / H as f32;
        let r = (3.0 + 9.0 * f) as u8;
        let g = (5.0 + 10.0 * f) as u8;
        let b = (18.0 + 34.0 * f) as u8;
        for x in 0..W {
            let i = ((y * W + x) * 4) as usize;
            data[i] = r;
            data[i + 1] = g;
            data[i + 2] = b;
            data[i + 3] = 255;
        }
    }
    for (x, y, bright) in stars {
        let (c, size) = match bright {
            0 => ([90u8, 130, 210], 1),
            1 => ([140u8, 110, 220], 1),
            _ => ([210u8, 230, 255], 2),
        };
        for dy in 0..size {
            for dx in 0..size {
                let px = (x + dx).clamp(0, W as i32 - 1) as u32;
                let py = (y + dy).clamp(0, H as i32 - 1) as u32;
                let i = ((py * W + px) * 4) as usize;
                data[i..i + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
            }
        }
    }
    let mut image = Image::new_fill(
        Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    image.data = Some(data);
    image
}

/// A soft round contact shadow, so the steel ball reads as sitting above the
/// playfield rather than floating in front of it.
fn ball_shadow_texture() -> Image {
    const S: u32 = 64;
    let mut data = vec![0u8; (S * S * 4) as usize];
    for y in 0..S {
        for x in 0..S {
            let px = (x as f32 + 0.5) / S as f32 * 2.0 - 1.0;
            let py = (y as f32 + 0.5) / S as f32 * 2.0 - 1.0;
            let r = (px * px + py * py).sqrt();
            let a = ((1.0 - r).clamp(0.0, 1.0)).powf(1.8) * 0.55;
            let i = ((y * S + x) * 4) as usize;
            data[i..i + 4].copy_from_slice(&[0, 0, 0, (a * 255.0) as u8]);
        }
    }
    let mut image = Image::new_fill(
        Extent3d {
            width: S,
            height: S,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    image.data = Some(data);
    image
}

/// A flat quad with UVs that put the top of the rendered image at the high-y
/// end of the table.
fn table_quad(width: f32, height: f32) -> Mesh {
    let hw = width * 0.5;
    let hh = height * 0.5;
    Mesh::new(
        PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [-hw, -hh, 0.0],
            [hw, -hh, 0.0],
            [hw, hh, 0.0],
            [-hw, hh, 0.0],
        ],
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; 4])
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
    )
    .with_inserted_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]))
}

/// Keep the perspective camera glued to the ball's screen, with the same
/// smooth follow the 2D build used.
pub fn follow_camera_3d(
    rig: Res<CameraRig>,
    mut camera: Query<&mut Transform, (With<TableCamera>, Without<Ball>)>,
) {
    for mut transform in &mut camera {
        transform.translation = Vec3::new(
            PLAY_W * 0.5,
            rig.y - CAMERA_BACK_OFFSET,
            CAMERA_HEIGHT,
        );
        transform.look_at(
            Vec3::new(PLAY_W * 0.5, rig.y + CAMERA_LOOK_AHEAD, 0.0),
            Vec3::Y,
        );
    }
}

/// Keep the playfield on the left 70% of a centred 4:3 rectangle. In windowed
/// 960x720 that rectangle is the whole window; in borderless fullscreen the
/// table and the sidebar stay perfectly proportioned with dark bars around
/// them, instead of stretching across a widescreen monitor.
pub fn resize_3d_viewport(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut table_camera: Query<&mut Camera, With<TableCamera>>,
    mut composite_camera: Query<
        &mut Camera,
        (With<crate::ui::HudCompositeCamera>, Without<TableCamera>),
    >,
    mut report: Local<(UVec2, u32)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let size = window.physical_size();
    let (x, y, w, h) = letterbox_rect(size.x, size.y);
    let play_w = ((w as f32) * (PLAY_W / DESIGN_W)).round() as u32;
    report.1 += 1;
    if report.1 == 1 || size != report.0 {
        report.0 = size;
        info!(
            "Display: {:?} {}x{} px, 4:3 content {}x{} at ({}, {}), table {} px wide",
            window.mode, size.x, size.y, w, h, x, y, play_w
        );
    }
    for mut camera in &mut table_camera {
        camera.viewport = Some(bevy::camera::Viewport {
            physical_position: UVec2::new(x, y),
            physical_size: UVec2::new(play_w.max(1), h.max(1)),
            ..default()
        });
    }
    for mut camera in &mut composite_camera {
        camera.viewport = Some(bevy::camera::Viewport {
            physical_position: UVec2::new(x, y),
            physical_size: UVec2::new(w.max(1), h.max(1)),
            ..default()
        });
    }
}

pub fn sync_ball_3d(
    balls: Query<&Transform, (With<Ball>, Without<Ball3D>, Without<BallShadow3D>)>,
    mut meshes: Query<&mut Transform, (With<Ball3D>, Without<Ball>, Without<BallShadow3D>)>,
    mut shadows: Query<&mut Transform, (With<BallShadow3D>, Without<Ball>, Without<Ball3D>)>,
) {
    let Ok(ball) = balls.single() else {
        for mut transform in &mut meshes {
            transform.translation.z = -500.0;
        }
        for mut transform in &mut shadows {
            transform.translation.z = -500.0;
        }
        return;
    };
    for mut transform in &mut meshes {
        transform.translation.x = ball.translation.x;
        transform.translation.y = ball.translation.y;
        transform.translation.z = BALL_HEIGHT;
        transform.rotation = Quat::from_rotation_z(0.0);
    }
    for mut transform in &mut shadows {
        transform.translation.x = ball.translation.x;
        transform.translation.y = ball.translation.y;
        transform.translation.z = 0.08;
    }
}

pub fn sync_plunger_3d(
    plunger: Res<Plunger>,
    mut parts: Query<(&mut Transform, &Plunger3D)>,
) {
    for (mut transform, part) in &mut parts {
        transform.translation.y = part.base_y - plunger.charge * 7.0;
    }
}

pub fn animate_3d(
    game: Res<Game>,
    bumpers: Query<&Bumper>,
    glow: Res<BumperGlow>,
    kick_glow: Res<KickbackGlow>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut flash = [0.0f32; 16];
    for bumper in &bumpers {
        if bumper.index < flash.len() {
            flash[bumper.index] = (bumper.flash / 0.14).clamp(0.0, 1.0);
        }
    }
    for (index, ring) in glow.rings.iter().enumerate() {
        if let Some(mut material) = materials.get_mut(ring) {
            let f = flash.get(index).copied().unwrap_or(0.0);
            let base = glow
                .base
                .get(index)
                .copied()
                .unwrap_or(LinearRgba::rgb(0.2, 0.2, 0.2));
            material.emissive = LinearRgba::rgb(
                base.red + f * 3.0,
                base.green + f * 5.0,
                base.blue + f * 5.0,
            );
        }
    }
    for (index, ring) in kick_glow.rings.iter().enumerate() {
        if let Some(mut material) = materials.get_mut(ring) {
            let armed = game.kickback.get(index).copied().unwrap_or(false);
            material.base_color = if armed {
                Color::srgb(0.25, 1.0, 0.55)
            } else {
                Color::srgb(1.0, 0.18, 0.25)
            };
            material.emissive = if armed {
                LinearRgba::rgb(0.1, 3.0, 0.8)
            } else {
                LinearRgba::rgb(2.6, 0.02, 0.05)
            };
        }
    }
}
