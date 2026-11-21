//! The Settings window: General, Machines, Canvas, Load Type and System, laid out like the
//! original's Design Space Settings. Online-only parts (content filters, community
//! notifications, cloud saving, firmware updates) have nothing to talk to here and are left out.

use super::projects::{GridMode, LoadType, MachineModel, PageSize};
use super::*;
use crate::materials::{CustomMaterial, Tool};

impl App {
    pub(super) fn settings_window(&mut self, ctx: &egui::Context) {
        if !self.show_settings {
            return;
        }
        let mut open = true;
        let before = serde_json::to_string(&self.prefs).unwrap_or_default();
        let mut toggle_theme = false;
        let mut forget_machine = false;
        let screen = ctx.content_rect();
        egui::Window::new(RichText::new("Settings").strong().size(18.0))
            .id(egui::Id::unique("settings-window"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_rect(Rect::from_center_size(screen.center(), vec2(720.0, 560.0)))
            .show(ctx, |ui| {
                segmented(ui, &mut self.settings_tab, &[(0u8, "General"), (1u8, "Machines"), (2u8, "Canvas"), (3u8, "Load Type"), (4u8, "System")]);
                ui.add_space(12.0);
                let row = |ui: &mut Ui, label: &str, add: &mut dyn FnMut(&mut Ui)| {
                    ui.horizontal(|ui| {
                        ui.allocate_ui_with_layout(vec2(200.0, 28.0), Layout::left_to_right(Align::Center), |ui| {
                            ui.label(RichText::new(label).size(14.5).color(INK()));
                        });
                        add(ui);
                    });
                    ui.add_space(8.0);
                };
                match self.settings_tab {
                    0 => {
                        row(ui, "Language", &mut |ui| {
                            ui.label("English");
                        });
                        row(ui, "Autosave", &mut |ui| {
                            ui.radio_value(&mut self.prefs.autosave, true, "Yes");
                            ui.radio_value(&mut self.prefs.autosave, false, "No");
                            ui.label(RichText::new("every minute, to My Stuff").size(12.0).color(MUTED()));
                        });
                        let mut is_dark = dark();
                        row(ui, "Appearance", &mut |ui| {
                            if ui.radio_value(&mut is_dark, false, "Light").changed() || ui.radio_value(&mut is_dark, true, "Dark").changed() {
                                toggle_theme = true;
                            }
                        });
                        row(ui, "Projects folder", &mut |ui| {
                            ui.label(RichText::new(documents_dir().display().to_string()).size(12.5).color(MUTED()));
                        });
                        row(ui, "About MatCraft", &mut |ui| {
                            ui.label(format!("Version {}  ·  {}-{}", env!("CARGO_PKG_VERSION"), std::env::consts::OS, std::env::consts::ARCH));
                        });
                    }
                    1 => {
                        let mac_tail: String = self.settings.mac.chars().filter(|c| c.is_ascii_hexdigit()).collect::<String>();
                        let tail = mac_tail.get(mac_tail.len().saturating_sub(4)..).unwrap_or("").to_uppercase();
                        ui.columns(2, |cols| {
                            let ui = &mut cols[0];
                            card(ui, |ui| {
                                ui.label(RichText::new("My machine").strong().color(INK()));
                                egui::Frame::NONE.fill(ACCENT_SOFT()).corner_radius(8).inner_margin(10).show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.label(RichText::new(format!("{} - {tail}", self.prefs.model.label())).strong().color(INK()));
                                    let (dot, txt) = if self.connected.is_some() { (ACCENT, "Connected") } else { (DANGER, "Disconnected") };
                                    ui.horizontal(|ui| {
                                        let (r, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                                        ui.painter().circle_filled(r.center(), 4.0, dot);
                                        ui.label(RichText::new(txt).size(12.0).color(MUTED()));
                                    });
                                });
                                ui.add_space(6.0);
                                ui.label(RichText::new("Model").size(12.5).color(MUTED()));
                                egui::ComboBox::from_id_salt("model").selected_text(self.prefs.model.label()).width(200.0).show_ui(ui, |ui| {
                                    for m in MachineModel::ALL {
                                        ui.selectable_value(&mut self.prefs.model, m, m.label());
                                    }
                                });
                                ui.label(RichText::new(format!("Address {}", self.settings.mac)).size(12.0).color(MUTED()));
                            });
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Autoconnect").color(INK()));
                                ui.checkbox(&mut self.prefs.autoconnect, "");
                            });
                            ui.label(RichText::new("Connect to this machine when MatCraft starts.").size(12.0).color(MUTED()));

                            let ui = &mut cols[1];
                            let section = |ui: &mut Ui, title: &str, text: &str| {
                                ui.label(RichText::new(title).strong().color(INK()));
                                ui.label(RichText::new(text).size(12.5).color(MUTED()));
                            };
                            section(ui, "Update firmware", "Firmware comes from the original's online service, which MatCraft doesn't use.");
                            ui.add_enabled(false, egui::Button::new("Update"));
                            ui.separator();
                            section(ui, "Machine calibration", "Calibration needs commands that aren't decoded yet.");
                            ui.add_enabled(false, egui::Button::new("Calibrate"));
                            ui.separator();
                            section(ui, "Custom material settings", "Your own materials appear at the end of the material list.");
                            let mut remove: Option<usize> = None;
                            for (k, m) in self.prefs.custom_materials.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("{}  ·  {}, pressure {}, {} pass(es)", m.name, m.tool.label(), m.pressure, m.passes)).size(12.5));
                                    if ui.small_button("Remove").clicked() {
                                        remove = Some(k);
                                    }
                                });
                            }
                            if let Some(k) = remove {
                                self.prefs.custom_materials.remove(k);
                            }
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut self.custom_draft.name).desired_width(130.0));
                                egui::ComboBox::from_id_salt("custom-tool").selected_text(self.custom_draft.tool.label()).width(130.0).show_ui(ui, |ui| {
                                    for t in Tool::ALL {
                                        ui.selectable_value(&mut self.custom_draft.tool, t, t.label());
                                    }
                                });
                            });
                            ui.horizontal(|ui| {
                                ui.add(egui::Slider::new(&mut self.custom_draft.pressure, 1..=7).text("pressure"));
                                ui.add(egui::DragValue::new(&mut self.custom_draft.passes).range(1..=10).suffix(" pass"));
                                if ui.button("Add").clicked() && !self.custom_draft.name.trim().is_empty() {
                                    self.prefs.custom_materials.push(CustomMaterial { name: self.custom_draft.name.trim().to_string(), ..self.custom_draft.clone() });
                                }
                            });
                            ui.separator();
                            section(ui, "Remove this machine", "Forget its address and settings. You can set it up again in the Machine panel.");
                            if ui
                                .add(egui::Button::new(RichText::new("Remove").color(DANGER)).fill(Color32::TRANSPARENT).stroke(Stroke::new(1.0, DANGER)).corner_radius(CornerRadius::same(14)))
                                .clicked()
                            {
                                forget_machine = true;
                            }
                        });
                    }
                    2 => {
                        row(ui, "Canvas grid", &mut |ui| {
                            ui.radio_value(&mut self.prefs.grid, GridMode::Full, "Full grid");
                            ui.radio_value(&mut self.prefs.grid, GridMode::Partial, "Partial grid");
                            ui.radio_value(&mut self.prefs.grid, GridMode::None, "No grid");
                        });
                        row(ui, "Units", &mut |ui| {
                            ui.radio_value(&mut self.prefs.metric, false, "Imperial (in)");
                            ui.radio_value(&mut self.prefs.metric, true, "Metric (mm, cm)");
                        });
                        row(ui, "Operation type", &mut |ui| {
                            ui.radio_value(&mut self.prefs.show_operation, true, "Yes");
                            ui.radio_value(&mut self.prefs.show_operation, false, "No");
                            ui.label(RichText::new("show the Operation menu in the edit bar").size(12.0).color(MUTED()));
                        });
                        row(ui, "Snap to align", &mut |ui| {
                            ui.radio_value(&mut self.prefs.snap, true, "Yes");
                            ui.radio_value(&mut self.prefs.snap, false, "No");
                            ui.label(RichText::new("line up with other layers while moving").size(12.0).color(MUTED()));
                        });
                    }
                    3 => {
                        row(ui, "Defaults for", &mut |ui| {
                            egui::ComboBox::from_id_salt("defaults-for").selected_text(self.prefs.model.label()).width(260.0).show_ui(ui, |ui| {
                                for m in MachineModel::ALL {
                                    ui.selectable_value(&mut self.prefs.model, m, m.label());
                                }
                            });
                        });
                        row(ui, "Load type", &mut |ui| {
                            egui::ComboBox::from_id_salt("load-type").selected_text(self.prefs.load.label()).width(260.0).show_ui(ui, |ui| {
                                for l in LoadType::ALL {
                                    let ok = l != LoadType::None || self.prefs.model.matless();
                                    ui.add_enabled_ui(ok, |ui| {
                                        ui.selectable_value(&mut self.prefs.load, l, l.label());
                                    });
                                }
                            });
                        });
                        row(ui, "Print then cut page size", &mut |ui| {
                            egui::ComboBox::from_id_salt("ptc-page").selected_text(self.prefs.page.label()).width(260.0).show_ui(ui, |ui| {
                                for p in PageSize::ALL {
                                    ui.selectable_value(&mut self.prefs.page, p, p.label());
                                }
                            });
                        });
                        let a = self.prefs.page.print_area();
                        ui.label(RichText::new(format!("Printable area for print then cut: {:.2} x {:.2} in.", a.x, a.y)).size(12.5).color(MUTED()));
                        if self.prefs.load == LoadType::None && !self.prefs.model.matless() {
                            ui.label(RichText::new("The original Maker needs a mat.").size(12.5).color(DANGER));
                        }
                    }
                    _ => {
                        let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
                        row(ui, "System", &mut |ui| {
                            ui.label(format!("{} {}  ·  {threads} processor threads", std::env::consts::OS, std::env::consts::ARCH));
                        });
                        row(ui, "Fonts found", &mut |ui| {
                            ui.label(fonts::families().map(|f| f.len().to_string()).unwrap_or_else(|| "looking...".into()));
                        });
                        row(ui, "Settings folder", &mut |ui| {
                            ui.label(RichText::new(config_dir().display().to_string()).size(12.5).color(MUTED()));
                        });
                        ui.add_space(6.0);
                        ui.label(RichText::new("Third-party parts").strong().color(INK()));
                        ui.label(
                            RichText::new(
                                "egui / eframe (MIT or Apache-2.0), geo and i_overlay (MIT or Apache-2.0), image, ttf-parser, fontdb, rfd, aes, serde \
                                 (MIT or Apache-2.0). Built-in typeface: Ubuntu Light (Ubuntu Font Licence 1.0).",
                            )
                            .size(12.5)
                            .color(MUTED()),
                        );
                    }
                }
                ui.add_space(10.0);
                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    if primary_button(ui, "Done").clicked() {
                        self.show_settings = false;
                    }
                });
            });
        if toggle_theme {
            self.toggle_dark(ctx);
        }
        if forget_machine {
            self.settings = MachineSettings::default();
            self.save_settings();
            let _ = self.machine.tx.send(Request::Disconnect);
        }
        if !open {
            self.show_settings = false;
        }
        if serde_json::to_string(&self.prefs).unwrap_or_default() != before {
            self.save_prefs();
        }
    }
}
