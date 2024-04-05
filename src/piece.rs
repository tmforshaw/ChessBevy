use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::board::{board_to_pixel_coords, pixel_to_board_coords, Board};

pub const PIECE_AMT: usize = 6;
pub const COLOUR_AMT: usize = 2;
pub const PIECE_WIDTH: f32 = 60.;
pub const PIECE_HEIGHT: f32 = 60.;

pub const PIECE_SCALE: f32 = 2.;

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
    on_drag_listener: On<Pointer<Drag>>,
    on_drag_end_listener: On<Pointer<DragEnd>>,
    on_drop_listener: On<Pointer<Drop>>,
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
            on_drag_end_listener: On::<Pointer<DragEnd>>::run(on_piece_drag_end),
            on_drop_listener: On::<Pointer<Drop>>::run(on_piece_drop),
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
) {
    for drag_data in drag_er.read() {
        let mut transform = transform_query.get_mut(drag_data.target).unwrap();

        // Find where the piece was moved from in board coordinates
        let original_pos =
            transform.translation.xy() - Vec2::new(drag_data.distance.x, -drag_data.distance.y);
        let (ori_i, ori_j) = pixel_to_board_coords(original_pos.x, original_pos.y);

        // Find the new position, snapped to board coords, and move the sprite there
        let (i, j) = pixel_to_board_coords(
            transform.translation.x + PIECE_WIDTH * PIECE_SCALE / 2.,
            transform.translation.y + PIECE_HEIGHT * PIECE_SCALE / 2.,
        );
        let (x, y) = board_to_pixel_coords(i, j);

        transform.translation = Vec3::new(x, y, 1.);

        // Update board to reflect these changes
        // let moved_piece = board.tiles[ori_i][ori_j];

        // if board.tiles[i][j] as usize == PieceEnum::Empty as usize {
        //     board.tiles[ori_i][ori_j] = PieceEnum::Empty;
        //     board.tiles[i][j] = moved_piece;
        // } else if (piece_is_white(board.tiles[i][j]) && piece_is_black(moved_piece))
        //     || (piece_is_black(board.tiles[i][j]) && piece_is_white(moved_piece))
        // {
        //     // If the square being moved to and the piece are different colours
        //     commands
        //         .entity(board.pieces_and_positions[i][j].unwrap())
        //         .despawn();

        //     println!("Moved: {moved_piece:?}, Onto: {:?}", board.tiles[i][j]);

        //     board.tiles[ori_i][ori_j] = PieceEnum::Empty;
        //     board.tiles[i][j] = moved_piece;
        // } else {
        //     transform.translation = Vec3::new(original_pos.x, original_pos.y, 1.);
        // }

        board.move_piece((ori_i, ori_j), (i, j), drag_data.target(), &mut commands)
    }
}

fn on_piece_drop(drop_er: EventReader<Pointer<Drop>>) {
    println!("SOMETHING WAS DROPPED: {drop_er:?}");
}

pub fn piece_is_white(piece: PieceEnum) -> bool {
    (PIECE_AMT..PIECE_AMT * COLOUR_AMT).contains(&(piece as usize))
}

pub fn piece_is_black(piece: PieceEnum) -> bool {
    (0..PIECE_AMT).contains(&(piece as usize))
}
