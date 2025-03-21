use bevy::prelude::*;
use crate::components::Protagonist;
use std::f32::consts::*;

#[derive(Component)]
pub struct UnderwaterSearchlight {
    sweep_offset: f32,
    sweep_speed: f32,
}

pub fn update_searchlight_rotation(
    time: Res<Time>,
    mut light_query: Query<(&mut Transform, &UnderwaterSearchlight)>,
) {
    for (mut light_transform, searchlight) in light_query.iter_mut() {
        let time_factor = time.elapsed_seconds() * searchlight.sweep_speed * 0.3;
        
        light_transform.rotation = Quat::from_euler(
            EulerRot::XYZ,
            -FRAC_PI_3 + (time_factor * 0.5).sin() * 0.4,
            (time_factor + searchlight.sweep_offset).sin() * 1.6,
            (time_factor * 0.7).cos() * 0.15,
        );
    }
}

pub fn underwater_searchlight_system(
    mut commands: Commands,
    query: Query<&Protagonist>,
    mut light_query: Query<(&mut DirectionalLight, Entity), With<UnderwaterSearchlight>>,
) {
    let is_swimming = query.single().is_swimming;
    let has_light = !light_query.is_empty();

    match (is_swimming, has_light) {
        (_, false) => {
            commands.spawn((
                DirectionalLightBundle {
                    directional_light: DirectionalLight {
                        color: if is_swimming {
                            Color::srgb(0.24, 0.62, 0.92) // Blue underwater color
                        } else {
                            Color::srgb(0.6, 0.3, 0.3) // Softer reddish color
                        },
                        illuminance: 2500.0,
                        shadows_enabled: true,
                        ..default()
                    },
                    transform: Transform::default(),
                    ..default()
                },
                UnderwaterSearchlight {
                    sweep_offset: fastrand::f32() * PI * 2.0,
                    sweep_speed: 0.8 + fastrand::f32() * 0.4,
                },
            ));
        }
        (is_swimming, true) => {
            for (mut light, _) in light_query.iter_mut() {
                light.color = if is_swimming {
                    Color::srgb(0.24, 0.62, 0.92) // Blue underwater color
                } else {
                    Color::srgb(0.6, 0.3, 0.3) // Softer reddish color
                };
            }
        }
    }
}
