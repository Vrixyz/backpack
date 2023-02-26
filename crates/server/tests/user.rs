use backpack_client::shared::CreateEmailPasswordData;
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
async fn an_error_flash_message_is_set_on_failure() {
    // Arrange
    let mut app = spawn_app().await;

    // Act
    TestUser::generate(&mut app.api_client)
        .await
        .expect("error when generating test user");
}
