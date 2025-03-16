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
        commands.entity(protagonist_entity)
            .insert(Collider::cuboid(2.5, 1.5, 3.0))
            .insert(GravityScale(5.0));

        if let Ok(children) = children_query.get(protagonist_entity) {
            if let Some(spotlight_entity) = children.first() {
                commands.entity(*spotlight_entity).insert(SpotLight {
                    intensity: 10000000.0,
                    color: Color::srgb(1.0, 0.2, 0.2),
                    outer_angle: 0.6,
                    inner_angle: 0.3,
                    shadows_enabled: false,
                    range: 100.0,
                    ..default()
                });
                commands.entity(*spotlight_entity).insert(Transform::from_xyz(0.0, 20.0, 0.0)
                    .looking_at(Vec3::ZERO, Vec3::Z));
            }
        }
        asset_server.load("models/tank.glb#Scene0")
    } else {
        // Reset to original protagonist collider
        commands.entity(protagonist_entity)
            .insert(Collider::cuboid(1.0, 0.25, 1.0))
            .insert(GravityScale(3.0));

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
    mut query: Query<(Entity, &Transform, &mut Protagonist, &mut Handle<Scene>)>,
    children_query: Query<&Children>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        for (entity, transform, mut protagonist, mut scene) in query.iter_mut() {
            let new_state = !protagonist.is_driving;
            set_driving_state(
                &mut protagonist,
                &mut scene,
                &asset_server,
                new_state,
                &mut commands,
                entity,
                &children_query,
            );
        }
    }
}

pub fn driving_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut protagonist_query: Query<(Entity, &mut Transform, &Protagonist)>,
    mut velocity_query: Query<&mut LinearVelocity, With<Protagonist>>,
    mut angular_velocity_query: Query<&mut AngularVelocity, With<Protagonist>>,
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
    const FIXED_HEIGHT: f32 = 4.0;  // Fixed height for the tank

    if let Ok((protagonist_entity, mut protagonist_transform, protagonist)) = protagonist_query.get_single_mut() {
        if !protagonist.is_driving {
            return;
        }

        // Keep tank at fixed height
        protagonist_transform.translation.y = FIXED_HEIGHT;
        
        let mut current_velocity = velocity_query.single_mut();
        current_velocity.0.y = 0.0;  // No vertical movement

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
