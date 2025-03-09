use bevy::prelude::*;
use std::f32::consts::PI;
use avian3d::prelude::*;
use fastrand;
use crate::components::Protagonist;
use crate::systems::player::driving::set_driving_state;
use crate::systems::core::screenplay::{MessageDisplay, display_message};

// Cave dimensions
const CAVE_POSITION_X: f32 = 1194.7814;
const CAVE_POSITION_Y: f32 = 3.797667;
const CAVE_POSITION_Z: f32 = -4356.0886;
const CAVE_RADIUS: f32 = 600.0;
const CAVE_WALL_THICKNESS: f32 = 200.0;
const CAVE_HEIGHT: f32 = 2400.0;

// Crystal generation
const CRYSTAL_SEGMENTS: usize = 64;
const CRYSTAL_HEIGHT_SEGMENTS: usize = 96;
const CRYSTAL_SPAWN_CHANCE: f32 = 0.3;
const CRYSTAL_BASE_SIZE: f32 = 80.0;

// Light settings
const LIGHT_INTENSITY: f32 = 100000.0;
const LIGHT_COLOR_R: f32 = 0.9;
const LIGHT_COLOR_G: f32 = 0.95;
const LIGHT_COLOR_B: f32 = 1.0;

// Particle settings
const PARTICLE_COUNT: usize = 30000;
const PARTICLE_SIZE: f32 = 4.0;
const PARTICLE_MIN_SPEED: f32 = 0.5;
const PARTICLE_MAX_SPEED: f32 = 1.5;
const PARTICLE_MAX_HEIGHT: f32 = 800.0;
const PARTICLE_MAX_RADIUS: f32 = 500.0;
const PARTICLE_BASE_COLOR: Color = Color::srgb(0.85, 0.92, 0.99);
const PARTICLE_EMISSIVE_STRENGTH: f32 = 0.08;
const PARTICLE_LIFETIME: f32 = 20.0;
const PARTICLE_ROOT_COUNT: i32 = 12;  // Number of root points for particle spawning
const PARTICLE_ROOT_SPREAD: f32 = 30.0;  // Spread radius for root particles

