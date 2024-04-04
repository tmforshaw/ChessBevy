// use bevy::prelude::*;

pub const PIECE_AMT: usize = 6;
pub const COLOUR_AMT: usize = 2;
pub const PIECE_WIDTH: f32 = 60.;
pub const PIECE_HEIGHT: f32 = 60.;

pub const PIECE_SCALE: f32 = 2.;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum PieceEnum {
    BQueen,
    BKing,
    BRook,
    BKnight,
    BBishop,
    BPawn,
    WQueen,
    WKing,
    WRook,
    WKnight,
    WBishop,
    WPawn,
    Empty,
}
