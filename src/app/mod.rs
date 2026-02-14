mod action;
mod input;
mod menu;

use crate::DATA;
use crate::app::input::{get_handle, read_input, set_console_to_input_mode};
use crate::app::menu::Menu;
use crate::app::{
    action::ActionType,
    menu::{RenderedMenu, main::MenuMain},
};
use crate::preset::{Keybind, Preset};
use color_eyre::eyre::Result;
use ratatui::layout::Alignment;
use ratatui::style::palette::tailwind::{EMERALD, ZINC};
use ratatui::widgets::{Block, BorderType};
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    crossterm::event::{self, KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Paragraph, Widget},
};
use std::fs;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Console::CONSOLE_MODE;

pub struct App {
    console_mode: CONSOLE_MODE,
    console_handle: HANDLE,
    rendered_change_hotkey: Option<(Arc<Mutex<Preset>>, Option<Keybind>)>,
    rendered_message: Option<String>,
    should_exit: bool,
    /// A screen stack
    path: Vec<RenderedMenu>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            console_mode: CONSOLE_MODE(0),
            console_handle: get_handle().unwrap(),
            rendered_change_hotkey: None,
            rendered_message: None,
            should_exit: false,
            path: vec![(Box::new(MenuMain) as Box<dyn Menu>).into()],
        }
    }
}

const HEADER_BACKGROUND_COLOR: Color = EMERALD.c800;
const HEADER_STYLE: Style = Style::new().fg(ZINC.c50).bg(HEADER_BACKGROUND_COLOR);
const NORMAL_ROW_BG: Color = ZINC.c900;
const ALT_ROW_BG_COLOR: Color = ZINC.c800;
const SELECTED_STYLE: Style = Style::new().bg(ZINC.c700);
const TEXT_FG_COLOR: Color = ZINC.c50;

