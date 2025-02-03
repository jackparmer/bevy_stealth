use bevy::prelude::*;
use crate::components::Protagonist;

#[derive(Component)]
pub struct DebugTimer(pub Timer);

pub fn setup_debug_timer(mut commands: Commands) {
    commands.spawn(DebugTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
}

pub fn print_protagonist_transform(
    time: Res<Time>,
    mut timer_query: Query<&mut DebugTimer>,
    protagonist_query: Query<(&Transform, &Protagonist), With<Protagonist>>,
    ambient_light: Res<AmbientLight>,
) {
    // for mut timer in &mut timer_query {
    //     if timer.0.tick(time.delta()).just_finished() {
    //         if let Ok((transform, protagonist)) = protagonist_query.get_single() {
    //             println!("Protagonist position: {:?}", transform.translation);
    //             println!("Protagonist state: {:?}", protagonist);
    //             println!("Ambient light intensity: {:?}", ambient_light.brightness);
    //             println!("--------------------");
    //         }
    //     }
    // }
}
