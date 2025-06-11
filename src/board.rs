use std::fmt;

use bevy::prelude::*;

use crate::{
    bitboard::BitBoards,
    display::BOARD_SIZE,
    piece::{Piece, COLOUR_AMT, PIECES},
    piece_move::PieceMove,
};

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub enum Player {
    #[default]
    White,
    Black,
}

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct TilePos {
    pub file: usize,
    pub rank: usize,
}

impl TilePos {
    pub fn new(file: usize, rank: usize) -> Self {
        Self { file, rank }
    }
}

#[derive(Resource, Clone)]
pub struct Board {
    pub positions: BitBoards,
    pub player: Player,
    castling_rights: [(bool, bool); COLOUR_AMT],
    en_passant_on_last_move: Option<TilePos>,
    pub half_move_counter: usize,
    pub full_move_counter: usize,
    entities: [[Option<Entity>; BOARD_SIZE]; BOARD_SIZE],
}

impl Default for Board {
    fn default() -> Self {
        const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

        Board::from_fen(DEFAULT_FEN).unwrap()
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Current Player: {:?}\n{}\n", self.player, self.positions)
    }
}

impl Board {
    fn from_fen<T: AsRef<str>>(fen_string: T) -> Result<Self, String> {
        let fen = fen_string.as_ref();

        let mut section_index = 0;

        let mut rank = 0;
        let mut file = 0;

        let mut board = Board {
            // squares: [[Piece::None; BOARD_SIZE]; BOARD_SIZE],
            positions: BitBoards::default(),
            player: Player::default(),
            castling_rights: [(false, false); COLOUR_AMT],
            en_passant_on_last_move: None,
            half_move_counter: 0,
            full_move_counter: 1,
            entities: [[None; BOARD_SIZE]; BOARD_SIZE],
        };

        for (chr_index, chr) in fen.char_indices() {
            match section_index {
                // Read positions from FEN
                0 => match chr {
                    '/' => {
                        file += 1;
                        rank = 0;
                    }
                    '1'..='8' => rank += (chr as u8 - b'0') as usize,
                    ' ' => section_index += 1,
                    _ => {
                        if let Some(piece) = Piece::from_algebraic(chr) {
                            let tile_pos = TilePos::new(BOARD_SIZE - 1 - file, rank);
                            board.set_piece(tile_pos, piece);
                            board.positions[piece].set_bit_at(tile_pos, true);

                            rank += 1;
                        } else {
                            return Err(format!("Could not create board using FEN string [{fen}]:\n'{chr}' is not algebraic notation for any piece"));
                        }
                    }
                },
                // Read the current player's turn from FEN
                1 => match chr {
                    'w' => board.player = Player::White,
                    'b' => board.player = Player::Black,
                    ' ' => section_index += 1,
                    _ => {
                        return Err(format!("Could not create board using FEN string [{fen}]:\n'{chr}' is not a valid player"));
                    }
                },
                // Read the castling rights from FEN
                2 => match chr {
                    'K' => board.castling_rights[Player::White as usize].0 = true,
                    'Q' => board.castling_rights[Player::White as usize].1 = true,
                    'k' => board.castling_rights[Player::Black as usize].0 = true,
                    'q' => board.castling_rights[Player::Black as usize].1 = true,
                    '-' => board.castling_rights = [(false, false); COLOUR_AMT],
                    ' ' => section_index += 1,
                    _ => {
                        return Err(format!("Could not create board using FEN string [{fen}]:\n'{chr}' does not provide valid castling rights information"));
                    }
                },
                // Reached the en passant part of FEN
                3 => match chr {
                    '-' => board.en_passant_on_last_move = None,
                    ' ' => section_index += 1,
                    _ => {
                        let algebraic_en_passant =
                            fen.chars().skip(chr_index - 1).take(2).collect::<Vec<_>>();

                        match (algebraic_en_passant[0], algebraic_en_passant[1]) {
                            ('a'..='h', '0'..='8') => {
                                board.en_passant_on_last_move = Some(TilePos::new(
                                    (algebraic_en_passant[0] as u8 - b'a') as usize,
                                    (algebraic_en_passant[1] as u8 - b'0') as usize,
                                ));
                            }
                            _ => {
                                return Err(format!("Could not create board using FEN string [{fen}]:\n\"{}{}\" is not a valid en passant square", algebraic_en_passant[0], algebraic_en_passant[1]));
                            }
                        }
                    }
                },
                _ => break,
            }
        }

        Ok(board)
    }

    pub fn move_piece(&mut self, piece_move: PieceMove) {
        let moved_piece = self.get_piece(piece_move.from);
        self.set_piece(piece_move.from, Piece::None);
        self.set_piece(piece_move.to, moved_piece);

        let moved_entity = self.get_entity(piece_move.from);
        self.set_entity(piece_move.from, None);
        self.set_entity(piece_move.to, moved_entity);
    }

    pub fn get_piece(&self, tile_pos: TilePos) -> Piece {
        for &piece in PIECES {
            if self.positions[piece].get_bit_at(tile_pos) {
                return piece;
            }
        }

        Piece::None
    }

    pub fn set_piece(&mut self, tile_pos: TilePos, piece: Piece) {
        // Clear all the other bitboards at this position, except this piece's position bitboard
        for &piece_i in PIECES {
            if piece_i == piece {
                self.positions[piece_i].set_bit_at(tile_pos, true);
            } else {
                self.positions[piece_i].set_bit_at(tile_pos, false);
            }
        }
    }

