fn main() {
    if std::env::var("TARGET").unwrap_or_default().contains("windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../../assets/app_icon.ico");
        res.set("ProductName", "ETDPicker");
        res.set("FileDescription", "ETDPicker Screen Color Picker");
        res.set("LegalCopyright", "Copyright (c) 2026 Emir Tuğra Dağ");
        res.set("CompanyName", "Emir Tuğra Dağ");
        let _ = res.compile();
    }
}
