#![windows_subsystem = "windows"]

mod color;
mod config;
mod i18n;
mod magnifier;
mod paint_guide;
mod settings_dialog;

use color::RgbColor;
use config::AppConfig;
use i18n::{get_strings, Language};
use settings_dialog::SettingsResult;
use std::sync::Mutex;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, FillRect, FrameRect,
    SelectObject, SetBkMode, SetTextColor, DT_CENTER, DT_LEFT, DT_NOPREFIX,
    DT_SINGLELINE, DT_VCENTER, FW_BOLD, FW_NORMAL, TRANSPARENT,
};
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, NIF_ICON, NIF_MESSAGE,
    NIF_TIP,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
    DestroyWindow, DispatchMessageW, GetCursorPos, GetMessageW, GetSystemMetrics, LoadCursorW,
    LoadIconW, PostQuitMessage, RegisterClassW, SetForegroundWindow, SetWindowTextW, ShowWindow,
    TrackPopupMenu, TranslateMessage, CS_HREDRAW, CS_VREDRAW, IDC_ARROW, MF_SEPARATOR,
    MF_STRING, MSG, SC_CLOSE, SM_CXSCREEN, SM_CYSCREEN, SW_HIDE, SW_RESTORE, SW_SHOW,
    TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RIGHTBUTTON, WM_CLOSE, WM_COMMAND, WM_DESTROY,
    WM_HOTKEY, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_PAINT, WM_RBUTTONUP, WM_SYSCOMMAND,
    WM_TIMER, WM_USER, WNDCLASSW, WS_CLIPCHILDREN, WS_MINIMIZEBOX, WS_OVERLAPPED,
    WS_SYSMENU, WS_VISIBLE,
};

const WM_TRAYICON: u32 = WM_USER + 1;
const HOTKEY_ID: i32 = 100;
const TOAST_TIMER_ID: usize = 1;

pub struct AppState {
    pub config: AppConfig,
    pub active_color: RgbColor,
    pub toast_visible: bool,
    pub toast_text: String,
    pub app_hwnd: isize,
    pub tray_active: bool,
}

unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

static APP_STATE: Mutex<AppState> = Mutex::new(AppState {
    config: AppConfig {
        language: Language::Turkish,
        hotkey_mod: 0x0001,
        hotkey_vk: 0x50,
        hotkey_name: String::new(),
        run_on_startup: false,
        show_tray_icon: true,
        recent_colors: Vec::new(),
    },
    active_color: RgbColor::new(52, 152, 219),
    toast_visible: false,
    toast_text: String::new(),
    app_hwnd: 0,
    tray_active: false,
});

fn main() {
    let cfg = AppConfig::load();
    cfg.apply_startup_registry();

    let initial_color = cfg.recent_colors.first().copied().unwrap_or(RgbColor::new(52, 152, 219));

    {
        let mut state = APP_STATE.lock().unwrap();
        state.config = cfg.clone();
        state.active_color = initial_color;
    }

    unsafe {
        let class_name: Vec<u16> = "ETDPickerMainWindow\0".encode_utf16().collect();
        let hinstance = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null());

        let icon = LoadIconW(hinstance, 1 as _);
        let cursor = LoadCursorW(0 as _, IDC_ARROW);

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(main_wnd_proc),
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

        let width = 500;
        let height = 480;
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let x = (screen_w - width) / 2;
        let y = (screen_h - height) / 2;

        let title = get_strings(cfg.language).app_title;
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

        {
            let mut state = APP_STATE.lock().unwrap();
            state.app_hwnd = hwnd as isize;
        }

        register_app_hotkey(hwnd, cfg.hotkey_mod, cfg.hotkey_vk);

        if cfg.show_tray_icon {
            add_tray_icon(hwnd, icon, title);
            let mut state = APP_STATE.lock().unwrap();
            state.tray_active = true;
        }

        let is_silent = std::env::args().any(|arg| arg == "--silent" || arg == "-s");
        if is_silent {
            ShowWindow(hwnd, SW_HIDE);
        } else {
            ShowWindow(hwnd, SW_SHOW);
        }

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0 as _, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        remove_tray_icon(hwnd);
        UnregisterHotKey(hwnd, HOTKEY_ID);
    }
}

unsafe fn register_app_hotkey(hwnd: HWND, modifiers: u32, vk: u32) {
    UnregisterHotKey(hwnd, HOTKEY_ID);
    let _ = RegisterHotKey(hwnd, HOTKEY_ID, modifiers, vk);
}

