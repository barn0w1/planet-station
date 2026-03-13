# 🛸 Space Elevator Simulator — Claude Code Instructions

## Project Overview

Build a **3D web game** in Rust using Bevy that runs in the browser via WebAssembly.
The player rides a space elevator from Earth's surface to an orbital space station,
then explores and plays mini-games on the station.

**Visual Style:** Inspired by Astroneer — low-poly, rounded geometry, vibrant pastel
colors, soft lighting, no harsh realism. Think "toy-like but beautiful."

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (stable) |
| Game Engine | Bevy 0.13 |
| 3D Renderer | Bevy PBR (`bevy/webgl2` feature) |
| Build Target | `wasm32-unknown-unknown` |
| WASM Bindgen | `wasm-bindgen` + `wasm-pack` |
| Host Page | Single `index.html` with `<canvas id="bevy-canvas">` |

---

## Project Structure

```
space-elevator/
├── Cargo.toml
├── index.html            # Single-file host page (inline CSS + JS)
├── build.sh              # One-command build script
├── src/
│   ├── main.rs           # App entry + plugin registration
│   ├── elevator.rs       # Elevator ride system
│   ├── station.rs        # Space station scene + interactions
│   ├── earth.rs          # Earth scene (ground level)
│   ├── ui.rs             # HUD, altitude meter, prompts
│   ├── audio.rs          # Sound effects (bevy_audio)
│   ├── assets.rs         # Procedural mesh helpers
│   └── game_state.rs     # GameState enum + transitions
```

---

## Step-by-Step Build Plan

### STEP 1 — Project Scaffold

```bash
cargo new space-elevator
cd space-elevator
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli   # if not installed
```

**Cargo.toml** must include:

```toml
[package]
name = "space-elevator"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
bevy = { version = "0.13", default-features = false, features = [
  "bevy_core_pipeline",
  "bevy_pbr",
  "bevy_render",
  "bevy_asset",
  "bevy_winit",
  "bevy_text",
  "bevy_ui",
  "bevy_sprite",
  "x11",
  "webgl2",
  "png",
] }
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["Window", "Document"] }
rand = { version = "0.8", features = ["wasm-bindgen"] }

[profile.release]
opt-level = "z"
lto        = true
codegen-units = 1
```

---

### STEP 2 — Game State Machine

Define a `GameState` enum used throughout:

```rust
// src/game_state.rs
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Earth,        // Ground — player sees Earth, enters elevator
    Riding,       // Elevator ascending — cinematic + UI
    Station,      // Orbital station — free exploration
}
```

Register with:
```rust
app.init_state::<GameState>()
```

---

### STEP 3 — Earth Scene (`earth.rs`)

Build the ground-level scene the player starts in.

**Requirements:**
- Large flat terrain disc (radius 80), dark teal color `#1a3a4a`
- Rounded low-poly mountains on the horizon (use `Sphere` with scale tricks or custom mesh)
- The elevator **cable**: a thin tall cylinder `(radius 0.15, height 5000)`, slightly glowing cyan
- Elevator **cabin** mesh: rounded box shape, teal/white color, visible at base
- Atmospheric haze: large sphere around the scene with alpha material, sky-blue tinted
- Sky: gradient from deep blue at horizon to near-black above (use background color + fog)
- 3 decorative low-poly trees (cones + cylinders) near the base
- **Press [SPACE] or click** the cabin to transition → `GameState::Riding`

**Camera:** Fixed 3rd-person, slightly above and behind the cabin. Use a smooth look-at toward cabin.

**Lighting:**
- Directional "sun" light, warm yellow, angle 30° from horizon
- Ambient light, pale blue, intensity 0.3

**Color palette (Astroneer-inspired):**
```
Ground:      #1B3A4B  (dark teal)
Cabin:       #E8F4F8  (near-white with blue tint)
Cable:       #00E5FF  (cyan, emissive)
Sky top:     #020818  (near black)
Sky horizon: #0A2A6E  (deep blue)
Mountains:   #0F2D3D  (darker teal)
Trees:       #2D6A4F  (muted green)
```

---

### STEP 4 — Elevator Ride (`elevator.rs`)

The core cinematic experience.

**Requirements:**
- Cabin ascends along the cable at `ELEVATOR_SPEED = 12.0` units/second
- Camera follows cabin, locked slightly below and to the side, looking upward along cable
- Total cable height: `5000` units (represents ~36,000 km GEO symbolically)
- **Altitude HUD** displayed in km (map 0–5000 units → 0–36,000 km)
- **Phase transitions** based on altitude:

| Altitude (units) | Phase | Sky color | Effect |
|---|---|---|---|
| 0–200 | Troposphere | Blue gradient | Clouds visible |
| 200–600 | Stratosphere | Darkening blue | Clouds fade out |
| 600–1200 | Mesosphere | Dark blue-purple | Stars appear |
| 1200–2000 | Thermosphere | Near-black, stars bright | Earth glow visible below |
| 2000–5000 | Space | Pure black | Stars full, Earth disc visible |

