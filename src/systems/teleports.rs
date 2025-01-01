use bevy::prelude::*;
use crate::systems::setup::{
    WALL_Y_POSITION,
    GEOTHERMAL_POSITION,
    RADIO_TOWER_POSITION,
    TRAM_POSITION,
    // Heights
    GEOTHERMAL_BASE_HEIGHT,
    RADIO_TOWER_HEIGHT,
    WALL_HEIGHT,
    // Widths
    GEOTHERMAL_BASE_RADIUS,
    RADIO_TOWER_WIDTH,
};

use crate::systems::ladder::{
    LADDER_START,
    LADDER_WIDTH,
};

const AIRLOCK_POSITION: Vec3 = Vec3::new(779.1837, 2.6249955, -423.27768);
const GAME_START: Vec3 = Vec3::new(165.00298, 2.6249952, -150.00085);

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
        let mut pos = GEOTHERMAL_POSITION;
        pos.y += GEOTHERMAL_BASE_HEIGHT / 2.0;
        pos.x += GEOTHERMAL_BASE_RADIUS / 2.0;
        Some((pos, "Geothermal"))
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
    } else if keyboard.just_pressed(KeyCode::Digit7) {
        Some((TRAM_POSITION, "Tram"))
    } else if keyboard.just_pressed(KeyCode::Digit8) {
        let pos = Vec3::new(
            40.0, 
            WALL_Y_POSITION + WALL_HEIGHT / 2.0, 
            0.0);
        Some((pos, "Wall"))
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

