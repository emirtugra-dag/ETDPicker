#![windows_subsystem = "windows"]

mod i18n;
mod registry;
mod shortcut;

use i18n::{get_installer_strings, Language};
use std::fs;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, FillRect, FrameRect,
    SelectObject, SetBkMode, SetTextColor, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE,
    DT_WORDBREAK, FW_BOLD, FW_NORMAL, TRANSPARENT,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::EnableWindow;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
    GetSystemMetrics, GetWindowTextW, IsDialogMessageW, LoadCursorW, LoadIconW,
    PostQuitMessage, RegisterClassW, SendMessageW, SetWindowTextW, ShowWindow,
    TranslateMessage, BM_GETCHECK, BM_SETCHECK, CB_ADDSTRING, CB_GETCURSEL,
    CB_SETCURSEL, CS_HREDRAW, CS_VREDRAW, IDC_ARROW, MSG, SM_CXSCREEN, SM_CYSCREEN,
    SW_HIDE, SW_SHOW, WM_COMMAND, WM_CREATE, WM_DESTROY, WM_PAINT, WNDCLASSW,
    WS_CHILD, WS_CLIPCHILDREN, WS_EX_DLGMODALFRAME, WS_EX_TOPMOST, WS_MINIMIZEBOX,
    WS_OVERLAPPED, WS_POPUP, WS_SYSMENU, WS_TABSTOP, WS_VISIBLE,
};

const CREATE_NO_WINDOW: u32 = 0x08000000;

// Embedded full assets
const ASSET_PICKER_EXE: &[u8] = include_bytes!("../../../target/release/etd-picker.exe");
const ASSET_ICON: &[u8] = include_bytes!("../../../assets/app_icon.ico");
const ASSET_LICENSE: &[u8] = include_bytes!("../../../LICENSE");
const ASSET_DISCLAIMER: &[u8] = include_bytes!("../../../DISCLAIMER.md");
const ASSET_README: &[u8] = include_bytes!("../../../README.md");
const ASSET_PAINT_GUIDE: &[u8] = include_bytes!("../../../docs/PAINT_GUIDE.md");

// State
static CURRENT_STEP: AtomicUsize = AtomicUsize::new(1);
static CURRENT_LANG: AtomicUsize = AtomicUsize::new(0); // 0 = Turkish, 1 = English
static IS_UNINSTALLER: AtomicBool = AtomicBool::new(false);

static mut MAIN_HWND: HWND = 0 as _;
static mut CHK_AGREE: HWND = 0 as _;
static mut EDT_DIR: HWND = 0 as _;
static mut BTN_BROWSE: HWND = 0 as _;
static mut CHK_DESKTOP: HWND = 0 as _;
static mut CHK_STARTMENU: HWND = 0 as _;
static mut CHK_STARTUP: HWND = 0 as _;
static mut CHK_LAUNCH: HWND = 0 as _;
static mut BTN_BACK: HWND = 0 as _;
static mut BTN_NEXT: HWND = 0 as _;
static mut BTN_CANCEL: HWND = 0 as _;

// Language prompt controls
static mut LANG_COMBO: HWND = 0 as _;
static mut LANG_SELECTED: Option<Language> = None;

fn get_active_lang() -> Language {
    if CURRENT_LANG.load(Ordering::SeqCst) == 0 {
        Language::Turkish
    } else {
        Language::English
    }
}

fn get_default_install_dir() -> PathBuf {
    let uninst_key: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\ETDPicker\0"
        .encode_utf16()
        .collect();
    let val_name: Vec<u16> = "InstallLocation\0".encode_utf16().collect();

    unsafe {
        let mut hkey = std::ptr::null_mut();
        if windows_sys::Win32::System::Registry::RegOpenKeyExW(
            windows_sys::Win32::System::Registry::HKEY_CURRENT_USER,
            uninst_key.as_ptr(),
            0,
            windows_sys::Win32::System::Registry::KEY_READ,
            &mut hkey,
        ) == 0
        {
            let mut buf = [0u16; 512];
            let mut size = (buf.len() * 2) as u32;
            let mut type_val = 0;
            if windows_sys::Win32::System::Registry::RegQueryValueExW(
                hkey,
                val_name.as_ptr(),
                std::ptr::null_mut(),
                &mut type_val,
                buf.as_mut_ptr() as *mut _,
                &mut size,
            ) == 0
            {
                let len = (size / 2) as usize;
                let path_str = String::from_utf16_lossy(&buf[..len.saturating_sub(1)]);
                if !path_str.trim().is_empty() {
                    windows_sys::Win32::System::Registry::RegCloseKey(hkey);
                    return PathBuf::from(path_str.trim());
                }
            }
            windows_sys::Win32::System::Registry::RegCloseKey(hkey);
        }
    }

    if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(appdata).join("Programs").join("ETDPicker")
    } else if let Ok(prog) = std::env::var("ProgramFiles") {
        PathBuf::from(prog).join("ETDPicker")
    } else {
        PathBuf::from("C:\\Program Files\\ETDPicker")
    }
}

