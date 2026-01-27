use crate::{display_settings::DisplaySettings, error::ThisError};
use color_eyre::eyre::Result;
use serde::Serialize;
use std::hash::Hash;
use windows::{
    Win32::{
        Graphics::Gdi::{
            CDS_UPDATEREGISTRY, ChangeDisplaySettingsExW, DEVMODEW, DISP_CHANGE_SUCCESSFUL,
            DISPLAY_DEVICEW, DM_DISPLAYORIENTATION, DM_PELSHEIGHT, DM_PELSWIDTH, DM_POSITION,
            EDS_ROTATEDMODE, ENUM_CURRENT_SETTINGS, EnumDisplayDevicesW, EnumDisplaySettingsExW,
        },
        UI::WindowsAndMessaging::EDD_GET_DEVICE_INTERFACE_NAME,
    },
    core::PCWSTR,
};

pub type MonitorName = String;

#[derive(Debug, Serialize)]
pub struct Monitor {
    #[serde(flatten)]
    pub name: MonitorName,

    #[serde(skip)]
    pub monitor: DISPLAY_DEVICEW,
    #[serde(skip)]
    pub adapter: DISPLAY_DEVICEW,
}

impl Eq for Monitor {}
impl PartialEq for Monitor {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Hash for Monitor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(&self.name.bytes().collect::<Vec<u8>>());
    }
}

impl Monitor {
    pub fn get_name(device: &DISPLAY_DEVICEW) -> MonitorName {
        wide_to_string(&device.DeviceName)
    }

    pub fn get_monitors() -> Vec<Monitor> {
        let mut monitors = Vec::new();
        unsafe {
            let mut device_index = 0;

            loop {
                let mut adapter = DISPLAY_DEVICEW::default();
                adapter.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;

                let success = EnumDisplayDevicesW(
                    PCWSTR::null(),
                    device_index,
                    &mut adapter,
                    EDD_GET_DEVICE_INTERFACE_NAME,
                );

                if !success.as_bool() {
                    break;
                }

                let mut monitor_index = 0;
                loop {
                    let mut monitor = DISPLAY_DEVICEW::default();
                    monitor.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;

                    let success = EnumDisplayDevicesW(
                        PCWSTR(adapter.DeviceName.as_ptr()),
                        monitor_index,
                        &mut monitor,
                        0,
                    );

                    if !success.as_bool() {
                        break;
                    }

                    monitor_index += 1;

                    monitors.push(Monitor {
                        monitor,
                        adapter,
                        name: Self::get_name(&monitor),
                    });
                }

                device_index += 1;
            }
        }
        monitors.sort_by(|a, b| a.name.cmp(&b.name));
        monitors
    }

    pub fn get_lpszdevicename(&self) -> PCWSTR {
        PCWSTR(self.adapter.DeviceName.as_ptr())
    }

    pub fn get_display_settings(&self) -> Result<DisplaySettings> {
        let dm = self.get_devmodew()?;
        Ok(dm.into())
    }

    pub fn get_devmodew(&self) -> Result<DEVMODEW> {
        let mut dm = DEVMODEW::default();
        dm.dmSize = std::mem::size_of::<DEVMODEW>() as u16;

        unsafe {
            if !EnumDisplaySettingsExW(
                self.get_lpszdevicename(),
                ENUM_CURRENT_SETTINGS,
                &mut dm,
                EDS_ROTATEDMODE,
            )
            .as_bool()
            {
                Err(ThisError::EnumDisplaySettings)?;
            }
        }

        Ok(dm)
    }

    pub fn set_settings(&self, display_settings: &DisplaySettings) -> Result<()> {
        unsafe {
            let mut dm = self.get_devmodew()?;

            if !DisplaySettings::from(dm)
                .orientation
                .is_same_direction(&display_settings.orientation)
            {
                std::mem::swap(&mut dm.dmPelsWidth, &mut dm.dmPelsHeight);
            }

            dm.Anonymous1.Anonymous2.dmDisplayOrientation = display_settings.orientation.into();
            dm.Anonymous1.Anonymous2.dmPosition = display_settings.position.into();

            dm.dmFields |= DM_DISPLAYORIENTATION | DM_PELSWIDTH | DM_PELSHEIGHT | DM_POSITION;

            let result = ChangeDisplaySettingsExW(
                self.get_lpszdevicename(),
                Some(&dm),
                None,
                CDS_UPDATEREGISTRY,
                None,
            );

            if result == DISP_CHANGE_SUCCESSFUL {
                Ok(())
            } else {
                Err(ThisError::DispChange(result))?
            }
        }
    }
}

fn wide_to_string(wide: &[u16]) -> String {
    let len = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    String::from_utf16_lossy(&wide[..len])
}
