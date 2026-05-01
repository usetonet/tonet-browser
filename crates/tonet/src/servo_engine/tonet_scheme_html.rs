//! HTML documents for internal `tonet://` pages when the active tab is rendered by Servo.
#![cfg(all(feature = "servo-engine", windows))]

use url::Url;

use crate::browser_log::{DownloadRecord, VisitRecord};
use crate::i18n::Locale;
use crate::internal_pages::{self, InternalRoute, SettingsNav, settings_nav_from_path, settings_page_url};
use crate::settings::{AppSettings, SearchEngine, UiTheme, UpdatePolicy};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TonetSchemeAction {
    ClearHistory,
    ClearDownloads,
    ClearServoSitePermissions,
}

/// Snapshot cloned from the shell each frame so [`super::runtime_win::TonetServoWebViewDelegate`]
/// can synthesize `tonet://` responses without touching egui state.
#[derive(Clone)]
pub(crate) struct TonetSchemeSharedState {
    pub loc: Locale,
    pub settings: AppSettings,
    pub visits: Vec<VisitRecord>,
    pub downloads: Vec<DownloadRecord>,
    pub pending_actions: Vec<TonetSchemeAction>,
}

impl Default for TonetSchemeSharedState {
    fn default() -> Self {
        Self {
            loc: Locale::En,
            settings: AppSettings::default(),
            visits: Vec::new(),
            downloads: Vec::new(),
            pending_actions: Vec::new(),
        }
    }
}

fn esc_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

