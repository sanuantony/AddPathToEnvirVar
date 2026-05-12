use std::env;
use std::error::Error;
use std::path::PathBuf;

use winreg::enums::*;
use winreg::{RegKey, RegValue};

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

fn open_env_key(system: bool, writable: bool) -> Result<RegKey, Box<dyn Error>> {
    let flags = if writable { KEY_READ | KEY_WRITE } else { KEY_READ };
    let key = if system {
        RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey_with_flags(SYSTEM_ENV_KEY, flags)?
    } else {
        RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(USER_ENV_KEY, flags)?
    };

    Ok(key)
}

pub fn read_path(system: bool) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let key = open_env_key(system, false)?;

    let raw = match key.get_raw_value(PATH_NAME) {
        Ok(v) => v,
        Err(_) => return Ok(Vec::new()),
    };

    let wide: Vec<u16> = raw.bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    let s = String::from_utf16_lossy(&wide);
    let s = s.trim_end_matches('\0');

    Ok(env::split_paths(s).collect())
}

pub fn write_path(paths: &[PathBuf], system: bool) -> Result<(), Box<dyn Error>> {
    let key = open_env_key(system, true)?;

    let joined = env::join_paths(paths)?;
    let value = joined.to_string_lossy().into_owned();

    let wide: Vec<u16> = value.encode_utf16().chain(Some(0)).collect();
    let bytes: Vec<u8> = wide.iter().flat_map(|w| w.to_le_bytes()).collect();
    let reg_value = RegValue {
        vtype: RegType::REG_EXPAND_SZ,
        bytes: bytes.into(),
    };
    key.set_raw_value(PATH_NAME, &reg_value)?;

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