mod game_state;
mod assets;
mod earth;
mod elevator;
mod station;
mod ui;
pub mod audio;

use bevy::prelude::*;
use game_state::GameState;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

/// Main entry point — called by the native binary and by WASM via wasm_bindgen.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Space Elevator Simulator".to_string(),
                    // On WASM, target the canvas element in index.html
                    #[cfg(target_arch = "wasm32")]
                    canvas: Some("#bevy-canvas".to_string()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            })
            .set(bevy::log::LogPlugin {
                level: bevy::log::Level::WARN,
                ..default()
            }),
    );

    // WASM is single-threaded — restrict task pools to avoid panics
    #[cfg(target_arch = "wasm32")]
    app.insert_resource(bevy::core::TaskPoolOptions {
        min_total_threads: 1,
        max_total_threads: 1,
        ..default()
    });

    app
        // Global ambient light (pale blue)
        .insert_resource(AmbientLight {
            color: Color::rgb(0.7, 0.85, 1.0),
            brightness: 0.3,
        })
        // Background / clear color (starts as deep-blue sky)
        .insert_resource(ClearColor(Color::rgb(0.008, 0.031, 0.094)))
        // Game state machine
        .init_state::<GameState>()
        // Plugins
        .add_plugins((
            earth::EarthPlugin,
            elevator::ElevatorPlugin,
            station::StationPlugin,
            ui::UiPlugin,
            audio::AudioPlugin,
        ))
        .run();
}
