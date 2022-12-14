use backpack_server::domains::oauth::TokenReply;
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

#[tokio::test]
async fn get_test_token() {
    let app = helper::spawn_app().await;
    let client = reqwest::Client::builder()
        .pool_max_idle_per_host(0)
        .build()
        .unwrap();

    dbg!(&app.address);
    let response = client
        .get(&format!("{}/oauth/fake/success", app.address))
        .header("keep-alive", "")
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(dbg!(response.status()).is_success());

    sqlx::query!("SELECT id FROM users",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
}
