use async_lock::Mutex;
use std::{
    sync::{Arc, RwLock},
    time::UNIX_EPOCH,
};

use backpack_client::{BackpackClient, RequestError};
use bevy::{gizmos, prelude::*, tasks::IoTaskPool, utils::Instant};
use shared::{
    AuthenticationToken, BiscuitInfo, CreateEmailPasswordData, ItemAmount, ItemId,
    LoginEmailPasswordData, RefreshToken, UserId,
};

pub struct BackpackClientPlugin;

impl Plugin for BackpackClientPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BackpackClientAuthRefresh>();
        app.add_event::<LoginTaskResultEvent>();
        app.add_systems(Update, handle_login_tasks);
        app.add_event::<SignupTaskResultEvent>();
        app.add_systems(Update, handle_signup_tasks);
        app.add_event::<GetItemsTaskResultEvent>();
        app.add_systems(Update, handle_get_items_tasks);
        app.add_event::<ModifyItemTaskResultEvent>();
        app.add_systems(Update, handle_modify_item_tasks);
    }
}

#[derive(Resource, Debug, Default)]
pub struct BackpackClientAuthRefresh {
    authentication_token: Arc<Mutex<Option<AuthenticationToken>>>,
}

impl BackpackClientAuthRefresh {
    pub fn set(&mut self, authentication_token: Option<AuthenticationToken>) {
        self.authentication_token = Arc::new(Mutex::new(authentication_token));
    }
    pub fn is_authenticated(&self) -> bool {
        let auth_token = futures::executor::block_on(self.authentication_token.lock());
        auth_token.is_some()
    }
    pub fn disconnect(&self) {
        let mut auth_token = futures::executor::block_on(self.authentication_token.lock());
        *auth_token = None;
    }
}

pub struct ClientTask<T> {
    pub result: Arc<RwLock<Option<Result<T, RequestError>>>>,
}

impl<T> Default for ClientTask<T> {
    fn default() -> Self {
        Self {
            result: Default::default(),
        }
    }
}

async fn check_refresh_and_report_token(
    unix_now: i64,
    client: &BackpackClient,
    authentication_token: &Arc<Mutex<Option<AuthenticationToken>>>,
) -> Result<Option<AuthenticationToken>, RequestError> {
    let mut guard = authentication_token.lock().await;
    let Some(token) = &*guard else {
        return Err(RequestError::ClientError("The authentication token is not set. call login() first.".into()));
    };
    let authentication_token = match check_and_refresh_token(unix_now, client, token).await {
        Ok(Some(new_token)) => {
            // TODO: #27 send an event to inform the authentication token did change
            *guard = Some(new_token.clone());
            new_token
        }
        Ok(None) => return Ok(None),
        Err(err) => {
            return Err(err);
        }
    };

    Ok(Some(authentication_token))
}

async fn check_and_refresh_token(
    unix_now: i64,
    client: &BackpackClient,
    token: &AuthenticationToken,
) -> Result<Option<AuthenticationToken>, RequestError> {
    // If authentication token expires "soon", we better refresh it.
    // As communication from client to server can be long or interrupted,
    // we shouldn't wait for the token to actually expire.
    let safe_time_to_refresh = 30;
    if !token
        .biscuit_info
        .will_expire(unix_now + safe_time_to_refresh)
    {
        return Ok(None);
    }
    client
        .refresh(&token.raw_biscuit, &token.refresh_token)
        .await
        .map(|token| Some(token))
}

#[derive(Component)]
pub struct LoginTask(ClientTask<AuthenticationToken>);

impl Default for LoginTask {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Debug, Event)]
pub struct LoginTaskResultEvent(pub Result<AuthenticationToken, RequestError>);

