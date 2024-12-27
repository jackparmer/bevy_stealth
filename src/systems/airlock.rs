use bevy::prelude::*;
use bevy::math::primitives::{Cylinder, Sphere};
use avian3d::prelude::*;
use crate::components::Protagonist;

pub const AIRLOCK_RADIUS: f32 = 16.0;
pub const AIRLOCK_LENGTH: f32 = 80.0;
pub const AIRLOCK_POSITION: Vec3 = Vec3::new(865.24176, 2.6249726, -424.97363);

#[derive(Component)]
pub struct AirlockLight {
    pub timer: Timer,
}

#[derive(Component)]
pub struct AirlockTrigger {
    pub is_entry: bool,
    pub cooldown: Timer,
}

pub fn spawn_airlock(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Main tunnel cylinder
    commands.spawn((
        RigidBody::Static,
        Collider::cylinder(AIRLOCK_RADIUS, AIRLOCK_LENGTH),
        PbrBundle {
            mesh: meshes.add(Cylinder {
                radius: AIRLOCK_RADIUS,
                half_height: AIRLOCK_LENGTH/2.0,
            }),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/container_metal.png")),
                perceptual_roughness: 0.8,
                metallic: 0.7,
                ..default()
            }),
            transform: Transform::from_translation(AIRLOCK_POSITION)
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
            ..default()
        },
        Name::new("AirlockTunnel"),
    ));

    // Add blinking lights at both ends with intense red
    for offset in [-AIRLOCK_LENGTH/2.0, AIRLOCK_LENGTH/2.0] {
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Sphere::default()),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.8, 0.0, 0.0),
                    emissive: LinearRgba::new(5.0, 0.0, 0.0, 1.0),  // Much higher intensity
                    ..default()
                }),
                transform: Transform::from_translation(AIRLOCK_POSITION + Vec3::new(offset, AIRLOCK_RADIUS - 1.0, 0.0))
                    .with_scale(Vec3::splat(AIRLOCK_RADIUS/10.0)),  // Larger light spheres
                ..default()
            },
            PointLight {
                color: Color::rgb(1.0, 0.0, 0.0),
                intensity: 0.0,  // Will be toggled in the blink system
                range: AIRLOCK_RADIUS * 200.0,
                ..default()
            },
            AirlockLight {
                timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            },
            Name::new("AirlockLight"),
        ));
    }

    // Add metallic doors at both ends
    for offset in [-AIRLOCK_LENGTH/2.0, AIRLOCK_LENGTH/2.0] {
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(
                    AIRLOCK_RADIUS * 0.2,  // width (halved)
                    AIRLOCK_RADIUS * 0.6,  // height (halved)
                    0.5,                   // depth (thickness)
                )),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.3, 0.0, 0.0), // Dark red
                    metallic: 1.0,
                    perceptual_roughness: 0.2, // Fairly smooth/shiny
                    ..default()
                }),
                transform: Transform::from_translation(AIRLOCK_POSITION + Vec3::new(offset, 0.0, 0.0))
                    .with_rotation(
                        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)
                    ),
                ..default()
            },
            Name::new("AirlockDoor"),
        ));
    }

    // Add trigger volumes at both ends - now using disc-shaped sensors
    for (offset, is_entry) in [(-AIRLOCK_LENGTH/2.0, true), (AIRLOCK_LENGTH/2.0, false)] {
        let transform = Transform::from_translation(AIRLOCK_POSITION + Vec3::new(offset, 0.0, 0.0))
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));

        commands.spawn((
            Sensor,
            Collider::cylinder(1.2, AIRLOCK_RADIUS),
            PbrBundle {
                mesh: meshes.add(Cylinder {
                    radius: AIRLOCK_RADIUS,
                    half_height: 1.1,
                }),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgba(1.0, 0.0, 0.0, 0.01),
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                }),
                transform,
                ..default()
            },
            AirlockTrigger { 
                is_entry,
                cooldown: Timer::from_seconds(1.0, TimerMode::Once),
            },
            Name::new("AirlockTrigger"),
        ));
    }

    /*
    for offset in [-AIRLOCK_LENGTH/2.0, AIRLOCK_LENGTH/2.0] {
        commands.spawn((
            RigidBody::Static,
            Collider::cylinder(0.1, AIRLOCK_RADIUS),
            PbrBundle {
                mesh: meshes.add(Cylinder {
                    radius: AIRLOCK_RADIUS,
                    half_height: 0.1,
                }),
                material: materials.add(StandardMaterial {
                    base_color_texture: Some(asset_server.load("textures/container_metal.png")),
                    perceptual_roughness: 0.8,
                    metallic: 0.7,
                    ..default()
                }),
                transform: Transform::from_translation(
                    AIRLOCK_POSITION + Vec3::new(offset, 0.0, 0.0)
                ),
                ..default()
            },
            Name::new("AirlockCap"),
        ));
    }
    */
}

pub fn blink_airlock_light(
    time: Res<Time>,
    mut lights: Query<(&mut AirlockLight, &mut Handle<StandardMaterial>, &mut PointLight)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mut light, material_handle, mut point_light) in &mut lights {
        light.timer.tick(time.delta());
        if light.timer.just_finished() {
            if let Some(material) = materials.get_mut(&*material_handle) {
                let is_on = material.emissive.red > 0.0;
                material.emissive = if is_on {
                    LinearRgba::new(0.0, 0.0, 0.0, 1.0)
                } else {
                    LinearRgba::new(5.0, 0.0, 0.0, 1.0)  // Much higher intensity when lit
                };
                point_light.intensity = if is_on { 0.0 } else { 1000.0 };
            }
        }
    }
}

pub fn handle_airlock_teleport(
    time: Res<Time>,
    collisions: Res<Collisions>,
    mut triggers: Query<(Entity, &mut AirlockTrigger)>,
    mut player: Query<(Entity, &mut Transform), With<Protagonist>>,
    mut velocity_query: Query<&mut LinearVelocity, With<Protagonist>>,
) {
    if let Ok((player_entity, mut player_transform)) = player.get_single_mut() {
        for (trigger_entity, mut trigger) in &mut triggers {
            if collisions.contains(player_entity, trigger_entity) {
                // If the cooldown is finished, teleport immediately
                if trigger.cooldown.finished() {
                    println!("Teleporting player! is_entry: {}", trigger.is_entry);
                    teleport_player(&mut player_transform, &mut velocity_query, trigger.is_entry);
                    trigger.cooldown.reset();
                }
                // Tick the timer while in contact
                trigger.cooldown.tick(time.delta());
            }
        }
    }
}

fn teleport_player(
    player_transform: &mut Transform, 
    velocity_query: &mut Query<&mut LinearVelocity, With<Protagonist>>, 
    is_entry: bool
) {
    // Calculate the target x position relative to the airlock
    let target_x = if is_entry {
        AIRLOCK_POSITION.x + AIRLOCK_LENGTH/2.0 + 6.0
    } else {
        AIRLOCK_POSITION.x - AIRLOCK_LENGTH/2.0 - 6.0
    };

    // Keep current y and z, only update x
    player_transform.translation.x = target_x;

    // Preserve vertical and lateral movement, only zero out forward velocity
    if let Ok(mut linear_velocity) = velocity_query.get_single_mut() {
        linear_velocity.0.x = 0.0;  // Zero out only the x component
        linear_velocity.0.y *= 0.25;
        linear_velocity.0.z *= 0.25;
    }
}
