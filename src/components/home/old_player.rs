use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use ratatui::{widgets::*, layout::*, style::*, text::*, Frame};

pub struct OldPlayer { // Renamed from Old_Player
    sink: Option<Arc<Mutex<Sink>>>,
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
}

impl OldPlayer { // Renamed from Old_Player
    pub fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().expect("Failed to init audio");
        Self {
            sink: None,
            _stream,
            stream_handle,
        }
    }

    pub fn play_file(&mut self, path: &str) {
        let file = File::open(path).expect("Cannot open file");
        let source = Decoder::new(BufReader::new(file)).expect("Cannot decode file");

        let sink = Sink::try_new(&self.stream_handle).expect("Failed to create sink");
        sink.append(source);
        sink.play();
        self.sink = Some(Arc::new(Mutex::new(sink)));
    }

    pub fn stop(&mut self) {
        if let Some(sink) = &self.sink {
            sink.lock().unwrap().stop();
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let info = Text::from(Line::from(vec![
            Span::styled("🎵 Playing: ", Style::default().fg(Color::Green)), // Translated "Грає"
            Span::raw("Track Name – Artist"),
        ]));

        let paragraph = Paragraph::new(info)
            .alignment(Alignment::Center)
            .block(Block::default().title("Player").borders(Borders::ALL)); // Translated "Плеєр"

        frame.render_widget(paragraph, area);
    }
}
