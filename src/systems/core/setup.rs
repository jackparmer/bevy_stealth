use crate::components::Protagonist;
use crate::resources::ProtagonistAnimations;
use crate::systems::environments::ladder::{spawn_ladder, LadderConfig};
use crate::systems::environments::ice_cave::spawn_ice_cave;
use crate::systems::environments::launch_silo::spawn_launch_silo;
use crate::systems::environments::reactor::spawn_reactor;

use avian3d::prelude::*;
use bevy::{
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
};

use rand::Rng;
use bevy::math::Vec3;
use bevy::render::texture::{ImageSampler, ImageAddressMode, ImageSamplerDescriptor};

// Constants for structure dimensions

pub const GEOTHERMAL_BASE_HEIGHT: f32 = 250.0;
pub const GEOTHERMAL_BASE_RADIUS: f32 = 100.0;
pub const GEOTHERMAL_POSITION: Vec3 = Vec3::new(200.0, 0.0, 200.0);

pub const RADIO_TOWER_HEIGHT: f32 = 400.0;
pub const RADIO_TOWER_WIDTH: f32 = 25.0;
pub const RADIO_TOWER_POSITION: Vec3 = Vec3::new(200.0, 200.0, 200.0);

pub const BRIDGE_LENGTH: f32 = 300.0;
pub const BRIDGE_HEIGHT: f32 = 10.0;
pub const BRIDGE_WIDTH: f32 = 20.0;
pub const BRIDGE_POSITION: Vec3 = Vec3::new(100.0, 394.99, 100.0);

pub const TRAM_POSITION: Vec3 = Vec3::new(100.0, 245.5, 100.0);

pub const WORLD_RADIUS: f32 = 5000.0;

pub const TOWER_LADDER_START: Vec3 = Vec3::new(214.0, 200.0, 200.0);

pub const PERIMETER_WALL_HEIGHT: f32 = 5000.0;

pub const ACQUIFIER_FLOOR_DEPTH: f32 = -1000.0;

pub struct ProtagonistStart {
    pub position: Vec3,
}

pub const PROTAGONIST_START: ProtagonistStart = ProtagonistStart {
    position: Vec3::new(204.0, 3.0, -35.0),
};

#[derive(Component)]
pub struct TramCar {
    pub origin: Vec3,
    pub time: f32,
    pub amplitude: f32,
    pub frequency: f32,
}

