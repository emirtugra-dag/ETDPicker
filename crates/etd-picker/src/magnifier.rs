use crate::color::RgbColor;
use crate::i18n::Language;
use std::sync::atomic::{AtomicBool, Ordering};
use windows_sys::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreatePen, CreateSolidBrush,
    DeleteDC, DeleteObject, DrawTextW, FillRect, FrameRect, GetDC, GetPixel, ReleaseDC,
    SelectObject, SetBkMode, SetTextColor, UpdateWindow, DT_CENTER, DT_NOPREFIX, DT_SINGLELINE,
    FW_BOLD, FW_NORMAL, PS_SOLID, SRCCOPY, TRANSPARENT,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_DOWN, VK_ESCAPE, VK_LEFT, VK_LBUTTON, VK_RBUTTON, VK_RETURN, VK_RIGHT,
    VK_SPACE, VK_UP,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetCursorPos,
    GetSystemMetrics, PeekMessageW, RegisterClassW, SetCursorPos, SetWindowsHookExW,
    SetWindowPos, ShowWindow, UnhookWindowsHookEx, CS_HREDRAW, CS_VREDRAW, HHOOK, KBDLLHOOKSTRUCT, MSG,
    PM_REMOVE, SM_CXSCREEN, SM_CYSCREEN, SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_SHOW,
    WH_KEYBOARD_LL, WM_DESTROY, WM_KEYDOWN, WM_SYSKEYDOWN, WNDCLASSW, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

static MAGNIFIER_ACTIVE: AtomicBool = AtomicBool::new(false);
static HOOK_DONE: AtomicBool = AtomicBool::new(false);
static HOOK_PICKED: AtomicBool = AtomicBool::new(false);

const GRID_SIZE: i32 = 13;
const PIXEL_ZOOM: i32 = 12;
const GRID_PIXELS: i32 = GRID_SIZE * PIXEL_ZOOM;
const WINDOW_WIDTH: i32 = 180;
const WINDOW_HEIGHT: i32 = 250;

pub struct MagnifierResult {
    pub selected: bool,
    pub color: RgbColor,
}

unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if n_code >= 0 && (wparam as u32 == WM_KEYDOWN || wparam as u32 == WM_SYSKEYDOWN) {
        let kbd = *(lparam as *const KBDLLHOOKSTRUCT);
        let vk = kbd.vkCode as u16;

        match vk {
            VK_LEFT => {
                let mut pt = POINT { x: 0, y: 0 };
                GetCursorPos(&mut pt);
                SetCursorPos(pt.x - 1, pt.y);
                return 1;
            }
            VK_RIGHT => {
                let mut pt = POINT { x: 0, y: 0 };
                GetCursorPos(&mut pt);
                SetCursorPos(pt.x + 1, pt.y);
                return 1;
            }
            VK_UP => {
                let mut pt = POINT { x: 0, y: 0 };
                GetCursorPos(&mut pt);
                SetCursorPos(pt.x, pt.y - 1);
                return 1;
            }
            VK_DOWN => {
                let mut pt = POINT { x: 0, y: 0 };
                GetCursorPos(&mut pt);
                SetCursorPos(pt.x, pt.y + 1);
                return 1;
            }
            VK_SPACE | VK_RETURN => {
                HOOK_PICKED.store(true, Ordering::SeqCst);
                HOOK_DONE.store(true, Ordering::SeqCst);
                return 1;
            }
            VK_ESCAPE => {
                HOOK_PICKED.store(false, Ordering::SeqCst);
                HOOK_DONE.store(true, Ordering::SeqCst);
                return 1;
            }
            _ => {}
        }
    }
    CallNextHookEx(0 as _, n_code, wparam, lparam)
}

