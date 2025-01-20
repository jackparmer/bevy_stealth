use bevy::prelude::*;
use avian3d::prelude::*;
use rand::Rng;
use crate::systems::environments::ladder::{spawn_ladder, LadderConfig};
use crate::components::Protagonist;

const CUBE_WALL_THICKNESS: f32 = 20.0;
const CUBE_SIZE: f32 = 400.0; // Size of the inner hollow space
const REACTOR_POSITION: Vec3 = Vec3::new(-485.0 * 2.0, 2.6249764, -1066.0 * 2.0);
const HOLE_SIZE: f32 = 50.0; // Size of the opening in the floor
const GOD_RAY_HEIGHT: f32 = 20.0; // Height of the god ray volume

// New constants for pillar dimensions
const PILLAR_MIN_HEIGHT_FACTOR: f32 = 0.8;
const PILLAR_MAX_HEIGHT_FACTOR: f32 = 1.1;
const PILLAR_BELOW_GROUND_HEIGHT: f32 = 400.0;
const PILLAR_MIN_WIDTH: f32 = 12.0;
const PILLAR_MAX_WIDTH: f32 = 20.0;

// Add new constant for blue sconce properties
const BLUE_SCONCE_BASE_INTENSITY: f32 = 4.0;
const BLUE_SCONCE_PULSE_SPEED: f32 = 1.2;

#[derive(Component)]
pub struct TwinklingLight {
    base_intensity: f32,
    phase_offset: f32,
}

// Add new component for pulsing blue lights
#[derive(Component)]
pub struct PulsingBlueLight {
    phase_offset: f32,
}

