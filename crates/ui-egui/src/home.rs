//! The Home tab (WPS's 首页): with it showing, the editor area becomes a start page — create a
//! document from a template (all of them, with a search), reopen a recent file, or browse for
//! one. Every document opens in its own tab ([`WordApp::open_in_tab`]).

use egui::{Align2, Rect, Sense, Stroke, Ui, pos2, vec2};
use serde_json::json;

use crate::theme::{Tokens, medium, regular, semibold};
use crate::{WordApp, icons};

pub fn show(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::CentralPanel::default().frame(egui::Frame::NONE.fill(t.canvas)).show(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // WPS's home page is one column down the middle, not edge to edge.
            let pad = ((ui.available_width() - 780.0) / 2.0).clamp(24.0, 220.0);
            ui.add_space(28.0);
            ui.horizontal(|ui| {
                ui.add_space(pad);
                ui.vertical(|ui| {
                    ui.set_max_width(760.0);
                    greeting(ui);
                    templates(app, ui);
                    ui.add_space(24.0);
                    recent(app, ui);
                    ui.add_space(48.0);
                });
            });
        });
    });
}

fn heading(ui: &mut Ui, s: &str) {
    ui.label(egui::RichText::new(tl!(s)).font(semibold(26.0)));
    ui.add_space(16.0);
}

fn greeting(ui: &mut Ui) {
    let hour = wordcraft_engine::cmd::now_iso().get(11..13).and_then(|h| h.parse::<u32>().ok()).unwrap_or(9);
    heading(
        ui,
        if hour < 12 {
            "Good morning"
        } else if hour < 18 {
            "Good afternoon"
        } else {
            "Good evening"
        },
    );
}

/// The built-in templates in gallery order: display label and the `file.new` template id.
const TEMPLATES: [(&str, &str); 5] =
    [("Blank document", "blank"), ("Studio handbook (sample)", "sample"), ("Letter", "letter"), ("Résumé", "resume"), ("Report", "report")];

/// The templates whose English or translated label, or id, contains `query` (case-insensitive;
/// the id lets "resume" find "Résumé"); all of them for an empty query.
fn matching_templates(query: &str) -> Vec<(&'static str, &'static str)> {
    let q = query.trim().to_lowercase();
    TEMPLATES
        .into_iter()
        .filter(|(label, id)| {
            q.is_empty() || id.contains(&q) || label.to_lowercase().contains(&q) || crate::i18n::tc("template", label).to_lowercase().contains(&q)
        })
        .collect()
}

/// The whole template gallery with a search box, as the old New page had it.
fn templates(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let id = egui::Id::new("home_template_search");
    let mut query = ui.ctx().data(|d| d.get_temp::<String>(id)).unwrap_or_default();
    let search = ui.add(egui::TextEdit::singleline(&mut query).hint_text(tl!("Search for templates")).desired_width(360.0));
    if search.changed() {
        ui.ctx().data_mut(|d| d.insert_temp(id, query.clone()));
    }
    ui.add_space(18.0);
    let found = matching_templates(&query);
    if found.is_empty() {
        ui.label(egui::RichText::new(tl!("No templates match your search.")).color(t.text_dim));
        return;
    }
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(18.0, 18.0);
        for (label, template) in found {
            template_tile(ui, app, label, template);
        }
    });
}

fn template_tile(ui: &mut Ui, app: &mut WordApp, label: &str, template: &str) {
    let t = Tokens::get(ui.ctx());
    ui.vertical(|ui| {
        // The label sits right under the thumbnail whatever row gap the gallery uses.
        ui.spacing_mut().item_spacing.y = 0.0;
        let (r, resp) = ui.allocate_exact_size(vec2(150.0, 194.0), Sense::click());
        ui.painter().rect(
            r,
            2.0,
            egui::Color32::WHITE,
            Stroke::new(if resp.hovered() { 2.0 } else { 1.0 }, if resp.hovered() { t.accent } else { t.border_strong }),
            egui::StrokeKind::Inside,
        );
        let ppp = ui.ctx().pixels_per_point();
        let key = format!("tpl:{template}:{ppp}");
        let region = app.session.region();
        let tex = ui.ctx().data(|d| d.get_temp::<egui::TextureHandle>(egui::Id::new(&key))).or_else(|| {
            let mut s = wordcraft_engine::Session::new(wordcraft_doc::Document::new());
            // The blank tile shows the region's paper (#838).
            s.system_region = region;
            let _ = s.run("file.new", &json!({"template": template}));
            let l = s.export_layout();
            let page = l.pages.first()?;
            let img = wordcraft_render::render_page(&s.doc, page, 150.0 / page.w * ppp, &crate::canvas::screen_render_options());
            let px = img.to_straight();
            let h = ui.ctx().load_texture(
                &key,
                egui::ColorImage::from_rgba_unmultiplied([img.width as usize, img.height as usize], &px),
                egui::TextureOptions::LINEAR,
            );
            ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new(&key), h.clone()));
            Some(h)
        });
        if let Some(h) = tex {
            ui.painter().image(h.id(), r.shrink(1.0), Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), egui::Color32::WHITE);
        }
        // The caption opens the template too, as in Word.
        let caption = ui.add(egui::Label::new(egui::RichText::new(crate::i18n::tc("template", label)).font(regular(12.5))).sense(Sense::click()));
        if resp.clicked() || caption.clicked() {
            let _ = app.run("file.new", json!({"template": template}));
            app.canvas.want_focus = true;
        }
    });
}

