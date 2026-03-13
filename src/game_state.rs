use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Earth,   // Ground — player sees Earth, enters elevator
    Riding,  // Elevator ascending — cinematic + UI
    Station, // Orbital station — free exploration
}
