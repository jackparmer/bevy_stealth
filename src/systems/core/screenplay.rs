use bevy::prelude::*;
use crate::systems::core::setup::{
    RADIO_TOWER_POSITION,
    TRAM_POSITION,
    GEOTHERMAL_BASE_HEIGHT,
    RADIO_TOWER_HEIGHT,
    RADIO_TOWER_WIDTH,
    PROTAGONIST_START,
};

use crate::systems::environments::launch_silo::WALL_Y_POSITION;
use crate::systems::environments::ladder::LADDER_START;

// Import constants from teleports.rs
use crate::systems::player::teleports::{
    AIRLOCK_POSITION,
    GAME_START,
    OUTSIDE,
    REACTOR,
    ICE_CAVE_POSITION,
    MAZE_START,
};

use crate::components::Protagonist;

const TEXT_DISPLAY_DURATION: f32 = 5.0; // seconds
const TRIGGER_DISTANCE: f32 = 20.0; // units

const OUTSIDE_AIRLOCK: Vec3 = Vec3::new(924.9679, 2.8701305, -430.01);

#[derive(Component)]
pub struct ScreenplayText;

#[derive(Resource)]
pub struct TextTimer(Timer);

pub fn setup_screenplay(mut commands: Commands) {
    // Spawn UI camera first
    commands.spawn(Camera2dBundle::default());

    // Then spawn our UI
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(20.0),
                    left: Val::Percent(5.0),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Test Message",
                    TextStyle {
                        font_size: 30.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                ScreenplayText,
            ));
        });

    commands.insert_resource(TextTimer(Timer::from_seconds(
        TEXT_DISPLAY_DURATION,
        TimerMode::Once,
    )));
}

pub fn screenplay_system(
    time: Res<Time>,
    mut timer: ResMut<TextTimer>,
    mut text_query: Query<&mut Text, With<ScreenplayText>>,
    protagonist_query: Query<&Transform, With<Protagonist>>,
) {
    if let Ok(protagonist_transform) = protagonist_query.get_single() {
        let pos = protagonist_transform.translation;
        
        // Check various locations and trigger text
        let message = if pos.distance(RADIO_TOWER_POSITION) < TRIGGER_DISTANCE {
            Some("Radio Tower: A towering structure piercing the sky...")
        } else if pos.distance(TRAM_POSITION) < TRIGGER_DISTANCE {
            Some("The old tram station, still operational after all these years...")
        } else if pos.distance(OUTSIDE_AIRLOCK) < TRIGGER_DISTANCE {
            Some("Quadruped sentries are nearby. Shelter in the closest armoured vehicle...")
        } else if (pos.y - WALL_Y_POSITION).abs() < TRIGGER_DISTANCE {
            Some("The great wall stretches endlessly into the distance...")
        } else if pos.distance(GAME_START) < TRIGGER_DISTANCE ||
            pos.distance(PROTAGONIST_START.position) < TRIGGER_DISTANCE {
            Some("You've infiltrated the south pole base... Try to find the airlock...")
        } else if pos.distance(AIRLOCK_POSITION) < TRIGGER_DISTANCE {
            Some("A gen-1 airlock. Hopefully it still works...")
        } else if pos.distance(ICE_CAVE_POSITION) < TRIGGER_DISTANCE {
            Some("Ice Cave: Ancient formations glisten in the dim light...")
        } else if pos.distance(LADDER_START) < TRIGGER_DISTANCE {
            Some("A maintenance ladder descends into the geothermal depths...")
        } else if pos.distance(OUTSIDE) < TRIGGER_DISTANCE {
            Some("The facility's exterior. Harsh winds carry particles of sand...")
        } else if pos.distance(REACTOR) < TRIGGER_DISTANCE {
            Some("The reactor core. The heart of the facility thrums with energy...")
        } else if pos.distance(MAZE_START) < TRIGGER_DISTANCE {
            Some("The maze entrance. A labyrinth of catwalks stretches before you...")
        } else {
            None
        };

        if let Some(new_text) = message {
            if let Ok(mut text) = text_query.get_single_mut() {
                text.sections[0].value = new_text.to_string();
                timer.0.reset();
            }
        }
    }

    // Update timer and hide text if expired
    if timer.0.tick(time.delta()).finished() {
        if let Ok(mut text) = text_query.get_single_mut() {
            text.sections[0].value.clear();
        }
    }
}
