use crate::{
    app::{
        action::{Action, ActionType},
        menu::Menu,
    },
    preset::Preset,
};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct MenuManagePreset {
    pub preset: Arc<Mutex<Preset>>,
}

impl Menu for MenuManagePreset {
    fn name(&self) -> String {
        String::from(format!("Editing {} ...", self.preset.lock().unwrap()))
    }

    fn with_actions(&self) -> Vec<Action> {
        vec![
            Action {
                name: String::from("✓ Apply"),
                action_type: vec![
                    ActionType::ApplyPreset(self.preset.clone()),
                    ActionType::DisplayMessage(format!(
                        "Preset {} successfully applied",
                        self.preset.lock().unwrap()
                    )),
                ],
            },
            Action {
                name: String::from("⌨ Edit keyboard shortcut"),
                action_type: vec![ActionType::ChangeHotkeyInput(self.preset.clone())],
            },
            Action {
                name: String::from("× Delete"),
                action_type: vec![
                    ActionType::DeletePreset(self.preset.clone()),
                    ActionType::GoBack,
                    ActionType::DisplayMessage(format!(
                        "Preset {} successfully deleted",
                        self.preset.lock().unwrap()
                    )),
                ],
            },
        ]
    }
}
