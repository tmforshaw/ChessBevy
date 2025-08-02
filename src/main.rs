#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]

use bevy::prelude::*;

use crate::{
    bitboard_event::{bitboard_event_handler, BitBoardDisplayEvent},
    board::BoardBevy,
    display::{background_colour_event_handler, display_board, BackgroundColourEvent},
    eval_bar::CurrentEval,
    game_end::{game_end_event_handler, GameEndEvent},
    keyboard::{keyboard_event_handler, KeyboardState},
    last_move::{last_move_event_handler, LastMoveEvent},
    move_history::{move_history_event_handler, MoveHistoryEvent},
    piece::{on_piece_drag, on_piece_drag_end, on_piece_drag_start},
    piece_move::{piece_move_event_handler, PieceMoveEvent},
    possible_moves::{possible_move_event_handler, PossibleMoveDisplayEvent},
    uci::communicate_to_uci,
    uci_event::{process_uci_to_board_threads, uci_to_board_event_handler, UciEvent},
};

pub mod bitboard_event;
pub mod board;
pub mod classification;
pub mod display;
pub mod eval_bar;
pub mod game_end;
pub mod keyboard;
pub mod last_move;
pub mod move_history;
pub mod piece;
pub mod piece_move;
pub mod possible_moves;
pub mod uci;
pub mod uci_event;
pub mod uci_info;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Chez.cum".into(),
                        resolution: (1920., 1280.).into(),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .build(),
            // DefaultPickingPlugins,
        ))
        .add_event::<PieceMoveEvent>()
        .add_event::<BitBoardDisplayEvent>()
        .add_event::<PossibleMoveDisplayEvent>()
        .add_event::<BackgroundColourEvent>()
        .add_event::<MoveHistoryEvent>()
        .add_event::<GameEndEvent>()
        .add_event::<UciEvent>()
        .add_event::<LastMoveEvent>()
        .init_resource::<BoardBevy>()
        .init_resource::<KeyboardState>()
        .init_resource::<CurrentEval>()
        .insert_resource(communicate_to_uci())
        .add_systems(Startup, (setup, display_board))
        .add_systems(PreUpdate, process_uci_to_board_threads)
        .add_systems(
            Update,
            (
                piece_move_event_handler,
                bitboard_event_handler,
                possible_move_event_handler,
                keyboard_event_handler,
                background_colour_event_handler,
                move_history_event_handler,
                game_end_event_handler,
                uci_to_board_event_handler,
                last_move_event_handler,
                // update_eval_bar,
                on_piece_drag_start,
                on_piece_drag,
                on_piece_drag_end,
            ),
        )
        .run();
}

#[allow(clippy::needless_pass_by_value)]
fn setup(mut commands: Commands, board: Res<BoardBevy>, mut background_ev: EventWriter<BackgroundColourEvent>) {
    commands.spawn((Camera2d, Camera::default(), Transform::default(), GlobalTransform::default()));

    background_ev.write(BackgroundColourEvent::new_from_player(board.board.get_player()));
}
