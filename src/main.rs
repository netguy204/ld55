use bevy::{prelude::*, text::BreakLineOn};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use components::Inventory;

mod systems;
mod components;


fn update_player(
    mut player: Query<(&mut Velocity, &GlobalTransform), With<components::Player>>,
    attractors: Query<(Entity, &GlobalTransform), (With<components::Attractor>, Without<components::Player>)>,
    mut commands: Commands,
) {
    // fix this or start the ai i intend?
    for (mut velocity, position) in player.iter_mut() {
        velocity.linvel *= 0.9;
        for (entity, attractor) in attractors.iter() {
            let direction = attractor.translation() - position.translation();
            let direction = direction.truncate();
            let distance = direction.length();
            let impulse = direction.normalize_or_zero() * 100.0 / (distance);
            if distance < 10.0 {
                commands.entity(entity).despawn();
            }
            // println!("impulse: {:?}, distance: {:?}", impulse, distance);
            velocity.linvel += impulse;
        }
    }
}

fn update_placer(
    wm: Res<components::WorldMouse>,
    buttons: Res<Input<MouseButton>>,
    mut placer: Query<(&mut Inventory, &mut Transform), With<components::Placer>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if let Some(pos) = wm.pos {
        if let Some((mut inventory, mut placer)) = placer.iter_mut().next() {
            placer.translation = pos;
            if buttons.just_pressed(MouseButton::Left) && inventory.count > 0 {
                commands.spawn(components::AttractorBundle::new(asset_server, pos));
                inventory.count -= 1;
            }
        }
    }
}

fn check_win_lose(
    player: Query<(&Transform, &Velocity), With<components::Player>>,
    inventory: Query<&Inventory, With<components::Placer>>,
    goal: Query<(Entity, &Transform), (With<components::Goal>, Without<components::Player>)>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    level: ResMut<LevelSelection>,
) {
    let mut advance = false;
    let indices = match level.into_inner() {
        LevelSelection::Indices(indices) => indices,
        _ => panic!("level selection should be indices"),
    };
    if let Some((player, velocity)) = player.iter().next() {
        for (entity, goal) in goal.iter() {
            let distance = player.translation.distance(goal.translation);
            if distance < 10.0 {
                if indices.level == 1 {
                    commands.spawn(TextBundle {
                        text: Text {
                            sections: vec![TextSection {
                                value: "You Win!".to_string(),
                                style: TextStyle {
                                    font_size: 40.0,
                                    color: Color::WHITE,
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                },
                            }],
                            alignment: TextAlignment::Center,
                            linebreak_behavior: BreakLineOn::WordBoundary,
                        },
                        ..Default::default()
                    });
                } else {
                    advance = true;
                }
                commands.entity(entity).despawn();
            }
            // println!("distance: {:?}", distance)
        }
        // game over if you're out of inventory and not moving
        let inventory = inventory.iter().next().unwrap();
        if inventory.count == 0 && velocity.linvel.length() < 0.01 {
            commands.spawn(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "You Lose!".to_string(),
                        style: TextStyle {
                            font_size: 40.0,
                            color: Color::WHITE,
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        },
                    }],
                    alignment: TextAlignment::Center,
                    linebreak_behavior: BreakLineOn::WordBoundary,
                },
                ..Default::default()
            });
        }
    }
    if advance {
        
        indices.level += 1;
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

#[derive(Component)]
pub struct PlacerText;

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
        placer: components::Placer,
        sprite_bundle: SpriteBundle {
            texture: asset_server.load("attractors.png"),
            ..default()
        },
        inventory: Inventory {
            count: 10,
        },
        ..default()
    }).with_children(|parent| {
        parent.spawn(Text2dBundle{
            text: Text {
                sections: vec![TextSection {
                    value: "Remaining: 0".to_string(),
                    style: TextStyle {
                        font_size: 5.0,
                        color: Color::WHITE,
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    },
                }],
                alignment: TextAlignment::Center,
                linebreak_behavior: BreakLineOn::WordBoundary,
            },
            transform: Transform::from_xyz(0.0, -10.0, 0.0),
            ..Default::default()
        }).insert(PlacerText);
    });
}

pub fn update_count(
    mut query: Query<(&Parent, &mut Text), With<PlacerText>>,
    placers: Query<&components::Inventory, With<components::Placer>>,
) {
    for (parent, mut text) in query.iter_mut() {
        let inventory = placers.get(**parent).unwrap();
        text.sections[0].value = format!("Remaining: {}", inventory.count);
    }
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
        .insert_resource(LevelSelection::Indices(LevelIndices{level: 0, ..default()}))
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
        .register_ldtk_entity::<components::GoalBundle>("Goal")
        .add_systems(Startup, setup)
        .add_systems(Update, (systems::camera_follow, systems::mouse_to_world, systems::spawn_wall_collision))
        .add_systems(Update, (update_placer, update_player, check_win_lose, update_count))
        .run();
}