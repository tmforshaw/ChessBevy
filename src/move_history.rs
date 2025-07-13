use bevy::prelude::*;
use thiserror::Error;

use std::fmt;

use crate::{
    board::{Board, TilePos},
    display::{board_to_pixel_coords, get_texture_atlas, BackgroundColourEvent},
    piece::{Piece, PieceBundle},
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
}

impl HistoryMove {
    #[must_use]
    pub const fn new(
        piece_move: PieceMove,
        captured_piece: Option<Piece>,
        en_passant_tile: Option<TilePos>,
    ) -> Self {
        Self {
            piece_move,
            captured_piece,
            en_passant_tile,
        }
    }
}

impl From<(PieceMove, Option<Piece>, Option<TilePos>)> for HistoryMove {
    fn from(value: (PieceMove, Option<Piece>, Option<TilePos>)) -> Self {
        Self {
            piece_move: value.0,
            captured_piece: value.1,
            en_passant_tile: value.2,
        }
    }
}

impl From<HistoryMove> for (PieceMove, Option<Piece>, Option<TilePos>) {
    fn from(value: HistoryMove) -> Self {
        (
            value.piece_move,
            value.captured_piece,
            value.en_passant_tile,
        )
    }
}

#[derive(Default, Clone, Debug)]
pub struct PieceMoveHistory {
    moves: Vec<HistoryMove>,
    current_idx: Option<usize>,
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
            }

            self.moves.push(HistoryMove::new(
                piece_move,
                captured_piece,
                en_passant_tile,
            ));
            let _ = self.increment_index();
        }
    }

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
        self.moves = self.moves[0..=self.current_idx.unwrap_or(0)].to_vec();
        self.current_idx = (!self.moves.is_empty()).then_some(self.moves.len() - 1);
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
                return Some(self.moves[self.current_idx.unwrap()]);
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
pub fn move_history_event_handler(
    mut move_history_ev: EventReader<MoveHistoryEvent>,
    mut board: ResMut<Board>,
    mut transform_query: Query<&mut Transform>,
    mut background_ev: EventWriter<BackgroundColourEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let (texture, texture_atlas_layout) =
        get_texture_atlas(asset_server, &mut texture_atlas_layouts);

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

        let (piece_move_original, captured_piece, en_passant_tile) = history_move.into();

        // Undo
        let piece_move = if ev.backwards {
            piece_move_original.rev()
        } else {
            piece_move_original
        };

        // TODO Need to set the en passant marker on each turn
        // Set the en_passant marker
        board.en_passant_on_last_move = en_passant_tile;

        let Some(piece_entity) = board.get_entity(piece_move.from) else {
            eprintln!(
                "Entity not found: {}\t\t{:?}\t\t{:?}",
                piece_move,
                board.get_entity(piece_move.from),
                board.move_history.current_idx
            );
            panic!()
        };

        // Move Entity
        let mut transform = transform_query.get_mut(piece_entity).unwrap();

        let (x, y) = board_to_pixel_coords(piece_move.to.file, piece_move.to.rank);
        transform.translation = Vec3::new(x, y, 1.);

        // Only create a piece for a captured piece when undo-ing moves
        if ev.backwards {
            // Move piece before spawning new entities
            board.move_piece(piece_move.with_show(false));

            if let Some(captured_piece) = captured_piece {
                let captured_piece_tile = if piece_move.en_passant_capture {
                    // En passant capture
                    TilePos::new(piece_move_original.to.file, piece_move_original.from.rank)
                } else {
                    // Normal capture
                    piece_move_original.to
                };

                // Create new entity for the captured piece
                let captured_entity = commands.spawn(PieceBundle::new(
                    captured_piece_tile.into(),
                    captured_piece,
                    texture.clone(),
                    texture_atlas_layout.clone(),
                ));

                // Update the board to make it aware of the spawned piece
                board.set_piece(captured_piece_tile, captured_piece);
                board.set_entity(captured_piece_tile, Some(captured_entity.id()));
            }
        } else {
            // Need to delete captured pieces on redo
            if let Some(_captured_piece) = captured_piece {
                let captured_piece_tile = if piece_move.en_passant_capture {
                    // En passant capture

                    TilePos::new(piece_move.to.file, piece_move.from.rank)
                } else {
                    // Normal capture
                    piece_move.to
                };

                if let Some(captured_entity) = board.get_entity(captured_piece_tile) {
                    // Despawn the entity which was captured on this turn (Don't need to modify bitboards since board.move_piece will overwrite it anyway)
                    commands.entity(captured_entity).despawn();
                    board.set_entity(captured_piece_tile, None);
                }
            }

            // Move piece after deleting captured entities
            board.move_piece(piece_move.with_show(false));
        }

        // Change background colour to show current move
        board.next_player();
        background_ev.send(BackgroundColourEvent::new(board.get_player()));
    }
}
