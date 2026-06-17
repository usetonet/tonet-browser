//! Integración del motor **Servo** (mismo binario `tonet`, sin crate aparte).
//!
//! La feature Cargo `servo-engine` enlaza el crate `servo` de crates.io. El **shell** (chrome,
//! pestañas, omnibox, i18n, etc.) sigue siendo **egui/eframe**.
//!
//! **Windows + `servo-engine`:** las pestañas `http://`, `https://` y las internas `tonet://…`
//! usan **solo Servo** como motor de contenido (HTML interceptado para `tonet://`; sin pintado DOM
//! con `tonet-engine`). El runtime del viewport está **siempre activo** salvo que se defina
//! `TONET_SERVO_VIEWPORT=0` para depuración o emergencias. La página nueva sigue en egui.
//!
//! **Referencia (Slint `examples/servo/`):** Servo pinta en un **surfman** `GPURenderingContext`
//! (swapchain OpenGL/ANGLE). Por defecto Tonet **no** usa ventana popup: lee el framebuffer a CPU
//! y lo muestra en el `CentralPanel` (composición vía egui). Opcional: `TONET_SERVO_WIN32_POPUP=1`
//! restaura el popup Win32 + `WindowRenderingContext` (subclass `WNDPROC` para puntero/rueda).
//!
//! **Fase 2–3 (Windows + `servo-engine`, popup legacy):** ventana Win32 **popup** (owned) con
//! `WindowRenderingContext` + `Servo` + `WebView`. El rect del `CentralPanel` se convierte a
//! pantalla con `SetWindowPos` + resize del `WebView` cada frame.
//! **Fase 4 (popup):** puntero y rueda vía **subclass** `WNDPROC` del popup (`WebView::notify_input_event`).
//! **Fase 4 (embed Slint, por defecto):** puntero/rueda desde egui (`feed_servo_slint_egui_pointer`).
//! El popup usa `WS_EX_NOACTIVATE` y `WM_MOUSEACTIVATE` → `MA_NOACTIVATE` para no activar la
//! superficie al clicar; además, si un hijo del `WebView` roba `GetFocus`, cada `tick` lo devuelve
//! al `HWND` dueño para que winit/egui sigan recibiendo texto. Tras un clic en la página,
//! `forward_captured_keyboard` (al inicio del frame) reenvía `Event::Text` / `Event::Key` a Servo
//! salvo atajos con Ctrl/Alt/Cmd, que permanecen en egui; Escape o un clic fuera del rect del
//! contenido suelta el modo “página captura teclado”.
//! **Fase 5 (shell):** cada frame se lee el estado del `WebView` (URL confirmada, título,
//! `LoadStatus`, lista atrás/adelante) y se vuelca en la pestaña activa: omnibox, título de ventana,
//! pestañas, spinner de carga, y botones < > / recarga usan la misma lógica que el motor Servo
//! (sin fetch HTML paralelo del pipeline Tonet).
//! En Linux con `servo-engine` el hook existe pero la ventana nativa aún no está implementada.
//!
//! Windows: MSVC + Windows SDK (ANGLE) y `LIBCLANG_PATH` para bindgen/mozjs.

pub(crate) mod visit_policy;

#[cfg(feature = "servo-engine")]
pub(crate) mod servo_favicon;

#[cfg(all(feature = "servo-engine", windows))]
pub(crate) mod permission_store;

#[cfg(all(feature = "servo-engine", windows))]
mod background_download;
#[cfg(all(feature = "servo-engine", windows))]
mod content_disposition;
#[cfg(all(feature = "servo-engine", windows))]
mod download_heuristic;
#[cfg(all(feature = "servo-engine", windows))]
pub mod embedder_devtools;
#[cfg(all(feature = "servo-engine", windows))]
pub mod embedder_ids;
#[cfg(all(feature = "servo-engine", windows))]
mod runtime_win;
#[cfg(all(feature = "servo-engine", windows))]
mod slint_embed;
#[cfg(all(feature = "servo-engine", windows))]
mod tonet_scheme_html;
#[cfg(all(feature = "servo-engine", windows))]
mod url_path;
#[cfg(all(feature = "servo-engine", windows))]
pub(crate) use tonet_scheme_html::TonetSchemeAction;

#[cfg(all(feature = "servo-engine", windows))]
use crate::tab::Tab;

