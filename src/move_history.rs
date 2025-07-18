use bevy::prelude::*;

use crate::{board::BoardBevy, display::BackgroundColourEvent, game_end::GameEndEvent};

#[derive(Event)]
pub struct MoveHistoryEvent {
    pub backwards: bool,
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::too_many_arguments)]
pub fn move_history_event_handler(
    mut move_history_ev: EventReader<MoveHistoryEvent>,
    mut board: ResMut<BoardBevy>,
    mut transform_query: Query<&mut Transform>,
    mut background_ev: EventWriter<BackgroundColourEvent>,
    mut game_end_ev: EventWriter<GameEndEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut texture_atlas_query: Query<&mut TextureAtlas>,
) {
    for ev in move_history_ev.read() {
        // Traverse the history in the specified direction
        let piece_move_history = if ev.backwards {
            board.board.move_history.traverse_prev()
        } else {
            board.board.move_history.traverse_next()
        };

        let Some(history_move) = piece_move_history else {
            // History is empty, or index went out of bounds (Don't perform any moves)
            return;
        };

        if ev.backwards {
            board.undo_move(
                &mut commands,
                &asset_server,
                &mut texture_atlas_layouts,
                &mut transform_query,
                &mut texture_atlas_query,
                &mut background_ev,
                history_move,
            );
        } else {
            let (piece_move_original, _, _, _) = history_move.into();

            let _ = board.apply_move(
                &mut commands,
                &mut transform_query,
                &mut texture_atlas_query,
                &mut background_ev,
                &mut game_end_ev,
                piece_move_original,
            );
        }
    }
}
