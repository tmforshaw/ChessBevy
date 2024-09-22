use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use piece::{piece_move_event_reader, PieceMoveEvent};

pub mod bitboard;
pub mod board;
pub mod display;
pub mod piece;

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
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .build(),
            DefaultPickingPlugins,
        ))
        // .insert_resource(bevy_mod_picking::debug::DebugPickingMode::Normal)
        .init_resource::<Board>()
        .add_systems(Startup, (setup, display_board))
        .add_systems(Update, piece_move_event_reader)
        .add_event::<PieceMoveEvent>()
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
