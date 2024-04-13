use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
struct Wall;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
struct WallBundle {
    wall: Wall
}


#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Player;


#[derive(Clone, Default, Bundle, LdtkEntity)]
struct PlayerBundle {
    #[sprite_sheet_bundle("sunny_sprites/player.png", 32.0, 32.0, 3, 1, 0.0, 0.0, 1)]
    pub sprite_bundle: SpriteSheetBundle,
    #[worldly]
    pub worldly: Worldly,
    #[from_entity_instance]
    entity_instance: EntityInstance,
    player: Player,
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let camera = Camera2dBundle::default();
    commands.spawn(camera);

    let ldtk_handle = asset_server.load("test.ldtk");
    commands.spawn(LdtkWorldBundle {
        ldtk_handle,
        ..Default::default()
    });
}

pub fn movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>) {

    let mut direction = Vec3::ZERO;
    for mut xform in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::A) {
            direction -= Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::D) {
            direction += Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::W) {
            direction += Vec3::new(0.0, 1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::S) {
            direction -= Vec3::new(0.0, 1.0, 0.0);
        }

        xform.translation += time.delta_seconds() * direction * 50.;
    }
}

fn camera_follow(
    player: Query<&Transform, With<Player>>,
    mut camera: Query<(&mut Transform, &mut OrthographicProjection), (With<Camera>, Without<Player>)>,
) {
    if player.is_empty() || camera.is_empty() {
        return;
    }

    let player = player.single();
    let (mut camera_xform, mut camera_proj) = camera.single_mut();
    camera_xform.translation = player.translation;
    camera_proj.scale = 0.25;
}


fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(LdtkPlugin)
        .insert_resource(LevelSelection::Uid(0))
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            set_clear_color: SetClearColor::FromLevelBackground,
            ..Default::default()
        })
        .register_ldtk_int_cell::<WallBundle>(2)
        .register_ldtk_entity::<PlayerBundle>("Player")
        .add_systems(Startup, setup)
        .add_systems(Update, (movement, camera_follow))
        .run();
}