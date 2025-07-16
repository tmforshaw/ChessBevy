use std::fmt;

use bevy::prelude::*;

use crate::{
    board::{Board, TilePos},
    checkmate::CheckmateEvent,
    display::{board_to_pixel_coords, BackgroundColourEvent, BOARD_SIZE},
    piece::{Piece, COLOUR_AMT},
    possible_moves::get_possible_moves,
};

#[derive(Event)]
pub struct PieceMoveEvent {
    pub piece_move: PieceMove,
    pub entity: Entity,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum PieceMoveType {
    #[default]
    Normal,
    EnPassant,
    Castling,
    Promotion {
        from_piece: Piece,
        to_piece: Piece,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PieceMove {
    pub from: TilePos,
    pub to: TilePos,
    pub move_type: PieceMoveType,
    pub show: bool,
}

impl PieceMove {
    #[must_use]
    pub const fn new(from: TilePos, to: TilePos) -> Self {
        Self {
            from,
            to,
            move_type: PieceMoveType::Normal,
            show: true,
        }
    }

    #[must_use]
    pub const fn with_show(&self, show: bool) -> Self {
        Self {
            from: self.from,
            to: self.to,
            move_type: self.move_type,
            show,
        }
    }

    #[must_use]
    pub const fn with_castling(&self) -> Self {
        Self {
            from: self.from,
            to: self.to,
            move_type: PieceMoveType::Castling,
            show: self.show,
        }
    }

    #[must_use]
    pub const fn with_en_passant_capture(&self) -> Self {
        Self {
            from: self.from,
            to: self.to,
            move_type: PieceMoveType::EnPassant,
            show: self.show,
        }
    }

    #[must_use]
    pub const fn with_promotion(&self, from_piece: Piece, to_piece: Piece) -> Self {
        Self {
            from: self.from,
            to: self.to,
            move_type: PieceMoveType::Promotion {
                from_piece,
                to_piece,
            },
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
            move_type: self.move_type,
            show: self.show,
        }
    }
}

impl std::fmt::Debug for PieceMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{from: {}, to: {}, show: {}, move_type: {:?}}}",
            self.from, self.to, self.show, self.move_type
        )
    }
}

impl std::fmt::Display for PieceMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{{}, {}, {}, {:?}}}",
            self.from, self.to, self.show, self.move_type
        )
    }
}

pub fn piece_move_event_handler(
    mut commands: Commands,
    mut ev_piece_move: EventReader<PieceMoveEvent>,
    mut transform_query: Query<&mut Transform>,
    mut board: ResMut<Board>,
    mut background_ev: EventWriter<BackgroundColourEvent>,
    mut checkmate_ev: EventWriter<CheckmateEvent>,
) {
    for ev in ev_piece_move.read() {
        let mut piece_move = ev.piece_move;

        // Entity Logic
        let mut piece_captured = false;
        let move_complete;
        {
            let moved_to = board.get_piece(piece_move.to);

            // Snap the moved entity to the grid (Don't move if there is a non-opponent piece there, or if you moved a piece on another player's turn, or if the move is impossible for that piece type)
            let pos = if !moved_to.is_player(board.player)
                && board.get_piece(piece_move.from).is_player(board.player)
                && get_possible_moves(&board, piece_move.from).contains(&piece_move.to)
            {
                // Need to capture
                if moved_to != Piece::None {
                    piece_captured = true;
                    let captured_entity = board.get_entity(piece_move.to).unwrap();

                    commands.entity(captured_entity).despawn();
                }

                move_complete = true;
                piece_move.to
            } else {
                // Reset position
                move_complete = false;
                piece_move.from
            };

            translate_piece_entity(ev.entity, pos, &mut transform_query);
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

                // Handle promotion

                // Handle en passant, if this move is en passant, or if this move allows en passant on the next move
                let en_passant_tile;
                (en_passant_tile, piece_move, piece_captured, piece_moved_to) = handle_en_passant(
                    &mut board,
                    &mut commands,
                    piece_move,
                    moved_piece,
                    piece_captured,
                    piece_moved_to,
                );

                // Handle Castling
                let castling_rights_before_move;
                (castling_rights_before_move, piece_move) =
                    handle_castling(&mut board, &mut transform_query, piece_move, moved_piece);

                let captured_piece = if piece_captured {
                    Some(piece_moved_to)
                } else {
                    None
                };

                board.move_history.make_move(
                    piece_move,
                    captured_piece,
                    en_passant_tile,
                    castling_rights_before_move,
                );

                // Change background colour to show current move
                board.next_player();
                background_ev.send(BackgroundColourEvent::new_from_player(board.get_player()));
            }

            board.move_piece(piece_move);
        }

        // Check if this move has caused a checkmate
        if board.is_checkmate() {
            checkmate_ev.send(CheckmateEvent::new(board.get_player()));
        }
    }
}

