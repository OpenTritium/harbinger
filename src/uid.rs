use serde::{Deserialize, Serialize};

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Uid(String);

impl From<String> for Uid {
    fn from(s: String) -> Self {
        Uid(s)
    }
}
