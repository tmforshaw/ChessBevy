use bevy::prelude::*;

use crate::{
    board::BoardBevy,
    uci::{transmit_to_uci, UciMessage},
    uci_info::UciScore,
};

#[derive(Resource, Default, PartialEq, Eq)]
pub struct CurrentEval {
    pub score: UciScore,
}

#[derive(Component)]
pub struct EvalBarWhite;

#[derive(Component)]
pub struct EvalBarBlack;

const BAR_HEIGHT: Val = Val::Px(50.0);

/// # Panics
/// Panics if the move history can't be turned into a piece move string
/// Panics if transmitting to the UCI cant be done
#[allow(clippy::needless_pass_by_value)]
pub fn create_eval_bar(mut commands: Commands, board: Res<BoardBevy>) {
    // Spawn the Eval bar
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: BAR_HEIGHT,
                ..default()
            },
            background_color: BackgroundColor(Color::BLACK),
            ..default()
        })
        .with_children(|parent| {
            // White's part
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(50.),
                        height: BAR_HEIGHT,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::WHITE),
                    ..default()
                })
                .insert(EvalBarWhite);

            // Black's part
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(50.),
                        height: BAR_HEIGHT,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::BLACK),
                    ..default()
                })
                .insert(EvalBarBlack);
        });

    // Ask the engine to update the eval bar
    transmit_to_uci(UciMessage::UpdateEval {
        move_history: board
            .board
            .move_history
            .to_piece_move_string()
            .expect("Could not convert move history into piece move string"),
        player_to_move: board.board.get_player(),
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

#[allow(clippy::needless_pass_by_value)]
pub fn update_eval_bar(eval: Res<CurrentEval>, mut query: Query<(&mut Style, Option<&EvalBarWhite>, Option<&EvalBarBlack>)>) {
    // Only update the eval bar when the evaluation changes
    if eval.is_changed() {
        println!("\nScore: {:?}\n", eval.score);
        let fraction = match eval.score {
            UciScore::Centipawn(cp) => {
                let capped = cp.clamp(-1000, 1000); // cap extreme scores
                0.5 + (capped as f32 / 2000.0) // between 0.0 and 1.0
            }
            UciScore::Mate(mate) => {
                if mate > 0 {
                    1.0 // White mates
                } else {
                    0.0 // Black mates
                }
            }
        };

        let white_percent = fraction * 100.0;
        let black_percent = 100.0 - white_percent;

        // Change the size of each part of the eval bar
        for (mut style, is_white, is_black) in &mut query {
            if is_white.is_some() {
                style.width = Val::Percent(white_percent);
            } else if is_black.is_some() {
                style.width = Val::Percent(black_percent);
            }
        }
    }
}
