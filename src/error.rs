use thiserror::Error;
use windows::Win32::Graphics::Gdi::DISP_CHANGE;

#[derive(Debug, Error)]
pub enum ThisError {
    #[error("Failed to change display settings")]
    DispChange(DISP_CHANGE),
    #[error("Failed to fetch display settings")]
    EnumDisplaySettings,
    #[error("Monitor not found for preset")]
    MonitorNotFound,
    #[error("Failed to parse keybind")]
    KeybindParsingFailed,
}
