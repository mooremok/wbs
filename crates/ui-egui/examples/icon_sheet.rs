//! Renders every icon in the set at several sizes for visual review:
//!
//! `cargo run -p wordcraft-ui-egui --example icon_sheet -- out.png`
//!
//! Set `DARK=1` for the dark theme. Headless (wgpu), so it works with a locked screen.

use std::sync::Arc;

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "/tmp/icon-sheet.png".to_string());
    let dark = std::env::var_os("DARK").is_some();
    let rows = wordcraft_ui_egui::icons::NAMES.len().div_ceil(10);
    let mut harness =
        egui_kittest::Harness::builder().with_size(egui::vec2(1240.0, 20.0 + rows as f32 * 84.0 + 20.0)).with_max_steps(50).wgpu().build_ui_state(
            |ui, _: &mut ()| {
                ui.ctx().set_theme(if dark { egui::Theme::Dark } else { egui::Theme::Light });
                let theme = ui.ctx().theme();
                let t = wordcraft_ui_egui::theme::Tokens::get(ui.ctx());
                let mut style = ui.ctx().style_of(theme);
                Arc::make_mut(&mut style).visuals.panel_fill = t.ribbon; // what knocked-out holes punch through
                ui.ctx().set_style_of(theme, style);
                egui::CentralPanel::default().frame(egui::Frame::NONE.fill(t.ribbon)).show(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                    for row in wordcraft_ui_egui::icons::NAMES.chunks(10) {
                        ui.horizontal(|ui| {
                            for name in row {
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(124.0, 84.0), egui::Sense::hover());
                                let p = ui.painter();
                                p.rect_stroke(rect, 2.0, egui::Stroke::new(0.5, t.border), egui::StrokeKind::Inside);
                                let cy = rect.center().y - 9.0;
                                for (k, sz) in [16.0, 20.0, 24.0, 32.0].into_iter().enumerate() {
                                    let r = egui::Rect::from_center_size(egui::pos2(rect.min.x + 18.0 + 26.0 * k as f32, cy), egui::vec2(sz, sz));
                                    wordcraft_ui_egui::icons::paint(p, r, name, t.icon, t.accent);
                                }
                                p.text(
                                    egui::pos2(rect.center().x, rect.max.y - 10.0),
                                    egui::Align2::CENTER_CENTER,
                                    name,
                                    egui::FontId::monospace(9.0),
                                    t.text_dim,
                                );
                            }
                        });
                    }
                });
            },
            (),
        );
    harness.input_mut().max_texture_side = Some(8192);
    for _ in 0..4 {
        harness.step();
    }
    match harness.render() {
        Ok(img) => {
            img.save(&out).expect("save png");
            println!("{out}");
        }
        Err(e) => eprintln!("render failed: {e}"),
    }
}
