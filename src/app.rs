//! Aplicación principal de Tonet: estado, hilos de red y coordinación con egui.
//!
//! La descarga HTTP se ejecuta en un hilo aparte para no bloquear el bucle de UI.

use std::sync::mpsc;

use eframe::egui;
use egui::ViewportCommand;

use crate::parser::{parse_html, DomNode, DomNodeType};
use crate::network::fetch_url;
use crate::renderer::render_nodes;
use crate::ui::{show_error_panel, show_loading, show_top_bar};

/// Resultado que viaja por el canal desde el hilo de red (errores como texto legible).
type FetchResult = Result<Vec<DomNode>, String>;

/// Estado global de la ventana Tonet.
pub struct TonetApp {
    /// Texto del campo de dirección.
    url_input: String,
    /// Descarga en curso.
    loading: bool,
    /// Mensaje de error visible (red, filtros, red, etc.).
    error_message: Option<String>,
    /// Nodos parseados listos para renderizar.
    dom: Vec<DomNode>,
    /// Título mostrado en la barra de ventana nativa.
    window_title: String,
    /// Receptor de resultados asíncronos (None si no hay trabajo en curso).
    fetch_rx: Option<mpsc::Receiver<FetchResult>>,
}

impl TonetApp {
    /// Crea la app con valores por defecto acordes al sitio público del proyecto.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            url_input: "https://usetonet.com".to_string(),
            loading: false,
            error_message: None,
            dom: Vec::new(),
            window_title: "Tonet".to_string(),
            fetch_rx: None,
        }
    }

    /// Inicia la descarga en segundo plano si la URL no está vacía.
    fn start_fetch(&mut self) {
        let trimmed = self.url_input.trim().to_string();
        if trimmed.is_empty() {
            self.error_message = Some("Escribe una URL válida (http o https).".to_string());
            self.dom.clear();
            return;
        }

        self.loading = true;
        self.error_message = None;
        self.dom.clear();

        let (tx, rx) = mpsc::channel();
        self.fetch_rx = Some(rx);

        std::thread::spawn(move || {
            let outcome: FetchResult = (|| {
                let html = fetch_url(&trimmed).map_err(|e| e.to_string())?;
                Ok(parse_html(&html))
            })();
            let _ = tx.send(outcome);
        });
    }

    /// Aplica el título de documento a la ventana si existe un nodo Title.
    fn sync_window_title(&mut self, ctx: &egui::Context) {
        let doc_title = self
            .dom
            .iter()
            .find(|n| n.kind == DomNodeType::Title)
            .map(|n| n.text.as_str());

        let new_title = doc_title.unwrap_or("Tonet");
        if new_title != self.window_title {
            self.window_title = new_title.to_string();
            ctx.send_viewport_cmd(ViewportCommand::Title(self.window_title.clone()));
        }
    }

    /// Drena el canal de resultados sin bloquear el fotograma.
    fn poll_fetch(&mut self, ctx: &egui::Context) {
        let Some(rx) = &self.fetch_rx else {
            return;
        };

        match rx.try_recv() {
            Ok(Ok(nodes)) => {
                self.dom = nodes;
                self.loading = false;
                self.fetch_rx = None;
                self.sync_window_title(ctx);
                ctx.request_repaint();
            }
            Ok(Err(msg)) => {
                self.error_message = Some(msg);
                self.loading = false;
                self.fetch_rx = None;
                ctx.send_viewport_cmd(ViewportCommand::Title("Tonet".into()));
                self.window_title = "Tonet".to_string();
                ctx.request_repaint();
            }
            Err(mpsc::TryRecvError::Empty) => {
                ctx.request_repaint();
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.loading = false;
                self.fetch_rx = None;
                self.error_message = Some("La tarea de red finalizó de forma inesperada.".to_string());
                ctx.request_repaint();
            }
        }
    }
}

impl eframe::App for TonetApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_fetch(ctx);

        egui::TopBottomPanel::top("tonet_top").show(ctx, |ui| {
            ui.add_space(4.0);
            if show_top_bar(ui, &mut self.url_input, self.loading) {
                self.start_fetch();
            }
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Errores de red o del Filtro de Pureza: panel destacado encima del scroll.
            if let Some(err) = &self.error_message {
                show_error_panel(ui, err);
                ui.add_space(8.0);
            }

            // Mientras llega la respuesta HTTP, mostramos un estado claro sin bloquear el bucle.
            if self.loading {
                show_loading(ui);
            }

            // Contenido principal: siempre en ScrollArea para páginas largas.
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if !self.loading && self.error_message.is_none() {
                        render_nodes(ui, &self.dom);
                    } else if !self.loading && self.error_message.is_some() {
                        ui.label(
                            egui::RichText::new("Corrige la URL o prueba otra página ligera.")
                                .italics()
                                .color(egui::Color32::GRAY),
                        );
                    }
                });
        });
    }
}
