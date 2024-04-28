use bevy::prelude::*;

use bevy_mod_picking::prelude::*;
use board::{move_piece, Board};
use game::{check_event_read, checkmate_event_read, keyboard_events, CheckEvent, CheckmateEvent};
use piece::PieceMoveEvent;

pub mod board;
pub mod game;
pub mod piece;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Chez.cum".into(),
                        resolution: (1920., 1080.).into(),
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
        .add_systems(Startup, (setup, Board::setup))
        .add_systems(
            FixedUpdate,
            (
                move_piece,
                checkmate_event_read,
                check_event_read,
                keyboard_events,
            ),
        )
        .add_event::<PieceMoveEvent>()
        .add_event::<CheckmateEvent>()
        .add_event::<CheckEvent>()
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
