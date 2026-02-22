use std::time::Instant;

#[derive(Debug, Clone, Copy)]
pub enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
}

impl TimeUnit {
    pub fn to_seconds(self, value: f64) -> f64 {
        match self {
            Self::Seconds => value,
            Self::Minutes => value * 60.0,
            Self::Hours => value * 60.0 * 60.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PhaseSpec {
    pub name: String,
    pub total_secs: f64,
}

impl PhaseSpec {
    pub fn new(name: String, duration: f64, unit: TimeUnit) -> Self {
        Self {
            name,
            total_secs: unit.to_seconds(duration),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TimerCommand {
    Start(PhaseSpec),
    Pause,
    Resume,
    Stop,
    Skip,
}

#[derive(Debug, Clone)]
pub struct TimerSnapshot {
    pub name: String,
    pub total_secs: f64,
    pub elapsed_secs: f64,
    pub remaining_secs: f64,
    pub progress: f64,
    pub paused: bool,
}

#[derive(Debug, Clone)]
pub enum TimerEvent {
    Tick(TimerSnapshot),
    Completed(TimerSnapshot),
    Stopped,
}

#[derive(Debug, Default)]
pub struct TimerEngine {
    active: Option<ActiveTimer>,
}

#[derive(Debug)]
struct ActiveTimer {
    spec: PhaseSpec,
    elapsed_secs: f64,
    paused: bool,
    last_tick: Instant,
}

impl TimerEngine {
    pub fn start(&mut self, spec: PhaseSpec) -> TimerSnapshot {
        self.start_at(spec, Instant::now())
    }

    fn start_at(&mut self, spec: PhaseSpec, now: Instant) -> TimerSnapshot {
        let active = ActiveTimer::new(spec, now);
        let snapshot = active.snapshot();
        self.active = Some(active);
        snapshot
    }

    pub fn pause(&mut self) -> Option<TimerSnapshot> {
        let active = self.active.as_mut()?;
        active.pause();
        Some(active.snapshot())
    }

    pub fn resume(&mut self) -> Option<TimerSnapshot> {
        self.resume_at(Instant::now())
    }

    fn resume_at(&mut self, now: Instant) -> Option<TimerSnapshot> {
        let active = self.active.as_mut()?;
        active.resume(now);
        Some(active.snapshot())
    }

    pub fn stop(&mut self) {
        self.active = None;
    }

    pub fn skip(&mut self) -> Option<TimerSnapshot> {
        let active = self.active.as_mut()?;
        active.elapsed_secs = active.spec.total_secs;
        Some(active.snapshot())
    }

    pub fn tick(&mut self, now: Instant) -> Option<TimerEvent> {
        let active = self.active.as_mut()?;
        if active.paused {
            active.last_tick = now;
            return None;
        }

        let dt = (now - active.last_tick).as_secs_f64();
        active.last_tick = now;

        if dt <= 0.0 {
            return None;
        }

        active.elapsed_secs = (active.elapsed_secs + dt).min(active.spec.total_secs);
        let snapshot = active.snapshot();

        if active.elapsed_secs >= active.spec.total_secs {
            self.active = None;
            Some(TimerEvent::Completed(snapshot))
        } else {
            Some(TimerEvent::Tick(snapshot))
        }
    }
}

impl ActiveTimer {
    pub fn new(spec: PhaseSpec, now: Instant) -> Self {
        Self {
            spec,
            elapsed_secs: 0.0,
            paused: false,
            last_tick: now,
        }
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self, now: Instant) {
        self.paused = false;
        self.last_tick = now;
    }

    pub fn snapshot(&self) -> TimerSnapshot {
        let total = self.spec.total_secs.max(f64::EPSILON);
        let elapsed = self.elapsed_secs.clamp(0.0, self.spec.total_secs);
        let remaining = (self.spec.total_secs - elapsed).max(0.0);
        let progress = (elapsed / total).clamp(0.0, 1.0);

        TimerSnapshot {
            name: self.spec.name.clone(),
            total_secs: total,
            elapsed_secs: elapsed,
            remaining_secs: remaining,
            progress,
            paused: self.paused,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{PhaseSpec, TimeUnit, TimerEngine, TimerEvent};

    #[test]
    fn time_unit_minutes_to_seconds() {
        assert_eq!(TimeUnit::Minutes.to_seconds(2.0), 120.0);
    }

    #[test]
    fn pause_stops_elapsed_until_resume() {
        let mut engine = TimerEngine::default();
        let t0 = std::time::Instant::now();
        let spec = PhaseSpec::new(String::from("Work"), 10.0, TimeUnit::Seconds);

        engine.start_at(spec, t0);
        let paused = engine.pause().expect("timer should be active");
        assert!(paused.paused);

        let paused_tick = engine.tick(t0 + Duration::from_secs(3));
        assert!(paused_tick.is_none());

        let resumed = engine
            .resume_at(t0 + Duration::from_secs(3))
            .expect("timer should resume");
        assert!(!resumed.paused);

        let post_resume_event = engine
            .tick(t0 + Duration::from_secs(5))
            .expect("timer should emit tick");

        match post_resume_event {
            TimerEvent::Tick(snapshot) => {
                assert!((snapshot.elapsed_secs - 2.0).abs() < 1e-9);
                assert!((snapshot.remaining_secs - 8.0).abs() < 1e-9);
            }
            TimerEvent::Completed(_) | TimerEvent::Stopped => {
                panic!("expected tick event after resume")
            }
        }
    }

    #[test]
    fn tick_emits_completed_at_duration_boundary() {
        let mut engine = TimerEngine::default();
        let t0 = std::time::Instant::now();
        let spec = PhaseSpec::new(String::from("Work"), 5.0, TimeUnit::Seconds);

        engine.start_at(spec, t0);

        let event = engine
            .tick(t0 + Duration::from_secs(5))
            .expect("timer should emit completion");

        match event {
            TimerEvent::Completed(snapshot) => {
                assert!((snapshot.elapsed_secs - 5.0).abs() < 1e-9);
                assert!((snapshot.progress - 1.0).abs() < 1e-9);
                assert!((snapshot.remaining_secs - 0.0).abs() < 1e-9);
            }
            TimerEvent::Tick(_) | TimerEvent::Stopped => {
                panic!("expected completed event at boundary")
            }
        }

        assert!(engine.tick(t0 + Duration::from_secs(6)).is_none());
    }
}
