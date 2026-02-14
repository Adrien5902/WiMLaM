mod app;
mod display_settings;
mod error;
mod monitor;
mod preset;

use crate::{app::App, monitor::Monitor, preset::Preset};
use color_eyre::{self, eyre::Result};
use once_cell::sync::Lazy;
use std::{
    path::PathBuf,
    process,
    sync::{Arc, Mutex},
};
use sysinfo::System;
use sysinfo::{ProcessRefreshKind, RefreshKind};
use win_hotkeys::HotkeyManager;

pub static DATA: Lazy<Mutex<Data>> = Lazy::new(|| Mutex::new(Data::default()));
pub const SYS_SPECIFCS: Lazy<RefreshKind> = Lazy::new(|| {
    RefreshKind::nothing()
        .with_processes(ProcessRefreshKind::nothing().with_exe(sysinfo::UpdateKind::Always))
});

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let headless = args.contains(&String::from(App::HEADLESS_ARG));

    let monitors = Monitor::get_monitors();
    let presets: Vec<Arc<Mutex<Preset>>> = Preset::read()?
        .into_iter()
        .map(|preset| Arc::new(Mutex::new(preset)))
        .collect();

    *DATA.lock().unwrap() = Data { monitors, presets };

    let mut sys = System::new_with_specifics(*SYS_SPECIFCS);
    kill_others(&mut sys);

    if headless {
        let mut hkm: HotkeyManager<Result<()>> = HotkeyManager::new();
        let presets = DATA.lock().unwrap().presets.clone();
        for preset in presets {
            let p = preset.clone();
            if let Some(keybind) = &preset.lock().unwrap().keybind {
                hkm.register_hotkey(keybind.trigger_key, &keybind.modifiers, move || {
                    let monitors = &DATA.lock().unwrap().monitors;
                    p.lock().unwrap().apply(monitors)?;
                    Ok(())
                })?;
            }
        }

        hkm.event_loop();

        Ok(())
    } else {
        color_eyre::install()?;
        ratatui::run(|terminal| App::default().run(terminal))?;
        Ok(())
    }
}

fn kill_others(sys: &mut System) {
    sys.refresh_specifics(*SYS_SPECIFCS);

    for (pid, other_self_process) in sys.processes().iter().filter(|(_pid, process)| {
        process
            .exe()
            .is_some_and(|exe| exe == std::env::current_exe().unwrap())
    }) {
        if pid.as_u32() != process::id() {
            other_self_process.kill();
        }
    }
}

pub fn author_path() -> PathBuf {
    dirs::config_dir().unwrap().join("Adrien5902")
}

#[derive(Default, Debug)]
pub struct Data {
    pub monitors: Vec<Monitor>,
    pub presets: Vec<Arc<Mutex<Preset>>>,
}

impl Data {
    fn save(&self) -> Result<()> {
        Preset::write(&self.presets)
    }
}
