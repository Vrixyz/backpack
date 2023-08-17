use ehttp::Request;
use serde::de::DeserializeOwned;
pub use shared;
use shared::{
    AuthenticationResponse, BiscuitInfo, CreateEmailPasswordData, CreatedUserEmailPasswordData,
    ItemAmount, ItemId, LoginEmailPasswordData, RefreshToken, RefreshTokenString, UserId,
    UserItemModify,
};
use thiserror::Error;

const AUTHORIZATION: &str = "Authorization";

type Callback<T> = Box<dyn FnOnce(Result<T, RequestError>) + Send>;
type RequestResult<T> = Result<T, RequestError>;

#[derive(Clone, Debug)]
pub struct BackpackClient {
    url: String,
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("could not serialize data")]
    SerializeError(#[from] serde_json::Error),
    #[error("Error in http request")]
    HttpError(String),
    #[error("Error 4xx or 5xx")]
    StatusError { status: u16, bytes: Vec<u8> },
    #[error("other error")]
    Other(String),
}

impl BackpackClient {
    pub fn new(url: String) -> Self {
        Self { url }
    }
    pub fn get_url(&self) -> &'_ str {
        &self.url
    }
    /*
    fn make_request_callback<T: DeserializeOwned + 'static>(request: Request, on_done: Callback<T>) {
        ehttp::fetch(request, move |response| match response {
            Err(error) => on_done(Err(RequestError::HttpError(error))),
            Ok(response) => {
                if (400..=599).contains(&response.status) {
                    on_done(Err(RequestError::StatusError {
                        status: response.status,
                        bytes: response.bytes,
                    }));
                    return;
                }
                let received: Result<T, _> =
                    serde_json::from_slice(&response.bytes).map_err(|err| err.into());
                on_done(received);
            }
        });
    }*/
    async fn make_request(mut request: Request) -> Result<Vec<u8>, RequestError> {
        if !request.body.is_empty() {
            request
                .headers
                .insert("Content-Type".into(), "application/json".into());
        }
        let response = ehttp::fetch_async(request).await;
        match dbg!(response) {
            Err(error) => Err(RequestError::HttpError(error)),
            Ok(response) => {
                if (400..=599).contains(&response.status) {
                    return Err(RequestError::StatusError {
                        status: response.status,
                        bytes: response.bytes,
                    });
                }
                Ok(response.bytes)
            }
        }
    }

    fn parse<T: DeserializeOwned + 'static>(bytes: Vec<u8>) -> RequestResult<T> {
        serde_json::from_slice(&bytes).map_err(|err| err.into())
    }

    fn get_auth_bearer_header(biscuit_raw: &[u8]) -> String {
        "Bearer ".to_string() + std::str::from_utf8(biscuit_raw).unwrap_or_default()
    }

    pub async fn signup(
        &self,
        data: &CreateEmailPasswordData,
    ) -> RequestResult<CreatedUserEmailPasswordData> {
        match serde_json::to_vec(&data) {
            Err(err) => Err(err.into()),
            Ok(data) => {
                let request = ehttp::Request::post(
                    self.url.clone() + "/authentication/email_password/create",
                    data,
                );
                Self::parse(Self::make_request(request).await?)
            }
        }
    }
    pub async fn login(
        &self,
        data: &LoginEmailPasswordData,
    ) -> RequestResult<(RefreshToken, Vec<u8>, BiscuitInfo)> {
        match serde_json::to_vec(&data) {
            Err(err) => Err(err.into()),
            Ok(data) => {
                let request = ehttp::Request::post(
                    self.url.clone() + "/authentication/email_password/login",
                    data,
                );
                let response = Self::make_request(request).await?;
                self.handle_authentication_response(response).await
            }
        }
    }

    async fn handle_authentication_response(
        &self,
        response: Vec<u8>,
    ) -> Result<(RefreshToken, Vec<u8>, BiscuitInfo), RequestError> {
        let authentication_response: AuthenticationResponse = Self::parse(response)?;
        let biscuit_raw = authentication_response.auth_token.as_bytes();

        let biscuit_raw_saved = biscuit_raw.to_vec();
        // FIXME: this whoami call could be avoided by decrypting the biscuit with server public key.
        // self is cloned because it could be destroyed during the network call.
        let biscuit_info = self.whoami(biscuit_raw).await;
        match biscuit_info {
            Err(e) => Err(e),
            Ok(biscuit_info) => Ok((
                authentication_response.refresh_token,
                biscuit_raw_saved,
                biscuit_info,
            )),
        }
    }

    /// FIXME: this route should be avoidable by decrypting biscuit information with server public key.
    /// Also, sending auth data could be done via secure http-only cookie.
    pub async fn whoami(&self, biscuit_raw: &[u8]) -> RequestResult<BiscuitInfo> {
        let request = Request {
            headers: ehttp::headers(&[(AUTHORIZATION, &Self::get_auth_bearer_header(biscuit_raw))]),
            ..ehttp::Request::get(self.url.clone() + "/authenticated/whoami")
        };
        Self::parse(Self::make_request(request).await?)
    }
    pub async fn delete(&self, biscuit_raw: &[u8]) -> RequestResult<()> {
        let request = Request {
            headers: ehttp::headers(&[(AUTHORIZATION, &Self::get_auth_bearer_header(biscuit_raw))]),
            method: "DELETE".to_owned(),
            ..ehttp::Request::get(self.url.clone() + "/authenticated/user")
        };
        Self::make_request(request).await?;
        Ok(())
    }

    pub async fn modify_item(
        &self,
        biscuit_raw: &[u8],
        item_id: ItemId,
        amount: i32,
        user_id: UserId,
    ) -> RequestResult<i32> {
        match serde_json::to_vec(&UserItemModify { amount }) {
            Err(err) => Err(err.into()),
            Ok(data) => {
                let request = Request {
                    headers: ehttp::headers(&[(
                        AUTHORIZATION,
                        &Self::get_auth_bearer_header(biscuit_raw),
                    )]),
                    ..ehttp::Request::post(
                        format!(
                            "{}/authenticated/item/{}/user/{}/modify",
                            self.url, *item_id, *user_id
                        ),
                        data,
                    )
                };
                Self::parse(Self::make_request(request).await?)
            }
        }
    }

    pub async fn get_items(
        &self,
        biscuit_raw: &[u8],
        user_id: &UserId,
    ) -> RequestResult<Vec<ItemAmount>> {
        let request = Request {
            headers: ehttp::headers(&[(AUTHORIZATION, &Self::get_auth_bearer_header(biscuit_raw))]),
            ..ehttp::Request::get(format!(
                "{}/authenticated/item/user/{}",
                self.url, user_id.0
            ))
        };
        Self::parse(Self::make_request(request).await?)
    }

    pub async fn get_item(
        &self,
        biscuit_raw: &[u8],
        user_id: &UserId,
        item_id: ItemId,
    ) -> RequestResult<ItemAmount> {
        let request = Request {
            headers: ehttp::headers(&[(AUTHORIZATION, &Self::get_auth_bearer_header(biscuit_raw))]),
            ..ehttp::Request::get(format!(
                "{}/authenticated/item/{}/user/{}",
                self.url, user_id.0, *item_id
            ))
        };
        Self::parse(Self::make_request(request).await?)
    }

    pub async fn refresh(
        &self,
        biscuit_raw: &[u8],
        refresh_token: RefreshToken,
    ) -> RequestResult<(RefreshToken, Vec<u8>, BiscuitInfo)> {
        match serde_json::to_vec(&refresh_token) {
            Err(err) => Err(err.into()),
            Ok(data) => {
                let request = Request {
                    headers: ehttp::headers(&[(
                        AUTHORIZATION,
                        &Self::get_auth_bearer_header(biscuit_raw),
                    )]),
                    ..ehttp::Request::post(
                        format!("{}/authentication/auth/refresh", self.url),
                        data,
                    )
                };
                self.handle_authentication_response(Self::make_request(request).await?)
                    .await
            }
        }
    }
}
