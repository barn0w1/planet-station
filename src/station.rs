use bevy::prelude::*;
use crate::game_state::GameState;
use crate::assets::palette;

// ── Constants ─────────────────────────────────────────────────────────────────

const PLAYER_SPEED: f32 = 8.0;
const GRAVITY_SCALE: f32 = 0.3;    // low-gravity feel
const INTERACT_RANGE: f32 = 5.0;

// ── Components ────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct StationSceneRoot;

#[derive(Component)]
pub struct StationPlayer {
    pub velocity: Vec3,
    pub on_ground: bool,
}

#[derive(Component)]
pub struct StationCamera;

#[derive(Component, Clone)]
pub struct Interactable {
    pub label: &'static str,
    pub action: InteractAction,
    pub triggered: bool,
}

#[derive(Clone, Debug)]
pub enum InteractAction {
    Telescope,
    PlantPod,
    ControlPanel,
    CoffeeMaker,
    Window,
}

/// Mission clock
#[derive(Resource, Default)]
pub struct MissionTime {
    pub elapsed: f32,
}

// ── Plugin ────────────────────────────────────────────────────────────────────

pub struct StationPlugin;

impl Plugin for StationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MissionTime>()
            .add_systems(OnEnter(GameState::Station), setup_station)
            .add_systems(OnExit(GameState::Station), cleanup_station)
            .add_systems(
                Update,
                (
                    move_player,
                    update_station_camera,
                    check_interactions,
                    tick_mission_time,
                )
                    .run_if(in_state(GameState::Station)),
            );
    }
}

// ── Setup ─────────────────────────────────────────────────────────────────────

