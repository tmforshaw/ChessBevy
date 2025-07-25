use bevy::prelude::*;

use crate::{
    board::BoardBevy,
    display::{board_to_pixel_coords, get_classification_texture_atlas, CLASSIFICATION_SIZE_IMG, PIECE_SIZE},
    uci_info::UciEval,
};

#[derive(Debug, Copy, Clone)]
pub enum MoveClassification {
    Best,
    Excellent,
    Good,
    Inaccuracy,
    Mistake,
    Miss,
    Blunder,
}

impl MoveClassification {
    #[must_use]
    pub const fn to_atlas_index(&self) -> usize {
        match self {
            Self::Best => 2,
            Self::Excellent => 3,
            Self::Good => 4,
            Self::Inaccuracy => 6,
            Self::Mistake => 7,
            Self::Blunder => 8,
            Self::Miss => 9,
        }
    }
}

#[derive(Component)]
pub struct MoveClassificationMarker;

#[must_use]
pub const fn classify_move(evaluation_after: UciEval, evaluation_best: UciEval) -> MoveClassification {
    match (evaluation_after, evaluation_best) {
        (UciEval::Centipawn(eval_after), UciEval::Centipawn(eval_best)) => {
            let delta = (eval_best - eval_after).unsigned_abs();

            match delta {
                0 => MoveClassification::Best,
                1..=20 => MoveClassification::Excellent,
                21..=50 => MoveClassification::Good,
                51..=100 => MoveClassification::Inaccuracy,
                101..=300 => MoveClassification::Mistake,
                _ => MoveClassification::Blunder,
            }
        }
        // Had mate available but no longer
        (UciEval::Centipawn(_), UciEval::Mate(_)) => MoveClassification::Miss,
        // Gave opponent mate
        (UciEval::Mate(_), UciEval::Centipawn(_)) => MoveClassification::Blunder,
        // Either gave opponent mate when player had mate, or continued this player's mating sequence
        (UciEval::Mate(mate_after), UciEval::Mate(mate_best)) => {
            // Check if the opponent was given mate instead of this player
            if mate_after.signum() == mate_best.signum() {
                let delta = (mate_best - mate_after).unsigned_abs();

                match delta {
                    0 => MoveClassification::Best,
                    1 => MoveClassification::Excellent,
                    _ => MoveClassification::Good,
                }
            } else {
                MoveClassification::Blunder
            }
        }
    }
}

pub fn clear_classifications(
    commands: &mut Commands,
    move_classification_entities: &Query<Entity, With<MoveClassificationMarker>>,
) {
    // Clear any move classification entities
    for entity in move_classification_entities.iter() {
        commands.entity(entity).despawn();
    }
}

/// # Errors
/// Returns an error if the move history can't get the current move
pub fn show_classification(
    commands: &mut Commands,
    board: &BoardBevy,
    move_classification_entities: &Query<Entity, With<MoveClassificationMarker>>,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    move_class: MoveClassification,
) -> Result<(), String> {
    // println!("Move Type: {move_class:?}\t\t{:?}\n", board.board.get_next_player());

    clear_classifications(commands, move_classification_entities);

    let Some(last_move) = board.board.move_history.get() else {
        return Err("Could not get move current move history".to_string());
    };
    let (last_move, _, _, _) = last_move.into();

    let (x, y) = board_to_pixel_coords(last_move.to.file, last_move.to.rank);

    let (texture, texture_atlas_layout) = get_classification_texture_atlas(asset_server, texture_atlas_layouts);

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

    Ok(())
}
