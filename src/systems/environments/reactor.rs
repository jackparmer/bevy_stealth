use bevy::prelude::*;
use avian3d::prelude::*;

pub const WALL_THICKNESS: f32 = 20.0;
const REACTOR_POSITION: Vec3 = Vec3::new(-455.0 * 2.0, 1.6, 915.0 * 2.0);

// Cylinder constants
const CYLINDER_INNER_RADIUS: f32 = 500.0;
const CYLINDER_RADIUS: f32 = CYLINDER_INNER_RADIUS + WALL_THICKNESS;
pub const CYLINDER_HEIGHT: f32 = 800.0;

// Constants for the glowing disc
const DISC_GAP: f32 = 10.0;
const DISC_THICKNESS: f32 = 2.0;
const SAFETY_MARGIN: f32 = 5.0;

pub fn spawn_reactor(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
) {
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/worn_corrugated_iron_diff_4k.png")),
        metallic: 0.1,
        perceptual_roughness: 0.8,
        ..default()
    });

    // Main cylinder
    let cylinder_mesh = meshes.add(Extrusion::new(
        Annulus {
            inner_circle: Circle::new(CYLINDER_INNER_RADIUS),
            outer_circle: Circle::new(CYLINDER_RADIUS),
        },
        CYLINDER_HEIGHT - DISC_GAP,
    ));

    // Cap meshes - using extrusion instead of circle for thickness
    let cap_mesh = meshes.add(Extrusion::new(
        Circle::new(CYLINDER_RADIUS),
        WALL_THICKNESS,
    ));

    // Create glowing material for the disc
    let glow_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        emissive: Color::rgb(1.0, 0.0, 0.0).into(), // Bright red glow
        ..default()
    });

    // Disc mesh
    let disc_mesh = meshes.add(Extrusion::new(
        Circle::new(CYLINDER_RADIUS),
        DISC_THICKNESS,
    ));

    // Spawn main cylinder
    commands.spawn((
        PbrBundle {
            mesh: cylinder_mesh,
            material: material.clone(),
            transform: Transform::from_translation(REACTOR_POSITION)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ..default()
        },
        RigidBody::Static,
        ColliderConstructor::TrimeshFromMesh,
    ));

    // Spawn top cap with offset - moved further up
    commands.spawn((
        PbrBundle {
            mesh: cap_mesh.clone(),
            material: material.clone(),
            transform: Transform::from_translation(REACTOR_POSITION + Vec3::new(0.0, CYLINDER_HEIGHT/2.0 + DISC_GAP + WALL_THICKNESS/2.0 + SAFETY_MARGIN, 0.0))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            ..default()
        },
        RigidBody::Static,
        ColliderConstructor::TrimeshFromMesh,
    ));

    // Spawn glowing disc - positioned with more space between cylinder and cap
    commands.spawn(PbrBundle {
        mesh: disc_mesh,
        material: glow_material,
        transform: Transform::from_translation(REACTOR_POSITION + Vec3::new(0.0, CYLINDER_HEIGHT/2.0 + SAFETY_MARGIN, 0.0))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });

    // Add intense point light in the lower half of the reactor
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 100000.0, // Very bright light
            color: Color::rgb(1.0, 0.2, 0.2), // Reddish color to match the reactor glow
            radius: 1.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_translation(
            REACTOR_POSITION + Vec3::new(0.0, -CYLINDER_HEIGHT/4.0, 0.0) // Positioned at 1/4 height from bottom
        ),
        ..default()
    });
}

