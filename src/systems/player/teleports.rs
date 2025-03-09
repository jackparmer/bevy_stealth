use bevy::prelude::*;
use crate::systems::environments::geothermal::{
    RADIO_TOWER_POSITION,
    // Heights
    GEOTHERMAL_BASE_HEIGHT,
    RADIO_TOWER_HEIGHT,
    // Widths
    RADIO_TOWER_WIDTH,
};

use crate::systems::environments::launch_silo::{
    WALL_Y_POSITION,
    WALL_HEIGHT,
};

use crate::systems::environments::ladder::{
    LADDER_START,
    LADDER_WIDTH,
};

use crate::systems::environments::reactor::{CYLINDER_HEIGHT, WALL_THICKNESS};

use crate::systems::environments::maze::{
    CELL_SIZE,
    CATWALK_HEIGHT_OFFSET,
    CATWALK_THICKNESS,
};

use crate::systems::environments::garage::GARAGE_POSITION;

pub const AIRLOCK_POSITION: Vec3 = Vec3::new(779.1837, 2.6249955, -423.27768);
pub const GAME_START: Vec3 = Vec3::new(165.00298, 2.6249952, -150.00085);
pub const OUTSIDE: Vec3 = Vec3::new(1102.4198, 2.6249733, -456.37585);
pub const REACTOR: Vec3 = Vec3::new(-910.0, 1.6, 1830.0);
pub const ICE_CAVE_POSITION: Vec3 = Vec3::new(-857.4748, -117.847946, 1850.5758);

pub static MAZE_START: Vec3 = Vec3::new(
    2700.0 + CELL_SIZE,  // X position (first cell)
    2.625 + CATWALK_HEIGHT_OFFSET + CATWALK_THICKNESS, // Removed WALL_HEIGHT, adjusted for player height
    2200.0 + CELL_SIZE   // Z position (first cell)
);

use crate::components::Protagonist;

pub fn teleport_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Protagonist>>,
) {
    let location = if keyboard.just_pressed(KeyCode::Digit1) {
        Some((GAME_START, "Start"))
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        Some((AIRLOCK_POSITION, "Airlock"))
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        let pos = Vec3::new(-455.44263, 3.0, 914.4542);
        Some((pos, "North Area"))
    } else if keyboard.just_pressed(KeyCode::Digit4) {
        Some((ICE_CAVE_POSITION, "Ice Cave"))
    } else if keyboard.just_pressed(KeyCode::Digit5) {
        let mut pos = RADIO_TOWER_POSITION;
        pos.y += RADIO_TOWER_HEIGHT / 2.0 + 10.0;
        pos.x += RADIO_TOWER_WIDTH / 2.0;
        pos.z += RADIO_TOWER_WIDTH / 2.0;
        Some((pos, "Radio Tower"))
    } else if keyboard.just_pressed(KeyCode::Digit6) {
        let mut pos = LADDER_START;
        pos.y += GEOTHERMAL_BASE_HEIGHT + 20.0;
        pos.z += LADDER_WIDTH / 2.0;
        Some((pos, "Ladder"))
    } else if keyboard.just_pressed(KeyCode::Digit8) {
        let pos = Vec3::new(
            40.0, 
            WALL_Y_POSITION + WALL_HEIGHT / 2.0, 
            0.0);
        Some((pos, "Wall"))
    } else if keyboard.just_pressed(KeyCode::Digit9) {
        Some((OUTSIDE, "Outside"))
    } else if keyboard.just_pressed(KeyCode::Digit0) {
        let mut pos = REACTOR;
        pos.y += CYLINDER_HEIGHT/2.0 + WALL_THICKNESS + 10.0;
        Some((pos, "Reactor Top"))
    } else if keyboard.just_pressed(KeyCode::KeyM) {
        Some((MAZE_START, "Maze"))
    } else if keyboard.just_pressed(KeyCode::KeyG) {
        Some((GARAGE_POSITION, "Garage"))
    } else if keyboard.just_pressed(KeyCode::KeyH) {
        let pos = Vec3::new(-718.20593, 2.6249816, -2007.5729);
        Some((pos, "Ice Cave Entrance"))
    } else {
        None
    };

    if let Some((pos, location_name)) = location {
        if let Ok(mut transform) = query.get_single_mut() {
            transform.translation = pos;
            println!("Teleported protagonist to {location_name} at {pos:?}");
        }
    }
}

