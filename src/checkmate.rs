use bevy::prelude::*;

use crate::{board::Player, display::BackgroundColourEvent};

#[derive(Event)]
pub struct CheckmateEvent {
    winning_player: Player,
}

impl CheckmateEvent {
    #[must_use]
    pub const fn new(winning_player: Player) -> Self {
        Self { winning_player }
    }
}

pub fn checkmate_event_handler(
    mut ev_checkmate: EventReader<CheckmateEvent>,
    mut background_ev: EventWriter<BackgroundColourEvent>,
) {
    for ev in ev_checkmate.read() {
        println!("Player {:?} wins by Checkmate", ev.winning_player);

        background_ev.send(BackgroundColourEvent::new(Color::rgb(1.0, 0.0, 1.0)));
    }
}
