use std::fmt;

use bevy::prelude::*;

use crate::{
    board::{Board, Player, TilePos},
    display::{board_to_pixel_coords, BackgroundColourEvent},
    piece::Piece,
    possible_moves::get_possible_moves,
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
        let move_complete;
        {
            let mut transform = transform_query.get_mut(ev.entity).unwrap();

            let moved_to = board.get_piece(ev.piece_move.to);

            // Snap the moved entity to the grid (Don't move if there is a non-opponent piece there, or if you moved a piece on another player's turn, or if the move is impossible for that piece type)
            let (x, y) = if !moved_to.is_player(board.player)
                && board.get_piece(ev.piece_move.from).is_player(board.player)
                && get_possible_moves(board.clone(), ev.piece_move.from).contains(&ev.piece_move.to)
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
            if ev.piece_move.show {
                board.move_history.make_move(ev.piece_move);
                board.next_player();

                // Change background colour to show current move
                background_ev.send(BackgroundColourEvent::new(match board.get_player() {
                    Player::White => Color::rgb(1., 1., 1.),
                    Player::Black => Color::rgb(0., 0., 0.),
                }));

                println!("{}", board.move_history);
            }

            // println!("{}", board.clone());
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct PieceMoveHistory {
    moves: Vec<PieceMove>,
    current_idx: usize,
}

impl PieceMoveHistory {
    pub fn new(moves: Vec<PieceMove>, current_idx: usize) -> Self {
        Self { moves, current_idx }
    }

    pub fn make_move(&mut self, piece_move: PieceMove) {
        // TODO Clear depending on if move matches history (when current_idx was not at final part of history)
        if piece_move.show {
            // Clear depending on where current_idx is
            if Some(self.current_idx) != self.moves.len().checked_sub(1) && self.current_idx > 0
            // && self.current_idx < self.moves.len()
            {
                self.moves = self.moves[0..self.current_idx].to_vec();
            }

            self.current_idx += 1;
            self.moves.push(piece_move);
        }
    }

    pub fn get_move(&self) -> PieceMove {
        self.moves[self.current_idx.saturating_sub(1)]
    }

    pub fn traverse_next(&mut self) -> Option<PieceMove> {
        self.traverse(true)
    }

    pub fn traverse_prev(&mut self) -> Option<PieceMove> {
        self.traverse(false)
    }

    fn traverse(&mut self, next: bool) -> Option<PieceMove> {
        if (next && self.current_idx < self.moves.len() - 1) || (!next && self.current_idx > 0) {
            if next {
                self.current_idx += 1
            } else {
                self.current_idx -= 1;
            };

            Some(self.moves[self.current_idx])
        } else {
            None
        }
    }
}

impl fmt::Display for PieceMoveHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut message = String::new();

        for piece_move in self.moves.iter() {
            message += format!("{}    ", piece_move.to_algebraic()).as_str();
        }

        message += format!("\t\tCurrent Index: {}", self.current_idx).as_str();

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
) {
    for ev in move_history_ev.read() {
        let mut piece_move = board.move_history.get_move();
        let mut transform = transform_query
            .get_mut(board.get_entity(piece_move.to).unwrap())
            .unwrap();

        if ev.backwards {
            piece_move = piece_move.rev();
            board.move_history.traverse_prev();
        } else {
            board.move_history.traverse_next();
        }

        println!("{}", board.move_history);

        // Move Entity
        if board.move_history.current_idx < board.move_history.moves.len() {
            let (x, y) = board_to_pixel_coords(piece_move.to.file, piece_move.to.rank);
            transform.translation = Vec3::new(x, y, 1.);
            board.move_piece(piece_move.with_show(false));
        }
    }
}