unsafe fn prompt_setup_language() -> Option<Language> {
    LANG_SELECTED = None;

    let class_name: Vec<u16> = "ETDInstallerLanguagePrompt\0".encode_utf16().collect();
    let hinstance = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null());

    let icon = LoadIconW(hinstance, 1 as _);
    let cursor = LoadCursorW(0 as _, IDC_ARROW);

    let wc = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(lang_prompt_wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: icon,
        hCursor: cursor,
        hbrBackground: CreateSolidBrush(0x001A1A1A) as _,
        lpszMenuName: std::ptr::null(),
        lpszClassName: class_name.as_ptr(),
    };
    RegisterClassW(&wc);

    let width = 390;
    let height = 210;
    let screen_w = GetSystemMetrics(SM_CXSCREEN);
    let screen_h = GetSystemMetrics(SM_CYSCREEN);
    let x = (screen_w - width) / 2;
    let y = (screen_h - height) / 2;

    let title_wide: Vec<u16> = "ETDPicker Setup - Language / Dil\0".encode_utf16().collect();

    let hwnd = CreateWindowExW(
        WS_EX_TOPMOST | WS_EX_DLGMODALFRAME,
        class_name.as_ptr(),
        title_wide.as_ptr(),
        WS_POPUP | WS_VISIBLE,
        x,
        y,
        width,
        height,
        0 as _,
        0 as _,
        hinstance,
        std::ptr::null_mut(),
    );

    let font_ui = CreateFontW(
        15, 0, 0, 0, FW_NORMAL as _, 0, 0, 0, 1, 0, 0, 0, 0,
        "Segoe UI\0".encode_utf16().collect::<Vec<_>>().as_ptr(),
    );

    let combo_class: Vec<u16> = "COMBOBOX\0".encode_utf16().collect();
    let btn_class: Vec<u16> = "BUTTON\0".encode_utf16().collect();

    LANG_COMBO = CreateWindowExW(
        0,
        combo_class.as_ptr(),
        std::ptr::null(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x0002 | 0x0200,
        30,
        78,
        320,
        150,
        hwnd,
        1001 as _,
        hinstance,
        std::ptr::null_mut(),
    );
    SendMessageW(LANG_COMBO, 0x0030, font_ui as _, 1);

    let tr_str: Vec<u16> = "Türkçe (Turkish)\0".encode_utf16().collect();
    let en_str: Vec<u16> = "English (İngilizce)\0".encode_utf16().collect();
    SendMessageW(LANG_COMBO, CB_ADDSTRING, 0, tr_str.as_ptr() as _);
    SendMessageW(LANG_COMBO, CB_ADDSTRING, 0, en_str.as_ptr() as _);
    SendMessageW(LANG_COMBO, CB_SETCURSEL, 0, 0);

    let ok_text: Vec<u16> = "Tamam / OK\0".encode_utf16().collect();
    CreateWindowExW(
        0,
        btn_class.as_ptr(),
        ok_text.as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x00000001,
        135,
        138,
        105,
        34,
        hwnd,
        1002 as _,
        hinstance,
        std::ptr::null_mut(),
    );

    let cancel_text: Vec<u16> = "İptal / Cancel\0".encode_utf16().collect();
    CreateWindowExW(
        0,
        btn_class.as_ptr(),
        cancel_text.as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP,
        250,
        138,
        100,
        34,
        hwnd,
        1003 as _,
        hinstance,
        std::ptr::null_mut(),
    );

    ShowWindow(hwnd, SW_SHOW);

    let mut msg: MSG = std::mem::zeroed();
    while GetMessageW(&mut msg, 0 as _, 0, 0) > 0 {
        if IsDialogMessageW(hwnd, &msg) == 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        if windows_sys::Win32::UI::WindowsAndMessaging::IsWindow(hwnd) == 0 {
            break;
        }
    }

    DeleteObject(font_ui as _);
    LANG_SELECTED
}

