use bevy::prelude::*;

use crate::{board::Board, piece::PieceEnum};

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

#[derive(Event)]
pub struct Checkmate {
    pub winning_player: PlayerEnum,
}

pub fn checkmate_event_read(mut ev_game_over: EventReader<Checkmate>) {
    for event in ev_game_over.read() {
        println!(
            "Game has ended: The winning player is {:?}",
            event.winning_player
        );
    }
}

#[derive(Event, Copy, Clone)]
pub struct Check {
    pub player_in_check: PlayerEnum,
    pub checking_piece: (usize, usize),
    pub in_check_on: (usize, usize),
}

pub fn check_opponent_for_checks(board: &mut ResMut<Board>) -> Vec<Check> {
    let player_being_tested = board.see_next_player();

    let all_moves = board.get_all_possible_moves(board.current_player);

    all_moves
        .iter()
        .filter_map(|&(attacking_piece_pos, (i, j))| {
            if (board.tiles[i][j] as usize == PieceEnum::BKing as usize
                && player_being_tested as usize == PlayerEnum::Black as usize)
                || (board.tiles[i][j] as usize == PieceEnum::WKing as usize
                    && player_being_tested as usize == PlayerEnum::White as usize)
            {
                Some(Check {
                    player_in_check: player_being_tested,
                    checking_piece: attacking_piece_pos,
                    in_check_on: (i, j),
                })
            } else {
                None
            }
        })
        .collect()
}

pub fn check_event_read(
    mut ev_game_over: EventReader<Check>,
    mut ev_checkmate: EventWriter<Checkmate>,
    mut board: ResMut<Board>,
) {
    for &check_event in ev_game_over.read() {
        println!("{:?} is in check", check_event.player_in_check);
        println!("{}", board.tiles_string());

        board.player_in_check = Some(check_event.player_in_check);

        let blocking_moves = board.get_check_stopping_moves(check_event);

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
            ev_checkmate.send(Checkmate {
                winning_player: board.get_next_player(check_event.player_in_check),
            });
        }

        board.blocking_moves[check_event.player_in_check as usize] = blocking_moves_to_add;
    }
}
