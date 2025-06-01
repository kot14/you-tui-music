use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Key(crossterm::event::KeyEvent),
    Noop,
    Help,
    PressTab, 
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionType {
    Tick,
    Render,
    Resize,
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error,
    Key,
    Noop,
    Help,
    PressTab,
}

impl Action {
    pub fn get_type(&self) -> ActionType {
        match self {
            Action::Tick => ActionType::Tick,
            Action::Render => ActionType::Render,
            Action::Resize(_, _) => ActionType::Resize,
            Action::Suspend => ActionType::Suspend,
            Action::Resume => ActionType::Resume,
            Action::Quit => ActionType::Quit,
            Action::ClearScreen => ActionType::ClearScreen,
            Action::Error(_) => ActionType::Error,
            Action::Key(_) => ActionType::Key,
            Action::Noop => ActionType::Noop,
            Action::Help => ActionType::Help,
            Action::PressTab => ActionType::PressTab,
        }
    }
}
