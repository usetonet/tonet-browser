//! Renders the simplified DOM into egui widgets.

use crate::css_resolve::{
    display_text_cow, resolve_max_width_cap_px, resolve_padding_inset_px, resolve_text_indent_px,
    DisplayHint, DomNodePaintHints, OverflowWrapHint, TextAlignHint, VisibilityHint, WhiteSpaceHint,
    WordBreakHint, AUTHOR_STYLE_ROOT_PX,
};
use crate::i18n;
use crate::i18n::Locale;
use crate::parser::{DomNode, DomNodeType};
use crate::theme;
use egui::text::TextWrapping;
use egui::{Align, Color32, FontSelection, Label, Layout, Link, RichText, Ui};

/// Draws parsed nodes in the scrollable page area. `link_target` receives an absolute URL when a link is activated.
///
/// When `author_hints` is `Some` and has the same length as `nodes`, author `color`, `font-size`,
/// `line-height`, `letter-spacing`, `font-weight`, `font-style`, `margin` / margins, `text-decoration`, `text-align`, `text-transform`, `text-indent`, `opacity`, `visibility`, `display` (`none` skips the node, including when `html`/`body` defaults resolve to `none`), `white-space` (`nowrap` → no soft wrap), `word-break` (`break-all`), `overflow-wrap` / `word-wrap` (`anywhere` / `break-word`), `max-width`, and `padding` / `padding-left` / `padding-right` (per-node only, not from `html`/`body`) override or extend built-in page chrome.
pub fn render_nodes(
    ui: &mut Ui,
    loc: Locale,
    nodes: &[DomNode],
    author_hints: Option<&[DomNodePaintHints]>,
    link_target: &mut Option<String>,
) {
    if nodes.is_empty() {
        ui.label(
            RichText::new(i18n::empty_page_hint(loc))
                .italics()
                .color(theme::loading_muted()),
        );
        return;
    }

    let hints_slice = author_hints.filter(|h| h.len() == nodes.len());

    for (i, node) in nodes.iter().enumerate() {
        let hint = hints_slice.and_then(|h| h.get(i)).copied();
        if matches!(hint.and_then(|h| h.display), Some(DisplayHint::None)) {
            continue;
        }

        match node.kind {
            DomNodeType::Title => {
                let (def_size, def_color) = (21.0, theme::page_title());
                let size = hint.and_then(|h| h.font_size).unwrap_or(def_size);
                let color = hint.and_then(|h| h.color).unwrap_or(def_color);
                let top = hint
                    .and_then(|h| h.margin_top)
                    .unwrap_or_else(|| default_margin_top(node.kind));
                ui.add_space(top);
                paint_read_text_block(ui, node, hint, size, color, link_target);
                let bottom = hint
                    .and_then(|h| h.margin_bottom)
                    .unwrap_or_else(|| default_margin_bottom(node.kind));
                ui.add_space(bottom);
            }
            DomNodeType::H1 => {
                let (def_size, def_color) = (26.0, theme::body_text());
                let size = hint.and_then(|h| h.font_size).unwrap_or(def_size);
                let color = hint.and_then(|h| h.color).unwrap_or(def_color);
                let top = hint
                    .and_then(|h| h.margin_top)
                    .unwrap_or_else(|| default_margin_top(node.kind));
                ui.add_space(top);
                paint_read_text_block(ui, node, hint, size, color, link_target);
                let bottom = hint
                    .and_then(|h| h.margin_bottom)
                    .unwrap_or_else(|| default_margin_bottom(node.kind));
                ui.add_space(bottom);
            }
            DomNodeType::H2 => {
                let (def_size, def_color) = (19.0, theme::body_text());
                let size = hint.and_then(|h| h.font_size).unwrap_or(def_size);
                let color = hint.and_then(|h| h.color).unwrap_or(def_color);
                let top = hint
                    .and_then(|h| h.margin_top)
                    .unwrap_or_else(|| default_margin_top(node.kind));
                ui.add_space(top);
                paint_read_text_block(ui, node, hint, size, color, link_target);
                let bottom = hint
                    .and_then(|h| h.margin_bottom)
                    .unwrap_or_else(|| default_margin_bottom(node.kind));
                ui.add_space(bottom);
            }
            DomNodeType::Paragraph => {
                let (def_size, def_color) = (15.0, theme::body_text());
                let size = hint.and_then(|h| h.font_size).unwrap_or(def_size);
                let color = hint.and_then(|h| h.color).unwrap_or(def_color);
                let top = hint
                    .and_then(|h| h.margin_top)
                    .unwrap_or_else(|| default_margin_top(node.kind));
                ui.add_space(top);
                paint_read_text_block(ui, node, hint, size, color, link_target);
                let bottom = hint
                    .and_then(|h| h.margin_bottom)
                    .unwrap_or_else(|| default_margin_bottom(node.kind));
                ui.add_space(bottom);
            }
            DomNodeType::Link => {
                let (def_size, def_color) = if node.href.is_some() {
                    (15.0, theme::link())
                } else {
                    (15.0, theme::body_text())
                };
                let size = hint.and_then(|h| h.font_size).unwrap_or(def_size);
                let color = hint.and_then(|h| h.color).unwrap_or(def_color);
                let top = hint
                    .and_then(|h| h.margin_top)
                    .unwrap_or_else(|| default_margin_top(node.kind));
                ui.add_space(top);
                paint_read_text_block(ui, node, hint, size, color, link_target);
                let bottom = hint
                    .and_then(|h| h.margin_bottom)
                    .unwrap_or_else(|| default_margin_bottom(node.kind));
                ui.add_space(bottom);
            }
        }
    }
}

