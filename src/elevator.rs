use bevy::prelude::*;
use crate::game_state::GameState;

// ── Constants ─────────────────────────────────────────────────────────────────

pub const ELEVATOR_SPEED: f32 = 12.0;
pub const CABLE_HEIGHT: f32 = 5000.0;

/// Maps altitude units (0–5000) to km (0–36000)
pub fn altitude_to_km(altitude: f32) -> f32 {
    altitude / CABLE_HEIGHT * 36_000.0
}

// ── Atmosphere phases ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum AtmospherePhase {
    Troposphere,   // 0–200
    Stratosphere,  // 200–600
    Mesosphere,    // 600–1200
    Thermosphere,  // 1200–2000
    Space,         // 2000–5000
}

impl AtmospherePhase {
    pub fn from_altitude(alt: f32) -> Self {
        if alt < 200.0 {
            AtmospherePhase::Troposphere
        } else if alt < 600.0 {
            AtmospherePhase::Stratosphere
        } else if alt < 1200.0 {
            AtmospherePhase::Mesosphere
        } else if alt < 2000.0 {
            AtmospherePhase::Thermosphere
        } else {
            AtmospherePhase::Space
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            AtmospherePhase::Troposphere => "TROPOSPHERE",
            AtmospherePhase::Stratosphere => "STRATOSPHERE",
            AtmospherePhase::Mesosphere => "MESOSPHERE",
            AtmospherePhase::Thermosphere => "THERMOSPHERE",
            AtmospherePhase::Space => "SPACE",
        }
    }

    pub fn sky_color(&self) -> Color {
        match self {
            AtmospherePhase::Troposphere => Color::rgb(0.039, 0.165, 0.431),
            AtmospherePhase::Stratosphere => Color::rgb(0.02, 0.08, 0.25),
            AtmospherePhase::Mesosphere => Color::rgb(0.01, 0.02, 0.12),
            AtmospherePhase::Thermosphere => Color::rgb(0.005, 0.005, 0.05),
            AtmospherePhase::Space => Color::rgb(0.0, 0.0, 0.0),
        }
    }
}

// ── Resources & Components ───────────────────────────────────────────────────

#[derive(Resource)]
pub struct ElevatorState {
    pub altitude: f32,           // 0–5000 units
    pub speed_multiplier: f32,   // 0.5–3.0
    pub phase: AtmospherePhase,
    pub camera_angle: f32,       // radians around Y axis
    pub stars_spawned: bool,
    pub earth_disc_spawned: bool,
    pub station_preview_spawned: bool,
}

impl Default for ElevatorState {
    fn default() -> Self {
        Self {
            altitude: 0.0,
            speed_multiplier: 1.0,
            phase: AtmospherePhase::Troposphere,
            camera_angle: 0.0,
            stars_spawned: false,
            earth_disc_spawned: false,
            station_preview_spawned: false,
        }
    }
}

/// Elevator cabin entity tag
#[derive(Component)]
pub struct ElevatorCabin;

/// Camera attached to elevator
#[derive(Component)]
pub struct ElevatorCamera;

/// Cloud entity
#[derive(Component)]
pub struct Cloud {
    pub drift_x: f32,
    pub drift_z: f32,
}

/// Star entity (spawned once)
#[derive(Component)]
pub struct Star;

/// Earth disc (spawned when altitude > 1200)
#[derive(Component)]
pub struct EarthDisc;

/// Station preview (spawned when altitude > 4500)
#[derive(Component)]
pub struct StationPreview;

/// Marks all riding-scene entities for cleanup
#[derive(Component)]
pub struct RideSceneRoot;

// ── Plugin ────────────────────────────────────────────────────────────────────

pub struct ElevatorPlugin;

impl Plugin for ElevatorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ElevatorState>()
            .add_systems(OnEnter(GameState::Riding), setup_ride)
            .add_systems(OnExit(GameState::Riding), cleanup_ride)
            .add_systems(
                Update,
                (
                    ascend_elevator,
                    update_camera,
                    drift_clouds,
                    spawn_stars_once,
                    spawn_earth_disc_once,
                    spawn_station_preview_once,
                    handle_ride_input,
                    check_arrival,
                )
                    .run_if(in_state(GameState::Riding)),
            );
    }
}

