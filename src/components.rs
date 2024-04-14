use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use bevy_rapier2d::prelude::*;


#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Wall;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct WallBundle {
    wall: Wall,
}


#[derive(Component, Clone)]
pub struct AnimationTimer(pub Timer);

impl Default for AnimationTimer {
    fn default() -> Self {
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Player;


#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    player: Player,
    #[sprite_sheet_bundle("raccoon.png", 32.0, 32.0, 4, 1, 0.0, 0.0, 0)]
    pub sprite_bundle: SpriteSheetBundle,
    #[from_entity_instance]
    collider: ColliderBundle,
    // #[worldly]
    // pub worldly: Worldly,
    #[from_entity_instance]
    entity_instance: EntityInstance,
    #[grid_coords]
    grid_coords: GridCoords,
    paused: Paused,
    animation_timer: AnimationTimer,
}

#[derive(Component, Default, Clone)]
pub struct Goal;

#[derive(Component, Clone)]
pub struct LevelEndTimer(pub Timer);

impl Default for LevelEndTimer {
    fn default() -> Self {
        LevelEndTimer(Timer::from_seconds(1.0, TimerMode::Once))
    }
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct GoalBundle {
    end_level_timer: LevelEndTimer,
    attractor: Attractor,
    goal: Goal,
    #[sprite_sheet_bundle]
    pub sprite_bundle: SpriteSheetBundle,
    // #[worldly]
    // pub worldly: Worldly,
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Clone, Default, Bundle, LdtkIntCell)]
pub struct ColliderBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub rotation_constraints: LockedAxes,
    pub gravity_scale: GravityScale,
    pub friction: Friction,
    pub density: ColliderMassProperties,
}


impl From<&EntityInstance> for ColliderBundle {
    fn from(entity_instance: &EntityInstance) -> ColliderBundle {
        let rotation_constraints = LockedAxes::ROTATION_LOCKED;

        match entity_instance.identifier.as_ref() {
            "Player" => ColliderBundle {
                collider: Collider::cuboid(6., 14.),
                rigid_body: RigidBody::Dynamic,
                friction: Friction {
                    coefficient: 0.0,
                    combine_rule: CoefficientCombineRule::Min,
                },
                rotation_constraints,
                ..Default::default()
            },
            _ => ColliderBundle::default(),
        }
    }
}


#[derive(Clone, Default, Bundle, LdtkIntCell)]
pub struct SensorBundle {
    pub collider: Collider,
    pub sensor: Sensor,
    pub active_events: ActiveEvents,
    pub rotation_constraints: LockedAxes,
}
impl From<IntGridCell> for SensorBundle {
    fn from(int_grid_cell: IntGridCell) -> SensorBundle {
        let rotation_constraints = LockedAxes::ROTATION_LOCKED;

        // wall
        if int_grid_cell.value == 2 {
            SensorBundle {
                collider: Collider::cuboid(8., 8.),
                sensor: Sensor,
                rotation_constraints,
                active_events: ActiveEvents::COLLISION_EVENTS,
            }
        } else {
            SensorBundle::default()
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Attractor;


#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct GoodieBundle {
    attractor: Attractor,
    pub sprite: SpriteSheetBundle,
}

impl GoodieBundle {
    pub fn new(asset_server: &Res<AssetServer>, pos: Vec3) -> Self {
        Self {
            attractor: Attractor,
            sprite: GoodieBundle::spritesheet(asset_server, pos),
        }
    }

    pub fn spritesheet(asset_server: &Res<AssetServer>, pos: Vec3) -> SpriteSheetBundle {
        let texture_atlas = TextureAtlas::from_grid(
            asset_server.load("garbage.png"), 
            Vec2::new(32.0, 32.0), 
            3,
            1,
            None, None
        );
        let texture_atlas = asset_server.add(texture_atlas);
        SpriteSheetBundle {
            sprite: TextureAtlasSprite{
                index: 2,
                ..default()
            },
            texture_atlas,
            transform: Transform::from_translation(pos),
            ..default()
        }
    }
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
struct GarbageBundle {
    attractor: Attractor,
    #[sprite_sheet_bundle]
    sprite_bundle: SpriteSheetBundle,
}


#[derive(Component, Default)]
pub struct Placer;


#[derive(Component, Default)]
pub struct Inventory {
    pub count: u32,
}


#[derive(Bundle, Default)]
pub struct PlacerBundle {
    pub placer: Placer,
    pub sprite_sheet_bundle: SpriteSheetBundle,
    pub inventory: Inventory,
}


#[derive(Resource, Default)]
pub struct WorldMouse {
    pub pos: Option<Vec3>
}

#[derive(Component, Default, Clone)]
pub struct Paused;