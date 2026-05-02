//! Windows-only Servo embed: **default** Slint `examples/servo`-style surfman [`GPURenderingContext`]
//! (readback → egui); optional legacy **borderless popup** + [`WindowRenderingContext`] when
//! `TONET_SERVO_WIN32_POPUP` is set (non-`0`).
//!
//! **Popup input:** pointer and wheel from a native [`WNDPROC`] subclass on the popup HWND.
//! **Slint embed input:** [`ServoWinHost::feed_egui_servo_embed_input`] from the egui content rect.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem::transmute;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use dpi::PhysicalSize;
use euclid::Scale;
use http::header::CONTENT_TYPE;
use http::{HeaderMap, HeaderValue, Method, StatusCode};
use raw_window_handle::{HasDisplayHandle as _, RawWindowHandle, Win32WindowHandle, WindowHandle};
use servo::{
    AuthenticationRequest, Code, ColorPicker, CompositionEvent, CompositionState, ConsoleLogLevel, ContextMenu,
    ContextMenuAction, ContextMenuItem, Cursor, Notification,
    DeviceIntPoint, DeviceIntRect, DeviceIntSize, DevicePoint, EmbedderControl, EmbedderControlId, EventLoopWaker,
    FilePicker, InputEvent, Key, KeyState, KeyboardEvent, LoadStatus, Location, Modifiers, MouseButton,
    MouseButtonAction, MouseButtonEvent, MouseLeftViewportEvent, MouseMoveEvent, NamedKey, PermissionFeature,
    PermissionRequest, RenderingContext, RgbColor, SelectElement, SelectElementOptionOrOptgroup, Servo, ServoBuilder,
    SimpleDialog, WebResourceLoad, WebResourceResponse, WebView, WebViewBuilder, WebViewDelegate, WebViewPoint,
    WheelDelta, WheelEvent, WheelMode, WindowRenderingContext,
};
use servo::ImeEvent as ServoImeEvent;
use url::Url;
use windows_sys::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    ClientToScreen, GetStockObject, ScreenToClient, UpdateWindow, HBRUSH, WHITE_BRUSH,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetFocus, SetFocus};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect,
    GetWindowLongPtrW, GetWindowRect, IsChild, LoadCursorW, PeekMessageW, RegisterClassExW, SetCursor,
    SetWindowLongPtrW, SetWindowPos, ShowWindow, TranslateMessage, CS_HREDRAW, CS_VREDRAW, GWLP_USERDATA,
    GWLP_WNDPROC, HTCLIENT, HWND_TOP, IDC_APPSTARTING, IDC_ARROW, IDC_CROSS, IDC_HAND, IDC_HELP, IDC_IBEAM,
    IDC_NO, IDC_SIZENESW, IDC_SIZENS, IDC_SIZENWSE, IDC_SIZEWE, IDC_SIZEALL, IDC_WAIT, MA_NOACTIVATE, MSG,
    PM_REMOVE, SWP_NOZORDER, SWP_SHOWWINDOW, SW_HIDE, SW_SHOW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_MOUSEACTIVATE,
    WM_NCDESTROY, WM_SETCURSOR, WNDCLASSEXW, WNDPROC, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP,
    WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WS_EX_NOACTIVATE, WS_POPUP,
    WS_VISIBLE,
};

use crate::i18n::Locale;
use crate::platform_windows::win32_hwnd_from_frame;
use crate::servo_engine::background_download;
use crate::servo_engine::download_heuristic;
use crate::servo_engine::servo_favicon;
use crate::servo_engine::slint_embed::GPURenderingContext;
use crate::servo_engine::tonet_scheme_html;
use crate::servo_engine::visit_policy;
use crate::tab::Tab;
use crate::ui::omnibox_id;

/// Legacy Win32-owned popup surface (`TONET_SERVO_WIN32_POPUP=1`). Default is Slint-style in-process surfman.
#[inline]
fn embed_uses_win32_popup() -> bool {
    std::env::var_os("TONET_SERVO_WIN32_POPUP").is_some_and(|v| v != "0")
}

/// Latest shell-facing state read from the Servo [`WebView`] (URL bar, title, loading, chrome nav).
#[derive(Clone)]
pub(crate) struct ServoShellSnapshot {
    pub committed_url: Option<String>,
    pub title: Option<String>,
    pub load_status: LoadStatus,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    /// PNG bytes from [`WebView::favicon`], when present and encodable.
    pub favicon_png: Option<Vec<u8>>,
}

impl ServoShellSnapshot {
    fn capture_from(webview: &WebView) -> Self {
        let favicon_png = webview
            .favicon()
            .and_then(|f| servo_favicon::encode_image_as_png(&*f));
        Self {
            committed_url: webview.url().map(|u| u.as_str().to_owned()),
            title: webview.page_title(),
            load_status: webview.load_status(),
            can_go_back: webview.can_go_back(),
            can_go_forward: webview.can_go_forward(),
            favicon_png,
        }
    }
}

struct Win32PopupWindow(HWND);

impl Drop for Win32PopupWindow {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                DestroyWindow(self.0);
            }
        }
    }
}

unsafe extern "system" fn servo_popup_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn register_popup_class(hinstance: HINSTANCE) -> Result<(), ()> {
    let class_name = windows_sys::core::w!("TonetServoExperimentalPopup");
    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(servo_popup_wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: std::ptr::null_mut(),
        hCursor: unsafe { LoadCursorW(std::ptr::null_mut(), IDC_ARROW) },
        hbrBackground: unsafe { GetStockObject(WHITE_BRUSH) as HBRUSH },
        lpszMenuName: std::ptr::null(),
        lpszClassName: class_name,
        hIconSm: std::ptr::null_mut(),
    };
    let atom = unsafe { RegisterClassExW(&wc) };
    if atom == 0 {
        let err = unsafe { windows_sys::Win32::Foundation::GetLastError() };
        if err != windows_sys::Win32::Foundation::ERROR_CLASS_ALREADY_EXISTS {
            return Err(());
        }
    }
    Ok(())
}

struct EguiRepaintWaker(egui::Context);

impl EventLoopWaker for EguiRepaintWaker {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(Self(self.0.clone()))
    }

    fn wake(&self) {
        self.0.request_repaint();
    }
}

fn servo_permission_feature_token(f: PermissionFeature) -> &'static str {
    match f {
        PermissionFeature::Geolocation => "geolocation",
        PermissionFeature::Notifications => "notifications",
        PermissionFeature::Push => "push",
        PermissionFeature::Midi => "midi",
        PermissionFeature::Camera => "camera",
        PermissionFeature::Microphone => "microphone",
        PermissionFeature::Speaker => "speaker",
        PermissionFeature::DeviceInfo => "device_info",
        PermissionFeature::BackgroundSync => "background_sync",
        PermissionFeature::Bluetooth => "bluetooth",
        PermissionFeature::PersistentStorage => "persistent_storage",
    }
}

fn servo_permission_cache_key(origin: &str, f: PermissionFeature) -> String {
    format!("{}\t{}", origin, servo_permission_feature_token(f))
}

fn servo_page_origin_for_permissions(webview: &WebView) -> String {
    let Some(url) = webview.url() else {
        return "unknown".to_string();
    };
    let s = url.origin().ascii_serialization();
    if s.is_empty() {
        "unknown".to_string()
    } else {
        s
    }
}

const SERVO_WEB_NOTIFICATION_BODY_MAX: usize = 800;
const SERVO_CONSOLE_MESSAGE_MAX: usize = 4_096;
const SERVO_CONSOLE_HOST_QUEUE_CAP: usize = 256;

#[inline]
fn map_console_level(level: ConsoleLogLevel) -> crate::tab::ServoConsoleLevel {
    match level {
        ConsoleLogLevel::Log => crate::tab::ServoConsoleLevel::Log,
        ConsoleLogLevel::Debug => crate::tab::ServoConsoleLevel::Debug,
        ConsoleLogLevel::Info => crate::tab::ServoConsoleLevel::Info,
        ConsoleLogLevel::Warn => crate::tab::ServoConsoleLevel::Warn,
        ConsoleLogLevel::Error => crate::tab::ServoConsoleLevel::Error,
        ConsoleLogLevel::Trace => crate::tab::ServoConsoleLevel::Trace,
    }
}

#[derive(Clone)]
struct ServoWebNotificationToast {
    title: Option<String>,
    body: String,
    until: Instant,
}

impl ServoWebNotificationToast {
    fn from_notification(n: Notification) -> Self {
        let title = {
            let t = n.title.trim();
            if t.is_empty() {
                None
            } else {
                Some(t.to_string())
            }
        };
        let mut body = n.body;
        if body.len() > SERVO_WEB_NOTIFICATION_BODY_MAX {
            body.truncate(SERVO_WEB_NOTIFICATION_BODY_MAX);
            body.push('…');
        }
        let secs = if n.require_interaction { 45 } else { 10 };
        Self {
            title,
            body,
            until: Instant::now() + Duration::from_secs(secs),
        }
    }
}

/// Marks paint needed and queues [`SimpleDialog`] (`alert` / `confirm` / `prompt`) for egui.
struct TonetServoWebViewDelegate {
    egui_ctx: egui::Context,
    needs_paint: Rc<Cell<bool>>,
    dialog_pending: Rc<RefCell<Option<SimpleDialog>>>,
    prompt_draft: Rc<RefCell<String>>,
    /// `egui::Window::open`: stays true until the user closes the chrome or the dialog completes.
    dialog_window_open: Rc<Cell<bool>>,
    context_menu_pending: Rc<RefCell<Option<ContextMenu>>>,
    context_menu_window_open: Rc<Cell<bool>>,
    select_pending: Rc<RefCell<Option<SelectElement>>>,
    /// Draft choice for [`SelectElement::select`] before [`SelectElement::submit`].
    select_draft: Rc<Cell<Option<usize>>>,
    select_window_open: Rc<Cell<bool>>,
    color_picker_pending: Rc<RefCell<Option<ColorPicker>>>,
    color_picker_draft: Rc<RefCell<[u8; 3]>>,
    color_picker_window_open: Rc<Cell<bool>>,
    file_picker_waiting: Rc<RefCell<Option<FilePicker>>>,
    file_picker_rx: Rc<RefCell<Option<mpsc::Receiver<Option<Vec<PathBuf>>>>>>,
    permission_pending: Rc<RefCell<Option<PermissionRequest>>>,
    permission_window_open: Rc<Cell<bool>>,
    /// In-process cache: [`servo_permission_cache_key`] → user allowed (true) or denied (false).
    permission_cache: Rc<RefCell<HashMap<String, bool>>>,
    /// Document origin ([`servo_page_origin_for_permissions`]) for the queued [`PermissionRequest`].
    permission_prompt_origin: Rc<RefCell<Option<String>>>,
    auth_pending: Rc<RefCell<Option<AuthenticationRequest>>>,
    auth_user_draft: Rc<RefCell<String>>,
    auth_pass_draft: Rc<RefCell<String>>,
    auth_window_open: Rc<Cell<bool>>,
    notification_toast: Rc<RefCell<Option<ServoWebNotificationToast>>>,
    console_pending: Rc<RefCell<VecDeque<(ConsoleLogLevel, String)>>>,
    background_download_done: Arc<Mutex<Vec<background_download::CompletedBackgroundDownload>>>,
    tonet_scheme_state: Arc<Mutex<tonet_scheme_html::TonetSchemeSharedState>>,
}

impl TonetServoWebViewDelegate {
    fn dismiss_auth_pending(&self) {
        let _ = self.auth_pending.borrow_mut().take();
    }

    fn dismiss_file_picker_in_flight(&self) {
        *self.file_picker_rx.borrow_mut() = None;
        if let Some(fp) = self.file_picker_waiting.borrow_mut().take() {
            fp.dismiss();
        }
    }

    fn dismiss_permission_pending(&self) {
        *self.permission_prompt_origin.borrow_mut() = None;
        if let Some(p) = self.permission_pending.borrow_mut().take() {
            p.deny();
        }
    }
}

impl WebViewDelegate for TonetServoWebViewDelegate {
    fn notify_new_frame_ready(&self, _: WebView) {
        self.needs_paint.set(true);
    }

    fn request_permission(&self, webview: WebView, req: PermissionRequest) {
        self.dismiss_auth_pending();
        let feat = req.feature();
        let origin = servo_page_origin_for_permissions(&webview);
        let key = servo_permission_cache_key(&origin, feat);
        if let Some(&allowed) = self.permission_cache.borrow().get(&key) {
            if allowed {
                req.allow();
            } else {
                req.deny();
            }
            self.needs_paint.set(true);
            self.egui_ctx.request_repaint();
            return;
        }

        self.dismiss_permission_pending();
        self.dismiss_file_picker_in_flight();
        if let Some(cm) = self.context_menu_pending.borrow_mut().take() {
            cm.dismiss();
        }
        if let Some(s) = self.select_pending.borrow_mut().take() {
            drop(s);
        }
        if let Some(c) = self.color_picker_pending.borrow_mut().take() {
            drop(c);
        }
        *self.permission_prompt_origin.borrow_mut() = Some(origin);
        *self.permission_pending.borrow_mut() = Some(req);
        self.permission_window_open.set(true);
        self.needs_paint.set(true);
        self.egui_ctx.request_repaint();
    }

    fn request_authentication(&self, _webview: WebView, request: AuthenticationRequest) {
        self.dismiss_auth_pending();
        self.dismiss_permission_pending();
        self.dismiss_file_picker_in_flight();
        if let Some(cm) = self.context_menu_pending.borrow_mut().take() {
            cm.dismiss();
        }
        if let Some(s) = self.select_pending.borrow_mut().take() {
            drop(s);
        }
        if let Some(c) = self.color_picker_pending.borrow_mut().take() {
            drop(c);
        }
        *self.auth_user_draft.borrow_mut() = String::new();
        *self.auth_pass_draft.borrow_mut() = String::new();
        *self.auth_pending.borrow_mut() = Some(request);
        self.auth_window_open.set(true);
        self.needs_paint.set(true);
        self.egui_ctx.request_repaint();
    }

