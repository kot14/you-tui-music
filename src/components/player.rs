use std::fs::File;
use std::io::{BufReader, Cursor};
use std::sync::{Arc, Mutex};

use color_eyre::Result;
use crossterm::event::KeyCode;
use ratatui::{prelude::*, widgets::*};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{action::Action, config::Config};
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
        let (title, artist) = self.current_track.clone().unwrap_or_else(|| ("Unknown".into(), "Unknown".into())); // Translated
        let position_secs = self.position.as_secs();
        let duration_secs = self.duration.as_secs();
        let remaining_secs = duration_secs.saturating_sub(position_secs);


        let format_time = |s: u64| format!("{}:{:02}", s / 60, s % 60);

        let text = Text::from(vec![
            Line::from(vec![
                Span::raw("Playing (pavilion "),
                Span::raw(" | Shuffle: On "),
                Span::raw(" | Repeat: Off "),
                Span::raw(" | Volume: 98%)"),
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

        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(Block::default()
                .title("Player") // Translated "Плеєр"
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightBlue)));

        frame.render_widget(paragraph, area);

        let gauge_area = Rect {
            x: area.x + 1,
            y: area.y,
            width: area.width - 2,
            height: 1,
        };

    let progress = if self.duration > Duration::ZERO {
    self.position.as_secs_f64() / self.duration.as_secs_f64()
} else {
    0.0
};

let gauge = Gauge::default()
    .gauge_style(Style::default().fg(Color::Yellow))
    .ratio(progress);

        frame.render_widget(gauge, gauge_area);
    }

    pub fn change_volume(&mut self, action: bool) {
        if action {
            if self.volume < 1.0 {
                self.volume += 0.1
            } else {
               if self.volume >= 0.1 {
                 self.volume -= 0.1
               }
            }
        }
      
    }
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

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
               if let Some(start) = self.playback_start_time {
                let now = Instant::now();
                let elapsed = now.saturating_duration_since(start);
                self.position = elapsed.min(self.duration);
}
            }
            Action::Render => {
                // e.g., start sound if needed
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