/// Mantiene el enlace al crate `servo` cuando compilas con `--features servo-engine`.
#[cfg(feature = "servo-engine")]
pub fn link_servo_when_enabled() {
    let _ = std::mem::size_of::<servo::ServoBuilder>();
}

#[cfg(not(feature = "servo-engine"))]
pub fn link_servo_when_enabled() {}

/// `true` when this binary was built with `--features servo-engine`.
#[allow(dead_code)] // Only referenced from `servo-engine` + Windows paths; kept as a public helper.
#[inline]
pub const fn servo_engine_feature_enabled() -> bool {
    cfg!(feature = "servo-engine")
}

/// Whether the native Servo viewport runtime should spin (Windows child `WebView` / host).
///
/// - **Windows + `servo-engine`:** always `true` unless `TONET_SERVO_VIEWPORT=0` (opt-out).
///   The `user_setting` argument is ignored on Windows (kept for call-site stability).
/// - **Other targets + `servo-engine`:** `user_setting` **or** env `TONET_SERVO_VIEWPORT=1`
///   (Linux/macOS embed still pending).
/// - **Without `servo-engine`:** always `false`.
#[allow(dead_code)] // Windows + `servo-engine` builds only; Linux callers may pass through.
#[inline]
pub fn viewport_runtime_requested(user_setting: bool) -> bool {
    if !servo_engine_feature_enabled() {
        return false;
    }
    #[cfg(all(feature = "servo-engine", windows))]
    {
        let _ = user_setting;
        if std::env::var_os("TONET_SERVO_VIEWPORT").as_deref() == Some(std::ffi::OsStr::new("0")) {
            return false;
        }
        true
    }
    #[cfg(all(feature = "servo-engine", not(windows)))]
    {
        user_setting
            || std::env::var_os("TONET_SERVO_VIEWPORT")
                .as_deref()
                .is_some_and(|v| v == "1")
    }
    #[cfg(not(feature = "servo-engine"))]
    {
        let _ = user_setting;
        false
    }
}

/// When true, the built-in `tonet-engine` DOM paint path should be skipped for this tab URL.
#[inline]
pub fn servo_supersedes_dom_paint(user_setting: bool, tab_url_trim: &str) -> bool {
    #[cfg(all(feature = "servo-engine", windows))]
    {
        viewport_runtime_requested(user_setting) && {
            let t = tab_url_trim.trim();
            let lc = t.to_ascii_lowercase();
            lc.starts_with("http://") || lc.starts_with("https://") || lc.starts_with("tonet://")
        }
    }
    #[cfg(not(all(feature = "servo-engine", windows)))]
    {
        let _ = (user_setting, tab_url_trim);
        false
    }
}

/// Owns optional Servo runtime (Windows popup). Always present on `TonetApp` via [`Default`].
pub struct ServoViewportRuntime {
    #[cfg(all(feature = "servo-engine", windows))]
    sessions: std::collections::HashMap<u32, runtime_win::ServoWinHost>,
    #[cfg(all(feature = "servo-engine", windows))]
    active_tab_id: Option<u32>,
    #[cfg(all(feature = "servo-engine", windows))]
    slint_embed_tex: std::cell::RefCell<std::collections::HashMap<u32, egui::TextureHandle>>,
    /// Shared snapshot for `tonet://` HTML served inside Servo (`load_web_resource` intercept).
    #[cfg(all(feature = "servo-engine", windows))]
    tonet_scheme_state: std::sync::Arc<std::sync::Mutex<tonet_scheme_html::TonetSchemeSharedState>>,
    /// One [`servo::Servo`] per process (`servo-config` [`Opts`] may only be initialized once).
    #[cfg(all(feature = "servo-engine", windows))]
    shared_servo: Option<std::rc::Rc<servo::Servo>>,
    #[cfg(not(all(feature = "servo-engine", windows)))]
    _marker: std::marker::PhantomData<()>,
}

impl Default for ServoViewportRuntime {
    fn default() -> Self {
        Self {
            #[cfg(all(feature = "servo-engine", windows))]
            sessions: std::collections::HashMap::new(),
            #[cfg(all(feature = "servo-engine", windows))]
            active_tab_id: None,
            #[cfg(all(feature = "servo-engine", windows))]
            slint_embed_tex: std::cell::RefCell::new(std::collections::HashMap::new()),
            #[cfg(all(feature = "servo-engine", windows))]
            tonet_scheme_state: std::sync::Arc::new(std::sync::Mutex::new(
                tonet_scheme_html::TonetSchemeSharedState::default(),
            )),
            shared_servo: None,
            #[cfg(not(all(feature = "servo-engine", windows)))]
            _marker: std::marker::PhantomData,
        }
    }
}

