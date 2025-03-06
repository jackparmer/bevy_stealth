use crate::components::Protagonist;
use crate::resources::{ProtagonistAnimations, PROTAGONIST_ANIMATIONS};


use bevy::{
    animation::RepeatAnimation,
    prelude::*,
};

use avian3d::prelude::*;
use std::time::Duration;

// Movement constants
const TURN_SPEED: f32 = 3.0;  // Radians per second
const MOVE_SPEED: f32 = 20.0;  // Base units per second
const RUN_SPEED: f32 = 80.0;  // Base running speed
const STRAFE_SPEED: f32 = 4.0;  // Base strafing speed
const UNDERWATER_SPEED: f32 = 80.0;  // Underwater movement speed

// Height-related constants
const HEIGHT_THRESHOLD: f32 = 100.0;
const HEIGHT_MULTIPLIER_HIGH: f32 = 2.0;
const HEIGHT_MULTIPLIER_NORMAL: f32 = 1.0;

// Swimming speed multipliers
const SWIM_SPEED_BOOST: f32 = 20.0;
const SWIM_SPEED_NORMAL: f32 = 10.0;
const SWIM_SPEED_BACKWARD_MULTIPLIER: f32 = 0.25;  // 4x slower for backward swimming
const SWIM_SPEED_VERTICAL_MULTIPLIER: f32 = 0.25;  // 4x slower for vertical swimming

// Teleportation distances
const TELEPORT_DOWN_DISTANCE: f32 = 10.0;
const TELEPORT_UP_DISTANCE: f32 = 15.0;

// Jump impulse values
const JUMP_BASE_IMPULSE: f32 = 5.0;
const JUMP_FORWARD_NORMAL: f32 = 2.0;
const JUMP_FORWARD_RUNNING: f32 = 4.0;
const JUMP_VERTICAL_RUNNING: f32 = 12.5;

// Animation transition duration
const ANIMATION_TRANSITION_MS: u64 = 250;

// Lighting values
const NIGHT_ILLUMINANCE: f32 = 10.0;
const ALARM_ILLUMINANCE: f32 = 1000.0;
const NIGHT_COLOR: Color = Color::rgb(0.2, 0.2, 0.3);
const ALARM_COLOR: Color = Color::rgb(1.0, 0.0, 0.0);

