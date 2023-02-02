use bevy::prelude::*;
use rand::prelude::*;

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

impl Plugin for Game {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameAssets {
            player: Handle::default(),
            enemy: Handle::default(),
        });
        app.init_resource::<GameDef>();
        app.add_startup_system(load_assets);
        app.add_state(GameState::Warmup);
        app.add_system_set(SystemSet::on_update(GameState::Warmup).with_system(update_enemy_count));
    }
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameAssets {
        player: asset_server.load("bevy.png"),
        enemy: asset_server.load("bevy_pixel_dark.png"),
    });
    let camera = Camera2dBundle {
        projection: OrthographicProjection {
            scale: 10.0,
            ..default()
        },
        ..Default::default()
    };
    commands.spawn(camera);
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
                        rng.gen_range(-1000f32, 1000f32),
                        rng.gen_range(-1000f32, 1000f32),
                        0f32,
                    )),
                    ..default()
                },
                Enemy,
            ));
        }
    } else {
        for e in q_enemies.iter().take(-diff as usize) {
            commands.entity(e).despawn();
        }
    }
}
