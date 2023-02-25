use anyhow::Result;
use macroquad::prelude::*;

use client::ws::Connection;

#[macroquad::main("game")]
async fn main() -> Result<()> {
    let mut connection = Connection::default();
    let url = "ws://localhost:3030/game";
    connection.connect(url)?;

    let mut game = client::Game::new().await;

    loop {
        game.update();
        game.draw();

        if game.quit {
            break;
        }

        next_frame().await
    }

    Ok(())
}