fn layout_job_for_rich_text(
    ui: &Ui,
    rt: RichText,
    extra_leading: f32,
    nowrap: bool,
    break_anywhere_when_wrapping: bool,
) -> egui::text::LayoutJob {
    let w = ui.available_width();
    let mut job = egui::WidgetText::from(rt).into_layout_job(
        ui.style(),
        FontSelection::default(),
        Align::Min,
    );
    if extra_leading.abs() > f32::EPSILON {
        if let Some(sec) = job.sections.first_mut() {
            sec.leading_space += extra_leading;
        }
    }
    job.wrap = if nowrap {
        TextWrapping::no_max_width()
    } else {
        TextWrapping::from_wrap_mode_and_width(ui.wrap_mode(), w)
    };
    if break_anywhere_when_wrapping && !nowrap {
        job.wrap.break_anywhere = true;
    }
    job
}

/// Paints author-styled text; uses a [`egui::text::LayoutJob`] when `text-indent`, `white-space: nowrap`, or line-break hints need layout control.
fn paint_styled_text(
    ui: &mut Ui,
    hint: Option<DomNodePaintHints>,
    used_font_size: f32,
    rt: RichText,
    href: Option<&str>,
    link_target: &mut Option<String>,
) {
    let text_visible = !matches!(
        hint.and_then(|h| h.visibility),
        Some(VisibilityHint::Hidden)
    );
    let line_width = ui.available_width();
    let indent_px = hint
        .and_then(|h| h.text_indent)
        .map(|s| resolve_text_indent_px(s, used_font_size, AUTHOR_STYLE_ROOT_PX, line_width))
        .unwrap_or(0.0);
    let nowrap = matches!(
        hint.and_then(|h| h.white_space),
        Some(WhiteSpaceHint::Nowrap)
    );
    let break_anywhere_when_wrapping = matches!(
        hint.and_then(|h| h.word_break),
        Some(WordBreakHint::BreakAll)
    ) || matches!(
        hint.and_then(|h| h.overflow_wrap),
        Some(OverflowWrapHint::Anywhere) | Some(OverflowWrapHint::BreakWord)
    );
    let job_needed = indent_px.abs() > f32::EPSILON;
    let use_layout_job = job_needed || nowrap || break_anywhere_when_wrapping;

    if let Some(href) = href {
        if use_layout_job {
            let job = layout_job_for_rich_text(
                ui,
                rt,
                indent_px,
                nowrap,
                break_anywhere_when_wrapping,
            );
            let r = ui.add_visible(text_visible, Link::new(job));
            if text_visible && r.clicked() {
                *link_target = Some(href.to_string());
            }
            if text_visible {
                r.on_hover_text(href);
            }
        } else {
            let r = ui.add_visible(text_visible, Link::new(rt));
            if text_visible && r.clicked() {
                *link_target = Some(href.to_string());
            }
            if text_visible {
                r.on_hover_text(href);
            }
        }
    } else if use_layout_job {
        let job = layout_job_for_rich_text(
            ui,
            rt,
            indent_px,
            nowrap,
            break_anywhere_when_wrapping,
        );
        ui.add_visible(text_visible, Label::new(job));
    } else {
        ui.add_visible(text_visible, Label::new(rt));
    }
}

fn with_text_align(ui: &mut Ui, hint: Option<DomNodePaintHints>, child: impl FnOnce(&mut Ui)) {
    let align = hint
        .and_then(|h| h.text_align)
        .unwrap_or(TextAlignHint::Start);
    let w = ui.available_width();
    match align {
        TextAlignHint::Start => child(ui),
        TextAlignHint::Center => {
            ui.allocate_ui_with_layout(
                egui::vec2(w, 0.0),
                Layout::top_down(Align::Center),
                |ui| {
                    ui.set_width(w);
                    child(ui);
                },
            );
        }
        TextAlignHint::End => {
            ui.allocate_ui_with_layout(
                egui::vec2(w, 0.0),
                Layout::right_to_left(Align::Min),
                |ui| {
                    ui.set_width(w);
                    child(ui);
                },
            );
        }
    }
}

