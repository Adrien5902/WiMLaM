use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use windows::Win32::{
    Foundation::POINTL,
    Graphics::Gdi::{DEVMODE_DISPLAY_ORIENTATION, DEVMODEW},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisplaySettings {
    pub position: Pos,
    pub orientation: DisplayOrientation,
}

impl Display for DisplaySettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{}/{},{}",
            self.position.x, self.position.y, self.orientation
        ))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum DisplayOrientation {
    Landscape = 0,
    Portrait = 1,
    LandscapeReversed = 2,
    PortraitReversed = 3,
}

impl Display for DisplayOrientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Landscape => "Landscape",
            Self::Portrait => "Portrait",
            Self::LandscapeReversed => "LandscapeReversed",
            Self::PortraitReversed => "PortraitReversed",
        })
    }
}

impl DisplayOrientation {
    pub fn is_same_direction(&self, other: &Self) -> bool {
        *self as u32 % 2 == *other as u32 % 2
    }
}

impl Into<DEVMODE_DISPLAY_ORIENTATION> for DisplayOrientation {
    fn into(self) -> DEVMODE_DISPLAY_ORIENTATION {
        DEVMODE_DISPLAY_ORIENTATION(self as u32)
    }
}

impl From<DEVMODE_DISPLAY_ORIENTATION> for DisplayOrientation {
    fn from(value: DEVMODE_DISPLAY_ORIENTATION) -> Self {
        DisplayOrientation::try_from(value.0).unwrap()
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl From<POINTL> for Pos {
    fn from(value: POINTL) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl Into<POINTL> for Pos {
    fn into(self) -> POINTL {
        POINTL {
            x: self.x,
            y: self.y,
        }
    }
}

impl From<DEVMODEW> for DisplaySettings {
    fn from(dm: DEVMODEW) -> Self {
        unsafe {
            let position = dm.Anonymous1.Anonymous2.dmPosition.into();
            let orientation = dm.Anonymous1.Anonymous2.dmDisplayOrientation.into();

            Self {
                orientation,
                position,
            }
        }
    }
}

impl Into<DEVMODEW> for DisplaySettings {
    fn into(self) -> DEVMODEW {
        let mut dm = DEVMODEW::default();
        dm.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
        dm.Anonymous1.Anonymous2.dmDisplayOrientation = self.orientation.into();
        dm.Anonymous1.Anonymous2.dmPosition = self.position.into();
        dm
    }
}
