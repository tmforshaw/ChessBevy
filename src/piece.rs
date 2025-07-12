use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::{
    board::{Player, TilePos},
    display::{board_to_pixel_coords, pixel_to_board_coords, PIECE_SIZE, PIECE_SIZE_IMG},
    piece_move::{PieceMove, PieceMoveEvent},
    possible_moves::PossibleMoveDisplayEvent,
};

pub const PIECE_AMT: usize = 6;
pub const COLOUR_AMT: usize = 2;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Piece {
    BQueen = 0,
    BKing = 1,
    BRook = 2,
    BKnight = 3,
    BBishop = 4,
    BPawn = 5,
    WQueen = 8,
    WKing = 9,
    WRook = 10,
    WKnight = 11,
    WBishop = 12,
    WPawn = 13,
    None = 14,
}

impl From<Piece> for usize {
    fn from(value: Piece) -> Self {
        value as Self - 1 - 2 * Self::from(value.is_black())
    }
}

impl From<usize> for Piece {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::BQueen,
            1 => Self::BKing,
            2 => Self::BRook,
            3 => Self::BKnight,
            4 => Self::BBishop,
            5 => Self::BPawn,
            8 => Self::WQueen,
            9 => Self::WKing,
            10 => Self::WRook,
            11 => Self::WKnight,
            12 => Self::WBishop,
            13 => Self::WPawn,
            _ => Self::None,
        }
    }
}

pub const PIECES: &[Piece] = &[
    Piece::BQueen,
    Piece::BKing,
    Piece::BRook,
    Piece::BKnight,
    Piece::BBishop,
    Piece::BPawn,
    Piece::WQueen,
    Piece::WKing,
    Piece::WRook,
    Piece::WKnight,
    Piece::WBishop,
    Piece::WPawn,
];

impl Piece {
    #[must_use]
    pub fn is_white(self) -> bool {
        ((self as u8 >> 3) & 1) == 1 && self != Self::None
    }

    #[must_use]
    pub fn is_black(self) -> bool {
        ((self as u8 >> 3) & 1) == 0 && self != Self::None
    }

    #[must_use]
    pub const fn to_player(self) -> Option<Player> {
        match self {
            Self::BQueen
            | Self::BKing
            | Self::BRook
            | Self::BKnight
            | Self::BBishop
            | Self::BPawn => Some(Player::Black),

            Self::WQueen
            | Self::WKing
            | Self::WRook
            | Self::WKnight
            | Self::WBishop
            | Self::WPawn => Some(Player::White),

            Self::None => None,
        }
    }

    #[must_use]
    pub fn is_player(self, player: Player) -> bool {
        match player {
            Player::White => self.is_white(),
            Player::Black => self.is_black(),
        }
    }

    #[must_use]
    pub fn to_bitboard_index(&self) -> usize {
        // Will panic if Piece::None is used as an index
        PIECES
            .iter()
            .enumerate()
            .find_map(|(i, piece)| if self == piece { Some(i) } else { None })
            .unwrap()
    }

    #[must_use]
    pub const fn to_algebraic(&self) -> char {
        match self {
            Self::None => '-',
            Self::WPawn => 'P',
            Self::WKnight => 'N',
            Self::WBishop => 'B',
            Self::WRook => 'R',
            Self::WQueen => 'Q',
            Self::WKing => 'K',
            Self::BPawn => 'p',
            Self::BKnight => 'n',
            Self::BBishop => 'b',
            Self::BRook => 'r',
            Self::BQueen => 'q',
            Self::BKing => 'k',
        }
    }

    #[must_use]
    pub const fn from_algebraic(chr: char) -> Option<Self> {
        match chr {
            '-' => Some(Self::None),
            'P' => Some(Self::WPawn),
            'N' => Some(Self::WKnight),
            'B' => Some(Self::WBishop),
            'R' => Some(Self::WRook),
            'Q' => Some(Self::WQueen),
            'K' => Some(Self::WKing),
            'p' => Some(Self::BPawn),
            'n' => Some(Self::BKnight),
            'b' => Some(Self::BBishop),
            'r' => Some(Self::BRook),
            'q' => Some(Self::BQueen),
            'k' => Some(Self::BKing),
            _ => None,
        }
    }
}

impl From<Piece> for char {
    fn from(val: Piece) -> Self {
        val.to_algebraic()
    }
}

// TODO Remove this
#[allow(clippy::fallible_impl_from)]
impl From<char> for Piece {
    fn from(val: char) -> Self {
        // TODO Stop the panic when incorrect letter is parsed
        Self::from_algebraic(val).unwrap()
    }
}

#[derive(Bundle)]
pub struct PieceBundle {
    pub sprite: SpriteSheetBundle,
    on_drag_start_listener: On<Pointer<DragStart>>,
    on_drag_listener: On<Pointer<Drag>>,
    on_drag_end_listener: On<Pointer<DragEnd>>,
}

impl PieceBundle {
    pub fn new(
        (file, rank): (usize, usize),
        key: Piece,
        texture: Handle<Image>,
        texture_atlas_layout: Handle<TextureAtlasLayout>,
    ) -> Self {
        assert!(key != Piece::None, "{key:?} used as bitboard index");
        let (x, y) = board_to_pixel_coords(file, rank);

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
            on_drag_start_listener: On::<Pointer<DragStart>>::run(on_piece_drag_start),
            on_drag_listener: On::<Pointer<Drag>>::run(on_piece_drag),
            on_drag_end_listener: On::<Pointer<DragEnd>>::run(on_piece_drag_end),
        }
    }
}

fn on_piece_drag_start(
    mut ev_drag: EventReader<Pointer<Drag>>,
    mut ev_draw_moves: EventWriter<PossibleMoveDisplayEvent>,
    mut transform_query: Query<&mut Transform>,
) {
    for ev in ev_drag.read() {
        let transform = transform_query.get_mut(ev.target).unwrap();

        let mouse_pos = transform.translation.xy() * Vec2::new(1., -1.);
        let (file, rank) = pixel_to_board_coords(mouse_pos.x, -mouse_pos.y);

        ev_draw_moves.send(PossibleMoveDisplayEvent {
            from: TilePos::new(file, rank),
            show: true,
        });
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
    mut drag_er: EventReader<Pointer<DragEnd>>,
    mut transform_query: Query<&mut Transform>,
    mut ev_draw_moves: EventWriter<PossibleMoveDisplayEvent>,
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

        ev_draw_moves.send(PossibleMoveDisplayEvent {
            from: TilePos::new(file, rank),
            show: false,
        });

        ev_piece_move.send(PieceMoveEvent {
            piece_move: PieceMove::new(TilePos::new(ori_file, ori_rank), TilePos::new(file, rank)),
            entity: drag_data.target,
        });
    }
}
