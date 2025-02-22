use bevy::prelude::*;
use avian3d::prelude::*;
use rand::prelude::*;
use crate::systems::environments::ladder::{spawn_ladder, LadderConfig};

// Maze configuration
pub const MAZE_POSITION: Vec3 = Vec3::new(
    2700.0,  // 2500 + 200
    2.625,   // Same height
    2200.0   // 2000 + 200
);
pub const CELL_SIZE: f32 = 40.0;
const WALL_HEIGHT: f32 = 100.0;
const WALL_THICKNESS: f32 = 24.0;
const MAZE_SIZE: usize = 20;

// Add catwalk configuration
pub const CATWALK_HEIGHT_OFFSET: f32 = 10.0;
pub const CATWALK_WIDTH: f32 = WALL_THICKNESS;
pub const CATWALK_THICKNESS: f32 = 2.0;

// Add neon configuration
const NEON_WIDTH: f32 = 0.5;
const NEON_HEIGHT: f32 = 0.3;
const NEON_OFFSET: f32 = CATWALK_WIDTH/2.0 + NEON_WIDTH/2.0;

// Add ladder configuration
const LADDER_WALL_OFFSET: f32 = 2.0; // Reduced offset from wall

// Add new constants for sphere and arrows (updated sizes)
const SPHERE_RADIUS: f32 = 20.0;
const ARROW_LENGTH: f32 = 1.5;  // Keep current length
const ARROW_WIDTH: f32 = 0.75;  // Keep current width
const ARROW_HEAD_LENGTH: f32 = 1.0;   // Increased from 0.5
const ARROW_HEAD_WIDTH: f32 = 2.0;    // Increased from 1.0

// Add height variation constants
const MAX_WALL_HEIGHT: f32 = 200.0; // Increased from 200.0
const MIN_WALL_HEIGHT: f32 = 80.0;  // Increased from 30.0

#[derive(Clone, Copy, PartialEq)]
enum Cell {
    Wall,
    Path,
}

// Helper function to find continuous horizontal segments
fn find_horizontal_segments(maze: &Vec<Vec<Cell>>) -> Vec<(usize, usize, usize)> {
    let mut segments = Vec::new();
    
    for row in 0..MAZE_SIZE-1 {
        let mut start_col = None;
        
        for col in 0..MAZE_SIZE-1 {
            if maze[row][col] == Cell::Wall && maze[row][col+1] == Cell::Wall {
                if start_col.is_none() {
                    start_col = Some(col);
                }
            } else if let Some(start) = start_col {
                segments.push((row, start, col));
                start_col = None;
            }
        }
        
        // Handle segment that extends to edge
        if let Some(start) = start_col {
            segments.push((row, start, MAZE_SIZE-1));
        }
    }
    
    segments
}

// Helper function to find continuous vertical segments
fn find_vertical_segments(maze: &Vec<Vec<Cell>>) -> Vec<(usize, usize, usize)> {
    let mut segments = Vec::new();
    
    for col in 0..MAZE_SIZE-1 {
        let mut start_row = None;
        
        for row in 0..MAZE_SIZE-1 {
            if maze[row][col] == Cell::Wall && maze[row+1][col] == Cell::Wall {
                if start_row.is_none() {
                    start_row = Some(row);
                }
            } else if let Some(start) = start_row {
                segments.push((col, start, row));
                start_row = None;
            }
        }
        
        // Handle segment that extends to edge
        if let Some(start) = start_row {
            segments.push((col, start, MAZE_SIZE-1));
        }
    }
    
    segments
}

// Helper function to calculate wall height based on position
fn get_wall_height(x: f32, z: f32) -> f32 {
    let normalized_x = x - MAZE_POSITION.x;
    let normalized_z = z - MAZE_POSITION.z;
    let max_distance = CELL_SIZE * MAZE_SIZE as f32;
    
    // Calculate distance from (0,0) as a percentage
    let distance_factor = ((normalized_x.powi(2) + normalized_z.powi(2)).sqrt()) / max_distance;
    
    // Use a more dramatic curve for height variation (power of 0.5 creates more extreme variation)
    MAX_WALL_HEIGHT - (MAX_WALL_HEIGHT - MIN_WALL_HEIGHT) * distance_factor.powf(0.5)
}