fn wrap_doc(title: &str, nav_html: &str, body_html: &str) -> Vec<u8> {
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>{title}</title>
<style>
:root {{ color-scheme: dark; }}
body {{ font-family: system-ui, Segoe UI, Roboto, sans-serif; margin: 0; background: #141414; color: #e8e8e8; }}
.layout {{ display: flex; min-height: 100vh; }}
nav {{ width: 220px; flex-shrink: 0; padding: 16px 12px; border-right: 1px solid #2a2a2a; background: #1a1a1a; }}
nav a {{ display: block; padding: 6px 10px; border-radius: 6px; color: #9ecbff; text-decoration: none; font-size: 14px; }}
nav a:hover {{ background: #252525; }}
nav a.sel {{ background: #2d3d52; color: #fff; }}
main {{ flex: 1; padding: 20px 28px; max-width: 900px; }}
h1 {{ font-size: 22px; margin: 0 0 12px; }}
p.hint {{ color: #a0a0a0; font-size: 13px; line-height: 1.45; margin: 0 0 18px; }}
table {{ width: 100%; border-collapse: collapse; font-size: 13px; }}
th, td {{ text-align: left; padding: 8px 10px; border-bottom: 1px solid #2a2a2a; vertical-align: top; }}
th {{ color: #b8b8b8; font-weight: 600; }}
.mono {{ font-family: ui-monospace, Consolas, monospace; word-break: break-all; }}
dl {{ margin: 0; }}
dt {{ color: #b0b0b0; margin-top: 10px; font-size: 12px; }}
dd {{ margin: 4px 0 0 0; }}
</style>
</head>
<body><div class="layout"><nav>{nav}</nav><main>{body}</main></div></body>
</html>"#,
        title = esc_html(title),
        nav = nav_html,
        body = body_html,
    );
    html.into_bytes()
}

fn settings_shell_hint(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Esta vista es HTML servido por Servo. Para cambiar ajustes interactivos (tema, escala, permisos del sitio, etc.) usa el botón de engranaje de la barra o el atajo Ctrl+coma.",
        Locale::De => "Diese Seite wird als HTML von Servo ausgeliefert. Für interaktive Einstellungen nutze das Zahnradsymbol in der Leiste oder Strg+Komma.",
        Locale::Fr => "Cette page est du HTML fourni par Servo. Pour les réglages interactifs, utilisez la roue dentée de la barre ou Ctrl+virgule.",
        Locale::En => "This page is HTML served by Servo. For full interactive settings (theme, scale, site permissions, etc.) use the toolbar gear button or Ctrl+Comma.",
    }
}

fn nav_link(_loc: Locale, href: &str, label: &str, selected: bool) -> String {
    let cls = if selected { " class=\"sel\"" } else { "" };
    format!(
        r#"<a href="{href}"{cls}>{label}</a>"#,
        href = esc_html(href),
        cls = cls,
        label = esc_html(label),
    )
}

#[derive(Clone, Copy)]
enum ChromeHighlight {
    Downloads,
    History,
    Settings(SettingsNav),
}

fn settings_nav_html(loc: Locale, highlight: ChromeHighlight) -> String {
    let items: Vec<(SettingsNav, &'static str, String)> = vec![
        (SettingsNav::GetStarted, internal_pages::settings_sidebar_label(loc, SettingsNav::GetStarted), settings_page_url(SettingsNav::GetStarted)),
        (SettingsNav::Appearance, internal_pages::settings_sidebar_label(loc, SettingsNav::Appearance), settings_page_url(SettingsNav::Appearance)),
        (SettingsNav::Content, internal_pages::settings_sidebar_label(loc, SettingsNav::Content), settings_page_url(SettingsNav::Content)),
        (SettingsNav::PrivacyFilters, internal_pages::settings_sidebar_label(loc, SettingsNav::PrivacyFilters), settings_page_url(SettingsNav::PrivacyFilters)),
        (SettingsNav::PrivacySecurity, internal_pages::settings_sidebar_label(loc, SettingsNav::PrivacySecurity), settings_page_url(SettingsNav::PrivacySecurity)),
        (SettingsNav::Onchain, internal_pages::settings_sidebar_label(loc, SettingsNav::Onchain), settings_page_url(SettingsNav::Onchain)),
        (SettingsNav::AiCompanion, internal_pages::settings_sidebar_label(loc, SettingsNav::AiCompanion), settings_page_url(SettingsNav::AiCompanion)),
        (SettingsNav::Sync, internal_pages::settings_sidebar_label(loc, SettingsNav::Sync), settings_page_url(SettingsNav::Sync)),
        (SettingsNav::SearchEngine, internal_pages::settings_sidebar_label(loc, SettingsNav::SearchEngine), settings_page_url(SettingsNav::SearchEngine)),
        (SettingsNav::Extensions, internal_pages::settings_sidebar_label(loc, SettingsNav::Extensions), settings_page_url(SettingsNav::Extensions)),
        (SettingsNav::Autofill, internal_pages::settings_sidebar_label(loc, SettingsNav::Autofill), settings_page_url(SettingsNav::Autofill)),
        (SettingsNav::Languages, internal_pages::settings_sidebar_label(loc, SettingsNav::Languages), settings_page_url(SettingsNav::Languages)),
        (SettingsNav::DownloadPreferences, internal_pages::settings_sidebar_label(loc, SettingsNav::DownloadPreferences), settings_page_url(SettingsNav::DownloadPreferences)),
        (SettingsNav::Accessibility, internal_pages::settings_sidebar_label(loc, SettingsNav::Accessibility), settings_page_url(SettingsNav::Accessibility)),
        (SettingsNav::System, internal_pages::settings_sidebar_label(loc, SettingsNav::System), settings_page_url(SettingsNav::System)),
        (SettingsNav::SystemShortcuts, internal_pages::settings_sidebar_label(loc, SettingsNav::SystemShortcuts), settings_page_url(SettingsNav::SystemShortcuts)),
        (SettingsNav::Updates, internal_pages::settings_sidebar_label(loc, SettingsNav::Updates), settings_page_url(SettingsNav::Updates)),
        (SettingsNav::ResetSettings, internal_pages::settings_sidebar_label(loc, SettingsNav::ResetSettings), settings_page_url(SettingsNav::ResetSettings)),
    ];
    let mut out = String::new();
    out.push_str(&nav_link(
        loc,
        internal_pages::InternalRoute::Downloads.canonical_url(),
        crate::i18n::internal_nav_downloads(loc),
        matches!(highlight, ChromeHighlight::Downloads),
    ));
    out.push_str(&nav_link(
        loc,
        internal_pages::InternalRoute::History.canonical_url(),
        crate::i18n::internal_nav_history(loc),
        matches!(highlight, ChromeHighlight::History),
    ));
    out.push_str("<hr style=\"border:none;border-top:1px solid #333;margin:12px 0\"/>");
    for (nav, label, url) in items {
        let sel = if let ChromeHighlight::Settings(sn) = highlight {
            sn == nav
        } else {
            false
        };
        out.push_str(&nav_link(loc, &url, label, sel));
    }
    out
}

fn fmt_time(loc: Locale, unix: i64) -> String {
    use chrono::{TimeZone, Utc};
    let naive = Utc.timestamp_opt(unix, 0);
    let formatted = match naive.single() {
        Some(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        None => unix.to_string(),
    };
    let tz_note = match loc {
        Locale::Es | Locale::De | Locale::Fr | Locale::En => " UTC",
    };
    format!("{formatted}{tz_note}")
}

fn search_engine_label(e: SearchEngine) -> &'static str {
    match e {
        SearchEngine::Duckduckgo => "DuckDuckGo",
        SearchEngine::Google => "Google",
        SearchEngine::Brave => "Brave",
    }
}

fn theme_label(t: UiTheme) -> &'static str {
    match t {
        UiTheme::Dark => "dark",
        UiTheme::Light => "light",
    }
}

fn update_policy_label(u: UpdatePolicy) -> &'static str {
    match u {
        UpdatePolicy::OnStartup => "on_startup",
        UpdatePolicy::Periodic => "periodic",
        UpdatePolicy::ManualOnly => "manual_only",
    }
}

fn action_link(url: &str, label: &str) -> String {
    format!(
        "<a href=\"{}\" style=\"display:inline-block;padding:6px 10px;background:#2a2a2a;border:1px solid #404040;border-radius:6px;color:#f5f5f5;text-decoration:none;margin-right:8px;\">{}</a>",
        esc_html(url),
        esc_html(label)
    )
}

fn settings_body(loc: Locale, nav: SettingsNav, s: &AppSettings) -> String {
    let heading = internal_pages::settings_sidebar_label(loc, nav);
    let mut body = format!(
        "<h1>{}</h1><p class=\"hint\">{}</p>",
        esc_html(heading),
        esc_html(settings_shell_hint(loc)),
    );
    match nav {
        SettingsNav::GetStarted => {
            body.push_str("<p>Welcome to Tonet.</p>");
        }
        SettingsNav::Appearance => {
            body.push_str(&format!(
                "<dl><dt>UI theme</dt><dd>{}</dd><dt>UI scale</dt><dd>{}×</dd></dl>",
                esc_html(theme_label(s.ui_theme)),
                esc_html(&format!("{:.2}", s.ui_scale)),
            ));
        }
        SettingsNav::SearchEngine => {
            body.push_str(&format!(
                "<dl><dt>Default search</dt><dd>{}</dd></dl>",
                esc_html(search_engine_label(s.search_engine)),
            ));
        }
        SettingsNav::Languages => {
            body.push_str(&format!(
                "<dl><dt>UI language</dt><dd>{}</dd></dl>",
                esc_html(s.ui_language.trim()),
            ));
        }
        SettingsNav::DownloadPreferences => {
            let dd = s
                .download_directory
                .as_deref()
                .map(|p| esc_html(p))
                .unwrap_or_else(|| "(default)".into());
            body.push_str(&format!(
                "<dl><dt>Download directory</dt><dd class=\"mono\">{}</dd></dl>",
                dd
            ));
        }
        SettingsNav::System => {
            body.push_str(&format!(
                "<dl><dt>Hardware acceleration</dt><dd>{}</dd>\
                 <dt>Close window when last tab</dt><dd>{}</dd>\
                 <dt>Memory saver</dt><dd>{}</dd></dl>",
                s.system.use_hardware_acceleration,
                s.system.close_window_when_last_tab,
                s.system.memory_saver_enabled,
            ));
            body.push_str("<div style=\"margin-top:14px\">");
            body.push_str(&action_link(
                "tonet://settings/system?action=clear_servo_permissions",
                crate::i18n::internal_settings_servo_clear_permissions(loc),
            ));
            body.push_str("</div>");
        }
        SettingsNav::Updates => {
            body.push_str(&format!(
                "<dl><dt>Update policy</dt><dd>{}</dd></dl>",
                esc_html(update_policy_label(s.update_policy)),
            ));
        }
        _ => {
            body.push_str("<p class=\"hint\">No extra fields on this page in the Servo view.</p>");
        }
    }
    body
}

fn downloads_body(loc: Locale, items: &[DownloadRecord]) -> String {
    let title = crate::i18n::internal_tab_title_downloads(loc);
    let intro = crate::i18n::internal_downloads_intro(loc);
    let mut rows = String::new();
    for d in items.iter().rev().take(500) {
        rows.push_str(&format!(
            "<tr><td class=\"mono\">{}</td><td>{}</td><td>{}</td></tr>",
            esc_html(&d.url),
            esc_html(&d.display_name),
            esc_html(&fmt_time(loc, d.finished_at_unix)),
        ));
    }
    if rows.is_empty() {
        rows.push_str("<tr><td colspan=\"3\">—</td></tr>");
    }
    let mut out = format!(
        "<h1>{}</h1><p class=\"hint\">{}</p><table><thead><tr><th>URL</th><th>{}</th><th>{}</th></tr></thead><tbody>{}</tbody></table>",
        esc_html(title),
        esc_html(intro),
        esc_html(match loc {
            Locale::Es => "Nombre",
            Locale::De => "Name",
            Locale::Fr => "Nom",
            Locale::En => "Name",
        }),
        esc_html(match loc {
            Locale::Es => "Fecha",
            Locale::De => "Zeit",
            Locale::Fr => "Date",
            Locale::En => "Time",
        }),
        rows,
    );
    out.push_str("<div style=\"margin-top:14px\">");
    out.push_str(&action_link(
        "tonet://downloads?action=clear",
        crate::i18n::internal_clear_all(loc),
    ));
    out.push_str("</div>");
    out
}

fn history_body(loc: Locale, items: &[VisitRecord]) -> String {
    let title = crate::i18n::internal_tab_title_history(loc);
    let intro = crate::i18n::internal_history_sidebar(loc);
    let mut rows = String::new();
    for v in items.iter().rev().take(500) {
        let t = v.title.as_deref().unwrap_or("—");
        rows.push_str(&format!(
            "<tr><td class=\"mono\">{}</td><td>{}</td><td>{}</td></tr>",
            esc_html(&v.url),
            esc_html(t),
            esc_html(&fmt_time(loc, v.visited_at_unix)),
        ));
    }
    if rows.is_empty() {
        rows.push_str("<tr><td colspan=\"3\">—</td></tr>");
    }
    let mut out = format!(
        "<h1>{}</h1><p class=\"hint\">{}</p><table><thead><tr><th>URL</th><th>{}</th><th>{}</th></tr></thead><tbody>{}</tbody></table>",
        esc_html(title),
        esc_html(intro),
        esc_html(match loc {
            Locale::Es => "Título",
            Locale::De => "Titel",
            Locale::Fr => "Titre",
            Locale::En => "Title",
        }),
        esc_html(match loc {
            Locale::Es => "Visitado",
            Locale::De => "Besucht",
            Locale::Fr => "Visité",
            Locale::En => "Visited",
        }),
        rows,
    );
    out.push_str("<div style=\"margin-top:14px\">");
    out.push_str(&action_link(
        "tonet://history?action=clear",
        crate::i18n::internal_clear_all(loc),
    ));
    out.push_str("</div>");
    out
}

/// UTF-8 HTML document for a `tonet://` main-frame URL, or `None` if the URL is not a Tonet internal page.
pub(crate) fn document_bytes_for_tonet_url(
    url: &Url,
    st: &mut TonetSchemeSharedState,
) -> Option<Vec<u8>> {
    if !matches!(url.scheme(), "tonet") {
        return None;
    }
    let parsed = internal_pages::parse_tonet_url(url.as_str())?;
    let loc = st.loc;
    for (k, v) in url.query_pairs() {
        if k == "action" {
            match (parsed.route, v.as_ref()) {
                (InternalRoute::History, "clear") => {
                    st.pending_actions.push(TonetSchemeAction::ClearHistory)
                }
                (InternalRoute::Downloads, "clear") => {
                    st.pending_actions.push(TonetSchemeAction::ClearDownloads)
                }
                (InternalRoute::Settings, "clear_servo_permissions") => st
                    .pending_actions
                    .push(TonetSchemeAction::ClearServoSitePermissions),
                _ => {}
            }
        }
    }
    match parsed.route {
        InternalRoute::Settings => {
            let nav_sel = settings_nav_from_path(parsed.settings_path.as_str());
            let nav_html = settings_nav_html(loc, ChromeHighlight::Settings(nav_sel));
            let body = settings_body(loc, nav_sel, &st.settings);
            let title = internal_pages::tab_title(InternalRoute::Settings, loc);
            Some(wrap_doc(title, &nav_html, &body))
        }
        InternalRoute::Downloads => {
            let nav_html = settings_nav_html(loc, ChromeHighlight::Downloads);
            let body = downloads_body(loc, &st.downloads);
            let title = internal_pages::tab_title(InternalRoute::Downloads, loc);
            Some(wrap_doc(title, &nav_html, &body))
        }
        InternalRoute::History => {
            let nav_html = settings_nav_html(loc, ChromeHighlight::History);
            let body = history_body(loc, &st.visits);
            let title = internal_pages::tab_title(InternalRoute::History, loc);
            Some(wrap_doc(title, &nav_html, &body))
        }
    }
}
