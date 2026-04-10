//! Preferencias persistentes de Tonet (JSON en el directorio de configuración del SO).

use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use serde::{Deserialize, Serialize};

/// Cómo y cuándo comprobar actualizaciones en GitHub Releases.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UpdatePolicy {
    /// Al abrir Tonet una vez (recomendado): comprobación única al arranque.
    #[default]
    OnStartup,
    /// Además, repetir cada 24 h mientras la app sigue abierta.
    Periodic,
    /// Nunca en automático; solo con «Comprobar ahora».
    ManualOnly,
}

/// Ajustes guardados en disco.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub update_policy: UpdatePolicy,
    /// Marca de tiempo UNIX de la última comprobación (éxito o fallo de red).
    #[serde(default)]
    pub last_update_check_unix: Option<i64>,
}

impl AppSettings {
    /// Ruta a `settings.json` (por ejemplo `%APPDATA%\tonet\` en Windows).
    pub fn file_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("tonet").join("settings.json"))
    }

    /// Carga ajustes o devuelve valores por defecto si no existen o hay error.
    pub fn load() -> Self {
        let Some(path) = Self::file_path() else {
            return Self::default();
        };
        if !path.exists() {
            return Self::default();
        }
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<AppSettings>(&s).ok())
            .unwrap_or_default()
    }

    /// Guarda en disco (crea directorios intermedios).
    pub fn save(&self) -> anyhow::Result<()> {
        let Some(path) = Self::file_path() else {
            anyhow::bail!("No hay directorio de configuración en este sistema.");
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("mkdir {:?}", parent))?;
        }
        let json = serde_json::to_string_pretty(self).context("serializar ajustes")?;
        fs::write(&path, json).with_context(|| format!("escribir {:?}", path))?;
        Ok(())
    }

    /// Texto breve para la UI.
    pub fn update_policy_label(policy: UpdatePolicy) -> &'static str {
        match policy {
            UpdatePolicy::OnStartup => "Solo al iniciar Tonet",
            UpdatePolicy::Periodic => "Automático (cada 24 h)",
            UpdatePolicy::ManualOnly => "Solo manual",
        }
    }

    /// Descripción ampliada para ayuda contextual.
    pub fn update_policy_help(policy: UpdatePolicy) -> &'static str {
        match policy {
            UpdatePolicy::OnStartup => {
                "Comprueba una vez al abrir la aplicación. Ligero y predecible."
            }
            UpdatePolicy::Periodic => {
                "Comprueba al iniciar y, si sigues usando Tonet, vuelve a comprobar cada 24 horas."
            }
            UpdatePolicy::ManualOnly => {
                "No se consulta GitHub hasta que pulses «Comprobar ahora»."
            }
        }
    }
}
