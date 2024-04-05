use bevy::prelude::*;

#[derive(Component, Copy, Clone)]
pub enum PlayerEnum {
    White,
    Black,
}

#[derive(Component, Copy, Clone)]
struct Player {
    kind: PlayerEnum,
    in_check: bool,
}

impl From<usize> for PlayerEnum {
    fn from(num: usize) -> Self {
        match num {
            1 => Self::Black,
            _ => Self::White,
        }
    }
}