    fn show_embedder_control(&self, _webview: WebView, control: EmbedderControl) {
        match control {
            EmbedderControl::SimpleDialog(d) => {
                self.dismiss_auth_pending();
                self.dismiss_permission_pending();
                self.dismiss_file_picker_in_flight();
                if let Some(cm) = self.context_menu_pending.borrow_mut().take() {
                    cm.dismiss();
                }
                if let Some(s) = self.select_pending.borrow_mut().take() {
                    drop(s);
                }
                if let Some(c) = self.color_picker_pending.borrow_mut().take() {
                    drop(c);
                }
                match &d {
                    SimpleDialog::Prompt(p) => {
                        *self.prompt_draft.borrow_mut() = p.current_value().to_owned();
                    }
                    _ => self.prompt_draft.borrow_mut().clear(),
                }
                *self.dialog_pending.borrow_mut() = Some(d);
                self.dialog_window_open.set(true);
                self.needs_paint.set(true);
                self.egui_ctx.request_repaint();
            }
            EmbedderControl::ContextMenu(cm) => {
                self.dismiss_auth_pending();
                self.dismiss_permission_pending();
                self.dismiss_file_picker_in_flight();
                if let Some(old) = self.context_menu_pending.borrow_mut().take() {
                    old.dismiss();
                }
                if let Some(s) = self.select_pending.borrow_mut().take() {
                    drop(s);
                }
                if let Some(c) = self.color_picker_pending.borrow_mut().take() {
                    drop(c);
                }
                *self.context_menu_pending.borrow_mut() = Some(cm);
                self.context_menu_window_open.set(true);
                self.needs_paint.set(true);
                self.egui_ctx.request_repaint();
            }
            EmbedderControl::SelectElement(s) => {
                self.dismiss_auth_pending();
                self.dismiss_permission_pending();
                self.dismiss_file_picker_in_flight();
                if let Some(cm) = self.context_menu_pending.borrow_mut().take() {
                    cm.dismiss();
                }
                if let Some(old) = self.select_pending.borrow_mut().take() {
                    drop(old);
                }
                if let Some(c) = self.color_picker_pending.borrow_mut().take() {
                    drop(c);
                }
                self.select_draft.set(s.selected_option());
                *self.select_pending.borrow_mut() = Some(s);
                self.select_window_open.set(true);
                self.needs_paint.set(true);
                self.egui_ctx.request_repaint();
            }
            EmbedderControl::ColorPicker(c) => {
                self.dismiss_auth_pending();
                self.dismiss_permission_pending();
                self.dismiss_file_picker_in_flight();
                if let Some(cm) = self.context_menu_pending.borrow_mut().take() {
                    cm.dismiss();
                }
                if let Some(old) = self.select_pending.borrow_mut().take() {
                    drop(old);
                }
                if let Some(old) = self.color_picker_pending.borrow_mut().take() {
                    drop(old);
                }
                *self.color_picker_draft.borrow_mut() = servo_rgb_to_array(c.current_color());
                *self.color_picker_pending.borrow_mut() = Some(c);
                self.color_picker_window_open.set(true);
                self.needs_paint.set(true);
                self.egui_ctx.request_repaint();
            }
            EmbedderControl::FilePicker(fp) => {
                self.dismiss_auth_pending();
                self.dismiss_permission_pending();
                self.dismiss_file_picker_in_flight();
                if let Some(cm) = self.context_menu_pending.borrow_mut().take() {
                    cm.dismiss();
                }
                if let Some(s) = self.select_pending.borrow_mut().take() {
                    drop(s);
                }
                if let Some(c) = self.color_picker_pending.borrow_mut().take() {
                    drop(c);
                }
                let patterns: Vec<String> = fp.filter_patterns().iter().map(|p| p.0.clone()).collect();
                let allow = fp.allow_select_multiple();
                *self.file_picker_waiting.borrow_mut() = Some(fp);
                let (tx, rx) = mpsc::channel();
                *self.file_picker_rx.borrow_mut() = Some(rx);
                thread::spawn(move || {
                    let out = run_native_file_pick(&patterns, allow);
                    let _ = tx.send(out);
                });
                self.needs_paint.set(true);
                self.egui_ctx.request_repaint();
            }
            EmbedderControl::InputMethod(_) => {
                // No separate embedder IME surface: `forward_egui_keyboard` maps egui `Event::Ime`
                // into Servo when the page holds keyboard capture.
            }
        }
    }

    fn hide_embedder_control(&self, _webview: WebView, control_id: EmbedderControlId) {
        {
            let mut slot = self.context_menu_pending.borrow_mut();
            let matches = slot.as_ref().is_some_and(|cm| cm.id() == control_id);
            if matches {
                if let Some(cm) = slot.take() {
                    cm.dismiss();
                    self.needs_paint.set(true);
                    self.egui_ctx.request_repaint();
                }
            }
        }
        {
            let mut slot = self.select_pending.borrow_mut();
            let matches = slot.as_ref().is_some_and(|s| s.id() == control_id);
            if matches {
                if let Some(s) = slot.take() {
                    drop(s);
                    self.needs_paint.set(true);
                    self.egui_ctx.request_repaint();
                }
            }
        }
        {
            let mut slot = self.color_picker_pending.borrow_mut();
            let matches = slot.as_ref().is_some_and(|c| c.id() == control_id);
            if matches {
                if let Some(c) = slot.take() {
                    drop(c);
                    self.needs_paint.set(true);
                    self.egui_ctx.request_repaint();
                }
            }
        }
        {
            let mut slot = self.file_picker_waiting.borrow_mut();
            let matches = slot.as_ref().is_some_and(|f| f.id() == control_id);
            if matches {
                *self.file_picker_rx.borrow_mut() = None;
                if let Some(fp) = slot.take() {
                    fp.dismiss();
                    self.needs_paint.set(true);
                    self.egui_ctx.request_repaint();
                }
            }
        }
    }

    fn show_notification(&self, _webview: WebView, n: Notification) {
        *self.notification_toast.borrow_mut() = Some(ServoWebNotificationToast::from_notification(n));
        self.needs_paint.set(true);
        self.egui_ctx.request_repaint();
    }

    fn show_console_message(&self, _webview: WebView, level: ConsoleLogLevel, mut message: String) {
        if message.len() > SERVO_CONSOLE_MESSAGE_MAX {
            message.truncate(SERVO_CONSOLE_MESSAGE_MAX);
            message.push('…');
        }
        let mut q = self.console_pending.borrow_mut();
        while q.len() >= SERVO_CONSOLE_HOST_QUEUE_CAP {
            q.pop_front();
        }
        q.push_back((level, message));
        self.needs_paint.set(true);
        self.egui_ctx.request_repaint();
    }

    fn load_web_resource(&self, _webview: WebView, load: WebResourceLoad) {
        let req = load.request();
        if req.url.scheme() == "tonet" && req.method == Method::GET {
            let bytes = self
                .tonet_scheme_state
                .lock()
                .ok()
                .and_then(|mut g| tonet_scheme_html::document_bytes_for_tonet_url(&req.url, &mut g))
                .unwrap_or_else(|| {
                    b"<!DOCTYPE html><html><head><meta charset=\"utf-8\"/><title>tonet</title></head><body><p>Unknown tonet URL.</p></body></html>".to_vec()
                });
            let mut headers = HeaderMap::new();
            let _ = headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            let resp = WebResourceResponse::new(req.url.clone())
                .status_code(StatusCode::OK)
                .headers(headers);
            let mut intercepted = load.intercept(resp);
            intercepted.send_body_data(bytes);
            intercepted.finish();
            self.needs_paint.set(true);
            self.egui_ctx.request_repaint();
            return;
        }
        let intercept = download_heuristic::should_intercept_main_frame_binary_get(
            &req.method,
            &req.url,
            req.is_for_main_frame,
            req.is_redirect,
        ) || (download_heuristic::should_head_probe_main_frame_binary_get(
            &req.method,
            &req.url,
            req.is_for_main_frame,
            req.is_redirect,
        ) && background_download::head_suggests_intercept_binary_get(req.url.as_str()));
        if !intercept {
            return;
        }
        let url_s = req.url.to_string();
        let done = self.background_download_done.clone();
        let resp = WebResourceResponse::new(req.url.clone()).status_code(StatusCode::NO_CONTENT);
        drop(load.intercept(resp));
        self.needs_paint.set(true);
        self.egui_ctx.request_repaint();
        thread::spawn(move || {
            if let Ok(completed) = background_download::download_url_to_user_picked_file(&url_s) {
                if let Ok(mut g) = done.lock() {
                    g.push(completed);
                }
            }
        });
    }
}

#[inline]
fn system_cursor_idc(c: Cursor) -> windows_sys::core::PCWSTR {
    match c {
        Cursor::None | Cursor::Default | Cursor::Alias | Cursor::Copy | Cursor::ContextMenu | Cursor::ZoomIn
        | Cursor::ZoomOut => IDC_ARROW,
        Cursor::Pointer | Cursor::Grab | Cursor::Grabbing => IDC_HAND,
        Cursor::Help => IDC_HELP,
        Cursor::Progress => IDC_APPSTARTING,
        Cursor::Wait => IDC_WAIT,
        Cursor::Cell | Cursor::Crosshair => IDC_CROSS,
        Cursor::Text | Cursor::VerticalText => IDC_IBEAM,
        Cursor::Move | Cursor::AllScroll => IDC_SIZEALL,
        Cursor::NoDrop | Cursor::NotAllowed => IDC_NO,
        Cursor::EResize | Cursor::WResize | Cursor::EwResize | Cursor::RowResize => IDC_SIZEWE,
        Cursor::NResize | Cursor::SResize | Cursor::NsResize | Cursor::ColResize => IDC_SIZENS,
        Cursor::NeResize | Cursor::SwResize | Cursor::NeswResize => IDC_SIZENESW,
        Cursor::NwResize | Cursor::SeResize | Cursor::NwseResize => IDC_SIZENWSE,
    }
}

/// Shared between [`ServoWinHost::tick`] and the popup [`WNDPROC`] subclass.
pub(super) struct PopupInputState {
    pub(super) ctx: egui::Context,
    pub(super) webview: WebView,
    pub(super) last_move: Cell<Option<DevicePoint>>,
    pub(super) left_down: Cell<bool>,
    /// After a click on the Servo surface, route keyboard from egui here until Escape or a chrome click.
    pub(super) page_captures_keyboard: Cell<bool>,
    pub(super) needs_paint: Rc<Cell<bool>>,
    /// Last cursor from [`WebView::cursor`], applied on `WM_SETCURSOR` (HTCLIENT).
    pub(super) servo_cursor: Cell<Cursor>,
    /// Last right-button-up in popup client device pixels (for context menu anchoring).
    pub(super) last_rbutton_client_px: Cell<Option<(i32, i32)>>,
}

/// Per-window subclass context (stored in `GWLP_USERDATA` until `WM_NCDESTROY`).
struct SubclassData {
    old_wnd_proc: isize,
    input: Rc<PopupInputState>,
}

#[inline]
unsafe fn client_point(lparam: LPARAM) -> DevicePoint {
    let x = (lparam & 0xFFFF) as i16 as i32;
    let y = ((lparam >> 16) & 0xFFFF) as i16 as i32;
    DevicePoint::new(x as f32, y as f32)
}

unsafe fn dispatch_mouse_to_servo(hwnd: HWND, state: &PopupInputState, msg: u32, wparam: WPARAM, lparam: LPARAM) {
    match msg {
        WM_MOUSEMOVE => {
            let dp = client_point(lparam);
            let moved = state
                .last_move
                .get()
                .map(|last| (last.x - dp.x).abs() > 0.25 || (last.y - dp.y).abs() > 0.25)
                .unwrap_or(true);
            if moved {
                state.webview.notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(
                    WebViewPoint::from(dp),
                )));
                state.last_move.set(Some(dp));
            }
        }
        WM_LBUTTONDOWN => {
            let dp = client_point(lparam);
            state.last_move.set(Some(dp));
            state.left_down.set(true);
            state.page_captures_keyboard.set(true);
            // So keys are not still routed to the omnibox while the Win32 surface has "logical" focus.
            state.ctx.memory_mut(|mem| mem.surrender_focus(omnibox_id()));
            state.webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                MouseButtonAction::Down,
                MouseButton::Left,
                WebViewPoint::from(dp),
            )));
        }
        WM_LBUTTONUP => {
            let dp = client_point(lparam);
            state.last_move.set(Some(dp));
            state.left_down.set(false);
            state.webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                MouseButtonAction::Up,
                MouseButton::Left,
                WebViewPoint::from(dp),
            )));
        }
        WM_RBUTTONDOWN => {
            let dp = client_point(lparam);
            state.last_rbutton_client_px.set(None);
            state.webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                MouseButtonAction::Down,
                MouseButton::Right,
                WebViewPoint::from(dp),
            )));
        }
        WM_RBUTTONUP => {
            let dp = client_point(lparam);
            state.last_rbutton_client_px.set(Some((dp.x.round() as i32, dp.y.round() as i32)));
            state.webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                MouseButtonAction::Up,
                MouseButton::Right,
                WebViewPoint::from(dp),
            )));
        }
        WM_MBUTTONDOWN => {
            let dp = client_point(lparam);
            state.webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                MouseButtonAction::Down,
                MouseButton::Middle,
                WebViewPoint::from(dp),
            )));
        }
        WM_MBUTTONUP => {
            let dp = client_point(lparam);
            state.webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                MouseButtonAction::Up,
                MouseButton::Middle,
                WebViewPoint::from(dp),
            )));
        }
        WM_MOUSEWHEEL => {
            let delta = ((wparam >> 16) as u32 & 0xFFFF) as i16 as i32;
            const WHEEL_DELTA: f64 = 120.0;
            let dy = (delta as f64 / WHEEL_DELTA) * 76.0;
            // `lParam` for WM_MOUSEWHEEL is screen coordinates; WebView expects client coords.
            let mut pt = POINT {
                x: (lparam & 0xFFFF) as i16 as i32,
                y: ((lparam >> 16) & 0xFFFF) as i16 as i32,
            };
            let _ = ScreenToClient(hwnd, &mut pt);
            let dp = DevicePoint::new(pt.x as f32, pt.y as f32);
            state.last_move.set(Some(dp));
            state.webview.notify_input_event(InputEvent::Wheel(WheelEvent::new(
                WheelDelta {
                    x: 0.0,
                    y: dy,
                    z: 0.0,
                    mode: WheelMode::DeltaLine,
                },
                WebViewPoint::from(dp),
            )));
        }
        WM_MOUSEHWHEEL => {
            let delta = ((wparam >> 16) as u32 & 0xFFFF) as i16 as i32;
            const WHEEL_DELTA: f64 = 120.0;
            let dx = (delta as f64 / WHEEL_DELTA) * 76.0;
            let mut pt = POINT {
                x: (lparam & 0xFFFF) as i16 as i32,
                y: ((lparam >> 16) & 0xFFFF) as i16 as i32,
            };
            let _ = ScreenToClient(hwnd, &mut pt);
            let dp = DevicePoint::new(pt.x as f32, pt.y as f32);
            state.last_move.set(Some(dp));
            state.webview.notify_input_event(InputEvent::Wheel(WheelEvent::new(
                WheelDelta {
                    x: dx,
                    y: 0.0,
                    z: 0.0,
                    mode: WheelMode::DeltaLine,
                },
                WebViewPoint::from(dp),
            )));
        }
        _ => {}
    }
}