// ── Setup ─────────────────────────────────────────────────────────────────────

fn setup_ride(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut elevator: ResMut<ElevatorState>,
    mut clear_color: ResMut<ClearColor>,
) {
    *elevator = ElevatorState::default();
    clear_color.0 = AtmospherePhase::Troposphere.sky_color();

    // ── Elevator cabin ───────────────────────────────────────────────────────
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 1.2, sectors: 12, stacks: 8 })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.91, 0.957, 0.975),
                metallic: 0.2,
                perceptual_roughness: 0.4,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 1.5, 0.0),
            ..default()
        },
        ElevatorCabin,
        RideSceneRoot,
    ));

    // ── Elevator cable ────────────────────────────────────────────────────────
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cylinder { radius: 0.12, height: 5000.0, resolution: 8, segments: 1 })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.0, 0.898, 1.0),
                emissive: Color::rgb(0.0, 2.0, 3.0),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 2500.0, 0.0),
            ..default()
        },
        RideSceneRoot,
    ));

    // ── Sun light ─────────────────────────────────────────────────────────────
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::rgb(1.0, 0.95, 0.8),
                illuminance: 20_000.0,
                shadows_enabled: false,
                ..default()
            },
            transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.4, 0.0)),
            ..default()
        },
        RideSceneRoot,
    ));

    // ── Clouds (40 flat discs at altitude 150–400) ────────────────────────────
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..40 {
        let cx = rng.gen_range(-80.0f32..80.0);
        let cy = rng.gen_range(150.0f32..400.0);
        let cz = rng.gen_range(-80.0f32..80.0);
        let cr = rng.gen_range(4.0f32..12.0);
        let dx: f32 = rng.gen_range(-0.5..0.5);
        let dz: f32 = rng.gen_range(-0.5..0.5);

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cylinder { radius: cr, height: 0.6, resolution: 12, segments: 1 })),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgba(1.0, 1.0, 1.0, 0.65),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                }),
                transform: Transform::from_xyz(cx, cy, cz),
                ..default()
            },
            Cloud { drift_x: dx, drift_z: dz },
            RideSceneRoot,
        ));
    }

    // ── Camera ────────────────────────────────────────────────────────────────
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(12.0, 0.0, 0.0)
                .looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y),
            ..default()
        },
        ElevatorCamera,
        RideSceneRoot,
    ));
}

// ── Systems ───────────────────────────────────────────────────────────────────

fn ascend_elevator(
    time: Res<Time>,
    mut elevator: ResMut<ElevatorState>,
    mut cabin_q: Query<&mut Transform, With<ElevatorCabin>>,
    mut clear_color: ResMut<ClearColor>,
) {
    let speed = ELEVATOR_SPEED * elevator.speed_multiplier;
    elevator.altitude = (elevator.altitude + speed * time.delta_seconds()).min(CABLE_HEIGHT);

    // Update cabin position
    for mut t in &mut cabin_q {
        t.translation.y = elevator.altitude + 1.5;
    }

    // Update sky color
    let new_phase = AtmospherePhase::from_altitude(elevator.altitude);
    if new_phase != elevator.phase {
        elevator.phase = new_phase;
    }
    // Smooth lerp toward target sky color
    let target = elevator.phase.sky_color();
    let current = clear_color.0;
    let t_factor = (time.delta_seconds() * 0.8).min(1.0);
    clear_color.0 = Color::rgb(
        current.r() + (target.r() - current.r()) * t_factor,
        current.g() + (target.g() - current.g()) * t_factor,
        current.b() + (target.b() - current.b()) * t_factor,
    );
}

fn handle_ride_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut elevator: ResMut<ElevatorState>,
    time: Res<Time>,
) {
    // Speed control
    if keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW) {
        elevator.speed_multiplier = (elevator.speed_multiplier + 1.5 * time.delta_seconds()).min(3.0);
    }
    if keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS) {
        elevator.speed_multiplier = (elevator.speed_multiplier - 1.5 * time.delta_seconds()).max(0.5);
    }

    // Camera rotation
    if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {
        elevator.camera_angle += 1.5 * time.delta_seconds();
    }
    if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
        elevator.camera_angle -= 1.5 * time.delta_seconds();
    }
}