unsafe fn add_tray_icon(hwnd: HWND, icon: windows_sys::Win32::UI::WindowsAndMessaging::HICON, tip: &str) {
    let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    nid.uCallbackMessage = WM_TRAYICON;
    nid.hIcon = icon;

    let tip_wide: Vec<u16> = tip.encode_utf16().take(127).chain(std::iter::once(0)).collect();
    for (i, &ch) in tip_wide.iter().enumerate() {
        if i < nid.szTip.len() {
            nid.szTip[i] = ch;
        }
    }

    Shell_NotifyIconW(NIM_ADD, &nid);
}

unsafe fn remove_tray_icon(hwnd: HWND) {
    let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    Shell_NotifyIconW(NIM_DELETE, &nid);
}

unsafe fn copy_to_clipboard(hwnd: HWND, text: &str) {
    if OpenClipboard(hwnd) != 0 {
        EmptyClipboard();

        let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let bytes = (utf16.len() * 2) as usize;

        let hmem = GlobalAlloc(GMEM_MOVEABLE, bytes);
        if hmem != 0 as _ {
            let ptr = GlobalLock(hmem) as *mut u16;
            if !ptr.is_null() {
                std::ptr::copy_nonoverlapping(utf16.as_ptr(), ptr, utf16.len());
                GlobalUnlock(hmem);
                SetClipboardData(13, hmem as _);
            }
        }

        CloseClipboard();

        {
            let mut state = APP_STATE.lock().unwrap();
            let strings = get_strings(state.config.language);
            state.toast_text = format!("{} ({})", strings.copied_to_clipboard, text);
            state.toast_visible = true;
        }

        windows_sys::Win32::UI::WindowsAndMessaging::SetTimer(hwnd, TOAST_TIMER_ID, 2000, None);
        windows_sys::Win32::Graphics::Gdi::InvalidateRect(hwnd, std::ptr::null(), 0);
    }
}

unsafe fn trigger_pick_color(hwnd: HWND) {
    let lang = {
        let state = APP_STATE.lock().unwrap();
        state.config.language
    };

    ShowWindow(hwnd, SW_HIDE);
    let res = magnifier::pick_color_interactive(lang);

    // After picking, ALWAYS restore and show the main window to show the selected color!
    ShowWindow(hwnd, SW_RESTORE);
    ShowWindow(hwnd, SW_SHOW);
    SetForegroundWindow(hwnd);

    if res.selected {
        {
            let mut state = APP_STATE.lock().unwrap();
            state.active_color = res.color;
            state.config.add_recent_color(res.color);
        }
        copy_to_clipboard(hwnd, &res.color.to_hex());
        windows_sys::Win32::Graphics::Gdi::InvalidateRect(hwnd, std::ptr::null(), 0);
    }
}

struct RectButton {
    rect: RECT,
    action_id: u32,
}

fn get_ui_buttons() -> Vec<RectButton> {
    vec![
        RectButton { rect: RECT { left: 340, top: 16, right: 380, bottom: 44 }, action_id: 101 },
        RectButton { rect: RECT { left: 390, top: 16, right: 430, bottom: 44 }, action_id: 102 },
        RectButton { rect: RECT { left: 440, top: 16, right: 480, bottom: 44 }, action_id: 103 },
        RectButton { rect: RECT { left: 340, top: 78, right: 480, bottom: 106 }, action_id: 201 },
        RectButton { rect: RECT { left: 340, top: 114, right: 480, bottom: 142 }, action_id: 202 },
        RectButton { rect: RECT { left: 240, top: 152, right: 315, bottom: 180 }, action_id: 301 },
        RectButton { rect: RECT { left: 325, top: 152, right: 400, bottom: 180 }, action_id: 302 },
        RectButton { rect: RECT { left: 410, top: 152, right: 480, bottom: 180 }, action_id: 303 },
        RectButton { rect: RECT { left: 20, top: 380, right: 480, bottom: 435 }, action_id: 500 },
    ]
}

