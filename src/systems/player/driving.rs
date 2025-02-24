use bevy::prelude::*;
use avian3d::prelude::*;

use crate::components::Protagonist;

pub fn toggle_driving(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &mut Protagonist, &mut Handle<Scene>, &mut Transform)>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Protagonist>)>,
    asset_server: Res<AssetServer>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        for (_, mut protagonist, mut scene, mut transform) in query.iter_mut() {
            // Toggle driving state
            protagonist.is_driving = !protagonist.is_driving;
            
            // Update the model and camera based on driving state
            *scene = if protagonist.is_driving {
                transform.scale = Vec3::splat(3.0);
                // Adjust camera position for tank view
                if let Ok(mut camera_transform) = camera_query.get_single_mut() {
                    camera_transform.translation = Vec3::new(0.0, 15.0, 25.0); // Higher and further back
                }
                asset_server.load("models/KB03-apc.glb#Scene0")
            } else {
                transform.scale = Vec3::ONE;
                // Reset camera position for protagonist view
                if let Ok(mut camera_transform) = camera_query.get_single_mut() {
                    camera_transform.translation = Vec3::new(0.0, 5.0, 8.0); // Normal following distance
                }
                asset_server.load("models/Protagonist.glb#Scene0")
            };
        }
    }
}

pub fn driving_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut protagonist_query: Query<(&mut Transform, &Protagonist)>,
    mut velocity_query: Query<&mut LinearVelocity, With<Protagonist>>,
    mut angular_velocity_query: Query<&mut AngularVelocity, With<Protagonist>>,
    time: Res<Time>,
) {
    // Driving parameters
    const TURN_SPEED: f32 = 3.0;
    const MAX_DRIVING_SPEED: f32 = 300.0;
    const MIN_DRIVING_SPEED: f32 = -150.0;
    const ACCELERATION: f32 = 50.0;
    const BRAKE_FORCE: f32 = 300.0;
    const DRIFT_FACTOR: f32 = 0.85;
    const TURN_SENSITIVITY: f32 = 2.0;
    const SPEED_TURN_FACTOR: f32 = 0.3;

    if let Ok((mut protagonist_transform, protagonist)) = protagonist_query.get_single_mut() {
        if !protagonist.is_driving {
            return;
        }

        // Extract only Y rotation and force upright orientation
        let (yaw, _, _) = protagonist_transform.rotation.to_euler(EulerRot::YXZ);
        protagonist_transform.rotation = Quat::from_rotation_y(yaw);

        let mut current_velocity = velocity_query.single_mut();
        let dt = time.delta_seconds();
        
        // Decompose velocity into forward and lateral components
        let forward_dir = protagonist_transform.left().as_vec3();
        let right_dir = protagonist_transform.forward().as_vec3();
        let current_speed = current_velocity.0.dot(forward_dir);
        let lateral_speed = current_velocity.0.dot(right_dir);

        // Calculate acceleration based on input
        let mut acceleration_amount = 0.0;
        if keyboard_input.pressed(KeyCode::KeyW) {
            acceleration_amount = ACCELERATION * dt;
        } else if keyboard_input.pressed(KeyCode::KeyS) {
            // Apply brakes if moving forward, otherwise accelerate backward
            if current_speed > 0.0 {
                acceleration_amount = -BRAKE_FORCE * dt;
            } else {
                acceleration_amount = -ACCELERATION * 0.5 * dt;
            }
        } else if current_speed.abs() > 1.0 {
            // Natural deceleration when no input
            acceleration_amount = -current_speed.signum() * BRAKE_FORCE * 0.5 * dt;
        }

        // Apply acceleration and clamp speed
        let new_speed = (current_speed + acceleration_amount)
            .clamp(MIN_DRIVING_SPEED, MAX_DRIVING_SPEED);

        // Calculate turning based on speed
        let turn_amount = if keyboard_input.pressed(KeyCode::KeyA) {
            TURN_SPEED
        } else if keyboard_input.pressed(KeyCode::KeyD) {
            -TURN_SPEED
        } else {
            0.0
        };

        // Adjust turn rate based on speed
        let speed_ratio = (current_speed.abs() / MAX_DRIVING_SPEED).powf(SPEED_TURN_FACTOR);
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
