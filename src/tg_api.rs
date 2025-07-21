use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct UpdateResponse {
    pub ok: bool,
    pub result: Vec<Update>,
}
// serialize because we serialize before sending it into wasm
#[derive(Debug, Deserialize, Serialize)]
pub struct Update {
    pub update_id: u32,
    pub message: Option<Message>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    pub text: Option<String>,
    pub chat: Chat,
    pub from: Option<User>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Chat {
    pub id: i64
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    id: i64,
    first_name: String,
    last_name: Option<String>
}

#[derive(Serialize)]
pub struct SendMessage {
    pub chat_id: i64,
    pub text: String,
}