- **Procedural stars:** 2000 `Mesh::from(shape::UVSphere { radius: 0.4 .. })` scattered at radius 3000 from origin, emissive white/blue
- **Earth disc:** As cabin rises above 1200 units, spawn a large flat sphere below (emissive, blue-green) to represent Earth from space
- **Clouds:** 40 flat disc meshes (white, semi-transparent) at altitude 150–400, drift slowly on X/Z
- **Space station visible** from altitude 4500+ — show glowing shape ahead
- Arrival at altitude 5000 → transition to `GameState::Station`

**Controls during ride:**
- `[W]/[S]` or `[↑]/[↓]` — speed up / slow down elevator (0.5× to 3× speed)
- `[←]/[→]` — rotate camera around cable (360°)
- `[ESC]` — pause menu

---

### STEP 5 — Space Station (`station.rs`)

Free-roam 3D exploration on an orbital station.

**Station layout (procedural geometry, no external assets):**
```
Central Hub (large sphere, r=8, metallic white)
    ├── 4 Habitat Arms (cylinders, length 20, radius 2) at 90° intervals
    │     └── each ends in a Habitat Pod (rounded box)
    ├── 2 Solar Panel Wings (flat boxes, 30×15×0.3, gold/yellow)
    ├── Observation Dome (hemisphere on top of hub, glass-blue, alpha 0.6)
    └── Docking Port (where elevator cable connects, bottom of hub)
```

**Player character:**
- Simple capsule mesh (white, rounded), first/third-person toggle with `[V]`
- WASD movement in 3D space (low-gravity feel: float slightly, smooth deceleration)
- `[SPACE]` to jump / float upward; `[SHIFT]` to descend

**Interactable objects (highlight on approach within 3 units):**

| Object | Location | Interaction |
|--------|----------|-------------|
| 🔭 Telescope | Observation Dome | Look at Earth — zoom camera, show "Earth from 36,000 km" text |
| 🌱 Plant Pod | Habitat Arm A | Water the plant — simple grow animation |
| 🎮 Arcade Cabinet | Habitat Arm B | Play a simple mini-game (see STEP 6) |
| 🔧 Control Panel | Hub center | Show station status readout (fake telemetry) |
| 🪟 Window | Each habitat | Look out — parallax star background |
| ☕ Coffee Maker | Habitat Arm C | "Brew zero-g coffee" — particle effect |

**Interaction system:**
- Proximity detection: when player within 3 units of interactable, show `[E] Interact` prompt
- Press `[E]` to trigger interaction
- Each interaction plays a sound cue and a short animation

**Lighting:**
- No ambient light (space is dark)
- 2 point lights inside station (warm white, intensity 800)
- Sun directional light from one side (harsh, no atmosphere)
- Station windows cast light rectangles (use spot lights aimed inward)

---

### STEP 6 — Arcade Mini-Game (inside Station)

A simple self-contained game within the game, accessible from the arcade cabinet.

**Game: "Asteroid Deflector"**
- 2D-style mini-game rendered in a `Camera2d` viewport overlaid on screen
- Asteroids (irregular polygon meshes) drift toward the station
- Player clicks/taps asteroids to "zap" them (they shatter into smaller pieces)
- Score counter, 60-second timer
- High score saved to `web_sys::window().local_storage()`

---

### STEP 7 — UI System (`ui.rs`)

Use Bevy UI (not egui) for all overlays.

**Earth scene UI:**
- Bottom center: `"[SPACE] Enter Elevator"` prompt, pulsing alpha animation
- Top left: small logo text "SPACE ELEVATOR SIM"

**Riding UI:**
- Left side vertical bar: altitude meter (0–36,000 km label, fill animates upward)
- Top center: current phase name (e.g., "THERMOSPHERE"), fade in/out
- Bottom: speed indicator + `[↑]/[↓] Speed`
- Right side: mini Earth-to-Station diagram showing current position dot

**Station UI:**
- Top right: compass / mini-map showing station layout from above
- Bottom center: context prompt `[E] <action name>` when near interactable
- Top left: "ORBITAL STATION ALPHA" title, small clock showing mission time

**Style guide:**
- Font: Use `bevy/default_font` or embed a monospace TTF (share via `embedded_asset!`)
- Colors: White text `#FFFFFF`, accent cyan `#00E5FF`, background panels dark `rgba(0,0,0,0.5)`
- All panels use rounded corners (`UiRect` with border radius)
- Subtle scanline overlay (CSS on canvas or shader)

---

### STEP 8 — Audio (`audio.rs`)

Use `bevy::audio`. Embed audio as bytes using `include_bytes!` with generated tones,
OR use bevy's `AudioSource` with `.ogg` files in `assets/audio/`.

