use std::{
    fs::File,
    io::{self, BufReader, Write},
    path::{Path, PathBuf},
    thread,
};

use rodio::{DeviceSinkBuilder, play};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct NotifyConfig {
    pub name: String,
    pub file: Option<PathBuf>,
    pub volume: f32,
}

impl Default for NotifyConfig {
    fn default() -> Self {
        Self {
            name: "Tomodoro".to_string(),
            file: None,
            volume: 1.0,
        }
    }
}

pub fn notif_phase_complete(phase_name: &str, config: &NotifyConfig) {
    let title = "Finished";
    let body = format!("{phase_name}ing");

    dispatch_notification(&config.name, title, &body);

    let sound_file = config.file.clone();
    let volume = config.volume.clamp(0.0, 1.0);
    thread::spawn(move || {
        if let Some(path) = sound_file {
            if play_sound(&path, volume).is_ok() {
                return;
            }

            print!("\x07");
            let _ = io::stdout().flush();
        }
    });
}

fn dispatch_notification(name: &str, title: &str, body: &str) {
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("notify-send")
            .arg("-a")
            .arg(name)
            .arg(title)
            .arg(body)
            .status();
    }
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        let script = format!(
            "dispatch notification \"{}\" with title \"{}\"",
            escape_applescript(body),
            escape_applescript(title)
        );
        let _ = Command::new("osascript").arg("-e").arg(script).status();
    }
}

fn play_sound(
    file_path: &Path,
    volume: f32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut sink_stream = DeviceSinkBuilder::open_default_sink()?;
    sink_stream.log_on_drop(false);
    let file = BufReader::new(File::open(file_path)?);

    let player = play(sink_stream.mixer(), file).unwrap();
    player.set_volume(volume.clamp(0.0, 1.0));
    player.sleep_until_end();

    Ok(())
}

#[cfg(target_os = "macos")]
fn escape_applescript(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
