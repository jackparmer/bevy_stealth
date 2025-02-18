use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use std::f32::consts::PI;
use avian3d::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::mesh::Indices;
use bevy::pbr::{StandardMaterial, NotShadowCaster};
use fastrand;

pub fn spawn_ice_cave(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
) {
    let sphere_pos = Vec3::new(-485.0 * 2.0, 2.6249764, -1066.0 * 2.0);
    let radius = 400.0;
    let segments = 128; // Reduced from 256 for better performance while maintaining good visual quality
    let noise_scale = 8.0;
    let noise_amplitude = 200.0;
    let opening_size = 0.5;
    let cube_size = 80.0;

    let perlin = Perlin::new(42);

    // Pre-allocate vectors with capacity
    let estimated_vertex_count = (segments + 1) * (segments + 1);
    let mut vertices = Vec::with_capacity(estimated_vertex_count);
    let mut normals = Vec::with_capacity(estimated_vertex_count);
    let mut uvs = Vec::with_capacity(estimated_vertex_count);
    let mut vertex_map = vec![vec![-1i32; segments as usize + 1]; segments as usize + 1];

    // Generate vertices
    let mut vertex_count = 0;
    for lat in 0..=segments {
        for lon in 0..=segments {
            let theta = (lat as f32 * PI) / segments as f32;
            let phi = (lon as f32 * 2.0 * PI) / segments as f32;
            
            // Skip vertices to create a hole facing the terrain
            if phi > 1.5 * PI && phi < 1.5 * PI + opening_size * PI && 
               theta > PI * 0.3 && theta < PI * 0.7 {
                vertex_map[lat as usize][lon as usize] = -1;
                continue;
            }
            
            let x = theta.sin() * phi.cos();
            let y = theta.cos();
            let z = theta.sin() * phi.sin();

            // Apply noise deformation
            let noise_input = [
                x as f64 * noise_scale as f64,
                y as f64 * noise_scale as f64,
                z as f64 * noise_scale as f64,
            ];
            let noise_value = perlin.get(noise_input) as f32;
            
            let deformed_radius = radius + noise_value * noise_amplitude;
            let vertex = Vec3::new(
                x * deformed_radius,
                y * deformed_radius,
                z * deformed_radius,
            );

            vertices.push(vertex);
            normals.push(vertex.normalize());
            uvs.push([lon as f32 / segments as f32, lat as f32 / segments as f32]);
            
            vertex_map[lat as usize][lon as usize] = vertex_count;
            vertex_count += 1;
        }
    }

    // Pre-allocate indices vector with estimated capacity
    let estimated_index_count = segments as usize * segments as usize * 6;
    let mut indices = Vec::with_capacity(estimated_index_count);

    // Generate indices
    for lat in 0..segments {
        for lon in 0..segments {
            let tl = vertex_map[lat as usize][lon as usize];
            let tr = vertex_map[lat as usize][(lon + 1) as usize];
            let bl = vertex_map[(lat + 1) as usize][lon as usize];
            let br = vertex_map[(lat + 1) as usize][(lon + 1) as usize];
            
            // Only create triangles if all vertices exist
            if tl >= 0 && tr >= 0 && bl >= 0 && br >= 0 {
                indices.extend_from_slice(&[
                    tl as u32, bl as u32, tr as u32,
                    tr as u32, bl as u32, br as u32,
                ]);
            }
        }
    }

    // Create the mesh
    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, 
        VertexAttributeValues::Float32x3(vertices.iter().map(|v| [v.x, v.y, v.z]).collect()));
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, 
        VertexAttributeValues::Float32x3(normals.iter().map(|n| [n.x, n.y, n.z]).collect()));
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));
    mesh.insert_indices(Indices::U32(indices.clone()));

    // Create material with a rocky texture - adjusted for better interior lighting
    let material = materials.add(StandardMaterial {
        base_color_texture: None,
        base_color: Color::srgb(0.01, 0.01, 0.01),  // Nearly pure black
        perceptual_roughness: 0.5,  // More roughness to reduce reflections
        metallic: 0.2,  // Much less metallic
        double_sided: true,
        cull_mode: None,
        unlit: false,
        reflectance: 0.1,  // Minimal reflectance
        emissive: Color::srgb(0.0, 0.0, 0.0).into(),
        alpha_mode: AlphaMode::Opaque,  // Ensure fully opaque
        ..default()
    });

    // Spawn the deformed sphere with collision
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh),
            material,
            transform: Transform::from_translation(sphere_pos),
            ..default()
        },
        Collider::trimesh(vertices.clone(), indices.chunks(3).map(|chunk| [chunk[0], chunk[1], chunk[2]]).collect()),
        ColliderDensity(1.0),
        RigidBody::Static,
    ));

    // Generate vertices for cube positions (but don't create the sphere mesh)
    let mut vertices: Vec<Vec3> = Vec::new();
    
    for lat in 0..=segments {
        for lon in 0..=segments {
            let theta = (lat as f32 * PI) / segments as f32;
            let phi = (lon as f32 * 2.0 * PI) / segments as f32;
            
            // Skip vertices to create a hole facing the terrain
            if phi > 1.5 * PI && phi < 1.5 * PI + opening_size * PI && 
               theta > PI * 0.3 && theta < PI * 0.7 {
                continue;
            }
            
            let x = theta.sin() * phi.cos();
            let y = theta.cos();
            let z = theta.sin() * phi.sin();

            // Apply noise deformation
            let noise_input = [
                x as f64 * noise_scale as f64,
                y as f64 * noise_scale as f64,
                z as f64 * noise_scale as f64,
            ];
            let noise_value = perlin.get(noise_input) as f32;
            
            let deformed_radius = radius + noise_value * noise_amplitude;
            vertices.push(Vec3::new(
                x * deformed_radius,
                y * deformed_radius,
                z * deformed_radius,
            ));
        }
    }

    // Spawn cubes more efficiently
    let cube_spawn_interval = 12; // Increased from 6 to spawn fewer cubes
    for (i, vertex) in vertices.iter().enumerate() {
        if i % cube_spawn_interval != 0 {
            continue;
        }

        let direction = -vertex.normalize();
        let position = sphere_pos + *vertex + (direction * (cube_size / 2.0));

        let radius_variation = cube_size * (0.8 + fastrand::f32() * 0.4);
        let hue_variation = Color::hsl(
            220.0 + fastrand::f32() * 40.0,  // blue-purple hue range: 220-260
            0.6 + fastrand::f32() * 0.3,     // medium-high saturation: 0.6-0.9
            0.5 + fastrand::f32() * 0.3,     // medium-high lightness: 0.5-0.8
        );
        let emissive_strength = 0.2 + fastrand::f32() * 0.4; // Stronger emissive between 0.2 and 0.6

        let texture_path = if fastrand::f32() < 0.5 {
            "textures/snow_01_diff_4k.png"
        } else {
            "textures/snow_02_diff_4k.png"
        };

        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(
                radius_variation,
                radius_variation * (0.3 + fastrand::f32() * 0.4),
                radius_variation * (0.5 + fastrand::f32() * 0.5)
            )),
            material: materials.add(StandardMaterial {
                base_color: hue_variation,
                metallic: 0.9,                // Increased metallic for crystal shine
                perceptual_roughness: 0.1,    // Decreased roughness for more shine
                reflectance: 0.8,             // Increased reflectance
                emissive: Color::srgb(
                    emissive_strength * 0.2,  // Slight red
                    emissive_strength * 0.4,  // Medium blue
                    emissive_strength         // Strong blue/purple
                ).into(),
                base_color_texture: Some(asset_server.load(texture_path)),
                uv_transform: if fastrand::bool() { 
                    StandardMaterial::FLIP_HORIZONTAL 
                } else { 
                    StandardMaterial::FLIP_VERTICAL 
                },
                ..default()
            }),
            transform: Transform::from_translation(position)
                .looking_to(direction, Vec3::Y)
                .with_rotation(Quat::from_rotation_y(fastrand::f32() * PI * 2.0)
                    * Quat::from_rotation_x(fastrand::f32() * PI * 0.5)),
            ..default()
        });
    }

    // Spawn water feature at a position relative to the cave entrance
    let water_feature_pos = sphere_pos + Vec3::new(
        radius * 0.3, // Slightly offset from center
        -radius * 0.4, // Lower in the cave
        radius * 0.7, // Towards the entrance
    );
    spawn_water_feature(commands, meshes, materials, water_feature_pos);
}

