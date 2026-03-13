use bevy::prelude::*;
use bevy::render::mesh::Mesh;

// Color palette (Astroneer-inspired)
pub mod palette {
    use bevy::prelude::Color;

    pub const GROUND: Color = Color::rgb(0.106, 0.227, 0.294);       // #1B3A4B
    pub const CABIN: Color = Color::rgb(0.910, 0.957, 0.975);        // #E8F4F8
    pub const CABLE: Color = Color::rgb(0.0, 0.898, 1.0);            // #00E5FF
    pub const SKY_TOP: Color = Color::rgb(0.008, 0.031, 0.094);      // #020818
    pub const SKY_HORIZON: Color = Color::rgb(0.039, 0.165, 0.431);  // #0A2A6E
    pub const MOUNTAIN: Color = Color::rgb(0.059, 0.176, 0.239);     // #0F2D3D
    pub const TREE: Color = Color::rgb(0.176, 0.416, 0.310);         // #2D6A4F
    pub const STAR: Color = Color::rgb(0.95, 0.95, 1.0);
    pub const EARTH_GLOW: Color = Color::rgb(0.1, 0.4, 0.8);
    pub const STATION_WHITE: Color = Color::rgb(0.9, 0.92, 0.95);
    pub const SOLAR_GOLD: Color = Color::rgb(0.85, 0.72, 0.1);
    pub const DOME_GLASS: Color = Color::rgba(0.4, 0.7, 0.9, 0.6);
    pub const UI_CYAN: Color = Color::rgb(0.0, 0.898, 1.0);
    pub const UI_BG: Color = Color::rgba(0.0, 0.0, 0.0, 0.55);
}

/// Build a PBR material with emissive glow
pub fn emissive_material(
    materials: &mut Assets<StandardMaterial>,
    color: Color,
    emissive: Color,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: color,
        emissive,
        unlit: false,
        ..default()
    })
}

/// Build a plain PBR material
pub fn solid_material(
    materials: &mut Assets<StandardMaterial>,
    color: Color,
    metallic: f32,
    roughness: f32,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: color,
        metallic,
        perceptual_roughness: roughness,
        ..default()
    })
}

/// Build a transparent PBR material
pub fn transparent_material(
    materials: &mut Assets<StandardMaterial>,
    color: Color,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: color,
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        cull_mode: None,
        ..default()
    })
}

/// Low-poly sphere (icosphere-like via UVSphere with low resolution)
pub fn low_poly_sphere(meshes: &mut Assets<Mesh>, radius: f32) -> Handle<Mesh> {
    meshes.add(
        Mesh::from(shape::UVSphere {
            radius,
            sectors: 8,
            stacks: 6,
        }),
    )
}

/// Smooth sphere
pub fn smooth_sphere(meshes: &mut Assets<Mesh>, radius: f32) -> Handle<Mesh> {
    meshes.add(
        Mesh::from(shape::UVSphere {
            radius,
            sectors: 32,
            stacks: 16,
        }),
    )
}

/// Cylinder mesh helper
pub fn cylinder_mesh(
    meshes: &mut Assets<Mesh>,
    radius: f32,
    height: f32,
    resolution: u32,
) -> Handle<Mesh> {
    meshes.add(
        Mesh::from(shape::Cylinder {
            radius,
            height,
            resolution,
            segments: 1,
        }),
    )
}

/// Flat disc (thin cylinder)
pub fn disc_mesh(meshes: &mut Assets<Mesh>, radius: f32) -> Handle<Mesh> {
    meshes.add(
        Mesh::from(shape::Cylinder {
            radius,
            height: 0.1,
            resolution: 16,
            segments: 1,
        }),
    )
}
