use bevy::prelude::*;

use chess_core::{piece_move::PieceMove, possible_moves::get_possible_moves};

use crate::{
    board::BoardBevy,
    display::{translate_piece_entity, BackgroundColourEvent},
    game_end::GameEndEvent,
};

#[derive(Event)]
pub struct PieceMoveEvent {
    pub piece_move: PieceMove,
    pub entity: Entity,
}

pub fn piece_move_event_handler(
    mut commands: Commands,
    mut ev_piece_move: EventReader<PieceMoveEvent>,
    mut transform_query: Query<&mut Transform>,
    mut texture_atlas_query: Query<&mut TextureAtlas>,
    mut board: ResMut<BoardBevy>,
    mut background_ev: EventWriter<BackgroundColourEvent>,
    mut game_end_ev: EventWriter<GameEndEvent>,
) {
    for ev in ev_piece_move.read() {
        let mut piece_move = ev.piece_move;

        // Snap the moved entity to the grid (Don't move if there is a non-opponent piece there, or if you moved a piece on another player's turn, or if the move is impossible for that piece type)
        if let Some(possible_moves) = get_possible_moves(&board.board, piece_move.from) {
            if !board
                .board
                .get_piece(piece_move.to)
                .is_player(board.board.player)
                && board
                    .board
                    .get_piece(piece_move.from)
                    .is_player(board.board.player)
                && possible_moves.contains(&piece_move.to)
            {
                // Apply the move to the board
                let (en_passant_tile, castling_rights_before_move, captured_piece);
                (
                    piece_move,
                    en_passant_tile,
                    castling_rights_before_move,
                    captured_piece,
                ) = board.apply_move(
                    &mut commands,
                    &mut transform_query,
                    &mut texture_atlas_query,
                    &mut background_ev,
                    &mut game_end_ev,
                    piece_move,
                );

                // Update the move history with this move
                board.board.move_history.make_move(
                    piece_move,
                    captured_piece,
                    en_passant_tile,
                    castling_rights_before_move,
                );
            } else {
                // Reset position
                translate_piece_entity(&mut transform_query, ev.entity, piece_move.from);
            }
        }
    }
}
