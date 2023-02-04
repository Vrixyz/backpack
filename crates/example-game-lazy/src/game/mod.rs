mod collisions;
mod mouse;
mod ui_warmup;

use bevy::prelude::*;
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use rand::prelude::*;

use crate::{
    backpack_client_bevy::bevy_modify_item, data::ItemId, AuthData, BackpackCom, BackpackItems,
};

use self::{
    collisions::StayCollisionEvent,
    mouse::{GameCamera, MousePos},
};

pub struct Game;

#[derive(Resource)]
struct GameAssets {
    pub player: Handle<Image>,
    pub enemy: Handle<Image>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Warmup,
    Playing,
    EndScreen,
}

#[derive(Resource, Default)]
pub struct GameDef {
    pub enemy_count: u32,
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

impl Plugin for Game {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameAssets {
            player: Handle::default(),
            enemy: Handle::default(),
        });
        app.add_plugin(mouse::MousePlugin);
        app.add_plugin(DebugLinesPlugin::default());
        app.add_plugin(collisions::CollisionsPlugin);
        app.init_resource::<GameDef>();
        app.add_startup_system(load_assets);
        app.add_state(GameState::Warmup);
        app.add_system_set(
            SystemSet::on_update(GameState::Warmup)
                .with_system(update_enemy_count)
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
            SystemSet::on_enter(GameState::Playing).with_system(enter_playing_use_currency),
        );
        app.add_system_set(
            SystemSet::on_update(GameState::Playing).with_system(
                update_wanted_movement_player
                    .before(update_movement)
                    .after(mouse::my_cursor_system),
            ),
        );
        app.add_system_set(SystemSet::on_enter(GameState::Warmup).with_system(create_player));
        app.add_system(ui_warmup::handle_get_items_result);
        app.add_system(update_movement.before(collisions::collision_player_enemies))
            .add_system(bounce_enemies.after(update_movement));
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
fn create_player(mut commands: Commands, assets: Res<GameAssets>, mut lines: ResMut<DebugLines>) {
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
        Vec2::new(1000f32, 1000f32).extend(0f32),
        Vec2::new(1000f32, -1000f32).extend(0f32),
        Vec2::new(-1000f32, -1000f32).extend(0f32),
        Vec2::new(-1000f32, 1000f32).extend(0f32),
    ];

    for i in 0..borders.len() {
        lines.line(borders[i], borders[(i + 1) % borders.len()], f32::INFINITY);
    }
}

fn update_enemy_count(
    mut commands: Commands,
    assets: Res<GameAssets>,
    def: Res<GameDef>,
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
                        rng.gen_range(-1000f32..1000f32),
                        rng.gen_range(-1000f32..1000f32),
                        0f32,
                    )),
                    ..default()
                },
                Enemy,
                WantedMovement {
                    direction: random_point_circle(1000f32, false),
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

fn update_movement(time: Res<Time>, mut q_movers: Query<(&mut Transform, &WantedMovement)>) {
    for (mut t, movement) in q_movers.iter_mut() {
        t.translation += (movement.direction * time.delta_seconds()).extend(0f32);
        t.translation.y = t.translation.y.clamp(-1000f32, 1000f32);
        t.translation.x = t.translation.x.clamp(-1000f32, 1000f32);
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

fn bounce_enemies(mut q_movers: Query<(&Transform, &mut WantedMovement)>) {
    for (t, mut movement) in q_movers.iter_mut() {
        if t.translation.y == 1000f32 || t.translation.y == -1000f32 {
            movement.direction.y *= -1f32;
        }
        if t.translation.x == 1000f32 || t.translation.x == -1000f32 {
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

fn enter_playing_use_currency(
    mut commands: Commands,
    auth_data: Res<AuthData>,
    items: Res<BackpackItems>,
    mut game_def: ResMut<GameDef>,
    backpack: Res<BackpackCom>,
) {
    let Some(auth) = &auth_data.data else {
        todo!("Not authentified: Set gamestate to warmup back.");
    };
    bevy_modify_item(
        &mut commands,
        &backpack.client,
        &auth.0,
        &ItemId(1),
        1,
        &auth.1.user_id,
    );
}
