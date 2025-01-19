use bevy::prelude::*;
use avian3d::prelude::*;
use rand::Rng;

const CUBE_WALL_THICKNESS: f32 = 20.0;
const CUBE_SIZE: f32 = 200.0; // Size of the inner hollow space
const REACTOR_POSITION: Vec3 = Vec3::new(-485.34103, 2.6249764, -1066.1226);
const HOLE_SIZE: f32 = 50.0; // Size of the opening in the floor
const GOD_RAY_HEIGHT: f32 = 400.0; // Height of the god ray volume
const WATER_DEPTH: f32 = 20.0; // Depth of the water pool

#[derive(Component)]
pub struct TwinklingLight {
    base_intensity: f32,
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
        metallic: 0.8,
        perceptual_roughness: 0.3,
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
         Vec3::new(CUBE_SIZE + CUBE_WALL_THICKNESS, CUBE_WALL_THICKNESS, (CUBE_SIZE - HOLE_SIZE)/2.0)),
        // Back segment
        (Vec3::new(0.0, 0.0, -(HOLE_SIZE + CUBE_SIZE)/4.0),
         Vec3::new(CUBE_SIZE + CUBE_WALL_THICKNESS, CUBE_WALL_THICKNESS, (CUBE_SIZE - HOLE_SIZE)/2.0)),
        // Left segment
        (Vec3::new(-(HOLE_SIZE + CUBE_SIZE)/4.0, 0.0, 0.0),
         Vec3::new((CUBE_SIZE - HOLE_SIZE)/2.0, CUBE_WALL_THICKNESS, HOLE_SIZE)),
        // Right segment
        (Vec3::new((HOLE_SIZE + CUBE_SIZE)/4.0, 0.0, 0.0),
         Vec3::new((CUBE_SIZE - HOLE_SIZE)/2.0, CUBE_WALL_THICKNESS, HOLE_SIZE)),
    ];

    // Lower floor segments with new texture
    let lower_floor_material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/concrete_floor_worn_001_diff_4k.png")),
        metallic: 0.0,
        perceptual_roughness: 0.9,
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

    // Water volume in the hole (adjusted to sit at y=0)
    let water_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 0.5, 1.0, 0.8),
        emissive: Color::srgb(0.2, 0.4, 0.8).into(),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(HOLE_SIZE, WATER_DEPTH, HOLE_SIZE)),
        material: water_material,
        transform: Transform::from_translation(
            REACTOR_POSITION + lower_floor_position + Vec3::new(0.0, 0.0, 0.0)
        ),
        ..default()
    });

    // Top wall
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(CUBE_SIZE + CUBE_WALL_THICKNESS, CUBE_WALL_THICKNESS, CUBE_SIZE + CUBE_WALL_THICKNESS),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(CUBE_SIZE + CUBE_WALL_THICKNESS, CUBE_WALL_THICKNESS, CUBE_SIZE + CUBE_WALL_THICKNESS)),
            material: material.clone(),
            transform: Transform::from_translation(REACTOR_POSITION + Vec3::new(0.0, CUBE_SIZE/2.0, 0.0)),
            ..default()
        },
    ));

    // Front wall
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(CUBE_SIZE + CUBE_WALL_THICKNESS, CUBE_SIZE, CUBE_WALL_THICKNESS),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(CUBE_SIZE + CUBE_WALL_THICKNESS, CUBE_SIZE, CUBE_WALL_THICKNESS)),
            material: material.clone(),
            transform: Transform::from_translation(REACTOR_POSITION + Vec3::new(0.0, 0.0, CUBE_SIZE/2.0)),
            ..default()
        },
    ));

    // Back wall
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(CUBE_SIZE + CUBE_WALL_THICKNESS, CUBE_SIZE, CUBE_WALL_THICKNESS),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(CUBE_SIZE + CUBE_WALL_THICKNESS, CUBE_SIZE, CUBE_WALL_THICKNESS)),
            material: material.clone(),
            transform: Transform::from_translation(REACTOR_POSITION + Vec3::new(0.0, 0.0, -(CUBE_SIZE/2.0))),
            ..default()
        },
    ));

    // Left wall
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(CUBE_WALL_THICKNESS, CUBE_SIZE, CUBE_SIZE),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(CUBE_WALL_THICKNESS, CUBE_SIZE, CUBE_SIZE)),
            material: material.clone(),
            transform: Transform::from_translation(REACTOR_POSITION + Vec3::new(-(CUBE_SIZE/2.0), 0.0, 0.0)),
            ..default()
        },
    ));

    // Right wall
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(CUBE_WALL_THICKNESS, CUBE_SIZE, CUBE_SIZE),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(CUBE_WALL_THICKNESS, CUBE_SIZE, CUBE_SIZE)),
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
        mesh: meshes.add(Cuboid::new(HOLE_SIZE * 1.2, GOD_RAY_HEIGHT, HOLE_SIZE * 1.2)),
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

    // Main support pillars (shortened)
    for i in 0..6 {
        let angle = (i as f32) * std::f32::consts::PI / 3.0;
        let pillar_pos = Vec3::new(
            angle.cos() * (CUBE_SIZE * 0.35),
            CUBE_SIZE * 0.5, // Lowered center point
            angle.sin() * (CUBE_SIZE * 0.35)
        );
        
        // Shortened pillar with collider
        commands.spawn((
            RigidBody::Static,
            Collider::cylinder(CUBE_SIZE, 8.0), // Shortened height
            PbrBundle {
                mesh: meshes.add(Cylinder::new(8.0, CUBE_SIZE * 2.0)), // Shortened height
                material: material.clone(),
                transform: Transform::from_translation(REACTOR_POSITION + pillar_pos),
                ..default()
            },
        ));

        // Add sconces around pillar at various heights
        for height_level in [-0.8, -0.6, -0.45, -0.35, -0.2, 0.2, 0.35, 0.45, 0.6, 0.8] {
            for sconce_angle in 0..3 {
                let sconce_rotation = Quat::from_rotation_y(sconce_angle as f32 * std::f32::consts::PI * 2.0 / 3.0);
                let offset = sconce_rotation * Vec3::new(9.0, 0.0, 0.0);
                
                commands.spawn(PbrBundle {
                    mesh: meshes.add(Cylinder::new(1.5, 1.0)),
                    material: sconce_material.clone(),
                    transform: Transform::from_translation(
                        REACTOR_POSITION + pillar_pos + offset + Vec3::new(0.0, CUBE_SIZE * height_level, 0.0)
                    ).with_rotation(sconce_rotation),
                    ..default()
                });
            }
        }
    }

    // Remove the old upper cube support pillars section and continue with random sconces
    let mut rng = rand::thread_rng();
    
    // Add random sconces to the full-height pillars
    for i in 0..6 {
        let angle = (i as f32) * std::f32::consts::PI / 3.0;
        let pillar_base_pos = Vec3::new(
            angle.cos() * (CUBE_SIZE * 0.35),
            0.0,  // Start from the bottom
            angle.sin() * (CUBE_SIZE * 0.35)
        );
        
        // Add 6-10 random sconces per pillar
        let num_sconces = rng.gen_range(6..11);
        for _ in 0..num_sconces {
            // Height offset from pillar base, keeping within the second cube's bounds
            let max_height = CUBE_SIZE - CUBE_WALL_THICKNESS; // Don't go past the top wall
            let height_offset = rng.gen_range(10.0..max_height - 10.0);
            let rotation_angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
            let distance_from_pillar = 8.0; // Fixed distance to attach to pillar
            
            let sconce_rotation = Quat::from_rotation_y(rotation_angle);
            let offset = sconce_rotation * Vec3::new(distance_from_pillar, 0.0, 0.0);
            
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Cylinder::new(2.0, 1.5)),
                    material: sconce_material.clone(),
                    transform: Transform::from_translation(
                        REACTOR_POSITION + pillar_base_pos + offset + Vec3::new(0.0, height_offset, 0.0)
                    ).with_rotation(sconce_rotation),
                    ..default()
                },
                TwinklingLight {
                    base_intensity: rng.gen_range(1.5..4.0),
                    phase_offset: rng.gen_range(0.0..std::f32::consts::PI * 2.0),
                }
            ));
        }
    }
}

pub fn update_twinkling_lights(
    time: Res<Time>,
    mut query: Query<(&TwinklingLight, &mut Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::thread_rng();
    
    for (light, material_handle) in query.iter_mut() {
        if let Some(material) = materials.get_mut(&*material_handle) {
            // Reduced chance to change state (about once every 5-20 seconds on average)
            if rng.gen::<f32>() < time.delta_seconds() * 0.05 {
                // 85% chance to be on when changing state
                let is_on = rng.gen::<f32>() < 0.85;
                let intensity = if is_on {
                    // Smoother transition to full brightness
                    light.base_intensity * rng.gen_range(0.9..1.0)
                } else {
                    0.0
                };
                material.emissive = Color::srgb(intensity, intensity * 0.5, intensity * 0.125).into();
            }
        }
    }
}
