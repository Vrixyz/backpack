mod collisions;
mod scoreboard;
mod scoring;
mod ui_endscreen;
mod ui_playing;
mod ui_warmup;

use std::time::Duration;

use bevy::{ecs::schedule::ScheduleLabel, math::Vec3Swizzles, prelude::*};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use lerp::Lerp;
use particles::ParticleExplosion;
use rand::prelude::*;

use crate::{
    backpack_client_bevy::{bevy_modify_item, GetItemsTaskResultEvent, ModifyItemTaskResultEvent},
    data::ItemId,
    utils::{
        self,
        mouse::{self, GameCamera, MousePos},
    },
    AuthData, BackpackCom, BackpackItems,
};

use self::{
    collisions::StayCollisionEvent,
    scoring::{Score, ScoreNear, ScoreNearDef},
};

pub struct Game;

#[derive(Resource)]
struct GameAssets {
    pub player: Handle<Image>,
    pub enemy: Handle<Image>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, States, ScheduleLabel)]
pub enum GameState {
    Warmup,
    LoadingPlay,
    Playing,
    EndScreen,
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Warmup
    }
}

#[derive(Resource, Default)]
pub struct GameDef {
    pub enemy_count: u32,
}
#[derive(Resource, Default)]
pub struct GameDefBorder {
    pub borders: Vec2,
}

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct PlayerUnit;

#[derive(Component, PartialEq, Eq)]
enum CollisionState {
    Colliding,
    NoCollision,
}

#[derive(Component)]
struct WantedMovement {
    pub direction: Vec2,
}
#[derive(Debug, Resource, Eq, PartialEq)]
enum LoadingPlayState {
    Unknown,
    Init,
    WaitingResponse,
    Failed,
    Ok,
}

impl Plugin for Game {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameAssets {
            player: Handle::default(),
            enemy: Handle::default(),
        });
        app.insert_resource(GameDefBorder {
            borders: Vec2::new(2000f32, 2000f32),
        });
        app.insert_resource(LoadingPlayState::Unknown);
        app.add_plugin(mouse::MousePlugin);
        // TODO: bevy 0.10
        // app.add_plugin(DebugLinesPlugin::default());
        app.add_plugin(collisions::CollisionsPlugin);
        app.add_plugin(scoreboard::ScoreboardPlugin);
        app.add_plugin(scoring::ScorePlugin);
        app.add_plugin(particles::ParticlesPlugin);
        app.init_resource::<GameDef>();
        app.add_startup_system(load_assets);
        app.add_state::<GameState>();
        app.add_system_set(
            SystemSet::on_update(GameState::Warmup)
                .with_system(
                    update_wanted_movement_player
                        .before(update_movement)
                        .after(mouse::my_cursor_system),
                )
                .with_system(collision_warmup.after(collisions::collision_player_enemies))
                .with_system(clear_collision_warmup.before(collision_warmup))
                .with_system(ui_warmup::ui_warmup)
                .with_system(ui_warmup::ui_tuto_start)
                .with_system(ui_warmup::handle_tap_to_start.after(collision_warmup)),
        );
        app.add_system_set(
            SystemSet::on_enter(GameState::LoadingPlay).with_system(init_loading_play),
        );
        app.add_system_set(
            SystemSet::on_update(GameState::LoadingPlay)
                .with_system(loading_play_use_currency.before(handle_modify_result)),
        );
        app.add_system_set(
            SystemSet::on_update(GameState::LoadingPlay).with_system(handle_modify_result),
        );
        app.add_system_set(
            SystemSet::on_update(GameState::Playing).with_system(
                update_wanted_movement_player
                    .before(update_movement)
                    .after(mouse::my_cursor_system),
            ),
        );
        app.add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(
                    update_collisions_player_playing.after(collisions::collision_player_enemies),
                )
                .with_system(juice_collisions)
                .with_system(juice_score),
        );
        app.add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(ui_playing::ui_playing)
                .with_system(more_enemies),
        );
        app.add_system_set(
            SystemSet::on_enter(GameState::Warmup)
                .with_system(utils::despawn::<PlayerUnit>)
                .before(create_player),
        );
        app.add_system_set(
            SystemSet::on_enter(GameState::Warmup)
                .with_system(utils::despawn::<Enemy>)
                .with_system(create_player),
        );
        app.add_system(ui_warmup::handle_get_items_result);
        app.add_system(ui_warmup::handle_modify_item_result);
        app.add_system(update_movement.before(collisions::collision_player_enemies))
            .add_system(bounce_enemies.after(update_movement))
            .add_system(update_enemy_count);

        app.add_system_set(
            SystemSet::on_update(GameState::EndScreen)
                .with_system(ui_endscreen::ui_endscreen)
                .with_system(ui_endscreen::ui_end_title_and_score),
        );
    }
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameAssets {
        player: asset_server.load("bevy.png"),
        enemy: asset_server.load("bevy_pixel_light.png"),
    });
    let camera = Camera2dBundle {
        projection: OrthographicProjection {
            scale: 10.0,
            ..default()
        },
        ..Default::default()
    };
    commands.spawn((camera, GameCamera));
}

