use bevy::prelude::*;
use crate::game_state::GameState;
use crate::elevator::{ElevatorState, altitude_to_km};
use crate::station::{MissionTime, Interactable, StationPlayer};

// ── Components ────────────────────────────────────────────────────────────────

/// Marker for the [SPACE] prompt text in the Earth scene
#[derive(Component)]
pub struct EnterPrompt;

/// Phase name text during riding
#[derive(Component)]
pub struct PhaseLabel;

/// Altitude fill bar
#[derive(Component)]
pub struct AltitudeFill;

/// Altitude km text
#[derive(Component)]
pub struct AltitudeText;

/// Speed display text
#[derive(Component)]
pub struct SpeedText;

/// Station top-bar title
#[derive(Component)]
pub struct StationTitle;

/// Mission time clock
#[derive(Component)]
pub struct MissionClock;

/// Context interaction prompt
#[derive(Component)]
pub struct InteractPrompt;

/// Tag for Earth UI root
#[derive(Component)]
pub struct EarthUiRoot;

/// Tag for Riding UI root
#[derive(Component)]
pub struct RidingUiRoot;

/// Tag for Station UI root
#[derive(Component)]
pub struct StationUiRoot;

// ── Text styles ───────────────────────────────────────────────────────────────

fn hud_style(font_size: f32) -> TextStyle {
    TextStyle {
        font_size,
        color: Color::WHITE,
        ..default()
    }
}

fn cyan_style(font_size: f32) -> TextStyle {
    TextStyle {
        font_size,
        color: Color::rgb(0.0, 0.898, 1.0),
        ..default()
    }
}

// ── Plugin ────────────────────────────────────────────────────────────────────

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // Earth UI
            .add_systems(OnEnter(GameState::Earth), setup_earth_ui)
            .add_systems(OnExit(GameState::Earth), cleanup_ui::<EarthUiRoot>)
            // Ride UI
            .add_systems(OnEnter(GameState::Riding), setup_ride_ui)
            .add_systems(OnExit(GameState::Riding), cleanup_ui::<RidingUiRoot>)
            .add_systems(
                Update,
                update_ride_ui.run_if(in_state(GameState::Riding)),
            )
            // Station UI
            .add_systems(OnEnter(GameState::Station), setup_station_ui)
            .add_systems(OnExit(GameState::Station), cleanup_ui::<StationUiRoot>)
            .add_systems(
                Update,
                (update_station_ui, update_interact_prompt)
                    .run_if(in_state(GameState::Station)),
            );
    }
}

// ── Earth UI ──────────────────────────────────────────────────────────────────

fn setup_earth_ui(mut commands: Commands) {
    // Root node
    let root = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
            EarthUiRoot,
        ))
        .id();

    // Top-left logo
    let top_bar = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            ..default()
        })
        .id();

    let logo = commands
        .spawn(TextBundle::from_section(
            "SPACE ELEVATOR SIM",
            cyan_style(18.0),
        ))
        .id();

    // Bottom enter-elevator prompt with pulsing background
    let bottom_area = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    padding: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
                ..default()
            },
        ))
        .id();

    let prompt_bg = commands
        .spawn((
            NodeBundle {
                style: Style {
                    padding: UiRect::axes(Val::Px(24.0), Val::Px(12.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(0.0, 0.898, 1.0, 0.15)),
                ..default()
            },
            EnterPrompt,
        ))
        .id();

    let prompt_text = commands
        .spawn(TextBundle::from_section(
            "[SPACE] Enter Elevator",
            cyan_style(22.0),
        ))
        .id();

    commands.entity(root).push_children(&[top_bar, bottom_area]);
    commands.entity(top_bar).push_children(&[logo]);
    commands.entity(bottom_area).push_children(&[prompt_bg]);
    commands.entity(prompt_bg).push_children(&[prompt_text]);
}

// ── Ride UI ───────────────────────────────────────────────────────────────────

fn setup_ride_ui(mut commands: Commands) {
    // Root full-screen container
    let root = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ..default()
            },
            RidingUiRoot,
        ))
        .id();

    // ── Left altitude meter ───────────────────────────────────────────────────
    let left_panel = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(80.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.45)),
            ..default()
        })
        .id();

    // Top label
    let alt_top = commands
        .spawn(TextBundle::from_section("36,000\nkm", hud_style(10.0)))
        .id();

    // Bar outer
    let bar_outer = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(16.0),
                height: Val::Px(400.0),
                flex_direction: FlexDirection::ColumnReverse,
                overflow: Overflow::clip(),
                ..default()
            },
            background_color: BackgroundColor(Color::rgba(1.0, 1.0, 1.0, 0.15)),
            ..default()
        })
        .id();

    // Bar fill
    let bar_fill = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(0.0), // updated each frame
                    ..default()
                },
                background_color: BackgroundColor(Color::rgb(0.0, 0.898, 1.0)),
                ..default()
            },
            AltitudeFill,
        ))
        .id();

    // Bottom label
    let alt_bottom = commands
        .spawn(TextBundle::from_section("0 km", hud_style(10.0)))
        .id();

    let alt_km_text = commands
        .spawn((
            TextBundle::from_section("0 km", cyan_style(14.0)),
            AltitudeText,
        ))
        .id();

    commands.entity(root).push_children(&[left_panel]);
    commands.entity(left_panel).push_children(&[alt_top, bar_outer, alt_bottom, alt_km_text]);
    commands.entity(bar_outer).push_children(&[bar_fill]);

    // ── Top-center phase label ────────────────────────────────────────────────
    let top_center = commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Percent(50.0),
                ..default()
            },
            ..default()
        })
        .id();

    let phase_text = commands
        .spawn((
            TextBundle::from_section("TROPOSPHERE", cyan_style(24.0)),
            PhaseLabel,
        ))
        .id();

    commands.entity(root).push_children(&[top_center]);
    commands.entity(top_center).push_children(&[phase_text]);

    // ── Bottom speed text ─────────────────────────────────────────────────────
    let bottom = commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Percent(50.0),
                ..default()
            },
            ..default()
        })
        .id();

    let speed_text = commands
        .spawn((
            TextBundle::from_section("Speed: 1.0x   [↑/↓]", hud_style(16.0)),
            SpeedText,
        ))
        .id();

    commands.entity(root).push_children(&[bottom]);
    commands.entity(bottom).push_children(&[speed_text]);
}

