use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};

use crate::{
    board::{board_to_pixel_coords, move_piece, move_piece_without_tests, Board},
    piece::{PieceEnum, PieceMove, PieceMoveEvent},
};

#[derive(Component, Copy, Clone, Debug)]
pub enum PlayerEnum {
    White,
    Black,
}

#[derive(Component, Copy, Clone)]
pub struct Player {
    pub kind: PlayerEnum,
    pub in_check: bool,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            kind: PlayerEnum::White,
            in_check: false,
        }
    }
}

impl From<usize> for PlayerEnum {
    fn from(num: usize) -> Self {
        match num {
            1 => Self::Black,
            _ => Self::White,
        }
    }
}

#[derive(Event, Copy, Clone)]
pub struct CheckEvent {
    pub player_in_check: PlayerEnum,
    pub checking_piece: (usize, usize),
    pub at: (usize, usize),
}

pub fn check_opponent_for_checks(board: &mut ResMut<Board>) -> Vec<CheckEvent> {
    let player_being_tested = board.see_next_player();

    let all_moves = board.get_all_possible_moves(board.current_player);

    all_moves
        .iter()
        .filter_map(|&piece_move| {
            if (board.tiles[piece_move.to.0][piece_move.to.1] as usize == PieceEnum::BKing as usize
                && player_being_tested as usize == PlayerEnum::Black as usize)
                || (board.tiles[piece_move.to.0][piece_move.to.1] as usize
                    == PieceEnum::WKing as usize
                    && player_being_tested as usize == PlayerEnum::White as usize)
            {
                Some(CheckEvent {
                    player_in_check: player_being_tested,
                    checking_piece: piece_move.from,
                    at: piece_move.to,
                })
            } else {
                None
            }
        })
        .collect()
}

pub fn check_event_read(
    mut ev_check: EventReader<CheckEvent>,
    mut ev_checkmate: EventWriter<CheckmateEvent>,
    mut board: ResMut<Board>,
) {
    for &check_event in ev_check.read() {
        println!("{:?} is in check", check_event.player_in_check);
        println!("{}", board.tiles_string());

        board.player_in_check = Some(check_event.player_in_check);

        let blocking_moves = board.get_check_stopping_moves(check_event);

        // TODO let kings walk away from check if possible

        let blocking_moves_to_add =
            if !board.blocking_moves[check_event.player_in_check as usize].is_empty() {
                // There are already check blocking moves for this player, so find the common moves and add those
                board.blocking_moves[check_event.player_in_check as usize]
                    .iter()
                    .filter(|&to_from| blocking_moves.contains(to_from))
                    .cloned()
                    .collect()
            } else {
                blocking_moves
            };

        // There are no moves to block the check
        if blocking_moves_to_add.is_empty() {
            // It is checkmate
            ev_checkmate.send(CheckmateEvent {
                winning_player: board.get_next_player(check_event.player_in_check),
            });
        }

        board.blocking_moves[check_event.player_in_check as usize] = blocking_moves_to_add;
    }
}

#[derive(Event)]
pub struct CheckmateEvent {
    pub winning_player: PlayerEnum,
}

pub fn checkmate_event_read(mut ev_checkmate: EventReader<CheckmateEvent>) {
    for event in ev_checkmate.read() {
        println!(
            "Game has ended: The winning player is {:?}",
            event.winning_player
        );

        // TODO Put a timer here
        // TODO reset board and game
    }
}

pub fn keyboard_events(
    mut commands: Commands,
    mut key_ev: EventReader<KeyboardInput>,
    mut board: ResMut<Board>,
    mut transform_query: Query<&mut Transform>,
) {
    use bevy::input::ButtonState;

    for ev in key_ev.read() {
        match ev.state {
            ButtonState::Pressed => match ev.logical_key {
                Key::ArrowLeft => {
                    let board_clone = board.clone();

                    let last_move = match board_clone.move_history.last() {
                        Some(piece_move) => piece_move,
                        None => continue,
                    };

                    let last_move_entity =
                        board.pieces_and_positions[last_move.to.0][last_move.to.1].unwrap();

                    let mut transform = transform_query.get_mut(last_move_entity).unwrap();

                    let (x, y) = board_to_pixel_coords(last_move.from.0, last_move.from.1);

                    transform.translation = Vec3::new(x, y, 1.);

                    board.move_history.pop();

                    move_piece_without_tests(
                        &mut commands,
                        &mut board,
                        &mut transform,
                        last_move.to,
                        last_move.from,
                        last_move_entity,
                    );

                    board.next_player();

                    println!("Moved {last_move:?}");
                }
                Key::ArrowRight => {}
                _ => {}
            },
            ButtonState::Released => {}
        }
    }
}
