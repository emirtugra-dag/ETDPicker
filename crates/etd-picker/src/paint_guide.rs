use crate::i18n::{get_strings, Language};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, FillRect, FrameRect,
    SelectObject, SetBkMode, SetTextColor, DT_LEFT, DT_NOPREFIX, DT_WORDBREAK,
    FW_BOLD, FW_NORMAL, TRANSPARENT,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::EnableWindow;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
    GetSystemMetrics, IsDialogMessageW, RegisterClassW, ShowWindow,
    TranslateMessage, CS_HREDRAW, CS_VREDRAW, MSG, SM_CXSCREEN, SM_CYSCREEN, SW_SHOW,
    WM_COMMAND, WM_DESTROY, WM_PAINT, WNDCLASSW, WS_CHILD, WS_EX_DLGMODALFRAME,
    WS_EX_TOPMOST, WS_POPUP, WS_TABSTOP, WS_VISIBLE,
};

pub fn show_paint_guide(parent_hwnd: HWND, lang: Language) {
    let strings = get_strings(lang);

    unsafe {
        let class_name: Vec<u16> = "ETDPaintGuideWindow\0".encode_utf16().collect();
        let hinstance = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null());

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(guide_wnd_proc),
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

        let width = 480;
        let height = 370;
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let x = (screen_w - width) / 2;
        let y = (screen_h - height) / 2;

        let title_wide: Vec<u16> = strings.paint_guide_title.encode_utf16().chain(std::iter::once(0)).collect();

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

        let btn_text: Vec<u16> = strings.paint_guide_close.encode_utf16().chain(std::iter::once(0)).collect();
        let btn_class: Vec<u16> = "BUTTON\0".encode_utf16().collect();
        CreateWindowExW(
            0,
            btn_class.as_ptr(),
            btn_text.as_ptr(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | 0x00000001, // BS_DEFPUSHBUTTON
            (width - 160) / 2,
            height - 54,
            160,
            36,
            hwnd,
            1001 as _,
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
    }
}

unsafe extern "system" fn guide_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as i32;
            if id == 1001 || id == 2 {
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
            let full_rc = RECT { left: 0, top: 0, right: 480, bottom: 370 };
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
            SetTextColor(hdc, 0x00FFD24C);

            let mut title_wide: Vec<u16> = strings.paint_guide_title.encode_utf16().collect();
            let mut title_rc = RECT { left: 24, top: 18, right: 456, bottom: 48 };
            DrawTextW(hdc, title_wide.as_mut_ptr(), title_wide.len() as _, &mut title_rc, DT_LEFT | DT_NOPREFIX);

            DeleteObject(font_title as _);

            let font_body = CreateFontW(
                15, 0, 0, 0, FW_NORMAL as _, 0, 0, 0, 1, 0, 0, 0, 0,
                "Segoe UI\0".encode_utf16().collect::<Vec<_>>().as_ptr(),
            );
            SelectObject(hdc, font_body as _);
            SetTextColor(hdc, 0x00E0E0E0);

            let steps = [
                strings.paint_guide_step1,
                strings.paint_guide_step2,
                strings.paint_guide_step3,
                strings.paint_guide_step4,
                strings.paint_guide_step5,
            ];

            let mut y = 62;
            for step in steps {
                let mut step_wide: Vec<u16> = step.encode_utf16().collect();
                let mut step_rc = RECT { left: 24, top: y, right: 456, bottom: y + 42 };
                DrawTextW(hdc, step_wide.as_mut_ptr(), step_wide.len() as _, &mut step_rc, DT_LEFT | DT_WORDBREAK | DT_NOPREFIX);
                y += 44;
            }

            SelectObject(hdc, old_font);
            DeleteObject(font_body as _);

            windows_sys::Win32::Graphics::Gdi::EndPaint(hwnd, &mut ps);
            0
        }
        WM_DESTROY => 0,
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