unsafe extern "system" fn lang_prompt_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as i32;
            if id == 1002 {
                let sel = SendMessageW(LANG_COMBO, CB_GETCURSEL, 0, 0);
                LANG_SELECTED = Some(if sel == 0 { Language::Turkish } else { Language::English });
                DestroyWindow(hwnd);
            } else if id == 1003 || id == 2 {
                LANG_SELECTED = None;
                DestroyWindow(hwnd);
            }
            0
        }
        WM_PAINT => {
            let mut ps = std::mem::zeroed();
            let hdc = windows_sys::Win32::Graphics::Gdi::BeginPaint(hwnd, &mut ps);

            let bg_brush = CreateSolidBrush(0x001B1A1A);
            let full_rc = RECT { left: 0, top: 0, right: 390, bottom: 210 };
            FillRect(hdc, &full_rc, bg_brush as _);
            DeleteObject(bg_brush as _);

            let border_brush = CreateSolidBrush(0x004A4A4A);
            FrameRect(hdc, &full_rc, border_brush as _);
            DeleteObject(border_brush as _);

            SetBkMode(hdc, TRANSPARENT as _);

            let font_title = CreateFontW(
                17, 0, 0, 0, FW_BOLD as _, 0, 0, 0, 1, 0, 0, 0, 0,
                "Segoe UI\0".encode_utf16().collect::<Vec<_>>().as_ptr(),
            );
            SelectObject(hdc, font_title as _);
            SetTextColor(hdc, 0x00FFFFFF);

            let mut t_wide: Vec<u16> = "Kurulum Dilini Seçin / Select Language".encode_utf16().collect();
            let mut t_rc = RECT { left: 30, top: 18, right: 360, bottom: 44 };
            DrawTextW(hdc, t_wide.as_mut_ptr(), t_wide.len() as _, &mut t_rc, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);

            DeleteObject(font_title as _);

            let font_label = CreateFontW(
                13, 0, 0, 0, FW_NORMAL as _, 0, 0, 0, 1, 0, 0, 0, 0,
                "Segoe UI\0".encode_utf16().collect::<Vec<_>>().as_ptr(),
            );
            SelectObject(hdc, font_label as _);
            SetTextColor(hdc, 0x00CCCCCC);

            let mut p_wide: Vec<u16> = "Lütfen kurulum dilini seçin / Please choose language:".encode_utf16().collect();
            let mut p_rc = RECT { left: 30, top: 48, right: 360, bottom: 70 };
            DrawTextW(hdc, p_wide.as_mut_ptr(), p_wide.len() as _, &mut p_rc, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);

            DeleteObject(font_label as _);

            windows_sys::Win32::Graphics::Gdi::EndPaint(hwnd, &ps);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn main() {
    let exe_name = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_lowercase()))
        .unwrap_or_default();

    let is_uninst_arg = std::env::args().any(|a| a == "--uninstall" || a == "-u");
    let is_uninst_name = exe_name.contains("uninstall");

    if is_uninst_arg || is_uninst_name {
        IS_UNINSTALLER.store(true, Ordering::SeqCst);
    } else {
        // Prompt for language at the very beginning of the installation wizard!
        let chosen_lang = unsafe { prompt_setup_language() };
        match chosen_lang {
            Some(Language::Turkish) => CURRENT_LANG.store(0, Ordering::SeqCst),
            Some(Language::English) => CURRENT_LANG.store(1, Ordering::SeqCst),
            None => return, // User cancelled language selection dialog
        }
    }

    unsafe {
        let class_name: Vec<u16> = "ETDInstallerWizardWindow\0".encode_utf16().collect();
        let hinstance = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null());

        let icon = LoadIconW(hinstance, 1 as _);
        let cursor = LoadCursorW(0 as _, IDC_ARROW);

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wizard_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: icon,
            hCursor: cursor,
            hbrBackground: CreateSolidBrush(0x001A1A1A) as _,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
        };
        RegisterClassW(&wc);

        let width = 560;
        let height = 440;
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let x = (screen_w - width) / 2;
        let y = (screen_h - height) / 2;

        let lang = get_active_lang();
        let title = if IS_UNINSTALLER.load(Ordering::SeqCst) {
            get_installer_strings(lang).uninst_title
        } else {
            get_installer_strings(lang).wizard_title
        };
        let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            title_wide.as_ptr(),
            WS_OVERLAPPED | WS_SYSMENU | WS_MINIMIZEBOX | WS_VISIBLE | WS_CLIPCHILDREN,
            x,
            y,
            width,
            height,
            0 as _,
            0 as _,
            hinstance,
            std::ptr::null_mut(),
        );

        MAIN_HWND = hwnd;

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0 as _, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