fn recent(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.label(egui::RichText::new(tl!("Recent")).font(semibold(16.0)));
    ui.add_space(6.0);
    // A file that can't be opened (deleted, moved, unreadable) says why here, until Home is
    // left (#2938).
    let error = ui.ctx().data(|d| d.get_temp::<String>(open_error_id())).unwrap_or_default();
    if !error.is_empty() {
        ui.label(egui::RichText::new(error).color(ui.visuals().error_fg_color));
        ui.add_space(6.0);
    }
    if app.ui.recent.is_empty() {
        ui.label(egui::RichText::new(tl!("Documents you open will show up here.")).color(t.text_dim));
    }
    for p in app.ui.recent.iter().take(app.ui.recent_shown()).cloned().collect::<Vec<_>>() {
        let name = std::path::Path::new(&p).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or(p.clone());
        let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width().min(700.0), 40.0), Sense::click());
        if resp.hovered() {
            ui.painter().rect_filled(r, 4.0, t.hover);
        }
        icons::paint(ui.painter(), Rect::from_center_size(pos2(r.min.x + 18.0, r.center().y), vec2(22.0, 22.0)), "printLayout", t.icon, t.accent);
        ui.painter().text(pos2(r.min.x + 40.0, r.min.y + 13.0), Align2::LEFT_CENTER, &name, medium(13.0), t.text);
        ui.painter().text(pos2(r.min.x + 40.0, r.min.y + 29.0), Align2::LEFT_CENTER, &p, regular(11.0), t.text_dim);
        if resp.clicked() {
            let failed = open_recent(app, &p).err();
            ui.ctx().data_mut(|d| d.insert_temp(open_error_id(), failed.unwrap_or_default()));
        }
    }
    ui.add_space(10.0);
    // A file the Recent list doesn't know.
    if ui.button(egui::RichText::new(tl!("Browse…")).font(medium(14.0))).clicked() {
        let _ = app.run("ui.openFileDialog", json!({}));
    }
    // Linux and the BSDs (#576): the system dialog needs a file chooser portal or zenity; when it
    // doesn't show up, WBS's own browser is always one click away.
    #[cfg(all(unix, not(target_os = "macos"), not(target_arch = "wasm32")))]
    if ui
        .button(egui::RichText::new(tl!("Browse in WBS…")).font(medium(14.0)))
        .on_hover_text(tl!("Open a file with WBS's own browser when the system file dialog doesn't appear."))
        .clicked()
    {
        let _ = app.run("ui.fileBrowser", json!({}));
    }
}

fn open_error_id() -> egui::Id {
    egui::Id::new("home_open_error")
}

/// Drop the Recent list's open error once Home is left, so it doesn't come back next time.
pub(crate) fn forget_open_error(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<String>(open_error_id()));
}

/// Open a file from the Recent list. A file that can't be opened stays on Home, where the list
/// can say why, rather than dropping the user into an empty document.
fn open_recent(app: &mut WordApp, path: &str) -> Result<(), String> {
    app.run("file.open", json!({"path": path}))?;
    app.canvas.want_focus = true;
    Ok(())
}

#[cfg(test)]
mod tests {
    use egui_kittest::kittest::Queryable;
    use serde_json::json;

    use crate::{Services, WordApp};

    fn harness(a: WordApp) -> egui_kittest::Harness<'static, WordApp> {
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        h
    }

    fn home_app() -> WordApp {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        a.run("ui.home", json!({})).unwrap();
        a
    }

    /// The templates whose label, id or translation matches, case-insensitively.
    #[test]
    fn template_search_matches_labels_case_insensitively() {
        let ids = |q: &str| super::matching_templates(q).into_iter().map(|(_, id)| id).collect::<Vec<_>>();
        assert_eq!(ids(""), ["blank", "sample", "letter", "resume", "report"]);
        assert_eq!(ids("  LETTER "), ["letter"]);
        assert_eq!(ids("re"), ["resume", "report"]);
        assert_eq!(ids("résumé"), ["resume"]);
        assert!(ids("no such template").is_empty());
    }

    /// The home view is the whole gallery (the old New page) and the Recent list (the old Home
    /// and Open pages) in one place (#141).
    #[test]
    fn home_shows_every_template_and_the_recent_list() {
        let h = harness(home_app());
        assert!(h.query_by_label("Report").is_some(), "every template shows");
        assert!(h.query_by_label("Recent").is_some(), "the recent list shows");
        assert!(h.query_by_label("Browse…").is_some(), "Browse is one click away");
    }

    /// #2938: opening a deleted file from Recent keeps Home up and records the error instead of
    /// dropping the user into an empty document.
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn opening_a_missing_recent_file_stays_home_with_the_error() {
        let mut app = home_app();
        let missing = std::env::temp_dir().join(format!("wordcraft-missing-{}.docx", std::process::id()));
        let err = super::open_recent(&mut app, &missing.to_string_lossy()).unwrap_err();
        assert!(!err.is_empty());
        assert!(app.home, "Home stays up");
    }

    /// #2931: a template tile on Home opens its document in its own tab and returns to the
    /// editor, without AccessKit losing the focused node.
    #[test]
    fn blank_document_from_home_returns_to_the_editor() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::from_text("Work")), Services::default());
        a.run("ui.home", json!({})).unwrap();
        let mut h = harness(a);
        h.get_by_label("Blank document").click();
        h.run_steps(4);
        assert!(!h.state().home, "Blank document returns to the editor");
        assert_eq!(h.state().tabs.len(), 1, "the old work keeps its own tab");
        assert_eq!(h.state().session.doc.plain_text(wordcraft_doc::StoryRef::Body), "", "a fresh blank document is showing");
        assert!(h.state().dialog.is_none(), "nothing asks first");
    }
}
