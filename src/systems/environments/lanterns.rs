use bevy::prelude::*;
use std::f32::consts::PI;

const LANTERN_POSITIONS: &[Vec3] = &[
    Vec3::new(-252.9185, 3.992052, -101.97985),
    Vec3::new(-4750.1074, 3.992052, -2112.2625),
    Vec3::new(-4302.358, 3.992052, -1381.9333),
    Vec3::new(-3277.1743, 3.992052, -996.9319),
    Vec3::new(-2491.45, 3.992052, -217.87823),
    Vec3::new(-1503.2008, 3.992052, 719.15466),
    Vec3::new(-908.6698, 3.992052, 237.21597),
    Vec3::new(-968.78375, 3.992052, -1228.4734),
    Vec3::new(-237.44571, 3.992052, -549.917),
    Vec3::new(-73.91964, 3.992052, 568.6425),
    Vec3::new(613.49396, 3.992052, 952.3612),
    Vec3::new(924.8113, 3.992052, 309.89914),
    Vec3::new(1389.7969, 3.992052, 49.68644),
    Vec3::new(-4455.5327, 3.992052, -3130.1377),
    Vec3::new(-3932.7915, 3.992052, -3874.0251),
    Vec3::new(-2952.263, 3.992052, -4638.2104),
    Vec3::new(-986.376, 3.992052, -5880.17),
    Vec3::new(1830.869, 3.992052, -6521.1133),
];

const FLOAT_HEIGHT: f32 = 30.0;
const FLOAT_SPEED: f32 = 0.8;
const LANTERN_RADIUS: f32 = 8.0;
const LIGHT_INTENSITY: f32 = 150000.0;
const WIND_STRENGTH: f32 = 8.0;
const WIND_SPEED: f32 = 0.4;

#[derive(Component)]
pub struct FloatingLantern {
    origin: Vec3,
    time_offset: f32,
}

pub fn spawn_lanterns(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let lantern_mesh = meshes.add(Sphere { radius: LANTERN_RADIUS });
    let lantern_material = materials.add(StandardMaterial {
        base_color: Color::rgb(1.0, 0.6, 0.2),
        emissive: Color::rgb(1.0, 0.4, 0.1).into(),
        metallic: 0.0,
        perceptual_roughness: 0.089,
        ..default()
    });

    for &position in LANTERN_POSITIONS {
        let time_offset = fastrand::f32() * PI * 2.0;
        
        commands.spawn((
            PbrBundle {
                mesh: lantern_mesh.clone(),
                material: lantern_material.clone(),
                transform: Transform::from_translation(position + Vec3::Y * FLOAT_HEIGHT),
                ..default()
            },
            FloatingLantern {
                origin: position,
                time_offset,
            },
        ))
        .with_children(|parent| {
            parent.spawn(PointLightBundle {
                point_light: PointLight {
                    color: Color::rgb(1.0, 0.6, 0.2),
                    intensity: LIGHT_INTENSITY,
                    radius: LANTERN_RADIUS * 2.0,
                    shadows_enabled: true,
                    ..default()
                },
                ..default()
            });
        });
    }
}

pub fn update_lanterns(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &FloatingLantern)>,
) {
    for (mut transform, lantern) in query.iter_mut() {
        let elapsed = time.elapsed_seconds() + lantern.time_offset;
        
        // Vertical floating motion
        let height_offset = (elapsed * FLOAT_SPEED).sin() * 5.0;
        
        // Enhanced wind effect
        let wind_x = (elapsed * WIND_SPEED).sin() * WIND_STRENGTH;
        let wind_z = (elapsed * WIND_SPEED * 0.7).cos() * WIND_STRENGTH;
        
        // Additional turbulence
        let turbulence_x = (elapsed * 1.2 + lantern.time_offset).sin() * 2.0;
        let turbulence_z = (elapsed * 1.4 + lantern.time_offset).cos() * 2.0;

        transform.translation = lantern.origin + Vec3::new(
            wind_x + turbulence_x,
            FLOAT_HEIGHT + height_offset,
            wind_z + turbulence_z,
        );

        // Add slight tilting based on wind direction
        let tilt_angle_x = (wind_z * 0.02).clamp(-0.2, 0.2);
        let tilt_angle_z = (-wind_x * 0.02).clamp(-0.2, 0.2);
        transform.rotation = Quat::from_euler(EulerRot::XYZ, tilt_angle_x, 0.0, tilt_angle_z);
    }
}
