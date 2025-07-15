// #![warn(clippy::all)]
// #![warn(clippy::pedantic)]
// #![warn(clippy::nursery)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::option_if_let_else)]

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::{
    bitboard::{bitboard_event_handler, BitBoardDisplayEvent},
    board::Board,
    checkmate::{checkmate_event_handler, CheckmateEvent},
    display::{background_colour_event_handler, display_board, BackgroundColourEvent},
    keyboard::{keyboard_event_handler, KeyboardState},
    move_history::{move_history_event_handler, MoveHistoryEvent},
    piece_move::{piece_move_event_handler, PieceMoveEvent},
    possible_moves::{possible_move_event_handler, PossibleMoveDisplayEvent},
};

pub mod bitboard;
pub mod board;
pub mod checkmate;
pub mod display;
pub mod keyboard;
pub mod move_history;
pub mod piece;
pub mod piece_move;
pub mod possible_moves;

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
            DefaultPickingPlugins,
        ))
        // .insert_resource(bevy_mod_picking::debug::DebugPickingMode::Normal)
        .init_resource::<Board>()
        .init_resource::<KeyboardState>()
        .insert_resource(ClearColor(Color::rgb(1., 1., 1.)))
        .add_systems(Startup, (setup, display_board))
        .add_systems(
            Update,
            (
                piece_move_event_handler,
                bitboard_event_handler,
                possible_move_event_handler,
                keyboard_event_handler,
                background_colour_event_handler,
                move_history_event_handler,
                checkmate_event_handler,
            ),
        )
        .add_event::<PieceMoveEvent>()
        .add_event::<BitBoardDisplayEvent>()
        .add_event::<PossibleMoveDisplayEvent>()
        .add_event::<BackgroundColourEvent>()
        .add_event::<MoveHistoryEvent>()
        .add_event::<CheckmateEvent>()
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
