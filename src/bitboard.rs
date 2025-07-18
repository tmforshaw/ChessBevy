use bevy::prelude::*;

use chess_core::piece::Piece;

use crate::{
    board::BoardBevy,
    display::{board_to_pixel_coords, PIECE_SIZE},
};

#[derive(Event, Debug)]
pub struct BitBoardDisplayEvent {
    pub board_type: Option<Piece>,
    pub clear: bool,
    pub show: bool,
}

impl BitBoardDisplayEvent {
    #[must_use]
    pub const fn new(board_type: Option<Piece>, clear: bool, show: bool) -> Self {
        Self {
            board_type,
            clear,
            show,
        }
    }
}

#[derive(Component)]
pub struct BitBoardMarker;

#[allow(clippy::needless_pass_by_value)]
pub fn bitboard_event_handler(
    mut ev_display: EventReader<BitBoardDisplayEvent>,
    board: Res<BoardBevy>,
    bitboard_entities: Query<Entity, With<BitBoardMarker>>,
    mut commands: Commands,
) {
    for ev in ev_display.read() {
        if ev.clear {
            for entity in bitboard_entities.iter() {
                commands.entity(entity).despawn();
            }
        }

        if ev.show {
            if let Some(board_type) = ev.board_type {
                let bitboard = board.board.positions[board_type];

                for pos in bitboard.get_positions() {
                    let (x, y) = board_to_pixel_coords(pos.file, pos.rank);

                    commands.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                color: Color::rgba(1., 0., 0., 0.75),
                                ..default()
                            },
                            transform: Transform::from_xyz(x, y, 2.)
                                .with_scale(Vec3::splat(PIECE_SIZE * 0.75)),
                            ..default()
                        },
                        BitBoardMarker,
                    ));
                }
            } else {
                let Some(pos) = board.board.en_passant_on_last_move else {
                    return;
                };
                let xy = [
                    board_to_pixel_coords(3, 5),
                    board_to_pixel_coords(1, 0),
                    board_to_pixel_coords(pos.file, pos.rank),
                ];

                for (x, y) in xy {
                    commands.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                color: Color::rgba(1., 0., 0., 0.75),
                                ..default()
                            },
                            transform: Transform::from_xyz(x, y, 2.)
                                .with_scale(Vec3::splat(PIECE_SIZE * 0.75)),
                            ..default()
                        },
                        BitBoardMarker,
                    ));
                }
            }
        }
    }
}
