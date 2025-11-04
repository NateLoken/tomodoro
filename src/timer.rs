use std::{sync::mpsc, thread, time::Duration};

use crate::app::Event;

#[derive(Debug, Clone, Copy)]
pub enum TimeUnit {
    SECONDS,
    MINUTES,
    HOURS,
}

#[derive(Debug)]
pub struct Timer {
    name: String,
    duration: f64,
    elapsed: f64,
    unit: TimeUnit,
    progress: f64,
    paused: bool,
}

impl Timer {
    pub fn new(name: String, dur: f64, time_unit: TimeUnit) -> Self {
        Self {
            name,
            duration: dur,
            elapsed: 0.0,
            unit: time_unit,
            progress: 0.0,
            paused: false,
        }
    }

    pub fn run(&mut self, tx: mpsc::Sender<Event>) {
        let tick_rate = Duration::from_millis(10);
        let seconds = self.as_seconds();

        while self.elapsed < seconds {
            if !self.paused {
                self.elapsed = (self.elapsed + 0.01_f64).min(seconds);
                self.progress = (self.elapsed / seconds).clamp(0.0, 1.0);
                let remaining = (seconds - self.elapsed).max(0.0);
                tx.send(Event::Progress {
                    progress: self.progress,
                    remaining_secs: remaining,
                })
                .unwrap();
                thread::sleep(tick_rate);
            }
        }
    }

    pub fn pause(&mut self) {
       self.paused = true; 
    }


    pub fn resume(&mut self) {
       self.paused = false; 
    }

    pub fn seconds_from(duration: f64, unit: TimeUnit) -> f64 {
        match unit {
            TimeUnit::SECONDS => duration,
            TimeUnit::MINUTES => duration * 60.0,
            TimeUnit::HOURS => duration * 60.0 * 60.0,
        }
    }

    fn as_seconds(&self) -> f64 {
        Self::seconds_from(self.duration, self.unit)
    }
}
