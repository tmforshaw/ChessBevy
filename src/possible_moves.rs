use bevy::prelude::*;

use crate::{
    board::{Board, TilePos},
    display::{board_to_pixel_coords, PIECE_SIZE},
    piece::Piece,
    piece_move::PieceMove,
};

const POSSIBLE_MOVE_COLOUR: Color = Color::rgba(0., 1., 0., 0.75);

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
    board: ResMut<Board>,
) {
    for ev in ev_display.read() {
        if ev.show {
            if let Some(possible_moves) = get_possible_moves(&board, ev.from) {
                for pos in possible_moves {
                    let (x, y) = board_to_pixel_coords(pos.file, pos.rank);

                    commands.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                color: POSSIBLE_MOVE_COLOUR,
                                ..default()
                            },
                            transform: Transform::from_xyz(x, y, 2.)
                                .with_scale(Vec3::splat(PIECE_SIZE * 0.75)),
                            ..default()
                        },
                        PossibleMoveMarker,
                    ));
                }
            }
        } else {
            // Stop displaying all entities
            for entity in possible_move_entities.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

#[must_use]
pub fn get_pseudolegal_moves(board: &Board, from: TilePos) -> Option<Vec<TilePos>> {
    let piece = board.get_piece(from);

    (match piece {
        Piece::BQueen | Piece::WQueen => Board::get_ortho_diagonal_moves,
        Piece::BKing | Piece::WKing => Board::get_king_moves,
        Piece::BRook | Piece::WRook => Board::get_orthogonal_moves,
        Piece::BKnight | Piece::WKnight => Board::get_knight_moves,
        Piece::BBishop | Piece::WBishop => Board::get_diagonal_moves,
        Piece::BPawn | Piece::WPawn => Board::get_pawn_moves,
        Piece::None => {
            const fn no_moves(_: &Board, _: TilePos) -> Option<Vec<TilePos>> {
                None
            }

            no_moves
        }
    })(board, from)
}

#[must_use]
pub fn get_possible_moves(board: &Board, from: TilePos) -> Option<Vec<TilePos>> {
    let player = board.get_piece(from).to_player()?;

    // Don't allow moves which cause the king to be attacked
    Some(get_pseudolegal_moves(board, from)?
        .into_iter()
        .filter(|&move_to_pos| {
            // Ensure that move won't cause the king to be attacked
            !board.move_makes_pos_attacked(PieceMove::new(from, move_to_pos), board.get_king_pos(player))
            // Ensure that the piece cant capture its own piece
           && board.get_piece(move_to_pos).to_player()
                != Some(player)
        })
        .collect::<Vec<_>>())
}
