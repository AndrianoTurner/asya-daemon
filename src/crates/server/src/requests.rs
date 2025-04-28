use serde::{Deserialize, Serialize};
use usecases::UsecaseMeta;

/// Represents different types of requests that can be made to the server.
///
/// The `Requests` enum is used to categorize and handle various actions
/// that the server can process. Each variant of the enum corresponds to
/// a specific type of request, with associated data as needed.
///
/// Example json request that turns off music:
///
/// ```json
/// {
///     "general": {
///         "action": "turnOffMusic"
///     }
/// }
/// ```
///
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Requests {
    /// A general request that includes an `Usecases` to be performed.
    Command {
        action: UsecaseMeta,
    },
    Human {
        message: String,
    },
}

// fn ebra() {
//     Usecase {
//         name: "turnOfMusic".to_string(),
//         id_receiver: "34af02f7-9307-44ec-b8ce-9da247704547".to_string(),
//         payload_type: "json".to_string(),
//         payload: serde_json::to_string("uscases").unwrap(),
//     };
// }
