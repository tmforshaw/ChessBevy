use std::fmt;

use bevy::prelude::*;

use crate::{
    board::{Board, TilePos},
    display::{board_to_pixel_coords, BackgroundColourEvent},
    piece::Piece,
    possible_moves::get_possible_moves,
};

#[derive(Event)]
pub struct PieceMoveEvent {
    pub piece_move: PieceMove,
    pub entity: Entity,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PieceMove {
    pub from: TilePos,
    pub to: TilePos,
    pub en_passant: Option<TilePos>,
    pub show: bool,
}

impl PieceMove {
    #[must_use]
    pub const fn new(from: TilePos, to: TilePos) -> Self {
        Self {
            from,
            to,
            en_passant: None,
            show: true,
        }
    }

    #[must_use]
    pub const fn new_unshown(from: TilePos, to: TilePos) -> Self {
        Self {
            from,
            to,
            en_passant: None,
            show: false,
        }
    }

    #[must_use]
    pub const fn with_show(&self, show: bool) -> Self {
        Self {
            from: self.from,
            to: self.to,
            en_passant: self.en_passant,
            show,
        }
    }

    #[must_use]
    pub const fn with_en_passant(&self, en_passant: Option<TilePos>) -> Self {
        Self {
            from: self.from,
            to: self.to,
            en_passant,
            show: self.show,
        }
    }

    pub fn to_algebraic(&self) -> Result<String, std::num::TryFromIntError> {
        Ok(format!(
            "{} {}",
            self.from.to_algebraic()?,
            self.to.to_algebraic()?
        ))
    }

    #[must_use]
    pub const fn rev(&self) -> Self {
        Self {
            from: self.to,
            to: self.from,
            en_passant: self.en_passant,
            show: self.show,
        }
    }
}

impl std::fmt::Debug for PieceMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{from: {}, to: {}, show: {}, en_passant: {:?}}}",
            self.from, self.to, self.show, self.en_passant
        )
    }
}

impl std::fmt::Display for PieceMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{{}, {}, {}, {:?}}}",
            self.from, self.to, self.show, self.en_passant
        )
    }
}

pub fn piece_move_event_reader(
    mut commands: Commands,
    mut ev_piece_move: EventReader<PieceMoveEvent>,
    mut transform_query: Query<&mut Transform>,
    mut board: ResMut<Board>,
    mut background_ev: EventWriter<BackgroundColourEvent>,
) {
    for ev in ev_piece_move.read() {
        let mut piece_move = ev.piece_move;

        // Entity Logic
        let mut piece_captured = false;
        let move_complete;
        {
            let mut transform = transform_query.get_mut(ev.entity).unwrap();

            let moved_to = board.get_piece(piece_move.to);

            // Snap the moved entity to the grid (Don't move if there is a non-opponent piece there, or if you moved a piece on another player's turn, or if the move is impossible for that piece type)
            let (x, y) = if !moved_to.is_player(board.player)
                && board.get_piece(piece_move.from).is_player(board.player)
                && get_possible_moves(&mut board, piece_move.from).contains(&piece_move.to)
            {
                // Need to capture
                if moved_to != Piece::None {
                    piece_captured = true;
                    let captured_entity = board.get_entity(piece_move.to).unwrap();

                    commands.entity(captured_entity).despawn();
                }

                move_complete = true;
                board_to_pixel_coords(piece_move.to.file, piece_move.to.rank)
            } else {
                // Reset position
                move_complete = false;
                board_to_pixel_coords(piece_move.from.file, piece_move.from.rank)
            };
            transform.translation = Vec3::new(x, y, 1.);
        }

        // Board Logic
        if (move_complete && piece_move.show) || (!move_complete && !piece_move.show) {
            if piece_move.show {
                let mut piece_moved_to = if piece_captured {
                    board.get_piece(piece_move.to)
                } else {
                    Piece::None
                };

                let moved_piece = board.get_piece(piece_move.from);

                // Check if piece moved to the en passant tile
                let _en_passant_capture = if let Some(en_passant) = board.en_passant_on_last_move {
                    if en_passant == piece_move.to {
                        // Get the captured piece type from the Board
                        let captured_piece_pos = TilePos::new(
                            piece_move.to.file,
                            piece_move.from.rank, // The rank which the piece moved from is the same as the piece it will capture
                        );
                        let captured_piece = board.get_piece(captured_piece_pos);

                        println!("En passant capture {captured_piece:?}");

                        // Mark that there was a piece captured via en passant
                        piece_captured = true;
                        piece_move = piece_move.with_en_passant(Some(en_passant));

                        // Delete the piece at the captured tile
                        let captured_entity = board.get_entity(captured_piece_pos).unwrap();
                        commands.entity(captured_entity).despawn();
                        board.set_piece(captured_piece_pos, Piece::None);

                        // Should never fail
                        assert!(
                            piece_moved_to == Piece::None,
                            "Piece moved to was not empty when trying to overwrite with en passant"
                        );

                        piece_moved_to = captured_piece;

                        Some(en_passant)
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Clear the en_passant marker
                board.en_passant_on_last_move = None;

                // Check if this move allows en passant on the next move
                if Board::double_pawn_move_check(moved_piece, piece_move.from)
                    && (isize::try_from(piece_move.from.rank).unwrap()
                        - isize::try_from(piece_move.to.rank).unwrap())
                    .abs()
                        == 2
                {
                    let en_passant_tile = TilePos::new(
                        piece_move.to.file,
                        usize::try_from(
                            isize::try_from(piece_move.from.rank).unwrap()
                                + Board::get_vertical_dir(moved_piece),
                        )
                        .unwrap(),
                    );

                    board.en_passant_on_last_move = Some(en_passant_tile);
                }

                let captured_piece = if piece_captured {
                    Some(piece_moved_to)
                } else {
                    None
                };

                board.move_history.make_move(piece_move, captured_piece);

                // Change background colour to show current move
                board.next_player();
                background_ev.send(BackgroundColourEvent::new(board.get_player()));
            }

            board.move_piece(piece_move);
        }
    }
}