pub fn spawn_water_feature(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    basin_pos: Vec3,
) {
    // Dimensions
    let wall_height = 0.5;  // 5x lower (was 2.5)
    let wall_thickness = 4.0;
    let basin_width = 60.0;  // 3x bigger (was 20.0)
    
    // Wall configurations: (is_vertical, offset_x, offset_z)
    let wall_configs = [
        (false,  0.0,  basin_width/2.0),  // North
        (false,  0.0, -basin_width/2.0),  // South
        (true,   basin_width/2.0,  0.0),  // East
        (true,  -basin_width/2.0,  0.0),  // West
    ];

    // Spawn walls
    for (is_vertical, offset_x, offset_z) in wall_configs {
        let (width, depth) = if is_vertical {
            (wall_thickness, basin_width)
        } else {
            (basin_width, wall_thickness)
        };

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(width, wall_height, depth)),
                material: materials.add(StandardMaterial {
                    base_color: Color::srgba(0.0, 0.0, 0.0, 1.0),  // Pure black
                    perceptual_roughness: 0.1,  // More shiny
                    metallic: 0.9,  // More metallic
                    ..default()
                }),
                transform: Transform::from_translation(
                    basin_pos + Vec3::new(offset_x, 0.0, offset_z)
                ),
                ..default()
            },
            RigidBody::Static,
            Collider::cuboid(width/2.0, wall_height/2.0, depth/2.0),
        ));
    }

    // Water surface (simple flat rectangle)
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(basin_width - 0.67, 0.033, basin_width - 0.67)),
        material: materials.add(StandardMaterial {
            base_color: Color::srgba(0.0, 0.8, 1.0, 0.8),
            emissive: Color::srgba(0.0, 1.0, 2.0, 1.0).into(),
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        transform: Transform::from_translation(basin_pos + Vec3::new(0.0, wall_height - 0.2, 0.0)),
        ..default()
    })
    .insert(NotShadowCaster);

    // Replace spotlight with emissive cuboid for god ray effect
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(4.0, 400.0, 4.0)),  // Tall, thin cuboid
        material: materials.add(StandardMaterial {
            base_color: Color::rgba(0.4, 0.8, 1.0, 0.15),  // Bright blue, mostly transparent
            emissive: Color::rgb(0.4, 0.8, 1.0).into(),   // Bright blue glow
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        transform: Transform::from_translation(basin_pos + Vec3::new(0.0, 200.0, 0.0)),  // Center height of the beam
        ..default()
    })
    .insert(NotShadowCaster);  // Ensure it doesn't cast shadows
}
