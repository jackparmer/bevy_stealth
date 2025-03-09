use bevy::prelude::*;
use bevy::math::primitives::{Cylinder, Sphere};
use avian3d::prelude::*;
use crate::components::Protagonist;
use crate::systems::core::screenplay::{MessageDisplay, display_message};

pub const AIRLOCK_RADIUS: f32 = 16.0;
pub const AIRLOCK_LENGTH: f32 = 800.0;
pub const AIRLOCK_POSITION: Vec3 = Vec3::new(865.24176, 2.6249726, -424.97363);
pub const AIRLOCK_TELEPORT_OFFSET: f32 = 20.0;

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
                base_color_texture: Some(asset_server.load("textures/rusty_metal_grid_diff_4k.png")),
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
                    base_color: Color::srgb(0.8, 0.0, 0.0),
                    emissive: LinearRgba::new(5.0, 0.0, 0.0, 1.0),  // Much higher intensity
                    ..default()
                }),
                transform: Transform::from_translation(AIRLOCK_POSITION + Vec3::new(offset, AIRLOCK_RADIUS - 1.0, 0.0))
                    .with_scale(Vec3::splat(AIRLOCK_RADIUS/10.0)),  // Larger light spheres
                ..default()
            },
            PointLight {
                color: Color::srgb(1.0, 0.0, 0.0),
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
        let door_height = AIRLOCK_RADIUS * 0.6;  // half-height of the door
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(
                    AIRLOCK_RADIUS * 0.05,     // thickness (halved)
                    door_height,               // height (halved)
                    AIRLOCK_RADIUS * 0.6,      // width (halved)
                )),
                material: materials.add(StandardMaterial {
                    base_color_texture: Some(asset_server.load("textures/airlock_door.png")),
                    metallic: 1.0,
                    perceptual_roughness: 0.2,
                    emissive: Color::srgb(0.1, 0.0, 0.0).into(), 
                    ..default()
                }),
                transform: Transform::from_translation(AIRLOCK_POSITION + Vec3::new(offset, door_height/2.0, 0.0))  // Changed to door_height/2.0
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
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
                    base_color: Color::srgba(1.0, 0.0, 0.0, 0.01),
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

    // Add spotlights at both ends - Adjusting position and intensity
    let offset = -AIRLOCK_LENGTH/2.0;  // Only interior spotlight
    let x = offset * 1.2;
    let z = 0.0;
    let spotlight_position = AIRLOCK_POSITION + Vec3::new(x, AIRLOCK_RADIUS * 0.4, z);

    // Spawn the spotlight
    commands.spawn(SpotLightBundle {
        spot_light: SpotLight {
            color: Color::srgb(1.0, 0.5, 0.1),
            intensity: 500000000.0,
            range: AIRLOCK_RADIUS * 50.0,
            outer_angle: 1.6,
            inner_angle: 0.8,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_translation(spotlight_position)
            .looking_to(Vec3::Y, -Vec3::new(x, 0.0, z).normalize()),
        ..default()
    });
}

pub fn blink_airlock_light(
    time: Res<Time>,
    mut lights: Query<(&mut AirlockLight, Option<&mut Handle<StandardMaterial>>, Option<&mut PointLight>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mut light, material_handle, mut point_light) in &mut lights {
        light.timer.tick(time.delta());
        if light.timer.just_finished() {
            if let Some(material_handle) = material_handle {
                if let Some(material) = materials.get_mut(&*material_handle) {
                    let is_on = material.emissive.red > 0.0;
                    material.emissive = if is_on {
                        LinearRgba::new(0.0, 0.0, 0.0, 1.0)
                    } else {
                        LinearRgba::new(5.0, 0.0, 0.0, 1.0)
                    };
                    
                    if let Some(point_light) = &mut point_light {
                        point_light.intensity = if is_on { 0.0 } else { 1000.0 };
                    }
                }
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
    mut message_display: ResMut<MessageDisplay>,
) {
    if let Ok((player_entity, mut player_transform)) = player.get_single_mut() {
        for (trigger_entity, mut trigger) in &mut triggers {
            if collisions.contains(player_entity, trigger_entity) {
                // If the cooldown is finished, teleport immediately
                if trigger.cooldown.finished() {
                    // Display appropriate message based on entry/exit
                    if trigger.is_entry {
                        display_message("ENTERING AIRLOCK", Color::WHITE, &mut message_display);
                    } else {
                        display_message("FIND THE GARAGE", Color::WHITE, &mut message_display);
                    }
                    
                    // Small delay to allow message to be visible
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    
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
        AIRLOCK_POSITION.x + AIRLOCK_LENGTH/2.0 + AIRLOCK_TELEPORT_OFFSET
    } else {
        AIRLOCK_POSITION.x - AIRLOCK_LENGTH/2.0 - AIRLOCK_TELEPORT_OFFSET
    };

    // Keep current z, update x and add vertical offset
    player_transform.translation.x = target_x;
    player_transform.translation.y += 5.0;  // Add 5 units of vertical offset

    // Preserve vertical and lateral movement, only zero out forward velocity
    if let Ok(mut linear_velocity) = velocity_query.get_single_mut() {
        linear_velocity.0.x = 0.0;  // Zero out only the x component
        linear_velocity.0.y *= 0.25;
        linear_velocity.0.z *= 0.25;
    }
}
