use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mimic {
    pub name: String,
    pub avatar_url: Option<String>,
}
