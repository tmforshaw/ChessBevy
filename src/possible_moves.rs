use bevy::prelude::*;

use chess_core::board::TilePos;

use crate::{
    board::BoardBevy,
    display::{board_to_pixel_coords, PIECE_SIZE},
};

const POSSIBLE_MOVE_COLOUR: Color = Color::linear_rgba(0., 1., 0., 0.75);

#[derive(Event, Debug)]
pub struct PossibleMoveDisplayEvent {
    pub from: TilePos,
    pub show: bool,
}

#[derive(Component)]
pub struct PossibleMoveMarker;

#[allow(clippy::needless_pass_by_value)]
pub fn possible_move_event_handler(
    mut ev_display: EventReader<PossibleMoveDisplayEvent>,
    possible_move_entities: Query<Entity, With<PossibleMoveMarker>>,
    mut commands: Commands,
    board: ResMut<BoardBevy>,
) {
    for ev in ev_display.read() {
        if ev.show {
            for pos in board.board.get_possible_moves(ev.from) {
                let (x, y) = board_to_pixel_coords(pos.to.file, pos.to.rank);

                commands.spawn((
                    Sprite {
                        color: POSSIBLE_MOVE_COLOUR,
                        ..default()
                    },
                    Transform::from_xyz(x, y, 2.).with_scale(Vec3::splat(PIECE_SIZE * 0.75)),
                    PossibleMoveMarker,
                ));
            }
        } else {
            // Stop displaying all entities
            for entity in possible_move_entities.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}
