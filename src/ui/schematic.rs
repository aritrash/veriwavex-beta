/* src/ui/schematic.rs */
use eframe::egui;
use crate::app::VerilogApp;
use crate::logic;

pub fn draw_schematic_window(app: &mut VerilogApp, ctx: &egui::Context) {
    let mut is_open = app.show_schematic;

    egui::Window::new("📊 RTL Schematic Viewer")
        .open(&mut is_open)
        .default_size([1000.0, 700.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("🚀 Synthesize Logic").clicked() {
                    if let Some(file) = &app.active_file {
                        app.console_output += &format!("Starting synthesis for {}...\n", file.display());
                        match logic::generate_schematic(file) {
                            Ok(_path) => {
                                app.console_output += "SUCCESS: Schematic generated.\n";
                                // Trigger a repaint to show the new image
                                ctx.request_repaint();
                            }
                            Err(e) => app.console_output += &format!("SYNTHESIS ERROR: {}\n", e),
                        }
                    }
                }
                ui.label(egui::RichText::new("(Powered by Yosys & Graphviz)").weak().size(10.0));
            });

            ui.separator();

            egui::ScrollArea::both().show(ui, |ui| {
                if let Some(file) = &app.active_file {
                    let png_path = file.with_extension("png");
                    if png_path.exists() {
                        // Use the image loaders you installed in main.rs
                        let uri = format!("file://{}", png_path.to_string_lossy());
                        ui.add(egui::Image::new(uri).shrink_to_fit());
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label("No schematic generated yet. Click 'Synthesize' to begin.");
                        });
                    }
                }
            });
        });

    app.show_schematic = is_open;
}