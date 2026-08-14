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
    pub paint_hint: &'static str,

    // Paint Guide
    pub paint_guide_title: &'static str,
    pub paint_guide_step1: &'static str,
    pub paint_guide_step2: &'static str,
    pub paint_guide_step3: &'static str,
    pub paint_guide_step4: &'static str,
    pub paint_guide_step5: &'static str,
    pub paint_guide_close: &'static str,

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
    pub tray_paint_guide: &'static str,
    pub tray_exit: &'static str,
}

pub const TR_STRINGS: Strings = Strings {
    app_title: "ETDPicker - Ekran Renk Seçici",
    pick_color_btn: "🎯 Renk Seç",
    copy_hex_btn: "HEX Kopyala",
    copy_rgb_btn: "RGB Kopyala (Paint)",
    recent_colors: "Son Seçilen Renkler",
    copied_to_clipboard: "✓ Panoya Kopyalandı!",
    paint_hint: "Paint'te 'Renkleri Düzenle'ye girip bu R, G, B değerlerini yazabilirsiniz.",

    paint_guide_title: "Paint'te Renk Nasıl Kullanılır?",
    paint_guide_step1: "1. ETDPicker ile ekrandan istediğiniz rengi seçin.",
    paint_guide_step2: "2. 'RGB Kopyala (Paint)' butonuna veya R, G, B değerlerine bakın.",
    paint_guide_step3: "3. MS Paint uygulamasını açıp üst menüden 'Renkleri Düzenle'ye tıklayın.",
    paint_guide_step4: "4. Sağ alttaki Kırmızı (R), Yeşil (G), Mavi (B) kutularına bu sayıları yazın.",
    paint_guide_step5: "5. 'Özel Renklere Ekle' ardından 'Tamam' diyerek rengi hemen kullanın!",
    paint_guide_close: "Anladım, Kapat",

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
    tray_paint_guide: "🎨 Paint Rehberi",
    tray_exit: "Çıkış",
};

pub const EN_STRINGS: Strings = Strings {
    app_title: "ETDPicker - Screen Color Picker",
    pick_color_btn: "🎯 Pick Color",
    copy_hex_btn: "Copy HEX",
    copy_rgb_btn: "Copy RGB (Paint)",
    recent_colors: "Recent Colors",
    copied_to_clipboard: "✓ Copied to Clipboard!",
    paint_hint: "In Paint, open 'Edit Colors' and enter these R, G, B values.",

    paint_guide_title: "How to Use Colors in MS Paint?",
    paint_guide_step1: "1. Pick any color from your screen using ETDPicker.",
    paint_guide_step2: "2. Click 'Copy RGB (Paint)' or look at the R, G, B numbers.",
    paint_guide_step3: "3. Open MS Paint and click 'Edit Colors' in the top toolbar.",
    paint_guide_step4: "4. Type the numbers into the Red, Green, Blue boxes on the bottom right.",
    paint_guide_step5: "5. Click 'Add to Custom Colors', then 'OK' and start drawing!",
    paint_guide_close: "Got It, Close",

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
    tray_paint_guide: "🎨 Paint Guide",
    tray_exit: "Exit",
};

pub fn get_strings(lang: Language) -> &'static Strings {
    match lang {
        Language::Turkish => &TR_STRINGS,
        Language::English => &EN_STRINGS,
    }
}
