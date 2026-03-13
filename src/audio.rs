use bevy::prelude::*;

/// Audio plugin — currently a stub.
/// Full procedural audio would use web_sys AudioContext on WASM.
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, _app: &mut App) {
        // Audio requires a user gesture in browsers. A full implementation would:
        // 1. Listen for the first click/keypress event.
        // 2. Create a web_sys::AudioContext.
        // 3. Generate oscillator nodes for each sound cue.
        // This stub satisfies the module interface without causing compile errors
        // on either native or WASM targets.
    }
}
