use bevy::prelude::*;
use avian3d::prelude::*;
use crate::components::Protagonist;

pub fn set_driving_state(
    protagonist: &mut Protagonist,
    scene: &mut Handle<Scene>,
    asset_server: &AssetServer,
    new_state: bool,
    commands: &mut Commands,
    protagonist_entity: Entity,
    children_query: &Query<&Children>,
) {
    protagonist.is_driving = new_state;
    *scene = if new_state {
        if let Ok(children) = children_query.get(protagonist_entity) {
            if let Some(spotlight_entity) = children.first() {
                commands.entity(*spotlight_entity).insert(SpotLight {
                    intensity: 10000000.0,
                    color: Color::srgb(1.0, 0.2, 0.2),
                    outer_angle: 0.6,
                    inner_angle: 0.3,
                    shadows_enabled: true,
                    range: 100.0,
                    ..default()
                });
                commands.entity(*spotlight_entity).insert(Transform::from_xyz(0.0, 20.0, 0.0)
                    .looking_at(Vec3::ZERO, Vec3::Z));
            }
        }
        asset_server.load("models/tank.glb#Scene0")
    } else {
        if let Ok(children) = children_query.get(protagonist_entity) {
            if let Some(spotlight_entity) = children.first() {
                commands.entity(*spotlight_entity).insert(SpotLight {
                    intensity: 5000000.0,
                    color: Color::srgb(1.0, 0.95, 0.9),
                    outer_angle: 0.5,
                    inner_angle: 0.2,
                    shadows_enabled: true,
                    range: 50.0,
                    ..default()
                });
                commands.entity(*spotlight_entity).insert(Transform::from_xyz(0.0, 15.0, 2.0)
                    .looking_at(Vec3::new(0.0, 0.0, -10.0), Vec3::Y));
            }
        }
        asset_server.load("models/Protagonist.glb#Scene0")
    };
}

pub fn toggle_driving(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &mut Protagonist, &mut Handle<Scene>)>,
    children_query: Query<&Children>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        for (entity, mut protagonist, mut scene) in query.iter_mut() {
            let new_state = !protagonist.is_driving;
            set_driving_state(&mut protagonist, &mut scene, &asset_server, new_state, &mut commands, entity, &children_query);
        }
    }
}

pub fn driving_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut protagonist_query: Query<(Entity, &mut Transform, &Protagonist)>,
    mut velocity_query: Query<&mut LinearVelocity, With<Protagonist>>,
    mut angular_velocity_query: Query<&mut AngularVelocity, With<Protagonist>>,
    spatial_query: SpatialQuery,
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
    const MIN_DRIVING_HEIGHT: f32 = 10.0;  // New constant for minimum height

    // Ground detection parameters
    const GROUND_CHECK_HEIGHT: f32 = 5.0;
    const GROUND_CHECK_DISTANCE: f32 = 100.0;
    const GROUND_SNAP_SPEED: f32 = 0.1;
    const SUSPENSION_HEIGHT: f32 = -2.0;

    if let Ok((protagonist_entity, mut protagonist_transform, protagonist)) = protagonist_query.get_single_mut() {
        if !protagonist.is_driving {
            return;
        }

        // Ensure tank stays above minimum height
        if protagonist_transform.translation.y < MIN_DRIVING_HEIGHT {
            protagonist_transform.translation.y = MIN_DRIVING_HEIGHT;
            if let Ok(mut velocity) = velocity_query.get_single_mut() {
                velocity.0.y = 0.0;
            }
        }

        // Multiple ground check points using raycasts
        let ray_positions = [
            protagonist_transform.translation + Vec3::new(3.0, GROUND_CHECK_HEIGHT, 3.0),   // Front right
            protagonist_transform.translation + Vec3::new(-3.0, GROUND_CHECK_HEIGHT, 3.0),  // Front left
            protagonist_transform.translation + Vec3::new(3.0, GROUND_CHECK_HEIGHT, -3.0),  // Back right
            protagonist_transform.translation + Vec3::new(-3.0, GROUND_CHECK_HEIGHT, -3.0), // Back left
            protagonist_transform.translation + Vec3::new(0.0, GROUND_CHECK_HEIGHT, 0.0),   // Center
        ];

        let ray_dir = Dir3::NEG_Y;
        let filter = SpatialQueryFilter::from_excluded_entities([protagonist_entity]);
        
        let mut ground_height = f32::NEG_INFINITY;
        let mut is_grounded = false;

        // Check all ray positions and find the highest ground point
        for ray_pos in ray_positions.iter() {
            if let Some(hit) = spatial_query.ray_hits(
                *ray_pos, 
                ray_dir, 
                GROUND_CHECK_DISTANCE,
                1,
                true,
                filter.clone()
            ).first() {
                is_grounded = true;
                let hit_height = ray_pos.y - hit.time_of_impact + SUSPENSION_HEIGHT;
                ground_height = ground_height.max(hit_height);
            }
        }

        let mut current_velocity = velocity_query.single_mut();

        // Ground adhesion and suspension logic
        if is_grounded {
            let target_height = ground_height + SUSPENSION_HEIGHT;  // Add suspension height to target
            protagonist_transform.translation.y = protagonist_transform.translation.y.lerp(
                target_height,
                GROUND_SNAP_SPEED
            );
            current_velocity.0.y = 0.0;
        } else {
            // Apply gravity when not grounded
            current_velocity.0.y = (current_velocity.0.y - 9.81 * time.delta_seconds())
                .max(-20.0);  // Terminal velocity
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
