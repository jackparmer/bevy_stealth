use bevy::prelude::*;
use avian3d::prelude::*;
use crate::components::Protagonist;
use crate::systems::player::dirigible::DirigibleBalloon;
use crate::systems::core::screenplay::{MessageDisplay, display_message};
use crate::systems::environments::ladder::{spawn_ladder, LadderConfig};
use crate::systems::player::driving::set_driving_state;

// Maze configuration
pub const MAZE_POSITION: Vec3 = Vec3::new(
    2232.0,
    600.0,
    6288.0
);
pub const CELL_SIZE: f32 = 400.0;
const WALL_HEIGHT: f32 = 20.0;
const WALL_THICKNESS: f32 = 200.0;
const MAZE_SIZE: usize = 20;

// Add new constants for sphere and arrows
const SPHERE_RADIUS: f32 = 20.0;

// Add new constants
const PYLON_WIDTH: f32 = 60.0;
const PYLON_HEIGHT: f32 = 600.0;  // Height from ground to platform
const NEON_STRIP_WIDTH: f32 = 4.0;
const NEON_STRIP_HEIGHT: f32 = 1.0;

// Add new constants
const HEIGHT_DROP_PER_SEGMENT: f32 = 20.0;  // How much each segment drops
const MIN_HEIGHT: f32 = 100.0;  // Minimum height before stopping descent

// Update neon color constants
const NEON_BLUE: Color = Color::rgba(0.0, 0.2, 0.99, 1.0);
const NEON_BLUE_EMISSION: Color = Color::rgb(0.0, 0.4, 0.8); 

// Add new component to track sphere position
#[derive(Component)]
pub struct DirigibleTriggerZone {
    pub position: Vec3,
    pub radius: f32,
    pub entity: Entity,
}

#[derive(Clone, Copy, PartialEq)]
enum Cell {
    Wall,
    Path,
}

// L-System rules
const ITERATIONS: usize = 3;
const ANGLE: f32 = 90.0;

const MAX_RADIUS: f32 = 5000.0;

#[derive(Clone, Debug)]
struct LSystemPath {
    points: Vec<Vec3>,
    direction: Vec3,
}

impl LSystemPath {
    fn new() -> Self {
        Self {
            points: vec![Vec3::ZERO],
            direction: Vec3::new(1.0, 0.0, 0.0),
        }
    }

    fn add_point(&mut self) -> bool {
        let last_point = *self.points.last().unwrap();
        let new_point = last_point + self.direction * CELL_SIZE;
        
        // Check if new point would exceed maximum radius
        if new_point.length() > MAX_RADIUS {
            return false;  // Don't add point if it exceeds radius
        }
        
        self.points.push(new_point);
        true
    }

    fn turn_left(&mut self) {
        let angle = ANGLE.to_radians();
        self.direction = Vec3::new(
            self.direction.z,
            0.0,
            -self.direction.x
        );
    }

    fn turn_right(&mut self) {
        let angle = -ANGLE.to_radians();
        self.direction = Vec3::new(
            -self.direction.z,
            0.0,
            self.direction.x
        );
    }
}

fn generate_l_system(iterations: usize) -> LSystemPath {
    let mut path = LSystemPath::new();
    let mut commands = String::from("F");

    // Apply L-system rules
    for _ in 0..iterations {
        let mut new_commands = String::new();
        for c in commands.chars() {
            match c {
                'F' => new_commands.push_str("F+F-F-FF+F+F-F"),
                '+' => new_commands.push('+'),
                '-' => new_commands.push('-'),
                _ => {}
            }
        }
        commands = new_commands;
    }

    // Generate path with radius restriction
    for c in commands.chars() {
        match c {
            'F' => {
                if !path.add_point() {
                    // If point wasn't added, try turning and adding again
                    path.turn_right();
                    if !path.add_point() {
                        path.turn_left();
                        path.turn_left();
                        if !path.add_point() {
                            // If all directions fail, stop generating
                            break;
                        }
                    }
                }
            },
            '+' => path.turn_left(),
            '-' => path.turn_right(),
            _ => {}
        }
    }

    path
}

