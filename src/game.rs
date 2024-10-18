use crossterm::{
    cursor,
    event::{self, KeyCode, KeyEvent},
    execute,
    terminal::{self, ClearType},
};
use rand::Rng;
use std::io::{self, Write};
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
struct Point {
    x: u16,
    y: u16,
}

struct FallingItem {
    position: Point,
    caught: bool,
}

struct CatchingGame {
    player: Point,
    falling_items: Vec<FallingItem>,
    score: u32,
    misses: u32,
    width: u16,
    height: u16,
    game_over: bool,
    game_time: Duration,
    start_time: Instant,
    target_score: u32,
    game_result: Option<String>, // Store the game result
}

impl CatchingGame {
    fn new(width: u16, height: u16, target_score: u32, game_time: Duration) -> Self {
        CatchingGame {
            player: Point {
                x: width / 2,
                y: height - 1,
            },
            falling_items: vec![],
            score: 0,
            misses: 0,
            width,
            height,
            game_over: false,
            game_time,
            start_time: Instant::now(),
            target_score,
            game_result: None, // Initialize with None
        }
    }

    fn run(&mut self) -> io::Result<Option<String>> {
        // Change return type to Option<String>
        terminal::enable_raw_mode()?; // Enter raw mode
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        let mut last_instant = Instant::now();
        let mut item_timer = Instant::now();

        while !self.game_over {
            if last_instant.elapsed() > Duration::from_millis(100) {
                self.update()?;
                self.draw(&mut stdout)?;
                last_instant = Instant::now();
            }

            // Spawn new falling items every second
            if item_timer.elapsed() > Duration::from_secs(1) {
                self.spawn_item();
                item_timer = Instant::now();
            }

            // Check for user input (to move the player)
            if event::poll(Duration::from_millis(1))? {
                if let event::Event::Key(KeyEvent {
                    code: KeyCode::Left,
                    ..
                }) = event::read()?
                {
                    if self.player.x > 0 {
                        self.player.x -= 1;
                    }
                } else if let event::Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    ..
                }) = event::read()?
                {
                    if self.player.x < self.width - 1 {
                        self.player.x += 1;
                    }
                }
            }
        }

        terminal::disable_raw_mode()?; // Exit raw mode

        // Determine game result
        if self.score >= self.target_score {
            self.game_result = Some("You Won! Caught enough items!".to_string());
        } else {
            self.game_result = Some("Game Over! Missed too many items.".to_string());
        }

        Ok(self.game_result.clone()) // Return the game result
    }

    fn update(&mut self) -> io::Result<()> {
        // Check if game time has expired
        if self.start_time.elapsed() >= self.game_time {
            self.game_over = true;
            return Ok(());
        }

        for item in &mut self.falling_items {
            item.position.y += 1; // Move items down

            // Check for catching items
            if item.position.y == self.player.y && item.position.x == self.player.x {
                item.caught = true;
                self.score += 1;

                // Check for success condition
                if self.score >= self.target_score {
                    self.game_over = true; // Success condition met
                }
            } else if item.position.y >= self.height {
                self.misses += 1;
                item.caught = true; // Mark as caught even if missed
            }
        }

        // Remove caught or missed items
        self.falling_items.retain(|item| !item.caught);

        // Check failure condition
        if self.misses >= 3 {
            self.game_over = true;
        }

        Ok(())
    }

    fn draw<W: Write>(&self, stdout: &mut W) -> io::Result<()> {
        execute!(stdout, terminal::Clear(ClearType::All))?;
        execute!(stdout, cursor::DisableBlinking)?;
        // Draw player
        execute!(stdout, cursor::MoveTo(self.player.x, self.player.y),)?;
        // Draw falling items
        for item in &self.falling_items {
            execute!(stdout, cursor::MoveTo(item.position.x, item.position.y),)?;
        }
        // Display score and misses
        execute!(stdout, cursor::MoveTo(0, 0),)?;

        if self.game_over {
            execute!(stdout, cursor::MoveTo(0, self.height / 2),)?;
        }

        stdout.flush()?;
        Ok(())
    }

    fn spawn_item(&mut self) {
        let mut rng = rand::thread_rng();
        let x_position = rng.gen_range(0..self.width);
        let new_item = FallingItem {
            position: Point {
                x: x_position,
                y: 0,
            },
            caught: false,
        };
        self.falling_items.push(new_item);
    }
}

// To fix
pub fn play_game() -> io::Result<()> {
    let target_score = 10; // Number of items to catch to win
    let game_time = Duration::new(30, 0); // Game time limit (30 seconds)
    let mut game = CatchingGame::new(20, 10, target_score, game_time); // Initialize with width and height
                                                                       // let result = game.run()?; // Start the game and get the result

    // Display the result after the game ends
    /* if let Some(message) = result {
        if message.contains("Won") {
            return Ok(());
        }
    }

    Err(io::Error::new(io::ErrorKind::Other, "Game Over!")) */
    Ok(())
}
