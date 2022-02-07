use std::time::Duration;

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::input::{ keyboard::KeyCode, Input };

use bevy_rapier2d::prelude::*;
use bevy_framepace::{ FramepacePlugin, FramerateLimit };
use bevy_inspector_egui::{ WorldInspectorPlugin, InspectableRegistry };

mod health;
mod bullet;
mod ship;
mod player_ship;
mod mouse_pos;
mod debug_systems;
mod layer_system;
mod inspector_impls;
mod collision_daemon;

use crate::collision_daemon::CollisionDaemonPlugin;
use crate::health::HealthComponent;
use crate::ship::{ ShipPlugin, ShipResources };
use crate::bullet::{ BulletPlugin, BulletResources, BulletAttributes };
use crate::player_ship::{ PlayerShipPlugin, PlayerShipBundle };
use crate::layer_system::{ Layer, LayerComponent, LayerPlugin };
use crate::debug_systems::GameDebugPlugin;

const WINDOW_HEIGHT : f32 = 675.0f32;
const WINDOW_WIDTH : f32 = 1200.0f32;
const ASPECT_RATIO : f32 = WINDOW_WIDTH / WINDOW_HEIGHT;

static TITLE : &'static str = "Solar Knight";

// TODO shooting
// TODO some enemies
fn test_setup(
    mut commands : Commands,
    mut rapier_config : ResMut<RapierConfiguration>,
    ship_reses : Res<ShipResources>,
    bullet_reses : Res<BulletResources>,
    asset_server : Res<AssetServer>,
) {
    rapier_config.gravity = vector!(0.0f32, 0.0f32);

    commands.spawn()
    .insert_bundle(OrthographicCameraBundle {
        orthographic_projection : OrthographicProjection {
            scaling_mode : ScalingMode::FixedVertical,
            ..OrthographicCameraBundle::new_2d().orthographic_projection
        },
        ..OrthographicCameraBundle::new_2d()
    })
    .insert(Name::new("Gameplay camera"));

    commands.spawn()
    .insert(Name::new("Background"))
    .insert_bundle(SpriteBundle {
        transform : Transform {
            translation : Vec3::new(0.0f32, 0.0f32, 0.0f32), 
            ..Default::default()
        },
        texture : asset_server.load("textures/background.png"),
        sprite : Sprite {
            custom_size : Some(Vec2::new(2.0f32 * WINDOW_WIDTH / WINDOW_HEIGHT, 2.0f32)),
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(
        LayerComponent {
            layer : Layer::BackgroundLayer,
            internal_offset : 0.0f32,
        }
    );

    bullet::spawn_test_bullet(&mut commands, &bullet_reses, 0.5f32, 0.0f32);
    
    commands.spawn()
    .insert_bundle(PlayerShipBundle::new(&ship_reses))
    .insert(Name::new("Player"))
    ;
  
    commands.spawn()
    .insert(Name::new("Bottom wall"))
    .insert_bundle(ColliderBundle {
        shape : ColliderShape::cuboid(3.0, 1.0).into(),
        position : Isometry::new(
            [0.0f32, -2.0f32].into(),
            0.0f32,
            //std::f32
        ).into(),
        ..ColliderBundle::default()
    })
    .insert(ColliderPositionSync::Discrete)
    ;
    
    commands.spawn()
    .insert(Name::new("Top wall"))
    .insert_bundle(ColliderBundle {
        shape : ColliderShape::cuboid(3.0, 1.0).into(),
        position : Isometry::new(
            [0.0f32, 2.0f32].into(),
            0.0f32,
            //std::f32
        ).into(),
        ..ColliderBundle::default()
    })
    .insert(ColliderPositionSync::Discrete)
    ;
    
    commands.spawn()
    .insert(Name::new("Left wall"))
    .insert_bundle(ColliderBundle {
        shape : ColliderShape::cuboid(3.0, 1.0).into(),
        position : Isometry::new(
            [-1.0f32 - ASPECT_RATIO, 0.0f32].into(),
            std::f32::consts::FRAC_PI_2
        ).into(),
        ..ColliderBundle::default()
    })
    .insert(ColliderPositionSync::Discrete)
    ;
    
    commands.spawn()
    .insert(Name::new("Right wall"))
    .insert_bundle(ColliderBundle {
        shape : ColliderShape::cuboid(3.0, 1.0).into(),
        position : Isometry::new(
            [1.0f32 + ASPECT_RATIO, 0.0f32].into(),
            std::f32::consts::FRAC_PI_2
        ).into(),
        ..ColliderBundle::default()
    })
    .insert(ColliderPositionSync::Discrete)
    ;
}

// Inittializes the app, loading the
// base components
fn create_app_base() -> App {
    let window_descriptor = 
        bevy::window::WindowDescriptor {
            title : TITLE.to_owned(),
            resizable : false,
            width : WINDOW_WIDTH,
            height : WINDOW_HEIGHT,
            ..Default::default()
        }
    ;

    let mut app = App::new();

    app.insert_resource(window_descriptor)
    .add_plugins(DefaultPlugins)
    .add_plugin(
        FramepacePlugin::framerate(60)
        .without_warnings()
    );

    app
}

fn load_gameplay_plugins(app : &mut App) {
    app
    .add_plugin(RapierPhysicsPlugin::<()>::default())
    .add_plugin(LayerPlugin)
    .add_plugin(ShipPlugin)
    .add_plugin(CollisionDaemonPlugin::<BulletAttributes>::new())
    .add_plugin(BulletPlugin)
    .add_plugin(PlayerShipPlugin);
}

fn load_debug_plugins(app : &mut App) {
    app.add_plugin(GameDebugPlugin);

    app.add_plugin(WorldInspectorPlugin::new());

    let mut registry =
        app.world.get_resource_mut::<InspectableRegistry>()
        .unwrap()
    ;
    registry.register::<LayerComponent>();
    registry.register::<BulletAttributes>();
    registry.register::<HealthComponent>();
    registry.register_raw(|comp : &mut ColliderPositionComponent, ui, ctx| {
        use inspector_impls::nalgebra::inspect_vec2;
        use bevy_inspector_egui::options::Vec2dAttributes;

        inspect_vec2(&mut comp.translation.vector, ui, Vec2dAttributes::default(), ctx)
    });
    
    registry.register_raw(|comp : &mut RigidBodyPositionComponent, ui, ctx| {
        use inspector_impls::nalgebra::inspect_vec2;
        use bevy_inspector_egui::options::Vec2dAttributes;

        inspect_vec2(&mut comp.position.translation.vector, ui, Vec2dAttributes::default(), ctx)
    });
}

fn main() {
    let debug = true;

    let mut app = create_app_base();

    load_gameplay_plugins(&mut app);
    if debug { load_debug_plugins(&mut app); }
    
    app.add_startup_system(test_setup);
    
    app.run();
}
