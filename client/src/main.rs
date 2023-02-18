use macroquad::prelude::*;

#[macroquad::main("game")]
async fn main() {
    let mut game = client::Game::new().await;

    loop {
        game.update();
        game.draw();

        if game.quit {
            return;
        }

        next_frame().await
    }
}
