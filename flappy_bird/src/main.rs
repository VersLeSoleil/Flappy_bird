use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics::{self, Color};
use ggez::input::keyboard;
use ggez::{Context, ContextBuilder, GameResult};
use rand::Rng;

const BIRD_WIDTH: f32 = 34.0;
const BIRD_HEIGHT: f32 = 24.0;
const PIPE_WIDTH: f32 = 52.0;
const PIPE_HEIGHT: f32 = 320.0;
const PIPE_SPACING: f32 = 400.0; // 调整障碍物间隔
const GRAVITY: f32 = 0.1; // 调整重力
const JUMP_STRENGTH: f32 = -4.0; // 调整跳跃力度
const WINDOW_WIDTH: f32 = 800.0; // 调整窗口宽度
const WINDOW_HEIGHT: f32 = 800.0; // 调整窗口高度

struct Bird {
    x: f32,
    y: f32,
    velocity: f32,
}

struct Pipe {
    x: f32,
    height: f32,
}

struct MainState {
    bird: Bird,
    pipes: Vec<Pipe>,
    score: i32,
    game_over: bool,
}

impl MainState {
    fn new() -> GameResult<MainState> {
        let s = MainState {
            bird: Bird {
                x: 100.0,
                y: 200.0,
                velocity: 0.0,
            },
            pipes: Vec::new(),
            score: 0,
            game_over: false,
        };
        Ok(s)
    }

    fn reset(&mut self) {
        self.bird.y = 200.0;
        self.bird.velocity = 0.0;
        self.pipes.clear();
        self.score = 0;
        self.game_over = false;
    }

    fn update_pipes(&mut self) {
        if let Some(pipe) = self.pipes.first() {
            if pipe.x < -PIPE_WIDTH {
                self.pipes.remove(0);
                self.score += 1;
            }
        }

        let mut rng = rand::thread_rng();
        if let Some(pipe) = self.pipes.last() {
            if pipe.x < WINDOW_WIDTH - PIPE_SPACING {
                let height = rng.gen_range(50.0..400.0); // 调整高度范围以适应新窗口高度
                self.pipes.push(Pipe { x: WINDOW_WIDTH, height });
            }
        } else {
            let height = rng.gen_range(50.0..400.0); // 调整高度范围以适应新窗口高度
            self.pipes.push(Pipe { x: WINDOW_WIDTH, height });
        }

        for pipe in &mut self.pipes {
            pipe.x -= 2.0;
        }
    }

    fn check_collision(&self) -> bool {
        for pipe in &self.pipes {
            if self.bird.x + BIRD_WIDTH > pipe.x
                && self.bird.x < pipe.x + PIPE_WIDTH
                && (self.bird.y < pipe.height || self.bird.y + BIRD_HEIGHT > pipe.height + 150.0) // 调整障碍物间隙以适应新窗口高度
            {
                return true;
            }
        }
        self.bird.y > WINDOW_HEIGHT || self.bird.y < 0.0
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if !self.game_over {
            self.bird.velocity += GRAVITY;
            self.bird.y += self.bird.velocity;

            self.update_pipes();

            if self.check_collision() {
                self.game_over = true;
            }
        } else {
            if keyboard::is_key_pressed(ctx, KeyCode::Space) {
                self.reset();
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::from_rgb(135, 206, 250));

        let bird_rect = graphics::Rect::new(self.bird.x, self.bird.y, BIRD_WIDTH, BIRD_HEIGHT);
        let bird_mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), bird_rect, Color::from_rgb(255, 255, 0))?;
        graphics::draw(ctx, &bird_mesh, graphics::DrawParam::default())?;

        for pipe in &self.pipes {
            let top_pipe_rect = graphics::Rect::new(pipe.x, 0.0, PIPE_WIDTH, pipe.height);
            let bottom_pipe_rect = graphics::Rect::new(pipe.x, pipe.height + 150.0, PIPE_WIDTH, WINDOW_HEIGHT - pipe.height - 150.0); // 调整障碍物间隙以适应新窗口高度
            let pipe_mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), top_pipe_rect, Color::from_rgb(0, 255, 0))?;
            graphics::draw(ctx, &pipe_mesh, graphics::DrawParam::default())?;
            let pipe_mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), bottom_pipe_rect, Color::from_rgb(0, 255, 0))?;
            graphics::draw(ctx, &pipe_mesh, graphics::DrawParam::default())?;
        }

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
        if keycode == KeyCode::Space && !self.game_over {
            self.bird.velocity = JUMP_STRENGTH;
        }
    }
}

fn main() -> GameResult {
    let (mut ctx, event_loop) = ContextBuilder::new("flappy_bird", "author")
        .window_setup(ggez::conf::WindowSetup::default().title("Flappy Bird"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT)) // 设置窗口尺寸
        .build()?;
    let state = MainState::new()?;
    event::run(ctx, event_loop, state)
}
