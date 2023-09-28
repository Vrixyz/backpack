mod helper;
#[cfg(test)]
mod tests {

    use backpack_client::shared::AppId;
    use backpack_server::biscuit::AUTHENTICATION_TOKEN_TTL;
    use time::OffsetDateTime;

    use crate::helper::{spawn_app, TestUser};

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
    async fn authentication_and_refresh_token() {
        // Arrange
        let mut app = spawn_app().await;

        let app_id = AppId(0);

        let mut time = app.settings.time.clone();

        // Act
        time.set_override(
            OffsetDateTime::now_utc()
                .checked_sub(time::Duration::seconds(AUTHENTICATION_TOKEN_TTL + 200)),
        );
        let user = TestUser::generate(&mut app.api_client)
            .await
            .expect("error when generating test user");

        let auth_info = user
            .login(&mut app.api_client, Some(app_id))
            .await
            .expect("login failed");

        time.set_override(None);

        app.api_client
            .whoami(&auth_info.raw_biscuit)
            .await
            .expect_err("Token should have expired.");
        let new_auth = app
            .api_client
            .refresh(&auth_info.raw_biscuit, &auth_info.refresh_token)
            .await
            .expect("Token should have correctly be refreshed.");
        let (auth_info, old_auth_info) = (new_auth, auth_info);
        app.api_client
            .whoami(&auth_info.raw_biscuit)
            .await
            .expect("Token should work.");
        time.set_override(OffsetDateTime::now_utc().checked_add(time::Duration::seconds(60 * 2)));
        app.api_client
            .refresh(&old_auth_info.raw_biscuit, &old_auth_info.refresh_token)
            .await
            .expect_err("Old token should not be usable.");
    }
}
