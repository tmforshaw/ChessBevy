use bevy::prelude::*;

use crate::uci_info::UciEval;

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
