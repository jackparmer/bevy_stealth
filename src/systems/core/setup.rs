use crate::components::Protagonist;
use crate::resources::ProtagonistAnimations;
use crate::systems::environments::ice_cave::spawn_ice_cave;
use crate::systems::environments::launch_silo::spawn_launch_silo;
use crate::systems::environments::reactor::spawn_reactor;
use crate::systems::environments::geothermal::spawn_geothermal;
use crate::systems::environments::glaciers::spawn_glaciers;
use crate::components::Sentry;
use crate::systems::core::sentry::SentryTiming;
use crate::systems::environments::acquifier::spawn_acquifier;

use avian3d::prelude::*;
use bevy::{
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
};

use bevy::math::Vec3;
use bevy::render::texture::{ImageSampler, ImageAddressMode, ImageSamplerDescriptor};
use bevy::render::view::RenderLayers;

// Constants for structure dimensions

pub const WORLD_RADIUS: f32 = 15000.0;
pub const PERIMETER_WALL_HEIGHT: f32 = 5000.0;
pub const ACQUIFIER_FLOOR_DEPTH: f32 = -1000.0;

pub struct ProtagonistStart {
    pub position: Vec3,
}

pub const PROTAGONIST_START: ProtagonistStart = ProtagonistStart {
    position: Vec3::new(204.0, 3.0, -35.0),
};

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
            camera: Camera {
                order: 99,  // Explicitly set main camera to render first
                ..default()
            },
            transform: Transform::from_xyz(0.7, 0.7, 10.0)
                .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
            ..default()
        },

        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 5.0,
        }

    ));

    // Add Ambient Light
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.1, 0.1, 0.3),
        brightness: 5.0,
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

    // Replace the glacier generation code with:
    spawn_glaciers(&mut commands, &asset_server);

    spawn_geothermal(&mut commands, &mut meshes, &mut materials, &asset_server);

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
            is_outside: false,
            is_birds_eye: false,
            last_climb_toggle: 0.0,
            is_jumping: false,
        },
        SceneBundle {       
            scene: asset_server
                .load(GltfAssetLabel::Scene(0)
                .from_asset("models/Protagonist.glb")),
            transform: Transform::from_translation(PROTAGONIST_START.position),
            ..default()
        },
        RenderLayers::from_layers(&[0, 1]),
        Name::new("Protagonist"),
    )).with_children(|parent| {
        // Add spotlight as child of protagonist
        parent.spawn(SpotLightBundle {
            transform: Transform::from_xyz(0.0, 15.0, 2.0) // Higher and slightly behind
                .looking_at(Vec3::new(0.0, 0.0, -10.0), Vec3::Y), // Point forward and down
            spot_light: SpotLight {
                intensity: 5000000.0, // Increased brightness
                color: Color::srgb(1.0, 0.95, 0.9), // Slightly warmer white
                outer_angle: 0.5,  // Slightly narrower beam
                inner_angle: 0.2,  // More focused core
                shadows_enabled: true,
                range: 50.0, // Explicit range to control falloff
                ..default()
            },
            ..default()
        });
    });

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
                base_color_texture: Some({
                    let texture_handle = asset_server.load("textures/nasa_arctic.png");
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
                perceptual_roughness: 0.2,
                metallic: 0.6,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        RenderLayers::from_layers(&[0, 1]),
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


    spawn_ice_cave(&mut commands, &mut meshes, &mut materials, &asset_server, &time);

    spawn_reactor(&mut commands, &mut meshes, &mut materials, &asset_server);

    // Add initial sentry near protagonist start position
    let initial_sentry_pos = Vec3::new(
        PROTAGONIST_START.position.x + 100.0,
        PROTAGONIST_START.position.y,
        PROTAGONIST_START.position.z
    );
    
    commands.spawn((
        SceneBundle {       
            scene: asset_server
                .load(GltfAssetLabel::Scene(0)
                .from_asset("models/tmpn3hy22ev.glb")),
            transform: Transform::from_translation(initial_sentry_pos)
                .with_scale(Vec3::splat(1.0)),
            ..default()
        },
        Sentry {
            view_distance: 500.0,
            view_angle: std::f32::consts::PI / 2.0,
            follow_speed: 10.0,
            velocity: Vec3::ZERO,
        },
        Name::new("InitialSentry"),
        SentryTiming {
            time_offset: rand::random::<f32>() * 100.0,
        },
    )).with_children(|parent| {
        // Add eerie spotlight as child of sentry
        parent.spawn(SpotLightBundle {
            transform: Transform::from_xyz(0.0, 10.0, 0.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            spot_light: SpotLight {
                intensity: 300000000.0, // Bright!
                color: Color::srgb(0.1, 0.3, 0.8), // Cool blue color
                outer_angle: 0.4, // Narrow beam
                inner_angle: 0.2,
                shadows_enabled: true,
                ..default()
            },
            ..default()
        });
    });

    // Replace aquifer and perimeter wall spawning with:
    spawn_acquifier(&mut commands, &mut meshes, &mut materials, &asset_server);
}
