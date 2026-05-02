//! UI strings for Tonet (browser chrome). Primary language: English; additional locales for international users.

use url::Url;

use crate::settings::{
    AppSettings, EnergySaverMode, SearchEngine, StartupPolicy, UpdatePolicy,
};

/// Active display locale resolved from settings + OS.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Locale {
    En,
    Es,
    De,
    Fr,
}

impl Locale {
    /// Map BCP 47 / POSIX tags (e.g. `en-US`, `es_ES`) to a supported locale.
    pub fn from_language_tag(tag: &str) -> Self {
        let primary = tag
            .split(&['-', '_'][..])
            .next()
            .unwrap_or("en")
            .to_ascii_lowercase();
        match primary.as_str() {
            "es" => Locale::Es,
            "de" => Locale::De,
            "fr" => Locale::Fr,
            _ => Locale::En,
        }
    }

    /// Best effort: OS / environment UI language.
    pub fn from_system() -> Self {
        sys_locale::get_locale()
            .map(|s| Self::from_language_tag(&s))
            .unwrap_or(Locale::En)
    }
}

/// Resolve locale from persisted `ui_language` (`auto`, `en`, `es`, `de`, `fr`).
pub fn effective_locale(settings: &AppSettings) -> Locale {
    match settings.ui_language.as_str() {
        "en" => Locale::En,
        "es" => Locale::Es,
        "de" => Locale::De,
        "fr" => Locale::Fr,
        _ => Locale::from_system(),
    }
}

// --- Toolbar & chrome ---

pub fn app_name(_: Locale) -> &'static str {
    "Tonet"
}

pub fn address_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Busca en Tonet o escribe una URL",
        Locale::De => "Tonet durchsuchen oder URL eingeben",
        Locale::Fr => "Rechercher dans Tonet ou saisir une URL",
        Locale::En => "Search Tonet or type a URL",
    }
}

/// Heading above omnibox visit-history autocomplete rows.
pub fn omnibox_history_heading(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Historial",
        Locale::De => "Verlauf",
        Locale::Fr => "Historique",
        Locale::En => "History",
    }
}

/// Keyboard shortcuts for visit-history rows under the omnibox (hover on “History” heading).
pub fn omnibox_history_keyboard_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "↑/↓: fila · Entrar: abrir · Esc: quitar resaltado",
        Locale::De => "↑/↓: Zeile · Eingabe: öffnen · Esc: Markierung aufheben",
        Locale::Fr => "↑/↓ : ligne · Entrée : ouvrir · Échap : effacer la surbrillance",
        Locale::En => "↑/↓: row · Enter: open · Esc: clear highlight",
    }
}

pub fn settings_tooltip(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Ajustes (Ctrl o ⌘ + coma)",
        Locale::De => "Einstellungen (Strg bzw. ⌘ + Komma)",
        Locale::Fr => "Paramètres (Ctrl ou ⌘ + virgule)",
        Locale::En => "Settings (Ctrl or ⌘ + comma)",
    }
}

pub fn back_tooltip(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Atrás",
        Locale::De => "Zurück",
        Locale::Fr => "Précédent",
        Locale::En => "Back",
    }
}

pub fn forward_tooltip(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Adelante",
        Locale::De => "Vor",
        Locale::Fr => "Suivant",
        Locale::En => "Forward",
    }
}

pub fn reload_tooltip(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Recargar",
        Locale::De => "Neu laden",
        Locale::Fr => "Actualiser",
        Locale::En => "Reload",
    }
}

pub fn stop_loading_tooltip(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Detener carga",
        Locale::De => "Laden abbrechen",
        Locale::Fr => "Arrêter le chargement",
        Locale::En => "Stop loading",
    }
}

/// Shown when Stop is disabled because Servo’s embedder API does not expose navigation cancel yet.
pub fn stop_loading_unavailable_servo_tooltip(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => {
            "Detener no está disponible con el motor Servo (la API aún no permite cancelar la carga)."
        }
        Locale::De => {
            "Abbrechen ist mit dem Servo-Motor nicht verfügbar (API erlaubt noch kein Abbruch des Ladens)."
        }
        Locale::Fr => {
            "Arrêt indisponible avec le moteur Servo (l’API ne permet pas encore d’interrompre le chargement)."
        }
        Locale::En => {
            "Stop isn’t available with the Servo engine yet (embedder API has no navigation cancel)."
        }
    }
}

pub fn reload_shortcuts_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "También: F5 o Ctrl/⌘+R",
        Locale::De => "Auch: F5 oder Strg/⌘+R",
        Locale::Fr => "Aussi : F5 ou Ctrl/⌘+R",
        Locale::En => "Also: F5 or Ctrl/⌘+R",
    }
}

pub fn omnibox_focus_shortcut_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Ctrl o ⌘ + L: enfocar y seleccionar la barra de direcciones",
        Locale::De => "Strg bzw. ⌘ + L: Adresszeile fokussieren und alles markieren",
        Locale::Fr => "Ctrl ou ⌘ + L : focus sur la barre d’adresse et tout sélectionner",
        Locale::En => "Ctrl or ⌘ + L: focus address bar and select all",
    }
}

pub fn tab_new_tooltip(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Nueva pestaña (Ctrl o ⌘ + T)",
        Locale::De => "Neuer Tab (Strg bzw. ⌘ + T)",
        Locale::Fr => "Nouvel onglet (Ctrl ou ⌘ + T)",
        Locale::En => "New tab (Ctrl or ⌘ + T)",
    }
}

pub fn tab_close_tooltip(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Cerrar pestaña (Ctrl o ⌘ + W)",
        Locale::De => "Tab schließen (Strg bzw. ⌘ + W)",
        Locale::Fr => "Fermer l’onglet (Ctrl ou ⌘ + W)",
        Locale::En => "Close tab (Ctrl or ⌘ + W)",
    }
}

pub fn tab_untitled(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Nueva pestaña",
        Locale::De => "Neuer Tab",
        Locale::Fr => "Nouvel onglet",
        Locale::En => "New tab",
    }
}

pub fn window_minimize(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Minimizar",
        Locale::De => "Minimieren",
        Locale::Fr => "Réduire",
        Locale::En => "Minimize",
    }
}

pub fn window_maximize(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Maximizar",
        Locale::De => "Maximieren",
        Locale::Fr => "Agrandir",
        Locale::En => "Maximize",
    }
}

pub fn window_restore(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Restaurar tamaño",
        Locale::De => "Wiederherstellen",
        Locale::Fr => "Restaurer",
        Locale::En => "Restore down",
    }
}

pub fn window_close(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Cerrar",
        Locale::De => "Schließen",
        Locale::Fr => "Fermer",
        Locale::En => "Close",
    }
}

pub fn window_drag_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Arrastra para mover la ventana (doble clic para maximizar o restaurar)",
        Locale::De => "Ziehen zum Verschieben (Doppelklick: maximieren/wiederherstellen)",
        Locale::Fr => "Glisser pour déplacer (double-clic : agrandir ou restaurer)",
        Locale::En => "Drag to move (double-click to maximize or restore)",
    }
}

pub fn security_chip_placeholder(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Seguridad",
        Locale::De => "Sicherheit",
        Locale::Fr => "Sécurité",
        Locale::En => "Security",
    }
}

/// Omnibox security line (`label`, `hover_tooltip`) from the current address text.
pub fn security_chip_pair(omnibox_url: &str, loc: Locale) -> (String, String) {
    let t = omnibox_url.trim();
    if t.is_empty() {
        return (
            security_chip_placeholder(loc).to_string(),
            security_chip_tooltip_idle(loc).to_string(),
        );
    }
    if let Ok(u) = Url::parse(t) {
        if u.scheme() == "tonet" {
            let host = u.host_str().unwrap_or("internal");
            return (
                format!("Tonet · {host}"),
                security_chip_tooltip_tonet(loc).to_string(),
            );
        }
        if u.scheme() == "https" {
            let host = u.host_str().unwrap_or("—");
            return (
                format!("HTTPS · {host}"),
                security_chip_tooltip_https(loc).to_string(),
            );
        }
        if u.scheme() == "http" {
            let host = u.host_str().unwrap_or("—");
            return (
                format!("HTTP · {host}"),
                security_chip_tooltip_http(loc).to_string(),
            );
        }
    }
    (
        security_chip_placeholder(loc).to_string(),
        security_chip_tooltip_invalid(loc).to_string(),
    )
}

fn security_chip_tooltip_tonet(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Página interna de Tonet (tonet://). No usa la red.",
        Locale::De => "Interne Tonet-Seite (tonet://). Kein Netzwerk.",
        Locale::Fr => "Page interne Tonet (tonet://). Pas de réseau.",
        Locale::En => "Tonet internal page (tonet://). No network access.",
    }
}

fn security_chip_tooltip_idle(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Escribe una URL para ver el esquema y el host.",
        Locale::De => "URL eingeben, um Schema und Host zu sehen.",
        Locale::Fr => "Saisissez une URL pour afficher le schéma et l’hôte.",
        Locale::En => "Enter a URL to show scheme and host.",
    }
}

fn security_chip_tooltip_https(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Conexión cifrada (HTTPS). Tonet no valida certificados todavía.",
        Locale::De => "Verschlüsselte Verbindung (HTTPS). Tonet prüft noch keine Zertifikate.",
        Locale::Fr => "Connexion chiffrée (HTTPS). Tonet ne vérifie pas encore les certificats.",
        Locale::En => "Encrypted connection (HTTPS). Tonet does not validate certificates yet.",
    }
}

fn security_chip_tooltip_http(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Conexión sin cifrar (HTTP). No introduzcas datos sensibles.",
        Locale::De => "Unverschlüsselte Verbindung (HTTP). Keine sensiblen Daten eingeben.",
        Locale::Fr => "Connexion non chiffrée (HTTP). Évitez les données sensibles.",
        Locale::En => "Unencrypted connection (HTTP). Do not enter sensitive data.",
    }
}