#[cfg(all(feature = "servo-engine", windows))]
impl ServoViewportRuntime {
    fn ensure_shared_servo(&mut self, ctx: &egui::Context) -> std::rc::Rc<servo::Servo> {
        if let Some(s) = self.shared_servo.as_ref() {
            return std::rc::Rc::clone(s);
        }
        let servo = std::rc::Rc::new(
            runtime_win::build_shared_servo(ctx),
        );
        self.shared_servo = Some(std::rc::Rc::clone(&servo));
        servo
    }

    fn spin_shared_servo(&self) {
        if let Some(servo) = self.shared_servo.as_ref() {
            servo.spin_event_loop();
        }
    }
    fn active_host(&self) -> Option<&runtime_win::ServoWinHost> {
        let id = self.active_tab_id?;
        self.sessions.get(&id)
    }

    fn active_host_mut(&mut self) -> Option<&mut runtime_win::ServoWinHost> {
        let id = self.active_tab_id?;
        self.sessions.get_mut(&id)
    }

    pub fn set_active_tab_id(&mut self, tab_id: u32) {
        self.active_tab_id = Some(tab_id);
    }

    pub fn close_tab_session(&mut self, tab_id: u32) {
        self.sessions.remove(&tab_id);
        self.slint_embed_tex.borrow_mut().remove(&tab_id);
        if self.active_tab_id == Some(tab_id) {
            self.active_tab_id = None;
        }
    }

}

impl ServoViewportRuntime {
    #[cfg(all(feature = "servo-engine", windows))]
    pub(crate) fn sync_tonet_scheme_snapshot(
        &mut self,
        loc: crate::i18n::Locale,
        settings: &crate::settings::AppSettings,
        log: &crate::browser_log::BrowserLog,
    ) {
        if let Ok(mut g) = self.tonet_scheme_state.lock() {
            g.loc = loc;
            g.settings = settings.clone();
            g.visits = log.visits.clone();
            g.downloads = log.downloads.clone();
        }
    }

    #[cfg(all(feature = "servo-engine", windows))]
    pub(crate) fn take_tonet_scheme_actions(
        &mut self,
    ) -> Vec<tonet_scheme_html::TonetSchemeAction> {
        if let Ok(mut g) = self.tonet_scheme_state.lock() {
            return std::mem::take(&mut g.pending_actions);
        }
        Vec::new()
    }

    /// Clear in-memory Servo permission decisions (after [`permission_store::remove_file`] or when
    /// reloading policy without restarting the process).
    #[cfg(all(feature = "servo-engine", windows))]
    pub fn clear_servo_permission_memory(&mut self) {
        for host in self.sessions.values_mut() {
            host.clear_servo_permission_memory();
        }
    }

    #[cfg(not(all(feature = "servo-engine", windows)))]
    #[allow(dead_code)]
    pub fn clear_servo_permission_memory(&mut self) {}

    /// Clear Servo embedder queues that mirror or feed the downloads / console UI (`ServoWinHost::clear_ephemeral_embedder_queues`).
    #[cfg(all(feature = "servo-engine", windows))]
    pub fn clear_servo_ephemeral_queues(&self) {
        for host in self.sessions.values() {
            host.clear_ephemeral_embedder_queues();
        }
    }

    #[cfg(not(all(feature = "servo-engine", windows)))]
    #[allow(dead_code)]
    pub fn clear_servo_ephemeral_queues(&self) {}

    /// Stop routing keys to Servo when the omnibox is focused or the user clicked outside the
    /// last known content rect (toolbar / tabs).
    #[cfg(all(feature = "servo-engine", windows))]
    pub fn release_servo_keyboard_capture(
        &mut self,
        ctx: &egui::Context,
        prev_content_rect: Option<egui::Rect>,
    ) {
        if ctx.memory(|m| m.has_focus(crate::ui::omnibox_id())) {
            for host in self.sessions.values() {
                host.clear_page_keyboard_capture();
            }
            return;
        }
        let Some(host) = self.active_host() else {
            return;
        };
        let Some(rect) = prev_content_rect else {
            return;
        };
        if !host.page_captures_keyboard() {
            return;
        }
        let clear = ctx.input(|i| {
            i.pointer.primary_clicked()
                && i.pointer
                    .interact_pos()
                    .is_some_and(|pos| !rect.contains(pos))
        });
        if clear {
            host.clear_page_keyboard_capture();
        }
    }

