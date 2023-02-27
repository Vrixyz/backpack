pub use shared;
use shared::{
    BiscuitInfo, CreateEmailPasswordData, CreatedUserEmailPasswordData, ItemAmount, ItemId,
    LoginEmailPasswordData, UserId, UserItemModify,
};

use reqwest::Client;

#[derive(Clone, Debug)]
pub struct BackpackClient {
    url: String,
    client: Client,
}

impl BackpackClient {
    pub fn new_with_client(url: String, client: Client) -> Self {
        Self { url, client }
    }
    pub fn new(url: String) -> Self {
        Self::new_with_client(url, Client::new())
    }
    pub fn get_url(&self) -> &'_ str {
        &self.url
    }
    pub fn get_client(&self) -> &'_ Client {
        &self.client
    }
    pub fn get_client_mut(&mut self) -> &'_ mut Client {
        &mut self.client
    }

    /* Json is: (see CreatedUserEmailPasswordData, return that from within client.signup)
    {
        "id": 2,
        "password": "XFbUnzBs~WP)y8u*"
      }
    */
    pub async fn signup(
        &self,
        data: &CreateEmailPasswordData,
    ) -> Result<CreatedUserEmailPasswordData, reqwest::Error> {
        self.client
            .post(dbg!(
                self.url.clone() + "/unauthenticated/email_password/create"
            ))
            .json(data)
            .send()
            .await?
            .error_for_status()?
            .json::<CreatedUserEmailPasswordData>()
            .await
    }
    pub async fn login(
        &self,
        data: &LoginEmailPasswordData,
    ) -> Result<(Vec<u8>, BiscuitInfo), reqwest::Error> {
        let biscuit_raw = self
            .client
            .post(self.url.clone() + "/unauthenticated/email_password/login")
            .json(dbg!(data))
            .send()
            .await?
            .text()
            .await?;
        self.whoami(biscuit_raw.as_bytes()).await
    }

    /// FIXME: this route should be avoidable by decrypting biscuit information with server public key.
    /// Also, sending auth data could be done via secure http-only cookie.
    pub async fn whoami(
        &self,
        biscuit_raw: &[u8],
    ) -> Result<(Vec<u8>, BiscuitInfo), reqwest::Error> {
        let biscuit = self
            .client
            .get(self.url.clone() + "/authenticated/whoami")
            .bearer_auth(std::str::from_utf8(biscuit_raw).expect("wrong auth token"))
            .send()
            .await?
            .json::<BiscuitInfo>()
            .await?;
        Ok((biscuit_raw.into(), biscuit))
    }

    pub async fn modify_item(
        &self,
        biscuit_raw: &[u8],
        item_id: ItemId,
        amount: i32,
        user_id: UserId,
    ) -> Result<i32, reqwest::Error> {
        let new_amount = self
            .client
            .post(format!(
                "{}/authenticated/item/{}/user/{}/modify",
                self.url, *item_id, *user_id
            ))
            .json(&UserItemModify { amount })
            .bearer_auth(std::str::from_utf8(biscuit_raw).expect("wrong auth token"))
            .send()
            .await?
            .json::<i32>()
            .await?;
        Ok(new_amount)
    }

    pub async fn get_items(
        &self,
        biscuit_raw: &[u8],
        user_id: &UserId,
    ) -> Result<Vec<ItemAmount>, reqwest::Error> {
        let res = self
            .client
            .get(format!(
                "{}/authenticated/item/user/{}",
                self.url, user_id.0
            ))
            .bearer_auth(std::str::from_utf8(biscuit_raw).expect("wrong auth token"))
            .send()
            .await?
            .json::<Vec<ItemAmount>>()
            .await?;
        Ok(res)
    }
    pub async fn get_item(
        &self,
        biscuit_raw: &[u8],
        user_id: &UserId,
        item_id: ItemId,
    ) -> Result<ItemAmount, reqwest::Error> {
        let res = self
            .client
            .get(format!(
                "{}/authenticated/item/{}/user/{}",
                self.url, user_id.0, *item_id
            ))
            .bearer_auth(std::str::from_utf8(biscuit_raw).expect("wrong auth token"))
            .send()
            .await?
            .json::<ItemAmount>()
            .await?;
        Ok(res)
    }
}
