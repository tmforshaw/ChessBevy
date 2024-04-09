use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};

use crate::{
    board::{move_piece_without_tests, Board, BOARD_HEIGHT},
    piece::{Piece, PieceEnum, COLOUR_AMT, PIECE_AMT, PIECE_HEIGHT_IMG, PIECE_WIDTH_IMG},
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

#[derive(Event, Copy, Clone, Debug)]
pub struct CheckEvent {
    pub player_in_check: PlayerEnum,
    pub checking_piece: (usize, usize),
    pub at: (usize, usize),
}

pub fn check_for_checks(board: &Board) -> Vec<CheckEvent> {
    let mut player = board.current_player;
    let mut checks = Vec::new();

    for _ in 0..COLOUR_AMT {
        let all_moves = board.get_all_possible_moves(player);

        let player_being_tested = board.get_next_player(player);

        checks.append(
            &mut all_moves
                .iter()
                .filter_map(|&piece_move| {
                    if (board.tiles[piece_move.to.0][piece_move.to.1] as usize
                        == PieceEnum::BKing as usize
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
                .collect::<Vec<_>>(),
        );

        player = player_being_tested;
    }

    checks
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

        // TODO Put a timer here, reset board and game
    }
}

pub fn keyboard_events(
    mut commands: Commands,
    mut key_ev: EventReader<KeyboardInput>,
    mut board: ResMut<Board>,
    mut transform_query: Query<&mut Transform>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    use bevy::input::ButtonState;

    for ev in key_ev.read() {
        match ev.state {
            ButtonState::Pressed => match ev.logical_key {
                Key::ArrowLeft => {
                    let board_clone = board.clone();

                    let last_move = match board_clone.move_history.last() {
                        Some(piece_move_history) => piece_move_history,
                        None => continue,
                    };

                    let last_move_entity = board.pieces_and_positions[last_move.from_to.to.0]
                        [last_move.from_to.to.1]
                        .unwrap();

                    // TODO THIS IS A CHEESE, NEED TO FIND TEXTURE ATLAS WITHOUT REPEATING CODE
                    let texture_atlas_layout =
                        texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                            Vec2::new(PIECE_WIDTH_IMG, PIECE_HEIGHT_IMG),
                            PIECE_AMT,
                            COLOUR_AMT,
                            None,
                            None,
                        ));

                    let texture = asset_server.get_handle(board.texture_file).unwrap();

                    move_piece_without_tests(
                        &mut commands,
                        &mut board,
                        &mut transform_query,
                        last_move.from_to.to,
                        last_move.from_to.from,
                        last_move_entity,
                    );

                    // Spawn in any captured pieces
                    if let Some((captured_piece, en_passant)) = last_move.captured {
                        let spawn_pos = if en_passant {
                            let i_dir = (last_move.from_to.to.0 as isize
                                - last_move.from_to.from.0 as isize)
                                .signum();

                            let below_i = ((last_move.from_to.to.0 as isize - i_dir) as usize)
                                .clamp(0, BOARD_HEIGHT);

                            (below_i, last_move.from_to.to.1)
                        } else {
                            last_move.from_to.to
                        };

                        let entity = commands.spawn(Piece::new(
                            spawn_pos,
                            captured_piece,
                            texture.clone(),
                            texture_atlas_layout,
                        ));

                        board.pieces_and_positions[spawn_pos.0][spawn_pos.1] = Some(entity.id());

                        board.tiles[spawn_pos.0][spawn_pos.1] = captured_piece;
                    }

                    board.move_history.pop();
                    board.next_player();
                }
                Key::ArrowRight => {}
                _ => {}
            },
            ButtonState::Released => {}
        }
    }
}
