use bevy::prelude::*;
use crate::components::Protagonist;

pub fn rotate_camera(
    time: Res<Time>,
    protagonist_query: Query<(&Transform, &Protagonist)>, // Added Protagonist component
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<Protagonist>)>,
) {
    if let Ok((protagonist_transform, protagonist)) = protagonist_query.get_single() {
        let protagonist_position = protagonist_transform.translation;
        let protagonist_rotation = protagonist_transform.rotation;

        for mut camera_transform in camera_query.iter_mut() {
            // Adjust follow offset based on state
            let follow_offset = match (protagonist.is_driving, protagonist.is_climbing) {
                (true, false) => Vec3::new(0.0, 20.0, 90.0),  // Driving
                (false, true) => Vec3::new(0.0, 2.0, 30.0),  // Climbing
                (false, false) => Vec3::new(0.0, 2.0, 15.0),  // Default state
                _ => Vec3::new(0.0, 2.0, 15.0),  // Fallback for invalid states
            };

            // Calculate the new camera position by applying the protagonist's rotation to the offset
            let rotated_offset = if protagonist.is_driving {
                // Add 90 degrees (π/2 radians) rotation when driving
                let driving_rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
                protagonist_rotation * driving_rotation * follow_offset
            } else {
                protagonist_rotation * follow_offset
            };
            let new_camera_position = protagonist_position + rotated_offset;

            // Smoothly move the camera to the new position
            camera_transform.translation = camera_transform
                .translation
                .lerp(new_camera_position, time.delta_seconds() * 5.0); // Adjust lerp speed as needed

            // Ensure the camera is always looking at the protagonist
            camera_transform.look_at(protagonist_position, Vec3::Y);
        }
    }
}