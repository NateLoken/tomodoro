use std::{
    io::{Write, stdout},
    time::Duration,
};

use tokio::time;

#[derive(Debug)]
pub struct Timer {
    name: String,
    start: u64,
    time_unit: TimeUnit,
}

#[derive(Debug)]
pub enum TimeUnit {
    SECONDS,
    MINUTES,
    HOURS,
}

impl Timer {
    pub fn new(name: String, start: u64, time_unit: TimeUnit) -> Self {
        Timer {
            name: name,
            start: start,
            time_unit: time_unit,
        }
    }

    fn to_seconds(&self) -> u64 {
        match self.time_unit {
            TimeUnit::HOURS => self.start * 60 * 60,
            TimeUnit::MINUTES => self.start * 60,
            TimeUnit::SECONDS => self.start,
        }
    }

    pub async fn start(&mut self) {
        let mut task_interval = time::interval(Duration::from_secs(1));
        let seconds = self.to_seconds();

        for i in (0..=seconds).rev() {
            task_interval.tick().await;
            print!("\r{:02}:{:02}", i / 60, i % 60);
            stdout().flush().unwrap();
        }
    }
}
