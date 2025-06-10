use bevy::{prelude::*, sprite::Mesh2dHandle};
use bevy_mod_picking::prelude::*;

use crate::{
    board::{Board, Player, TilePos},
    display::{board_to_pixel_coords, pixel_to_board_coords, PIECE_SIZE, PIECE_SIZE_IMG},
};

pub const PIECE_AMT: usize = 6;
pub const COLOUR_AMT: usize = 2;

#[derive(Event)]
pub struct PieceMoveEvent {
    pub piece_move: PieceMove,
    pub entity: Entity,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PieceMove {
    pub from: TilePos,
    pub to: TilePos,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Piece {
    None = 0,
    WQueen = 1,
    WKing = 2,
    WRook = 3,
    WKnight = 4,
    WBishop = 5,
    WPawn = 6,
    BQueen = 9,
    BKing = 10,
    BRook = 11,
    BKnight = 12,
    BBishop = 13,
    BPawn = 14,
}

impl From<Piece> for usize {
    fn from(value: Piece) -> usize {
        value as usize - 1 - 2 * (value.is_black() as usize)
    }
}

impl From<usize> for Piece {
    fn from(value: usize) -> Piece {
        match value {
            1 => Piece::WQueen,
            2 => Piece::WKing,
            3 => Piece::WRook,
            4 => Piece::WKnight,
            5 => Piece::WBishop,
            6 => Piece::WPawn,
            9 => Piece::BQueen,
            10 => Piece::BKing,
            11 => Piece::BRook,
            12 => Piece::BKnight,
            13 => Piece::BBishop,
            14 => Piece::BPawn,
            _ => Piece::None,
        }
    }
}

pub const PIECES: &[Piece] = &[
    Piece::WQueen,
    Piece::WKing,
    Piece::WRook,
    Piece::WKnight,
    Piece::WBishop,
    Piece::WPawn,
    Piece::BQueen,
    Piece::BKing,
    Piece::BRook,
    Piece::BKnight,
    Piece::BBishop,
    Piece::BPawn,
];

impl Piece {
    pub fn is_white(self) -> bool {
        ((self as u8 >> 3) & 1) == 0 && self != Piece::None
    }

    pub fn is_black(self) -> bool {
        ((self as u8 >> 3) & 1) == 1 && self != Piece::None
    }

    pub fn is_player(self, player: Player) -> bool {
        match player {
            Player::White => self.is_white(),
            Player::Black => self.is_black(),
        }
    }

    pub fn to_bitboard_index(&self) -> usize {
        PIECES
            .iter()
            .enumerate()
            .find_map(|(i, piece)| if self == piece { Some(i) } else { None })
            .unwrap()
    }

    pub fn to_algebraic(&self) -> char {
        match self {
            Piece::None => '-',
            Piece::WPawn => 'P',
            Piece::WKnight => 'N',
            Piece::WBishop => 'B',
            Piece::WRook => 'R',
            Piece::WQueen => 'Q',
            Piece::WKing => 'K',
            Piece::BPawn => 'p',
            Piece::BKnight => 'n',
            Piece::BBishop => 'b',
            Piece::BRook => 'r',
            Piece::BQueen => 'q',
            Piece::BKing => 'k',
        }
    }

    pub fn from_algebraic(chr: char) -> Option<Self> {
        match chr {
            '-' => Some(Piece::None),
            'P' => Some(Piece::WPawn),
            'N' => Some(Piece::WKnight),
            'B' => Some(Piece::WBishop),
            'R' => Some(Piece::WRook),
            'Q' => Some(Piece::WQueen),
            'K' => Some(Piece::WKing),
            'p' => Some(Piece::BPawn),
            'n' => Some(Piece::BKnight),
            'b' => Some(Piece::BBishop),
            'r' => Some(Piece::BRook),
            'q' => Some(Piece::BQueen),
            'k' => Some(Piece::BKing),
            _ => None,
        }
    }
}

impl From<Piece> for char {
    fn from(val: Piece) -> Self {
        val.to_algebraic()
    }
}

impl From<char> for Piece {
    fn from(val: char) -> Self {
        // TODO
        Piece::from_algebraic(val).unwrap()
    }
}

#[derive(Bundle)]
pub struct PieceBundle {
    pub sprite: SpriteSheetBundle,
    // on_drag_start_listener: On<Pointer<DragStart>>,
    on_drag_listener: On<Pointer<Drag>>,
    on_drag_end_listener: On<Pointer<DragEnd>>,
}

impl PieceBundle {
    pub fn new(
        (i, j): (usize, usize),
        key: Piece,
        texture: Handle<Image>,
        texture_atlas_layout: Handle<TextureAtlasLayout>,
    ) -> Self {
        let (x, y) = board_to_pixel_coords(i, j);

        // Create a bundle with this piece's spritesheet and some listeners for picking up the pieces
        Self {
            sprite: SpriteSheetBundle {
                texture,
                atlas: TextureAtlas {
                    layout: texture_atlas_layout,
                    index: key.to_bitboard_index(),
                },
                transform: Transform::from_scale(Vec3::splat(PIECE_SIZE / PIECE_SIZE_IMG))
                    .with_translation(Vec3::new(x, y, 1.)),
                ..default()
            },
            // on_drag_start_listener: On::<Pointer<DragStart>>::run(draw_possible_moves),
            on_drag_listener: On::<Pointer<Drag>>::run(on_piece_drag),
            on_drag_end_listener: On::<Pointer<DragEnd>>::run(on_piece_drag_end),
        }
    }
}

// Move the piece when it is dragged by a mouse
fn on_piece_drag(
    mut drag_er: EventReader<Pointer<Drag>>,
    mut transform_query: Query<&mut Transform>,
) {
    for drag_data in drag_er.read() {
        let mut transform = transform_query.get_mut(drag_data.target).unwrap();
        transform.translation += Vec3::new(drag_data.delta.x, -drag_data.delta.y, 0.);
        transform.translation.z = 10.;
    }
}

// Finalise the movement of a piece, either snapping it to the grid, or by moving it back
fn on_piece_drag_end(
    mut commands: Commands,
    mut drag_er: EventReader<Pointer<DragEnd>>,
    mut transform_query: Query<&mut Transform>,
    possible_move_meshes: Query<Entity, With<Mesh2dHandle>>,
    mut ev_piece_move: EventWriter<PieceMoveEvent>,
) {
    for drag_data in drag_er.read() {
        let transform = transform_query.get_mut(drag_data.target).unwrap();

        // Find where the piece was moved from in board coordinates
        let original_pos = transform.translation.xy()
            - Vec2::new(drag_data.distance.x, -drag_data.distance.y)
            + Vec2::new(PIECE_SIZE, PIECE_SIZE) / 2.;
        let (ori_file, ori_rank) = pixel_to_board_coords(original_pos.x, original_pos.y);

        // Find the new position, snapped to board coords, and move the sprite there
        let (file, rank) = pixel_to_board_coords(
            transform.translation.x + PIECE_SIZE / 2.,
            transform.translation.y + PIECE_SIZE / 2.,
        );

        ev_piece_move.send(PieceMoveEvent {
            piece_move: PieceMove {
                from: TilePos::new(ori_file, ori_rank),
                to: TilePos::new(file, rank),
            },
            entity: drag_data.target,
        });

        // Clean up the possible move markers
        for mesh in possible_move_meshes.iter() {
            commands.entity(mesh).despawn();
        }
    }
}

pub fn piece_move_event_reader(
    mut commands: Commands,
    mut ev_piece_move: EventReader<PieceMoveEvent>,
    mut transform_query: Query<&mut Transform>,
    mut board: ResMut<Board>,
) {
    for ev in ev_piece_move.read() {
        // Entity Logic
        let move_complete;
        {
            let mut transform = transform_query.get_mut(ev.entity).unwrap();

            let moved_to = board.get_piece(ev.piece_move.to);

            println!(
                "{:?}\t{moved_to:?}\t\t{}\t{:?}",
                board.get_piece(ev.piece_move.from),
                board.get_piece(ev.piece_move.from).is_player(board.player),
                board.player,
            );

            // Snap the moved entity to the grid (Don't move if there is a non-opponent piece there, or if you moved a piece on another player's turn)
            let (x, y) = if !moved_to.is_player(board.player)
                && board.get_piece(ev.piece_move.from).is_player(board.player)
            {
                if moved_to != Piece::None {
                    let moved_to = board.get_entity(ev.piece_move.to).unwrap();

                    commands.entity(moved_to).despawn();
                }

                move_complete = true;
                board_to_pixel_coords(ev.piece_move.to.file, ev.piece_move.to.rank)
            } else {
                // Reset position
                move_complete = false;
                board_to_pixel_coords(ev.piece_move.from.file, ev.piece_move.from.rank)
            };
            transform.translation = Vec3::new(x, y, 1.);
        }

        // Board Logic
        if move_complete {
            board.move_piece(ev.piece_move);
            println!("{}", board.clone());
            board.next_player();
        }
    }
}
