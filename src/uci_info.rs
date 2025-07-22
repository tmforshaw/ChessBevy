use chess_core::board::Player;

use crate::{uci::UciError, uci_event::UciToBoardMessage};

#[derive(Default, Debug, Clone)]
pub struct UciInfo {
    pub depth: u32,
    pub score: UciScore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UciScore {
    Centipawn(i32),
    Mate(i32),
}

impl UciScore {
    #[must_use]
    pub fn new<S: AsRef<str>>(score_type: S, score_val: i32) -> Self {
        match score_type.as_ref() {
            "cp" => Self::Centipawn(score_val),
            _ => Self::Mate(score_val), // score_type should only be "mate"
        }
    }
}

impl Default for UciScore {
    fn default() -> Self {
        Self::Centipawn(0)
    }
}

/// # Errors
/// Returns an error if any matched string cannot be parsed into an integer
pub fn uci_parse_info<S: AsRef<str>>(line: S) -> Result<UciInfo, UciError> {
    let tokens = line.as_ref().split_whitespace().collect::<Vec<_>>();

    let mut uci_info = UciInfo::default();

    let mut i = 1;
    while i < tokens.len() {
        match tokens[i] {
            "depth" => {
                uci_info.depth = tokens[i + 1].parse::<u32>()?;

                i += 2;
            }
            "score" => {
                uci_info.score = UciScore::new(tokens[i + 1], tokens[i + 2].parse::<i32>()?);

                i += 3;
            }
            "pv" => {
                // TODO Principal Variation
                let _pv_moves = &tokens[i + 1..];

                break;
            }
            "multipv" => {
                // TODO Multiple Principle Variation
                let _multi_pv = tokens[i + 1].parse::<u32>()?;

                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    Ok(uci_info)
}

/// # Errors
/// Error if the ``uci_parse_info`` cannot parse the information from uci
pub fn send_uci_info<S: AsRef<str>>(
    line: S,
    board_tx: &crossbeam_channel::Sender<UciToBoardMessage>,
    player_to_move: Player,
) -> Result<(), UciError> {
    // Parse the final info line from the UCI reply
    let uci_info = uci_parse_info(line.as_ref().trim())?;

    // Flip the score if black was moving since the score is always from the current player's perspective
    let player_modifier = if player_to_move == Player::Black { -1 } else { 1 };

    // The score is in centipawns
    board_tx.send(match uci_info.score {
        UciScore::Centipawn(score) => UciToBoardMessage::Score(player_modifier * score),
        UciScore::Mate(mate_in) => UciToBoardMessage::Mate(player_modifier * mate_in),
    })?;

    Ok(())
}
