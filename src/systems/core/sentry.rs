use bevy::prelude::*;
use crate::components::Sentry;
use crate::components::Protagonist;
use bevy::ecs::system::ParamSet;
use rand;
use avian3d::prelude::*;
use crate::systems::core::setup::PROTAGONIST_START;
use std::f32::consts::PI;
use std::time::Duration;
use crate::systems::core::minimap::{MinimapMarker, SentryMinimapMarker, MinimapResources, MINIMAP_MARKER_HEIGHT};

const SENTRY_SPAWN_INTERVAL: f32 = 2.0; // Time in seconds between sentry spawns

#[derive(Component)]
pub struct ExplosionParticle {
    velocity: Vec3,
    lifetime: Timer,
    is_smoke: bool,  // New field to differentiate smoke particles
    initial_scale: f32,
    origin: Vec3, // Add origin point to component
    start_time: f32,  // Change spawn_time to start_time for clarity
}

#[derive(Component)]
pub struct ExplosionLight {
    intensity: f32,
    timer: Timer,
    start_time: f32,  // Add start_time field
}

#[derive(Component)]
pub struct SentryExplosion {
    timer: Timer,
    initial_scale: Vec3,
    start_time: f32,  // Add start time to track individual explosion timing
}

#[derive(Component)]
pub struct SentrySpawnTimer(Timer);

// Add new resource to track active explosions
#[derive(Resource)]
pub struct ExplosionCounter {
    count: usize,
    max_allowed: usize,
}

impl Default for ExplosionCounter {
    fn default() -> Self {
        Self {
            count: 0,
            max_allowed: 10, // Increased to allow more concurrent smoke columns
        }
    }
}

// Add new resource for shared explosion materials
#[derive(Resource)]
pub struct ExplosionMaterials {
    particle_mesh: Handle<Mesh>,
    red_material: Handle<StandardMaterial>,
    sentry_red_material: Handle<StandardMaterial>,
    glow_cone_mesh: Handle<Mesh>,
    glow_cone_red_material: Handle<StandardMaterial>,
    glow_cone_blue_material: Handle<StandardMaterial>,
}

// Add new component for light cone animation
#[derive(Component)]
pub struct LightConeAnimation {
    timer: Timer,
    base_scale: Vec3,
    is_red: bool,
    color_timer: Timer,
}

// Add a new component for individual sentry timing
#[derive(Component)]
pub struct SentryTiming {
    time_offset: f32,
}

// Add new system for animating the light cone
pub fn animate_light_cones(
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    explosion_materials: Res<ExplosionMaterials>,
    mut query: Query<(&mut Transform, &mut LightConeAnimation, &mut Handle<StandardMaterial>, &Parent, &GlobalTransform)>,
    timing_query: Query<&SentryTiming>,
) {
    for (mut transform, mut anim, mut material_handle, parent, global_transform) in query.iter_mut() {
        let time_offset = timing_query.get(parent.get()).map_or(0.0, |timing| timing.time_offset);
        let individual_time = time.elapsed_seconds() + time_offset;
        
        anim.timer.tick(time.delta());
        anim.color_timer.tick(time.delta());
        
        if anim.color_timer.just_finished() {
            anim.is_red = !anim.is_red;
            *material_handle = if anim.is_red {
                explosion_materials.glow_cone_red_material.clone()
            } else {
                explosion_materials.glow_cone_blue_material.clone()
            };
        }
        
        let intensity_factor = (individual_time * 1.5 * PI).sin() * 0.5 + 0.5;
        
        if let Some(material) = materials.get_mut(material_handle.id()) {
            if anim.is_red {
                material.base_color = Color::srgba(
                    0.9 * intensity_factor,
                    0.0,
                    0.0,
                    0.9 * intensity_factor
                );
                material.emissive = Color::srgb(
                    1.0 * intensity_factor,
                    0.0,
                    0.0
                ).into();
            } else {
                material.base_color = Color::srgba(
                    0.0,
                    0.0,
                    0.3 * intensity_factor,
                    0.8 * intensity_factor
                );
                material.emissive = Color::srgb(
                    0.0,
                    0.0,
                    0.8 * intensity_factor
                ).into();
            }
        }
        
        let is_ground_disc = transform.translation.y < 0.0;
        
        if is_ground_disc {
            let ground_pulse = (individual_time * 1.5).sin() * 0.8;
            let ground_scale = 1.0 + ground_pulse.abs();
            transform.scale = Vec3::new(
                anim.base_scale.x * ground_scale,
                anim.base_scale.y * ground_scale,
                anim.base_scale.z * ground_scale
            );
            
            // Keep disc directly under sentry
            transform.translation = Vec3::new(0.0, -0.4, 0.0);
            transform.rotation = Quat::IDENTITY * Quat::from_rotation_x(-PI / 2.0);
        } else {
            let pulse = (individual_time * 1.5).sin() * 0.3;
            let radius_scale = 1.0 + pulse.abs();
            transform.scale = Vec3::new(
                anim.base_scale.x * radius_scale,
                anim.base_scale.y,  // Height stays constant
                anim.base_scale.z * radius_scale
            );
        }
    }
}