fn setup_station(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut clear_color: ResMut<ClearColor>,
    mut mission_time: ResMut<MissionTime>,
) {
    clear_color.0 = Color::rgb(0.0, 0.0, 0.0); // pure black — space
    mission_time.elapsed = 0.0;

    // ── Station origin is at y=5010 ───────────────────────────────────────────
    let base = Vec3::new(0.0, 5010.0, 0.0);

    let station_white_mat = materials.add(StandardMaterial {
        base_color: palette::STATION_WHITE,
        metallic: 0.4,
        perceptual_roughness: 0.5,
        ..default()
    });
    let gold_mat = materials.add(StandardMaterial {
        base_color: palette::SOLAR_GOLD,
        metallic: 0.6,
        perceptual_roughness: 0.4,
        ..default()
    });
    let dome_mat = materials.add(StandardMaterial {
        base_color: palette::DOME_GLASS,
        alpha_mode: AlphaMode::Blend,
        metallic: 0.1,
        perceptual_roughness: 0.2,
        ..default()
    });

    // ── Central hub (large sphere r=8) ───────────────────────────────────────
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::UVSphere { radius: 8.0, sectors: 16, stacks: 12 }.into()),
            material: station_white_mat.clone(),
            transform: Transform::from_translation(base),
            ..default()
        },
        StationSceneRoot,
    ));

    // ── Observation dome (hemisphere top) ────────────────────────────────────
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::UVSphere { radius: 5.0, sectors: 12, stacks: 6 }.into()),
            material: dome_mat,
            transform: Transform::from_translation(base + Vec3::new(0.0, 10.0, 0.0)),
            ..default()
        },
        StationSceneRoot,
    ));

    // ── 4 Habitat Arms + pods (at 90° intervals) ──────────────────────────────
    let arm_angles: &[(f32, &str)] = &[
        (0.0, "A"),
        (std::f32::consts::FRAC_PI_2, "B"),
        (std::f32::consts::PI, "C"),
        (3.0 * std::f32::consts::FRAC_PI_2, "D"),
    ];
    for (angle, _label) in arm_angles {
        let dir = Vec3::new(angle.cos(), 0.0, angle.sin());
        let arm_center = base + dir * 14.0;
        let pod_center = base + dir * 26.0;

        // Arm cylinder
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(
                    shape::Cylinder { radius: 2.0, height: 20.0, resolution: 10, segments: 1 }.into(),
                ),
                material: station_white_mat.clone(),
                transform: Transform::from_translation(arm_center).with_rotation(
                    Quat::from_rotation_z(std::f32::consts::FRAC_PI_2) *
                    Quat::from_rotation_y(*angle),
                ),
                ..default()
            },
            StationSceneRoot,
        ));

        // Habitat pod (sphere at end)
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(shape::UVSphere { radius: 4.0, sectors: 12, stacks: 8 }.into()),
                material: station_white_mat.clone(),
                transform: Transform::from_translation(pod_center),
                ..default()
            },
            StationSceneRoot,
        ));
    }

    // ── Solar panel wings (2, along Z axis) ──────────────────────────────────
    for sign in [-1.0f32, 1.0f32] {
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(shape::Box::new(30.0, 0.3, 15.0).into()),
                material: gold_mat.clone(),
                transform: Transform::from_translation(base + Vec3::new(0.0, 2.0, sign * 40.0)),
                ..default()
            },
            StationSceneRoot,
        ));
    }

    // ── Docking port (bottom, where elevator cable connects) ──────────────────
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(
                shape::Cylinder { radius: 3.0, height: 4.0, resolution: 12, segments: 1 }.into(),
            ),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.3, 0.3, 0.35),
                metallic: 0.7,
                ..default()
            }),
            transform: Transform::from_translation(base + Vec3::new(0.0, -10.0, 0.0)),
            ..default()
        },
        StationSceneRoot,
    ));

    // ── Interactable objects ──────────────────────────────────────────────────

    // Telescope (Observation Dome)
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Cylinder { radius: 0.5, height: 3.0, resolution: 8, segments: 1 }.into()),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.2, 0.2, 0.3),
                metallic: 0.8,
                ..default()
            }),
            transform: Transform::from_translation(base + Vec3::new(0.0, 12.5, 0.0)),
            ..default()
        },
        Interactable {
            label: "Telescope",
            action: InteractAction::Telescope,
            triggered: false,
        },
        StationSceneRoot,
    ));

    // Plant Pod (Arm A pod center)
    let plant_pos = base + Vec3::new(26.0, 0.5, 0.0);
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::UVSphere { radius: 0.8, sectors: 8, stacks: 6 }.into()),
            material: materials.add(StandardMaterial {
                base_color: palette::TREE,
                emissive: Color::rgb(0.1, 0.4, 0.2),
                ..default()
            }),
            transform: Transform::from_translation(plant_pos),
            ..default()
        },
        Interactable {
            label: "Plant Pod",
            action: InteractAction::PlantPod,
            triggered: false,
        },
        StationSceneRoot,
    ));

    // Control Panel (hub center)
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Box::new(2.0, 1.0, 0.3).into()),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.1, 0.15, 0.25),
                emissive: Color::rgb(0.0, 0.5, 1.0),
                metallic: 0.6,
                ..default()
            }),
            transform: Transform::from_translation(base + Vec3::new(0.0, -4.0, 6.0)),
            ..default()
        },
        Interactable {
            label: "Control Panel",
            action: InteractAction::ControlPanel,
            triggered: false,
        },
        StationSceneRoot,
    ));

    // Coffee Maker (Arm C)
    let coffee_pos = base + Vec3::new(-24.0, 0.0, 0.0);
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Cylinder { radius: 0.4, height: 1.2, resolution: 8, segments: 1 }.into()),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.15, 0.1, 0.08),
                metallic: 0.5,
                ..default()
            }),
            transform: Transform::from_translation(coffee_pos),
            ..default()
        },
        Interactable {
            label: "Coffee Maker",
            action: InteractAction::CoffeeMaker,
            triggered: false,
        },
        StationSceneRoot,
    ));

    // ── Stars background (3000-unit sphere) ───────────────────────────────────
    {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let star_mat = materials.add(StandardMaterial {
            base_color: Color::rgb(0.95, 0.95, 1.0),
            emissive: Color::rgb(4.0, 4.0, 4.5),
            unlit: true,
            ..default()
        });
        let star_mesh =
            meshes.add(shape::UVSphere { radius: 0.4, sectors: 4, stacks: 3 }.into());

        for _ in 0..1500 {
            let phi: f32 = rng.gen_range(0.0f32..std::f32::consts::TAU);
            let cos_theta: f32 = rng.gen_range(-1.0f32..1.0);
            let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
            let pos = Vec3::new(
                3000.0 * sin_theta * phi.cos(),
                5010.0 + 3000.0 * cos_theta,
                3000.0 * sin_theta * phi.sin(),
            );
            commands.spawn((
                PbrBundle {
                    mesh: star_mesh.clone(),
                    material: star_mat.clone(),
                    transform: Transform::from_translation(pos),
                    ..default()
                },
                StationSceneRoot,
            ));
        }
    }

    // ── Lighting ──────────────────────────────────────────────────────────────
    // No ambient — space is dark
    // Interior point lights
    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                color: Color::rgb(1.0, 0.98, 0.95),
                intensity: 800.0,
                range: 30.0,
                ..default()
            },
            transform: Transform::from_translation(base + Vec3::new(0.0, -2.0, 0.0)),
            ..default()
        },
        StationSceneRoot,
    ));
    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                color: Color::rgb(0.9, 0.95, 1.0),
                intensity: 600.0,
                range: 25.0,
                ..default()
            },
            transform: Transform::from_translation(base + Vec3::new(0.0, 8.0, 0.0)),
            ..default()
        },
        StationSceneRoot,
    ));

    // Sun directional (harsh, no atmosphere)
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::rgb(1.0, 1.0, 0.95),
                illuminance: 100_000.0,
                shadows_enabled: false,
                ..default()
            },
            transform: Transform::from_rotation(Quat::from_euler(
                EulerRot::XYZ, -0.3, 0.8, 0.0,
            )),
            ..default()
        },
        StationSceneRoot,
    ));

    // ── Player (capsule) ───────────────────────────────────────────────────────
    // Spawn player at docking port entrance
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(
                shape::Capsule { radius: 0.5, depth: 1.5, ..default() }.into(),
            ),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.9, 0.92, 0.95),
                metallic: 0.1,
                ..default()
            }),
            transform: Transform::from_translation(base + Vec3::new(0.0, -8.0, 0.0)),
            ..default()
        },
        StationPlayer {
            velocity: Vec3::ZERO,
            on_ground: false,
        },
        StationSceneRoot,
    ));

    // ── Camera ────────────────────────────────────────────────────────────────
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(base + Vec3::new(0.0, -5.0, 20.0))
                .looking_at(base + Vec3::new(0.0, -8.0, 0.0), Vec3::Y),
            ..default()
        },
        StationCamera,
        StationSceneRoot,
    ));
}

