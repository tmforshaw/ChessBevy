use bevy::prelude::*;
use thiserror::Error;

use std::fmt;

use crate::{
    board::{Board, TilePos},
    display::BackgroundColourEvent,
    game_end::GameEndEvent,
    piece::{Piece, COLOUR_AMT},
    piece_move::PieceMove,
};

#[derive(Error, Debug)]
pub enum MoveHistoryError {
    #[error("Index not changed after history traversal")]
    IndexNotChanged,
}

#[derive(Clone, Debug, Copy)]
pub struct HistoryMove {
    piece_move: PieceMove,
    captured_piece: Option<Piece>,
    en_passant_tile: Option<TilePos>,
    castling_rights: [(bool, bool); COLOUR_AMT],
}

impl HistoryMove {
    #[must_use]
    pub const fn new(
        piece_move: PieceMove,
        captured_piece: Option<Piece>,
        en_passant_tile: Option<TilePos>,
        castling_rights: [(bool, bool); COLOUR_AMT],
    ) -> Self {
        Self {
            piece_move,
            captured_piece,
            en_passant_tile,
            castling_rights,
        }
    }
}

impl
    From<(
        PieceMove,
        Option<Piece>,
        Option<TilePos>,
        [(bool, bool); COLOUR_AMT],
    )> for HistoryMove
{
    fn from(
        value: (
            PieceMove,
            Option<Piece>,
            Option<TilePos>,
            [(bool, bool); COLOUR_AMT],
        ),
    ) -> Self {
        Self {
            piece_move: value.0,
            captured_piece: value.1,
            en_passant_tile: value.2,
            castling_rights: value.3,
        }
    }
}

impl From<HistoryMove>
    for (
        PieceMove,
        Option<Piece>,
        Option<TilePos>,
        [(bool, bool); COLOUR_AMT],
    )
{
    fn from(value: HistoryMove) -> Self {
        (
            value.piece_move,
            value.captured_piece,
            value.en_passant_tile,
            value.castling_rights,
        )
    }
}

#[derive(Default, Clone, Debug)]
pub struct PieceMoveHistory {
    pub moves: Vec<HistoryMove>,
    pub current_idx: Option<usize>,
}

impl PieceMoveHistory {
    #[must_use]
    pub const fn new(moves: Vec<HistoryMove>, current_idx: Option<usize>) -> Self {
        Self { moves, current_idx }
    }

    pub fn make_move(
        &mut self,
        piece_move: PieceMove,
        captured_piece: Option<Piece>,
        en_passant_tile: Option<TilePos>,
        castling_rights: [(bool, bool); COLOUR_AMT],
    ) {
        if piece_move.show {
            // Clear history depending on where current_idx is (if the move is different from the history)
            if let Some(current_idx) = self.current_idx {
                // If the suggested move is different to the current move in history, and is not the last move in the history
                if piece_move != self.moves[current_idx].piece_move
                    && current_idx + 1 < self.moves.len()
                {
                    self.clear_excess_moves();
                }
            } else if !self.moves.is_empty() {
                self.clear_excess_moves();
            }

            self.moves.push(HistoryMove::new(
                piece_move,
                captured_piece,
                en_passant_tile,
                castling_rights,
            ));
            let _ = self.increment_index();
        }
    }

    /// # Errors
    /// Returns an error if the index was moved out of bounds
    pub const fn increment_index(&mut self) -> Result<(), MoveHistoryError> {
        let mut index_changed = true;

        // Increment the index, unless it is at maximum index
        self.current_idx = Some(match self.current_idx {
            Some(idx) if idx < self.moves.len() - 1 => idx + 1,
            Some(idx) => {
                index_changed = false;
                idx
            }
            None => 0,
        });

        // Return whether the index was changed
        if index_changed {
            Ok(())
        } else {
            Err(MoveHistoryError::IndexNotChanged)
        }
    }

    /// # Errors
    /// Returns an error if the index was moved out of bounds
    pub const fn decrement_index(&mut self) -> Result<(), MoveHistoryError> {
        let mut index_changed = true;

        // Decrement the index, unless it is at -1st index (Index == None)
        self.current_idx = match self.current_idx {
            Some(0) => None,
            Some(idx) => Some(idx - 1),
            None => {
                index_changed = false;
                None
            }
        };

        // Return whether the index was changed
        if index_changed {
            Ok(())
        } else {
            Err(MoveHistoryError::IndexNotChanged)
        }
    }

    pub fn clear_excess_moves(&mut self) {
        if let Some(current_idx) = self.current_idx {
            self.moves = self.moves[0..=current_idx].to_vec();
            self.current_idx = (!self.moves.is_empty()).then_some(self.moves.len() - 1);
        } else {
            self.moves = vec![];
        }
    }

    #[must_use]
    pub fn get(&self) -> Option<HistoryMove> {
        if self.moves.is_empty() {
            None
        } else {
            Some(self.moves[self.current_idx.unwrap_or(0)])
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut HistoryMove> {
        if self.moves.is_empty() {
            None
        } else {
            Some(&mut self.moves[self.current_idx.unwrap_or(0)])
        }
    }

    pub fn traverse_next(&mut self) -> Option<HistoryMove> {
        if !self.moves.is_empty() {
            // Increment index and if it was changed, return the move at the new index
            if self.increment_index().is_ok() {
                return Some(self.moves[self.current_idx.unwrap_or(0)]);
            }
        }

        None
    }

    pub fn traverse_prev(&mut self) -> Option<HistoryMove> {
        if !self.moves.is_empty() {
            let history_move = self.moves[self.current_idx.unwrap_or(0)];

            if self.decrement_index().is_ok() {
                return Some(history_move);
            }
        }

        None
    }
}

impl fmt::Display for PieceMoveHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut message = String::new();

        // TODO Print out other parts of HistoryMove
        for history_move in &self.moves {
            let Ok(algebraic) = history_move.piece_move.to_algebraic() else {
                return Err(fmt::Error);
            };

            message += format!("{algebraic}\t").as_str();
        }

        message += format!("\t\tCurrent Index: {:?}", self.current_idx).as_str();

        write!(f, "{message}")
    }
}

#[derive(Event)]
pub struct MoveHistoryEvent {
    pub backwards: bool,
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::too_many_arguments)]
pub fn move_history_event_handler(
    mut move_history_ev: EventReader<MoveHistoryEvent>,
    mut board: ResMut<Board>,
    mut transform_query: Query<&mut Transform>,
    mut background_ev: EventWriter<BackgroundColourEvent>,
    mut game_end_ev: EventWriter<GameEndEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut texture_atlas_query: Query<&mut TextureAtlas>,
) {
    for ev in move_history_ev.read() {
        // Traverse the history in the specified direction
        let piece_move_history = if ev.backwards {
            board.move_history.traverse_prev()
        } else {
            board.move_history.traverse_next()
        };

        let Some(history_move) = piece_move_history else {
            // History is empty, or index went out of bounds (Don't perform any moves)
            return;
        };

        if ev.backwards {
            board.undo_move(
                &mut commands,
                &asset_server,
                &mut texture_atlas_layouts,
                &mut transform_query,
                &mut texture_atlas_query,
                &mut background_ev,
                history_move,
            );
        } else {
            let (piece_move_original, _, _, _) = history_move.into();

            let _ = board.apply_move(
                &mut commands,
                &mut transform_query,
                &mut texture_atlas_query,
                &mut background_ev,
                &mut game_end_ev,
                piece_move_original,
            );
        }
    }
}
