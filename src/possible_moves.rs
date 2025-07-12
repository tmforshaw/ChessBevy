use bevy::prelude::*;

use crate::{
    board::{Board, TilePos},
    display::{board_to_pixel_coords, PIECE_SIZE},
    piece::Piece,
};

#[derive(Event, Debug)]
pub struct PossibleMoveDisplayEvent {
    pub from: TilePos,
    pub show: bool,
}

#[derive(Component)]
pub struct PossibleMoveMarker;

#[allow(clippy::needless_pass_by_value)]
pub fn possible_move_event_handler(
    mut ev_display: EventReader<PossibleMoveDisplayEvent>,
    possible_move_entities: Query<Entity, With<PossibleMoveMarker>>,
    mut commands: Commands,
    mut board: ResMut<Board>,
) {
    for ev in ev_display.read() {
        if ev.show {
            let positions = get_possible_moves(&mut board, ev.from);
            for pos in positions {
                let (x, y) = board_to_pixel_coords(pos.file, pos.rank);

                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgba(0., 1., 0., 0.75),
                            ..default()
                        },
                        transform: Transform::from_xyz(x, y, 2.)
                            .with_scale(Vec3::splat(PIECE_SIZE * 0.75)),
                        ..default()
                    },
                    PossibleMoveMarker,
                ));
            }
        } else {
            for entity in possible_move_entities.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
#[must_use]
pub fn get_possible_moves(board: &mut Board, from: TilePos) -> Vec<TilePos> {
    let piece = board.get_piece(from);

    (match piece {
        Piece::BQueen | Piece::WQueen => Board::get_ortho_diagonal_moves,
        Piece::BKing | Piece::WKing => Board::get_king_moves,
        Piece::BRook | Piece::WRook => Board::get_orthogonal_moves,
        Piece::BKnight | Piece::WKnight => Board::get_knight_moves,
        Piece::BBishop | Piece::WBishop => Board::get_diagonal_moves,
        Piece::BPawn | Piece::WPawn => Board::get_pawn_moves,
        Piece::None => {
            const fn no_moves(_: &mut Board, _: TilePos) -> Vec<TilePos> {
                Vec::new()
            }

            no_moves
        }
    })(board, from)
}
