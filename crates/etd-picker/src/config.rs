use crate::color::RgbColor;
use crate::i18n::Language;
use std::fs;
use std::path::PathBuf;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY_CURRENT_USER, KEY_SET_VALUE,
    REG_SZ,
};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub language: Language,
    pub hotkey_mod: u32,
    pub hotkey_vk: u32,
    pub hotkey_name: String,
    pub run_on_startup: bool,
    pub show_tray_icon: bool,
    pub recent_colors: Vec<RgbColor>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: Language::Turkish,
            hotkey_mod: 0x0001, // MOD_ALT
            hotkey_vk: 0x50,    // 'P' key
            hotkey_name: "Alt + P".to_string(),
            run_on_startup: false,
            show_tray_icon: true,
            recent_colors: vec![
                RgbColor::new(52, 152, 219),  // Blue
                RgbColor::new(46, 204, 113),  // Green
                RgbColor::new(231, 76, 60),   // Red
                RgbColor::new(241, 196, 15),  // Yellow
                RgbColor::new(155, 89, 182),  // Purple
                RgbColor::new(26, 188, 156),  // Teal
                RgbColor::new(230, 126, 34),  // Orange
                RgbColor::new(52, 73, 94),    // Dark Blue
                RgbColor::new(236, 240, 241), // Light Gray
                RgbColor::new(44, 62, 80),    // Slate
            ],
        }
    }
}

impl AppConfig {
    pub fn get_config_path() -> PathBuf {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        let local_ini = exe_dir.join("config.ini");
        if local_ini.exists() {
            return local_ini;
        }

        if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
            let app_dir = PathBuf::from(appdata).join("ETDPicker");
            let _ = fs::create_dir_all(&app_dir);
            return app_dir.join("config.ini");
        }

        local_ini
    }

    pub fn load() -> Self {
        let mut cfg = Self::default();
        let path = Self::get_config_path();

        if let Ok(content) = fs::read_to_string(&path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                    continue;
                }

                if let Some((k, v)) = line.split_once('=') {
                    let k = k.trim();
                    let v = v.trim();
                    match k {
                        "language" => cfg.language = Language::from_code(v),
                        "hotkey_mod" => {
                            if let Ok(m) = v.parse::<u32>() {
                                cfg.hotkey_mod = m;
                            }
                        }
                        "hotkey_vk" => {
                            if let Ok(vk) = v.parse::<u32>() {
                                cfg.hotkey_vk = vk;
                            }
                        }
                        "hotkey_name" => {
                            if !v.is_empty() {
                                cfg.hotkey_name = v.to_string();
                            }
                        }
                        "run_on_startup" => cfg.run_on_startup = v == "true" || v == "1",
                        "show_tray_icon" => cfg.show_tray_icon = v == "true" || v == "1",
                        "recent_colors" => {
                            let mut list = Vec::new();
                            for hex in v.split(',') {
                                if let Some(c) = RgbColor::from_hex(hex) {
                                    list.push(c);
                                }
                            }
                            if !list.is_empty() {
                                cfg.recent_colors = list;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        cfg
    }

    pub fn save(&self) {
        let path = Self::get_config_path();
        let colors_str = self
            .recent_colors
            .iter()
            .map(|c| c.to_hex())
            .collect::<Vec<_>>()
            .join(",");

        let content = format!(
            "[Settings]\nlanguage={}\nhotkey_mod={}\nhotkey_vk={}\nhotkey_name={}\nrun_on_startup={}\nshow_tray_icon={}\nrecent_colors={}\n",
            self.language.to_code(),
            self.hotkey_mod,
            self.hotkey_vk,
            self.hotkey_name,
            if self.run_on_startup { "true" } else { "false" },
            if self.show_tray_icon { "true" } else { "false" },
            colors_str
        );

        let _ = fs::write(path, content);
    }

    pub fn add_recent_color(&mut self, color: RgbColor) {
        self.recent_colors.retain(|c| c != &color);
        self.recent_colors.insert(0, color);
        if self.recent_colors.len() > 10 {
            self.recent_colors.truncate(10);
        }
        self.save();
    }

    pub fn apply_startup_registry(&self) {
        let run_key_wide: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run\0"
            .encode_utf16()
            .collect();
        let value_name_wide: Vec<u16> = "ETDPicker\0".encode_utf16().collect();

        unsafe {
            let mut hkey = std::ptr::null_mut();
            if RegOpenKeyExW(
                HKEY_CURRENT_USER,
                run_key_wide.as_ptr(),
                0,
                KEY_SET_VALUE,
                &mut hkey,
            ) == 0
            {
                if self.run_on_startup {
                    if let Ok(exe_path) = std::env::current_exe() {
                        let path_str = format!("\"{}\" --silent", exe_path.display());
                        let val_wide: Vec<u16> =
                            path_str.encode_utf16().chain(std::iter::once(0)).collect();
                        RegSetValueExW(
                            hkey,
                            value_name_wide.as_ptr(),
                            0,
                            REG_SZ,
                            val_wide.as_ptr() as *const _,
                            (val_wide.len() * 2) as u32,
                        );
                    }
                } else {
                    RegDeleteValueW(hkey, value_name_wide.as_ptr());
                }
                RegCloseKey(hkey);
            }
        }
    }
}
