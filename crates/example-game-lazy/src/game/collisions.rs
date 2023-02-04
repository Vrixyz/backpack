use bevy::prelude::*;

use super::{Enemy, PlayerUnit};

#[derive(Component)]
pub struct CollisionTrigger {
    pub is_colliding: bool,
}

pub struct StayCollisionEvent {
    pub entity_player: Entity,
    pub entity_enemy: Entity,
}

pub struct CollisionsPlugin;

impl Plugin for CollisionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StayCollisionEvent>();
        app.add_system(collision_player_enemies);
    }
}

pub(super) fn collision_player_enemies(
    mut transforms: ParamSet<(
        Query<(Entity, &Transform), With<PlayerUnit>>,
        Query<(Entity, &Transform), With<Enemy>>,
    )>,
    mut collision_events: EventWriter<StayCollisionEvent>,
) {
    let player_pos: Vec<_> = transforms
        .p0()
        .iter()
        .map(|(e, t)| (e, t.translation))
        .collect();
    for p_t in player_pos {
        for (e_entity, e_t) in transforms.p1().iter() {
            let distance_to_player = p_t.1.distance(e_t.translation);
            let enemy_size = 100f32;
            let player_size = 200f32;
            if distance_to_player <= enemy_size + player_size {
                collision_events.send(StayCollisionEvent {
                    entity_player: p_t.0,
                    entity_enemy: e_entity,
                })
            }
        }
    }
}
