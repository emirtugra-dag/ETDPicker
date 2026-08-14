#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Turkish,
    English,
}

pub struct InstallerStrings {
    pub wizard_title: &'static str,
    pub step1_title: &'static str,
    pub step1_desc: &'static str,
    pub license_agree: &'static str,
    pub step2_title: &'static str,
    pub step2_desc: &'static str,
    pub browse_btn: &'static str,
    pub step3_title: &'static str,
    pub step3_desc: &'static str,
    pub opt_desktop: &'static str,
    pub opt_startmenu: &'static str,
    pub opt_startup: &'static str,
    pub step4_title: &'static str,
    pub step4_desc: &'static str,
    pub step5_title: &'static str,
    pub step5_desc: &'static str,
    pub opt_launch: &'static str,
    pub next_btn: &'static str,
    pub back_btn: &'static str,
    pub install_btn: &'static str,
    pub finish_btn: &'static str,
    pub cancel_btn: &'static str,

    // Uninstall
    pub uninst_title: &'static str,
    pub uninst_prompt: &'static str,
    pub uninst_btn: &'static str,
    pub uninst_success: &'static str,
}

pub const TR_STRINGS: InstallerStrings = InstallerStrings {
    wizard_title: "ETDPicker Kurulum Sihirbazı",
    step1_title: "Hoş Geldiniz ve Yasal Bilgilendirme",
    step1_desc: "ETDPicker kurulumuna hoş geldiniz.\n\n• Kod tabanı MIT Lisansı ile sunulmaktadır.\n• ETDPicker adı ve logosu Emir Tuğra Dağ'ın fikri mülkiyetidir.\n• Program 'OLDUĞU GİBİ' sağlanmakta olup, kullanımından doğacak her türlü durumdan kullanıcının kendisi mesuldür.",
    license_agree: "Lisans, Fikri Mülkiyet ve Sorumluluk Reddi şartlarını kabul ediyorum.",
    step2_title: "Hedef Kurulum Klasörü",
    step2_desc: "ETDPicker aşağıdaki klasöre kurulacak. Farklı bir konuma kurmak için 'Gözat' butonuna tıklayabilirsiniz:",
    browse_btn: "Gözat...",
    step3_title: "Ek Görevler ve Kısayollar",
    step3_desc: "Kurulum sırasında yapılmasını istediğiniz ek seçenekleri işaretleyin:",
    opt_desktop: "Masaüstü kısayolu oluştur",
    opt_startmenu: "Başlat Menüsü kısayolu oluştur",
    opt_startup: "Windows açıldığında ETDPicker'ı otomatik başlat",
    step4_title: "Kuruluyor...",
    step4_desc: "ETDPicker dosyaları kopyalanıyor ve sistem kayıtları yapılıyor...",
    step5_title: "Kurulum Tamamlandı!",
    step5_desc: "ETDPicker bilgisayarınıza başarıyla kuruldu.",
    opt_launch: "ETDPicker uygulamasını hemen başlat",
    next_btn: "İleri >",
    back_btn: "< Geri",
    install_btn: "Kur",
    finish_btn: "Bitir",
    cancel_btn: "İptal",

    uninst_title: "ETDPicker Kaldırma Sihirbazı",
    uninst_prompt: "ETDPicker uygulamasını ve tüm bileşenlerini bilgisayarınızdan kaldırmak istediğinize emin misiniz?",
    uninst_btn: "Kaldır",
    uninst_success: "ETDPicker bilgisayarınızdan başarıyla kaldırıldı.",
};

pub const EN_STRINGS: InstallerStrings = InstallerStrings {
    wizard_title: "ETDPicker Setup Wizard",
    step1_title: "Welcome & Legal Information",
    step1_desc: "Welcome to ETDPicker Setup.\n\n• The codebase is licensed under the MIT License.\n• ETDPicker name and logo are the intellectual property of Emir Tuğra Dağ.\n• The software is provided 'AS IS', and users bear all responsibility for its use.",
    license_agree: "I accept the License, Intellectual Property, and Disclaimer terms.",
    step2_title: "Destination Directory",
    step2_desc: "ETDPicker will be installed in the following folder. To choose a different location, click 'Browse':",
    browse_btn: "Browse...",
    step3_title: "Additional Tasks & Shortcuts",
    step3_desc: "Select the additional shortcuts you would like Setup to create:",
    opt_desktop: "Create a desktop shortcut",
    opt_startmenu: "Create a Start Menu shortcut",
    opt_startup: "Launch ETDPicker automatically on Windows startup",
    step4_title: "Installing...",
    step4_desc: "Copying ETDPicker files and registering system entries...",
    step5_title: "Installation Completed!",
    step5_desc: "ETDPicker has been successfully installed on your computer.",
    opt_launch: "Launch ETDPicker now",
    next_btn: "Next >",
    back_btn: "< Back",
    install_btn: "Install",
    finish_btn: "Finish",
    cancel_btn: "Cancel",

    uninst_title: "ETDPicker Uninstall Wizard",
    uninst_prompt: "Are you sure you want to completely remove ETDPicker and all of its components?",
    uninst_btn: "Uninstall",
    uninst_success: "ETDPicker was successfully removed from your computer.",
};

pub fn get_installer_strings(lang: Language) -> &'static InstallerStrings {
    match lang {
        Language::Turkish => &TR_STRINGS,
        Language::English => &EN_STRINGS,
    }
}