    /// When the Servo surface has keyboard focus, drain egui key/text/IME events into Servo.
    /// Must run **early** in the frame (before omnibox / shortcuts) so keys are not consumed twice.
    #[cfg(all(feature = "servo-engine", windows))]
    pub fn forward_captured_keyboard(
        &mut self,
        ctx: &egui::Context,
        user_setting: bool,
        tab_url: &str,
    ) {
        if !viewport_runtime_requested(user_setting) {
            return;
        }
        if !servo_supersedes_dom_paint(user_setting, tab_url) {
            return;
        }
        if ctx.memory(|m| m.has_focus(crate::ui::omnibox_id())) {
            for host in self.sessions.values() {
                host.clear_page_keyboard_capture();
            }
            return;
        }
        let Some(host) = self.active_host() else {
            return;
        };
        let mut any = false;
        if host.page_captures_keyboard() {
            any = host.forward_egui_keyboard(ctx);
        } else if let Some(rect) = host.last_content_rect_egui() {
            let over_page = ctx.input(|i| {
                runtime_win::servo_embed_pointer_over_rect(&i.pointer, rect)
            });
            if over_page {
                any = host.forward_egui_keyboard_navigation_hover(ctx);
            }
        }
        if any {
            self.spin_shared_servo();
        }
    }

    /// Pump Servo / Win32 for the experimental viewport when enabled; tear down when disabled.
    ///
    /// On **non-Windows** (or without `servo-engine`), this is a **no-op**; see checklist
    /// **§ Linux / macOS Servo embed** for the portability plan.
    pub fn tick(
        &mut self,
        ctx: &egui::Context,
        frame: &eframe::Frame,
        user_setting: bool,
        active_tab_id: u32,
        tab_url: &str,
        content_rect: Option<egui::Rect>,
        pending_load: Option<(String, bool)>,
    ) {
        #[cfg(all(feature = "servo-engine", windows))]
        {
            if !viewport_runtime_requested(user_setting) {
                self.sessions.clear();
                self.slint_embed_tex.borrow_mut().clear();
                self.active_tab_id = None;
                return;
            }
            self.active_tab_id = Some(active_tab_id);
            let ppp = ctx.pixels_per_point();
            let shared_servo = self.ensure_shared_servo(ctx);
            let servo_page = servo_supersedes_dom_paint(user_setting, tab_url);
            if servo_page && !self.sessions.contains_key(&active_tab_id) {
                if let Ok(host) = runtime_win::ServoWinHost::try_new(
                    ctx,
                    frame,
                    tab_url,
                    content_rect,
                    ppp,
                    std::sync::Arc::clone(&self.tonet_scheme_state),
                    shared_servo,
                ) {
                    self.sessions.insert(active_tab_id, host);
                }
            }
            if let Some((url, reload)) = pending_load {
                if let Some(host) = self.sessions.get_mut(&active_tab_id) {
                    host.navigate_url(url.as_str(), reload);
                }
            }
            let active = active_tab_id;
            if !self.sessions.is_empty() {
                self.spin_shared_servo();
            }
            for (tab_id, host) in self.sessions.iter_mut() {
                if *tab_id == active && servo_page {
                    host.tick(tab_url, content_rect, ppp, ctx, false);
                }
            }
        }
        #[cfg(not(all(feature = "servo-engine", windows)))]
        {
            let _ = (
                ctx,
                frame,
                user_setting,
                active_tab_id,
                tab_url,
                content_rect,
                pending_load,
            );
        }
    }

    /// Pointer / scroll from egui for Slint-style embed (ignored when using the Win32 popup path).
    #[cfg(all(feature = "servo-engine", windows))]
    pub fn feed_servo_slint_egui_pointer(
        &self,
        ctx: &egui::Context,
        content_rect: egui::Rect,
        ppp: f32,
        pointer_hint: Option<egui::Pos2>,
    ) {
        if let Some(host) = self.active_host() {
            host.feed_egui_servo_embed_input(ctx, content_rect, ppp, pointer_hint);
        }
    }

    #[cfg(all(feature = "servo-engine", windows))]
    pub fn scroll_thumb_ratio(&self) -> f32 {
        self.active_host()
            .map(|h| h.scroll_thumb_ratio())
            .unwrap_or(0.0)
    }

