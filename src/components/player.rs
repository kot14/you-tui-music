use std::fs::File;
use std::io::{BufReader, Cursor};
use std::sync::{Arc, Mutex};

use color_eyre::Result;
use crossterm::event::KeyCode;
use ratatui::{prelude::*, widgets::*};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{action::{Action, ActionType}, config::Config}; // Added ActionType
use std::time::{Duration, Instant};
#[derive(Default)]
pub struct Player {
   command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    current_track: Option<(String, String)>, // Title, author
    volume: f32,
    position: Duration,
    duration: Duration,
    playback_start_time: Option<Instant>,
    sink: Option<Arc<Mutex<Sink>>>,
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
}

impl Player {
    pub fn new() -> Self {
        
        let (_stream, stream_handle) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(_) => (None, None),
        };

        let sink = stream_handle
            .as_ref()
            .map(|handle| Sink::try_new(handle).ok())
            .flatten()
            .map(|s| Arc::new(Mutex::new(s)));

        Self {
            command_tx: None,
            config: Config::default(),
            current_track: Some(("Unknown Track".to_string(), "Unknown Author".to_string())), // Translated
            volume: 0.5,
            position: Duration::from_secs(0),
            duration: Duration::from_secs(0),
            playback_start_time: None,
            sink,
            _stream,
            stream_handle,
        }
    }

        fn render_player(&self, frame: &mut Frame, area: Rect) {
        let (title, artist) = self.current_track.clone().unwrap_or_else(|| ("Unknown".into(), "Unknown".into()));
        let position_secs = self.position.as_secs();
        let duration_secs = self.duration.as_secs();
        let remaining_secs = duration_secs.saturating_sub(position_secs);

        let format_time = |s: u64| format!("{}:{:02}", s / 60, s % 60);

        // Define the main block for the player
        let player_block = Block::default()
            .title("Player")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightBlue));

        // Get the inner area of the block
        let inner_area = player_block.inner(area);

        // Render the main block itself
        frame.render_widget(player_block, area);

        // Create a layout to split the inner_area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),       // Top chunk for text (takes remaining space)
                Constraint::Length(1),    // Bottom chunk for Gauge (fixed height 1)
            ])
            .split(inner_area);

        let text_area = chunks[0];
        let gauge_area = chunks[1];

        // Prepare text for the top chunk
        let text_content = Text::from(vec![
            Line::from(vec![
                Span::raw("Playing (pavilion "),
                Span::raw(" | Shuffle: On "),
                Span::raw(" | Repeat: Off "),
                Span::raw(format!(" | Volume: {:.0}%", self.volume * 100.0)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(&title, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(artist),
            Line::from(""),
            Line::from(vec![
                Span::styled(format_time(position_secs), Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC)),
                Span::styled(format!("/{}", format_time(duration_secs)), Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC)),
                Span::styled(format!(" (-{})", format_time(remaining_secs)), Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC)),
            ]),
        ]);

        let paragraph = Paragraph::new(text_content).alignment(Alignment::Center);
        frame.render_widget(paragraph, text_area); // Render paragraph in the top chunk

        // Prepare and render Gauge in the bottom chunk
        let progress = if self.duration > Duration::ZERO {
            self.position.as_secs_f64() / self.duration.as_secs_f64()
        } else {
            0.0
        };

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Yellow))
            .ratio(progress);
        frame.render_widget(gauge, gauge_area); // Render gauge in the bottom chunk
    }

    // pub fn change_volume(&mut self, action: bool) { // Method removed
    //     if action {
    //         if self.volume < 1.0 {
    //             self.volume += 0.1
    //         } else {
    //            if self.volume >= 0.1 {
    //              self.volume -= 0.1
    //            }
    //         }
    //     }
    // }

    pub fn play_sample(&mut self, name: &str, ext: &str) {
        // Creating path to file
        let path = format!("local_music/{}.{}", name, ext);

        // Checking if Sink already exists - if yes, stop
           if let Some(sink) = &self.sink {
        // Stopping current track
        sink.lock().unwrap().stop();

        // Forming path to file
        let path = format!("./local_music/{}.{}", name, ext);
        if let Ok(file) = File::open(&path) {
            let source = Decoder::new(BufReader::new(file));
            if let Ok(source) = source {
                let duration = source.total_duration().unwrap_or(Duration::from_secs(0));
                sink.lock().unwrap().append(source);

                // Updating internal state
                self.current_track = Some((name.to_string(), "Unknown Author".to_string())); // Translated, original comment: replace "Unknown Author" if you can extract
                self.position = Duration::from_secs(0);
                self.duration = duration;
                self.playback_start_time = Some(Instant::now());
            }
        } else {
            eprintln!("Failed to open file: {}", path); // Translated
        }
    }

        // Creating new Sink
        if let Some(handle) = &self.stream_handle {
            if let Ok(new_sink) = Sink::try_new(handle) {
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    if let Ok(source) = Decoder::new(reader) {
                        new_sink.append(source);
                        self.sink = Some(Arc::new(Mutex::new(new_sink)));
                    } else {
                        eprintln!("❌ Failed to decode: {}", path); // Translated
                    }
                } else {
                    eprintln!("❌ Failed to open: {}", path); // Translated
                }
            } else {
                eprintln!("❌ Failed to create Sink"); // Translated
            }
        } else {
            eprintln!("❌ stream_handle is missing"); // Translated
        }
    }
}