// ── Systems ───────────────────────────────────────────────────────────────────

fn move_player(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut player_q: Query<(&mut Transform, &mut StationPlayer)>,
    cam_q: Query<&Transform, (With<StationCamera>, Without<StationPlayer>)>,
) {
    let dt = time.delta_seconds();
    let Ok(cam_t) = cam_q.get_single() else { return };

    // Compute camera-relative forward/right
    let cam_fwd = {
        let f = cam_t.forward();
        Vec3::new(f.x, 0.0, f.z).normalize_or_zero()
    };
    let cam_right = {
        let r = cam_t.right();
        Vec3::new(r.x, 0.0, r.z).normalize_or_zero()
    };

    for (mut t, mut player) in &mut player_q {
        let mut move_dir = Vec3::ZERO;
        if keys.pressed(KeyCode::W) { move_dir += cam_fwd; }
        if keys.pressed(KeyCode::S) { move_dir -= cam_fwd; }
        if keys.pressed(KeyCode::A) { move_dir -= cam_right; }
        if keys.pressed(KeyCode::D) { move_dir += cam_right; }

        if move_dir.length_squared() > 0.0 {
            move_dir = move_dir.normalize();
        }

        // Horizontal movement
        let horizontal_vel = move_dir * PLAYER_SPEED;

        // Vertical
        if keys.pressed(KeyCode::Space) {
            player.velocity.y = (player.velocity.y + 12.0 * dt).min(8.0);
        } else if keys.pressed(KeyCode::ShiftLeft) {
            player.velocity.y = (player.velocity.y - 12.0 * dt).max(-8.0);
        } else {
            // Low gravity drift back to zero
            player.velocity.y *= 1.0 - (GRAVITY_SCALE * dt * 3.0).min(1.0);
        }

        // Smooth deceleration on horizontal
        player.velocity.x = horizontal_vel.x;
        player.velocity.z = horizontal_vel.z;

        t.translation += player.velocity * dt;

        // Station floor constraint (inside hub: y >= base_y - 8)
        let floor_y = 5010.0 - 9.0;
        if t.translation.y < floor_y {
            t.translation.y = floor_y;
            player.velocity.y = 0.0;
        }
    }
}