pub fn spawn_maze(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut rng = StdRng::seed_from_u64(42);
    let maze = generate_maze(&mut rng);
    
    let wall_material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/ice_texture3.png")),
        perceptual_roughness: 0.1,
        metallic: 0.8,
        ..default()
    });

    // Find continuous wall segments
    let h_segments = find_horizontal_segments(&maze);
    let v_segments = find_vertical_segments(&maze);

    // Spawn horizontal wall segments
    for (row, start_col, end_col) in &h_segments {
        let segment_length = (*end_col - *start_col + 1) as f32 * CELL_SIZE;
        let x = MAZE_POSITION.x + (*start_col as f32 * CELL_SIZE) + segment_length/2.0;
        let z = MAZE_POSITION.z + (*row as f32 * CELL_SIZE);
        
        let wall_height = get_wall_height(x, z);
        
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(segment_length, wall_height, WALL_THICKNESS)),
                material: wall_material.clone(),
                transform: Transform::from_xyz(x, MAZE_POSITION.y + wall_height/2.0, z),
                ..default()
            },
            RigidBody::Static,
            Collider::cuboid(segment_length, wall_height, WALL_THICKNESS),
        ));
    }

    // Spawn vertical wall segments
    for (col, start_row, end_row) in &v_segments {
        let segment_length = (*end_row - *start_row + 1) as f32 * CELL_SIZE;
        let x = MAZE_POSITION.x + (*col as f32 * CELL_SIZE);
        let z = MAZE_POSITION.z + (*start_row as f32 * CELL_SIZE) + segment_length/2.0;
        
        let wall_height = get_wall_height(x, z);
        
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(WALL_THICKNESS, wall_height, segment_length)),
                material: wall_material.clone(),
                transform: Transform::from_xyz(x, MAZE_POSITION.y + wall_height/2.0, z),
                ..default()
            },
            RigidBody::Static,
            Collider::cuboid(WALL_THICKNESS, wall_height, segment_length),
        ));
    }

    // Catwalk material
    let catwalk_material = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        perceptual_roughness: 0.9,
        metallic: 0.1,
        ..default()
    });

    // Add neon material
    let neon_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.0, 1.0), // Blue color
        emissive: Color::srgb(0.0, 0.0, 1.0).into(),   // Strong blue glow
        perceptual_roughness: 0.0,
        metallic: 1.0,
        ..default()
    });

    let catwalk_y = MAZE_POSITION.y + WALL_HEIGHT + CATWALK_HEIGHT_OFFSET;

    // Find all segments first
    let h_segments = find_horizontal_segments(&maze);
    let v_segments = find_vertical_segments(&maze);
    
    // Create sphere material with dirigible texture
    let sphere_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(asset_server.load("textures/american-flag-background.png")),
        metallic: 0.8,
        perceptual_roughness: 0.1,
        reflectance: 0.7,
        ..default()
    });

    // Create arrow material
    let arrow_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 0.0),
        emissive: Color::srgb(1.0, 1.0, 0.0).into(),
        metallic: 0.0,
        perceptual_roughness: 0.1,
        ..default()
    });

    // Place sphere at opposite corner (MAZE_SIZE-2, MAZE_SIZE-2) instead of (1,1)
    let end_point = (MAZE_SIZE-2, MAZE_SIZE-2);

    // Place sphere regardless of path existence
    let sphere_x = MAZE_POSITION.x + (end_point.0 as f32 * CELL_SIZE);
    let sphere_z = MAZE_POSITION.z + (end_point.1 as f32 * CELL_SIZE);
    let sphere_y = catwalk_y + SPHERE_RADIUS * 1.2;
    let sphere_pos = Vec3::new(sphere_x, sphere_y, sphere_z);

    commands.spawn(PbrBundle {
        mesh: meshes.add(Sphere::new(SPHERE_RADIUS)),
        material: sphere_material,
        transform: Transform::from_translation(sphere_pos),
        ..default()
    });

    // Update sphere target position for arrows
    let sphere_target = Vec3::new(
        MAZE_POSITION.x + (end_point.0 as f32 * CELL_SIZE),
        catwalk_y + CATWALK_THICKNESS/2.0 + 0.01,
        MAZE_POSITION.z + (end_point.1 as f32 * CELL_SIZE)
    );

    // For horizontal segments
    let mut processed_h_segments = std::collections::HashSet::new();

    for (row, start_col, end_col) in &h_segments {
        // Skip if we've already processed this segment as part of a bridge
        if processed_h_segments.contains(&(*row, *start_col)) {
            continue;
        }

        let current_end = *end_col;
        let mut total_length = (*end_col - *start_col + 1) as f32 * CELL_SIZE;
        
        // Look ahead for segments we can bridge to
        let mut next_col = current_end + 1;
        while next_col < MAZE_SIZE {
            if let Some((_, bridge_start, bridge_end)) = h_segments.iter()
                .find(|&&(r, s, _)| r == *row && s == next_col) {
                // Found a segment to bridge to
                total_length += (*bridge_end - *bridge_start + 2) as f32 * CELL_SIZE;
                next_col = *bridge_end + 1;
                processed_h_segments.insert((*row, *bridge_start));
            } else {
                break;
            }
        }

        let start_x = MAZE_POSITION.x + (*start_col as f32 * CELL_SIZE);
        let z = MAZE_POSITION.z + (*row as f32 * CELL_SIZE);

        // Update catwalk spawning for horizontal segments
        let wall_height = get_wall_height(
            start_x + total_length/2.0,
            z
        );
        let catwalk_y = wall_height + MAZE_POSITION.y + CATWALK_HEIGHT_OFFSET;

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(
                    total_length + CATWALK_WIDTH,
                    CATWALK_THICKNESS,
                    CATWALK_WIDTH,
                )),
                material: catwalk_material.clone(),
                transform: Transform::from_xyz(
                    start_x + total_length/2.0,
                    catwalk_y,
                    z
                ),
                ..default()
            },
            RigidBody::Static,
            Collider::cuboid(
                total_length + CATWALK_WIDTH,
                CATWALK_THICKNESS,
                CATWALK_WIDTH
            ),
        ));

        // Update neon strips height
        for offset in [-NEON_OFFSET, NEON_OFFSET] {
            commands.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(
                    total_length + CATWALK_WIDTH,
                    NEON_HEIGHT,
                    NEON_WIDTH,
                )),
                material: neon_material.clone(),
                transform: Transform::from_xyz(
                    start_x + total_length/2.0,
                    catwalk_y + CATWALK_THICKNESS/2.0,
                    z + offset
                ),
                ..default()
            });
        }

        // In the horizontal segments loop, update arrow spawning:
        let segment_center = Vec3::new(
            start_x + total_length/2.0,
            catwalk_y + CATWALK_THICKNESS/2.0 + 0.01,
            z
        );

        // Point directly to sphere instead of path
        let mut direction = (sphere_target - segment_center).normalize();
        direction.y = 0.0; // Force Y component to zero
        direction = direction.normalize();

        let rotation = Quat::from_rotation_arc(Vec3::X, direction);

        // Spawn arrow shaft flush with surface
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(ARROW_LENGTH, 0.01, ARROW_WIDTH)),
            material: arrow_material.clone(),
            transform: Transform::from_translation(segment_center)
                .with_rotation(rotation),
            ..default()
        });

        // Spawn arrow head flush with surface
        commands.spawn(PbrBundle {
            mesh: meshes.add(Triangle3d::new(
                Vec3::new(ARROW_HEAD_LENGTH, 0.0, -ARROW_HEAD_WIDTH/2.0),
                Vec3::new(ARROW_HEAD_LENGTH, 0.0, ARROW_HEAD_WIDTH/2.0),
                Vec3::new(ARROW_HEAD_LENGTH + ARROW_HEAD_LENGTH, 0.0, 0.0)
            )),
            material: arrow_material.clone(),
            transform: Transform::from_translation(segment_center)
                .with_rotation(rotation),
            ..default()
        });
    }

    // For vertical segments
    let mut processed_v_segments = std::collections::HashSet::new();

    for (col, start_row, end_row) in &v_segments {
        // Skip if we've already processed this segment as part of a bridge
        if processed_v_segments.contains(&(*col, *start_row)) {
            continue;
        }

        let current_end = *end_row;
        let mut total_length = (*end_row - *start_row + 1) as f32 * CELL_SIZE;
        
        // Look ahead for segments we can bridge to
        let mut next_row = current_end + 1;
        while next_row < MAZE_SIZE {
            if let Some((bridge_col, bridge_start, bridge_end)) = v_segments.iter()
                .find(|&&(c, s, _)| c == *col && s == next_row) {
                // Found a segment to bridge to
                total_length += (*bridge_end - *bridge_start + 2) as f32 * CELL_SIZE;
                next_row = *bridge_end + 1;
                processed_v_segments.insert((*bridge_col, *bridge_start));
            } else {
                break;
            }
        }

        let x = MAZE_POSITION.x + (*col as f32 * CELL_SIZE);
        let start_z = MAZE_POSITION.z + (*start_row as f32 * CELL_SIZE);

        // Spawn single continuous catwalk with one collider
        let wall_height = get_wall_height(x, start_z + total_length/2.0);
        let catwalk_y = wall_height + MAZE_POSITION.y + CATWALK_HEIGHT_OFFSET;

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(
                    CATWALK_WIDTH,
                    CATWALK_THICKNESS,
                    total_length + CATWALK_WIDTH,
                )),
                material: catwalk_material.clone(),
                transform: Transform::from_xyz(
                    x,
                    catwalk_y,
                    start_z + total_length/2.0
                ),
                ..default()
            },
            RigidBody::Static,
            Collider::cuboid(
                CATWALK_WIDTH,
                CATWALK_THICKNESS,
                total_length + CATWALK_WIDTH
            ),
        ));

        // Single continuous neon strips
        for offset in [-NEON_OFFSET, NEON_OFFSET] {
            commands.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(
                    NEON_WIDTH,
                    NEON_HEIGHT,
                    total_length + CATWALK_WIDTH,
                )),
                material: neon_material.clone(),
                transform: Transform::from_xyz(
                    x + offset,
                    catwalk_y + CATWALK_THICKNESS/2.0,
                    start_z + total_length/2.0
                ),
                ..default()
            });
        }

        // Similarly for vertical segments:
        let segment_center = Vec3::new(
            x,
            catwalk_y + CATWALK_THICKNESS/2.0 + 0.01,
            start_z + total_length/2.0
        );

        // Point directly to sphere
        let mut direction = (sphere_target - segment_center).normalize();
        direction.y = 0.0;
        direction = direction.normalize();

        let rotation = Quat::from_rotation_arc(Vec3::X, direction);

        // Spawn arrow shaft flush with surface
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(ARROW_LENGTH, 0.01, ARROW_WIDTH)),
            material: arrow_material.clone(),
            transform: Transform::from_translation(segment_center)
                .with_rotation(rotation),
            ..default()
        });

        // Spawn arrow head flush with surface
        commands.spawn(PbrBundle {
            mesh: meshes.add(Triangle3d::new(
                Vec3::new(ARROW_HEAD_LENGTH, 0.0, -ARROW_HEAD_WIDTH/2.0),
                Vec3::new(ARROW_HEAD_LENGTH, 0.0, ARROW_HEAD_WIDTH/2.0),
                Vec3::new(ARROW_HEAD_LENGTH + ARROW_HEAD_LENGTH, 0.0, 0.0)
            )),
            material: arrow_material.clone(),
            transform: Transform::from_translation(segment_center)
                .with_rotation(rotation),
            ..default()
        });
    }

    // Add ground-to-wall ladders at wall ends, but only on certain conditions
    for row in 0..MAZE_SIZE-1 {
        for col in 0..MAZE_SIZE-1 {
            // Changed from % 300 to % 5 to spawn many more ladders
            if (row + col) % 5 != 0 {
                continue;
            }

            if maze[row][col] == Cell::Wall {
                let x = MAZE_POSITION.x + (col as f32 * CELL_SIZE);
                let z = MAZE_POSITION.z + (row as f32 * CELL_SIZE);

                // Check if this wall has a path next to it
                let has_path_north = row > 0 && maze[row-1][col] == Cell::Path;
                let has_path_south = row < MAZE_SIZE-1 && maze[row+1][col] == Cell::Path;
                let has_path_east = col < MAZE_SIZE-1 && maze[row][col+1] == Cell::Path;
                let has_path_west = col > 0 && maze[row][col-1] == Cell::Path;

                // Calculate total ladder height
                let ladder_height = WALL_HEIGHT + CATWALK_HEIGHT_OFFSET + CATWALK_THICKNESS;

                // Only spawn ladders if there's a path on one side and walls on the adjacent sides
                // This reduces the number of ladders and places them in more strategic locations
                if has_path_north && !has_path_east && !has_path_west {
                    let wall_height = get_wall_height(x, z);
                    let ladder_height = wall_height + CATWALK_HEIGHT_OFFSET + CATWALK_THICKNESS;
                    spawn_ladder(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &asset_server,
                        LadderConfig {
                            position: Vec3::new(x, MAZE_POSITION.y, z - WALL_THICKNESS/2.0 - LADDER_WALL_OFFSET),
                            rotation: Quat::from_rotation_y(std::f32::consts::PI * 0.5),
                            height: ladder_height,
                            rung_count: 150,
                        },
                    );
                }
                if has_path_south && !has_path_east && !has_path_west {
                    let wall_height = get_wall_height(x, z);
                    let ladder_height = wall_height + CATWALK_HEIGHT_OFFSET + CATWALK_THICKNESS;
                    spawn_ladder(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &asset_server,
                        LadderConfig {
                            position: Vec3::new(x, MAZE_POSITION.y, z + WALL_THICKNESS/2.0 + LADDER_WALL_OFFSET),
                            rotation: Quat::from_rotation_y(std::f32::consts::PI * 1.5),
                            height: ladder_height,
                            rung_count: 150,
                        },
                    );
                }
                if has_path_east && !has_path_north && !has_path_south {
                    let wall_height = get_wall_height(x, z);
                    let ladder_height = wall_height + CATWALK_HEIGHT_OFFSET + CATWALK_THICKNESS;
                    spawn_ladder(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &asset_server,
                        LadderConfig {
                            position: Vec3::new(x + WALL_THICKNESS/2.0 + LADDER_WALL_OFFSET, MAZE_POSITION.y, z),
                            rotation: Quat::from_rotation_y(0.0),
                            height: ladder_height,
                            rung_count: 150,
                        },
                    );
                }
                if has_path_west && !has_path_north && !has_path_south {
                    let wall_height = get_wall_height(x, z);
                    let ladder_height = wall_height + CATWALK_HEIGHT_OFFSET + CATWALK_THICKNESS;
                    spawn_ladder(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &asset_server,
                        LadderConfig {
                            position: Vec3::new(x - WALL_THICKNESS/2.0 - LADDER_WALL_OFFSET, MAZE_POSITION.y, z),
                            rotation: Quat::from_rotation_y(std::f32::consts::PI),
                            height: ladder_height,
                            rung_count: 150,
                        },
                    );
                }
            }
        }
    }
}

