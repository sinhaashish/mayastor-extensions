// use serde::{Serialize, Deserialize};
// use std::fmt::Debug;

// #[derive(Serialize, Deserialize, Debug)]
// pub struct EventMessage {
//     pub category: String,
//     pub action: String,
//     pub target: String,
//     pub metadata: EventMeta,
// }

// // #[derive(Derivative)]
// #[derive(Serialize, Deserialize, Debug)]
// pub struct EventMeta {
//     pub id: String,
//     pub source: EventSource,
//     pub eventTimestamp: String,
//     pub version: String,
// }

// // #[derive(Derivative)]
// #[derive(Serialize, Deserialize, Debug)]
// pub struct EventSource {
//     pub component: String,
//     pub node: String,
// }