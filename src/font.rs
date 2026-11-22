//! A tiny 5x7 bitmap font drawn from code, plus a text sprite that can be
//! updated when its string changes. This keeps the game pixel-crisp without
//! needing a font asset.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use bevy::camera::visibility::RenderLayers;
use crate::common::FONT_ROWS;

#[derive(Component, Clone)]
pub struct PixelText {
    pub text: String,
    pub color: [u8; 4],
    pub scale: f32,
    pub shadow: bool,
}

impl PixelText {
    pub fn new(text: impl Into<String>, color: [u8; 4], scale: f32) -> Self {
        Self {
            text: text.into(),
            color,
            scale,
            shadow: true,
        }
    }
}

/// Spawn a text sprite. `pos` is the top-left corner in the coordinate space
/// of the chosen render layer. z is the sorting depth.
pub trait TextColor {
    fn rgba(self) -> [u8; 4];
}

impl TextColor for [u8; 3] {
    fn rgba(self) -> [u8; 4] {
        [self[0], self[1], self[2], 255]
    }
}

impl TextColor for [u8; 4] {
    fn rgba(self) -> [u8; 4] {
        self
    }
}

pub fn spawn_text<C: TextColor>(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    layer: usize,
    pos: Vec2,
    z: f32,
    text: impl Into<String>,
    color: C,
    scale: f32,
) -> Entity {
    let text = text.into();
    let color = color.rgba();
    let image = text_image(&text, color, true);
    let size = image_size(&image);
    let handle = images.add(image);
    let entity = commands.spawn((
        Sprite {
            image: handle,
            custom_size: Some(size * scale),
            color: Color::WHITE,
            ..default()
        },
        Anchor::TOP_LEFT,
        Transform::from_xyz(pos.x, pos.y, z),
        PixelText::new(text, color, scale),
        RenderLayers::layer(layer),
    ));
    entity.id()
}


pub fn text_width(text: &str, scale: f32) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    (text.chars().count() as f32 * 6.0 - 1.0) * scale
}

fn image_size(image: &Image) -> Vec2 {
    Vec2::new(image.width() as f32, image.height() as f32)
}

pub fn text_image(text: &str, color: [u8; 4], shadow: bool) -> Image {
    let chars: Vec<char> = text.to_ascii_uppercase().chars().collect();
    let visible = chars.len().max(1);
    let extra = if shadow { 1 } else { 0 };
    let width = (visible * 6 - 1 + extra) as u32;
    let height = (FONT_ROWS + extra) as u32;
    let mut data = vec![0u8; (width * height * 4) as usize];

    let mut put = |x: i32, y: i32, c: [u8; 4]| {
        if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
            return;
        }
        let i = ((y as u32 * width + x as u32) * 4) as usize;
        data[i..i + 4].copy_from_slice(&c);
    };

    for (ci, ch) in chars.iter().enumerate() {
        let glyph = glyph(*ch);
        let x0 = ci as i32 * 6;
        for row in 0..FONT_ROWS {
            let bits = glyph[row];
            for col in 0..5 {
                if bits & (1 << (4 - col)) != 0 {
                    if shadow {
                        put(x0 + col as i32 + 1, row as i32 + 1, [0, 0, 0, 190]);
                    }
                    put(x0 + col as i32, row as i32, color);
                }
            }
        }
    }

    let mut image = Image::new_fill(
        Extent3d {
            width,
            height,
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

/// Replace the image behind a text entity when the text or colour changed.
///
/// When a label changes length (for example "CLOSE TO THE EDGE" becoming
/// "TOP SCREEN"), the GPU texture must be recreated at the new size. Mutating
/// an already-uploaded image in place keeps the old width and produces
/// smeared, overlapping glyphs, so a differently sized string gets a fresh
/// image asset instead.
pub fn refresh_text_images(
    mut query: Query<(&PixelText, &mut Sprite), Changed<PixelText>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (text, mut sprite) in &mut query {
        let new_image = text_image(&text.text, text.color, text.shadow);
        let new_width = new_image.width();
        let new_height = new_image.height();
        let size = Vec2::new(new_width as f32, new_height as f32);
        let handle = sprite.image.clone();
        let same_size = images
            .get(&handle)
            .map(|image| image.width() == new_width && image.height() == new_height)
            .unwrap_or(false);
        if same_size {
            if let Some(mut image) = images.get_mut(&handle) {
                *image = new_image;
            }
        } else {
            sprite.image = images.add(new_image);
        }
        sprite.custom_size = Some(size * text.scale);
    }
}

fn glyph(c: char) -> [u8; FONT_ROWS] {
    match c {
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' => [0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111],
        'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' => [0b01111, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111],
        'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111],
        'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100],
        'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
        'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
        '3' => [0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        ':' => [0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000],
        '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00110, 0b00110],
        ',' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00110, 0b00100, 0b01000],
        '-' => [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
        '+' => [0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000],
        '/' => [0b00001, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b10000],
        '>' => [0b10000, 0b01000, 0b00100, 0b00010, 0b00100, 0b01000, 0b10000],
        '<' => [0b00001, 0b00010, 0b00100, 0b01000, 0b00100, 0b00010, 0b00001],
        '!' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100],
        '?' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100],
        '*' => [0b00000, 0b10101, 0b01110, 0b11111, 0b01110, 0b10101, 0b00000],
        '\'' => [0b00100, 0b00100, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
        '"' => [0b01010, 0b01010, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
        '(' => [0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010],
        ')' => [0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000],
        '[' => [0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110],
        ']' => [0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110],
        '=' => [0b00000, 0b11111, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
        _ => [0; 7],
    }
}
