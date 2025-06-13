use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use bitboard::{bitboard_event_handler, BitBoardDisplayEvent};
use display::{background_colour_event_handler, BackgroundColourEvent};
use keyboard::{keyboard_event_handler, KeyboardState};
use piece_move::{
    move_history_event_handler, piece_move_event_reader, MoveHistoryEvent, PieceMoveEvent,
};
use possible_moves::{possible_move_event_handler, PossibleMoveDisplayEvent};

pub mod bitboard;
pub mod board;
pub mod display;
pub mod keyboard;
pub mod piece;
pub mod piece_move;
pub mod possible_moves;

use crate::{board::Board, display::display_board};

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
                piece_move_event_reader,
                bitboard_event_handler,
                possible_move_event_handler,
                keyboard_event_handler,
                background_colour_event_handler,
                move_history_event_handler,
            ),
        )
        .add_event::<PieceMoveEvent>()
        .add_event::<BitBoardDisplayEvent>()
        .add_event::<PossibleMoveDisplayEvent>()
        .add_event::<BackgroundColourEvent>()
        .add_event::<MoveHistoryEvent>()
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
