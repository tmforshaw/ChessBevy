use bevy::prelude::*;

use chess_core::piece_move::PieceMove;

use crate::{
    board::BoardBevy,
    classification::{MoveClassification, MoveClassificationMarker},
    display::{
        board_to_pixel_coords, get_classification_texture_atlas, BackgroundColourEvent, CLASSIFICATION_SIZE_IMG, PIECE_SIZE,
    },
    eval_bar::CurrentEval,
    game_end::GameEndEvent,
    last_move::LastMoveEvent,
    uci_info::UciEval,
};

#[derive(Debug, Resource, Clone)]
pub enum UciToBoardMessage {
    BestMove(PieceMove),
    Centipawn(i32),
    Mate(i32),
    MoveClassification(MoveClassification),
}

#[derive(Event, Resource, Debug, Clone)]
pub struct UciEvent {
    message: UciToBoardMessage,
}
impl UciEvent {
    #[allow(dead_code)]
    #[must_use]
    pub const fn new(message: UciToBoardMessage) -> Self {
        Self { message }
    }
}

#[derive(Resource)]
pub struct UciToBoardReceiver(pub crossbeam_channel::Receiver<UciToBoardMessage>);

#[allow(clippy::too_many_arguments)]
#[allow(clippy::needless_pass_by_value)]
pub fn uci_to_board_event_handler(
    mut ev_uci_to_board: EventReader<UciEvent>,
    mut commands: Commands,
    mut board: ResMut<BoardBevy>,
    mut background_ev: EventWriter<BackgroundColourEvent>,
    mut transform_query: Query<&mut Transform>,
    mut texture_atlas_query: Query<&mut TextureAtlas>,
    mut game_end_ev: EventWriter<GameEndEvent>,
    mut last_move_ev: EventWriter<LastMoveEvent>,
    mut current_eval: ResMut<CurrentEval>,
    move_classification_entities: Query<Entity, With<MoveClassificationMarker>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Listen for messages from the Engine Listener thread, then apply moves
    for ev in ev_uci_to_board.read() {
        match ev.message {
            UciToBoardMessage::BestMove(piece_move) => {
                // Apply the move to the board
                board.apply_move(
                    &mut commands,
                    &mut transform_query,
                    &mut texture_atlas_query,
                    &mut background_ev,
                    &mut game_end_ev,
                    &mut last_move_ev,
                    piece_move,
                );
            }
            UciToBoardMessage::Centipawn(eval) => {
                current_eval.eval = UciEval::Centipawn(eval);
            }
            UciToBoardMessage::Mate(mate_in) => {
                current_eval.eval = UciEval::Mate(mate_in);
            }
            UciToBoardMessage::MoveClassification(move_class) => {
                println!("Move Type: {move_class:?}\t\t{:?}\n", board.board.get_next_player());

                // Clear any move classification entities
                for entity in move_classification_entities.iter() {
                    commands.entity(entity).despawn();
                }

                let Some(last_move) = board.board.move_history.get() else {
                    continue;
                };
                let (last_move, _, _, _) = last_move.into();

                let (x, y) = board_to_pixel_coords(last_move.to.file, last_move.to.rank);

                let (texture, texture_atlas_layout) = get_classification_texture_atlas(&asset_server, &mut texture_atlas_layouts);

                commands.spawn((
                    SpriteSheetBundle {
                        texture,
                        atlas: TextureAtlas {
                            layout: texture_atlas_layout,
                            index: move_class.to_atlas_index(),
                        },
                        transform: Transform::from_scale(Vec3::splat((PIECE_SIZE * 0.4) / CLASSIFICATION_SIZE_IMG))
                            .with_translation(Vec3::new(x + PIECE_SIZE / 2.25, y + PIECE_SIZE / 2.25, 1.5)),
                        ..default()
                    },
                    MoveClassificationMarker,
                ));
            }
        }
    }
}

// Take the messages sent via crossbeam_channel and send them to Bevy as Events
#[allow(clippy::needless_pass_by_value)]
pub fn process_uci_to_board_threads(tx_rx: Res<UciToBoardReceiver>, mut uci_to_board_ev: EventWriter<UciEvent>) {
    for ev in tx_rx.0.try_iter() {
        uci_to_board_ev.send(UciEvent::new(ev));
    }
}
