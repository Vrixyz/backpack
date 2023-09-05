use backpack_client_bevy_egui::*;
use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};
use bevy_egui::*;

fn main() {
    App::new().add_plugins(GamePlugin).run();
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        drop(dotenvy::dotenv());
        let email = dotenvy::var("BACKPACK_GAME_EXAMPLE_USERNAME").unwrap();
        let password = dotenvy::var("BACKPACK_GAME_EXAMPLE_PASSWORD").unwrap();
        let host = dotenvy::var("BACKPACK_SERVER_BASE_URL").unwrap();
        app.add_plugins(DefaultPlugins.set(LogPlugin {
            level: Level::DEBUG,
            filter: "wgpu=info,bevy_render=info,bevy_ecs=info,wgpu_core=warn".to_string(),
        }))
        .add_plugins(EguiPlugin)
        .insert_resource(AuthInput {
            email: email.to_string(),
            password: password.to_string(),
            sign_in: password.is_empty(),
        })
        .add_plugins(AuthPlugin {
            host: dbg!(host.to_string()),
        });
    }
}