pub fn spawn_maze(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let concrete_material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/ice_texture3.png")),
        perceptual_roughness: 0.1,
        metallic: 0.8,
        ..default()
    });

    let neon_material = materials.add(StandardMaterial {
        base_color: NEON_BLUE,
        emissive: NEON_BLUE_EMISSION.into(),
        perceptual_roughness: 0.0,
        metallic: 1.0,
        ..default()
    });

    let pylon_material = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        perceptual_roughness: 0.7,
        metallic: 0.3,
        ..default()
    });

    // Generate L-system path
    let mut path = generate_l_system(ITERATIONS);
    
    // Center and position path
    let bounds = path.points.iter().fold(
        (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)),
        |(min, max), &p| (
            Vec3::new(min.x.min(p.x), min.y.min(p.y), min.z.min(p.z)),
            Vec3::new(max.x.max(p.x), max.y.max(p.y), max.z.max(p.z))
        )
    );
    let center = (bounds.0 + bounds.1) / 2.0;
    
    // Adjust points with descending height - ensure flat segments
    let mut current_height = MAZE_POSITION.y;  // Start at maze position height
    
    // First flatten X-Z coordinates
    for point in path.points.iter_mut() {
        point.x -= center.x;
        point.z -= center.z;
        point.x += MAZE_POSITION.x;
        point.z += MAZE_POSITION.z;
    }

    // Then handle Y coordinates for flat segments
    for i in 0..path.points.len()-1 {
        path.points[i].y = current_height;
        path.points[i+1].y = current_height;
        current_height -= HEIGHT_DROP_PER_SEGMENT;
    }

    // Spawn segments and pylons
    for (i, window) in path.points.windows(2).enumerate() {
        let start = window[0];
        let end = window[1];
        
        if start.y < MIN_HEIGHT {
            continue;
        }
        
        let direction = Vec3::new(
            end.x - start.x,
            0.0,  // Force horizontal direction
            end.z - start.z
        ).normalize();
        
        let length = Vec2::new(end.x - start.x, end.z - start.z).length();
        let rotation = Quat::from_rotation_arc(Vec3::X, direction);
        let segment_center = start + direction * (length / 2.0);
        
        // Spawn platform segment
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(length, WALL_HEIGHT, WALL_THICKNESS)),
                material: concrete_material.clone(),
                transform: Transform::from_translation(segment_center).with_rotation(rotation),
                ..default()
            },
            RigidBody::Static,
            Collider::cuboid(length, WALL_HEIGHT, WALL_THICKNESS),
        ))
        .with_children(|parent| {
            // Add neon strips along both edges
            for z_offset in [-WALL_THICKNESS/2.0, WALL_THICKNESS/2.0] {
                parent.spawn(PbrBundle {
                    mesh: meshes.add(Cuboid::new(length, NEON_STRIP_HEIGHT, NEON_STRIP_WIDTH)),
                    material: neon_material.clone(),
                    transform: Transform::from_xyz(0.0, WALL_HEIGHT/2.0 + NEON_STRIP_HEIGHT/2.0, z_offset),
                    ..default()
                });
            }
        });

        // Spawn pylons inset from the edges
        for (pylon_index, &point) in [start, end].iter().enumerate() {
            // Calculate inset position
            let inset_direction = if pylon_index == 0 { direction } else { -direction };
            let inset_point = point + inset_direction * (PYLON_WIDTH / 2.0);
            
            let pylon_height = point.y;
            let pylon_pos = Vec3::new(
                inset_point.x, 
                point.y - pylon_height/2.0,
                inset_point.z
            );
            
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Cuboid::new(PYLON_WIDTH, pylon_height, PYLON_WIDTH)),
                    material: pylon_material.clone(),
                    transform: Transform::from_translation(pylon_pos),
                    ..default()
                },
                RigidBody::Static,
                Collider::cuboid(PYLON_WIDTH, pylon_height, PYLON_WIDTH),
            ));

            // Add ladder on the first pylon of the first segment
            if i == 0 && pylon_index == 0 {
                let ladder_pos = Vec3::new(
                    inset_point.x - PYLON_WIDTH/2.0 - 1.0,  // Opposite side of inset pylon
                    0.0,
                    inset_point.z
                );

                // Spawn tank exit zone at the ladder's position
                commands.spawn(TankExitZone {
                    position: ladder_pos + Vec3::new(0.0, pylon_height/2.0, 0.0), // Position at middle of ladder
                });

                spawn_ladder(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &asset_server,
                    LadderConfig {
                        position: ladder_pos,
                        rotation: Quat::from_rotation_y(std::f32::consts::PI),
                        height: pylon_height + WALL_HEIGHT/2.0,  // Add half wall height to make flush
                        rung_count: ((pylon_height + WALL_HEIGHT/2.0) / 2.0 * 5.0) as usize,
                    }
                );
            }
        }
    }

    // Find the last valid segment and spawn dirigible trigger
    let last_valid_segment = path.points.windows(2)
        .rev()  // Start from the end
        .find(|window| window[0].y >= MIN_HEIGHT);  // Find first segment above min height

    if let Some(last_window) = last_valid_segment {
        let start = last_window[0];
        let end = last_window[1];
        let segment_center = start + (end - start) / 2.0;  // Calculate true center
        
        // Spawn dirigible trigger well above the last segment
        spawn_dirigible_trigger(&mut commands, &mut meshes, &mut materials, &asset_server, segment_center);
    }
}

