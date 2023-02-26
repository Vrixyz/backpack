use bevy::prelude::*;

use super::{
    ui_playing::{ui_score, ui_scoring},
    GameState, PlayerUnit,
};

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>();
        app.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(init_scoring));
        app.add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(collision_scoring)
                .with_system(update_scoring)
                .with_system(ui_scoring)
                .with_system(ui_score),
        );
    }
}

#[derive(Resource, Default)]
pub struct Score {
    pub score: i32,
}

#[derive(PartialEq)]
pub struct Scoring {
    pub entity_first_colliding: Entity,
    pub start_time: f32,
}

#[derive(Component, PartialEq)]
pub enum ScoreNear {
    Scoring(Scoring),
    NotNear,
    Gained,
}
#[derive(Component)]
pub struct ScoreNearDef {
    pub time_to_score: f32,
    pub score: i32,
}

fn init_scoring(time: Res<Time>, mut score: ResMut<Score>) {
    score.score = 0;
}

fn update_scoring(
    time: Res<Time>,
    mut score: ResMut<Score>,
    mut query: Query<(Entity, &mut ScoreNear, &ScoreNearDef)>,
) {
    for (e, mut score_near, def) in query.iter_mut() {
        match &*score_near {
            ScoreNear::Scoring(scoring) => {
                scoring.start_time + def.time_to_score < time.elapsed_seconds();
                if scoring.start_time + def.time_to_score < time.elapsed_seconds() {
                    score.score += def.score;
                    *score_near = ScoreNear::Gained;
                }
            }
            ScoreNear::NotNear => {}
            ScoreNear::Gained => {}
        }
    }
}

pub(super) fn collision_scoring(
    time: Res<Time>,
    mut transforms: ParamSet<(
        Query<(Entity, &Transform), With<PlayerUnit>>,
        Query<(Entity, &Transform, &mut ScoreNear)>,
    )>,
) {
    let player_pos: Vec<_> = transforms
        .p0()
        .iter()
        .map(|(e, t)| (e, t.translation))
        .collect();
    for p_t in player_pos {
        for (e_entity, e_t, mut score) in transforms.p1().iter_mut() {
            if matches!(*score, ScoreNear::Gained) {
                continue;
            }
            let distance_to_player = p_t.1.distance(e_t.translation);
            let enemy_size = 48f32 + 250f32;
            let player_size = 128f32;
            if distance_to_player <= enemy_size + player_size {
                if *score == ScoreNear::NotNear {
                    *score = ScoreNear::Scoring(Scoring {
                        entity_first_colliding: p_t.0,
                        start_time: time.elapsed_seconds(),
                    });
                }
            } else if let ScoreNear::Scoring(scoring) = &*score {
                // It's not ideal because if another player entity is near, we'll miss a frame,
                // where the scorer will be marked as "NotNear".
                if scoring.entity_first_colliding == p_t.0 {
                    *score = ScoreNear::NotNear;
                }
            }
        }
    }
}
