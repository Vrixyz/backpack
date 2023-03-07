use crate::data::{
    BiscuitInfo, CreateEmailPasswordData, ItemAmount, ItemId, LoginEmailPasswordData, UserId,
    UserItemModify,
};

use reqwest::Client;

#[derive(Clone, Debug)]
pub struct BackpackClient {
    url: String,
    client: Client,
}

impl BackpackClient {
    pub fn new(url: String) -> Self {
        Self {
            url,
            client: Client::new(),
        }
    }
    pub async fn signup(&self, data: &CreateEmailPasswordData) -> Result<(), reqwest::Error> {
        self.client
            .post(dbg!(
                self.url.clone() + "/unauthenticated/email_password/create"
            ))
            .json(data)
            .send()
            .await?
            .text()
            .await
            .map(|_| ())
    }
    pub async fn login(
        &self,
        data: &LoginEmailPasswordData,
    ) -> Result<(Vec<u8>, BiscuitInfo), reqwest::Error> {
        let biscuit_raw = dbg!(
            self.client
                .post(dbg!(
                    self.url.clone() + "/unauthenticated/email_password/login"
                ))
                .json(data)
                .send()
                .await?
                .text()
                .await?
        );
        self.whoami(biscuit_raw.as_bytes()).await
    }

    /// FIXME: this route should be avoidable by decrypting biscuit information with server public key.
    /// Also, sending auth data could be done via secure http-only cookie.
    pub async fn whoami(
        &self,
        biscuit_raw: &[u8],
    ) -> Result<(Vec<u8>, BiscuitInfo), reqwest::Error> {
        let biscuit = dbg!(
            dbg!(self
                .client
                .get(self.url.clone() + "/authenticated/whoami")
                .bearer_auth(std::str::from_utf8(biscuit_raw).expect("wrong auth token")))
            .send()
            .await?
        )
        .json::<BiscuitInfo>()
        .await?;
        Ok((biscuit_raw.into(), dbg!(biscuit)))
    }

    pub async fn modify_item(
        &self,
        biscuit_raw: &[u8],
        item_id: ItemId,
        amount: i32,
        user_id: UserId,
    ) -> Result<i32, reqwest::Error> {
        let new_amount = dbg!(
            dbg!(self
                .client
                .post(format!(
                    "{}/authenticated/item/{}/user/{}/modify",
                    self.url, *item_id, *user_id
                ))
                .json(&UserItemModify { amount })
                .bearer_auth(std::str::from_utf8(biscuit_raw).expect("wrong auth token")))
            .send()
            .await?
        )
        .json::<i32>()
        .await?;
        Ok(dbg!(new_amount))
    }

    pub async fn get_items(
        &self,
        biscuit_raw: &[u8],
        user_id: &UserId,
    ) -> Result<Vec<ItemAmount>, reqwest::Error> {
        let res = dbg!(
            dbg!(self
                .client
                .get(format!(
                    "{}/authenticated/item/user/{}",
                    self.url, user_id.0
                ))
                .bearer_auth(std::str::from_utf8(biscuit_raw).expect("wrong auth token")))
            .send()
            .await?
        )
        .json::<Vec<ItemAmount>>()
        .await?;
        Ok(dbg!(res))
    }
    pub async fn get_item(
        &self,
        biscuit_raw: &[u8],
        user_id: &UserId,
        item_id: ItemId,
    ) -> Result<ItemAmount, reqwest::Error> {
        let res = dbg!(
            dbg!(self
                .client
                .get(format!(
                    "{}/authenticated/item/{}/user/{}",
                    self.url, user_id.0, *item_id
                ))
                .bearer_auth(std::str::from_utf8(biscuit_raw).expect("wrong auth token")))
            .send()
            .await?
        )
        .json::<ItemAmount>()
        .await?;
        Ok(dbg!(res))
    }
}
