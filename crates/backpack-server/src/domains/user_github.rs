#[derive(Debug, Deserialize, Serialize)]
pub struct GithubUser {
    login: String,
    id: u32,
}