unsafe extern "system" fn servo_popup_subclass_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_NCDESTROY {
        let data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SubclassData;
        if !data_ptr.is_null() {
            let data = Box::from_raw(data_ptr);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            let old = data.old_wnd_proc;
            SetWindowLongPtrW(hwnd, GWLP_WNDPROC, old);
            let prev: WNDPROC = transmute(old);
            let ret = CallWindowProcW(prev, hwnd, msg, wparam, lparam);
            drop(data);
            return ret;
        }
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }

    let data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const SubclassData;
    if data_ptr.is_null() {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    let data = &*data_ptr;
    let old = data.old_wnd_proc;
    let state = &data.input;

    match msg {
        WM_MOUSEACTIVATE => {
            // Keep keyboard focus on the Tonet owner: Servo may host child HWNDs that would
            // otherwise activate and receive WM_CHAR while egui/winit only sees the owner.
            let ht = (lparam as usize) & 0xFFFF;
            if ht == HTCLIENT as usize {
                return MA_NOACTIVATE as LRESULT;
            }
        }
        WM_SETCURSOR => {
            let ht = (lparam as usize) & 0xFFFF;
            if ht == HTCLIENT as usize {
                unsafe {
                    let idc = system_cursor_idc(state.servo_cursor.get());
                    let h = LoadCursorW(std::ptr::null_mut(), idc);
                    if !h.is_null() {
                        SetCursor(h);
                        return 1 as LRESULT;
                    }
                }
            }
        }
        WM_MOUSEMOVE
        | WM_LBUTTONDOWN
        | WM_LBUTTONUP
        | WM_RBUTTONDOWN
        | WM_RBUTTONUP
        | WM_MBUTTONDOWN
        | WM_MBUTTONUP
        | WM_MOUSEWHEEL
        | WM_MOUSEHWHEEL => {
            dispatch_mouse_to_servo(hwnd, state, msg, wparam, lparam);
            state.needs_paint.set(true);
            state.ctx.request_repaint();
            return 0;
        }
        _ => {}
    }

    let prev: WNDPROC = transmute(old);
    CallWindowProcW(prev, hwnd, msg, wparam, lparam)
}

unsafe fn install_popup_subclass(hwnd: HWND, state: Rc<PopupInputState>) -> Result<(), ()> {
    let old = GetWindowLongPtrW(hwnd, GWLP_WNDPROC);
    if old == 0 {
        return Err(());
    }
    let data = Box::new(SubclassData {
        old_wnd_proc: old,
        input: Rc::clone(&state),
    });
    SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(data) as isize);
    SetWindowLongPtrW(
        hwnd,
        GWLP_WNDPROC,
        servo_popup_subclass_wnd_proc as *const () as isize,
    );
    Ok(())
}

fn initial_nav_url(tab_url: &str) -> Url {
    let t = tab_url.trim();
    let tl = t.to_ascii_lowercase();
    if tl.starts_with("http://") || tl.starts_with("https://") {
        Url::parse(t).unwrap_or_else(|_| Url::parse("about:blank").expect("static url"))
    } else if let Some(p) = crate::internal_pages::parse_tonet_url(t) {
        let u = p.normalized_url();
        Url::parse(&u).unwrap_or_else(|_| Url::parse("about:blank").expect("static url"))
    } else {
        // Idle embedder surface while the active Tonet tab is not `http(s)` (new tab, …).
        Url::parse("about:blank").expect("static url")
    }
}

