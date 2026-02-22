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

## Custom Phases Checklist

Use this checklist to add file-based and CLI-configured custom phases while keeping
defaults as a fallback:

- [x] Add a dedicated config loader module (`src/config.rs`) responsible for parsing,
      validation, and conversion into app phase presets.
- [x] Define a serializable phase config model (`name`, `duration`, `unit`, `color`)
      and conversion logic into runtime `PhasePreset` values.
- [x] Add config-file support (recommended: TOML), with a default lookup path.
- [ ] Implement safe fallback behavior: if config is missing or invalid, print a clear
      warning and use built-in default phases.
- [ ] Add CLI phase overrides (for example repeated `--phase` args) that can fully
      replace file/default phases.
- [ ] Implement source precedence in startup wiring: CLI phases > config file >
      built-in defaults.
- [ ] Move hardcoded phase construction out of `App::run` and inject resolved phases
      from startup (`main.rs`) into app state.
- [ ] Keep timer runtime logic unchanged; only feed timer `PhaseSpec` values derived
      from resolved presets.
- [ ] Add validation rules for user-defined phases (non-empty name, positive duration,
      known unit, parseable color names).
- [ ] Add tests for config parsing, validation failures, precedence rules, and fallback
      behavior.
- [ ] Update README usage docs with config-file examples and CLI examples for custom
      phases.

## License

MIT © 2024 Tomidoro contributors.
