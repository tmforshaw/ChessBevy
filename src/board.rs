use crate::piece::{
    piece_is_black, piece_is_white, Piece, PieceEnum, COLOUR_AMT, PIECE_AMT, PIECE_HEIGHT,
    PIECE_SCALE, PIECE_WIDTH,
};

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_mod_picking::prelude::*;

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
        (((y / (PIECE_HEIGHT * PIECE_SCALE + BOARD_SPACING.1)) - 0.5 + BOARD_HEIGHT as f32 / 2.)
            as usize)
            .clamp(0, BOARD_HEIGHT - 1),
        (((x / (PIECE_WIDTH * PIECE_SCALE + BOARD_SPACING.0)) - 0.5 + BOARD_WIDTH as f32 / 2.)
            as usize)
            .clamp(0, BOARD_WIDTH - 1),
    )
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
        (ori_i, ori_j): (usize, usize),
        (i, j): (usize, usize),
        moved_piece_entity: Entity,
        transform: &mut Transform,
        commands: &mut Commands,
    ) {
        let moved_piece = self.tiles[ori_i][ori_j];
        let (ori_x, ori_y) = board_to_pixel_coords(ori_i, ori_j);

        // Exit function if both pieces are the same colour
        if (piece_is_white(self.tiles[i][j]) && piece_is_white(self.tiles[ori_i][ori_j]))
            || (piece_is_black(self.tiles[i][j]) && piece_is_black(self.tiles[ori_i][ori_j]))
            || !self.can_move_piece_to((ori_i, ori_j), (i, j))
        {
            // Move back to original position
            transform.translation = Vec3::new(ori_x, ori_y, 1.);
            return;
        }

        // Pieces are different colours and new tile is not empty
        if self.tiles[i][j] as usize != PieceEnum::Empty as usize {
            if let Some(entity) = self.pieces_and_positions[i][j] {
                commands.entity(entity).despawn();
            }
        }

        let (x, y) = board_to_pixel_coords(i, j);
        transform.translation = Vec3::new(x, y, 1.);

        self.tiles[ori_i][ori_j] = PieceEnum::Empty;
        self.tiles[i][j] = moved_piece;
        self.pieces_and_positions[ori_i][ori_j] = None;
        self.pieces_and_positions[i][j] = Some(moved_piece_entity);
    }

    // Movement
    fn can_move_piece_to(&self, (ori_i, ori_j): (usize, usize), (i, j): (usize, usize)) -> bool {
        match self.tiles[ori_i][ori_j] {
            PieceEnum::WPawn | PieceEnum::BPawn => {
                let i_diff = i as isize - ori_i as isize;
                let j_diff = j as isize - ori_j as isize;

                let vertical_bool = match self.tiles[ori_i][ori_j] {
                    PieceEnum::WPawn => i_diff == 1 || (ori_i == 1 && i_diff.abs() == 2),
                    PieceEnum::BPawn => {
                        i_diff == -1 || (ori_i == BOARD_HEIGHT - 2 && i_diff.abs() == 2)
                    }
                    _ => unreachable!(),
                };

                vertical_bool
                    && (j_diff == 0 && self.tiles[i][j] as usize == PieceEnum::Empty as usize)
                    || (j_diff.abs() == 1 && self.tiles[i][j] as usize != PieceEnum::Empty as usize)
            }
            PieceEnum::WRook | PieceEnum::BRook => self.can_move_straight((ori_i, ori_j), (i, j)),
            PieceEnum::WBishop | PieceEnum::BBishop => {
                self.can_move_diagonal((ori_i, ori_j), (i, j))
            }
            PieceEnum::WQueen | PieceEnum::BQueen => {
                self.can_move_straight((ori_i, ori_j), (i, j))
                    || self.can_move_diagonal((ori_i, ori_j), (i, j))
            }
            PieceEnum::WKnight | PieceEnum::BKnight => {
                let i_diff = i as isize - ori_i as isize;
                let j_diff = j as isize - ori_j as isize;

                (i_diff.abs() == 1 && j_diff.abs() == 2) || (j_diff.abs() == 1 && i_diff.abs() == 2)
            }
            PieceEnum::WKing | PieceEnum::BKing => {
                let i_diff = i as isize - ori_i as isize;
                let j_diff = j as isize - ori_j as isize;

                i_diff.abs() <= 1
                    && j_diff.abs() <= 1
                    && !self.is_path_blocked((ori_i, ori_j), (i, j))
            }
            PieceEnum::Empty => false,
        }
    }

    fn can_move_straight(&self, (ori_i, ori_j): (usize, usize), (i, j): (usize, usize)) -> bool {
        let i_diff = i as isize - ori_i as isize;
        let j_diff = j as isize - ori_j as isize;

        let straight_movement =
            (i_diff.abs() > 0 && j_diff == 0) || (j_diff.abs() > 0 && i_diff == 0);

        straight_movement && !self.is_path_blocked((ori_i, ori_j), (i, j))
    }

    fn can_move_diagonal(&self, (ori_i, ori_j): (usize, usize), (i, j): (usize, usize)) -> bool {
        let i_diff = i as isize - ori_i as isize;
        let j_diff = j as isize - ori_j as isize;

        let diagonal_movement = i_diff.abs() == j_diff.abs();

        diagonal_movement && !self.is_path_blocked((ori_i, ori_j), (i, j))
    }

    fn is_path_blocked(&self, (ori_i, ori_j): (usize, usize), (i, j): (usize, usize)) -> bool {
        let i_diff = i as isize - ori_i as isize;
        let j_diff = j as isize - ori_j as isize;

        for k in 1..i_diff.abs() {
            if self.tiles[((ori_i as isize + k * i_diff.signum()) as usize).clamp(0, BOARD_HEIGHT)]
                [((ori_j as isize + k * j_diff.signum()) as usize).clamp(0, BOARD_WIDTH)]
                as usize
                != PieceEnum::Empty as usize
            {
                return true;
            }
        }

        false
    }

    pub fn get_possible_moves(&self, (i, j): (usize, usize)) {}
}

pub fn draw_possible_moves(
    mut drag_er: EventReader<Pointer<DragStart>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // let shapes = [
    //     Mesh2dHandle(meshes.add(Circle { radius: 50.0 })),
    //     Mesh2dHandle(meshes.add(Ellipse::new(25.0, 50.0))),
    //     Mesh2dHandle(meshes.add(Capsule2d::new(25.0, 50.0))),
    //     Mesh2dHandle(meshes.add(Rectangle::new(50.0, 100.0))),
    //     Mesh2dHandle(meshes.add(RegularPolygon::new(50.0, 6))),
    //     Mesh2dHandle(meshes.add(Triangle2d::new(
    //         Vec2::Y * 50.0,
    //         Vec2::new(-50.0, -50.0),
    //         Vec2::new(50.0, -50.0),
    //     ))),
    // ];
    // let num_shapes = shapes.len();

    // for (i, shape) in shapes.into_iter().enumerate() {
    //     // Distribute colors evenly across the rainbow.
    //     let color = Color::hsl(360. * i as f32 / num_shapes as f32, 0.95, 0.7);

    //     commands.spawn(MaterialMesh2dBundle {
    //         mesh: shape,
    //         material: materials.add(color),
    //         transform: Transform::from_xyz(
    //             // Distribute shapes from -X_EXTENT to +X_EXTENT.
    //             BOARD_WIDTH as f32 / -2. + i as f32 / (num_shapes - 1) as f32 * BOARD_WIDTH as f32,
    //             0.0,
    //             0.0,
    //         ),
    //         ..default()
    //     });
    // }
}
