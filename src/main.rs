use warp::Filter;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // access `localhost:3030/status`
    let status = warp::path!("status").map(|| warp::reply::html("hello"));

    let port = 3030;
    warp::serve(status).run(([0, 0, 0, 0], port)).await;
}
