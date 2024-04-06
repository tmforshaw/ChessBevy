use bevy::prelude::*;

use bevy_mod_picking::prelude::*;
use board::Board;

pub mod board;
pub mod piece;
pub mod player;

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
        // .add_systems(FixedUpdate, event_readers)
        // .add_event::<GameOver>()
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

// fn event_readers(mut ev_piece_moved: EventReader<PieceMove>, mut board: ResMut<Board>) {
//     for piece_move in ev_piece_moved {
//         board.move_piece(piece_move.from,piece_move.to ,piece_move.entity , , )
//     }
// }
