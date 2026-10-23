//! Tab bar (logo menu, Home and document tabs, Quick Access Toolbar, search, account) and
//! status bar.

use egui::{Align2, Rect, Sense, Stroke, Ui, pos2, vec2};
use serde_json::json;
use wordcraft_doc::StoryRef;

use crate::theme::{Tokens, TypeRung, medium, paint_text, regular, semibold, text_width};
use crate::{WordApp, icons};

/// The Quick Access Toolbar's button for command `id`.
pub(crate) fn qat_button_id(id: &str) -> egui::Id {
    egui::Id::new(("qat_button", id))
}

fn qat_button(ui: &mut Ui, app: &mut WordApp, icon: &str, tip: &str, id: &str, enabled: bool) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let (r, _) = ui.allocate_exact_size(vec2(26.0, 24.0), Sense::hover());
    let resp = ui.interact(r, qat_button_id(id), Sense::click());
    if resp.hovered() && enabled {
        ui.painter().rect_filled(r, 4.0, t.hover);
    }
    let c = if enabled { t.icon } else { t.text_disabled };
    icons::paint(ui.painter(), Rect::from_center_size(r.center(), vec2(16.0, 16.0)), icon, c, if enabled { t.accent } else { c });
    let sc = crate::widgets::shortcut_text(app, id);
    let resp = resp.on_hover_text(if sc.is_empty() { tip.to_string() } else { format!("{tip} ({sc})") });
    if resp.clicked() && enabled {
        let _ = app.run(id, json!({}));
    }
    resp
}

/// Commands the Quick Access Toolbar's menu offers to show or hide with one click (#376).
const QAT_MENU: [&str; 10] =
    ["file.new", "file.open", "file.save", "file.print", "edit.undo", "edit.redo", "edit.repeat", "review.spelling", "view.readMode", "insert.table"];

/// Match the toolbar's enabled state for keyboard activation too.
pub(crate) fn qat_enabled(app: &WordApp, id: &str) -> bool {
    match id {
        "edit.undo" => app.session.can_undo(),
        "edit.redo" => app.session.can_redo(),
        "file.save" => app.has_document(),
        _ => crate::widgets::enabled(app, id),
    }
}

/// The toolbar's commands: the user's own once customized, or WPS's default set — Save, Export
/// PDF, Print, Undo, Redo (the engine's stored default is Save/Undo/Redo; the extras are this
/// bar's arrangement, so a saved customization keeps working unchanged).
pub(crate) fn qat_ids(app: &WordApp) -> Vec<&str> {
    const DEFAULT: [&str; 5] = ["file.save", "file.exportPdf", "file.print", "edit.undo", "edit.redo"];
    match &app.session.ribbon.qat {
        Some(q) => q.iter().map(String::as_str).collect(),
        None => DEFAULT.to_vec(),
    }
}

/// The Quick Access Toolbar (#379): the user's commands, each with a right-click menu, then the
/// menu button that customizes it (#376).
fn quick_access(ui: &mut Ui, app: &mut WordApp) {
    let ids: Vec<String> = qat_ids(app).into_iter().map(str::to_string).collect();
    for (index, id) in ids.iter().enumerate() {
        let id = id.as_str();
        let tip = match id {
            "edit.undo" => crate::i18n::prefixed("Undo", app.session.undo_label().unwrap_or("")),
            "edit.redo" => tl!("Redo").to_string(),
            "file.save" => tl!("Save").to_string(),
            _ => app.session.registry.get(id).map(|s| tl!(s.label).to_string()).unwrap_or_default(),
        };
        let enabled = qat_enabled(app, id);
        let resp = qat_button(ui, app, crate::widgets::command_icon(id), &tip, id, enabled);
        if app.ui.keytips == crate::keytips::Phase::Tabs && ui.is_rect_visible(resp.rect) {
            let label = crate::keytips::qat_label(index, ids.len());
            if label.starts_with(&app.ui.keytip_prefix) {
                app.keytip_rects.push((resp.rect, label));
            }
        }
        crate::widgets::command_menu(&resp, app, id);
        if id == "edit.undo" {
            undo_list_button(ui, app, enabled);
        }
    }
    let t = Tokens::get(ui.ctx());
    let (r, _) = ui.allocate_exact_size(vec2(26.0, 24.0), Sense::hover());
    let resp = ui.interact(r, qat_menu_button_id(), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(r, 4.0, t.hover);
    }
    icons::paint(ui.painter(), Rect::from_center_size(r.center(), vec2(16.0, 16.0)), "dropdown", t.icon, t.accent);
    let resp = resp.on_hover_text(tl!("Customize Quick Access Toolbar"));
    egui::Popup::menu(&resp).show(|ui| {
        ui.set_min_width(220.0);
        qat_menu(ui, app);
    });
}

/// The Undo button's drop-down arrow (#701), which opens [`undo_list`].
pub(crate) fn undo_list_button_id() -> egui::Id {
    egui::Id::new("qat_undo_list")
}

/// Row `i` of the Undo list (0 = the newest step).
fn undo_list_row_id(i: usize) -> egui::Id {
    undo_list_button_id().with(("row", i))
}

/// The footer Cancel row's id (#1903).
pub(crate) fn undo_list_cancel_id() -> egui::Id {
    undo_list_button_id().with("cancel")
}

/// The Undo list's rows, newest first: each step named as the Undo button's tooltip names it.
pub(crate) fn undo_list_rows(app: &WordApp) -> Vec<String> {
    app.session.undo_history().into_iter().map(|l| tl!(l).to_string()).collect()
}

/// The Undo list's footer: how many steps a click undoes, or Cancel while none is highlighted.
pub(crate) fn undo_list_footer(n: usize) -> String {
    match n {
        0 => tl!("Cancel").to_string(),
        1 => tl!("Undo 1 Action").to_string(),
        n => crate::i18n::fmt(tl!("Undo {n} Actions"), &[("n", &n.to_string())]),
    }
}

/// The small arrow beside Undo: opens the list of steps to undo several at once.
fn undo_list_button(ui: &mut Ui, app: &mut WordApp, enabled: bool) {
    let t = Tokens::get(ui.ctx());
    let (r, _) = ui.allocate_exact_size(vec2(12.0, 24.0), Sense::hover());
    let resp = ui.interact(r, undo_list_button_id(), if enabled { Sense::click() } else { Sense::hover() });
    if resp.hovered() && enabled {
        ui.painter().rect_filled(r, 4.0, t.hover);
    }
    let c = if enabled { t.icon } else { t.text_disabled };
    icons::paint(ui.painter(), Rect::from_center_size(r.center(), vec2(9.0, 9.0)), "dropdown", c, c);
    let resp = resp.on_hover_text(tl!("Undo History"));
    egui::Popup::menu(&resp).show(|ui| undo_list(ui, app));
}