// Helper function for spawning a sentry
fn spawn_sentry_at(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    position: Vec3,
    materials: &ExplosionMaterials,
    minimap_resources: &Res<MinimapResources>,
    spatial_query: &SpatialQuery,
) -> bool {  // Return bool to indicate if spawn was successful
    // Cast a ray up and down to check if position is inside a building
    let ray_pos = position + Vec3::new(0.0, 0.5, 0.0);
    let ray_dir = Dir3::Y;
    let max_distance = 2000.0;
    let filter = SpatialQueryFilter::default();
    
    let overhead_hits = spatial_query.ray_hits(
        ray_pos,
        ray_dir,
        max_distance,
        1,
        true,
        filter
    );

    // If we have hits above, we're indoors
    if !overhead_hits.is_empty() {
        return false; // Don't spawn if inside
    }

    let sentry_entity = commands.spawn((
        SceneBundle {       
            scene: asset_server
                .load(GltfAssetLabel::Scene(0)
                .from_asset("models/tmpn3hy22ev.glb")),
            transform: Transform::from_translation(position)
                .with_scale(Vec3::splat(1.0)),
            ..default()
        },
        materials.sentry_red_material.clone(),
        Sentry {
            view_distance: 500.0,
            view_angle: std::f32::consts::PI / 2.0,
            follow_speed: 10.0,
            velocity: Vec3::ZERO,
        },
        Name::new("Sentry"),
        SentryTiming {
            time_offset: rand::random::<f32>() * 100.0, // Random phase offset
        },
    )).with_children(|parent| {
        // Top siren light (subtle pulsing)
        parent.spawn((
            PbrBundle {
                mesh: materials.glow_cone_mesh.clone(),
                material: materials.glow_cone_red_material.clone(),
                transform: Transform::from_xyz(0.0, 0.8, 0.0)
                    .with_scale(Vec3::new(2.0, 1.0, 2.0)),
                ..default()
            },
            LightConeAnimation {
                timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                base_scale: Vec3::new(2.0, 1.0, 2.0),
                is_red: true,
                color_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            },
        ));
        
        // Ground disc (dramatic pulsing)
        parent.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(Circle::new(2.0))),
                material: materials.glow_cone_red_material.clone(),
                transform: Transform::from_xyz(0.0, -0.4, 0.0)
                    .with_rotation(Quat::from_rotation_x(-PI / 2.0)),
                ..default()
            },
            LightConeAnimation {
                timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                base_scale: Vec3::new(1.0, 1.0, 1.0),
                is_red: true,
                color_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            },
        ));
    }).id();

    // Spawn minimap marker for the sentry
    commands.spawn((
        PbrBundle {
            mesh: minimap_resources.sentry_mesh.clone(),
            material: minimap_resources.sentry_material.clone(),
            transform: Transform::from_xyz(position.x, MINIMAP_MARKER_HEIGHT, position.z),
            ..default()
        },
        MinimapMarker,
        SentryMinimapMarker(sentry_entity),
    ));

    true // Spawn successful
}

// System function for initial spawn
pub fn spawn_sentry(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    explosion_materials: Res<ExplosionMaterials>,
    minimap_resources: Res<MinimapResources>,
    spatial_query: SpatialQuery,
) {
    let sentry_position = Vec3::new(
        PROTAGONIST_START.position.x + 300.0,
        PROTAGONIST_START.position.y,
        PROTAGONIST_START.position.z - 200.0
    );
    spawn_sentry_at(&mut commands, &asset_server, &mut meshes, sentry_position, &explosion_materials, &minimap_resources, &spatial_query);
}

