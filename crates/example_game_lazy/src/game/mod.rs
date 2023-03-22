mod collisions;
mod scoreboard;
mod scoring;
mod ui_endscreen;
mod ui_playing;
mod ui_warmup;

use std::time::Duration;

use bevy::{
    math::{Affine3A, Vec3Swizzles},
    prelude::*,
};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use lerp::Lerp;
use particles::ParticleExplosion;
use rand::prelude::*;

use crate::{
    backpack_client_bevy::{bevy_modify_item, ModifyItemTaskResultEvent},
    utils::{
        self,
        mouse::{self, GameCamera, MousePos},
    },
    AuthData, BackpackCom, BackpackItems,
};
use shared::ItemId;

use self::{
    collisions::StayCollisionEvent,
    scoring::{Score, ScoreNear, ScoreNearDef},
};

pub struct Game;

#[derive(Resource)]
struct GameAssets {
    pub player: Handle<Image>,
    pub enemy: Handle<Image>,
    pub warning: Handle<Image>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, States)]
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
pub struct EnemySpawnTimer {
    pub timer: Timer,
}

impl EnemySpawnTimer {
    pub fn reset_for_enemy_amount(&mut self, amount: u32) {
        self.timer = Timer::new(
            Duration::from_secs(1f32.lerp_bounded(10f32, (amount as f32) / 6f32) as u64),
            TimerMode::Repeating,
        );
    }
}

#[derive(Component, Default)]
pub struct PlannedSpawn {
    pub position: Vec2,
    pub time_to_spawn: f32,
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
    StartedWithBenefit,
    StartedWithoutBenefit,
}

impl Plugin for Game {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameAssets {
            player: Handle::default(),
            enemy: Handle::default(),
            warning: Handle::default(),
        });
        app.insert_resource(GameDefBorder {
            borders: Vec2::new(2000f32, 2000f32),
        });
        app.insert_resource(LoadingPlayState::Unknown);
        app.insert_resource(EnemySpawnTimer { ..default() });
        app.add_state::<GameState>();
        app.add_plugin(mouse::MousePlugin);
        app.add_plugin(DebugLinesPlugin::default());
        app.add_plugin(collisions::CollisionsPlugin);
        app.add_plugin(scoreboard::ScoreboardPlugin);
        app.add_plugin(scoring::ScorePlugin);
        app.add_plugin(particles::ParticlesPlugin);
        app.init_resource::<GameDef>();
        app.add_startup_system(load_assets);
        app.add_systems(
            (
                update_wanted_movement_player
                    .before(update_movement)
                    .after(mouse::my_cursor_system),
                collision_warmup.after(collisions::collision_player_enemies),
                clear_collision_warmup.before(collision_warmup),
                update_enemy_count,
                ui_warmup::ui_warmup,
                ui_warmup::ui_tuto_start,
                ui_warmup::handle_tap_to_start.after(collision_warmup),
            )
                .in_set(OnUpdate(GameState::Warmup)),
        );
        app.add_system(
            init_loading_play
                .in_schedule(OnEnter(GameState::LoadingPlay))
                .before(loading_play_use_currency),
        );
        app.add_system(
            loading_play_use_currency
                .in_schedule(OnEnter(GameState::LoadingPlay))
                .before(handle_modify_result),
        );
        app.add_system(handle_modify_result.in_set(OnUpdate(GameState::LoadingPlay)));
        app.add_systems(
            (
                update_wanted_movement_player
                    .before(update_movement)
                    .after(mouse::my_cursor_system),
                spawn_planned,
            )
                .in_set(OnUpdate(GameState::Playing)),
        );
        app.add_system(init_timer.in_schedule(OnEnter(GameState::Playing)));
        app.add_systems(
            (
                update_collisions_player_playing.after(collisions::collision_player_enemies),
                juice_collisions,
                juice_score,
            )
                .in_set(OnUpdate(GameState::Playing)),
        );
        app.add_systems(
            (ui_playing::ui_playing, more_enemies).in_set(OnUpdate(GameState::Playing)),
        );
        app.add_systems(
            (utils::despawn::<Enemy>, utils::despawn::<PlannedSpawn>)
                .in_schedule(OnEnter(GameState::Warmup)),
        );
        app.add_systems(
            (utils::despawn::<PlayerUnit>, create_player)
                .chain()
                .in_schedule(OnEnter(GameState::Warmup)),
        );
        app.add_system(ui_warmup::handle_get_items_result);
        app.add_system(ui_warmup::handle_modify_item_result);
        app.add_system(update_movement.before(collisions::collision_player_enemies))
            .add_system(bounce_enemies.after(update_movement));

        app.add_systems(
            (
                ui_endscreen::ui_endscreen,
                ui_endscreen::ui_end_title_and_score,
            )
                .in_set(OnUpdate(GameState::EndScreen)),
        );
    }
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameAssets {
        player: asset_server.load("bevy.png"),
        enemy: asset_server.load("bevy_pixel_light.png"),
        warning: asset_server.load("bevy_pixel_dark.png"),
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
    mut lines: ResMut<DebugLines>,
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
        lines.line(borders[i], borders[(i + 1) % borders.len()], f32::INFINITY);
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
            let position = Vec2::new(
                rng.gen_range(-borders.borders.x..borders.borders.x),
                rng.gen_range(-borders.borders.y..borders.borders.y),
            );
            spawn_enemy(&mut commands, &assets, position);
        }
    } else {
        for e in q_enemies.iter().take(-diff as usize) {
            commands.entity(e).despawn();
        }
    }
}