fn security_chip_tooltip_invalid(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "URL no válida o esquema no admitido (http, https o tonet).",
        Locale::De => "Ungültige URL oder Schema — http, https oder tonet.",
        Locale::Fr => "URL invalide ou schéma non pris en charge (http, https ou tonet).",
        Locale::En => "Invalid URL or unsupported scheme (http, https, or tonet).",
    }
}

// --- Errors & states ---

pub fn error_title(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Algo salió mal",
        Locale::De => "Etwas ist schiefgelaufen",
        Locale::Fr => "Un problème est survenu",
        Locale::En => "Something went wrong",
    }
}

pub fn loading_title(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Cargando…",
        Locale::De => "Wird geladen…",
        Locale::Fr => "Chargement…",
        Locale::En => "Loading…",
    }
}

pub fn loading_sub(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Tonet solo acepta páginas ligeras (≤ 1 MB).",
        Locale::De => "Tonet lädt nur leichte Seiten (≤ 1 MB).",
        Locale::Fr => "Tonet n’accepte que des pages légères (≤ 1 Mo).",
        Locale::En => "Tonet only loads lightweight pages (≤ 1 MB).",
    }
}


pub fn err_fetch_disconnected(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "La tarea de red finalizó de forma inesperada.",
        Locale::De => "Die Netzwerkaufgabe wurde unerwartet beendet.",
        Locale::Fr => "La tâche réseau s’est terminée de façon inattendue.",
        Locale::En => "The network task ended unexpectedly.",
    }
}

pub fn suggestion_fix_url(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Corrige la URL o prueba otra página ligera.",
        Locale::De => "URL anpassen oder eine leichtere Seite versuchen.",
        Locale::Fr => "Corrigez l’URL ou essayez une page plus légère.",
        Locale::En => "Fix the URL or try another lightweight page.",
    }
}

/// Best-effort localization of fetch-layer English errors (`network.rs` / anyhow).
pub fn localize_fetch_error(loc: Locale, msg: &str) -> String {
    let m = msg.trim();
    if m.contains("Only http and https URLs are allowed") {
        return match loc {
            Locale::Es => "Solo se admiten URL http y https.".to_string(),
            Locale::De => "Nur http- und https-URLs sind erlaubt.".to_string(),
            Locale::Fr => "Seules les URL http et https sont autorisées.".to_string(),
            Locale::En => "Only http and https URLs are allowed.".to_string(),
        };
    }
    if m.contains("Invalid URL:") || m.contains("relative URL without a base") {
        return match loc {
            Locale::Es => "La dirección no es una URL válida.".to_string(),
            Locale::De => "Die Eingabe ist keine gültige URL.".to_string(),
            Locale::Fr => "L’adresse n’est pas une URL valide.".to_string(),
            Locale::En => "The address is not a valid URL.".to_string(),
        };
    }
    if m.contains("Could not build HTTP client") {
        return match loc {
            Locale::Es => "No se pudo iniciar el cliente HTTP.".to_string(),
            Locale::De => "HTTP-Client konnte nicht erstellt werden.".to_string(),
            Locale::Fr => "Impossible de créer le client HTTP.".to_string(),
            Locale::En => "Could not start the HTTP client.".to_string(),
        };
    }
    if m.contains("Request failed for") {
        return match loc {
            Locale::Es => "Fallo de red al contactar el servidor.".to_string(),
            Locale::De => "Netzwerkfehler beim Kontaktieren des Servers.".to_string(),
            Locale::Fr => "Échec réseau en joignant le serveur.".to_string(),
            Locale::En => "Network error while contacting the server.".to_string(),
        };
    }
    if m.contains("HTTP error: server responded with") {
        return match loc {
            Locale::Es => "El servidor respondió con un código distinto de 200 OK.".to_string(),
            Locale::De => "Der Server antwortete nicht mit 200 OK.".to_string(),
            Locale::Fr => "Le serveur n’a pas répondu avec 200 OK.".to_string(),
            Locale::En => "The server did not respond with 200 OK.".to_string(),
        };
    }
    if m.contains("page too large") || m.contains("too large (over 1 MB)") {
        return match loc {
            Locale::Es => "Página demasiado grande (más de 1 MB).".to_string(),
            Locale::De => "Seite zu groß (über 1 MB).".to_string(),
            Locale::Fr => "Page trop volumineuse (plus de 1 Mo).".to_string(),
            Locale::En => "Page too large (over 1 MB).".to_string(),
        };
    }
    if m.contains("not valid UTF-8") || m.contains("Body is not valid UTF-8") {
        return match loc {
            Locale::Es => "El cuerpo de la página no es UTF-8 válido.".to_string(),
            Locale::De => "Der Seiteninhalt ist kein gültiges UTF-8.".to_string(),
            Locale::Fr => "Le corps de la page n’est pas en UTF-8 valide.".to_string(),
            Locale::En => "The page body is not valid UTF-8.".to_string(),
        };
    }
    if m.contains("timed out") || m.contains("timeout") {
        return match loc {
            Locale::Es => "Tiempo de espera agotado al cargar la página.".to_string(),
            Locale::De => "Zeitüberschreitung beim Laden der Seite.".to_string(),
            Locale::Fr => "Délai dépassé lors du chargement de la page.".to_string(),
            Locale::En => "Timed out while loading the page.".to_string(),
        };
    }
    msg.to_string()
}

// --- Update banner ---

pub fn update_banner_title(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Nueva versión disponible",
        Locale::De => "Neue Version verfügbar",
        Locale::Fr => "Nouvelle version disponible",
        Locale::En => "Update available",
    }
}

pub fn update_dismiss(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Ocultar",
        Locale::De => "Ausblenden",
        Locale::Fr => "Masquer",
        Locale::En => "Dismiss",
    }
}

pub fn update_download(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Descargar",
        Locale::De => "Herunterladen",
        Locale::Fr => "Télécharger",
        Locale::En => "Download",
    }
}

// --- Settings window ---

pub fn settings_window_title(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Ajustes",
        Locale::De => "Einstellungen",
        Locale::Fr => "Paramètres",
        Locale::En => "Settings",
    }
}

pub fn settings_section_language(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Idioma",
        Locale::De => "Sprache",
        Locale::Fr => "Langue",
        Locale::En => "Language",
    }
}

pub fn settings_section_updates(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Actualizaciones",
        Locale::De => "Aktualisierungen",
        Locale::Fr => "Mises à jour",
        Locale::En => "Updates",
    }
}

pub fn settings_section_search(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Buscador predeterminado",
        Locale::De => "Standard-Suchmaschine",
        Locale::Fr => "Moteur de recherche par défaut",
        Locale::En => "Default search engine",
    }
}

pub fn settings_search_help(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Se usa cuando escribes texto en la barra de direcciones o en la búsqueda de «Nueva pestaña» y no es una URL.",
        Locale::De => "Wird verwendet, wenn in Adresszeile oder „Neuer Tab“-Suche kein Link eingegeben wird.",
        Locale::Fr => "Utilisé lorsque la barre d’adresse ou la recherche « Nouvel onglet » ne contient pas une URL.",
        Locale::En => "Used when the address bar or New Tab search box contains text that is not a URL.",
    }
}

pub fn search_engine_label(_loc: Locale, engine: SearchEngine) -> &'static str {
    match engine {
        SearchEngine::Duckduckgo => "DuckDuckGo",
        SearchEngine::Google => "Google",
        SearchEngine::Brave => "Brave Search",
    }
}

pub fn search_engine_help(loc: Locale, engine: SearchEngine) -> &'static str {
    match engine {
        SearchEngine::Duckduckgo => match loc {
            Locale::Es => "Privacidad por defecto en duckduckgo.com.",
            Locale::De => "Datenschutzfokus auf duckduckgo.com.",
            Locale::Fr => "Orientation confidentialité sur duckduckgo.com.",
            Locale::En => "Privacy-focused results on duckduckgo.com.",
        },
        SearchEngine::Google => match loc {
            Locale::Es => "Búsqueda en google.com (sujeta a políticas de Google).",
            Locale::De => "Suche über google.com (unterliegt den Google-Richtlinien).",
            Locale::Fr => "Recherche via google.com (politique Google).",
            Locale::En => "Search via google.com (subject to Google’s policies).",
        },
        SearchEngine::Brave => match loc {
            Locale::Es => "Búsqueda en search.brave.com.",
            Locale::De => "Suche über search.brave.com.",
            Locale::Fr => "Recherche sur search.brave.com.",
            Locale::En => "Search via search.brave.com.",
        },
    }
}

pub fn settings_language_help(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "«Automático» sigue el idioma del sistema.",
        Locale::De => "„Automatisch“ verwendet die Systemsprache.",
        Locale::Fr => "« Auto » suit la langue du système.",
        Locale::En => "“Auto” follows your system language.",
    }
}

pub fn lang_option_auto(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Automático (sistema)",
        Locale::De => "Automatisch (System)",
        Locale::Fr => "Auto (système)",
        Locale::En => "Auto (system)",
    }
}

pub fn lang_option_en(_: Locale) -> &'static str {
    "English"
}

pub fn lang_option_es(_: Locale) -> &'static str {
    "Español"
}

pub fn lang_option_de(_: Locale) -> &'static str {
    "Deutsch"
}

pub fn lang_option_fr(_: Locale) -> &'static str {
    "Français"
}

pub fn installed_version(loc: Locale, ver: &str) -> String {
    match loc {
        Locale::Es => format!("Versión instalada: {ver}"),
        Locale::De => format!("Installierte Version: {ver}"),
        Locale::Fr => format!("Version installée : {ver}"),
        Locale::En => format!("Installed version: {ver}"),
    }
}

pub fn update_policy_question(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "¿Cuándo debe comprobar Tonet si hay versiones nuevas en el manifiesto de actualizaciones?",
        Locale::De => "Wann soll Tonet das Update-Manifest auf neue Versionen prüfen?",
        Locale::Fr => "Quand Tonet doit-il vérifier le manifeste de mises à jour pour de nouvelles versions ?",
        Locale::En => "When should Tonet check the update manifest for new releases?",
    }
}

