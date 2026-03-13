use bevy::prelude::*;
use crate::assets::palette;
use crate::game_state::GameState;

// ── Components ──────────────────────────────────────────────────────────────

/// Marks the elevator cabin mesh in the Earth scene
#[derive(Component)]
pub struct EarthCabin;

/// Marks entities that belong only to the Earth scene (cleanup on exit)
#[derive(Component)]
pub struct EarthSceneRoot;

/// Marker for the Earth scene camera
#[derive(Component)]
pub struct EarthCamera;

// ── Plugin ───────────────────────────────────────────────────────────────────

pub struct EarthPlugin;

impl Plugin for EarthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Earth), setup_earth)
            .add_systems(OnExit(GameState::Earth), cleanup_earth)
            .add_systems(
                Update,
                (handle_earth_input, animate_cabin_prompt)
                    .run_if(in_state(GameState::Earth)),
            );
    }
}

// ── Setup ─────────────────────────────────────────────────────────────────────

fn setup_earth(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // ── Ground disc ──────────────────────────────────────────────────────────
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cylinder { radius: 80.0, height: 0.5, resolution: 32, segments: 1 })),
            material: materials.add(StandardMaterial {
                base_color: palette::GROUND,
                perceptual_roughness: 0.9,
                metallic: 0.0,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, -0.25, 0.0),
            ..default()
        },
        EarthSceneRoot,
    ));

    // ── Elevator cable (thin glowing cyan cylinder) ───────────────────────────
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cylinder { radius: 0.15, height: 5000.0, resolution: 8, segments: 1 })),
            material: materials.add(StandardMaterial {
                base_color: palette::CABLE,
                emissive: Color::rgb(0.0, 3.0, 4.0),
                unlit: false,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 2500.0, 0.0),
            ..default()
        },
        EarthSceneRoot,
    ));

    // ── Elevator cabin ───────────────────────────────────────────────────────
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 1.2, sectors: 12, stacks: 8 })),
            material: materials.add(StandardMaterial {
                base_color: palette::CABIN,
                metallic: 0.2,
                perceptual_roughness: 0.4,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 1.5, 0.0),
            ..default()
        },
        EarthCabin,
        EarthSceneRoot,
    ));

    // ── Low-poly mountains (horizon ring) ────────────────────────────────────
    let mountain_positions: &[(f32, f32, f32, f32)] = &[
        ( 60.0,  3.0,  0.0,  5.0),
        (-55.0,  4.0, 25.0,  7.0),
        ( 20.0,  5.0, 65.0,  6.0),
        (-40.0,  3.5,-60.0,  4.5),
        ( 70.0,  2.5,-30.0,  3.5),
        (  0.0,  4.0,-70.0,  5.5),
        (-70.0,  3.0, -5.0,  4.0),
        ( 45.0,  6.0, 50.0,  8.0),
    ];

    for (x, _y, z, r) in mountain_positions {
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere { radius: *r, sectors: 6, stacks: 4 })),
                material: materials.add(StandardMaterial {
                    base_color: palette::MOUNTAIN,
                    perceptual_roughness: 0.85,
                    ..default()
                }),
                transform: Transform::from_xyz(*x, *r * 0.3, *z),
                ..default()
            },
            EarthSceneRoot,
        ));
    }

    // ── Low-poly trees ────────────────────────────────────────────────────────
    let tree_positions: &[(f32, f32)] = &[
        (8.0, 12.0),
        (-10.0, 8.0),
        (14.0, -6.0),
    ];

    for (tx, tz) in tree_positions {
        // Trunk
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cylinder { radius: 0.3, height: 3.0, resolution: 6, segments: 1 })),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.35, 0.22, 0.1),
                    perceptual_roughness: 0.9,
                    ..default()
                }),
                transform: Transform::from_xyz(*tx, 1.5, *tz),
                ..default()
            },
            EarthSceneRoot,
        ));
        // Foliage (low-poly sphere)
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 1.8, sectors: 6, stacks: 4 })),
                material: materials.add(StandardMaterial {
                    base_color: palette::TREE,
                    perceptual_roughness: 0.8,
                    ..default()
                }),
                transform: Transform::from_xyz(*tx, 4.5, *tz),
                ..default()
            },
            EarthSceneRoot,
        ));
    }

    // ── Atmospheric haze sphere ───────────────────────────────────────────────
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 200.0, sectors: 16, stacks: 12 })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgba(0.039, 0.165, 0.431, 0.18),
                alpha_mode: AlphaMode::Blend,
                cull_mode: None,
                double_sided: true,
                unlit: true,
                ..default()
            }),
            ..default()
        },
        EarthSceneRoot,
    ));

    // ── Lighting ──────────────────────────────────────────────────────────────
    // Sun directional light
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::rgb(1.0, 0.95, 0.8),
                illuminance: 25_000.0,
                shadows_enabled: false,
                ..default()
            },
            transform: Transform::from_rotation(Quat::from_euler(
                EulerRot::XYZ,
                -0.5,
                0.4,
                0.0,
            )),
            ..default()
        },
        EarthSceneRoot,
    ));

    // Ambient light is set globally in main

    // ── Camera (fixed 3rd-person behind cabin) ────────────────────────────────
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(8.0, 6.0, 18.0)
                .looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y),
            ..default()
        },
        EarthCamera,
        EarthSceneRoot,
    ));
}

// ── Systems ──────────────────────────────────────────────────────────────────

fn handle_earth_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Left) {
        next_state.set(GameState::Riding);
    }
}

/// Pulsing alpha on the enter-elevator prompt (handled in ui.rs, but we drive
/// the time-based alpha from here via a dedicated component query; the actual
/// text entity is spawned in ui.rs).
fn animate_cabin_prompt(
    time: Res<Time>,
    mut query: Query<&mut BackgroundColor, With<crate::ui::EnterPrompt>>,
) {
    for mut bg in &mut query {
        // Pulse opacity between 0.3 and 1.0
        let alpha = 0.3 + 0.7 * (time.elapsed_seconds() * 2.0).sin().abs();
        bg.0 = bg.0.with_a(alpha);
    }
}

// ── Cleanup ───────────────────────────────────────────────────────────────────

fn cleanup_earth(mut commands: Commands, query: Query<Entity, With<EarthSceneRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
