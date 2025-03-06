use bevy::prelude::*;
use avian3d::prelude::*;
use crate::components::{Tank, Protagonist};
use crate::systems::player::driving::set_driving_state;

// Constants for the garage structure
pub const GARAGE_POSITION: Vec3 = Vec3::new(1800.4492, 2.6249862, -707.7545); // Near protagonist position
const ROOF_WIDTH: f32 = 360.0;
const ROOF_LENGTH: f32 = 360.0;
const ROOF_HEIGHT: f32 = 180.0;
const ROOF_THICKNESS: f32 = 12.0;
const PILLAR_WIDTH: f32 = 24.0;
const LIGHT_PANEL_THICKNESS: f32 = 24.0;
const LIGHT_PANEL_OFFSET: f32 = 24.0;

pub fn spawn_garage(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Load the rusty metal texture
    let metal_texture = asset_server.load("textures/rusty_metal_02_diff_4k.png");

    // Spawn main roof structure with rusty metal texture
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(ROOF_WIDTH, ROOF_THICKNESS, ROOF_LENGTH)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(metal_texture.clone()),
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

    // Update pillars to use the same rusty metal texture
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
                    base_color_texture: Some(metal_texture.clone()),
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

    // Spawn emissive blue light panel with increased thickness (directly below white panel)
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(ROOF_WIDTH - 16.0, LIGHT_PANEL_THICKNESS, ROOF_LENGTH - 16.0)),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.0, 0.2, 1.0),
            emissive: Color::rgb(0.0, 0.2, 1.0).into(),
            ..default()
        }),
        transform: Transform::from_translation(
            GARAGE_POSITION + Vec3::new(0.0, ROOF_HEIGHT - LIGHT_PANEL_OFFSET - LIGHT_PANEL_THICKNESS, 0.0)
        ),
        ..default()
    });

    // Spawn the tank model
    commands.spawn((
        SceneBundle {
            scene: asset_server.load("models/KB03-apc.glb#Scene0"),
            transform: Transform::from_translation(GARAGE_POSITION + Vec3::new(0.0, 0.0, 0.0))
                .with_scale(Vec3::splat(3.0)),
            ..default()
        },
        Tank,
        Sensor,
        RigidBody::Static,
        Collider::cuboid(30.0, 20.0, 40.0),
        Name::new("Tank"),
    ));

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

pub fn handle_tank_interaction(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionStarted>,
    tank_query: Query<Entity, With<Tank>>,
    mut protagonist_query: Query<(Entity, &mut Protagonist, &mut Handle<Scene>)>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
) {
    if time.elapsed_seconds() < 1.0 {
        collision_events.clear();
        return;
    }

    for CollisionStarted(e1, e2) in collision_events.read() {
        let tank = tank_query.iter().next();
        
        if let Some(tank_entity) = tank {
            if *e1 == tank_entity || *e2 == tank_entity {
                if let Ok((_, mut protagonist, mut scene)) = protagonist_query.get_single_mut() {
                    info!("Tank interaction triggered!");
                    commands.entity(tank_entity).despawn_recursive();
                    
                    protagonist.is_outside = true;
                    set_driving_state(&mut protagonist, &mut scene, &asset_server, true);
                }
            }
        }
    }
}