pub fn update_policy_label(loc: Locale, policy: UpdatePolicy) -> &'static str {
    match (loc, policy) {
        (_, UpdatePolicy::OnStartup) => match loc {
            Locale::Es => "Solo al iniciar Tonet",
            Locale::De => "Nur beim Start",
            Locale::Fr => "Au lancement uniquement",
            Locale::En => "On startup only",
        },
        (_, UpdatePolicy::Periodic) => match loc {
            Locale::Es => "Automático (cada 24 h)",
            Locale::De => "Automatisch (alle 24 h)",
            Locale::Fr => "Automatique (toutes les 24 h)",
            Locale::En => "Automatic (every 24 hours)",
        },
        (_, UpdatePolicy::ManualOnly) => match loc {
            Locale::Es => "Solo manual",
            Locale::De => "Nur manuell",
            Locale::Fr => "Manuel seulement",
            Locale::En => "Manual only",
        },
    }
}

pub fn update_policy_help(loc: Locale, policy: UpdatePolicy) -> &'static str {
    match (loc, policy) {
        (_, UpdatePolicy::OnStartup) => match loc {
            Locale::Es => "Una comprobación al abrir la aplicación.",
            Locale::De => "Einmal prüfen, wenn die App startet.",
            Locale::Fr => "Une vérification à l’ouverture.",
            Locale::En => "Check once when you open the app.",
        },
        (_, UpdatePolicy::Periodic) => match loc {
            Locale::Es => "Al iniciar y cada 24 h mientras Tonet sigue abierto.",
            Locale::De => "Beim Start und alle 24 h, solange Tonet läuft.",
            Locale::Fr => "Au démarrage puis toutes les 24 h tant que Tonet est ouvert.",
            Locale::En => "On startup and every 24 hours while Tonet is open.",
        },
        (_, UpdatePolicy::ManualOnly) => match loc {
            Locale::Es => "No consultar el manifiesto hasta que pulses «Comprobar ahora».",
            Locale::De => "Keine Prüfung, bis du auf „Jetzt prüfen“ klickst.",
            Locale::Fr => "Pas de vérification tant que vous n’utilisez pas « Vérifier maintenant ».",
            Locale::En => "No checks until you press “Check now”.",
        },
    }
}

pub fn check_now(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Comprobar ahora",
        Locale::De => "Jetzt prüfen",
        Locale::Fr => "Vérifier maintenant",
        Locale::En => "Check now",
    }
}

pub fn checking(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Consultando…",
        Locale::De => "Wird geprüft…",
        Locale::Fr => "Vérification…",
        Locale::En => "Checking…",
    }
}

pub fn check_busy_hover(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Ya hay una comprobación en curso…",
        Locale::De => "Eine Prüfung läuft bereits…",
        Locale::Fr => "Une vérification est déjà en cours…",
        Locale::En => "A check is already in progress…",
    }
}

pub fn save_preferences(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Guardar preferencias",
        Locale::De => "Einstellungen speichern",
        Locale::Fr => "Enregistrer les préférences",
        Locale::En => "Save preferences",
    }
}

pub fn open_downloads_page(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Abrir página de descargas",
        Locale::De => "Download-Seite öffnen",
        Locale::Fr => "Ouvrir la page des téléchargements",
        Locale::En => "Open downloads page",
    }
}

pub fn open_downloads_tooltip(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Abre la página de descargas en el navegador del sistema",
        Locale::De => "Öffnet die Download-Seite im Standardbrowser",
        Locale::Fr => "Ouvre la page de téléchargement dans le navigateur",
        Locale::En => "Opens the downloads page in your default browser",
    }
}

// --- Dynamic update status ---

pub fn update_checking_github(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Consultando manifiesto de actualizaciones…",
        Locale::De => "Update-Manifest wird abgefragt…",
        Locale::Fr => "Interrogation du manifeste de mises à jour…",
        Locale::En => "Checking update manifest…",
    }
}

pub fn update_up_to_date(loc: Locale, ver: &str) -> String {
    match loc {
        Locale::Es => format!("Estás al día (v{ver})."),
        Locale::De => format!("Du bist auf dem neuesten Stand (v{ver})."),
        Locale::Fr => format!("Vous êtes à jour (v{ver})."),
        Locale::En => format!("You’re up to date (v{ver})."),
    }
}

pub fn update_new_version(loc: Locale, version: &str) -> String {
    match loc {
        Locale::Es => format!(
            "Nueva versión: v{version}. Usa «Descargar» o la página de releases."
        ),
        Locale::De => format!(
            "Neue Version: v{version}. „Herunterladen“ oder Releases-Seite nutzen."
        ),
        Locale::Fr => format!(
            "Nouvelle version : v{version}. Utilisez « Télécharger » ou la page des releases."
        ),
        Locale::En => format!(
            "New version: v{version}. Use “Download” or the releases page."
        ),
    }
}

pub fn update_check_failed(loc: Locale, err: &str) -> String {
    match loc {
        Locale::Es => format!("No se pudo comprobar: {err}"),
        Locale::De => format!("Prüfung fehlgeschlagen: {err}"),
        Locale::Fr => format!("Échec de la vérification : {err}"),
        Locale::En => format!("Could not check for updates: {err}"),
    }
}

pub fn update_interrupted(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "La comprobación de actualizaciones se interrumpió.",
        Locale::De => "Die Aktualisierungsprüfung wurde unterbrochen.",
        Locale::Fr => "La vérification des mises à jour a été interrompue.",
        Locale::En => "The update check was interrupted.",
    }
}

// --- New Tab page ---

pub fn new_tab_search_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Busca con Tonet o escribe una URL",
        Locale::De => "Suche mit Tonet oder gib eine URL ein",
        Locale::Fr => "Rechercher avec Tonet ou saisir une URL",
        Locale::En => "Search with Tonet or enter URL",
    }
}

pub fn new_tab_add_title(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Añadir acceso",
        Locale::De => "Verknüpfung hinzufügen",
        Locale::Fr => "Ajouter un raccourci",
        Locale::En => "Add shortcut",
    }
}

pub fn new_tab_add_intro(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "URL con http://, https:// o tonet://. Se guarda en tus preferencias.",
        Locale::De => "URL mit http://, https:// oder tonet://. Wird in den Einstellungen gespeichert.",
        Locale::Fr => "URL en http://, https:// ou tonet://. Enregistré dans vos préférences.",
        Locale::En => "Use http://, https://, or tonet://. Saved with your preferences.",
    }
}

pub fn new_tab_add_icon_label(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Icono (texto)",
        Locale::De => "Symbol (Text)",
        Locale::Fr => "Icône (texte)",
        Locale::En => "Icon (text)",
    }
}

pub fn new_tab_add_label_label(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Etiqueta",
        Locale::De => "Bezeichnung",
        Locale::Fr => "Libellé",
        Locale::En => "Label",
    }
}

pub fn new_tab_add_url_label(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "URL",
        Locale::De => "URL",
        Locale::Fr => "URL",
        Locale::En => "URL",
    }
}

pub fn new_tab_add_save(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Añadir",
        Locale::De => "Hinzufügen",
        Locale::Fr => "Ajouter",
        Locale::En => "Add",
    }
}

pub fn new_tab_add_cancel(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Cancelar",
        Locale::De => "Abbrechen",
        Locale::Fr => "Annuler",
        Locale::En => "Cancel",
    }
}

/// Primary action for Servo-driven `alert` / `confirm` / `prompt` windows.
#[cfg(all(feature = "servo-engine", windows))]
pub fn servo_dialog_ok(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Aceptar",
        Locale::De => "OK",
        Locale::Fr => "OK",
        Locale::En => "OK",
    }
}

/// Title for Servo context menu (`EmbedderControl::ContextMenu`).
#[cfg(all(feature = "servo-engine", windows))]
pub fn servo_context_menu_title(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Menú de página",
        Locale::De => "Seitenmenü",
        Locale::Fr => "Menu contextuel",
        Locale::En => "Page menu",
    }
}

/// Servo context menu: open the hit-tested link URL in a new Tonet tab (shell), not a second Servo WebView.
#[cfg(all(feature = "servo-engine", windows))]
pub fn servo_context_menu_open_link_new_tonet_tab(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Abrir enlace en nueva pestaña Tonet",
        Locale::De => "Link in neuem Tonet-Tab öffnen",
        Locale::Fr => "Ouvrir le lien dans un nouvel onglet Tonet",
        Locale::En => "Open link in new Tonet tab",
    }
}

#[cfg(all(feature = "servo-engine", windows))]
pub fn servo_select_title(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Elegir opción",
        Locale::De => "Option wählen",
        Locale::Fr => "Choisir une option",
        Locale::En => "Choose option",
    }
}

#[cfg(all(feature = "servo-engine", windows))]
pub fn servo_color_picker_title(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Color",
        Locale::De => "Farbe",
        Locale::Fr => "Couleur",
        Locale::En => "Color",
    }
}

#[cfg(all(feature = "servo-engine", windows))]
pub fn servo_color_picker_label(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Elegir:",
        Locale::De => "Wählen:",
        Locale::Fr => "Choisir :",
        Locale::En => "Pick:",
    }
}

/// Title for Servo `PermissionRequest` (geolocation, camera, etc.).
#[cfg(all(feature = "servo-engine", windows))]
pub fn servo_permission_title(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Permiso del sitio",
        Locale::De => "Website-Berechtigung",
        Locale::Fr => "Autorisation du site",
        Locale::En => "Site permission",
    }
}

#[cfg(all(feature = "servo-engine", windows))]
pub fn servo_permission_intro(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "La página quiere usar:",
        Locale::De => "Die Seite möchte nutzen:",
        Locale::Fr => "La page souhaite utiliser :",
        Locale::En => "This page wants to use:",
    }
}

#[cfg(all(feature = "servo-engine", windows))]
pub fn servo_permission_allow(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Permitir",
        Locale::De => "Erlauben",
        Locale::Fr => "Autoriser",
        Locale::En => "Allow",
    }
}

#[cfg(all(feature = "servo-engine", windows))]
pub fn servo_permission_deny(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Denegar",
        Locale::De => "Ablehnen",
        Locale::Fr => "Refuser",
        Locale::En => "Deny",
    }
}