// Helper function to spawn dirigible trigger
fn spawn_dirigible_trigger(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
    position: Vec3,
) {
    let sphere_position = position + Vec3::new(0.0, 50.0, 0.0);
    let trigger_entity = commands.spawn_empty().id();
    
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Sphere::new(30.0))),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 1.0),
                base_color_texture: Some(asset_server.load("textures/american-flag-background.png")),
                metallic: 0.8,
                perceptual_roughness: 0.1,
                reflectance: 0.7,
                ..default()
            }),
            transform: Transform::from_translation(sphere_position),
            ..default()
        },
        DirigibleTriggerZone {
            position: sphere_position,
            radius: SPHERE_RADIUS,
            entity: trigger_entity,
        },
    ));
}

// Update the check_dirigible_trigger function
pub fn check_dirigible_trigger(
    trigger_query: Query<(Entity, &DirigibleTriggerZone)>,
    mut player_query: Query<(Entity, &Transform, &mut Protagonist)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut message_display: ResMut<MessageDisplay>,
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
                // Display message using the same system as garage
                display_message(
                    "FIND THE LAUNCH SILO (SHIFT TO FLOAT)",
                    Color::WHITE,
                    &mut message_display
                );

                protagonist.is_dirigible = true;
                protagonist.is_swimming = false;
                protagonist.is_falling = false;
                protagonist.is_climbing = false;

                // Spawn the dirigible balloon
                commands.entity(player_entity).with_children(|parent| {
                    parent.spawn((
                        PbrBundle {
                            mesh: meshes.add(Mesh::from(Sphere::new(20.0))),
                            material: materials.add(StandardMaterial {
                                base_color: Color::srgb(1.0, 1.0, 1.0),
                                base_color_texture: Some(asset_server.load("textures/american-flag-background.png")),
                                metallic: 0.8,
                                perceptual_roughness: 0.1,
                                reflectance: 0.7,
                                ..default()
                            }),
                            transform: Transform::from_xyz(0.0, 30.0, 0.0),
                            ..default()
                        },
                        DirigibleBalloon,
                    ));
                });

                // Despawn both the trigger entity and the sphere
                commands.entity(trigger_entity).despawn_recursive();
            }
        }
    }
}

#[derive(Component)]
pub struct TankExitZone {
    pub position: Vec3,
}

// Replace tank exit system
pub fn check_tank_exit(
    mut protagonist_query: Query<(Entity, &Transform, &mut Protagonist, &mut Handle<Scene>)>,
    transform_query: Query<&Transform>,
    tank_exit_query: Query<&TankExitZone>,
    mut commands: Commands,
    children_query: Query<&Children>,
    asset_server: Res<AssetServer>,
    mut message_display: ResMut<MessageDisplay>,
) {
    if let (Ok((entity, transform, mut protagonist, mut scene)), Ok(exit_zone)) = (
        protagonist_query.get_single_mut(),
        tank_exit_query.get_single()
    ) {
        if protagonist.is_driving && transform.translation.distance(exit_zone.position) < 20.0 {
            set_driving_state(
                &mut protagonist,
                &mut scene,
                &asset_server,
                false,  // Set to false to exit tank
                &mut commands,
                entity,
                &children_query,
            );
            display_message("CLIMB THE LADDER", Color::WHITE, &mut message_display);
        }
    }
}
