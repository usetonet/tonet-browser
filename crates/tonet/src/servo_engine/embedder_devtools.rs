//! Data captured from Servo for DevTools (network log + DOM snapshot).

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use http::Method;
use url::Url;

const NETWORK_LOG_CAP: usize = 600;

static NEXT_NETWORK_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug)]
pub struct ServoNetworkEntry {
    pub id: u64,
    pub started: Instant,
    pub method: String,
    pub url: String,
    pub kind: NetworkResourceKind,
    pub status: NetworkEntryStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkResourceKind {
    Document,
    Stylesheet,
    Script,
    Image,
    Font,
    Media,
    Xhr,
    Other,
}

impl NetworkResourceKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Stylesheet => "stylesheet",
            Self::Script => "script",
            Self::Image => "image",
            Self::Font => "font",
            Self::Media => "media",
            Self::Xhr => "xhr",
            Self::Other => "other",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkEntryStatus {
    Pending,
    /// Passed through to Servo's network stack (normal fetch).
    Sent,
    /// Tonet intercepted (`tonet://`, background download, etc.).
    Intercepted,
}

impl NetworkEntryStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pending => "…",
            Self::Sent => "sent",
            Self::Intercepted => "intercepted",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ServoDomTreeNode {
    pub tag: String,
    pub id_attr: String,
    pub class_attr: String,
    pub attrs: Vec<(String, String)>,
    pub text_preview: Option<String>,
    pub children: Vec<ServoDomTreeNode>,
}

pub fn classify_network_resource(url: &Url, main_frame: bool, method: &Method) -> NetworkResourceKind {
    if main_frame && *method == Method::GET {
        return NetworkResourceKind::Document;
    }
    let path = url.path().to_ascii_lowercase();
    if path.ends_with(".css") {
        return NetworkResourceKind::Stylesheet;
    }
    if path.ends_with(".js") || path.ends_with(".mjs") {
        return NetworkResourceKind::Script;
    }
    if path.ends_with(".woff2")
        || path.ends_with(".woff")
        || path.ends_with(".ttf")
        || path.ends_with(".otf")
    {
        return NetworkResourceKind::Font;
    }
    if path.ends_with(".png")
        || path.ends_with(".jpg")
        || path.ends_with(".jpeg")
        || path.ends_with(".gif")
        || path.ends_with(".webp")
        || path.ends_with(".svg")
        || path.ends_with(".ico")
    {
        return NetworkResourceKind::Image;
    }
    if path.ends_with(".mp4")
        || path.ends_with(".webm")
        || path.ends_with(".mp3")
        || path.ends_with(".ogg")
    {
        return NetworkResourceKind::Media;
    }
    NetworkResourceKind::Other
}

pub fn append_network_entry(
    log: &mut VecDeque<ServoNetworkEntry>,
    method: &Method,
    url: &Url,
    main_frame: bool,
    status: NetworkEntryStatus,
) -> u64 {
    let id = NEXT_NETWORK_ID.fetch_add(1, Ordering::Relaxed);
    let entry = ServoNetworkEntry {
        id,
        started: Instant::now(),
        method: method.to_string(),
        url: url.to_string(),
        kind: classify_network_resource(url, main_frame, method),
        status,
    };
    log.push_back(entry);
    while log.len() > NETWORK_LOG_CAP {
        log.pop_front();
    }
    id
}

pub fn patch_network_status(log: &mut VecDeque<ServoNetworkEntry>, id: u64, status: NetworkEntryStatus) {
    if let Some(e) = log.iter_mut().find(|e| e.id == id) {
        e.status = status;
    }
}

/// JS executed in the page to produce a compact DOM JSON tree for the Elements panel.
pub const DOM_SNAPSHOT_SCRIPT: &str = r##"(function(){
  function summarize(el, depth) {
    if (!el || depth > 14) return null;
    if (el.nodeType === 3) {
      var t = (el.textContent || "").replace(/\s+/g, " ").trim();
      if (!t) return null;
      return { tag: "text", text: t.slice(0, 120), children: [] };
    }
    if (el.nodeType !== 1) return null;
    var tag = (el.tagName || "").toLowerCase();
    var attrs = {};
    if (el.attributes) {
      for (var i = 0; i < el.attributes.length; i++) {
        var a = el.attributes[i];
        attrs[a.name] = String(a.value).slice(0, 160);
      }
    }
    var kids = [];
    for (var c = el.firstChild; c; c = c.nextSibling) {
      var sub = summarize(c, depth + 1);
      if (sub) kids.push(sub);
    }
    return {
      tag: tag,
      id: el.id || "",
      cls: (typeof el.className === "string" ? el.className : "") || "",
      attrs: attrs,
      children: kids
    };
  }
  var root = document.documentElement;
  return JSON.stringify(summarize(root, 0));
})()"##;

#[derive(serde::Deserialize)]
struct DomJsonNode {
    tag: String,
    #[serde(default)]
    id: String,
    #[serde(default)]
    cls: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    attrs: std::collections::HashMap<String, String>,
    #[serde(default)]
    children: Vec<DomJsonNode>,
}

pub fn parse_dom_snapshot_json(json: &str) -> Result<ServoDomTreeNode, String> {
    let root: DomJsonNode =
        serde_json::from_str(json).map_err(|e| format!("JSON: {e}"))?;
    Ok(dom_from_json(root))
}

fn dom_from_json(n: DomJsonNode) -> ServoDomTreeNode {
    let mut attrs: Vec<(String, String)> = n
        .attrs
        .into_iter()
        .map(|(k, v)| (k, v))
        .collect();
    attrs.sort_by(|a, b| a.0.cmp(&b.0));
    let text_preview = n.text.filter(|t| !t.is_empty());
    ServoDomTreeNode {
        tag: n.tag,
        id_attr: n.id,
        class_attr: n.cls,
        attrs,
        text_preview,
        children: n.children.into_iter().map(dom_from_json).collect(),
    }
}