impl Component for Player {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn interested_actions(&self) -> Vec<ActionType> {
        vec![
            ActionType::Tick,
            ActionType::Render,
            ActionType::VolumeUp,
            ActionType::VolumeDown,
        ]
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                if let Some(sink_arc) = &self.sink {
                    let sink_guard = sink_arc.lock().unwrap();
                    if !sink_guard.is_paused() && !sink_guard.empty() {
                        if let Some(start) = self.playback_start_time {
                            let now = Instant::now();
                            let elapsed = now.saturating_duration_since(start);
                            self.position = elapsed.min(self.duration);
                        }
                    }
                }
            }
            Action::Render => {
                // e.g., start sound if needed
            }
            Action::VolumeUp => {
                self.volume = (self.volume + 0.05).min(1.0);
                if let Some(sink_ref) = &self.sink {
                    sink_ref.lock().unwrap().set_volume(self.volume);
                }
            }
            Action::VolumeDown => {
                self.volume = (self.volume - 0.05).max(0.0);
                if let Some(sink_ref) = &self.sink {
                    sink_ref.lock().unwrap().set_volume(self.volume);
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
    self.render_player(frame, area);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;
    use std::time::Duration;

    // Helper to create a Player instance for testing.
    // It won't have a real audio sink for most tests.
    fn test_player() -> Player {
        Player {
            command_tx: None,
            config: Config::default(),
            current_track: None,
            volume: 0.5, // Initial volume for predictable tests
            position: Duration::from_secs(0),
            duration: Duration::from_secs(0),
            playback_start_time: None,
            sink: None, // No sink for these unit tests to avoid audio backend init
            _stream: None,
            stream_handle: None,
        }
    }

    #[test]
    fn test_volume_up() {
        let mut player = test_player();
        let initial_volume = player.volume;

        // Simulate VolumeUp action
        player.update(Action::VolumeUp).unwrap();
        assert!(player.volume > initial_volume, "Volume should increase");
        assert!(player.volume <= 1.0, "Volume should not exceed 1.0");

        // Test clamping at max volume
        player.volume = 0.98; // Set close to max
        player.update(Action::VolumeUp).unwrap(); // Should go to 1.0
        player.update(Action::VolumeUp).unwrap(); // Should stay at 1.0
        assert_eq!(player.volume, 1.0, "Volume should be capped at 1.0");
    }

    #[test]
    fn test_volume_down() {
        let mut player = test_player();
        let initial_volume = player.volume;

        // Simulate VolumeDown action
        player.update(Action::VolumeDown).unwrap();
        assert!(player.volume < initial_volume, "Volume should decrease");
        assert!(player.volume >= 0.0, "Volume should not be less than 0.0");

        // Test clamping at min volume
        player.volume = 0.02; // Set close to min
        player.update(Action::VolumeDown).unwrap(); // Should go to 0.0
        player.update(Action::VolumeDown).unwrap(); // Should stay at 0.0
        assert_eq!(player.volume, 0.0, "Volume should be capped at 0.0");
    }

    #[test]
    fn test_volume_step() {
        let mut player = test_player();
        player.volume = 0.5;
        player.update(Action::VolumeUp).unwrap();
        // Using 0.05 as the step defined in previous implementation
        assert_eq!(player.volume, 0.55, "Volume should increase by 0.05");

        player.update(Action::VolumeDown).unwrap();
        player.update(Action::VolumeDown).unwrap();
        // 0.55 -> 0.50 -> 0.45
        assert_eq!(player.volume, 0.45, "Volume should decrease by 0.05 twice");
    }
}