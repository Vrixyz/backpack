use backpack_client::shared::{AppId, CreateEmailPasswordData};
use serde::Serialize;
use uuid::Uuid;

use crate::helper::{spawn_app, TestUser};

mod helper;
/*
#[tokio::test]
async fn not_authenticated() {
    let app = helper::spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/api/v1/oauth/github/fake", app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(401, response.status());
    assert_eq!(Some(0), response.content_length());
}*/

#[derive(Serialize)]
struct UuidInput {
    uuid: Uuid,
}

#[tokio::test]
async fn signup_and_login_as_admin() {
    // Arrange
    let mut app = spawn_app().await;

    // Act
    let user = TestUser::generate(&mut app.api_client)
        .await
        .expect("error when generating test user");
    let auth_info = user
        .login(&mut app.api_client, None)
        .await
        .expect("login failed");
}

#[tokio::test]
async fn signup_and_login_as_user() {
    // Arrange
    let mut app = spawn_app().await;

    let app_id = AppId(0);

    // Act
    let user = TestUser::generate(&mut app.api_client)
        .await
        .expect("error when generating test user");

    let auth_info = user
        .login(&mut app.api_client, Some(app_id))
        .await
        .expect("login failed");
}

#[tokio::test]
async fn signup_login_delete_user() {
    // Arrange
    let mut app = spawn_app().await;

    let app_id = AppId(0);

    // Act
    let user = TestUser::generate(&mut app.api_client)
        .await
        .expect("error when generating test user");

    let auth_info = user
        .login(&mut app.api_client, Some(app_id))
        .await
        .expect("login failed");
    app.api_client
        .delete(&auth_info.biscuit_raw)
        .await
        .expect("delete user failed");
}
