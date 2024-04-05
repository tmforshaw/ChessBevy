use crate::piece::{PieceEnum, PIECE_HEIGHT, PIECE_SCALE, PIECE_WIDTH};

use bevy::prelude::*;

pub const BOARD_WIDTH: usize = 8;
pub const BOARD_HEIGHT: usize = 8;
pub const BOARD_SPACING: (f32, f32) = (4., 4.);

pub fn board_to_pixel_coords(i: usize, j: usize) -> (f32, f32) {
    (
        (j as f32 - BOARD_WIDTH as f32 / 2. + 0.5) * (PIECE_WIDTH * PIECE_SCALE + BOARD_SPACING.0),
        (i as f32 - BOARD_HEIGHT as f32 / 2. + 0.5)
            * (PIECE_HEIGHT * PIECE_SCALE + BOARD_SPACING.1),
    )
}

pub fn pixel_to_board_coords(x: f32, y: f32) -> (usize, usize) {
    (
        ((y / (PIECE_HEIGHT * PIECE_SCALE + BOARD_SPACING.1)) - 0.5 + BOARD_HEIGHT as f32 / 2.)
            as usize,
        ((x / (PIECE_WIDTH * PIECE_SCALE + BOARD_SPACING.0)) - 0.5 + BOARD_WIDTH as f32 / 2.)
            as usize,
    )
}

#[derive(Resource, Clone, Copy)]
pub struct Board {
    pub tiles: [[PieceEnum; BOARD_WIDTH]; BOARD_HEIGHT],
}

impl Default for Board {
    fn default() -> Self {
        let mut tiles = [[PieceEnum::Empty; BOARD_WIDTH]; BOARD_HEIGHT];

        tiles[0][0] = PieceEnum::WRook;
        tiles[0][1] = PieceEnum::WKnight;
        tiles[0][2] = PieceEnum::WBishop;
        tiles[0][3] = PieceEnum::WQueen;
        tiles[0][4] = PieceEnum::WKing;
        tiles[0][5] = PieceEnum::WBishop;
        tiles[0][6] = PieceEnum::WKnight;
        tiles[0][7] = PieceEnum::WRook;

        for i in 0..BOARD_WIDTH {
            tiles[1][i] = PieceEnum::WPawn;
            tiles[BOARD_HEIGHT - 2][i] = PieceEnum::BPawn;
        }

        tiles[BOARD_HEIGHT - 1][0] = PieceEnum::BRook;
        tiles[BOARD_HEIGHT - 1][1] = PieceEnum::BKnight;
        tiles[BOARD_HEIGHT - 1][2] = PieceEnum::BBishop;
        tiles[BOARD_HEIGHT - 1][3] = PieceEnum::BQueen;
        tiles[BOARD_HEIGHT - 1][4] = PieceEnum::BKing;
        tiles[BOARD_HEIGHT - 1][5] = PieceEnum::BBishop;
        tiles[BOARD_HEIGHT - 1][6] = PieceEnum::BKnight;
        tiles[BOARD_HEIGHT - 1][7] = PieceEnum::BRook;

        Board { tiles }
    }
}

// This just helps with debugging, seeing the internal state of the board
impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut message = String::new();
        for i in (0..self.tiles.len()).rev() {
            for j in 0..self.tiles[i].len() {
                message.push_str(
                    format!(
                        "{} ",
                        match self.tiles[i][j] {
                            PieceEnum::Empty => "*",
                            PieceEnum::BQueen => "q",
                            PieceEnum::BKing => "k",
                            PieceEnum::BKnight => "n",
                            PieceEnum::BBishop => "b",
                            PieceEnum::BRook => "r",
                            PieceEnum::BPawn => "p",
                            PieceEnum::WQueen => "Q",
                            PieceEnum::WKing => "K",
                            PieceEnum::WKnight => "N",
                            PieceEnum::WBishop => "B",
                            PieceEnum::WRook => "R",
                            PieceEnum::WPawn => "P",
                        }
                    )
                    .as_str(),
                );
            }

            message.push('\n');
        }

        write!(f, "{message}")
    }
}
