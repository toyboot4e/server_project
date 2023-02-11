pub mod routes;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use vek::Vec2;
use warp::ws::Message;

pub type OutBoundChannel = mpsc::UnboundedSender<Result<Message, warp::Error>>;

#[derive(Deserialize, Serialize, Clone)]
pub struct State {
    pub pos: Vec2<f32>,
    pub radius: f32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct RemoteState {
    pub id: usize,
    pub pos: Vec<f32>,
    pub rot: f32,
}

/// Message sent from server to client
#[derive(Deserialize, Serialize, Clone)]
pub enum ServerMessage {
    Welcome(usize),
    GoodBye(usize),
    Update(Vec<RemoteState>),
}

/// Message sent from client to server
#[derive(Deserialize, Serialize, Clone)]
pub enum ClientMessage {
    State(State),
}

pub async fn send_msg(tx: &OutBoundChannel, msg: &ServerMessage) {
    let buffer = serde_json::to_vec(&msg).unwrap();
    let msg = Message::binary(buffer);
    tx.send(Ok(msg)).unwrap();
}
