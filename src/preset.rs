use crate::{
    author_path,
    display_settings::DisplaySettings,
    error::ThisError,
    monitor::{Monitor, MonitorName},
};
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize, de::Visitor};
use std::{
    collections::HashMap,
    fmt::Display,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use win_hotkeys::VKey;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Preset {
    pub keybind: Option<Keybind>,
    map: HashMap<MonitorName, DisplaySettings>,
}

impl Display for Preset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            &self
                .map
                .iter()
                .map(|(monitor_name, settings)| format!("{},{}", monitor_name.clone(), settings))
                .collect::<Vec<String>>()
                .join(" "),
        )
    }
}

#[derive(Debug, Clone)]
pub struct Keybind {
    pub trigger_key: VKey,
    pub modifiers: Vec<VKey>,
}

impl Display for Keybind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut vec: Vec<String> = self
            .modifiers
            .iter()
            .map(|m| m.to_string().replace("VK_", ""))
            .collect();
        vec.push(self.trigger_key.to_string().replace("VK_", ""));

        f.write_str(&vec.join("+"))
    }
}

impl Serialize for Keybind {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Keybind {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(KeybindVisitor)
    }
}

struct KeybindVisitor;
impl<'de> Visitor<'de> for KeybindVisitor {
    type Value = Keybind;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter
            .write_str("a string representing a keyboard shortcut for example \"CTRL+SHIFT+ALT+A\"")
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Keybind::try_from(v).map_err(|_| E::custom(format!("{}", ThisError::KeybindParsingFailed)))
    }
}

impl TryFrom<&str> for Keybind {
    type Error = color_eyre::eyre::ErrReport;
    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let s = value.to_uppercase();
        let mut split = s.split("+").collect::<Vec<_>>().into_iter();

        let trigger_key =
            VKey::from_keyname(split.next_back().ok_or(ThisError::KeybindParsingFailed)?)?;

        let modifiers = split
            .map(|key| Ok(VKey::from_keyname(key)?))
            .collect::<Result<_>>()?;

        Ok(Self {
            trigger_key,
            modifiers,
        })
    }
}

impl Preset {
    fn get_path() -> PathBuf {
        author_path().join("monitors_config.json")
    }

    pub fn write(list: &[Arc<Mutex<Self>>]) -> Result<()> {
        fs::create_dir_all(author_path())?;
        fs::write(Self::get_path(), serde_json::to_string(list)?)?;
        Ok(())
    }

    pub fn read() -> Result<Vec<Self>> {
        let path = Self::get_path();
        if !fs::exists(&path)? {
            return Ok(Vec::new());
        }

        let s = fs::read_to_string(&path)?;
        let value = serde_json::from_str(&s)?;
        Ok(value)
    }

    pub fn from_current_config(monitors: &[Monitor]) -> Result<Self> {
        Ok(Self {
            keybind: None,
            map: monitors
                .iter()
                .map(|monitor| Ok((monitor.name.clone(), monitor.get_display_settings()?)))
                .collect::<Result<_>>()?,
        })
    }

    pub fn get_monitor_map<'a, 'b>(
        &'a self,
        monitors: &'b [Monitor],
    ) -> Result<HashMap<&'b Monitor, &'a DisplaySettings>> {
        Ok(self
            .map
            .iter()
            .map(|(name, settings)| {
                let monitor = monitors
                    .iter()
                    .find(|m| m.name == *name)
                    .ok_or(ThisError::MonitorNotFound)?;

                Ok((monitor, settings))
            })
            .collect::<Result<HashMap<_, _>>>()?)
    }

    pub fn apply(&self, monitors: &[Monitor]) -> Result<()> {
        for (monitor, settings) in self.get_monitor_map(monitors)?.into_iter() {
            monitor.set_settings(settings)?
        }

        Ok(())
    }
}
