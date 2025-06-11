use bevy::{input::keyboard::KeyboardInput, prelude::*};

use crate::{bitboard::BitBoardDisplayEvent, piece::Piece};

pub fn keyboard_event_handler(
    mut ev_keyboard: EventReader<KeyboardInput>,
    mut ev_display_event: EventWriter<BitBoardDisplayEvent>,
) {
    for ev in ev_keyboard.read() {
        if ev.state.is_pressed() {
            if let Some(display_event) = match ev.key_code {
                KeyCode::Digit0 => Some(BitBoardDisplayEvent {
                    board_type: None,
                    show: false,
                }),
                KeyCode::Digit1 => Some(BitBoardDisplayEvent {
                    board_type: Some(Piece::WPawn),
                    show: true,
                }),
                KeyCode::Digit2 => Some(BitBoardDisplayEvent {
                    board_type: Some(Piece::BPawn),
                    show: true,
                }),
                KeyCode::Digit3 => Some(BitBoardDisplayEvent {
                    board_type: Some(Piece::WBishop),
                    show: true,
                }),
                KeyCode::Digit4 => Some(BitBoardDisplayEvent {
                    board_type: Some(Piece::BBishop),
                    show: true,
                }),
                KeyCode::Digit5 => Some(BitBoardDisplayEvent {
                    board_type: Some(Piece::WKnight),
                    show: true,
                }),
                KeyCode::Digit6 => Some(BitBoardDisplayEvent {
                    board_type: Some(Piece::BKnight),
                    show: true,
                }),
                KeyCode::Digit7 => Some(BitBoardDisplayEvent {
                    board_type: Some(Piece::WRook),
                    show: true,
                }),
                KeyCode::Digit8 => Some(BitBoardDisplayEvent {
                    board_type: Some(Piece::BRook),
                    show: true,
                }),
                KeyCode::Digit9 => Some(BitBoardDisplayEvent {
                    board_type: Some(Piece::WQueen),
                    show: true,
                }),
                _ => None,
            } {
                ev_display_event.send(display_event);
            }
        }
    }
}
