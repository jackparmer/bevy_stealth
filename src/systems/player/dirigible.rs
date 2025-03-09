use bevy::prelude::*;
use avian3d::prelude::*;
use crate::components::Protagonist;

// Make the marker component public
#[derive(Component)]
pub struct DirigibleBalloon;

// Add this new component for floating animation
#[derive(Component)]
pub struct FloatingBalloon {
    timer: Timer,
}

impl FloatingBalloon {
    fn new() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Once),
        }
    }
}

pub fn toggle_dirigible(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Protagonist, &Children)>,
    balloon_query: Query<&DirigibleBalloon>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyY) {
        for (entity, mut protagonist, children) in query.iter_mut() {
            // Don't allow dirigible mode if driving
            if protagonist.is_driving {
                return;
            }
            
            protagonist.is_dirigible = !protagonist.is_dirigible;
            protagonist.is_swimming = false;
            protagonist.is_falling = false;
            protagonist.is_climbing = false;
            
            // Only remove existing balloon children with floating animation
            for &child in children.iter() {
                if balloon_query.get(child).is_ok() {
                    let mut entity_commands = commands.entity(child);
                    if !protagonist.is_dirigible {
                        // Add floating animation before despawning
                        entity_commands.insert(FloatingBalloon::new());
                    } else {
                        entity_commands.despawn_recursive();
                    }
                }
            }
            
            if protagonist.is_dirigible {
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        PbrBundle {
                            mesh: meshes.add(Mesh::from(Sphere::new(2.0))),
                            material: materials.add(StandardMaterial {
                                base_color: Color::srgb(1.0, 1.0, 1.0),
                                base_color_texture: Some(asset_server.load("textures/american-flag-background.png")),
                                metallic: 0.8,
                                perceptual_roughness: 0.1,
                                reflectance: 0.7,
                                ..default()
                            }),
                            transform: Transform::from_xyz(0.0, 8.0, 0.0),
                            ..default()
                        },
                        DirigibleBalloon,
                    ));
                });
                protagonist.is_birds_eye = false;
            }
        }
    }
}

// Add this new system to handle floating animation
pub fn animate_floating_balloon(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut FloatingBalloon)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut floating) in query.iter_mut() {
        floating.timer.tick(time.delta());
        
        // Move balloon upward and fade out
        transform.translation.y += 50.0 * time.delta_seconds();
        
        if floating.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn dirigible_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut protagonist_query: Query<(&mut Transform, &Protagonist)>,
    mut velocity_query: Query<&mut LinearVelocity, With<Protagonist>>,
    mut angular_velocity_query: Query<&mut AngularVelocity, With<Protagonist>>,
    time: Res<Time>,
) {
    const VERTICAL_SPEED: f32 = 600.0;
    const FORWARD_SPEED: f32 = 3000.0;
    const TURN_SPEED: f32 = 1.2;
    const ACCELERATION: f32 = 15.0;     // Reduced further for more stability
    const DECELERATION: f32 = 8.0;      // Increased for better stopping
    const SWAY_AMPLITUDE: f32 = 0.1;    // Reduced sway
    const SWAY_FREQUENCY: f32 = 0.2;    // Slower sway
    const DAMPENING: f32 = 0.95;        // New dampening factor

    if let Ok((mut transform, protagonist)) = protagonist_query.get_single_mut() {
        if !protagonist.is_dirigible {
            return;
        }

        // Handle turning with A/D keys with smooth interpolation
        if let Ok(mut angular_velocity) = angular_velocity_query.get_single_mut() {
            let target_turn = if keyboard_input.pressed(KeyCode::KeyD) {
                Vec3::new(0.0, -TURN_SPEED, 0.0)
            } else if keyboard_input.pressed(KeyCode::KeyA) {
                Vec3::new(0.0, TURN_SPEED, 0.0)
            } else {
                Vec3::ZERO
            };
            
            // Apply dampening to current angular velocity
            angular_velocity.0 *= DAMPENING;
            
            // Smooth angular velocity changes
            angular_velocity.0 = Vec3::lerp(
                angular_velocity.0,
                target_turn,
                ACCELERATION * time.delta_seconds()
            );
        }

        // Add gentle sway with dampening
        let sway = SWAY_AMPLITUDE * (time.elapsed_seconds() * SWAY_FREQUENCY).sin();
        transform.rotate_local_z(sway * time.delta_seconds() * DAMPENING);

        // Position clamping
        let pos = transform.translation;
        transform.translation = Vec3::new(
            pos.x.clamp(-10000.0, 10000.0),
            pos.y.clamp(-10000.0, 10000.0),
            pos.z.clamp(-10000.0, 10000.0)
        );

        let mut velocity = velocity_query.single_mut();
        let dt = time.delta_seconds();
        let mut movement = Vec3::ZERO;

        // Vertical movement
        if keyboard_input.pressed(KeyCode::ShiftLeft) {
            movement.y += VERTICAL_SPEED;
        }
        if keyboard_input.pressed(KeyCode::Space) {
            movement.y -= VERTICAL_SPEED;
        }

        // Forward/Backward movement (corrected directions)
        if keyboard_input.pressed(KeyCode::KeyW) {
            movement += transform.forward() * FORWARD_SPEED;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            movement -= transform.forward() * FORWARD_SPEED;
        }

        // Apply dampening to current velocity before adding new movement
        velocity.0 *= DAMPENING;

        velocity.0 = if movement != Vec3::ZERO {
            Vec3::lerp(
                velocity.0,
                movement,
                ACCELERATION * dt
            )
        } else {
            Vec3::lerp(
                velocity.0,
                Vec3::ZERO,
                DECELERATION * dt
            )
        };

        // Tighter velocity clamping
        velocity.0 = Vec3::new(
            velocity.0.x.clamp(-50.0, 50.0),
            velocity.0.y.clamp(-30.0, 30.0),
            velocity.0.z.clamp(-50.0, 50.0)
        );
    }
}