/// Title for Servo `AuthenticationRequest` (HTTP 401 / proxy 407).
#[cfg(all(feature = "servo-engine", windows))]
pub fn servo_http_auth_title(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Credenciales del sitio",
        Locale::De => "Website-Anmeldung",
        Locale::Fr => "Identifiants du site",
        Locale::En => "Website login",
    }
}

#[cfg(all(feature = "servo-engine", windows))]
pub fn servo_http_auth_intro(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "El sitio pide un usuario y una contraseña:",
        Locale::De => "Die Website fordert Benutzername und Passwort:",
        Locale::Fr => "Le site demande un identifiant et un mot de passe :",
        Locale::En => "The site is asking for a username and password:",
    }
}

#[cfg(all(feature = "servo-engine", windows))]
pub fn servo_http_auth_proxy_note(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Autenticación del servidor proxy.",
        Locale::De => "Authentifizierung für den Proxy-Server.",
        Locale::Fr => "Authentification du serveur mandataire.",
        Locale::En => "This is for your proxy server.",
    }
}

#[cfg(all(feature = "servo-engine", windows))]
pub fn servo_http_auth_user_label(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Usuario",
        Locale::De => "Benutzername",
        Locale::Fr => "Nom d’utilisateur",
        Locale::En => "Username",
    }
}

#[cfg(all(feature = "servo-engine", windows))]
pub fn servo_http_auth_password_label(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Contraseña",
        Locale::De => "Passwort",
        Locale::Fr => "Mot de passe",
        Locale::En => "Password",
    }
}

#[cfg(all(feature = "servo-engine", windows))]
pub fn servo_http_auth_sign_in(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Entrar",
        Locale::De => "Anmelden",
        Locale::Fr => "Se connecter",
        Locale::En => "Sign in",
    }
}

#[cfg(all(feature = "servo-engine", windows))]
pub fn servo_notification_fallback_title(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Notificación",
        Locale::De => "Benachrichtigung",
        Locale::Fr => "Notification",
        Locale::En => "Notification",
    }
}

#[cfg(all(feature = "servo-engine", windows))]
pub fn servo_notification_dismiss(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Cerrar",
        Locale::De => "Schließen",
        Locale::Fr => "Fermer",
        Locale::En => "Dismiss",
    }
}

#[cfg(all(feature = "servo-engine", windows))]
pub fn servo_page_console_header(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Consola de la página (Servo)",
        Locale::De => "Seitenkonsole (Servo)",
        Locale::Fr => "Console de la page (Servo)",
        Locale::En => "Page console (Servo)",
    }
}

#[cfg(all(feature = "servo-engine", windows))]
pub fn servo_page_console_clear(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Vaciar",
        Locale::De => "Leeren",
        Locale::Fr => "Effacer",
        Locale::En => "Clear",
    }
}

/// Localized label for [`servo::PermissionFeature`] in the permission modal.
#[cfg(all(feature = "servo-engine", windows))]
pub fn servo_permission_feature_name(loc: Locale, f: servo::PermissionFeature) -> &'static str {
    use servo::PermissionFeature as P;
    match f {
        P::Geolocation => match loc {
            Locale::Es => "Geolocalización",
            Locale::De => "Standort",
            Locale::Fr => "Géolocalisation",
            Locale::En => "Geolocation",
        },
        P::Notifications => match loc {
            Locale::Es => "Notificaciones",
            Locale::De => "Benachrichtigungen",
            Locale::Fr => "Notifications",
            Locale::En => "Notifications",
        },
        P::Push => match loc {
            Locale::Es => "Push",
            Locale::De => "Push",
            Locale::Fr => "Push",
            Locale::En => "Push",
        },
        P::Midi => match loc {
            Locale::Es => "MIDI",
            Locale::De => "MIDI",
            Locale::Fr => "MIDI",
            Locale::En => "MIDI",
        },
        P::Camera => match loc {
            Locale::Es => "Cámara",
            Locale::De => "Kamera",
            Locale::Fr => "Caméra",
            Locale::En => "Camera",
        },
        P::Microphone => match loc {
            Locale::Es => "Micrófono",
            Locale::De => "Mikrofon",
            Locale::Fr => "Microphone",
            Locale::En => "Microphone",
        },
        P::Speaker => match loc {
            Locale::Es => "Altavoz",
            Locale::De => "Lautsprecher",
            Locale::Fr => "Haut-parleur",
            Locale::En => "Speaker",
        },
        P::DeviceInfo => match loc {
            Locale::Es => "Información del dispositivo",
            Locale::De => "Geräteinformationen",
            Locale::Fr => "Infos sur l’appareil",
            Locale::En => "Device info",
        },
        P::BackgroundSync => match loc {
            Locale::Es => "Sincronización en segundo plano",
            Locale::De => "Hintergrundsynchronisation",
            Locale::Fr => "Synchro en arrière-plan",
            Locale::En => "Background sync",
        },
        P::Bluetooth => match loc {
            Locale::Es => "Bluetooth",
            Locale::De => "Bluetooth",
            Locale::Fr => "Bluetooth",
            Locale::En => "Bluetooth",
        },
        P::PersistentStorage => match loc {
            Locale::Es => "Almacenamiento persistente",
            Locale::De => "Dauerhafte Speicherung",
            Locale::Fr => "Stockage persistant",
            Locale::En => "Persistent storage",
        },
    }
}

pub fn new_tab_add_url_invalid(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "La URL debe empezar por http://, https:// o tonet://",
        Locale::De => "URL muss mit http://, https:// oder tonet:// beginnen",
        Locale::Fr => "L’URL doit commencer par http://, https:// ou tonet://",
        Locale::En => "URL must start with http://, https://, or tonet://",
    }
}

pub fn new_tab_add_max_tiles(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Límite de accesos en la página de nueva pestaña (24).",
        Locale::De => "Limit für Kacheln auf der neuen Registerkarte (24).",
        Locale::Fr => "Limite de raccourcis sur la page nouvel onglet (24).",
        Locale::En => "New Tab shortcut limit reached (24).",
    }
}

pub fn new_tab_add_tile_label(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Añadir acceso",
        Locale::De => "Verknüpfung",
        Locale::Fr => "Ajouter",
        Locale::En => "Add shortcut",
    }
}

pub fn new_tab_add_tile_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Añadir un acceso a esta cuadrícula",
        Locale::De => "Eine Verknüpfung zu dieser Kachelansicht hinzufügen",
        Locale::Fr => "Ajouter un raccourci à cette grille",
        Locale::En => "Add a shortcut to this grid",
    }
}

pub fn new_tab_remove(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Quitar de nueva pestaña",
        Locale::De => "Von neuer Registerkarte entfernen",
        Locale::Fr => "Retirer du nouvel onglet",
        Locale::En => "Remove from New Tab",
    }
}

// --- Renderer ---

pub fn empty_page_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "No se encontró contenido reconocible (<title>, <h1>, <h2>, <p>, enlaces).",
        Locale::De => "Kein erkanntes Inhalts-Markup (<title>, <h1>, <h2>, <p>, Links).",
        Locale::Fr => "Aucun contenu reconnu (<title>, <h1>, <h2>, <p>, liens).",
        Locale::En => "No recognizable content found (<title>, <h1>, <h2>, <p>, links).",
    }
}

// --- tonet:// internal pages ---

pub fn internal_tab_title_settings(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Ajustes",
        Locale::De => "Einstellungen",
        Locale::Fr => "Paramètres",
        Locale::En => "Settings",
    }
}

pub fn internal_tab_title_downloads(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Descargas",
        Locale::De => "Downloads",
        Locale::Fr => "Téléchargements",
        Locale::En => "Downloads",
    }
}

pub fn internal_tab_title_history(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Historial",
        Locale::De => "Verlauf",
        Locale::Fr => "Historique",
        Locale::En => "History",
    }
}

pub fn internal_nav_history(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Historial",
        Locale::De => "Verlauf",
        Locale::Fr => "Historique",
        Locale::En => "History",
    }
}

pub fn internal_nav_downloads(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Descargas",
        Locale::De => "Downloads",
        Locale::Fr => "Téléchargements",
        Locale::En => "Downloads",
    }
}

pub fn internal_nav_settings(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Ajustes",
        Locale::De => "Einstellungen",
        Locale::Fr => "Paramètres",
        Locale::En => "Settings",
    }
}

pub fn internal_search_history(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Buscar en el historial",
        Locale::De => "Verlauf durchsuchen",
        Locale::Fr => "Rechercher dans l’historique",
        Locale::En => "Search history",
    }
}

pub fn internal_search_downloads(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Buscar descargas",
        Locale::De => "Downloads durchsuchen",
        Locale::Fr => "Rechercher des téléchargements",
        Locale::En => "Search downloads",
    }
}

pub fn internal_clear_all(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Borrar todo",
        Locale::De => "Alle löschen",
        Locale::Fr => "Tout effacer",
        Locale::En => "Clear all",
    }
}

pub fn internal_history_sidebar(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Historial de Tonet",
        Locale::De => "Tonet-Verlauf",
        Locale::Fr => "Historique Tonet",
        Locale::En => "Tonet history",
    }
}

pub fn internal_history_other_devices(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Pestañas de otros dispositivos",
        Locale::De => "Tabs von anderen Geräten",
        Locale::Fr => "Onglets d’autres appareils",
        Locale::En => "Tabs from other devices",
    }
}

pub fn internal_history_other_devices_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Aún no disponible.",
        Locale::De => "Noch nicht verfügbar.",
        Locale::Fr => "Pas encore disponible.",
        Locale::En => "Not available yet.",
    }
}

pub fn internal_history_delete_data(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Borrar datos de navegación…",
        Locale::De => "Browserdaten löschen…",
        Locale::Fr => "Effacer les données de navigation…",
        Locale::En => "Delete browsing data…",
    }
}

