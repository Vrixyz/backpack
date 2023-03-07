use async_compat::{Compat, CompatExt};
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;

use crate::{
    backpack_client::BackpackClient,
    data::{
        BiscuitInfo, CreateEmailPasswordData, ItemAmount, ItemId, LoginEmailPasswordData, UserId,
    },
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

#[derive(Component)]
pub struct LoginTask(Task<Result<(Vec<u8>, BiscuitInfo), reqwest::Error>>);
pub struct LoginTaskResultEvent(pub Result<(Vec<u8>, BiscuitInfo), reqwest::Error>);

pub fn bevy_login(commands: &mut Commands, client: &BackpackClient, data: &LoginEmailPasswordData) {
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
    mut result_event: EventWriter<LoginTaskResultEvent>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(res) = future::block_on(Compat::new(future::poll_once(&mut task.0))) {
            result_event.send(LoginTaskResultEvent(res));
            // Task is complete, so remove task component from entity
            commands.entity(entity).remove::<LoginTask>();
        }
    }
}
#[derive(Component)]
pub struct SignupTask(Task<Result<(), reqwest::Error>>);
pub struct SignupTaskResultEvent(pub Result<(), reqwest::Error>);

pub fn bevy_signup(
    commands: &mut Commands,
    client: &BackpackClient,
    data: &CreateEmailPasswordData,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    // FIXME: Cloning the client is problematic if we ever want to use cookies. But we're cloning here to be able to send into the task.
    let client = client.clone();
    let data = data.clone();
    let task = thread_pool.spawn(async move { client.signup(&data.clone()).compat().await });
    commands.spawn(SignupTask(task));
}
fn handle_signup_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut SignupTask)>,
    mut result_event: EventWriter<SignupTaskResultEvent>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(res) = future::block_on(Compat::new(future::poll_once(&mut task.0))) {
            result_event.send(SignupTaskResultEvent(res));
            // Task is complete, so remove task component from entity
            commands.entity(entity).remove::<SignupTask>();
        }
    }
}

#[derive(Component)]
pub struct GetItemsTask(Task<Result<Vec<ItemAmount>, reqwest::Error>>);
pub struct GetItemsTaskResultEvent(pub Result<Vec<ItemAmount>, reqwest::Error>);

pub fn bevy_get_items(
    commands: &mut Commands,
    client: &BackpackClient,
    biscuit_raw: &[u8],
    user_id: &UserId,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    // FIXME: Cloning the client is problematic if we ever want to use cookies. But we're cloning here to be able to send into the task.
    let client = client.clone();
    let data = (biscuit_raw.to_vec(), *user_id);
    let task = thread_pool.spawn(async move { client.get_items(&data.0, &data.1).compat().await });
    commands.spawn(GetItemsTask(task));
}
fn handle_get_items_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut GetItemsTask)>,
    mut result_event: EventWriter<GetItemsTaskResultEvent>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(res) = future::block_on(Compat::new(future::poll_once(&mut task.0))) {
            result_event.send(GetItemsTaskResultEvent(res));
            // Task is complete, so remove task component from entity
            commands.entity(entity).remove::<GetItemsTask>();
        }
    }
}

#[derive(Component)]
pub struct ModifyItemTask(Task<Result<(ItemId, UserId, i32), reqwest::Error>>);
pub struct ModifyItemTaskResultEvent(pub Result<(ItemId, UserId, i32), reqwest::Error>);

pub fn bevy_modify_item(
    commands: &mut Commands,
    client: &BackpackClient,
    biscuit_raw: &[u8],
    item_id: &ItemId,
    amount: i32,
    user_id: &UserId,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    // FIXME: Cloning the client is problematic if we ever want to use cookies. But we're cloning here to be able to send into the task.
    let client = client.clone();
    let data = (biscuit_raw.to_vec(), *item_id, *user_id);
    let task = thread_pool.spawn(async move {
        client
            .modify_item(&data.0, data.1, amount, data.2)
            .compat()
            .await
            .map(|amount| (data.1, data.2, amount))
    });
    commands.spawn(ModifyItemTask(task));
}
fn handle_modify_item_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ModifyItemTask)>,
    mut result_event: EventWriter<ModifyItemTaskResultEvent>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(res) = future::block_on(Compat::new(future::poll_once(&mut task.0))) {
            result_event.send(ModifyItemTaskResultEvent(res));
            // Task is complete, so remove task component from entity
            commands.entity(entity).remove::<ModifyItemTask>();
        }
    }
}