pub fn translate_piece_entity(
    piece_entity: Entity,
    pos: TilePos,
    transform_query: &mut Query<&mut Transform>,
) {
    let mut transform = transform_query.get_mut(piece_entity).unwrap();
    let (x, y) = board_to_pixel_coords(pos.file, pos.rank);
    transform.translation = Vec3::new(x, y, 1.);
}

// Returns the en_passant tile for this move
fn handle_en_passant(
    board: &mut Board,
    commands: &mut Commands,
    mut piece_move: PieceMove,
    moved_piece: Piece,
    mut piece_captured: bool,
    mut piece_moved_to: Piece,
) -> (Option<TilePos>, PieceMove, bool, Piece) {
    // Check if piece moved to the en passant tile
    if let Some(en_passant) = board.en_passant_on_last_move {
        if en_passant == piece_move.to {
            // Get the captured piece type from the Board
            let captured_piece_pos = TilePos::new(
                piece_move.to.file,
                piece_move.from.rank, // The rank which the piece moved from is the same as the piece it will capture
            );
            let captured_piece = board.get_piece(captured_piece_pos);

            // Mark that there was a piece captured via en passant
            piece_captured = true;
            piece_move = piece_move.with_en_passant_capture();

            // Delete the piece at the captured tile
            let captured_entity = board.get_entity(captured_piece_pos).unwrap();
            commands.entity(captured_entity).despawn();
            board.set_piece(captured_piece_pos, Piece::None);

            piece_moved_to = captured_piece;
        }
    }

    // Clear the en_passant marker, caching it for use in the history_move.make_move() function
    let en_passant_tile = board.en_passant_on_last_move;
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

    (en_passant_tile, piece_move, piece_captured, piece_moved_to)
}

pub fn handle_castling(
    board: &mut Board,
    transform_query: &mut Query<&mut Transform>,
    mut piece_move: PieceMove,
    moved_piece: Piece,
) -> ([(bool, bool); COLOUR_AMT], PieceMove) {
    // Remember the castling rights before this move
    let castling_rights_before_move = board.castling_rights;

    // Handle castling rights
    {
        let player_index = board.get_player().to_index();

        // Only update if the castling rights aren't already false
        if board.castling_rights[player_index] != (false, false) {
            // King was moved
            if moved_piece == board.get_player_king(board.get_player()) {
                board.castling_rights[player_index] = (false, false);
            }
            // Rook was moved
            else if moved_piece == board.get_player_piece(board.get_player(), Piece::WRook) {
                // Kingside
                if piece_move.from.file == BOARD_SIZE - 1 {
                    board.castling_rights[player_index].0 = false;
                }
                // Queenside
                else if piece_move.from.file == 0 {
                    board.castling_rights[player_index].1 = false;
                }
            }
        }
    }

    // If piece is this player's king, and the king moved 2 spaces
    let file_diff_isize = isize::try_from(piece_move.to.file).unwrap()
        - isize::try_from(piece_move.from.file).unwrap();
    if moved_piece == board.get_player_king(board.get_player())
        && file_diff_isize.unsigned_abs() == 2
    {
        piece_move = piece_move.with_castling();

        fn move_rook_for_castle(
            board: &mut Board,
            transform_query: &mut Query<&mut Transform>,
            file: usize,
            new_file: usize,
            from_rank: usize,
        ) {
            let rook_pos = TilePos::new(file, from_rank);
            let new_rook_pos = TilePos::new(new_file, rook_pos.rank);

            // Move the rook entity
            translate_piece_entity(
                board
                    .get_entity(rook_pos)
                    .expect("Rook entity was not at Rook pos"),
                new_rook_pos,
                transform_query,
            );

            // Move the rook (and its entity ID) internally
            board.move_piece(PieceMove::new(rook_pos, new_rook_pos));
        }

        // Kingside Castle
        if file_diff_isize > 0 {
            move_rook_for_castle(
                board,
                transform_query,
                BOARD_SIZE - 1,
                BOARD_SIZE - 3,
                piece_move.from.rank,
            );
        } else {
            move_rook_for_castle(board, transform_query, 0, 3, piece_move.from.rank);
        }
    }

    (castling_rights_before_move, piece_move)
}
