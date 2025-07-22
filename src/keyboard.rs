use bevy::{input::keyboard::KeyboardInput, prelude::*};

use chess_core::piece::Piece;

use crate::{bitboard_event::BitBoardDisplayEvent, move_history::MoveHistoryEvent};

#[derive(Resource, Clone, Default)]
pub struct KeyboardState {
    pub shift_pressed: bool,
}

#[allow(clippy::too_many_lines)]
pub fn keyboard_event_handler(
    mut keyboard_state: ResMut<KeyboardState>,
    mut ev_keyboard: EventReader<KeyboardInput>,
    mut ev_display_event: EventWriter<BitBoardDisplayEvent>,
    mut ev_move_history: EventWriter<MoveHistoryEvent>,
) {
    for ev in ev_keyboard.read() {
        if ev.state.is_pressed() {
            if ev.key_code == KeyCode::ShiftLeft || ev.key_code == KeyCode::ShiftRight {
                keyboard_state.shift_pressed = true;
            }
            if ev.key_code == KeyCode::Digit0 {
                ev_display_event.send(BitBoardDisplayEvent::new(None, true, false, 0));
            }
            if ev.key_code == KeyCode::Digit1 {
                ev_display_event.send(BitBoardDisplayEvent::new(
                    Some(Piece::WPawn),
                    keyboard_state.shift_pressed,
                    true,
                    0,
                ));
            }
            if ev.key_code == KeyCode::Digit2 {
                ev_display_event.send(BitBoardDisplayEvent::new(
                    Some(Piece::BPawn),
                    keyboard_state.shift_pressed,
                    true,
                    0,
                ));
            }
            if ev.key_code == KeyCode::Digit3 {
                ev_display_event.send(BitBoardDisplayEvent::new(
                    Some(Piece::WBishop),
                    keyboard_state.shift_pressed,
                    true,
                    0,
                ));
            }
            if ev.key_code == KeyCode::Digit4 {
                ev_display_event.send(BitBoardDisplayEvent::new(
                    Some(Piece::BBishop),
                    keyboard_state.shift_pressed,
                    true,
                    0,
                ));
            }
            if ev.key_code == KeyCode::Digit5 {
                ev_display_event.send(BitBoardDisplayEvent::new(
                    Some(Piece::WKnight),
                    keyboard_state.shift_pressed,
                    true,
                    0,
                ));
            }
            if ev.key_code == KeyCode::Digit6 {
                ev_display_event.send(BitBoardDisplayEvent::new(
                    Some(Piece::BKnight),
                    keyboard_state.shift_pressed,
                    true,
                    0,
                ));
            }
            if ev.key_code == KeyCode::Digit7 {
                ev_display_event.send(BitBoardDisplayEvent::new(
                    Some(Piece::WRook),
                    keyboard_state.shift_pressed,
                    true,
                    0,
                ));
            }
            if ev.key_code == KeyCode::Digit8 {
                ev_display_event.send(BitBoardDisplayEvent::new(
                    Some(Piece::BRook),
                    keyboard_state.shift_pressed,
                    true,
                    0,
                ));
            }
            if ev.key_code == KeyCode::Digit9 {
                ev_display_event.send(BitBoardDisplayEvent::new(
                    Some(Piece::WQueen),
                    keyboard_state.shift_pressed,
                    true,
                    0,
                ));
            }
            if ev.key_code == KeyCode::KeyQ {
                ev_display_event.send(BitBoardDisplayEvent::new(None, keyboard_state.shift_pressed, false, 1));
            }

            if ev.key_code == KeyCode::KeyE {
                ev_display_event.send(BitBoardDisplayEvent::new(None, keyboard_state.shift_pressed, false, 2));
            }

            if ev.key_code == KeyCode::KeyR {
                ev_display_event.send(BitBoardDisplayEvent::new(None, keyboard_state.shift_pressed, false, 3));
            }

            if ev.key_code == KeyCode::ArrowLeft {
                ev_move_history.send(MoveHistoryEvent { backwards: true });
            }
            if ev.key_code == KeyCode::ArrowRight {
                ev_move_history.send(MoveHistoryEvent { backwards: false });
            }
        } else {
            match ev.key_code {
                KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                    keyboard_state.shift_pressed = false;
                }
                _ => {}
            }
        }
    }
}
