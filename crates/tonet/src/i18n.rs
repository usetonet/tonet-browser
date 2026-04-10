//! UI strings for Tonet (browser chrome). Primary language: English; additional locales for international users.

use crate::settings::{AppSettings, UpdatePolicy};

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

pub fn go(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Ir",
        Locale::De => "Los",
        Locale::Fr => "Aller",
        Locale::En => "Go",
    }
}

pub fn go_loading(_: Locale) -> &'static str {
    "…"
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

pub fn security_chip_placeholder(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Seguridad",
        Locale::De => "Sicherheit",
        Locale::Fr => "Sécurité",
        Locale::En => "Security",
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

pub fn err_empty_url(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Escribe una URL válida (http o https).",
        Locale::De => "Bitte eine gültige http(s)-URL eingeben.",
        Locale::Fr => "Saisissez une URL http ou https valide.",
        Locale::En => "Enter a valid http or https URL.",
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

// --- Renderer ---

pub fn empty_page_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "No se encontró contenido reconocible (<title>, <h1>, <h2>, <p>).",
        Locale::De => "Kein erkanntes Inhalts-Markup (<title>, <h1>, <h2>, <p>).",
        Locale::Fr => "Aucun contenu reconnu (<title>, <h1>, <h2>, <p>).",
        Locale::En => "No recognizable content found (<title>, <h1>, <h2>, <p>).",
    }
}
