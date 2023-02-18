use warp::Filter;

use server::{routes, RemoteStates, UserChannels};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let users = UserChannels::default();
    let states = RemoteStates::default();

    // spawn update loop
    {
        let arc_users = users.clone();
        let arc_states = states.clone();

        tokio::spawn(async move {
            server::update_loop(arc_users, arc_states).await;
        });
    }

    let users = warp::any().map(move || users.clone());
    let states = warp::any().map(move || states.clone());

    // create REST endpoints with WebSocket connetions using `warp`:
    let routes = {
        // access `localhost:3030/status`
        let status = warp::path("status").map(|| warp::reply::html("hello"));

        let game = warp::path("game")
            .and(warp::ws())
            .and(users)
            .and(states)
            .map(|ws: warp::ws::Ws, users, states| {
                // handle user message in the loop
                ws.on_upgrade(move |socket| routes::user_message_loop(socket, users, states))
            });

        status.or(game)
    };

    let port = 3030;
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
