use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph},
    text::{Line, Span, Text},
};

use tokio::sync::mpsc::UnboundedSender;

mod player;
use player::Player;

use super::Component;
use crate::{action::Action, config::Config};

#[derive(Default)]
pub struct Home {
    // player: Player,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    selected_widget: usize,
    selected_index: usize,
    selected_song_index: usize,
    list_items: Vec<ListItem<'static>>,
    song_items: Vec<(String, String)>,
}

impl Home {
    pub fn new() -> Self {
        Self {
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
            song_items: vec![
                ("Super long string ".into(), "3:32".into());
                4
            ],
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

    fn next_widget(&mut self) {
        self.selected_widget = (self.selected_widget + 1) % 3;
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
        let items: Vec<ListItem> = self.song_items.iter().map(|(title, duration)| {
            let padded_title = format!("{:<30}", title);
            let spans = Line::from(vec![
                Span::raw(padded_title),
                Span::styled(duration, Style::default().fg(Color::Gray)),
            ]);
            ListItem::new(spans)
        }).collect();

        let mut state = ListState::default();
        state.select(Some(self.selected_song_index));

        let list = List::new(items)
            .block(Block::default()
                .title("Пісні")
                .borders(Borders::ALL)
                .border_style(self.border_style(1)))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
            .highlight_symbol("➤ ");

        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_player(&self, frame: &mut Frame, area: Rect) {
        let text = Text::from(vec![
            Line::from(vec![
                Span::raw("Playing (pavilion "),
                Span::raw(" | Shuffle: On "),
                Span::raw(" | Repeat: Off "),
                Span::raw(" | Volume: 98%)"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Truck", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from("Hovvdy"),
            Line::from(""),
            Line::from(vec![
                Span::styled("0:38", Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC)),
                Span::styled("/3:59", Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC)),
                Span::styled(" (-3:20)", Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC)),
            ]),
        ]);

        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(Block::default()
                .title("Плеєр")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightBlue)));

        frame.render_widget(paragraph, area);

        let gauge_area = Rect {
            x: area.x + 1,
            y: area.y,
            width: area.width - 2,
            height: 1,
        };

        let progress = 38 * 100 / 239; // 0:38 із 3:59
        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Yellow))
            .ratio(progress as f64 / 100.0);

        frame.render_widget(gauge, gauge_area);
    }
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
        self.render_player(frame, right_chunks[1]);

        Ok(())
    }
}