pub fn bevy_login(
    commands: &mut Commands,
    client: &BackpackClient,
    authentication: &BackpackClientAuthRefresh,
    data: LoginEmailPasswordData,
) {
    let thread_pool = IoTaskPool::get();
    let task = LoginTask::default();
    let fill_result_rwlock = task.0.result.clone();
    let client = client.clone();

    // TODO: get a handle to authentication mutex then provide it with the authentication data ; probably also in signup?
    let mutex_to_update_auth_token = authentication.authentication_token.clone();
    thread_pool
        .spawn(async move {
            let response = client.login(&data).await;
            if let Ok(new_token) = &response {
                let mut auth_token_update = mutex_to_update_auth_token.lock().await;
                *auth_token_update = Some(new_token.clone());
            }

            *fill_result_rwlock.write().unwrap() = Some(response);
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
#[derive(Debug, Event)]
pub struct SignupTaskResultEvent(pub Result<shared::CreatedUserEmailPasswordData, RequestError>);

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
            let response = client.signup(&data).await;
            *fill_result_rwlock.write().unwrap() = Some(response);
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
#[derive(Debug, Event)]
pub struct GetItemsTaskResultEvent(pub Result<Vec<ItemAmount>, RequestError>);

pub fn bevy_get_items(
    commands: &mut Commands,
    time: &Time,
    client: &BackpackClient,
    authentication: &BackpackClientAuthRefresh,
    user_id: &UserId,
) {
    let thread_pool = IoTaskPool::get();
    let client = client.clone();
    let user_id = *user_id;
    let task = GetItemsTask::default();
    let fill_result_rwlock = task.0.result.clone();

    // TODO: #22 fix wasm
    let unix_now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let authentication_mutex = authentication.authentication_token.clone();
    thread_pool
        .spawn(async move {
            async fn call_get_items(
                client: BackpackClient,
                authentication_token: &AuthenticationToken,
                data: UserId,
            ) -> Result<Vec<ItemAmount>, RequestError> {
                client.get_items(&authentication_token.raw_biscuit, &data).await
            }

            let auth_token = check_refresh_and_report_token(
                unix_now,
                &client,
                &authentication_mutex,
            );
            *fill_result_rwlock.write().unwrap() = Some(
                match dbg!(auth_token
                .await)
                {
                    Err(err) => {
                        Err(err)
                    }
                    Ok(Some(authentication_token)) => {
                        dbg!(call_get_items(client, &authentication_token, user_id).await)
                    }
                    Ok(None) => {
                        let guard = authentication_mutex.lock().await;
                        match &*guard {
                            Some(authentication_token) => {
                                dbg!(call_get_items(client, authentication_token, user_id).await)
                            }
                            None => {
                                Err(RequestError::Other("Authentication token is None, there might have been a log out with bad timing.".into()))
                            }
                        }
                    }
                },
            );
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
#[derive(Debug, Event)]
pub struct ModifyItemTaskResultEvent(pub Result<(ItemId, UserId, i32), RequestError>);

pub fn bevy_modify_item(
    commands: &mut Commands,
    time: &Time,
    client: &BackpackClient,
    authentication: &BackpackClientAuthRefresh,
    item_id: &ItemId,
    amount: i32,
    user_id: &UserId,
) {
    let thread_pool = IoTaskPool::get();
    let client = client.clone();
    let data = (*item_id, *user_id);
    let task = ModifyItemTask::default();
    let fill_result_rwlock = task.0.result.clone();

    // TODO: #22 fix wasm
    let unix_now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let authentication_mutex = authentication.authentication_token.clone();
    thread_pool
        .spawn(async move {
            async fn call_modify_item(
                client: BackpackClient,
                authentication_token: &AuthenticationToken,
                data: (ItemId, UserId),
                amount: i32,
            ) -> Result<(ItemId, UserId, i32), RequestError> {
                client
                    .modify_item(&authentication_token.raw_biscuit, data.0, amount, data.1)
                    .await
                    .map(|r| (data.0, data.1, r))
            }

            let auth_token = check_refresh_and_report_token(
                unix_now,
                &client,
                &authentication_mutex,
            );
            *fill_result_rwlock.write().unwrap() = Some(
                match auth_token
                .await
                {
                    Err(err) => {
                        Err(err)
                    }
                    Ok(Some(authentication_token)) => {
                        call_modify_item(client, &authentication_token, data, amount).await
                    }
                    Ok(None) => {
                        let guard = authentication_mutex.lock().await;
                        match &*guard {
                            Some(authentication_token) => {
                                call_modify_item(client, authentication_token, data, amount).await
                            }
                            None => {
                                Err(RequestError::Other("Authentication token is None, there might have been a log out with bad timing.".into()))
                            }
                        }
                    }
                },
            );
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
