use bevy::prelude::*;

use chess_core::{
    board::{Player, TilePos},
    piece::Piece,
};

use crate::{
    board::BoardBevy,
    display::{board_to_pixel_coords, PIECE_SIZE},
};

#[derive(Event, Debug)]
pub struct BitBoardDisplayEvent {
    pub board_type: Option<Piece>,
    pub clear: bool,
    pub show: bool,
    pub board_unshow_type: usize,
}

impl BitBoardDisplayEvent {
    #[must_use]
    pub const fn new(
        board_type: Option<Piece>,
        clear: bool,
        show: bool,
        board_unshow_type: usize,
    ) -> Self {
        Self {
            board_type,
            clear,
            show,
            board_unshow_type,
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

        if let Some(board_type) = ev.board_type {
            let bitboard = board.board.positions[board_type];

            for pos in bitboard.to_tile_positions() {
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
            let mut xy = Vec::new();

            match ev.board_unshow_type {
                1 => {
                    // Show en passant tile
                    if board.board.positions.en_passant_tile == 0 {
                        return;
                    }

                    let pos =
                        TilePos::from_index(board.board.positions.en_passant_tile.trailing_zeros());

                    xy.push(board_to_pixel_coords(pos.file, pos.rank));
                }
                2 | 3 => {
                    // Show attacked tiles

                    let player = if ev.board_unshow_type == 2 {
                        Player::White
                    } else {
                        Player::Black
                    };

                    let mut attacked = board
                        .board
                        .positions
                        .get_attacked_tiles(player)
                        .to_tile_positions()
                        .iter()
                        .map(|&pos| Into::<(u32, u32)>::into(pos))
                        .map(|(i, j)| board_to_pixel_coords(i, j))
                        .collect::<Vec<_>>();

                    xy.append(&mut attacked);
                }
                _ => {
                    continue;
                }
            }

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