pub fn setup(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut images: ResMut<Assets<Image>>,
    time: Res<Time>,
) {

    // Protagonist animations
    let mut protagonist_graph = AnimationGraph::new();
    const PROTAGONIST_ANIMATIONS: usize = 44;
    let protagonist_animations = protagonist_graph
        .add_clips(
            (0..=PROTAGONIST_ANIMATIONS)
                .map(|i| GltfAssetLabel::Animation(i).from_asset("models/Protagonist.glb"))
                .map(|path| asset_server.load(path)),
            1.0,
            protagonist_graph.root,
        )
        .collect();

    let protagonist_graph = graphs.add(protagonist_graph);
    commands.insert_resource(ProtagonistAnimations {
        animations: protagonist_animations,
        graph: protagonist_graph.clone(),
    });

    // Add Camera and Ambient Lighting

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.7, 0.7, 10.0)
                .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
            ..default()
        },
        /*
        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/specular_rgb9e5_zstd.ktx2"),
            intensity: 250.0,
        }
        */
    ));

    // Add Ambient Light
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.1, 0.1, 0.3),
        brightness: 100.0,
    });

    // Add Directional Lighting

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10.0,
            shadows_enabled: true,
            color: Color::srgb(0.2, 0.2, 0.3),
            ..default()
        },
        // This is a relatively small scene, so use tighter shadow
        // cascade bounds than the default for better quality.
        // We also adjusted the shadow map to be larger since we're
        // only using a single cascade.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .into(),
        ..default()
    });

    // Static "floor"

    let floor_material = materials.add(StandardMaterial {
        perceptual_roughness: 0.9,
        metallic: 0.1,
        base_color_texture: Some(asset_server.load("textures/8k_mars.png")),
        ..default()
    });

    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(90.0, 0.2, 90.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(90.0, 0.2, 90.0)),
            material: floor_material,
            ..default()
        },
        Name::new("Floor"),
    ));

    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(90.0, 0.2, 90.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(90.0, 0.2, 90.0)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/ice_texture2.png")),
                metallic: 1.0,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, -0.2, 0.0),
            ..default()
        },
        Name::new("SubFloor"),
    ));    

    commands.spawn((
        RigidBody::Static,
        ColliderConstructor::TrimeshFromMesh,
        PbrBundle {
            mesh: meshes.add(Extrusion::new(Annulus::new(5.0, 10.0), 10.0)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some({
                    let texture_handle = asset_server.load("textures/ice_texture3.png");
                    if let Some(image) = images.get_mut(&texture_handle) {
                        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                            address_mode_u: ImageAddressMode::Repeat,
                            address_mode_v: ImageAddressMode::Repeat,
                            address_mode_w: ImageAddressMode::Repeat,
                            ..default()
                        });
                    }
                    texture_handle
                }),
                perceptual_roughness: 0.9,
                metallic: 0.0,
                ..default()
            }),
            transform: Transform::from_xyz(-10.0, -3.5, 10.0)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ..default()
        },
    ));

    // Starship
    commands.spawn((
        SceneBundle {
            scene: asset_server
                .load(GltfAssetLabel::Scene(0).from_asset("models/starhopper.glb")),
            transform: Transform::from_xyz(
                RADIO_TOWER_POSITION.x, 
                RADIO_TOWER_POSITION.y + RADIO_TOWER_HEIGHT/2.0, 
                RADIO_TOWER_POSITION.z
            )
                .with_scale(Vec3::splat(1.0)),
            ..default()
        },
        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
        RigidBody::Static,
    ));


    // Glaciers

    let mut rng_glacier = rand::thread_rng();

    for _ in 0..30 {
        // Generate a random position between tundra and acquifier floor
        let distance = rng_glacier.gen_range(100.0..WORLD_RADIUS);
        let y = rng_glacier.gen_range(-950.0..-50.0); // Fixed: start < end
        let angle = rng_glacier.gen_range(0.0..std::f32::consts::TAU);
        let x = distance * angle.cos();
        let z = distance * angle.sin();
    
        // Random rotation and scale
        let rotation = Quat::from_euler(
            EulerRot::XYZ,
            rng_glacier.gen_range(-0.2..0.2),
            rng_glacier.gen_range(0.0..std::f32::consts::TAU),
            rng_glacier.gen_range(-0.2..0.2),
        );
        let scale = rng_glacier.gen_range(1.0..15.0);
    
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

    // Replace the wall spawning code with:
    spawn_launch_silo(&mut commands, &mut meshes, &mut materials, &asset_server);

    // GLTF Protagonist

    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 0.25, 1.0),
        ExternalImpulse::default(),
        LockedAxes::new().lock_rotation_x().lock_rotation_z(),
        Friction::new(0.5),
        GravityScale(3.0),
        Protagonist { 
            is_climbing: false,
            was_climbing: false,
            is_falling: false,
            is_swimming: false,
            was_swimming: false,
            is_driving: false,
            is_dirigible: false,
            is_outside: false
        },
        SceneBundle {       
            scene: asset_server
                .load(GltfAssetLabel::Scene(0)
                .from_asset("models/Protagonist.glb")),
            transform: Transform::from_translation(PROTAGONIST_START.position),
            ..default()
        },
        Name::new("Protagonist"),
    ));

    // Load the stars texture
    let stars_texture_handle = asset_server.load("textures/8k_stars.png");
    
    // Create a material with the stars texture
    let sky_material = materials.add(StandardMaterial {
        base_color_texture: Some(stars_texture_handle),
        unlit: true, // Make sure it's unlit so it glows like the sky
        ..default()
    });

    // Spawn the sky dome (large sphere surrounding the scene)
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere {
                radius: WORLD_RADIUS,   // Large enough to enclose the scene
            }),
            material: sky_material,
            transform: Transform::from_translation(Vec3::ZERO)
                .with_scale(Vec3::new(-1.0, 1.0, 1.0)), // Invert normals by scaling on one axis
            ..default()
        },
        Name::new("SkyDome"),
    ));

    // Geothermal station base (before radio tower)
    commands.spawn((
        RigidBody::Static,
        Collider::capsule(GEOTHERMAL_BASE_RADIUS, GEOTHERMAL_BASE_HEIGHT),
        PbrBundle {
            mesh: meshes.add(Capsule3d {
                radius: GEOTHERMAL_BASE_RADIUS,
                half_length: GEOTHERMAL_BASE_HEIGHT / 2.0,
            }),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/concrete.png")),
                perceptual_roughness: 0.9,
                metallic: 0.1,
                ..default()
            }),
            transform: Transform::from_translation(GEOTHERMAL_POSITION),
            ..default()
        },
        Name::new("GeothermalBase"),
    ));

    // Radio tower
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(RADIO_TOWER_WIDTH, RADIO_TOWER_HEIGHT, RADIO_TOWER_WIDTH),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(RADIO_TOWER_WIDTH, RADIO_TOWER_HEIGHT, RADIO_TOWER_WIDTH)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/concrete.png")),
                perceptual_roughness: 0.9,
                metallic: 0.1,
                ..default()
            }),
            transform: Transform::from_translation(RADIO_TOWER_POSITION),
            ..default()
        },
        Name::new("RadioTower"),
    ));

    // Large white cylinder tundra
    commands.spawn((
        RigidBody::Static,
        Collider::cylinder(WORLD_RADIUS, 5.0),
        PbrBundle {
            mesh: meshes.add(Cylinder {
                radius: WORLD_RADIUS,
                half_height: 2.5,
            }),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/nasa_arctic.png")),
                perceptual_roughness: 0.2,
                metallic: 0.6,
                ..default()
            }),
            transform: Transform::from_xyz(0.0,0.0, 0.0), 
            ..default()
        },
        Name::new("Tundra"),
    ));   

    // Add connecting perimeter wall between tundra and aquifer
    commands.spawn((
        RigidBody::Static,
        ColliderConstructor::TrimeshFromMesh,
        PbrBundle {
            mesh: meshes.add(Extrusion::new(
                Annulus::new(WORLD_RADIUS - 100.0, WORLD_RADIUS), 
                PERIMETER_WALL_HEIGHT
            )),
            material: materials.add(StandardMaterial {
                base_color: Color::BLACK,           // Changed to pure black
                base_color_texture: None,           // Removed texture
                perceptual_roughness: 0.0,         // Smooth surface
                metallic: 0.0,                     // Non-metallic
                double_sided: true,
                cull_mode: None,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, -PERIMETER_WALL_HEIGHT/2.0, 0.0)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ..default()
        },
        Name::new("PerimeterWall"),
    ));

    // Large white acquifier floor
    commands.spawn((
        RigidBody::Static,
        Collider::cylinder(WORLD_RADIUS, 50.0),
        PbrBundle {
            mesh: meshes.add(Cylinder {
                radius: WORLD_RADIUS,
                half_height: 25.0,
            }),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/ice_texture2.png")),
                perceptual_roughness: 0.2,
                metallic: 0.9,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, ACQUIFIER_FLOOR_DEPTH, 0.0), 
            ..default()
        },
        Name::new("AcquifierFloor"),
    ));    

    // Add invisible floor
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(90.0, 0.2, 90.0),
        Name::new("InvisibleFloor"),
    )).insert(Transform::from_xyz(0.0, -5.2, 0.0));  // 5 units below SubFloor

    // Concrete bridge connecting platform to main area
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(BRIDGE_LENGTH, BRIDGE_HEIGHT, BRIDGE_WIDTH),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(BRIDGE_LENGTH, BRIDGE_HEIGHT, BRIDGE_WIDTH)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/concrete.png")),
                perceptual_roughness: 0.9,
                metallic: 0.1,
                ..default()
            }),
            transform: Transform::from_translation(BRIDGE_POSITION)
                .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_4)),
            ..default()
        },
        Name::new("Bridge"),
    ));

    // Spawn the tram car platform
    commands.spawn((
        RigidBody::Kinematic,
        Collider::cuboid(5.0, 1.0, 5.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(5.0, 1.0, 5.0)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/container_metal.png")),
                metallic: 0.8,
                perceptual_roughness: 0.3,
                ..default()
            }),
            transform: Transform::from_translation(TRAM_POSITION)
                .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_4)),
            ..default()
        },
        TramCar {
            origin: TRAM_POSITION,
            time: 0.0,
            amplitude: 100.0,
            frequency: 0.3,
        },
        Name::new("TramCar"),
    ));

    // Replace the ladder spawn with the new configured version
    spawn_ladder(
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
        LadderConfig {
            position: TOWER_LADDER_START,
            rotation: Quat::IDENTITY,
            height: 150.0,
            rung_count: 200,
        },
    );   

    spawn_ice_cave(&mut commands, &mut meshes, &mut materials, &asset_server);

    spawn_reactor(&mut commands, &mut meshes, &mut materials, &asset_server);
}
