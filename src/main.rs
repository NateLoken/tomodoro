mod app;
mod timer;

use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use app::{App, Event};
use color_eyre::eyre::Result;
use crossterm::event::{self, Event as CrosstermEvent};
use timer::TimerCommand;

use crate::timer::{TimerEngine, TimerEvent};

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();

    let (app_evt_tx, app_evt_rx) = mpsc::channel::<Event>();

    let (timer_cmd_tx, timer_cmd_rx) = mpsc::channel::<TimerCommand>();

    let tx_input_events = app_evt_tx.clone();
    let tx_timer_events = app_evt_tx.clone();

    let mut app = App::new();

    thread::spawn(move || handle_input_events(tx_input_events));
    thread::spawn(move || timer_worker(timer_cmd_rx, tx_timer_events));

    let app_result = app.run(&mut terminal, app_evt_rx, timer_cmd_tx);

    ratatui::restore();

    app_result
}

fn handle_input_events(tx: mpsc::Sender<Event>) {
    loop {
        match event::read() {
            Ok(CrosstermEvent::Key(key_event)) => {
                if tx.send(Event::Input(key_event)).is_err() {
                    break;
                }
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }
}

fn timer_worker(rx: mpsc::Receiver<TimerCommand>, tx: mpsc::Sender<Event>) {
    let mut engine = TimerEngine::default();
    let tick_rate = Duration::from_millis(50);
    loop {
        match rx.recv_timeout(tick_rate) {
            Ok(cmd) => match cmd {
                TimerCommand::Start(spec) => {
                    let snap = engine.start(spec);
                    if tx.send(Event::Timer(TimerEvent::Tick(snap))).is_err() {
                        break;
                    }
                }
                TimerCommand::Pause => {
                    if let Some(snap) = engine.pause()
                        && tx.send(Event::Timer(TimerEvent::Tick(snap))).is_err()
                    {
                        break;
                    }
                }
                TimerCommand::Resume => {
                    if let Some(snap) = engine.resume()
                        && tx.send(Event::Timer(TimerEvent::Tick(snap))).is_err()
                    {
                        break;
                    }
                }
                TimerCommand::Skip => {
                    if let Some(snap) = engine.skip()
                        && tx.send(Event::Timer(TimerEvent::Completed(snap))).is_err()
                    {
                        break;
                    }
                }
                TimerCommand::Stop => {
                    engine.stop();
                    if tx.send(Event::Timer(TimerEvent::Stopped)).is_err() {
                        break;
                    }
                }
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Some(timer_event) = engine.tick(Instant::now())
                    && tx.send(Event::Timer(timer_event)).is_err()
                {
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
}
