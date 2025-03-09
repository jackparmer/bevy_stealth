use bevy::prelude::*;
use avian3d::prelude::*;
use crate::components::{Tank, Protagonist};
use crate::systems::player::driving::set_driving_state;
use crate::systems::core::screenplay::{MessageDisplay, display_message};

// Constants for the garage structure
pub const GARAGE_POSITION: Vec3 = Vec3::new(1800.4492, 2.6249862, -707.7545); // Near protagonist position
const ROOF_WIDTH: f32 = 360.0;
const ROOF_LENGTH: f32 = 360.0;
const ROOF_HEIGHT: f32 = 180.0;
const ROOF_THICKNESS: f32 = 12.0;
const PILLAR_WIDTH: f32 = 24.0;
const LIGHT_PANEL_THICKNESS: f32 = 24.0;
const LIGHT_PANEL_OFFSET: f32 = 24.0;

// Add new constant for trigger volume
const TRIGGER_VOLUME_SIZE: Vec3 = Vec3::new(600.0, 100.0, 600.0);

// Add new component for ring lights
#[derive(Component)]
pub struct GarageRingLight;

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
            emissive: Color::srgb(1.0, 1.0, 1.0).into(),
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
            base_color: Color::srgb(0.0, 0.2, 1.0),
            emissive: Color::srgb(0.0, 0.2, 1.0).into(),
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
                .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2))  // 90-degree rotation
                .with_scale(Vec3::splat(6.0)),
            ..default()
        },
        Tank,
        Sensor,
        RigidBody::Static,
        Collider::cuboid(30.0, 30.0, 40.0),  // Reduced collider size
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
                color: Color::srgb(0.9, 0.9, 1.0),
                intensity: 18000.0,
                range: 300.0,
                ..default()
            },
            transform: Transform::from_translation(GARAGE_POSITION + position),
            ..default()
        });
    }

    // Add spotlight above the tank
    commands.spawn(SpotLightBundle {
        spot_light: SpotLight {
            color: Color::srgb(1.0, 0.9, 0.8),
            intensity: 10000000000.0,
            range: 400.0,
            outer_angle: 0.6,
            inner_angle: 0.3,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_translation(GARAGE_POSITION + Vec3::new(0.0, 10.0, 0.0))
            .looking_at(GARAGE_POSITION, Vec3::Y),
        ..default()
    });

    // Add ring of red point lights around the tank
    const NUM_RING_LIGHTS: i32 = 8;
    const RING_RADIUS: f32 = 60.0;
    const RING_HEIGHT: f32 = 20.0;  // Lower to be closer to tank

    for i in 0..NUM_RING_LIGHTS {
        let angle = (i as f32) * 2.0 * std::f32::consts::PI / (NUM_RING_LIGHTS as f32);
        let x = RING_RADIUS * angle.cos();
        let z = RING_RADIUS * angle.sin();
        
        commands.spawn((
            PointLightBundle {
                point_light: PointLight {
                    color: Color::srgb(0.99, 0.2, 0.2),
                    intensity: 10000000.0,
                    range: 150.0,
                    ..default()
                },
                transform: Transform::from_translation(GARAGE_POSITION + Vec3::new(x, RING_HEIGHT, z)),
                ..default()
            },
            GarageRingLight,  // Add this component
        ));
    }

    // Add trigger volume around the garage
    commands.spawn((
        TransformBundle::from(Transform::from_translation(GARAGE_POSITION)),
        Sensor,
        Collider::cuboid(TRIGGER_VOLUME_SIZE.x/2.0, TRIGGER_VOLUME_SIZE.y/2.0, TRIGGER_VOLUME_SIZE.z/2.0),
        Name::new("GarageTrigger"),
    ));
}

pub fn handle_tank_interaction(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionStarted>,
    tank_query: Query<Entity, With<Tank>>,
    mut protagonist_query: Query<(Entity, &mut Protagonist, &mut Handle<Scene>)>,
    children_query: Query<&Children>,
    ring_lights_query: Query<Entity, With<GarageRingLight>>,
    asset_server: Res<AssetServer>,
    mut message_display: ResMut<MessageDisplay>,
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
                if let Ok((protagonist_entity, mut protagonist, mut scene)) = protagonist_query.get_single_mut() {
                    info!("Tank interaction triggered!");
                    
                    // Set driving state first, before despawning the tank
                    protagonist.is_outside = true;
                    set_driving_state(
                        &mut protagonist,
                        &mut scene,
                        &asset_server,
                        true,
                        &mut commands,
                        protagonist_entity,
                        &children_query
                    );
                    
                    // Despawn the tank after setting driving state
                    commands.entity(tank_entity).despawn_recursive();
                    
                    // Despawn ring lights
                    for ring_light in ring_lights_query.iter() {
                        commands.entity(ring_light).despawn_recursive();
                    }
                    
                    // Display message when entering the tank
                    display_message("DRIVE TO ICE CAVE", Color::srgb(0.01, 0.55, 0.99), &mut message_display);
                }
            }
        }
    }
}

// Add new system for handling garage approach
pub fn handle_garage_approach(
    mut collision_events: EventReader<CollisionStarted>,
    trigger_query: Query<Entity, With<Name>>,
    protagonist_query: Query<Entity, With<Protagonist>>,
    mut message_display: ResMut<MessageDisplay>,
    time: Res<Time>,
) {
    if time.elapsed_seconds() < 1.0 {
        collision_events.clear();
        return;
    }

    for CollisionStarted(e1, e2) in collision_events.read() {
        let trigger = trigger_query.iter().find(|&e| e.index() == e1.index() || e.index() == e2.index());
        let protagonist = protagonist_query.get_single().ok();
        
        if let (Some(trigger_entity), Some(protagonist_entity)) = (trigger, protagonist) {
            if (*e1 == protagonist_entity && *e2 == trigger_entity) || 
               (*e2 == protagonist_entity && *e1 == trigger_entity) {
                display_message("SHELTER IN THE VEHICLE", Color::srgb(0.99, 0.2, 0.2), &mut message_display);
            }
        }
    }
}
