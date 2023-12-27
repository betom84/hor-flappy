#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]

use bracket_lib::prelude::*;

struct State {
    mode: GameMode,
    player: Player,
    obstacle: Obstacle,
    score: i32,
    frame_time: f32,
}

impl State {
    fn new() -> Self {
        Self {
            mode: GameMode::Menu,
            player: Player::new(5, 25),
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
            score: 0,
            frame_time: 0.0,
        }
    }

    fn menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();

        ctx.print_centered(20, "Welcome to Flappy Dragon");
        ctx.print_centered(22, "(P) Play");
        ctx.print_centered(24, "(Q) Quit");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.start(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn start(&mut self) {
        self.mode = GameMode::Playing;
        self.player = Player::new(5, 25);
        self.frame_time= 0.0;
        self.score = 0;
        self.obstacle = Obstacle::new(SCREEN_WIDTH, self.score);
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);

        // engine "ticks" as fast as possible.. 
        self.frame_time += ctx.frame_time_ms;

        // ..but we only want to "change the world" every 75ms
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.player.move_and_gravity();
        }

        // input gets processed as fast as possible
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }

        self.player.render(ctx);

        self.obstacle.render(ctx, self.player.x);
        if self.player.x > self.obstacle.x {
            self.score += 1;
            self.obstacle = Obstacle::new(self.player.x + SCREEN_WIDTH, self.score);
        }

        ctx.print(0, 0, "Press SPACE to flap.");
        ctx.print_right(SCREEN_WIDTH, 0, format!("Score: {}", self.score));

        if self.player.y > SCREEN_HEIGHT || self.obstacle.collision(&self.player) {
            self.mode = GameMode::End;
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();

        ctx.print_centered(20, "You're dead!");
        ctx.print_centered(22, "(P) Play again");
        ctx.print_centered(24, "(Q) Quit");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.start(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.menu(ctx),
            GameMode::Playing => self.play(ctx),
            GameMode::End => self.dead(ctx),
        }
    }
}

enum GameMode {
    Menu,
    Playing,
    End,
}

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 75.0;

struct Player {
    x: i32,
    y: i32,
    velocity: f32,
}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Player {
            x,
            y,
            velocity: 0.0,
        }
    }

    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set(0, self.y, YELLOW, BLACK, to_cp437('@'));
    }

    fn move_and_gravity(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 0.2;
        }

        self.y += self.velocity as i32;
        self.x += 1;

        if self.y < 0 {
            self.y = 0;
        }
    }

    fn flap(&mut self) {
        self.velocity = -2.0;
    }
}

struct Obstacle {
    x: i32,
    gap_y: i32,
    size: i32,
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();

        Obstacle{
            x,
            gap_y:  random.range(10, SCREEN_HEIGHT - 10),
            size: i32::max(2, 20 - score),
        }
    }

    fn render(&self, ctx: &mut BTerm, player_x: i32) {
        let screen_x = self.x - player_x;

        for y in 0..(self.gap_y - (self.size / 2)) {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));
        }

        for y in (self.gap_y + (self.size / 2))..SCREEN_HEIGHT {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));
        }
    }

    fn collision(&self, player: &Player) -> bool {
        (self.x == player.x) &&
        ((player.y < (self.gap_y - (self.size / 2))) ||
        (player.y > (self.gap_y + (self.size / 2))))
    }
}

fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("Flappy Dragon")
        .build()?;

    main_loop(context, State::new())
}
