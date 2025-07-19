use bevy::prelude::*;

use chess_core::{
    board::{Player, TilePos},
    piece::{Piece, COLOUR_AMT, PIECE_AMT},
};

use crate::{board::BoardBevy, piece::PieceBundle};

pub const BOARD_SIZE: u32 = 8;
pub const PIECE_SIZE: f32 = 200.;
pub const PIECE_SIZE_IMG: f32 = 150.;
pub const BOARD_SPACING: f32 = 0.;

pub const PIECE_TEXTURE_FILE: &str = "ChessPiecesArray.png";

#[must_use]
pub fn board_to_pixel_coords(file: u32, rank: u32) -> (f32, f32) {
    (
        (file as f32 - BOARD_SIZE as f32 / 2. + 0.5) * (PIECE_SIZE + BOARD_SPACING),
        (rank as f32 - BOARD_SIZE as f32 / 2. + 0.5) * (PIECE_SIZE + BOARD_SPACING),
    )
}

#[must_use]
pub fn pixel_to_board_coords(x: f32, y: f32) -> (u32, u32) {
    (
        ((((x / (PIECE_SIZE + BOARD_SPACING)) - 0.5 + BOARD_SIZE as f32 / 2.) as isize)
            .unsigned_abs() as u32)
            .clamp(0, BOARD_SIZE - 1),
        ((((y / (PIECE_SIZE + BOARD_SPACING)) - 0.5 + BOARD_SIZE as f32 / 2.) as isize)
            .unsigned_abs() as u32)
            .clamp(0, BOARD_SIZE - 1),
    )
}

#[allow(clippy::needless_pass_by_value)]
pub fn display_board(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut board: ResMut<BoardBevy>,
) {
    // Spawn Board Squares
    for rank in 0..BOARD_SIZE {
        for file in 0..BOARD_SIZE {
            let (x, y) = board_to_pixel_coords(file, rank);

            // Create a board with alternating light and dark squares
            // Starting with a light square on A1 (Bottom Left for White)
            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color: if (file + rank) % 2 == 0 {
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

    let (texture, texture_atlas_layout) =
        get_texture_atlas(&asset_server, &mut texture_atlas_layouts);

    // Spawn all the pieces where they are in the board.tiles array
    for rank in 0..BOARD_SIZE {
        for file in 0..BOARD_SIZE {
            if board.board.get_piece(TilePos::new(file, rank)) != Piece::None {
                let entity = commands.spawn(PieceBundle::new(
                    (file, rank),
                    board.board.get_piece(TilePos::new(file, rank)),
                    texture.clone(),
                    texture_atlas_layout.clone(),
                ));

                board.set_entity(TilePos::new(file, rank), Some(entity.id()));
            }
        }
    }
}

pub fn get_texture_atlas(
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) -> (
    bevy::prelude::Handle<Image>,
    bevy::prelude::Handle<TextureAtlasLayout>,
) {
    // Texture atlas for all the pieces
    (
        asset_server.load(PIECE_TEXTURE_FILE),
        texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            Vec2::new(PIECE_SIZE_IMG, PIECE_SIZE_IMG),
            PIECE_AMT,
            COLOUR_AMT,
            None,
            None,
        )),
    )
}

/// # Panics
/// Panics if the transform query cannot find the entity at the specified position
pub fn translate_piece_entity(
    transform_query: &mut Query<&mut Transform>,
    piece_entity: Entity,
    pos: TilePos,
) {
    let mut transform = transform_query
        .get_mut(piece_entity)
        .unwrap_or_else(|e| panic!("Could get piece entity at pos {pos}\n\t{e:?}"));
    let (x, y) = board_to_pixel_coords(pos.file, pos.rank);
    transform.translation = Vec3::new(x, y, 1.);
}

#[derive(Event)]
pub struct BackgroundColourEvent {
    colour: Color,
}

impl BackgroundColourEvent {
    #[must_use]
    pub const fn new_from_player(player: Player) -> Self {
        Self {
            colour: match player {
                Player::White => Color::rgb(1., 1., 1.),
                Player::Black => Color::rgb(0., 0., 0.),
            },
        }
    }

    #[must_use]
    pub const fn new(colour: Color) -> Self {
        Self { colour }
    }
}

pub fn background_colour_event_handler(
    mut background_ev: EventReader<BackgroundColourEvent>,
    mut clear_colour: ResMut<ClearColor>,
) {
    for ev in background_ev.read() {
        clear_colour.0 = ev.colour;
    }
}
