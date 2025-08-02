use bevy::prelude::*;

use chess_core::board::Player;

use crate::display::BackgroundColourEvent;

#[derive(Event)]
pub struct GameEndEvent {
    winning_player: Option<Player>,
}

impl GameEndEvent {
    #[must_use]
    pub const fn new(winning_player: Option<Player>) -> Self {
        Self { winning_player }
    }
}

pub fn game_end_event_handler(mut ev_game_end: EventReader<GameEndEvent>, mut background_ev: EventWriter<BackgroundColourEvent>) {
    for ev in ev_game_end.read() {
        if let Some(winning_player) = ev.winning_player {
            // Checkmate
            println!("Player {winning_player:?} wins by Checkmate");
            background_ev.write(BackgroundColourEvent::new(Color::linear_rgb(1.0, 0.0, 1.0)));
        } else {
            // Stalemate
            println!("Game ends in stalemate");
            background_ev.write(BackgroundColourEvent::new(Color::linear_rgb(0.0, 1.0, 1.0)));
        }
    }
}
