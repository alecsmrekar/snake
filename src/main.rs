use rand::prelude::*;
use raylib::prelude::*;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

const SQUARE_SIZE: usize = 20;
const GAME_SIZE: usize = 20;
const FONT_SIZE: i32 = 20;
const PERIOD_CHANGE: f32 = 0.95;
const INITIAL_PERIOD: u64 = 300;

#[derive(Clone, PartialEq)]
struct GamePoint {
    x: usize,
    y: usize,
}

#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl GamePoint {
    fn random() -> GamePoint {
        let mut rng = rand::thread_rng();
        GamePoint {
            x: rng.gen_range(0..GAME_SIZE),
            y: rng.gen_range(0..GAME_SIZE),
        }
    }
    fn to_pixel(&self) -> (i32, i32) {
        let x: i32 = (self.x * SQUARE_SIZE).try_into().unwrap();
        let y: i32 = (self.y * SQUARE_SIZE).try_into().unwrap();
        (x, y)
    }
    fn matches(&self, other: &GamePoint) -> bool {
        self.x == other.x && self.y == other.y
    }
}

struct Food {
    position: GamePoint,
}

impl Food {
    fn new() -> Food {
        Food {
            position: GamePoint::random(),
        }
    }
    fn draw<'a>(&self, mut d: RaylibDrawHandle<'a>) -> RaylibDrawHandle<'a> {
        let (x, y) = self.position.to_pixel();
        d.draw_rectangle(x, y, SQUARE_SIZE as i32, SQUARE_SIZE as i32, Color::RED);
        d
    }
    fn mov(&mut self, _snake: &Snake) {
        // todo check to not place on snake.
        self.position = GamePoint::random();
    }
}

struct Snake {
    body: VecDeque<GamePoint>,
    direction: Direction,
}

impl Snake {
    fn new() -> Snake {
        Snake {
            body: VecDeque::from([GamePoint::random()]),
            direction: Direction::Up,
        }
    }
    fn get_head(&self) -> &GamePoint {
        self.body.front().unwrap()
    }
    fn mov(&mut self, food: &mut Food) -> Option<bool> {
        let mut new_head = self.get_head().clone();
        let limits = GAME_SIZE - 1;
        if self.direction == Direction::Up && new_head.y == 0 {
            return None;
        }
        if self.direction == Direction::Down && new_head.y == limits {
            return None;
        }
        if self.direction == Direction::Left && new_head.x == 0 {
            return None;
        }
        if self.direction == Direction::Right && new_head.x == limits {
            return None;
        }
        match self.direction {
            Direction::Up => new_head.y -= 1,
            Direction::Down => new_head.y += 1,
            Direction::Left => new_head.x -= 1,
            Direction::Right => new_head.x += 1,
        }

        // Check collision.
        let collisions = self
            .body
            .iter()
            .filter(|x| x.matches(&new_head))
            .collect::<Vec<&GamePoint>>();
        if !collisions.is_empty() {
            return None;
        }

        self.body.push_front(new_head.clone());
        let eaten = self.is_on_food(food);
        match eaten {
            true => food.mov(self),
            false => _ = self.body.pop_back(),
        }
        Some(eaten)
    }

    fn draw<'a>(&self, mut d: RaylibDrawHandle<'a>) -> RaylibDrawHandle<'a> {
        for segment in self.body.iter() {
            let (x, y) = segment.to_pixel();
            d.draw_rectangle(
                x,
                y,
                SQUARE_SIZE as i32,
                SQUARE_SIZE as i32,
                Color::ROYALBLUE,
            );
        }
        d
    }
    fn is_on_food(&self, food: &Food) -> bool {
        self.get_head().matches(&food.position)
    }
}

struct GameState {
    snake: Snake,
    food: Food,
    time: Instant,
    period: Duration,
    finished: bool,
}

fn main() {
    let window_size: i32 = (SQUARE_SIZE * GAME_SIZE).try_into().unwrap();
    let (mut rl, thread) = raylib::init()
        .size(window_size, window_size)
        .title("Hello, Vito")
        .build();
    rl.set_target_fps(60);
    let mut game_state = GameState {
        finished: false,
        snake: Snake::new(),
        food: Food::new(),
        time: Instant::now(),
        period: Duration::from_millis(INITIAL_PERIOD),
    };

    while !rl.window_should_close() {
        if !game_state.finished {
            run_loop(&mut game_state, &mut rl, &thread);
        }
        if game_state.finished {
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::WHITE);
            d.draw_text("GAME OVER", 12, 12, FONT_SIZE, Color::BLACK);
            d.draw_text(
                format!("Score: {}", game_state.snake.body.len()).as_str(),
                12,
                30,
                FONT_SIZE,
                Color::BLACK,
            );
        }
    }
}

fn run_loop(state: &mut GameState, rl: &mut RaylibHandle, thread: &RaylibThread) {
    if state.time.elapsed() > state.period {
        state.time = Instant::now();
        match state.snake.mov(&mut state.food) {
            Some(eaten) => {
                if eaten {
                    state.period = state.period.mul_f32(PERIOD_CHANGE);
                }
            }
            None => {
                state.finished = true;
                return;
            }
        };
    }
    let mut d = rl.begin_drawing(thread);
    if d.is_key_down(KeyboardKey::KEY_DOWN) && state.snake.direction != Direction::Up {
        state.snake.direction = Direction::Down;
    } else if d.is_key_down(KeyboardKey::KEY_UP) && state.snake.direction != Direction::Down {
        state.snake.direction = Direction::Up;
    } else if d.is_key_down(KeyboardKey::KEY_LEFT) && state.snake.direction != Direction::Right {
        state.snake.direction = Direction::Left;
    } else if d.is_key_down(KeyboardKey::KEY_RIGHT) && state.snake.direction != Direction::Left {
        state.snake.direction = Direction::Right;
    }
    d.clear_background(Color::WHITE);
    d.draw_text(
        format!("FPS: {}", d.get_fps()).as_str(),
        12,
        12,
        FONT_SIZE,
        Color::BLACK,
    );
    d.draw_text(
        format!("Score: {}", state.snake.body.len()).as_str(),
        12,
        30,
        FONT_SIZE,
        Color::BLACK,
    );
    d = state.food.draw(d);
    _ = state.snake.draw(d);
}
