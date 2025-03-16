use bevy::prelude::*;
use avian3d::prelude::*;

// Constants for structure dimensions
pub const WALL_HEIGHT: f32 = 800.0;
pub const WALL_Y_POSITION: f32 = WALL_HEIGHT/2.0;
const WALL_NORTH_POSITION: Vec3 = Vec3::new(0.0, WALL_Y_POSITION, -150.0);
const WALL_SOUTH_POSITION: Vec3 = Vec3::new(0.0, WALL_Y_POSITION, 150.0);
const WALL_EAST_POSITION: Vec3 = Vec3::new(120.0, WALL_Y_POSITION, 0.0);
const WALL_WEST_POSITION: Vec3 = Vec3::new(-120.0, WALL_Y_POSITION, 0.0);

pub fn spawn_launch_silo(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
) {
    // North wall with high friction
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(300.0, WALL_HEIGHT, 60.0),
        Friction::new(10.0),  // Add high friction
        PbrBundle {
            mesh: meshes.add(Cuboid::new(300.0, WALL_HEIGHT, 60.0)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/concrete.png")),
                metallic: 1.0,
                ..default()
            }),
            transform: Transform::from_translation(WALL_NORTH_POSITION),
            ..default()
        },
    ));

    // South wall with high friction
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(300.0, WALL_HEIGHT, 60.0),
        Friction::new(10.0),  // Add high friction
        PbrBundle {
            mesh: meshes.add(Cuboid::new(300.0, WALL_HEIGHT, 60.0)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/concrete.png")),
                metallic: 1.0,
                ..default()
            }),
            transform: Transform::from_translation(WALL_SOUTH_POSITION),
            ..default()
        },
    ));

    // East wall with high friction
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(60.0, WALL_HEIGHT, 240.0),
        Friction::new(10.0),  // Add high friction
        PbrBundle {
            mesh: meshes.add(Cuboid::new(60.0, WALL_HEIGHT, 240.0)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/concrete.png")),
                metallic: 1.0,
                ..default()
            }),
            transform: Transform::from_translation(WALL_EAST_POSITION),
            ..default()
        },
    ));

    // West wall with high friction
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(60.0, WALL_HEIGHT, 240.0),
        Friction::new(10.0),  // Add high friction
        PbrBundle {
            mesh: meshes.add(Cuboid::new(60.0, WALL_HEIGHT, 240.0)),
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
