use bevy::prelude::*;
use avian3d::prelude::*;
use crate::systems::core::setup::ACQUIFIER_FLOOR_DEPTH;
use rand::Rng;
use crate::components::Protagonist;
use crate::systems::core::screenplay::{MessageDisplay, display_message};

// Constants for the pipe
const PIPE_INNER_RADIUS: f32 = 100.0;
const PIPE_THICKNESS: f32 = 20.0;
const PIPE_RADIUS: f32 = PIPE_INNER_RADIUS + PIPE_THICKNESS;
const PIPE_LENGTH: f32 = 2200.0;
const PIPE_BOTTOM_GAP: f32 = 200.0;
// Moved another 500 units closer to origin on both X and Z
pub const PIPE_POSITION: Vec3 = Vec3::new(-7447.827, ACQUIFIER_FLOOR_DEPTH + PIPE_BOTTOM_GAP + PIPE_LENGTH/2.0, 9928.124);

#[derive(Component)]
pub struct PipeLiftTrigger {
    pub cooldown: Timer,
}

pub fn spawn_big_pipe(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Copper material with texture
    let copper_material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/negative-space-copper.png")),
        metallic: 0.99,
        perceptual_roughness: 0.05,
        reflectance: 0.99,
        emissive: Color::rgb(0.02, 0.01, 0.005).into(),
        ..default()
    });

    // Create pipe mesh as an extruded annulus
    let pipe_mesh = meshes.add(Extrusion::new(
        Annulus {
            inner_circle: Circle::new(PIPE_INNER_RADIUS),
            outer_circle: Circle::new(PIPE_RADIUS),
        },
        PIPE_LENGTH,
    ));

    // Spawn the pipe vertically
    commands.spawn((
        PbrBundle {
            mesh: pipe_mesh,
            material: copper_material,
            transform: Transform::from_translation(PIPE_POSITION)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ..default()
        },
        RigidBody::Static,
        ColliderConstructor::TrimeshFromMesh,
    ))
    .with_children(|parent| {
        // Add trigger cylinder as child of pipe
        parent.spawn((
            Sensor,
            Collider::cylinder(PIPE_INNER_RADIUS * 0.8, 200.0), // Made 10x longer
            PbrBundle {
                mesh: meshes.add(Cylinder {
                    radius: PIPE_INNER_RADIUS * 0.8,
                    half_height: 100.0, // Made 10x longer to match collider
                }),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgba(1.0, 0.0, 0.0, 0.8),
                    emissive: Color::rgba(1.0, 0.0, 0.0, 0.5).into(),
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                }),
                // Moved up 50 units in local space
                transform: Transform::from_xyz(0.0, 0.0, PIPE_LENGTH/2.0 - 150.0)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                ..default()
            },
            PipeLiftTrigger {
                cooldown: Timer::from_seconds(1.0, TimerMode::Once),
            },
            Name::new("PipeLiftTrigger"),
        ));
    });

    // Add bright white light inside the pipe
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 2000000.0,
            color: Color::rgb(1.0, 0.95, 0.9),
            radius: PIPE_INNER_RADIUS * 0.9,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_translation(PIPE_POSITION),
        ..default()
    });

    // Create sconce mesh
    let sconce_mesh = meshes.add(Sphere { radius: 8.0 }); // Visible sphere for each sconce

    // Create two glowing materials for the sconces
    let bright_blue_material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.0, 0.8, 1.0),
        emissive: Color::rgb(0.0, 0.8, 0.99).into(), // Bright cyan-blue glow
        metallic: 0.0,
        perceptual_roughness: 0.0,
        ..default()
    });

    let deep_blue_material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.2, 0.2, 1.0),
        emissive: Color::rgb(0.2, 0.2, 0.99).into(), // Deep blue glow
        metallic: 0.0,
        perceptual_roughness: 0.0,
        ..default()
    });

    // Add random sconce lights with visible meshes
    let mut rng = rand::thread_rng();
    for _ in 0..1000 {
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let height = rng.gen_range(-PIPE_LENGTH/2.0..PIPE_LENGTH/2.0);
        
        // Position sconces directly on pipe surface
        let position = PIPE_POSITION + Vec3::new(
            PIPE_RADIUS * angle.cos(),
            height,
            PIPE_RADIUS * angle.sin()
        );

        let normal = Vec3::new(angle.cos(), 0.0, angle.sin());
        
        let use_bright_blue = rng.gen_bool(0.5);
        let (color, intensity, material) = if use_bright_blue {
            (Color::rgb(0.0, 0.8, 1.0), 100000.0, bright_blue_material.clone())
        } else {
            (Color::rgb(0.2, 0.2, 1.0), 120000.0, deep_blue_material.clone())
        };

        // Spawn visible sconce
        commands.spawn(PbrBundle {
            mesh: sconce_mesh.clone(),
            material,
            transform: Transform::from_translation(position)
                .looking_to(normal, Vec3::Y), // Orient sconces along pipe surface normal
            ..default()
        })
        .with_children(|parent| {
            // Add light as child
            parent.spawn(PointLightBundle {
                point_light: PointLight {
                    intensity,
                    color,
                    radius: 25.0,
                    shadows_enabled: false,
                    ..default()
                },
                ..default()
            });
        });
    }

    // Add bright white light at bottom of pipe
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 3000000.0,
            color: Color::rgb(1.0, 0.95, 0.9),
            radius: PIPE_RADIUS * 2.5,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_translation(
            PIPE_POSITION + Vec3::new(0.0, -PIPE_LENGTH/2.0, 0.0)
        ),
        ..default()
    });
}

// Add this system to main.rs
pub fn handle_pipe_lift(
    time: Res<Time>,
    collisions: Res<Collisions>,
    mut triggers: Query<(Entity, &mut PipeLiftTrigger)>,
    mut player: Query<(Entity, &mut Transform, &mut LinearVelocity), With<Protagonist>>,
    mut message_display: ResMut<MessageDisplay>,
) {
    if let Ok((player_entity, mut player_transform, mut velocity)) = player.get_single_mut() {
        for (trigger_entity, mut trigger) in &mut triggers {
            if collisions.contains(player_entity, trigger_entity) {
                if trigger.cooldown.finished() {
                    // Zero out vertical velocity
                    velocity.0.y = 0.0;
                    
                    // Translate player
                    player_transform.translation.y += 750.0;
                    player_transform.translation.x += 500.0;
                    
                    display_message("Shelter in the vehicle...", Color::rgb(0.0, 0.8, 1.0), &mut message_display);
                    trigger.cooldown.reset();
                }
                trigger.cooldown.tick(time.delta());
            }
        }
    }
}
