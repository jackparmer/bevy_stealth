use avian3d::prelude::*;
use std::time::Duration;
use bevy::{
    animation::RepeatAnimation,
    prelude::*,
};

use crate::components::Protagonist;
use crate::resources::{ProtagonistAnimations, PROTAGONIST_ANIMATIONS};

pub fn handle_climbing(
    mut collision_started: EventReader<CollisionStarted>,
    mut collision_ended: EventReader<CollisionEnded>,
    mut protagonist_query: Query<&mut Protagonist>,
    name_query: Query<&Name>,
    _time: Res<Time>,
) {
    // Handle collision start events
    for collision in collision_started.read() {
        // Check if one entity is the protagonist and the other is the ladder sensor
        if let Ok(mut protagonist) = protagonist_query.get_mut(collision.0) {
            if name_query.get(collision.1).map_or(false, |name| name.as_str() == "LadderSensor") {
                // Only set climbing if we haven't just toggled it
                if !protagonist.was_climbing {
                    protagonist.is_climbing = true;
                    protagonist.was_climbing = false;
                    protagonist.is_falling = false;
                    println!("Started climbing: Protagonist entered ladder sensor zone");
                }
            }
        } else if let Ok(mut protagonist) = protagonist_query.get_mut(collision.1) {
            if name_query.get(collision.0).map_or(false, |name| name.as_str() == "LadderSensor") {
                if !protagonist.was_climbing {
                    protagonist.is_climbing = true;
                    protagonist.was_climbing = false;
                    protagonist.is_falling = false;
                    println!("Started climbing: Protagonist entered ladder sensor zone");
                }
            }
        }
    }

    // Handle collision end events
    for collision in collision_ended.read() {
        // Reset climbing state when leaving the ladder sensor
        if let Ok(mut protagonist) = protagonist_query.get_mut(collision.0) {
            if name_query.get(collision.1).map_or(false, |name| name.as_str() == "LadderSensor") {
                protagonist.is_climbing = false;
                println!("Stopped climbing: Protagonist left ladder sensor zone");
            }
        } else if let Ok(mut protagonist) = protagonist_query.get_mut(collision.1) {
            if name_query.get(collision.0).map_or(false, |name| name.as_str() == "LadderSensor") {
                protagonist.is_climbing = false;
                println!("Stopped climbing: Protagonist left ladder sensor zone");
            }
        }
    }
}

