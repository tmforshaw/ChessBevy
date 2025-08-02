use bevy::prelude::*;

use crate::{
    board::BoardBevy,
    display::{board_to_pixel_coords, PIECE_SIZE},
};

#[derive(Event)]
pub struct LastMoveEvent;

#[derive(Component)]
pub struct LastMoveMarker;

#[allow(clippy::needless_pass_by_value)]
pub fn last_move_event_handler(
    mut ev_last_move: EventReader<LastMoveEvent>,
    board: Res<BoardBevy>,
    last_move_entities: Query<Entity, With<LastMoveMarker>>,
    mut commands: Commands,
) {
    for _ in ev_last_move.read() {
        // Clear any last_move entities
        for entity in last_move_entities.iter() {
            commands.entity(entity).despawn();
        }

        let Some(last_move) = board.board.move_history.get() else {
            continue;
        };

        let (last_move, _, _, _) = last_move.into();

        let xy = [
            board_to_pixel_coords(last_move.from.file, last_move.from.rank),
            board_to_pixel_coords(last_move.to.file, last_move.to.rank),
        ];

        // Spawn an entity for the from and to positions of this piece move
        for (x, y) in xy {
            commands.spawn((
                Sprite {
                    color: Color::linear_rgba(0.8, 0.8, 0.1, 1.0),
                    ..default()
                },
                Transform::from_xyz(x, y, 0.5).with_scale(Vec3::splat(PIECE_SIZE * 1.0)),
                LastMoveMarker,
            ));
        }
    }
}
