use bevy::prelude::*;

use crate::systems::core::setup::PROTAGONIST_START;
use crate::systems::environments::launch_silo::WALL_Y_POSITION;
use crate::systems::environments::ladder::LADDER_START;
use crate::systems::environments::garage::{GARAGE_POSITION_1, GARAGE_POSITION_2};
use crate::systems::environments::geothermal::RADIO_TOWER_POSITION;

// Import constants from teleports.rs
use crate::systems::player::teleports::{
    AIRLOCK_POSITION,
    GAME_START,
    OUTSIDE,
    REACTOR,
    ICE_CAVE_POSITION,
};

use crate::components::Protagonist;

const TEXT_DISPLAY_DURATION: f32 = 5.0; // seconds
const TRIGGER_DISTANCE: f32 = 20.0; // units

const OUTSIDE_AIRLOCK: Vec3 = Vec3::new(924.9679, 2.8701305, -430.01);
const AIRLOCK_DOORS: Vec3 = Vec3::new(437.97876, 2.624982, -395.71606);
const VEHICLE_SHELTER: Vec3 = Vec3::new(1420.3849, 2.6249862, -505.01404);
const WRONG_WAY: Vec3 = Vec3::new(3073.6443, 5.597332, -69.24335);
const PROTAGONIST_POSITION: Vec3 = Vec3::new(1383.2694, 3.992052, -20.53674);

// Add new constant for the acquifier message trigger position
const ACQUIFIER_HINT: Vec3 = Vec3::new(1856.5652, 3.992052, -6542.7266);

// Change from const to pub const
pub const TANK_EXIT_POSITION: Vec3 = Vec3::new(1920.8418, 3.992052, 6551.8755);

// Add new constant after other position constants
const TANK_HINT_POSITION: Vec3 = Vec3::new(2158.844, 3.992052, 6459.8257);

// Add new constant after other position constants
const LEVEL_COMPLETE_POSITION: Vec3 = Vec3::new(-398.6294, 800.12476, -397.60477);

const SEQUENCES: &[(&str, &[(&str, Color)])] = &[
    ("tank_exit", &[
        ("CLIMB THE LADDER", Color::WHITE),
    ]),
    ("acquifier_hint", &[
        ("FIND THE ACQUIFIER ENTRY", Color::WHITE),
    ]),
    ("protagonist_area", &[
        ("Follow the emergency lanterns", Color::WHITE),
    ]),
    ("start", &[
        ("Follow the light...", Color::WHITE),
        ("RUN (Shift-W). A sentry is behind you.", Color::srgb(1.0, 0.0, 0.0)),
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
        ("Get to the ice cave...", Color::WHITE),
        ("Find the acquifier...", Color::WHITE),
    ]),
    ("airlock_doors", &[
        ("Pry the airlock doors...", Color::WHITE),
    ]),
    ("vehicle_shelter", &[
        ("Shelter in the vehicle...", Color::srgb(1.0, 0.0, 0.0)),
    ]),
    ("wrong_way", &[
        ("Wrong way - turn around", Color::srgb(1.0, 0.0, 0.0)),
    ]),
    ("tank_hint", &[
        ("PRESS T TO EXIT THE TANK", Color::WHITE),
    ]),
    ("level_complete", &[
        ("LEVEL 0 COMPLETED", Color::WHITE),
    ]),
];

#[derive(Component)]
pub struct ScreenplayText;

#[derive(Resource)]
pub struct MessageState {
    current_sequence: Option<String>,
    sequence_index: usize,
    has_shown_current: bool,
    completed_sequences: Vec<String>,
}

#[derive(Resource)]
pub struct MessageDisplay {
    message: Option<(String, Color)>,
    timer: Timer,
}

impl Default for MessageDisplay {
    fn default() -> Self {
        Self {
            message: None,
            timer: Timer::from_seconds(TEXT_DISPLAY_DURATION, TimerMode::Once),
        }
    }
}

impl MessageDisplay {
    pub fn display_message(&mut self, text: impl Into<String>, color: Color) {
        self.message = Some((text.into(), color));
        self.timer.reset();
    }

