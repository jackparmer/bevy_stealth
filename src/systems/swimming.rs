use bevy::prelude::*;
use crate::components::Protagonist;

pub fn swimming_system(
    mut protagonist_query: Query<(Entity, &Transform, &mut Protagonist)>,
    mut ambient_light: ResMut<AmbientLight>,
) {
    for (_entity, transform, mut protagonist) in protagonist_query.iter_mut() {
        if transform.translation.y < -5.0 {
            protagonist.is_swimming = true;
            protagonist.was_swimming = true;
            protagonist.is_falling = false;
            protagonist.is_climbing = false;
            
            ambient_light.color = Color::srgb(0.0, 0.2, 0.4);
            ambient_light.brightness = 1000.0;
        } else {
            if protagonist.is_swimming {
                protagonist.is_swimming = false;
                protagonist.was_swimming = true;
                ambient_light.color = Color::srgb(0.2, 0.2, 0.3);
                ambient_light.brightness = 6000.0;
            }
        }
    }
}