/// The Undo list (#701): recent steps, newest first. Hovering a row highlights it and every newer
/// one; clicking it undoes them all in one go (`edit.undoTo`).
fn undo_list(ui: &mut Ui, app: &mut WordApp) {
    const W: f32 = 210.0;
    const ROW: f32 = 20.0;
    let t = Tokens::get(ui.ctx());
    let rows = undo_list_rows(app);
    ui.set_width(W);
    let (mut hovered, mut clicked) = (None, None);
    egui::ScrollArea::vertical().max_height(ROW * 18.0).auto_shrink([false, true]).show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        let rects: Vec<Rect> = (0..rows.len())
            .map(|i| {
                let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::hover());
                let resp = ui.interact(r, undo_list_row_id(i), Sense::click());
                if resp.hovered() {
                    hovered = Some(i);
                }
                if resp.clicked() {
                    clicked = Some(i);
                }
                r
            })
            .collect();
        for (i, (r, label)) in rects.iter().zip(&rows).enumerate() {
            if hovered.is_some_and(|h| i <= h) {
                ui.painter().rect_filled(*r, 0.0, t.selection);
            }
            // Clipped to its row: a long label never runs into the scroll bar.
            ui.painter_at(r.shrink2(vec2(6.0, 0.0))).text(pos2(r.min.x + 6.0, r.center().y), Align2::LEFT_CENTER, label, regular(12.0), t.text);
        }
    });
    ui.separator();
    let n = hovered.map_or(0, |h| h + 1);
    if n == 0 {
        // Cancel is a real row: it closes the popup without undoing anything (#1903).
        let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::hover());
        let resp = ui.interact(r, undo_list_cancel_id(), Sense::click());
        if resp.hovered() {
            ui.painter().rect_filled(r, 0.0, t.selection);
        }
        ui.painter().text(pos2(r.min.x + 6.0, r.center().y), Align2::LEFT_CENTER, undo_list_footer(0), regular(12.0), t.text);
        if resp.clicked() {
            ui.close();
        }
    } else {
        ui.vertical_centered(|ui| ui.label(egui::RichText::new(undo_list_footer(n)).font(regular(12.0)).color(t.text)));
    }
    if let Some(i) = clicked {
        crate::widgets::run_from_ribbon(app, "edit.undoTo", json!({"count": i + 1}));
        ui.close();
    }
}

/// The Quick Access Toolbar's menu button (…), which opens [`qat_menu`].
pub(crate) fn qat_menu_button_id() -> egui::Id {
    egui::Id::new("qat_menu_button")
}

/// The Quick Access Toolbar's menu: common commands to show or hide, and More Commands…
pub(crate) fn qat_menu(ui: &mut Ui, app: &mut WordApp) {
    ui.label(egui::RichText::new(tl!("Customize Quick Access Toolbar")).font(semibold(12.0)));
    ui.separator();
    for id in QAT_MENU {
        let Some(label) = app.session.registry.get(id).map(|s| tl!(s.label).to_string()) else { continue };
        let on = qat_ids(app).contains(&id);
        let mark = egui::RichText::new("✓");
        let mark = if on { mark } else { mark.color(egui::Color32::TRANSPARENT) };
        if ui.add(egui::Button::new((mark, label)).selected(on)).clicked() {
            let key = if on { "qatRemove" } else { "qatAdd" };
            let _ = app.run("tools.customizeRibbon", json!({ key: id }));
        }
    }
    ui.separator();
    if ui.button(tl!("More Commands…")).clicked() {
        crate::dialogs_ribbon::open(app, crate::dialogs_ribbon::Page::QuickAccess);
        ui.close();
    }
    ui.separator();
    // Last, as in Word (#1945); the ribbon's right-click menu brings the toolbar back.
    if ui.button(tl!("Hide Quick Access Toolbar")).clicked() {
        let _ = app.run("ui.quickAccessToolbar", json!({"value": false}));
        ui.close();
    }
}

/// Height of the macOS integrated title bar (#225). AppKit draws the traffic lights itself, centred
/// in the standard 28 pt title-bar band at the top of the window; moving them would take AppKit
/// calls (`unsafe`, which the workspace forbids). So the title bar is that band: the lights sit
/// vertically centred in it, with the controls beside them.
pub const MAC_TITLE_BAR: f32 = 28.0;
/// The traffic lights as AppKit lays them out: button diameter, centre-to-centre pitch and the
/// first button's left edge (8–10 pt depending on the macOS version; the larger value keeps the
/// gap after them from shrinking).
const MAC_LIGHT: f32 = 12.0;
const MAC_LIGHT_PITCH: f32 = 20.0;
const MAC_LIGHTS_LEFT: f32 = 10.0;
/// Where title-bar content starts to the right of the traffic lights (the title bar here, the
/// Backstage back button, #222): the end of the three lights plus the same gap the lights keep
/// from the top of the window.
pub const MAC_CONTENT_LEFT: f32 = MAC_LIGHTS_LEFT + 2.0 * MAC_LIGHT_PITCH + MAC_LIGHT + (MAC_TITLE_BAR - MAC_LIGHT) / 2.0;

/// The macOS title bar as laid out around the traffic lights: its height and where content starts
/// after the lights. The app measures the real buttons (they grew in macOS 26); until then, or if it
/// can't, the standard layout above.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MacTitleBar {
    pub height: f32,
    pub content_left: f32,
}

impl Default for MacTitleBar {
    fn default() -> Self {
        Self { height: MAC_TITLE_BAR, content_left: MAC_CONTENT_LEFT }
    }
}

impl MacTitleBar {
    /// The bar for traffic lights whose right edge is `lights_right` and whose centre is
    /// `lights_centre_y` below the top of the window: the lights centred in it, and content the
    /// same gap after them as they keep from the top. Never smaller than the standard layout.
    pub fn around_lights(lights_right: f32, lights_centre_y: f32, light: f32) -> Self {
        let height = (2.0 * lights_centre_y).max(MAC_TITLE_BAR);
        let gap = (height - light) / 2.0;
        Self { height, content_left: (lights_right + gap).clamp(MAC_CONTENT_LEFT, f32::from(i8::MAX)) }
    }
}

/// The drawn bar keeps this much room below the measured traffic lights: AppKit pins the lights
/// to the standard title-bar position, so the extra height reads as padding under them (they end
/// up just above the bar's centre, which the wider tab row hides).
const MAC_BAR_EXTRA: f32 = 6.0;

