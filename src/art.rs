//! Procedurally generated pixel-art textures. No external image assets are
//! required; everything is drawn from code so the project stays self-contained.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

#[derive(Resource, Clone)]
pub struct Art {
    pub ball: Handle<Image>,
    pub bumper_cyan: Handle<Image>,
    pub bumper_pink: Handle<Image>,
    pub bumper_gold: Handle<Image>,
    pub flipper: Handle<Image>,
    pub flipper_right: Handle<Image>,
    pub pivot: Handle<Image>,
    pub hole_on: Handle<Image>,
    pub hole_off: Handle<Image>,
    pub symbol_bell: Handle<Image>,
    pub symbol_cherry: Handle<Image>,
    pub symbol_eggplant: Handle<Image>,
    pub spinner: Handle<Image>,
    pub arrow_down: Handle<Image>,
}

impl Art {
    pub fn symbols(&self) -> [Handle<Image>; 3] {
        [
            self.symbol_bell.clone(),
            self.symbol_cherry.clone(),
            self.symbol_eggplant.clone(),
        ]
    }

    pub fn bumper(&self, i: usize) -> Handle<Image> {
        match i % 3 {
            0 => self.bumper_cyan.clone(),
            1 => self.bumper_pink.clone(),
            _ => self.bumper_gold.clone(),
        }
    }
}

pub fn build_art(images: &mut Assets<Image>) -> Art {
    Art {
        ball: images.add(ball_texture()),
        bumper_cyan: images.add(bumper_texture([80, 235, 255], [35, 80, 190])),
        bumper_pink: images.add(bumper_texture([255, 95, 190], [140, 30, 120])),
        bumper_gold: images.add(bumper_texture([255, 220, 70], [190, 95, 20])),
        flipper: images.add(flipper_texture(false)),
        flipper_right: images.add(flipper_texture(true)),
        pivot: images.add(pivot_texture()),
        hole_on: images.add(hole_texture([80, 255, 210])),
        hole_off: images.add(hole_texture([255, 70, 80])),
        symbol_bell: images.add(ascii_sprite(&BELL, &PALETTE)),
        symbol_cherry: images.add(ascii_sprite(&CHERRY, &PALETTE)),
        symbol_eggplant: images.add(ascii_sprite(&EGGPLANT, &PALETTE)),
        spinner: images.add(spinner_texture()),
        arrow_down: images.add(arrow_texture(false)),
    }
}

fn image_from<F: Fn(i32, i32) -> [u8; 4]>(w: u32, h: u32, f: F) -> Image {
    let mut data = vec![0u8; (w * h * 4) as usize];
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let c = f(x, y);
            let i = ((y as u32 * w + x as u32) * 4) as usize;
            data[i..i + 4].copy_from_slice(&c);
        }
    }
    let mut image = Image::new_fill(
        Extent3d {
            width: w,
            height: h,
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

fn ball_texture() -> Image {
    const S: i32 = 12;
    image_from(S as u32, S as u32, |x, y| {
        let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) - Vec2::splat(S as f32 * 0.5);
        let d = p.length();
        if d > 6.0 {
            [0, 0, 0, 0]
        } else if d > 4.8 {
            [20, 22, 38, 255]
        } else {
            // Shiny steel ball with a highlight towards the top-left.
            let light = (-p.x - p.y) * 9.0 + 150.0;
            let v = (light.clamp(70.0, 250.0)) as u8;
            if p.x < -1.0 && p.y < -1.0 && d < 2.8 {
                [255, 255, 255, 255]
            } else {
                [v, v, v.saturating_add(6), 255]
            }
        }
    })
}

fn bumper_texture(ring: [u8; 3], dark: [u8; 3]) -> Image {
    const S: i32 = 27;
    image_from(S as u32, S as u32, |x, y| {
        let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) - Vec2::splat(S as f32 * 0.5);
        let d = p.length();
        if d > 13.0 {
            [0, 0, 0, 0]
        } else if d > 11.4 {
            [245, 245, 255, 255]
        } else if d > 9.1 {
            [ring[0], ring[1], ring[2], 255]
        } else if d > 7.8 {
            [dark[0], dark[1], dark[2], 255]
        } else if d > 5.2 {
            [245, 245, 255, 255]
        } else if d > 3.0 {
            [ring[0], ring[1], ring[2], 255]
        } else {
            [255, 250, 230, 255]
        }
    })
}

