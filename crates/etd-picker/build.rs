fn main() {
    if std::env::var("TARGET").unwrap_or_default().contains("windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../../assets/app_icon.ico");
        res.set("ProductName", "ETDPicker");
        res.set("FileDescription", "ETDPicker Screen Color Picker");
        res.set("LegalCopyright", "Copyright (c) 2026 Emir Tuğra Dağ");
        res.set("CompanyName", "Emir Tuğra Dağ");

        let windres_candidate = "C:\\Users\\vboxuser\\Desktop\\mingw64\\bin\\windres.exe";
        let ar_candidate = "C:\\Users\\vboxuser\\Desktop\\mingw64\\bin\\ar.exe";
        if std::path::Path::new(windres_candidate).exists() {
            res.set_windres_path(windres_candidate);
        }
        if std::path::Path::new(ar_candidate).exists() {
            res.set_ar_path(ar_candidate);
        }

        res.compile().expect("Failed to compile Windows resources");
    }
}
