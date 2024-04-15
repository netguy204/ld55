use std::cmp::min;

use bevy::{asset::AssetMetaCheck, prelude::*, text::BreakLineOn};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use components::*;

mod systems;
mod components;

const LAST_LEVEL: usize = 2;
const PHYSICS_SCALE: f32 = 100.0;
const INITIAL_INVENTORY: u32 = 10;

fn update_player(
    time: Res<Time>,
    mut state: ResMut<CurrentState>,
    mut player: Query<(Entity, &mut Velocity, &GlobalTransform, &mut AnimationTimer, &mut TextureAtlasSprite, &mut Transform, &mut LevelEndTimer), With<Player>>,
    attractors: Query<(Entity, &GlobalTransform), (With<Attractor>, Without<Player>)>,
    goal: Query<(Entity, &Transform), (With<Goal>, Without<Player>)>,
    exit: Query<&GlobalTransform, With<Exit>>,
    rapier: Res<RapierContext>,
    mut commands: Commands,
    mut gizmos: Gizmos
) {
    for (pentity, mut p_vel, p_pos, mut timer, mut atlas, mut p_xform, mut death_timer) in player.iter_mut() {
        // find the closest attractor and move towards it
        let mut closest_d = f32::MAX;
        let mut closest = None;
        p_vel.linvel *= 0.9;

        if state.0 == GameState::Running {
            for (e_attr, p_attr) in attractors.iter() {
                let to_attr = (p_attr.translation() - p_pos.translation()).truncate();
                let distance = to_attr.length();
    
                // verify that the ray doesn't collide with something else first
                let filter = QueryFilter::exclude_dynamic();
                if let Some(_) = rapier.cast_ray(p_pos.translation().truncate(), to_attr, 1.0, false, filter) {
                    continue;
                }
                gizmos.line_2d(p_pos.translation().truncate(), p_attr.translation().truncate(), Color::WHITE);
    
                if distance < closest_d {
                    closest_d = distance;
                    closest = Some(to_attr);
                }
    
                // if the attractor is in range and not the goal then collect it
                if distance < 10.0 && goal.get(e_attr).is_err() {
                    commands.entity(e_attr).despawn();
                }
            }
            if let Some(to_attr) = closest {
                p_vel.linvel += to_attr.normalize_or_zero() * 10.0;
            } else {
                // game over if no longer moving moving
                if p_vel.linvel.length() < 0.01 {
                    state.0 = GameState::GameLose;
                }
            }
        } else if state.0 == GameState::GameWin {
            for e_exit in exit.iter() {
                let to_exit = (e_exit.translation() - p_pos.translation()).truncate();
                p_vel.linvel += to_exit.normalize_or_zero() * 10.0;
                if to_exit.length() < 10.0 {
                    state.0 = GameState::WinDance;
                    commands.entity(pentity).despawn();
                }
            }
        } else if state.0 == GameState::GameLose {
            // advance the goal animation linearly through time
            death_timer.0.tick(time.delta());
            let remaining = death_timer.0.remaining_secs();
            let since_start = death_timer.0.duration().as_secs_f32() - remaining;
            let completion_fraction = since_start / death_timer.0.duration().as_secs_f32();

            // animation is in frame 5-8
            let frame = (completion_fraction * 4.0) as usize;
            atlas.index = min(8, 4 + frame);
        }
        

        if p_vel.linvel.length() > 10.0 {
            if timer.0.tick(time.delta()).just_finished() {
                atlas.index = (atlas.index + 1) % 4;
            }
            // the sprite native orientation is straight up. orient it in the direction of the velocity vector
            let angle = p_vel.linvel.angle_between(Vec2::Y);
            p_xform.rotation = Quat::from_rotation_z(-angle);
        }
        
    }
}


#[derive(Component)]
struct HUD;

impl HUD {
    fn spawn(
        commands: &mut Commands,
        asset_server: Res<AssetServer>,
    ) {
        commands.spawn(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "Hello, World!".to_string(),
                    style: TextStyle {
                        font_size: 40.0,
                        color: Color::WHITE,
                        font: asset_server.load("fonts/PixelifySans-Bold.ttf"),
                    },
                }],
                alignment: TextAlignment::Center,
                linebreak_behavior: BreakLineOn::WordBoundary,
            },
            ..Default::default()
        }).insert(HUD);
    }
}

