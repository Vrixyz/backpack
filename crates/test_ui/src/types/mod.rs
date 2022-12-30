use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

mod auth;
pub use auth::*;
mod profiles;
pub use profiles::*;

/// Conduit api error info for Unprocessable Entity error
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ErrorInfo {
    pub errors: HashMap<String, Vec<String>>,
}

pub type DeleteWrapper = HashMap<(), ()>;