pub fn internal_confirm_clear_history(loc: Locale) -> &'static str {
    #[cfg(all(feature = "servo-engine", windows))]
    {
        match loc {
            Locale::Es => "¿Borrar todo el historial de visitas guardado en este dispositivo? También se eliminan los permisos de sitio guardados del visor experimental Servo (archivo en la carpeta de configuración).",
            Locale::De => "Den gesamten gespeicherten Verlauf auf diesem Gerät löschen? Gespeicherte Website-Berechtigungen des experimentellen Servo-Viewports werden ebenfalls entfernt (Datei im Konfigurationsordner).",
            Locale::Fr => "Supprimer tout l’historique de navigation enregistré sur cet appareil ? Les autorisations de site enregistrées pour la vue Servo expérimentale seront aussi supprimées (fichier du dossier de configuration).",
            Locale::En => "Clear all saved visit history on this device? This also removes stored Servo experimental viewport site permissions (config folder file).",
        }
    }
    #[cfg(not(all(feature = "servo-engine", windows)))]
    {
        match loc {
            Locale::Es => "¿Borrar todo el historial de visitas guardado en este dispositivo?",
            Locale::De => "Den gesamten gespeicherten Verlauf auf diesem Gerät löschen?",
            Locale::Fr => "Supprimer tout l’historique de navigation enregistré sur cet appareil ?",
            Locale::En => "Clear all saved visit history on this device?",
        }
    }
}

pub fn internal_confirm_clear_downloads(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => {
            "¿Borrar la lista de descargas? (solo el registro en Tonet; las instantáneas HTML guardadas no se eliminan.)"
        }
        Locale::De => {
            "Download-Liste löschen? (nur das Tonet-Protokoll; gespeicherte HTML-Momentaufnahmen bleiben.)"
        }
        Locale::Fr => {
            "Effacer la liste des téléchargements ? (journal Tonet seul ; les instantanés HTML conservés restent sur le disque.)"
        }
        Locale::En => {
            "Clear the downloads list? (Tonet log only; saved HTML snapshots on disk are not deleted.)"
        }
    }
}

pub fn internal_btn_cancel(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Cancelar",
        Locale::De => "Abbrechen",
        Locale::Fr => "Annuler",
        Locale::En => "Cancel",
    }
}

pub fn internal_btn_clear(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Borrar",
        Locale::De => "Löschen",
        Locale::Fr => "Effacer",
        Locale::En => "Clear",
    }
}

pub fn internal_hist_today(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Hoy",
        Locale::De => "Heute",
        Locale::Fr => "Aujourd’hui",
        Locale::En => "Today",
    }
}

pub fn internal_hist_yesterday(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Ayer",
        Locale::De => "Gestern",
        Locale::Fr => "Hier",
        Locale::En => "Yesterday",
    }
}

pub fn internal_from_url(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Desde",
        Locale::De => "Von",
        Locale::Fr => "Depuis",
        Locale::En => "From",
    }
}

pub fn internal_copy_link(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Copiar enlace",
        Locale::De => "Link kopieren",
        Locale::Fr => "Copier le lien",
        Locale::En => "Copy link",
    }
}

pub fn internal_open_folder(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Abrir carpeta",
        Locale::De => "Ordner öffnen",
        Locale::Fr => "Ouvrir le dossier",
        Locale::En => "Open folder",
    }
}

pub fn internal_open_folder_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => {
            "Abre el HTML guardado en el gestor de archivos cuando esta visita tuvo instantánea en disco."
        }
        Locale::De => {
            "Öffnet die gespeicherte HTML-Datei im Dateimanager, falls für diesen Eintrag eine Momentaufnahme existiert."
        }
        Locale::Fr => {
            "Ouvre le fichier HTML enregistré dans le gestionnaire de fichiers lorsqu’un instantané existe pour cette entrée."
        }
        Locale::En => {
            "Opens the saved HTML snapshot in the file manager when this row has a file on disk."
        }
    }
}

pub fn internal_saved_snapshot_label(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Instantánea:",
        Locale::De => "Momentaufnahme:",
        Locale::Fr => "Instantané :",
        Locale::En => "Snapshot:",
    }
}

pub fn internal_remove_row(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Quitar",
        Locale::De => "Entfernen",
        Locale::Fr => "Retirer",
        Locale::En => "Remove",
    }
}

pub fn internal_open_in_tonet(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Abrir en Tonet",
        Locale::De => "In Tonet öffnen",
        Locale::Fr => "Ouvrir dans Tonet",
        Locale::En => "Open in Tonet",
    }
}

pub fn internal_remove_selected(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Quitar selección",
        Locale::De => "Auswahl entfernen",
        Locale::Fr => "Supprimer la sélection",
        Locale::En => "Remove selected",
    }
}

pub fn internal_settings_intro(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Preferencias de Tonet. Los cambios se guardan al pulsar «Guardar preferencias».",
        Locale::De => "Tonet-Einstellungen. Änderungen mit „Einstellungen speichern“ sichern.",
        Locale::Fr => "Préférences Tonet. Enregistrez avec « Enregistrer les préférences ».",
        Locale::En => "Tonet preferences. Use “Save preferences” to persist changes.",
    }
}

pub fn internal_settings_nav_general(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Inicio",
        Locale::De => "Start",
        Locale::Fr => "Démarrer",
        Locale::En => "Get started",
    }
}

pub fn internal_settings_nav_search(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Buscador",
        Locale::De => "Suche",
        Locale::Fr => "Moteur de recherche",
        Locale::En => "Search engine",
    }
}

pub fn internal_settings_nav_updates(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Actualizaciones",
        Locale::De => "Updates",
        Locale::Fr => "Mises à jour",
        Locale::En => "Updates",
    }
}

pub fn internal_downloads_intro(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => {
            "Páginas cargadas recientemente (registro). El HTML de cada visita HTTP(S) se puede guardar bajo «Descargas efectivas»/Tonet/page-snapshots cuando hay carpeta de destino válida."
        }
        Locale::De => {
            "Kürzlich geladene Seiten (Protokoll). HTML jeder HTTP(S)-Ladung kann unter dem effektiven Download-Ordner in Tonet/page-snapshots gespeichert werden, sofern ein gültiger Zielordner existiert."
        }
        Locale::Fr => {
            "Pages récemment chargées (journal). Le HTML de chaque chargement HTTP(S) peut être enregistré sous le dossier de téléchargement effectif, dans Tonet/page-snapshots, si un dossier cible valide est disponible."
        }
        Locale::En => {
            "Recently loaded pages (log). Each HTTP(S) visit can save HTML under your effective download folder in Tonet/page-snapshots when a valid destination directory is available."
        }
    }
}

pub fn internal_no_items(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "No hay elementos.",
        Locale::De => "Keine Einträge.",
        Locale::Fr => "Aucun élément.",
        Locale::En => "No items yet.",
    }
}

pub fn internal_settings_get_started_heading(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Primeros pasos",
        Locale::De => "Erste Schritte",
        Locale::Fr => "Pour commencer",
        Locale::En => "Get started",
    }
}

pub fn internal_settings_profile_row(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Nombre e icono del perfil",
        Locale::De => "Profilname und -symbol",
        Locale::Fr => "Nom et icône du profil",
        Locale::En => "Profile name and icon",
    }
}

pub fn internal_settings_profile_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Próximamente.",
        Locale::De => "Demnächst.",
        Locale::Fr => "Bientôt disponible.",
        Locale::En => "Coming soon.",
    }
}

pub fn internal_settings_startup_heading(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Al iniciar",
        Locale::De => "Beim Start",
        Locale::Fr => "Au démarrage",
        Locale::En => "On startup",
    }
}

pub fn internal_settings_startup_urls_label(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Páginas al iniciar (una por línea)",
        Locale::De => "Seiten beim Start (eine pro Zeile)",
        Locale::Fr => "Pages au démarrage (une par ligne)",
        Locale::En => "Pages to open on startup (one per line)",
    }
}

pub fn internal_settings_startup_urls_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => {
            "URLs completas, tonet://… o texto de búsqueda. Guarda las preferencias al terminar."
        }
        Locale::De => {
            "Volle URLs, tonet://… oder Suchtext. Änderungen mit „Einstellungen speichern“ sichern."
        }
        Locale::Fr => {
            "URL complètes, tonet://… ou texte de recherche. Enregistrez avec « Enregistrer les préférences »."
        }
        Locale::En => {
            "Full URLs, tonet://…, or search text. Use “Save preferences” after editing."
        }
    }
}

pub fn internal_settings_startup_urls_placeholder(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "https://example.org\ntonet://settings\nduck",
        Locale::De => "https://example.org\ntonet://settings\nduck",
        Locale::Fr => "https://example.org\ntonet://settings\nduck",
        Locale::En => "https://example.org\ntonet://settings\nduck",
    }
}

pub fn internal_settings_startup_label(loc: Locale, pol: StartupPolicy) -> &'static str {
    match (loc, pol) {
        (Locale::Es, StartupPolicy::NewTabPage) => "Abrir la página Nueva pestaña",
        (Locale::De, StartupPolicy::NewTabPage) => "Neue-Tab-Seite öffnen",
        (Locale::Fr, StartupPolicy::NewTabPage) => "Ouvrir la page Nouvel onglet",
        (Locale::En, StartupPolicy::NewTabPage) => "Open the New Tab page",
        (Locale::Es, StartupPolicy::RestoreSession) => "Continuar donde lo dejaste",
        (Locale::De, StartupPolicy::RestoreSession) => "Weiter dort, wo du aufgehört hast",
        (Locale::Fr, StartupPolicy::RestoreSession) => "Reprendre où vous vous étiez arrêté",
        (Locale::En, StartupPolicy::RestoreSession) => "Continue where you left off",
        (Locale::Es, StartupPolicy::OpenSpecificPages) => "Abrir una página o conjunto específico",
        (Locale::De, StartupPolicy::OpenSpecificPages) => "Bestimmte Seiten öffnen",
        (Locale::Fr, StartupPolicy::OpenSpecificPages) => "Ouvrir une ou plusieurs pages",
        (Locale::En, StartupPolicy::OpenSpecificPages) => "Open a specific page or set of pages",
    }
}

