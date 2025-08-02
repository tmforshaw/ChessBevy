use bevy::prelude::*;

use chess_core::{board::TilePos, piece::Piece, piece_move::PieceMove};

use crate::{
    display::{board_to_pixel_coords, pixel_to_board_coords, PIECE_SIZE, PIECE_SIZE_IMG},
    piece_move::PieceMoveEvent,
    possible_moves::PossibleMoveDisplayEvent,
};

pub struct PieceBundle;

impl PieceBundle {
    /// # Panics
    /// Panics if ``Piece::None`` used as a bitboard index for the texture atlas
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new(
        (file, rank): (u32, u32),
        key: Piece,
        texture: Handle<Image>,
        // texture_atlas_layout: Handle<TextureAtlasLayout>,
    ) -> (Sprite, Transform, GlobalTransform) {
        assert!(key != Piece::None, "{key:?} used as bitboard index");

        let (x, y) = board_to_pixel_coords(file, rank);

        // Create a bundle with this piece's spritesheet and some listeners for picking up the pieces
        (
            // texture_atlas_layout, // Handle<TextureAtlasLayout>
            Sprite {
                image: texture,
                custom_size: None,
                color: Color::WHITE,
                ..default()
            },
            Transform::from_scale(Vec3::splat(PIECE_SIZE / PIECE_SIZE_IMG)).with_translation(Vec3::new(x, y, 1.)),
            GlobalTransform::default(),
        )
    }
}

/// # Panics
/// Panics if the dragged entity's transform cannot be found
pub fn on_piece_drag_start(
    mut ev_drag: EventReader<Pointer<DragStart>>,
    mut ev_draw_moves: EventWriter<PossibleMoveDisplayEvent>,
    mut transform_query: Query<&mut Transform>,
) {
    for ev in ev_drag.read() {
        let transform = transform_query
            .get_mut(ev.target)
            .expect("Dragged entity's transform could not be found");

        let mouse_pos = transform.translation.xy() * Vec2::new(1., -1.);
        let (file, rank) = pixel_to_board_coords(mouse_pos.x, -mouse_pos.y);

        ev_draw_moves.write(PossibleMoveDisplayEvent {
            from: TilePos::new(file, rank),
            show: true,
        });
    }
}

/// Move the piece when it is dragged by a mouse
/// # Panics
/// Panics if the dragged entity's transform cannot be found
pub fn on_piece_drag(mut drag_er: EventReader<Pointer<Drag>>, mut transform_query: Query<&mut Transform>) {
    for drag_data in drag_er.read() {
        let mut transform = transform_query
            .get_mut(drag_data.target)
            .expect("Dragged entity's transform could not be found");

        transform.translation += Vec3::new(drag_data.delta.x, -drag_data.delta.y, 0.);
        transform.translation.z = 10.;
    }
}

/// Finalise the movement of a piece, either snapping it to the grid, or by moving it back
/// # Panics
/// Panics if the dragged entity's transform cannot be found
pub fn on_piece_drag_end(
    mut drag_er: EventReader<Pointer<DragEnd>>,
    mut transform_query: Query<&mut Transform>,
    mut ev_draw_moves: EventWriter<PossibleMoveDisplayEvent>,
    mut ev_piece_move: EventWriter<PieceMoveEvent>,
) {
    for drag_data in drag_er.read() {
        let transform = transform_query
            .get_mut(drag_data.target)
            .expect("Dragged entity's transform could not be found");

        // Find where the piece was moved from in board coordinates
        let original_pos = transform.translation.xy() - Vec2::new(drag_data.distance.x, -drag_data.distance.y)
            + Vec2::new(PIECE_SIZE, PIECE_SIZE) / 2.;
        let (ori_file, ori_rank) = pixel_to_board_coords(original_pos.x, original_pos.y);

        // Find the new position, snapped to board coords, and move the sprite there
        let (file, rank) = pixel_to_board_coords(
            transform.translation.x + PIECE_SIZE / 2.,
            transform.translation.y + PIECE_SIZE / 2.,
        );

        ev_draw_moves.write(PossibleMoveDisplayEvent {
            from: TilePos::new(file, rank),
            show: false,
        });

        ev_piece_move.write(PieceMoveEvent {
            piece_move: PieceMove::new(TilePos::new(ori_file, ori_rank), TilePos::new(file, rank)),
            entity: drag_data.target,
        });
    }
}