fn update_hud(
    mut query: Query<&mut Text, With<HUD>>,
    state: Res<CurrentState>,
) {
    for mut text in query.iter_mut() {
        match state.0 {
            GameState::Instructions => {}
            GameState::Focusing => {
                text.sections[0].value = "Focusing".to_string();
            }
            GameState::Planning => {
                text.sections[0].value = "Planning. Click to place. WASD to move camera. Space to Execute.".to_string();
            }
            GameState::Running => {
                text.sections[0].value = "Running".to_string();
            }
            GameState::AdvanceLevel => {
                text.sections[0].value = "Level Complete!".to_string();
            }
            GameState::GameWin => {
                text.sections[0].value = "You Win!".to_string();
            }
            GameState::WinDance => {
                text.sections[0].value = "Close the window to exit. Press R to restart.".to_string();
            }
            GameState::GameLose => {
                text.sections[0].value = "The raccoon starved to death! Press R to restart.".to_string();
            }
        
        }
    }
}

fn animate_exit(
    time: Res<Time>,
    state: Res<CurrentState>,
    mut exit: Query<(&mut AnimationTimer, &mut TextureAtlasSprite), With<Exit>>,
) {
    if state.0 == GameState::GameWin {
        for (mut timer, mut sprite) in exit.iter_mut() {
            if timer.0.tick(time.delta()).just_finished() {
                sprite.index = (sprite.index + 1) % 4;
            }
        }
    }
    if state.0 == GameState::WinDance {
        for (mut timer, mut sprite) in exit.iter_mut() {
            if timer.0.tick(time.delta()).just_finished() {
                sprite.index = 4 + ((sprite.index + 1) % 3);
            }
        }
    }
}

fn check_win(
    player: Query<&Transform, With<Player>>,
    goal: Query<&Transform, (With<Goal>, Without<Player>)>,
    mut state: ResMut<CurrentState>,
) {
    if state.0 == GameState::Running {
        if let Some(player) = player.iter().next() {
            for goal in goal.iter() {
                let distance = player.translation.distance(goal.translation);
                if distance < 10.0 {
                    state.0 = GameState::AdvanceLevel;
                }
            }
        }
    }
}

fn update_state(
    time: Res<Time>,
    mut state: ResMut<CurrentState>,
    level: ResMut<LevelSelection>,
    focus: Query<&GlobalTransform, With<CameraFocus>>,
    player: Query<Entity, With<Player>>,
    placer: Query<Entity, With<Placer>>,
    mut goal: Query<(&mut LevelEndTimer, &mut TextureAtlasSprite), With<Goal>>,
    mut camera: Query<&mut Transform, With<MainCamera>>,
    mut commands: Commands,
) {
    let indices = match level.into_inner() {
        LevelSelection::Indices(indices) => indices,
        _ => panic!("level selection should be indices"),
    };
    let placer = placer.iter().next();
    if state.0 == GameState::Running {
        if let Some(placer) = placer {
            commands.entity(placer).insert(Visibility::Hidden);
        }
    }
    if state.0 == GameState::Focusing {
        for focus in &focus {
            for mut camera in camera.iter_mut() {
                if focus.translation() != Vec3::new(0.0, 0.0, 0.0) {
                    camera.translation = focus.translation();
                    if indices.level > LAST_LEVEL {
                        state.0 = GameState::GameWin;
                        if let Some(placer) = placer {
                            commands.entity(placer).insert(Visibility::Hidden);
                        }
                    } else {
                        state.0 = GameState::Planning;
                        if let Some(placer) = placer {
                            commands.entity(placer).insert(Visibility::Inherited);
                        }
                    }
                }
            }
        }
    }
    
    if state.0 == GameState::AdvanceLevel {
        let (mut level_end_timer, mut goal_anim) = goal.single_mut();
        if level_end_timer.0.tick(time.delta()).just_finished() {
            indices.level += 1;
            state.0 = GameState::Focusing;
        } else {
            // despawn the player if they're still around
            for player in player.iter() {
                commands.entity(player).despawn();
            }
            // advance the goal animation linearly through time
            let remaining = level_end_timer.0.remaining_secs();
            let since_start = level_end_timer.0.duration().as_secs_f32() - remaining;
            let completion_fraction = since_start / level_end_timer.0.duration().as_secs_f32();

            // animation is in frame 1-7
            let frame = (completion_fraction * 6.0) as usize;
            goal_anim.index = 1 + frame;
        }
    }
}

fn update_placer(
    state: Res<CurrentState>,
    wm: Res<WorldMouse>,
    buttons: Res<Input<MouseButton>>,
    mut placer: Query<(&mut Inventory, &mut Transform), With<Placer>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if state.0 == GameState::Planning {
        if let Some(pos) = wm.pos {
            if let Some((mut inventory, mut placer)) = placer.iter_mut().next() {
                placer.translation = pos.xy().extend(2.0);
                if buttons.just_pressed(MouseButton::Left) && inventory.count > 0 {
                    commands.spawn(GoodieBundle::new(&asset_server, pos.xy().extend(2.0)));
                    inventory.count -= 1;
                }
            }
        }
    }
}


