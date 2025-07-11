use std::fmt;

use bevy::prelude::*;

use crate::{
    board::{Board, Player, TilePos},
    display::{board_to_pixel_coords, BackgroundColourEvent, PIECE_SIZE_IMG, PIECE_TEXTURE_FILE},
    piece::{Piece, PieceBundle, COLOUR_AMT, PIECE_AMT},
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
    pub show: bool,
}

impl PieceMove {
    pub fn new(from: TilePos, to: TilePos) -> Self {
        Self {
            from,
            to,
            show: true,
        }
    }

    pub fn new_unshown(from: TilePos, to: TilePos) -> Self {
        Self {
            from,
            to,
            show: false,
        }
    }

    pub fn with_show(&self, show: bool) -> Self {
        Self {
            from: self.from,
            to: self.to,
            show,
        }
    }

    pub fn to_algebraic(&self) -> String {
        format!("{} {}", self.from.to_algebraic(), self.to.to_algebraic())
    }

    pub fn rev(&self) -> Self {
        Self {
            from: self.to,
            to: self.from,
            show: self.show,
        }
    }
}

impl std::fmt::Debug for PieceMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{from: {}, to: {}, show: {}}}",
            self.from, self.to, self.show
        )
    }
}

impl std::fmt::Display for PieceMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{{}, {}, {}}}", self.from, self.to, self.show)
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
        // if !ev.piece_move.show {
        //     board.move_piece(ev.piece_move);

        //     continue;
        // }

        // Entity Logic
        let mut piece_captured = false;
        let move_complete;
        {
            let mut transform = transform_query.get_mut(ev.entity).unwrap();

            let moved_to = board.get_piece(ev.piece_move.to);

            // Snap the moved entity to the grid (Don't move if there is a non-opponent piece there, or if you moved a piece on another player's turn, or if the move is impossible for that piece type)
            let (x, y) = if !moved_to.is_player(board.player)
                && board.get_piece(ev.piece_move.from).is_player(board.player)
                && get_possible_moves(board.clone(), ev.piece_move.from).contains(&ev.piece_move.to)
            {
                // Need to capture
                if moved_to != Piece::None {
                    piece_captured = true;
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
        if (move_complete && ev.piece_move.show) || (!move_complete && !ev.piece_move.show) {
            if ev.piece_move.show {
                let piece_moved_to = if piece_captured {
                    board.get_piece(ev.piece_move.to)
                } else {
                    Piece::None
                };

                // TODO could check for if move matches history here
                let same_as_history_move = if let Some((history_move, _)) = board.move_history.get()
                {
                    // Made Different Move to history
                    history_move == ev.piece_move
                } else {
                    false
                };

                // println!(
                //     "{:?}\t\t{:?}",
                //     board.move_history.get(),
                //     piece_captured.then_some(piece_moved_to),
                // );

                if !same_as_history_move {
                    if let Some((_, captured_piece)) = board.move_history.get_mut() {
                        if piece_captured {
                            captured_piece.replace(piece_moved_to);
                        } else {
                            captured_piece.take();
                        }
                    }

                    board.move_history.make_move(ev.piece_move, None);
                } else {
                    board.move_history.increment_index();
                }

                // Change background colour to show current move
                board.next_player();
                background_ev.send(BackgroundColourEvent::new(match board.get_player() {
                    Player::White => Color::rgb(1., 1., 1.),
                    Player::Black => Color::rgb(0., 0., 0.),
                }));

                // println!("{}", board.move_history);
            }
            board.move_piece(ev.piece_move);

            // println!("{}", board.clone());
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct PieceMoveHistory {
    moves: Vec<(PieceMove, Option<Piece>)>,
    current_idx: Option<usize>,
}

impl PieceMoveHistory {
    pub fn new(moves: Vec<(PieceMove, Option<Piece>)>, current_idx: Option<usize>) -> Self {
        Self { moves, current_idx }
    }

    pub fn make_move(&mut self, piece_move: PieceMove, captured_piece: Option<Piece>) {
        // TODO Clear depending on if move matches history (when current_idx was not at final part of history)
        if piece_move.show {
            // Clear depending on where current_idx is
            if let Some(current_idx) = self.current_idx {
                if current_idx + 1 < self.moves.len() {
                    self.clear_excess_moves();
                }
            }

            self.current_idx = if let Some(current_idx) = self.current_idx {
                Some(current_idx + 1)
            } else {
                Some(0)
            };
            self.moves.push((piece_move, captured_piece));
        }
    }

    pub fn increment_index(&mut self) {
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

    pub fn get(&self) -> Option<(PieceMove, Option<Piece>)> {
        if !self.moves.is_empty() {
            Some(self.moves[self.current_idx.unwrap_or(0)])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self) -> Option<(&mut PieceMove, &mut Option<Piece>)> {
        if !self.moves.is_empty() {
            let history_move = &mut self.moves[self.current_idx.unwrap_or(0)];
            Some((&mut history_move.0, &mut history_move.1))
        } else {
            None
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
}

impl fmt::Display for PieceMoveHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut message = String::new();

        for piece_move in self.moves.iter() {
            message += format!("{}    ", piece_move.0.to_algebraic()).as_str();
        }

        message += format!("\t\tCurrent Index: {:?}", self.current_idx).as_str();

        write!(f, "{message}")
    }
}

#[derive(Event)]
pub struct MoveHistoryEvent {
    pub backwards: bool,
}

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

                let entity = match board.get_entity(piece_move.from) {
                    Some(entity) => entity,
                    None => {
                        eprintln!(
                            "Entity not found: {}\t\t{:?}\t\t{:?}",
                            piece_move,
                            board.get_entity(piece_move.from),
                            board.move_history.current_idx
                        );
                        panic!()
                    }
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
