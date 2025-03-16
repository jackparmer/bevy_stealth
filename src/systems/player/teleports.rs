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

use crate::systems::environments::garage::GARAGE_POSITION_1;

use crate::systems::core::screenplay::{MessageDisplay, display_message};

use crate::systems::environments::maze::MAZE_POSITION;

pub const AIRLOCK_POSITION: Vec3 = Vec3::new(779.1837, 2.6249955, -423.27768);
pub const GAME_START: Vec3 = Vec3::new(165.00298, 2.6249952, -150.00085);
pub const OUTSIDE: Vec3 = Vec3::new(1102.4198, 2.6249733, -456.37585);
pub const REACTOR: Vec3 = Vec3::new(-910.0, 1.6, 1830.0);
pub const ICE_CAVE_POSITION: Vec3 = Vec3::new(-857.4748, -117.847946, 1850.5758);

use crate::components::Protagonist;

pub const UNDERWATER_PIPE: Vec3 = Vec3::new(-8447.827, -484.835, 10928.124); // Slightly above the pipe for visibility

pub const OUTER_RIM: Vec3 = Vec3::new(-5996.718, 3.992052, 669.30194);

pub const PLATFORM_POSITION: Vec3 = Vec3::new(2195.3164, 2.6249852, 6492.4253);

pub fn teleport_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Protagonist>>,
    mut message_display: ResMut<MessageDisplay>,
) {
    let location = if keyboard.just_pressed(KeyCode::Digit1) {
        Some((GAME_START, "Start", Color::WHITE))
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        Some((AIRLOCK_POSITION, "Airlock", Color::WHITE))
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        let pos = Vec3::new(-455.44263, 3.0, 914.4542);
        Some((pos, "North Area", Color::WHITE))
    } else if keyboard.just_pressed(KeyCode::Digit4) {
        Some((ICE_CAVE_POSITION, "Ice Cave", Color::srgb(0.0, 0.8, 1.0)))
    } else if keyboard.just_pressed(KeyCode::Digit5) {
        let mut pos = RADIO_TOWER_POSITION;
        pos.y += RADIO_TOWER_HEIGHT / 2.0 + 10.0;
        pos.x += RADIO_TOWER_WIDTH / 2.0;
        pos.z += RADIO_TOWER_WIDTH / 2.0;
        Some((pos, "Radio Tower", Color::WHITE))
    } else if keyboard.just_pressed(KeyCode::Digit6) {
        let mut pos = LADDER_START;
        pos.y += GEOTHERMAL_BASE_HEIGHT + 20.0;
        pos.z += LADDER_WIDTH / 2.0;
        Some((pos, "Ladder", Color::WHITE))
    } else if keyboard.just_pressed(KeyCode::Digit8) {
        let pos = Vec3::new(40.0, WALL_Y_POSITION + WALL_HEIGHT / 2.0, 0.0);
        Some((pos, "Wall", Color::WHITE))
    } else if keyboard.just_pressed(KeyCode::Digit9) {
        Some((OUTSIDE, "Outside", Color::srgb(0.0, 0.8, 0.2)))
    } else if keyboard.just_pressed(KeyCode::Digit0) {
        let pos = Vec3::new(4376.0146, 3.992052, 7767.32);
        Some((pos, "Reactor Top", Color::srgb(1.0, 0.2, 0.2)))
    } else if keyboard.just_pressed(KeyCode::KeyG) {
        Some((GARAGE_POSITION_1, "Garage", Color::srgb(0.01, 0.55, 0.99)))
    } else if keyboard.just_pressed(KeyCode::KeyH) {
        let pos = Vec3::new(-718.20593, 2.6249816, -2007.5729);
        Some((pos, "Ice Cave Entrance", Color::srgb(0.0, 0.8, 1.0)))
    } else if keyboard.just_pressed(KeyCode::KeyU) {
        Some((UNDERWATER_PIPE, "Underwater Pipe", Color::srgb(0.85, 0.53, 0.35)))
    } else if keyboard.just_pressed(KeyCode::KeyM) {
        Some((MAZE_POSITION + Vec3::new(0.0, 50.0, 0.0), "L-System Maze", Color::srgb(0.8, 0.2, 0.0)))
    } else if keyboard.just_pressed(KeyCode::KeyO) {
        Some((OUTER_RIM, "Outer Rim", Color::srgb(0.7, 0.7, 1.0)))
    } else if keyboard.just_pressed(KeyCode::KeyP) {
        Some((PLATFORM_POSITION, "Platform", Color::srgb(0.7, 0.7, 1.0)))
    } else {
        None
    };

    if let Some((pos, location_name, color)) = location {
        if let Ok(mut transform) = query.get_single_mut() {
            transform.translation = pos;
            println!("Teleported protagonist to {location_name} at {pos:?}");
            display_message(format!("TELEPORTED TO {}", location_name), color, &mut message_display);
        }
    }
}

