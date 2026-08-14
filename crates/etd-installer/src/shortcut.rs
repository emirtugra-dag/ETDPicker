use std::path::Path;
use std::process::Command;

pub fn create_shortcut(
    target_exe: &Path,
    shortcut_lnk: &Path,
    icon_path: &Path,
    description: &str,
) -> bool {
    let script = format!(
        "$WshShell = New-Object -ComObject WScript.Shell; \
         $Shortcut = $WshShell.CreateShortcut('{}'); \
         $Shortcut.TargetPath = '{}'; \
         $Shortcut.WorkingDirectory = '{}'; \
         $Shortcut.IconLocation = '{}, 0'; \
         $Shortcut.Description = '{}'; \
         $Shortcut.Save()",
        shortcut_lnk.to_string_lossy().replace('\'', "''"),
        target_exe.to_string_lossy().replace('\'', "''"),
        target_exe
            .parent()
            .unwrap_or(target_exe)
            .to_string_lossy()
            .replace('\'', "''"),
        icon_path.to_string_lossy().replace('\'', "''"),
        description.replace('\'', "''")
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output();

    output.map(|o| o.status.success()).unwrap_or(false)
}

pub fn get_desktop_dir() -> Option<std::path::PathBuf> {
    if let Ok(profile) = std::env::var("USERPROFILE") {
        let p = std::path::PathBuf::from(profile).join("Desktop");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn get_start_menu_dir() -> Option<std::path::PathBuf> {
    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = std::path::PathBuf::from(appdata)
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs");
        if p.exists() {
            return Some(p);
        }
    }
    None
}
