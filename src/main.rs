use bevy::prelude::*;

use bevy_mod_picking::prelude::*;
use board::{board_to_pixel_coords, Board, BOARD_HEIGHT, BOARD_WIDTH};
use piece::{Piece, PieceEnum, COLOUR_AMT, PIECE_AMT, PIECE_HEIGHT, PIECE_SCALE, PIECE_WIDTH};

pub mod board;
pub mod piece;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Big Time Bevy Time WOoop".into(),
                        resolution: (1920., 1080.).into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .build(),
            DefaultPickingPlugins,
        ))
        // .insert_resource(DebugPickingMode::Normal)
        .init_resource::<Board>()
        .add_systems(Startup, setup)
        // .add_systems(
        //     Update,
        //     on_piece_drag.run_if(on_event::<PieceBeingDragged>()),
        // )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    board: ResMut<Board>,
) {
    // Spawn camera
    commands.spawn(Camera2dBundle::default());

    // Spawn Board
    for i in 0..BOARD_HEIGHT {
        for j in 0..BOARD_WIDTH {
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
    let texture = asset_server.load("ChessPiecesArray.png");
    let layout = TextureAtlasLayout::from_grid(
        Vec2::new(PIECE_WIDTH, PIECE_HEIGHT),
        PIECE_AMT,
        COLOUR_AMT,
        None,
        None,
    );
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    // Spawn all the pieces in their respective locations
    for i in 0..BOARD_HEIGHT {
        for j in 0..BOARD_WIDTH {
            if PieceEnum::Empty as usize != board.tiles[i][j] as usize {
                // let (x, y) = board_to_pixel_coords(i, j);

                // commands.spawn((
                //     SpriteSheetBundle {
                //         texture: texture.clone(),
                //         atlas: TextureAtlas {
                //             layout: texture_atlas_layout.clone(),
                //             index: board.tiles[i][j] as usize,
                //         },
                //         transform: Transform::from_scale(Vec3::splat(PIECE_SCALE))
                //             .with_translation(Vec3::new(x, y, 1.)),
                //         ..default()
                //     },
                //     On::<Pointer<Drag>>::target_component_mut::<Transform>(on_piece_drag),
                //     On::<Pointer<DragEnd>>::target_component_mut::<Transform>(on_piece_dropped),
                // ));
                commands.spawn((Piece::new(
                    (i, j),
                    board.tiles[i][j],
                    texture.clone(),
                    texture_atlas_layout.clone(),
                ),));
            }
        }
    }
}