pub fn spawn_ice_cave(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
    time: &Res<Time>,
) {
    let position = Vec3::new(CAVE_POSITION_X, CAVE_POSITION_Y, CAVE_POSITION_Z);
    
    // Main cylinder
    let cylinder_mesh = meshes.add(Extrusion::new(
        Annulus {
            inner_circle: Circle::new(CAVE_RADIUS),
            outer_circle: Circle::new(CAVE_RADIUS + CAVE_WALL_THICKNESS),
        },
        CAVE_HEIGHT,
    ));

    // Create material for the ice cave walls
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.01, 0.01, 0.01),
        perceptual_roughness: 0.5,
        metallic: 0.2,
        double_sided: true,
        cull_mode: None,
        unlit: false,
        reflectance: 0.1,
        ..default()
    });

    // Spawn main cylinder
    commands.spawn((
        PbrBundle {
            mesh: cylinder_mesh,
            material: material.clone(),
            transform: Transform::from_translation(position)
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)), // Lay on side
            ..default()
        },
        RigidBody::Static,
        ColliderConstructor::TrimeshFromMesh,
    ));

    // Replace spotlights with point lights
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            color: Color::srgb(LIGHT_COLOR_R, LIGHT_COLOR_G, LIGHT_COLOR_B),
            intensity: LIGHT_INTENSITY,
            range: CAVE_RADIUS * 4.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_translation(position + Vec3::new(-CAVE_HEIGHT * 0.25, 0.0, 0.0)),
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            color: Color::srgb(LIGHT_COLOR_R, LIGHT_COLOR_G, LIGHT_COLOR_B),
            intensity: LIGHT_INTENSITY,
            range: CAVE_RADIUS * 4.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_translation(position + Vec3::new(CAVE_HEIGHT * 0.25, 0.0, 0.0)),
        ..default()
    });
    
    for h in 0..CRYSTAL_HEIGHT_SEGMENTS {
        for s in 0..CRYSTAL_SEGMENTS {
            if fastrand::f32() > CRYSTAL_SPAWN_CHANCE { continue; }

            let angle = (s as f32 * 2.0 * PI) / CRYSTAL_SEGMENTS as f32;
            let height_offset = (h as f32 * CAVE_HEIGHT) / CRYSTAL_HEIGHT_SEGMENTS as f32 - CAVE_HEIGHT / 2.0;
            
            // Position crystals on the inner surface, matching cylinder's Z-rotation
            let direction = Vec3::new(
                angle.cos(),
                angle.sin(),
                0.0
            ).normalize();

            let base_pos = Vec3::new(
                angle.cos() * CAVE_RADIUS,
                angle.sin() * CAVE_RADIUS,
                height_offset  // Z is now height
            );
            
            let crystal_pos = position + base_pos;

            // Rest of crystal spawning code remains the same, but remove the direction calculation
            let radius_variation = CRYSTAL_BASE_SIZE * (0.8 + fastrand::f32() * 0.4);
            let y_normalized = (base_pos.y - (-CAVE_RADIUS)) / (CAVE_RADIUS * 2.0);
            let hue = 180.0 + y_normalized * 60.0;

            let hue_variation = Color::hsl(
                hue + fastrand::f32() * 5.0,
                0.8 + fastrand::f32() * 0.15,
                0.15 + fastrand::f32() * 0.15,
            );
            let emissive_strength = 0.1 + fastrand::f32() * 0.2;

            commands.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(
                    radius_variation,
                    radius_variation * (0.3 + fastrand::f32() * 0.4),
                    radius_variation * (0.5 + fastrand::f32() * 0.5)
                )),
                material: materials.add(StandardMaterial {
                    base_color: hue_variation,
                    metallic: 0.9,
                    perceptual_roughness: 0.1,
                    reflectance: 0.8,
                    emissive: Color::hsl(
                        hue + fastrand::f32() * 5.0,
                        0.8 + fastrand::f32() * 0.15,
                        emissive_strength,
                    ).into(),
                    base_color_texture: Some(asset_server.load("textures/amethyst.png")),
                    ..default()
                }),
                transform: Transform::from_translation(crystal_pos)
                    .looking_to(direction, Vec3::Z)
                    .with_rotation(
                        Quat::from_rotation_x(fastrand::f32() * PI) *
                        Quat::from_rotation_y(fastrand::f32() * PI) *
                        Quat::from_rotation_z(fastrand::f32() * PI)
                    ),
                ..default()
            });
        }
    }

    // Spawn particles with multiple root points
    let particle_material = materials.add(StandardMaterial {
        base_color: PARTICLE_BASE_COLOR.with_alpha(0.6),
        emissive: Color::srgb(
            PARTICLE_BASE_COLOR.to_linear().red * PARTICLE_EMISSIVE_STRENGTH,
            PARTICLE_BASE_COLOR.to_linear().green * PARTICLE_EMISSIVE_STRENGTH,
            PARTICLE_BASE_COLOR.to_linear().blue * PARTICLE_EMISSIVE_STRENGTH,
        ).into(),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    let particle_mesh = meshes.add(Mesh::from(Sphere { radius: PARTICLE_SIZE }));

    for _ in 0..PARTICLE_COUNT {
        let random_speed = PARTICLE_MIN_SPEED + fastrand::f32() * (PARTICLE_MAX_SPEED - PARTICLE_MIN_SPEED);
        let root_offset = if fastrand::f32() < (PARTICLE_ROOT_COUNT as f32 / PARTICLE_COUNT as f32) {
            // Create root particle spread
            let angle = fastrand::f32() * PI * 2.0;
            let radius = fastrand::f32() * PARTICLE_ROOT_SPREAD;
            Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius)
        } else {
            Vec3::ZERO
        };
        
        commands.spawn((
            PbrBundle {
                mesh: particle_mesh.clone(),
                material: particle_material.clone(),
                transform: Transform::from_translation(position + root_offset),
                ..default()
            },
            IceCaveParticle {
                velocity: Vec3::new(0.0, random_speed, 0.0),
                origin: position,
                start_time: time.elapsed_seconds(),
            },
        ));
    }

    // Add transportation disc
    let disc_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 0.4, 0.99, 0.8),
        emissive: Color::srgb(0.0, 0.2, 0.99).into(),
        alpha_mode: AlphaMode::Blend,
        metallic: 0.8,
        perceptual_roughness: 0.1,
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(50.0)),  // 50.0 radius disc
        material: disc_material,
        transform: Transform::from_translation(position + Vec3::new(0.0, 1.0, 0.0))  // Slightly above ground
            .with_rotation(Quat::from_rotation_x(-PI / 2.0)),  // Lay flat
        ..default()
    });
}

// Add this component
#[derive(Component)]
pub struct IceCaveParticle {
    velocity: Vec3,
    origin: Vec3,
    start_time: f32,
}

