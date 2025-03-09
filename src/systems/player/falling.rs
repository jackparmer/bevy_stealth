use bevy::prelude::*;
use bevy::animation::RepeatAnimation;
use avian3d::prelude::*;
use crate::components::Protagonist;
use crate::resources::{ProtagonistAnimations, PROTAGONIST_ANIMATIONS};
use std::time::Duration;

pub fn check_falling(
    mut commands: Commands,
    mut protagonist_query: Query<(Entity, &mut Protagonist, &Transform, &LinearVelocity, &Children)>,
    mut spotlight_query: Query<&mut SpotLight>,
    spatial_query: SpatialQuery,
    mut gizmos: Gizmos,
    mut ambient_light: ResMut<AmbientLight>,
) {
    for (entity, mut protagonist, transform, _velocity, children) in protagonist_query.iter_mut() {
        // Don't check falling for vehicles at all
        if protagonist.is_driving {
            continue;  // Skip the entire check for vehicles
        }

        // Don't set falling state if swimming or climbing
        if protagonist.is_swimming || protagonist.is_climbing {
            protagonist.is_falling = false;
            continue;
        }

        // Skip falling state update if driving, but continue with ground check
        if protagonist.is_driving {
            protagonist.is_falling = false;
        }

        // Add overhead raycast
        let overhead_ray_pos = transform.translation;
        let overhead_ray_dir = Dir3::Y;
        let overhead_max_distance = 2000.0;
        let overhead_filter = SpatialQueryFilter::from_excluded_entities([entity]);
        
        let overhead_hits = spatial_query.ray_hits(
            overhead_ray_pos,
            overhead_ray_dir,
            overhead_max_distance,
            1,
            true,
            overhead_filter
        );

        // Update is_outside status and adjust ambient light based on overhead hits
        protagonist.is_outside = overhead_hits.is_empty();
        if !protagonist.is_outside {
            ambient_light.brightness = 200.0; // Reduced brightness when under cover
            for child in children {
                if let Ok(mut spotlight) = spotlight_query.get_mut(*child) {
                    spotlight.intensity = 0.0;
                }
            }
        } else {
            ambient_light.brightness = 400.0; // Normal brightness in open areas
            // Turn on spotlight when outside
            for child in children {
                if let Ok(mut spotlight) = spotlight_query.get_mut(*child) {
                    spotlight.intensity = 5000000.0;
                }
            }
        }

        let ray_pos = transform.translation + Vec3::new(0.0, 0.5, 0.0);
        let ray_dir = Dir3::NEG_Y;
        let max_distance = 2.0;
        let solid = true;
        let filter = SpatialQueryFilter::from_excluded_entities([entity]);
        
        let hits = spatial_query.ray_hits(
            ray_pos, 
            ray_dir, 
            max_distance,
            1,
            solid,
            filter
        );

        let is_grounded = !hits.is_empty();
        
        // Reset jumping state when landing
        if is_grounded && (protagonist.is_jumping || protagonist.is_falling) {
            protagonist.is_jumping = false;
            protagonist.is_falling = false;
        } else if !is_grounded && !protagonist.is_jumping {
            protagonist.is_falling = true;
        }

        // Draw the ray
        // let ray_end = ray_pos + (ray_dir.as_vec3() * max_distance);
        // if is_grounded {
        //     gizmos.line(ray_pos, ray_end, Color::srgb(0.0, 1.0, 0.0));  // Green
            
        //     // Draw hit point if there is one
        //     if let Some(hit) = hits.first() {
        //         let hit_point = ray_pos + (ray_dir.as_vec3() * hit.time_of_impact);
        //         gizmos.sphere(hit_point, Quat::IDENTITY, 0.1, Color::srgb(1.0, 1.0, 0.0));  // Yellow
                
        //         // Draw normal at hit point
        //         let normal_end = hit_point + (hit.normal * 0.3);
        //         gizmos.line(hit_point, normal_end, Color::srgb(0.0, 0.0, 1.0));  // Blue
        //     }
        // } else {
        //     gizmos.line(ray_pos, ray_end, Color::srgb(1.0, 0.0, 0.0));  // Red
        // }
    }
}

pub fn handle_falling_animation(
    protagonist_query: Query<(&Protagonist, &Children)>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    _velocity_query: Query<&mut LinearVelocity>,
    animations: Res<ProtagonistAnimations>,
) {
    for (protagonist, _) in protagonist_query.iter() {
        // Skip falling animation if driving
        if (protagonist.is_falling || protagonist.is_jumping) && !protagonist.is_driving {
            for (mut player, mut transitions) in animation_players.iter_mut() {
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
            }
        }
    }
}
