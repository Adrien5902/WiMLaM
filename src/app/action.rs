use crate::{
    app::{TEXT_FG_COLOR, menu::Menu},
    preset::Preset,
};
use ratatui::{text::Line, widgets::ListItem};
use std::sync::{Arc, Mutex};

pub struct Action {
    pub name: String,
    pub action_type: Vec<ActionType>,
}

impl Action {
    pub fn single_action(name: impl Into<String>, action_type: ActionType) -> Self {
        Self {
            name: name.into(),
            action_type: vec![action_type],
        }
    }

    pub fn go_back() -> Self {
        Self::single_action("← Go back", ActionType::GoBack)
    }
}

#[derive(Clone)]
pub enum ActionType {
    ApplyPreset(Arc<Mutex<Preset>>),
    DeletePreset(Arc<Mutex<Preset>>),
    /// Retrieves the current screen layout and saves it as a preset
    SaveCurrentConfigAsPreset,
    /// Displays a text input the user can type in
    ChangeHotkeyInput(Arc<Mutex<Preset>>),
    /// Displays a dismmisable message with content and reloads the current menu's actions
    DisplayMessage(String),
    /// Pushes a menu to the stack
    OpenMenu(Box<dyn Menu>),
    /// Pops a menu from the stack, exits the app if none found
    GoBack,
    /// Starts a headless process of the app
    StartHeadless,
    /// Toggle open on startup, force ?
    ToggleStartup,
}

impl From<&Action> for ListItem<'_> {
    fn from(value: &Action) -> Self {
        let line = Line::styled(value.name.clone(), TEXT_FG_COLOR);
        ListItem::new(line)
    }
}
