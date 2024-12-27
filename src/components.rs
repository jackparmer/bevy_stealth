use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Protagonist {
    pub is_falling: bool,
    pub is_climbing: bool,
    pub was_climbing: bool,
    pub is_swimming: bool,    // Add this field
    pub was_swimming: bool,   // Add this field
}

impl Default for Protagonist {
    fn default() -> Self {
        Self {
            is_falling: false,
            is_climbing: false,
            was_climbing: false,
            is_swimming: false,
            was_swimming: false,
        }
    }
}