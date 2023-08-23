mod game;
mod password;

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
    LoginEmailPasswordData, UserId,
};

mod backpack_client_bevy;
pub mod utils;

use backpack_client::*;
use backpack_client_bevy::*;
use dotenvy_macro::dotenv;
use game::Game;

fn main() {
    drop(dotenvy::dotenv());
    let email = dotenvy::var("BACKPACK_GAME_EXAMPLE_USERNAME").unwrap();
    let password = dotenvy::var("BACKPACK_GAME_EXAMPLE_PASSWORD").unwrap();
    let host = dotenvy::var("BACKPACK_SERVER_BASE_URL").unwrap();

    let mut app = App::new();
    let pkv_store = PkvStore::new("Backpack", "Example_lazy");
    if let Ok(authentication_token) = pkv_store.get::<AuthenticationToken>("authentication_token") {
        let mut backpack_auth_refresher = BackpackClientAuthRefresh::default();
        backpack_auth_refresher.set(Some(authentication_token));
        app.insert_resource(backpack_auth_refresher);
    }
    app.insert_resource(pkv_store);
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
    })
    .add_systems(Startup, fix_wasm_input)
    .run();
}

struct AuthPlugin {
    pub host: String,
}

/// workaround for wasm input
fn fix_wasm_input(mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    window.prevent_default_event_handling = false;
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

#[derive(Clone, Eq, PartialEq, Debug, Hash, States)]
enum PopupSignupSuccess {
    Shown,
    Hidden,
}

impl Default for PopupSignupSuccess {
    fn default() -> Self {
        PopupSignupSuccess::Hidden
    }
}

#[derive(Resource, Default)]
struct BackpackItems {
    pub items: Vec<ItemAmount>,
}

impl Plugin for AuthPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BackpackClientPlugin);
        app.add_plugins(Game);
        app.add_systems(Update, ui_auth.run_if(in_state(game::GameState::Warmup)));
        app.add_systems(
            Update,
            handle_authentication_change.run_if(resource_changed::<BackpackClientAuthRefresh>()),
        );
        app.add_systems(Update, handle_signup_result);
        app.add_state::<PopupSignupSuccess>();
        app.add_systems(
            Update,
            ui_signup_successful.run_if(in_state(PopupSignupSuccess::Shown)),
        );
        app.init_resource::<AuthInput>();
        app.insert_resource(BackpackCom::new(dbg!(self.host.clone()) + "/api/v1"));
        app.init_resource::<BackpackItems>();
    }
}

fn ui_signup_successful(
    mut ctxs: EguiContexts,
    mut popup_signup_state: ResMut<NextState<PopupSignupSuccess>>,
) {
    egui::Window::new("Popup Signup Success").show(ctxs.ctx_mut(), |ui| {
        ui.label("Successful signed up!");
        ui.label("We sent you an email, check your spam folder too.");
        if ui.button("I received the mail").clicked() {
            popup_signup_state.set(PopupSignupSuccess::Hidden);
        }
    });
}

fn ui_auth(
    mut commands: Commands,
    mut ctxs: EguiContexts,
    mut auth_input: ResMut<AuthInput>,
    mut authentication: ResMut<BackpackClientAuthRefresh>,
    backpack: Res<BackpackCom>,
    login_task: Query<Entity, With<LoginTask>>,
    signup_task: Query<Entity, With<SignupTask>>,
) {
    egui::Window::new("Auth").show(ctxs.ctx_mut(), |ui| {
        //ui.label(format!("current role: {:?}", auth_data));
        if authentication.current_authentication_token.is_some() {
            if ui.button("Disconnect").clicked() {
                authentication.set(None);
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
                ui.add(password::password(&mut auth_input.password));
            });
            if login_task.is_empty() {
                if ui.button("Sign in").clicked() {
                    bevy_login(
                        &mut commands,
                        &backpack.client,
                        &*authentication,
                        LoginEmailPasswordData {
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

/// Called only when BackpackClientAuthRefresh has changed.
fn handle_authentication_change(
    mut pkv: ResMut<PkvStore>,
    authentication: Res<BackpackClientAuthRefresh>,
) {
    dbg!("authentication changed");
    pkv.set(
        "authentication_token",
        &authentication.current_authentication_token,
    )
    .expect("failed to store authentication token.");
}

fn handle_signup_result(
    mut events: EventReader<SignupTaskResultEvent>,
    mut auth_input: ResMut<AuthInput>,
    mut popup_signup_state: ResMut<NextState<PopupSignupSuccess>>,
) {
    for res in events.iter() {
        if res.0.is_ok() {
            auth_input.sign_in = true;
            popup_signup_state.set(PopupSignupSuccess::Shown);
        } else {
            dbg!("Signup failed.");
        }
    }
}