fn flipper_texture(mirror_for_right: bool) -> Image {
    let w = 48i32;
    let h = 10i32;
    image_from(w as u32, h as u32, |x, y| {
        let fx = x as f32 + 0.5;
        let fy = y as f32 + 0.5;
        let center_y = h as f32 * 0.5;
        // A tapered rubber bat: fat at the hinge, slimmer towards the tip,
        // with a round tip at x = 42.
        let along = (fx / 42.0).clamp(0.0, 1.0);
        let half = 4.4 - 1.25 * along;
        let tip_center = Vec2::new(42.0, center_y);
        let d = if fx <= 42.0 {
            (fy - center_y).abs()
        } else {
            (Vec2::new(fx, fy) - tip_center).length()
        };
        if fx > 46.0 || d > half {
            [0, 0, 0, 0]
        } else if d > half - 1.0 {
            [38, 24, 18, 255]
        } else {
            // Mirror the paint for the right flipper, which is drawn rotated
            // by 180 degrees, so the light edge is on top in both cases.
            let paint_y = if mirror_for_right { h as f32 - fy } else { fy };
            if paint_y < 3.3 {
                [255, 242, 150, 255]
            } else if paint_y < 5.8 {
                [255, 198, 48, 255]
            } else if paint_y < 8.1 {
                [232, 108, 30, 255]
            } else {
                [188, 52, 20, 255]
            }
        }
    })
}

fn pivot_texture() -> Image {
    const S: i32 = 9;
    image_from(S as u32, S as u32, |x, y| {
        let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) - Vec2::splat(S as f32 * 0.5);
        let d = p.length();
        if d > 4.4 {
            [0, 0, 0, 0]
        } else if d > 3.4 {
            [245, 240, 210, 255]
        } else if d > 1.8 {
            [170, 170, 200, 255]
        } else {
            [70, 58, 90, 255]
        }
    })
}

fn hole_texture(ring: [u8; 3]) -> Image {
    const S: i32 = 24;
    image_from(S as u32, S as u32, |x, y| {
        let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) - Vec2::splat(S as f32 * 0.5);
        let d = p.length();
        if d > 12.0 {
            [0, 0, 0, 0]
        } else if d > 10.0 {
            [ring[0], ring[1], ring[2], 255]
        } else if d > 8.8 {
            [30, 30, 45, 255]
        } else {
            [2, 2, 8, 255]
        }
    })
}

fn spinner_texture() -> Image {
    let w = 26;
    let h = 4;
    image_from(w, h, |x, y| {
        if y == 0 || y == h as i32 - 1 {
            [20, 40, 70, 255]
        } else if x < 3 || x > w as i32 - 4 {
            [90, 150, 210, 255]
        } else {
            [210, 255, 255, 255]
        }
    })
}

fn arrow_texture(up: bool) -> Image {
    let w = 11;
    let h = 8;
    image_from(w, h, |x, y| {
        let yy = if up { y } else { h as i32 - 1 - y };
        let row = yy.clamp(0, 7) as usize;
        let half = (row as i32) * 3 / 4 + 1;
        let center = 5;
        if (x - center).abs() <= half {
            [255, 220, 70, 255]
        } else {
            [0, 0, 0, 0]
        }
    })
}

const PALETTE: [(char, [u8; 3]); 7] = [
    ('Y', [255, 220, 70]),
    ('O', [220, 130, 30]),
    ('R', [240, 55, 70]),
    ('G', [80, 210, 100]),
    ('P', [170, 80, 220]),
    ('W', [255, 255, 255]),
    ('D', [30, 20, 40]),
];

const BELL: [&str; 13] = [
    "............",
    ".....YY.....",
    "....YYYY....",
    "...YYYYYY...",
    "...YYYYYY...",
    "..YYYYYYYY..",
    "..YYYYYYYY..",
    ".YYYYYYYYYY.",
    "YYYYYYYYYYYY",
    "YYYYYYYYYYYY",
    "OOOOOOOOOOOO",
    ".....OO.....",
    "............",
];

const CHERRY: [&str; 13] = [
    "............",
    "....GG......",
    "...G..G.....",
    "..G....G....",
    ".RR....RR...",
    "RRRR..RRRR..",
    "RRRR..RRRR..",
    "RRRR..RRRR..",
    ".RR....RR...",
    "............",
    "............",
    "............",
    "............",
];

const EGGPLANT: [&str; 13] = [
    ".....GG.....",
    "....GGGG....",
    "...GGGGG....",
    "..PPPPPPP...",
    ".PPPPPPPPP..",
    ".PPPPPPPPP..",
    ".PPPPPPPPP..",
    "..PPPPPPP...",
    "...PPPPP....",
    "....PPP.....",
    ".....P......",
    "............",
    "............",
];

fn ascii_sprite(rows: &[&str], palette: &[(char, [u8; 3])]) -> Image {
    let h = rows.len() as u32;
    let w = rows.iter().map(|r| r.len()).max().unwrap_or(1) as u32;
    image_from(w, h, |x, y| {
        let row = rows.get(y as usize).copied().unwrap_or("");
        let ch = row.chars().nth(x as usize).unwrap_or('.');
        if ch == '.' {
            return [0, 0, 0, 0];
        }
        for (p, col) in palette {
            if *p == ch {
                return [col[0], col[1], col[2], 255];
            }
        }
        [255, 255, 255, 255]
    })
}