fn update_station_camera(
    player_q: Query<&Transform, With<StationPlayer>>,
    mut cam_q: Query<&mut Transform, (With<StationCamera>, Without<StationPlayer>)>,
) {
    let Ok(pt) = player_q.get_single() else { return };
    let target = pt.translation;

    for mut ct in &mut cam_q {
        let desired = target + Vec3::new(0.0, 6.0, 18.0);
        ct.translation = ct.translation.lerp(desired, 0.08);
        ct.look_at(target + Vec3::new(0.0, 1.0, 0.0), Vec3::Y);
    }
}

fn check_interactions(
    keys: Res<Input<KeyCode>>,
    player_q: Query<&Transform, With<StationPlayer>>,
    mut interactables: Query<(&Transform, &mut Interactable), Without<StationPlayer>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(pt) = player_q.get_single() else { return };
    let player_pos = pt.translation;

    for (it, mut interactable) in &mut interactables {
        let dist = player_pos.distance(it.translation);
        if dist <= INTERACT_RANGE && keys.just_pressed(KeyCode::E) && !interactable.triggered {
            interactable.triggered = true;
            trigger_interaction(&interactable.action, it.translation, &mut commands, &mut meshes, &mut materials);
        }
    }
}

fn trigger_interaction(
    action: &InteractAction,
    pos: Vec3,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    match action {
        InteractAction::PlantPod => {
            // Grow effect — spawn a bigger sphere briefly
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(shape::UVSphere { radius: 1.4, sectors: 8, stacks: 6 }.into()),
                    material: materials.add(StandardMaterial {
                        base_color: Color::rgb(0.2, 0.7, 0.3),
                        emissive: Color::rgb(0.3, 1.0, 0.4),
                        ..default()
                    }),
                    transform: Transform::from_translation(pos + Vec3::Y * 1.0),
                    ..default()
                },
                StationSceneRoot,
            ));
        }
        InteractAction::CoffeeMaker => {
            // Particle effect — spawn a few rising spheres
            for i in 0..5 {
                let offset = Vec3::new(i as f32 * 0.1 - 0.2, i as f32 * 0.5 + 1.0, 0.0);
                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(shape::UVSphere { radius: 0.12, sectors: 6, stacks: 4 }.into()),
                        material: materials.add(StandardMaterial {
                            base_color: Color::rgb(0.7, 0.5, 0.3),
                            emissive: Color::rgb(0.4, 0.3, 0.1),
                            alpha_mode: AlphaMode::Blend,
                            ..default()
                        }),
                        transform: Transform::from_translation(pos + offset),
                        ..default()
                    },
                    StationSceneRoot,
                ));
            }
        }
        // Other interactions are handled / displayed via UI
        _ => {}
    }
}

fn tick_mission_time(time: Res<Time>, mut mt: ResMut<MissionTime>) {
    mt.elapsed += time.delta_seconds();
}

// ── Cleanup ───────────────────────────────────────────────────────────────────

fn cleanup_station(mut commands: Commands, query: Query<Entity, With<StationSceneRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
