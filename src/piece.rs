use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_mod_picking::prelude::*;

use crate::board::{board_to_pixel_coords, pixel_to_board_coords, Board};

pub const PIECE_AMT: usize = 6;
pub const COLOUR_AMT: usize = 2;
pub const PIECE_WIDTH: f32 = 60.;
pub const PIECE_HEIGHT: f32 = 60.;

pub const PIECE_SCALE: f32 = 2.;

pub fn piece_is_white(piece: PieceEnum) -> bool {
    (PIECE_AMT..PIECE_AMT * COLOUR_AMT).contains(&(piece as usize))
}

pub fn piece_is_black(piece: PieceEnum) -> bool {
    (0..PIECE_AMT).contains(&(piece as usize))
}

#[derive(Clone, Copy, Component, Debug)]
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
    pub sprite: SpriteSheetBundle,
    on_drag_start_listener: On<Pointer<DragStart>>,
    on_drag_listener: On<Pointer<Drag>>,
    on_drag_end_listener: On<Pointer<DragEnd>>,
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
            on_drag_start_listener: On::<Pointer<DragStart>>::run(draw_possible_moves),
            on_drag_listener: On::<Pointer<Drag>>::run(on_piece_drag),
            on_drag_end_listener: On::<Pointer<DragEnd>>::run(on_piece_drag_end),
        }
    }
}

fn on_piece_drag(
    mut drag_er: EventReader<Pointer<Drag>>,
    mut transform_query: Query<&mut Transform>,
) {
    for drag_data in drag_er.read() {
        let mut transform = transform_query.get_mut(drag_data.target).unwrap();
        transform.translation += Vec3::new(drag_data.delta.x, -drag_data.delta.y, 0.);
        transform.translation.z = 10.;
    }
}

fn on_piece_drag_end(
    mut commands: Commands,
    mut drag_er: EventReader<Pointer<DragEnd>>,
    mut transform_query: Query<&mut Transform>,
    mut board: ResMut<Board>,
    meshes: Query<Entity, With<Mesh2dHandle>>,
) {
    for drag_data in drag_er.read() {
        let mut transform = transform_query.get_mut(drag_data.target).unwrap();

        // Find where the piece was moved from in board coordinates
        let original_pos = transform.translation.xy()
            - Vec2::new(drag_data.distance.x, -drag_data.distance.y)
            + Vec2::new(PIECE_WIDTH, PIECE_HEIGHT) * PIECE_SCALE / 2.;
        let (ori_i, ori_j) = pixel_to_board_coords(original_pos.x, original_pos.y);

        // Find the new position, snapped to board coords, and move the sprite there
        let (i, j) = pixel_to_board_coords(
            transform.translation.x + PIECE_WIDTH * PIECE_SCALE / 2.,
            transform.translation.y + PIECE_HEIGHT * PIECE_SCALE / 2.,
        );

        board.move_piece(
            (ori_i, ori_j),
            (i, j),
            drag_data.target(),
            &mut transform,
            &mut commands,
        );

        // Clean up the possible move markers
        for mesh in meshes.iter() {
            commands.entity(mesh).despawn();
        }
    }
}

pub fn draw_possible_moves(
    mut drag_er: EventReader<Pointer<DragStart>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut transform_query: Query<&mut Transform>,
    board: Res<Board>,
) {
    for drag_data in drag_er.read() {
        let transform = transform_query.get_mut(drag_data.target).unwrap();
        let (piece_i, piece_j) =
            pixel_to_board_coords(transform.translation.x, transform.translation.y);

        let possible_moves = board.get_possible_moves((piece_i, piece_j));

        let shape = Mesh2dHandle(meshes.add(Circle {
            radius: PIECE_HEIGHT * PIECE_SCALE * 0.8 / 2.,
        }));
        let colour = Color::hsla(285., 0.60, 0.5, 0.85);

        for pos in possible_moves.iter() {
            let (x, y) = board_to_pixel_coords(pos.0, pos.1);

            commands.spawn(MaterialMesh2dBundle {
                mesh: shape.clone(),
                material: materials.add(colour),
                transform: Transform::from_xyz(
                    // Distribute shapes from -X_EXTENT to +X_EXTENT.
                    x, y, 2.0,
                ),
                ..default()
            });
        }
    }
}
