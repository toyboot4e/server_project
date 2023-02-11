use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::sync::mpsc;
use warp::ws::{Message, WebSocket};

use crate::OutBoundChannel;

pub async fn user_connected(ws: WebSocket) {
    use futures_util::StreamExt;

    let (ws_send, mut ws_recv) = ws.split();
    let send_channel = self::create_send_channel(ws_send);

    let my_id = self::send_welcome(&send_channel).await;
    log::debug!("new user connected: {}", my_id);

    while let Some(result) = ws_recv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(err) => {
                log::warn!("websocket receive error: `{}`", err);
                break;
            }
        };

        log::debug!("user sent message: {:?}", msg);
    }

    log::debug!("user disconnected: {}", my_id);
}

fn create_send_channel(
    ws_sender: futures_util::stream::SplitSink<WebSocket, Message>,
) -> OutBoundChannel {
    use futures_util::{FutureExt, StreamExt};
    use tokio_stream::wrappers::UnboundedReceiverStream;

    let (send, recv) = mpsc::unbounded_channel();
    let rx = UnboundedReceiverStream::new(recv);

    tokio::task::spawn(rx.forward(ws_sender).map(|result| {
        if let Err(err) = result {
            log::error!("websocket send erreor: {}", err);
        }
    }));

    send
}

static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

async fn send_welcome(out: &OutBoundChannel) -> usize {
    let id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);
    let states = crate::ServerMessage::Welcome(id);
    crate::send_msg(out, &states).await;

    unimplemented!()
}
