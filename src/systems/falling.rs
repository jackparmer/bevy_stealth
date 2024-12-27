use bevy::prelude::*;
use bevy::animation::RepeatAnimation;
use avian3d::prelude::*;
use crate::components::Protagonist;
use crate::resources::{Animations, SCENES};
use std::time::Duration;

pub fn check_falling(
    mut protagonist_query: Query<(Entity, &mut Protagonist, &Transform, &LinearVelocity)>,
    spatial_query: SpatialQuery,
    mut gizmos: Gizmos,
    mut ambient_light: ResMut<AmbientLight>,
) {
    for (entity, mut protagonist, transform, velocity) in protagonist_query.iter_mut() {
        // Don't set falling state if swimming or climbing
        if protagonist.is_swimming || protagonist.is_climbing {
            protagonist.is_falling = false;
            continue;
        }

        // Add overhead raycast
        let overhead_ray_pos = transform.translation;
        let overhead_ray_dir = Dir3::Y;
        let overhead_max_distance = 100.0;
        let overhead_filter = SpatialQueryFilter::from_excluded_entities([entity]);
        
        let overhead_hits = spatial_query.ray_hits(
            overhead_ray_pos,
            overhead_ray_dir,
            overhead_max_distance,
            1,
            true,
            overhead_filter
        );

        // Adjust ambient light if there's something above
        if !overhead_hits.is_empty() {
            ambient_light.brightness = 2000.0; // Reduced brightness when under cover
        } else {
            ambient_light.brightness = 6000.0; // Normal brightness in open areas
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
        
        protagonist.is_falling = !is_grounded;

        // Draw the ray
        let ray_end = ray_pos + (ray_dir.as_vec3() * max_distance);
        if is_grounded {
            gizmos.line(ray_pos, ray_end, Color::rgb(0.0, 1.0, 0.0));  // Green
            
            // Draw hit point if there is one
            if let Some(hit) = hits.first() {
                let hit_point = ray_pos + (ray_dir.as_vec3() * hit.time_of_impact);
                gizmos.sphere(hit_point, Quat::IDENTITY, 0.1, Color::rgb(1.0, 1.0, 0.0));  // Yellow
                
                // Draw normal at hit point
                let normal_end = hit_point + (hit.normal * 0.3);
                gizmos.line(hit_point, normal_end, Color::rgb(0.0, 0.0, 1.0));  // Blue
            }
        } else {
            gizmos.line(ray_pos, ray_end, Color::rgb(1.0, 0.0, 0.0));  // Red
        }
    }
}

pub fn handle_falling_animation(
    protagonist_query: Query<(&Protagonist, &Children)>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    mut velocity_query: Query<&mut LinearVelocity>,
    animations: Res<Animations>,
) {
    for (protagonist, _) in protagonist_query.iter() {
        if protagonist.is_falling {
            for (mut player, mut transitions) in animation_players.iter_mut() {
                if let Some(fly) = SCENES.get("FLY") {
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
