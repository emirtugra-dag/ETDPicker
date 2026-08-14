use std::path::Path;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyW, RegDeleteKeyW, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW,
    HKEY_CURRENT_USER, KEY_ALL_ACCESS, KEY_SET_VALUE, REG_DWORD, REG_SZ,
};

pub fn register_uninstaller(install_dir: &Path) -> bool {
    let uninst_key: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\ETDPicker\0"
        .encode_utf16()
        .collect();

    unsafe {
        let mut hkey = std::ptr::null_mut();
        if RegCreateKeyW(
            HKEY_CURRENT_USER,
            uninst_key.as_ptr(),
            &mut hkey,
        ) == 0
        {
            let set_str = |name: &str, val: &str| {
                let name_w: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
                let val_w: Vec<u16> = val.encode_utf16().chain(std::iter::once(0)).collect();
                RegSetValueExW(
                    hkey,
                    name_w.as_ptr(),
                    0,
                    REG_SZ,
                    val_w.as_ptr() as *const _,
                    (val_w.len() * 2) as u32,
                );
            };

            let set_dword = |name: &str, val: u32| {
                let name_w: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
                RegSetValueExW(
                    hkey,
                    name_w.as_ptr(),
                    0,
                    REG_DWORD,
                    &val as *const _ as *const _,
                    4,
                );
            };

            let uninstaller_path = install_dir.join("Uninstall.exe");
            let icon_path = install_dir.join("app_icon.ico");

            set_str("DisplayName", "ETDPicker");
            set_str("DisplayVersion", "1.0.0");
            set_str("Publisher", "Emir Tuğra Dağ");
            set_str("DisplayIcon", &icon_path.to_string_lossy());
            set_str("UninstallString", &format!("\"{}\"", uninstaller_path.display()));
            set_str("QuietUninstallString", &format!("\"{}\" --silent", uninstaller_path.display()));
            set_str("InstallLocation", &install_dir.to_string_lossy());
            set_str("Comments", "ETDPicker Screen Color Picker by Emir Tuğra Dağ");

            set_dword("NoModify", 1);
            set_dword("NoRepair", 1);
            set_dword("EstimatedSize", 4096);

            RegCloseKey(hkey);
            true
        } else {
            false
        }
    }
}

pub fn unregister_uninstaller() -> bool {
    let uninst_key: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\0"
        .encode_utf16()
        .collect();
    let sub_key: Vec<u16> = "ETDPicker\0".encode_utf16().collect();

    unsafe {
        let mut hkey = std::ptr::null_mut();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            uninst_key.as_ptr(),
            0,
            KEY_ALL_ACCESS,
            &mut hkey,
        ) == 0
        {
            RegDeleteKeyW(hkey, sub_key.as_ptr());
            RegCloseKey(hkey);
        }

        // Also remove startup entry if present
        let run_key: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run\0"
            .encode_utf16()
            .collect();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            run_key.as_ptr(),
            0,
            KEY_SET_VALUE,
            &mut hkey,
        ) == 0
        {
            RegDeleteValueW(hkey, sub_key.as_ptr());
            RegCloseKey(hkey);
        }

        true
    }
}
