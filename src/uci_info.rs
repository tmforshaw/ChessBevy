use crate::uci::UciError;

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
    pub fn new(score_type: &str, score_val: i32) -> Self {
        match score_type {
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
pub fn uci_parse_info(line: &str) -> Result<UciInfo, UciError> {
    let tokens = line.split_whitespace().collect::<Vec<_>>();

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
