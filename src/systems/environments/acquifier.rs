use bevy::prelude::*;
use avian3d::prelude::*;

use crate::systems::core::setup::{WORLD_RADIUS, PERIMETER_WALL_HEIGHT, ACQUIFIER_FLOOR_DEPTH};
use crate::systems::player::dirigible::DirigibleBalloon;
use crate::systems::core::screenplay::{MessageDisplay, display_message};
use crate::components::Protagonist;

// Add new component
#[derive(Component)]
pub struct AcquifierDirigibleTrigger {
    pub position: Vec3,
    pub radius: f32,
    pub entity: Entity,
}

pub fn spawn_acquifier(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
) {
    // Add connecting perimeter wall between tundra and aquifer
    commands.spawn((
        RigidBody::Static,
        ColliderConstructor::TrimeshFromMesh,
        PbrBundle {
            mesh: meshes.add(Extrusion::new(
                Annulus::new(WORLD_RADIUS - 100.0, WORLD_RADIUS), 
                PERIMETER_WALL_HEIGHT
            )),
            material: materials.add(StandardMaterial {
                base_color: Color::BLACK,
                base_color_texture: None,
                perceptual_roughness: 0.0,
                metallic: 0.0,
                double_sided: true,
                cull_mode: None,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, -PERIMETER_WALL_HEIGHT/2.0, 0.0)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ..default()
        },
        Name::new("PerimeterWall"),
    ));

    // Large white acquifier floor
    commands.spawn((
        RigidBody::Static,
        Collider::cylinder(WORLD_RADIUS, 50.0),
        PbrBundle {
            mesh: meshes.add(Cylinder {
                radius: WORLD_RADIUS,
                half_height: 25.0,
            }),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/ice_texture2.png")),
                perceptual_roughness: 0.2,
                metallic: 0.9,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, ACQUIFIER_FLOOR_DEPTH, 0.0), 
            ..default()
        },
        Name::new("AcquifierFloor"),
    ));

    // Add illuminated dirigible sphere
    let sphere_radius = 20.0;
    let sphere_pos = Vec3::new(0.0, ACQUIFIER_FLOOR_DEPTH + sphere_radius * 1.2, 0.0);
    
    let sphere_entity = commands.spawn_empty().id();
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(sphere_radius)),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                base_color_texture: Some(asset_server.load("textures/american-flag-background.png")),
                metallic: 0.8,
                perceptual_roughness: 0.1,
                reflectance: 0.7,
                emissive: Color::srgb(0.5, 0.5, 0.5).into(),
                ..default()
            }),
            transform: Transform::from_translation(sphere_pos),
            ..default()
        },
        AcquifierDirigibleTrigger {
            position: sphere_pos,
            radius: sphere_radius,
            entity: sphere_entity,
        }
    ));
}

// Add new system to check for trigger and handle transition
pub fn check_acquifier_dirigible_trigger(
    trigger_query: Query<(Entity, &AcquifierDirigibleTrigger)>,
    mut player_query: Query<(Entity, &Transform, &mut Protagonist)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut message_display: ResMut<MessageDisplay>,
    time: Res<Time>,
) {
    if let (Ok((trigger_entity, trigger)), Ok((player_entity, player_transform, mut protagonist))) = (
        trigger_query.get_single(),
        player_query.get_single_mut()
    ) {
        let player_pos = player_transform.translation;
        let horizontal_dist = Vec2::new(
            player_pos.x - trigger.position.x,
            player_pos.z - trigger.position.z
        ).length();

        if horizontal_dist < trigger.radius * 1.5 && player_pos.y < trigger.position.y {
            if !protagonist.is_dirigible {
                // Display message
                display_message(
                    "DIRIGIBLE MODE ACTIVATED - SPACE TO FLOAT, SHIFT TO DESCEND",
                    Color::WHITE,
                    &mut message_display
                );

                protagonist.is_dirigible = true;
                protagonist.is_swimming = false;
                protagonist.is_falling = false;
                protagonist.is_climbing = false;

                // Spawn the dirigible balloon (10x larger)
                commands.entity(player_entity).with_children(|parent| {
                    parent.spawn((
                        PbrBundle {
                            mesh: meshes.add(Mesh::from(Sphere::new(200.0))), // 10x larger radius
                            material: materials.add(StandardMaterial {
                                base_color: Color::srgb(1.0, 1.0, 1.0),
                                base_color_texture: Some(asset_server.load("textures/american-flag-background.png")),
                                metallic: 0.8,
                                perceptual_roughness: 0.1,
                                reflectance: 0.7,
                                ..default()
                            }),
                            transform: Transform::from_xyz(0.0, 300.0, 0.0), // 10x higher position
                            ..default()
                        },
                        DirigibleBalloon,
                    ));
                });

                // Add smooth animation component and disable collisions
                commands.entity(player_entity)
                    .insert(DirigibleTransition {
                        start_pos: player_transform.translation,
                        target_pos: player_transform.translation + Vec3::new(0.0, 4000.0, 0.0), // 10x higher animation
                        start_time: time.elapsed_seconds(),
                        duration: 5.0, // 5 second animation
                    })
                    .remove::<Collider>(); // Remove collider during transition

                // Despawn trigger sphere
                commands.entity(trigger_entity).despawn_recursive();
            }
        }
    }
}

// Add new component for smooth transition
#[derive(Component)]
pub struct DirigibleTransition {
    start_pos: Vec3,
    target_pos: Vec3,
    start_time: f32,
    duration: f32,
}

