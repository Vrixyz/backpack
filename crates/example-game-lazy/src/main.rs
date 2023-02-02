mod data;
mod game;
mod password;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use data::{
    BiscuitInfo, CreateEmailPasswordData, ItemAmount, ItemId, LoginEmailPasswordData, UserId,
    UserItemModify,
};
use dotenv::dotenv;

mod backpack_client;
mod backpack_client_bevy;

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
        app.add_system(ui_game);
        app.add_system(handle_login_result);
        app.add_system(handle_get_items_result);
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
) {
    egui::Window::new("Auth").show(egui_context.ctx_mut(), |ui| {
        //ui.label(format!("current role: {:?}", auth_data));
        ui.horizontal(|ui| {
            ui.label("Your email: ");
            ui.text_edit_singleline(&mut auth_input.email);
        });
        ui.horizontal(|ui| {
            ui.checkbox(&mut auth_input.sign_in, "Already signed up?");
        });
        if auth_input.sign_in {
            ui.horizontal(|ui| {
                ui.label("Password: ");
                password_ui(ui, &mut auth_input.password);
            });
            if ui.button("Sign in").clicked() {
                bevy_login(
                    &mut commands,
                    &backpack.client,
                    &LoginEmailPasswordData {
                        email: auth_input.email.clone(),
                        password_plain: auth_input.password.clone(),
                    },
                );
            }
        } else if ui.button("Sign up").clicked() {
            dbg!("Signup not implemented");
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

fn ui_game(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    auth_data: Res<AuthData>,
    items: Res<BackpackItems>,
    mut game_def: ResMut<GameDef>,
    backpack: Res<BackpackCom>,
) {
    if auth_data.data.is_none() {
        return;
    }
    egui::Window::new("Game")
        .auto_sized()
        .show(egui_context.ctx_mut(), |ui| {
            if let Some(auth) = &auth_data.data {
                if ui.button("Get items").clicked() {
                    bevy_get_items(&mut commands, &backpack.client, &auth.0, &auth.1.user_id);
                }
                if items.items.len() > 0 {
                    ui.group(|ui| {
                        for item in (*items.items).iter() {
                            if item.item.id.0 == 1 {
                                ui.horizontal(|ui| {
                                    ui.label(format!(
                                        "{}({}): {}",
                                        item.item.name,
                                        item.item.id.0,
                                        item.amount - game_def.enemy_count as i32
                                    ));
                                    ui.vertical(|ui| {
                                        if item.amount > game_def.enemy_count as i32 {
                                            if ui.button("+1 enemy").clicked() {
                                                game_def.enemy_count += 1;
                                                // TODO: pay 1 item
                                            }
                                        } else {
                                            // Not enough item amount
                                            if ui.button("Not enough enemy in stock").clicked() {
                                                dbg!("not enough item amount");
                                            }
                                        }
                                        if game_def.enemy_count > 0 {
                                            // Can remove enemies
                                            if ui.button("-1 enemy").clicked() {
                                                game_def.enemy_count -= 1;
                                                // TODO: pay 1 item
                                            }
                                        }
                                    });
                                });
                            }
                        }
                    });
                }
            }
        });
}

fn handle_get_items_result(
    mut events: EventReader<GetItemsTaskResultEvent>,
    mut resource_items: ResMut<BackpackItems>,
) {
    for res in events.iter() {
        if let Ok(items) = &res.0 {
            dbg!(items);
            resource_items.items = (*items).clone();
        } else {
            dbg!("get items failed.");
        }
    }
}
