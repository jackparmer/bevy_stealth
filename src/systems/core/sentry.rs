use bevy::prelude::*;
use crate::components::Sentry;
use crate::components::Protagonist;
use bevy::ecs::system::ParamSet;
use rand;
use avian3d::prelude::*;

#[derive(Component)]
pub struct ExplosionParticle {
    velocity: Vec3,
    lifetime: Timer,
}

#[derive(Component)]
pub struct ExplosionLight {
    intensity: f32,
    timer: Timer,
}

#[derive(Component)]
pub struct SentryExplosion {
    timer: Timer,
    initial_scale: Vec3,
}

#[derive(Component)]
pub struct SentrySpawnTimer(Timer);

#[derive(Component)]
pub struct ExternalImpulse {
    impulse: Vec3,
}

#[derive(Component)]
pub struct PendingImpulse(Vec3);

// Helper function for spawning a sentry
fn spawn_sentry_at(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
) {
    let fixed_position = Vec3::new(position.x, 3.0, position.z);
    println!("🤖 Spawning sentry...");

    commands.spawn((
        SceneBundle {       
            scene: asset_server
                .load(GltfAssetLabel::Scene(0)
                .from_asset("models/tmpn3hy22ev.glb")),
            transform: Transform::from_translation(fixed_position)
                .with_scale(Vec3::splat(2.0)),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::sphere(1.0),  // Simple sphere collider with radius 1.0
        LockedAxes::new().lock_rotation_x().lock_rotation_z().lock_translation_y(),
        Friction::new(0.5),
        ExternalImpulse { impulse: Vec3::ZERO },
        Sentry {
            view_distance: 500.0,
            view_angle: std::f32::consts::PI / 2.0,
            follow_speed: 10.0,
            velocity: Vec3::ZERO,
        },
        Name::new("Sentry"),
    ))
    .with_children(|parent| {
        // Add point light as child
        parent.spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 10000000.0,
                range: 20.0,
                color: Color::srgb(1.0, 0.8, 0.8),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 6.0, 0.0),
            ..default()
        });
    });
}

// System function for initial spawn
pub fn spawn_sentry(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_sentry_at(&mut commands, &asset_server, &mut meshes, &mut materials, Vec3::new(214.0, 3.0, -35.0));
}

