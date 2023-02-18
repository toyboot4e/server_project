use macroquad::prelude::{self as mq, Color};
use vek::Vec2;

#[derive(Default)]
pub struct PlayerState {
    pub pos: Vec2<f32>,
    pub rot: f32,
}

pub struct Game {
    pub quit: bool,
    pub player_state: PlayerState,
    pub texture: mq::Texture2D,
}

impl Game {
    pub async fn new() -> Self {
        let texture = mq::load_texture("assets/plane.png").await.unwrap();

        Self {
            player_state: PlayerState {
                pos: Vec2::new(100f32, 100f32),
                rot: 0f32,
            },
            texture,
            quit: false,
        }
    }

    pub fn update(&mut self) {
        if mq::is_key_down(mq::KeyCode::Escape) {
            self.quit = true;
        }
        const ROT_SPEED: f32 = 0.015;

        if mq::is_key_down(mq::KeyCode::Right) {
            self.player_state.rot += ROT_SPEED;
        }
        if mq::is_key_down(mq::KeyCode::Left) {
            self.player_state.rot -= ROT_SPEED;
        }

        const SPEED: f32 = 0.6;

        self.player_state.pos += self::vec2_from_angle(self.player_state.rot) * SPEED;

        if self.player_state.pos.x > mq::screen_width() {
            self.player_state.pos.x = -self.texture.width();
        } else if self.player_state.pos.x < -self.texture.width() {
            self.player_state.pos.x = mq::screen_width();
        }

        if self.player_state.pos.y > mq::screen_height() {
            self.player_state.pos.y = -self.texture.height();
        } else if self.player_state.pos.y < -self.texture.height() {
            self.player_state.pos.y = mq::screen_height();
        }
    }

    pub fn draw(&self) {
        mq::clear_background(mq::color_u8!(255, 255, 255, 255));

        mq::draw_texture_ex(
            self.texture,
            self.player_state.pos.x,
            self.player_state.pos.y,
            mq::WHITE,
            mq::DrawTextureParams {
                rotation: self.player_state.rot,
                ..Default::default()
            },
        );

        self::draw_box(Vec2::new(400f32, 200f32), Vec2::new(50f32, 20f32));
    }
}

pub fn vec2_from_angle(angle: f32) -> Vec2<f32> {
    // TODO: use `unit_x` and `rotated_z` instead
    let angle = angle - std::f32::consts::FRAC_PI_2;
    Vec2::new(angle.cos(), angle.sin())
}

fn draw_box(pos: Vec2<f32>, size: Vec2<f32>) {
    let dimension = size * 2.;
    let upper_left = pos - size;

    mq::draw_rectangle(
        upper_left.x,
        upper_left.y,
        dimension.x,
        dimension.y,
        mq::BLACK,
    );
}
