use crate::components::Protagonist;
use crate::resources::{ProtagonistAnimations, PROTAGONIST_ANIMATIONS};


use bevy::{
    animation::RepeatAnimation,
    prelude::*,
};

use avian3d::prelude::*;
use std::time::Duration;

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

    let turn_speed = 3.0; // Fixed rotation speed (radians per second)
    let move_speed = 5.0; // Base units per second
    let run_speed = 30.0; // Base running speed
    let strafe_speed = 4.0; // Base strafing speed
    let underwater_speed = 30.0; // Underwater movement speed

    if let Ok((mut protagonist_transform, mut protagonist)) = protagonist_query.get_single_mut() {
        // Calculate height multiplier
        let height_multiplier = if protagonist_transform.translation.y > 100.0 { 2.0 } else { 1.0 };
        let adjusted_move_speed = move_speed * height_multiplier;
        let adjusted_run_speed = run_speed * height_multiplier;
        let adjusted_strafe_speed = strafe_speed * height_multiplier;

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
                let swimming_speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
                    underwater_speed * 20.0
                } else {
                    underwater_speed * 10.0  
                };
                
                // Forward/Backward movement
                if keyboard_input.pressed(KeyCode::KeyW) {
                    movement += protagonist_transform.forward().as_vec3();
                }
                if keyboard_input.pressed(KeyCode::KeyS) {
                    movement -= protagonist_transform.forward().as_vec3();
                }
                
                // Up/Down movement
                if keyboard_input.pressed(KeyCode::Space) {
                    movement.y += 1.0;
                }
                if keyboard_input.pressed(KeyCode::ShiftLeft) {
                    movement.y -= 1.0;
                }

                // Manual up/down teleportation
                if keyboard_input.just_pressed(KeyCode::KeyV) {
                    protagonist_transform.translation.y -= 10.0;
                    info!("Teleported 10 units down!");
                    // Don't interrupt the current animation
                    continue;  // Skip the rest of the animation logic for this frame
                }
                if keyboard_input.just_pressed(KeyCode::KeyB) {
                    protagonist_transform.translation.y += 15.0;
                    info!("Teleported 15 units up!");
                    // Don't interrupt the current animation
                    continue;  // Skip the rest of the animation logic for this frame
                }

                // Handle rotation while swimming
                let target_rotation = if keyboard_input.pressed(KeyCode::KeyA) {
                    turn_speed
                } else if keyboard_input.pressed(KeyCode::KeyD) {
                    -turn_speed
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
                                    Duration::from_millis(250),
                                )
                                .set_speed(2.0)
                                .set_repeat(RepeatAnimation::Forever);
                        }
                        
                        for mut linear_velocity in velocity_query.iter_mut() {
                            linear_velocity.0 = movement.normalize() * swimming_speed;
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
                                    Duration::from_millis(250),
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
                                Duration::from_millis(250),
                            )
                            .set_repeat(RepeatAnimation::Never);
                    }
                }
                
                // Add rotation handling while falling
                let target_rotation = if keyboard_input.pressed(KeyCode::KeyA) {
                    turn_speed
                } else if keyboard_input.pressed(KeyCode::KeyD) {
                    -turn_speed
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
                                        Duration::from_millis(250),
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
                                        Duration::from_millis(250),
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
                                    Duration::from_millis(250),
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
                                    Duration::from_millis(250),
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
                                    Duration::from_millis(250),
                                )
                                .set_repeat(RepeatAnimation::Never);
                        }
                    }
                }

                // Handle rotation
                let target_rotation = if keyboard_input.pressed(KeyCode::KeyA) {
                    turn_speed
                } else if keyboard_input.pressed(KeyCode::KeyD) {
                    -turn_speed
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
                            Duration::from_millis(250),
                        )
                        .set_repeat(RepeatAnimation::Never);

                    for mut impulse in impulse_query.iter_mut() {
                        let mut jump_impulse = Vec3::new(0.0, 5.0, 0.0);
                        
                        // Add forward impulse if W is pressed
                        if keyboard_input.pressed(KeyCode::KeyW) {
                            let forward_strength = if keyboard_input.pressed(KeyCode::ShiftLeft) {
                                4.0 // Stronger forward leap when running
                            } else {
                                2.0 // Normal forward leap
                            };
                            jump_impulse += protagonist_transform.forward() * forward_strength;
                            
                            // Increase vertical impulse for running leap
                            if keyboard_input.pressed(KeyCode::ShiftLeft) {
                                jump_impulse.y = 12.5;
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

            // Teleport the character 10 units down when V is pressed
            if keyboard_input.just_pressed(KeyCode::KeyV) {
                protagonist_transform.translation.y -= 10.0;
                info!("Teleported 10 units down!");
            }

            // Teleport the character 15 units up when B is pressed
            if keyboard_input.just_pressed(KeyCode::KeyB) {
                protagonist_transform.translation.y += 15.0;
                info!("Teleported 15 units up!");
            }

            // Toggle lighting with K for night and L for alarm
            if keyboard_input.just_pressed(KeyCode::KeyK) {
                for mut light in directional_light_query.iter_mut() {
                    // Switch to dark night mode
                    light.illuminance = 10.0;
                    light.color = Color::srgb(0.2, 0.2, 0.3); // Very dark blue night
                }
            }

            if keyboard_input.just_pressed(KeyCode::KeyL) {
                for mut light in directional_light_query.iter_mut() {
                    // Switch to red alarm lights
                    light.illuminance = 1000.0;
                    light.color = Color::srgb(1.0, 0.0, 0.0); // Bright red alarm light
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
