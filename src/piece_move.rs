use bevy::prelude::*;

use chess_core::piece_move::PieceMove;

use crate::{
    board::BoardBevy,
    display::{translate_piece_entity, BackgroundColourEvent},
    game_end::GameEndEvent,
    last_move::LastMoveEvent,
    uci::{transmit_to_uci, UciMessage, ENGINE_PLAYER},
};

#[derive(Event)]
pub struct PieceMoveEvent {
    pub piece_move: PieceMove,
    pub entity: Entity,
}

/// # Panics
/// Panics if the move history can't be converted to a string to send to via uci to the engine
/// Panics if message cannot be sent via uci
#[allow(clippy::too_many_arguments)]
pub fn piece_move_event_handler(
    mut commands: Commands,
    mut ev_piece_move: EventReader<PieceMoveEvent>,
    mut transform_query: Query<&mut Transform>,
    mut texture_atlas_query: Query<&mut TextureAtlas>,
    mut board: ResMut<BoardBevy>,
    mut background_ev: EventWriter<BackgroundColourEvent>,
    mut game_end_ev: EventWriter<GameEndEvent>,
    mut last_move_ev: EventWriter<LastMoveEvent>,
) {
    for ev in ev_piece_move.read() {
        let piece_move = ev.piece_move;

        // Snap the moved entity to the grid (Don't move if there is a non-opponent piece there, or if you moved a piece on another player's turn, or if the move is impossible for that piece type)

        // TODO ENGINE_PLAYER can't be white
        if !board.board.get_piece(piece_move.to).is_player(board.board.player)
            && board.board.get_piece(piece_move.from).is_player(board.board.player)
            && board.board.get_possible_moves(piece_move.from).contains(&piece_move)
            && board.board.get_player() != ENGINE_PLAYER
        {
            // Apply the move to the board
            if board
                .apply_move(
                    &mut commands,
                    &mut transform_query,
                    &mut texture_atlas_query,
                    &mut background_ev,
                    &mut game_end_ev,
                    &mut last_move_ev,
                    piece_move,
                )
                .is_some()
            {
                // Send the moves to the chess engine, if the game hasn't ended
                transmit_to_uci(UciMessage::NewMove {
                    move_history: board
                        .board
                        .move_history
                        .to_piece_move_string()
                        .expect("Could not convert move history into piece move string"),
                    player_to_move: board.board.get_player(),
                })
                .unwrap_or_else(|e| panic!("{e}"));
            }
        } else {
            // Reset position
            translate_piece_entity(&mut transform_query, ev.entity, piece_move.from);
        }
    }
}