    #[cfg(all(feature = "servo-engine", windows))]
    pub fn set_scroll_thumb_ratio(&self, ratio: f32) {
        if let Some(host) = self.active_host() {
            host.set_scroll_thumb_ratio(ratio);
        }
    }

    pub fn request_dom_snapshot(&self) {
        if let Some(host) = self.active_host() {
            host.request_dom_snapshot();
        }
    }

    pub fn clear_network_log(&self) {
        if let Some(host) = self.active_host() {
            host.clear_network_log();
        }
    }

    pub fn dom_snapshot_inflight(&self) -> bool {
        self.active_host()
            .is_some_and(|h| h.dom_snapshot_inflight())
    }

    #[cfg(all(feature = "servo-engine", windows))]
    pub fn scroll_by_pixels(
        &self,
        dx_px: f32,
        dy_px: f32,
        content: egui::Rect,
        ppp: f32,
        pointer_hint: Option<egui::Pos2>,
    ) {
        let Some(host) = self.active_host() else {
            return;
        };
        host.scroll_by_pixels(dx_px, dy_px, content, ppp, pointer_hint);
    }

    /// Draw the latest Servo framebuffer (GPU readback) into `rect`; clears the texture cache when there is no frame yet.
    #[cfg(all(feature = "servo-engine", windows))]
    pub fn paint_servo_slint_embed_in_rect(
        &self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        rect: egui::Rect,
    ) {
        let Some(tab_id) = self.active_tab_id else {
            return;
        };
        let Some(host) = self.sessions.get(&tab_id) else {
            return;
        };
        let Some(img) = host.slint_egui_frame_snapshot() else {
            self.slint_embed_tex.borrow_mut().remove(&tab_id);
            ui.allocate_rect(rect, egui::Sense::hover());
            return;
        };
        let opts = egui::TextureOptions::LINEAR;
        let tex_name = format!("tonet_servo_frame_{tab_id}");
        {
            let mut map = self.slint_embed_tex.borrow_mut();
            match map.get_mut(&tab_id) {
                Some(tex) => tex.set(img, opts),
                None => {
                    map.insert(
                        tab_id,
                        ctx.load_texture(tex_name, img, opts),
                    );
                }
            }
        }
        let map = self.slint_embed_tex.borrow();
        let Some(texture) = map.get(&tab_id) else {
            return;
        };
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.set_min_size(rect.size());
            ui.add(
                egui::Image::new(egui::load::SizedTexture::new(
                    texture.id(),
                    texture.size_vec2(),
                ))
                .fit_to_exact_size(rect.size()),
            );
        });
    }

    /// Copy URL / title / loading / back-forward from the Servo [`WebView`] into the active tab.
    #[cfg(all(feature = "servo-engine", windows))]
    pub fn sync_active_tab_from_servo(
        &self,
        ctx: &egui::Context,
        user_setting: bool,
        tab: &mut Tab,
        browser_log: &mut crate::browser_log::BrowserLog,
    ) {
        if !viewport_runtime_requested(user_setting) {
            tab.servo_document_title = None;
            tab.servo_chrome_nav = None;
            tab.servo_console.clear();
            return;
        }
        if !servo_supersedes_dom_paint(user_setting, tab.url_input.trim()) {
            tab.servo_document_title = None;
            tab.servo_chrome_nav = None;
            tab.servo_console.clear();
            return;
        }
        let Some(host) = self.active_host() else {
            return;
        };
        host.sync_into_tab(tab, ctx, browser_log);
    }

    /// Script dialogs (`SimpleDialog`), **HTTP auth** (`AuthenticationRequest` → egui username/password), **site permissions** (`PermissionRequest`, with origin+feature cache loaded/saved via `permission_store` → `servo_permissions.json`), context menus (`ContextMenu`), `<select>` (`SelectElement`), and `<input type=color>` (`ColorPicker`) as egui windows; `<input type=file>` uses a native dialog (`rfd` on a worker thread), completed each frame by `ServoWinHost::poll_file_picker_completion`.
    #[cfg(all(feature = "servo-engine", windows))]
    pub fn show_embedder_modals(
        &mut self,
        ctx: &egui::Context,
        user_setting: bool,
        tab_url: &str,
        loc: crate::i18n::Locale,
    ) {
        if !viewport_runtime_requested(user_setting) {
            return;
        }
        if !servo_supersedes_dom_paint(user_setting, tab_url) {
            return;
        }
        let Some(host) = self.active_host_mut() else {
            return;
        };
        let spin = host.show_simple_dialog_if_pending(ctx, loc)
            || host.show_http_auth_if_pending(ctx, loc)
            || host.show_permission_request_if_pending(ctx, loc)
            || host.show_context_menu_if_pending(ctx, loc)
            || host.show_select_element_if_pending(ctx, loc)
            || host.show_color_picker_if_pending(ctx, loc)
            || host.poll_file_picker_completion(ctx);
        if spin {
            self.spin_shared_servo();
        }
    }

    #[cfg(all(feature = "servo-engine", windows))]
    pub fn webview_reload(&self) {
        if let Some(host) = self.active_host() {
            host.webview_reload();
        }
    }

    #[cfg(all(feature = "servo-engine", windows))]
    pub fn webview_go_back(&self) -> bool {
        self.active_host().is_some_and(|h| h.webview_go_back())
    }

    #[cfg(all(feature = "servo-engine", windows))]
    pub fn webview_go_forward(&self) -> bool {
        self.active_host().is_some_and(|h| h.webview_go_forward())
    }

    /// URL from the Servo page context menu (“Open link in new Tonet tab”), if the user chose it this frame.
    #[cfg(all(feature = "servo-engine", windows))]
    pub fn take_pending_open_link_new_tonet_tab(&self) -> Option<String> {
        self.active_host()
            .and_then(|h| h.take_pending_open_link_new_tonet_tab())
    }

    #[cfg(not(all(feature = "servo-engine", windows)))]
    #[allow(dead_code)]
    pub fn take_pending_open_link_new_tonet_tab(&self) -> Option<String> {
        None
    }

    /// Web Notification API from Servo: egui toast in the top chrome when the experimental viewport is active.
    #[cfg(all(feature = "servo-engine", windows))]
    pub fn show_web_notification_toast(&self, ctx: &egui::Context, loc: crate::i18n::Locale) {
        if let Some(host) = self.active_host() {
            host.show_web_notification_toast(ctx, loc);
        }
    }

    #[cfg(not(all(feature = "servo-engine", windows)))]
    #[allow(dead_code)]
    pub fn show_web_notification_toast(&self, _ctx: &egui::Context, _loc: crate::i18n::Locale) {}
}

