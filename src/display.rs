use bevy::prelude::*;

use crate::{
    board::{Board, TilePos},
    piece::{Piece, PieceBundle, COLOUR_AMT, PIECE_AMT},
};

pub const BOARD_SIZE: usize = 8;
pub const PIECE_SIZE: f32 = 200.;
pub const PIECE_SIZE_IMG: f32 = 150.;
pub const BOARD_SPACING: f32 = PIECE_SIZE * 0.027;

const PIECE_TEXTURE_FILE: &str = "ChessPiecesArray.png";

pub fn board_to_pixel_coords(i: usize, j: usize) -> (f32, f32) {
    (
        (j as f32 - BOARD_SIZE as f32 / 2. + 0.5) * (PIECE_SIZE + BOARD_SPACING),
        (i as f32 - BOARD_SIZE as f32 / 2. + 0.5) * (PIECE_SIZE + BOARD_SPACING),
    )
}

pub fn pixel_to_board_coords(x: f32, y: f32) -> (usize, usize) {
    (
        (((y / (PIECE_SIZE + BOARD_SPACING)) - 0.5 + BOARD_SIZE as f32 / 2.) as usize)
            .clamp(0, BOARD_SIZE - 1),
        (((x / (PIECE_SIZE + BOARD_SPACING)) - 0.5 + BOARD_SIZE as f32 / 2.) as usize)
            .clamp(0, BOARD_SIZE - 1),
    )
}

pub fn display_board(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut board: ResMut<Board>,
) {
    // Spawn Board Squares
    for i in 0..BOARD_SIZE {
        for j in 0..BOARD_SIZE {
            let (x, y) = board_to_pixel_coords(i, j);

            // Create a board with alternating light and dark squares
            // Starting with a light square on A1 (Bottom Left for White)
            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color: if (i + j) % 2 == 0 {
                        Color::WHITE
                    } else {
                        Color::PURPLE
                    },
                    custom_size: Some(Vec2::new(PIECE_SIZE, PIECE_SIZE)),
                    ..default()
                },
                transform: Transform::from_xyz(x, y, 0.),
                ..default()
            });
        }
    }

    // Texture atlas for all the pieces
    let texture = asset_server.load(PIECE_TEXTURE_FILE);
    let texture_atlas_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        Vec2::new(PIECE_SIZE_IMG, PIECE_SIZE_IMG),
        PIECE_AMT,
        COLOUR_AMT,
        None,
        None,
    ));

    // Spawn all the pieces where they are in the board.tiles array
    for file in 0..BOARD_SIZE {
        for rank in 0..BOARD_SIZE {
            if board.get_piece(TilePos::new(file, rank)) != Piece::None {
                let entity = commands.spawn(PieceBundle::new(
                    (file, rank),
                    board.get_piece(TilePos::new(file, rank)),
                    texture.clone(),
                    texture_atlas_layout.clone(),
                ));

                board.set_entity(TilePos::new(file, rank), Some(entity.id()));
            }
        }
    }
}
