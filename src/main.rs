use warp::Filter;

use server_project::{routes, UserChannels};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // access `localhost:3030/status`
    let status = warp::path("status").map(|| warp::reply::html("hello"));

    let users = UserChannels::default();
    let users = warp::any().map(move || users.clone());

    let game = warp::path("game")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(move |socket| routes::user_connected(socket, user)));

    let routes = status.or(game);

    let port = 3030;
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
