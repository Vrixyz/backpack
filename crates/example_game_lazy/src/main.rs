mod game;

use backpack_client_bevy_egui::*;
use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};
use bevy_egui::{
    egui::{self, Color32, RichText},
    EguiContexts, EguiPlugin,
};
use bevy_pkv::PkvStore;
use shared::{
    AppId, AuthenticationToken, BiscuitInfo, CreateEmailPasswordData, ItemAmount,
    LoginEmailPasswordData, Role, UserId,
};

pub mod utils;

use backpack_client_bevy::*;
use game::Game;

fn main() {
    drop(dotenvy::dotenv());
    let email = dotenvy::var("BACKPACK_GAME_EXAMPLE_USERNAME").unwrap();
    let password = dotenvy::var("BACKPACK_GAME_EXAMPLE_PASSWORD").unwrap();
    let host = dotenvy::var("BACKPACK_SERVER_BASE_URL").unwrap();

    let mut app = App::new();

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
    .insert_resource(BackpackRole(Role::User(AppId(1))))
    .add_plugins(AuthPlugin {
        host: dbg!(host.to_string()),
    })
    .add_plugins(Game)
    .run();
}