pub fn sentry_follow_system(
    mut commands: Commands,
    mut query_set: ParamSet<(
        Query<&Transform, With<Protagonist>>,
        Query<(Entity, &mut Transform, &mut Sentry), With<Sentry>>,
    )>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Get protagonist data first
    let protagonist_pos = {
        let protagonist_query = query_set.p0();
        if let Ok(transform) = protagonist_query.get_single() {
            println!("Found protagonist at: {:?}", transform.translation);
            transform.translation
        } else {
            println!("No protagonist found!");
            return;
        }
    };

    // Get sentry query
    let mut sentry_query = query_set.p1();
    for (entity, mut transform, mut sentry) in sentry_query.iter_mut() {
        // Clamp Y position between 3.0 and 4.0
        transform.translation.y = transform.translation.y.clamp(3.0, 4.0);
        
        println!("Processing sentry at: {:?}", transform.translation);
        let direction = protagonist_pos - transform.translation;
        let distance = direction.length();

        println!("Distance to protagonist: {}", distance);
        println!("View distance: {}", sentry.view_distance);

        if distance < sentry.view_distance {
            println!("Sentry in range, moving towards protagonist");
            // Add jerky, sudden movements
            let jitter = Vec3::new(
                (time.elapsed_seconds() * 10.0).sin() * 0.2,
                (time.elapsed_seconds() * 15.0).cos() * 0.1,
                (time.elapsed_seconds() * 12.0).sin() * 0.2,
            );

            // Occasionally freeze movement
            let speed_multiplier = if (time.elapsed_seconds() * 3.0).sin() > 0.95 {
                0.0 // Momentarily freeze
            } else if (time.elapsed_seconds() * 2.0).sin() > 0.8 {
                3.0 // Occasional burst of speed
            } else {
                1.0 // Normal speed
            };

            let direction = (direction.normalize() + jitter).normalize();
            let movement = direction * sentry.follow_speed * speed_multiplier * time.delta_seconds();
            
            // Store velocity before moving
            sentry.velocity = movement / time.delta_seconds();
            transform.translation += movement;
            
            println!("Movement: {:?}", movement);
            
            // Check if within explosion range and trigger explosion
            if distance < 1.0 {
                println!("Triggering sentry explosion!");
                // Store the current velocity as a pending impulse
                commands.entity(entity).insert(PendingImpulse(sentry.velocity * 50.0));
                
                // Trigger explosion
                commands.entity(entity).despawn_recursive();
                
                // Spawn explosion effect
                commands.spawn((
                    TransformBundle {
                        local: transform.clone(),
                        ..default()
                    },
                    SentryExplosion {
                        timer: Timer::from_seconds(1.0, TimerMode::Once),
                        initial_scale: transform.scale,
                    },
                ));

                // Spawn particles
                for i in 0..1000 {
                    let random_dir = Vec3::new(
                        rand::random::<f32>() * 2.0 - 1.0,
                        rand::random::<f32>() * 2.0 - 1.0,
                        rand::random::<f32>() * 2.0 - 1.0,
                    ).normalize();
                    
                    let speed = rand::random::<f32>() * 10.0 + 2.0;
                    let (base_color, emissive) = if i < 10 { // 10% of 20 particles
                        (Color::srgb(1.0, 0.0, 0.0), Color::srgb(2.0, 0.0, 0.0))
                    } else {
                        (Color::BLACK, Color::BLACK)
                    };
                    
                    commands.spawn((
                        PbrBundle {
                            mesh: meshes.add(Mesh::from(Cuboid::new(0.2, 0.2, 0.2))),
                            material: materials.add(StandardMaterial {
                                base_color,
                                emissive: emissive.into(),
                                ..default()
                            }),
                            transform: Transform::from_translation(transform.translation),
                            ..default()
                        },
                        ExplosionParticle {
                            velocity: random_dir * speed,
                            lifetime: Timer::from_seconds(1.0, TimerMode::Once),
                        },
                    ));
                }

                // Spawn explosion light
                commands.spawn((
                    PointLightBundle {
                        point_light: PointLight {
                            intensity: 100000.0,
                            color: Color::rgb(1.0, 0.5, 0.0),
                            ..default()
                        },
                        transform: Transform::from_translation(transform.translation),
                        ..default()
                    },
                    ExplosionLight {
                        intensity: 100000.0,
                        timer: Timer::from_seconds(0.5, TimerMode::Once),
                    },
                ));
            }

            // Jerky rotation
            let look_target = protagonist_pos + jitter;
            transform.look_at(look_target, Vec3::Y);
        }
    }
}

pub fn sentry_explosion_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query_set: ParamSet<(
        Query<(Entity, &Transform, &mut Sentry)>,
        Query<(Entity, &mut Transform, &mut SentryExplosion)>
    )>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Handle initiating new explosions
    for (entity, transform, _sentry) in query_set.p0().iter_mut() {
        if keyboard.just_pressed(KeyCode::KeyG) {
            commands.entity(entity).despawn();
            
            // Spawn explosion effect
            commands.spawn((
                TransformBundle {
                    local: transform.clone(),
                    ..default()
                },
                SentryExplosion {
                    timer: Timer::from_seconds(1.0, TimerMode::Once),
                    initial_scale: transform.scale,
                },
            ));

            // Spawn particles
            for i in 0..10000 {
                let random_dir = Vec3::new(
                    rand::random::<f32>() * 2.0 - 1.0,
                    rand::random::<f32>() * 2.0 - 1.0,
                    rand::random::<f32>() * 2.0 - 1.0,
                ).normalize();
                
                let speed = rand::random::<f32>() * 10.0 + 2.0;
                let (base_color, emissive) = if i < 10 { // 10% of 20 particles
                    (Color::srgb(1.0, 0.0, 0.0), Color::srgb(2.0, 0.0, 0.0))
                } else {
                    (Color::BLACK, Color::BLACK)
                };
                
                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(Mesh::from(Cuboid::new(0.2, 0.2, 0.2))),
                        material: materials.add(StandardMaterial {
                            base_color,
                            emissive: emissive.into(),
                            ..default()
                        }),
                        transform: Transform::from_translation(transform.translation),
                        ..default()
                    },
                    ExplosionParticle {
                        velocity: random_dir * speed,
                        lifetime: Timer::from_seconds(1.0, TimerMode::Once),
                    },
                ));
            }

            // Spawn explosion light
            commands.spawn((
                PointLightBundle {
                    point_light: PointLight {
                        intensity: 100000.0,
                        color: Color::rgb(1.0, 0.5, 0.0),
                        ..default()
                    },
                    transform: Transform::from_translation(transform.translation),
                    ..default()
                },
                ExplosionLight {
                    intensity: 100000.0,
                    timer: Timer::from_seconds(0.5, TimerMode::Once),
                },
            ));
        }
    }

    // Handle ongoing explosions
    for (entity, mut transform, mut explosion) in query_set.p1().iter_mut() {
        explosion.timer.tick(time.delta());
        
        if explosion.timer.finished() {
            commands.entity(entity).despawn();
        } else {
            transform.scale = explosion.initial_scale * (1.0 - explosion.timer.fraction());
        }
    }
}

