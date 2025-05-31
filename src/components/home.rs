use std::{fs, io, path::Path};

use symphonia::core::{
    audio::Signal,
    codecs::DecoderOptions,
    formats::FormatOptions,
    meta::MetadataOptions,
    probe::Hint,
    io::MediaSourceStream,
};
use symphonia::default::{get_codecs, get_probe};



use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph},
    text::{Line, Span, Text},
};

use tokio::sync::mpsc::UnboundedSender;


use crate::{
    action::Action, 
    components::{Component, player::Player},  
    config::Config
};

#[derive(Default)]
pub struct Home {
    player: Player,  
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    selected_widget: usize,
    selected_index: usize,
    selected_song_index: usize,
    list_items: Vec<ListItem<'static>>,
    song_items: Vec<(String, String, u64)>,
}

impl Home {
    pub fn new() -> Self {
        let list = Self::get_audio_files("local_music").unwrap_or_default();

        Self {
            player: Player::new(),  
            //  is_playing: false,
            // volume: 1.0,
            selected_widget: 0,
            selected_index: 0,
            selected_song_index: 0,
            list_items: vec![
                ListItem::new(" Тренди"),
                ListItem::new(" Улюблені"),
                ListItem::new(" Списки відтворення"),
            ],
            song_items: list,
            ..Default::default()
        }
    }

    

    fn border_style(&self, index: usize) -> Style {
        if self.selected_widget == index {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        }
    }



pub fn get_audio_files(path: &str) -> io::Result<Vec<(String, String, u64)>> {
    let mut results = vec![];

    for entry in fs::read_dir(Path::new(path))? {
        let entry = entry?;
        let path = entry.path();

        // Перевірка на розширення
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Підтримувані формати
        let supported = ["mp3", "flac", "wav", "aac", "m4a"];

        if !supported.contains(&ext.as_str()) {
            continue;
        }

        let file = std::fs::File::open(&path)?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        hint.with_extension(&ext);

        let probed = get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to parse audio"))?;

        let mut format = probed.format;
        let track = format.default_track().ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "No default track found")
        })?;

        let decoder = get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to create decoder"))?;

        // Отримуємо тривалість
        let duration = if let (Some(frames), Some(rate)) =
            (track.codec_params.n_frames, track.codec_params.sample_rate)
        {
            frames as u64 / rate as u64
        } else {
            0
        };

        let name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        results.push((name, ext.clone(), duration));
    }

    Ok(results)
}

    fn next_widget(&mut self) {
        self.selected_widget = (self.selected_widget + 1) % 3;
    }

    
    fn format_duration(&self, secs: &u64) -> String {
        let minutes = secs / 60;
        let seconds = secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    fn handle_list_navigation(&mut self, code: KeyCode) {
        let (index, max) = match self.selected_widget {
            0 => (&mut self.selected_index, self.list_items.len()),
            1 => (&mut self.selected_song_index, self.song_items.len()),
            _ => return,
        };

        match code {
            KeyCode::Up if *index > 0 => *index -= 1,
            KeyCode::Down if *index + 1 < max => *index += 1,
            _ => {}
        }
    }

    fn render_list(&self, frame: &mut Frame, area: Rect) {
        let mut state = ListState::default();
        state.select(Some(self.selected_index));

        let list = List::new(self.list_items.clone())
            .block(Block::default()
                .title("Список")
                .borders(Borders::ALL)
                .border_style(self.border_style(0)))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
            .highlight_symbol("➤ ");

        frame.render_stateful_widget(list, area, &mut state);
    }


fn render_song_list(&self, frame: &mut Frame, area: Rect) {
    let area_width = area.width as usize;

    let items: Vec<ListItem> = self.song_items.iter().map(|(title, ext, duration)| {
        let song_duration = self.format_duration(duration);
        let left = format!("{}.{:<4}", title, ext); // назва + розширення
        let right = song_duration;

        // Загальна довжина без пробілів
        let total_len = left.len() + right.len();
        let space = if area_width > total_len {
            area_width - total_len - 4 // залишаємо трохи місця на "➤ " та рамки
        } else {
            1
        };

        let spacing = " ".repeat(space);
        let line = Line::from(vec![
            Span::raw(left),
            Span::raw(spacing),
            Span::styled(right, Style::default().fg(Color::Gray)),
        ]);

        ListItem::new(line)
    }).collect();

    let mut state = ListState::default();
    state.select(Some(self.selected_song_index));

    let list = List::new(items)
        .block(
            Block::default()
                .title("Пісні")
                .borders(Borders::ALL)
                .border_style(self.border_style(1)),
        )
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
        .highlight_symbol("➤ ");

    frame.render_stateful_widget(list, area, &mut state);
}
    // fn render_player(&self, frame: &mut Frame, area: Rect) {
    //     let text = Text::from(vec![
    //         Line::from(vec![
    //             Span::raw("Playing (pavilion "),
    //             Span::raw(" | Shuffle: On "),
    //             Span::raw(" | Repeat: Off "),
    //             Span::raw(" | Volume: 98%)"),
    //         ]),
    //         Line::from(""),
    //         Line::from(vec![
    //             Span::styled("Truck", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    //         ]),
    //         Line::from(""),
    //         Line::from("Hovvdy"),
    //         Line::from(""),
    //         Line::from(vec![
    //             Span::styled("0:38", Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC)),
    //             Span::styled("/3:59", Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC)),
    //             Span::styled(" (-3:20)", Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC)),
    //         ]),
    //     ]);

    //     let paragraph = Paragraph::new(text)
    //         .alignment(Alignment::Center)
    //         .block(Block::default()
    //             .title("Плеєр")
    //             .borders(Borders::ALL)
    //             .border_style(Style::default().fg(Color::LightBlue)));

    //     frame.render_widget(paragraph, area);

    //     let gauge_area = Rect {
    //         x: area.x + 1,
    //         y: area.y,
    //         width: area.width - 2,
    //         height: 1,
    //     };

    //     let progress = 38 * 100 / 239; // 0:38 із 3:59
    //     let gauge = Gauge::default()
    //         .gauge_style(Style::default().fg(Color::Yellow))
    //         .ratio(progress as f64 / 100.0);

    //     frame.render_widget(gauge, gauge_area);
    // }
}

impl Component for Home {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(Action::Key(key));
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Key(key) => match key.code {
                KeyCode::Tab if key.modifiers == KeyModifiers::NONE => self.next_widget(),
                KeyCode::Up | KeyCode::Down => self.handle_list_navigation(key.code),
                KeyCode::Enter => self.player.play_sample(  
                    &self.song_items[self.selected_song_index].0,
                    &self.song_items[self.selected_song_index].1,
                ),
                _ => {}
            },
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(area);

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        self.render_list(frame, chunks[0]);
        self.render_song_list(frame, right_chunks[0]);
        // self.render_player(frame, right_chunks[1]);
        self.player.draw(frame, right_chunks[1])?;

        Ok(())
    }
}
