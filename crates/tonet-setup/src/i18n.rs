//! Setup UI strings (Chrome-style minimal installer).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Locale {
    Es,
    En,
    De,
    Fr,
}

impl Locale {
    pub fn detect() -> Self {
        let tag = sys_locale::get_locale().unwrap_or_else(|| "en".into());
        let lower = tag.to_lowercase();
        if lower.starts_with("es") {
            Self::Es
        } else if lower.starts_with("de") {
            Self::De
        } else if lower.starts_with("fr") {
            Self::Fr
        } else {
            Self::En
        }
    }
}

pub fn window_title(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Instalador de Tonet",
        Locale::De => "Tonet-Installation",
        Locale::Fr => "Installation de Tonet",
        Locale::En => "Tonet Setup",
    }
}

pub fn phase_preparing(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Preparando…",
        Locale::De => "Vorbereitung…",
        Locale::Fr => "Préparation…",
        Locale::En => "Preparing…",
    }
}

pub fn phase_downloading(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Descargando…",
        Locale::De => "Wird heruntergeladen…",
        Locale::Fr => "Téléchargement…",
        Locale::En => "Downloading…",
    }
}

pub fn phase_installing(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Instalando…",
        Locale::De => "Installation…",
        Locale::Fr => "Installation…",
        Locale::En => "Installing…",
    }
}

pub fn phase_updating(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Actualizando…",
        Locale::De => "Aktualisierung…",
        Locale::Fr => "Mise à jour…",
        Locale::En => "Updating…",
    }
}

pub fn phase_updating_detail(loc: Locale, from: &str, to: &str) -> String {
    match loc {
        Locale::Es => format!("Actualizando de {from} a {to}…"),
        Locale::De => format!("Aktualisierung von {from} auf {to}…"),
        Locale::Fr => format!("Mise à jour de {from} vers {to}…"),
        Locale::En => format!("Updating from {from} to {to}…"),
    }
}

pub fn phase_installing_update(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Aplicando actualización…",
        Locale::De => "Update wird angewendet…",
        Locale::Fr => "Application de la mise à jour…",
        Locale::En => "Applying update…",
    }
}

pub fn phase_done_updated(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Actualizado",
        Locale::De => "Aktualisiert",
        Locale::Fr => "Mis à jour",
        Locale::En => "Updated",
    }
}

pub fn already_installed(loc: Locale, version: &str) -> String {
    match loc {
        Locale::Es => format!(
            "Tonet {version} ya está instalado y es la última versión.\nNo es necesario volver a instalar."
        ),
        Locale::De => format!(
            "Tonet {version} ist bereits installiert und auf dem neuesten Stand.\nEine erneute Installation ist nicht nötig."
        ),
        Locale::Fr => format!(
            "Tonet {version} est déjà installé et à jour.\nUne nouvelle installation n’est pas nécessaire."
        ),
        Locale::En => format!(
            "Tonet {version} is already installed and up to date.\nRe-installing is not needed."
        ),
    }
}

pub fn phase_done(loc: Locale) -> &'static str {
    match loc {
        Locale::Es => "Instalado",
        Locale::De => "Installiert",
        Locale::Fr => "Installé",
        Locale::En => "Installed",
    }
}

pub fn brand_wordmark(loc: Locale) -> &'static str {
    let _ = loc;
    "tonet"
}