fn update_ride_ui(
    elevator: Res<ElevatorState>,
    mut fill_q: Query<&mut Style, With<AltitudeFill>>,
    mut alt_q: Query<&mut Text, (With<AltitudeText>, Without<PhaseLabel>, Without<SpeedText>)>,
    mut phase_q: Query<&mut Text, (With<PhaseLabel>, Without<AltitudeText>, Without<SpeedText>)>,
    mut speed_q: Query<&mut Text, (With<SpeedText>, Without<AltitudeText>, Without<PhaseLabel>)>,
) {
    let pct = (elevator.altitude / crate::elevator::CABLE_HEIGHT * 100.0).clamp(0.0, 100.0);
    let km = altitude_to_km(elevator.altitude);

    // Fill bar
    for mut style in &mut fill_q {
        style.height = Val::Percent(pct);
    }

    // Altitude text
    for mut text in &mut alt_q {
        text.sections[0].value = format!("{:.0} km", km);
    }

    // Phase label
    for mut text in &mut phase_q {
        text.sections[0].value = elevator.phase.name().to_string();
    }

    // Speed text
    for mut text in &mut speed_q {
        text.sections[0].value =
            format!("Speed: {:.1}x   [↑/↓]", elevator.speed_multiplier);
    }
}

// ── Station UI ────────────────────────────────────────────────────────────────

fn setup_station_ui(mut commands: Commands) {
    let root = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ..default()
            },
            StationUiRoot,
        ))
        .id();

    // Top bar
    let top_bar = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(16.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.5)),
            ..default()
        })
        .id();

    let title = commands
        .spawn((
            TextBundle::from_section("ORBITAL STATION ALPHA", cyan_style(20.0)),
            StationTitle,
        ))
        .id();

    let clock = commands
        .spawn((
            TextBundle::from_section("T+00:00", hud_style(16.0)),
            MissionClock,
        ))
        .id();

    // Bottom interact prompt (hidden by default — shown when near interactable)
    let bottom = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                padding: UiRect::bottom(Val::Px(30.0)),
                ..default()
            },
            ..default()
        })
        .id();

    let interact_box = commands
        .spawn(NodeBundle {
            style: Style {
                padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                ..default()
            },
            background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            ..default()
        })
        .id();

    let interact_text = commands
        .spawn((
            TextBundle::from_section("", cyan_style(20.0)),
            InteractPrompt,
        ))
        .id();

    // Controls hint (top-right corner area)
    let controls = commands
        .spawn(TextBundle {
            text: Text::from_section(
                "WASD: Move\nSpace: Up  Shift: Down\nE: Interact",
                hud_style(13.0),
            ),
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(60.0),
                right: Val::Px(16.0),
                ..default()
            },
            ..default()
        })
        .id();

    commands.entity(root).push_children(&[top_bar, bottom, controls]);
    commands.entity(top_bar).push_children(&[title, clock]);
    commands.entity(bottom).push_children(&[interact_box]);
    commands.entity(interact_box).push_children(&[interact_text]);
}

fn update_station_ui(
    mt: Res<MissionTime>,
    mut clock_q: Query<&mut Text, With<MissionClock>>,
) {
    let secs = mt.elapsed as u32;
    let minutes = secs / 60;
    let seconds = secs % 60;
    for mut text in &mut clock_q {
        text.sections[0].value = format!("T+{:02}:{:02}", minutes, seconds);
    }
}

fn update_interact_prompt(
    player_q: Query<&Transform, With<StationPlayer>>,
    interactables: Query<(&Transform, &Interactable), Without<StationPlayer>>,
    mut prompt_q: Query<&mut Text, With<InteractPrompt>>,
) {
    let Ok(pt) = player_q.get_single() else { return };
    let player_pos = pt.translation;

    let mut nearest_label: Option<&str> = None;
    let mut nearest_dist = f32::MAX;

    for (it, interactable) in &interactables {
        let dist = player_pos.distance(it.translation);
        if dist <= crate::station::INTERACT_RANGE && dist < nearest_dist {
            nearest_dist = dist;
            nearest_label = Some(interactable.label);
        }
    }

    for mut text in &mut prompt_q {
        text.sections[0].value = match nearest_label {
            Some(label) => format!("[E] {}", label),
            None => String::new(),
        };
    }
}

// ── Cleanup ───────────────────────────────────────────────────────────────────

fn cleanup_ui<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
