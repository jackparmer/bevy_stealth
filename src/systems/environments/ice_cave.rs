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
    let sphere_pos = Vec3::new(-455.0, 1.6, 915.0);
    let radius = 400.0;
    let segments = 256; // Resolution of the sphere
    let noise_scale = 8.0; 
    let noise_amplitude = 200.0; 
    let opening_size = 0.5; // Size of the hole (0.0 to 1.0)

    let perlin = Perlin::new(42); // Seed of 42

    // Generate vertices and indices for the sphere
    let mut vertices: Vec<Vec3> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut normals: Vec<Vec3> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();

    // Generate vertices
    let mut vertex_count = 0;
    let mut vertex_map = vec![vec![-1i32; segments as usize + 1]; segments as usize + 1];

    // Generate vertices
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
        base_color_texture: Some(asset_server.load("textures/ice_texture2.png")),
        base_color: Color::srgb(0.7, 0.8, 1.0),  // Added blue tint
        perceptual_roughness: 0.9,
        metallic: 0.1,
        double_sided: true,
        cull_mode: None,
        unlit: false,
        reflectance: 1.0,
        emissive: Color::srgb(0.0, 0.02, 0.1).into(),  // Added blue emissive glow
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

    // Parameters for the cube layer
    let cube_size = 80.0;

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

    // Spawn cubes
    for (i, vertex) in vertices.iter().enumerate() {
        if i % 6 != 0 {
            continue;
        }

        let direction = -vertex.normalize();
        let position = sphere_pos + *vertex + (direction * (cube_size / 2.0));

        let radius_variation = cube_size * (0.8 + fastrand::f32() * 0.4);
        let hue_variation = Color::hsl(
            200.0 + fastrand::f32() * 20.0,  // blue hue range: 200-220
            0.1 + fastrand::f32() * 0.2,     // low saturation: 0.1-0.3
            0.8 + fastrand::f32() * 0.2,     // high lightness: 0.8-1.0
        );
        let emissive_strength = fastrand::f32() * 0.15; // Random emissive strength between 0 and 0.15

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
                metallic: 0.1,
                perceptual_roughness: 0.7,
                reflectance: 0.3,
                emissive: Color::srgb(0.0, emissive_strength, emissive_strength * 2.0).into(),
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

    // Add ring of spotlights around interior perimeter with water basins
    let num_lights = 1;
    let ring_radius = 70.0;
    let light_height = 1.0;
    
    for i in 0..num_lights {
        let angle = (i as f32 / num_lights as f32) * 2.0 * PI;
        let x = angle.cos() * ring_radius;
        let z = angle.sin() * ring_radius;
        let light_pos = sphere_pos + Vec3::new(x, light_height, z);
        
        // Spawn spotlight pointing sideways (tangent to the circle)
        commands.spawn(SpotLightBundle {
            spot_light: SpotLight {
                intensity: 10000000000.0,
                color: Color::srgb(0.1, 0.5, 1.0),
                shadows_enabled: true,
                outer_angle: 1.2,
                inner_angle: 0.8,
                range: 200.0,
                radius: 8.0,
                ..default()
            },
            transform: Transform::from_translation(light_pos)
                .looking_to(Vec3::new(-z, 0.0, x).normalize(), Vec3::Y),
            ..default()
        });

        // Spawn water basin at each spotlight location
        spawn_water_feature(commands, meshes, materials, light_pos);
    }

    // Remove the original water feature spawn call since it's now handled in the loop
}

pub fn spawn_water_feature(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    basin_pos: Vec3,
) {
    // Dimensions
    let wall_height = 2.5;  // Reduced from 4.0
    let wall_thickness = 4.0;  // Increased from 0.67
    let basin_width = 20.0;  // Kept the same
    
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
        mesh: meshes.add(Cuboid::new(19.33, 0.033, 19.33)),
        material: materials.add(StandardMaterial {
            base_color: Color::srgba(0.0, 0.8, 1.0, 0.8),
            emissive: Color::srgba(0.0, 1.0, 2.0, 1.0).into(),
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        transform: Transform::from_translation(basin_pos + Vec3::new(0.0, wall_height - 1.0, 0.0)),  // 1 meter below wall height
        ..default()
    })
    .insert(NotShadowCaster);
}
