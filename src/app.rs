use std::sync::mpsc;

use color_eyre::eyre::{Result, eyre};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Gauge, Paragraph, Widget},
};

use crate::config::{PhaseConfig, PhasePreset};
use crate::timer::{TimerCommand, TimerEvent, TimerSnapshot};

pub enum Event {
    Input(KeyEvent),
    Timer(TimerEvent),
}

pub struct App {
    exit: bool,
    paused: bool,
    prog_bar_color: Color,
    timer_progress: f64,
    phase_name: String,
    remaining_secs: f64,
    total_secs: f64,
}

impl App {
    pub fn new() -> Self {
        Self {
            exit: false,
            paused: false,
            prog_bar_color: Color::Red,
            timer_progress: 0.0,
            phase_name: String::from("Work"),
            remaining_secs: 0.0,
            total_secs: 0.0,
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: mpsc::Receiver<Event>,
        timer_tx: mpsc::Sender<TimerCommand>,
    ) -> Result<()> {
        let config = PhaseConfig::new()?;

        if config.phases.is_empty() {
            return Err(eyre!("config loaded but contains no phases"));
        }

        let mut phase_index: usize = 0;

        self.apply_phase(&config.phases[phase_index]);
        timer_tx.send(TimerCommand::Start(config.phases[phase_index].to_spec()))?;

        while !self.exit {
            let event = match rx.recv() {
                Ok(event) => event,
                Err(_) => break,
            };

            match event {
                Event::Input(key_event) => {
                    if let Some(cmd) = self.handle_key_event(key_event) {
                        timer_tx.send(cmd)?;
                    }
                }
                Event::Timer(timer_event) => {
                    self.handle_timer_event(
                        timer_event,
                        &timer_tx,
                        &config.phases,
                        &mut phase_index,
                    )?;
                }
            }

            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    pub fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Option<TimerCommand> {
        if key_event.kind != KeyEventKind::Press {
            return None;
        }

        match key_event.code {
            KeyCode::Char('q') => {
                self.exit = true;
                Some(TimerCommand::Stop)
            }
            KeyCode::Char(' ') => {
                self.paused = !self.paused;
                if self.paused {
                    Some(TimerCommand::Pause)
                } else {
                    Some(TimerCommand::Resume)
                }
            }
            KeyCode::Char('n') => Some(TimerCommand::Skip),
            _ => None,
        }
    }

    fn handle_timer_event(
        &mut self,
        event: TimerEvent,
        timer_tx: &mpsc::Sender<TimerCommand>,
        phases: &[PhasePreset],
        phase_index: &mut usize,
    ) -> Result<()> {
        match event {
            TimerEvent::Tick(snapshot) => self.apply_snapshot(&snapshot),
            TimerEvent::Completed(snapshot) => {
                self.apply_snapshot(&snapshot);
                self.start_new_timer(timer_tx, phases, phase_index)?;
            }
            TimerEvent::Stopped => {
                self.paused = false;
                self.timer_progress = 0.0;
                self.remaining_secs = 0.0;
            }
        }

        Ok(())
    }

    fn start_new_timer(
        &mut self,
        tx: &mpsc::Sender<TimerCommand>,
        phases: &[PhasePreset],
        phase_index: &mut usize,
    ) -> Result<()> {
        *phase_index = (*phase_index + 1) % phases.len();
        self.apply_phase(&phases[*phase_index]);
        tx.send(TimerCommand::Start(phases[*phase_index].to_spec()))?;

        Ok(())
    }

    fn apply_phase(&mut self, phase: &PhasePreset) {
        self.phase_name = phase.name.clone();
        self.total_secs = phase.total_seconds();
        self.remaining_secs = self.total_secs;
        self.timer_progress = 0.0;
        self.paused = false;
        self.prog_bar_color = phase.color;
    }

    fn apply_snapshot(&mut self, snapshot: &TimerSnapshot) {
        self.phase_name = snapshot.name.clone();
        self.total_secs = snapshot.total_secs;
        self.remaining_secs = snapshot.remaining_secs;
        self.timer_progress = snapshot.progress;
        self.paused = snapshot.paused;
    }

    fn format_time(secs: f64) -> String {
        let total = secs.max(0.0).round() as u64;
        format!("{:02}:{:02}", total / 60, total % 60)
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let vertical_layout = Layout::vertical([Constraint::Length(5), Constraint::Min(3)]);
        let [title_area, gauge_area] = vertical_layout.areas(area);

        let timer_block = Block::bordered()
            .title(Line::from("Timer Overview").centered())
            .border_set(border::ROUNDED);

        let state_label = if self.paused { "Paused" } else { "Running" };

        let elapsed = App::format_time(self.total_secs - self.remaining_secs);
        let total = App::format_time(self.total_secs);
        let time_line = if title_area.width < 24 {
            format!("{elapsed}/{total}")
        } else {
            format!("Time: {elapsed} / {total}")
        };

        let overview = Paragraph::new(vec![
            Line::from(format!("Phase: {}", self.phase_name)).centered(),
            Line::from(format!("State: {}", state_label)).centered(),
            Line::from(time_line).centered(),
        ])
        .block(timer_block);

        overview.render(title_area, buf);

        let instructions = Line::from(vec![
            " Pause/Resume ".into(),
            "<Space>".white().bold(),
            " Next ".into(),
            "<N>".white().bold(),
            " Quit ".into(),
            "<Q>".white().bold(),
        ])
        .centered();

        let block = Block::bordered()
            .title(Line::from(" Timer Progress ").centered())
            .title_bottom(instructions)
            .border_set(border::ROUNDED);

        let gauge_ratio = if self.timer_progress >= 1.0 {
            1.0 - f64::EPSILON
        } else {
            self.timer_progress.max(0.0)
        };

        let progress_bar = Gauge::default()
            .gauge_style(Style::default().fg(self.prog_bar_color))
            .block(block)
            .label(format!("Timer: {:.2}%", self.timer_progress * 100_f64))
            .ratio(gauge_ratio);

        progress_bar.render(
            Rect {
                x: gauge_area.left(),
                y: gauge_area.top(),
                width: gauge_area.width,
                height: 3,
            },
            buf,
        );
    }
}
