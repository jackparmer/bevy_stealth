use bevy::prelude::*;
use crate::components::{Protagonist, HighAltitudeIndicator};

pub fn rotate_camera(
    time: Res<Time>,
    protagonist_query: Query<(Entity, &Transform, &Protagonist, Option<&Children>)>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<Protagonist>)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    high_altitude_indicator_query: Query<Entity, With<HighAltitudeIndicator>>,
) {
    if let Ok((protagonist_entity, protagonist_transform, protagonist, children)) = protagonist_query.get_single() {
        let protagonist_position = protagonist_transform.translation;
        let protagonist_rotation = protagonist_transform.rotation;

        // Handle high altitude indicator
        let is_high_altitude = protagonist_position.y > 100.0;
        let has_indicator = high_altitude_indicator_query.iter().next().is_some();

        if is_high_altitude && 
        !has_indicator && 
        !protagonist.is_falling && 
        !protagonist.is_dirigible &&
        !protagonist.is_climbing {
            // Create emissive disk
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Circle::new(2.0)),
                    material: materials.add(StandardMaterial {
                        base_color: Color::srgba(1.0, 0.8, 0.0, 0.5),
                        emissive: Color::srgb(2.0, 1.0, 0.0).into(),
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    }),
                    transform: Transform::from_translation(Vec3::new(0.0, 0.1, 0.0))
                        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                    ..default()
                },
                HighAltitudeIndicator,
            )).set_parent(protagonist_entity);
        } else if (!is_high_altitude && has_indicator) || protagonist.is_dirigible {
            // Remove indicator when below threshold
            for entity in high_altitude_indicator_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }

        for mut camera_transform in camera_query.iter_mut() {
            // Adjust follow offset based on state
            let follow_offset = if protagonist.is_birds_eye {
                Vec3::new(0.0, 50.0, 0.1) // Birds-eye view (high up, looking straight down)
            } else if protagonist_position.y > 100.0 {
                Vec3::new(0.0, 30.0, 100.0)
            } else if protagonist.is_driving && !protagonist.is_climbing {
                Vec3::new(0.0, 20.0, 90.0)  // Driving
            } else if !protagonist.is_driving && protagonist.is_climbing {
                Vec3::new(0.0, 2.0, 30.0)   // Climbing
            } else {
                Vec3::new(0.0, 2.0, 15.0)   // Default state
            };

            // Calculate the new camera position
            let rotated_offset = if protagonist.is_birds_eye {
                // Directly use the protagonist's rotation for birds-eye view
                protagonist_rotation * follow_offset
            } else if protagonist.is_driving {
                let driving_rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
                protagonist_rotation * driving_rotation * follow_offset
            } else {
                protagonist_rotation * follow_offset
            };
            let new_camera_position = protagonist_position + rotated_offset;

            // Smoothly move the camera to the new position
            camera_transform.translation = camera_transform
                .translation
                .lerp(new_camera_position, time.delta_seconds() * 5.0);

            // Look at logic for birds-eye view
            if protagonist.is_birds_eye {
                // Look straight down while maintaining the protagonist's rotation
                let up = protagonist_rotation * -Vec3::Z;
                camera_transform.look_at(protagonist_position, up);
            } else {
                camera_transform.look_at(protagonist_position, Vec3::Y);
            }
        }
    }
}