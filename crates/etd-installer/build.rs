fn main() {
    if std::env::var("TARGET").unwrap_or_default().contains("windows") {
        let mut res = winres::WindowsResource::new();

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let icon_path = std::path::Path::new(&manifest_dir)
            .join("../../assets/app_icon.ico")
            .canonicalize()
            .unwrap();

        res.set_icon(&icon_path.to_string_lossy());
        res.set("ProductName", "ETDPicker Setup");
        res.set("FileDescription", "ETDPicker Installer Wizard");
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
