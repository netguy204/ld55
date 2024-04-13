use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

mod systems;
mod components;


fn update_player(
    mut player: Query<(&mut Velocity, &Transform), With<components::Player>>,
    attractors: Query<(Entity, &Transform), (With<components::Attractor>, Without<components::Player>)>,
    mut commands: Commands,
) {
    // fix this or start the ai i intend?
    for (mut velocity, position) in player.iter_mut() {
        velocity.linvel *= 0.9;
        for (entity, attractor) in attractors.iter() {
            let direction = attractor.translation - position.translation;
            let direction = direction.truncate();
            let distance = direction.length();
            let impulse = direction.normalize_or_zero() * 100.0 / distance;
            if distance < 10.0 {
                commands.entity(entity).despawn();
            }
            println!("impulse: {:?}, distance: {:?}", impulse, distance);
            velocity.linvel += impulse;
        }
    }
}

fn update_placer(
    wm: Res<components::WorldMouse>,
    buttons: Res<Input<MouseButton>>,
    mut placer: Query<&mut Transform, With<components::Placer>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if let Some(pos) = wm.pos {
        for mut placer in placer.iter_mut() {
            placer.translation = pos;
            // println!("placer pos: {:?}", placer.translation);
        }
        if buttons.just_pressed(MouseButton::Left) {
            commands.spawn(components::AttractorBundle::new(asset_server, pos));
        }
    }
}

pub fn movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Velocity, With<components::Player>>) {

    let mut direction = Vec2::ZERO;
    for mut xform in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::A) {
            direction -= Vec2::new(1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::D) {
            direction += Vec2::new(1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::W) {
            direction += Vec2::new(0.0, 1.0);
        }

        if keyboard_input.pressed(KeyCode::S) {
            direction -= Vec2::new(0.0, 1.0);
        }

        xform.linvel = direction * 50.;
    }
}

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let camera = Camera2dBundle::default();
    commands.spawn(camera).insert(components::MainCamera);

    let ldtk_handle = asset_server.load("test.ldtk");
    commands.spawn(LdtkWorldBundle {
        ldtk_handle,
        ..Default::default()
    });

    commands.spawn(components::PlacerBundle {
        placer: components::Placer::default(),
        sprite_bundle: SpriteBundle {
            texture: asset_server.load("attractors.png"),
            ..Default::default()
        }
    });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(LdtkPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, 0.0),
            ..Default::default()
        })
        .insert_resource(LevelSelection::Uid(0))
        .insert_resource(components::WorldMouse::default())
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            set_clear_color: SetClearColor::FromLevelBackground,
            ..Default::default()
        })
        .register_ldtk_int_cell::<components::WallBundle>(2)
        .register_ldtk_entity::<components::PlayerBundle>("Player")
        .add_systems(Startup, setup)
        .add_systems(Update, (systems::camera_follow, systems::mouse_to_world, systems::spawn_wall_collision))
        .add_systems(Update, (update_placer, update_player))
        .run();
}