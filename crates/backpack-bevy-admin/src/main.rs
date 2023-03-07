mod data;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use data::BiscuitInfo;

// use system_uri::{install, open, SystemUriError};

fn install_and_open() -> Result<(), ()> {
    Ok(())
}

fn main() {
    dotenvy::dotenv().ok();
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(AuthPlugin)
        .run();
}

struct AuthPlugin;

#[derive(Resource, Debug)]
struct AuthData {
    data: Option<(Vec<u8>, BiscuitInfo)>,
}

impl Plugin for AuthPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AuthData { data: None });
        app.add_system(ui_auth);
    }
}

fn ui_auth(mut egui_context: ResMut<EguiContext>, auth_data: Res<AuthData>) {
    egui::Window::new("Auth").show(egui_context.ctx_mut(), |ui| {
        ui.label(format!("current role: {auth_data:?}"));
        if ui.button("Authenticate with Github").clicked() {
            drop(open::that(format!(
                "https://github.com/login/oauth/authorize?client_id={}",
                dotenvy::var("BACKPACK_GITHUB_CLIENT_ID").unwrap()
            )));
        }
    });
}