#[cfg(all(test, feature = "servo-engine", windows))]
mod dom_paint_gate_tests {
    use super::{servo_supersedes_dom_paint, viewport_runtime_requested};

    #[test]
    fn servo_supersedes_https_default_ignores_user_setting_false() {
        assert!(servo_supersedes_dom_paint(false, "https://example/"));
    }

    #[test]
    fn viewport_runtime_requested_default_ignores_user_setting_false() {
        assert!(viewport_runtime_requested(false));
    }

    #[test]
    fn servo_supersedes_http_when_runtime_on() {
        assert!(servo_supersedes_dom_paint(false, "http://127.0.0.1/"));
    }

    #[test]
    fn servo_supersedes_trims_tab_url_before_scheme_check() {
        assert!(servo_supersedes_dom_paint(false, "  https://x/y  "));
    }

    #[test]
    fn servo_supersedes_tonet_scheme() {
        assert!(servo_supersedes_dom_paint(false, "tonet://settings"));
        assert!(servo_supersedes_dom_paint(false, "  TONET://DOWNLOADS  "));
    }

    #[test]
    fn servo_supersedes_rejects_other_schemes() {
        assert!(!servo_supersedes_dom_paint(false, "about:blank"));
        assert!(!servo_supersedes_dom_paint(false, "file:///tmp/x"));
    }

    /// Scheme check uses ASCII lowercase so omnibox / history casing does not disable Servo.
    #[test]
    fn servo_supersedes_accepts_uppercase_http_scheme() {
        assert!(servo_supersedes_dom_paint(false, "HTTP://example/"));
        assert!(servo_supersedes_dom_paint(false, "HTTPS://example/"));
    }

    #[test]
    fn servo_supersedes_rejects_empty_or_whitespace_url() {
        assert!(!servo_supersedes_dom_paint(false, ""));
        assert!(!servo_supersedes_dom_paint(false, "   "));
    }

    #[test]
    fn servo_supersedes_rejects_javascript_and_ftp() {
        assert!(!servo_supersedes_dom_paint(
            false,
            "javascript:alert(1)",
        ));
        assert!(!servo_supersedes_dom_paint(false, "ftp://files.example/x"));
    }
}
