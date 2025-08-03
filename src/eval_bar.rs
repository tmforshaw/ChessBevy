use bevy::prelude::*;

use crate::{
    board::BoardBevy,
    uci::{transmit_to_uci, UciMessage},
    uci_info::UciEval,
};

#[derive(Resource, Default, PartialEq, Eq, Clone)]
pub struct CurrentEval {
    pub eval: UciEval,
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
    commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: BAR_HEIGHT,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            BackgroundColor(Color::linear_rgb(0.3, 0.3, 0.3)),
        ))
        .with_children(|parent| {
            // White Part
            parent.spawn((
                Node {
                    width: Val::Percent(50.0),
                    height: BAR_HEIGHT,
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                EvalBarWhite,
            ));

            // Black Part
            parent.spawn((
                Node {
                    width: Val::Percent(50.0),
                    height: BAR_HEIGHT,
                    ..default()
                },
                BackgroundColor(Color::BLACK),
                EvalBarBlack,
            ));
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
pub fn update_eval_bar(eval: Res<CurrentEval>, mut query: Query<(&mut Node, Option<&EvalBarWhite>, Option<&EvalBarBlack>)>) {
    // Only update the eval bar when the evaluation changes
    if eval.is_changed() {
        let fraction = match eval.eval {
            UciEval::Centipawn(cp) => {
                let capped = cp.clamp(-1000, 1000); // cap extreme evals
                0.5 + (capped as f32 / 2000.0) // between 0.0 and 1.0
            }
            UciEval::Mate(mate) => {
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
        for (mut node, is_white, is_black) in &mut query {
            if is_white.is_some() {
                node.width = Val::Percent(white_percent);
            } else if is_black.is_some() {
                node.width = Val::Percent(black_percent);
            }
        }
    }
}
