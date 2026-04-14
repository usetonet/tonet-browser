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
        Locale::Es => "https://ejemplo.com",
        Locale::De => "https://beispiel.de",
        Locale::Fr => "https://exemple.fr",
        Locale::En => "https://example.com",
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
        Locale::Es => "¿Cuándo debe comprobar Tonet si hay versiones nuevas en GitHub?",
        Locale::De => "Wann soll Tonet auf neue Versionen auf GitHub prüfen?",
        Locale::Fr => "Quand Tonet doit-il vérifier les nouvelles versions sur GitHub ?",
        Locale::En => "When should Tonet check GitHub for new releases?",
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
            Locale::Es => "No consultar GitHub hasta que pulses «Comprobar ahora».",
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
        Locale::Es => "Abre el navegador del sistema en GitHub Releases",
        Locale::De => "Öffnet GitHub Releases im Standardbrowser",
        Locale::Fr => "Ouvre GitHub Releases dans le navigateur",
        Locale::En => "Opens GitHub Releases in your default browser",
    }
}

// --- Dynamic update status ---

pub fn update_checking_github(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Consultando GitHub…",
        Locale::De => "GitHub wird abgefragt…",
        Locale::Fr => "Interrogation de GitHub…",
        Locale::En => "Contacting GitHub…",
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
    match loc {
        Locale::Es => "¿Borrar todo el historial de visitas guardado en este dispositivo?",
        Locale::De => "Den gesamten gespeicherten Verlauf auf diesem Gerät löschen?",
        Locale::Fr => "Supprimer tout l’historique de navigation enregistré sur cet appareil ?",
        Locale::En => "Clear all saved visit history on this device?",
    }
}

pub fn internal_confirm_clear_downloads(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "¿Borrar la lista de descargas (solo registro, no archivos en disco)?",
        Locale::De => "Download-Liste löschen? (nur Protokoll, keine Dateien auf der Festplatte)",
        Locale::Fr => "Effacer la liste des téléchargements (journal seulement, pas les fichiers) ?",
        Locale::En => "Clear the downloads list? (log only; no files on disk are removed.)",
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
        Locale::Es => "Tonet aún no guarda archivos en disco desde esta lista.",
        Locale::De => "Tonet speichert aus dieser Liste noch keine Dateien auf der Festplatte.",
        Locale::Fr => "Tonet n’enregistre pas encore de fichiers sur le disque depuis cette liste.",
        Locale::En => "Tonet does not save files to disk from this list yet.",
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
        Locale::Es => "Páginas cargadas recientemente (registro). No es un gestor de archivos descargados todavía.",
        Locale::De => "Kürzlich geladene Seiten (Protokoll). Noch kein klassischer Datei-Download-Manager.",
        Locale::Fr => "Pages récemment chargées (journal). Pas encore un gestionnaire de fichiers.",
        Locale::En => "Recently loaded pages (log). Not a classic file download manager yet.",
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
        (Locale::Es, StartupPolicy::RestoreSession) => "Aún no restaura pestañas; se guardará en una versión futura.",
        (Locale::De, StartupPolicy::RestoreSession) => "Sitzungswiederherstellung folgt in einer späteren Version.",
        (Locale::Fr, StartupPolicy::RestoreSession) => "La restauration de session arrive dans une version ultérieure.",
        (Locale::En, StartupPolicy::RestoreSession) => "Session restore is not implemented yet.",
        (Locale::Es, StartupPolicy::OpenSpecificPages) => "Aún no disponible.",
        (Locale::De, StartupPolicy::OpenSpecificPages) => "Noch nicht verfügbar.",
        (Locale::Fr, StartupPolicy::OpenSpecificPages) => "Pas encore disponible.",
        (Locale::En, StartupPolicy::OpenSpecificPages) => "Not available yet.",
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
        Locale::Es => "Ver y personalizar atajos de teclado.",
        Locale::De => "Tastenkürzel anzeigen und anpassen.",
        Locale::Fr => "Afficher et personnaliser les raccourcis.",
        Locale::En => "View and customize keyboard shortcuts.",
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
        Locale::Es => "La personalización de atajos llegará en una versión futura.",
        Locale::De => "Anpassbare Tastenkürzel kommen in einer späteren Version.",
        Locale::Fr => "La personnalisation des raccourcis arrive dans une version ultérieure.",
        Locale::En => "Custom shortcut editing is not available yet.",
    }
}

pub fn internal_settings_shortcuts_footer_note(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => {
            "Lista de referencia tipo Chromium; Tonet aún no guarda atajos personalizados. El botón «Añadir» está reservado."
        }
        Locale::De => {
            "Chromium-Referenzliste; benutzerdefinierte Kürzel werden in Tonet noch nicht gespeichert. „Hinzufügen“ ist vorbehalten."
        }
        Locale::Fr => {
            "Liste de référence de type Chromium ; les raccourcis personnalisés ne sont pas encore enregistrés. Le bouton « Ajouter » est réservé."
        }
        Locale::En => {
            "Chromium-style reference list; Tonet does not store custom shortcuts yet. The Add button is reserved for a future release."
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

stub_settings_page!(
    internal_settings_appearance_title,
    internal_settings_appearance_body,
    "Aspecto",
    "Darstellung",
    "Apparence",
    "Appearance",
    "Temas y modo claro/oscuro: próximamente.",
    "Themes und Hell/Dunkel: demnächst.",
    "Thèmes et mode clair/sombre : bientôt.",
    "Themes and light/dark mode: coming soon."
);

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
    internal_settings_shields_title,
    internal_settings_shields_body,
    "Escudos",
    "Shields",
    "Boucliers",
    "Shields",
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
    internal_settings_web3_title,
    internal_settings_web3_body,
    "Web3",
    "Web3",
    "Web3",
    "Web3",
    "Cartera y dApps: no disponible en esta versión.",
    "Wallet und dApps: in dieser Version nicht verfügbar.",
    "Portefeuille et dApps : non disponible pour l’instant.",
    "Wallet and dApps: not available in this build."
);

stub_settings_page!(
    internal_settings_leo_title,
    internal_settings_leo_body,
    "Leo",
    "Leo",
    "Leo",
    "Leo",
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

stub_settings_page!(
    internal_settings_dl_prefs_title,
    internal_settings_dl_prefs_body,
    "Descargas",
    "Downloads",
    "Téléchargements",
    "Downloads",
    "Carpeta de descarga y comportamiento: próximamente.",
    "Download-Ordner und Verhalten: demnächst.",
    "Dossier et comportement des téléchargements : bientôt.",
    "Download folder and behavior: coming soon."
);

stub_settings_page!(
    internal_settings_a11y_title,
    internal_settings_a11y_body,
    "Accesibilidad",
    "Bedienungshilfen",
    "Accessibilité",
    "Accessibility",
    "Opciones de accesibilidad: próximamente.",
    "Barrierefreiheit: demnächst.",
    "Options d’accessibilité : bientôt.",
    "Accessibility options: coming soon."
);