fn pump_messages_for_hwnd(hwnd: HWND) {
    unsafe {
        let mut msg = std::mem::zeroed::<MSG>();
        while PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE) != 0 {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

/// Fallback outer placement when no egui rect yet (points → physical uses same as default size).
fn fallback_outer_from_owner(owner: HWND) -> (i32, i32, i32, i32) {
    let mut wr = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    unsafe {
        if GetWindowRect(owner, &mut wr) == 0 {
            return (40, 60, 920, 640);
        }
    }
    let w = wr.right - wr.left;
    let h = wr.bottom - wr.top;
    let iw = (w - 80).max(480);
    let ih = (h - 120).max(360);
    (wr.left + 40, wr.top + 60, iw, ih)
}

/// Top-left (screen px) and outer width/height (px) for a borderless window matching the egui rect.
fn outer_screen_from_egui_rect(owner: HWND, rect: egui::Rect, ppp: f32) -> Option<(i32, i32, i32, i32)> {
    let ppp = ppp.max(0.01);
    let mut lt = POINT {
        x: (rect.min.x * ppp).round() as i32,
        y: (rect.min.y * ppp).round() as i32,
    };
    unsafe {
        if ClientToScreen(owner, &mut lt) == 0 {
            return None;
        }
    }
    let ow = (rect.width() * ppp).round().max(1.0) as i32;
    let oh = (rect.height() * ppp).round().max(1.0) as i32;
    Some((lt.x, lt.y, ow, oh))
}

/// Map Servo **viewport client** device pixels (same space as [`PopupInputState::last_rbutton_client_px`])
/// to egui points when the page is composited inside [`ServoWinHost::last_content_rect_egui`].
fn slint_egui_anchor_from_viewport_client_px(
    content: egui::Rect,
    client_x: i32,
    client_y: i32,
    ppp: f32,
) -> Option<egui::Pos2> {
    let ppp = ppp.max(0.01);
    Some(
        content.min
            + egui::vec2(client_x as f32 / ppp, client_y as f32 / ppp),
    )
}

/// Map a point in the Servo popup’s **client** device pixels to egui **logical** points in the
/// Tonet owner window’s client area (same space as [`outer_screen_from_egui_rect`]).
fn owner_egui_pos_from_popup_client_px(
    owner: HWND,
    popup: HWND,
    client_x: i32,
    client_y: i32,
    ppp: f32,
) -> Option<egui::Pos2> {
    let ppp = ppp.max(0.01);
    let mut pt = POINT {
        x: client_x,
        y: client_y,
    };
    unsafe {
        if ClientToScreen(popup, &mut pt) == 0 {
            return None;
        }
        if ScreenToClient(owner, &mut pt) == 0 {
            return None;
        }
    }
    Some(egui::pos2(pt.x as f32 / ppp, pt.y as f32 / ppp))
}

#[inline]
fn servo_rgb_to_array(c: Option<RgbColor>) -> [u8; 3] {
    c.map(|c| [c.red, c.green, c.blue]).unwrap_or([0, 0, 0])
}

#[inline]
fn rgb_array_to_servo(rgb: [u8; 3]) -> RgbColor {
    RgbColor {
        red: rgb[0],
        green: rgb[1],
        blue: rgb[2],
    }
}

/// Blocking native file dialog (`rfd`); run from a background thread only.
fn run_native_file_pick(filter_extensions: &[String], allow_multiple: bool) -> Option<Vec<PathBuf>> {
    let mut d = rfd::FileDialog::new();
    for pat in filter_extensions {
        let ext = pat.trim_start_matches('.');
        if ext.is_empty() {
            continue;
        }
        let label = format!("*.{ext}");
        d = d.add_filter(&label, &[ext]);
    }
    if allow_multiple {
        d.pick_files()
    } else {
        d.pick_file().map(|p| vec![p])
    }
}

#[inline]
fn servo_modifiers(m: &egui::Modifiers) -> Modifiers {
    let mut r = Modifiers::empty();
    if m.alt {
        r |= Modifiers::ALT;
    }
    if m.ctrl {
        r |= Modifiers::CONTROL;
    }
    if m.shift {
        r |= Modifiers::SHIFT;
    }
    if m.mac_cmd {
        r |= Modifiers::META;
    }
    r
}

fn egui_key_to_code(key: egui::Key) -> Code {
    use egui::Key as E;
    match key {
        E::Num0 => Code::Digit0,
        E::Num1 => Code::Digit1,
        E::Num2 => Code::Digit2,
        E::Num3 => Code::Digit3,
        E::Num4 => Code::Digit4,
        E::Num5 => Code::Digit5,
        E::Num6 => Code::Digit6,
        E::Num7 => Code::Digit7,
        E::Num8 => Code::Digit8,
        E::Num9 => Code::Digit9,
        E::A => Code::KeyA,
        E::B => Code::KeyB,
        E::C => Code::KeyC,
        E::D => Code::KeyD,
        E::E => Code::KeyE,
        E::F => Code::KeyF,
        E::G => Code::KeyG,
        E::H => Code::KeyH,
        E::I => Code::KeyI,
        E::J => Code::KeyJ,
        E::K => Code::KeyK,
        E::L => Code::KeyL,
        E::M => Code::KeyM,
        E::N => Code::KeyN,
        E::O => Code::KeyO,
        E::P => Code::KeyP,
        E::Q => Code::KeyQ,
        E::R => Code::KeyR,
        E::S => Code::KeyS,
        E::T => Code::KeyT,
        E::U => Code::KeyU,
        E::V => Code::KeyV,
        E::W => Code::KeyW,
        E::X => Code::KeyX,
        E::Y => Code::KeyY,
        E::Z => Code::KeyZ,
        E::Escape => Code::Escape,
        E::Tab => Code::Tab,
        E::Backspace => Code::Backspace,
        E::Enter => Code::Enter,
        E::Space => Code::Space,
        E::Insert => Code::Insert,
        E::Delete => Code::Delete,
        E::Home => Code::Home,
        E::End => Code::End,
        E::PageUp => Code::PageUp,
        E::PageDown => Code::PageDown,
        E::ArrowDown => Code::ArrowDown,
        E::ArrowLeft => Code::ArrowLeft,
        E::ArrowRight => Code::ArrowRight,
        E::ArrowUp => Code::ArrowUp,
        E::Comma => Code::Comma,
        E::Period => Code::Period,
        E::Minus => Code::Minus,
        E::Plus => Code::Equal,
        E::Equals => Code::Equal,
        E::Semicolon => Code::Semicolon,
        E::Quote => Code::Quote,
        E::OpenBracket => Code::BracketLeft,
        E::CloseBracket => Code::BracketRight,
        E::Backtick => Code::Backquote,
        E::Backslash => Code::Backslash,
        E::Slash => Code::Slash,
        E::Colon => Code::Semicolon,
        E::Pipe => Code::IntlBackslash,
        E::Questionmark => Code::Slash,
        E::F1 => Code::F1,
        E::F2 => Code::F2,
        E::F3 => Code::F3,
        E::F4 => Code::F4,
        E::F5 => Code::F5,
        E::F6 => Code::F6,
        E::F7 => Code::F7,
        E::F8 => Code::F8,
        E::F9 => Code::F9,
        E::F10 => Code::F10,
        E::F11 => Code::F11,
        E::F12 => Code::F12,
        E::F13 => Code::F13,
        E::F14 => Code::F14,
        E::F15 => Code::F15,
        E::F16 => Code::F16,
        E::F17 => Code::F17,
        E::F18 => Code::F18,
        E::F19 => Code::F19,
        E::F20 => Code::F20,
        E::F21 => Code::F21,
        E::F22 => Code::F22,
        E::F23 => Code::F23,
        E::F24 => Code::F24,
        E::F25 => Code::F25,
        E::F26 => Code::F26,
        E::F27 => Code::F27,
        E::F28 => Code::F28,
        E::F29 => Code::F29,
        E::F30 => Code::F30,
        E::F31 => Code::F31,
        E::F32 => Code::F32,
        E::F33 => Code::F33,
        E::F34 => Code::F34,
        E::F35 => Code::F35,
        E::Copy => Code::Unidentified,
        E::Cut => Code::Unidentified,
        E::Paste => Code::Unidentified,
    }
}

fn egui_key_to_servo_key(key: egui::Key) -> Option<Key> {
    use egui::Key as E;
    use NamedKey as N;
    Some(match key {
        E::ArrowDown => Key::Named(N::ArrowDown),
        E::ArrowLeft => Key::Named(N::ArrowLeft),
        E::ArrowRight => Key::Named(N::ArrowRight),
        E::ArrowUp => Key::Named(N::ArrowUp),
        E::Escape => Key::Named(N::Escape),
        E::Tab => Key::Named(N::Tab),
        E::Backspace => Key::Named(N::Backspace),
        E::Enter => Key::Named(N::Enter),
        E::Space => Key::Character(" ".into()),
        E::Insert => Key::Named(N::Insert),
        E::Delete => Key::Named(N::Delete),
        E::Home => Key::Named(N::Home),
        E::End => Key::Named(N::End),
        E::PageUp => Key::Named(N::PageUp),
        E::PageDown => Key::Named(N::PageDown),
        E::Copy => Key::Named(N::Copy),
        E::Cut => Key::Named(N::Cut),
        E::Paste => Key::Named(N::Paste),
        E::Colon => Key::Character(":".into()),
        E::Comma => Key::Character(",".into()),
        E::Backslash => Key::Character("\\".into()),
        E::Slash => Key::Character("/".into()),
        E::Pipe => Key::Character("|".into()),
        E::Questionmark => Key::Character("?".into()),
        E::OpenBracket => Key::Character("[".into()),
        E::CloseBracket => Key::Character("]".into()),
        E::Backtick => Key::Character("`".into()),
        E::Minus => Key::Character("-".into()),
        E::Period => Key::Character(".".into()),
        E::Plus => Key::Character("+".into()),
        E::Equals => Key::Character("=".into()),
        E::Semicolon => Key::Character(";".into()),
        E::Quote => Key::Character("'".into()),
        E::Num0 => Key::Character("0".into()),
        E::Num1 => Key::Character("1".into()),
        E::Num2 => Key::Character("2".into()),
        E::Num3 => Key::Character("3".into()),
        E::Num4 => Key::Character("4".into()),
        E::Num5 => Key::Character("5".into()),
        E::Num6 => Key::Character("6".into()),
        E::Num7 => Key::Character("7".into()),
        E::Num8 => Key::Character("8".into()),
        E::Num9 => Key::Character("9".into()),
        E::A => Key::Character("a".into()),
        E::B => Key::Character("b".into()),
        E::C => Key::Character("c".into()),
        E::D => Key::Character("d".into()),
        E::E => Key::Character("e".into()),
        E::F => Key::Character("f".into()),
        E::G => Key::Character("g".into()),
        E::H => Key::Character("h".into()),
        E::I => Key::Character("i".into()),
        E::J => Key::Character("j".into()),
        E::K => Key::Character("k".into()),
        E::L => Key::Character("l".into()),
        E::M => Key::Character("m".into()),
        E::N => Key::Character("n".into()),
        E::O => Key::Character("o".into()),
        E::P => Key::Character("p".into()),
        E::Q => Key::Character("q".into()),
        E::R => Key::Character("r".into()),
        E::S => Key::Character("s".into()),
        E::T => Key::Character("t".into()),
        E::U => Key::Character("u".into()),
        E::V => Key::Character("v".into()),
        E::W => Key::Character("w".into()),
        E::X => Key::Character("x".into()),
        E::Y => Key::Character("y".into()),
        E::Z => Key::Character("z".into()),
        E::F1 => Key::Named(N::F1),
        E::F2 => Key::Named(N::F2),
        E::F3 => Key::Named(N::F3),
        E::F4 => Key::Named(N::F4),
        E::F5 => Key::Named(N::F5),
        E::F6 => Key::Named(N::F6),
        E::F7 => Key::Named(N::F7),
        E::F8 => Key::Named(N::F8),
        E::F9 => Key::Named(N::F9),
        E::F10 => Key::Named(N::F10),
        E::F11 => Key::Named(N::F11),
        E::F12 => Key::Named(N::F12),
        E::F13 => Key::Named(N::F13),
        E::F14 => Key::Named(N::F14),
        E::F15 => Key::Named(N::F15),
        E::F16 => Key::Named(N::F16),
        E::F17 => Key::Named(N::F17),
        E::F18 => Key::Named(N::F18),
        E::F19 => Key::Named(N::F19),
        E::F20 => Key::Named(N::F20),
        E::F21 => Key::Named(N::F21),
        E::F22 => Key::Named(N::F22),
        E::F23 => Key::Named(N::F23),
        E::F24 => Key::Named(N::F24),
        E::F25 => Key::Named(N::F25),
        E::F26 => Key::Named(N::F26),
        E::F27 => Key::Named(N::F27),
        E::F28 => Key::Named(N::F28),
        E::F29 => Key::Named(N::F29),
        E::F30 => Key::Named(N::F30),
        E::F31 => Key::Named(N::F31),
        E::F32 => Key::Named(N::F32),
        E::F33 => Key::Named(N::F33),
        E::F34 => Key::Named(N::F34),
        E::F35 => Key::Named(N::F35),
    })
}

#[inline]
fn egui_key_should_forward_as_key_event(key: egui::Key, m: &egui::Modifiers) -> bool {
    if m.ctrl || m.command || m.alt {
        return true;
    }
    !matches!(
        key,
        egui::Key::A
            | egui::Key::B
            | egui::Key::C
            | egui::Key::D
            | egui::Key::E
            | egui::Key::F
            | egui::Key::G
            | egui::Key::H
            | egui::Key::I
            | egui::Key::J
            | egui::Key::K
            | egui::Key::L
            | egui::Key::M
            | egui::Key::N
            | egui::Key::O
            | egui::Key::P
            | egui::Key::Q
            | egui::Key::R
            | egui::Key::S
            | egui::Key::T
            | egui::Key::U
            | egui::Key::V
            | egui::Key::W
            | egui::Key::X
            | egui::Key::Y
            | egui::Key::Z
            | egui::Key::Num0
            | egui::Key::Num1
            | egui::Key::Num2
            | egui::Key::Num3
            | egui::Key::Num4
            | egui::Key::Num5
            | egui::Key::Num6
            | egui::Key::Num7
            | egui::Key::Num8
            | egui::Key::Num9
    )
}

pub struct ServoWinHost {
    owner: HWND,
    /// `true` when using the old borderless HWND popup; `false` = Slint-style surfman GPU (readback to egui).
    embed_win32_popup: bool,
    popup: Option<Win32PopupWindow>,
    rendering_context: Option<Rc<WindowRenderingContext>>,
    slint_gpu: Option<Rc<GPURenderingContext>>,
    /// Latest Servo framebuffer for egui (`Slint`-style compositing without D3D11↔wgpu interop).
    slint_egui_frame: RefCell<Option<egui::ColorImage>>,
    servo: Servo,
    last_tab_url: String,
    last_client: Option<(u32, u32)>,
    last_ppp: f32,
    /// Tracks `ShowWindow(SW_HIDE)` so we repaint once when the overlay is shown again.
    popup_visible: bool,
    input: Rc<PopupInputState>,
    shell_snapshot: RefCell<ServoShellSnapshot>,
    /// Last frame’s `LoadStatus::Complete` (for detecting reload / new navigation).
    prev_shell_complete: Cell<bool>,
    /// URL last passed to [`crate::browser_log::BrowserLog::record_visit`] for this WebView.
    last_recorded_visit_url: RefCell<Option<String>>,
    /// Fingerprint of last PNG applied to egui (`0` = none yet).
    last_favicon_png_hash: Cell<u64>,
    dialog_pending: Rc<RefCell<Option<SimpleDialog>>>,
    prompt_draft: Rc<RefCell<String>>,
    dialog_window_open: Rc<Cell<bool>>,
    context_menu_pending: Rc<RefCell<Option<ContextMenu>>>,
    context_menu_window_open: Rc<Cell<bool>>,
    select_pending: Rc<RefCell<Option<SelectElement>>>,
    select_draft: Rc<Cell<Option<usize>>>,
    select_window_open: Rc<Cell<bool>>,
    color_picker_pending: Rc<RefCell<Option<ColorPicker>>>,
    color_picker_draft: Rc<RefCell<[u8; 3]>>,
    color_picker_window_open: Rc<Cell<bool>>,
    file_picker_waiting: Rc<RefCell<Option<FilePicker>>>,
    file_picker_rx: Rc<RefCell<Option<mpsc::Receiver<Option<Vec<PathBuf>>>>>>,
    permission_pending: Rc<RefCell<Option<PermissionRequest>>>,
    permission_window_open: Rc<Cell<bool>>,
    permission_cache: Rc<RefCell<HashMap<String, bool>>>,
    permission_prompt_origin: Rc<RefCell<Option<String>>>,
    auth_pending: Rc<RefCell<Option<AuthenticationRequest>>>,
    auth_user_draft: Rc<RefCell<String>>,
    auth_pass_draft: Rc<RefCell<String>>,
    auth_window_open: Rc<Cell<bool>>,
    /// `http(s)` link chosen from the Servo context menu: open a new Tonet tab (shell), not Servo’s new-WebView action.
    pending_open_link_new_tonet_tab: RefCell<Option<String>>,
    notification_toast: Rc<RefCell<Option<ServoWebNotificationToast>>>,
    console_pending: Rc<RefCell<VecDeque<(ConsoleLogLevel, String)>>>,
    background_download_done: Arc<Mutex<Vec<background_download::CompletedBackgroundDownload>>>,
    /// When the active tab is not `http(s)` but experimental viewport stays on, spin Servo less often.
    idle_spin_counter: u8,
    /// Last laid-out Servo content rect in egui space (Slint embed: map context menu, etc.).
    last_content_rect_egui: Cell<Option<egui::Rect>>,
}

impl ServoWinHost {
    pub fn try_new(
        ctx: &egui::Context,
        frame: &eframe::Frame,
        tab_url: &str,
        content_rect: Option<egui::Rect>,
        ppp: f32,
        tonet_scheme_state: Arc<Mutex<tonet_scheme_html::TonetSchemeSharedState>>,
    ) -> Result<Self, ()> {
        let owner_isize = win32_hwnd_from_frame(frame).ok_or(())?;
        let owner: HWND = owner_isize as *mut core::ffi::c_void;

        if !embed_uses_win32_popup() {
            return Self::try_new_slint_gpu(ctx, tab_url, content_rect, ppp, owner, tonet_scheme_state);
        }

        let hinstance = unsafe { GetModuleHandleW(std::ptr::null()) };
        if hinstance.is_null() {
            return Err(());
        }

        register_popup_class(hinstance)?;

        let (x, y, w, h) = content_rect
            .and_then(|r| outer_screen_from_egui_rect(owner, r, ppp))
            .unwrap_or_else(|| fallback_outer_from_owner(owner));

        let style: WINDOW_STYLE = WS_POPUP | WS_VISIBLE;
        // Keep keyboard focus on the main Tonet window so egui (omnibox, tabs) keeps receiving keys.
        let ex: WINDOW_EX_STYLE = WS_EX_NOACTIVATE;
        let title = windows_sys::core::w!("");
        let class_name = windows_sys::core::w!("TonetServoExperimentalPopup");

        let popup_hwnd = unsafe {
            CreateWindowExW(
                ex,
                class_name,
                title,
                style,
                x,
                y,
                w,
                h,
                owner,
                std::ptr::null_mut(),
                hinstance,
                std::ptr::null(),
            )
        };
        if popup_hwnd.is_null() {
            return Err(());
        }

        unsafe {
            let _ = ShowWindow(popup_hwnd, SW_SHOW);
            let _ = UpdateWindow(popup_hwnd);
        }

        let mut cr = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        unsafe {
            if GetClientRect(popup_hwnd, &mut cr) == 0 {
                DestroyWindow(popup_hwnd);
                return Err(());
            }
        }
        let cw = (cr.right - cr.left).max(1) as u32;
        let ch = (cr.bottom - cr.top).max(1) as u32;
        let phys = PhysicalSize::new(cw, ch);

        let display_handle = frame.display_handle().map_err(|_| ())?;
        let win32 = Win32WindowHandle::new(
            std::num::NonZeroIsize::new(popup_hwnd as isize).ok_or(())?,
        );
        let raw = RawWindowHandle::Win32(win32);
        let window_handle = unsafe { WindowHandle::borrow_raw(raw) };

        let rendering_context = match WindowRenderingContext::new(display_handle, window_handle, phys) {
            Ok(rc) => Rc::new(rc),
            Err(_) => {
                unsafe {
                    DestroyWindow(popup_hwnd);
                }
                return Err(());
            }
        };

        let _ = rendering_context.make_current();

        let servo = ServoBuilder::default()
            .event_loop_waker(Box::new(EguiRepaintWaker(ctx.clone())))
            .build();
        servo.setup_logging();

        let needs_paint = Rc::new(Cell::new(true));
        let dialog_pending = Rc::new(RefCell::new(None));
        let prompt_draft = Rc::new(RefCell::new(String::new()));
        let dialog_window_open = Rc::new(Cell::new(true));
        let context_menu_pending = Rc::new(RefCell::new(None));
        let context_menu_window_open = Rc::new(Cell::new(true));
        let select_pending = Rc::new(RefCell::new(None));
        let select_draft = Rc::new(Cell::new(None));
        let select_window_open = Rc::new(Cell::new(true));
        let color_picker_pending = Rc::new(RefCell::new(None));
        let color_picker_draft = Rc::new(RefCell::new([0u8; 3]));
        let color_picker_window_open = Rc::new(Cell::new(true));
        let file_picker_waiting = Rc::new(RefCell::new(None));
        let file_picker_rx = Rc::new(RefCell::new(None));
        let permission_pending = Rc::new(RefCell::new(None));
        let permission_window_open = Rc::new(Cell::new(true));
        let permission_cache = Rc::new(RefCell::new(super::permission_store::load()));
        let permission_prompt_origin = Rc::new(RefCell::new(None));
        let auth_pending = Rc::new(RefCell::new(None));
        let auth_user_draft = Rc::new(RefCell::new(String::new()));
        let auth_pass_draft = Rc::new(RefCell::new(String::new()));
        let auth_window_open = Rc::new(Cell::new(true));
        let notification_toast = Rc::new(RefCell::new(None));
        let console_pending = Rc::new(RefCell::new(VecDeque::new()));
        let background_download_done = Arc::new(Mutex::new(Vec::new()));
        let start_url = initial_nav_url(tab_url);
        let tonet_scheme_state_delegate = Arc::clone(&tonet_scheme_state);
        let delegate = Rc::new(TonetServoWebViewDelegate {
            egui_ctx: ctx.clone(),
            needs_paint: needs_paint.clone(),
            dialog_pending: dialog_pending.clone(),
            prompt_draft: prompt_draft.clone(),
            dialog_window_open: dialog_window_open.clone(),
            context_menu_pending: context_menu_pending.clone(),
            context_menu_window_open: context_menu_window_open.clone(),
            select_pending: select_pending.clone(),
            select_draft: select_draft.clone(),
            select_window_open: select_window_open.clone(),
            color_picker_pending: color_picker_pending.clone(),
            color_picker_draft: color_picker_draft.clone(),
            color_picker_window_open: color_picker_window_open.clone(),
            file_picker_waiting: file_picker_waiting.clone(),
            file_picker_rx: file_picker_rx.clone(),
            permission_pending: permission_pending.clone(),
            permission_window_open: permission_window_open.clone(),
            permission_cache: permission_cache.clone(),
            permission_prompt_origin: permission_prompt_origin.clone(),
            auth_pending: auth_pending.clone(),
            auth_user_draft: auth_user_draft.clone(),
            auth_pass_draft: auth_pass_draft.clone(),
            auth_window_open: auth_window_open.clone(),
            notification_toast: notification_toast.clone(),
            console_pending: console_pending.clone(),
            background_download_done: background_download_done.clone(),
            tonet_scheme_state: tonet_scheme_state_delegate,
        });

        let webview = WebViewBuilder::new(&servo, rendering_context.clone())
            .url(start_url.clone())
            .hidpi_scale_factor(Scale::new(ppp))
            .delegate(delegate)
            .build();

        let shell_snapshot = RefCell::new(ServoShellSnapshot::capture_from(&webview));

        let input = Rc::new(PopupInputState {
            ctx: ctx.clone(),
            webview,
            last_move: Cell::new(None),
            left_down: Cell::new(false),
            page_captures_keyboard: Cell::new(false),
            needs_paint: needs_paint.clone(),
            servo_cursor: Cell::new(Cursor::Default),
            last_rbutton_client_px: Cell::new(None),
        });

        unsafe {
            install_popup_subclass(popup_hwnd, Rc::clone(&input))?;
        }

        Ok(Self {
            owner,
            embed_win32_popup: true,
            popup: Some(Win32PopupWindow(popup_hwnd)),
            rendering_context: Some(rendering_context),
            slint_gpu: None,
            slint_egui_frame: RefCell::new(None),
            servo,
            last_tab_url: start_url.as_str().to_owned(),
            last_client: Some((cw, ch)),
            last_ppp: ppp,
            popup_visible: true,
            input,
            shell_snapshot,
            prev_shell_complete: Cell::new(false),
            last_recorded_visit_url: RefCell::new(None),
            last_favicon_png_hash: Cell::new(0),
            dialog_pending,
            prompt_draft,
            dialog_window_open,
            context_menu_pending,
            context_menu_window_open,
            select_pending,
            select_draft,
            select_window_open,
            color_picker_pending,
            color_picker_draft,
            color_picker_window_open,
            file_picker_waiting,
            file_picker_rx,
            permission_pending,
            permission_window_open,
            permission_cache,
            permission_prompt_origin,
            auth_pending,
            auth_user_draft,
            auth_pass_draft,
            auth_window_open,
            pending_open_link_new_tonet_tab: RefCell::new(None),
            notification_toast,
            console_pending,
            background_download_done,
            idle_spin_counter: 0,
            last_content_rect_egui: Cell::new(None),
        })
    }

    /// Slint `examples/servo`-style surfman GPU swapchain (no separate Win32 `WebView` HWND).
    fn try_new_slint_gpu(
        ctx: &egui::Context,
        tab_url: &str,
        content_rect: Option<egui::Rect>,
        ppp: f32,
        owner: HWND,
        tonet_scheme_state: Arc<Mutex<tonet_scheme_html::TonetSchemeSharedState>>,
    ) -> Result<Self, ()> {
        let ppp = ppp.max(0.01);
        let (cw, ch) = content_rect
            .map(|r| {
                let w = (r.width() * ppp).round().max(1.0) as u32;
                let h = (r.height() * ppp).round().max(1.0) as u32;
                (w.max(1), h.max(1))
            })
            .unwrap_or((920, 640));
        let phys = PhysicalSize::new(cw, ch);
        let gpu = Rc::new(GPURenderingContext::new(phys).map_err(|_| ())?);
        let _ = gpu.make_current();

        let surf: Rc<dyn RenderingContext> = gpu.clone();

        let servo = ServoBuilder::default()
            .event_loop_waker(Box::new(EguiRepaintWaker(ctx.clone())))
            .build();
        servo.setup_logging();

        let needs_paint = Rc::new(Cell::new(true));
        let dialog_pending = Rc::new(RefCell::new(None));
        let prompt_draft = Rc::new(RefCell::new(String::new()));
        let dialog_window_open = Rc::new(Cell::new(true));
        let context_menu_pending = Rc::new(RefCell::new(None));
        let context_menu_window_open = Rc::new(Cell::new(true));
        let select_pending = Rc::new(RefCell::new(None));
        let select_draft = Rc::new(Cell::new(None));
        let select_window_open = Rc::new(Cell::new(true));
        let color_picker_pending = Rc::new(RefCell::new(None));
        let color_picker_draft = Rc::new(RefCell::new([0u8; 3]));
        let color_picker_window_open = Rc::new(Cell::new(true));
        let file_picker_waiting = Rc::new(RefCell::new(None));
        let file_picker_rx = Rc::new(RefCell::new(None));
        let permission_pending = Rc::new(RefCell::new(None));
        let permission_window_open = Rc::new(Cell::new(true));
        let permission_cache = Rc::new(RefCell::new(super::permission_store::load()));
        let permission_prompt_origin = Rc::new(RefCell::new(None));
        let auth_pending = Rc::new(RefCell::new(None));
        let auth_user_draft = Rc::new(RefCell::new(String::new()));
        let auth_pass_draft = Rc::new(RefCell::new(String::new()));
        let auth_window_open = Rc::new(Cell::new(true));
        let notification_toast = Rc::new(RefCell::new(None));
        let console_pending = Rc::new(RefCell::new(VecDeque::new()));
        let background_download_done = Arc::new(Mutex::new(Vec::new()));
        let start_url = initial_nav_url(tab_url);
        let tonet_scheme_state_delegate = Arc::clone(&tonet_scheme_state);
        let delegate = Rc::new(TonetServoWebViewDelegate {
            egui_ctx: ctx.clone(),
            needs_paint: needs_paint.clone(),
            dialog_pending: dialog_pending.clone(),
            prompt_draft: prompt_draft.clone(),
            dialog_window_open: dialog_window_open.clone(),
            context_menu_pending: context_menu_pending.clone(),
            context_menu_window_open: context_menu_window_open.clone(),
            select_pending: select_pending.clone(),
            select_draft: select_draft.clone(),
            select_window_open: select_window_open.clone(),
            color_picker_pending: color_picker_pending.clone(),
            color_picker_draft: color_picker_draft.clone(),
            color_picker_window_open: color_picker_window_open.clone(),
            file_picker_waiting: file_picker_waiting.clone(),
            file_picker_rx: file_picker_rx.clone(),
            permission_pending: permission_pending.clone(),
            permission_window_open: permission_window_open.clone(),
            permission_cache: permission_cache.clone(),
            permission_prompt_origin: permission_prompt_origin.clone(),
            auth_pending: auth_pending.clone(),
            auth_user_draft: auth_user_draft.clone(),
            auth_pass_draft: auth_pass_draft.clone(),
            auth_window_open: auth_window_open.clone(),
            notification_toast: notification_toast.clone(),
            console_pending: console_pending.clone(),
            background_download_done: background_download_done.clone(),
            tonet_scheme_state: tonet_scheme_state_delegate,
        });

        let webview = WebViewBuilder::new(&servo, surf)
            .url(start_url.clone())
            .hidpi_scale_factor(Scale::new(ppp))
            .delegate(delegate)
            .build();

        let shell_snapshot = RefCell::new(ServoShellSnapshot::capture_from(&webview));

        let input = Rc::new(PopupInputState {
            ctx: ctx.clone(),
            webview,
            last_move: Cell::new(None),
            left_down: Cell::new(false),
            page_captures_keyboard: Cell::new(false),
            needs_paint: needs_paint.clone(),
            servo_cursor: Cell::new(Cursor::Default),
            last_rbutton_client_px: Cell::new(None),
        });

        Ok(Self {
            owner,
            embed_win32_popup: false,
            popup: None,
            rendering_context: None,
            slint_gpu: Some(gpu),
            slint_egui_frame: RefCell::new(None),
            servo,
            last_tab_url: start_url.as_str().to_owned(),
            last_client: Some((cw, ch)),
            last_ppp: ppp,
            popup_visible: false,
            input,
            shell_snapshot,
            prev_shell_complete: Cell::new(false),
            last_recorded_visit_url: RefCell::new(None),
            last_favicon_png_hash: Cell::new(0),
            dialog_pending,
            prompt_draft,
            dialog_window_open,
            context_menu_pending,
            context_menu_window_open,
            select_pending,
            select_draft,
            select_window_open,
            color_picker_pending,
            color_picker_draft,
            color_picker_window_open,
            file_picker_waiting,
            file_picker_rx,
            permission_pending,
            permission_window_open,
            permission_cache,
            permission_prompt_origin,
            auth_pending,
            auth_user_draft,
            auth_pass_draft,
            auth_window_open,
            pending_open_link_new_tonet_tab: RefCell::new(None),
            notification_toast,
            console_pending,
            background_download_done,
            idle_spin_counter: 0,
            last_content_rect_egui: Cell::new(None),
        })
    }

    pub(crate) fn take_pending_open_link_new_tonet_tab(&self) -> Option<String> {
        self.pending_open_link_new_tonet_tab.borrow_mut().take()
    }

    /// [`WebViewDelegate::show_notification`]: egui toast under the top chrome (no OS notification center).
    pub(crate) fn show_web_notification_toast(&self, ctx: &egui::Context, loc: Locale) {
        {
            let mut slot = self.notification_toast.borrow_mut();
            match *slot {
                None => return,
                Some(ref t) if Instant::now() > t.until => {
                    *slot = None;
                    return;
                }
                Some(_) => {}
            }
        }
        let snapshot = match self.notification_toast.borrow().clone() {
            Some(s) => s,
            None => return,
        };
        if Instant::now() > snapshot.until {
            *self.notification_toast.borrow_mut() = None;
            return;
        }
        let toast_slot = self.notification_toast.clone();
        let title_owned = match snapshot.title.as_deref() {
            Some(t) if !t.is_empty() => t.to_string(),
            _ => crate::i18n::servo_notification_fallback_title(loc).to_string(),
        };
        let body = snapshot.body.clone();
        egui::Area::new(egui::Id::new("tonet_servo_web_notification"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 52.0))
            .show(ctx, |ui| {
                egui::Frame::default()
                    .fill(crate::theme::update_banner_bg())
                    .stroke(egui::Stroke::new(1.0, crate::theme::update_banner_stroke()))
                    .rounding(10.0)
                    .inner_margin(14.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(&title_owned).strong());
                                if !body.is_empty() {
                                    ui.label(egui::RichText::new(&body).small());
                                }
                            });
                            if ui
                                .small_button(crate::i18n::servo_notification_dismiss(loc))
                                .clicked()
                            {
                                *toast_slot.borrow_mut() = None;
                            }
                        });
                    });
            });
        ctx.request_repaint();
    }

    fn persist_servo_permissions(&self) {
        super::permission_store::save(&self.permission_cache.borrow());
    }

    pub(crate) fn clear_servo_permission_memory(&mut self) {
        self.permission_cache.borrow_mut().clear();
    }

    /// Clears in-memory Servo embedder tails (page console queue, web-notification toast, completed
    /// download rows waiting for `BrowserLog`) without tearing down the `WebView`. Used when the
    /// user clears **Downloads** from internal pages so UI state matches the empty log.
    pub(crate) fn clear_ephemeral_embedder_queues(&self) {
        self.console_pending.borrow_mut().clear();
        *self.notification_toast.borrow_mut() = None;
        if let Ok(mut q) = self.background_download_done.lock() {
            q.clear();
        }
    }

    /// Modal `alert` / `confirm` / `prompt` from Servo. Call once per frame while the experimental
    /// viewport is active. Returns whether [`Self::spin_event_loop`] should run afterward.
    pub(crate) fn show_simple_dialog_if_pending(&mut self, ctx: &egui::Context, loc: Locale) -> bool {
        if self.auth_pending.borrow().is_some() {
            return false;
        }
        if self.permission_pending.borrow().is_some() {
            return false;
        }
        if self.file_picker_waiting.borrow().is_some() {
            return false;
        }
        if self.dialog_pending.borrow().is_none() {
            return false;
        }

        let (title, needs_cancel, needs_prompt) = {
            let slot = self.dialog_pending.borrow();
            let Some(ref d) = *slot else {
                return false;
            };
            let title = match d {
                SimpleDialog::Alert(_) => "JavaScript alert",
                SimpleDialog::Confirm(_) => "JavaScript confirm",
                SimpleDialog::Prompt(_) => "JavaScript prompt",
            };
            (
                title,
                !matches!(d, SimpleDialog::Alert(_)),
                matches!(d, SimpleDialog::Prompt(_)),
            )
        };

        let mut open = self.dialog_window_open.get();
        let ok_clicked = Cell::new(false);
        let cancel_clicked = Cell::new(false);

        let _ = egui::Window::new(title)
            .collapsible(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut open)
            .show(ctx, |ui| {
                let message = self
                    .dialog_pending
                    .borrow()
                    .as_ref()
                    .map(|d| d.message().to_owned())
                    .unwrap_or_default();
                ui.label(message);
                if needs_prompt {
                    egui::TextEdit::singleline(&mut *self.prompt_draft.borrow_mut())
                        .desired_width(ui.available_width().min(480.0))
                        .show(ui);
                }
                ui.horizontal(|ui| {
                    if needs_cancel
                        && ui
                            .button(crate::i18n::new_tab_add_cancel(loc))
                            .clicked()
                    {
                        cancel_clicked.set(true);
                    }
                    if ui.button(crate::i18n::servo_dialog_ok(loc)).clicked() {
                        ok_clicked.set(true);
                    }
                });
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    cancel_clicked.set(true);
                }
            });

        self.dialog_window_open.set(open);

        let mut spin = false;

        if !open && self.dialog_pending.borrow().is_some() {
            if let Some(d) = self.dialog_pending.borrow_mut().take() {
                d.dismiss();
                spin = true;
            }
            self.dialog_window_open.set(true);
            self.input.needs_paint.set(true);
            ctx.request_repaint();
            return spin;
        }

        if ok_clicked.get() {
            if let Some(d) = self.dialog_pending.borrow_mut().take() {
                match d {
                    SimpleDialog::Prompt(mut p) => {
                        p.set_current_value(&self.prompt_draft.borrow());
                        p.confirm();
                    }
                    d => d.confirm(),
                }
                spin = true;
            }
            self.dialog_window_open.set(true);
            self.input.needs_paint.set(true);
            ctx.request_repaint();
            return spin;
        }

        if cancel_clicked.get() {
            if let Some(d) = self.dialog_pending.borrow_mut().take() {
                d.dismiss();
                spin = true;
            }
            self.dialog_window_open.set(true);
            self.input.needs_paint.set(true);
            ctx.request_repaint();
            return spin;
        }

        false
    }

    /// HTTP(S) **401** / **407** style credentials from Servo [`AuthenticationRequest`]. Deferred while
    /// a script dialog or native file pick is active.
    pub(crate) fn show_http_auth_if_pending(&mut self, ctx: &egui::Context, loc: Locale) -> bool {
        if self.dialog_pending.borrow().is_some() || self.file_picker_waiting.borrow().is_some() {
            return false;
        }
        if self.auth_pending.borrow().is_none() {
            return false;
        }

        let for_proxy = {
            let slot = self.auth_pending.borrow();
            let Some(ref a) = *slot else {
                return false;
            };
            a.for_proxy()
        };
        let url_display = {
            let slot = self.auth_pending.borrow();
            let Some(ref a) = *slot else {
                return false;
            };
            a.url().as_str().to_owned()
        };

        let mut open = self.auth_window_open.get();
        let ok_clicked = Cell::new(false);
        let cancel_clicked = Cell::new(false);

        let _ = egui::Window::new(crate::i18n::servo_http_auth_title(loc))
            .id(egui::Id::new("tonet_servo_http_auth"))
            .collapsible(false)
            .resizable(true)
            .constrain(true)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(crate::i18n::servo_http_auth_intro(loc));
                ui.monospace(&url_display);
                if for_proxy {
                    ui.small(crate::i18n::servo_http_auth_proxy_note(loc));
                }
                ui.add_space(6.0);
                ui.label(crate::i18n::servo_http_auth_user_label(loc));
                egui::TextEdit::singleline(&mut *self.auth_user_draft.borrow_mut())
                    .desired_width(ui.available_width().min(440.0))
                    .show(ui);
                ui.label(crate::i18n::servo_http_auth_password_label(loc));
                egui::TextEdit::singleline(&mut *self.auth_pass_draft.borrow_mut())
                    .password(true)
                    .desired_width(ui.available_width().min(440.0))
                    .show(ui);
                ui.horizontal(|ui| {
                    if ui.button(crate::i18n::new_tab_add_cancel(loc)).clicked() {
                        cancel_clicked.set(true);
                    }
                    if ui.button(crate::i18n::servo_http_auth_sign_in(loc)).clicked() {
                        ok_clicked.set(true);
                    }
                });
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    cancel_clicked.set(true);
                }
            });

        self.auth_window_open.set(open);

        let mut spin = false;

        if !open && self.auth_pending.borrow().is_some() {
            if let Some(a) = self.auth_pending.borrow_mut().take() {
                drop(a);
                self.auth_user_draft.borrow_mut().clear();
                self.auth_pass_draft.borrow_mut().clear();
                spin = true;
            }
            self.auth_window_open.set(true);
            self.input.needs_paint.set(true);
            ctx.request_repaint();
            return spin;
        }

        if cancel_clicked.get() {
            if let Some(a) = self.auth_pending.borrow_mut().take() {
                drop(a);
                self.auth_user_draft.borrow_mut().clear();
                self.auth_pass_draft.borrow_mut().clear();
                spin = true;
            }
            self.auth_window_open.set(true);
            self.input.needs_paint.set(true);
            ctx.request_repaint();
            return spin;
        }

        if ok_clicked.get() {
            if let Some(a) = self.auth_pending.borrow_mut().take() {
                let u = self.auth_user_draft.borrow().trim().to_owned();
                let p = self.auth_pass_draft.borrow().clone();
                self.auth_user_draft.borrow_mut().clear();
                self.auth_pass_draft.borrow_mut().clear();
                a.authenticate(u, p);
                spin = true;
            }
            self.auth_window_open.set(true);
            self.input.needs_paint.set(true);
            ctx.request_repaint();
            return spin;
        }

        false
    }

    /// Servo [`PermissionRequest`] (camera, geolocation, etc.). Deferred while a script dialog or
    /// native file pick is active.
    pub(crate) fn show_permission_request_if_pending(&mut self, ctx: &egui::Context, loc: Locale) -> bool {
        if self.dialog_pending.borrow().is_some()
            || self.auth_pending.borrow().is_some()
            || self.file_picker_waiting.borrow().is_some()
        {
            return false;
        }
        let feat = {
            let slot = self.permission_pending.borrow();
            let Some(ref p) = *slot else {
                return false;
            };
            p.feature()
        };

        let feature_label = crate::i18n::servo_permission_feature_name(loc, feat);
        let mut open = self.permission_window_open.get();
        let allow_clicked = Cell::new(false);
        let deny_clicked = Cell::new(false);

        let _ = egui::Window::new(crate::i18n::servo_permission_title(loc))
            .id(egui::Id::new("tonet_servo_permission"))
            .collapsible(false)
            .resizable(false)
            .constrain(true)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(crate::i18n::servo_permission_intro(loc));
                ui.strong(feature_label);
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(crate::i18n::servo_permission_deny(loc)).clicked() {
                        deny_clicked.set(true);
                    }
                    if ui.button(crate::i18n::servo_permission_allow(loc)).clicked() {
                        allow_clicked.set(true);
                    }
                });
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    deny_clicked.set(true);
                }
            });

        self.permission_window_open.set(open);

        let mut spin = false;

        if !open && self.permission_pending.borrow().is_some() {
            if let Some(p) = self.permission_pending.borrow_mut().take() {
                let feat = p.feature();
                p.deny();
                if let Some(origin) = self.permission_prompt_origin.borrow_mut().take() {
                    let key = servo_permission_cache_key(&origin, feat);
                    self.permission_cache.borrow_mut().insert(key, false);
                }
                self.persist_servo_permissions();
                spin = true;
            }
            self.permission_window_open.set(true);
            self.input.needs_paint.set(true);
            ctx.request_repaint();
            return spin;
        }

        if deny_clicked.get() {
            if let Some(p) = self.permission_pending.borrow_mut().take() {
                let feat = p.feature();
                p.deny();
                if let Some(origin) = self.permission_prompt_origin.borrow_mut().take() {
                    let key = servo_permission_cache_key(&origin, feat);
                    self.permission_cache.borrow_mut().insert(key, false);
                }
                self.persist_servo_permissions();
                spin = true;
            }
            self.permission_window_open.set(true);
            self.input.needs_paint.set(true);
            ctx.request_repaint();
            return spin;
        }

        if allow_clicked.get() {
            if let Some(p) = self.permission_pending.borrow_mut().take() {
                let feat = p.feature();
                p.allow();
                if let Some(origin) = self.permission_prompt_origin.borrow_mut().take() {
                    let key = servo_permission_cache_key(&origin, feat);
                    self.permission_cache.borrow_mut().insert(key, true);
                }
                self.persist_servo_permissions();
                spin = true;
            }
            self.permission_window_open.set(true);
            self.input.needs_paint.set(true);
            ctx.request_repaint();
            return spin;
        }

        false
    }

    /// Servo [`ContextMenu`] (right-click). Deferred until [`Self::show_simple_dialog_if_pending`]
    /// has nothing open. Returns whether [`Self::spin_event_loop`] should run afterward.
    pub(crate) fn show_context_menu_if_pending(&mut self, ctx: &egui::Context, loc: Locale) -> bool {
        if self.dialog_pending.borrow().is_some()
            || self.auth_pending.borrow().is_some()
            || self.permission_pending.borrow().is_some()
            || self.color_picker_pending.borrow().is_some()
            || self.file_picker_waiting.borrow().is_some()
            || self.select_pending.borrow().is_some()
        {
            return false;
        }
        if self.context_menu_pending.borrow().is_none() {
            return false;
        }

        #[derive(Clone)]
        enum MenuRow {
            Sep,
            Item {
                label: String,
                action: ContextMenuAction,
                enabled: bool,
            },
            OpenLinkInNewTonetTab {
                url: String,
            },
        }

        let rows: Vec<MenuRow> = {
            let slot = self.context_menu_pending.borrow();
            let Some(cm) = slot.as_ref() else {
                return false;
            };
            let mut rows: Vec<MenuRow> = cm
                .items()
                .iter()
                .map(|it| match it {
                    ContextMenuItem::Separator => MenuRow::Sep,
                    ContextMenuItem::Item {
                        label,
                        action,
                        enabled,
                    } => MenuRow::Item {
                        label: label.clone(),
                        action: *action,
                        enabled: *enabled,
                    },
                })
                .collect();
            let tonet_url = cm.element_info().link_url.as_ref().and_then(|u| {
                let s = u.as_str();
                if s.starts_with("http://") || s.starts_with("https://") {
                    Some(s.to_string())
                } else {
                    None
                }
            });
            if let Some(url) = tonet_url {
                if !rows.is_empty() {
                    rows.push(MenuRow::Sep);
                }
                rows.push(MenuRow::OpenLinkInNewTonetTab { url });
            }
            rows
        };

        let ppp = ctx.pixels_per_point();
        let anchor_egui = {
            let slot = self.context_menu_pending.borrow();
            let Some(cm) = slot.as_ref() else {
                return false;
            };
            let r = cm.position();
            let w = r.width();
            let h = r.height();
            let has_rect = w > 0 && h > 0;
            let px = self.input.last_rbutton_client_px.get();

            if self.embed_win32_popup {
                self.popup.as_ref().and_then(|pop| {
                    if let Some((cx, cy)) = px {
                        let (mx, my) = if has_rect {
                            (cx.clamp(r.min.x, r.max.x), cy.clamp(r.min.y, r.max.y))
                        } else {
                            (cx, cy)
                        };
                        owner_egui_pos_from_popup_client_px(self.owner, pop.0, mx, my, ppp)
                    } else if has_rect {
                        owner_egui_pos_from_popup_client_px(self.owner, pop.0, r.min.x, r.min.y, ppp)
                    } else {
                        None
                    }
                })
            } else if let Some(content) = self.last_content_rect_egui.get() {
                if let Some((cx, cy)) = px {
                    let (mx, my) = if has_rect {
                        (cx.clamp(r.min.x, r.max.x), cy.clamp(r.min.y, r.max.y))
                    } else {
                        (cx, cy)
                    };
                    slint_egui_anchor_from_viewport_client_px(content, mx, my, ppp)
                } else if has_rect {
                    slint_egui_anchor_from_viewport_client_px(content, r.min.x, r.min.y, ppp)
                } else {
                    None
                }
            } else {
                None
            }
        };

        let mut open = self.context_menu_window_open.get();
        let chosen = Cell::new(None::<ContextMenuAction>);
        let open_link_new_tonet_tab = Cell::new(false);
        let tonet_new_tab_url = Rc::new(RefCell::new(None::<String>));
        let escape = Cell::new(false);

        let mut win = egui::Window::new(crate::i18n::servo_context_menu_title(loc))
            .id(egui::Id::new("tonet_servo_context_menu"))
            .collapsible(false)
            .resizable(false)
            .constrain(true)
            .movable(false);
        win = if let Some(pos) = anchor_egui {
            win.fixed_pos(pos)
        } else {
            win.anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        };

        let tonet_url_flag = Rc::clone(&tonet_new_tab_url);
        let _ = win
            .open(&mut open)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(320.0)
                    .show(ui, |ui| {
                        for row in &rows {
                            match row {
                                MenuRow::Sep => {
                                    ui.separator();
                                }
                                MenuRow::Item {
                                    label,
                                    action,
                                    enabled,
                                } => {
                                    if ui
                                        .add_enabled(*enabled, egui::Button::new(label.as_str()))
                                        .clicked()
                                    {
                                        chosen.set(Some(*action));
                                    }
                                }
                                MenuRow::OpenLinkInNewTonetTab { url } => {
                                    if ui
                                        .button(crate::i18n::servo_context_menu_open_link_new_tonet_tab(
                                            loc,
                                        ))
                                        .on_hover_text(url.as_str())
                                        .clicked()
                                    {
                                        *tonet_url_flag.borrow_mut() = Some(url.clone());
                                        open_link_new_tonet_tab.set(true);
                                    }
                                }
                            }
                        }
                    });
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    escape.set(true);
                }
            });

        self.context_menu_window_open.set(open);

        let mut spin = false;

        if !open && self.context_menu_pending.borrow().is_some() {
            if let Some(cm) = self.context_menu_pending.borrow_mut().take() {
                self.input.last_rbutton_client_px.set(None);
                cm.dismiss();
                spin = true;
            }
            self.context_menu_window_open.set(true);
            self.input.needs_paint.set(true);
            ctx.request_repaint();
            return spin;
        }

        if escape.get() {
            if let Some(cm) = self.context_menu_pending.borrow_mut().take() {
                self.input.last_rbutton_client_px.set(None);
                cm.dismiss();
                spin = true;
            }
            self.context_menu_window_open.set(true);
            self.input.needs_paint.set(true);
            ctx.request_repaint();
            return spin;
        }

        if open_link_new_tonet_tab.get() {
            if let Some(cm) = self.context_menu_pending.borrow_mut().take() {
                self.input.last_rbutton_client_px.set(None);
                cm.dismiss();
                if let Some(u) = tonet_new_tab_url.borrow_mut().take() {
                    *self.pending_open_link_new_tonet_tab.borrow_mut() = Some(u);
                }
                spin = true;
            }
            self.context_menu_window_open.set(true);
            self.input.needs_paint.set(true);
            ctx.request_repaint();
            return spin;
        }

        if let Some(action) = chosen.get() {
            if let Some(cm) = self.context_menu_pending.borrow_mut().take() {
                self.input.last_rbutton_client_px.set(None);
                cm.select(action);
                spin = true;
            }
            self.context_menu_window_open.set(true);
            self.input.needs_paint.set(true);
            ctx.request_repaint();
            return spin;
        }

        false
    }

    /// Native `<select>` list from Servo. Deferred while a script dialog or context menu is open.
    pub(crate) fn show_select_element_if_pending(&mut self, ctx: &egui::Context, loc: Locale) -> bool {
        if self.dialog_pending.borrow().is_some()
            || self.auth_pending.borrow().is_some()
            || self.permission_pending.borrow().is_some()
            || self.context_menu_pending.borrow().is_some()
            || self.color_picker_pending.borrow().is_some()
            || self.file_picker_waiting.borrow().is_some()
        {
            return false;
        }
        if self.select_pending.borrow().is_none() {
            return false;
        }

        #[derive(Clone)]
        enum SelRow {
            Heading(String),
            Choice {
                id: usize,
                label: String,
                disabled: bool,
            },
        }

        fn flatten(opts: &[SelectElementOptionOrOptgroup], out: &mut Vec<SelRow>) {
            for o in opts {
                match o {
                    SelectElementOptionOrOptgroup::Option(sel) => {
                        out.push(SelRow::Choice {
                            id: sel.id,
                            label: sel.label.clone(),
                            disabled: sel.is_disabled,
                        });
                    }
                    SelectElementOptionOrOptgroup::Optgroup { label, options } => {
                        out.push(SelRow::Heading(label.clone()));
                        for child in options {
                            out.push(SelRow::Choice {
                                id: child.id,
                                label: child.label.clone(),
                                disabled: child.is_disabled,
                            });
                        }
                    }
                }
            }
        }

        let rows: Vec<SelRow> = {
            let slot = self.select_pending.borrow();
            let Some(s) = slot.as_ref() else {
                return false;
            };
            let mut v = Vec::new();
            flatten(s.options(), &mut v);
            v
        };

        let mut open = self.select_window_open.get();
        let ok_clicked = Cell::new(false);
        let cancel_clicked = Cell::new(false);
        let select_draft = self.select_draft.clone();

        let _ = egui::Window::new(crate::i18n::servo_select_title(loc))
            .id(egui::Id::new("tonet_servo_select_element"))
            .collapsible(false)
            .resizable(true)
            .constrain(true)
            .default_width(280.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut open)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                    let cur = select_draft.get();
                    for row in &rows {
                        match row {
                            SelRow::Heading(h) => {
                                ui.add_space(4.0);
                                ui.label(egui::RichText::new(h).strong());
                            }
                            SelRow::Choice { id, label, disabled } => {
                                let checked = cur == Some(*id);
                                if ui
                                    .add_enabled(
                                        !*disabled,
                                        egui::RadioButton::new(checked, label.as_str()),
                                    )
                                    .clicked()
                                {
                                    select_draft.set(Some(*id));
                                }
                            }
                        }
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button(crate::i18n::new_tab_add_cancel(loc)).clicked() {
                        cancel_clicked.set(true);
                    }
                    if ui.button(crate::i18n::servo_dialog_ok(loc)).clicked() {
                        ok_clicked.set(true);
                    }
                });
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    cancel_clicked.set(true);
                }
            });

        self.select_window_open.set(open);

        let mut spin = false;

        if !open && self.select_pending.borrow().is_some() {
            if let Some(s) = self.select_pending.borrow_mut().take() {
                drop(s);
                spin = true;
            }
            self.select_window_open.set(true);
            self.input.needs_paint.set(true);
            ctx.request_repaint();
            return spin;
        }

        if cancel_clicked.get() {
            if let Some(s) = self.select_pending.borrow_mut().take() {
                drop(s);
                spin = true;
            }
            self.select_window_open.set(true);
            self.input.needs_paint.set(true);
            ctx.request_repaint();
            return spin;
        }

        if ok_clicked.get() {
            if let Some(mut s) = self.select_pending.borrow_mut().take() {
                s.select(self.select_draft.get());
                s.submit();
                spin = true;
            }
            self.select_window_open.set(true);
            self.input.needs_paint.set(true);
            ctx.request_repaint();
            return spin;
        }

        false
    }

    /// `<input type=color>` from Servo. Deferred while dialog, context menu, or `<select>` is open.
    pub(crate) fn show_color_picker_if_pending(&mut self, ctx: &egui::Context, loc: Locale) -> bool {
        if self.dialog_pending.borrow().is_some()
            || self.auth_pending.borrow().is_some()
            || self.permission_pending.borrow().is_some()
            || self.context_menu_pending.borrow().is_some()
            || self.select_pending.borrow().is_some()
            || self.file_picker_waiting.borrow().is_some()
        {
            return false;
        }
        if self.color_picker_pending.borrow().is_none() {
            return false;
        }

        let mut open = self.color_picker_window_open.get();
        let ok_clicked = Cell::new(false);
        let cancel_clicked = Cell::new(false);
        let draft = self.color_picker_draft.clone();

        let _ = egui::Window::new(crate::i18n::servo_color_picker_title(loc))
            .id(egui::Id::new("tonet_servo_color_picker"))
            .collapsible(false)
            .resizable(false)
            .constrain(true)
            .default_width(260.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(crate::i18n::servo_color_picker_label(loc));
                    let mut rgb = *draft.borrow();
                    ui.color_edit_button_srgb(&mut rgb);
                    *draft.borrow_mut() = rgb;
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(crate::i18n::new_tab_add_cancel(loc)).clicked() {
                        cancel_clicked.set(true);
                    }
                    if ui.button(crate::i18n::servo_dialog_ok(loc)).clicked() {
                        ok_clicked.set(true);
                    }
                });
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    cancel_clicked.set(true);
                }
            });

        self.color_picker_window_open.set(open);

        let mut spin = false;

        if !open && self.color_picker_pending.borrow().is_some() {
            if let Some(c) = self.color_picker_pending.borrow_mut().take() {
                drop(c);
                spin = true;
            }
            self.color_picker_window_open.set(true);
            self.input.needs_paint.set(true);
            ctx.request_repaint();
            return spin;
        }

        if cancel_clicked.get() {
            if let Some(c) = self.color_picker_pending.borrow_mut().take() {
                drop(c);
                spin = true;
            }
            self.color_picker_window_open.set(true);
            self.input.needs_paint.set(true);
            ctx.request_repaint();
            return spin;
        }

        if ok_clicked.get() {
            if let Some(mut c) = self.color_picker_pending.borrow_mut().take() {
                let rgb = *self.color_picker_draft.borrow();
                c.select(Some(rgb_array_to_servo(rgb)));
                c.submit();
                spin = true;
            }
            self.color_picker_window_open.set(true);
            self.input.needs_paint.set(true);
            ctx.request_repaint();
            return spin;
        }

        false
    }

    /// Completes `<input type=file>` after the background [`rfd`] dialog returns.
    pub(crate) fn poll_file_picker_completion(&mut self, ctx: &egui::Context) -> bool {
        if self.permission_pending.borrow().is_some() || self.auth_pending.borrow().is_some() {
            return false;
        }
        let rx_cell = self.file_picker_rx.borrow();
        let Some(rx) = rx_cell.as_ref() else {
            return false;
        };
        let recv = match rx.try_recv() {
            Ok(v) => v,
            Err(mpsc::TryRecvError::Empty) => return false,
            Err(mpsc::TryRecvError::Disconnected) => None,
        };
        drop(rx_cell);
        *self.file_picker_rx.borrow_mut() = None;
        let Some(mut fp) = self.file_picker_waiting.borrow_mut().take() else {
            return false;
        };
        let mut spin = false;
        match recv {
            None => fp.dismiss(),
            Some(paths) => {
                fp.select(&paths);
                fp.submit();
                spin = true;
            }
        }
        if !spin {
            spin = true;
        }
        self.input.needs_paint.set(true);
        ctx.request_repaint();
        spin
    }

    fn reset_pointer_state(&mut self) {
        self.input.page_captures_keyboard.set(false);
        if self.input.left_down.get() {
            if let Some(last) = self.input.last_move.get() {
                self.input.webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                    MouseButtonAction::Up,
                    MouseButton::Left,
                    WebViewPoint::from(last),
                )));
            }
            self.input.left_down.set(false);
        }
        if self.input.last_move.get().is_some() {
            self.input.webview.notify_input_event(InputEvent::MouseLeftViewport(
                MouseLeftViewportEvent::default(),
            ));
        }
        self.input.last_move.set(None);
    }

    pub(crate) fn page_captures_keyboard(&self) -> bool {
        self.input.page_captures_keyboard.get()
    }

    pub(crate) fn clear_page_keyboard_capture(&self) {
        self.input.page_captures_keyboard.set(false);
    }

    /// Drain egui keyboard events and send them to Servo. Call early in the frame (before chrome
    /// TextEdits) while [`Self::page_captures_keyboard`] is true.
    /// Returns whether any event was sent (caller may [`Self::spin_event_loop`]).
    pub(crate) fn forward_egui_keyboard(&self, ctx: &egui::Context) -> bool {
        if !self.input.page_captures_keyboard.get() {
            return false;
        }
        let mut any = false;
        ctx.input_mut(|input| {
            let mut kept = Vec::new();
            let drained = std::mem::take(&mut input.events);
            for ev in drained {
                match ev {
                    egui::Event::Text(s) if !s.is_empty() => {
                        any = true;
                        let mods = servo_modifiers(&input.modifiers);
                        for ch in s.chars() {
                            let key = Key::Character(ch.to_string());
                            let down = KeyboardEvent::new_without_event(
                                KeyState::Down,
                                key.clone(),
                                Code::Unidentified,
                                Location::Standard,
                                mods,
                                false,
                                false,
                            );
                            let up = KeyboardEvent::new_without_event(
                                KeyState::Up,
                                key,
                                Code::Unidentified,
                                Location::Standard,
                                mods,
                                false,
                                false,
                            );
                            self.input.webview.notify_input_event(InputEvent::Keyboard(down));
                            self.input.webview.notify_input_event(InputEvent::Keyboard(up));
                        }
                    }
                    egui::Event::Key {
                        key,
                        physical_key,
                        pressed,
                        repeat,
                        modifiers,
                    } => {
                        // Shell shortcuts use Ctrl/Alt on Windows (`command` is the macOS meta key).
                        // Do not forward those Key events to Servo or egui never sees Ctrl+T, Ctrl+W, …
                        if modifiers.command || modifiers.ctrl || modifiers.alt {
                            kept.push(egui::Event::Key {
                                key,
                                physical_key,
                                pressed,
                                repeat,
                                modifiers,
                            });
                            continue;
                        }
                        if key == egui::Key::Escape && pressed {
                            self.input.page_captures_keyboard.set(false);
                            kept.push(egui::Event::Key {
                                key,
                                physical_key,
                                pressed,
                                repeat,
                                modifiers,
                            });
                            continue;
                        }
                        if !egui_key_should_forward_as_key_event(key, &modifiers) {
                            continue;
                        }
                        let Some(skey) = egui_key_to_servo_key(key) else {
                            kept.push(egui::Event::Key {
                                key,
                                physical_key,
                                pressed,
                                repeat,
                                modifiers,
                            });
                            continue;
                        };
                        let code = physical_key
                            .map(egui_key_to_code)
                            .unwrap_or_else(|| egui_key_to_code(key));
                        let state = if pressed {
                            KeyState::Down
                        } else {
                            KeyState::Up
                        };
                        let kb = KeyboardEvent::new_without_event(
                            state,
                            skey,
                            code,
                            Location::Standard,
                            servo_modifiers(&modifiers),
                            repeat,
                            false,
                        );
                        self.input.webview.notify_input_event(InputEvent::Keyboard(kb));
                        any = true;
                    }
                    egui::Event::Ime(ime) => {
                        let servo_ime = match ime {
                            egui::ImeEvent::Enabled => ServoImeEvent::Composition(CompositionEvent {
                                state: CompositionState::Start,
                                data: String::new(),
                            }),
                            egui::ImeEvent::Preedit(s) => ServoImeEvent::Composition(CompositionEvent {
                                state: CompositionState::Update,
                                data: s,
                            }),
                            egui::ImeEvent::Commit(s) => ServoImeEvent::Composition(CompositionEvent {
                                state: CompositionState::End,
                                data: s,
                            }),
                            egui::ImeEvent::Disabled => ServoImeEvent::Dismissed,
                        };
                        self.input
                            .webview
                            .notify_input_event(InputEvent::Ime(servo_ime));
                        any = true;
                    }
                    other => kept.push(other),
                }
            }
            input.events = kept;
        });
        if any {
            self.input.needs_paint.set(true);
            self.input.ctx.request_repaint();
        }
        any
    }

    pub(crate) fn spin_event_loop(&mut self) {
        self.servo.spin_event_loop();
    }

    /// Push Servo-reported URL / title / load / BF state into the active [`Tab`].
    pub(crate) fn sync_into_tab(
        &self,
        tab: &mut Tab,
        ctx: &egui::Context,
        browser_log: &mut crate::browser_log::BrowserLog,
    ) {
        let snap = self.shell_snapshot.borrow().clone();
        tab.servo_chrome_nav = Some((snap.can_go_back, snap.can_go_forward));
        tab.servo_document_title = snap.title.clone();
        tab.loading = snap.load_status != LoadStatus::Complete;
        if !ctx.memory(|m| m.has_focus(omnibox_id())) {
            if let Some(ref u) = snap.committed_url {
                let u = u.trim();
                if !u.is_empty() && tab.url_input.trim() != u {
                    tab.url_input = u.to_string();
                }
            }
        }

        let complete = snap.load_status == LoadStatus::Complete;
        let was_complete = self.prev_shell_complete.get();
        let committed_trim = snap
            .committed_url
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());
        if visit_policy::should_record_visit(
            complete,
            committed_trim,
            was_complete,
            self.last_recorded_visit_url.borrow().as_deref(),
        ) {
            if let Some(u) = committed_trim {
                let title = snap.title.clone();
                browser_log.record_visit(u.to_string(), title.clone());
                // Match native `poll_fetch`: append the same navigation to the downloads / fetch log.
                // Servo does not expose committed HTML here, so no `page-snapshots` path yet.
                browser_log.record_page_fetch(u, title, None);
                *self.last_recorded_visit_url.borrow_mut() = Some(u.to_string());
            }
        }
        self.prev_shell_complete.set(complete);

        if let Some(ref png) = snap.favicon_png {
            if !png.is_empty() {
                let mut h = DefaultHasher::new();
                png.hash(&mut h);
                let digest = h.finish();
                if self.last_favicon_png_hash.get() != digest {
                    let page_key = committed_trim.unwrap_or_else(|| tab.url_input.trim());
                    let uri = crate::app::favicon_cache_uri(page_key, "png");
                    ctx.include_bytes(
                        uri.clone(),
                        egui::load::Bytes::Shared(std::sync::Arc::from(png.as_slice())),
                    );
                    tab.favicon_uri = uri;
                    self.last_favicon_png_hash.set(digest);
                    ctx.request_repaint();
                }
            }
        }

        let pending: Vec<_> = self.console_pending.borrow_mut().drain(..).collect();
        if !pending.is_empty() {
            ctx.request_repaint();
            for (lvl, msg) in pending {
                tab.push_servo_console_line(map_console_level(lvl), msg);
            }
        }

        if let Ok(mut q) = self.background_download_done.lock() {
            if !q.is_empty() {
                ctx.request_repaint();
                for rec in q.drain(..) {
                    browser_log.record_page_fetch(&rec.source_url, rec.display_name.clone(), Some(rec.saved_path));
                }
            }
        }
    }

    pub(crate) fn webview_go_back(&self) -> bool {
        if !self.input.webview.can_go_back() {
            return false;
        }
        let _ = self.input.webview.go_back(1);
        self.input.needs_paint.set(true);
        true
    }

    pub(crate) fn webview_go_forward(&self) -> bool {
        if !self.input.webview.can_go_forward() {
            return false;
        }
        let _ = self.input.webview.go_forward(1);
        self.input.needs_paint.set(true);
        true
    }

    pub(crate) fn webview_reload(&self) {
        self.input.webview.reload();
        self.input.needs_paint.set(true);
    }

    fn refresh_slint_egui_frame(&self, gpu: &GPURenderingContext, ctx: &egui::Context) {
        let sz = gpu.size.get();
        let vw = sz.width.max(1) as i32;
        let vh = sz.height.max(1) as i32;
        let clip = DeviceIntRect::from_origin_and_size(
            DeviceIntPoint::zero(),
            DeviceIntSize::new(vw, vh),
        );
        let _ = gpu.make_current();
        gpu.prepare_for_rendering();
        if let Some(rgba) = gpu.read_to_image(clip) {
            let (w, h) = rgba.dimensions();
            let raw = rgba.into_raw();
            let mut pixels = Vec::with_capacity((w * h) as usize);
            for chunk in raw.chunks_exact(4) {
                pixels.push(egui::Color32::from_rgba_unmultiplied(
                    chunk[0], chunk[1], chunk[2], chunk[3],
                ));
            }
            *self.slint_egui_frame.borrow_mut() = Some(egui::ColorImage {
                size: [w as usize, h as usize],
                pixels,
            });
            ctx.request_repaint();
        }
    }

    pub(crate) fn slint_egui_frame_snapshot(&self) -> Option<egui::ColorImage> {
        self.slint_egui_frame.borrow().clone()
    }

    /// Pointer / scroll from egui for Slint-style embed (Win32 path uses `WNDPROC` instead).
    pub(crate) fn feed_egui_servo_embed_input(&self, ctx: &egui::Context, content: egui::Rect, ppp: f32) {
        if self.embed_win32_popup || self.slint_gpu.is_none() {
            return;
        }
        let ppp = ppp.max(0.01);
        let (interact, scroll, primary_down, primary_up, secondary_down, secondary_up) =
            ctx.input(|i| {
                (
                    i.pointer.interact_pos(),
                    i.smooth_scroll_delta,
                    i.pointer.primary_pressed(),
                    i.pointer.primary_released(),
                    i.pointer.secondary_pressed(),
                    i.pointer.secondary_released(),
                )
            });
        let Some(pos) = interact.filter(|p| content.contains(*p)) else {
            return;
        };
        let lp = (pos - content.min) * ppp;
        let dp = DevicePoint::new(lp.x, lp.y);
        let wp = WebViewPoint::from(dp);
        self.input
            .webview
            .notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(wp)));
        if scroll.x != 0.0 || scroll.y != 0.0 {
            let dy = -scroll.y * 40.0;
            let dx = -scroll.x * 40.0;
            self.input.webview.notify_input_event(InputEvent::Wheel(WheelEvent::new(
                WheelDelta {
                    x: dx as f64,
                    y: dy as f64,
                    z: 0.0,
                    mode: WheelMode::DeltaLine,
                },
                wp,
            )));
        }
        if primary_down {
            self.input.page_captures_keyboard.set(true);
            self.input
                .ctx
                .memory_mut(|mem| mem.surrender_focus(omnibox_id()));
            self.input.webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                MouseButtonAction::Down,
                MouseButton::Left,
                wp,
            )));
        }
        if primary_up {
            self.input.webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                MouseButtonAction::Up,
                MouseButton::Left,
                wp,
            )));
        }
        if secondary_down {
            let px = (dp.x.round() as i32, dp.y.round() as i32);
            self.input.last_rbutton_client_px.set(Some(px));
            self.input.webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                MouseButtonAction::Down,
                MouseButton::Right,
                wp,
            )));
        }
        if secondary_up {
            self.input.webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                MouseButtonAction::Up,
                MouseButton::Right,
                wp,
            )));
        }
        if primary_down || scroll != egui::Vec2::ZERO || secondary_down {
            self.input.needs_paint.set(true);
        }
    }

    pub fn tick(
        &mut self,
        tab_url: &str,
        content_rect: Option<egui::Rect>,
        ppp: f32,
        ctx: &egui::Context,
        throttle_servo_spin: bool,
    ) {
        if self.embed_win32_popup {
            if let Some(pop) = &self.popup {
                pump_messages_for_hwnd(pop.0);
                unsafe {
                    let f = GetFocus();
                    if !f.is_null() && IsChild(pop.0, f) != 0 {
                        let _ = SetFocus(self.owner);
                    }
                }
            }
        }

        let Some(rect) = content_rect else {
            self.reset_pointer_state();
            self.servo.spin_event_loop();
            self.last_content_rect_egui.set(None);
            if self.embed_win32_popup && self.popup_visible {
                if let Some(pop) = &self.popup {
                    unsafe {
                        let _ = ShowWindow(pop.0, SW_HIDE);
                    }
                }
                self.popup_visible = false;
            }
            return;
        };

        self.last_content_rect_egui.set(Some(rect));

        ctx.input(|i| {
            if i.pointer.primary_clicked() {
                if let Some(pos) = i.pointer.interact_pos() {
                    if !rect.contains(pos) {
                        self.input.page_captures_keyboard.set(false);
                    }
                }
            }
        });

        if throttle_servo_spin {
            self.idle_spin_counter = self.idle_spin_counter.wrapping_add(1);
            if self.idle_spin_counter % 4 == 0 {
                self.servo.spin_event_loop();
            }
        } else {
            self.idle_spin_counter = 0;
            self.servo.spin_event_loop();
        }

        if self.embed_win32_popup {
            if !self.popup_visible {
                if let Some(pop) = &self.popup {
                    unsafe {
                        let _ = ShowWindow(pop.0, SW_SHOW);
                    }
                }
                self.popup_visible = true;
                self.input.needs_paint.set(true);
            }

            if let (Some(pop), Some((sx, sy, ow, oh))) = (
                &self.popup,
                outer_screen_from_egui_rect(self.owner, rect, ppp),
            ) {
                let flags = SWP_NOZORDER | SWP_SHOWWINDOW;
                unsafe {
                    let _ = SetWindowPos(pop.0, HWND_TOP, sx, sy, ow, oh, flags);
                }
            }

            let (cw, ch) = if let Some(pop) = &self.popup {
                let mut cr = RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
                unsafe {
                    let _ = GetClientRect(pop.0, &mut cr);
                }
                (
                    (cr.right - cr.left).max(1) as u32,
                    (cr.bottom - cr.top).max(1) as u32,
                )
            } else {
                (1, 1)
            };
            if self.last_client.map(|p| p != (cw, ch)).unwrap_or(true) {
                self.input.webview.resize(PhysicalSize::new(cw, ch));
                self.last_client = Some((cw, ch));
                self.input.needs_paint.set(true);
            }
        } else {
            let ppp = ppp.max(0.01);
            let cw = (rect.width() * ppp).round().max(1.0) as u32;
            let ch = (rect.height() * ppp).round().max(1.0) as u32;
            if self.last_client.map(|p| p != (cw, ch)).unwrap_or(true) {
                self.input.webview.resize(PhysicalSize::new(cw, ch));
                if let Some(gpu) = &self.slint_gpu {
                    gpu.resize(PhysicalSize::new(cw, ch));
                }
                self.last_client = Some((cw, ch));
                self.input.needs_paint.set(true);
            }
        }

        let ppp = ppp.max(0.01);
        if (ppp - self.last_ppp).abs() > 1e-4 {
            self.input.webview.set_hidpi_scale_factor(Scale::new(ppp));
            self.last_ppp = ppp;
            self.input.needs_paint.set(true);
        }

        // While the user edits the omnibox, do not push partial URLs into Servo each frame.
        if !ctx.memory(|m| m.has_focus(omnibox_id())) {
            let next = initial_nav_url(tab_url);
            let next_s = next.to_string();
            if next_s != self.last_tab_url {
                self.input.webview.load(next);
                self.last_tab_url = next_s;
                self.input.needs_paint.set(true);
            }
        }

        *self.shell_snapshot.borrow_mut() = ServoShellSnapshot::capture_from(&self.input.webview);
        self.input.servo_cursor.set(self.input.webview.cursor());

        let painted = self.input.needs_paint.replace(false);
        if painted {
            self.input.webview.paint();
            if let Some(rc) = &self.rendering_context {
                rc.present();
            } else if let Some(gpu) = &self.slint_gpu {
                let _ = gpu.make_current();
                gpu.present();
                self.refresh_slint_egui_frame(gpu, ctx);
            }
        }
    }

    /// Dismiss or complete queued embedder UI so Servo gets IPC responses before [`WebView`] /
    /// [`Servo`] are torn down (e.g. experimental viewport off or app exit).
    fn teardown_pending_embedder_controls(&self) {
        if let Some(d) = self.dialog_pending.borrow_mut().take() {
            d.dismiss();
        }
        if let Some(cm) = self.context_menu_pending.borrow_mut().take() {
            cm.dismiss();
        }
        if let Some(s) = self.select_pending.borrow_mut().take() {
            drop(s);
        }
        if let Some(c) = self.color_picker_pending.borrow_mut().take() {
            drop(c);
        }
        *self.file_picker_rx.borrow_mut() = None;
        if let Some(fp) = self.file_picker_waiting.borrow_mut().take() {
            fp.dismiss();
        }
        if let Some(p) = self.permission_pending.borrow_mut().take() {
            *self.permission_prompt_origin.borrow_mut() = None;
            p.deny();
        }
        if let Some(a) = self.auth_pending.borrow_mut().take() {
            drop(a);
        }
        self.auth_user_draft.borrow_mut().clear();
        self.auth_pass_draft.borrow_mut().clear();
        *self.notification_toast.borrow_mut() = None;
        self.console_pending.borrow_mut().clear();
        if let Ok(mut q) = self.background_download_done.lock() {
            q.clear();
        }
    }
}

impl Drop for ServoWinHost {
    fn drop(&mut self) {
        self.teardown_pending_embedder_controls();
        self.servo.spin_event_loop();
    }
}
