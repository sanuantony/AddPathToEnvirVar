use std::env;
use std::error::Error;
use std::path::PathBuf;

use winreg::enums::*;
use winreg::RegKey;

#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SendMessageTimeoutW,
    WM_SETTINGCHANGE,
    SMTO_ABORTIFHUNG,
    HWND_BROADCAST,
};

const USER_ENV_KEY: &str = "Environment";
const SYSTEM_ENV_KEY: &str =
    "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment";
const PATH_NAME: &str = "PATH";

fn open_env_key(system: bool) -> Result<RegKey, Box<dyn Error>> {
    let key = if system {
        RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey_with_flags(
            SYSTEM_ENV_KEY,
            KEY_READ | KEY_WRITE,
        )?
    } else {
        RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(
            USER_ENV_KEY,
            KEY_READ | KEY_WRITE,
        )?
    };

    Ok(key)
}

pub fn read_path(system: bool) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let key = open_env_key(system)?;

    let raw: String = key.get_value(PATH_NAME).unwrap_or_default();

    Ok(env::split_paths(&raw).collect())
}

pub fn write_path(paths: &[PathBuf], system: bool) -> Result<(), Box<dyn Error>> {
    let key = open_env_key(system)?;

    let joined = env::join_paths(paths)?;
    let value = joined.to_string_lossy().into_owned();

    key.set_value(PATH_NAME, &value)?;

    broadcast_env_change();

    Ok(())
}

fn broadcast_env_change() {
    #[cfg(windows)]
    unsafe {
        let mut result: usize = 0;

        // "Environment" wide string (null-terminated)
        let msg: Vec<u16> = "Environment".encode_utf16().chain(Some(0)).collect();

        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            msg.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            5000,
            &mut result,
        );
    }
}