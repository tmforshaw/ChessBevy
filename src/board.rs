use crate::{
    game::{check_for_checks, CheckEvent, PlayerEnum},
    piece::{
        piece_is_black, piece_is_white, Piece, PieceEnum, PieceMove, PieceMoveEvent,
        PieceMoveHistory, COLOUR_AMT, PIECE_AMT, PIECE_HEIGHT, PIECE_HEIGHT_IMG, PIECE_WIDTH,
        PIECE_WIDTH_IMG,
    },
};

use bevy::prelude::*;

pub const BOARD_WIDTH: usize = 8;
pub const BOARD_HEIGHT: usize = 8;
pub const BOARD_SPACING: (f32, f32) = (4., 4.);

#[derive(Resource, Clone)]
pub struct Board {
    pub tiles: [[PieceEnum; BOARD_WIDTH]; BOARD_HEIGHT],
    pub texture_file: &'static str,
    pub pieces_and_positions: [[Option<Entity>; BOARD_WIDTH]; BOARD_HEIGHT],
    pub current_player: PlayerEnum,
    pub player_in_check: Option<PlayerEnum>,
    pub blocking_moves: [Vec<PieceMove>; COLOUR_AMT],
    pub move_history: Vec<PieceMoveHistory>,
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
                    let entity = commands.spawn(Piece::new(
                        (i, j),
                        board.tiles[i][j],
                        texture.clone(),
                        texture_atlas_layout.clone(),
                    ));

