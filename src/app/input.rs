use color_eyre::eyre::Result;
use win_hotkeys::VKey;
use windows::Win32::{
    Foundation::HANDLE,
    System::Console::{
        CONSOLE_MODE, ENABLE_EXTENDED_FLAGS, ENABLE_MOUSE_INPUT, ENABLE_WINDOW_INPUT,
        GetConsoleMode, GetStdHandle, INPUT_RECORD, KEY_EVENT, LEFT_ALT_PRESSED, LEFT_CTRL_PRESSED,
        RIGHT_ALT_PRESSED, RIGHT_CTRL_PRESSED, ReadConsoleInputW, SHIFT_PRESSED, STD_INPUT_HANDLE,
        SetConsoleMode,
    },
};

use crate::preset::Keybind;

pub fn get_handle() -> Result<HANDLE> {
    let hstdin = unsafe { GetStdHandle(STD_INPUT_HANDLE)? };

    Ok(hstdin)
}

pub fn set_console_to_input_mode(hstdin: HANDLE) -> Result<CONSOLE_MODE> {
    let mut mode = CONSOLE_MODE(0);
    let old;
    unsafe {
        GetConsoleMode(hstdin, &mut mode)?;
        old = mode.clone();

        mode |= ENABLE_EXTENDED_FLAGS | ENABLE_WINDOW_INPUT | ENABLE_MOUSE_INPUT;

        // keep keyboard input alive
        mode |= windows::Win32::System::Console::ENABLE_PROCESSED_INPUT;

        SetConsoleMode(hstdin, mode)?;
    }
    Ok(old)
}

pub fn read_input(hstdin: HANDLE) -> Result<Option<Keybind>> {
    let mut record = [INPUT_RECORD::default()];
    let mut read = 0u32;

    unsafe {
        ReadConsoleInputW(hstdin, &mut record, &mut read)?;
    }

    if read == 0 {
        return Ok(None);
    }

    let event = record[0];

    if event.EventType as u32 == KEY_EVENT {
        let key = unsafe { event.Event.KeyEvent };
        let down = key.bKeyDown.as_bool();

        if !down {
            let trigger_vk_code = key.wVirtualKeyCode;
            let trigger_key = VKey::from_vk_code(trigger_vk_code);

            let mut modifiers = Vec::new();
            for (flag, vkey) in [
                (RIGHT_ALT_PRESSED, VKey::RMenu),
                (LEFT_ALT_PRESSED, VKey::LMenu),
                (RIGHT_CTRL_PRESSED, VKey::RControl),
                (LEFT_CTRL_PRESSED, VKey::LControl),
                (SHIFT_PRESSED, VKey::Shift),
            ] {
                if flag & key.dwControlKeyState != 0 {
                    modifiers.push(vkey);
                }
            }

            return Ok(Some(Keybind {
                modifiers,
                trigger_key,
            }));
        }
    }

    Ok(None)
}
