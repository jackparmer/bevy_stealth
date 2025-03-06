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
use bevy::render::view::RenderLayers;

const SENTRY_SPAWN_INTERVAL: f32 = 1.0; // Reduced from 2.0 for more frequent spawns

// Sentry movement constants
const SENTRY_VIEW_DISTANCE: f32 = 800.0;
const SENTRY_VIEW_ANGLE: f32 = std::f32::consts::PI / 1.5;
const SENTRY_FOLLOW_SPEED: f32 = 15.0;
const SENTRY_CLOSE_RANGE_MULTIPLIER: f32 = 2.0;
const SENTRY_MID_RANGE_MULTIPLIER: f32 = 1.5;
const SENTRY_CLOSE_RANGE_THRESHOLD: f32 = 20.0;
const SENTRY_MID_RANGE_THRESHOLD: f32 = 50.0;
const SENTRY_VERTICAL_SPEED_MULTIPLIER: f32 = 0.7;
const SENTRY_SCALE: f32 = 1.5;

// Explosion constants
const EXPLOSION_MAX_ALLOWED: usize = 10;
const EXPLOSION_BASE_PARTICLES_DRIVING: i32 = 1000;
const EXPLOSION_BASE_PARTICLES_NORMAL: i32 = 500;
const EXPLOSION_SCALE_DRIVING: f32 = 6.0;
const EXPLOSION_SCALE_NORMAL: f32 = 3.0;
const EXPLOSION_FADE_START: f32 = 3.0;
const EXPLOSION_FADE_DURATION: f32 = 2.0;
const EXPLOSION_SMOKE_HEIGHT_MAX: f32 = 2000.0;
const EXPLOSION_PARTICLE_LIFETIME: f32 = 160.0;
const EXPLOSION_SMOKE_SPACING: f32 = 1.6;
const EXPLOSION_INITIAL_PARTICLES: i32 = 10;

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
            max_allowed: EXPLOSION_MAX_ALLOWED,
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
    pub time_offset: f32,
}

// Add new resource to track sentry count
#[derive(Resource)]
pub struct SentryCounter {
    count: usize,
    max_allowed: usize,
}

impl Default for SentryCounter {
    fn default() -> Self {
        Self {
            count: 0,
            max_allowed: 10, // Maximum number of sentries allowed
        }
    }
}

// Add new system for animating the light cone
pub fn animate_light_cones(
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    explosion_materials: Res<ExplosionMaterials>,
    mut query: Query<(&mut Transform, &mut LightConeAnimation, &mut Handle<StandardMaterial>, &Parent, &GlobalTransform)>,
    timing_query: Query<&SentryTiming>,
) {
    for (mut transform, mut anim, mut material_handle, parent, _) in query.iter_mut() {
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
                    0.3 * intensity_factor // More transparent
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
                    0.2 * intensity_factor // More transparent
                );
                material.emissive = Color::srgb(
                    0.0,
                    0.0,
                    0.8 * intensity_factor
                ).into();
            }
        }
        
        let pulse = (individual_time * 1.5).sin() * 0.3;
        let scale = 1.0 + pulse.abs();
        transform.scale = Vec3::splat(anim.base_scale.x * scale);
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
    sentry_counter: &mut ResMut<SentryCounter>,
) -> bool {
    if sentry_counter.count >= sentry_counter.max_allowed {
        return false;
    }

    // Cast a ray down to find the ground position
    let ray_start = position + Vec3::new(0.0, 10.0, 0.0); // Start higher to ensure we find ground
    let ray_dir = Dir3::NEG_Y;
    let max_distance = 20.0;
    let filter = SpatialQueryFilter::default();
    
    let ground_hit = spatial_query.ray_hits(
        ray_start,
        ray_dir,
        max_distance,
        1,
        true,
        filter.clone()  // Clone the filter here
    ).first().copied();

    // If we don't find ground, don't spawn
    let ground_position = if let Some(hit) = ground_hit {
        ray_start + Vec3::NEG_Y * hit.time_of_impact
    } else {
        return false;
    };

    // Cast a ray up to check if position is inside a building
    let overhead_hits = spatial_query.ray_hits(
        ground_position + Vec3::new(0.0, 0.5, 0.0),
        Dir3::Y,
        max_distance,
        1,
        true,
        filter
    );

    // If we have hits above, we're indoors
    if !overhead_hits.is_empty() {
        return false;
    }

    let sentry_entity = commands.spawn((
        SceneBundle {       
            scene: asset_server
                .load(GltfAssetLabel::Scene(0)
                .from_asset("models/tmpn3hy22ev.glb")),
            transform: Transform::from_translation(ground_position)
                .with_scale(Vec3::splat(SENTRY_SCALE)),
            ..default()
        },
        materials.sentry_red_material.clone(),
        Sentry {
            view_distance: SENTRY_VIEW_DISTANCE,
            view_angle: SENTRY_VIEW_ANGLE,
            follow_speed: SENTRY_FOLLOW_SPEED,
            velocity: Vec3::ZERO,
        },
        Name::new("Sentry"),
        SentryTiming {
            time_offset: rand::random::<f32>() * 100.0,
        },
    )).with_children(|parent| {
        // Sphere light (pulsing)
        parent.spawn((
            PbrBundle {
                mesh: materials.glow_cone_mesh.clone(),
                material: materials.glow_cone_red_material.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.0) // Center on sentry
                    .with_scale(Vec3::splat(3.0)), // Adjust size to envelope sentry
                ..default()
            },
            LightConeAnimation {
                timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                base_scale: Vec3::splat(3.0), // Base scale for sphere
                is_red: true,
                color_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            },
        ));
    }).id();

    // Update minimap marker to use ground position and include RenderLayers
    commands.spawn((
        PbrBundle {
            mesh: minimap_resources.sentry_mesh.clone(),
            material: minimap_resources.sentry_material.clone(),
            transform: Transform::from_xyz(
                ground_position.x,
                MINIMAP_MARKER_HEIGHT,
                ground_position.z
            ),
            ..default()
        },
        MinimapMarker,
        SentryMinimapMarker(sentry_entity),
        RenderLayers::layer(1), // Add RenderLayers component
    ));

    sentry_counter.count += 1;
    true
}

