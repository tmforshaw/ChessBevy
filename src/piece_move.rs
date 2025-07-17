use std::fmt;

use bevy::prelude::*;

use crate::{
    board::{Board, TilePos},
    checkmate::CheckmateEvent,
    display::{translate_piece_entity, BackgroundColourEvent, BOARD_SIZE},
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
    Promotion(Piece),
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
    pub const fn with_promotion(&self, promoted_to: Piece) -> Self {
        Self {
            from: self.from,
            to: self.to,
            move_type: PieceMoveType::Promotion(promoted_to),
            show: self.show,
        }
    }

    /// # Errors
    /// Returns an error if the from and to tiles contain files which cannot be converted to integers
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
    mut texture_atlas_query: Query<&mut TextureAtlas>,
    mut board: ResMut<Board>,
    mut background_ev: EventWriter<BackgroundColourEvent>,
    mut checkmate_ev: EventWriter<CheckmateEvent>,
) {
    for ev in ev_piece_move.read() {
        let mut piece_move = ev.piece_move;

        // Snap the moved entity to the grid (Don't move if there is a non-opponent piece there, or if you moved a piece on another player's turn, or if the move is impossible for that piece type)
        if let Some(possible_moves) = get_possible_moves(&board, piece_move.from) {
            if !board.get_piece(piece_move.to).is_player(board.player)
                && board.get_piece(piece_move.from).is_player(board.player)
                && possible_moves.contains(&piece_move.to)
            {
                // Apply the move to the board
                let (en_passant_tile, castling_rights_before_move, captured_piece);
                (
                    piece_move,
                    en_passant_tile,
                    castling_rights_before_move,
                    captured_piece,
                ) = board.apply_move(
                    &mut commands,
                    &mut transform_query,
                    &mut texture_atlas_query,
                    &mut background_ev,
                    &mut checkmate_ev,
                    piece_move,
                );

                // Update the move history with this move
                board.move_history.make_move(
                    piece_move,
                    captured_piece,
                    en_passant_tile,
                    castling_rights_before_move,
                );
            } else {
                // Reset position
                translate_piece_entity(&mut transform_query, ev.entity, piece_move.from);
            }
        }
    }
}

pub fn apply_promotion(
    board: &mut Board,
    texture_atlas_query: &mut Query<&mut TextureAtlas>,
    moved_piece: Piece,
    mut piece_move: PieceMove,
) -> PieceMove {
    // Pawn was moved onto final file (Player doesn't matter here since pawn cannot move backwards)
    if moved_piece == board.get_player_piece(board.get_player(), Piece::WPawn)
        && (piece_move.to.rank == BOARD_SIZE - 1 || piece_move.to.rank == 0)
    {
        let promoted_to = match piece_move.move_type {
            PieceMoveType::Promotion(promoted_to) => promoted_to,
            _ => board.get_player_piece(board.get_player(), Piece::WQueen), // TODO Allow choosing which piece to promote to
        };

        piece_move = piece_move.with_promotion(promoted_to);

        perform_promotion(board, texture_atlas_query, piece_move.from, promoted_to);
    }

    piece_move
}

/// # Panics
/// Panics if the piece at the position doesn't have an associated entity, with a texture atlas
pub fn perform_promotion(
    board: &mut Board,
    texture_atlas_query: &mut Query<&mut TextureAtlas>,
    from: TilePos,
    new_piece_type: Piece,
) {
    // Change the type of the piece in the internal board
    board.set_piece(from, new_piece_type);

    // Change the entity texture to the correct piece
    let piece_entity = board
        .get_entity(from)
        .unwrap_or_else(|| panic!("Entity not found for piece at pos {from}"));
    let mut texture_atlas = texture_atlas_query
        .get_mut(piece_entity)
        .expect("Could not find piece entity in texture atlas");
    texture_atlas.index = new_piece_type.to_bitboard_index();
}

// Returns the en_passant tile for this move
/// # Errors
/// Returns ``None`` if the captured piece's entity could not be found
/// Returns ``None`` if the file or rank could not be converted to isize (or back to usize after computation)
pub fn handle_en_passant(
    board: &mut Board,
    commands: &mut Commands,
    mut piece_move: PieceMove,
    moved_piece: Piece,
    mut piece_captured: bool,
    mut piece_moved_to: Piece,
) -> Option<(Option<TilePos>, PieceMove, bool, Piece)> {
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
            let captured_entity = board.get_entity(captured_piece_pos)?;
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
        && (isize::try_from(piece_move.from.rank).ok()?
            - isize::try_from(piece_move.to.rank).ok()?)
        .abs()
            == 2
    {
        let en_passant_tile = TilePos::new(
            piece_move.to.file,
            usize::try_from(
                isize::try_from(piece_move.from.rank).ok()? + Board::get_vertical_dir(moved_piece),
            )
            .ok()?,
        );

        board.en_passant_on_last_move = Some(en_passant_tile);
    }

    Some((en_passant_tile, piece_move, piece_captured, piece_moved_to))
}

pub fn handle_castling(
    board: &mut Board,
    transform_query: &mut Query<&mut Transform>,
    mut piece_move: PieceMove,
    moved_piece: Piece,
) -> Option<([(bool, bool); COLOUR_AMT], PieceMove)> {
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

    piece_move = perform_castling(board, transform_query, piece_move, moved_piece, false)?;

    Some((castling_rights_before_move, piece_move))
}

pub fn perform_castling(
    board: &mut Board,
    transform_query: &mut Query<&mut Transform>,
    mut piece_move: PieceMove,
    moved_piece: Piece,
    undo: bool,
) -> Option<PieceMove> {
    // If piece is this player's king, and the king moved 2 spaces
    let file_diff_isize =
        isize::try_from(piece_move.to.file).ok()? - isize::try_from(piece_move.from.file).ok()?;
    if moved_piece == board.get_player_king(board.get_player())
        && file_diff_isize.unsigned_abs() == 2
        || undo
    {
        piece_move = piece_move.with_castling();

        // Kingside Castle
        if file_diff_isize > 0 {
            move_rook_for_castle(
                board,
                transform_query,
                BOARD_SIZE - 1,
                BOARD_SIZE - 3,
                piece_move.from.rank,
                undo,
            );
        } else {
            move_rook_for_castle(board, transform_query, 0, 3, piece_move.from.rank, undo);
        }
    }

    Some(piece_move)
}

fn move_rook_for_castle(
    board: &mut Board,
    transform_query: &mut Query<&mut Transform>,
    file: usize,
    new_file: usize,
    from_rank: usize,
    undo: bool,
) {
    let mut rook_pos = TilePos::new(file, from_rank);
    let mut new_rook_pos = TilePos::new(new_file, rook_pos.rank);

    if undo {
        std::mem::swap(&mut rook_pos, &mut new_rook_pos);
    }

    // Move the rook entity
    translate_piece_entity(
        transform_query,
        board
            .get_entity(rook_pos)
            .expect("Rook entity was not at Rook pos"),
        new_rook_pos,
    );

    // Move the rook (and its entity ID) internally
    board.move_piece(PieceMove::new(rook_pos, new_rook_pos));
}
