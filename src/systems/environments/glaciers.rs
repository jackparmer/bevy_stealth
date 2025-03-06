use avian3d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

const NUM_GLACIERS: i32 = 30;
const MIN_DISTANCE: f32 = 100.0;
const MAX_DISTANCE: f32 = 5000.0;
const MIN_HEIGHT: f32 = -950.0;
const MAX_HEIGHT: f32 = -500.0;
const MIN_TILT: f32 = -0.2;
const MAX_TILT: f32 = 0.2;
const MIN_SCALE: f32 = 10.0;
const MAX_SCALE: f32 = 30.0;

pub fn spawn_glaciers(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    let mut rng_glacier = rand::thread_rng();

    for _ in 0..NUM_GLACIERS {
        let distance = rng_glacier.gen_range(MIN_DISTANCE..MAX_DISTANCE);
        let y = rng_glacier.gen_range(MIN_HEIGHT..MAX_HEIGHT);
        let angle = rng_glacier.gen_range(0.0..std::f32::consts::TAU);
        let x = distance * angle.cos();
        let z = distance * angle.sin();
    
        let rotation = Quat::from_euler(
            EulerRot::XYZ,
            rng_glacier.gen_range(MIN_TILT..MAX_TILT),
            rng_glacier.gen_range(0.0..std::f32::consts::TAU),
            rng_glacier.gen_range(MIN_TILT..MAX_TILT),
        );
        let scale = rng_glacier.gen_range(MIN_SCALE..MAX_SCALE);
    
        commands.spawn((
            SceneBundle {
                scene: asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset("python/Tall_Monolithic_Rock.glb")),
                transform: Transform::from_xyz(x, y, z)
                    .with_rotation(rotation)
                    .with_scale(Vec3::splat(scale)),
                ..default()
            },
            ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
            RigidBody::Static,
        ));
    }
}