/// Narrows the child [`Ui`] when author `max-width` resolves smaller than the read area (CSS non-inheritance: only matching rules).
fn with_max_width(ui: &mut Ui, hint: Option<DomNodePaintHints>, used_font_size: f32, child: impl FnOnce(&mut Ui)) {
    let avail = ui.available_width();
    let cap = hint
        .and_then(|h| h.max_width)
        .and_then(|spec| resolve_max_width_cap_px(spec, used_font_size, AUTHOR_STYLE_ROOT_PX, avail));
    if let Some(mut w) = cap {
        w = w.max(1.0).min(avail);
        if avail - w > 0.5 {
            ui.allocate_ui_with_layout(egui::vec2(w, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(w);
                child(ui);
            });
        } else {
            child(ui);
        }
    } else {
        child(ui);
    }
}

/// Horizontal `padding-left` / `padding-right` (per-node), then `max-width`, `text-align`, and text.
fn paint_read_text_block(
    ui: &mut Ui,
    node: &DomNode,
    hint: Option<DomNodePaintHints>,
    size: f32,
    color: Color32,
    link_target: &mut Option<String>,
) {
    let full_w = ui.available_width().max(1.0);
    let pl = hint
        .and_then(|h| h.padding_left)
        .map(|s| resolve_padding_inset_px(s, size, AUTHOR_STYLE_ROOT_PX, full_w))
        .unwrap_or(0.0);
    let pr = hint
        .and_then(|h| h.padding_right)
        .map(|s| resolve_padding_inset_px(s, size, AUTHOR_STYLE_ROOT_PX, full_w))
        .unwrap_or(0.0);

    if pl <= f32::EPSILON && pr <= f32::EPSILON {
        let rt = styled_rich_text(node, hint, size, color);
        let href = node.href.as_deref();
        with_max_width(ui, hint, size, |ui| {
            with_text_align(ui, hint, |ui| {
                paint_styled_text(ui, hint, size, rt, href, link_target);
            });
        });
    } else {
        let inner_w = (full_w - pl - pr).max(1.0);
        ui.horizontal(|ui| {
            if pl > 0.01 {
                ui.add_space(pl);
            }
            ui.allocate_ui_with_layout(
                egui::vec2(inner_w, 0.0),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.set_width(inner_w);
                    let rt = styled_rich_text(node, hint, size, color);
                    let href = node.href.as_deref();
                    with_max_width(ui, hint, size, |ui| {
                        with_text_align(ui, hint, |ui| {
                            paint_styled_text(ui, hint, size, rt, href, link_target);
                        });
                    });
                },
            );
            if pr > 0.01 {
                ui.add_space(pr);
            }
        });
    }
}

fn default_margin_top(kind: DomNodeType) -> f32 {
    match kind {
        DomNodeType::Title => 4.0,
        DomNodeType::H1 => 6.0,
        DomNodeType::H2 => 4.0,
        DomNodeType::Paragraph | DomNodeType::Link => 0.0,
    }
}

fn default_margin_bottom(kind: DomNodeType) -> f32 {
    match kind {
        DomNodeType::Title => 8.0,
        DomNodeType::H1 => 4.0,
        DomNodeType::H2 => 2.0,
        DomNodeType::Paragraph => 6.0,
        DomNodeType::Link => 4.0,
    }
}

/// Title / headings default to bold (`strong`); body text does not. Author `font-weight` / `font-style` override.
fn styled_rich_text(
    node: &DomNode,
    hint: Option<DomNodePaintHints>,
    size: f32,
    color: Color32,
) -> RichText {
    let default_bold = matches!(
        node.kind,
        DomNodeType::Title | DomNodeType::H1 | DomNodeType::H2
    );
    let default_weight = if default_bold { 700u16 } else { 400u16 };
    let weight = hint.and_then(|h| h.font_weight).unwrap_or(default_weight);

    let text = display_text_cow(&node.text, hint.and_then(|h| h.text_transform));
    let mut color = color;
    if let Some(o) = hint.and_then(|h| h.opacity) {
        color = color.gamma_multiply(o);
    }
    let mut rt = RichText::new(text.as_ref()).size(size).color(color);
    if weight >= 600 {
        rt = rt.strong();
    }
    if matches!(hint.and_then(|h| h.font_style_italic), Some(true)) {
        rt = rt.italics();
    }
    if matches!(hint.and_then(|h| h.underline), Some(true)) {
        rt = rt.underline();
    }
    if let Some(lh) = hint.and_then(|h| h.line_height_px) {
        rt = rt.line_height(Some(lh));
    }
    if let Some(ls) = hint.and_then(|h| h.letter_spacing_px) {
        rt = rt.extra_letter_spacing(ls);
    }
    rt
}
