//! Parser HTML mínimo y hecho a mano (sin motor HTML completo).
//!
//! Solo extrae, en orden de aparición en el documento, el contenido textual de
//! `<title>`, `<h1>`, `<h2>` y `<p>`. El resto de etiquetas se ignora salvo para
//! eliminar marcas internas al texto extraído.

/// Tipo semántico de un nodo extraído del documento.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomNodeType {
    Title,
    H1,
    H2,
    Paragraph,
}

impl DomNodeType {
    /// Nombre de etiqueta HTML en minúsculas asociado a este tipo.
    fn tag_name(self) -> &'static str {
        match self {
            DomNodeType::Title => "title",
            DomNodeType::H1 => "h1",
            DomNodeType::H2 => "h2",
            DomNodeType::Paragraph => "p",
        }
    }

    /// Todos los tipos que Tonet reconoce en el orden preferido de detección.
    fn all() -> [DomNodeType; 4] {
        [
            DomNodeType::Title,
            DomNodeType::H1,
            DomNodeType::H2,
            DomNodeType::Paragraph,
        ]
    }
}

/// Nodo del DOM simplificado de Tonet.
#[derive(Debug, Clone)]
pub struct DomNode {
    pub kind: DomNodeType,
    pub text: String,
}

/// Parsea HTML de forma muy limitada y devuelve los nodos detectados en orden.
pub fn parse_html(html: &str) -> Vec<DomNode> {
    let mut out = Vec::new();
    let mut pos = 0usize;

    while pos < html.len() {
        // Buscar la siguiente etiqueta de apertura que nos interese.
        let Some((idx, kind)) = find_next_target_open_tag(html, pos) else {
            break;
        };

        let tag = kind.tag_name();
        let Some((raw_inner, end_after_close)) = extract_inner_until_close(html, idx, tag) else {
            // Si no hay cierre bien formado, avanzamos un byte para no bloquear el bucle.
            pos = idx.saturating_add(1);
            continue;
        };

        let cleaned = normalize_whitespace(&strip_html_tags(raw_inner));
        if !cleaned.is_empty() {
            out.push(DomNode {
                kind,
                text: cleaned,
            });
        }

        pos = end_after_close;
    }

    out
}

/// Elimina etiquetas HTML del fragmento, conservando el texto visible aproximado.
fn strip_html_tags(fragment: &str) -> String {
    let mut out = String::with_capacity(fragment.len());
    let mut in_tag = false;
    for ch in fragment.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

/// Colapsa espacios en blanco consecutivos y recorta extremos.
fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Encuentra la próxima apertura `<title`, `<h1`, `<h2` o `<p` desde `pos`, sin distinguir mayúsculas.
fn find_next_target_open_tag(html: &str, pos: usize) -> Option<(usize, DomNodeType)> {
    let bytes = html.as_bytes();
    let mut i = pos;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            for kind in DomNodeType::all() {
                if open_tag_matches(html, i, kind.tag_name()) {
                    return Some((i, kind));
                }
            }
        }
        i += 1;
    }
    None
}

/// Comprueba si en `idx` comienza `<tag` seguido de fin de nombre de etiqueta (`>`, espacio, `/`).
fn open_tag_matches(html: &str, idx: usize, tag: &str) -> bool {
    let Some(rest) = html.get(idx..) else {
        return false;
    };
    let mut pattern = String::with_capacity(tag.len() + 1);
    pattern.push('<');
    pattern.push_str(tag);
    let prefix = match rest.get(..pattern.len().min(rest.len())) {
        Some(p) => p,
        None => return false,
    };
    if !eq_ignore_ascii_case_prefix(prefix, &pattern) {
        return false;
    }
    let after = idx + pattern.len();
    let c = html.as_bytes().get(after).copied();
    matches!(c, Some(b'>') | Some(b'/') | Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r'))
}

fn eq_ignore_ascii_case_prefix(a: &str, b: &str) -> bool {
    if a.len() < b.len() {
        return false;
    }
    a[..b.len()].eq_ignore_ascii_case(b)
}

/// Extrae el interior entre la etiqueta de apertura en `open_idx` y el cierre `</tag>`.
/// Devuelve (interior sin procesar, índice tras el `>` del cierre).
fn extract_inner_until_close<'a>(html: &'a str, open_idx: usize, tag: &str) -> Option<(&'a str, usize)> {
    let bytes = html.as_bytes();
    let gt = find_byte(bytes, b'>', open_idx)?;
    let inner_start = gt + 1;
    let close_start = find_closing_tag(html, inner_start, tag)?;
    let tail = html.get(close_start..)?;
    let rel_gt = tail.find('>')?;
    let close_end = close_start + rel_gt + 1;
    Some((html.get(inner_start..close_start)?, close_end))
}

fn find_byte(bytes: &[u8], needle: u8, from: usize) -> Option<usize> {
    bytes[from..].iter().position(|&b| b == needle).map(|p| from + p)
}

/// Localiza `</tag>` ignorando mayúsculas, empezando en `from`.
fn find_closing_tag(html: &str, from: usize, tag: &str) -> Option<usize> {
    let needle = format!("</{tag}>");
    let bytes = html.as_bytes();
    let mut i = from;
    while i + 2 < bytes.len() {
        if bytes[i] == b'<' && bytes.get(i + 1) == Some(&b'/') {
            let rest = html.get(i..)?;
            if rest.len() >= needle.len() {
                let slice = &rest[..needle.len()];
                if slice.eq_ignore_ascii_case(needle.as_str()) {
                    return Some(i);
                }
            }
        }
        i += 1;
    }
    None
}
