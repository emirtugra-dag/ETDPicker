#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Turkish,
    English,
}

impl Language {
    pub fn from_code(code: &str) -> Self {
        match code.to_lowercase().as_str() {
            "tr" | "turkish" | "turkce" => Language::Turkish,
            _ => Language::English,
        }
    }

    pub fn to_code(&self) -> &'static str {
        match self {
            Language::Turkish => "tr",
            Language::English => "en",
        }
    }
}

pub struct Strings {
    pub app_title: &'static str,
    pub pick_color_btn: &'static str,
    pub copy_hex_btn: &'static str,
    pub copy_rgb_btn: &'static str,
    pub recent_colors: &'static str,
    pub copied_to_clipboard: &'static str,
    pub guide_hint: &'static str,

    // Color Guide
    pub color_guide_title: &'static str,
    pub color_guide_step1: &'static str,
    pub color_guide_step2: &'static str,
    pub color_guide_step3: &'static str,
    pub color_guide_step4: &'static str,
    pub color_guide_step5: &'static str,
    pub color_guide_close: &'static str,

    // Settings
    pub settings_title: &'static str,
    pub lang_label: &'static str,
    pub startup_label: &'static str,
    pub tray_label: &'static str,
    pub hotkey_label: &'static str,
    pub save_btn: &'static str,
    pub cancel_btn: &'static str,
    pub exit_app_btn: &'static str,

    // Tray Menu
    pub tray_pick: &'static str,
    pub tray_show: &'static str,
    pub tray_settings: &'static str,
    pub tray_color_guide: &'static str,
    pub tray_exit: &'static str,
}

pub const TR_STRINGS: Strings = Strings {
    app_title: "ETDPicker - Ekran Renk Seçici",
    pick_color_btn: "🎯 Renk Seç",
    copy_hex_btn: "HEX Kopyala",
    copy_rgb_btn: "RGB Kopyala",
    recent_colors: "Son Seçilen Renkler",
    copied_to_clipboard: "✓ Panoya Kopyalandı!",
    guide_hint: "Bu R, G, B değerlerini çizim ve tasarım uygulamalarında kullanabilirsiniz.",

    color_guide_title: "Renk Formatları ve Kullanım Kılavuzu",
    color_guide_step1: "1. ETDPicker ile ekrandan istediğiniz rengi seçin.",
    color_guide_step2: "2. Renk otomatik olarak panonuza HEX formatında (#RRGGBB) kopyalanır.",
    color_guide_step3: "3. 'RGB Kopyala' butonuna tıklayarak doğrudan R, G, B formatında alabilirsiniz.",
    color_guide_step4: "4. Çizim ve tasarım uygulamalarının renk paletine bu R, G, B sayılarını girin.",
    color_guide_step5: "5. Seçtiğiniz son 10 renk alt kısımdaki geçmiş paletinde saklanır.",
    color_guide_close: "Anladım, Kapat",

    settings_title: "ETDPicker - Ayarlar",
    lang_label: "Uygulama Dili:",
    startup_label: "Windows açıldığında otomatik başlat",
    tray_label: "Sistem tepsisinde simgeyi göster",
    hotkey_label: "Renk Seçme Kısayol Tuşu:",
    save_btn: "Kaydet",
    cancel_btn: "İptal",
    exit_app_btn: "Uygulamayı Tamamen Kapat",

    tray_pick: "🎯 Renk Seç",
    tray_show: "Pencereyi Göster",
    tray_settings: "⚙️ Ayarlar",
    tray_color_guide: "🎨 Renk Kılavuzu",
    tray_exit: "Çıkış",
};

pub const EN_STRINGS: Strings = Strings {
    app_title: "ETDPicker - Screen Color Picker",
    pick_color_btn: "🎯 Pick Color",
    copy_hex_btn: "Copy HEX",
    copy_rgb_btn: "Copy RGB",
    recent_colors: "Recent Colors",
    copied_to_clipboard: "✓ Copied to Clipboard!",
    guide_hint: "You can use these R, G, B values in drawing and design applications.",

    color_guide_title: "Color Formats & Usage Guide",
    color_guide_step1: "1. Pick any color from your screen using ETDPicker.",
    color_guide_step2: "2. The color is automatically copied to your clipboard in HEX format (#RRGGBB).",
    color_guide_step3: "3. Click 'Copy RGB' to copy in standard R, G, B format.",
    color_guide_step4: "4. Enter these R, G, B numbers into any design application's color palette.",
    color_guide_step5: "5. Your last 10 picked colors are saved in the recent history palette.",
    color_guide_close: "Got It, Close",

    settings_title: "ETDPicker - Settings",
    lang_label: "Application Language:",
    startup_label: "Launch automatically on Windows startup",
    tray_label: "Show icon in system tray",
    hotkey_label: "Color Picker Hotkey:",
    save_btn: "Save",
    cancel_btn: "Cancel",
    exit_app_btn: "Exit Application Completely",

    tray_pick: "🎯 Pick Color",
    tray_show: "Show Window",
    tray_settings: "⚙️ Settings",
    tray_color_guide: "🎨 Color Guide",
    tray_exit: "Exit",
};

pub fn get_strings(lang: Language) -> &'static Strings {
    match lang {
        Language::Turkish => &TR_STRINGS,
        Language::English => &EN_STRINGS,
    }
}
