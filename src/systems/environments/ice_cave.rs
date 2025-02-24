use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use std::f32::consts::PI;
use avian3d::prelude::*;
use fastrand;

pub fn spawn_ice_cave(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
    time: &Res<Time>,
) {
    let position = Vec3::new(-485.0 * 2.0, 2.6249764, -1066.0 * 2.0);
    let radius = 300.0;
    let wall_thickness = 200.0;  // Thicker walls for crystal embedding
    let height = 2400.0;

    // Main cylinder
    let cylinder_mesh = meshes.add(Extrusion::new(
        Annulus {
            inner_circle: Circle::new(radius),
            outer_circle: Circle::new(radius + wall_thickness),
        },
        height,
    ));

    // Create material for the ice cave walls
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.01, 0.01, 0.01),
        perceptual_roughness: 0.5,
        metallic: 0.2,
        double_sided: true,
        cull_mode: None,
        unlit: false,
        reflectance: 0.1,
        ..default()
    });

    // Spawn main cylinder
    commands.spawn((
        PbrBundle {
            mesh: cylinder_mesh,
            material: material.clone(),
            transform: Transform::from_translation(position)
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)), // Lay on side
            ..default()
        },
        RigidBody::Static,
        ColliderConstructor::TrimeshFromMesh,
    ));

    // Replace spotlights with point lights
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            color: Color::rgb(0.9, 0.95, 1.0),
            intensity: 100000.0,
            range: radius * 4.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_translation(position + Vec3::new(-height * 0.25, 0.0, 0.0)),
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            color: Color::rgb(0.9, 0.95, 1.0),
            intensity: 100000.0,
            range: radius * 4.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_translation(position + Vec3::new(height * 0.25, 0.0, 0.0)),
        ..default()
    });

    // Spawn crystals around the cylinder
    let perlin = Perlin::new(42);
    let segments = 64;
    let height_segments = 96;
    let cube_size = 80.0;

    for h in 0..height_segments {
        for s in 0..segments {
            if fastrand::f32() > 0.3 { continue; } // Reduce crystal density

            let angle = (s as f32 * 2.0 * PI) / segments as f32;
            let height_offset = (h as f32 * height) / height_segments as f32 - height / 2.0;
            
            // Position crystals on the inner surface, matching cylinder's Z-rotation
            let direction = Vec3::new(
                angle.cos(),
                angle.sin(),
                0.0
            ).normalize();

            let base_pos = Vec3::new(
                angle.cos() * radius,
                angle.sin() * radius,
                height_offset  // Z is now height
            );
            
            let crystal_pos = position + base_pos;

            // Rest of crystal spawning code remains the same, but remove the direction calculation
            let radius_variation = cube_size * (0.8 + fastrand::f32() * 0.4);
            let y_normalized = (base_pos.y - (-radius)) / (radius * 2.0);
            let hue = 180.0 + y_normalized * 60.0;

            let hue_variation = Color::hsl(
                hue + fastrand::f32() * 5.0,
                0.8 + fastrand::f32() * 0.15,
                0.15 + fastrand::f32() * 0.15,
            );
            let emissive_strength = 0.1 + fastrand::f32() * 0.2;

            commands.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(
                    radius_variation,
                    radius_variation * (0.3 + fastrand::f32() * 0.4),
                    radius_variation * (0.5 + fastrand::f32() * 0.5)
                )),
                material: materials.add(StandardMaterial {
                    base_color: hue_variation,
                    metallic: 0.9,
                    perceptual_roughness: 0.1,
                    reflectance: 0.8,
                    emissive: Color::hsl(
                        hue + fastrand::f32() * 5.0,
                        0.8 + fastrand::f32() * 0.15,
                        emissive_strength,
                    ).into(),
                    base_color_texture: Some(asset_server.load("textures/amethyst.png")),
                    ..default()
                }),
                transform: Transform::from_translation(crystal_pos)
                    .looking_to(direction, Vec3::Z)
                    .with_rotation(
                        Quat::from_rotation_x(fastrand::f32() * PI) *
                        Quat::from_rotation_y(fastrand::f32() * PI) *
                        Quat::from_rotation_z(fastrand::f32() * PI)
                    ),
                ..default()
            });
        }
    }

    // Add particle emitters along the cylinder
    let particle_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.8, 1.0), // Added blue tint
        emissive: Color::srgb(1.4, 1.6, 4.0).into(), // Much brighter, especially in blue channel
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let particle_mesh = meshes.add(Mesh::from(Sphere { radius: 0.1 })); // 10x smaller

    // Increase particle count for density
    for _ in 0..20000 { // Doubled particle count
        let random_speed = 5.0 + fastrand::f32() * 5.0; // Reduced speed by 4x
        
        commands.spawn((
            PbrBundle {
                mesh: particle_mesh.clone(),
                material: particle_material.clone(),
                transform: Transform::from_translation(position),
                ..default()
            },
            IceCaveParticle {
                velocity: Vec3::new(0.0, random_speed, 0.0),
                origin: position,
                start_time: time.elapsed_seconds(),
            },
        ));
    }
}

// Add this component
#[derive(Component)]
pub struct IceCaveParticle {
    velocity: Vec3,
    origin: Vec3,
    start_time: f32,
}

// Add this system to your main.rs
pub fn update_ice_particles(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut IceCaveParticle)>,
) {
    for (_, mut transform, particle) in query.iter_mut() {
        let effect_age = time.elapsed_seconds() - particle.start_time;
        
        // Vertical rising motion with increased diffusion based on height
        let height = transform.translation.y - particle.origin.y;
        let height_fraction = (height / 300.0).clamp(0.0, 1.0);
        
        // Softer, more organic radial expansion - 20x wider
        let base_angle = effect_age * 0.5 + fastrand::f32() * PI * 2.0;
        let expansion_rate = height_fraction.powf(0.5) * 4000.0; // Increased from 200 to 4000
        
        // Add swirling motion that increases with height
        let swirl_angle = base_angle + height_fraction * PI * 2.0;
        let swirl_radius = expansion_rate * (1.0 + (effect_age * 0.5).sin() * 0.3);
        
        let radial_offset = Vec3::new(
            swirl_angle.cos() * swirl_radius,
            0.0,
            swirl_angle.sin() * swirl_radius
        );
        
        // Increased turbulence scales
        let fast_turbulence = Vec3::new(
            (effect_age * 3.0).sin() * 100.0, // 20x larger
            (effect_age * 2.5).cos() * 60.0,
            (effect_age * 2.8).sin() * 100.0
        );
        
        let slow_turbulence = Vec3::new(
            (effect_age * 0.8 + height * 0.02).sin() * 300.0 * height_fraction, // 20x larger
            (effect_age * 0.6).cos() * 200.0 * height_fraction,
            (effect_age * 0.7 + height * 0.02).sin() * 300.0 * height_fraction
        );
        
        // Combine movements with smoother transitions
        transform.translation += 
            particle.velocity * 1.0 - height_fraction * 0.5 +
            radial_offset * time.delta_seconds() * 0.3 +
            fast_turbulence * time.delta_seconds() +
            slow_turbulence * time.delta_seconds();
        
        // Reset when too high or too far from center
        let horizontal_distance = Vec2::new(
            transform.translation.x - particle.origin.x,
            transform.translation.z - particle.origin.z
        ).length();
        
        if height > 300.0 || horizontal_distance > 300.0 {
            transform.translation = particle.origin;
        }
    }
}