    pub fn is_displaying(&self) -> bool {
        self.message.is_some() && !self.timer.finished()
    }
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
                    top: Val::Px(20.0),
                    left: Val::Percent(50.0),
                    margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(0.0), Val::Px(0.0)),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 30.0,
                        color: Color::WHITE,
                        ..default()
                    },
                )
                .with_style(Style {
                    justify_content: JustifyContent::Center,
                    ..default()
                }),
                ScreenplayText,
            ));
        });

    commands.insert_resource(MessageDisplay::default());
    commands.insert_resource(MessageState {
        current_sequence: None,
        sequence_index: 0,
        has_shown_current: false,
        completed_sequences: Vec::new(),
    });
}

pub fn screenplay_system(
    time: Res<Time>,
    mut message_display: ResMut<MessageDisplay>,
    mut message_state: ResMut<MessageState>,
    mut text_query: Query<&mut Text, With<ScreenplayText>>,
    protagonist_query: Query<&Transform, With<Protagonist>>,
) {
    // Update display timer
    message_display.timer.tick(time.delta());

    // Only clear the message if the timer is finished AND it wasn't just set
    if message_display.timer.finished() && !message_display.is_displaying() {
        if let Ok(mut text) = text_query.get_single_mut() {
            text.sections[0].value.clear();
            message_display.message = None;
        }
    }

    // Display current message if one exists
    if let Some((message, color)) = &message_display.message {
        if let Ok(mut text) = text_query.get_single_mut() {
            text.sections[0].value = message.clone();
            text.sections[0].style.color = *color;
        }
    }

    // Only process screenplay sequences if we're not displaying a direct message
    if !message_display.is_displaying() {
        // Rest of the screenplay logic
        if let Ok(protagonist_transform) = protagonist_query.get_single() {
            let pos = protagonist_transform.translation;
            
            // Check trigger conditions for each sequence
            let triggered_sequence = if pos.distance(LEVEL_COMPLETE_POSITION) < TRIGGER_DISTANCE {
                Some("level_complete")
            } else if pos.distance(TANK_HINT_POSITION) < TRIGGER_DISTANCE {
                Some("tank_hint")
            } else if pos.distance(TANK_EXIT_POSITION) < TRIGGER_DISTANCE {
                Some("tank_exit")
            } else if pos.distance(ACQUIFIER_HINT) < TRIGGER_DISTANCE {
                Some("acquifier_hint")
            } else if pos.distance(PROTAGONIST_POSITION) < TRIGGER_DISTANCE {
                Some("protagonist_area")
            } else if !message_state.completed_sequences.contains(&"start".to_string()) &&
                (pos.distance(GAME_START) < TRIGGER_DISTANCE ||
                 pos.distance(PROTAGONIST_START.position) < TRIGGER_DISTANCE) {
                Some("start")
            } else if pos.distance(AIRLOCK_DOORS) < TRIGGER_DISTANCE {
                Some("airlock_doors")
            } else if pos.distance(VEHICLE_SHELTER) < TRIGGER_DISTANCE {
                Some("vehicle_shelter")
            } else if pos.distance(WRONG_WAY) < TRIGGER_DISTANCE {
                Some("wrong_way")
            } else if pos.distance(GARAGE_POSITION_1) < TRIGGER_DISTANCE || pos.distance(GARAGE_POSITION_2) < TRIGGER_DISTANCE {
                Some("garage")
            } else if pos.distance(RADIO_TOWER_POSITION) < TRIGGER_DISTANCE {
                Some("radio_tower")
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
                        let (text, color) = messages[message_state.sequence_index];
                        message_display.display_message(text, color);
                    }
                }
            }
        }

        // Update timer and handle message progression
        if message_display.timer.tick(time.delta()).finished() {
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
}

// Public function to display messages from other systems
pub fn display_message(message: impl Into<String>, color: Color, message_display: &mut MessageDisplay) {
    message_display.display_message(message, color);
}
