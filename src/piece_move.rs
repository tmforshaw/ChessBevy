use bevy::prelude::*;

use crate::{
    board::{Board, TilePos},
    display::board_to_pixel_coords,
    piece::Piece,
};

#[derive(Event)]
pub struct PieceMoveEvent {
    pub piece_move: PieceMove,
    pub entity: Entity,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PieceMove {
    pub from: TilePos,
    pub to: TilePos,
}

impl PieceMove {
    pub fn new(from: TilePos, to: TilePos) -> Self {
        Self { from, to }
    }
}

pub fn piece_move_event_reader(
    mut commands: Commands,
    mut ev_piece_move: EventReader<PieceMoveEvent>,
    mut transform_query: Query<&mut Transform>,
    mut board: ResMut<Board>,
) {
    for ev in ev_piece_move.read() {
        // Entity Logic
        let move_complete;
        {
            let mut transform = transform_query.get_mut(ev.entity).unwrap();

            let moved_to = board.get_piece(ev.piece_move.to);

            // Snap the moved entity to the grid (Don't move if there is a non-opponent piece there, or if you moved a piece on another player's turn)
            let (x, y) = if !moved_to.is_player(board.player)
                && board.get_piece(ev.piece_move.from).is_player(board.player)
            {
                if moved_to != Piece::None {
                    let moved_to = board.get_entity(ev.piece_move.to).unwrap();

                    commands.entity(moved_to).despawn();
                }

                move_complete = true;
                board_to_pixel_coords(ev.piece_move.to.file, ev.piece_move.to.rank)
            } else {
                // Reset position
                move_complete = false;
                board_to_pixel_coords(ev.piece_move.from.file, ev.piece_move.from.rank)
            };
            transform.translation = Vec3::new(x, y, 1.);
        }

        // Board Logic
        if move_complete {
            board.move_piece(ev.piece_move);
            board.next_player();
            println!("{}", board.clone());
        }
    }
}
