use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics::{self, Color};
use ggez::{Context, ContextBuilder, GameResult};
use rand::Rng;
use std::net::TcpStream;
use std::io::{Read, Write};
use std::time::Instant;
use std::time::Duration;

const BIRD_WIDTH: f32 = 34.0;
const BIRD_HEIGHT: f32 = 24.0;
const PIPE_WIDTH: f32 = 52.0;
const PIPE_SPACING: f32 = 400.0;
const GRAVITY: f32 = 0.1;
const JUMP_STRENGTH: f32 = -4.0;
const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 800.0;

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
    stream: TcpStream,
    result: Option<String>,
    client_id: u32,
    last_score_update: Instant,
}

impl MainState {
    fn new(client_id: u32) -> GameResult<MainState> {
        let stream = TcpStream::connect("192.168.3.10:7878").unwrap();
        let mut s = MainState {
            bird: Bird {
                x: 100.0,
                y: 200.0,
                velocity: 0.0,
            },
            pipes: Vec::new(),
            score: 0,
            game_over: false,
            stream,
            result: None,
            client_id,
            last_score_update: Instant::now(),
        };

        s.stream.write(&client_id.to_be_bytes()).unwrap();
        println!("Sent client ID to server: {}", client_id);

        Ok(s)
    }


    fn update_pipes(&mut self) {
        let mut rng = rand::thread_rng();
        
        if self.pipes.is_empty() || self.pipes.last().unwrap().x < WINDOW_WIDTH - PIPE_SPACING {
            let height = rng.gen_range(50.0..400.0);
            self.pipes.push(Pipe { x: WINDOW_WIDTH, height });
        }

        for pipe in &mut self.pipes {
            pipe.x -= 2.0; // 向左移动管道
        }

        if let Some(first_pipe) = self.pipes.first() {
            if first_pipe.x < -PIPE_WIDTH {
                self.pipes.remove(0); // 移除超出窗口的管道
            }
        }
    }

    fn check_collision(&self) -> bool {
        for pipe in &self.pipes {
            if self.bird.x + BIRD_WIDTH > pipe.x
                && self.bird.x < pipe.x + PIPE_WIDTH
                && (self.bird.y < pipe.height || self.bird.y + BIRD_HEIGHT > pipe.height + 150.0)
            {
                return true;
            }
        }
        self.bird.y > WINDOW_HEIGHT || self.bird.y < 0.0
    }

    fn handle_game_over(&mut self) {
        self.stream.write(&self.score.to_be_bytes()).unwrap();
        println!("Sent score to server: {}", self.score);
    
        let mut buffer = [0; 128];
        let result = loop {
            match self.stream.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let result_str = String::from_utf8_lossy(&buffer[..n]).to_string();
                    println!("Received raw result from server: {}", result_str);
                    break Some(result_str);
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to read from stream: {}", e);
                    break None;
                }
            }
        };
        self.result = result;
        if let Some(ref result) = self.result {
            println!("Received result from server: {}", result);
        }
    
        if let Err(e) = self.stream.shutdown(std::net::Shutdown::Both) {
            eprintln!("Failed to shutdown stream: {}", e);
        }
    
        self.game_over = true;
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if !self.game_over {
            self.bird.velocity += GRAVITY;
            self.bird.y += self.bird.velocity;

            self.update_pipes();

            if self.last_score_update.elapsed() >= Duration::from_secs(1) {
                self.score += 1;
                self.last_score_update = Instant::now();
                println!("Score: {}", self.score);
            }

            if self.check_collision() {
                self.handle_game_over();
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
            let bottom_pipe_rect = graphics::Rect::new(pipe.x, pipe.height + 150.0, PIPE_WIDTH, WINDOW_HEIGHT - pipe.height - 150.0);
            let pipe_mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), top_pipe_rect, Color::from_rgb(0, 255, 0))?;
            let bottom_pipe_mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), bottom_pipe_rect, Color::from_rgb(0, 255, 0))?;
            graphics::draw(ctx, &pipe_mesh, graphics::DrawParam::default())?;
            graphics::draw(ctx, &bottom_pipe_mesh, graphics::DrawParam::default())?;
        }

        let score_display = graphics::Text::new((self.score.to_string(), graphics::Font::default(), 24.0));
        graphics::draw(ctx, &score_display, (ggez::mint::Point2 { x: 10.0, y: 10.0 }, 0.0, Color::from_rgb(0, 0, 0)))?;

        if self.game_over {
            let game_over_text = if let Some(ref result) = self.result {
                format!("Game Over: {}\n", result)
            } else {
                "Game Over\n".to_string()
            };
            let game_over_display = graphics::Text::new((game_over_text, graphics::Font::default(), 24.0));
            graphics::draw(ctx, &game_over_display, (ggez::mint::Point2 { x: 200.0, y: 200.0 }, 0.0, Color::from_rgb(0, 0, 0)))?;
        }

        graphics::present(ctx)?;

        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
        match keycode {
            KeyCode::Space => {
                if !self.game_over {
                    self.bird.velocity = JUMP_STRENGTH;
                }
            }
          
            KeyCode::Escape => event::quit(ctx),
            _ => (),
        }
    }
}

fn main() -> GameResult {
    let client_id = 1; 
    let (ctx, event_loop) = ContextBuilder::new("flappy_bird2", "Author Name")
        .window_setup(ggez::conf::WindowSetup::default().title("Flappy Bird2"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build()
        .unwrap();

    let state = MainState::new(client_id)?;
    event::run(ctx, event_loop, state)
}
