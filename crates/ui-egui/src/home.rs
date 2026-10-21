//! The Home tab (WPS's 首页): with it showing, the editor area becomes a start page — create a
//! document from a template, reopen a recent file, or browse for one. It reuses the Backstage's
//! Home page ([`crate::backstage::home_page`]), which opens every document in its own tab
//! ([`WordApp::open_in_tab`]).

use egui::Ui;
use serde_json::json;

use crate::WordApp;
use crate::theme::{Tokens, medium};

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
                    crate::backstage::home_page(app, ui);
                    ui.add_space(18.0);
                    // A file the Recent list doesn't know.
                    if ui.button(egui::RichText::new(tl!("Browse…")).font(medium(14.0))).clicked() {
                        let _ = app.run("ui.openFileDialog", json!({}));
                    }
                    ui.add_space(48.0);
                });
            });
        });
    });
}
