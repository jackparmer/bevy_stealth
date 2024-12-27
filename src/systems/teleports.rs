use bevy::prelude::*;
use crate::systems::setup::{
    WALL_Y_POSITION,
    GEOTHERMAL_POSITION,
    RADIO_TOWER_POSITION,
    BRIDGE_POSITION,
    TRAM_POSITION,
    // Heights
    GEOTHERMAL_BASE_HEIGHT,
    RADIO_TOWER_HEIGHT,
    BRIDGE_HEIGHT,
    WALL_HEIGHT,
    // Widths
    GEOTHERMAL_BASE_RADIUS,
    RADIO_TOWER_WIDTH,
    BRIDGE_WIDTH,
};

use crate::systems::ladder::{
    LADDER_START,
    LADDER_WIDTH,
};

const AIRLOCK_POSITION: Vec3 = Vec3::new(779.1837, 2.6249955, -423.27768);

use crate::components::Protagonist;

pub fn teleport_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Protagonist>>,
) {
    let location = if keyboard.just_pressed(KeyCode::Digit1) {
        let mut pos = GEOTHERMAL_POSITION;
        pos.y += GEOTHERMAL_BASE_HEIGHT / 2.0;
        pos.x += GEOTHERMAL_BASE_RADIUS / 2.0;
        Some((pos, "Geothermal"))
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        let mut pos = RADIO_TOWER_POSITION;
        pos.y += RADIO_TOWER_HEIGHT / 2.0 + 10.0;
        pos.x += RADIO_TOWER_WIDTH / 2.0;
        pos.z += RADIO_TOWER_WIDTH / 2.0;
        Some((pos, "Radio Tower"))
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        let pos: Vec3 = Vec3::new(165.00298, 2.6249952, -150.00085);
        Some((pos, "Start"))
    } else if keyboard.just_pressed(KeyCode::Digit4) {
        Some((TRAM_POSITION, "Tram"))
    } else if keyboard.just_pressed(KeyCode::Digit5) {
        let mut pos = LADDER_START;
        pos.y += GEOTHERMAL_BASE_HEIGHT + 20.0;
        pos.z += LADDER_WIDTH / 2.0;
        Some((pos, "Ladder"))
    } else if keyboard.just_pressed(KeyCode::Digit6) {
        let pos = Vec3::new(
            40.0, 
            WALL_Y_POSITION + WALL_HEIGHT / 2.0, 
            0.0);
        Some((pos, "Wall"))
    } else if keyboard.just_pressed(KeyCode::Digit7) {
        Some((AIRLOCK_POSITION, "Airlock"))
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

pub fn teleport_to_location(location: &str) -> Vec3 {
    match location {
        "geothermal" => Vec3::new(
            GEOTHERMAL_POSITION.x,
            GEOTHERMAL_POSITION.y + GEOTHERMAL_BASE_HEIGHT + 1.0,
            GEOTHERMAL_POSITION.z,
        ),
        "radio_tower" => Vec3::new(
            RADIO_TOWER_POSITION.x + RADIO_TOWER_WIDTH / 2.0,
            RADIO_TOWER_POSITION.y + RADIO_TOWER_HEIGHT + 1.0,
            RADIO_TOWER_POSITION.z + RADIO_TOWER_WIDTH / 2.0
        ),
        "bridge" => Vec3::new(
            BRIDGE_POSITION.x,
            BRIDGE_POSITION.y + BRIDGE_HEIGHT + 1.0,
            BRIDGE_POSITION.z + BRIDGE_WIDTH / 2.0
        ),
        "tram" => Vec3::new(
            TRAM_POSITION.x,
            TRAM_POSITION.y + 1.0,
            TRAM_POSITION.z
        ),
        "ladder_top" => Vec3::new(
            LADDER_START.x,
            LADDER_START.y + WALL_HEIGHT + 1.0,
            LADDER_START.z + LADDER_WIDTH / 2.0
        ),
        "airlock" => Vec3::new(
            AIRLOCK_POSITION.x - 20.0,
            AIRLOCK_POSITION.y + 10.0,
            AIRLOCK_POSITION.z - 20.0
        ),
        // ... other teleport locations ...
        _ => Vec3::ZERO,
    }
}
