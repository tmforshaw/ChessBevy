use std::fmt;

use bevy::prelude::*;

use crate::{
    board::{Board, Player},
    display::{board_to_pixel_coords, BackgroundColourEvent, PIECE_SIZE_IMG, PIECE_TEXTURE_FILE},
    piece::{Piece, PieceBundle, COLOUR_AMT, PIECE_AMT},
    piece_move::PieceMove,
};

#[derive(Default, Clone, Debug)]
pub struct PieceMoveHistory {
    moves: Vec<(PieceMove, Option<Piece>)>,
    current_idx: Option<usize>,
}

impl PieceMoveHistory {
    #[must_use]
    pub const fn new(moves: Vec<(PieceMove, Option<Piece>)>, current_idx: Option<usize>) -> Self {
        Self { moves, current_idx }
    }

    pub fn make_move(&mut self, piece_move: PieceMove, captured_piece: Option<Piece>) {
        if piece_move.show {
            // Clear depending on where current_idx is
            if let Some(current_idx) = self.current_idx {
                if current_idx + 1 < self.moves.len() {
                    self.clear_excess_moves();
                }
            }

            self.current_idx = self
                .current_idx
                .map_or(Some(0), |current_idx| Some(current_idx + 1));
            self.moves.push((piece_move, captured_piece));
        }
    }

    pub const fn increment_index(&mut self) {
        self.current_idx = Some(match self.current_idx {
            Some(idx) if idx < self.moves.len() => idx + 1,
            Some(idx) => idx,
            None => 0,
        });
    }

    pub fn clear_excess_moves(&mut self) {
        self.moves = self.moves[0..=self.current_idx.unwrap_or(0)].to_vec();
        self.current_idx = (!self.moves.is_empty()).then_some(self.moves.len() - 1);
    }

    #[must_use]
    pub fn get(&self) -> Option<(PieceMove, Option<Piece>)> {
        if self.moves.is_empty() {
            None
        } else {
            Some(self.moves[self.current_idx.unwrap_or(0)])
        }
    }

    pub fn get_mut(&mut self) -> Option<(&mut PieceMove, &mut Option<Piece>)> {
        if self.moves.is_empty() {
            None
        } else {
            let history_move = &mut self.moves[self.current_idx.unwrap_or(0)];
            Some((&mut history_move.0, &mut history_move.1))
        }
    }

    pub fn traverse_next(&mut self) -> Option<(PieceMove, Option<Piece>)> {
        if let Some(current_idx) = self.current_idx {
            if current_idx + 1 < self.moves.len() {
                self.current_idx = Some((current_idx + 1).min(self.moves.len() - 1));
                self.current_idx
                    .map(|_| self.moves[self.current_idx.unwrap_or(0)])
            } else {
                None
            }
        } else {
            self.current_idx = Some(0);
            Some(self.moves[0])
        }
    }

    pub fn traverse_prev(&mut self) -> Option<(PieceMove, Option<Piece>)> {
        if let Some(current_idx) = self.current_idx {
            let piece_move = Some(self.moves[self.current_idx.unwrap_or(0)]);

            self.current_idx = if current_idx > 0 {
                Some(current_idx - 1)
            } else {
                None
            };

            piece_move
        } else {
            None
        }
    }

    pub fn peek_prev(&self) -> Option<(PieceMove, Option<Piece>)> {
        if let Some(current_idx) = self.current_idx {
            if current_idx > 0 {
                return Some(self.moves[current_idx - 1]);
            }
        }

        None
    }
}

impl fmt::Display for PieceMoveHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut message = String::new();

        for piece_move in &self.moves {
            let Ok(algebraic) = piece_move.0.to_algebraic() else {
                return Err(fmt::Error);
            };

            message += format!("{algebraic}    ").as_str();
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
    // TODO THIS CODE IS REPEATED CODE, MOVE INTO FUNCTION
    // Texture atlas for all the pieces
    let texture = asset_server.load(PIECE_TEXTURE_FILE);
    let texture_atlas_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        Vec2::new(PIECE_SIZE_IMG, PIECE_SIZE_IMG),
        PIECE_AMT,
        COLOUR_AMT,
        None,
        None,
    ));

    for ev in move_history_ev.read() {
        if let Some((mut piece_move, _)) = board.move_history.get() {
            let traversal_succeeded = if ev.backwards {
                if let Some((history_move, _)) = board.move_history.traverse_prev() {
                    piece_move = history_move.rev();

                    true
                } else {
                    board.move_history.moves.is_empty()
                }
            } else if let Some((history_move, _)) = board.move_history.traverse_next() {
                piece_move = history_move;

                true
            } else {
                false
            };

            let piece_move_original = if ev.backwards {
                piece_move.rev()
            } else {
                piece_move
            };

            if traversal_succeeded {
                // TODO This is duplicated code
                // Check if there is a piece at the new location
                if !ev.backwards {
                    let piece_moved_to = board.get_piece(piece_move.to);
                    if piece_moved_to != Piece::None {
                        let moved_to = board.get_entity(piece_move.to).unwrap();

                        commands.entity(moved_to).despawn();
                    }
                }

                let Some(entity) = board.get_entity(piece_move.from) else {
                    eprintln!(
                        "Entity not found: {}\t\t{:?}\t\t{:?}",
                        piece_move,
                        board.get_entity(piece_move.from),
                        board.move_history.current_idx
                    );
                    panic!()
                };

                // Move Entity
                let mut transform = transform_query.get_mut(entity).unwrap();

                let (x, y) = board_to_pixel_coords(piece_move.to.file, piece_move.to.rank);
                transform.translation = Vec3::new(x, y, 1.);
                board.move_piece(piece_move.with_show(false));

                // Create a piece for captured pieces which were taken on this move
                if ev.backwards {
                    if let Some((_, Some(piece_to_spawn))) = board.move_history.get() {
                        let entity = commands.spawn(PieceBundle::new(
                            piece_move_original.to.into(),
                            piece_to_spawn,
                            texture.clone(),
                            texture_atlas_layout.clone(),
                        ));

                        // println!("{:?}", board.get_entity(piece_move.from));

                        println!("{}\n", board.positions);
                        board.set_piece(piece_move.from, piece_to_spawn);
                        board.set_entity(piece_move.from, Some(entity.id()));
                        println!("{}\n\n", board.positions);
                    }
                }

                // Change background colour to show current move
                board.next_player();
                background_ev.send(BackgroundColourEvent::new(match board.get_player() {
                    Player::White => Color::rgb(1., 1., 1.),
                    Player::Black => Color::rgb(0., 0., 0.),
                }));
            }
        }
    }
}
