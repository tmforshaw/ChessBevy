use bevy::prelude::*;

#[derive(Component, Copy, Clone, Debug)]
pub enum PlayerEnum {
    White,
    Black,
}

#[derive(Component, Copy, Clone)]
pub struct Player {
    pub kind: PlayerEnum,
    pub in_check: bool,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            kind: PlayerEnum::White,
            in_check: false,
        }
    }
}

impl From<usize> for PlayerEnum {
    fn from(num: usize) -> Self {
        match num {
            1 => Self::Black,
            _ => Self::White,
        }
    }
}

// #[derive(Event, Debug)]
// pub struct GameOver {
//     pub winning_player: PlayerEnum,
// }

// pub fn game_over_read(mut ev_gameover: EventReader<GameOver>) {
//     for event in ev_gameover.read() {
//         println!("game over happened {event:?}");
//     }
// }