pub fn pick_color_interactive(lang: Language) -> MagnifierResult {
    if MAGNIFIER_ACTIVE.swap(true, Ordering::SeqCst) {
        return MagnifierResult {
            selected: false,
            color: RgbColor::new(0, 0, 0),
        };
    }

    HOOK_DONE.store(false, Ordering::SeqCst);
    HOOK_PICKED.store(false, Ordering::SeqCst);

    let is_picked;
    let mut picked_color = RgbColor::new(0, 0, 0);

    unsafe {
        let class_name: Vec<u16> = "ETDMagnifierWindow\0".encode_utf16().collect();
        let hinstance = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null());

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(magnifier_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: 0 as _,
            hCursor: 0 as _,
            hbrBackground: 0 as _,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
        };
        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class_name.as_ptr(),
            std::ptr::null(),
            WS_POPUP,
            0,
            0,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            0 as _,
            0 as _,
            hinstance,
            std::ptr::null_mut(),
        );

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        // Install Low-Level Keyboard Hook to intercept arrow keys and space/esc
        let hook: HHOOK = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(low_level_keyboard_proc),
            hinstance,
            0,
        );

        let mut msg: MSG = std::mem::zeroed();
        let hdc_screen = GetDC(0 as _);
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        let hbm_mem = CreateCompatibleBitmap(hdc_screen, WINDOW_WIDTH, WINDOW_HEIGHT);
        let old_bmp = SelectObject(hdc_mem, hbm_mem as _);

        let hdc_capture = CreateCompatibleDC(hdc_screen);
        let hbm_capture = CreateCompatibleBitmap(hdc_screen, GRID_SIZE, GRID_SIZE);
        let old_cap_bmp = SelectObject(hdc_capture, hbm_capture as _);

        let font_title = CreateFontW(
            14, 0, 0, 0, FW_BOLD as _, 0, 0, 0, 1, 0, 0, 0, 0,
            "Segoe UI\0".encode_utf16().collect::<Vec<_>>().as_ptr(),
        );
        let font_small = CreateFontW(
            11, 0, 0, 0, FW_NORMAL as _, 0, 0, 0, 1, 0, 0, 0, 0,
            "Segoe UI\0".encode_utf16().collect::<Vec<_>>().as_ptr(),
        );

        std::thread::sleep(std::time::Duration::from_millis(100));

        loop {
            while PeekMessageW(&mut msg, 0 as _, 0, 0, PM_REMOVE) != 0 {
                DispatchMessageW(&msg);
            }

            if HOOK_DONE.load(Ordering::SeqCst) {
                is_picked = HOOK_PICKED.load(Ordering::SeqCst);
                break;
            }

            // Mouse click checks
            if (GetAsyncKeyState(VK_LBUTTON as i32) as u16 & 0x8000) != 0 {
                is_picked = true;
                break;
            }

            if (GetAsyncKeyState(VK_RBUTTON as i32) as u16 & 0x8000) != 0 {
                is_picked = false;
                break;
            }

            let mut pt = POINT { x: 0, y: 0 };
            GetCursorPos(&mut pt);

            let clr_ref: COLORREF = GetPixel(hdc_screen, pt.x, pt.y);
            picked_color = RgbColor::new(
                (clr_ref & 0xFF) as u8,
                ((clr_ref >> 8) & 0xFF) as u8,
                ((clr_ref >> 16) & 0xFF) as u8,
            );

            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);

            let mut win_x = pt.x + 24;
            let mut win_y = pt.y + 24;

            if win_x + WINDOW_WIDTH > screen_w {
                win_x = pt.x - WINDOW_WIDTH - 24;
            }
            if win_y + WINDOW_HEIGHT > screen_h {
                win_y = pt.y - WINDOW_HEIGHT - 24;
            }
            if win_x < 0 {
                win_x = 0;
            }
            if win_y < 0 {
                win_y = 0;
            }

            SetWindowPos(
                hwnd,
                0 as _,
                win_x,
                win_y,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );

            let half = GRID_SIZE / 2;
            BitBlt(
                hdc_capture,
                0,
                0,
                GRID_SIZE,
                GRID_SIZE,
                hdc_screen,
                pt.x - half,
                pt.y - half,
                SRCCOPY,
            );

            let bg_rect = RECT {
                left: 0,
                top: 0,
                right: WINDOW_WIDTH,
                bottom: WINDOW_HEIGHT,
            };
            let bg_brush = CreateSolidBrush(0x00221E1E);
            FillRect(hdc_mem, &bg_rect, bg_brush as _);
            DeleteObject(bg_brush as _);

            let grid_offset_x = (WINDOW_WIDTH - GRID_PIXELS) / 2;
            let grid_offset_y = 12;

            for gy in 0..GRID_SIZE {
                for gx in 0..GRID_SIZE {
                    let pixel_color = GetPixel(hdc_capture, gx, gy);
                    let p_brush = CreateSolidBrush(pixel_color);
                    let pr = RECT {
                        left: grid_offset_x + gx * PIXEL_ZOOM,
                        top: grid_offset_y + gy * PIXEL_ZOOM,
                        right: grid_offset_x + (gx + 1) * PIXEL_ZOOM,
                        bottom: grid_offset_y + (gy + 1) * PIXEL_ZOOM,
                    };
                    FillRect(hdc_mem, &pr, p_brush as _);
                    DeleteObject(p_brush as _);
                }
            }

            let grid_pen = CreatePen(PS_SOLID as _, 1, 0x00333333);
            let old_pen = SelectObject(hdc_mem, grid_pen as _);
            for i in 0..=GRID_SIZE {
                let gx = grid_offset_x + i * PIXEL_ZOOM;
                windows_sys::Win32::Graphics::Gdi::MoveToEx(hdc_mem, gx, grid_offset_y, std::ptr::null_mut());
                windows_sys::Win32::Graphics::Gdi::LineTo(hdc_mem, gx, grid_offset_y + GRID_PIXELS);

                let gy = grid_offset_y + i * PIXEL_ZOOM;
                windows_sys::Win32::Graphics::Gdi::MoveToEx(hdc_mem, grid_offset_x, gy, std::ptr::null_mut());
                windows_sys::Win32::Graphics::Gdi::LineTo(hdc_mem, grid_offset_x + GRID_PIXELS, gy);
            }

            let center_rect = RECT {
                left: grid_offset_x + half * PIXEL_ZOOM - 1,
                top: grid_offset_y + half * PIXEL_ZOOM - 1,
                right: grid_offset_x + (half + 1) * PIXEL_ZOOM + 1,
                bottom: grid_offset_y + (half + 1) * PIXEL_ZOOM + 1,
            };
            let hl_pen = CreatePen(PS_SOLID as _, 2, if picked_color.is_dark() { 0x00FFFFFF } else { 0x00000000 });
            SelectObject(hdc_mem, hl_pen as _);
            let null_brush = windows_sys::Win32::Graphics::Gdi::GetStockObject(windows_sys::Win32::Graphics::Gdi::NULL_BRUSH as _);
            let old_hl_brush = SelectObject(hdc_mem, null_brush);
            windows_sys::Win32::Graphics::Gdi::Rectangle(
                hdc_mem,
                center_rect.left,
                center_rect.top,
                center_rect.right,
                center_rect.bottom,
            );
            SelectObject(hdc_mem, old_hl_brush);
            DeleteObject(hl_pen as _);

            let outer_rect = RECT {
                left: grid_offset_x - 1,
                top: grid_offset_y - 1,
                right: grid_offset_x + GRID_PIXELS + 1,
                bottom: grid_offset_y + GRID_PIXELS + 1,
            };
            let border_brush = CreateSolidBrush(0x00555555);
            FrameRect(hdc_mem, &outer_rect, border_brush as _);
            DeleteObject(border_brush as _);

            SelectObject(hdc_mem, old_pen);
            DeleteObject(grid_pen as _);

            let info_top = grid_offset_y + GRID_PIXELS + 10;
            let swatch_rect = RECT {
                left: 14,
                top: info_top,
                right: 38,
                bottom: info_top + 24,
            };
            let swatch_brush = CreateSolidBrush(
                (picked_color.r as u32) | ((picked_color.g as u32) << 8) | ((picked_color.b as u32) << 16),
            );
            FillRect(hdc_mem, &swatch_rect, swatch_brush as _);
            DeleteObject(swatch_brush as _);

            let swatch_border = CreateSolidBrush(0x00888888);
            FrameRect(hdc_mem, &swatch_rect, swatch_border as _);
            DeleteObject(swatch_border as _);

            SetBkMode(hdc_mem, TRANSPARENT as _);
            SetTextColor(hdc_mem, 0x00FFFFFF);

            let old_font = SelectObject(hdc_mem, font_title as _);
            let hex_str = picked_color.to_hex();
            let mut hex_wide: Vec<u16> = hex_str.encode_utf16().collect();
            let mut hex_rc = RECT {
                left: 46,
                top: info_top,
                right: WINDOW_WIDTH - 12,
                bottom: info_top + 14,
            };
            DrawTextW(hdc_mem, hex_wide.as_mut_ptr(), hex_wide.len() as _, &mut hex_rc, DT_SINGLELINE);

            SelectObject(hdc_mem, font_small as _);
            SetTextColor(hdc_mem, 0x00B0B0B0);
            let rgb_str = format!("RGB: {}, {}, {}", picked_color.r, picked_color.g, picked_color.b);
            let mut rgb_wide: Vec<u16> = rgb_str.encode_utf16().collect();
            let mut rgb_rc = RECT {
                left: 46,
                top: info_top + 14,
                right: WINDOW_WIDTH - 12,
                bottom: info_top + 28,
            };
            DrawTextW(hdc_mem, rgb_wide.as_mut_ptr(), rgb_wide.len() as _, &mut rgb_rc, DT_SINGLELINE);

            let inst_str = match lang {
                Language::Turkish => "[Tık/Space: Seç] [Esc: İptal]",
                Language::English => "[Click/Space: Pick] [Esc: Cancel]",
            };
            let mut inst_wide: Vec<u16> = inst_str.encode_utf16().collect();
            let mut inst_rc = RECT {
                left: 4,
                top: info_top + 34,
                right: WINDOW_WIDTH - 4,
                bottom: WINDOW_HEIGHT - 4,
            };
            SetTextColor(hdc_mem, 0x00808080);
            DrawTextW(hdc_mem, inst_wide.as_mut_ptr(), inst_wide.len() as _, &mut inst_rc, DT_CENTER | DT_SINGLELINE | DT_NOPREFIX);

            SelectObject(hdc_mem, old_font);

            let win_border_rect = RECT {
                left: 0,
                top: 0,
                right: WINDOW_WIDTH,
                bottom: WINDOW_HEIGHT,
            };
            let win_border_brush = CreateSolidBrush(0x004A4A4A);
            FrameRect(hdc_mem, &win_border_rect, win_border_brush as _);
            DeleteObject(win_border_brush as _);

            let hdc_win = GetDC(hwnd);
            BitBlt(
                hdc_win,
                0,
                0,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
                hdc_mem,
                0,
                0,
                SRCCOPY,
            );
            ReleaseDC(hwnd, hdc_win);

            std::thread::sleep(std::time::Duration::from_millis(15));
        }

        if hook != 0 as _ {
            UnhookWindowsHookEx(hook);
        }

        SelectObject(hdc_mem, old_bmp);
        DeleteObject(hbm_mem as _);
        DeleteDC(hdc_mem);

        SelectObject(hdc_capture, old_cap_bmp);
        DeleteObject(hbm_capture as _);
        DeleteDC(hdc_capture);

        ReleaseDC(0 as _, hdc_screen);
        DeleteObject(font_title as _);
        DeleteObject(font_small as _);

        DestroyWindow(hwnd);
    }

    MAGNIFIER_ACTIVE.store(false, Ordering::SeqCst);

    MagnifierResult {
        selected: is_picked,
        color: picked_color,
    }
}

unsafe extern "system" fn magnifier_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => 0,
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
