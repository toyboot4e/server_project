use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::sync::mpsc;
use warp::ws::{self, WebSocket};

use crate::{
    ClientMessage, OutBoundChannel, RemoteState, ServerMessage, States, UserChannels, UserId,
};

pub async fn user_connected(ws: WebSocket, users: UserChannels, states: States) {
    use futures_util::StreamExt;

    let (ws_send, mut ws_recv) = ws.split();
    let send_channel = self::create_send_channel(ws_send);

    // register
    let my_id = self::send_welcome(&send_channel).await;
    log::debug!("new user connected: {my_id:?}");

    users.write().await.insert(my_id, send_channel);

    // receiver loop
    while let Some(result) = ws_recv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(err) => {
                log::warn!("websocket error ({my_id:?}):`{err}`");
                break;
            }
        };

        log::debug!("user sent message: {msg:?}");

        if let Some(msg) = self::parse_message(msg) {
            self::user_message(my_id, msg, &states).await;
        }
    }

    log::debug!("user disconnected: {}", my_id);

    // unregister
    users.write().await.remove(&my_id);
    states.write().await.remove(&my_id);

    self::broadcast(ServerMessage::GoodBye(my_id), &users).await;
}

fn create_send_channel(
    ws_sender: futures_util::stream::SplitSink<WebSocket, ws::Message>,
) -> OutBoundChannel {
    use futures_util::{FutureExt, StreamExt};
    use tokio_stream::wrappers::UnboundedReceiverStream;

    let (send, recv) = mpsc::unbounded_channel();
    let rx = UnboundedReceiverStream::new(recv);

    tokio::task::spawn(rx.forward(ws_sender).map(|result| {
        if let Err(err) = result {
            log::error!("websocket send erreor: {err}");
        }
    }));

    send
}

static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

async fn send_welcome(out: &OutBoundChannel) -> UserId {
    let id = UserId(NEXT_USER_ID.fetch_add(1, Ordering::Relaxed));
    let states = ServerMessage::Welcome(id);
    crate::send_msg(out, &states).await;

    unimplemented!()
}

/// Broadcasts a message from one user to every other.
async fn broadcast(msg: ServerMessage, users: &UserChannels) {
    let users = users.read().await;

    // TODO: exclude the original sender
    // TODO: consider joining those futures
    for tx in users.iter().map(|x| x.1) {
        crate::send_msg(tx, &msg).await;
    }
}

/// Parses a websocket message into a client message.
fn parse_message(msg: ws::Message) -> Option<ClientMessage> {
    if msg.is_binary() {
        let msg = msg.into_bytes();
        // malformed input will just be `None`
        serde_json::from_slice(msg.as_slice()).ok()
    } else {
        None
    }
}

async fn user_message(id: UserId, msg: ClientMessage, states: &States) {
    match msg {
        ClientMessage::State(state) => {
            let msg = RemoteState {
                id,
                pos: state.pos,
                rot: state.rot,
            };

            states.write().await.insert(msg.id, msg);
        }
    }
}
