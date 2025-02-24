use bevy::{
    prelude::*,
    render::camera::Viewport,
    render::view::RenderLayers,
};
use crate::components::Protagonist;
use crate::components::Sentry;

// Add marker component for minimap elements
#[derive(Component)]
pub struct MinimapMarker;

#[derive(Component)]
pub struct MinimapCamera;

// Add new component for sentry minimap markers
#[derive(Component)]
pub struct SentryMinimapMarker(pub Entity); // Stores the entity ID of the actual sentry

// Make constants public
pub const MINIMAP_CAMERA_HEIGHT: f32 = 2000.0;
pub const MINIMAP_SMOOTHING_SPEED: f32 = 5.0;
pub const MINIMAP_POSITION_PRECISION: f32 = 100.0; // For rounding
pub const MINIMAP_MARKER_HEIGHT: f32 = 500.0; // Half of MINIMAP_CAMERA_HEIGHT
pub const MINIMAP_MARKER_SIZE: f32 = 3.0;

// Add resource for shared minimap assets
#[derive(Resource)]
pub struct MinimapResources {
    pub sentry_mesh: Handle<Mesh>,
    pub sentry_material: Handle<StandardMaterial>,
}

pub fn setup_minimap(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>) {

    // Spawn minimap camera (initially hidden)
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                viewport: Some(Viewport {
                    physical_position: UVec2::new(0, 0),
                    physical_size: UVec2::new(400, 400),
                    ..default()
                }),
                order: 2,
                ..default()
            },
            transform: Transform::from_xyz(0.0, MINIMAP_CAMERA_HEIGHT, 0.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        MinimapCamera,
        Visibility::Hidden,
        RenderLayers::layer(1),
    ));

    // Spawn red dot for protagonist (initially hidden)
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
            visibility: Visibility::Hidden,  // Start hidden
            ..default()
        },
        MinimapMarker,
        RenderLayers::layer(1),
    ));

    // Add shared mesh and material resources for sentry markers
    commands.insert_resource(MinimapResources {
        sentry_mesh: meshes.add(Mesh::from(Sphere { radius: MINIMAP_MARKER_SIZE * 0.8, ..default() })),
        sentry_material: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.6, 0.0), // Orange color
            emissive: Color::srgba(1.0, 0.6, 0.0, 0.5).into(),
            ..default()
        }),
    });
}

// Update the marker position to follow protagonist
pub fn update_minimap(
    protagonist_query: Query<(&Transform, &Protagonist)>,
    mut param_set: ParamSet<(
        Query<&mut Transform, (With<MinimapCamera>, Without<Protagonist>, Without<MinimapMarker>)>,
        Query<&mut Transform, (With<MinimapMarker>, Without<MinimapCamera>, Without<Protagonist>)>,
        Query<&mut Camera, With<MinimapCamera>>,
        Query<&mut Visibility, (With<MinimapMarker>, Without<Protagonist>)>,
    )>,
    time: Res<Time>,
) {
    if let Ok((player_transform, _protagonist)) = protagonist_query.get_single() {
        // Always show minimap
        for mut camera in param_set.p2().iter_mut() {
            camera.viewport = Some(Viewport {
                physical_position: UVec2::new(0, 0),
                physical_size: UVec2::new(400, 400),
                ..default()
            });
        }

        // Always show markers
        for mut visibility in param_set.p3().iter_mut() {
            *visibility = Visibility::Visible;
        }

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

// Add system to update sentry markers
pub fn update_sentry_markers(
    mut param_set: ParamSet<(
        Query<(&Transform, Entity), With<Sentry>>,
        Query<(&mut Transform, &SentryMinimapMarker)>,
    )>,
    time: Res<Time>,
) {
    // Collect sentry positions first
    let sentry_positions: Vec<(Entity, Vec3)> = param_set.p0()
        .iter()
        .map(|(transform, entity)| (entity, transform.translation))
        .collect();

    // Update marker positions
    for (mut marker_transform, marker) in param_set.p1().iter_mut() {
        if let Some((_, sentry_pos)) = sentry_positions.iter().find(|(entity, _)| *entity == marker.0) {
            let target_pos = Vec3::new(
                sentry_pos.x,
                MINIMAP_MARKER_HEIGHT,
                sentry_pos.z,
            );
            
            marker_transform.translation = marker_transform.translation.lerp(
                target_pos,
                time.delta_seconds() * MINIMAP_SMOOTHING_SPEED,
            );
        }
    }
}
