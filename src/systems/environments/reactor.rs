use bevy::prelude::*;
use avian3d::prelude::*;

const CUBE_WALL_THICKNESS: f32 = 20.0;
const CUBE_SIZE: f32 = 200.0; // Size of the inner hollow space
const REACTOR_POSITION: Vec3 = Vec3::new(-485.34103, 2.6249764, -1066.1226);

pub fn spawn_reactor(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
) {
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/rusty_metal_03_diff_4k.png")),
        metallic: 1.0,
        ..default()
    });

    // Bottom wall
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(CUBE_SIZE + CUBE_WALL_THICKNESS, CUBE_WALL_THICKNESS, CUBE_SIZE + CUBE_WALL_THICKNESS),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(CUBE_SIZE + CUBE_WALL_THICKNESS, CUBE_WALL_THICKNESS, CUBE_SIZE + CUBE_WALL_THICKNESS)),
            material: material.clone(),
            transform: Transform::from_translation(REACTOR_POSITION + Vec3::new(0.0, -(CUBE_SIZE/2.0), 0.0)),
            ..default()
        },
    ));

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
}
