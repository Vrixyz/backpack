mod data;
mod game;
mod password;

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Color32, RichText},
    EguiContext, EguiPlugin,
};
use data::{
    AppId, BiscuitInfo, CreateEmailPasswordData, ItemAmount, ItemId, LoginEmailPasswordData,
    UserId, UserItemModify,
};
use dotenv::dotenv;

mod backpack_client;
mod backpack_client_bevy;
pub mod utils;

use backpack_client::*;
use backpack_client_bevy::*;
use game::{Game, GameDef};
use password::password_ui;

fn main() {
    dotenv().ok();
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(AuthPlugin {
            email: std::env::var("BACKPACK_GAME_EXAMPLE_USERNAME").unwrap_or("".to_string()),
            password: std::env::var("BACKPACK_GAME_EXAMPLE_PASSWORD").unwrap_or("".to_string()),
        })
        .run();
}

struct AuthPlugin {
    pub email: String,
    pub password: String,
}

#[derive(Resource, Debug)]
struct AuthData {
    data: Option<(Vec<u8>, BiscuitInfo)>,
}

#[derive(Resource, Debug, Default)]
struct AuthInput {
    pub email: String,
    pub password: String,
    pub sign_in: bool,
}

#[derive(Resource)]
struct BackpackCom {
    pub client: BackpackClient,
}

impl BackpackCom {
    fn new(url: String) -> Self {
        Self {
            client: BackpackClient::new(url),
        }
    }
}

#[derive(Resource, Default)]
struct BackpackItems {
    pub items: Vec<ItemAmount>,
}

impl Plugin for AuthPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AuthData { data: None });
        app.add_plugin(BackpackClientPlugin);
        app.add_plugin(Game);
        app.add_system(ui_auth);
        app.add_system(handle_login_result);
        app.add_system(handle_signup_result);
        app.insert_resource(AuthInput {
            email: self.email.clone(),
            password: self.password.clone(),
            sign_in: true,
        });
        app.insert_resource(BackpackCom::new("http://127.0.0.1:8080/api/v1".into()));
        app.init_resource::<BackpackItems>();
    }
}

fn ui_auth(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    mut auth_input: ResMut<AuthInput>,
    mut auth_data: ResMut<AuthData>,
    backpack: Res<BackpackCom>,
    login_task: Query<Entity, With<LoginTask>>,
    signup_task: Query<Entity, With<SignupTask>>,
) {
    egui::Window::new("Auth").show(egui_context.ctx_mut(), |ui| {
        //ui.label(format!("current role: {:?}", auth_data));
        if auth_data.data.is_some() {
            if ui.button("Disconnect").clicked() {
                auth_data.data = None;
            }
            return;
        }
        ui.horizontal(|ui| {
            ui.label("Your email: ");
            ui.text_edit_singleline(&mut auth_input.email);
        });
        ui.horizontal(|ui| {
            if login_task.is_empty() && signup_task.is_empty() {
                ui.checkbox(&mut auth_input.sign_in, "Already signed up?");
            } else {
                let mut not_interactable = auth_input.sign_in;
                ui.checkbox(
                    &mut not_interactable,
                    RichText::new("Already signed up?").color(Color32::GRAY),
                );
            }
        });
        if auth_input.sign_in {
            ui.horizontal(|ui| {
                ui.label("Password: ");
                password_ui(ui, &mut auth_input.password);
            });
            if login_task.is_empty() {
                if ui.button("Sign in").clicked() {
                    bevy_login(
                        &mut commands,
                        &backpack.client,
                        &LoginEmailPasswordData {
                            email: auth_input.email.clone(),
                            password_plain: auth_input.password.clone(),
                            as_app_user: Some(AppId(1)),
                        },
                    );
                }
            } else {
                ui.label("Logging in...");
            }
        } else if ui.button("Sign up").clicked() {
            bevy_signup(
                &mut commands,
                &backpack.client,
                &CreateEmailPasswordData {
                    email: auth_input.email.clone(),
                },
            );
        }
    });
}

fn handle_login_result(
    mut events: EventReader<LoginTaskResultEvent>,
    mut auth_data: ResMut<AuthData>,
) {
    for res in events.iter() {
        if let Ok(biscuit_raw) = &res.0 {
            auth_data.data = Some(biscuit_raw.clone())
        } else {
            dbg!("Login failed.");
        }
    }
}

fn handle_signup_result(
    mut events: EventReader<SignupTaskResultEvent>,
    mut auth_input: ResMut<AuthInput>,
) {
    for res in events.iter() {
        if res.0.is_ok() {
            auth_input.sign_in = true;
        } else {
            dbg!("Login failed.");
        }
    }
}
