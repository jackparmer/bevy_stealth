use bevy::prelude::*;
use avian3d::prelude::*;

// Constants for structure dimensions
pub const WALL_HEIGHT: f32 = 600.0;
pub const WALL_Y_POSITION: f32 = 100.0;
const WALL_NORTH_POSITION: Vec3 = Vec3::new(0.0, WALL_Y_POSITION, -50.0);
const WALL_SOUTH_POSITION: Vec3 = Vec3::new(0.0, WALL_Y_POSITION, 50.0);
const WALL_EAST_POSITION: Vec3 = Vec3::new(40.0, WALL_Y_POSITION, 0.0);
const WALL_WEST_POSITION: Vec3 = Vec3::new(-40.0, WALL_Y_POSITION, 0.0);

pub fn spawn_launch_silo(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
) {
    // North wall
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(100.0, WALL_HEIGHT, 20.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(100.0, WALL_HEIGHT, 20.0)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/concrete.png")),
                metallic: 1.0,
                ..default()
            }),
            transform: Transform::from_translation(WALL_NORTH_POSITION),
            ..default()
        },
    ));

    // South wall 
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(100.0, WALL_HEIGHT, 20.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(100.0, WALL_HEIGHT, 20.0)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/concrete.png")),
                metallic: 1.0,
                ..default()
            }),
            transform: Transform::from_translation(WALL_SOUTH_POSITION),
            ..default()
        },
    ));

    // East wall
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(20.0, WALL_HEIGHT, 80.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(20.0, WALL_HEIGHT, 80.0)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/concrete.png")),
                metallic: 1.0,
                ..default()
            }),
            transform: Transform::from_translation(WALL_EAST_POSITION),
            ..default()
        },
    ));

    // West wall
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(20.0, WALL_HEIGHT, 80.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(20.0, WALL_HEIGHT, 80.0)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/concrete.png")),
                metallic: 1.0,
                ..default()
            }),
            transform: Transform::from_translation(WALL_WEST_POSITION),
            ..default()
        },
    ));
}
