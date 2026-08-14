use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    if env::var("TARGET").unwrap_or_default().contains("windows") {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let icon_path = Path::new(&manifest_dir)
            .join("../../assets/app_icon.ico")
            .canonicalize()
            .unwrap();

        let icon_path_str = icon_path.to_string_lossy().replace('\\', "/");
        let out_dir = env::var("OUT_DIR").unwrap();
        let rc_file = format!("{}/app_resources.rc", out_dir);
        let obj_file = format!("{}/app_resources.o", out_dir);

        let rc_content = format!(
            "1 ICON \"{}\"\n\
             1 VERSIONINFO\n\
             FILEVERSION 1,0,0,0\n\
             PRODUCTVERSION 1,0,0,0\n\
             FILEFLAGSMASK 0x3fL\n\
             FILEFLAGS 0x0L\n\
             FILEOS 0x40004L\n\
             FILETYPE 0x1L\n\
             FILESUBTYPE 0x0L\n\
             BEGIN\n\
                 BLOCK \"StringFileInfo\"\n\
                 BEGIN\n\
                     BLOCK \"040904b0\"\n\
                     BEGIN\n\
                         VALUE \"CompanyName\", \"Emir Tuğra Dağ\\0\"\n\
                         VALUE \"FileDescription\", \"ETDPicker Screen Color Picker\\0\"\n\
                         VALUE \"FileVersion\", \"1.0.0.0\\0\"\n\
                         VALUE \"InternalName\", \"ETDPicker\\0\"\n\
                         VALUE \"LegalCopyright\", \"Copyright (c) 2026 Emir Tuğra Dağ\\0\"\n\
                         VALUE \"OriginalFilename\", \"ETDPicker_Portable.exe\\0\"\n\
                         VALUE \"ProductName\", \"ETDPicker\\0\"\n\
                         VALUE \"ProductVersion\", \"1.0.0.0\\0\"\n\
                     END\n\
                 END\n\
                 BLOCK \"VarFileInfo\"\n\
                 BEGIN\n\
                     VALUE \"Translation\", 0x409, 1200\n\
                 END\n\
             END\n",
            icon_path_str
        );

        fs::write(&rc_file, rc_content).expect("Failed to write .rc file");

        let windres = "C:\\Users\\vboxuser\\Desktop\\mingw64\\bin\\windres.exe";
        let status = Command::new(windres)
            .args(["-i", &rc_file, "-o", &obj_file, "-O", "coff"])
            .status()
            .expect("Failed to execute windres");

        if !status.success() {
            panic!("windres failed with exit code {:?}", status.code());
        }

        println!("cargo:rustc-link-arg={}", obj_file);
        println!("cargo:rerun-if-changed={}", icon_path.display());
    }
}