**Required sounds:**
| Event | Sound description |
|-------|------------------|
| Elevator start | Low hum ramp-up |
| Riding ambient | Constant low-frequency vibration |
| Phase change | Chime / notification tone |
| Station arrival | Docking clunk + success jingle |
| Interaction | Short UI click |
| Mini-game zap | Laser pew |

If generating audio procedurally: use a simple sine-wave generator with `web_sys` AudioContext
as a fallback — implement in `audio.rs` with conditional compilation `#[cfg(target_arch = "wasm32")]`.

---

### STEP 9 — Build & Deploy

**`build.sh`:**
```bash
#!/bin/bash
set -e

echo "🔨 Building WASM..."
cargo build --target wasm32-unknown-unknown --release

echo "🔗 Running wasm-bindgen..."
wasm-bindgen \
  target/wasm32-unknown-unknown/release/space_elevator.wasm \
  --out-dir web/pkg \
  --target web \
  --no-typescript

echo "📦 Optimizing with wasm-opt..."
wasm-opt -Oz web/pkg/space_elevator_bg.wasm -o web/pkg/space_elevator_bg.wasm || true

echo "✅ Build complete! Serve the web/ directory."
echo "   python3 -m http.server 8080 --directory web"
```

**`index.html`** (place in `web/`):
```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>🛸 Space Elevator Simulator</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body { background: #000; display: flex; align-items: center;
           justify-content: center; height: 100vh; overflow: hidden; }
    canvas#bevy-canvas { width: 100vw; height: 100vh; display: block; }
    #loading { position: absolute; color: #00E5FF; font-family: monospace;
               font-size: 1.5rem; animation: pulse 1s infinite; }
    @keyframes pulse { 0%,100%{opacity:1} 50%{opacity:0.3} }
  </style>
</head>
<body>
  <div id="loading">Loading Space Elevator...</div>
  <canvas id="bevy-canvas"></canvas>
  <script type="module">
    import init from './pkg/space_elevator.js';
    await init();
    document.getElementById('loading').remove();
  </script>
</body>
</html>
```

---

## Visual Style Reference

### Do's ✅
- Rounded geometry: prefer `UVSphere`, `Capsule`, rounded boxes
- Flat / slightly metallic PBR materials — no textures, color only
- Emissive glow on key objects (cable, stars, indicators)
- Soft point lights inside enclosed spaces
- Low polygon count (stylized, not detailed)
- Pastel / saturated colors with dark backgrounds

### Don'ts ❌
- No photorealistic textures
- No sharp angular geometry (avoid raw `Box` meshes without rounding)
- No bloom overload (subtle glow only)
- No real-world accurate scale (symbolic scale is fine)
- No complex shaders — keep to standard Bevy PBR

---

## Implementation Order

Execute in this order to always have a runnable game:

1. `game_state.rs` — state machine (no visuals yet)
2. `main.rs` — App setup, window, camera, state transitions
3. `earth.rs` — Earth scene (first playable moment)
4. `elevator.rs` — Ride system (second playable moment)
5. `station.rs` — Station scene, player movement
6. `ui.rs` — All HUD elements
7. `assets.rs` — Shared mesh/material helpers (refactor as needed)
8. `audio.rs` — Sound (add last, non-blocking)
9. Mini-game in `station.rs` (bonus, implement last)

**After each step:** run `cargo check --target wasm32-unknown-unknown` to verify WASM compatibility.

---

## Common WASM Gotchas

- **No `std::time::Instant`** in WASM — use `bevy::time::Time` resource only
- **No filesystem access** — all assets must be embedded or loaded via HTTP
- **No threads** — disable Bevy's task pool parallelism:
  ```rust
  .insert_resource(TaskPoolOptions {
      compute: TaskPoolThreadAssignmentPolicy { min_threads: 1, max_threads: 1, percent: 1.0 },
      ..default()
  })
  ```
- **`wasm-bindgen` version** must match `wasm-bindgen-cli` version exactly
- **Audio requires user gesture** in browsers — start audio only after first click/keypress
- **Canvas must exist before Bevy starts** — ensure `<canvas id="bevy-canvas">` is in DOM

---

## Definition of Done

The game is complete when:
- [ ] Player can start on Earth and see the elevator cable
- [ ] Pressing SPACE starts the elevator ride
- [ ] Sky transitions through atmosphere phases with visual changes
- [ ] Player arrives at and can explore the space station
- [ ] At least 3 interactable objects work on the station
- [ ] Altitude HUD is visible during the ride
- [ ] Game runs in Chrome/Firefox/Safari without errors
- [ ] `build.sh` produces a deployable `web/` directory
- [ ] FPS stays above 30 on a mid-range laptop

---

## Running Locally

```bash
# Development (native, faster iteration)
cargo run

# WASM build
./build.sh

# Serve locally
python3 -m http.server 8080 --directory web
# Open http://localhost:8080
```

---

*Good luck! Build it step by step, keep it fun, keep it low-poly.*