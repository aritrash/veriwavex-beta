use eframe::egui;
use std::fs;
use crate::app::VerilogApp;
use crate::models::WizardType;

pub fn draw_file_wizard(app: &mut VerilogApp, ctx: &egui::Context) {
    egui::Window::new("New Verilog File")
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut app.wiz_type, WizardType::Module, "Module");
                ui.selectable_value(&mut app.wiz_type, WizardType::Testbench, "Testbench");
            });
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label("Component Name:");
                ui.text_edit_singleline(&mut app.wiz_mod_name);
            });

            if app.wiz_type == WizardType::Module {
                ui.separator();
                ui.label("Define Pins:");
                let mut to_remove = None;
                for (i, (name, dir)) in app.wiz_pins.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(name);
                        egui::ComboBox::from_id_salt(i)
                            .selected_text(dir.clone())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(dir, "input".into(), "input");
                                ui.selectable_value(dir, "output".into(), "output");
                            });
                        if ui.button("❌").clicked() { to_remove = Some(i); }
                    });
                }
                if let Some(i) = to_remove { app.wiz_pins.remove(i); }
                if ui.button("➕ Add Pin").clicked() {
                    app.wiz_pins.push(("new_pin".into(), "input".into()));
                }
            }

            ui.add_space(20.0);
            ui.horizontal(|ui| {
                if ui.button("✅ Create").clicked() {
                    let generated_code = match app.wiz_type {
                        WizardType::Module => {
                            let mut s = format!("module {} (\n", app.wiz_mod_name);
                            for (i, (n, d)) in app.wiz_pins.iter().enumerate() {
                                let comma = if i == app.wiz_pins.len() - 1 { "" } else { "," };
                                s.push_str(&format!("    {} {}{}\n", d, n, comma));
                            }
                            s.push_str(");\n\n// Logic here\n\nendmodule");
                            s
                        },
                        WizardType::Testbench => {
                            format!("module {}_tb;\nreg clk;\n\n{} uut (.clk(clk));\n\ninitial begin\n    $dumpfile(\"dump.vcd\");\n    $dumpvars(0, {}_tb);\n    clk = 0; #100 $finish;\nend\n\nalways #5 clk = ~clk;\nendmodule", 
                                app.wiz_mod_name, app.wiz_mod_name, app.wiz_mod_name)
                        }
                    };

                    if let Some(proj) = &mut app.project {
                        let ext = if app.wiz_type == WizardType::Testbench { "_tb.v" } else { ".v" };
                        let path = proj.root.join(format!("{}{}", app.wiz_mod_name, ext));
                        let _ = fs::write(&path, &generated_code);
                        proj.source_files.push(path.clone());
                        app.active_file = Some(path);
                        app.code = generated_code;
                        let _ = fs::write(proj.root.join(format!("{}.vwx", proj.name)), 
                            serde_json::to_string_pretty(proj).unwrap());
                    }
                    app.show_file_wizard = false;
                }
                if ui.button("Cancel").clicked() { app.show_file_wizard = false; }
            });
        });
}