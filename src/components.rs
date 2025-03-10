use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Protagonist {
    pub is_falling: bool,
    pub is_climbing: bool,
    pub was_climbing: bool,
    pub is_swimming: bool,
    pub was_swimming: bool,
    pub is_driving: bool,
    pub is_dirigible: bool,
    pub is_outside: bool,
    pub is_birds_eye: bool,
    pub is_jumping: bool,
    pub last_climb_toggle: f32,
}

impl Default for Protagonist {
    fn default() -> Self {
        Self {
            is_falling: false,
            is_climbing: false,
            was_climbing: false,
            is_swimming: false,
            was_swimming: false,
            is_driving: false,
            is_dirigible: false,
            is_outside: true,
            is_birds_eye: false,
            is_jumping: false,
            last_climb_toggle: 0.0,
        }
    }
}

#[derive(Component)]
pub struct HighAltitudeIndicator;

#[derive(Component)]
pub struct Sentry {
    pub view_distance: f32,
    pub view_angle: f32,
    pub follow_speed: f32,
    pub velocity: Vec3,
}

#[derive(Component)]
pub struct Tank;

#[derive(Component)]
pub struct MazeMessageTrigger {
    pub triggered: bool,
}