pub fn internal_settings_startup_help(loc: Locale, pol: StartupPolicy) -> &'static str {
    match (loc, pol) {
        (Locale::Es, StartupPolicy::NewTabPage) => "Página de inicio predeterminada de Tonet.",
        (Locale::De, StartupPolicy::NewTabPage) => "Tonet-Standard-Startseite.",
        (Locale::Fr, StartupPolicy::NewTabPage) => "Page d’accueil par défaut de Tonet.",
        (Locale::En, StartupPolicy::NewTabPage) => "Tonet’s default home experience.",
        (Locale::Es, StartupPolicy::RestoreSession) => {
            "Reabre las mismas pestañas y la pestaña activa de la última vez que cerraste Tonet."
        }
        (Locale::De, StartupPolicy::RestoreSession) => {
            "Öffnet dieselben Tabs und den aktiven Tab wie beim letzten Schließen von Tonet."
        }
        (Locale::Fr, StartupPolicy::RestoreSession) => {
            "Rouvre les mêmes onglets et l’onglet actif de la dernière fermeture de Tonet."
        }
        (Locale::En, StartupPolicy::RestoreSession) => {
            "Reopens the same tabs and active tab from when you last closed Tonet."
        }
        (Locale::Es, StartupPolicy::OpenSpecificPages) => {
            "Abre cada línea no vacía como pestaña (mismas reglas que la barra de direcciones)."
        }
        (Locale::De, StartupPolicy::OpenSpecificPages) => {
            "Öffnet jede nicht leere Zeile als Tab (gleiche Regeln wie die Adressleiste)."
        }
        (Locale::Fr, StartupPolicy::OpenSpecificPages) => {
            "Ouvre chaque ligne non vide comme onglet (mêmes règles que la barre d’adresse)."
        }
        (Locale::En, StartupPolicy::OpenSpecificPages) => {
            "Opens each non-empty line as a tab (same rules as the address bar)."
        }
    }
}

pub fn internal_settings_system_heading(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Sistema",
        Locale::De => "System",
        Locale::Fr => "Système",
        Locale::En => "System",
    }
}

pub fn internal_settings_shortcuts_row(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Atajos",
        Locale::De => "Tastenkürzel",
        Locale::Fr => "Raccourcis",
        Locale::En => "Shortcuts",
    }
}

pub fn internal_settings_shortcuts_row_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Ver la lista de referencia de atajos de teclado.",
        Locale::De => "Tastenkürzel-Referenzliste anzeigen.",
        Locale::Fr => "Voir la liste de référence des raccourcis clavier.",
        Locale::En => "View the keyboard shortcut reference list.",
    }
}

pub fn internal_settings_bg_apps(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Seguir ejecutando apps en segundo plano al cerrar Tonet",
        Locale::De => "Apps im Hintergrund weiterlaufen lassen, wenn Tonet geschlossen ist",
        Locale::Fr => "Continuer à exécuter des applications en arrière-plan",
        Locale::En => "Continue running background apps when Tonet is closed",
    }
}

pub fn internal_settings_hw_accel(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Usar aceleración por hardware cuando esté disponible",
        Locale::De => "Hardwarebeschleunigung verwenden, falls verfügbar",
        Locale::Fr => "Utiliser l’accélération matérielle si disponible",
        Locale::En => "Use hardware acceleration when available",
    }
}

#[cfg_attr(not(feature = "servo-engine"), allow(dead_code))]
pub fn internal_settings_servo_heading(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Motor Servo",
        Locale::De => "Servo-Engine",
        Locale::Fr => "Moteur Servo",
        Locale::En => "Servo engine",
    }
}

#[cfg_attr(not(feature = "servo-engine"), allow(dead_code))]
pub fn internal_settings_servo_body(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => {
            "En Windows, las páginas http(s) se renderizan solo con Servo (ventana nativa GL/surfman \
             sobre el lienzo de Tonet). Las rutas tonet:// y la página nueva siguen con el motor \
             Tonet. La integración tipo Slint (mismo wgpu que la UI) está prevista como evolución."
        }
        Locale::De => {
            "Unter Windows werden http(s)-Seiten nur mit Servo gerendert ( natives GL/surfman-Fenster \
             über der Tonet-Oberfläche). tonet://-Routen und die neue Registerkarte nutzen weiter Tonet. \
             Eine Slint-ähnliche Einbettung (gemeinsames wgpu mit der UI) ist als nächster Schritt geplant."
        }
        Locale::Fr => {
            "Sous Windows, les pages http(s) sont rendues uniquement avec Servo (fenêtre native GL/surfman \
             au-dessus de Tonet). Les routes tonet:// et la page nouvel onglet restent sur le moteur Tonet. \
             Une intégration façon Slint (wgpu partagé avec l’UI) est l’évolution prévue."
        }
        Locale::En => {
            "On Windows, http(s) pages render with Servo only (native GL/surfman window over Tonet). \
             tonet:// routes and the new-tab page still use the Tonet stack. A Slint-style embed \
             (shared wgpu with the UI) is the planned next step."
        }
    }
}

/// Windows + `servo-engine`: Servo is always on for http(s); this line explains opt-out.
#[cfg(all(feature = "servo-engine", windows))]
pub fn internal_settings_servo_windows_note(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => {
            "En esta compilación, http(s) usa Servo por defecto. Para desactivarlo temporalmente: \
             variable de entorno TONET_SERVO_VIEWPORT=0 (vuelve el motor Tonet limitado para la web)."
        }
        Locale::De => {
            "In diesem Build nutzt http(s) standardmäßig Servo. Zum vorübergehenden Abschalten: \
             Umgebungsvariable TONET_SERVO_VIEWPORT=0 (dann wieder der eingeschränkte Tonet-Webstack)."
        }
        Locale::Fr => {
            "Dans cette version, http(s) utilise Servo par défaut. Pour le désactiver temporairement : \
             variable d’environnement TONET_SERVO_VIEWPORT=0 (retour au moteur Tonet limité pour le web)."
        }
        Locale::En => {
            "In this build, http(s) uses Servo by default. To turn it off temporarily, set environment \
             variable TONET_SERVO_VIEWPORT=0 (Tonet’s limited in-process web stack is used instead)."
        }
    }
}

#[cfg_attr(not(feature = "servo-engine"), allow(dead_code))]
#[cfg_attr(all(feature = "servo-engine", windows), allow(dead_code))]
pub fn internal_settings_servo_viewport(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Activar la ruta de viewport Servo (solo fuera de Windows)",
        Locale::De => "Servo-Viewport aktivieren (nur außerhalb von Windows)",
        Locale::Fr => "Activer le viewport Servo (hors Windows seulement)",
        Locale::En => "Enable Servo viewport path (non-Windows only)",
    }
}

#[cfg(all(feature = "servo-engine", windows))]
pub fn internal_settings_servo_clear_permissions(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Borrar permisos de sitio Servo guardados",
        Locale::De => "Gespeicherte Servo-Website-Berechtigungen löschen",
        Locale::Fr => "Effacer les autorisations de site Servo enregistrées",
        Locale::En => "Clear saved Servo site permissions",
    }
}

#[cfg(all(feature = "servo-engine", windows))]
pub fn internal_settings_servo_clear_permissions_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => {
            "Elimina el archivo de permisos y la memoria en esta sesión. No borra el historial de visitas."
        }
        Locale::De => {
            "Entfernt die Berechtigungsdatei und den Arbeitsspeicher in dieser Sitzung. Der Besuchsverlauf bleibt erhalten."
        }
        Locale::Fr => {
            "Supprime le fichier d’autorisations et la mémoire de session. L’historique de navigation n’est pas effacé."
        }
        Locale::En => {
            "Removes the permissions file and in-session memory. Visit history is not cleared."
        }
    }
}

#[cfg_attr(not(feature = "servo-engine"), allow(dead_code))]
#[cfg_attr(all(feature = "servo-engine", windows), allow(dead_code))]
pub fn servo_compiled_activate_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => {
            "Este binario incluye Servo, pero el viewport nativo no está activo en esta plataforma. \
             Actívalo en Ajustes → Sistema o con TONET_SERVO_VIEWPORT=1. En Windows con Servo, http(s) \
             ya usa el motor Servo por defecto."
        }
        Locale::De => {
            "Dieses Binary enthält Servo, aber der native Viewport ist auf dieser Plattform nicht aktiv. \
             Aktivieren unter Einstellungen → System oder TONET_SERVO_VIEWPORT=1. Unter Windows mit \
             Servo nutzt http(s) standardmäßig bereits Servo."
        }
        Locale::Fr => {
            "Ce binaire inclut Servo, mais le viewport natif n’est pas actif sur cette plateforme. \
             Activez-le dans Réglages → Système ou avec TONET_SERVO_VIEWPORT=1. Sous Windows avec \
             Servo, http(s) utilise déjà Servo par défaut."
        }
        Locale::En => {
            "This build includes Servo, but the native viewport is not active on this platform. \
             Enable it under Settings → System, or set TONET_SERVO_VIEWPORT=1. On Windows with \
             Servo enabled, http(s) already uses Servo by default."
        }
    }
}

/// Shown in the content area when `TONET_SERVO_VIEWPORT=0` on Windows (Tonet stack fallback).
#[cfg(all(feature = "servo-engine", windows))]
pub fn servo_windows_engine_disabled_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => {
            "Servo está desactivado por TONET_SERVO_VIEWPORT=0. Esta URL se intenta mostrar con el \
             motor Tonet (HTML/CSS limitado), no con Servo."
        }
        Locale::De => {
            "Servo ist durch TONET_SERVO_VIEWPORT=0 deaktiviert. Diese URL wird mit dem eingeschränkten \
             Tonet-Stack (begrenztes HTML/CSS) statt Servo dargestellt."
        }
        Locale::Fr => {
            "Servo est désactivé par TONET_SERVO_VIEWPORT=0. Cette URL est affichée avec le moteur Tonet \
             (HTML/CSS limité), pas avec Servo."
        }
        Locale::En => {
            "Servo is turned off (TONET_SERVO_VIEWPORT=0). This URL is shown with Tonet’s in-process \
             engine (limited HTML/CSS), not Servo."
        }
    }
}