pub fn climbing_keyboard_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut protagonist_query: Query<(&Transform, &mut Protagonist)>,
    mut velocity_query: Query<(&mut LinearVelocity, &mut AngularVelocity), With<Protagonist>>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Protagonist>)>,
    mut spotlight_query: Query<&mut Transform, (With<SpotLight>, Without<Camera>, Without<Protagonist>)>,
    animations: Res<ProtagonistAnimations>,
    time: Res<Time>,
) {
    let (protagonist_transform, mut protagonist) = match protagonist_query.get_single_mut() {
        Ok(p) => p,
        Err(_) => return,
    };

    // Update spotlight position when climbing
    if let Ok(mut spotlight_transform) = spotlight_query.get_single_mut() {
        if protagonist.is_climbing {
            // Position light behind and slightly above player when climbing
            spotlight_transform.translation = protagonist_transform.translation + Vec3::new(0.0, 2.0, 4.0);
            spotlight_transform.look_at(protagonist_transform.translation, Vec3::Y);
        } else {
            // Reset to original position when not climbing
            spotlight_transform.translation = Vec3::new(0.0, 15.0, 2.0);
            spotlight_transform.look_at(Vec3::new(0.0, 0.0, -10.0), Vec3::Y);
        }
    }

    // Add space key detection to stop climbing
    if protagonist.is_climbing && keyboard_input.just_pressed(KeyCode::Space) {
        protagonist.is_climbing = false;
        protagonist.was_climbing = true;
        return;  // Exit early since we're no longer climbing
    }

    if protagonist.is_climbing {
        protagonist.is_falling = false;

        // Add speed multiplier based on shift key
        let speed_multiplier = if keyboard_input.pressed(KeyCode::ShiftLeft) || 
                                keyboard_input.pressed(KeyCode::ShiftRight) { 2.0 } else { 1.0 };

        // Calculate movement speeds with multiplier
        let climb_speed = if keyboard_input.pressed(KeyCode::KeyW) { 2.0 * speed_multiplier } 
                        else if keyboard_input.pressed(KeyCode::KeyS) { -2.0 * speed_multiplier }
                        else { 0.0 };
        
        let side_speed = if keyboard_input.pressed(KeyCode::KeyQ) { 2.0 * speed_multiplier }
                        else if keyboard_input.pressed(KeyCode::KeyE) { -2.0 * speed_multiplier }
                        else { 0.0 };

        // Update velocity with multiplied speeds
        for (mut linear_velocity, mut angular_velocity) in velocity_query.iter_mut() {
            linear_velocity.0 = Vec3::new(0.0, climb_speed * 2.5, side_speed * 2.5);
            angular_velocity.0 = Vec3::ZERO;
        }

        for (mut player, mut transitions) in &mut animation_players {
            if protagonist.is_climbing {
                let climb_speed = if keyboard_input.pressed(KeyCode::KeyW) { 2.0 * speed_multiplier } 
                                else if keyboard_input.pressed(KeyCode::KeyS) { -2.0 * speed_multiplier }
                                else { 0.0 };
                
                if let Some(climb) = PROTAGONIST_ANIMATIONS.get("CLIMB") {
                    let anim_handle = animations.animations[*climb];
                    if climb_speed != 0.0 {
                        // Resume or start animation when moving
                        if !player.is_playing_animation(anim_handle) {
                            transitions
                                .play(
                                    &mut player,
                                    anim_handle,
                                    Duration::from_millis(250),
                                )
                                .set_speed(climb_speed)
                                .set_repeat(RepeatAnimation::Forever);
                        }
                    }
                }
            }
        }
    }

    // Handle camera rotation while climbing
    if let Ok(mut camera_transform) = camera_query.get_single_mut() {
        let rotation_speed = 2.0;
        
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::KeyD) {
            let rotation = Quat::from_rotation_y(
                if keyboard_input.pressed(KeyCode::KeyA) { rotation_speed } 
                else { -rotation_speed } 
                * time.delta_seconds()
            );
            
            // Remove the distance multiplier
            let relative_pos = camera_transform.translation - protagonist_transform.translation;
            
            let rotated_pos = rotation * relative_pos;
            camera_transform.translation = protagonist_transform.translation + rotated_pos;
            camera_transform.look_at(protagonist_transform.translation, Vec3::Y);
        }
    }
}

pub fn check_ladder_presence(
    mut protagonist_query: Query<(&Transform, &mut Protagonist)>,
    spatial_query: SpatialQuery,
) {
    for (transform, mut protagonist) in protagonist_query.iter_mut() {
        if protagonist.is_climbing {
            // Cast a shorter ray forward from the protagonist
            let ray_pos = transform.translation;
            let ray_dir = transform.forward();
            let max_distance = 0.5; // Reduced from 1.0 to make it easier to exit
            let filter = SpatialQueryFilter::default();

            let hits = spatial_query.ray_hits(
                ray_pos,
                ray_dir,
                max_distance,
                1,
                true,
                filter
            );

            // If there's nothing in front, stop climbing
            if hits.is_empty() {
                protagonist.is_climbing = false;
                protagonist.was_climbing = true;
            }
        }
    }
}

pub fn handle_ladder_top(
    mut collision_started: EventReader<CollisionStarted>,
    mut protagonist_query: Query<(&mut Transform, &mut Protagonist)>,
    name_query: Query<&Name>,
) {
    for collision in collision_started.read() {
        if let Ok((mut transform, mut protagonist)) = protagonist_query.get_mut(collision.0) {
            if name_query.get(collision.1).map_or(false, |name| name.as_str() == "LadderTopSensor") {
                if protagonist.is_climbing {
                    let forward = transform.forward().as_vec3();
                    transform.translation.y += 2.0;
                    transform.translation += forward * 2.0;
                    protagonist.is_climbing = false;
                }
            }
        } else if let Ok((mut transform, mut protagonist)) = protagonist_query.get_mut(collision.1) {
            if name_query.get(collision.0).map_or(false, |name| name.as_str() == "LadderTopSensor") {
                if protagonist.is_climbing {
                    let forward = transform.forward().as_vec3();
                    transform.translation.y += 2.0;
                    transform.translation += forward * 2.0;
                    protagonist.is_climbing = false;
                }
            }
        }
    }
}
