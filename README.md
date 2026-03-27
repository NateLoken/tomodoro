# Tomodoro

Tomodoro is a simple Pomodoro-style countdown timer that runs entirely in the terminal.
It uses [ratatui](https://github.com/ratatui-org/ratatui) for the UI layer and pairs a
background timer worker with a responsive TUI to show progress, phase information, and
shortcut hints.

## Features

- Alternates automatically between “Work” and “Rest” phases.
- Live progress gauge with colours that reflect the current phase.
- Displays phase name plus elapsed/remaining time in `MM:SS` format.
- Keyboard shortcuts for pause/resume, skipping phases, quitting.

## Requirements

- Rust toolchain (stable; 1.75+ recommended).
- A terminal that supports ANSI colours and the alternate screen buffer.

## Getting Started

```bash
git clone https://github.com/<your-user>/tomidoro.git
cd tomidoro
cargo run
```

The app launches in the terminal and starts the first phase immediately. Press `q` at
any time to exit.

## Project Structure

- `src/main.rs` – sets up the channels, threads, and entry point.
- `src/app.rs` – manages UI state, phase cycling, and drawing code.
- `src/timer.rs` – background timer that sends progress events to the UI.

## Configuration (`config.toml`)

Tomodoro reads phase settings from `config.toml` in the current working directory.

- If `config.toml` is missing, Tomodoro writes a starter file with default phases.
- If `config.toml` exists but cannot be parsed, Tomodoro prints a warning and falls
  back to built-in defaults.

Example `config.toml`:

```toml
[[ phases ]]
name = "Focus"
duration = 25.0
unit = "Minutes"
color = "Red"

[[ phases ]]
name = "Rest"
duration = 5.0
unit = "Minutes"
color = "Blue"
```

Field reference:

- `name` (string): label shown in the UI.
- `duration` (number): phase length for the selected unit.
- `unit` (string): `Seconds`, `Minutes`, or `Hours`.
- `color` (string): a `ratatui` color name like `Red`, `Blue`, `Green`, `Yellow`,
  or `Cyan`.

## License

MIT © 2024 Tomidoro contributors.

Sounds from [Notification Sounds](https://notificationsounds.com/).
