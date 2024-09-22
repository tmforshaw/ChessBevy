use bevy::prelude::*;

use crate::{
    bitboard::BitBoards,
    display::BOARD_SIZE,
    piece::{Piece, COLOUR_AMT, PIECE_AMT},
};

#[derive(Default)]
pub enum Player {
    #[default]
    White,
    Black,
}

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct TilePos {
    pub file: usize,
    pub rank: usize,
}

impl TilePos {
    pub fn new(file: usize, rank: usize) -> Self {
        Self { file, rank }
    }
}

#[derive(Resource)]
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
                            let tile_pos = TilePos::new(file, rank);
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

    pub fn get_piece(&self, tile_pos: TilePos) -> Piece {
        for i in 0..(PIECE_AMT * COLOUR_AMT) {
            if self.positions[Into::<Piece>::into(i)].get_bit_at(tile_pos) {
                return Into::<Piece>::into(i);
            }
        }

        Piece::None
    }

    pub fn set_piece(&mut self, tile_pos: TilePos, piece: Piece) {
        // Clear all the other bitboards at this position, except this piece's position bitboard
        for i in 0..(PIECE_AMT * COLOUR_AMT) {
            let piece_i = Into::<Piece>::into(i);
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
}