// Add this system to your main.rs
pub fn update_ice_particles(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut IceCaveParticle, &Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (_, mut transform, mut particle, material_handle) in query.iter_mut() {
        let effect_age = time.elapsed_seconds() - particle.start_time;
        let lifetime_fraction = (effect_age / PARTICLE_LIFETIME).clamp(0.0, 1.0);
        
        let height = transform.translation.y - particle.origin.y;
        let height_fraction = (height / PARTICLE_MAX_HEIGHT).clamp(0.0, 1.0);

        // Motion calculations remain the same
        let pos = transform.translation - particle.origin;
        
        let vertical_speed = (PARTICLE_MIN_SPEED + 
            (fastrand::f32() * (PARTICLE_MAX_SPEED - PARTICLE_MIN_SPEED))) * 
            (1.0 - height_fraction.powf(2.0));
        
        let flow = Vec3::new(
            (time.elapsed_seconds() * 0.1 + pos.z * 0.01).sin() * 2.0,
            0.0,
            (time.elapsed_seconds() * 0.15 + pos.x * 0.01).cos() * 2.0
        );

        let medium_turb = Vec3::new(
            (time.elapsed_seconds() * 0.3 + pos.y * 0.02).sin() * 1.0,
            (time.elapsed_seconds() * 0.25 + pos.x * 0.02).cos() * 0.5,
            (time.elapsed_seconds() * 0.35 + pos.z * 0.02).sin() * 1.0
        );

        let noise = Vec3::new(
            (time.elapsed_seconds() * 2.0 + pos.y * 0.1).sin() * 0.2,
            (time.elapsed_seconds() * 1.8 + pos.z * 0.1).cos() * 0.2,
            (time.elapsed_seconds() * 1.9 + pos.x * 0.1).sin() * 0.2
        );

        let angle = time.elapsed_seconds() * 0.1 + height * 0.01;
        let radius = PARTICLE_MAX_RADIUS * (1.0 - height_fraction.powf(0.5));
        let drift = Vec3::new(
            angle.cos() * radius * 0.01,
            0.0,
            angle.sin() * radius * 0.01
        );

        transform.translation += 
            Vec3::Y * vertical_speed * time.delta_seconds() +
            flow * time.delta_seconds() +
            medium_turb * time.delta_seconds() +
            noise * time.delta_seconds() +
            drift * time.delta_seconds();

        let horizontal_distance = Vec2::new(
            transform.translation.x - particle.origin.x,
            transform.translation.z - particle.origin.z
        ).length();

        if lifetime_fraction >= 1.0 || 
           height > PARTICLE_MAX_HEIGHT || 
           horizontal_distance > PARTICLE_MAX_RADIUS {
            let random_angle = fastrand::f32() * PI * 2.0;
            let random_radius = fastrand::f32() * PARTICLE_MAX_RADIUS * 0.5;
            transform.translation = particle.origin + Vec3::new(
                random_angle.cos() * random_radius,
                0.0,
                random_angle.sin() * random_radius
            );
            particle.start_time = time.elapsed_seconds();
        }

        // Simplified material updates
        if let Some(material) = materials.get_mut(material_handle) {
            let fade = (1.0 - lifetime_fraction) * (1.0 - height_fraction);
            
            material.base_color = PARTICLE_BASE_COLOR.with_alpha(0.3 * fade);
            material.emissive = (PARTICLE_BASE_COLOR.to_linear() * (PARTICLE_EMISSIVE_STRENGTH * fade)).into();
        }
    }
}

// Add this system
pub fn handle_ice_cave_interactions(
    mut query_set: ParamSet<(
        Query<(Entity, &mut Transform, &mut Protagonist, &mut Handle<Scene>)>,
        Query<&Transform, With<IceCaveParticle>>,
    )>,
    mut commands: Commands,
    children_query: Query<&Children>,
    asset_server: Res<AssetServer>,
    mut message_display: ResMut<MessageDisplay>,
) {
    let cave_pos = Vec3::new(CAVE_POSITION_X, CAVE_POSITION_Y, CAVE_POSITION_Z);
    
    // Get particle positions first
    let particle_positions: Vec<Vec3> = query_set.p1()
        .iter()
        .map(|transform| transform.translation)
        .collect();

    // Then handle protagonist
    if let Ok((entity, mut transform, mut protagonist, mut scene)) = query_set.p0().get_single_mut() {
        // Check if inside cave (using horizontal distance)
        let horizontal_distance = Vec2::new(
            transform.translation.x - cave_pos.x,
            transform.translation.z - cave_pos.z
        ).length();
        
        // Check if within cave bounds (both radius and height)
        let vertical_distance = (transform.translation.y - cave_pos.y).abs();
        if horizontal_distance < CAVE_RADIUS && vertical_distance < CAVE_HEIGHT / 2.0 {
            if protagonist.is_driving {
                set_driving_state(
                    &mut protagonist,
                    &mut scene,
                    &asset_server,
                    false,
                    &mut commands,
                    entity,
                    &children_query
                );
                display_message("FIND THE ACQUIFIER", Color::srgb(0.0, 0.2, 1.0), &mut message_display);
            }
        }

        // Check particle collisions
        for particle_pos in particle_positions {
            let distance = transform.translation.distance(particle_pos);
            if distance < 5.0 {
                transform.translation.y -= 30.0;
                break;
            }
        }
    }
}