    pub fn get_entity(&self, tile_pos: TilePos) -> Option<Entity> {
        self.entities[tile_pos.file][tile_pos.rank]
    }

    pub fn set_entity(&mut self, tile_pos: TilePos, entity: Option<Entity>) {
        self.entities[tile_pos.file][tile_pos.rank] = entity;
    }

    pub fn get_player(&self) -> Player {
        self.player
    }

    pub fn get_next_player(&self) -> Player {
        match self.player {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }

    pub fn next_player(&mut self) {
        self.player = self.get_next_player();
    }

    fn get_moves_in_dir(&self, from: TilePos, dirs: Vec<(isize, isize)>) -> Vec<TilePos> {
        let mut positions = Vec::new();

        for dir in dirs.into_iter() {
            for k in 1..(BOARD_SIZE as isize) {
                let new_file = from.file as isize + dir.0 * k;
                let new_rank = from.rank as isize + dir.1 * k;

                // New pos is within the board
                if new_file >= 0
                    && new_file < BOARD_SIZE as isize
                    && new_rank >= 0
                    && new_rank < BOARD_SIZE as isize
                {
                    let new_pos = TilePos::new(new_file as usize, new_rank as usize);

                    let piece = self.get_piece(from);
                    let captured_piece = self.get_piece(new_pos);
                    if captured_piece != Piece::None {
                        if captured_piece.to_player() != piece.to_player() {
                            positions.push(new_pos);
                        }

                        break;
                    } else {
                        positions.push(new_pos);
                    }
                }
            }
        }

        positions
    }

    pub fn get_orthogonal_moves(&self, from: TilePos) -> Vec<TilePos> {
        self.get_moves_in_dir(from, vec![(1, 0), (0, 1), (-1, 0), (0, -1)])
    }

    pub fn get_diagonal_moves(&self, from: TilePos) -> Vec<TilePos> {
        self.get_moves_in_dir(from, vec![(1, 1), (1, -1), (-1, 1), (-1, -1)])
    }

    pub fn get_ortho_diagonal_moves(&self, from: TilePos) -> Vec<TilePos> {
        let mut positions = self.get_orthogonal_moves(from);
        positions.append(&mut self.get_diagonal_moves(from));

        positions
    }

    pub fn get_knight_moves(&self, from: TilePos) -> Vec<TilePos> {
        let mut positions = Vec::new();

        for i in [-2, -1, 1, 2_isize] {
            for j in [-2, -1, 1, 2_isize] {
                if i.abs() != j.abs()
                    && from.file as isize + i >= 0
                    && from.file as isize + i < BOARD_SIZE as isize
                    && from.rank as isize + j >= 0
                    && from.rank as isize + j < BOARD_SIZE as isize
                {
                    let new_pos = TilePos::new(
                        (from.file as isize + i) as usize,
                        (from.rank as isize + j) as usize,
                    );

                    let captured_piece = self.get_piece(new_pos);
                    if captured_piece.to_player() != self.get_piece(from).to_player()
                        || captured_piece == Piece::None
                    {
                        positions.push(new_pos);
                    }
                }
            }
        }

        positions
    }

    pub fn get_king_moves(&self, from: TilePos) -> Vec<TilePos> {
        let mut positions = Vec::new();

        for i in [-1, 0, 1] {
            for j in [-1, 0, 1] {
                if !(i == 0 && j == 0) {
                    let vertical = from.file as isize + i;
                    let horizontal = from.rank as isize + j;

                    if vertical >= 0
                        && vertical < BOARD_SIZE as isize
                        && horizontal >= 0
                        && horizontal < BOARD_SIZE as isize
                    {
                        let new_pos = TilePos::new(
                            (from.file as isize + i) as usize,
                            (from.rank as isize + j) as usize,
                        );

                        if self.get_piece(new_pos).to_player() != self.get_piece(from).to_player() {
                            positions.push(new_pos)
                        }
                    }
                }
            }
        }

        positions
    }

    pub fn get_pawn_moves(&self, from: TilePos) -> Vec<TilePos> {
        let piece = self.get_piece(from);
        let vertical_dir = piece.is_white() as isize * 2 - 1;

        let mut positions = Vec::new();

        // Single Move Vertically and Diagonal Captures
        let new_vertical_pos = from.file as isize + vertical_dir;
        if new_vertical_pos > 0 && new_vertical_pos < BOARD_SIZE as isize {
            // Single Move Vertically
            let new_pos = TilePos::new((from.file as isize + vertical_dir) as usize, from.rank);
            if self.get_piece(new_pos) == Piece::None {
                positions.push(new_pos);
            }

            // Diagonal Captures
            for k in [-1, 1] {
                let new_horizontal_pos = from.rank as isize + k;

                let new_pos = TilePos::new(new_vertical_pos as usize, new_horizontal_pos as usize);
                if new_horizontal_pos > 0 && new_horizontal_pos < BOARD_SIZE as isize {
                    if let Some(player) = piece.to_player() {
                        if let Some(captured_player) = self.get_piece(new_pos).to_player() {
                            if player != captured_player {
                                positions.push(new_pos);
                            }
                        }
                    }
                }
            }
        }

        // Double Vertical Move
        if (piece.is_white() && from.file == 1) || (piece.is_black() && from.file == BOARD_SIZE - 2)
        {
            let new_pos = TilePos::new((from.file as isize + 2 * vertical_dir) as usize, from.rank);
            if self.get_piece(new_pos) == Piece::None {
                positions.push(new_pos);
            }
        }
        positions
    }
}