impl App {
    pub const HEADLESS_ARG: &'static str = "--headless";

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Some(key) = event::read()?.as_key_press_event() {
                self.handle_key(key)?;
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        if self.rendered_message.is_some() {
            self.rendered_message = None;
            return Ok(());
        }

        if let Some((preset, input)) = &mut self.rendered_change_hotkey {
            match key.code {
                KeyCode::Delete => {
                    let preset = preset.lock().unwrap();
                    self.rendered_message = Some(format!("Deleted keybind for preset {}", preset));
                }
                KeyCode::Enter => {
                    if let Some(keybind) = input {
                        preset.lock().unwrap().keybind = Some(keybind.clone());
                        self.rendered_message = Some(format!(
                            "Set keybind as {} for preset {}",
                            keybind,
                            preset.lock().unwrap()
                        ));

                        DATA.lock().unwrap().save()?;
                    } else {
                        self.go_back();
                    }
                }
                KeyCode::Esc => {
                    self.go_back();
                }
                _other => {
                    if let Some(keybind) = read_input(self.console_handle).unwrap() {
                        *input = Some(keybind)
                    }
                }
            }

            return Ok(());
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_exit = true;
                Ok(())
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.go_back();
                Ok(())
            }
            _ => {
                if let Some(action) = self
                    .current_menu_mut()
                    .and_then(|menu| menu.handle_key(key))
                {
                    self.handle_actions(action)
                } else {
                    Ok(())
                }
            }
        }
    }

    fn go_back(&mut self) {
        if self.rendered_change_hotkey.is_some() {
            self.rendered_change_hotkey = None;
            return;
        }

        if self.rendered_message.is_some() {
            self.rendered_message = None;
            return;
        }

        self.path.pop();
        if self.path.len() <= 0 {
            self.should_exit = true;
        }
    }

    fn current_menu_mut(&mut self) -> Option<&mut RenderedMenu> {
        self.path.last_mut()
    }

    fn render_content(&mut self, area: Rect, buf: &mut Buffer) {
        if let Some(message) = &self.rendered_message {
            self.rendered_change_hotkey = None;

            let layout =
                Layout::vertical([Constraint::Fill(1); 3]).flex(ratatui::layout::Flex::Center);
            let a = layout.split(area).to_vec();
            Paragraph::new(message.as_str())
                .centered()
                .block(
                    Block::bordered()
                        .title_alignment(Alignment::Center)
                        .title_style(Style::new().fg(TEXT_FG_COLOR))
                        .border_type(BorderType::Rounded)
                        .border_style(Style::new().fg(HEADER_BACKGROUND_COLOR))
                        .style(Style::new().bg(NORMAL_ROW_BG)),
                )
                .render(a[1], buf);

            // Reloads actions after displaying message
            self.current_menu_mut().unwrap().reload_actions();
            return;
        }

        if let Some((preset, input)) = &mut self.rendered_change_hotkey {
            let block = Block::bordered()
                .title(format!("Editing Shortcut for {}", preset.lock().unwrap()))
                .title_alignment(Alignment::Center)
                .title_style(Style::new().fg(TEXT_FG_COLOR))
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(HEADER_BACKGROUND_COLOR))
                .style(Style::new().bg(NORMAL_ROW_BG));

            let inner_area = block.inner(area);
            block.render(area, buf);

            let layout =
                Layout::vertical([Constraint::Fill(1); 3]).flex(ratatui::layout::Flex::Center);
            let a = layout.split(inner_area).to_vec();

            Paragraph::new(
                input
                    .as_ref()
                    .map(|keybind| keybind.to_string())
                    .unwrap_or_default(),
            )
            .centered()
            .render(a[1], buf);
            return;
        }

        self.current_menu_mut().render(area, buf);
    }

    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("WiMLaM (Windows Monitor Layout Manager)")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(if self.rendered_message.is_some() {
            "Press any key to continue ↩"
        } else if self.rendered_change_hotkey.is_some() {
            "Press Escape to cancel, Enter to confirm"
        } else {
            "Use ←↓↑→ or hjkl to move"
        })
        .centered()
        .render(area, buf);
    }

    fn handle_actions(&mut self, actions: Vec<ActionType>) -> Result<()> {
        actions
            .into_iter()
            .map(|action| self.handle_action(action))
            .collect()
    }

    fn handle_action(&mut self, action: ActionType) -> Result<()> {
        match action {
            ActionType::ApplyPreset(preset) => {
                preset.lock().unwrap().apply(&DATA.lock().unwrap().monitors)
            }
            ActionType::OpenMenu(menu) => {
                self.path.push(RenderedMenu::from(menu));
                Ok(())
            }
            ActionType::GoBack => {
                self.go_back();
                Ok(())
            }
            ActionType::SaveCurrentConfigAsPreset => {
                let preset = { Preset::from_current_config(&DATA.lock().unwrap().monitors)? };
                DATA.lock()
                    .unwrap()
                    .presets
                    .push(Arc::new(Mutex::new(preset)));
                DATA.lock().unwrap().save()?;
                Ok(())
            }
            ActionType::DeletePreset(preset) => {
                let (index, _) = DATA
                    .lock()
                    .unwrap()
                    .presets
                    .iter()
                    .enumerate()
                    .find(|(_, a)| Arc::as_ptr(*a) == Arc::as_ptr(&preset))
                    .unwrap();
                DATA.lock().unwrap().presets.remove(index);
                DATA.lock().unwrap().save()?;
                Ok(())
            }
            ActionType::DisplayMessage(message) => {
                self.rendered_message = Some(message);
                Ok(())
            }
            ActionType::ChangeHotkeyInput(preset) => {
                self.rendered_change_hotkey = Some((preset, None));
                self.console_mode = set_console_to_input_mode(self.console_handle)?;
                Ok(())
            }
            ActionType::StartHeadless => {
                Command::new(std::env::current_exe()?)
                    .arg(Self::HEADLESS_ARG)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()?;

                self.should_exit = true;

                Ok(())
            }
            ActionType::ToggleStartup => {
                let startup_dir = dirs::template_dir()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join("Start Menu/Programs/Startup");

                let startup_path = startup_dir.join("WiMLaM.vbs");

                if !startup_path.exists() {
                    fs::write(
                        startup_path,
                        format!(
                            "CreateObject(\"Wscript.Shell\").Run \"{} {}\", 0, True",
                            std::env::current_exe()?.to_string_lossy(),
                            Self::HEADLESS_ARG
                        ),
                    )?;

                    self.handle_action(ActionType::DisplayMessage(format!(
                        "Successfully added app to opening on windows startup ",
                    )))?;
                } else {
                    fs::remove_file(startup_path)?;

                    self.handle_action(ActionType::DisplayMessage(format!(
                        "Successfully removed app from opening on windows startup ",
                    )))?;
                }

                Ok(())
            }
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let main_layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]);
        let [header_area, content_area, footer_area] = area.layout(&main_layout);

        App::render_header(header_area, buf);
        self.render_footer(footer_area, buf);
        self.render_content(content_area, buf)
    }
}