fn create_player(
    mut commands: Commands,
    mut game_def: ResMut<GameDef>,
    game_def_borders: Res<GameDefBorder>,
    assets: Res<GameAssets>,
    //mut lines: ResMut<DebugLines>,
) {
    game_def.enemy_count = 0;
    commands.spawn((
        SpriteBundle {
            texture: assets.player.clone(),
            transform: Transform::from_translation(Vec3::ZERO),
            ..default()
        },
        PlayerUnit,
        WantedMovement {
            direction: Vec2::ZERO,
        },
        CollisionState::NoCollision,
    ));
    let borders = [
        (game_def_borders.borders * Vec2::new(1f32, 1f32)).extend(0f32),
        (game_def_borders.borders * Vec2::new(1f32, -1f32)).extend(0f32),
        (game_def_borders.borders * Vec2::new(-1f32, -1f32)).extend(0f32),
        (game_def_borders.borders * Vec2::new(-1f32, 1f32)).extend(0f32),
    ];

    for i in 0..borders.len() {
        //lines.line(borders[i], borders[(i + 1) % borders.len()], f32::INFINITY);
    }
}

fn update_enemy_count(
    mut commands: Commands,
    assets: Res<GameAssets>,
    def: Res<GameDef>,
    borders: Res<GameDefBorder>,
    q_enemies: Query<Entity, With<Enemy>>,
) {
    if !def.is_changed() {
        return;
    }
    let diff = def.enemy_count as i32 - q_enemies.iter().count() as i32;
    if diff > 0 {
        let mut rng = rand::thread_rng();
        for _ in 0..diff {
            commands.spawn((
                SpriteBundle {
                    texture: assets.enemy.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        rng.gen_range(-borders.borders.x..borders.borders.x),
                        rng.gen_range(-borders.borders.y..borders.borders.y),
                        0f32,
                    )),
                    ..default()
                },
                Enemy,
                WantedMovement {
                    direction: random_point_circle(750f32, false),
                },
                ScoreNear::NotNear,
                ScoreNearDef {
                    time_to_score: 2.5,
                    score: 1,
                },
            ));
        }
    } else {
        for e in q_enemies.iter().take(-diff as usize) {
            commands.entity(e).despawn();
        }
    }
}

fn random_point_circle(range: f32, random_range_value: bool) -> Vec2 {
    let mut range = range;
    let angle = rand::thread_rng().gen_range(0f32..(1f32 * std::f32::consts::TAU));
    // if inside
    if random_range_value {
        range *= rand::thread_rng().gen_range(0f32..1f32).sqrt();
    }
    let x = range * angle.cos();
    let y = range * angle.sin();
    Vec2 { x, y }
}

fn update_movement(
    time: Res<Time>,
    borders: Res<GameDefBorder>,
    mut q_movers: Query<(&mut Transform, &WantedMovement)>,
) {
    for (mut t, movement) in q_movers.iter_mut() {
        t.translation += (movement.direction * time.delta_seconds()).extend(0f32);
        t.translation.y = t.translation.y.clamp(-borders.borders.x, borders.borders.x);
        t.translation.x = t.translation.x.clamp(-borders.borders.y, borders.borders.y);
    }
}
fn update_wanted_movement_player(
    time: Res<Time>,
    mut q_movers: Query<(&Transform, &mut WantedMovement), With<PlayerUnit>>,
    mouse: Res<MousePos>,
) {
    if !mouse.is_changed() {
        return;
    }
    for (t, mut wanted_movement) in q_movers.iter_mut() {
        let to_target = mouse.0 - t.translation.truncate();
        let to_target_length = to_target.length();

        if to_target_length < 1f32 {
            wanted_movement.direction = Vec2::ZERO;
            continue;
        }
        let target_speed = 1500f32;
        // aka slowing radius
        // from https://gamedevelopment.tutsplus.com/tutorials/understanding-steering-behaviors-flee-and-arrival--gamedev-1303
        let estimated_move_in_a_frame = time.delta_seconds() * target_speed;

        if to_target_length < estimated_move_in_a_frame {
            wanted_movement.direction = to_target.normalize_or_zero()
                * target_speed
                * (to_target_length / estimated_move_in_a_frame)
        } else {
            wanted_movement.direction = to_target.normalize_or_zero() * target_speed;
        }
    }
}

fn bounce_enemies(
    mut borders: Res<GameDefBorder>,
    mut q_movers: Query<(&Transform, &mut WantedMovement)>,
) {
    for (t, mut movement) in q_movers.iter_mut() {
        if t.translation.y == borders.borders.y || t.translation.y == -borders.borders.y {
            movement.direction.y *= -1f32;
        }
        if t.translation.x == borders.borders.x || t.translation.x == -borders.borders.x {
            movement.direction.x *= -1f32;
        }
    }
}

