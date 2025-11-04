# Tomodoro

Tomodoro is a simple Pomodoro-style countdown timer that runs entirely in the terminal.
It uses [ratatui](https://github.com/ratatui-org/ratatui) for the UI layer and pairs a
background timer worker with a responsive TUI to show progress, phase information, and
shortcut hints.

## Features

- Alternates automatically between “Work” and “Rest” phases.
- Live progress gauge with colours that reflect the current phase.
- Displays phase name plus elapsed/remaining time in `MM:SS` format.
- Keyboard shortcuts for quitting (and ready for extending with pause/resume).

## Requirements

- Rust toolchain (stable; 1.75+ recommended).
- A terminal that supports ANSI colours and the alternate screen buffer.

## Getting Started

```bash
git clone https://github.com/<your-user>/tomidoro.git
cd tomidoro
cargo run
```

The app launches in the terminal, prompts for the phase durations, and immediately
starts the first cycle. Press `q` at any time to exit.

## Project Structure

- `src/main.rs` – sets up the channels, threads, and entry point.
- `src/app.rs` – manages UI state, phase cycling, and drawing code.
- `src/timer.rs` – background timer that sends progress events to the UI.

## Customisation

Adjust the default phase durations or colours in `src/app.rs` (see the `phases` vector).
To add more complex behaviour (pauses, configuration, notifications), hook additional
commands/events into the existing channels.

## License

MIT © 2024 Tomidoro contributors.
