use std::fmt;

use bevy::prelude::*;

use crate::{
    bitboard::BitBoards,
    display::{board_to_pixel_coords, BOARD_SIZE, PIECE_SIZE},
    piece::{Piece, PieceMove, COLOUR_AMT, PIECES},
};

#[derive(Default, Clone, Copy, Debug)]
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
}

#[derive(Event, Debug)]
pub struct PossibleMoveDisplayEvent {
    pub from: TilePos,
    pub show: bool,
}

#[derive(Component)]
pub struct PossibleMoveMarker;

pub fn possible_move_event_handler(
    mut ev_display: EventReader<PossibleMoveDisplayEvent>,
    possible_move_entities: Query<Entity, With<PossibleMoveMarker>>,
    mut commands: Commands,
) {
    for ev in ev_display.read() {
        if ev.show {
            // TODO Get possible moves
            let positions = vec![TilePos::new(3, 3)];

            for pos in positions {
                let (x, y) = board_to_pixel_coords(pos.file, pos.rank);

                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgba(1., 0., 1., 0.75),
                            ..default()
                        },
                        transform: Transform::from_xyz(x, y, 2.)
                            .with_scale(Vec3::splat(PIECE_SIZE * 0.75)),
                        ..default()
                    },
                    PossibleMoveMarker,
                ));
            }
        } else {
            for entity in possible_move_entities.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}
