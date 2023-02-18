use warp::Filter;

use server_project::{routes, States, UserChannels};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let users = UserChannels::default();
    let users = warp::any().map(move || users.clone());

    let states = States::default();
    let states = warp::any().map(move || states.clone());

    let routes = {
        // access `localhost:3030/status`
        let status = warp::path("status").map(|| warp::reply::html("hello"));

        let game = warp::path("game")
            .and(warp::ws())
            .and(users)
            .and(states)
            .map(|ws: warp::ws::Ws, users, states| {
                //
                ws.on_upgrade(move |socket| routes::user_connected(socket, users, states))
            });

        status.or(game)
    };

    let port = 3030;
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
