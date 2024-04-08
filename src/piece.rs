use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_mod_picking::prelude::*;

use crate::board::{board_to_pixel_coords, pixel_to_board_coords, Board, BOARD_WIDTH};

pub const PIECE_AMT: usize = 6;
pub const COLOUR_AMT: usize = 2;
pub const PIECE_AMT_PER_SIDE: usize = PIECE_AMT * 2 - 4 + BOARD_WIDTH;

pub const PIECE_WIDTH: f32 = 120.;
pub const PIECE_HEIGHT: f32 = 120.;
pub const PIECE_WIDTH_IMG: f32 = 60.;
pub const PIECE_HEIGHT_IMG: f32 = 60.;

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

        // Create a bundle with this piece's spritesheet and some listeners for picking up the pieces
        Self {
            sprite: SpriteSheetBundle {
                texture,
                atlas: TextureAtlas {
                    layout: texture_atlas_layout,
                    index: key as usize,
                },
                transform: Transform::from_scale(Vec3::splat(PIECE_WIDTH / PIECE_WIDTH_IMG))
                    .with_translation(Vec3::new(x, y, 1.)),
                ..default()
            },
            on_drag_start_listener: On::<Pointer<DragStart>>::run(draw_possible_moves),
            on_drag_listener: On::<Pointer<Drag>>::run(on_piece_drag),
            on_drag_end_listener: On::<Pointer<DragEnd>>::run(on_piece_drag_end),
        }
    }
}

// Move the piece when it is dragged by a mouse
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

// Finalise the movement of a piece, either snapping it to the grid, or by moving it back
fn on_piece_drag_end(
    mut commands: Commands,
    mut drag_er: EventReader<Pointer<DragEnd>>,
    mut transform_query: Query<&mut Transform>,
    possible_move_meshes: Query<Entity, With<Mesh2dHandle>>,
    mut ev_piece_move: EventWriter<PieceMoveEvent>,
) {
    for drag_data in drag_er.read() {
        let transform = transform_query.get_mut(drag_data.target).unwrap();

        // Find where the piece was moved from in board coordinates
        let original_pos = transform.translation.xy()
            - Vec2::new(drag_data.distance.x, -drag_data.distance.y)
            + Vec2::new(PIECE_WIDTH, PIECE_HEIGHT) / 2.;
        let (ori_i, ori_j) = pixel_to_board_coords(original_pos.x, original_pos.y);

        // Find the new position, snapped to board coords, and move the sprite there
        let (i, j) = pixel_to_board_coords(
            transform.translation.x + PIECE_WIDTH / 2.,
            transform.translation.y + PIECE_HEIGHT / 2.,
        );

        ev_piece_move.send(PieceMoveEvent {
            to_from: PieceMove {
                from: (ori_i, ori_j),
                to: (i, j),
            },
            entity: drag_data.target,
        });

        // Clean up the possible move markers
        for mesh in possible_move_meshes.iter() {
            commands.entity(mesh).despawn();
        }
    }
}

pub fn draw_moves(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    moves: Vec<(usize, usize)>,
) {
    for pos in moves.iter() {
        let (x, y) = board_to_pixel_coords(pos.0, pos.1);

        commands.spawn(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Circle {
                radius: PIECE_HEIGHT * 0.8 / 2., // Circle with 0.8x the width of a tile
            })),
            material: materials.add(Color::hsla(285., 0.60, 0.5, 0.85)),
            transform: Transform::from_xyz(x, y, 2.0), // z = 2.0 puts it above all pieces except the one being held
            ..default()
        });
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
        let (i, j) = pixel_to_board_coords(transform.translation.x, transform.translation.y);

        let possible_moves = board.get_possible_moves((i, j));

        // Draw markers on each of the possible move tiles
        draw_moves(&mut commands, &mut meshes, &mut materials, possible_moves);
    }
}

#[derive(Event)]
pub struct PieceMoveEvent {
    pub to_from: PieceMove,
    pub entity: Entity,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PieceMove {
    pub from: (usize, usize),
    pub to: (usize, usize),
}

#[derive(Clone, Copy, Debug)]
pub struct PieceMoveHistory {
    pub from_to: PieceMove,
    pub captured: Option<(PieceEnum, bool)>,
}