// System function for initial spawn
pub fn spawn_sentry(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    explosion_materials: Res<ExplosionMaterials>,
    minimap_resources: Res<MinimapResources>,
    spatial_query: SpatialQuery,
    mut sentry_counter: ResMut<SentryCounter>,
) {
    let sentry_position = Vec3::new(
        PROTAGONIST_START.position.x + 300.0,
        PROTAGONIST_START.position.y,
        PROTAGONIST_START.position.z - 200.0
    );
    spawn_sentry_at(&mut commands, &asset_server, &mut meshes, sentry_position, &explosion_materials, &minimap_resources, &spatial_query, &mut sentry_counter);
}

pub fn sentry_follow_system(
    mut commands: Commands,
    mut query_set: ParamSet<(
        Query<(&Transform, &Protagonist), With<Protagonist>>,
        Query<(Entity, &mut Transform, &mut Sentry, &SentryTiming)>,
    )>,
    time: Res<Time>,
    explosion_materials: Res<ExplosionMaterials>,
    mut explosion_counter: ResMut<ExplosionCounter>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    spatial_query: SpatialQuery,
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
        for (entity, transform, _, _) in sentry_query.iter() {
            sentry_positions.push((entity, transform.translation));
        }
    }

    // Get sentry query
    let mut sentry_query = query_set.p1();
    for (entity, mut transform, mut sentry, timing) in sentry_query.iter_mut() {
        // Check for collisions with other sentries
        for &(other_entity, other_pos) in sentry_positions.iter() {
            if entity != other_entity {
                let distance_to_other = (transform.translation - other_pos).length();
                if distance_to_other < 3.0 {
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
                                start_time: time.elapsed_seconds(),
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
                    return;
                }
            }
        }

        let individual_time = time.elapsed_seconds() + timing.time_offset;
        let direction = protagonist_pos - transform.translation;
        let distance = direction.length();

        // More aggressive movement when closer
        let speed_multiplier = if distance < SENTRY_CLOSE_RANGE_THRESHOLD {
            SENTRY_CLOSE_RANGE_MULTIPLIER
        } else if distance < SENTRY_MID_RANGE_THRESHOLD {
            SENTRY_MID_RANGE_MULTIPLIER
        } else {
            1.0
        };

        if distance < sentry.view_distance {
            // Split movement into horizontal and vertical components
            let horizontal_direction = Vec3::new(direction.x, 0.0, direction.z).normalize();
            let vertical_movement = direction.y;

            // Check for vertical surface contact using raycasts
            let side_ray_distance = 1.0;
            let side_rays = [
                (transform.translation, Vec3::X),
                (transform.translation, Vec3::NEG_X),
                (transform.translation, Vec3::Z),
                (transform.translation, Vec3::NEG_Z),
            ];

            let mut can_climb = false;
            for (ray_origin, ray_dir) in side_rays.iter() {
                let hits = spatial_query.ray_hits(
                    *ray_origin,
                    Dir3::new(ray_dir.normalize()).unwrap(),
                    side_ray_distance,
                    1,
                    true,
                    SpatialQueryFilter::default()
                );

                if !hits.is_empty() {
                    can_climb = true;
                    break;
                }
            }

            // Calculate movement based on conditions
            let mut movement = horizontal_direction * sentry.follow_speed * speed_multiplier * time.delta_seconds();
            
            // Reduce vertical movement speed as well
            if can_climb && vertical_movement.abs() > 0.1 {
                movement.y = vertical_movement.signum() * (sentry.follow_speed * SENTRY_VERTICAL_SPEED_MULTIPLIER) * time.delta_seconds();
            } else {
                movement.y = -9.81 * time.delta_seconds();
            }

            // Ground check to prevent sinking
            let ground_ray = spatial_query.ray_hits(
                transform.translation,
                Dir3::NEG_Y,
                1.0,
                1,
                true,
                SpatialQueryFilter::default()
            );

            if !ground_ray.is_empty() {
                movement.y = movement.y.max(0.0);
            }

            transform.translation += movement;

            // Update rotation with wobble effect
            if direction.length_squared() > 0.001 {
                let wobble = Quat::from_rotation_z((individual_time * 12.0).sin() * 0.25); // Faster, more pronounced wobble
                transform.look_at(protagonist_pos, Vec3::Y);
                transform.rotation *= wobble;
            }
            
            // Trigger explosion at slightly longer range
            if distance < 3.0 { // Increased from 2.0
                commands.entity(entity).despawn_recursive();
                
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
                        start_time: time.elapsed_seconds(),
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
    // Replace siren_shape with sphere
    let siren_shape = Mesh::from(Sphere { radius: 1.0 });

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
            base_color: Color::srgba(0.9, 0.0, 0.2, 0.3), // More transparent
            emissive: Color::srgb(1.0, 0.0, 0.2).into(),
            alpha_mode: AlphaMode::Blend,
            double_sided: true, // Add this for sphere visibility
            ..default()
        }),
        glow_cone_blue_material: materials.add(StandardMaterial {
            base_color: Color::srgba(0.1, 0.0, 0.3, 0.2), // More transparent
            emissive: Color::srgb(0.2, 0.0, 0.8).into(),
            alpha_mode: AlphaMode::Blend,
            double_sided: true, // Add this for sphere visibility
            ..default()
        }),
    };
    commands.insert_resource(materials);
    commands.insert_resource(ExplosionCounter::default());
    commands.insert_resource(SentryCounter::default());
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

    let base_particles = if is_driving { EXPLOSION_BASE_PARTICLES_DRIVING } else { EXPLOSION_BASE_PARTICLES_NORMAL };
    let distance_scale = (1.0 - (camera_distance / 200.0).clamp(0.0, 0.9)) as f32;
    let particle_count = (base_particles as f32 * distance_scale) as i32;

    let scale = if is_driving { EXPLOSION_SCALE_DRIVING } else { EXPLOSION_SCALE_NORMAL };

    // Initial explosion particles
    for _ in 0..EXPLOSION_INITIAL_PARTICLES {
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
                velocity: random_dir * 15.0,
                lifetime: Timer::from_seconds(2.0, TimerMode::Once),
                is_smoke: false,
                initial_scale: 2.0,
                origin: position,
                start_time: time.elapsed_seconds(),
            },
        ));
    }

    // Smoke column particles
    for i in 0..(particle_count / 4) {
        let initial_lifetime = rand::random::<f32>() * EXPLOSION_PARTICLE_LIFETIME;
        let mut timer = Timer::from_seconds(EXPLOSION_PARTICLE_LIFETIME, TimerMode::Once);
        timer.tick(Duration::from_secs_f32(initial_lifetime));

        let height_offset = i as f32 * EXPLOSION_SMOKE_SPACING;
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
    mut sentry_counter: ResMut<SentryCounter>,
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

            // Try to spawn multiple sentries at once
            let spawn_count = if rand::random::<f32>() < 0.3 { 2 } else { 1 };
            
            for _ in 0..spawn_count {
                for _ in 0..5 {
                    let angle = rand::random::<f32>() * PI / 1.5 - PI / 3.0;
                    let distance = 60.0 + rand::random::<f32>() * 100.0; // Closer spawn range (was 80-200)
                    
                    // Calculate spawn position in front arc, but keep Y at ground level
                    let rotation = Quat::from_rotation_y(angle);
                    let spawn_direction = rotation * protagonist_transform.forward();
                    let spawn_pos = Vec3::new(
                        protagonist_transform.translation.x + spawn_direction.x * distance,
                        3.0, // Fixed ground level height
                        protagonist_transform.translation.z + spawn_direction.z * distance,
                    );
                    
                    if spawn_sentry_at(&mut commands, &asset_server, &mut meshes, spawn_pos, &explosion_materials, &minimap_resources, &spatial_query, &mut sentry_counter) {
                        break; // Successfully spawned
                    }
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
        let fade_factor = ((EXPLOSION_FADE_START - effect_age) / EXPLOSION_FADE_DURATION).clamp(0.0, 1.0);

        particle.lifetime.tick(time.delta());
        
        if particle.lifetime.finished() {
            if particle.is_smoke && effect_age < EXPLOSION_FADE_START + EXPLOSION_FADE_DURATION {
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
                    let height_fraction = (height / EXPLOSION_SMOKE_HEIGHT_MAX).clamp(0.0, 1.0);
                    
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
            let height_fraction = (height / EXPLOSION_SMOKE_HEIGHT_MAX).clamp(0.0, 1.0);
            
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

// Add system to update counter when sentries are destroyed
pub fn update_sentry_counter(
    mut sentry_counter: ResMut<SentryCounter>,
    sentry_query: Query<&Sentry>,
) {
    sentry_counter.count = sentry_query.iter().count();
}
