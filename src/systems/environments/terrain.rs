use bevy::prelude::*;
use bevy::render::mesh::Indices;
use avian3d::prelude::*;
use avian3d::collision::{ColliderMarker, CollisionLayers, LayerMask, PhysicsLayer};
use noise::OpenSimplex;
use noise::NoiseFn;
use crate::components::Protagonist;

// Terrain Generation Parameters
const TERRAIN_RADIUS: f32 = 5000.0;
const TERRAIN_RESOLUTION: i32 = 1000;
const TERRAIN_SEED: u32 = 42;
const BASE_HEIGHT: f32 = -30.0;
const Y_OFFSET: f32 = 30.0;

// Noise Parameters
const OCTAVE_COUNT: i32 = 5;
const FREQUENCY_BASE: f64 = 2.0;
const AMPLITUDE_BASE: f64 = 0.5;
const NOISE_SCALE: f64 = 0.002;
const HEIGHT_MULTIPLIER: f64 = 100.0;

#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
pub enum GameLayer {
    #[default]
    Default,
    Terrain,
}

#[derive(Component)]
pub struct Terrain {
    pub is_icy: bool,
}

pub fn spawn_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let noise = OpenSimplex::new(TERRAIN_SEED);
    
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut uvs = Vec::new();
    
    let mut vertex_map = vec![vec![-1i32; TERRAIN_RESOLUTION as usize + 1]; TERRAIN_RESOLUTION as usize + 1];
    let mut vertex_count = 0;
    
    // Generate vertices in a grid
    for z in 0..=TERRAIN_RESOLUTION {
        for x in 0..=TERRAIN_RESOLUTION {
            let px = (x as f32 / TERRAIN_RESOLUTION as f32) * 2.0 - 1.0;
            let pz = (z as f32 / TERRAIN_RESOLUTION as f32) * 2.0 - 1.0;
            
            let dist = (px * px + pz * pz).sqrt();
            if dist <= 1.0 {
                let wx = px * TERRAIN_RADIUS;
                let wz = pz * TERRAIN_RADIUS;
                
                let height = (0..OCTAVE_COUNT).map(|octave| {
                    let frequency = FREQUENCY_BASE.powi(octave);
                    let amplitude = AMPLITUDE_BASE.powi(octave);
                    noise.get([
                        wx as f64 * NOISE_SCALE * frequency,
                        wz as f64 * NOISE_SCALE * frequency
                    ]) * amplitude * HEIGHT_MULTIPLIER
                }).sum::<f64>() as f32;

                let final_height = height + BASE_HEIGHT;
                vertices.push([wx, final_height, wz]);
                uvs.push([
                    (px + 1.0) * 0.5,
                    (pz + 1.0) * 0.5,
                ]);
                
                vertex_map[z as usize][x as usize] = vertex_count;
                vertex_count += 1;
            }
        }
    }
    
    // Generate triangles
    for z in 0..TERRAIN_RESOLUTION {
        for x in 0..TERRAIN_RESOLUTION {
            let tl = vertex_map[z as usize][x as usize];
            let tr = vertex_map[z as usize][(x + 1) as usize];
            let bl = vertex_map[(z + 1) as usize][x as usize];
            let br = vertex_map[(z + 1) as usize][(x + 1) as usize];
            
            if tl >= 0 && tr >= 0 && bl >= 0 && br >= 0 {
                indices.extend_from_slice(&[
                    tl as u32, tr as u32, bl as u32,
                    tr as u32, br as u32, bl as u32,
                ]);
            }
        }
    }
    
    // Calculate normals
    let mut normals = vec![[0.0, 1.0, 0.0]; vertices.len()];
    for chunk in indices.chunks(3) {
        if let [i1, i2, i3] = chunk {
            let v1 = Vec3::from(vertices[*i1 as usize]);
            let v2 = Vec3::from(vertices[*i2 as usize]);
            let v3 = Vec3::from(vertices[*i3 as usize]);
            
            let normal = (v2 - v1).cross(v3 - v1).normalize();
            normals[*i1 as usize] = normal.into();
            normals[*i2 as usize] = normal.into();
            normals[*i3 as usize] = normal.into();
        }
    }

    let heights = vertices.clone().iter()
        .map(|v| v[1])
        .collect::<Vec<f32>>()
        .chunks(TERRAIN_RESOLUTION as usize + 1)
        .step_by(5)  // Take every 5th row
        .map(|chunk| {
            chunk.iter()
                .step_by(5)  // Take every 5th column
                .take(200)   // Fixed width
                .cloned()
                .map(|h| if h == 0.0 { BASE_HEIGHT } else { h })  // Replace invalid points
                .collect::<Vec<f32>>()
        })
        .take(200)   // Fixed height
        .collect::<Vec<Vec<f32>>>();

    let heights_width = heights[0].len();
    let heights_height = heights.len();

    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    commands.spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/8k_mars.png")),
                perceptual_roughness: 1.0,
                metallic: 0.0,
                reflectance: 0.0,
                alpha_mode: AlphaMode::Opaque,
                double_sided: true,
                cull_mode: None,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, Y_OFFSET, 0.0),
            ..default()
        })
        .insert(ColliderConstructor::TrimeshFromMesh)
        .insert(RigidBody::Static)
        .insert(ColliderMarker)
        .insert(Friction {
            dynamic_coefficient: 0.2,
            static_coefficient: 0.2,
            combine_rule: CoefficientCombine::Min,
        })
        .insert(Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombine::Min,
        })
        .insert(CollisionLayers {
            memberships: GameLayer::Terrain.into(),  // Terrain layer
            filters: LayerMask(0b111),  // Can collide with Default, Player, and Terrain
        })
        .insert(Terrain { is_icy: false });
}

pub fn toggle_terrain_texture(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut terrain_query: Query<(&mut Terrain, &Handle<StandardMaterial>)>,
    protagonist_query: Query<(&Transform, &Protagonist)>,
) {
    if keyboard.just_pressed(KeyCode::KeyI) {
        for (mut terrain, material_handle) in terrain_query.iter_mut() {
            terrain.is_icy = !terrain.is_icy;
            if let Some(material) = materials.get_mut(material_handle) {
                material.base_color_texture = Some(
                    if terrain.is_icy {
                        asset_server.load("textures/ice_texture.png")
                    } else {
                        asset_server.load("textures/8k_mars.png")
                    }
                );
            }
        }
    }

    if let Ok((_protagonist_transform, protagonist)) = protagonist_query.get_single() {
        let should_be_icy = protagonist.is_swimming;

        for (mut terrain, material_handle) in terrain_query.iter_mut() {
            if terrain.is_icy != should_be_icy {
                terrain.is_icy = should_be_icy;
                if let Some(material) = materials.get_mut(material_handle) {
                    material.base_color_texture = Some(
                        if should_be_icy {
                            asset_server.load("textures/ice_texture.png")
                        } else {
                            asset_server.load("textures/8k_mars.png")
                        }
                    );
                }
            }
        }
    }
}
