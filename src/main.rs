use bevy::prelude::*;

use bevy_mod_picking::prelude::*;
use board::{Board, BOARD_HEIGHT, BOARD_SPACING, BOARD_WIDTH};
use piece::{PieceEnum, COLOUR_AMT, PIECE_AMT, PIECE_HEIGHT, PIECE_SCALE, PIECE_WIDTH};

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
        .insert_resource(Board::default())
        .add_systems(Startup, setup)
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
                let (x, y) = board_to_pixel_coords(i, j);

                commands.spawn((
                    SpriteSheetBundle {
                        texture: texture.clone(),
                        atlas: TextureAtlas {
                            layout: texture_atlas_layout.clone(),
                            index: board.tiles[i][j] as usize,
                        },
                        transform: Transform::from_scale(Vec3::splat(PIECE_SCALE))
                            .with_translation(Vec3::new(x, y, 1.)),
                        ..default()
                    },
                    On::<Pointer<Drag>>::target_component_mut::<Transform>(on_piece_drag),
                    On::<Pointer<DragEnd>>::target_component_mut::<Transform>(on_piece_dropped),
                ));
            }
        }
    }
}

fn on_piece_drag(drag: &mut ListenerInput<Pointer<Drag>>, transform: &mut Transform) {
    transform.translation += Vec3::new(drag.delta.x, -drag.delta.y, 0.);
    transform.translation.z = 10.;
}

fn on_piece_dropped(_: &mut ListenerInput<Pointer<DragEnd>>, transform: &mut Transform) {
    let (i, j) = pixel_to_board_coords(
        transform.translation.x + PIECE_WIDTH * PIECE_SCALE / 2.,
        transform.translation.y + PIECE_HEIGHT * PIECE_SCALE / 2.,
    );
    let (x, y) = board_to_pixel_coords(i, j);

    transform.translation = Vec3::new(x, y, 1.)
}

fn board_to_pixel_coords(i: usize, j: usize) -> (f32, f32) {
    (
        (j as f32 - BOARD_WIDTH as f32 / 2. + 0.5) * (PIECE_WIDTH * PIECE_SCALE + BOARD_SPACING.0),
        (i as f32 - BOARD_HEIGHT as f32 / 2. + 0.5)
            * (PIECE_HEIGHT * PIECE_SCALE + BOARD_SPACING.1),
    )
}

fn pixel_to_board_coords(x: f32, y: f32) -> (usize, usize) {
    (
        ((y / (PIECE_HEIGHT * PIECE_SCALE + BOARD_SPACING.1)) - 0.5 + BOARD_HEIGHT as f32 / 2.)
            as usize,
        ((x / (PIECE_WIDTH * PIECE_SCALE + BOARD_SPACING.0)) - 0.5 + BOARD_WIDTH as f32 / 2.)
            as usize,
    )
}