unsafe fn create_step_controls(hwnd: HWND) {
    let hinstance = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null());
    let btn_class: Vec<u16> = "BUTTON\0".encode_utf16().collect();
    let edit_class: Vec<u16> = "EDIT\0".encode_utf16().collect();

    let font_ui = CreateFontW(
        15, 0, 0, 0, FW_NORMAL as _, 0, 0, 0, 1, 0, 0, 0, 0,
        "Segoe UI\0".encode_utf16().collect::<Vec<_>>().as_ptr(),
    );

    let lang = get_active_lang();
    let strings = get_installer_strings(lang);

    // Step 1: License agreement checkbox
    let agree_text: Vec<u16> = strings.license_agree.encode_utf16().chain(std::iter::once(0)).collect();
    CHK_AGREE = CreateWindowExW(
        0,
        btn_class.as_ptr(),
        agree_text.as_ptr(),
        WS_CHILD | WS_TABSTOP | 0x00000003,
        30,
        280,
        500,
        30,
        hwnd,
        101 as _,
        hinstance,
        std::ptr::null_mut(),
    );
    SendMessageW(CHK_AGREE, 0x0030, font_ui as _, 1);

    // Step 2: Directory edit + Browse button
    let def_path = get_default_install_dir();
    let path_str: Vec<u16> = def_path.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();
    EDT_DIR = CreateWindowExW(
        0x00000200,
        edit_class.as_ptr(),
        path_str.as_ptr(),
        WS_CHILD | WS_TABSTOP | 0x0080,
        30,
        210,
        380,
        28,
        hwnd,
        102 as _,
        hinstance,
        std::ptr::null_mut(),
    );
    SendMessageW(EDT_DIR, 0x0030, font_ui as _, 1);

    let browse_text: Vec<u16> = strings.browse_btn.encode_utf16().chain(std::iter::once(0)).collect();
    BTN_BROWSE = CreateWindowExW(
        0,
        btn_class.as_ptr(),
        browse_text.as_ptr(),
        WS_CHILD | WS_TABSTOP,
        420,
        209,
        100,
        30,
        hwnd,
        103 as _,
        hinstance,
        std::ptr::null_mut(),
    );
    SendMessageW(BTN_BROWSE, 0x0030, font_ui as _, 1);

    // Step 3: Checkboxes (Desktop, StartMenu, Startup)
    let d_text: Vec<u16> = strings.opt_desktop.encode_utf16().chain(std::iter::once(0)).collect();
    CHK_DESKTOP = CreateWindowExW(
        0,
        btn_class.as_ptr(),
        d_text.as_ptr(),
        WS_CHILD | WS_TABSTOP | 0x00000003,
        30,
        180,
        450,
        28,
        hwnd,
        104 as _,
        hinstance,
        std::ptr::null_mut(),
    );
    SendMessageW(CHK_DESKTOP, 0x0030, font_ui as _, 1);
    SendMessageW(CHK_DESKTOP, BM_SETCHECK, 1, 0);

    let sm_text: Vec<u16> = strings.opt_startmenu.encode_utf16().chain(std::iter::once(0)).collect();
    CHK_STARTMENU = CreateWindowExW(
        0,
        btn_class.as_ptr(),
        sm_text.as_ptr(),
        WS_CHILD | WS_TABSTOP | 0x00000003,
        30,
        215,
        450,
        28,
        hwnd,
        105 as _,
        hinstance,
        std::ptr::null_mut(),
    );
    SendMessageW(CHK_STARTMENU, 0x0030, font_ui as _, 1);
    SendMessageW(CHK_STARTMENU, BM_SETCHECK, 1, 0);

    let su_text: Vec<u16> = strings.opt_startup.encode_utf16().chain(std::iter::once(0)).collect();
    CHK_STARTUP = CreateWindowExW(
        0,
        btn_class.as_ptr(),
        su_text.as_ptr(),
        WS_CHILD | WS_TABSTOP | 0x00000003,
        30,
        250,
        450,
        28,
        hwnd,
        106 as _,
        hinstance,
        std::ptr::null_mut(),
    );
    SendMessageW(CHK_STARTUP, 0x0030, font_ui as _, 1);
    SendMessageW(CHK_STARTUP, BM_SETCHECK, 0, 0);

    // Step 5: Launch checkbox
    let l_text: Vec<u16> = strings.opt_launch.encode_utf16().chain(std::iter::once(0)).collect();
    CHK_LAUNCH = CreateWindowExW(
        0,
        btn_class.as_ptr(),
        l_text.as_ptr(),
        WS_CHILD | WS_TABSTOP | 0x00000003,
        30,
        220,
        450,
        28,
        hwnd,
        107 as _,
        hinstance,
        std::ptr::null_mut(),
    );
    SendMessageW(CHK_LAUNCH, 0x0030, font_ui as _, 1);
    SendMessageW(CHK_LAUNCH, BM_SETCHECK, 1, 0);

    // Navigation buttons
    let back_t: Vec<u16> = strings.back_btn.encode_utf16().chain(std::iter::once(0)).collect();
    BTN_BACK = CreateWindowExW(
        0,
        btn_class.as_ptr(),
        back_t.as_ptr(),
        WS_CHILD | WS_TABSTOP | WS_VISIBLE,
        230,
        350,
        95,
        36,
        hwnd,
        201 as _,
        hinstance,
        std::ptr::null_mut(),
    );
    SendMessageW(BTN_BACK, 0x0030, font_ui as _, 1);

    let next_t: Vec<u16> = strings.next_btn.encode_utf16().chain(std::iter::once(0)).collect();
    BTN_NEXT = CreateWindowExW(
        0,
        btn_class.as_ptr(),
        next_t.as_ptr(),
        WS_CHILD | WS_TABSTOP | WS_VISIBLE | 0x00000001,
        335,
        350,
        95,
        36,
        hwnd,
        202 as _,
        hinstance,
        std::ptr::null_mut(),
    );
    SendMessageW(BTN_NEXT, 0x0030, font_ui as _, 1);

    let cancel_t: Vec<u16> = strings.cancel_btn.encode_utf16().chain(std::iter::once(0)).collect();
    BTN_CANCEL = CreateWindowExW(
        0,
        btn_class.as_ptr(),
        cancel_t.as_ptr(),
        WS_CHILD | WS_TABSTOP | WS_VISIBLE,
        440,
        350,
        95,
        36,
        hwnd,
        203 as _,
        hinstance,
        std::ptr::null_mut(),
    );
    SendMessageW(BTN_CANCEL, 0x0030, font_ui as _, 1);

    update_control_visibility();
}

