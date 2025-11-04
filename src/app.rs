use std::sync::mpsc;

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Gauge, Paragraph, Widget},
};

use crate::timer::{TimeUnit, Timer};

#[derive(Debug, Clone)]
pub struct TimerType {
    pub name: String,
    pub duration: f64,
    pub unit: TimeUnit,
    pub color: Color,
}

impl TimerType {
    pub fn total_seconds(&self) -> f64 {
        Timer::seconds_from(self.duration, self.unit)
    }
}

pub enum TimerCommand {
    Phase(TimerType),
}

pub enum Event {
    Input(KeyEvent),
    Progress { progress: f64, remaining_secs: f64 },
    TimerDone(bool),
}

pub struct App {
    exit: bool,
    prog_bar_color: Color,
    timer_progress: f64,
    timer_name: String,
    remaining_secs: f64,
    total_secs: f64,
}

impl App {
    pub fn new() -> Self {
        Self {
            exit: false,
            prog_bar_color: Color::Red,
            timer_progress: 0.0,
            timer_name: String::from("Work"),
            remaining_secs: 0.0,
            total_secs: 0.0,
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: mpsc::Receiver<Event>,
        phase_tx: mpsc::Sender<TimerCommand>,
    ) -> Result<()> {
        let mut phase_index: usize = 0;
        let phases = vec![
            TimerType {
                name: String::from("Work"),
                duration: 25.0,
                unit: TimeUnit::MINUTES,
                color: Color::Red,
            },
            TimerType {
                name: String::from("Rest"),
                duration: 5.0,
                unit: TimeUnit::MINUTES,
                color: Color::Blue,
            },
        ];

        self.apply_phase(&phases[phase_index]);
        phase_tx
            .send(TimerCommand::Phase(phases[phase_index].clone()))
            .unwrap();

        while !self.exit {
            match rx.recv().expect("Thread panicked.") {
                Event::Input(key_event) => self.handle_key_event(key_event)?,
                Event::Progress {
                    progress,
                    remaining_secs,
                } => {
                    self.timer_progress = progress;
                    self.remaining_secs = remaining_secs;
                }
                Event::TimerDone(_) => {
                    self.start_new_timer(phase_tx.clone(), &phases, &mut phase_index);
                }
            }
            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    pub fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Char('q') => self.exit = true,
                _ => {}
            }
        }
        Ok(())
    }

    fn start_new_timer(
        &mut self,
        tx: mpsc::Sender<TimerCommand>,
        phases: &[TimerType],
        phase_index: &mut usize,
    ) {
        *phase_index = (*phase_index + 1) % phases.len();
        self.apply_phase(&phases[*phase_index]);

        tx.send(TimerCommand::Phase(phases[*phase_index].clone()))
            .unwrap();
    }

    fn apply_phase(&mut self, phase: &TimerType) {
        self.timer_name = phase.name.clone();
        self.total_secs = phase.total_seconds();
        self.remaining_secs = self.total_secs;
        self.timer_progress = 0.0;
        self.prog_bar_color = phase.color;
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
        let vertical_layout =
            Layout::vertical([Constraint::Percentage(10), Constraint::Percentage(90)]);
        let [title_area, gauge_area] = vertical_layout.areas(area);

        let timer_block = Block::bordered()
            .title(Line::from("Timer Overview").centered())
            .border_set(border::THICK)
            .border_set(border::ROUNDED);

        let overview = Paragraph::new(vec![
            Line::from(format!("Phase: {}", self.timer_name)).centered(),
            Line::from(format!(
                "Time: {} / {}",
                App::format_time(self.total_secs - self.remaining_secs),
                App::format_time(self.total_secs)
            ))
            .centered(),
        ])
        .block(timer_block);

        overview.render(title_area, buf);

        let instructions = Line::from(vec![
            " Pause ".into(),
            "<Space>".white().bold(),
            " Resume ".into(),
            "<R>".white().bold(),
            " Quit ".into(),
            "<Q>".white().bold(),
        ])
        .centered();

        let block = Block::bordered()
            .title(Line::from(" Timer Progess ").centered())
            .title_bottom(instructions)
            .border_set(border::THICK)
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
