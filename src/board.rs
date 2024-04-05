use crate::piece::{
    piece_is_black, piece_is_white, Piece, PieceEnum, COLOUR_AMT, PIECE_AMT, PIECE_HEIGHT,
    PIECE_SCALE, PIECE_WIDTH,
};

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
    pub texture_file: &'static str,
    pub pieces_and_positions: [[Option<Entity>; BOARD_WIDTH]; BOARD_HEIGHT],
}

impl Board {
    pub fn setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
        mut board: ResMut<Board>,
    ) {
        // Spawn Board Squares
        for i in 0..board.tiles.len() {
            for j in 0..board.tiles[i].len() {
                let (x, y) = board_to_pixel_coords(i, j);

                commands.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: if (i + j) % 2 == 0 {
                            Color::WHITE
                        } else {
                            Color::PURPLE
                        },
                        custom_size: Some(Vec2::new(PIECE_WIDTH, PIECE_HEIGHT)),
                        ..default()
                    },
                    transform: Transform::from_scale(Vec3::splat(PIECE_SCALE))
                        .with_translation(Vec3::new(x, y, 0.)),
                    ..default()
                });
            }
        }

        // Texture atlas for different pieces
        let texture = asset_server.load(board.texture_file);
        let texture_atlas_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            Vec2::new(PIECE_WIDTH, PIECE_HEIGHT),
            PIECE_AMT,
            COLOUR_AMT,
            None,
            None,
        ));

        // Spawn all the pieces in their respective locations
        for i in 0..board.tiles.len() {
            for j in 0..board.tiles[i].len() {
                if PieceEnum::Empty as usize != board.tiles[i][j] as usize {
                    let entity = commands.spawn((Piece::new(
                        (i, j),
                        board.tiles[i][j],
                        texture.clone(),
                        texture_atlas_layout.clone(),
                    ),));

                    board.pieces_and_positions[i][j] = Some(entity.id());
                }
            }
        }
    }

    pub fn move_piece(
        &mut self,
        original_position: (usize, usize),
        new_position: (usize, usize),
        moved_piece_entity: Entity,
        commands: &mut Commands,
    ) {
        let moved_piece = self.tiles[original_position.0][original_position.1];

        // If the square being moved to and the piece are different colours
        if (piece_is_white(self.tiles[new_position.0][new_position.1])
            && piece_is_black(moved_piece))
            || (piece_is_black(self.tiles[new_position.0][new_position.1])
                && piece_is_white(moved_piece))
        {
            if let Some(entity) = self.pieces_and_positions[new_position.0][new_position.1] {
                commands.entity(entity).despawn();
            }

            println!(
                "Moved: {moved_piece:?}, Onto: {:?}",
                self.tiles[new_position.0][new_position.1]
            );
        }

        self.tiles[original_position.0][original_position.1] = PieceEnum::Empty;
        self.tiles[new_position.0][new_position.1] = moved_piece;
        self.pieces_and_positions[original_position.0][original_position.1] = None;
        self.pieces_and_positions[new_position.0][new_position.1] = Some(moved_piece_entity);
        // else {
        //    transform.translation = Vec3::new(original_pos.x, original_pos.y, 1.);
        // }
    }
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

        Board {
            tiles,
            texture_file: "ChessPiecesArray.png",
            pieces_and_positions: [[None; BOARD_WIDTH]; BOARD_HEIGHT],
        }
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