                    board.pieces_and_positions[i][j] = Some(entity.id());
                }
            }
        }
    }

    pub fn tiles_string(&self) -> String {
        let mut message = String::new();

        for i in (0..self.tiles.len()).rev() {
            for j in 0..self.tiles[i].len() {
                message.push(match self.tiles[i][j] {
                    PieceEnum::Empty => '*',
                    PieceEnum::BQueen => 'q',
                    PieceEnum::BKing => 'k',
                    PieceEnum::BKnight => 'n',
                    PieceEnum::BBishop => 'b',
                    PieceEnum::BRook => 'r',
                    PieceEnum::BPawn => 'p',
                    PieceEnum::WQueen => 'Q',
                    PieceEnum::WKing => 'K',
                    PieceEnum::WKnight => 'N',
                    PieceEnum::WBishop => 'B',
                    PieceEnum::WRook => 'R',
                    PieceEnum::WPawn => 'P',
                });
                message.push(' ');
            }

            message.push('\n');
        }

        message
    }

    // Movement
    fn can_piece_move_to(&self, piece_move: PieceMove) -> bool {
        let (ori_i, ori_j) = piece_move.from;
        let (i, j) = piece_move.to;

        // Can't take your own pieces
        if (piece_is_white(self.tiles[ori_i][ori_j]) && piece_is_white(self.tiles[i][j]))
            || (piece_is_black(self.tiles[ori_i][ori_j]) && piece_is_black(self.tiles[i][j]))
        {
            return false;
        }

        let i_diff = i as isize - ori_i as isize;
        let j_diff = j as isize - ori_j as isize;

        // TODO Add castling

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

                let en_passant = if let Some(piece_move_history) = self.move_history.last() {
                    let piece_move = piece_move_history.from_to;

                    // Opponent moved a pawn two spaces on last turn
                    let opponent_i_diff = piece_move.from.0 as isize - piece_move.to.0 as isize;
                    let vertical_opponent_and_self =
                        match self.tiles[piece_move.to.0][piece_move.to.1] {
                            PieceEnum::WPawn => opponent_i_diff == -2 && i_diff == -1,
                            PieceEnum::BPawn => opponent_i_diff == 2 && i_diff == 1,
                            _ => false,
                        };

                    vertical_opponent_and_self
                        && (piece_move.to.1 as isize - ori_j as isize).abs() == 1
                        && (piece_move.to.0 == ori_i)
                        && (piece_move.to.1 == j)
                } else {
                    false
                };

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

                first_move || vertical_movement && (normal_movement || capture_bool) || en_passant
            }
            PieceEnum::WRook | PieceEnum::BRook => self.can_move_straight(piece_move),
            PieceEnum::WBishop | PieceEnum::BBishop => self.can_move_diagonal(piece_move),
            PieceEnum::WQueen | PieceEnum::BQueen => {
                self.can_move_straight(piece_move) || self.can_move_diagonal(piece_move)
            }
            PieceEnum::WKnight | PieceEnum::BKnight => {
                // Move one square in one direction, and two in the other
                (i_diff.abs() == 1 && j_diff.abs() == 2) || (j_diff.abs() == 1 && i_diff.abs() == 2)
            }
            PieceEnum::WKing | PieceEnum::BKing => {
                // Move 1 in either (or both) directions
                i_diff.abs() <= 1 && j_diff.abs() <= 1 && !self.is_path_blocked(piece_move)
            }
            // Should never reach this point
            PieceEnum::Empty => {
                eprintln!("Tried to move an empty piece. ({ori_i}, {ori_j}) to ({i}, {j}).");
                false
            }
        }
    }

    fn can_move_straight(&self, piece_move: PieceMove) -> bool {
        let (ori_i, ori_j) = piece_move.from;
        let (i, j) = piece_move.to;

        let i_diff = i as isize - ori_i as isize;
        let j_diff = j as isize - ori_j as isize;

        // Allow movement in only one direction at a time
        let straight_movement =
            (i_diff.abs() > 0 && j_diff == 0) || (j_diff.abs() > 0 && i_diff == 0);

        straight_movement && !self.is_path_blocked(piece_move)
    }

    fn can_move_diagonal(&self, piece_move: PieceMove) -> bool {
        let (ori_i, ori_j) = piece_move.from;
        let (i, j) = piece_move.to;

        let i_diff = i as isize - ori_i as isize;
        let j_diff = j as isize - ori_j as isize;

        // Allow movement only along 45 degree diagonal (y = x)
        let diagonal_movement = i_diff.abs() == j_diff.abs();

        diagonal_movement && !self.is_path_blocked(piece_move)
    }

    fn is_path_blocked(&self, piece_move: PieceMove) -> bool {
        let (ori_i, ori_j) = piece_move.from;
        let (i, j) = piece_move.to;

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

    pub fn get_possible_moves(&self, (i, j): (usize, usize)) -> Vec<(usize, usize)> {
        // Check every tile on the board to see if the piece at this position can move to them
        let mut possible_tiles = Vec::new();
        for k in 0..BOARD_HEIGHT {
            for l in 0..BOARD_WIDTH {
                if self.can_piece_move_to(PieceMove {
                    from: (i, j),
                    to: (k, l),
                }) {
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

    pub fn get_all_possible_moves(&self, player: PlayerEnum) -> Vec<PieceMove> {
        let player_pieces = self.get_player_piece_positions(player);

        // TODO very very inefficeient, makes me sad
        let mut possible_tiles = Vec::new();
        for piece_pos in player_pieces {
            // Check every tile on the board to see if the piece at this position can move to them
            for k in 0..BOARD_HEIGHT {
                for l in 0..BOARD_WIDTH {
                    let piece_move = PieceMove {
                        from: piece_pos,
                        to: (k, l),
                    };

                    if self.can_piece_move_to(piece_move) {
                        possible_tiles.push(piece_move)
                    }
                }
            }
        }

        possible_tiles
    }

    pub fn get_check_stopping_moves(&self, check: CheckEvent) -> Vec<PieceMove> {
        let all_moves = self.get_all_possible_moves(check.player_in_check);

        let i_diff = check.at.0 as isize - check.checking_piece.0 as isize;
        let j_diff = check.at.1 as isize - check.checking_piece.1 as isize;

        let mut tiles_under_attack = Vec::new();
        for k in 0..i_diff.abs().max(j_diff.abs()) {
            tiles_under_attack.push((
                ((check.checking_piece.0 as isize + k * i_diff.signum()) as usize)
                    .clamp(0, BOARD_HEIGHT - 1),
                ((check.checking_piece.1 as isize + k * j_diff.signum()) as usize)
                    .clamp(0, BOARD_WIDTH - 1),
            ));
        }

        all_moves
            .iter()
            .filter_map(|&piece_move| {
                // Move must block check, king can only block check if it is capturing a piece
                if (tiles_under_attack.contains(&piece_move.to)
                    && !(piece_move.from == check.at && piece_move.to != check.checking_piece))
                // King can move away from tiles which are underattack
                ||(piece_move.from == check.at
                    && !tiles_under_attack.contains(&piece_move.to))
                {
                    // Don't allow putting yourself into check
                    let mut board_clone = self.clone();
                    board_clone.tiles[piece_move.to.0][piece_move.to.1] =
                        self.tiles[piece_move.from.0][piece_move.from.1];
                    board_clone.tiles[piece_move.from.0][piece_move.from.1] = PieceEnum::Empty;
                    let possible_checks = check_for_checks(&board_clone);

                    let mut return_val = Some(piece_move);
                    if !possible_checks.is_empty() {
                        for check in possible_checks {
                            if check.player_in_check as usize == self.current_player as usize {
                                return_val = None;
                                break;
                            }
                        }
                    }

                    return_val
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn next_player(&mut self) -> PlayerEnum {
        let next_player = self.see_next_player();
        self.current_player = next_player;

        next_player
    }

    pub fn see_next_player(&self) -> PlayerEnum {
        self.get_next_player(self.current_player)
    }

    pub fn get_next_player(&self, player: PlayerEnum) -> PlayerEnum {
        (player as usize + 1).into()
    }
}

pub fn move_piece_without_tests(
    commands: &mut Commands,
    board: &mut ResMut<Board>,
    transform: &mut Transform,
    (ori_i, ori_j): (usize, usize),
    (i, j): (usize, usize),
    piece_entity: Entity,
) -> Option<(PieceEnum, bool)> {
    // Delete pieces on capture
    let mut captured_piece = None;
    if board.tiles[i][j] as usize != PieceEnum::Empty as usize {
        if let Some(entity) = board.pieces_and_positions[i][j] {
            commands.entity(entity).despawn();

            captured_piece = Some((board.tiles[i][j], false));
        }
    } else {
        // Test for en passant
        let below_i =
            ((i as isize + (ori_i as isize - i as isize).signum()) as usize).clamp(0, BOARD_HEIGHT);

        // If en passant has occurred, delete the pawn underneath
        if matches!(
            (board.tiles[ori_i][ori_j], board.tiles[below_i][j]),
            (PieceEnum::WPawn, PieceEnum::BPawn) | (PieceEnum::BPawn, PieceEnum::WPawn)
        ) {
            if let Some(entity) = board.pieces_and_positions[below_i][j] {
                commands.entity(entity).despawn();

                captured_piece = Some((board.tiles[below_i][j], true));
            }
        }
    }

    let (x, y) = board_to_pixel_coords(i, j);
    transform.translation = Vec3::new(x, y, 1.); // z = 1 places the piece above the board, but below the held piece

    // Update board.tiles to reflect the new board position
    let moved_piece = board.tiles[ori_i][ori_j];
    board.tiles[ori_i][ori_j] = PieceEnum::Empty;
    board.tiles[i][j] = moved_piece;
    board.pieces_and_positions[ori_i][ori_j] = None;
    board.pieces_and_positions[i][j] = Some(piece_entity);

    captured_piece
}

pub fn move_piece(
    mut commands: Commands,
    mut ev_piece_move: EventReader<PieceMoveEvent>,
    mut ev_check: EventWriter<CheckEvent>,
    mut transform_query: Query<&mut Transform>,
    mut board: ResMut<Board>,
) {
    for piece_move_event in ev_piece_move.read() {
        let (ori_i, ori_j) = piece_move_event.to_from.from;
        let (i, j) = piece_move_event.to_from.to;

        let (ori_x, ori_y) = board_to_pixel_coords(ori_i, ori_j);

        let mut transform = transform_query.get_mut(piece_move_event.entity).unwrap();

        // Restrict Moves if player's king is in check
        if let Some(player_in_check) = board.player_in_check {
            if player_in_check as usize == board.current_player as usize {
                // This move does not block check
                if !board.blocking_moves[board.current_player as usize].contains(&PieceMove {
                    from: (ori_i, ori_j),
                    to: (i, j),
                }) {
                    // Move back to original position
                    transform.translation = Vec3::new(ori_x, ori_y, 1.); // z = 1 places the piece above the board, but below the held piece

                    return;
                } else {
                    board.blocking_moves[player_in_check as usize] = Vec::new();
                    board.player_in_check = None;
                }
            }
        }

        // Exit function if piece can't be moved there
        if !board.can_piece_move_to(PieceMove{from:(ori_i, ori_j), to:(i,j)})
            // Don't allow movement on opponents turn
            || ((piece_is_white(board.tiles[ori_i][ori_j])
                && board.current_player as usize == PlayerEnum::Black as usize)
                || (piece_is_black(board.tiles[ori_i][ori_j])
                    && board.current_player as usize == PlayerEnum::White as usize))
        {
            // Move back to original position
            transform.translation = Vec3::new(ori_x, ori_y, 1.); // z = 1 places the piece above the board, but below the held piece
            return;
        }

        // Move the piece and return if there was a captured piece
        let captured_piece = move_piece_without_tests(
            &mut commands,
            &mut board,
            &mut transform,
            (ori_i, ori_j),
            (i, j),
            piece_move_event.entity,
        );

        // Check to see if this move has a player in check
        for &check in check_for_checks(&board).iter() {
            ev_check.send(check);
        }

        // Add move to move history
        board.move_history.push(PieceMoveHistory {
            from_to: PieceMove {
                from: (ori_i, ori_j),
                to: (i, j),
            },
            captured: captured_piece,
        });

        // Change to the next player in the game
        board.next_player();
    }
}

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

        Board {
            tiles,
            texture_file: "ChessPiecesArray.png",
            pieces_and_positions: [[None; BOARD_WIDTH]; BOARD_HEIGHT],
            current_player: PlayerEnum::White,
            player_in_check: None,
            blocking_moves: std::array::from_fn(|_| Vec::new()),
            move_history: Vec::new(),
        }
    }
}
