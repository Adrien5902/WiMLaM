use crate::{
    DATA,
    app::{
        action::{Action, ActionType},
        menu::{Menu, manage_preset::MenuManagePreset},
    },
};

#[derive(Clone)]
pub struct MenuPresetList;

impl Menu for MenuPresetList {
    fn name(&self) -> String {
        String::from("Preset to config")
    }

    fn with_actions(&self) -> Vec<Action> {
        let mut vec: Vec<Action> = DATA
            .lock()
            .unwrap()
            .presets
            .iter()
            .map(|preset| {
                Action::single_action(
                    preset.lock().unwrap().to_string(),
                    ActionType::OpenMenu(Box::new(MenuManagePreset {
                        preset: preset.clone(),
                    })),
                )
            })
            .collect();
        vec.sort_by(|a, b| a.name.cmp(&b.name));
        vec
    }
}
