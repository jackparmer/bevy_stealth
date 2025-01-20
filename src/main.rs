mod systems {
    pub mod core;
    pub mod environments;
    pub mod player;
}

mod components;
mod resources;

use crate::components::Protagonist;
use crate::resources::{Animations, SCENES};

use systems::core::setup::setup;
use systems::core::camera::rotate_camera;
use systems::core::input::keyboard_animation_control;
use systems::core::timer::{setup_debug_timer, print_protagonist_transform};
use systems::core::minimap::{setup_minimap, update_minimap};

use systems::player::climbing::{handle_climbing, climbing_keyboard_control, check_ladder_presence};
use systems::player::swimming::swimming_system;
use systems::player::driving::{toggle_driving, driving_control};
use systems::player::teleports::teleport_system;
use systems::player::falling;
use systems::player::dirigible::{toggle_dirigible, dirigible_control, animate_floating_balloon};

use systems::environments::portal::portal_system;
use systems::environments::terrain::{spawn_terrain, toggle_terrain_texture};
use systems::environments::airlock::{spawn_airlock, handle_airlock_teleport, blink_airlock_light};
use systems::environments::tram::move_tram;
use systems::environments::searchlight::{underwater_searchlight_system, update_searchlight_rotation};
use systems::environments::reactor::{handle_reactor_ladder_exit};

use avian3d::prelude::*;
use bevy::{
    animation::animate_targets,
    pbr::DirectionalLightShadowMap,
    prelude::*,
};

use std::f32::consts::*;
use std::time::Duration;

fn main() {
    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        // Enable physics
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_systems(Startup, setup)
        .add_systems(Startup, spawn_terrain)
        .add_systems(Startup, spawn_airlock)
        .add_systems(Update, handle_airlock_teleport)
        .add_systems(Update, blink_airlock_light)
        .add_systems(Update, toggle_terrain_texture)
        .add_systems(Update, toggle_driving)
        .add_systems(Update, driving_control)
        .add_systems(Update, move_tram)
        .add_systems(Update, animate_light_direction)
        .add_systems(Update, rotate_camera)
        .add_systems(Update, setup_scene_once_loaded.before(animate_targets))
        .add_systems(Update, keyboard_animation_control)
        .add_systems(Update, reset_game_on_command_r) // Add reset system
        .add_systems(Update, portal_system)      
        .add_systems(Update, handle_climbing)
        .add_systems(Update, check_ladder_presence.after(handle_climbing))
        .add_systems(Update, climbing_keyboard_control)
        .add_systems(Update, swimming_system)
        .add_systems(Update, underwater_searchlight_system)
        .add_systems(Update, update_searchlight_rotation)
        .add_systems(Update, teleport_system)
        .add_systems(Update, handle_reactor_ladder_exit)
        .add_systems(Update, (
            falling::check_falling,
            falling::handle_falling_animation,
        ))
        .add_systems(Startup, setup_debug_timer)
        .add_systems(Update, print_protagonist_transform)
        .add_systems(Startup, setup_minimap)
        .add_systems(Update, update_minimap)
        .add_systems(Update, (
            toggle_dirigible,
            dirigible_control,
            animate_floating_balloon,
        ))
        .run();
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_seconds() * PI / 5.0,
            -FRAC_PI_4,
        );
    }
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();
        let stretch = *SCENES.get("IDLE_STRETCH").unwrap();
        info!("starting pose: {}", stretch);
        // Make sure to start the animation via the `AnimationTransitions`
        // component. The `AnimationTransitions` component wants to manage all
        // the animations and will get confused if the animations are started
        // directly via the `AnimationPlayer`.
        transitions
            .play(&mut player, 
                animations.animations[stretch], 
                Duration::from_millis(1000),
            ).repeat();

        commands
            .entity(entity)
            .insert(animations.graph.clone())
            .insert(transitions);
    }
}

/// System to reset the game when Command-R or Ctrl-R is pressed
fn reset_game_on_command_r(
    mut commands: Commands,
    camera_query: Query<Entity, With<Camera>>,
    protagonist_query: Query<Entity, With<Protagonist>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    graphs: ResMut<Assets<AnimationGraph>>,
    images: ResMut<Assets<Image>>,
    time: Res<Time>,
) {
    // Check for Command-R (Mac) or Ctrl-R (Other systems)
    let is_command_r = keyboard_input.pressed(KeyCode::SuperLeft) && keyboard_input.just_pressed(KeyCode::KeyR);
    let is_ctrl_r = keyboard_input.pressed(KeyCode::ControlLeft) && keyboard_input.just_pressed(KeyCode::KeyR);

    if is_command_r || is_ctrl_r {
        println!("Resetting the game...");

        for entity in camera_query.iter() {
            println!("Despawning camera entity: {:?}", entity);
            commands.entity(entity).despawn();
        }

        for entity in protagonist_query.iter() {
            commands.entity(entity).despawn();
        }        

        // Re-run the setup logic to reinitialize the game
        setup(commands, asset_server, meshes, materials, graphs, images, time);
    }
}