unsafe fn update_control_visibility() {
    let step = CURRENT_STEP.load(Ordering::SeqCst);
    let lang = get_active_lang();
    let strings = get_installer_strings(lang);
    let is_uninstaller = IS_UNINSTALLER.load(Ordering::SeqCst);

    if is_uninstaller {
        ShowWindow(CHK_AGREE, SW_HIDE);
        ShowWindow(EDT_DIR, SW_HIDE);
        ShowWindow(BTN_BROWSE, SW_HIDE);
        ShowWindow(CHK_DESKTOP, SW_HIDE);
        ShowWindow(CHK_STARTMENU, SW_HIDE);
        ShowWindow(CHK_STARTUP, SW_HIDE);
        ShowWindow(CHK_LAUNCH, SW_HIDE);
        ShowWindow(BTN_BACK, SW_HIDE);

        let uninst_t: Vec<u16> = strings.uninst_btn.encode_utf16().chain(std::iter::once(0)).collect();
        SetWindowTextW(BTN_NEXT, uninst_t.as_ptr());
        return;
    }

    ShowWindow(CHK_AGREE, if step == 1 { SW_SHOW } else { SW_HIDE });
    ShowWindow(EDT_DIR, if step == 2 { SW_SHOW } else { SW_HIDE });
    ShowWindow(BTN_BROWSE, if step == 2 { SW_SHOW } else { SW_HIDE });
    ShowWindow(CHK_DESKTOP, if step == 3 { SW_SHOW } else { SW_HIDE });
    ShowWindow(CHK_STARTMENU, if step == 3 { SW_SHOW } else { SW_HIDE });
    ShowWindow(CHK_STARTUP, if step == 3 { SW_SHOW } else { SW_HIDE });
    ShowWindow(CHK_LAUNCH, if step == 5 { SW_SHOW } else { SW_HIDE });

    EnableWindow(BTN_BACK, if step > 1 && step < 5 { 1 } else { 0 });

    let next_lbl = match step {
        3 => strings.install_btn,
        5 => strings.finish_btn,
        _ => strings.next_btn,
    };
    let next_t: Vec<u16> = next_lbl.encode_utf16().chain(std::iter::once(0)).collect();
    SetWindowTextW(BTN_NEXT, next_t.as_ptr());

    if step == 1 {
        let is_checked = SendMessageW(CHK_AGREE, BM_GETCHECK, 0, 0);
        EnableWindow(BTN_NEXT, if is_checked == 1 { 1 } else { 0 });
    } else {
        EnableWindow(BTN_NEXT, 1);
    }
}

