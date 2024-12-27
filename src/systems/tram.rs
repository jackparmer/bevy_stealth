use bevy::prelude::*;
use crate::systems::setup::TramCar;

pub fn move_tram(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut TramCar)>,
) {
    for (mut transform, mut tram) in query.iter_mut() {
        tram.time += time.delta_seconds();
        
        // Calculate the offset using a sine wave
        let offset = tram.amplitude * (tram.time * tram.frequency).sin();
        
        // Distribute the offset equally between X and Z to create 45-degree movement
        let offset_x = offset * 0.707; // cos(45°) ≈ 0.707
        let offset_z = offset * 0.707; // sin(45°) ≈ 0.707
        
        transform.translation = tram.origin + Vec3::new(offset_x, 0.0, offset_z);
    }
}
