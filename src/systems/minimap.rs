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
                radius: 5.0,
                ..default()
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.0, 0.0),
                emissive: Color::srgba(1.0, 0.0, 0.0, 0.5).into(),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 500.0, 0.0),
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
) {
    if let Ok(player_transform) = protagonist_query.get_single() {
        // Update camera position but keep static rotation
        for mut camera_transform in param_set.p0().iter_mut() {
            camera_transform.translation = Vec3::new(
                player_transform.translation.x,
                MINIMAP_CAMERA_HEIGHT,
                player_transform.translation.z
            );
        }

        // Update marker position and rotation to show player direction
        for mut marker_transform in param_set.p1().iter_mut() {
            marker_transform.translation = Vec3::new(
                player_transform.translation.x,
                500.0,
                player_transform.translation.z
            );
        }
    }
}