/// The tab bar, WPS's window layout: the logo button (the file menu) at the far left, the Home
/// tab (首页), one tab per open document and a + for a new one; the Quick Access Toolbar, AutoSave
/// and the account (Options) button at the right. The bar is also the window's title bar: empty
/// stretches of it drag the window, and a double-click maximizes it.
pub fn tab_bar(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    // macOS: the traffic lights decide the height; elsewhere the bar is its own size.
    let mac = app.integrated_titlebar;
    let bar = app.mac_titlebar;
    let left = if mac { bar.content_left as i8 } else { 8 };
    let control_h = if mac { 26.0 } else { 30.0 };
    egui::Panel::top("tab_bar")
        .exact_size(if mac { bar.height + MAC_BAR_EXTRA } else { 46.0 })
        .frame(egui::Frame::NONE.fill(t.title_bar).inner_margin(egui::Margin { left, right: 10, top: 0, bottom: 0 }))
        .show(ui, |ui| {
            let full = ui.max_rect();
            // Drag the window by the tab bar (integrated title bar on macOS).
            let drag = ui.interact(full, ui.id().with("title_drag"), Sense::click_and_drag());
            if drag.drag_started() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            if drag.double_clicked() {
                let max = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!max));
            }
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing = vec2(2.0, 0.0);
                // Left: the logo (the file menu), Home, the document tabs, +.
                let file_label = tl!("File");
                let file_w = text_width(ui.painter(), file_label, &TypeRung::Control.medium(), 0.0, t.text) + 18.0;
                // WPS words the tab bar's Home tab (首页) differently from the ribbon's Home
                // tab (开始); other languages keep the one word they have.
                let home_label = crate::i18n::tc("homeTab", "Home");
                let home_w = text_width(ui.painter(), home_label, &TypeRung::Control.medium(), TypeRung::Control.tracking(), t.text) + 36.0;
                let infos = app.tab_infos();
                let n = infos.len();
                let title_ws: Vec<f32> = infos
                    .iter()
                    .map(|(title, _, _)| text_width(ui.painter(), title, &TypeRung::Control.medium(), TypeRung::Control.tracking(), t.text))
                    .collect();
                // Room for the label, the padding around it, the unsaved dot and the close ✕.
                let wants: Vec<f32> = title_ws.iter().map(|w| (w + 64.0).clamp(74.0, 190.0)).collect();
                // What is left of the bar once the fixed ends are paid for. The Quick Access
                // Toolbar sits at the right end, with AutoSave — and gives way entirely when the
                // bar is too narrow to hold the tabs beside it (its commands stay reachable
                // through the file menu and their shortcuts).
                let autosave_w = text_width(ui.painter(), tl!("AutoSave"), &regular(11.5), 0.0, t.text_dim) + 42.0;
                let fixed = f32::from(left) + 10.0 + file_w + home_w + 30.0 + 26.0 + autosave_w + 20.0;
                let wants_qat = app.ui.quick_access_toolbar;
                let qat_w = if wants_qat && full.width() - fixed - qat_width(app) > 0.0 { qat_width(app) } else { 0.0 };
                let budget = (full.width() - fixed - qat_w).max(0.0);
                // Tabs that don't fit share the room equally and are cut with …; when even a
                // narrow tab doesn't fit, the last ones wait in the » menu.
                let (shown, widths): (usize, Vec<f32>) = {
                    let sum: f32 = wants.iter().sum();
                    if sum <= budget {
                        (n, wants)
                    } else {
                        let share = budget / n as f32;
                        if share >= 44.0 {
                            (n, wants.iter().map(|w| w.min(share)).collect())
                        } else {
                            let mut acc = 0.0;
                            let mut k = 0;
                            for w in &wants {
                                let ww = w.min(44.0);
                                if acc + ww > budget {
                                    break;
                                }
                                acc += ww;
                                k += 1;
                            }
                            (k.max(1), wants.iter().map(|w| w.min(44.0)).collect())
                        }
                    }
                };
                // The file menu: plain text, as WPS's, opening its menu in place (not a page).
                let (r, _) = ui.allocate_exact_size(vec2(file_w, control_h), Sense::hover());
                let resp = ui.interact(r, egui::Id::new("file_menu_button"), Sense::click());
                let menu_open = egui::Popup::is_id_open(ui.ctx(), egui::Id::new("file_menu_button").with("popup"));
                if resp.hovered() || menu_open {
                    ui.painter().rect_filled(r, 4.0, t.hover);
                }
                // Centered on the galley itself, like the Home tab: a nominal line height
                // sits low with a CJK interface font's taller line box.
                ui.painter().text(r.center(), Align2::CENTER_CENTER, file_label, TypeRung::Control.medium(), t.text);
                resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, file_label));
                if app.ui.keytips == crate::keytips::Phase::Tabs && "F".starts_with(&app.ui.keytip_prefix) {
                    app.keytip_rects.push((r, "F".to_string()));
                }
                let doc = app.has_document();
                egui::Popup::menu(&resp).show(|ui| {
                    ui.set_min_width(216.0);
                    // A drawn row: its own icon, its label, an arrow cursor (a plain label
                    // leaves the page's text-edit cursor showing through the popup).
                    let item = |ui: &mut Ui, label: &str, icon: &str, on: bool| {
                        let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 30.0), if on { Sense::click() } else { Sense::hover() });
                        if on && resp.hovered() {
                            ui.painter().rect_filled(r, 4.0, t.hover);
                        }
                        let ink = if on { t.icon } else { t.text_dim };
                        icons::paint(ui.painter(), Rect::from_center_size(pos2(r.min.x + 16.0, r.center().y), vec2(17.0, 17.0)), icon, ink, t.accent);
                        paint_text(
                            ui.painter(),
                            pos2(r.min.x + 34.0, r.center().y),
                            Align2::LEFT_CENTER,
                            label,
                            &regular(13.0),
                            0.0,
                            if on { t.text } else { t.text_dim },
                        );
                        resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, on, label));
                        if on {
                            let resp = resp.on_hover_cursor(egui::CursorIcon::Default);
                            resp.clicked()
                        } else {
                            false
                        }
                    };
                    if item(ui, tl!("New"), "blankPage", true) {
                        let _ = app.run("ui.home", json!({}));
                        ui.close();
                    }
                    if item(ui, tl!("Open"), "open", true) {
                        let _ = app.run("file.open", json!({}));
                        ui.close();
                    }
                    ui.separator();
                    if item(ui, tl!("Save"), "save", doc) {
                        let _ = app.run("file.save", json!({}));
                        ui.close();
                    }
                    if item(ui, tl!("Save As"), "saveAs", doc) {
                        let _ = app.run("file.saveAs", json!({}));
                        ui.close();
                    }
                    ui.separator();
                    if item(ui, tl!("Print"), "print", doc) {
                        let _ = app.run("ui.dialog", json!({"name": "print"}));
                        ui.close();
                    }
                    if item(ui, tl!("Export"), "export", doc) {
                        let _ = app.run("ui.dialog", json!({"name": "export"}));
                        ui.close();
                    }
                    if item(ui, tl!("Info"), "info", doc) {
                        let _ = app.run("ui.dialog", json!({"name": "info"}));
                        ui.close();
                    }
                    ui.separator();
                    if item(ui, tl!("Close"), "close", doc) {
                        let _ = app.run("file.close", json!({}));
                        ui.close();
                    }
                    if item(ui, tl!("Options"), "options", true) {
                        let _ = app.run("ui.dialog", json!({"name": "options"}));
                        ui.close();
                    }
                    // The recent files, as WPS lists them under its file menu.
                    let recents: Vec<String> = app.ui.recent.iter().take(app.ui.recent_shown()).cloned().collect();
                    if !recents.is_empty() {
                        ui.separator();
                        ui.label(egui::RichText::new(tl!("Recent")).font(medium(11.5)).color(t.text_dim));
                        for path in recents {
                            let name = std::path::Path::new(&path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or(path.clone());
                            let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 28.0), Sense::click());
                            if resp.hovered() {
                                ui.painter().rect_filled(r, 4.0, t.hover);
                            }
                            icons::paint(
                                ui.painter(),
                                Rect::from_center_size(pos2(r.min.x + 16.0, r.center().y), vec2(15.0, 15.0)),
                                "printLayout",
                                t.icon,
                                t.accent,
                            );
                            // The name, cut with … when the path is long.
                            let room = r.width() - 44.0;
                            if text_width(ui.painter(), &name, &regular(13.0), 0.0, t.text) > room {
                                let mut job =
                                    egui::text::LayoutJob::single_section(name.clone(), egui::text::TextFormat::simple(regular(13.0), t.text));
                                job.wrap = egui::text::TextWrapping {
                                    max_width: room.max(1.0),
                                    max_rows: 1,
                                    break_anywhere: true,
                                    overflow_character: Some('…'),
                                };
                                let galley = ui.painter().layout_job(job);
                                ui.painter().galley(pos2(r.min.x + 34.0, r.center().y - galley.size().y / 2.0), galley, t.text);
                            } else {
                                paint_text(ui.painter(), pos2(r.min.x + 34.0, r.center().y), Align2::LEFT_CENTER, &name, &regular(13.0), 0.0, t.text);
                            }
                            let name_label = name.clone();
                            resp.widget_info(move || egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &name_label));
                            let resp = resp.on_hover_cursor(egui::CursorIcon::Default);
                            if resp.clicked() {
                                let _ = app.run("file.open", json!({"path": path}));
                                ui.close();
                            }
                        }
                    }
                });
                // The Home tab (首页).
                let (r, resp) = ui.allocate_exact_size(vec2(home_w, control_h), Sense::click());
                let active = app.home;
                tab_look(ui, r, resp.hovered(), active, &t);
                let label = home_label;
                ui.painter().text(
                    r.center(),
                    Align2::CENTER_CENTER,
                    label,
                    if active { semibold(12.5) } else { TypeRung::Control.medium() },
                    if active { t.on_accent } else { t.text_dim },
                );
                resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, active, label));
                if resp.clicked() {
                    let _ = app.run("ui.home", json!({}));
                }
                // A tab per document, then +. A tab counts as active only while its document is
                // really showing: with the Home tab up, clicking any document tab — its own
                // included — goes back to the document rather than renaming it.
                let showing = !app.home;
                for (i, (title, dirty, active)) in infos.iter().enumerate().take(shown) {
                    let w = widths.get(i).copied().unwrap_or(66.0);
                    let (r, resp) = ui.allocate_exact_size(vec2(w, control_h), Sense::click());
                    let hovered = resp.hovered();
                    let active = *active && showing;
                    tab_look(ui, r, hovered, active, &t);
                    // The close ✕, over the tab's right end.
                    let cx = Rect::from_min_max(pos2(r.max.x - 20.0, r.min.y), r.max);
                    let close = ui.interact(cx, ui.id().with(("tab_close", i)), Sense::click());
                    if close.hovered() {
                        // On the active tab the chip sits on the accent pill, so it lightens the
                        // accent instead of taking the bar's hover fill.
                        ui.painter().rect_filled(cx, 4.0, if active { egui::Color32::WHITE.linear_multiply(0.18) } else { t.hover });
                    }
                    let c = close.rect.center();
                    let x = if active { t.on_accent } else { t.text_dim };
                    ui.painter().line_segment([c + vec2(-3.5, -3.5), c + vec2(3.5, 3.5)], Stroke::new(1.2, x));
                    ui.painter().line_segment([c + vec2(-3.5, 3.5), c + vec2(3.5, -3.5)], Stroke::new(1.2, x));
                    let close = close.on_hover_text(tl!("Close"));
                    if close.clicked() {
                        let _ = app.run("ui.tabClose", json!({"index": i}));
                    }
                    // The unsaved dot, then the title, cut with … when the tab is narrow.
                    let close_room = 20.0;
                    let dot_room = if *dirty { 9.0 } else { 0.0 };
                    if *dirty {
                        ui.painter().circle_filled(pos2(r.min.x + 10.0, r.center().y), 2.5, if active { t.on_accent } else { t.accent });
                    }
                    let room = w - close_room - dot_room - 12.0;
                    let font = if active { semibold(12.0) } else { TypeRung::Control.medium() };
                    let color = if active { t.on_accent } else { t.text_dim };
                    if title_ws.get(i).is_some_and(|tw| *tw > room) {
                        let mut job = egui::text::LayoutJob::single_section(title.clone(), egui::text::TextFormat::simple(font, color));
                        job.wrap =
                            egui::text::TextWrapping { max_width: room.max(1.0), max_rows: 1, break_anywhere: true, overflow_character: Some('…') };
                        let galley = ui.painter().layout_job(job);
                        ui.painter().galley(pos2(r.min.x + 6.0 + dot_room, r.center().y - galley.size().y / 2.0), galley, color);
                    } else {
                        ui.painter().text(pos2(r.min.x + 6.0 + dot_room, r.center().y), Align2::LEFT_CENTER, title, font, color);
                    }
                    // The active tab stands in for the old title: clicking it renames the
                    // document, as clicking the title did; its label carries the file's state.
                    let state = if *dirty {
                        tl!("Edited")
                    } else if app.session.path.is_some() {
                        tl!("Saved")
                    } else {
                        tl!("Not saved")
                    };
                    let label = if active { format!("{title} — {state}") } else { format!("{title}{}", if *dirty { " •" } else { "" }) };
                    resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, active, &label));
                    if resp.middle_clicked() {
                        let _ = app.run("ui.tabClose", json!({"index": i}));
                    } else if resp.clicked() && active {
                        let _ = app.run("file.rename", json!({}));
                    } else if resp.clicked() {
                        let _ = app.run("ui.tabActivate", json!({"index": i}));
                    }
                }
                if shown < n {
                    let (r, resp) = ui.allocate_exact_size(vec2(24.0, control_h), Sense::click());
                    ui.painter().text(r.center(), Align2::CENTER_CENTER, "»", TypeRung::Control.medium(), t.text_dim);
                    if resp.hovered() {
                        ui.painter().rect_filled(r, 4.0, t.hover);
                    }
                    let left_out = infos.iter().enumerate().skip(shown).collect::<Vec<_>>();
                    egui::Popup::menu(&resp).show(|ui| {
                        for (i, (title, dirty, _)) in left_out {
                            if ui.selectable_label(false, format!("{title}{}", if *dirty { " •" } else { "" })).clicked() {
                                let _ = app.run("ui.tabActivate", json!({"index": i}));
                                ui.close();
                            }
                        }
                    });
                }
                let (r, resp) = ui.allocate_exact_size(vec2(26.0, control_h), Sense::click());
                if resp.hovered() {
                    ui.painter().rect_filled(r, 4.0, t.hover);
                }
                icons::paint(ui.painter(), Rect::from_center_size(r.center(), vec2(16.0, 16.0)), "plus", t.text_dim, t.accent);
                let new = tl!("New");
                resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, new));
                if resp.on_hover_text(new).clicked() {
                    let _ = app.run("ui.tabNew", json!({}));
                }
                // Right: the Quick Access Toolbar, AutoSave and the account (Options) button.
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = vec2(2.0, 0.0);
                    // Account: opens Options › General (user name and initials); the options
                    // gear, not the initials avatar.
                    let (r, resp) = ui.allocate_exact_size(vec2(control_h, control_h), Sense::click());
                    if resp.hovered() {
                        ui.painter().rect_filled(r, 4.0, t.hover);
                    }
                    icons::paint(ui.painter(), Rect::from_center_size(r.center(), vec2(17.0, 17.0)), "options", t.icon, t.accent);
                    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tl!("Account")));
                    if resp.on_hover_text(format!("{} — set your name in the Options window", app.session.author)).clicked() {
                        let _ = app.run("ui.dialog", json!({"name": "options", "pane": "general"}));
                    }
                    ui.add_space(8.0);
                    // AutoSave switch: greyed out, with the reason as its tooltip, when AutoSave
                    // can't cover the document (#196, #176); clicking it then offers Save As
                    // where that helps.
                    let block = app.autosave_block();
                    let on = app.autosaves();
                    let label = if block.is_some() { t.text_dim.gamma_multiply(0.6) } else { t.text_dim };
                    let aw = text_width(ui.painter(), tl!("AutoSave"), &regular(11.5), 0.0, label);
                    ui.allocate_ui_with_layout(vec2(aw + 36.0, control_h), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(tl!("AutoSave")).font(regular(11.5)).color(label));
                        let (r, resp) = ui.allocate_exact_size(vec2(30.0, 16.0), Sense::click());
                        let (fill, stroke, knob_fill) = match (&block, on) {
                            (Some(_), _) => (t.input.gamma_multiply(0.5), t.border.gamma_multiply(0.7), t.text_dim.gamma_multiply(0.45)),
                            (None, true) => (t.accent, t.accent, t.on_accent),
                            (None, false) => (t.input, t.border_strong, t.text_dim),
                        };
                        ui.painter().rect(r, 8.0, fill, Stroke::new(1.0, stroke), egui::StrokeKind::Inside);
                        let knob = if on { r.max.x - 8.0 } else { r.min.x + 8.0 };
                        ui.painter().circle_filled(pos2(knob, r.center().y), 5.0, knob_fill);
                        let tip = block
                            .as_ref()
                            .map_or_else(|| tl!("AutoSave saves every change to the file (needs a saved document)").to_string(), |b| b.reason());
                        if resp.on_hover_text(tip).clicked() {
                            match block {
                                None => {
                                    let _ = app.run("file.autosave", json!({}));
                                }
                                // AutoSave comes on once Save As has saved (maybe on a later
                                // frame, #94).
                                Some(b) if b.save_as_helps() => {
                                    let _ = app.save_as(crate::file_dialogs::AfterSave::AutoSave);
                                }
                                Some(b) => app.status(b.reason()),
                            }
                        }
                    });
                    ui.add_space(8.0);
                    // The Quick Access Toolbar, laid out left to right inside the right-aligned
                    // end of the bar (absent only when the bar is too narrow for it, or hidden
                    // by choice).
                    if qat_w > 0.0 {
                        let qw = qat_width(app);
                        ui.allocate_ui_with_layout(vec2(qw, control_h), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.spacing_mut().item_spacing = vec2(2.0, 0.0);
                            quick_access(ui, app);
                        });
                    }
                });
            });
        });
}

