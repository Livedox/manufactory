use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Client(ClientMessage),
    Server(ServerMessage),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    Position(f32, f32, f32),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    AddPlayer((u32, String)),
    RemovePlayer(u32),
    Players(HashMap<u32, (f32, f32, f32)>)
}