fn generate_maze(rng: &mut StdRng) -> Vec<Vec<Cell>> {
    let mut maze = vec![vec![Cell::Wall; MAZE_SIZE]; MAZE_SIZE];
    
    // Clear the perimeter walls
    for i in 0..MAZE_SIZE {
        maze[0][i] = Cell::Path;
        maze[MAZE_SIZE-1][i] = Cell::Path;
        maze[i][0] = Cell::Path;
        maze[i][MAZE_SIZE-1] = Cell::Path;
    }
    
    // Start from (1,1)
    maze[1][1] = Cell::Path;
    let mut frontier = Vec::new();
    add_frontiers(1, 1, &maze, &mut frontier);
    
    // Rest of maze generation remains the same
    while !frontier.is_empty() {
        let idx = rng.gen_range(0..frontier.len());
        let (x, y) = frontier.swap_remove(idx);
        
        let mut adjacent_paths = Vec::new();
        for (dx, dy) in &[(0, -2), (2, 0), (0, 2), (-2, 0)] {
            let nx = (x as i32 + dx) as usize;
            let ny = (y as i32 + dy) as usize;
            if nx < MAZE_SIZE && ny < MAZE_SIZE && maze[ny][nx] == Cell::Path {
                adjacent_paths.push((nx, ny));
            }
        }
        
        if let Some(&(px, py)) = adjacent_paths.choose(rng) {
            maze[y][x] = Cell::Path;
            maze[(y + py) / 2][(x + px) / 2] = Cell::Path;
            add_frontiers(x, y, &maze, &mut frontier);
        }
    }

    // Add random loops
    for _ in 0..MAZE_SIZE * 2 {
        let x = rng.gen_range(2..MAZE_SIZE-2);
        let y = rng.gen_range(2..MAZE_SIZE-2);
        if maze[y][x] == Cell::Wall && count_adjacent_paths(&maze, x, y) >= 2 {
            maze[y][x] = Cell::Path;
        }
    }

    maze[1][1] = Cell::Path;
    
    maze
}

fn add_frontiers(x: usize, y: usize, maze: &Vec<Vec<Cell>>, frontier: &mut Vec<(usize, usize)>) {
    for (dx, dy) in &[(0, -2), (2, 0), (0, 2), (-2, 0)] {
        let nx = (x as i32 + dx) as usize;
        let ny = (y as i32 + dy) as usize;
        if nx < MAZE_SIZE-1 && ny < MAZE_SIZE-1 && maze[ny][nx] == Cell::Wall {
            if !frontier.contains(&(nx, ny)) {
                frontier.push((nx, ny));
            }
        }
    }
}

// Helper function to count adjacent path cells
fn count_adjacent_paths(maze: &Vec<Vec<Cell>>, x: usize, y: usize) -> usize {
    let directions = [(0, -1), (1, 0), (0, 1), (-1, 0)];
    directions.iter()
        .filter(|&&(dx, dy)| {
            let new_x = x as i32 + dx;
            let new_y = y as i32 + dy;
            if new_x >= 0 && new_x < MAZE_SIZE as i32 && 
               new_y >= 0 && new_y < MAZE_SIZE as i32 {
                maze[new_y as usize][new_x as usize] == Cell::Path
            } else {
                false
            }
        })
        .count()
}