/// A tab's background: the active one is a solid accent pill (its label and ✕ switch to the
/// on-accent colour), the hovered one gains a soft fill.
fn tab_look(ui: &mut Ui, r: Rect, hovered: bool, active: bool, t: &Tokens) {
    if active {
        ui.painter().rect_filled(r, 7.0, t.accent);
    } else if hovered {
        ui.painter().rect_filled(r.shrink2(vec2(0.0, 3.0)), 5.0, t.hover);
    }
}

/// The Quick Access Toolbar's width, measured as the tab bar reserves room for it.
fn qat_width(app: &WordApp) -> f32 {
    let ids = qat_ids(app);
    let n = ids.len() as f32;
    let undo = f32::from(ids.contains(&"edit.undo"));
    26.0 * n + 26.0 + 2.0 * (n + 1.0) + 14.0 * undo
}

pub fn status_bar(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::bottom("status")
        .exact_size(26.0)
        .frame(
            egui::Frame::NONE
                .fill(t.status_bar)
                .inner_margin(egui::Margin { left: 12, right: 12, top: 0, bottom: 0 })
                .stroke(Stroke::new(1.0, t.border)),
        )
        .show(ui, |ui| {
            // No document open (#1129): an empty status bar.
            if !app.has_document() {
                return;
            }
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing = vec2(14.0, 0.0);
                let l = app.session.layout();
                let page = if app.session.selects_whole_story() {
                    app.session.page_hint.min(l.pages.len().saturating_sub(1)) + 1
                } else {
                    l.caret_on(&app.session.sel.focus, app.session.page_hint).map(|c| c.page + 1).unwrap_or(1)
                };
                let st = |ui: &mut Ui, s: &str| {
                    let font = TypeRung::Caption.regular();
                    let track = TypeRung::Caption.tracking();
                    let w = text_width(ui.painter(), s, &font, track, t.text_dim);
                    let (r, resp) = ui.allocate_exact_size(vec2(w + 1.0, TypeRung::Caption.line_height()), Sense::click());
                    paint_text(ui.painter(), pos2(r.min.x, r.min.y), Align2::LEFT_TOP, s, &font, track, t.text_dim);
                    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, s));
                    resp
                };
                if st(ui, &crate::i18n::fmt(tl!("Page: {page}/{pages}"), &[("page", &page.to_string()), ("pages", &l.pages.len().to_string())]))
                    .on_hover_text(tl!("Go To (⌘⌥G)"))
                    .clicked()
                {
                    let _ = app.run("ui.dialog", json!({"name": "goto"}));
                }
                let words = app.cached_word_count();
                // A selected picture/shape/text box isn't a text selection: show the total.
                let wtxt = if app.session.sel.is_collapsed() || crate::objects::selected(app).is_some() {
                    crate::i18n::fmt(tl!("Words: {words}"), &[("words", &words.to_string())])
                } else {
                    let sw = wordcraft_doc::count_words(&app.session.selected_text());
                    crate::i18n::fmt(tl!("Words: {selected}/{words}"), &[("selected", &sw.to_string()), ("words", &words.to_string())])
                };
                if st(ui, &wtxt).clicked() {
                    let _ = app.run("ui.dialog", json!({"name": "wordCount"}));
                }
                if app.session.sel.focus.story != StoryRef::Body && !crate::canvas::in_text_box(app) {
                    st(ui, tl!("Editing header/footer"));
                }
                // The language at the caret; a click sets the proofing language (Review › Language).
                let lang = crate::ribbon::caret_language(app);
                let missing = crate::ribbon::caret_missing_dictionary(app);
                let unreadable = crate::ribbon::caret_unreadable_dictionary(app);
                let shown = if unreadable.is_some() {
                    crate::i18n::fmt(tl!("{language} (dictionary unreadable)"), &[("language", &lang)])
                } else if missing.is_some() {
                    crate::i18n::fmt(tl!("{language} (no dictionary)"), &[("language", &lang)])
                } else {
                    lang.clone()
                };
                let tip = match (&unreadable, missing) {
                    (Some((file, reason)), _) => crate::i18n::fmt(
                        tl!("The spelling dictionary for {language} couldn't be read, so it isn't checked. File: {file}. Reason: {reason}"),
                        &[("language", &lang), ("file", &file.to_string_lossy()), ("reason", reason)],
                    ),
                    (None, Some(name)) => crate::i18n::fmt(
                        tl!("No spelling dictionary for {language} is installed, so it isn't checked. WBS looks for the Hunspell dictionary {name} in the WORDCRAFT_DICTIONARIES folder and in the usual system locations."),
                        &[("language", &lang), ("name", name)],
                    ),
                    (None, None) => tl!("Language").to_string(),
                };
                let resp = st(ui, &shown).on_hover_text(tip);
                egui::Popup::menu(&resp).show(|ui| {
                    ui.set_min_width(200.0);
                    crate::ribbon::language_menu(ui, app);
                });
                if app.session.doc.settings.track_changes {
                    st(ui, tl!("Track Changes: On"));
                }
                if app.session.prefs.overtype && st(ui, tl!("Overtype")).on_hover_text(tl!("Click to switch back to inserting")).clicked() {
                    let _ = app.run("text.overtype", json!({"value": false}));
                }
                if let Some((msg, at)) = &app.status_msg
                    && crate::now_ms() - at < 6000.0
                {
                    let font = TypeRung::Caption.regular();
                    let track = TypeRung::Caption.tracking();
                    paint_text(ui.painter(), pos2(ui.cursor().min.x, ui.cursor().min.y), Align2::LEFT_TOP, msg, &font, track, t.accent_text);
                    ui.allocate_space(vec2(text_width(ui.painter(), msg, &font, track, t.accent_text) + 1.0, TypeRung::Caption.line_height()));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
                    // Full Screen, the bar's last entry (as WPS's is): toggles view.fullScreen.
                    {
                        let on = app.session.view.full_screen;
                        let name = tl!("Full Screen");
                        let (r, resp) = ui.allocate_exact_size(vec2(22.0, 20.0), Sense::click());
                        if on {
                            ui.painter().rect_filled(r, 3.0, t.checked);
                        } else if resp.hovered() {
                            ui.painter().rect_filled(r, 3.0, t.hover);
                        }
                        icons::paint(ui.painter(), r.shrink(3.0), "fullScreen", t.icon, t.accent);
                        resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, on, name));
                        if resp.on_hover_text(name).clicked() {
                            let _ = app.run("view.fullScreen", json!({}));
                        }
                        ui.add_space(6.0);
                    }
                    let reading = app.session.view.read_mode;
                    let z = (if reading { app.session.view.read_zoom } else { app.canvas.scale / crate::canvas::PX_PER_PT } * 100.0).round();
                    let ztxt = format!("{z}%");
                    let zfont = TypeRung::Caption.regular();
                    let ztrack = TypeRung::Caption.tracking();
                    let zw = text_width(ui.painter(), &ztxt, &zfont, ztrack, t.text_dim);
                    // A fixed width for three digits plus a small reserve, so the controls to the
                    // left don't shift as the percentage gains or loses digits (#1975).
                    let zslot = text_width(ui.painter(), "000%", &zfont, ztrack, t.text_dim) + 4.0;
                    let (zr, zresp) = ui.allocate_exact_size(vec2(zw.max(zslot) + 1.0, TypeRung::Caption.line_height()), Sense::click());
                    paint_text(ui.painter(), pos2(zr.max.x - 1.0, zr.min.y), Align2::RIGHT_TOP, &ztxt, &zfont, ztrack, t.text_dim);
                    if zresp.clicked() && !reading {
                        let _ = app.run("ui.dialog", json!({ "name": "zoom" }));
                    }
                    if small_icon(ui, "plus", tl!("Zoom In")) {
                        if reading {
                            let _ = app.run("view.readTextSize", json!({"value": (z + 10.0).min(150.0)}));
                        } else {
                            let _ = app.run("view.zoomIn", json!({}));
                        }
                    }
                    // Reading zoom changes only reading presentation, never the normal view zoom.
                    let mut v = if reading { app.session.view.read_zoom } else { app.canvas.scale / crate::canvas::PX_PER_PT };
                    let slider = egui::Slider::new(&mut v, if reading { 0.75..=1.5 } else { 0.1..=5.0 }).logarithmic(!reading).show_value(false);
                    ui.spacing_mut().slider_width = 110.0;
                    if ui.add(slider).changed() {
                        let command = if reading { "view.readTextSize" } else { "view.zoom" };
                        let _ = app.run(command, json!({"value": (v * 100.0).round()}));
                    }
                    if small_icon(ui, "minus", tl!("Zoom Out")) {
                        if reading {
                            let _ = app.run("view.readTextSize", json!({"value": (z - 10.0).max(75.0)}));
                        } else {
                            let _ = app.run("view.zoomOut", json!({}));
                        }
                    }
                    ui.add_space(10.0);
                    let mode = app.session.view.mode;
                    let read = app.session.view.read_mode;
                    for (icon, id, name, on) in [
                        ("webLayout", "view.webLayout", tl!("Web Layout"), mode == wordcraft_layout::ViewMode::Web),
                        ("printLayout", "view.printLayout", tl!("Print Layout"), mode == wordcraft_layout::ViewMode::Print && !read),
                        ("readMode", "view.readMode", tl!("Read Mode"), read),
                    ] {
                        let (r, resp) = ui.allocate_exact_size(vec2(22.0, 20.0), Sense::click());
                        if on {
                            ui.painter().rect_filled(r, 3.0, t.checked);
                        } else if resp.hovered() {
                            ui.painter().rect_filled(r, 3.0, t.hover);
                        }
                        icons::paint(ui.painter(), r.shrink(3.0), icon, t.icon, t.accent);
                        resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, on, name));
                        if resp.on_hover_text(name).clicked() {
                            let _ = app.run(id, json!({}));
                        }
                    }
                    ui.add_space(8.0);
                    let focus = tl!("Focus");
                    let focus_w = ui.ctx().fonts_mut(|f| f.layout_no_wrap(focus.to_string(), regular(11.5), t.text_dim).size().x);
                    let (r, resp) = ui.allocate_exact_size(vec2((focus_w + 26.0).max(56.0), 20.0), Sense::click());
                    if resp.hovered() {
                        ui.painter().rect_filled(r, 3.0, t.hover);
                    }
                    icons::paint(ui.painter(), Rect::from_min_size(r.min + vec2(2.0, 2.0), vec2(16.0, 16.0)), "focus", t.icon, t.accent);
                    let ffont = TypeRung::Caption.regular();
                    let ftrack = TypeRung::Caption.tracking();
                    paint_text(
                        ui.painter(),
                        pos2(r.min.x + 22.0, r.center().y - TypeRung::Caption.line_height() / 2.0),
                        Align2::LEFT_TOP,
                        focus,
                        &ffont,
                        ftrack,
                        t.text_dim,
                    );
                    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, focus));
                    if resp.clicked() {
                        let _ = app.run("view.focus", json!({}));
                    }
                });
            });
        });
}

