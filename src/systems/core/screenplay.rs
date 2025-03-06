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

const SEQUENCES: &[(&str, &[(&str, Color)])] = &[
    ("start", &[
        ("Follow the light...", Color::WHITE),
        ("RUN (Shift-W). A sentry has found you.", Color::srgb(1.0, 0.0, 0.0)),
        ("Find the airlock...", Color::WHITE),
    ]),
    ("airlock", &[
        ("A gen1 airlock...", Color::WHITE),
        ("Hopefully it still opens...", Color::WHITE)
    ]),
    ("outside_airlock", &[
        ("Sentries can find you outside...", Color::srgb(1.0,0.0,0.0)),
        ("Run to the garage.", Color::WHITE),
    ]),
    ("garage", &[
        ("The garage is locked...", Color::WHITE),
        ("Find the keycard...", Color::WHITE),
    ]),
];

#[derive(Component)]
pub struct ScreenplayText;

#[derive(Resource)]
pub struct TextTimer(Timer);

#[derive(Resource)]
pub struct MessageState {
    current_sequence: Option<String>,
    sequence_index: usize,
    has_shown_current: bool,
    completed_sequences: Vec<String>,
}

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

    commands.insert_resource(MessageState {
        current_sequence: None,
        sequence_index: 0,
        has_shown_current: false,
        completed_sequences: Vec::new(),
    });
}

pub fn screenplay_system(
    time: Res<Time>,
    mut timer: ResMut<TextTimer>,
    mut message_state: ResMut<MessageState>,
    mut text_query: Query<&mut Text, With<ScreenplayText>>,
    protagonist_query: Query<&Transform, With<Protagonist>>,
) {
    if let Ok(protagonist_transform) = protagonist_query.get_single() {
        let pos = protagonist_transform.translation;
        
        // Check trigger conditions for each sequence
        let triggered_sequence = if !message_state.completed_sequences.contains(&"start".to_string()) &&
            (pos.distance(GAME_START) < TRIGGER_DISTANCE ||
             pos.distance(PROTAGONIST_START.position) < TRIGGER_DISTANCE) {
            Some("start")
        } else if pos.distance(RADIO_TOWER_POSITION) < TRIGGER_DISTANCE {
            Some("radio_tower")
        } else if pos.distance(TRAM_POSITION) < TRIGGER_DISTANCE {
            Some("tram")
        } else if pos.distance(OUTSIDE_AIRLOCK) < TRIGGER_DISTANCE {
            Some("outside_airlock")
        } else if pos.distance(AIRLOCK_POSITION) < TRIGGER_DISTANCE {
            Some("airlock")
        } else if (pos.y - WALL_Y_POSITION).abs() < TRIGGER_DISTANCE {
            Some("wall")
        } else if pos.distance(ICE_CAVE_POSITION) < TRIGGER_DISTANCE {
            Some("ice_cave")
        } else if pos.distance(LADDER_START) < TRIGGER_DISTANCE {
            Some("ladder")
        } else if pos.distance(OUTSIDE) < TRIGGER_DISTANCE {
            Some("outside")
        } else if pos.distance(REACTOR) < TRIGGER_DISTANCE {
            Some("reactor")
        } else if pos.distance(MAZE_START) < TRIGGER_DISTANCE {
            Some("maze")
        } else {
            None
        };

        // Handle sequence state
        if let Some(sequence_id) = triggered_sequence {
            // Start new sequence if not already playing one
            if message_state.current_sequence.as_deref() != Some(sequence_id) && 
               !message_state.completed_sequences.contains(&sequence_id.to_string()) {
                message_state.current_sequence = Some(sequence_id.to_string());
                message_state.sequence_index = 0;
                message_state.has_shown_current = false;
            }
        }

        // Display current message if in a sequence
        if let Some(sequence_id) = &message_state.current_sequence {
            if let Some((_, messages)) = SEQUENCES.iter().find(|(id, _)| *id == sequence_id) {
                if !message_state.has_shown_current && message_state.sequence_index < messages.len() {
                    message_state.has_shown_current = true;
                    timer.0.reset();
                    let (text, color) = messages[message_state.sequence_index];
                    if let Ok(mut text_component) = text_query.get_single_mut() {
                        text_component.sections[0].value = text.to_string();
                        text_component.sections[0].style.color = color;
                    }
                }
            }
        }
    }

    // Update timer and handle message progression
    if timer.0.tick(time.delta()).finished() {
        if let Ok(mut text) = text_query.get_single_mut() {
            text.sections[0].value.clear();
            
            if let Some(sequence_id) = message_state.current_sequence.clone() {
                if let Some((_, messages)) = SEQUENCES.iter().find(|(id, _)| *id == sequence_id) {
                    message_state.sequence_index += 1;
                    message_state.has_shown_current = false;
                    
                    if message_state.sequence_index >= messages.len() {
                        message_state.completed_sequences.push(sequence_id);
                        message_state.current_sequence = None;
                    }
                }
            }
        }
    }
}