pub fn keyboard_animation_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut impulse_query: Query<&mut ExternalImpulse, With<Protagonist>>,
    mut protagonist_query: Query<(&mut Transform, &mut Protagonist)>,
    mut velocity_query: Query<&mut LinearVelocity, With<Protagonist>>,
    mut angular_velocity_query: Query<&mut AngularVelocity, With<Protagonist>>,
    mut directional_light_query: Query<&mut DirectionalLight>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    animations: Res<ProtagonistAnimations>,
) {

    if let Ok((mut protagonist_transform, mut protagonist)) = protagonist_query.get_single_mut() {
        // Manual up/down teleportation
        if keyboard_input.just_pressed(KeyCode::KeyV) {
            protagonist_transform.translation.y -= TELEPORT_DOWN_DISTANCE;
            info!("Teleported {} units down!", TELEPORT_DOWN_DISTANCE);
            return;  // Skip the rest of the input handling for this frame
        }
        if keyboard_input.just_pressed(KeyCode::KeyB) {
            protagonist_transform.translation.y += TELEPORT_UP_DISTANCE;
            info!("Teleported {} units up!", TELEPORT_UP_DISTANCE);
            return;  // Skip the rest of the input handling for this frame
        }

        // Calculate height multiplier
        let height_multiplier = if protagonist_transform.translation.y > HEIGHT_THRESHOLD { 
            HEIGHT_MULTIPLIER_HIGH 
        } else { 
            HEIGHT_MULTIPLIER_NORMAL 
        };
        let adjusted_move_speed = MOVE_SPEED * height_multiplier;
        let adjusted_run_speed = RUN_SPEED * height_multiplier;
        let adjusted_strafe_speed = STRAFE_SPEED * height_multiplier;

        // Extract only Y rotation and force upright orientation
        let (yaw, _, _) = protagonist_transform.rotation.to_euler(EulerRot::YXZ);
        protagonist_transform.rotation = Quat::from_rotation_y(yaw);

        // Handle animations separately
        for (mut player, mut transitions) in &mut animation_players {
            if protagonist.is_driving {
                continue; // Skip animation handling when driving
            }

            // Skip if climbing or driving
            if protagonist.is_climbing || protagonist.was_climbing || protagonist.is_driving {
                continue;
            }

            // Modify the movement logic to handle underwater state
            if protagonist.is_swimming {
                // Handle underwater movement
                let mut movement = Vec3::ZERO;
                let base_swimming_speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
                    UNDERWATER_SPEED * SWIM_SPEED_BOOST
                } else {
                    UNDERWATER_SPEED * SWIM_SPEED_NORMAL
                };
                
                // Forward/Backward movement with different speeds
                if keyboard_input.pressed(KeyCode::KeyW) {
                    movement += protagonist_transform.forward().as_vec3();
                }
                if keyboard_input.pressed(KeyCode::KeyS) {
                    movement -= protagonist_transform.forward().as_vec3() * SWIM_SPEED_BACKWARD_MULTIPLIER;
                }
                
                // Up/Down movement with reduced speed
                if keyboard_input.pressed(KeyCode::Space) {
                    movement.y += SWIM_SPEED_VERTICAL_MULTIPLIER;
                }
                if keyboard_input.pressed(KeyCode::ShiftLeft) {
                    movement.y -= SWIM_SPEED_VERTICAL_MULTIPLIER;
                }

                // Handle rotation while swimming
                let target_rotation = if keyboard_input.pressed(KeyCode::KeyA) {
                    TURN_SPEED
                } else if keyboard_input.pressed(KeyCode::KeyD) {
                    -TURN_SPEED
                } else {
                    0.0
                };

                // Apply rotation through angular velocity
                for mut angular_velocity in angular_velocity_query.iter_mut() {
                    angular_velocity.0 = Vec3::new(0.0, target_rotation, 0.0);
                }

                if movement != Vec3::ZERO {
                    if let Some(swim) = PROTAGONIST_ANIMATIONS.get("SWIM") {
                        // Only start the swimming animation if we're not already playing it
                        if !player.is_playing_animation(animations.animations[*swim]) {
                            player.stop_all();  // Stop other animations first
                            transitions
                                .play(
                                    &mut player,
                                    animations.animations[*swim],
                                    Duration::from_millis(ANIMATION_TRANSITION_MS),
                                )
                                .set_speed(2.0)
                                .set_repeat(RepeatAnimation::Forever);
                        }
                        
                        for mut linear_velocity in velocity_query.iter_mut() {
                            linear_velocity.0 = movement.normalize() * base_swimming_speed;
                        }
                    }
                } else {
                    // Only start the tread animation if we're not already playing it
                    if let Some(tread) = PROTAGONIST_ANIMATIONS.get("TREAD") {
                        if !player.is_playing_animation(animations.animations[*tread]) {
                            player.stop_all();  // Stop other animations first
                            transitions
                                .play(
                                    &mut player,
                                    animations.animations[*tread],
                                    Duration::from_millis(ANIMATION_TRANSITION_MS),
                                )
                                .set_repeat(RepeatAnimation::Forever);
                        }
                    }
                    
                    for mut linear_velocity in velocity_query.iter_mut() {
                        linear_velocity.0 = Vec3::ZERO;
                    }
                }
                
                continue; // Skip normal movement handling when underwater
            }

            // Check if falling first
            if protagonist.is_falling && 
               !protagonist.is_climbing &&
               !protagonist.is_swimming &&  // Add swimming check
               !keyboard_input.pressed(KeyCode::KeyV) &&
               !keyboard_input.pressed(KeyCode::KeyB)
            {
                if let Some(fly) = PROTAGONIST_ANIMATIONS.get("FLY") {
                    if !player.is_playing_animation(animations.animations[*fly]) {
                        transitions
                            .play(
                                &mut player,
                                animations.animations[*fly],
                                Duration::from_millis(ANIMATION_TRANSITION_MS),
                            )
                            .set_repeat(RepeatAnimation::Never);
                    }
                }
                
                // Add rotation handling while falling
                let target_rotation = if keyboard_input.pressed(KeyCode::KeyA) {
                    TURN_SPEED
                } else if keyboard_input.pressed(KeyCode::KeyD) {
                    -TURN_SPEED
                } else {
                    0.0
                };

                // Apply rotation through angular velocity
                for mut angular_velocity in angular_velocity_query.iter_mut() {
                    angular_velocity.0 = Vec3::new(0.0, target_rotation, 0.0);
                }
                
                continue; // Skip other animations while falling
            } else {
                // Handle normal movement
                if !protagonist.is_swimming && !protagonist.is_falling {
                    // Handle movement speed based on driving state
                    let movement_speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
                        adjusted_run_speed
                    } else {
                        adjusted_move_speed
                    };

                    // Apply movement without animations if driving
                    if protagonist.is_driving {
                        if keyboard_input.pressed(KeyCode::KeyW) {
                            for mut linear_velocity in velocity_query.iter_mut() {
                                linear_velocity.0 = protagonist_transform.forward() * movement_speed;
                            }
                        } else if keyboard_input.pressed(KeyCode::KeyS) {
                            for mut linear_velocity in velocity_query.iter_mut() {
                                linear_velocity.0 = -protagonist_transform.forward() * movement_speed;
                            }
                        }
                        continue; // Skip animation handling when driving
                    }

                    // Handle strafing
                    if keyboard_input.pressed(KeyCode::KeyE) {
                        let strafe_anim = if keyboard_input.pressed(KeyCode::ShiftLeft) {
                            "STRAFE_JOG_LEFT"
                        } else {
                            "STRAFE_RIGHT"
                        };
                        
                        if let Some(strafe) = PROTAGONIST_ANIMATIONS.get(strafe_anim) {
                            if !player.is_playing_animation(animations.animations[*strafe]) {
                                let _animation = transitions
                                    .play(
                                        &mut player,
                                        animations.animations[*strafe],
                                        Duration::from_millis(ANIMATION_TRANSITION_MS),
                                    )
                                    .set_repeat(RepeatAnimation::Forever);                                
                            }
                            
                            let movement_speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
                                adjusted_strafe_speed * 2.0
                            } else {
                                adjusted_strafe_speed
                            };
                            
                            for mut linear_velocity in velocity_query.iter_mut() {
                                linear_velocity.0 = protagonist_transform.right() * movement_speed;
                            }
                            continue; // Skip other movement handling
                        }
                    } else if keyboard_input.pressed(KeyCode::KeyQ) {
                        let strafe_anim = if keyboard_input.pressed(KeyCode::ShiftLeft) {
                            "STRAFE_JOG_RIGHT"
                        } else {
                            "STRAFE_LEFT"
                        };
                        
                        if let Some(strafe) = PROTAGONIST_ANIMATIONS.get(strafe_anim) {
                            if !player.is_playing_animation(animations.animations[*strafe]) {
                                let _animation = transitions
                                    .play(
                                        &mut player,
                                        animations.animations[*strafe],
                                        Duration::from_millis(ANIMATION_TRANSITION_MS),
                                    )
                                    .set_repeat(RepeatAnimation::Forever);
                            }
                            
                            let movement_speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
                                adjusted_strafe_speed * 2.0
                            } else {
                                adjusted_strafe_speed
                            };
                            
                            for mut linear_velocity in velocity_query.iter_mut() {
                                linear_velocity.0 = -protagonist_transform.right() * movement_speed;
                            }
                            continue; // Skip other movement handling
                        }
                    }
                }

                if keyboard_input.pressed(KeyCode::KeyW) {
                    if let Some(run) = PROTAGONIST_ANIMATIONS.get("ADVANCE") {
                        if !player.is_playing_animation(animations.animations[*run]) {
                            let animation = transitions
                                .play(
                                    &mut player,
                                    animations.animations[*run],
                                    Duration::from_millis(ANIMATION_TRANSITION_MS),
                                )
                                .set_repeat(RepeatAnimation::Forever);
                            
                            // Set animation speed based on movement speed
                            if keyboard_input.pressed(KeyCode::ShiftLeft) {
                                animation.set_speed(2.0);
                            }
                        }
                        let movement_speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
                            adjusted_run_speed
                        } else {
                            adjusted_move_speed
                        };

                        for mut linear_velocity in velocity_query.iter_mut() {
                            linear_velocity.0 = protagonist_transform.forward() * movement_speed;
                        }
                    }
                } else if keyboard_input.pressed(KeyCode::KeyS) {
                    if let Some(walk_backward) = PROTAGONIST_ANIMATIONS.get("WALK_BACK") {
                        if !player.is_playing_animation(animations.animations[*walk_backward]) {
                            let animation = transitions
                                .play(
                                    &mut player,
                                    animations.animations[*walk_backward],
                                    Duration::from_millis(ANIMATION_TRANSITION_MS),
                                )
                                .set_repeat(RepeatAnimation::Forever);
                            
                            // Set animation speed based on movement speed
                            if keyboard_input.pressed(KeyCode::ShiftLeft) {
                                animation.set_speed(2.0);
                            }
                        }
                        let movement_speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
                            adjusted_run_speed
                        } else {
                            adjusted_move_speed
                        };
                        for mut linear_velocity in velocity_query.iter_mut() {
                            linear_velocity.0 = -protagonist_transform.forward() * movement_speed;
                        }
                    }
                } else if !keyboard_input.pressed(KeyCode::KeyW) && !keyboard_input.pressed(KeyCode::KeyS) {
                    if let Some(idle) = PROTAGONIST_ANIMATIONS.get("CROUCH") {
                        if !player.is_playing_animation(animations.animations[*idle]) {
                            transitions
                                .play(
                                    &mut player,
                                    animations.animations[*idle],
                                    Duration::from_millis(ANIMATION_TRANSITION_MS),
                                )
                                .set_repeat(RepeatAnimation::Never);
                        }
                    }
                }

                // Handle rotation
                let target_rotation = if keyboard_input.pressed(KeyCode::KeyA) {
                    TURN_SPEED
                } else if keyboard_input.pressed(KeyCode::KeyD) {
                    -TURN_SPEED
                } else {
                    0.0
                };

                if !protagonist.is_climbing {
                    // Apply rotation through angular velocity with strict control
                    for mut angular_velocity in angular_velocity_query.iter_mut() {
                        // Only allow Y-axis rotation and immediately stop when no input
                        angular_velocity.0 = Vec3::ZERO;  // Reset all velocity first
                        if target_rotation != 0.0 {
                            angular_velocity.0.y = target_rotation;
                        }
                    }
                }
            }

            // Handle jumping
            if keyboard_input.just_pressed(KeyCode::Space) 
                && !protagonist.is_swimming
                && !protagonist.is_climbing
                && !protagonist.is_falling
            {
                if let Some(jump) = PROTAGONIST_ANIMATIONS.get("FLY") {
                    transitions
                        .play(
                            &mut player,
                            animations.animations[*jump],
                            Duration::from_millis(ANIMATION_TRANSITION_MS),
                        )
                        .set_repeat(RepeatAnimation::Never);

                    for mut impulse in impulse_query.iter_mut() {
                        let mut jump_impulse = Vec3::new(0.0, JUMP_BASE_IMPULSE, 0.0);
                        
                        // Add forward impulse if W is pressed
                        if keyboard_input.pressed(KeyCode::KeyW) {
                            let forward_strength = if keyboard_input.pressed(KeyCode::ShiftLeft) {
                                JUMP_FORWARD_RUNNING
                            } else {
                                JUMP_FORWARD_NORMAL
                            };
                            jump_impulse += protagonist_transform.forward() * forward_strength;
                            
                            // Increase vertical impulse for running leap
                            if keyboard_input.pressed(KeyCode::ShiftLeft) {
                                jump_impulse.y = JUMP_VERTICAL_RUNNING;
                            }
                        }
                        
                        impulse.apply_impulse(jump_impulse);
                    }

                    break;
                }
            }

            // Handle other special actions
            if keyboard_input.just_pressed(KeyCode::KeyC) {
                info!("C key pressed");
                // ... rest of charge placement code ...
            }

            // Toggle lighting with K for night and L for alarm
            if keyboard_input.just_pressed(KeyCode::KeyK) {
                for mut light in directional_light_query.iter_mut() {
                    // Switch to dark night mode
                    light.illuminance = NIGHT_ILLUMINANCE;
                    light.color = NIGHT_COLOR;
                }
            }

            if keyboard_input.just_pressed(KeyCode::KeyL) {
                for mut light in directional_light_query.iter_mut() {
                    // Switch to red alarm lights
                    light.illuminance = ALARM_ILLUMINANCE;
                    light.color = ALARM_COLOR;
                }
            }

            // Replace the Tab animation cycling with camera toggle
            if keyboard_input.just_pressed(KeyCode::Tab) {
                protagonist.is_birds_eye = !protagonist.is_birds_eye;
                info!("Camera view: {}", if protagonist.is_birds_eye { "Birds-eye" } else { "Normal" });
            }
        }        
    }
}
