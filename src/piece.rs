use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::board::{board_to_pixel_coords, pixel_to_board_coords};

pub const PIECE_AMT: usize = 6;
pub const COLOUR_AMT: usize = 2;
pub const PIECE_WIDTH: f32 = 60.;
pub const PIECE_HEIGHT: f32 = 60.;

pub const PIECE_SCALE: f32 = 2.;

#[allow(dead_code)]
#[derive(Clone, Copy, Component)]
pub enum PieceEnum {
    BQueen,
    BKing,
    BRook,
    BKnight,
    BBishop,
    BPawn,
    WQueen,
    WKing,
    WRook,
    WKnight,
    WBishop,
    WPawn,
    Empty,
}

#[derive(Bundle)]
pub struct Piece {
    pub key: PieceEnum,
    pub on_drag_listener: On<Pointer<Drag>>,
    pub on_drop_listener: On<Pointer<DragEnd>>,
    pub sprite: SpriteSheetBundle,
}

impl Piece {
    pub fn new(
        (i, j): (usize, usize),
        key: PieceEnum,
        texture: Handle<Image>,
        texture_atlas_layout: Handle<TextureAtlasLayout>,
    ) -> Self {
        let (x, y) = board_to_pixel_coords(i, j);

        Self {
            key,
            sprite: SpriteSheetBundle {
                texture,
                atlas: TextureAtlas {
                    layout: texture_atlas_layout,
                    index: key as usize,
                },
                transform: Transform::from_scale(Vec3::splat(PIECE_SCALE))
                    .with_translation(Vec3::new(x, y, 1.)),
                ..default()
            },
            on_drag_listener: On::<Pointer<Drag>>::run(on_piece_drag),
            on_drop_listener: On::<Pointer<DragEnd>>::run(on_piece_dropped),
        }
    }
}

pub fn on_piece_drag(
    mut drag_er: EventReader<Pointer<Drag>>,
    mut transform_query: Query<&mut Transform>,
) {
    for drag_data in drag_er.read() {
        let mut transform = transform_query.get_mut(drag_data.target).unwrap();
        transform.translation += Vec3::new(drag_data.delta.x, -drag_data.delta.y, 0.);
        transform.translation.z = 10.;
    }
}

pub fn on_piece_dropped(
    mut drag_er: EventReader<Pointer<DragEnd>>,
    mut transform_query: Query<&mut Transform>,
) {
    for drag_data in drag_er.read() {
        let mut transform = transform_query.get_mut(drag_data.target).unwrap();

        let (i, j) = pixel_to_board_coords(
            transform.translation.x + PIECE_WIDTH * PIECE_SCALE / 2.,
            transform.translation.y + PIECE_HEIGHT * PIECE_SCALE / 2.,
        );
        let (x, y) = board_to_pixel_coords(i, j);

        transform.translation = Vec3::new(x, y, 1.)
    }
}