pub fn internal_settings_proxy(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Abrir la configuración de proxy del sistema",
        Locale::De => "System-Proxy-Einstellungen öffnen",
        Locale::Fr => "Ouvrir les paramètres proxy du système",
        Locale::En => "Open your computer’s proxy settings",
    }
}

pub fn internal_settings_open_system(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Abrir sistema",
        Locale::De => "System öffnen",
        Locale::Fr => "Ouvrir",
        Locale::En => "Open",
    }
}

pub fn internal_settings_close_last_tab(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Cerrar la ventana al cerrar la última pestaña",
        Locale::De => "Fenster schließen, wenn der letzte Tab geschlossen wird",
        Locale::Fr => "Fermer la fenêtre en fermant le dernier onglet",
        Locale::En => "Close window when closing last tab",
    }
}

pub fn internal_settings_warn_close(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Advertir antes de cerrar la ventana con varias pestañas",
        Locale::De => "Warnen, bevor ein Fenster mit mehreren Tabs geschlossen wird",
        Locale::Fr => "Avertir avant de fermer une fenêtre avec plusieurs onglets",
        Locale::En => "Warn me before closing window with multiple tabs",
    }
}

pub fn internal_settings_fullscreen_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Mostrar recordatorio de pantalla completa (Esc)",
        Locale::De => "Vollbild-Hinweis (Esc) anzeigen",
        Locale::Fr => "Rappel plein écran (Échap)",
        Locale::En => "Show full screen reminder to press Esc on exit",
    }
}

pub fn internal_settings_vpn_heading(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "VPN (vista previa)",
        Locale::De => "VPN (Vorschau)",
        Locale::Fr => "VPN (aperçu)",
        Locale::En => "VPN (preview)",
    }
}

pub fn internal_settings_vpn_wireguard(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Usar protocolo WireGuard en VPN",
        Locale::De => "WireGuard-Protokoll in der VPN verwenden",
        Locale::Fr => "Utiliser le protocole WireGuard pour le VPN",
        Locale::En => "Use WireGuard protocol in VPN",
    }
}

pub fn internal_settings_vpn_tray(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Mostrar icono de VPN en la bandeja",
        Locale::De => "VPN-Symbol in der Taskleiste anzeigen",
        Locale::Fr => "Afficher l’icône VPN dans la barre des tâches",
        Locale::En => "Show VPN tray icon",
    }
}

pub fn internal_settings_vpn_tray_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Cuando está activo, la barra de tareas puede mostrar el icono de VPN.",
        Locale::De => "Wenn aktiviert, erscheint ein VPN-Symbol in der Taskleiste.",
        Locale::Fr => "Si activé, une icône VPN peut apparaître dans la barre des tâches.",
        Locale::En => "When enabled, the taskbar may show the VPN tray icon.",
    }
}

pub fn internal_settings_memory_heading(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Memoria",
        Locale::De => "Speicher",
        Locale::Fr => "Mémoire",
        Locale::En => "Memory",
    }
}

pub fn internal_settings_memory_body(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Tonet puede liberar memoria de pestañas inactivas (cuando esté disponible).",
        Locale::De => "Tonet kann Speicher inaktiver Tabs freigeben (wenn verfügbar).",
        Locale::Fr => "Tonet peut libérer la mémoire des onglets inactifs (lorsque disponible).",
        Locale::En => "Tonet can free memory from inactive tabs when the feature is available.",
    }
}

pub fn internal_settings_memory_saver(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Ahorro de memoria",
        Locale::De => "Speicher sparen",
        Locale::Fr => "Économiseur de mémoire",
        Locale::En => "Memory saver",
    }
}

pub fn internal_settings_keep_sites(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Mantener siempre activos estos sitios",
        Locale::De => "Diese Websites immer aktiv halten",
        Locale::Fr => "Toujours garder ces sites actifs",
        Locale::En => "Always keep these sites active",
    }
}

pub fn internal_settings_keep_sites_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Los sitios que añadas no entrarán en suspensión.",
        Locale::De => "Hinzugefügte Websites werden nicht ausgelagert.",
        Locale::Fr => "Les sites ajoutés ne seront pas mis en veille.",
        Locale::En => "Sites you add will not be suspended.",
    }
}

pub fn internal_settings_add(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Añadir",
        Locale::De => "Hinzufügen",
        Locale::Fr => "Ajouter",
        Locale::En => "Add",
    }
}

pub fn internal_settings_no_sites(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "No hay sitios añadidos",
        Locale::De => "Keine Websites hinzugefügt",
        Locale::Fr => "Aucun site ajouté",
        Locale::En => "No sites added",
    }
}

pub fn internal_settings_power_heading(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Energía",
        Locale::De => "Energie",
        Locale::Fr => "Énergie",
        Locale::En => "Power",
    }
}

pub fn internal_settings_power_body(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Tonet puede reducir actividad en segundo plano para ahorrar batería (cuando esté disponible).",
        Locale::De => "Tonet kann Hintergrundaktivität reduzieren (wenn verfügbar).",
        Locale::Fr => "Tonet peut limiter l’activité en arrière-plan pour économiser la batterie.",
        Locale::En => "Tonet can limit background activity to save battery when available.",
    }
}

pub fn internal_settings_energy_saver(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Ahorro de energía",
        Locale::De => "Energiesparmodus",
        Locale::Fr => "Économiseur d’énergie",
        Locale::En => "Energy saver",
    }
}

pub fn internal_settings_energy_mode(loc: Locale, m: EnergySaverMode) -> &'static str {
    match (loc, m) {
        (Locale::Es, EnergySaverMode::WhenBatteryLow) => {
            "Activar solo cuando la batería esté al 20% o menos"
        }
        (Locale::De, EnergySaverMode::WhenBatteryLow) => {
            "Nur aktivieren, wenn der Akku 20 % oder weniger hat"
        }
        (Locale::Fr, EnergySaverMode::WhenBatteryLow) => {
            "Activer seulement lorsque la batterie est à 20 % ou moins"
        }
        (Locale::En, EnergySaverMode::WhenBatteryLow) => {
            "Turn on only when your battery is at 20% or lower"
        }
        (Locale::Es, EnergySaverMode::WhenUnplugged) => "Activar cuando el equipo no esté enchufado",
        (Locale::De, EnergySaverMode::WhenUnplugged) => "Aktivieren, wenn nicht am Netz",
        (Locale::Fr, EnergySaverMode::WhenUnplugged) => "Activer sur batterie",
        (Locale::En, EnergySaverMode::WhenUnplugged) => "Turn on when your computer is unplugged",
    }
}

pub fn internal_settings_shortcuts_back(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Atrás",
        Locale::De => "Zurück",
        Locale::Fr => "Retour",
        Locale::En => "Back",
    }
}

pub fn internal_settings_shortcuts_title(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Atajos de teclado",
        Locale::De => "Tastenkürzel",
        Locale::Fr => "Raccourcis clavier",
        Locale::En => "Keyboard shortcuts",
    }
}

pub fn internal_settings_shortcuts_search_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Busca un comando o atajo.",
        Locale::De => "Nach Befehl oder Kürzel suchen.",
        Locale::Fr => "Rechercher une commande ou un raccourci.",
        Locale::En => "Search for a command or shortcut.",
    }
}

pub fn internal_settings_shortcuts_filter_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Filtrar…",
        Locale::De => "Filtern…",
        Locale::Fr => "Filtrer…",
        Locale::En => "Filter…",
    }
}

pub fn internal_settings_add_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Cambiar combinaciones de teclado: aún no disponible (solo referencia).",
        Locale::De => "Tastenkürzel umbelegen: noch nicht verfügbar (nur Referenz).",
        Locale::Fr => "Modifier les raccourcis clavier : pas encore disponible (référence seule).",
        Locale::En => "Rebinding keys is not available yet (reference list only).",
    }
}

pub fn internal_settings_shortcuts_footer_note(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => {
            "Lista de referencia tipo Chromium para atajos de teclado; esos enlaces no se personalizan aquí. Los accesos de la página «Nueva pestaña» sí se guardan en preferencias."
        }
        Locale::De => {
            "Chromium-Referenz für Tastenkürzel; hier nicht editierbar. Kacheln auf der Seite „Neuer Tab“ werden dagegen in den Einstellungen gespeichert."
        }
        Locale::Fr => {
            "Liste de référence Chromium pour les raccourcis clavier, non modifiable ici. Les tuiles de la page nouvel onglet sont enregistrées dans les préférences."
        }
        Locale::En => {
            "Chromium-style keyboard shortcut reference (not editable here). New Tab page tiles are saved in your preferences."
        }
    }
}

pub fn internal_settings_reset_heading(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Restablecer Tonet",
        Locale::De => "Tonet zurücksetzen",
        Locale::Fr => "Réinitialiser Tonet",
        Locale::En => "Reset Tonet",
    }
}

pub fn internal_settings_reset_body(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Esto restaura preferencias, motor de búsqueda y opciones del sistema a los valores predeterminados.",
        Locale::De => "Setzt Einstellungen, Suchmaschine und Systemoptionen auf Standardwerte zurück.",
        Locale::Fr => "Rétablit les préférences, le moteur de recherche et les options système par défaut.",
        Locale::En => "Restores preferences, search engine, and system options to their defaults.",
    }
}

pub fn internal_settings_reset_button(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Restablecer ahora",
        Locale::De => "Jetzt zurücksetzen",
        Locale::Fr => "Réinitialiser maintenant",
        Locale::En => "Reset now",
    }
}

pub fn internal_settings_reset_confirm_title(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "¿Restablecer todos los ajustes?",
        Locale::De => "Alle Einstellungen zurücksetzen?",
        Locale::Fr => "Réinitialiser tous les paramètres ?",
        Locale::En => "Reset all settings?",
    }
}

pub fn internal_settings_reset_confirm_body(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Esta acción no se puede deshacer. ¿Continuar?",
        Locale::De => "Dies kann nicht rückgängig gemacht werden. Fortfahren?",
        Locale::Fr => "Cette action est irréversible. Continuer ?",
        Locale::En => "This cannot be undone. Continue?",
    }
}

