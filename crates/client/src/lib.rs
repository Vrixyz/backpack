use ehttp::Request;
use serde::de::DeserializeOwned;
pub use shared;
use shared::{
    BiscuitInfo, CreateEmailPasswordData, CreatedUserEmailPasswordData, ItemAmount, ItemId,
    LoginEmailPasswordData, UserId, UserItemModify,
};
use thiserror::Error;

const AUTHORIZATION: &str = "Authorization";

type Callback<T> = Box<dyn FnOnce(Result<T, RequestError>) + Send>;

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
}

impl BackpackClient {
    pub fn new(url: String) -> Self {
        Self { url }
    }
    pub fn get_url(&self) -> &'_ str {
        &self.url
    }

    fn make_request<T: DeserializeOwned + 'static>(request: Request, on_done: Callback<T>) {
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
    }

    fn get_auth_bearer_header(biscuit_raw: &[u8]) -> String {
        "Bearer: ".to_string() + std::str::from_utf8(biscuit_raw).unwrap_or_default()
    }

    pub fn signup(
        &self,
        data: &CreateEmailPasswordData,
        on_done: Callback<CreatedUserEmailPasswordData>,
    ) {
        match serde_json::to_vec(&data) {
            Err(err) => on_done(Err(err.into())),
            Ok(data) => {
                let request = ehttp::Request::post(
                    self.url.clone() + "/unauthenticated/email_password/create",
                    data,
                );
                Self::make_request(request, on_done);
            }
        }
    }
    pub fn login(&self, data: &LoginEmailPasswordData, on_done: Callback<(Vec<u8>, BiscuitInfo)>) {
        match serde_json::to_vec(&data) {
            Err(err) => on_done(Err(err.into())),
            Ok(data) => {
                let request = ehttp::Request::post(
                    self.url.clone() + "/unauthenticated/email_password/login",
                    data,
                );
                let cloned_self = self.clone();
                let callback: Callback<String> = Box::new(move |biscuit_raw| {
                    match biscuit_raw {
                        Err(err) => on_done(Err(err)),
                        Ok(biscuit_raw) => {
                            let biscuit_raw = biscuit_raw.as_bytes();
                            let biscuit_raw_saved = biscuit_raw.to_vec();
                            // FIXME: this whoami call could be avoided by decrypting the biscuit with server public key.
                            cloned_self.whoami(
                                biscuit_raw,
                                Box::new(move |biscuit_info| match biscuit_info {
                                    Err(_) => todo!(),
                                    Ok(biscuit_info) => {
                                        on_done(Ok((biscuit_raw_saved, biscuit_info)))
                                    }
                                }),
                            );
                        }
                    }
                });
                Self::make_request::<String>(request, callback);
            }
        }
    }

    /// FIXME: this route should be avoidable by decrypting biscuit information with server public key.
    /// Also, sending auth data could be done via secure http-only cookie.
    pub fn whoami(&self, biscuit_raw: &[u8], on_done: Callback<BiscuitInfo>) {
        let request = Request {
            headers: ehttp::headers(&[(AUTHORIZATION, &Self::get_auth_bearer_header(biscuit_raw))]),
            ..ehttp::Request::get(self.url.clone() + "/unauthenticated/email_password/create")
        };
        Self::make_request(request, on_done);
    }

    pub fn modify_item(
        &self,
        biscuit_raw: &[u8],
        item_id: ItemId,
        amount: i32,
        user_id: UserId,
        on_done: Callback<i32>,
    ) {
        match serde_json::to_vec(&UserItemModify { amount }) {
            Err(err) => on_done(Err(err.into())),
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
                Self::make_request(request, on_done);
            }
        }
    }

    pub fn get_items(
        &self,
        biscuit_raw: &[u8],
        user_id: &UserId,
        on_done: Callback<Vec<ItemAmount>>,
    ) {
        let request = Request {
            headers: ehttp::headers(&[(AUTHORIZATION, &Self::get_auth_bearer_header(biscuit_raw))]),
            ..ehttp::Request::get(format!(
                "{}/authenticated/item/user/{}",
                self.url, user_id.0
            ))
        };
        Self::make_request(request, on_done);
    }

    pub fn get_item(
        &self,
        biscuit_raw: &[u8],
        user_id: &UserId,
        item_id: ItemId,
        on_done: Callback<ItemAmount>,
    ) {
        let request = Request {
            headers: ehttp::headers(&[(AUTHORIZATION, &Self::get_auth_bearer_header(biscuit_raw))]),
            ..ehttp::Request::get(format!(
                "{}/authenticated/item/{}/user/{}",
                self.url, user_id.0, *item_id
            ))
        };
        Self::make_request(request, on_done);
    }
}
