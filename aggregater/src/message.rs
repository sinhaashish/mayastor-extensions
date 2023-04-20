use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EventMessage {
    id: String,
    pub category: String,
    pub action: String,
    pub target: String,
    pub node: String,
}