use warp::Filter;

use server_project::routes;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // access `localhost:3030/status`
    let status = warp::path("status").map(|| warp::reply::html("hello"));

    let game = warp::path("game")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(|socket| routes::user_connected(socket)));

    let routes = status.or(game);

    let port = 3030;
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
