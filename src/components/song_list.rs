use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};


use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{action::Action, config::Config};

#[derive(Default)]
pub struct SongList {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    selected_song_index: usize,
    song_items: Vec<(String, String)>,
    border_style: Style,   
}

impl SongList {
    pub fn default() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            selected_song_index: 0,
            song_items: vec![
                ("Super long string ".into(), "3:32".into());
                4
            ],
            border_style: Style::default().fg(Color::DarkGray),
        }
    }
}

impl Component for SongList {
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
                // логіка на кожен тік
            }
            Action::Render => {
                // наприклад, запускати звук якщо потрібно
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
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
                .border_style(self.border_style))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
            .highlight_symbol("➤ ");

        frame.render_stateful_widget(list, area, &mut state);
        Ok(())
    }
}