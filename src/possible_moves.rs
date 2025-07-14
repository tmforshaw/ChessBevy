use bevy::prelude::*;

use crate::{
    board::{Board, Player, TilePos},
    display::{board_to_pixel_coords, PIECE_SIZE},
    piece::Piece,
    piece_move::PieceMove,
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
    board: ResMut<Board>,
) {
    for ev in ev_display.read() {
        if ev.show {
            let positions = get_possible_moves(&board, ev.from);
            println!("Possible: {positions:?}");
            // let positions = get_pseudolegal_moves(&board, ev.from);
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

#[must_use]
pub fn get_pseudolegal_moves(board: &Board, from: TilePos) -> Vec<TilePos> {
    let piece = board.get_piece(from);

    (match piece {
        Piece::BQueen | Piece::WQueen => Board::get_ortho_diagonal_moves,
        Piece::BKing | Piece::WKing => Board::get_king_moves,
        Piece::BRook | Piece::WRook => Board::get_orthogonal_moves,
        Piece::BKnight | Piece::WKnight => Board::get_knight_moves,
        Piece::BBishop | Piece::WBishop => Board::get_diagonal_moves,
        Piece::BPawn | Piece::WPawn => Board::get_pawn_moves,
        Piece::None => {
            const fn no_moves(_: &Board, _: TilePos) -> Vec<TilePos> {
                Vec::new()
            }

            no_moves
        }
    })(board, from)
}

#[must_use]
pub fn get_possible_moves(board: &Board, from: TilePos) -> Vec<TilePos> {
    let current_player_king = match board.get_player() {
        Player::White => Piece::WKing,
        Player::Black => Piece::BKing,
    };

    let king_pos = board.positions[current_player_king].get_positions()[0]; // Should always have a king

    println!("From: {from:?}\tKing_Pos: {king_pos:?}\tKing: {current_player_king:?}");

    // Don't allow moves which cause the king to be attacked
    get_pseudolegal_moves(board, from)
        .into_iter()
        .filter(|&move_to_pos| {
            // Ensure that move won't cause the king to be attacked
            !board.move_makes_pos_attacked(PieceMove::new(from, move_to_pos), king_pos)
        })
        .collect::<Vec<_>>()
}
