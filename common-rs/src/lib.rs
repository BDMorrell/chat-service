use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ChatMessage {
    pub user: User,
    pub time: DateTime<Utc>,
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct User {
    pub id: u64,
    pub name: String,
}

pub fn get_schemas_to_export() -> Vec<(&'static str, schemars::schema::RootSchema)> {
    macro_rules! export_schemas {
        ( $($type:ty),* ) => {
            vec![
                    $(  (std::stringify!($type), schemars::schema_for!($type)),  )*
            ]
        }
    }

    export_schemas!(ChatMessage, User)
}