pub fn controls(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut inventory: Query<&mut Inventory>,
    mut camera: Query<(&mut Transform, &mut OrthographicProjection), With<MainCamera>>,
    mut state: ResMut<CurrentState>,
    mut level: ResMut<LevelSelection>,
    instructions: Query<Entity, With<Instructions>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if state.0 == GameState::Instructions {
        if keyboard_input.just_pressed(KeyCode::Space) {
            if let Some(instructions) = instructions.iter().next() {
                commands.entity(instructions).despawn();
                state.0 = GameState::Focusing;
                finish_setup(commands, asset_server);
            }
        }
    }
    if keyboard_input.just_pressed(KeyCode::R) {
        state.0 = GameState::Focusing;
        *level = LevelSelection::Indices(LevelIndices{level: 0, ..default()});
        if let Some(mut inventory) = inventory.iter_mut().next() {
            inventory.count = INITIAL_INVENTORY;
        }
    }
    if state.0 == GameState::Planning {
        let (mut camera, mut proj) = camera.single_mut();
        if keyboard_input.just_pressed(KeyCode::Space) {
            state.0 = GameState::Running;
        }
        proj.scale = 0.5;
        let camera_speed = 200.0;
        if keyboard_input.pressed(KeyCode::W) {
            camera.translation.y += camera_speed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::S) {
            camera.translation.y -= camera_speed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::D) {
            camera.translation.x += camera_speed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::A) {
            camera.translation.x -= camera_speed * time.delta_seconds();
        }
    }
}

#[derive(Component)]
pub struct Instructions;

#[derive(Component)]
pub struct PlacerText;

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let camera = Camera2dBundle::default();
    commands.spawn(camera).insert(MainCamera);

    commands.spawn(SpriteBundle {
        texture: asset_server.load("instructions.png"),
        transform: Transform::from_xyz(0.0, 0.0, -2.0).with_scale(Vec3::splat(1.5)),
        ..Default::default()
    }).insert(Instructions);
}

pub fn finish_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let ldtk_handle = asset_server.load("attic.ldtk");
    commands.spawn(LdtkWorldBundle {
        ldtk_handle,
        ..Default::default()
    });

    commands.spawn(PlacerBundle {
        placer: Placer,
        sprite_sheet_bundle: GoodieBundle::spritesheet(&asset_server, Vec3::new(0.0, 0.0, 0.0)),
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
                        font_size: 10.0,
                        color: Color::WHITE,
                        font: asset_server.load("fonts/PixelifySans-Bold.ttf"),
                    },
                }],
                alignment: TextAlignment::Center,
                linebreak_behavior: BreakLineOn::WordBoundary,
            },
            transform: Transform::from_xyz(0.0, -10.0, 0.0),
            ..Default::default()
        }).insert(PlacerText);
    });
    let hud = HUD::spawn(&mut commands, asset_server);
    commands.spawn(hud);
}

pub fn update_count(
    mut query: Query<(&Parent, &mut Text), With<PlacerText>>,
    placers: Query<&Inventory, With<Placer>>,
) {
    for (parent, mut text) in query.iter_mut() {
        let inventory = placers.get(**parent).unwrap();
        text.sections[0].value = format!("Remaining: {}", inventory.count);
    }
}

#[derive(Component, Default, Clone)]
struct CameraFocus;

#[derive(Bundle, LdtkEntity, Default, Clone)]
struct CameraFocusBundle {
    focus: CameraFocus,
    transform: TransformBundle,
    #[from_entity_instance]
    entity_instance: EntityInstance,
}


fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never) // meta confuses wasm
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(LdtkPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PHYSICS_SCALE))
        // .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, 0.0),
            ..Default::default()
        })
        .insert_resource(LevelSelection::Indices(LevelIndices{level: 0, ..default()}))
        .insert_resource(WorldMouse::default())
        .insert_resource(CurrentState::default())
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            set_clear_color: SetClearColor::FromLevelBackground,
            ..Default::default()
        })
        .register_ldtk_int_cell::<WallBundle>(2)
        .register_ldtk_entity::<PlayerBundle>("Raccoon")
        .register_ldtk_entity::<GoalBundle>("Trash")
        .register_ldtk_entity::<GarbageBundle>("Garbage")
        .register_ldtk_entity::<ExitBundle>("Exit")
        .register_ldtk_entity::<CameraFocusBundle>("Focus")
        .add_systems(Startup, setup)
        .add_systems(Update, (systems::camera_follow, systems::mouse_to_world, systems::spawn_wall_collision))
        .add_systems(Update, (update_placer, update_count, controls, update_hud, update_state, animate_exit, check_win))
        .add_systems(PostUpdate, update_player)
        .run();
}