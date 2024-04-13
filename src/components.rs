use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use bevy_rapier2d::prelude::*;


#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Wall;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct WallBundle {
    wall: Wall,
}


#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Player;


#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    #[sprite_sheet_bundle("sunny_sprites/player.png", 32.0, 32.0, 3, 1, 0.0, 0.0, 1)]
    pub sprite_bundle: SpriteSheetBundle,
    #[from_entity_instance]
    collider: ColliderBundle,
    #[worldly]
    pub worldly: Worldly,
    #[from_entity_instance]
    entity_instance: EntityInstance,
    player: Player,
}

#[derive(Component, Default, Clone)]
pub struct Goal;

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct GoalBundle {
    goal: Goal,
    #[sprite_sheet_bundle("sunny_sprites/cherry.png", 21.0, 21.0, 3, 1, 0.0, 0.0, 1)]
    pub sprite_bundle: SpriteSheetBundle,
    #[worldly]
    pub worldly: Worldly,
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
pub struct AttractorBundle {
    attractor: Attractor,
    pub sprite: SpriteBundle,
}

impl AttractorBundle {
    pub fn new(asset_server: Res<AssetServer>, pos: Vec3) -> Self {
        Self {
            attractor: Attractor,
            sprite: SpriteBundle {
                texture: asset_server.load("attractors.png"),
                transform: Transform::from_translation(pos),
                ..Default::default()
            }
        }
    }
}


#[derive(Component, Default)]
pub struct Placer {
    // selected_item: u8,
}

#[derive(Bundle)]
pub struct PlacerBundle {
    pub placer: Placer,
    pub sprite_bundle: SpriteBundle,
}


#[derive(Resource, Default)]
pub struct WorldMouse {
    pub pos: Option<Vec3>
}
