use bevy::prelude::*;
use avian3d::prelude::*;

// Constants for the garage structure
pub const GARAGE_POSITION: Vec3 = Vec3::new(1800.4492, 2.6249862, -707.7545); // Near protagonist position
const ROOF_WIDTH: f32 = 360.0;
const ROOF_LENGTH: f32 = 360.0;
const ROOF_HEIGHT: f32 = 120.0;
const ROOF_THICKNESS: f32 = 12.0;
const PILLAR_WIDTH: f32 = 24.0;
const LIGHT_PANEL_THICKNESS: f32 = 6.0;
const LIGHT_PANEL_OFFSET: f32 = 12.0;

pub fn spawn_garage(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Spawn main roof structure
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(ROOF_WIDTH, ROOF_THICKNESS, ROOF_LENGTH)),
            material: materials.add(StandardMaterial {
                base_color: Color::BLACK,
                perceptual_roughness: 0.9,
                metallic: 0.1,
                ..default()
            }),
            transform: Transform::from_translation(GARAGE_POSITION + Vec3::new(0.0, ROOF_HEIGHT, 0.0)),
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(ROOF_WIDTH/2.0, ROOF_THICKNESS/2.0, ROOF_LENGTH/2.0),
    ));

    // Spawn pillars at corners
    let pillar_positions = [
        Vec3::new(-ROOF_WIDTH/2.0 + PILLAR_WIDTH/2.0, ROOF_HEIGHT/2.0, -ROOF_LENGTH/2.0 + PILLAR_WIDTH/2.0),
        Vec3::new(ROOF_WIDTH/2.0 - PILLAR_WIDTH/2.0, ROOF_HEIGHT/2.0, -ROOF_LENGTH/2.0 + PILLAR_WIDTH/2.0),
        Vec3::new(-ROOF_WIDTH/2.0 + PILLAR_WIDTH/2.0, ROOF_HEIGHT/2.0, ROOF_LENGTH/2.0 - PILLAR_WIDTH/2.0),
        Vec3::new(ROOF_WIDTH/2.0 - PILLAR_WIDTH/2.0, ROOF_HEIGHT/2.0, ROOF_LENGTH/2.0 - PILLAR_WIDTH/2.0),
    ];

    for position in pillar_positions {
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(PILLAR_WIDTH, ROOF_HEIGHT, PILLAR_WIDTH)),
                material: materials.add(StandardMaterial {
                    base_color: Color::BLACK,
                    perceptual_roughness: 0.9,
                    metallic: 0.1,
                    ..default()
                }),
                transform: Transform::from_translation(GARAGE_POSITION + position),
                ..default()
            },
            RigidBody::Static,
            Collider::cuboid(PILLAR_WIDTH/2.0, ROOF_HEIGHT/2.0, PILLAR_WIDTH/2.0),
        ));
    }

    // Spawn emissive white light panel with increased thickness
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(ROOF_WIDTH - 8.0, LIGHT_PANEL_THICKNESS, ROOF_LENGTH - 8.0)),
        material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            emissive: Color::rgb(1.0, 1.0, 1.0).into(),
            ..default()
        }),
        transform: Transform::from_translation(
            GARAGE_POSITION + Vec3::new(0.0, ROOF_HEIGHT - LIGHT_PANEL_OFFSET, 0.0)
        ),
        ..default()
    });

    // Spawn emissive blue light panel with increased thickness
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(ROOF_WIDTH - 16.0, LIGHT_PANEL_THICKNESS, ROOF_LENGTH - 16.0)),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.0, 0.2, 1.0),
            emissive: Color::rgb(0.0, 0.2, 1.0).into(),
            ..default()
        }),
        transform: Transform::from_translation(
            GARAGE_POSITION + Vec3::new(0.0, ROOF_HEIGHT - LIGHT_PANEL_OFFSET * 2.0, 0.0)
        ),
        ..default()
    });

    // Spawn the tank model
    commands.spawn(SceneBundle {
        scene: asset_server.load("models/KB03-apc.glb#Scene0"),
        transform: Transform::from_translation(GARAGE_POSITION + Vec3::new(0.0, 0.0, 0.0))
            .with_scale(Vec3::splat(3.0)),
        ..default()
    });

    // Add point lights for enhanced lighting effect
    let light_positions = [
        Vec3::new(-ROOF_WIDTH/4.0, ROOF_HEIGHT - 2.0, -ROOF_LENGTH/4.0),
        Vec3::new(ROOF_WIDTH/4.0, ROOF_HEIGHT - 2.0, -ROOF_LENGTH/4.0),
        Vec3::new(-ROOF_WIDTH/4.0, ROOF_HEIGHT - 2.0, ROOF_LENGTH/4.0),
        Vec3::new(ROOF_WIDTH/4.0, ROOF_HEIGHT - 2.0, ROOF_LENGTH/4.0),
    ];

    for position in light_positions {
        commands.spawn(PointLightBundle {
            point_light: PointLight {
                color: Color::rgb(0.9, 0.9, 1.0),
                intensity: 18000.0,
                range: 300.0,
                ..default()
            },
            transform: Transform::from_translation(GARAGE_POSITION + position),
            ..default()
        });
    }
}
