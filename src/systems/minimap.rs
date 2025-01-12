use bevy::{
    prelude::*,
    render::camera::Viewport,
};
use crate::components::Protagonist;

// Add marker component for minimap elements
#[derive(Component)]
pub struct MinimapMarker;

#[derive(Component)]
pub struct MinimapCamera;

const MINIMAP_CAMERA_HEIGHT: f32 = 1000.0;
const MINIMAP_SMOOTHING_SPEED: f32 = 5.0;
const MINIMAP_POSITION_PRECISION: f32 = 100.0; // For rounding
const MINIMAP_MARKER_HEIGHT: f32 = 200.0; // Half of MINIMAP_CAMERA_HEIGHT
const MINIMAP_MARKER_SIZE: f32 = 3.0;

pub fn setup_minimap(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>) {

    // Spawn minimap camera
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                viewport: Some(Viewport {
                    physical_position: UVec2::new(0, 0),
                    physical_size: UVec2::new(400, 400),
                    ..default()
                }),
                order: 1,
                ..default()
            },
            transform: Transform::from_xyz(0.0, MINIMAP_CAMERA_HEIGHT, 0.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        MinimapCamera,
    ));

    // Spawn red dot for protagonist with neutral initial rotation
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Sphere {
                radius: MINIMAP_MARKER_SIZE,
                ..default()
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.0, 0.0),
                emissive: Color::srgba(1.0, 0.0, 0.0, 0.5).into(),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, MINIMAP_MARKER_HEIGHT, 0.0),
            ..default()
        },
        MinimapMarker,
    ));
}

// Update the marker position to follow protagonist
pub fn update_minimap(
    protagonist_query: Query<&Transform, With<Protagonist>>,
    mut param_set: ParamSet<(
        Query<&mut Transform, (With<MinimapCamera>, Without<Protagonist>, Without<MinimapMarker>)>,
        Query<&mut Transform, (With<MinimapMarker>, Without<MinimapCamera>, Without<Protagonist>)>,
    )>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = protagonist_query.get_single() {
        // Update camera position with smoothing
        for mut camera_transform in param_set.p0().iter_mut() {
            let target_pos = Vec3::new(
                (player_transform.translation.x * MINIMAP_POSITION_PRECISION).round() / MINIMAP_POSITION_PRECISION,
                (player_transform.translation.y + MINIMAP_CAMERA_HEIGHT).round(),
                (player_transform.translation.z * MINIMAP_POSITION_PRECISION).round() / MINIMAP_POSITION_PRECISION,
            );
            
            camera_transform.translation = camera_transform.translation.lerp(
                target_pos,
                time.delta_seconds() * MINIMAP_SMOOTHING_SPEED,
            );
        }

        // Update marker position with the same smoothing
        for mut marker_transform in param_set.p1().iter_mut() {
            let target_pos = Vec3::new(
                (player_transform.translation.x * MINIMAP_POSITION_PRECISION).round() / MINIMAP_POSITION_PRECISION,
                (player_transform.translation.y + MINIMAP_MARKER_HEIGHT).round(),
                (player_transform.translation.z * MINIMAP_POSITION_PRECISION).round() / MINIMAP_POSITION_PRECISION,
            );
            
            marker_transform.translation = marker_transform.translation.lerp(
                target_pos,
                time.delta_seconds() * MINIMAP_SMOOTHING_SPEED,
            );
        }
    }
}
