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
        if !ev.piece_move.show {
            board.move_piece(ev.piece_move);
            continue;
        }
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
                    println!("got piece_captured");
                    board.get_piece(ev.piece_move.to)
                } else {
                    Piece::None
                };

                board
                    .move_history
                    .make_move(ev.piece_move, piece_captured.then_some(piece_moved_to));

                // Change background colour to show current move
                background_ev.send(BackgroundColourEvent::new(match board.get_player() {
                    Player::White => Color::rgb(1., 1., 1.),
                    Player::Black => Color::rgb(0., 0., 0.),
                }));

                println!("{}", board.move_history);
            }
            board.next_player();
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
            if self.current_idx != self.moves.len().checked_sub(1) && self.current_idx.is_some()
            // && self.current_idx < self.moves.len()
            {
                self.moves = self.moves[0..self.current_idx.unwrap()].to_vec();
            }

            self.current_idx = if let Some(current_idx) = self.current_idx {
                Some(current_idx + 1)
            } else {
                Some(0)
            };
            self.moves.push((piece_move, captured_piece));
        }
    }

    pub fn get(&self) -> Option<(PieceMove, Option<Piece>)> {
        self.current_idx.map(|current_idx| self.moves[current_idx])
    }

    pub fn traverse_next(&mut self) -> Option<(PieceMove, Option<Piece>)> {
        self.traverse(true)
    }

    pub fn traverse_prev(&mut self) -> Option<(PieceMove, Option<Piece>)> {
        self.traverse(false)
    }

    fn traverse(&mut self, next: bool) -> Option<(PieceMove, Option<Piece>)> {
        if let Some(current_idx) = self.current_idx {
            // if (next && current_idx < self.moves.len() - 1) || (!next && current_idx > 0) {
            self.current_idx = if next {
                Some(current_idx + 1)
            } else if current_idx > 0 {
                Some(current_idx - 1)
            } else {
                None
            };

            // if current_idx == 0 && !next {
            //     return Some(self.moves[0]);
            // }

            return self.current_idx.map(|current_idx| self.moves[current_idx]);
            // }
        } else if next {
            // TODO
            eprintln!("AHHHHHH");
            self.current_idx = Some(0);
            return Some(self.moves[0]);
        } else {
            eprintln!("AHHHHHH");
        }

        None
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
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // // TODO THIS CODE IS REPEATED CODE, MOVE INTO FUNCTION
    // // Texture atlas for all the pieces
    // let texture = asset_server.load(PIECE_TEXTURE_FILE);
    // let texture_atlas_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
    //     Vec2::new(PIECE_SIZE_IMG, PIECE_SIZE_IMG),
    //     PIECE_AMT,
    //     COLOUR_AMT,
    //     None,
    //     None,
    // ));

    for ev in move_history_ev.read() {
        // TODO Unwrap
        if let Some((mut piece_move, captured_piece)) = board.move_history.get() {
            // // if captured_piece.is_some() {
            // let piece_entity = match board.get_entity(piece_move.from) {
            //     Some(entity) => entity,
            //     None => {
            //         if captured_piece.is_some() {
            //             eprintln!("Could not find entity from board");
            //             eprintln!(
            //                 "{:?}\t {:?}\t\t {:?}\t {:?}",
            //                 board.get_piece(piece_move.from),
            //                 piece_move.from,
            //                 board.get_piece(piece_move.to),
            //                 piece_move.to,
            //             );

            //             let entity = commands.spawn(PieceBundle::new(
            //                 (piece_move.to.file, piece_move.to.rank),
            //                 board.get_piece(TilePos::new(piece_move.to.file, piece_move.to.rank)),
            //                 texture.clone(),
            //                 texture_atlas_layout.clone(),
            //             ));

            //             board.set_entity(
            //                 TilePos::new(piece_move.from.file, piece_move.from.rank),
            //                 Some(entity.id()),
            //             );

            //             entity.id()
            //         } else {
            //             todo!()
            //         }
            //     }
            // };

            let traversal_succeeded = if ev.backwards {
                piece_move = piece_move.rev();
                board.move_history.traverse_prev().is_some()
            } else {
                board.move_history.traverse_next().is_some()
            };

            println!("{}", board.move_history);

            // Move Entity
            let mut transform = transform_query
                .get_mut(board.get_entity(piece_move.from).unwrap())
                .unwrap();

            let (x, y) = board_to_pixel_coords(piece_move.to.file, piece_move.to.rank);
            transform.translation = Vec3::new(x, y, 1.);
            board.move_piece(piece_move.with_show(false));
        }
    }
    // }
}
