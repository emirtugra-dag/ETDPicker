use crate::config::AppConfig;
use crate::i18n::{get_strings, Language};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, FillRect, FrameRect,
    SelectObject, SetBkMode, SetTextColor, DT_LEFT, DT_NOPREFIX, FW_BOLD, FW_NORMAL,
    TRANSPARENT,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::EnableWindow;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
    GetSystemMetrics, IsDialogMessageW, PostQuitMessage, RegisterClassW, SendMessageW,
    ShowWindow, TranslateMessage, BM_GETCHECK, BM_SETCHECK, CB_ADDSTRING, CB_GETCURSEL,
    CB_SETCURSEL, CS_HREDRAW, CS_VREDRAW, MSG, SM_CXSCREEN, SM_CYSCREEN, SW_SHOW,
    WM_COMMAND, WM_DESTROY, WM_PAINT, WNDCLASSW, WS_CHILD, WS_EX_DLGMODALFRAME,
    WS_EX_TOPMOST, WS_POPUP, WS_TABSTOP, WS_VISIBLE,
};

static mut CHK_STARTUP: HWND = 0 as _;
static mut CMB_LANG: HWND = 0 as _;
static mut CMB_MOD: HWND = 0 as _;
static mut CMB_KEY: HWND = 0 as _;
static mut SETTINGS_SAVED: bool = false;

