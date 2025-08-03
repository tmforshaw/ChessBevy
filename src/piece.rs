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
    pub fn spawn<'a>(
        commands: &'a mut Commands,
        (file, rank): (u32, u32),
        key: Piece,
        texture: Handle<Image>,
        texture_atlas_layout: Handle<TextureAtlasLayout>,
    ) -> EntityCommands<'a> {
        assert!(key != Piece::None, "{key:?} used as bitboard index");

        let (x, y) = board_to_pixel_coords(file, rank);

        // Create a bundle with this piece's spritesheet, and Pickable marker
        let mut entity = commands.spawn((
            Sprite::from_atlas_image(
                texture,
                TextureAtlas {
                    layout: texture_atlas_layout,
                    index: key.to_bitboard_index(), // choose whichever sprite index you want
                },
            ),
            Transform::from_scale(Vec3::splat(PIECE_SIZE / PIECE_SIZE_IMG)).with_translation(Vec3::new(x, y, 1.)),
            GlobalTransform::default(),
            Pickable::default(),
        ));

        // Add triggers for drag events
        entity.observe(on_piece_drag);
        entity.observe(on_piece_drag_start);
        entity.observe(on_piece_drag_end);

        entity
    }
}

/// # Panics
/// Panics if the dragged entity's transform cannot be found
#[allow(clippy::needless_pass_by_value)]
pub fn on_piece_drag_start(
    ev_drag: Trigger<Pointer<DragStart>>,
    mut ev_draw_moves: EventWriter<PossibleMoveDisplayEvent>,
    mut transform_query: Query<&mut Transform, With<Pickable>>,
) {
    let ev = ev_drag.event();

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

/// Move the piece when it is dragged by a mouse
/// # Panics
/// Panics if the dragged entity's transform cannot be found
#[allow(clippy::needless_pass_by_value)]
pub fn on_piece_drag(ev_drag: Trigger<Pointer<Drag>>, mut transform_query: Query<&mut Transform, With<Pickable>>) {
    let ev = ev_drag.event();

    let mut transform = transform_query
        .get_mut(ev.target)
        .expect("Dragged entity's transform could not be found");

    transform.translation += Vec3::new(ev.delta.x, -ev.delta.y, 0.);
    transform.translation.z = 10.;
}

/// Finalise the movement of a piece, either snapping it to the grid, or by moving it back
/// # Panics
/// Panics if the dragged entity's transform cannot be found
#[allow(clippy::needless_pass_by_value)]
pub fn on_piece_drag_end(
    ev_drag: Trigger<Pointer<DragEnd>>,
    mut transform_query: Query<&mut Transform, With<Pickable>>,
    mut ev_draw_moves: EventWriter<PossibleMoveDisplayEvent>,
    mut ev_piece_move: EventWriter<PieceMoveEvent>,
) {
    let ev = ev_drag.event();

    let transform = transform_query
        .get_mut(ev.target)
        .expect("Dragged entity's transform could not be found");

    // Find where the piece was moved from in board coordinates
    let original_pos =
        transform.translation.xy() - Vec2::new(ev.distance.x, -ev.distance.y) + Vec2::new(PIECE_SIZE, PIECE_SIZE) / 2.;
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
        entity: ev.target,
    });
}
