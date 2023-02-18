//! Where I follow [Multiplayer Game Development in Rust][the-book]
//!
//! [the-book]: https://www.manning.com/books/multiplayer-game-development-in-rust

pub mod routes;

use std::{
    collections::HashMap,
    fmt,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use vek::Vec2;
use warp::ws::Message;

const UPDATE_INTERVAL: Duration = Duration::from_millis(100);

/// In-memoery game state: `UserId`-> `RemoteState`.
// consider using database (?) for scalability
pub type States = Arc<RwLock<HashMap<UserId, RemoteState>>>;

/// Channel of a connected user.
pub type OutBoundChannel = mpsc::UnboundedSender<Result<Message, warp::Error>>;

/// Channels of currently connected users (clients).
pub type UserChannels = Arc<RwLock<HashMap<UserId, OutBoundChannel>>>;

/// User ID given to each client by the server.
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(usize);

impl UserId {
    pub fn from_usize(id: usize) -> Self {
        Self(id)
    }

    fn create_new_user_id() -> Self {
        static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

        UserId(NEXT_USER_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Message sent from server to client.
#[derive(Deserialize, Serialize, Clone)]
pub enum ServerMessage {
    Welcome(UserId),
    GoodBye(UserId),
    Update(Vec<RemoteState>),
}

/// Game state stored in the server side.
#[derive(Deserialize, Serialize, Clone)]
pub struct RemoteState {
    pub id: UserId,
    pub pos: Vec2<f32>,
    pub rot: f32,
}

/// Message sent from client to server.
#[derive(Deserialize, Serialize, Clone)]
pub enum ClientMessage {
    State(ClientState),
}

/// Game state sent from the client side.
#[derive(Deserialize, Serialize, Clone)]
pub struct ClientState {
    pub pos: Vec2<f32>,
    pub rot: f32,
}

/// Sends a server message to a client.
// TODO: add error handling instead of `unwrap`
pub async fn send_server_message(tx: &OutBoundChannel, msg: &ServerMessage) {
    let buffer = serde_json::to_vec(&msg).unwrap();
    let msg = Message::binary(buffer);
    tx.send(Ok(msg)).unwrap();
}

/// Server-side update loop.
pub async fn update_loop(users: UserChannels, states: States) {
    loop {
        let states = states.read().await.values().cloned().collect::<Vec<_>>();

        if !states.is_empty() {
            for (&uid, tx) in users.read().await.iter() {
                let states = states
                    .iter()
                    .filter_map(|state| {
                        if state.id == uid {
                            None
                        } else {
                            Some(state.clone())
                        }
                    })
                    .collect::<Vec<_>>();

                let states = ServerMessage::Update(states);

                self::send_server_message(tx, &states).await;
            }
        }

        tokio::time::sleep(UPDATE_INTERVAL).await;
    }
}
