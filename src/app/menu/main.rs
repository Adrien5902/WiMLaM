use crate::{
    DATA,
    app::{
        action::{Action, ActionType},
        menu::{Menu, manage_preset::MenuManagePreset, preset_list::MenuPresetList},
    },
};

#[derive(Clone)]
pub struct MenuMain;

impl Menu for MenuMain {
    fn name(&self) -> String {
        String::from("What do you want to do ?")
    }

    fn with_actions(&self) -> Vec<Action> {
        vec![
            Action::single_action(
                "▸ Start the backgound app process",
                ActionType::StartHeadless,
            ),
            Action::single_action(
                "≡ Manage presets",
                ActionType::OpenMenu(Box::new(MenuPresetList)),
            ),
            Action {
                name: String::from("↓ Save current config as preset"),
                action_type: vec![
                    ActionType::SaveCurrentConfigAsPreset,
                    ActionType::DisplayMessage(format!(
                        "Current preset successfully saved as {}",
                        DATA.lock().unwrap().presets.last().unwrap().lock().unwrap()
                    )),
                    ActionType::OpenMenu(Box::new(MenuManagePreset {
                        preset: DATA.lock().unwrap().presets.last().unwrap().clone(),
                    })),
                ],
            },
        ]
    }
}