unsafe fn perform_installation(_hwnd: HWND) {
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "ETDPicker.exe"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    let mut dir_buf = [0u16; 512];
    let len = GetWindowTextW(EDT_DIR, dir_buf.as_mut_ptr(), 512);
    let install_dir_str = String::from_utf16_lossy(&dir_buf[..len as usize]);
    let install_dir = PathBuf::from(install_dir_str);

    let _ = fs::create_dir_all(&install_dir);

    let target_exe = install_dir.join("ETDPicker.exe");
    let target_icon = install_dir.join("app_icon.ico");
    let target_uninst = install_dir.join("Uninstall.exe");
    let target_license = install_dir.join("LICENSE");
    let target_disclaimer = install_dir.join("DISCLAIMER.md");
    let target_readme = install_dir.join("README.md");
    let target_guide = install_dir.join("PAINT_GUIDE.md");

    let _ = fs::write(&target_exe, ASSET_PICKER_EXE);

    if let Ok(curr_exe) = std::env::current_exe() {
        let _ = fs::copy(&curr_exe, &target_uninst);
    }

    let _ = fs::write(&target_icon, ASSET_ICON);
    let _ = fs::write(&target_license, ASSET_LICENSE);
    let _ = fs::write(&target_disclaimer, ASSET_DISCLAIMER);
    let _ = fs::write(&target_readme, ASSET_README);
    let _ = fs::write(&target_guide, ASSET_PAINT_GUIDE);

    let config_ini = install_dir.join("config.ini");
    if !config_ini.exists() {
        let lang = get_active_lang();
        let cfg_content = format!(
            "[Settings]\nlanguage={}\nhotkey_mod=1\nhotkey_vk=80\nhotkey_name=Alt + P\nrun_on_startup=false\nshow_tray_icon=true\nrecent_colors=#3498db,#2ecc71,#e74c3c,#f1c40f,#9b59b6\n",
            lang.to_code()
        );
        let _ = fs::write(config_ini, cfg_content);
    }

    registry::register_uninstaller(&install_dir);

    let make_desktop = SendMessageW(CHK_DESKTOP, BM_GETCHECK, 0, 0) == 1;
    let make_startmenu = SendMessageW(CHK_STARTMENU, BM_GETCHECK, 0, 0) == 1;
    let make_startup = SendMessageW(CHK_STARTUP, BM_GETCHECK, 0, 0) == 1;

    if make_desktop {
        if let Some(desktop) = shortcut::get_desktop_dir() {
            let link = desktop.join("ETDPicker.lnk");
            shortcut::create_shortcut(&target_exe, &link, &target_icon, "ETDPicker Screen Color Picker");
        }
    }

    if make_startmenu {
        if let Some(startmenu) = shortcut::get_start_menu_dir() {
            let link = startmenu.join("ETDPicker.lnk");
            shortcut::create_shortcut(&target_exe, &link, &target_icon, "ETDPicker Screen Color Picker");
        }
    }

    if make_startup {
        let run_key_wide: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run\0"
            .encode_utf16()
            .collect();
        let value_name_wide: Vec<u16> = "ETDPicker\0".encode_utf16().collect();
        let path_str = format!("\"{}\" --silent", target_exe.display());
        let val_wide: Vec<u16> = path_str.encode_utf16().chain(std::iter::once(0)).collect();

        let mut hkey = std::ptr::null_mut();
        if windows_sys::Win32::System::Registry::RegOpenKeyExW(
            windows_sys::Win32::System::Registry::HKEY_CURRENT_USER,
            run_key_wide.as_ptr(),
            0,
            windows_sys::Win32::System::Registry::KEY_SET_VALUE,
            &mut hkey,
        ) == 0
        {
            windows_sys::Win32::System::Registry::RegSetValueExW(
                hkey,
                value_name_wide.as_ptr(),
                0,
                windows_sys::Win32::System::Registry::REG_SZ,
                val_wide.as_ptr() as *const _,
                (val_wide.len() * 2) as u32,
            );
            windows_sys::Win32::System::Registry::RegCloseKey(hkey);
        }
    }
}

unsafe fn perform_uninstallation() {
    registry::unregister_uninstaller();

    if let Some(desktop) = shortcut::get_desktop_dir() {
        let _ = fs::remove_file(desktop.join("ETDPicker.lnk"));
    }
    if let Some(startmenu) = shortcut::get_start_menu_dir() {
        let _ = fs::remove_file(startmenu.join("ETDPicker.lnk"));
    }

    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "ETDPicker.exe"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    if let Ok(curr_exe) = std::env::current_exe() {
        if let Some(parent) = curr_exe.parent() {
            let parent_str = parent.to_string_lossy();
            let cmd = format!("timeout /t 1 /nobreak > NUL & rmdir /s /q \"{}\"", parent_str);
            let _ = Command::new("cmd")
                .args(["/C", &cmd])
                .creation_flags(CREATE_NO_WINDOW)
                .spawn();
        }
    }
}

