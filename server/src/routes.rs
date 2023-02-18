//! User connection handling in routes.

use tokio::sync::mpsc;
use warp::ws::{self, WebSocket};

use crate::{
    ClientMessage, OutBoundChannel, RemoteState, RemoteStates, ServerMessage, UserChannels, UserId,
};

/// User message handling loop.
pub async fn user_message_loop(
    ws: WebSocket,
    user_channels: UserChannels,
    remote_states: RemoteStates,
) {
    use futures_util::StreamExt;

    let (ws_send, mut ws_recv) = ws.split();
    let send_channel = self::create_send_channel(ws_send);

    // register
    let user_id = self::send_welcome_message(&send_channel).await;
    log::debug!("new user connected: {user_id:?}");

    user_channels.write().await.insert(user_id, send_channel);

    // receiver loop
    while let Some(result) = ws_recv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(err) => {
                log::warn!("websocket error ({user_id:?}):`{err}`");
                break;
            }
        };

        log::debug!("user sent message: {msg:?}");

        if let Some(msg) = self::parse_user_message(msg) {
            self::handle_user_message(user_id, msg, &remote_states).await;
        }
    }

    log::debug!("user disconnected: {}", user_id);

    // unregister
    user_channels.write().await.remove(&user_id);
    remote_states.write().await.remove(&user_id);

    self::broadcast(ServerMessage::GoodBye(user_id), &user_channels).await;
}

/// Creates a send channel from server to a client.
fn create_send_channel(
    ws_sender: futures_util::stream::SplitSink<WebSocket, ws::Message>,
) -> OutBoundChannel {
    use futures_util::{FutureExt, StreamExt};
    use tokio_stream::wrappers::UnboundedReceiverStream;

    let (send, recv) = mpsc::unbounded_channel();
    let rx = UnboundedReceiverStream::new(recv);

    tokio::task::spawn(rx.forward(ws_sender).map(|result| {
        if let Err(err) = result {
            log::error!("websocket send error: {err}");
        }
    }));

    send
}

/// Sends a welcome meesage to a client.
async fn send_welcome_message(out: &OutBoundChannel) -> UserId {
    let id = UserId::create_new_user_id();
    let states = ServerMessage::Welcome(id);
    crate::send_server_message(out, &states).await;
    id
}

/// Broadcasts a message from one user to every other.
async fn broadcast(msg: ServerMessage, users: &UserChannels) {
    let users = users.read().await;

    // TODO: exclude the original sender
    // TODO: consider joining those futures
    for tx in users.values() {
        crate::send_server_message(tx, &msg).await;
    }
}

/// Parses a user message encoded as a websocket message.
fn parse_user_message(msg: ws::Message) -> Option<ClientMessage> {
    if msg.is_binary() {
        let msg = msg.into_bytes();
        // malformed input will just be `None`
        serde_json::from_slice(msg.as_slice()).ok()
    } else {
        None
    }
}

async fn handle_user_message(id: UserId, msg: ClientMessage, states: &RemoteStates) {
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
