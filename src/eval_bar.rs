use bevy::prelude::*;

use crate::uci_info::UciScore;

#[derive(Resource, Default, PartialEq, Eq)]
pub struct CurrentEval(pub UciScore);

#[derive(Component)]
pub struct EvalBarWhite;

#[derive(Component)]
pub struct EvalBarBlack;

const BAR_HEIGHT: Val = Val::Px(50.0);

pub fn create_eval_bar(mut commands: Commands) {
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
}

#[allow(clippy::needless_pass_by_value)]
pub fn update_eval_bar(
    eval: Res<CurrentEval>, // your resource storing Stockfish eval
    mut query: Query<(&mut Style, Option<&EvalBarWhite>, Option<&EvalBarBlack>)>,
) {
    let fraction = match eval.0 {
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

    // TODO Flip these since stockfish is showing percentage from point of view of black
    let black_percent = fraction * 100.0;
    let white_percent = 100.0 - black_percent;

    // Change the size of each part of the eval bar
    for (mut style, is_white, is_black) in &mut query {
        if is_white.is_some() {
            style.width = Val::Percent(white_percent);
        } else if is_black.is_some() {
            style.width = Val::Percent(black_percent);
        }
    }
}
