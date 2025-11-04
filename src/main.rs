mod app;
mod timer;

use std::{sync::mpsc, thread};

use app::{App, Event, TimerCommand};
use color_eyre::eyre::Result;
use crossterm::event;
use timer::Timer;

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();

    let (event_tx, event_rx) = mpsc::channel::<Event>();
    let tx_input_events = event_tx.clone();
    let tx_timer_progress = event_tx.clone();

    let (timer_event_tx, timer_event_rx) = mpsc::channel::<TimerCommand>();

    let mut app = App::new();

    thread::spawn(move || handle_input_events(tx_input_events));
    thread::spawn(move || timer_worker(timer_event_rx, tx_timer_progress));

    let app_result = app.run(&mut terminal, event_rx, timer_event_tx);

    ratatui::restore();

    app_result
}

fn handle_input_events(tx: mpsc::Sender<Event>) {
    loop {
        match event::read().unwrap() {
            event::Event::Key(key_event) => tx.send(Event::Input(key_event)).unwrap(),
            _ => {}
        }
    }
}

fn timer_worker(rx: mpsc::Receiver<TimerCommand>, tx: mpsc::Sender<Event>) {
    while let Ok(cmd) = rx.recv() {
        match cmd {
            TimerCommand::Phase(phase) => {
                let mut timer = Timer::new(phase.name, phase.duration, phase.unit);
                timer.run(tx.clone());
                tx.send(Event::TimerDone(true)).unwrap();
            }
        }
    }
}