fn small_icon(ui: &mut Ui, icon: &str, tip: &str) -> bool {
    let t = Tokens::get(ui.ctx());
    let (r, resp) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(r, 3.0, t.hover);
    }
    icons::paint(ui.painter(), r.shrink(3.0), icon, t.icon, t.accent);
    resp.on_hover_text(tip).clicked()
}

impl WordApp {
    /// Word count, recomputed only when the document changes.
    pub fn cached_word_count(&mut self) -> usize {
        // The count option changes the count without a document change.
        let key = self.session.rev().wrapping_mul(2) | u64::from(self.session.prefs.count_notes);
        if self.word_count.0 != key {
            self.word_count = (key, self.session.word_count());
        }
        self.word_count.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> egui::Context {
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        crate::theme::apply(&ctx, crate::theme::Appearance::Light);
        ctx.run_ui(egui::RawInput::default(), |_| {}).drop_without_applying_deltas();
        ctx
    }

    fn frame(ctx: &egui::Context, n: &mut u32, events: Vec<egui::Event>, a: &mut WordApp) {
        *n += 1;
        // A window-sized screen: the bar's right end (AutoSave, the account) is laid out from the
        // right edge, which an unbounded default screen would push off any real window.
        let screen = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1440.0, 900.0));
        let input =
            egui::RawInput { time: Some(f64::from(*n) / 60.0), predicted_dt: 1.0 / 60.0, events, screen_rect: Some(screen), ..Default::default() };
        ctx.run_ui(input, |ui| tab_bar(a, ui)).drop_without_applying_deltas();
    }

    fn click(ctx: &egui::Context, n: &mut u32, pos: egui::Pos2, a: &mut WordApp) {
        let button = |pressed| egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() };
        frame(ctx, n, vec![egui::Event::PointerMoved(pos), button(true)], a);
        frame(ctx, n, vec![button(false)], a);
    }

    /// #701: the arrow beside Undo lists the session's undo steps, newest first; clicking the third
    /// row undoes three steps at once.
    #[test]
    fn undo_list_matches_the_session_and_undoes_up_to_the_clicked_row() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        a.run("text.insert", json!({"text": "Hello"})).unwrap();
        a.run("select.all", json!({})).unwrap();
        a.run("format.bold", json!({})).unwrap();
        a.run("format.italic", json!({})).unwrap();
        a.run("caret.docEnd", json!({})).unwrap();
        a.run("text.insert", json!({"text": " world"})).unwrap();
        let session: Vec<String> = a.session.undo_history().into_iter().map(str::to_string).collect();
        assert_eq!(session, ["Typing", "Italic", "Bold", "Typing"]);
        assert_eq!(undo_list_rows(&a), session);
        assert_eq!(undo_list_footer(0), "Cancel");
        assert_eq!(undo_list_footer(1), "Undo 1 Action");
        assert_eq!(undo_list_footer(3), "Undo 3 Actions");

        let (ctx, mut n) = (ctx(), 0);
        frame(&ctx, &mut n, vec![], &mut a);
        let arrow = ctx.read_response(undo_list_button_id()).unwrap();
        let popup = egui::Popup::default_response_id(&arrow);
        click(&ctx, &mut n, arrow.rect.center(), &mut a);
        assert!(egui::Popup::is_id_open(&ctx, popup), "the arrow opens the Undo list");
        frame(&ctx, &mut n, vec![], &mut a);
        for i in 0..4 {
            assert!(ctx.read_response(undo_list_row_id(i)).is_some(), "row {i} is drawn");
        }
        assert!(ctx.read_response(undo_list_row_id(4)).is_none(), "one row per step");
        // The popup's place settles one frame after it opens (egui anchors it on a first guess
        // before its width is measured), so let a frame pass before reading where the rows are.
        frame(&ctx, &mut n, vec![], &mut a);
        let third = ctx.read_response(undo_list_row_id(2)).unwrap();
        click(&ctx, &mut n, third.rect.center(), &mut a);
        assert_eq!(a.session.undo_history(), ["Typing"], "three steps undone in one go");
        assert_eq!(a.session.doc.plain_text(wordcraft_doc::StoryRef::Body), "Hello");
        frame(&ctx, &mut n, vec![], &mut a);
        assert!(!egui::Popup::is_id_open(&ctx, popup), "the list closes");
    }

    /// #1975: the zoom percentage gains and loses digits (9% → 100% → 500%); the controls to its
    /// left must not shift with it.
    #[test]
    fn status_bar_controls_hold_still_across_zoom_levels() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        let ctx = ctx();
        ctx.enable_accesskit();
        let names = ["Focus", "Print Layout", "Web Layout"];
        let mut seen: Vec<(f32, Vec<f64>)> = Vec::new();
        for (n, zoom) in [10.0_f32, 9.0, 100.0, 500.0].into_iter().enumerate() {
            a.canvas.scale = zoom / 100.0 * crate::canvas::PX_PER_PT;
            let input = egui::RawInput {
                time: Some(n as f64 / 60.0),
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(1200.0, 800.0))),
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ui| status_bar(&mut a, ui));
            out.textures_delta.clear();
            let update = out.platform_output.accesskit_update.take().unwrap();
            let xs: Vec<f64> = names
                .iter()
                .map(|name| {
                    let node =
                        update.nodes.iter().find(|(_, node)| node.label() == Some(*name)).unwrap_or_else(|| panic!("{name} is drawn")).1.clone();
                    node.bounds().unwrap().x0
                })
                .collect();
            seen.push((zoom, xs));
        }
        let first = &seen[0].1;
        for (zoom, xs) in &seen {
            assert_eq!(xs, first, "status bar controls moved at {zoom}% zoom");
        }
    }

    /// The status bar's WPS wording and its last entry: `Page: 1/1`, `Words: N` (a selection
    /// `Words: S/N`), and a Full Screen toggle that runs `view.fullScreen`.
    #[test]
    fn status_bar_shows_wps_wording_and_full_screen() {
        use egui_kittest::kittest::Queryable;
        let mut h = {
            let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::from_text("Hello world")), crate::Services::default());
            let _ = a.run("select.all", json!({}));
            egui_kittest::Harness::builder().with_size(egui::vec2(1200.0, 800.0)).build_ui_state(
                |ui, app: &mut WordApp| {
                    let ctx = ui.ctx().clone();
                    app.logic(&ctx);
                    app.ui(ui);
                },
                a,
            )
        };
        h.run_steps(6);
        assert!(h.query_by_label("Page: 1/1").is_some(), "the page reads Page: 1/1");
        assert!(h.query_by_label("Words: 2/2").is_some(), "a selection reads Words: S/N");
        h.state_mut().run("select.collapse", json!({"end": true})).unwrap();
        h.run_steps(3);
        assert!(h.query_by_label("Words: 2").is_some(), "no selection reads Words: N");
        assert!(!h.state().session.view.full_screen, "full screen starts off");
        h.get_by_label("Full Screen").click();
        h.run_steps(3);
        assert!(h.state().session.view.full_screen, "the status bar's last button toggles full screen");
    }

    /// #1903: the footer's Cancel row closes the Undo list without undoing anything.
    #[test]
    fn undo_list_cancel_closes_without_undoing() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        a.run("text.insert", json!({"text": "Hello"})).unwrap();
        let before: Vec<String> = a.session.undo_history().into_iter().map(str::to_string).collect();
        assert!(!before.is_empty());

        let (ctx, mut n) = (ctx(), 0);
        frame(&ctx, &mut n, vec![], &mut a);
        let arrow = ctx.read_response(undo_list_button_id()).unwrap();
        let popup = egui::Popup::default_response_id(&arrow);
        click(&ctx, &mut n, arrow.rect.center(), &mut a);
        assert!(egui::Popup::is_id_open(&ctx, popup), "the arrow opens the Undo list");
        frame(&ctx, &mut n, vec![], &mut a);
        let cancel = ctx.read_response(undo_list_cancel_id()).expect("Cancel is drawn");
        click(&ctx, &mut n, cancel.rect.center(), &mut a);
        let after: Vec<String> = a.session.undo_history().into_iter().map(str::to_string).collect();
        assert_eq!(after, before, "nothing undone");
        frame(&ctx, &mut n, vec![], &mut a);
        assert!(!egui::Popup::is_id_open(&ctx, popup), "Cancel closes the list");
    }

    #[test]
    fn mac_title_bar_centres_and_clears_the_traffic_lights() {
        // The lights are centred in the bar, and content keeps the same gap after them.
        let margin = (MAC_TITLE_BAR - MAC_LIGHT) / 2.0;
        let lights_end = MAC_LIGHTS_LEFT + 2.0 * MAC_LIGHT_PITCH + MAC_LIGHT;
        assert_eq!(MAC_CONTENT_LEFT - lights_end, margin);
        // `egui::Margin` is in `i8`.
        assert!(MAC_CONTENT_LEFT <= f32::from(i8::MAX));
    }

    #[test]
    fn mac_title_bar_follows_the_measured_lights() {
        // The standard lights give the standard bar.
        let lights_end = MAC_LIGHTS_LEFT + 2.0 * MAC_LIGHT_PITCH + MAC_LIGHT;
        assert_eq!(MacTitleBar::around_lights(lights_end, MAC_TITLE_BAR / 2.0, MAC_LIGHT), MacTitleBar::default());
        // Larger, lower lights (macOS 26): a taller bar, and content clear of the green button.
        let bar = MacTitleBar::around_lights(78.0, 18.0, 14.0);
        assert_eq!(bar.height, 36.0);
        assert_eq!(bar.content_left, 89.0);
        // Never tighter than the standard layout.
        assert_eq!(MacTitleBar::around_lights(40.0, 10.0, 12.0), MacTitleBar::default());
    }

    /// #1945: Hide Quick Access Toolbar, the last item of the toolbar's menu, takes the toolbar's
    /// buttons and its … button out of the title bar, which keeps its title; the command brings
    /// them back.
    #[test]
    fn hide_quick_access_toolbar_takes_the_toolbar_out_of_the_title_bar() {
        use egui_kittest::kittest::Queryable;
        let a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                // The fonts `logic` installs take effect on the next frame, as `WordApp::ui` waits.
                if app.fonts_frames < 2 {
                    app.fonts_frames += 1;
                    return;
                }
                tab_bar(app, ui);
            },
            a,
        );
        h.run_steps(4);
        let drawn = |h: &egui_kittest::Harness<'static, WordApp>, id: &str| h.ctx.read_response(qat_button_id(id)).is_some();
        for id in ["file.save", "edit.undo", "edit.redo"] {
            assert!(drawn(&h, id), "{id} is on the toolbar");
        }
        let at = h.ctx.read_response(qat_menu_button_id()).expect("the … button").rect.center();
        h.event(egui::Event::PointerMoved(at));
        for pressed in [true, false] {
            h.event(egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() });
        }
        h.run_steps(2);
        h.get_by_label("Hide Quick Access Toolbar").click();
        // `read_response` also reads the frame before last, so run two frames past the change.
        h.run_steps(3);
        assert!(!h.state().ui.quick_access_toolbar, "the menu's last item hides the toolbar");
        assert!(h.query_by_label("Hide Quick Access Toolbar").is_none(), "and closes the menu");
        for id in ["file.save", "edit.undo", "edit.redo"] {
            assert!(!drawn(&h, id), "{id} is gone");
        }
        assert!(h.ctx.read_response(qat_menu_button_id()).is_none(), "so is the … button");
        assert!(h.query_by_label_contains("Not saved").is_some(), "the title is still drawn");
        h.state_mut().run("ui.quickAccessToolbar", json!({"value": true})).unwrap();
        h.run_steps(2);
        for id in ["file.save", "edit.undo", "edit.redo"] {
            assert!(drawn(&h, id), "{id} is back");
        }
        assert!(h.ctx.read_response(qat_menu_button_id()).is_some());
    }

    /// The Quick Access Toolbar sits at the bar's right end, beside AutoSave (the account/Options
    /// button at the very corner), with WPS's default set (Save, Export PDF, Print, Undo, Redo)
    /// until the user customizes it; a stored customization keeps showing exactly what the user
    /// saved, and the buttons still work there.
    #[test]
    fn quick_access_sits_right_with_wps_default_set() {
        use egui_kittest::kittest::Queryable;
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        a.run("text.insert", json!({"text": "Hello"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                if app.fonts_frames < 2 {
                    app.fonts_frames += 1;
                    return;
                }
                tab_bar(app, ui);
            },
            a,
        );
        h.run_steps(4);
        let account = h.get_by_label("Account").rect();
        let home = h.get_by_label("Home").rect();
        let ids = ["file.save", "file.exportPdf", "file.print", "edit.undo", "edit.redo"];
        for id in ids {
            let r = h.ctx.read_response(qat_button_id(id)).unwrap_or_else(|| panic!("{id} is on the toolbar")).rect;
            assert!(r.min.x > home.max.x && r.max.x < account.min.x, "{id} sits at the right end, beside AutoSave");
        }
        let customize = h.ctx.read_response(qat_menu_button_id()).expect("the customize button").rect;
        let undo = h.ctx.read_response(qat_button_id("edit.undo")).unwrap().rect;
        assert!(customize.min.x > undo.max.x, "the customize button follows the commands");
        // The buttons work from their new place: the Undo button undoes the typing.
        let at = undo.center();
        h.event(egui::Event::PointerMoved(at));
        for pressed in [true, false] {
            h.event(egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() });
        }
        h.run_steps(3);
        assert_eq!(h.state().session.doc.plain_text(wordcraft_doc::StoryRef::Body), "", "the toolbar's Undo ran");
        // A user's saved customization replaces the default set as is.
        h.state_mut().session.ribbon.qat = Some(vec!["edit.cut".to_string()]);
        h.run_steps(3);
        assert!(h.ctx.read_response(qat_button_id("file.save")).is_none(), "the default set is gone");
        assert!(h.ctx.read_response(qat_button_id("edit.cut")).is_some(), "the saved command shows");
    }

    /// With the Home tab showing, clicking the active document's own tab goes back to the
    /// document (nothing to activate, so it must not be read as the rename click it is when the
    /// document is showing); with the document showing, the same click renames it.
    #[test]
    fn clicking_the_active_tab_from_home_returns_to_the_document() {
        use egui_kittest::kittest::Queryable;
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        let _ = a.run("ui.home", json!({}));
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(6);
        assert!(h.state().home, "the Home tab shows");
        h.get_by_label("Document1").click();
        h.run_steps(3);
        assert!(!h.state().home, "the click returned to the document");
        assert!(h.state().dialog.is_none(), "it renamed nothing");
        // With the document showing, the same click is the rename the title bar used to offer.
        h.get_by_label("Document1 — Not saved").click();
        h.run_steps(3);
        assert!(matches!(h.state().dialog, Some(crate::dialogs::Dialog::RenameDocument { .. })));
    }

    /// #3040: clicking a row in the Undo list hands the keyboard back to the document.
    #[test]
    fn undo_list_row_returns_focus_to_the_document() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        a.run("text.insert", json!({"text": "Hello"})).unwrap();
        assert_eq!(a.session.undo_history().len(), 1);

        let (ctx, mut n) = (ctx(), 0);
        frame(&ctx, &mut n, vec![], &mut a);
        let arrow = ctx.read_response(undo_list_button_id()).unwrap();
        let popup = egui::Popup::default_response_id(&arrow);
        click(&ctx, &mut n, arrow.rect.center(), &mut a);
        assert!(egui::Popup::is_id_open(&ctx, popup), "the arrow opens the Undo list");
        frame(&ctx, &mut n, vec![], &mut a);
        // A frame for the popup's place to settle (see the test above).
        frame(&ctx, &mut n, vec![], &mut a);
        let row = ctx.read_response(undo_list_row_id(0)).unwrap();
        a.canvas.want_focus = false;
        click(&ctx, &mut n, row.rect.center(), &mut a);
        assert!(a.canvas.want_focus, "the row click returns focus to the document");
        assert!(a.session.undo_history().is_empty(), "the step was undone");
    }
}
