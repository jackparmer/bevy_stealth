fn main() {
    App::new()
        .insert_resource(SpawnTimer(Timer::from_seconds(
            30.0,  // Spawn a new sentry every 30 seconds
            TimerMode::Repeating
        )))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            }),
            PhysicsPlugins::default()
        ))
        .init_resource::<MessageDisplay>()
        .add_systems(Startup, (
            setup,
            setup_explosion_materials,
            spawn_sentry,
            spawn_garages,
            spawn_big_pipe
        ).chain())
        .add_systems(Update, setup_scene_once_loaded.before(animate_targets))
        .add_systems(Update, keyboard_animation_control)
        .add_systems(Update, (
            sentry_follow_system,
            update_explosion_particles,
            update_explosion_light,
            periodic_sentry_spawn,
            animate_light_cones
        ).chain())
        .add_systems(Update, portal_system)
        .run();
}