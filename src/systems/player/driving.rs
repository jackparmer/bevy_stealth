use bevy::prelude::*;
use avian3d::prelude::*;

use crate::components::Protagonist;
use crate::systems::environments::terrain::Terrain;

pub fn set_driving_state(
    protagonist: &mut Protagonist,
    scene: &mut Handle<Scene>,
    asset_server: &AssetServer,
    new_state: bool
) {
    protagonist.is_driving = new_state;
    *scene = if new_state {
        asset_server.load("models/KB03-apc.glb#Scene0")
    } else {
        asset_server.load("models/Protagonist.glb#Scene0")
    };
}

pub fn toggle_driving(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &mut Protagonist, &mut Handle<Scene>)>,
    asset_server: Res<AssetServer>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        for (_, mut protagonist, mut scene) in query.iter_mut() {
            let new_state = !protagonist.is_driving;  // Store the value first
            set_driving_state(&mut protagonist, &mut scene, &asset_server, new_state);
        }
    }
}

pub fn driving_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut protagonist_query: Query<(Entity, &mut Transform, &Protagonist)>,
    mut velocity_query: Query<&mut LinearVelocity, With<Protagonist>>,
    mut angular_velocity_query: Query<&mut AngularVelocity, With<Protagonist>>,
    terrain_query: Query<Entity, With<Terrain>>,
    spatial_query: SpatialQuery,
    mut collision_events: EventReader<CollisionStarted>,
    time: Res<Time>,
) {
    // Base driving parameters
    const TURN_SPEED: f32 = 4.0;
    const BASE_MAX_SPEED: f32 = 800.0;
    const BASE_MIN_SPEED: f32 = -400.0;
    const BASE_ACCELERATION: f32 = 200.0;
    const BRAKE_FORCE: f32 = 600.0;
    const DRIFT_FACTOR: f32 = 0.92;
    const TURN_SENSITIVITY: f32 = 2.5;
    const SPEED_TURN_FACTOR: f32 = 0.4;
    const GROUND_ADHESION: f32 = -980.0;
    const TERRAIN_SPEED_MULTIPLIER: f32 = 1000.0;

    if let Ok((protagonist_entity, mut protagonist_transform, protagonist)) = protagonist_query.get_single_mut() {
        if !protagonist.is_driving {
            return;
        }

        // Multiple ground check points using raycasts
        let ray_positions = [
            protagonist_transform.translation + Vec3::new(2.0, 0.5, 2.0),   // Front right
            protagonist_transform.translation + Vec3::new(-2.0, 0.5, 2.0),  // Front left
            protagonist_transform.translation + Vec3::new(2.0, 0.5, -2.0),  // Back right
            protagonist_transform.translation + Vec3::new(-2.0, 0.5, -2.0), // Back left
            protagonist_transform.translation + Vec3::new(0.0, 0.5, 0.0),   // Center
        ];

        let ray_dir = Dir3::NEG_Y;
        let max_distance = 5.0;
        let filter = SpatialQueryFilter::from_excluded_entities([protagonist_entity]);
        
        let mut is_grounded = false;
        let mut ground_height = protagonist_transform.translation.y;

        // Check all ray positions and find the highest ground point
        for ray_pos in ray_positions.iter() {
            let hits = spatial_query.ray_hits(
                *ray_pos, 
                ray_dir, 
                max_distance,
                1,
                true,
                filter.clone()
            );

            if let Some(hit) = hits.first() {
                is_grounded = true;
                let hit_height = ray_pos.y - hit.time_of_impact;
                ground_height = ground_height.max(hit_height);
            }
        }

        // Check terrain collision using events
        let mut on_terrain = false;
        if let Ok(terrain_entity) = terrain_query.get_single() {
            for CollisionStarted(e1, e2) in collision_events.read() {
                if (*e1 == terrain_entity || *e2 == terrain_entity) && 
                   (*e1 == protagonist_entity || *e2 == protagonist_entity) {
                    on_terrain = true;
                    break;
                }
            }
        }

        let mut current_velocity = velocity_query.single_mut();
        
        // Enhanced ground adhesion logic
        if is_grounded {
            // Smoothly adjust height to match ground
            protagonist_transform.translation.y = ground_height;
            current_velocity.0.y = 0.0;
        } else {
            // Apply gravity when not grounded
            current_velocity.0.y += GROUND_ADHESION * time.delta_seconds();
        }

        // Extract only Y rotation and force upright orientation
        let (yaw, _, _) = protagonist_transform.rotation.to_euler(EulerRot::YXZ);
        protagonist_transform.rotation = Quat::from_rotation_y(yaw);

        // Decompose velocity into forward and lateral components
        let forward_dir = protagonist_transform.left().as_vec3();
        let right_dir = protagonist_transform.forward().as_vec3();
        let current_speed = current_velocity.0.dot(forward_dir);
        let lateral_speed = current_velocity.0.dot(right_dir);

        let mut acceleration_amount = 0.0;
        if keyboard_input.pressed(KeyCode::KeyW) {
            acceleration_amount = BASE_ACCELERATION * time.delta_seconds();
        } else if keyboard_input.pressed(KeyCode::KeyS) {
            if current_speed > 0.0 {
                acceleration_amount = -BRAKE_FORCE * time.delta_seconds();
            } else {
                acceleration_amount = -BASE_ACCELERATION * 0.5 * time.delta_seconds();
            }
        } else if current_speed.abs() > 1.0 {
            acceleration_amount = -current_speed.signum() * BRAKE_FORCE * 0.5 * time.delta_seconds();
        }

        let new_speed = (current_speed + acceleration_amount)
            .clamp(BASE_MIN_SPEED, BASE_MAX_SPEED);

        // Calculate turning based on speed
        let turn_amount = if keyboard_input.pressed(KeyCode::KeyA) {
            TURN_SPEED
        } else if keyboard_input.pressed(KeyCode::KeyD) {
            -TURN_SPEED
        } else {
            0.0
        };

        // Adjust turn rate based on speed
        let speed_ratio = (current_speed.abs() / BASE_MAX_SPEED).powf(SPEED_TURN_FACTOR);
        let adjusted_turn = turn_amount * TURN_SENSITIVITY * speed_ratio;

        // Apply drift mechanics
        let new_lateral = lateral_speed * DRIFT_FACTOR;

        // Combine forward and lateral velocities
        current_velocity.0 = forward_dir * new_speed + right_dir * new_lateral;

        // Apply rotation through angular velocity
        for mut angular_velocity in angular_velocity_query.iter_mut() {
            angular_velocity.0 = Vec3::new(0.0, adjusted_turn, 0.0);
        }

        // Optional: Add drift particles or effects when drifting
        if new_lateral.abs() > 50.0 {
            info!("Drifting! Speed: {}", new_lateral);
        }
    }
}