fn update_camera(
    elevator: Res<ElevatorState>,
    mut cam_q: Query<&mut Transform, With<ElevatorCamera>>,
) {
    for mut t in &mut cam_q {
        let cabin_y = elevator.altitude + 1.5;
        let angle = elevator.camera_angle;
        let cam_x = 12.0 * angle.cos();
        let cam_z = 12.0 * angle.sin();
        t.translation = Vec3::new(cam_x, cabin_y - 2.0, cam_z);
        t.look_at(Vec3::new(0.0, cabin_y + 4.0, 0.0), Vec3::Y);
    }
}

fn drift_clouds(time: Res<Time>, mut q: Query<(&mut Transform, &Cloud)>) {
    for (mut t, cloud) in &mut q {
        t.translation.x += cloud.drift_x * time.delta_seconds();
        t.translation.z += cloud.drift_z * time.delta_seconds();
    }
}

fn spawn_stars_once(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut elevator: ResMut<ElevatorState>,
) {
    if elevator.stars_spawned || elevator.altitude < 600.0 {
        return;
    }
    elevator.stars_spawned = true;

    use rand::Rng;
    let mut rng = rand::thread_rng();
    let star_mat = materials.add(StandardMaterial {
        base_color: Color::rgb(0.95, 0.95, 1.0),
        emissive: Color::rgb(4.0, 4.0, 4.5),
        unlit: true,
        ..default()
    });
    let star_mesh = meshes.add(Mesh::from(shape::UVSphere { radius: 0.4, sectors: 4, stacks: 3 }));

    for _ in 0..2000 {
        // Place on sphere of radius 3000 centred at 0
        let phi: f32 = rng.gen_range(0.0f32..std::f32::consts::TAU);
        let cos_theta: f32 = rng.gen_range(-1.0f32..1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let pos = Vec3::new(
            3000.0 * sin_theta * phi.cos(),
            rng.gen_range(200.0f32..6000.0),
            3000.0 * sin_theta * phi.sin(),
        );
        commands.spawn((
            PbrBundle {
                mesh: star_mesh.clone(),
                material: star_mat.clone(),
                transform: Transform::from_translation(pos),
                ..default()
            },
            Star,
            RideSceneRoot,
        ));
    }
}

fn spawn_earth_disc_once(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut elevator: ResMut<ElevatorState>,
) {
    if elevator.earth_disc_spawned || elevator.altitude < 1200.0 {
        return;
    }
    elevator.earth_disc_spawned = true;

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 600.0, sectors: 24, stacks: 16 })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.08, 0.35, 0.7),
                emissive: Color::rgb(0.1, 0.4, 1.0),
                unlit: true,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, -700.0, 0.0),
            ..default()
        },
        EarthDisc,
        RideSceneRoot,
    ));
}

fn spawn_station_preview_once(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut elevator: ResMut<ElevatorState>,
) {
    if elevator.station_preview_spawned || elevator.altitude < 4500.0 {
        return;
    }
    elevator.station_preview_spawned = true;

    // Central hub glow
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 10.0, sectors: 16, stacks: 10 })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.9, 0.92, 0.95),
                emissive: Color::rgb(0.5, 0.6, 0.7),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 5012.0, 0.0),
            ..default()
        },
        StationPreview,
        RideSceneRoot,
    ));
}

fn check_arrival(
    elevator: Res<ElevatorState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if elevator.altitude >= CABLE_HEIGHT {
        next_state.set(GameState::Station);
    }
}

// ── Cleanup ───────────────────────────────────────────────────────────────────

fn cleanup_ride(
    mut commands: Commands,
    query: Query<Entity, With<RideSceneRoot>>,
    mut elevator: ResMut<ElevatorState>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    // Reset elevator for next time. Keep altitude at max so if we come back it
    // stays at station (shouldn't happen, but defensive).
    elevator.altitude = 0.0;
    elevator.stars_spawned = false;
    elevator.earth_disc_spawned = false;
    elevator.station_preview_spawned = false;
}