unsafe extern "system" fn wizard_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            create_step_controls(hwnd);
            0
        }
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as u32;
            let step = CURRENT_STEP.load(Ordering::SeqCst);

            if id == 101 {
                update_control_visibility();
            } else if id == 201 {
                if step > 1 {
                    CURRENT_STEP.store(step - 1, Ordering::SeqCst);
                    update_control_visibility();
                    windows_sys::Win32::Graphics::Gdi::InvalidateRect(hwnd, std::ptr::null(), 1);
                }
            } else if id == 202 {
                if IS_UNINSTALLER.load(Ordering::SeqCst) {
                    perform_uninstallation();
                    let lang = get_active_lang();
                    let success_msg = get_installer_strings(lang).uninst_success;
                    let msg_wide: Vec<u16> = success_msg.encode_utf16().chain(std::iter::once(0)).collect();
                    let title_wide: Vec<u16> = "ETDPicker\0".encode_utf16().collect();
                    windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW(
                        hwnd,
                        msg_wide.as_ptr(),
                        title_wide.as_ptr(),
                        0,
                    );
                    DestroyWindow(hwnd);
                    return 0;
                }

                if step == 1 {
                    CURRENT_STEP.store(2, Ordering::SeqCst);
                } else if step == 2 {
                    CURRENT_STEP.store(3, Ordering::SeqCst);
                } else if step == 3 {
                    CURRENT_STEP.store(4, Ordering::SeqCst);
                    update_control_visibility();
                    windows_sys::Win32::Graphics::Gdi::InvalidateRect(hwnd, std::ptr::null(), 1);

                    perform_installation(hwnd);

                    CURRENT_STEP.store(5, Ordering::SeqCst);
                } else if step == 5 {
                    let should_launch = SendMessageW(CHK_LAUNCH, BM_GETCHECK, 0, 0) == 1;
                    if should_launch {
                        let mut dir_buf = [0u16; 512];
                        let len = GetWindowTextW(EDT_DIR, dir_buf.as_mut_ptr(), 512);
                        let install_dir_str = String::from_utf16_lossy(&dir_buf[..len as usize]);
                        let target_exe = PathBuf::from(install_dir_str).join("ETDPicker.exe");
                        let _ = Command::new(&target_exe)
                            .creation_flags(CREATE_NO_WINDOW)
                            .spawn();
                    }
                    DestroyWindow(hwnd);
                    return 0;
                }
                update_control_visibility();
                windows_sys::Win32::Graphics::Gdi::InvalidateRect(hwnd, std::ptr::null(), 1);
            } else if id == 203 {
                DestroyWindow(hwnd);
            }
            0
        }
        WM_PAINT => {
            let mut ps = std::mem::zeroed();
            let hdc = windows_sys::Win32::Graphics::Gdi::BeginPaint(hwnd, &mut ps);

            let lang = get_active_lang();
            let strings = get_installer_strings(lang);
            let step = CURRENT_STEP.load(Ordering::SeqCst);
            let is_uninstaller = IS_UNINSTALLER.load(Ordering::SeqCst);

            let bg_brush = CreateSolidBrush(0x001B1A1A);
            let full_rc = RECT { left: 0, top: 0, right: 560, bottom: 440 };
            FillRect(hdc, &full_rc, bg_brush as _);
            DeleteObject(bg_brush as _);

            let header_brush = CreateSolidBrush(0x00242222);
            let header_rc = RECT { left: 0, top: 0, right: 560, bottom: 85 };
            FillRect(hdc, &header_rc, header_brush as _);
            DeleteObject(header_brush as _);

            let line_brush = CreateSolidBrush(0x003E3A3A);
            let sep_rc = RECT { left: 0, top: 84, right: 560, bottom: 85 };
            FillRect(hdc, &sep_rc, line_brush as _);

            let bottom_sep_rc = RECT { left: 0, top: 335, right: 560, bottom: 336 };
            FillRect(hdc, &bottom_sep_rc, line_brush as _);
            DeleteObject(line_brush as _);

            SetBkMode(hdc, TRANSPARENT as _);

            let font_title = CreateFontW(20, 0, 0, 0, FW_BOLD as _, 0, 0, 0, 1, 0, 0, 0, 0, "Segoe UI\0".encode_utf16().collect::<Vec<_>>().as_ptr());
            let font_sub = CreateFontW(14, 0, 0, 0, FW_NORMAL as _, 0, 0, 0, 1, 0, 0, 0, 0, "Segoe UI\0".encode_utf16().collect::<Vec<_>>().as_ptr());
            let font_body = CreateFontW(15, 0, 0, 0, FW_NORMAL as _, 0, 0, 0, 1, 0, 0, 0, 0, "Segoe UI\0".encode_utf16().collect::<Vec<_>>().as_ptr());

            let (h_title, h_sub) = if is_uninstaller {
                (strings.uninst_title, strings.uninst_prompt)
            } else {
                match step {
                    1 => (strings.step1_title, "ETDPicker v1.0.0"),
                    2 => (strings.step2_title, strings.step2_desc),
                    3 => (strings.step3_title, strings.step3_desc),
                    4 => (strings.step4_title, strings.step4_desc),
                    5 => (strings.step5_title, strings.step5_desc),
                    _ => (strings.wizard_title, ""),
                }
            };

            SelectObject(hdc, font_title as _);
            SetTextColor(hdc, 0x00FFFFFF);
            let mut ht_wide: Vec<u16> = h_title.encode_utf16().collect();
            let mut ht_rc = RECT { left: 30, top: 16, right: 480, bottom: 42 };
            DrawTextW(hdc, ht_wide.as_mut_ptr(), ht_wide.len() as _, &mut ht_rc, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);

            SelectObject(hdc, font_sub as _);
            SetTextColor(hdc, 0x00AAAAAA);
            let mut hs_wide: Vec<u16> = h_sub.encode_utf16().collect();
            let mut hs_rc = RECT { left: 30, top: 46, right: 480, bottom: 78 };
            DrawTextW(hdc, hs_wide.as_mut_ptr(), hs_wide.len() as _, &mut hs_rc, DT_LEFT | DT_WORDBREAK | DT_NOPREFIX);

            SelectObject(hdc, font_body as _);
            SetTextColor(hdc, 0x00DDDDDD);

            if is_uninstaller {
                let mut un_wide: Vec<u16> = strings.uninst_prompt.encode_utf16().collect();
                let mut un_rc = RECT { left: 30, top: 120, right: 530, bottom: 250 };
                DrawTextW(hdc, un_wide.as_mut_ptr(), un_wide.len() as _, &mut un_rc, DT_LEFT | DT_WORDBREAK | DT_NOPREFIX);
            } else {
                match step {
                    1 => {
                        let mut b1_wide: Vec<u16> = strings.step1_desc.encode_utf16().collect();
                        let mut b1_rc = RECT { left: 30, top: 110, right: 530, bottom: 260 };
                        DrawTextW(hdc, b1_wide.as_mut_ptr(), b1_wide.len() as _, &mut b1_rc, DT_LEFT | DT_WORDBREAK | DT_NOPREFIX);
                    }
                    2 => {
                        let mut b2_wide: Vec<u16> = strings.step2_desc.encode_utf16().collect();
                        let mut b2_rc = RECT { left: 30, top: 110, right: 530, bottom: 180 };
                        DrawTextW(hdc, b2_wide.as_mut_ptr(), b2_wide.len() as _, &mut b2_rc, DT_LEFT | DT_WORDBREAK | DT_NOPREFIX);
                    }
                    3 => {
                        let mut b3_wide: Vec<u16> = strings.step3_desc.encode_utf16().collect();
                        let mut b3_rc = RECT { left: 30, top: 110, right: 530, bottom: 160 };
                        DrawTextW(hdc, b3_wide.as_mut_ptr(), b3_wide.len() as _, &mut b3_rc, DT_LEFT | DT_WORDBREAK | DT_NOPREFIX);
                    }
                    4 => {
                        let prog_rc = RECT { left: 30, top: 180, right: 530, bottom: 215 };
                        let bar_brush = CreateSolidBrush(0x002A2A2A);
                        FillRect(hdc, &prog_rc, bar_brush as _);
                        DeleteObject(bar_brush as _);

                        let fill_rc = RECT { left: 30, top: 180, right: 420, bottom: 215 };
                        let active_bar = CreateSolidBrush(0x00D66822);
                        FillRect(hdc, &fill_rc, active_bar as _);
                        DeleteObject(active_bar as _);
                    }
                    5 => {
                        let mut b5_wide: Vec<u16> = strings.step5_desc.encode_utf16().collect();
                        let mut b5_rc = RECT { left: 30, top: 110, right: 530, bottom: 180 };
                        DrawTextW(hdc, b5_wide.as_mut_ptr(), b5_wide.len() as _, &mut b5_rc, DT_LEFT | DT_WORDBREAK | DT_NOPREFIX);
                    }
                    _ => {}
                }
            }

            DeleteObject(font_title as _);
            DeleteObject(font_sub as _);
            DeleteObject(font_body as _);

            windows_sys::Win32::Graphics::Gdi::EndPaint(hwnd, &ps);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
