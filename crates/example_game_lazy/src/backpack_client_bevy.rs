use std::sync::{Arc, RwLock};

use async_compat::CompatExt;
use backpack_client::BackpackClient;
use bevy::{prelude::*, tasks::IoTaskPool};
use shared::{
    BiscuitInfo, CreateEmailPasswordData, ItemAmount, ItemId, LoginEmailPasswordData, UserId,
};

pub struct BackpackClientPlugin;

impl Plugin for BackpackClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoginTaskResultEvent>();
        app.add_system(handle_login_tasks);
        app.add_event::<SignupTaskResultEvent>();
        app.add_system(handle_signup_tasks);
        app.add_event::<GetItemsTaskResultEvent>();
        app.add_system(handle_get_items_tasks);
        app.add_event::<ModifyItemTaskResultEvent>();
        app.add_system(handle_modify_item_tasks);
    }
}

pub struct ClientTask<T> {
    pub result: Arc<RwLock<Option<Result<T, reqwest::Error>>>>,
}

impl<T> Default for ClientTask<T> {
    fn default() -> Self {
        Self {
            result: Default::default(),
        }
    }
}

#[derive(Component, Default)]
pub struct LoginTask(ClientTask<(Vec<u8>, BiscuitInfo)>);
pub struct LoginTaskResultEvent(pub Result<(Vec<u8>, BiscuitInfo), reqwest::Error>);

pub fn bevy_login(commands: &mut Commands, client: &BackpackClient, data: &LoginEmailPasswordData) {
    let thread_pool = IoTaskPool::get();
    // FIXME: Cloning the client is problematic if we ever want to use cookies. But we're cloning here to be able to send into the task.
    let client = client.clone();
    let data = data.clone();
    let task = LoginTask::default();
    let fill_result_rwlock = task.0.result.clone();
    thread_pool
        .spawn(async move {
            let res = client.login(&data.clone()).compat().await;
            *fill_result_rwlock.write().unwrap() = Some(res);
        })
        .detach();
    commands.spawn(task);
}
fn handle_login_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &LoginTask)>,
    mut result_event: EventWriter<LoginTaskResultEvent>,
) {
    for (entity, task) in &mut tasks {
        let Ok(mut guard) = task.0.result.try_write() else {
            continue;
        };
        if guard.as_ref().is_none() {
            continue;
        }
        if let Some(received) = guard.take().take() {
            result_event.send(LoginTaskResultEvent(received));
            // Task is complete, so remove task component from entity
            commands.entity(entity).remove::<LoginTask>();
        }
    }
}
#[derive(Component, Default)]
pub struct SignupTask(ClientTask<shared::CreatedUserEmailPasswordData>);
pub struct SignupTaskResultEvent(pub Result<shared::CreatedUserEmailPasswordData, reqwest::Error>);

pub fn bevy_signup(
    commands: &mut Commands,
    client: &BackpackClient,
    data: &CreateEmailPasswordData,
) {
    let thread_pool = IoTaskPool::get();
    let client = client.clone();
    let data = data.clone();
    let task = SignupTask::default();
    let fill_result_rwlock = task.0.result.clone();
    thread_pool
        .spawn(async move {
            let res = client.signup(&data.clone()).compat().await;
            *fill_result_rwlock.write().unwrap() = Some(res);
        })
        .detach();
    commands.spawn(task);
}

fn handle_signup_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut SignupTask)>,
    mut result_event: EventWriter<SignupTaskResultEvent>,
) {
    for (entity, task) in &mut tasks {
        let Ok(mut guard) = task.0.result.try_write() else {
            continue;
        };
        if guard.as_ref().is_none() {
            continue;
        }
        if let Some(received) = guard.take().take() {
            result_event.send(SignupTaskResultEvent(received));
            // Task is complete, so remove task component from entity
            commands.entity(entity).remove::<SignupTask>();
        }
    }
}

#[derive(Component, Default)]
pub struct GetItemsTask(ClientTask<Vec<ItemAmount>>);
pub struct GetItemsTaskResultEvent(pub Result<Vec<ItemAmount>, reqwest::Error>);

pub fn bevy_get_items(
    commands: &mut Commands,
    client: &BackpackClient,
    biscuit_raw: &[u8],
    user_id: &UserId,
) {
    let thread_pool = IoTaskPool::get();
    let client = client.clone();
    let data = (biscuit_raw.to_vec(), *user_id);
    let task = GetItemsTask::default();
    let fill_result_rwlock = task.0.result.clone();
    thread_pool
        .spawn(async move {
            let res = client.get_items(&data.0, &data.1).compat().await;
            *fill_result_rwlock.write().unwrap() = Some(res);
        })
        .detach();
    commands.spawn(task);
}
fn handle_get_items_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut GetItemsTask)>,
    mut result_event: EventWriter<GetItemsTaskResultEvent>,
) {
    for (entity, task) in &mut tasks {
        let Ok(mut guard) = task.0.result.try_write() else {
            continue;
        };
        if guard.as_ref().is_none() {
            continue;
        }
        if let Some(received) = guard.take().take() {
            result_event.send(GetItemsTaskResultEvent(received));
            // Task is complete, so remove task component from entity
            commands.entity(entity).remove::<GetItemsTask>();
        }
    }
}

#[derive(Component, Default)]
pub struct ModifyItemTask(ClientTask<(ItemId, UserId, i32)>);
pub struct ModifyItemTaskResultEvent(pub Result<(ItemId, UserId, i32), reqwest::Error>);

pub fn bevy_modify_item(
    commands: &mut Commands,
    client: &BackpackClient,
    biscuit_raw: &[u8],
    item_id: &ItemId,
    amount: i32,
    user_id: &UserId,
) {
    let thread_pool = IoTaskPool::get();
    let client = client.clone();
    let data = (biscuit_raw.to_vec(), *item_id, *user_id);
    let task = ModifyItemTask::default();
    let fill_result_rwlock = task.0.result.clone();
    thread_pool
        .spawn(async move {
            let res = client
                .modify_item(&data.0, data.1, amount, data.2)
                .compat()
                .await
                .map(|r| (data.1, data.2, r));
            *fill_result_rwlock.write().unwrap() = Some(res);
        })
        .detach();
    commands.spawn(task);
}
fn handle_modify_item_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ModifyItemTask)>,
    mut result_event: EventWriter<ModifyItemTaskResultEvent>,
) {
    for (entity, task) in &mut tasks {
        let Ok(mut guard) = task.0.result.try_write() else {
            continue;
        };
        if guard.as_ref().is_none() {
            continue;
        }
        if let Some(received) = guard.take().take() {
            result_event.send(ModifyItemTaskResultEvent(received));
            // Task is complete, so remove task component from entity
            commands.entity(entity).remove::<ModifyItemTask>();
        }
    }
}
