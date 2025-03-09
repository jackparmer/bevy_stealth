use bevy::prelude::*;
use avian3d::prelude::*;

pub const GEOTHERMAL_BASE_HEIGHT: f32 = 250.0;
pub const GEOTHERMAL_BASE_RADIUS: f32 = 100.0;
pub const GEOTHERMAL_POSITION: Vec3 = Vec3::new(200.0, 0.0, 200.0);

pub const RADIO_TOWER_HEIGHT: f32 = 400.0;
pub const RADIO_TOWER_WIDTH: f32 = 25.0;
pub const RADIO_TOWER_POSITION: Vec3 = Vec3::new(200.0, 200.0, 200.0);

pub const BRIDGE_LENGTH: f32 = 300.0;
pub const BRIDGE_HEIGHT: f32 = 10.0;
pub const BRIDGE_WIDTH: f32 = 20.0;
pub const BRIDGE_POSITION: Vec3 = Vec3::new(100.0, 394.99, 100.0);

pub fn spawn_geothermal(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
) {
    // Geothermal station base
    commands.spawn((
        RigidBody::Static,
        Collider::capsule(GEOTHERMAL_BASE_RADIUS, GEOTHERMAL_BASE_HEIGHT),
        PbrBundle {
            mesh: meshes.add(Capsule3d {
                radius: GEOTHERMAL_BASE_RADIUS,
                half_length: GEOTHERMAL_BASE_HEIGHT / 2.0,
            }),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/concrete.png")),
                perceptual_roughness: 0.9,
                metallic: 0.1,
                ..default()
            }),
            transform: Transform::from_translation(GEOTHERMAL_POSITION),
            ..default()
        },
        Name::new("GeothermalBase"),
    ));

    // Radio tower
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(RADIO_TOWER_WIDTH, RADIO_TOWER_HEIGHT, RADIO_TOWER_WIDTH),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(RADIO_TOWER_WIDTH, RADIO_TOWER_HEIGHT, RADIO_TOWER_WIDTH)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/concrete.png")),
                perceptual_roughness: 0.9,
                metallic: 0.1,
                ..default()
            }),
            transform: Transform::from_translation(RADIO_TOWER_POSITION),
            ..default()
        },
        Name::new("RadioTower"),
    ));

    // Concrete bridge connecting platform to main area
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(BRIDGE_LENGTH, BRIDGE_HEIGHT, BRIDGE_WIDTH),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(BRIDGE_LENGTH, BRIDGE_HEIGHT, BRIDGE_WIDTH)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/concrete.png")),
                perceptual_roughness: 0.9,
                metallic: 0.1,
                ..default()
            }),
            transform: Transform::from_translation(BRIDGE_POSITION)
                .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_4)),
            ..default()
        },
        Name::new("Bridge"),
    ));
    // Starship
    commands.spawn((
        SceneBundle {
            scene: asset_server
                .load(GltfAssetLabel::Scene(0).from_asset("models/starhopper.glb")),
            transform: Transform::from_xyz(
                RADIO_TOWER_POSITION.x, 
                RADIO_TOWER_POSITION.y + RADIO_TOWER_HEIGHT/2.0, 
                RADIO_TOWER_POSITION.z
            )
                .with_scale(Vec3::splat(1.0)),
            ..default()
        },
        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
        RigidBody::Static,
    ));
}