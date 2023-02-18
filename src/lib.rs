pub mod routes;

use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use vek::Vec2;
use warp::ws::Message;

/// Channel of a connected user
pub type OutBoundChannel = mpsc::UnboundedSender<Result<Message, warp::Error>>;

/// Channels of currently connected users (clients)
pub type UserChannels = Arc<RwLock<HashMap<UserId, OutBoundChannel>>>;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(usize);

impl UserId {
    pub fn from_usize(id: usize) -> Self {
        Self(id)
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct State {
    pub pos: Vec2<f32>,
    pub radius: f32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct RemoteState {
    pub id: UserId,
    pub pos: Vec<f32>,
    pub rot: f32,
}

/// Message sent from server to client
#[derive(Deserialize, Serialize, Clone)]
pub enum ServerMessage {
    Welcome(UserId),
    GoodBye(UserId),
    Update(Vec<RemoteState>),
}

/// Message sent from client to server
#[derive(Deserialize, Serialize, Clone)]
pub enum ClientMessage {
    State(State),
}

// TODO: add error handling instead of `unwrap`
pub async fn send_msg(tx: &OutBoundChannel, msg: &ServerMessage) {
    let buffer = serde_json::to_vec(&msg).unwrap();
    let msg = Message::binary(buffer);
    tx.send(Ok(msg)).unwrap();
}