pub fn internal_settings_reset_confirm(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Restablecer",
        Locale::De => "Zurücksetzen",
        Locale::Fr => "Réinitialiser",
        Locale::En => "Reset",
    }
}

macro_rules! stub_settings_page {
    ($fn_title:ident, $fn_body:ident, $es_t:literal, $de_t:literal, $fr_t:literal, $en_t:literal, $es_b:literal, $de_b:literal, $fr_b:literal, $en_b:literal) => {
        pub fn $fn_title(loc: Locale) -> &'static str {
            match loc {
                Locale::Es => $es_t,
                Locale::De => $de_t,
                Locale::Fr => $fr_t,
                Locale::En => $en_t,
            }
        }
        pub fn $fn_body(loc: Locale) -> &'static str {
            match loc {
                Locale::Es => $es_b,
                Locale::De => $de_b,
                Locale::Fr => $fr_b,
                Locale::En => $en_b,
            }
        }
    };
}

pub fn internal_settings_appearance_heading(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Aspecto",
        Locale::De => "Darstellung",
        Locale::Fr => "Apparence",
        Locale::En => "Appearance",
    }
}

pub fn internal_settings_appearance_intro(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => {
            "Elige si Tonet usa una paleta oscura o clara. Pulsa Guardar preferencias para conservarla."
        }
        Locale::De => {
            "Wähle, ob Tonet eine dunkle oder helle Palette verwendet. Zum Speichern auf Einstellungen speichern klicken."
        }
        Locale::Fr => {
            "Choisissez si Tonet utilise une palette sombre ou claire. Cliquez sur Enregistrer les préférences pour conserver."
        }
        Locale::En => {
            "Choose whether Tonet uses a dark or light palette. Click Save preferences to persist."
        }
    }
}

pub fn internal_settings_theme_dark(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Oscuro",
        Locale::De => "Dunkel",
        Locale::Fr => "Sombre",
        Locale::En => "Dark",
    }
}

pub fn internal_settings_theme_light(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Claro",
        Locale::De => "Hell",
        Locale::Fr => "Clair",
        Locale::En => "Light",
    }
}

stub_settings_page!(
    internal_settings_content_title,
    internal_settings_content_body,
    "Contenido",
    "Inhalt",
    "Contenu",
    "Content",
    "PDF, fuentes y contenido incrustado: próximamente.",
    "PDF, Schriftarten, eingebettete Inhalte: demnächst.",
    "PDF, polices et contenu intégré : bientôt.",
    "PDFs, fonts, and embedded content: coming soon."
);

stub_settings_page!(
    internal_settings_privacy_filters_title,
    internal_settings_privacy_filters_body,
    "Filtros de privacidad",
    "Datenschutzfilter",
    "Filtres de confidentialité",
    "Privacy filters",
    "Bloqueo de rastreadores y anuncios: próximamente.",
    "Tracker- und Werbeblockierung: demnächst.",
    "Blocage des traqueurs et publicités : bientôt.",
    "Tracker and ad blocking: coming soon."
);

stub_settings_page!(
    internal_settings_privacy_title,
    internal_settings_privacy_body,
    "Privacidad y seguridad",
    "Datenschutz und Sicherheit",
    "Confidentialité et sécurité",
    "Privacy and security",
    "Cookies, permisos del sitio y datos: próximamente.",
    "Cookies, Website-Berechtigungen: demnächst.",
    "Cookies, autorisations et données : bientôt.",
    "Cookies, site permissions, and data: coming soon."
);

stub_settings_page!(
    internal_settings_onchain_title,
    internal_settings_onchain_body,
    "Aplicaciones on-chain",
    "On-Chain-Apps",
    "Applications on-chain",
    "On-chain apps",
    "Cartera y dApps: no disponible en esta versión.",
    "Wallet und dApps: in dieser Version nicht verfügbar.",
    "Portefeuille et dApps : non disponible pour l’instant.",
    "Wallet and dApps: not available in this build."
);

stub_settings_page!(
    internal_settings_assistant_title,
    internal_settings_assistant_body,
    "Asistente",
    "Assistent",
    "Assistant",
    "Assistant",
    "Asistente de IA: no integrado aún.",
    "KI-Assistent: noch nicht integriert.",
    "Assistant IA : pas encore intégré.",
    "AI assistant: not integrated yet."
);

stub_settings_page!(
    internal_settings_sync_title,
    internal_settings_sync_body,
    "Sincronización",
    "Synchronisation",
    "Synchronisation",
    "Sync",
    "Sincronización entre dispositivos: próximamente.",
    "Gerätesynchronisation: demnächst.",
    "Synchronisation multi-appareils : bientôt.",
    "Cross-device sync: coming soon."
);

stub_settings_page!(
    internal_settings_extensions_title,
    internal_settings_extensions_body,
    "Extensiones",
    "Erweiterungen",
    "Extensions",
    "Extensions",
    "Las extensiones del navegador no están soportadas aún.",
    "Browser-Erweiterungen werden noch nicht unterstützt.",
    "Les extensions ne sont pas encore prises en charge.",
    "Browser extensions are not supported yet."
);

stub_settings_page!(
    internal_settings_autofill_title,
    internal_settings_autofill_body,
    "Autocompletar y contraseñas",
    "Autofill und Passwörter",
    "Saisie automatique et mots de passe",
    "Autofill and passwords",
    "Gestor de contraseñas integrado: próximamente.",
    "Integrierte Passwortverwaltung: demnächst.",
    "Gestionnaire de mots de passe intégré : bientôt.",
    "Built-in password manager: coming soon."
);

pub fn internal_settings_dl_prefs_title(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Descargas",
        Locale::De => "Downloads",
        Locale::Fr => "Téléchargements",
        Locale::En => "Downloads",
    }
}

pub fn internal_settings_dl_prefs_intro(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => {
            "Elige una carpeta existente como destino preferido cuando Tonet guarde archivos. Si la ruta no es válida, se usa la carpeta de descargas del sistema. Pulsa «Guardar preferencias» abajo para persistir los cambios."
        }
        Locale::De => {
            "Wähle einen vorhandenen Ordner als bevorzugtes Ziel, wenn Tonet Dateien speichert. Ungültige Pfade fallen auf den System-Download-Ordner zurück. Zum Speichern unten auf «Einstellungen speichern» klicken."
        }
        Locale::Fr => {
            "Choisissez un dossier existant comme emplacement préféré lorsque Tonet enregistre des fichiers. Si le chemin est invalide, le dossier Téléchargements du système est utilisé. Utilisez « Enregistrer les préférences » en bas pour conserver les changements."
        }
        Locale::En => {
            "Pick an existing folder as the preferred destination when Tonet saves files to disk. If the path is invalid, the system Downloads folder is used. Use Save preferences below to persist changes."
        }
    }
}

pub fn internal_settings_dl_effective_label(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Ubicación efectiva",
        Locale::De => "Aktiver Speicherort",
        Locale::Fr => "Emplacement effectif",
        Locale::En => "Effective location",
    }
}

pub fn internal_settings_dl_path_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Carpeta personalizada (opcional; vacío = carpeta del sistema)",
        Locale::De => "Benutzerdefinierter Ordner (optional; leer = Systemstandard)",
        Locale::Fr => "Dossier personnalisé (facultatif ; vide = dossier système)",
        Locale::En => "Custom folder path (optional; empty = system default)",
    }
}

pub fn internal_settings_dl_use_system_default(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Usar carpeta del sistema",
        Locale::De => "Systemstandard verwenden",
        Locale::Fr => "Utiliser le dossier système",
        Locale::En => "Use system default",
    }
}

pub fn internal_settings_dl_open_folder(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Abrir en el explorador",
        Locale::De => "Im Dateimanager öffnen",
        Locale::Fr => "Ouvrir dans le gestionnaire de fichiers",
        Locale::En => "Open in file manager",
    }
}

pub fn internal_settings_dl_invalid_override(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => {
            "La ruta personalizada no existe o no es una carpeta; se usa la ubicación del sistema."
        }
        Locale::De => {
            "Der benutzerdefinierte Pfad existiert nicht oder ist kein Ordner; es wird der Systemort verwendet."
        }
        Locale::Fr => {
            "Le chemin personnalisé est absent ou n’est pas un dossier ; l’emplacement système est utilisé."
        }
        Locale::En => {
            "The custom path is missing or not a folder; using the system location instead."
        }
    }
}

pub fn internal_settings_a11y_heading(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Accesibilidad",
        Locale::De => "Bedienungshilfen",
        Locale::Fr => "Accessibilité",
        Locale::En => "Accessibility",
    }
}

pub fn internal_settings_a11y_intro(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => {
            "Ajusta el tamaño de la interfaz respecto a la escala nativa de la ventana. Pulsa Guardar preferencias para conservar."
        }
        Locale::De => {
            "Skaliert die Oberfläche relativ zur nativen Fensterskala. Zum Speichern auf Einstellungen speichern klicken."
        }
        Locale::Fr => {
            "Ajuste l’interface par rapport à l’échelle native de la fenêtre. Cliquez sur Enregistrer les préférences pour conserver."
        }
        Locale::En => {
            "Scales the interface on top of the window’s native DPI. Click Save preferences to persist."
        }
    }
}

pub fn internal_settings_ui_scale_label(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Escala de la interfaz",
        Locale::De => "Oberflächenskalierung",
        Locale::Fr => "Échelle de l’interface",
        Locale::En => "Interface scale",
    }
}

pub fn internal_settings_ui_scale_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Rango: 75 % a 200 %. El valor 1,0× equivale al tamaño predeterminado del sistema.",
        Locale::De => "Bereich 75 % bis 200 %. 1,0× entspricht der Systemvorgabe.",
        Locale::Fr => "Plage 75 % à 200 %. 1,0× correspond au réglage système.",
        Locale::En => "Range 75%–200%. 1.0× matches the default system size.",
    }
}

pub fn internal_settings_ui_scale_reset(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Restablecer a 100 %",
        Locale::De => "Auf 100 % zurücksetzen",
        Locale::Fr => "Réinitialiser à 100 %",
        Locale::En => "Reset to 100%",
    }
}
