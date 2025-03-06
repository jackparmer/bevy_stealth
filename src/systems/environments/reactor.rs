use bevy::prelude::*;
use avian3d::prelude::*;

pub const WALL_THICKNESS: f32 = 20.0;
const REACTOR_POSITION: Vec3 = Vec3::new(-455.0 * 2.0, 1.6, 915.0 * 2.0);

// Cylinder constants
const CYLINDER_INNER_RADIUS: f32 = 500.0;
const CYLINDER_RADIUS: f32 = CYLINDER_INNER_RADIUS + WALL_THICKNESS;
pub const CYLINDER_HEIGHT: f32 = 800.0;

// Constants for the glowing disc
const DISC_GAP: f32 = 10.0;
const DISC_THICKNESS: f32 = 2.0;
const SAFETY_MARGIN: f32 = 5.0;

// Glider constants
const WING_SPAN: f32 = 160.0;   // Much longer wings for solar gliding
const BODY_LENGTH: f32 = 30.0;   // Slightly longer, very slim body
const WING_SWEEP: f32 = 15.0;    // Less sweep for better lift
const TAIL_SPAN: f32 = 40.0;     // Larger tail for stability

pub fn spawn_reactor(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
) {
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/worn_corrugated_iron_diff_4k.png")),
        metallic: 0.1,
        perceptual_roughness: 0.8,
        ..default()
    });

    // Main cylinder
    let cylinder_mesh = meshes.add(Extrusion::new(
        Annulus {
            inner_circle: Circle::new(CYLINDER_INNER_RADIUS),
            outer_circle: Circle::new(CYLINDER_RADIUS),
        },
        CYLINDER_HEIGHT - DISC_GAP,
    ));

    // Cap meshes - using extrusion instead of circle for thickness
    let cap_mesh = meshes.add(Extrusion::new(
        Circle::new(CYLINDER_RADIUS),
        WALL_THICKNESS,
    ));

    // Create glowing material for the disc
    let glow_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        emissive: Color::rgb(1.0, 0.0, 0.0).into(), // Bright red glow
        ..default()
    });

    // Disc mesh
    let disc_mesh = meshes.add(Extrusion::new(
        Circle::new(CYLINDER_RADIUS),
        DISC_THICKNESS,
    ));

    // Spawn main cylinder
    commands.spawn((
        PbrBundle {
            mesh: cylinder_mesh,
            material: material.clone(),
            transform: Transform::from_translation(REACTOR_POSITION)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ..default()
        },
        RigidBody::Static,
        ColliderConstructor::TrimeshFromMesh,
    ));

    // Spawn top cap with offset - moved further up
    commands.spawn((
        PbrBundle {
            mesh: cap_mesh.clone(),
            material: material.clone(),
            transform: Transform::from_translation(REACTOR_POSITION + Vec3::new(0.0, CYLINDER_HEIGHT/2.0 + DISC_GAP + WALL_THICKNESS/2.0 + SAFETY_MARGIN, 0.0))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            ..default()
        },
        RigidBody::Static,
        ColliderConstructor::TrimeshFromMesh,
    ));

    // Spawn glowing disc - positioned with more space between cylinder and cap
    commands.spawn(PbrBundle {
        mesh: disc_mesh,
        material: glow_material,
        transform: Transform::from_translation(REACTOR_POSITION + Vec3::new(0.0, CYLINDER_HEIGHT/2.0 + SAFETY_MARGIN, 0.0))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });

    // Add intense point light in the lower half of the reactor
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 100000.0, // Very bright light
            color: Color::rgb(1.0, 0.2, 0.2), // Reddish color to match the reactor glow
            radius: 1.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_translation(
            REACTOR_POSITION + Vec3::new(0.0, -CYLINDER_HEIGHT/4.0, 0.0) // Positioned at 1/4 height from bottom
        ),
        ..default()
    });

    // Hang glider materials
    let frame_material = materials.add(StandardMaterial {
        base_color: Color::rgb(1.0, 0.3, 0.0), // Bright orange frame
        emissive: Color::rgb(0.2, 0.1, 0.0).into(),
        metallic: 0.9,
        perceptual_roughness: 0.1, // Glossy
        ..default()
    });

    let fabric_material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.0, 0.8, 1.0), // Tropical blue
        emissive: Color::rgb(0.0, 0.2, 0.3).into(),
        metallic: 0.0,
        perceptual_roughness: 0.5, // Fabric-like
        ..default()
    });

    let accent_material = materials.add(StandardMaterial {
        base_color: Color::rgb(1.0, 0.8, 0.0), // Sunny yellow
        emissive: Color::rgb(0.2, 0.2, 0.0).into(),
        metallic: 0.0,
        perceptual_roughness: 0.5,
        ..default()
    });

    let glider_position = REACTOR_POSITION + Vec3::new(
        CYLINDER_RADIUS * 0.8,
        CYLINDER_HEIGHT/2.0 + DISC_GAP + WALL_THICKNESS + 15.0,
        0.0
    );

    // Parent entity for the hang glider
    let glider = commands.spawn((
        TransformBundle::from(Transform::from_translation(glider_position)
            .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_4))),
        RigidBody::Dynamic,
        Collider::compound(vec![
            (Vec3::ZERO, Quat::IDENTITY, Collider::capsule(0.5, BODY_LENGTH)), // Control bar
            (Vec3::ZERO, Quat::IDENTITY, Collider::sphere(WING_SPAN/2.0)), // Rough wing area
        ]),
    )).id();

    // Main keel (center tube)
    commands.spawn(PbrBundle {
        mesh: meshes.add(Capsule3d::new(BODY_LENGTH/2.0, 0.5)),
        material: frame_material.clone(),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    }).set_parent(glider);

    // Leading edge tubes (left and right)
    let edge_tube = meshes.add(Capsule3d::new(WING_SPAN/3.0, 0.5));
    
    // Left leading edge
    commands.spawn(PbrBundle {
        mesh: edge_tube.clone(),
        material: frame_material.clone(),
        transform: Transform::from_xyz(-WING_SPAN/4.0, 0.0, -BODY_LENGTH/2.0)
            .with_rotation(Quat::from_rotation_y(WING_SWEEP.to_radians())),
        ..default()
    }).set_parent(glider);

    // Right leading edge
    commands.spawn(PbrBundle {
        mesh: edge_tube.clone(),
        material: frame_material.clone(),
        transform: Transform::from_xyz(WING_SPAN/4.0, 0.0, -BODY_LENGTH/2.0)
            .with_rotation(Quat::from_rotation_y(-WING_SWEEP.to_radians())),
        ..default()
    }).set_parent(glider);

    // Control bar (triangle)
    let control_bar = meshes.add(Capsule3d::new(5.0, 0.3));
    
    // Bottom bar
    commands.spawn(PbrBundle {
        mesh: control_bar.clone(),
        material: frame_material.clone(),
        transform: Transform::from_xyz(0.0, -3.0, 0.0)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        ..default()
    }).set_parent(glider);

    // Left diagonal
    commands.spawn(PbrBundle {
        mesh: control_bar.clone(),
        material: frame_material.clone(),
        transform: Transform::from_xyz(-2.5, -1.5, 0.0)
            .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_4)),
        ..default()
    }).set_parent(glider);

    // Right diagonal
    commands.spawn(PbrBundle {
        mesh: control_bar.clone(),
        material: frame_material.clone(),
        transform: Transform::from_xyz(2.5, -1.5, 0.0)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
        ..default()
    }).set_parent(glider);

    // Wing fabric sections (triangular panels)
    const FABRIC_SECTIONS: i32 = 15;
    let section_width = WING_SPAN / (FABRIC_SECTIONS as f32);
    
    for i in 0..FABRIC_SECTIONS {
        let x_offset = (i as f32 - FABRIC_SECTIONS as f32 / 2.0) * section_width;
        let fabric_mesh = meshes.add(Cuboid::new(section_width * 0.95, 0.05, BODY_LENGTH * 0.8));
        
        commands.spawn(PbrBundle {
            mesh: fabric_mesh,
            material: if i % 2 == 0 { fabric_material.clone() } else { accent_material.clone() },
            transform: Transform::from_xyz(x_offset, 0.1, -BODY_LENGTH/4.0)
                .with_rotation(Quat::from_rotation_y(
                    (x_offset / (WING_SPAN/2.0)) * WING_SWEEP.to_radians()
                )),
            ..default()
        }).set_parent(glider);
    }
}

