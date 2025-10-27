use std::{
    io::{self, Write, stdout},
    time::Duration,
};

use crate::timer::Timer;

mod timer;

/*
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::{
    io, thread,
    time::{Duration, Instant, SystemTime},
};
use tui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
};

fn ui<B: Backend>(frame: &mut Frame<B>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(frame.size());
    let block = Block::default().title("Block 1").borders(Borders::ALL);
    frame.render_widget(block, chunks[0]);
    let block = Block::default().title("Block 2").borders(Borders::ALL);
    frame.render_widget(block, chunks[1]);
    let block = Block::default().title("Block 3").borders(Borders::ALL);
    frame.render_widget(block, chunks[2]);
}
*/

#[tokio::main]
async fn main() {
    /*
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|f| {
            ui(f);
        })?;

        let current_time = Instant::now();
        let tick_rate = Duration::from_millis(200);

        thread::sleep(Duration::from_millis(1000));



        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if current_time.elapsed() >= tick_rate {
            print!("{}", current_time.elapsed().as_secs_f64());
        }
    */
    println!("Countdown Timer is starting...");
    println!("Please enter countdown name:");
    stdout().flush().unwrap();

    let mut name = String::new();
    io::stdin()
        .read_line(&mut name)
        .expect("Failed to read name");

    println!("Please enter countdown start:");
    stdout().flush().unwrap();
    let mut start_input = String::new();
    io::stdin()
        .read_line(&mut start_input)
        .expect("Failed to read start");

    let start: u64 = match start_input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Please enter a valid integer.");
            return;
        }
    };

    let mut timer = Timer::new(name.trim().to_string(), start, timer::TimeUnit::MINUTES);
    println!("Timer: {:?}", timer);

    timer.start().await;
}
