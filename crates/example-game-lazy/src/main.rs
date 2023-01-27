mod data;

use std::future;

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use data::{BiscuitInfo, CreateEmailPasswordData, LoginEmailPasswordData};
use reqwest::{Client, Response};

fn main() {
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

#[derive(Resource, Debug, Default)]
struct AuthInput {
    pub email: String,
    pub password: String,
    pub sign_in: bool,
}

#[derive(Resource, Debug)]
struct BackpackClient {
    url: String,
    client: Client,
}

impl BackpackClient {
    pub fn signup(&self) -> String {
        self.url.clone() + "/unauthenticated/login"
    }
    pub async fn login(&self, data: &LoginEmailPasswordData) -> Result<String, reqwest::Error> {
        self.client
            .post(self.url.clone() + "/unauthenticated/login")
            .send()
            .await?
            .text()
            .await
    }
}

#[derive(Component)]
struct LoginTask(Task<Result<String, reqwest::Error>>);

fn bevy_login(mut commands: &Commands, client: &BackpackClient, data: &LoginEmailPasswordData) {
    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move { client.login(data).await });
    commands.spawn(LoginTask(task));
}
fn handle_tasks(mut commands: Commands, mut transform_tasks: Query<(Entity, &mut LoginTask)>) {
    for (entity, mut task) in &mut transform_tasks {
        if let Some(transform) = future::block_on(future::poll_once(&mut task.0)) {
            // Add our new PbrBundle of components to our tagged entity
            commands.entity(entity).insert(PbrBundle {
                mesh: box_mesh_handle.clone(),
                material: box_material_handle.clone(),
                transform,
                ..default()
            });

            // Task is complete, so remove task component from entity
            commands.entity(entity).remove::<ComputeTransform>();
        }
    }
}
impl Plugin for AuthPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AuthData { data: None });
        app.add_system(ui_auth);
        app.init_resource::<AuthInput>();
        app.insert_resource(BackpackClient {
            url: "http://localhost:8080/api/v1".into(),
            client: Client::new(),
        });
    }
}

fn ui_auth(
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
                if let Ok(biscuit_raw) = backpack.login(&LoginEmailPasswordData {
                    email: auth_input.email.clone(),
                    password_plain: auth_input.password.clone(),
                }) {
                    auth_data.data = Some((vec![], BiscuitInfo::from(biscuit)))
                }
            }
        } else {
            if ui.button("Sign up").clicked() {}
        }
    });
}