pub fn periodic_sentry_spawn(
    mut commands: Commands,
    time: Res<Time>,
    mut timer_query: Query<&mut SentrySpawnTimer>,
    protagonist_query: Query<&Transform, With<Protagonist>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Initialize timer if it doesn't exist
    if timer_query.is_empty() {
        commands.spawn(SentrySpawnTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
        return;
    }

    let mut timer = timer_query.single_mut();
    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        if let Ok(protagonist_transform) = protagonist_query.get_single() {
            // Generate random position within 100 units
            let random_offset = Vec3::new(
                rand::random::<f32>() * 200.0 - 100.0,
                0.0, // Keep on same Y level as original spawn
                rand::random::<f32>() * 200.0 - 100.0,
            );
            let spawn_pos = protagonist_transform.translation + random_offset;
            
            spawn_sentry_at(&mut commands, &asset_server, &mut meshes, &mut materials, spawn_pos);
        }
    }
}

pub fn sentry_respawn_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if keyboard.just_pressed(KeyCode::KeyH) {
        spawn_sentry_at(&mut commands, &asset_server, &mut meshes, &mut materials, Vec3::new(214.0, 3.0, -35.0));
    }
}

pub fn update_explosion_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut ExplosionParticle)>,
) {
    for (entity, mut transform, mut particle) in query.iter_mut() {
        particle.lifetime.tick(time.delta());
        
        if particle.lifetime.finished() {
            commands.entity(entity).despawn();
        } else {
            transform.translation += particle.velocity * time.delta_seconds();
            // Add some downward acceleration for gravity
            particle.velocity.y -= 9.8 * time.delta_seconds();
            
            // Shrink the particle over time
            let scale = 1.0 - particle.lifetime.fraction();
            transform.scale = Vec3::splat(scale);
        }
    }
}

pub fn update_explosion_light(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PointLight, &mut ExplosionLight)>,
) {
    for (entity, mut light, mut explosion_light) in query.iter_mut() {
        explosion_light.timer.tick(time.delta());
        
        if explosion_light.timer.finished() {
            commands.entity(entity).despawn();
        } else {
            // Fade out the light
            light.intensity = explosion_light.intensity * (1.0 - explosion_light.timer.fraction());
        }
    }
}

pub fn sentry_light_system(
    time: Res<Time>,
    mut query: Query<&mut PointLight, With<Sentry>>,
) {
    for mut light in query.iter_mut() {
        // Random flickering effect
        let flicker = (time.elapsed_seconds() * 30.0).sin() > 0.97;
        
        // Occasionally turn blood red
        if (time.elapsed_seconds() * 2.0).sin() > 0.95 || flicker {
            light.color = Color::srgb(1.0, 0.0, 0.0); // Blood red
            light.intensity = 1000000.0;
        } else {
            light.color = Color::srgb(1.0, 0.8, 0.8); // Normal color
            light.intensity = 100000.0;
        }
    }
}

pub fn apply_pending_impulse(
    mut commands: Commands,
    mut query: Query<(Entity, &PendingImpulse, &mut ExternalImpulse)>,
) {
    for (entity, pending, mut impulse) in query.iter_mut() {
        impulse.impulse = pending.0;
        commands.entity(entity).remove::<PendingImpulse>();
    }
}