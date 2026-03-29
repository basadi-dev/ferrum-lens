use egui::{Ui, RichText};
use crate::theme::colors::*;

use std::sync::mpsc::{Receiver, channel};
use app_core::models::connection::{ConnectionConfig, DatabaseEngine};

pub enum TestState {
    None,
    Testing,
    Success,
    Error(String),
}

pub struct ConnectionManagerPanel {
    pub connection_name: String,
    pub host: String,
    pub port: String,
    pub database: String,
    pub user: String,
    pub password: String,
    pub engine: String,
    pub use_ssl: bool,

    pub test_state: TestState,
    pub test_rx: Option<Receiver<Result<(), String>>>,
    pub store: app_core::config::ConfigStore,
}

impl Default for ConnectionManagerPanel {
    fn default() -> Self {
        Self {
            connection_name: "Local Prod Replica".to_string(),
            host: "localhost".to_string(),
            port: "5432".to_string(),
            database: "workbench_production".to_string(),
            user: "admin".to_string(),
            password: "".to_string(),
            engine: "PostgreSQL".to_string(),
            use_ssl: true,
            test_state: TestState::None,
            test_rx: None,
            store: app_core::config::ConfigStore::new().expect("Failed to init config store"),
        }
    }
}

impl ConnectionManagerPanel {
    pub fn build_config(&self) -> ConnectionConfig {
        ConnectionConfig::new(
            self.connection_name.clone(),
            self.engine.parse().unwrap_or(DatabaseEngine::PostgreSql),
            self.host.clone(),
            self.port.parse().ok(),
            self.database.clone(),
            self.user.clone(),
            Some(self.password.clone()),
            self.use_ssl
        )
    }
    pub fn show(&mut self, ui: &mut Ui, event_tx: &std::sync::mpsc::Sender<crate::app::AppEvent>) {
        let conns = self.store.load_connections().unwrap_or_default();

        egui::SidePanel::left("connections_list_panel")
            .resizable(false)
            .exact_width(280.0)
            .frame(egui::Frame::NONE.fill(SURFACE_CONTAINER_LOW))
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    // Header
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.label(RichText::new("SAVED CONNECTIONS").size(10.0).strong().color(ON_SURFACE_VARIANT));
                    });
                    ui.add_space(8.0);
                    ui.separator();

                    // Scroll list
                    egui::ScrollArea::vertical().id_salt("conn_list_scroll").show(ui, |ui| {
                        for conn in &conns {
                            let is_selected = self.connection_name == conn.name;
                            let bg_color = if is_selected { SURFACE_CONTAINER_HIGHEST } else { egui::Color32::TRANSPARENT };
                            
                            let (rect, response) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 50.0), egui::Sense::click());
                            if is_selected || response.hovered() {
                                ui.painter().rect_filled(rect, 0.0, bg_color);
                            }
                            
                            if is_selected {
                                // Orange notch
                                ui.painter().rect_filled(
                                    egui::Rect::from_min_max(rect.min + egui::vec2(280.0 - 4.0, 0.0), rect.max),
                                    0.0,
                                    SECONDARY
                                );
                            }

                            ui.painter().text(
                                rect.min + egui::vec2(16.0, 10.0),
                                egui::Align2::LEFT_TOP,
                                "🗄",
                                egui::FontId::proportional(16.0),
                                if is_selected { SECONDARY } else { ON_SURFACE_VARIANT }
                            );

                            let text_color = if is_selected { ON_SURFACE } else { ON_SURFACE_VARIANT };
                            ui.painter().text(
                                rect.min + egui::vec2(44.0, 8.0),
                                egui::Align2::LEFT_TOP,
                                &conn.name,
                                egui::FontId::proportional(14.0),
                                text_color
                            );

                            ui.painter().text(
                                rect.min + egui::vec2(44.0, 26.0),
                                egui::Align2::LEFT_TOP,
                                format!("{}:{}", conn.host, conn.port.unwrap_or(0)),
                                egui::FontId::monospace(10.0),
                                ON_SURFACE_VARIANT
                            );

                            ui.painter().line_segment([rect.min + egui::vec2(0.0, 49.0), rect.max], egui::Stroke::new(1.0, OUTLINE_VARIANT.linear_multiply(0.2)));

                            if response.clicked() {
                                self.connection_name = conn.name.clone();
                                self.engine = conn.engine.to_string();
                                self.host = conn.host.clone();
                                self.port = conn.port.map(|p| p.to_string()).unwrap_or_default();
                                self.database = conn.database.clone();
                                self.user = conn.username.clone();
                                self.password = conn.password.clone().unwrap_or_default();
                                self.use_ssl = conn.use_ssl;
                            }
                        }
                    });

                    // Import JSON (Bottom)
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        ui.add_space(16.0);
                        ui.add(egui::Button::new(RichText::new("IMPORT JSON").size(11.0).strong().color(ON_SURFACE_VARIANT)).frame(false)).clicked();
                        ui.add_space(16.0);
                        ui.separator();
                    });
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(SURFACE_CONTAINER))
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    // Header Area
                    egui::Frame::NONE.fill(SURFACE_CONTAINER).inner_margin(24.0).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.heading(RichText::new("Connection Settings").size(24.0).strong().color(ON_SURFACE));
                                ui.label(RichText::new("Configure your Rust Workbench instance connection parameters.").size(12.0).color(ON_SURFACE_VARIANT));
                            });
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.horizontal(|ui| {
                                    let rect = ui.allocate_space(egui::vec2(80.0, 20.0)).1;
                                    ui.painter().rect_filled(rect, 4.0, crate::theme::colors::PRIMARY_CONTAINER);
                                    ui.painter().text(
                                        rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "v1.2.0 driver",
                                        egui::FontId::proportional(10.0),
                                        crate::theme::colors::ON_PRIMARY_CONTAINER
                                    );
                                });
                            });
                        });
                    });
                    ui.separator();

                    let label_fmt = |text: &str| RichText::new(text).size(10.0).strong().color(ON_SURFACE_VARIANT);
                    let text_edit_frame = |ui: &mut Ui, add_contents: &mut dyn FnMut(&mut Ui)| {
                        egui::Frame::NONE.fill(SURFACE_CONTAINER_LOWEST).corner_radius(4.0).stroke(egui::Stroke::new(1.0, OUTLINE_VARIANT.linear_multiply(0.3))).inner_margin(6.0).show(ui, add_contents);
                    };

                    // Form Area
                    egui::ScrollArea::vertical().id_salt("form_scroll").show(ui, |ui| {
                        ui.add_space(24.0);
                        ui.horizontal(|ui| {
                            ui.add_space(24.0);
                            egui::Grid::new("connection_grid")
                                .num_columns(2)
                                .spacing([32.0, 24.0])
                                .show(ui, |ui| {
                                    // Row 1
                                    ui.vertical(|ui| {
                                        ui.label(label_fmt("DATABASE ENGINE"));
                                        ui.add_space(4.0);
                                        text_edit_frame(ui, &mut |ui| {
                                            egui::ComboBox::from_id_salt("engine_combo")
                                                .width(200.0)
                                                .selected_text(RichText::new(&self.engine).color(ON_SURFACE))
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(&mut self.engine, "PostgreSQL".into(), "PostgreSQL");
                                                    ui.selectable_value(&mut self.engine, "MySQL".into(), "MySQL");
                                                    ui.selectable_value(&mut self.engine, "SQLite".into(), "SQLite");
                                                });
                                        });
                                    });

                                    ui.vertical(|ui| {
                                        ui.label(label_fmt("CONNECTION NAME"));
                                        ui.add_space(4.0);
                                        text_edit_frame(ui, &mut |ui| {
                                            ui.add(egui::TextEdit::singleline(&mut self.connection_name).desired_width(400.0).text_color(ON_SURFACE).frame(false));
                                        });
                                    });
                                    ui.end_row();

                                    if self.engine != "SQLite" {
                                        // Row 2 (Host & Port)
                                        ui.vertical(|ui| {
                                            ui.label(label_fmt("HOST ADDRESS"));
                                            ui.add_space(4.0);
                                            text_edit_frame(ui, &mut |ui| {
                                                ui.add(egui::TextEdit::singleline(&mut self.host).desired_width(400.0).text_color(ON_SURFACE).frame(false));
                                            });
                                        });

                                        ui.vertical(|ui| {
                                            ui.label(label_fmt("PORT"));
                                            ui.add_space(4.0);
                                            text_edit_frame(ui, &mut |ui| {
                                                ui.add(egui::TextEdit::singleline(&mut self.port).desired_width(100.0).text_color(ON_SURFACE).frame(false));
                                            });
                                        });
                                        ui.end_row();
                                    }

                                    // Row 3
                                    ui.vertical(|ui| {
                                        let mut db_label = "DATABASE NAME";
                                        if self.engine == "SQLite" {
                                            db_label = "DATABASE FILE PATH";
                                        }
                                        ui.label(label_fmt(db_label));
                                        ui.add_space(4.0);
                                        
                                        text_edit_frame(ui, &mut |ui| {
                                            if self.engine == "SQLite" {
                                                ui.horizontal(|ui| {
                                                    ui.add(egui::TextEdit::singleline(&mut self.database).desired_width(320.0).text_color(ON_SURFACE).frame(false));
                                                    if ui.button("Browse...").clicked()
                                                        && let Some(path) = rfd::FileDialog::new()
                                                            .add_filter("SQLite Database", &["db", "sqlite", "sqlite3"])
                                                            .add_filter("All Files", &["*"])
                                                            .pick_file() 
                                                        {
                                                            self.database = path.display().to_string();
                                                        }
                                                });
                                            } else {
                                                ui.add(egui::TextEdit::singleline(&mut self.database).desired_width(400.0).text_color(ON_SURFACE).frame(false));
                                            }
                                        });
                                    });
                                    ui.label(""); // empty cell
                                    ui.end_row();

                                    if self.engine != "SQLite" {
                                        // Row 4 (User & Pass)
                                        ui.vertical(|ui| {
                                            ui.label(label_fmt("USER"));
                                            ui.add_space(4.0);
                                            text_edit_frame(ui, &mut |ui| {
                                                ui.add(egui::TextEdit::singleline(&mut self.user).desired_width(400.0).text_color(ON_SURFACE).frame(false));
                                            });
                                        });

                                        ui.vertical(|ui| {
                                            ui.label(label_fmt("PASSWORD"));
                                            ui.add_space(4.0);
                                            text_edit_frame(ui, &mut |ui| {
                                                ui.add(egui::TextEdit::singleline(&mut self.password).password(true).desired_width(400.0).text_color(ON_SURFACE).frame(false));
                                            });
                                        });
                                        ui.end_row();
                                    }
                                });
                        });

                        ui.add_space(32.0);
                        
                        // SSL Toggle and advanced
                        if self.engine != "SQLite" {
                            ui.horizontal(|ui| {
                                ui.add_space(24.0);
                                ui.vertical(|ui| {
                                    ui.separator();
                                    ui.add_space(16.0);
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut self.use_ssl, "");
                                        ui.vertical(|ui| {
                                            ui.label(RichText::new("Enable SSL/TLS").strong().color(ON_SURFACE).size(12.0));
                                            ui.label(RichText::new("Verify server certificate for secure transport").color(ON_SURFACE_VARIANT).size(10.0));
                                        });
                                        
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(RichText::new("ADVANCED PARAMETERS").strong().color(PRIMARY).size(10.0));
                                        });
                                    });
                                });
                                ui.add_space(24.0);
                            });
                        }
                        
                        // Error states
                        match &self.test_state {
                            TestState::None => {}
                            TestState::Testing => {
                                ui.add_space(16.0);
                                ui.label(RichText::new("Testing connection...").color(ON_SURFACE_VARIANT));
                            }
                            TestState::Success => {
                                ui.add_space(16.0);
                                ui.label(RichText::new("Success! Connection is valid.").color(SUCCESS_BADGE));
                            }
                            TestState::Error(err) => {
                                ui.add_space(16.0);
                                ui.label(RichText::new(format!("Error: {}", err)).color(ERROR));
                            }
                        }
                    });

                    // Poll receiver if testing
                    if let Some(rx) = &self.test_rx
                        && let Ok(res) = rx.try_recv() {
                            match res {
                                Ok(_) => self.test_state = TestState::Success,
                                Err(e) => self.test_state = TestState::Error(e),
                            }
                            self.test_rx = None;
                        }

                    // Footer Actions
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        egui::Frame::NONE.fill(SURFACE_CONTAINER_LOW.linear_multiply(0.5)).inner_margin(24.0).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                if ui.button(RichText::new("⚡ Test Connection").strong().size(14.0).color(ON_SURFACE_VARIANT)).clicked() {
                                    let (tx, rx) = channel();
                                    self.test_rx = Some(rx);
                                    self.test_state = TestState::Testing;
                                    
                                    let config = self.build_config();
                                    let ctx = ui.ctx().clone();
                                    
                                    tokio::spawn(async move {
                                        let res = app_core::db_pool::DbPool::test_connection(&config).await;
                                        let _ = tx.send(res.map_err(|e| e.to_string()));
                                        ctx.request_repaint(); // Wake up UI to check rx
                                    });
                                }
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    // Connect Button
                                    let btn_rect = ui.allocate_space(egui::vec2(120.0, 32.0)).1;
                                    let interact = ui.interact(btn_rect, ui.id().with("connect"), egui::Sense::click());
                                    let color = if interact.hovered() { SECONDARY_CONTAINER } else { SECONDARY };
                                    ui.painter().rect_filled(btn_rect, 4.0, color);
                                    ui.painter().text(btn_rect.center(), egui::Align2::CENTER_CENTER, "Connect", egui::FontId::proportional(14.0), ON_SECONDARY);
                                    
                                    if interact.clicked() {
                                        let config = self.build_config();
                                        let tx = event_tx.clone();
                                        
                                        // Use test channel to surface connection errors
                                        let (err_tx, err_rx) = std::sync::mpsc::channel();
                                        self.test_rx = Some(err_rx);
                                        self.test_state = TestState::Testing;
                                        
                                        let ctx = ui.ctx().clone();

                                        tokio::spawn(async move {
                                            match app_core::db_pool::DbPool::connect(&config).await {
                                                Ok(pool) => {
                                                    let _ = tx.send(crate::app::AppEvent::Connect(pool, config));
                                                    ctx.request_repaint();
                                                }
                                                Err(e) => {
                                                    let _ = err_tx.send(Err(e.to_string()));
                                                    ctx.request_repaint();
                                                }
                                            }
                                        });
                                    }

                                    // Save Button
                                    ui.add_space(8.0);
                                    if ui.add(egui::Button::new(RichText::new("Save").color(PRIMARY).size(14.0).strong()).min_size(egui::vec2(80.0, 32.0))).clicked() {
                                        let mut conns = self.store.load_connections().unwrap_or_default();
                                        conns.retain(|c| c.name != self.connection_name); // update if exists
                                        conns.push(self.build_config());
                                        if let Err(e) = self.store.save_connections(&conns) {
                                            self.test_state = TestState::Error(format!("Save failed: {}", e));
                                        } else {
                                            self.test_state = TestState::Success;
                                        }
                                    }

                                    // Cancel Button
                                    ui.add_space(8.0);
                                    if ui.button(RichText::new("Cancel").color(ON_SURFACE_VARIANT).size(14.0).strong()).clicked() {}
                                });
                            });
                        });
                        ui.separator();
                    });

                });
            });
    }
}
