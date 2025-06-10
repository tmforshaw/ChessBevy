use std::{
    fmt::{self, Display},
    ops,
};

use crate::{
    board::TilePos,
    display::BOARD_SIZE,
    piece::{Piece, COLOUR_AMT, PIECES, PIECE_AMT},
};

#[derive(Copy, Clone, Default)]
pub struct BitBoard {
    bits: u64,
}

#[allow(dead_code)]
impl BitBoard {
    pub fn get_bit(&self, index: usize) -> bool {
        (self.bits >> index) & 1 == 1
    }

    pub fn get_bit_at(&self, tile_pos: TilePos) -> bool {
        (self.bits >> (tile_pos.file * BOARD_SIZE + tile_pos.rank)) & 1 == 1
    }

    pub fn set_bit(&mut self, index: usize, value: bool) {
        // Clear the bit, then set it
        self.bits &= !(1 << index);
        self.bits |= (value as u64) << index;
    }

    pub fn set_bit_at(&mut self, tile_pos: TilePos, value: bool) {
        self.set_bit(tile_pos.file * BOARD_SIZE + tile_pos.rank, value);
    }

    pub fn set_file(&mut self, file: usize, file_value: u8) {
        // Clear file, then set bits
        self.bits &= !(0xFF << (file * BOARD_SIZE));
        self.bits |= (file_value as u64) << (file * BOARD_SIZE);
    }

    pub fn set_rank(&mut self, rank: usize, rank_value: u8) {
        // Clear rank, then set each bit by spacing out the rank_value bits by (BOARD_SIZE - 1) many zeros
        self.bits &= !(0x0101010101010101 << rank);
        self.bits |= ((rank_value as u64) & 1) << rank
            | ((rank_value as u64) & (1 << 1)) << (BOARD_SIZE - 1 + rank)
            | ((rank_value as u64) & (1 << 2)) << (2 * (BOARD_SIZE - 1) + rank)
            | ((rank_value as u64) & (1 << 3)) << (3 * (BOARD_SIZE - 1) + rank)
            | ((rank_value as u64) & (1 << 4)) << (4 * (BOARD_SIZE - 1) + rank)
            | ((rank_value as u64) & (1 << 5)) << (5 * (BOARD_SIZE - 1) + rank)
            | ((rank_value as u64) & (1 << 6)) << (6 * (BOARD_SIZE - 1) + rank)
            | ((rank_value as u64) & (1 << 7)) << (7 * (BOARD_SIZE - 1) + rank);
    }
}

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut message = String::new();

        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                message += format!(
                    "{} ",
                    if (self.bits >> (i * BOARD_SIZE + j)) & 1 == 1 {
                        '#'
                    } else {
                        '-'
                    }
                )
                .as_str();
            }

            if i < BOARD_SIZE - 1 {
                message.push('\n');
            }
        }

        write!(f, "{message}")
    }
}

#[derive(Default, Clone)]
pub struct BitBoards {
    boards: [BitBoard; PIECE_AMT * COLOUR_AMT],
}

impl Display for BitBoards {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut message = String::new();

        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                let piece = {
                    let found_pieces = self
                        .boards
                        .iter()
                        .zip(PIECES)
                        .filter_map(|(board, &piece)| {
                            if (board.bits >> (i * BOARD_SIZE + j)) & 1 == 1 {
                                Some(piece)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    if found_pieces.is_empty() {
                        Piece::None
                    } else {
                        // Should only ever have one piece on each type
                        found_pieces[0]
                    }
                };

                let piece_char = Into::<char>::into(piece);

                message += format!("{} ", piece_char).as_str();
            }

            if i < BOARD_SIZE - 1 {
                message.push('\n');
            }
        }

        write!(f, "{message}")
    }
}

impl ops::Index<Piece> for BitBoards {
    type Output = BitBoard;

    fn index(&self, piece: Piece) -> &Self::Output {
        match piece {
            Piece::None => todo!(),
            _ => &self.boards[Into::<usize>::into(piece)],
        }
    }
}

impl ops::IndexMut<Piece> for BitBoards {
    fn index_mut(&mut self, piece: Piece) -> &mut Self::Output {
        match piece {
            Piece::None => todo!(),
            _ => &mut self.boards[Into::<usize>::into(piece)],
        }
    }
}
