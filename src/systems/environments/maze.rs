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
const MAZE_SIZE: usize = 8;

// Add catwalk configuration
const CATWALK_HEIGHT_OFFSET: f32 = 10.0; // Height above walls
const CATWALK_WIDTH: f32 = WALL_THICKNESS;
const CATWALK_THICKNESS: f32 = 2.0;

// Add neon configuration
const NEON_WIDTH: f32 = 0.5;
const NEON_HEIGHT: f32 = 0.3;
const NEON_OFFSET: f32 = CATWALK_WIDTH/2.0 + NEON_WIDTH/2.0;

// Add ladder configuration
const LADDER_STRUCTURE_WIDTH: f32 = 4.0; // Width of the ladder structure

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

pub fn spawn_maze(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut rng = StdRng::seed_from_u64(42);
    let maze = generate_maze(&mut rng);
    
    let wall_material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/concrete.png")),
        perceptual_roughness: 0.95,
        metallic: 0.1,
        ..default()
    });

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

    // Spawn visual meshes for walls
    for row in 0..MAZE_SIZE-1 {
        for col in 0..MAZE_SIZE-1 {
            let x = MAZE_POSITION.x + (col as f32 * CELL_SIZE);
            let z = MAZE_POSITION.z + (row as f32 * CELL_SIZE);

            // Horizontal wall visuals
            if maze[row][col] == Cell::Wall && maze[row][col+1] == Cell::Wall {
                commands.spawn(PbrBundle {
                    mesh: meshes.add(Cuboid::new(
                        CELL_SIZE - WALL_THICKNESS,
                        WALL_HEIGHT,
                        WALL_THICKNESS,
                    )),
                    material: wall_material.clone(),
                    transform: Transform::from_xyz(
                        x + CELL_SIZE/2.0, 
                        MAZE_POSITION.y + WALL_HEIGHT/2.0, 
                        z
                    ),
                    ..default()
                });
            }

            // Vertical wall visuals
            if maze[row][col] == Cell::Wall && maze[row+1][col] == Cell::Wall {
                commands.spawn(PbrBundle {
                    mesh: meshes.add(Cuboid::new(
                        WALL_THICKNESS,
                        WALL_HEIGHT,
                        CELL_SIZE - WALL_THICKNESS,
                    )),
                    material: wall_material.clone(),
                    transform: Transform::from_xyz(
                        x, 
                        MAZE_POSITION.y + WALL_HEIGHT/2.0, 
                        z + CELL_SIZE/2.0
                    ),
                    ..default()
                });
            }

            // Corner posts visuals
            if maze[row][col] == Cell::Wall {
                commands.spawn(PbrBundle {
                    mesh: meshes.add(Cuboid::new(
                        WALL_THICKNESS,
                        WALL_HEIGHT,
                        WALL_THICKNESS,
                    )),
                    material: wall_material.clone(),
                    transform: Transform::from_xyz(x, MAZE_POSITION.y + WALL_HEIGHT/2.0, z),
                    ..default()
                });
            }
        }
    }

    // Spawn consolidated horizontal wall colliders
    for (row, start_col, end_col) in find_horizontal_segments(&maze) {
        let segment_length = (end_col - start_col) as f32 * CELL_SIZE;
        let start_x = MAZE_POSITION.x + (start_col as f32 * CELL_SIZE);
        let z = MAZE_POSITION.z + (row as f32 * CELL_SIZE);

        commands.spawn((
            TransformBundle::from(Transform::from_xyz(
                start_x + segment_length/2.0,
                MAZE_POSITION.y + WALL_HEIGHT/2.0,
                z
            )),
            RigidBody::Static,
            Collider::cuboid(
                segment_length,
                WALL_HEIGHT,
                WALL_THICKNESS + 4.0
            ),
        ));
    }

    // Spawn consolidated vertical wall colliders
    for (col, start_row, end_row) in find_vertical_segments(&maze) {
        let segment_length = (end_row - start_row + 1) as f32 * CELL_SIZE;
        let x = MAZE_POSITION.x + (col as f32 * CELL_SIZE);
        let start_z = MAZE_POSITION.z + (start_row as f32 * CELL_SIZE);

        commands.spawn((
            TransformBundle::from(Transform::from_xyz(
                x,
                MAZE_POSITION.y + WALL_HEIGHT/2.0,
                start_z + segment_length/2.0
            )),
            RigidBody::Static,
            Collider::cuboid(
                WALL_THICKNESS + 4.0,
                WALL_HEIGHT,
                segment_length
            ),
        ));
    }

    let catwalk_y = MAZE_POSITION.y + WALL_HEIGHT + CATWALK_HEIGHT_OFFSET;

    // Spawn horizontal catwalks with neon strips
    for (row, start_col, end_col) in find_horizontal_segments(&maze) {
        let segment_length = (end_col - start_col + 1) as f32 * CELL_SIZE;
        let start_x = MAZE_POSITION.x + (start_col as f32 * CELL_SIZE);
        let z = MAZE_POSITION.z + (row as f32 * CELL_SIZE);

        // Main catwalk spawn
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(
                    segment_length,
                    CATWALK_THICKNESS,
                    CATWALK_WIDTH,
                )),
                material: catwalk_material.clone(),
                transform: Transform::from_xyz(
                    start_x + segment_length/2.0,
                    catwalk_y,
                    z
                ),
                ..default()
            },
            RigidBody::Static,
            Collider::cuboid(
                segment_length + 4.0,
                CATWALK_THICKNESS,
                CATWALK_WIDTH + 4.0
            ),
        ));

        // Modified neon strips to stop before corners
        for offset in [-NEON_OFFSET, NEON_OFFSET] {
            let strip_length = segment_length - CATWALK_WIDTH; // Subtract platform width to prevent overlap
            commands.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(
                    strip_length,
                    NEON_HEIGHT,
                    NEON_WIDTH,
                )),
                material: neon_material.clone(),
                transform: Transform::from_xyz(
                    start_x + segment_length/2.0,
                    catwalk_y + CATWALK_THICKNESS/2.0,
                    z + offset
                ),
                ..default()
            });
        }
    }

    // Spawn vertical catwalks with neon strips
    for (col, start_row, end_row) in find_vertical_segments(&maze) {
        let segment_length = (end_row - start_row + 1) as f32 * CELL_SIZE;
        let x = MAZE_POSITION.x + (col as f32 * CELL_SIZE);
        let start_z = MAZE_POSITION.z + (start_row as f32 * CELL_SIZE);

        // Main catwalk spawn
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(
                    CATWALK_WIDTH,
                    CATWALK_THICKNESS,
                    segment_length,
                )),
                material: catwalk_material.clone(),
                transform: Transform::from_xyz(
                    x,
                    catwalk_y,
                    start_z + segment_length/2.0
                ),
                ..default()
            },
            RigidBody::Static,
            Collider::cuboid(
                CATWALK_WIDTH + 4.0,
                CATWALK_THICKNESS,
                segment_length + 4.0
            ),
        ));

        // Modified neon strips to stop before corners
        for offset in [-NEON_OFFSET, NEON_OFFSET] {
            let strip_length = segment_length - CATWALK_WIDTH; // Subtract platform width to prevent overlap
            commands.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(
                    NEON_WIDTH,
                    NEON_HEIGHT,
                    strip_length,
                )),
                material: neon_material.clone(),
                transform: Transform::from_xyz(
                    x + offset,
                    catwalk_y + CATWALK_THICKNESS/2.0,
                    start_z + segment_length/2.0
                ),
                ..default()
            });
        }
    }

    // Remove the intersection platform neon strips since they're no longer needed
    for row in 0..MAZE_SIZE-1 {
        for col in 0..MAZE_SIZE-1 {
            if maze[row][col] == Cell::Wall {
                let x = MAZE_POSITION.x + (col as f32 * CELL_SIZE);
                let z = MAZE_POSITION.z + (row as f32 * CELL_SIZE);
                
                // Spawn only the platform without neon strips
                commands.spawn(PbrBundle {
                    mesh: meshes.add(Cuboid::new(
                        CATWALK_WIDTH,
                        CATWALK_THICKNESS,
                        CATWALK_WIDTH,
                    )),
                    material: catwalk_material.clone(),
                    transform: Transform::from_xyz(x, catwalk_y, z),
                    ..default()
                });
            }
        }
    }

    // Add ground-to-wall ladders at wall ends
    for row in 0..MAZE_SIZE-1 {
        for col in 0..MAZE_SIZE-1 {
            if maze[row][col] == Cell::Wall {
                let x = MAZE_POSITION.x + (col as f32 * CELL_SIZE);
                let z = MAZE_POSITION.z + (row as f32 * CELL_SIZE);

                // Check if this wall has a path next to it (to place ladder against the wall)
                let has_path_north = row > 0 && maze[row-1][col] == Cell::Path;
                let has_path_south = row < MAZE_SIZE-1 && maze[row+1][col] == Cell::Path;
                let has_path_east = col < MAZE_SIZE-1 && maze[row][col+1] == Cell::Path;
                let has_path_west = col > 0 && maze[row][col-1] == Cell::Path;

                // Calculate total ladder height to reach catwalks (including catwalk thickness)
                let ladder_height = WALL_HEIGHT + CATWALK_HEIGHT_OFFSET + CATWALK_THICKNESS;

                // Spawn ladders against walls where there's a path
                if has_path_north {
                    spawn_ladder(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &asset_server,
                        LadderConfig {
                            position: Vec3::new(x, MAZE_POSITION.y, z - WALL_THICKNESS/2.0 - LADDER_STRUCTURE_WIDTH/2.0),
                            rotation: Quat::from_rotation_y(std::f32::consts::PI * 0.5),
                            height: ladder_height,
                            rung_count: 150,
                        },
                    );
                }
                if has_path_south {
                    spawn_ladder(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &asset_server,
                        LadderConfig {
                            position: Vec3::new(x, MAZE_POSITION.y, z + WALL_THICKNESS/2.0 + LADDER_STRUCTURE_WIDTH/2.0),
                            rotation: Quat::from_rotation_y(std::f32::consts::PI * 1.5),
                            height: ladder_height,
                            rung_count: 150,
                        },
                    );
                }
                if has_path_east {
                    spawn_ladder(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &asset_server,
                        LadderConfig {
                            position: Vec3::new(x + WALL_THICKNESS/2.0 + LADDER_STRUCTURE_WIDTH/2.0, MAZE_POSITION.y, z),
                            rotation: Quat::from_rotation_y(0.0),
                            height: ladder_height,
                            rung_count: 150,
                        },
                    );
                }
                if has_path_west {
                    spawn_ladder(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &asset_server,
                        LadderConfig {
                            position: Vec3::new(x - WALL_THICKNESS/2.0 - LADDER_STRUCTURE_WIDTH/2.0, MAZE_POSITION.y, z),
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
    let mut stack = Vec::new();
    
    // Start at a random position
    let start_x = 1;
    let start_y = 1;
    maze[start_y][start_x] = Cell::Path;
    stack.push((start_x, start_y));

    while let Some(&(current_x, current_y)) = stack.last() {
        let directions = vec![
            (0, -2), // North
            (2, 0),  // East
            (0, 2),  // South
            (-2, 0), // West
        ];

        let valid_moves: Vec<_> = directions.iter()
            .filter(|&&(dx, dy)| {
                let new_x = current_x as i32 + dx;
                let new_y = current_y as i32 + dy;
                new_x > 0 && new_x < (MAZE_SIZE - 1) as i32 &&
                new_y > 0 && new_y < (MAZE_SIZE - 1) as i32 &&
                maze[new_y as usize][new_x as usize] == Cell::Wall
            })
            .collect();

        if valid_moves.is_empty() {
            stack.pop();
            continue;
        }

        let &(dx, dy) = valid_moves.choose(rng).unwrap();
        let new_x = (current_x as i32 + dx) as usize;
        let new_y = (current_y as i32 + dy) as usize;
        
        // Carve the path
        maze[new_y][new_x] = Cell::Path;
        maze[(current_y + new_y) / 2][(current_x + new_x) / 2] = Cell::Path;
        
        stack.push((new_x, new_y));
    }

    // Create an opening at the edge
    // Choose a random edge cell that's adjacent to a path
    let edge_candidates: Vec<(usize, usize)> = (1..MAZE_SIZE-1)
        .flat_map(|i| vec![
            (i, 0),           // North edge
            (i, MAZE_SIZE-1), // South edge
            (0, i),           // West edge
            (MAZE_SIZE-1, i)  // East edge
        ])
        .filter(|&(x, y)| {
            // Check if adjacent cell is a path
            let has_adjacent_path = match (x, y) {
                (_, 0) => maze[1][x] == Cell::Path,           // North edge
                (_, y) if y == MAZE_SIZE-1 => maze[MAZE_SIZE-2][x] == Cell::Path, // South edge
                (0, y) => maze[y][1] == Cell::Path,           // West edge
                (x, y) if x == MAZE_SIZE-1 => maze[y][MAZE_SIZE-2] == Cell::Path, // East edge
                _ => false
            };
            has_adjacent_path
        })
        .collect();

    if let Some(&(x, y)) = edge_candidates.choose(rng) {
        maze[y][x] = Cell::Path;
    }

    maze
}