fn spawn_enemy(commands: &mut Commands, assets: &Res<GameAssets>, position: Vec2) {
    commands.spawn((
        SpriteBundle {
            texture: assets.enemy.clone(),
            transform: Transform::from_translation(position.extend(0f32)),
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
        if movement.direction.length_squared() > 0f32 {
            t.rotation = Quat::from_affine3(&Affine3A::look_to_rh(
                Vec3::ZERO,
                Vec3::Z,
                movement.direction.extend(0f32),
            )) * Quat::from_rotation_z(0.25f32 * std::f32::consts::TAU);
        }
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
    borders: Res<GameDefBorder>,
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
    game_def: ResMut<GameDef>,
    backpack: Res<BackpackCom>,
    mut game_state: ResMut<NextState<GameState>>,
    mut loading_state: ResMut<LoadingPlayState>,
) {
    if *loading_state != LoadingPlayState::Init {
        return;
    }
    let Some(auth) = &auth_data.data else {
        *loading_state = LoadingPlayState::StartedWithoutBenefit;
        //dbg!(game_state.set(GameState::Warmup));
        dbg!(game_state.set(GameState::Playing));
        return;
    };
    if game_def.enemy_count == 0 {
        *loading_state = LoadingPlayState::StartedWithoutBenefit;
        game_state.set(GameState::Playing);
        return;
    }
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

fn init_timer(game_def: Res<GameDef>, mut timer: ResMut<EnemySpawnTimer>) {
    timer.reset_for_enemy_amount(game_def.enemy_count);
}

fn update_collisions_player_playing(
    mut collision_event: EventReader<StayCollisionEvent>,
    mut game_state: ResMut<NextState<GameState>>,
    leaderboard: ResMut<bevy_jornet::Leaderboard>,
    score: Res<Score>,
) {
    for _ in collision_event.iter() {
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

fn more_enemies(
    mut commands: Commands,
    assets: Res<GameAssets>,
    time: Res<Time>,
    total_enemies: Query<Entity, Or<(&PlannedSpawn, &Enemy)>>,
    mut timer: ResMut<EnemySpawnTimer>,
) {
    timer.timer.tick(time.delta());
    if timer.timer.just_finished() {
        let mut rng = rand::thread_rng();
        let position = Vec2::new(
            rng.gen_range(-1000f32..1000f32),
            rng.gen_range(-1000f32..1000f32),
        );
        commands.spawn((
            PlannedSpawn {
                position,
                time_to_spawn: time.elapsed_seconds() + 1f32,
            },
            SpriteBundle {
                texture: assets.warning.clone(),
                transform: Transform::from_translation(position.extend(0f32)),
                ..default()
            },
        ));
        timer.reset_for_enemy_amount(total_enemies.iter().count() as u32);
    }
}

fn spawn_planned(
    mut commands: Commands,
    assets: Res<GameAssets>,
    time: Res<Time>,
    planned_enemies: Query<(Entity, &PlannedSpawn)>,
) {
    for (e, spawn) in planned_enemies.iter() {
        if spawn.time_to_spawn < time.elapsed_seconds() {
            commands.entity(e).despawn();
            spawn_enemy(&mut commands, &assets, spawn.position)
        }
    }
}
