use crate::{
    piece::{
        piece_is_black, piece_is_white, Piece, PieceEnum, COLOUR_AMT, PIECE_AMT,
        PIECE_AMT_PER_SIDE, PIECE_HEIGHT, PIECE_HEIGHT_IMG, PIECE_WIDTH, PIECE_WIDTH_IMG,
    },
    player::PlayerEnum,
};

use bevy::prelude::*;

pub const BOARD_WIDTH: usize = 8;
pub const BOARD_HEIGHT: usize = 8;
pub const BOARD_SPACING: (f32, f32) = (4., 4.);

pub fn board_to_pixel_coords(i: usize, j: usize) -> (f32, f32) {
    (
        (j as f32 - BOARD_WIDTH as f32 / 2. + 0.5) * (PIECE_WIDTH + BOARD_SPACING.0),
        (i as f32 - BOARD_HEIGHT as f32 / 2. + 0.5) * (PIECE_HEIGHT + BOARD_SPACING.1),
    )
}

pub fn pixel_to_board_coords(x: f32, y: f32) -> (usize, usize) {
    (
        (((y / (PIECE_HEIGHT + BOARD_SPACING.1)) - 0.5 + BOARD_HEIGHT as f32 / 2.) as usize)
            .clamp(0, BOARD_HEIGHT - 1),
        (((x / (PIECE_WIDTH + BOARD_SPACING.0)) - 0.5 + BOARD_WIDTH as f32 / 2.) as usize)
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

        tiles[2][6] = PieceEnum::BPawn;

        Board {
            tiles,
            texture_file: "ChessPiecesArray.png",
            pieces_and_positions: [[None; BOARD_WIDTH]; BOARD_HEIGHT],
            current_player: PlayerEnum::White,
            player_in_check: None,
            checks: [[None; PIECE_AMT_PER_SIDE]; COLOUR_AMT],
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
    pub current_player: PlayerEnum,
    pub player_in_check: Option<PlayerEnum>,
    pub checks: [[Option<((usize, usize), (usize, usize))>; PIECE_AMT_PER_SIDE]; COLOUR_AMT],
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

                // Create a board with alternating light and dark squares
                // Starting with a light square on A1 (Bottom Left for White)
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
                    transform: Transform::from_xyz(x, y, 0.),
                    ..default()
                });
            }
        }

        // Texture atlas for all the pieces
        let texture = asset_server.load(board.texture_file);
        let texture_atlas_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            Vec2::new(PIECE_WIDTH_IMG, PIECE_HEIGHT_IMG),
            PIECE_AMT,
            COLOUR_AMT,
            None,
            None,
        ));

        // Spawn all the pieces where they are in the board.tiles array
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
        // Restrict Moves if player's king is in check
        if let Some(player_in_check) = self.player_in_check {
            if player_in_check as usize == self.current_player as usize {
                let mut blocking_moves = Vec::new();

                for res_data in self.checks[self.current_player as usize] {
                    match res_data {
                        Some((check_from, king_pos)) => {
                            let mut blocks = self.get_check_stopping_moves(
                                self.current_player,
                                check_from,
                                king_pos,
                            );

                            blocking_moves.append(&mut blocks);
                        }
                        None => {
                            break;
                        }
                    }
                }

                // This move does not block check
                if !blocking_moves.contains(&((ori_i, ori_j), (i, j))) {
                    let (ori_x, ori_y) = board_to_pixel_coords(ori_i, ori_j);
                    // Move back to original position
                    transform.translation = Vec3::new(ori_x, ori_y, 1.); // z = 1 places the piece above the board, but below the held piece
                    return;
                } else {
                    self.player_in_check = None;
                }
            }
        }

        // Exit function if piece can't be moved there
        if !self.can_move_piece_to((ori_i, ori_j), (i, j))
            // Don't allow movement on opponents turn
            || ((piece_is_white(self.tiles[ori_i][ori_j])
                && self.current_player as usize == PlayerEnum::Black as usize)
                || (piece_is_black(self.tiles[ori_i][ori_j])
                    && self.current_player as usize == PlayerEnum::White as usize))
        {
            let (ori_x, ori_y) = board_to_pixel_coords(ori_i, ori_j);
            // Move back to original position
            transform.translation = Vec3::new(ori_x, ori_y, 1.); // z = 1 places the piece above the board, but below the held piece
            return;
        }

        // Delete pieces on capture
        if self.tiles[i][j] as usize != PieceEnum::Empty as usize {
            if let Some(entity) = self.pieces_and_positions[i][j] {
                commands.entity(entity).despawn();
            }
        }

        let (x, y) = board_to_pixel_coords(i, j);
        transform.translation = Vec3::new(x, y, 1.); // z = 1 places the piece above the board, but below the held piece

        // Update board.tiles to reflect the new board position
        let moved_piece = self.tiles[ori_i][ori_j];
        self.tiles[ori_i][ori_j] = PieceEnum::Empty;
        self.tiles[i][j] = moved_piece;
        self.pieces_and_positions[ori_i][ori_j] = None;
        self.pieces_and_positions[i][j] = Some(moved_piece_entity);

        // Check to see if this move has left the opponent in check
        let checks = self.check_for_checks(self.current_player);

        // Change to the next player in the game
        self.current_player = (self.current_player as usize + 1).into();

        if !checks.is_empty() {
            self.player_in_check = Some(self.current_player);

            let mut check_arr = [None; PIECE_AMT_PER_SIDE];

            (0..check_arr.len()).for_each(|i| match checks.get(i) {
                Some(&value) => check_arr[i] = Some(value),
                None => check_arr[i] = None,
            });

            self.checks[self.current_player as usize] = check_arr;

            println!("{:?} is in check", self.current_player);
            println!("{self}");
        };
    }

    // Movement
    fn can_move_piece_to(&self, (ori_i, ori_j): (usize, usize), (i, j): (usize, usize)) -> bool {
        // Can't take your own pieces
        if (piece_is_white(self.tiles[ori_i][ori_j]) && piece_is_white(self.tiles[i][j]))
            || (piece_is_black(self.tiles[ori_i][ori_j]) && piece_is_black(self.tiles[i][j]))
        {
            return false;
        }

        let i_diff = i as isize - ori_i as isize;
        let j_diff = j as isize - ori_j as isize;

        match self.tiles[ori_i][ori_j] {
            PieceEnum::WPawn | PieceEnum::BPawn => {
                // Allow double movement on pawn's first move
                let first_move = match self.tiles[ori_i][ori_j] {
                    PieceEnum::WPawn => ori_i == 1 && i_diff == 2,
                    PieceEnum::BPawn => ori_i == BOARD_HEIGHT - 2 && i_diff == -2,
                    _ => unreachable!(),
                } && self.tiles
                    [((ori_i as isize + i_diff.signum()) as usize).clamp(0, BOARD_HEIGHT)][ori_j]
                    as usize
                    == PieceEnum::Empty as usize
                    && self.tiles
                        [((ori_i as isize + 2 * i_diff.signum()) as usize).clamp(0, BOARD_HEIGHT)]
                        [ori_j] as usize
                        == PieceEnum::Empty as usize
                    && j_diff == 0;

                // Allow the pawn to move up or down depending on player colour
                // This affects captures as well
                let vertical_movement = match self.tiles[ori_i][ori_j] {
                    PieceEnum::WPawn => i_diff == 1,
                    PieceEnum::BPawn => i_diff == -1,
                    _ => unreachable!(),
                };

                // Restrict up or down movement to only directly up and down
                let normal_movement =
                    j_diff == 0 && self.tiles[i][j] as usize == PieceEnum::Empty as usize;

                // Allow diagonal capturing of pieces with pawns
                let capture_bool = j_diff.abs() == 1
                    && self.tiles[i][j] as usize != PieceEnum::Empty as usize
                    && i_diff.abs() == 1;

                first_move || vertical_movement && (normal_movement || capture_bool)
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
                // Move one square in one direction, and two in the other
                (i_diff.abs() == 1 && j_diff.abs() == 2) || (j_diff.abs() == 1 && i_diff.abs() == 2)
            }
            PieceEnum::WKing | PieceEnum::BKing => {
                // Move 1 in either (or both) directions
                i_diff.abs() <= 1
                    && j_diff.abs() <= 1
                    && !self.is_path_blocked((ori_i, ori_j), (i, j))
            }
            // Should never reach this point
            PieceEnum::Empty => {
                eprintln!("Tried to move an empty piece. ({ori_i}, {ori_j}) to ({i}, {j}).");
                false
            }
        }
    }

    fn can_move_straight(&self, (ori_i, ori_j): (usize, usize), (i, j): (usize, usize)) -> bool {
        let i_diff = i as isize - ori_i as isize;
        let j_diff = j as isize - ori_j as isize;

        // Allow movement in only one direction at a time
        let straight_movement =
            (i_diff.abs() > 0 && j_diff == 0) || (j_diff.abs() > 0 && i_diff == 0);

        straight_movement && !self.is_path_blocked((ori_i, ori_j), (i, j))
    }

    fn can_move_diagonal(&self, (ori_i, ori_j): (usize, usize), (i, j): (usize, usize)) -> bool {
        let i_diff = i as isize - ori_i as isize;
        let j_diff = j as isize - ori_j as isize;

        // Allow movement only along 45 degree diagonal (y = x)
        let diagonal_movement = i_diff.abs() == j_diff.abs();

        diagonal_movement && !self.is_path_blocked((ori_i, ori_j), (i, j))
    }

    fn is_path_blocked(&self, (ori_i, ori_j): (usize, usize), (i, j): (usize, usize)) -> bool {
        let i_diff = i as isize - ori_i as isize;
        let j_diff = j as isize - ori_j as isize;

        // Traverse the maximum distance towards the position in both directions and check for obstructions
        // Going up to exactly this maximum distance would prevent captures
        for k in 1..i_diff.abs().max(j_diff.abs()) {
            let new_i = ((ori_i as isize + k * i_diff.signum()) as usize).clamp(0, BOARD_HEIGHT);
            let new_j = ((ori_j as isize + k * j_diff.signum()) as usize).clamp(0, BOARD_WIDTH);

            if self.tiles[new_i][new_j] as usize != PieceEnum::Empty as usize {
                return true;
            }
        }

        false
    }

    // fn path_would_be_blocked(
    //     &self,
    //     (ori_i, ori_j): (usize, usize),
    //     (i, j): (usize, usize),
    //     board_changes: Vec<(PieceEnum, (usize, usize))>,
    // ) -> bool {
    //     let i_diff = i as isize - ori_i as isize;
    //     let j_diff = j as isize - ori_j as isize;

    //     let mut tiles = self.tiles;

    //     for &(piece, (i, j)) in board_changes.iter() {
    //         println!("Testing ({i}, {j}) to {piece:?}");
    //         tiles[i][j] = piece;
    //     }

    //     // Traverse the maximum distance towards the position in both directions and check for obstructions
    //     // Going up to exactly this maximum distance would prevent captures
    //     for k in 1..i_diff.abs().max(j_diff.abs()) {
    //         let new_i = ((ori_i as isize + k * i_diff.signum()) as usize).clamp(0, BOARD_HEIGHT);
    //         let new_j = ((ori_j as isize + k * j_diff.signum()) as usize).clamp(0, BOARD_WIDTH);

    //         if tiles[new_i][new_j] as usize != PieceEnum::Empty as usize {
    //             return true;
    //         }
    //     }

    //     false
    // }

    pub fn get_possible_moves(&self, (i, j): (usize, usize)) -> Vec<(usize, usize)> {
        // Check every tile on the board to see if the piece at this position can move to them
        let mut possible_tiles = Vec::new();
        for k in 0..BOARD_HEIGHT {
            for l in 0..BOARD_WIDTH {
                if self.can_move_piece_to((i, j), (k, l)) {
                    possible_tiles.push((k, l))
                }
            }
        }

        possible_tiles
    }

    pub fn get_player_piece_positions(&self, player: PlayerEnum) -> Vec<(usize, usize)> {
        self.tiles
            .iter()
            .enumerate()
            .flat_map(|(i, arr)| {
                let arr = arr.iter().enumerate();
                std::iter::repeat(i).zip(arr)
            })
            .filter_map(|(i, (j, &piece))| {
                match player {
                    PlayerEnum::White => piece_is_white(piece),
                    PlayerEnum::Black => piece_is_black(piece),
                }
                .then_some((i, j))
            })
            .collect()
    }

    pub fn get_all_possible_moves(
        &self,
        player: PlayerEnum,
    ) -> Vec<((usize, usize), (usize, usize))> {
        let player_pieces = self.get_player_piece_positions(player);

        // TODO very very inefficeient, makes me sad
        let mut possible_tiles = Vec::new();
        for piece_pos in player_pieces {
            // Check every tile on the board to see if the piece at this position can move to them
            for k in 0..BOARD_HEIGHT {
                for l in 0..BOARD_WIDTH {
                    if self.can_move_piece_to(piece_pos, (k, l)) {
                        possible_tiles.push((piece_pos, (k, l)))
                    }
                }
            }
        }

        possible_tiles
    }

    pub fn get_check_stopping_moves(
        &self,
        player: PlayerEnum,
        in_check_from: (usize, usize),
        king_pos: (usize, usize),
    ) -> Vec<((usize, usize), (usize, usize))> {
        let all_moves = self.get_all_possible_moves(player);

        let i_diff = king_pos.0 as isize - in_check_from.0 as isize;
        let j_diff = king_pos.1 as isize - in_check_from.1 as isize;

        let mut tiles_to_block = Vec::new();
        for k in 0..i_diff.abs().max(j_diff.abs()) {
            tiles_to_block.push((
                ((in_check_from.0 as isize + k * i_diff.signum()) as usize)
                    .clamp(0, BOARD_HEIGHT - 1),
                ((in_check_from.1 as isize + k * j_diff.signum()) as usize)
                    .clamp(0, BOARD_WIDTH - 1),
            ));
        }

        all_moves
            .iter()
            .filter_map(|&(from_pos, to_pos)| {
                if tiles_to_block.contains(&to_pos) && from_pos != king_pos {
                    Some((from_pos, to_pos))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn check_for_checks(&self, player: PlayerEnum) -> Vec<((usize, usize), (usize, usize))> {
        let all_moves = self.get_all_possible_moves(player);

        all_moves
            .iter()
            .filter_map(|&(attacking_piece_pos, (i, j))| match self.tiles[i][j] {
                PieceEnum::BKing if player as usize == PlayerEnum::White as usize => {
                    Some((attacking_piece_pos, (i, j)))
                }
                PieceEnum::WKing if player as usize == PlayerEnum::Black as usize => {
                    Some((attacking_piece_pos, (i, j)))
                }
                _ => None,
            })
            .collect()
    }
}