fn collision_warmup(
    mut collision_event: EventReader<StayCollisionEvent>,
    mut query: Query<(&mut Sprite, &mut CollisionState)>,
) {
    for ev in collision_event.iter() {
        if let Ok((mut sprite, mut state)) = query.get_mut(ev.entity_player) {
            sprite.color = Color::RED;
            *state = CollisionState::Colliding;
        }
    }
}

fn clear_collision_warmup(mut query: Query<(&mut Sprite, &mut CollisionState), With<PlayerUnit>>) {
    for (mut sprite, mut state) in query.iter_mut() {
        sprite.color = Color::WHITE;
        *state = CollisionState::NoCollision;
    }
}

fn loading_play_use_currency(
    mut commands: Commands,
    auth_data: Res<AuthData>,
    items: Res<BackpackItems>,
    mut game_def: ResMut<GameDef>,
    backpack: Res<BackpackCom>,
    mut game_state: ResMut<NextState<GameState>>,
    mut loading_state: ResMut<LoadingPlayState>,
) {
    if *loading_state != LoadingPlayState::Init {
        return;
    }
    let Some(auth) = &auth_data.data else {
        *loading_state =  LoadingPlayState::Failed;
        //dbg!(game_state.set(GameState::Warmup));
        dbg!(game_state.set(GameState::Playing));
        return;
    };
    bevy_modify_item(
        &mut commands,
        &backpack.client,
        &auth.0,
        &ItemId(1),
        -(game_def.enemy_count as i32),
        &auth.1.user_id,
    );
    *loading_state = LoadingPlayState::WaitingResponse;
}

fn init_loading_play(mut loading_state: ResMut<LoadingPlayState>) {
    *loading_state = dbg!(LoadingPlayState::Init);
}

fn handle_modify_result(
    mut events: EventReader<ModifyItemTaskResultEvent>,
    mut items: ResMut<BackpackItems>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for ev in events.iter() {
        let Ok(ev) = ev.0 else {
            dbg!(game_state.set(GameState::Warmup));
            return;
        };
        if let Some(elem) = items
            .items
            .iter_mut()
            .enumerate()
            .find(|element| element.1.item.id == ev.0)
        {
            elem.1.amount = ev.2
        }
        game_state.set(GameState::Playing);
    }
}

fn update_collisions_player_playing(
    mut collision_event: EventReader<StayCollisionEvent>,
    mut game_state: ResMut<NextState<GameState>>,
    mut leaderboard: ResMut<bevy_jornet::Leaderboard>,
    score: Res<Score>,
) {
    for ev in collision_event.iter() {
        leaderboard.send_score(score.score as f32);
        game_state.set(GameState::EndScreen);
    }
}

fn juice_collisions(
    time: Res<Time>,
    mut particles: EventWriter<ParticleExplosion>,
    q_scores: Query<(&ScoreNear, &Transform, &ScoreNearDef)>,
) {
    for (score, transform, def) in q_scores.iter() {
        if let ScoreNear::Scoring(scoring) = score {
            let elapsed = time.elapsed_seconds() - scoring.start_time;
            let completion_ratio = elapsed / def.time_to_score;
            // FIXME: #10 change spawn rate + avoid being framerate dependent
            let particle = ParticleExplosion {
                location: transform.translation.xy(),
                color: Color::from(
                    Vec4::from(Color::RED).lerp(Vec4::from(Color::GREEN), completion_ratio),
                ),
                count: 1,
                size: 25f32,
            };
            particles.send(particle);
        }
    }
}
fn juice_score(
    mut particles: EventWriter<ParticleExplosion>,
    q_scores: Query<(&ScoreNear, &Transform), Changed<ScoreNear>>,
) {
    for (score, transform) in q_scores.iter() {
        match score {
            ScoreNear::Scoring(_) => {}
            ScoreNear::NotNear => {
                let particle = ParticleExplosion {
                    location: transform.translation.xy(),
                    color: Color::RED,
                    count: 15,
                    size: 20f32,
                };
                particles.send(particle);
            }
            ScoreNear::Gained => {
                let particle = ParticleExplosion {
                    location: transform.translation.xy(),
                    color: Color::GREEN,
                    count: 15,
                    size: 30f32,
                };
                particles.send(particle);
            }
        }
    }
}

pub(super) fn more_enemies(
    time: Res<Time>,
    mut game_def: ResMut<GameDef>,
    // FIXME: this should not be local, because we want to reset it on game start.
    mut timer: Local<Timer>,
) {
    if timer.duration() == Duration::default() {
        timer.set_duration(Duration::from_secs(
            1f32.lerp_bounded(10f32, (game_def.enemy_count as f32) / 6f32) as u64,
        ));
        return;
    }
    timer.tick(time.delta());
    if timer.just_finished() {
        game_def.enemy_count += 1;
        let new_duration = timer.duration() + Duration::from_secs(1);
        timer.set_duration(new_duration);
        timer.reset();
    }
}