pub fn show_settings_dialog(parent_hwnd: HWND, cfg: &mut AppConfig) -> bool {
    unsafe {
        SETTINGS_SAVED = false;
        let strings = get_strings(cfg.language);

        let class_name: Vec<u16> = "ETDSettingsWindow\0".encode_utf16().collect();
        let hinstance = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null());

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(settings_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: 0 as _,
            hCursor: 0 as _,
            hbrBackground: CreateSolidBrush(0x001E1E1E) as _,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
        };
        RegisterClassW(&wc);

        let width = 440;
        let height = 340;
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let x = (screen_w - width) / 2;
        let y = (screen_h - height) / 2;

        let title_wide: Vec<u16> = strings.settings_title.encode_utf16().chain(std::iter::once(0)).collect();

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_DLGMODALFRAME,
            class_name.as_ptr(),
            title_wide.as_ptr(),
            WS_POPUP | WS_VISIBLE,
            x,
            y,
            width,
            height,
            parent_hwnd,
            0 as _,
            hinstance,
            std::ptr::null_mut(),
        );

        if parent_hwnd != 0 as _ {
            EnableWindow(parent_hwnd, 0);
        }

        let font_ui = CreateFontW(
            15, 0, 0, 0, FW_NORMAL as _, 0, 0, 0, 1, 0, 0, 0, 0,
            "Segoe UI\0".encode_utf16().collect::<Vec<_>>().as_ptr(),
        );

        let combo_class: Vec<u16> = "COMBOBOX\0".encode_utf16().collect();
        let btn_class: Vec<u16> = "BUTTON\0".encode_utf16().collect();

        // 1. Language Combobox
        CMB_LANG = CreateWindowExW(
            0,
            combo_class.as_ptr(),
            std::ptr::null(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x0002 | 0x0200,
            180,
            68,
            220,
            150,
            hwnd,
            2001 as _,
            hinstance,
            std::ptr::null_mut(),
        );
        SendMessageW(CMB_LANG, 0x0030, font_ui as _, 1);
        let tr_str: Vec<u16> = "Türkçe (TR)\0".encode_utf16().collect();
        let en_str: Vec<u16> = "English (EN)\0".encode_utf16().collect();
        SendMessageW(CMB_LANG, CB_ADDSTRING, 0, tr_str.as_ptr() as _);
        SendMessageW(CMB_LANG, CB_ADDSTRING, 0, en_str.as_ptr() as _);
        SendMessageW(CMB_LANG, CB_SETCURSEL, if cfg.language == Language::Turkish { 0 } else { 1 }, 0);

        // 2. Hotkey Modifier Combobox
        CMB_MOD = CreateWindowExW(
            0,
            combo_class.as_ptr(),
            std::ptr::null(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x0002 | 0x0200,
            180,
            116,
            100,
            150,
            hwnd,
            2002 as _,
            hinstance,
            std::ptr::null_mut(),
        );
        SendMessageW(CMB_MOD, 0x0030, font_ui as _, 1);
        let mod_options = ["Alt\0", "Ctrl + Alt\0", "Ctrl + Shift\0", "Shift + Alt\0"];
        for m in mod_options {
            let m_wide: Vec<u16> = m.encode_utf16().collect();
            SendMessageW(CMB_MOD, CB_ADDSTRING, 0, m_wide.as_ptr() as _);
        }
        let mod_idx = match cfg.hotkey_mod {
            0x0001 => 0,
            0x0003 => 1,
            0x0006 => 2,
            0x0005 => 3,
            _ => 0,
        };
        SendMessageW(CMB_MOD, CB_SETCURSEL, mod_idx, 0);

        // 3. Hotkey Key Combobox
        CMB_KEY = CreateWindowExW(
            0,
            combo_class.as_ptr(),
            std::ptr::null(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x0002 | 0x0200,
            290,
            116,
            110,
            200,
            hwnd,
            2003 as _,
            hinstance,
            std::ptr::null_mut(),
        );
        SendMessageW(CMB_KEY, 0x0030, font_ui as _, 1);
        let key_options = [
            "P (0x50)\0", "C (0x43)\0", "X (0x58)\0", "Z (0x5A)\0", "S (0x53)\0",
            "F8\0", "F9\0", "F10\0", "F11\0", "F12\0",
        ];
        for k in key_options {
            let k_wide: Vec<u16> = k.encode_utf16().collect();
            SendMessageW(CMB_KEY, CB_ADDSTRING, 0, k_wide.as_ptr() as _);
        }
        let key_idx = match cfg.hotkey_vk {
            0x50 => 0,
            0x43 => 1,
            0x58 => 2,
            0x5A => 3,
            0x53 => 4,
            0x77 => 5,
            0x78 => 6,
            0x79 => 7,
            0x7A => 8,
            0x7B => 9,
            _ => 0,
        };
        SendMessageW(CMB_KEY, CB_SETCURSEL, key_idx, 0);

        // 4. Startup Checkbox
        let chk_label: Vec<u16> = strings.startup_label.encode_utf16().chain(std::iter::once(0)).collect();
        CHK_STARTUP = CreateWindowExW(
            0,
            btn_class.as_ptr(),
            chk_label.as_ptr(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x00000003,
            24,
            168,
            380,
            26,
            hwnd,
            2004 as _,
            hinstance,
            std::ptr::null_mut(),
        );
        SendMessageW(CHK_STARTUP, 0x0030, font_ui as _, 1);
        SendMessageW(CHK_STARTUP, BM_SETCHECK, if cfg.run_on_startup { 1 } else { 0 }, 0);

        // 5. Save Button
        let save_text: Vec<u16> = strings.save_btn.encode_utf16().chain(std::iter::once(0)).collect();
        CreateWindowExW(
            0,
            btn_class.as_ptr(),
            save_text.as_ptr(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x00000001,
            190,
            230,
            100,
            36,
            hwnd,
            2005 as _,
            hinstance,
            std::ptr::null_mut(),
        );

        // 6. Cancel Button
        let cancel_text: Vec<u16> = strings.cancel_btn.encode_utf16().chain(std::iter::once(0)).collect();
        CreateWindowExW(
            0,
            btn_class.as_ptr(),
            cancel_text.as_ptr(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP,
            300,
            230,
            100,
            36,
            hwnd,
            2006 as _,
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

        if parent_hwnd != 0 as _ {
            EnableWindow(parent_hwnd, 1);
            windows_sys::Win32::UI::WindowsAndMessaging::SetForegroundWindow(parent_hwnd);
        }

        DeleteObject(font_ui as _);

        if SETTINGS_SAVED {
            let lang_idx = SendMessageW(CMB_LANG, CB_GETCURSEL, 0, 0);
            cfg.language = if lang_idx == 0 { Language::Turkish } else { Language::English };

            let mod_sel = SendMessageW(CMB_MOD, CB_GETCURSEL, 0, 0);
            let (mod_val, mod_name) = match mod_sel {
                0 => (0x0001, "Alt"),
                1 => (0x0003, "Ctrl + Alt"),
                2 => (0x0006, "Ctrl + Shift"),
                3 => (0x0005, "Shift + Alt"),
                _ => (0x0001, "Alt"),
            };

            let key_sel = SendMessageW(CMB_KEY, CB_GETCURSEL, 0, 0);
            let (vk_val, key_name) = match key_sel {
                0 => (0x50, "P"),
                1 => (0x43, "C"),
                2 => (0x58, "X"),
                3 => (0x5A, "Z"),
                4 => (0x53, "S"),
                5 => (0x77, "F8"),
                6 => (0x78, "F9"),
                7 => (0x79, "F10"),
                8 => (0x7A, "F11"),
                9 => (0x7B, "F12"),
                _ => (0x50, "P"),
            };

            cfg.hotkey_mod = mod_val;
            cfg.hotkey_vk = vk_val;
            cfg.hotkey_name = format!("{} + {}", mod_name, key_name);

            let startup_checked = SendMessageW(CHK_STARTUP, BM_GETCHECK, 0, 0);
            cfg.run_on_startup = startup_checked == 1;

            cfg.apply_startup_registry();
            cfg.save();
            return true;
        }

        false
    }
}

unsafe extern "system" fn settings_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as i32;
            if id == 2005 {
                SETTINGS_SAVED = true;
                DestroyWindow(hwnd);
            } else if id == 2006 || id == 2 {
                SETTINGS_SAVED = false;
                DestroyWindow(hwnd);
            }
            0
        }
        WM_PAINT => {
            let mut ps = std::mem::zeroed();
            let hdc = windows_sys::Win32::Graphics::Gdi::BeginPaint(hwnd, &mut ps);

            let cfg = crate::config::AppConfig::load();
            let strings = get_strings(cfg.language);

            let bg_brush = CreateSolidBrush(0x001F1E1E);
            let full_rc = RECT { left: 0, top: 0, right: 440, bottom: 340 };
            FillRect(hdc, &full_rc, bg_brush as _);
            DeleteObject(bg_brush as _);

            let border_brush = CreateSolidBrush(0x004A4A4A);
            FrameRect(hdc, &full_rc, border_brush as _);
            DeleteObject(border_brush as _);

            SetBkMode(hdc, TRANSPARENT as _);

            let font_title = CreateFontW(
                20, 0, 0, 0, FW_BOLD as _, 0, 0, 0, 1, 0, 0, 0, 0,
                "Segoe UI\0".encode_utf16().collect::<Vec<_>>().as_ptr(),
            );
            let old_font = SelectObject(hdc, font_title as _);
            SetTextColor(hdc, 0x00FFFFFF);

            let mut title_wide: Vec<u16> = strings.settings_title.encode_utf16().collect();
            let mut title_rc = RECT { left: 24, top: 18, right: 416, bottom: 48 };
            DrawTextW(hdc, title_wide.as_mut_ptr(), title_wide.len() as _, &mut title_rc, DT_LEFT | DT_NOPREFIX);

            DeleteObject(font_title as _);

            let font_label = CreateFontW(
                15, 0, 0, 0, FW_NORMAL as _, 0, 0, 0, 1, 0, 0, 0, 0,
                "Segoe UI\0".encode_utf16().collect::<Vec<_>>().as_ptr(),
            );
            SelectObject(hdc, font_label as _);
            SetTextColor(hdc, 0x00CCCCCC);

            let mut l1_wide: Vec<u16> = strings.lang_label.encode_utf16().collect();
            let mut l1_rc = RECT { left: 24, top: 72, right: 170, bottom: 96 };
            DrawTextW(hdc, l1_wide.as_mut_ptr(), l1_wide.len() as _, &mut l1_rc, DT_LEFT | DT_NOPREFIX);

            let mut l2_wide: Vec<u16> = strings.hotkey_label.encode_utf16().collect();
            let mut l2_rc = RECT { left: 24, top: 120, right: 170, bottom: 144 };
            DrawTextW(hdc, l2_wide.as_mut_ptr(), l2_wide.len() as _, &mut l2_rc, DT_LEFT | DT_NOPREFIX);

            SelectObject(hdc, old_font);
            DeleteObject(font_label as _);

            windows_sys::Win32::Graphics::Gdi::EndPaint(hwnd, &mut ps);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
