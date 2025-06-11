use bevy::prelude::*;

use crate::{
    board::TilePos,
    display::{board_to_pixel_coords, PIECE_SIZE},
};

#[derive(Event, Debug)]
pub struct PossibleMoveDisplayEvent {
    pub from: TilePos,
    pub show: bool,
}

#[derive(Component)]
pub struct PossibleMoveMarker;

pub fn possible_move_event_handler(
    mut ev_display: EventReader<PossibleMoveDisplayEvent>,
    possible_move_entities: Query<Entity, With<PossibleMoveMarker>>,
    mut commands: Commands,
) {
    for ev in ev_display.read() {
        if ev.show {
            // TODO Get possible moves
            let from = ev.from;
            let positions = vec![
                ev.from,
                TilePos::new(from.file + 1, from.rank),
                TilePos::new(from.file, from.rank + 1),
                TilePos::new(from.file, from.rank - 1),
            ];

            for pos in positions {
                let (x, y) = board_to_pixel_coords(pos.file, pos.rank);

                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgba(1., 0., 1., 0.75),
                            ..default()
                        },
                        transform: Transform::from_xyz(x, y, 2.)
                            .with_scale(Vec3::splat(PIECE_SIZE * 0.75)),
                        ..default()
                    },
                    PossibleMoveMarker,
                ));
            }
        } else {
            for entity in possible_move_entities.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}
