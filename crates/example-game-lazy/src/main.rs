mod data;

use async_compat::{Compat, CompatExt};
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use data::{BiscuitInfo, CreateEmailPasswordData, LoginEmailPasswordData};
use dotenv::dotenv;
use futures_lite::future;
use reqwest::{Client, Response};

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

#[derive(Resource, Clone, Debug)]
struct BackpackClient {
    url: String,
    client: Client,
}

impl BackpackClient {
    pub fn new(url: String) -> Self {
        Self {
            url,
            client: Client::new(),
        }
    }
    pub fn signup(&self) -> String {
        self.url.clone() + "/unauthenticated/email_password/create"
    }
    pub async fn login(
        &self,
        data: &LoginEmailPasswordData,
    ) -> Result<(Vec<u8>, BiscuitInfo), reqwest::Error> {
        let biscuit_raw = dbg!(
            self.client
                .post(dbg!(
                    self.url.clone() + "/unauthenticated/email_password/login"
                ))
                .json(data)
                .send()
                .await?
                .text()
                .await?
        );
        self.whoami(biscuit_raw.as_bytes()).await
    }

    /// FIXME: this route should be avoidable by decrypting biscuit information with server public key.
    /// Also, sending auth data could be done via secure http-only cookie.
    pub async fn whoami(
        &self,
        biscuit_raw: &[u8],
    ) -> Result<(Vec<u8>, BiscuitInfo), reqwest::Error> {
        let biscuit = dbg!(
            dbg!(self
                .client
                .get(self.url.clone() + "/authenticated/whoami")
                .bearer_auth(std::str::from_utf8(biscuit_raw).expect("wrong auth token")))
            .send()
            .await?
        )
        .json::<BiscuitInfo>()
        .await?;
        Ok((biscuit_raw.into(), dbg!(biscuit)))
    }
}

#[derive(Component)]
struct LoginTask(Task<Result<(Vec<u8>, BiscuitInfo), reqwest::Error>>);

fn bevy_login(commands: &mut Commands, client: &BackpackClient, data: &LoginEmailPasswordData) {
    let thread_pool = AsyncComputeTaskPool::get();
    // FIXME: Cloning the client is problematic if we ever want to use cookies. But we're cloning here to be able to send into the task.
    let client = client.clone();
    let data = data.clone();
    let task = thread_pool.spawn(async move { client.login(&data.clone()).compat().await });
    commands.spawn(LoginTask(task));
}

fn handle_login_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut LoginTask)>,
    mut auth_data: ResMut<AuthData>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(res) = future::block_on(Compat::new(future::poll_once(&mut task.0))) {
            // Add our new PbrBundle of components to our tagged entity
            if let Ok(biscuit_raw) = res {
                auth_data.data = Some(biscuit_raw)
            } else {
                dbg!("failed");
            }

            // Task is complete, so remove task component from entity
            commands.entity(entity).remove::<LoginTask>();
        }
    }
}
impl Plugin for AuthPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AuthData { data: None });
        app.add_system(ui_auth);
        app.insert_resource(AuthInput {
            email: self.email.clone(),
            password: self.password.clone(),
            sign_in: true,
        });
        app.insert_resource(BackpackClient::new("http://127.0.0.1:8080/api/v1".into()));
        app.add_system(handle_login_tasks);
    }
}

fn ui_auth(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    mut auth_input: ResMut<AuthInput>,
    mut auth_data: ResMut<AuthData>,
    backpack: Res<BackpackClient>,
) {
    egui::Window::new("Auth").show(egui_context.ctx_mut(), |ui| {
        ui.label(format!("current role: {:?}", auth_data));
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
                ui.text_edit_singleline(&mut auth_input.password);
            });
            if ui.button("Sign in").clicked() {
                bevy_login(
                    &mut commands,
                    backpack.as_ref(),
                    &LoginEmailPasswordData {
                        email: auth_input.email.clone(),
                        password_plain: auth_input.password.clone(),
                    },
                );
            }
        } else {
            if ui.button("Sign up").clicked() {}
        }
    });
}
