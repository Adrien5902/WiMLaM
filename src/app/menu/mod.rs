pub mod main;
pub mod manage_preset;
mod preset_list;

use crate::app::{
    ALT_ROW_BG_COLOR, HEADER_STYLE, NORMAL_ROW_BG, SELECTED_STYLE,
    action::{Action, ActionType},
};
use clone_dyn::dependency::clone_dyn_meta;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::{Color, Stylize},
    symbols,
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, StatefulWidget, Widget,
    },
};

#[clone_dyn_meta::clone_dyn]
pub trait Menu {
    fn name(&self) -> String;
    fn with_actions(&self) -> Vec<Action>;

    fn actions(&self) -> Vec<Action> {
        let mut vec = self.with_actions();
        vec.extend(self.default_actions());
        vec
    }

    fn default_actions(&self) -> Vec<Action> {
        vec![Action::go_back()]
    }
}

impl Widget for &mut RenderedMenu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw(self.menu.name()).centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        let items: Vec<ListItem> = self
            .actions
            .iter()
            .enumerate()
            .map(|(i, action)| {
                let color = alternate_colors(i);
                ListItem::from(action).bg(color)
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.state);
    }
}

const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}

pub struct RenderedMenu {
    menu: Box<dyn Menu>,
    actions: Vec<Action>,
    state: ListState,
}

impl RenderedMenu {
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Vec<ActionType>> {
        match key.code {
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => self.handle_selected_actions(),
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.select_next();
                None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.select_previous();
                None
            }
            _ => None,
        }
    }

    pub fn handle_selected_actions(&mut self) -> Option<Vec<ActionType>> {
        self.state
            .selected()
            .map(|i| self.actions[i].action_type.clone())
    }

    pub fn reload_actions(&mut self) {
        self.actions = self.menu.actions()
    }
}

impl From<Box<dyn Menu>> for RenderedMenu {
    fn from(menu: Box<dyn Menu>) -> Self {
        let mut state = ListState::default();
        state.select_first();

        Self {
            actions: menu.actions(),
            menu,
            state,
        }
    }
}