pub fn sentry_follow_system(
    mut commands: Commands,
    mut query_set: ParamSet<(
        Query<(&Transform, &Protagonist), With<Protagonist>>,
        Query<(Entity, &mut Transform, &mut Sentry, Option<&Collider>, &SentryTiming)>,
    )>,
    time: Res<Time>,
    explosion_materials: Res<ExplosionMaterials>,
    mut explosion_counter: ResMut<ExplosionCounter>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Get protagonist data first
    let (protagonist_pos, is_driving) = {
        let protagonist_query = query_set.p0();
        if let Ok((transform, protagonist)) = protagonist_query.get_single() {
            (transform.translation, protagonist.is_driving)
        } else {
            return;
        }
    };

    // Collect sentry positions and entities for collision checking
    let mut sentry_positions: Vec<(Entity, Vec3)> = Vec::new();
    {
        let sentry_query = query_set.p1();
        for (entity, transform, _, _, _) in sentry_query.iter() {
            sentry_positions.push((entity, transform.translation));
        }
    }

    // Get sentry query
    let mut sentry_query = query_set.p1();
    for (entity, mut transform, mut sentry, collider, timing) in sentry_query.iter_mut() {
        // Update collider based on driving state
        if is_driving {
            // Remove collider when protagonist is driving
            if collider.is_some() {
                commands.entity(entity).remove::<Collider>();
            }
        } else {
            // Add collider when protagonist is not driving
            if collider.is_none() {
                commands.entity(entity).insert(Collider::sphere(1.0));
            }
        }

        // Check for collisions with other sentries
        for &(other_entity, other_pos) in sentry_positions.iter() {
            if entity != other_entity {
                let distance_to_other = (transform.translation - other_pos).length();
                if distance_to_other < 2.0 {  // Collision threshold
                    // Trigger explosion for both sentries
                    for &entity_to_explode in &[entity, other_entity] {
                        commands.entity(entity_to_explode).despawn_recursive();
                        
                        commands.spawn((
                            TransformBundle {
                                local: transform.clone(),
                                ..default()
                            },
                            SentryExplosion {
                                timer: Timer::from_seconds(1.0, TimerMode::Once),
                                initial_scale: transform.scale * if is_driving { 3.0 } else { 1.0 },
                                start_time: time.elapsed_seconds(),  // Add start time
                            },
                        ));

                        spawn_explosion_effects(
                            &mut commands,
                            &explosion_materials,
                            transform.translation,
                            &mut explosion_counter,
                            50.0,
                            is_driving,
                            &mut materials,
                            &time,
                        );
                    }
                    return;  // Exit early since this sentry is now destroyed
                }
            }
        }

        // Clamp Y position between 3.0 and 4.0
        transform.translation.y = transform.translation.y.clamp(3.0, 4.0);
        
        let individual_time = time.elapsed_seconds() + timing.time_offset;
        let direction = protagonist_pos - transform.translation;
        let distance = direction.length();

        if distance < sentry.view_distance {
            let direction = direction.normalize();
            
            // More erratic and unsettling movement
            let horror_offset = Vec3::new(
                (individual_time * 3.0).sin() * 2.0 + (individual_time * 7.0).cos() * 1.0,
                (individual_time * 4.0).cos() * 0.5 + (individual_time * 2.0).sin() * 0.3,
                (individual_time * 3.5).sin() * 2.0 + (individual_time * 6.0).cos() * 1.0
            );

            // More frequent and intense sudden movements
            let sudden_movement = if (individual_time * 0.5).sin() > 0.8 {
                let intensity = (individual_time * 0.5).sin().powf(2.0) * 3.0;
                Vec3::new(
                    rand::random::<f32>() * intensity - intensity/2.0,
                    rand::random::<f32>() * intensity/2.0,
                    rand::random::<f32>() * intensity - intensity/2.0
                )
            } else {
                Vec3::ZERO
            };

            // Add circular strafing behavior
            let strafe_radius = 5.0;
            let strafe_speed = 2.0;
            let strafe_offset = Vec3::new(
                (individual_time * strafe_speed).cos() * strafe_radius,
                0.0,
                (individual_time * strafe_speed).sin() * strafe_radius
            );

            // Combine all movement components with distance-based intensity
            let distance_factor = (1.0 - (distance / sentry.view_distance)).powf(0.5);
            let movement = (
                direction * sentry.follow_speed + 
                horror_offset * distance_factor * 1.5 + 
                sudden_movement * distance_factor + 
                strafe_offset * distance_factor
            ) * time.delta_seconds();
            
            // Ignore Y movement to keep sentry at constant height
            let movement = Vec3::new(movement.x, 0.0, movement.z);
            
            sentry.velocity = movement / time.delta_seconds();
            transform.translation += movement;

            // Look at target but only rotate around Y axis
            let direction_xz = Vec3::new(direction.x, 0.0, direction.z).normalize();
            transform.rotation = Quat::from_rotation_y(direction_xz.z.atan2(direction_xz.x));
            
            // Trigger explosion at closer range
            if distance < 2.0 {  // Increased explosion trigger distance
                commands.entity(entity).despawn_recursive();
                
                // Spawn drone carcass at explosion location
                commands.spawn(SceneBundle {
                    scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/drone_carcass.glb")),
                    transform: transform.clone(),
                    ..default()
                });

                commands.spawn((
                    TransformBundle {
                        local: transform.clone(),
                        ..default()
                    },
                    SentryExplosion {
                        timer: Timer::from_seconds(1.0, TimerMode::Once),
                        initial_scale: transform.scale * if is_driving { 3.0 } else { 1.0 },
                        start_time: time.elapsed_seconds(),  // Add start time
                    },
                ));

                spawn_explosion_effects(
                    &mut commands,
                    &explosion_materials,
                    transform.translation,
                    &mut explosion_counter,
                    50.0,
                    is_driving,
                    &mut materials,
                    &time,
                );
            }
        }
    }
}