pub fn spawn_reactor(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
    time: Res<Time>,
) {
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/rusty_metal_03_diff_4k.png")),
        metallic: 0.1,
        perceptual_roughness: 0.8,
        ..default()
    });

    // Bottom wall (now with hole) - split into 4 segments
    let floor_segments = [
        // Front segment
        (Vec3::new(0.0, -(CUBE_SIZE/2.0), (HOLE_SIZE + CUBE_SIZE)/4.0), 
         Vec3::new(CUBE_SIZE + CUBE_WALL_THICKNESS, CUBE_WALL_THICKNESS, (CUBE_SIZE - HOLE_SIZE)/2.0)),
        // Back segment
        (Vec3::new(0.0, -(CUBE_SIZE/2.0), -(HOLE_SIZE + CUBE_SIZE)/4.0),
         Vec3::new(CUBE_SIZE + CUBE_WALL_THICKNESS, CUBE_WALL_THICKNESS, (CUBE_SIZE - HOLE_SIZE)/2.0)),
        // Left segment
        (Vec3::new(-(HOLE_SIZE + CUBE_SIZE)/4.0, -(CUBE_SIZE/2.0), 0.0),
         Vec3::new((CUBE_SIZE - HOLE_SIZE)/2.0, CUBE_WALL_THICKNESS, HOLE_SIZE)),
        // Right segment
        (Vec3::new((HOLE_SIZE + CUBE_SIZE)/4.0, -(CUBE_SIZE/2.0), 0.0),
         Vec3::new((CUBE_SIZE - HOLE_SIZE)/2.0, CUBE_WALL_THICKNESS, HOLE_SIZE)),
    ];

    for (position, size) in floor_segments {
        commands.spawn((
            RigidBody::Static,
            Collider::cuboid(size.x, size.y, size.z),
            PbrBundle {
                mesh: meshes.add(Cuboid::new(size.x, size.y, size.z)),
                material: material.clone(),
                transform: Transform::from_translation(REACTOR_POSITION + position),
                ..default()
            },
        ));
    }

    // Lower floor segments (at the bottom of the god ray)
    let lower_floor_position = Vec3::new(0.0, -REACTOR_POSITION.y, 0.0);
    let lower_floor_segments = [
        // Front segment
        (Vec3::new(0.0, 0.0, (HOLE_SIZE + CUBE_SIZE)/4.0), 
         Vec3::new(CUBE_SIZE - CUBE_WALL_THICKNESS * 2.0, CUBE_WALL_THICKNESS, (CUBE_SIZE - HOLE_SIZE)/2.0)),
        // Back segment
        (Vec3::new(0.0, 0.0, -(HOLE_SIZE + CUBE_SIZE)/4.0),
         Vec3::new(CUBE_SIZE - CUBE_WALL_THICKNESS * 2.0, CUBE_WALL_THICKNESS, (CUBE_SIZE - HOLE_SIZE)/2.0)),
        // Left segment
        (Vec3::new(-(HOLE_SIZE + CUBE_SIZE)/4.0, 0.0, 0.0),
         Vec3::new((CUBE_SIZE - HOLE_SIZE)/2.0, CUBE_WALL_THICKNESS, HOLE_SIZE)),
        // Right segment
        (Vec3::new((HOLE_SIZE + CUBE_SIZE)/4.0, 0.0, 0.0),
         Vec3::new((CUBE_SIZE - HOLE_SIZE)/2.0, CUBE_WALL_THICKNESS, HOLE_SIZE)),
    ];

    // Lower floor segments with new texture
    let lower_floor_material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/painted_concrete_diff_4k.png")),
        metallic: 0.1,                             // Non-metallic for concrete
        perceptual_roughness: 0.95,                // Very rough for concrete
        emissive: Color::srgb(0.05, 0.05, 0.05).into(),
        ..default()
    });

    // Spawn floor segments with proper collision
    for (position, size) in lower_floor_segments {
        commands.spawn((
            RigidBody::Static,
            Collider::cuboid(size.x, size.y, size.z),
            PbrBundle {
                mesh: meshes.add(Cuboid::new(size.x, size.y, size.z)),
                material: lower_floor_material.clone(), // Using new concrete texture
                transform: Transform::from_translation(REACTOR_POSITION + lower_floor_position + position),
                ..default()
            },
        ));
    }

    // Top wall (dark painted concrete)
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(CUBE_SIZE + CUBE_WALL_THICKNESS + 0.01, CUBE_WALL_THICKNESS, CUBE_SIZE + CUBE_WALL_THICKNESS + 0.01),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(CUBE_SIZE + CUBE_WALL_THICKNESS, CUBE_WALL_THICKNESS, CUBE_SIZE + CUBE_WALL_THICKNESS)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/painted_concrete_diff_4k.png")),
                metallic: 0.1,                             // Non-metallic for concrete
                perceptual_roughness: 0.95,                // Very rough for concrete
                ..default()
            }),
            transform: Transform::from_translation(REACTOR_POSITION + Vec3::new(0.0, CUBE_SIZE/2.0 - CUBE_WALL_THICKNESS/2.0, 0.0)),
            ..default()
        },
    ));

    // Front wall (full size)
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(CUBE_SIZE + CUBE_WALL_THICKNESS, CUBE_SIZE - CUBE_WALL_THICKNESS * 2.0, CUBE_WALL_THICKNESS),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(CUBE_SIZE + CUBE_WALL_THICKNESS, CUBE_SIZE - CUBE_WALL_THICKNESS * 2.0, CUBE_WALL_THICKNESS)),
            material: material.clone(),
            transform: Transform::from_translation(REACTOR_POSITION + Vec3::new(0.0, 0.0, CUBE_SIZE/2.0)),
            ..default()
        },
    ));

    // Back wall (full size)
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(CUBE_SIZE + CUBE_WALL_THICKNESS, CUBE_SIZE - CUBE_WALL_THICKNESS * 2.0, CUBE_WALL_THICKNESS),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(CUBE_SIZE + CUBE_WALL_THICKNESS, CUBE_SIZE - CUBE_WALL_THICKNESS * 2.0, CUBE_WALL_THICKNESS)),
            material: material.clone(),
            transform: Transform::from_translation(REACTOR_POSITION + Vec3::new(0.0, 0.0, -(CUBE_SIZE/2.0))),
            ..default()
        },
    ));

    // Left wall (reduced depth to fit between front/back walls)
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(CUBE_WALL_THICKNESS, CUBE_SIZE - CUBE_WALL_THICKNESS, CUBE_SIZE - CUBE_WALL_THICKNESS),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(CUBE_WALL_THICKNESS, CUBE_SIZE - CUBE_WALL_THICKNESS, CUBE_SIZE - CUBE_WALL_THICKNESS)),
            material: material.clone(),
            transform: Transform::from_translation(REACTOR_POSITION + Vec3::new(-(CUBE_SIZE/2.0), 0.0, 0.0)),
            ..default()
        },
    ));

    // Right wall (reduced depth to fit between front/back walls)
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(CUBE_WALL_THICKNESS, CUBE_SIZE - CUBE_WALL_THICKNESS, CUBE_SIZE - CUBE_WALL_THICKNESS),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(CUBE_WALL_THICKNESS, CUBE_SIZE - CUBE_WALL_THICKNESS, CUBE_SIZE - CUBE_WALL_THICKNESS)),
            material: material.clone(),
            transform: Transform::from_translation(REACTOR_POSITION + Vec3::new(CUBE_SIZE/2.0, 0.0, 0.0)),
            ..default()
        },
    ));

    // God ray shooting down from upper cube's aperture
    let god_ray_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.0, 0.0, 0.15),
        emissive: Color::srgb(0.8, 0.0, 0.0).into(),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(HOLE_SIZE, GOD_RAY_HEIGHT, HOLE_SIZE)),
        material: god_ray_material,
        transform: Transform::from_translation(
            Vec3::new(
                REACTOR_POSITION.x + (time.elapsed_seconds() * 3.0).sin() * 15.0,
                REACTOR_POSITION.y - CUBE_SIZE/2.0 - GOD_RAY_HEIGHT/2.0,
                REACTOR_POSITION.z + (time.elapsed_seconds() * 3.0).cos() * 15.0
            )
        ),
        ..default()
    });

    // Add structural support beams in corners (now full height)
    let beam_material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/rusty_metal_03_diff_4k.png")),
        metallic: 0.9,
        perceptual_roughness: 0.2,
        ..default()
    });

    // Add sconce material definition
    let sconce_material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.8, 0.6, 0.2),
        emissive: Color::rgb(3.0, 1.5, 0.375).into(),
        metallic: 0.5,
        perceptual_roughness: 0.4,
        ..default()
    });

    // Main support pillars (now cuboids with varying heights)
    let pillar_material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/metal_plate_02_diff_4k.png")),
        base_color: Color::rgb(0.1, 0.1, 0.1), // Very dark tint
        metallic: 0.1,                         // Much less metallic
        perceptual_roughness: 0.9,             // Very rough (not shiny)
        ..default()
    });

    let mut rng = rand::thread_rng();
    for i in 0..12 {
        let angle = (i as f32) * std::f32::consts::PI / 6.0;
        let pillar_radius = CUBE_SIZE * rng.gen_range(0.25..0.4);
        let pillar_pos = Vec3::new(
            angle.cos() * pillar_radius,
            0.0,
            angle.sin() * pillar_radius
        );
        
        let above_ground_height = CUBE_SIZE * rng.gen_range(PILLAR_MIN_HEIGHT_FACTOR..PILLAR_MAX_HEIGHT_FACTOR);
        let below_ground_height = PILLAR_BELOW_GROUND_HEIGHT;
        let total_height = above_ground_height + below_ground_height;
        let pillar_width = rng.gen_range(PILLAR_MIN_WIDTH..PILLAR_MAX_WIDTH);
        
        commands.spawn((
            RigidBody::Static,
            Collider::cuboid(pillar_width, total_height, pillar_width),
            PbrBundle {
                mesh: meshes.add(Cuboid::new(pillar_width, total_height, pillar_width)),
                material: pillar_material.clone(),
                transform: Transform::from_translation(
                    REACTOR_POSITION + pillar_pos + Vec3::new(0.0, (above_ground_height - below_ground_height)/2.0, 0.0)
                ).with_rotation(Quat::from_rotation_y(rng.gen_range(0.0..std::f32::consts::PI / 6.0))),
                ..default()
            },
        ));

        // Adjust sconce placement for new pillar heights
        for height_level in [-0.8, -0.6, -0.45, -0.35, -0.2, 0.2, 0.35, 0.45, 0.6, 0.8] {
            let actual_height = height_level * above_ground_height;
            for sconce_angle in 0..4 {
                let sconce_rotation = Quat::from_rotation_y(sconce_angle as f32 * std::f32::consts::PI * 2.0 / 4.0);
                let offset = sconce_rotation * Vec3::new(pillar_width/2.0 + 1.0, 0.0, 0.0);
                
                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(Cylinder::new(1.0, 0.6)),
                        material: sconce_material.clone(),
                        transform: Transform::from_translation(
                            REACTOR_POSITION + pillar_pos + offset + Vec3::new(0.0, actual_height, 0.0)
                        ).with_rotation(sconce_rotation),
                        ..default()
                    },
                    TwinklingLight {
                        base_intensity: 3.0,
                        phase_offset: rng.gen_range(0.0..std::f32::consts::PI * 2.0),
                    },
                ));
            }
        }

        // Add blue sconces with random placement
        let num_blue_sconces = rng.gen_range(100..200); 
        for _ in 0..num_blue_sconces {
            let height_level = rng.gen_range(-0.7..0.7) * above_ground_height;
            let sconce_angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
            let sconce_rotation = Quat::from_rotation_y(sconce_angle);
            let offset = sconce_rotation * Vec3::new(pillar_width/2.0 + 1.0, 0.0, 0.0);
            
            let blue_sconce_material = materials.add(StandardMaterial {
                base_color: Color::srgb(0.0, 0.1, 0.4),      // Very deep blue base
                emissive: Color::srgb(0.0, 0.3, 2.0).into(), // Reduced intensity but still blue-dominant
                metallic: 0.2,                                // Keep low metallic for glow
                perceptual_roughness: 0.1,                    // Keep smooth for shine
                ..default()
            });

            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Cylinder::new(0.8, 0.4)),
                    material: blue_sconce_material,
                    transform: Transform::from_translation(
                        REACTOR_POSITION + pillar_pos + offset + Vec3::new(0.0, height_level, 0.0)
                    ).with_rotation(sconce_rotation),
                    ..default()
                },
                PulsingBlueLight {
                    phase_offset: rng.gen_range(0.0..std::f32::consts::PI * 2.0),
                },
            ));
        }
    }

    // Add ladder in reactor
    let ladder_config = LadderConfig {
        position: REACTOR_POSITION + Vec3::new(0.0, -REACTOR_POSITION.y, -50.0),
        rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2), // 90 degree rotation
        height: CUBE_SIZE - CUBE_WALL_THICKNESS - 50.0, // Reduce height to stop at roof level
        rung_count: 1000,
    };
    spawn_ladder(commands, meshes, materials, asset_server, ladder_config);

    // Add teleport trigger zone at top of ladder
    commands.spawn((
        Sensor,
        Collider::cuboid(30.0, 30.0, 30.0),
        PbrBundle {
            transform: Transform::from_translation(
                REACTOR_POSITION + Vec3::new(0.0, CUBE_SIZE/2.0 - CUBE_WALL_THICKNESS - 50.0, -50.0)
            ),
            ..default()
        },
        Name::new("ReactorLadderExit"),
    ));
}

pub fn handle_reactor_ladder_exit(
    mut collision_events: EventReader<CollisionStarted>,
    mut protagonist_query: Query<(&mut Transform, &mut Protagonist)>,
    name_query: Query<&Name>,
) {
    for CollisionStarted(entity1, entity2) in collision_events.read() {
        // Try both entities to find which one is the protagonist
        let (protagonist_entity, other_entity) = if protagonist_query.contains(*entity1) {
            (entity1, entity2)
        } else if protagonist_query.contains(*entity2) {
            (entity2, entity1)
        } else {
            continue;
        };

        // Check if other entity is the ladder exit
        if let Ok(name) = name_query.get(*other_entity) {
            if name.as_str() == "ReactorLadderExit" {
                if let Ok((mut transform, mut protagonist)) = protagonist_query.get_mut(*protagonist_entity) {
                    transform.translation = REACTOR_POSITION + Vec3::new(0.0, CUBE_SIZE/2.0 + CUBE_WALL_THICKNESS/2.0, 0.0);
                    protagonist.is_climbing = false;
                }
            }
        }
    }
}
