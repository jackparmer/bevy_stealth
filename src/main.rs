mod systems {
    pub mod core;
    pub mod environments;
    pub mod player;
}

mod components;
mod resources;

use crate::resources::{ProtagonistAnimations, PROTAGONIST_ANIMATIONS};
use crate::systems::core::screenplay::MessageDisplay;

use systems::core::setup::setup;
use systems::core::camera::rotate_camera;
use systems::core::keyboard_input::keyboard_animation_control;
use systems::core::timer::{setup_debug_timer, print_protagonist_transform};
use systems::core::minimap::{setup_minimap, update_minimap, update_sentry_markers};
use systems::core::screenplay::{setup_screenplay, screenplay_system};

use systems::player::climbing::{
    handle_climbing, 
    climbing_keyboard_control, 
    check_ladder_presence, 
    handle_ladder_top,
};
use systems::player::swimming::swimming_system;
use systems::player::driving::{toggle_driving, driving_control };
use systems::player::teleports::teleport_system;
use systems::player::falling;
use systems::player::dirigible::{toggle_dirigible, dirigible_control, animate_floating_balloon};

use systems::environments::acquifier::check_acquifier_dirigible_trigger;
use systems::environments::portal::portal_system;
use systems::environments::terrain::{spawn_terrain, toggle_terrain_texture};
use systems::environments::airlock::{spawn_airlock, handle_airlock_teleport, blink_airlock_light};
use systems::environments::searchlight::{underwater_searchlight_system, update_searchlight_rotation};
use systems::environments::maze::{spawn_maze, check_dirigible_trigger, check_tank_exit};
use systems::environments::ice_cave::{update_ice_particles, handle_ice_cave_interactions};
use systems::environments::garage::{spawn_garages, handle_tank_interaction, handle_garage_approach};
use systems::environments::big_pipe::{spawn_big_pipe, handle_pipe_lift};
use systems::environments::lanterns::{spawn_lanterns, update_lanterns};

use systems::core::sentry::{
    spawn_sentry,
    sentry_follow_system,
    update_explosion_particles,
    update_explosion_light,
    periodic_sentry_spawn,
    setup_explosion_materials,
    animate_light_cones,
};

use avian3d::prelude::*;
use bevy::{
    animation::animate_targets,
    pbr::DirectionalLightShadowMap,
    prelude::*,
};
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use std::f32::consts::*;
use std::time::Duration;

fn main() {
    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins((
            DefaultPlugins, 
            PhysicsPlugins::default(),
            // WorldInspectorPlugin::new(),
        ))
        .init_resource::<MessageDisplay>()
        .add_systems(Startup, (
            setup,
            setup_explosion_materials,
            spawn_sentry,
            spawn_garages,
            spawn_big_pipe,
        ).chain())
        .add_systems(Startup, spawn_maze)
        .add_systems(Startup, spawn_terrain)
        .add_systems(Startup, spawn_airlock)
        .add_systems(Update, handle_airlock_teleport)
        .add_systems(Update, check_acquifier_dirigible_trigger)
        .add_systems(Update, blink_airlock_light)
        .add_systems(Update, toggle_terrain_texture)
        .add_systems(Update, toggle_driving)
        .add_systems(Update, driving_control)
        .add_systems(Update, check_tank_exit)
        .add_systems(Update, toggle_dirigible)
        .add_systems(Update, dirigible_control)
        .add_systems(Update, animate_floating_balloon)
        .add_systems(Update, animate_light_direction)
        .add_systems(Update, rotate_camera)
        .add_systems(Update, setup_scene_once_loaded.before(animate_targets))
        .add_systems(Update, keyboard_animation_control)
        .add_systems(Update, (
            sentry_follow_system,
            update_explosion_particles,
            update_explosion_light,
            periodic_sentry_spawn,
            animate_light_cones,
        ))
        .add_systems(Update, portal_system)      
        .add_systems(Update, handle_climbing)
        .add_systems(Update, handle_ladder_top)
        .add_systems(Update, check_ladder_presence.after(handle_climbing))
        .add_systems(Update, climbing_keyboard_control)
        .add_systems(Update, swimming_system)
        .add_systems(Update, underwater_searchlight_system)
        .add_systems(Update, update_searchlight_rotation)
        .add_systems(Update, teleport_system)
        .add_systems(Update, (
            falling::check_falling,
            falling::handle_falling_animation,
        ))
        .add_systems(Startup, setup_debug_timer)
        .add_systems(Update, print_protagonist_transform)
        .add_systems(Startup, setup_minimap)
        .add_systems(Update, (
            update_minimap,
            update_sentry_markers,
        ))
        .add_systems(Startup, setup_screenplay)
        .add_systems(Update, screenplay_system)
        .add_systems(Update, update_ice_particles)
        .add_systems(Update, handle_ice_cave_interactions)
        .add_systems(Update, handle_tank_interaction)
        .add_systems(Update, handle_garage_approach)
        .add_systems(Update, check_dirigible_trigger)
        .add_systems(Update, handle_pipe_lift)
        .add_systems(Startup, spawn_lanterns)
        .add_systems(Update, update_lanterns)
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
    protagonist_animations: Res<ProtagonistAnimations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();
        let stretch = *PROTAGONIST_ANIMATIONS.get("IDLE_STRETCH").unwrap();
        transitions
            .play(&mut player, 
                protagonist_animations.animations[stretch], 
                Duration::from_millis(1000),
            ).repeat();

        commands
            .entity(entity)
            .insert(protagonist_animations.graph.clone())
            .insert(transitions);
    }
}
