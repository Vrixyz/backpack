use serde::Serialize;
use uuid::Uuid;

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