unsafe extern "system" fn main_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_HOTKEY => {
            if wparam as i32 == HOTKEY_ID {
                trigger_pick_color(hwnd);
            }
            0
        }
        WM_SYSCOMMAND => {
            let cmd = (wparam & 0xFFF0) as u32;
            if cmd == SC_CLOSE {
                // Hide to background instead of quitting!
                ShowWindow(hwnd, SW_HIDE);
                return 0;
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_CLOSE => {
            // Hide to background instead of quitting!
            ShowWindow(hwnd, SW_HIDE);
            0
        }
        WM_TRAYICON => {
            let event = (lparam & 0xFFFF) as u32;
            if event == WM_LBUTTONUP {
                ShowWindow(hwnd, SW_RESTORE);
                ShowWindow(hwnd, SW_SHOW);
                SetForegroundWindow(hwnd);
            } else if event == WM_RBUTTONUP {
                let mut pt = POINT { x: 0, y: 0 };
                GetCursorPos(&mut pt);

                let hmenu = CreatePopupMenu();
                let lang = {
                    let state = APP_STATE.lock().unwrap();
                    state.config.language
                };
                let strings = get_strings(lang);

                let pick_str: Vec<u16> = strings.tray_pick.encode_utf16().chain(std::iter::once(0)).collect();
                let show_str: Vec<u16> = strings.tray_show.encode_utf16().chain(std::iter::once(0)).collect();
                let guide_str: Vec<u16> = strings.tray_paint_guide.encode_utf16().chain(std::iter::once(0)).collect();
                let set_str: Vec<u16> = strings.tray_settings.encode_utf16().chain(std::iter::once(0)).collect();
                let exit_str: Vec<u16> = strings.tray_exit.encode_utf16().chain(std::iter::once(0)).collect();

                AppendMenuW(hmenu, MF_STRING, 9001, pick_str.as_ptr());
                AppendMenuW(hmenu, MF_STRING, 9002, show_str.as_ptr());
                AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());
                AppendMenuW(hmenu, MF_STRING, 9003, guide_str.as_ptr());
                AppendMenuW(hmenu, MF_STRING, 9004, set_str.as_ptr());
                AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());
                AppendMenuW(hmenu, MF_STRING, 9005, exit_str.as_ptr());

                SetForegroundWindow(hwnd);
                TrackPopupMenu(hmenu, TPM_RIGHTBUTTON | TPM_LEFTALIGN | TPM_BOTTOMALIGN, pt.x, pt.y, 0, hwnd, std::ptr::null());
                DestroyMenu(hmenu);
            }
            0
        }
        WM_COMMAND => {
            let cmd_id = (wparam & 0xFFFF) as u32;
            match cmd_id {
                9001 => trigger_pick_color(hwnd),
                9002 => {
                    ShowWindow(hwnd, SW_RESTORE);
                    ShowWindow(hwnd, SW_SHOW);
                    SetForegroundWindow(hwnd);
                }
                9003 => {
                    let lang = {
                        let state = APP_STATE.lock().unwrap();
                        state.config.language
                    };
                    paint_guide::show_paint_guide(hwnd, lang);
                    ShowWindow(hwnd, SW_SHOW);
                    SetForegroundWindow(hwnd);
                    windows_sys::Win32::Graphics::Gdi::InvalidateRect(hwnd, std::ptr::null(), 0);
                }
                9004 => {
                    let mut cfg_copy = {
                        let state = APP_STATE.lock().unwrap();
                        state.config.clone()
                    };
                    match settings_dialog::show_settings_dialog(hwnd, &mut cfg_copy) {
                        SettingsResult::Saved => {
                            register_app_hotkey(hwnd, cfg_copy.hotkey_mod, cfg_copy.hotkey_vk);
                            let title = get_strings(cfg_copy.language).app_title;
                            let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
                            SetWindowTextW(hwnd, title_wide.as_ptr());

                            let hinstance = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null());
                            let icon = LoadIconW(hinstance, 1 as _);

                            {
                                let mut state = APP_STATE.lock().unwrap();
                                if cfg_copy.show_tray_icon && !state.tray_active {
                                    add_tray_icon(hwnd, icon, title);
                                    state.tray_active = true;
                                } else if !cfg_copy.show_tray_icon && state.tray_active {
                                    remove_tray_icon(hwnd);
                                    state.tray_active = false;
                                }
                                state.config = cfg_copy;
                            }
                        }
                        SettingsResult::ExitApplication => {
                            DestroyWindow(hwnd);
                            return 0;
                        }
                        SettingsResult::Cancelled => {}
                    }
                    ShowWindow(hwnd, SW_SHOW);
                    SetForegroundWindow(hwnd);
                    windows_sys::Win32::Graphics::Gdi::InvalidateRect(hwnd, std::ptr::null(), 0);
                }
                9005 => {
                    DestroyWindow(hwnd);
                }
                _ => {}
            }
            0
        }
        WM_TIMER => {
            if wparam == TOAST_TIMER_ID {
                {
                    let mut state = APP_STATE.lock().unwrap();
                    state.toast_visible = false;
                }
                windows_sys::Win32::UI::WindowsAndMessaging::KillTimer(hwnd, TOAST_TIMER_ID);
                windows_sys::Win32::Graphics::Gdi::InvalidateRect(hwnd, std::ptr::null(), 0);
            }
            0
        }
        WM_LBUTTONDOWN => {
            let x = (lparam & 0xFFFF) as i32;
            let y = ((lparam >> 16) & 0xFFFF) as i32;

            if y >= 310 && y <= 350 {
                let color_opt = {
                    let state = APP_STATE.lock().unwrap();
                    let mut found = None;
                    for (i, c) in state.config.recent_colors.iter().enumerate().take(10) {
                        let sx = 20 + (i as i32 * 46);
                        if x >= sx && x <= sx + 38 {
                            found = Some(*c);
                            break;
                        }
                    }
                    found
                };

                if let Some(c) = color_opt {
                    {
                        let mut state = APP_STATE.lock().unwrap();
                        state.active_color = c;
                    }
                    copy_to_clipboard(hwnd, &c.to_hex());
                    windows_sys::Win32::Graphics::Gdi::InvalidateRect(hwnd, std::ptr::null(), 0);
                    return 0;
                }
            }

            for btn in get_ui_buttons() {
                if x >= btn.rect.left && x <= btn.rect.right && y >= btn.rect.top && y <= btn.rect.bottom {
                    match btn.action_id {
                        101 => {
                            let new_lang = {
                                let mut state = APP_STATE.lock().unwrap();
                                state.config.language = match state.config.language {
                                    Language::Turkish => Language::English,
                                    Language::English => Language::Turkish,
                                };
                                state.config.save();
                                state.config.language
                            };
                            let title = get_strings(new_lang).app_title;
                            let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
                            SetWindowTextW(hwnd, title_wide.as_ptr());
                            windows_sys::Win32::Graphics::Gdi::InvalidateRect(hwnd, std::ptr::null(), 0);
                        }
                        102 => {
                            let lang = {
                                let state = APP_STATE.lock().unwrap();
                                state.config.language
                            };
                            paint_guide::show_paint_guide(hwnd, lang);
                            ShowWindow(hwnd, SW_SHOW);
                            SetForegroundWindow(hwnd);
                            windows_sys::Win32::Graphics::Gdi::InvalidateRect(hwnd, std::ptr::null(), 0);
                        }
                        103 => {
                            let mut cfg_copy = {
                                let state = APP_STATE.lock().unwrap();
                                state.config.clone()
                            };
                            match settings_dialog::show_settings_dialog(hwnd, &mut cfg_copy) {
                                SettingsResult::Saved => {
                                    register_app_hotkey(hwnd, cfg_copy.hotkey_mod, cfg_copy.hotkey_vk);
                                    let title = get_strings(cfg_copy.language).app_title;
                                    let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
                                    SetWindowTextW(hwnd, title_wide.as_ptr());

                                    let hinstance = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null());
                                    let icon = LoadIconW(hinstance, 1 as _);

                                    {
                                        let mut state = APP_STATE.lock().unwrap();
                                        if cfg_copy.show_tray_icon && !state.tray_active {
                                            add_tray_icon(hwnd, icon, title);
                                            state.tray_active = true;
                                        } else if !cfg_copy.show_tray_icon && state.tray_active {
                                            remove_tray_icon(hwnd);
                                            state.tray_active = false;
                                        }
                                        state.config = cfg_copy;
                                    }
                                }
                                SettingsResult::ExitApplication => {
                                    DestroyWindow(hwnd);
                                    return 0;
                                }
                                SettingsResult::Cancelled => {}
                            }
                            ShowWindow(hwnd, SW_SHOW);
                            SetForegroundWindow(hwnd);
                            windows_sys::Win32::Graphics::Gdi::InvalidateRect(hwnd, std::ptr::null(), 0);
                        }
                        201 => {
                            let hex = {
                                let state = APP_STATE.lock().unwrap();
                                state.active_color.to_hex()
                            };
                            copy_to_clipboard(hwnd, &hex);
                        }
                        202 => {
                            let rgb = {
                                let state = APP_STATE.lock().unwrap();
                                state.active_color.to_rgb_string()
                            };
                            copy_to_clipboard(hwnd, &rgb);
                        }
                        301 => {
                            let r_val = {
                                let state = APP_STATE.lock().unwrap();
                                state.active_color.r
                            };
                            copy_to_clipboard(hwnd, &format!("{}", r_val));
                        }
                        302 => {
                            let g_val = {
                                let state = APP_STATE.lock().unwrap();
                                state.active_color.g
                            };
                            copy_to_clipboard(hwnd, &format!("{}", g_val));
                        }
                        303 => {
                            let b_val = {
                                let state = APP_STATE.lock().unwrap();
                                state.active_color.b
                            };
                            copy_to_clipboard(hwnd, &format!("{}", b_val));
                        }
                        500 => trigger_pick_color(hwnd),
                        _ => {}
                    }
                    return 0;
                }
            }
            0
        }
        WM_PAINT => {
            let mut ps = std::mem::zeroed();
            let hdc = windows_sys::Win32::Graphics::Gdi::BeginPaint(hwnd, &mut ps);

            let (lang, active_color, recent_colors, toast_visible, toast_text, hotkey_name) = {
                let state = APP_STATE.lock().unwrap();
                (
                    state.config.language,
                    state.active_color,
                    state.config.recent_colors.clone(),
                    state.toast_visible,
                    state.toast_text.clone(),
                    state.config.hotkey_name.clone(),
                )
            };
            let strings = get_strings(lang);

            let bg_brush = CreateSolidBrush(0x00181616);
            let full_rc = RECT { left: 0, top: 0, right: 500, bottom: 480 };
            FillRect(hdc, &full_rc, bg_brush as _);
            DeleteObject(bg_brush as _);

            SetBkMode(hdc, TRANSPARENT as _);

            let font_logo = CreateFontW(18, 0, 0, 0, FW_BOLD as _, 0, 0, 0, 1, 0, 0, 0, 0, "Segoe UI\0".encode_utf16().collect::<Vec<_>>().as_ptr());
            let font_label = CreateFontW(13, 0, 0, 0, FW_NORMAL as _, 0, 0, 0, 1, 0, 0, 0, 0, "Segoe UI\0".encode_utf16().collect::<Vec<_>>().as_ptr());
            let font_val = CreateFontW(16, 0, 0, 0, FW_BOLD as _, 0, 0, 0, 1, 0, 0, 0, 0, "Consolas\0".encode_utf16().collect::<Vec<_>>().as_ptr());
            let font_btn = CreateFontW(14, 0, 0, 0, FW_BOLD as _, 0, 0, 0, 1, 0, 0, 0, 0, "Segoe UI\0".encode_utf16().collect::<Vec<_>>().as_ptr());
            let font_big_btn = CreateFontW(18, 0, 0, 0, FW_BOLD as _, 0, 0, 0, 1, 0, 0, 0, 0, "Segoe UI\0".encode_utf16().collect::<Vec<_>>().as_ptr());

            SelectObject(hdc, font_logo as _);
            SetTextColor(hdc, 0x00E0E0E0);
            let mut brand_wide: Vec<u16> = "ETDPicker".encode_utf16().collect();
            let mut brand_rc = RECT { left: 20, top: 16, right: 200, bottom: 44 };
            DrawTextW(hdc, brand_wide.as_mut_ptr(), brand_wide.len() as _, &mut brand_rc, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

            let btn_bg_brush = CreateSolidBrush(0x002A2828);
            let btn_border_brush = CreateSolidBrush(0x004A4848);

            for btn in get_ui_buttons() {
                if btn.action_id >= 101 && btn.action_id <= 103 {
                    FillRect(hdc, &btn.rect, btn_bg_brush as _);
                    FrameRect(hdc, &btn.rect, btn_border_brush as _);

                    SelectObject(hdc, font_label as _);
                    SetTextColor(hdc, 0x00D0D0D0);
                    let label = match btn.action_id {
                        101 => if lang == Language::Turkish { "TR" } else { "EN" },
                        102 => "🎨",
                        103 => "⚙️",
                        _ => "",
                    };
                    let mut l_wide: Vec<u16> = label.encode_utf16().collect();
                    let mut r = btn.rect;
                    DrawTextW(hdc, l_wide.as_mut_ptr(), l_wide.len() as _, &mut r, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
                }
            }

            let swatch_box = RECT { left: 20, top: 68, right: 190, bottom: 238 };
            let active_clr_ref = (active_color.r as u32) | ((active_color.g as u32) << 8) | ((active_color.b as u32) << 16);
            let swatch_brush = CreateSolidBrush(active_clr_ref);
            FillRect(hdc, &swatch_box, swatch_brush as _);
            DeleteObject(swatch_brush as _);

            let swatch_border = CreateSolidBrush(0x00555555);
            FrameRect(hdc, &swatch_box, swatch_border as _);
            DeleteObject(swatch_border as _);

            SelectObject(hdc, font_label as _);
            SetTextColor(hdc, 0x00909090);
            let mut hex_lbl_wide: Vec<u16> = "HEX".encode_utf16().collect();
            let mut hex_lbl_rc = RECT { left: 210, top: 68, right: 330, bottom: 84 };
            DrawTextW(hdc, hex_lbl_wide.as_mut_ptr(), hex_lbl_wide.len() as _, &mut hex_lbl_rc, DT_LEFT | DT_SINGLELINE);

            SelectObject(hdc, font_val as _);
            SetTextColor(hdc, 0x00FFFFFF);
            let hex_val = active_color.to_hex();
            let mut hex_val_wide: Vec<u16> = hex_val.encode_utf16().collect();
            let mut hex_val_rc = RECT { left: 210, top: 84, right: 330, bottom: 106 };
            DrawTextW(hdc, hex_val_wide.as_mut_ptr(), hex_val_wide.len() as _, &mut hex_val_rc, DT_LEFT | DT_SINGLELINE);

            SelectObject(hdc, font_label as _);
            SetTextColor(hdc, 0x00909090);
            let mut rgb_lbl_wide: Vec<u16> = "RGB (Paint)".encode_utf16().collect();
            let mut rgb_lbl_rc = RECT { left: 210, top: 108, right: 330, bottom: 124 };
            DrawTextW(hdc, rgb_lbl_wide.as_mut_ptr(), rgb_lbl_wide.len() as _, &mut rgb_lbl_rc, DT_LEFT | DT_SINGLELINE);

            SelectObject(hdc, font_val as _);
            SetTextColor(hdc, 0x00FFFFFF);
            let rgb_val = active_color.to_rgb_string();
            let mut rgb_val_wide: Vec<u16> = rgb_val.encode_utf16().collect();
            let mut rgb_val_rc = RECT { left: 210, top: 124, right: 330, bottom: 146 };
            DrawTextW(hdc, rgb_val_wide.as_mut_ptr(), rgb_val_wide.len() as _, &mut rgb_val_rc, DT_LEFT | DT_SINGLELINE);

            for btn in get_ui_buttons() {
                if btn.action_id == 201 || btn.action_id == 202 {
                    FillRect(hdc, &btn.rect, btn_bg_brush as _);
                    FrameRect(hdc, &btn.rect, btn_border_brush as _);

                    SelectObject(hdc, font_label as _);
                    SetTextColor(hdc, 0x00E0E0E0);
                    let label = if btn.action_id == 201 { strings.copy_hex_btn } else { strings.copy_rgb_btn };
                    let mut l_wide: Vec<u16> = label.encode_utf16().collect();
                    let mut r = btn.rect;
                    DrawTextW(hdc, l_wide.as_mut_ptr(), l_wide.len() as _, &mut r, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
                } else if btn.action_id >= 301 && btn.action_id <= 303 {
                    FillRect(hdc, &btn.rect, btn_bg_brush as _);
                    FrameRect(hdc, &btn.rect, btn_border_brush as _);

                    SelectObject(hdc, font_label as _);
                    let (prefix, val) = match btn.action_id {
                        301 => ("R: ", active_color.r),
                        302 => ("G: ", active_color.g),
                        303 => ("B: ", active_color.b),
                        _ => ("", 0),
                    };
                    let label = format!("{}{}", prefix, val);
                    let mut l_wide: Vec<u16> = label.encode_utf16().collect();
                    let mut r = btn.rect;
                    SetTextColor(hdc, match btn.action_id {
                        301 => 0x005555FF,
                        302 => 0x0055FF55,
                        303 => 0x00FF8855,
                        _ => 0x00FFFFFF,
                    });
                    DrawTextW(hdc, l_wide.as_mut_ptr(), l_wide.len() as _, &mut r, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
                }
            }

            let (h, s, l) = active_color.to_hsl();
            let (c, m, y, k) = active_color.to_cmyk();
            SelectObject(hdc, font_label as _);
            SetTextColor(hdc, 0x00A0A0A0);

            let extra_str = format!("HSL: {}°, {}%, {}%  |  CMYK: {}%, {}%, {}%, {}%", h, s, l, c, m, y, k);
            let mut extra_wide: Vec<u16> = extra_str.encode_utf16().collect();
            let mut extra_rc = RECT { left: 210, top: 190, right: 480, bottom: 210 };
            DrawTextW(hdc, extra_wide.as_mut_ptr(), extra_wide.len() as _, &mut extra_rc, DT_LEFT | DT_SINGLELINE);

            SetTextColor(hdc, 0x00808080);
            let mut hint_wide: Vec<u16> = strings.paint_hint.encode_utf16().collect();
            let mut hint_rc = RECT { left: 210, top: 210, right: 480, bottom: 244 };
            DrawTextW(hdc, hint_wide.as_mut_ptr(), hint_wide.len() as _, &mut hint_rc, DT_LEFT | DT_NOPREFIX);

            SelectObject(hdc, font_btn as _);
            SetTextColor(hdc, 0x00CCCCCC);
            let mut rec_wide: Vec<u16> = strings.recent_colors.encode_utf16().collect();
            let mut rec_rc = RECT { left: 20, top: 280, right: 300, bottom: 304 };
            DrawTextW(hdc, rec_wide.as_mut_ptr(), rec_wide.len() as _, &mut rec_rc, DT_LEFT | DT_SINGLELINE | DT_NOPREFIX);

            for (i, c) in recent_colors.iter().enumerate().take(10) {
                let sx = 20 + (i as i32 * 46);
                let swatch_r = RECT { left: sx, top: 310, right: sx + 38, bottom: 348 };
                let clr = (c.r as u32) | ((c.g as u32) << 8) | ((c.b as u32) << 16);
                let b = CreateSolidBrush(clr);
                FillRect(hdc, &swatch_r, b as _);
                DeleteObject(b as _);

                let frame = CreateSolidBrush(0x004A4A4A);
                FrameRect(hdc, &swatch_r, frame as _);
                DeleteObject(frame as _);
            }

            let big_btn = &get_ui_buttons()[get_ui_buttons().len() - 1];
            let action_brush = CreateSolidBrush(0x00D66822);
            FillRect(hdc, &big_btn.rect, action_brush as _);
            DeleteObject(action_brush as _);

            let action_border = CreateSolidBrush(0x00FF8E3D);
            FrameRect(hdc, &big_btn.rect, action_border as _);
            DeleteObject(action_border as _);

            let hotkey_disp = if hotkey_name.is_empty() { "Alt + P".to_string() } else { hotkey_name };
            let pick_btn_label = format!("{} ({})", strings.pick_color_btn, hotkey_disp);
            let mut pick_wide: Vec<u16> = pick_btn_label.encode_utf16().collect();
            SelectObject(hdc, font_big_btn as _);
            SetTextColor(hdc, 0x00FFFFFF);
            let mut br = big_btn.rect;
            DrawTextW(hdc, pick_wide.as_mut_ptr(), pick_wide.len() as _, &mut br, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);

            if toast_visible {
                let toast_rc = RECT { left: 80, top: 248, right: 420, bottom: 276 };
                let toast_bg = CreateSolidBrush(0x002E7D32);
                FillRect(hdc, &toast_rc, toast_bg as _);
                DeleteObject(toast_bg as _);

                let toast_border = CreateSolidBrush(0x004CAF50);
                FrameRect(hdc, &toast_rc, toast_border as _);
                DeleteObject(toast_border as _);

                SelectObject(hdc, font_btn as _);
                SetTextColor(hdc, 0x00FFFFFF);
                let mut t_wide: Vec<u16> = toast_text.encode_utf16().collect();
                let mut tr = toast_rc;
                DrawTextW(hdc, t_wide.as_mut_ptr(), t_wide.len() as _, &mut tr, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
            }

            DeleteObject(btn_bg_brush as _);
            DeleteObject(btn_border_brush as _);
            DeleteObject(font_logo as _);
            DeleteObject(font_label as _);
            DeleteObject(font_val as _);
            DeleteObject(font_btn as _);
            DeleteObject(font_big_btn as _);

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