// Helper function to initialize shared materials
pub fn setup_explosion_materials(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let siren_shape = Mesh::from(Cylinder {
        radius: 0.05,
        half_height: 0.025, 
    });

    let materials = ExplosionMaterials {
        particle_mesh: meshes.add(Mesh::from(Cuboid::new(0.2, 0.2, 0.2))),
        red_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.99, 0.0, 0.0),  // Darker red
            emissive: Color::srgb(0.99, 0.0, 0.0).into(),  // Bright red glow
            ..default()
        }),
        sentry_red_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.0, 0.0),  // Darker red
            emissive: Color::srgb(1.0, 0.0, 0.0).into(),  // Bright red glow
            ..default()
        }),
        glow_cone_mesh: meshes.add(siren_shape),
        glow_cone_red_material: materials.add(StandardMaterial {
            base_color: Color::srgba(0.9, 0.0, 0.2, 0.9),
            emissive: Color::srgb(1.0, 0.0, 0.2).into(),
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        glow_cone_blue_material: materials.add(StandardMaterial {
            base_color: Color::srgba(0.1, 0.0, 0.3, 0.8),  // Darker blue
            emissive: Color::srgb(0.2, 0.0, 0.8).into(),  // Subtle glow
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
    };
    commands.insert_resource(materials);
    commands.insert_resource(ExplosionCounter::default());
}

// Update spawn_explosion_effects to be more efficient
fn spawn_explosion_effects(
    commands: &mut Commands,
    explosion_materials: &ExplosionMaterials,
    position: Vec3,
    explosion_counter: &mut ExplosionCounter,
    camera_distance: f32,
    is_driving: bool,
    materials: &mut Assets<StandardMaterial>,
    time: &Res<Time>,
) {
    if explosion_counter.count >= explosion_counter.max_allowed {
        return;
    }
    explosion_counter.count += 1;

    let base_particles = if is_driving { 1000 } else { 500 }; // Halved particle count
    let distance_scale = (1.0 - (camera_distance / 200.0).clamp(0.0, 0.9)) as f32;
    let particle_count = (base_particles as f32 * distance_scale) as i32;

    let scale = if is_driving { 6.0 } else { 3.0 }; // Increased scale to compensate for fewer particles

    // Initial explosion particles - reduced count but longer lasting
    for _ in 0..10 { // Reduced from 50 to 10
        let random_dir = Vec3::new(
            rand::random::<f32>() * 2.0 - 1.0,
            rand::random::<f32>() * 2.0 + 1.0,
            rand::random::<f32>() * 2.0 - 1.0,
        ).normalize();
        
        commands.spawn((
            PbrBundle {
                mesh: explosion_materials.particle_mesh.clone(),
                material: explosion_materials.red_material.clone(),
                transform: Transform::from_translation(position)
                    .with_scale(Vec3::splat(2.0 * scale)),
                ..default()
            },
            ExplosionParticle {
                velocity: random_dir * 15.0, // Slower initial velocity
                lifetime: Timer::from_seconds(2.0, TimerMode::Once), // Shorter but more reasonable
                is_smoke: false,
                initial_scale: 2.0,
                origin: position,
                start_time: time.elapsed_seconds(),
            },
        ));
    }

    // Smoke column particles - reduced count but maintain visual density
    for i in 0..(particle_count / 4) { // Reduced to 1/4
        let initial_lifetime = rand::random::<f32>() * 160.0;
        let mut timer = Timer::from_seconds(160.0, TimerMode::Once);
        timer.tick(Duration::from_secs_f32(initial_lifetime));

        let height_offset = i as f32 * 1.6; // Doubled spacing to compensate for fewer particles
        let random_offset = Vec3::new(
            rand::random::<f32>() * 2.0 - 1.0,
            height_offset,
            rand::random::<f32>() * 2.0 - 1.0
        );
        
        commands.spawn((
            PbrBundle {
                mesh: explosion_materials.particle_mesh.clone(),
                material: materials.add(StandardMaterial {
                    base_color: Color::srgba(0.95, 0.3, 0.1, 0.98),
                    emissive: Color::srgba(0.5, 0.15, 0.0, 0.98).into(), // Halved initial intensity
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                }),
                transform: Transform::from_translation(position + random_offset)
                    .with_scale(Vec3::splat(12.0 * scale)),
                ..default()
            },
            ExplosionParticle {
                velocity: Vec3::new(0.0, 6.0 + rand::random::<f32>() * 3.0, 0.0),
                lifetime: timer,
                is_smoke: true,
                initial_scale: 12.0,
                origin: position,
                start_time: time.elapsed_seconds(),
            },
        ));
    }
}

pub fn periodic_sentry_spawn(
    mut commands: Commands,
    time: Res<Time>,
    mut timer_query: Query<&mut SentrySpawnTimer>,
    protagonist_query: Query<(&Transform, &Protagonist)>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    explosion_materials: Res<ExplosionMaterials>,
    minimap_resources: Res<MinimapResources>,
    spatial_query: SpatialQuery,
) {
    // Initialize timer if it doesn't exist
    if timer_query.is_empty() {
        commands.spawn(SentrySpawnTimer(Timer::from_seconds(SENTRY_SPAWN_INTERVAL, TimerMode::Repeating)));
        return;
    }

    let mut timer = timer_query.single_mut();
    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        if let Ok((protagonist_transform, protagonist)) = protagonist_query.get_single() {
            // Don't spawn if protagonist is swimming, in dirigible, or not outside
            if protagonist.is_swimming || protagonist.is_dirigible || !protagonist.is_outside {
                return;
            }

            // Get protagonist's forward direction
            let forward = protagonist_transform.forward();
            
            // Try up to 5 times to find a valid spawn position
            for _ in 0..5 {
                // Generate random angle within 120-degree arc in front of protagonist
                let angle = rand::random::<f32>() * PI / 1.5 - PI / 3.0; // -60° to +60°
                let distance = 80.0 + rand::random::<f32>() * 120.0; // 80-200 units away
                
                // Calculate spawn position in front arc
                let rotation = Quat::from_rotation_y(angle);
                let spawn_direction = rotation * forward;
                let spawn_pos = protagonist_transform.translation + spawn_direction * distance;
                
                if spawn_sentry_at(&mut commands, &asset_server, &mut meshes, spawn_pos, &explosion_materials, &minimap_resources, &spatial_query) {
                    break; // Successfully spawned
                }
            }
        }
    }
}

pub fn update_explosion_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut ExplosionParticle, &Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut explosion_counter: ResMut<ExplosionCounter>,
) {
    for (entity, mut transform, mut particle, material_handle) in query.iter_mut() {
        let effect_age = time.elapsed_seconds() - particle.start_time;
        let fade_start = 3.0; // Reduced from 15.0 for faster color transition
        let fade_duration = 2.0; // Reduced from 10.0 for quicker fadeout
        let fade_factor = ((fade_start - effect_age) / fade_duration).clamp(0.0, 1.0);

        particle.lifetime.tick(time.delta());
        
        if particle.lifetime.finished() {
            if particle.is_smoke && effect_age < fade_start + fade_duration {
                transform.translation = particle.origin + Vec3::new(
                    rand::random::<f32>() * 0.2 - 0.1,
                    0.0,
                    rand::random::<f32>() * 0.2 - 0.1
                );
                transform.scale = Vec3::splat(6.0 * fade_factor);
                particle.lifetime.reset();
                particle.velocity = Vec3::new(
                    0.0,
                    (2.0 + rand::random::<f32>() * 1.0) * fade_factor,
                    0.0
                );
                
                if let Some(material) = materials.get_mut(material_handle) {
                    let height = transform.translation.y - particle.origin.y;
                    let height_fraction = (height / 2000.0).clamp(0.0, 1.0);
                    
                    let fire_color = Color::srgba(0.95, 0.3, 0.1, 0.98);
                    let smoke_color = Color::srgba(0.2, 0.2, 0.2, 0.7);
                    
                    let color_mix = height_fraction.powf(0.5);
                    let fire_rgba = fire_color.to_srgba();
                    let smoke_rgba = smoke_color.to_srgba();
                    let base_color = Color::srgba(
                        fire_rgba.red * (1.0 - color_mix) + smoke_rgba.red * color_mix,
                        fire_rgba.green * (1.0 - color_mix) + smoke_rgba.green * color_mix,
                        fire_rgba.blue * (1.0 - color_mix) + smoke_rgba.blue * color_mix,
                        fire_rgba.alpha * (1.0 - color_mix) + smoke_rgba.alpha * color_mix,
                    );
                    
                    material.base_color = Color::srgba(
                        base_color.to_srgba().red,
                        base_color.to_srgba().green,
                        base_color.to_srgba().blue,
                        base_color.to_srgba().alpha,
                    );
                    
                    let emissive_intensity = (1.0 - height_fraction) * fade_factor;
                    material.emissive = Color::srgb(
                        emissive_intensity,
                        emissive_intensity * 0.3,
                        0.0
                    ).into();
                }
            } else {
                commands.entity(entity).despawn();
                if !particle.is_smoke {
                    explosion_counter.count = explosion_counter.count.saturating_sub(1);
                }
            }
        } else if particle.is_smoke {
            let height = transform.translation.y - particle.origin.y;
            let height_fraction = (height / 2000.0).clamp(0.0, 1.0); // 100x taller (was 20.0)
            
            let chaos_factor = height_fraction * height_fraction * fade_factor;
            let wobble = Vec3::new(
                ((time.elapsed_seconds() + particle.start_time) * 0.8).sin() * (0.1 + chaos_factor * 2.0), // More dramatic wobble
                (chaos_factor * 0.5).sin() * 0.5, // More vertical distortion
                ((time.elapsed_seconds() + particle.start_time) * 0.8).cos() * (0.1 + chaos_factor * 2.0)
            );
            
            transform.translation += (particle.velocity * fade_factor + wobble) * time.delta_seconds();
            // Particles grow larger as they rise
            transform.scale = Vec3::splat(particle.initial_scale * (1.0 + height_fraction * 2.0) * fade_factor);
            
            if let Some(material) = materials.get_mut(material_handle) {
                let height_fraction = height_fraction.powf(0.15);
                
                // Much darker smoke transition
                material.base_color = Color::srgba(
                    0.95 * (1.0 - height_fraction).powf(1.5),  // Red fades to black
                    0.3 * (1.0 - height_fraction).powf(2.0),   // Green fades faster
                    0.1 * (1.0 - height_fraction).powf(2.0),   // Blue minimal
                    (0.95 - height_fraction * 0.3) * fade_factor  // More opaque smoke
                );

                // Keep strong emissive at base only
                material.emissive = Color::srgb(
                    1.0 * (1.0 - height_fraction).powf(2.0),   // Red glow fades quickly
                    0.3 * (1.0 - height_fraction).powf(3.0),   // Orange fades faster
                    0.0                                         // No blue
                ).into();
            }
        }
    }
}

pub fn update_explosion_light(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PointLight, &mut ExplosionLight)>,
) {
    for (entity, mut light, explosion_light) in query.iter_mut() {
        let elapsed = time.elapsed_seconds() - explosion_light.start_time;
        let fraction = (elapsed / explosion_light.timer.duration().as_secs_f32()).clamp(0.0, 1.0);
        
        if fraction >= 1.0 {
            commands.entity(entity).despawn();
        } else {
            light.intensity = explosion_light.intensity * (1.0 - fraction);
        }
    }
}
