//! Módulo de red: descarga HTTP bloqueante con políticas de Tonet.
//!
//! Incluye validación de esquema (solo http/https), User-Agent fijo y el
//! **Filtro de Pureza** que rechaza respuestas cuyo cuerpo supere 1 MiB.

use anyhow::{anyhow, Context};
use url::Url;

/// Tamaño máximo permitido para el HTML descargado (1 MB en bytes).
const MAX_BODY_BYTES: usize = 1_000_000;

/// Realiza un GET bloqueante a la URL indicada y devuelve el cuerpo como texto UTF-8.
///
/// - User-Agent: `Tonet/0.1 (Minimalist Browser)`.
/// - Solo se aceptan esquemas `http` y `https`.
/// - Solo se acepta código de estado **200 OK** (cualquier otro produce error claro).
/// - Si el cuerpo supera 1 MB, se rechaza con el mensaje del Filtro de Pureza.
pub fn fetch_url(url: &str) -> Result<String, anyhow::Error> {
    let parsed = Url::parse(url).with_context(|| format!("URL inválida: {url}"))?;
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(anyhow!(
            "Solo se admiten URLs http y https (recibido esquema: {scheme})"
        ));
    }

    let client = reqwest::blocking::Client::builder()
        .user_agent("Tonet/0.1 (Minimalist Browser)")
        .build()
        .context("No se pudo crear el cliente HTTP")?;

    let response = client
        .get(parsed.as_str())
        .send()
        .with_context(|| format!("Fallo al solicitar {url}"))?;

    let status = response.status();
    if status != reqwest::StatusCode::OK {
        return Err(anyhow!(
            "Error HTTP: el servidor respondió con {} (se esperaba 200 OK)",
            status
        ));
    }

    let bytes = response
        .bytes()
        .context("No se pudo leer el cuerpo de la respuesta")?;

    // Filtro de Pureza: rechazar páginas demasiado pesadas.
    if bytes.len() > MAX_BODY_BYTES {
        return Err(anyhow!(
            "Error de Tonet: Página demasiado pesada (más de 1 MB). Tonet solo carga contenido ligero."
        ));
    }

    let text = String::from_utf8(bytes.to_vec())
        .map_err(|e| anyhow!("El cuerpo no es UTF-8 válido: {e}"))?;

    Ok(text)
}
