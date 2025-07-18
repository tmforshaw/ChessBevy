use bevy::prelude::*;

use chess_core::piece_move::PieceMove;

use crate::{board::BoardBevy, display::BackgroundColourEvent, game_end::GameEndEvent};

#[derive(Debug, Resource, Clone)]
pub enum UciToBoardMessage {
    BestMove(PieceMove),
}

#[derive(Event, Resource, Debug, Clone)]
pub struct UciEvent {
    message: UciToBoardMessage,
}
impl UciEvent {
    #[allow(dead_code)]
    #[must_use]
    pub const fn new(message: UciToBoardMessage) -> Self {
        Self { message }
    }
}

#[derive(Resource)]
pub struct UciToBoardReceiver(pub crossbeam_channel::Receiver<UciToBoardMessage>);

pub fn uci_to_board_event_handler(
    mut ev_uci_to_board: EventReader<UciEvent>,
    mut commands: Commands,
    mut board: ResMut<BoardBevy>,
    mut background_ev: EventWriter<BackgroundColourEvent>,
    mut transform_query: Query<&mut Transform>,
    mut texture_atlas_query: Query<&mut TextureAtlas>,
    mut game_end_ev: EventWriter<GameEndEvent>,
) {
    // Listen for messages from the Engine Listener thread, then apply moves
    for ev in ev_uci_to_board.read() {
        match ev.message {
            UciToBoardMessage::BestMove(mut piece_move) => {
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
            }
        }
    }
}

// Take the messages sent via crossbeam_channel and send them to Bevy as Events
#[allow(clippy::needless_pass_by_value)]
pub fn process_uci_to_board_threads(
    tx_rx: Res<UciToBoardReceiver>,
    mut uci_to_board_ev: EventWriter<UciEvent>,
) {
    for ev in tx_rx.0.try_iter() {
        uci_to_board_ev.send(UciEvent::new(ev));
    }
}
