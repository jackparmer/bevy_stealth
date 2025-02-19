use bevy::prelude::*;
use avian3d::prelude::*;

// Ladder dimensions
pub const LADDER_HEIGHT: f32 = 150.0;
pub const LADDER_WIDTH: f32 = 4.0;
pub const LADDER_THICKNESS: f32 = 1.4;

pub const LADDER_START: Vec3 = Vec3::new(214.0, 100.0, 200.0);

#[derive(Clone)]
pub struct LadderConfig {
    pub position: Vec3,
    pub rotation: Quat,
    pub height: f32,
    pub rung_count: usize,
}

impl Default for LadderConfig {
    fn default() -> Self {
        Self {
            position: LADDER_START,
            rotation: Quat::IDENTITY,
            height: LADDER_HEIGHT,
            rung_count: 200,
        }
    }
}

pub fn spawn_ladder(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
    config: LadderConfig,
) {
    // Add concrete wall behind ladder
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(LADDER_WIDTH, config.height, LADDER_WIDTH),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(LADDER_WIDTH, config.height, LADDER_WIDTH)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/ice_texture3.png")),
                perceptual_roughness: 0.3,
                metallic: 0.8,
                base_color: Color::srgb(0.3, 0.3, 0.3),
                ..default()
            }),
            transform: Transform::from_translation(
                config.position + Vec3::new(0.0, config.height/2.0, 0.0)
            ).with_rotation(config.rotation),
            ..default()
        },
        Name::new("LadderWall"),
    ))
    .with_children(|parent| {
        let rung_spacing = config.height / (config.rung_count as f32);
        
        // Add vertical support struts
        for x_offset in [-LADDER_WIDTH/2.0, LADDER_WIDTH/2.0] {
            parent.spawn((
                PbrBundle {
                    mesh: meshes.add(Cuboid::new(LADDER_THICKNESS, config.height, LADDER_THICKNESS)),
                    material: materials.add(StandardMaterial {
                        base_color_texture: Some(asset_server.load("textures/ice_texture3.png")),
                        perceptual_roughness: 0.3,
                        metallic: 0.8,
                        base_color: Color::srgb(0.3, 0.3, 0.3),
                        ..default()
                    }),
                    transform: Transform::from_translation(Vec3::new(
                        LADDER_THICKNESS + 1.0,
                        0.0,
                        x_offset
                    )),
                    ..default()
                },
                Name::new("LadderStrut"),
            ));
        }

        // Spawn ladder rungs as children
        for i in 0..config.rung_count {
            parent.spawn((
                PbrBundle {
                    mesh: meshes.add(Cylinder {
                        radius: 0.02,
                        half_height: LADDER_WIDTH/2.0,
                    }),
                    material: materials.add(StandardMaterial {
                        base_color: Color::srgb(0.7, 0.7, 0.7),
                        perceptual_roughness: 0.9,
                        metallic: 0.8,
                        emissive: Color::srgb(0.2, 0.2, 0.3).into(),
                        ..default()
                    }),
                    transform: Transform::from_translation(Vec3::new(
                        LADDER_THICKNESS + 1.0, 
                        (i as f32 * rung_spacing) - config.height/2.0, 
                        0.0))
                        .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                    ..default()
                },
                Name::new("LadderRung"),
            ));
        }

        // Spawn ladder sensor as a child
        parent.spawn((
            RigidBody::Static,
            Collider::cuboid(
                LADDER_THICKNESS/2.0, 
                config.height,
                LADDER_WIDTH + 2.0),
            Sensor,
            Name::new("LadderSensor"),
            PbrBundle {
                mesh: meshes.add(Cuboid::new(
                    LADDER_THICKNESS/2.0, 
                    config.height,
                    LADDER_WIDTH)),
                material: materials.add(StandardMaterial {
                    base_color: Color::srgba(0.1, 0.0, 0.0, 0.1),
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                }),
                transform: Transform::from_translation(Vec3::new(
                    LADDER_THICKNESS + 1.0, 
                    0.0, 
                    0.0)),
                ..default()
            },
        ));

        // Add top platform sensor
        parent.spawn((
            RigidBody::Static,
            Collider::cuboid(
                LADDER_WIDTH * 1.5,
                LADDER_THICKNESS,
                LADDER_WIDTH * 1.5
            ),
            Sensor,
            Name::new("LadderTopSensor"),
            PbrBundle {
                mesh: meshes.add(Cuboid::new(
                    LADDER_WIDTH * 1.5,
                    LADDER_THICKNESS,
                    LADDER_WIDTH * 1.5
                )),
                material: materials.add(StandardMaterial {
                    base_color: Color::srgba(0.0, 1.0, 0.0, 0.1),
                    emissive: Color::srgb(0.0, 0.5, 0.0).into(),
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                }),
                transform: Transform::from_translation(Vec3::new(
                    LADDER_THICKNESS + 1.0,
                    config.height/2.0 + LADDER_THICKNESS,
                    0.0
                )),
                ..default()
            },
        ));
    });
}
