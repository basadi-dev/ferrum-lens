use egui::{RichText, Sense, Ui};
use crate::theme::colors::*;
use app_core::queries::SavedQuery;
use std::collections::HashMap;

#[derive(Default)]
pub struct QueryExplorerPanel {
    pub active_item: Option<String>,
    pub tables: Vec<app_core::schema::TableMetadata>,
    pub saved_queries: Vec<SavedQuery>,
    pub queries_changed: bool,
    pub table_columns: HashMap<String, Vec<app_core::schema::ColumnMetadata>>,
    
    // UI state
    new_query_name: String,
    is_adding_query: bool,
    editing_query_id: Option<String>,
}

impl QueryExplorerPanel {
    pub fn set_tables(&mut self, tables: Vec<app_core::schema::TableMetadata>) {
        self.tables = tables;
    }

    pub fn set_saved_queries(&mut self, queries: Vec<SavedQuery>) {
        self.saved_queries = queries;
    }

    pub fn add_columns_to_cache(&mut self, table_name: String, columns: Vec<app_core::schema::ColumnMetadata>) {
        self.table_columns.insert(table_name, columns);
    }

    pub fn show(&mut self, ui: &mut Ui, event_tx: &std::sync::mpsc::Sender<crate::app::AppEvent>) {
        ui.vertical(|ui| {
            // Sidebar Header
            egui::Frame::NONE.fill(SURFACE_CONTAINER_HIGH.linear_multiply(0.3)).inner_margin(8.0).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("DATABASE EXPLORER").size(10.0).strong().color(ON_SURFACE_VARIANT));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.button(RichText::new("🔍").size(10.0)).clicked();
                        if ui.button(RichText::new("↻").size(10.0)).clicked() {}
                    });
                });
                ui.add_space(8.0);
                
                let btn_rect = ui.allocate_space(egui::vec2(ui.available_width(), 26.0)).1;
                let interact = ui.interact(btn_rect, ui.id().with("btn_connect_server"), egui::Sense::click());
                let color = if interact.hovered() { SURFACE_CONTAINER_HIGHEST } else { SURFACE_CONTAINER_HIGH };
                ui.painter().rect_filled(btn_rect, 2.0, color);
                ui.painter().rect_stroke(btn_rect, 2.0, egui::Stroke::new(1.0, OUTLINE_VARIANT.linear_multiply(0.2)), egui::StrokeKind::Inside);
                ui.painter().text(btn_rect.center(), egui::Align2::CENTER_CENTER, "➕ Connect Server", egui::FontId::proportional(11.0), SECONDARY);
            });
            ui.separator();

            let bottom_height = 42.0;
            let available_height = ui.available_height() - bottom_height;

            // Trees (Scrollable)
            egui::ScrollArea::vertical()
                .id_salt("explorer_scroll")
                .max_height(available_height) // Limits expansion to leave space for bottom bar
                .auto_shrink([false, false])  // Replaces Fill Behavior
                .show(ui, |ui| {
                    ui.add_space(4.0);
                    
                    self.render_tree_node(ui, "Databases", "🗄", true, |ui, this| {
                        if this.tables.is_empty() {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.add_space(24.0);
                                ui.label(RichText::new("Loading...").color(ON_SURFACE_VARIANT).size(11.0));
                            });
                            ui.add_space(4.0);
                        } else {
                            let table_names: Vec<String> = this.tables.iter().map(|t| t.name.clone()).collect();
                            for name in table_names {
                                this.render_table_node(ui, &name, event_tx);
                            }
                        }
                    });

                    self.render_tree_node(ui, "Saved Queries", "📄", true, |ui, this| {
                        let queries = this.saved_queries.clone();
                        for query in &queries {
                            this.render_saved_query_item(ui, query, event_tx);
                        }

                        if this.is_adding_query {
                            ui.horizontal(|ui| {
                                ui.add_space(24.0);
                                let r = ui.add(egui::TextEdit::singleline(&mut this.new_query_name).desired_width(120.0));
                                if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                    if !this.new_query_name.trim().is_empty() {
                                        this.saved_queries.push(SavedQuery {
                                            id: uuid::Uuid::new_v4().to_string(),
                                            name: this.new_query_name.clone(),
                                            content: "-- New query".to_string(),
                                            created_at: std::time::SystemTime::now(),
                                        });
                                        this.queries_changed = true;
                                    }
                                    this.is_adding_query = false;
                                    this.new_query_name.clear();
                                } else if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                    this.is_adding_query = false;
                                    this.new_query_name.clear();
                                }
                                r.request_focus();
                            });
                        }

                        ui.horizontal(|ui| {
                            ui.add_space(24.0);
                            if ui.button(RichText::new("➕ New Query").size(10.0).color(ON_SURFACE_VARIANT).strong()).clicked() {
                                this.is_adding_query = true;
                                this.new_query_name.clear();
                            }
                        });
                    });
                }); // close ScrollArea
            
            // Bottom user profile area
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.separator();
                egui::Frame::NONE.fill(SURFACE_CONTAINER_LOW).inner_margin(8.0).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let avatar_rect = ui.allocate_space(egui::vec2(24.0, 24.0)).1;
                        ui.painter().rect_filled(avatar_rect, 4.0, SURFACE_CONTAINER_HIGHEST);
                        ui.painter().text(avatar_rect.center(), egui::Align2::CENTER_CENTER, "👤", egui::FontId::proportional(12.0), ON_SURFACE_VARIANT);
                        
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Local User").size(10.0).strong().color(ON_SURFACE));
                            ui.label(RichText::new("CONNECTED").size(8.0).strong().color(ON_SURFACE_VARIANT));
                        });
                    });
                });
            });
        }); // close vertical layout
    }

    fn render_tree_node<F>(&mut self, ui: &mut Ui, label: &str, icon: &str, default_open: bool, add_children: F)
    where
        F: FnOnce(&mut Ui, &mut Self),
    {
        let id = ui.make_persistent_id(label);
        let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, default_open);
        
        let (rect, response) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 26.0), Sense::click());
        
        if response.clicked() {
            state.toggle(ui);
        }

        if response.hovered() {
            ui.painter().rect_filled(rect, 0.0, SURFACE_CONTAINER_HIGH);
        }

        let is_open = state.is_open();
        let arrow = if is_open { "▼" } else { "▶" };
        
        ui.painter().text(
            rect.min + egui::vec2(8.0, 6.0),
            egui::Align2::LEFT_TOP,
            format!("{} {} {}", arrow, icon, label),
            egui::FontId::proportional(11.0),
            ON_SURFACE_VARIANT
        );

        state.show_body_unindented(ui, |ui| {
            // Draw a subtle left border for children
            let start_x = rect.min.x + 16.0;
            
            let frame = egui::Frame::NONE.inner_margin(egui::Margin { left: 16, right: 0, top: 0, bottom: 0 });
            frame.show(ui, |ui| {
                add_children(ui, self);
            });
            
            // Subtle left border line
            let end_y = ui.min_rect().max.y;
            ui.painter().line_segment(
                [egui::pos2(start_x, rect.max.y), egui::pos2(start_x, end_y)],
                egui::Stroke::new(1.0, OUTLINE_VARIANT.linear_multiply(0.1))
            );
        });
    }

    fn render_table_node(&mut self, ui: &mut Ui, table_name: &str, event_tx: &std::sync::mpsc::Sender<crate::app::AppEvent>) {
        let is_active = self.active_item.as_deref() == Some(table_name);
        let id = ui.make_persistent_id(format!("table_node_{}", table_name));
        let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);

        let (rect, response) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 26.0), Sense::click());

        if response.clicked() {
            self.active_item = Some(table_name.to_string());
            state.toggle(ui);
            // Fetch columns on expand if missing
            if !self.table_columns.contains_key(table_name) {
                let _ = event_tx.send(crate::app::AppEvent::LoadColumnsForSidebar(table_name.to_string()));
            }
        }

        if response.double_clicked() {
            self.active_item = Some(table_name.to_string());
            let _ = event_tx.send(crate::app::AppEvent::OpenTable(table_name.to_string()));
            let sql = format!("SELECT * FROM {} LIMIT 100;", table_name);
            let _ = event_tx.send(crate::app::AppEvent::OpenQueryEditorWithText(sql.clone()));
            let _ = event_tx.send(crate::app::AppEvent::ExecuteQuery(sql));
        }

        response.context_menu(|ui| {
            if ui.button("SELECT Top 100 Rows").clicked() {
                let sql = format!("SELECT * FROM {} LIMIT 100;", table_name);
                let _ = event_tx.send(crate::app::AppEvent::OpenQueryEditorWithText(sql.clone()));
                let _ = event_tx.send(crate::app::AppEvent::ExecuteQuery(sql));
                ui.close_menu();
            }
            if ui.button("Describe Table").clicked() {
                let _ = event_tx.send(crate::app::AppEvent::OpenTable(table_name.to_string()));
                ui.close_menu();
            }
            if ui.button("Copy Table Name").clicked() {
                ui.ctx().copy_text(table_name.to_string());
                ui.close_menu();
            }
            if ui.button("Generate DROP TABLE").clicked() {
                let sql = format!("DROP TABLE {};", table_name);
                let _ = event_tx.send(crate::app::AppEvent::OpenQueryEditorWithText(sql));
                ui.close_menu();
            }
        });

        // Background styling
        if is_active {
            ui.painter().rect_filled(rect, 0.0, SECONDARY.linear_multiply(0.05));
            ui.painter().rect_filled(
                egui::Rect::from_min_max(rect.min, rect.min + egui::vec2(2.0, 26.0)),
                0.0,
                SECONDARY
            );
        } else if response.hovered() {
            ui.painter().rect_filled(rect, 0.0, SURFACE_CONTAINER_HIGH);
        }

        let is_open = state.is_open();
        let arrow = if is_open { "▼" } else { "▶" };
        let text_color = if is_active || response.hovered() { SECONDARY } else { ON_SURFACE_VARIANT };

        ui.painter().text(
            rect.min + egui::vec2(12.0, 6.0),
            egui::Align2::LEFT_TOP,
            format!("{} ▦ {}", arrow, table_name),
            egui::FontId::proportional(11.0),
            text_color,
        );

        state.show_body_unindented(ui, |ui| {
            let start_x = rect.min.x + 16.0;
            let frame = egui::Frame::NONE.inner_margin(egui::Margin { left: 16, right: 0, top: 0, bottom: 0 });
            frame.show(ui, |ui| {
                if let Some(cols) = self.table_columns.get(table_name) {
                    for col in cols {
                        let icon = if col.is_primary { "🔑" } else { "⛁" };
                        let label = format!("{} {} ({})", icon, col.name, col.data_type);
                        
                        let (c_rect, c_resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 20.0), Sense::click());
                        if c_resp.hovered() {
                            ui.painter().rect_filled(c_rect, 0.0, SURFACE_CONTAINER_HIGH);
                        }
                        ui.painter().text(
                            c_rect.min + egui::vec2(12.0, 4.0),
                            egui::Align2::LEFT_TOP,
                            label,
                            egui::FontId::proportional(10.0),
                            ON_SURFACE_VARIANT.linear_multiply(0.8)
                        );
                    }
                } else {
                    let (c_rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 20.0), Sense::hover());
                    ui.painter().text(
                        c_rect.min + egui::vec2(12.0, 4.0),
                        egui::Align2::LEFT_TOP,
                        "Loading...",
                        egui::FontId::proportional(10.0),
                        ON_SURFACE_VARIANT.linear_multiply(0.5)
                    );
                }
            });
            let end_y = ui.min_rect().max.y;
            ui.painter().line_segment(
                [egui::pos2(start_x, rect.max.y), egui::pos2(start_x, end_y)],
                egui::Stroke::new(1.0, OUTLINE_VARIANT.linear_multiply(0.1))
            );
        });
    }

    fn render_saved_query_item(&mut self, ui: &mut Ui, query: &SavedQuery, event_tx: &std::sync::mpsc::Sender<crate::app::AppEvent>) {
        let is_active = self.active_item.as_deref() == Some(query.name.as_str());

        // Handle inline editing
        if self.editing_query_id.as_deref() == Some(query.id.as_str()) {
            let mut edit_name = query.name.clone();
            let r = ui.add(egui::TextEdit::singleline(&mut edit_name).desired_width(120.0));
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if !edit_name.trim().is_empty()
                    && let Some(pos) = self.saved_queries.iter().position(|q| q.id == query.id) {
                        self.saved_queries[pos].name = edit_name;
                        self.queries_changed = true;
                    }
                self.editing_query_id = None;
            } else if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.editing_query_id = None;
            }
            r.request_focus();
            return;
        }

        let (rect, response) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 26.0), Sense::click());

        if response.clicked() {
            self.active_item = Some(query.name.clone());
            let _ = event_tx.send(crate::app::AppEvent::OpenSavedQuery(query.name.clone(), query.content.clone()));
        }

        if response.double_clicked() {
            // Immediately execute upon double click
            let _ = event_tx.send(crate::app::AppEvent::OpenSavedQuery(query.name.clone(), query.content.clone()));
            let _ = event_tx.send(crate::app::AppEvent::ExecuteQuery(query.content.clone()));
        }

        response.context_menu(|ui| {
            if ui.button("Rename").clicked() {
                self.editing_query_id = Some(query.id.clone());
                ui.close_menu();
            }
            if ui.button(egui::RichText::new("Delete").color(egui::Color32::from_rgb(255, 50, 50))).clicked() {
                self.saved_queries.retain(|q| q.id != query.id);
                self.queries_changed = true;
                if self.active_item.as_deref() == Some(query.name.as_str()) {
                    self.active_item = None;
                }
                ui.close_menu();
            }
        });

        if is_active {
            ui.painter().rect_filled(rect, 0.0, SECONDARY.linear_multiply(0.05));
            ui.painter().rect_filled(
                egui::Rect::from_min_max(rect.min, rect.min + egui::vec2(2.0, 26.0)),
                0.0,
                SECONDARY
            );
        } else if response.hovered() {
            ui.painter().rect_filled(rect, 0.0, SURFACE_CONTAINER_HIGH);
        }

        let text_color = if is_active || response.hovered() { SECONDARY } else { ON_SURFACE_VARIANT };

        ui.painter().text(
            rect.min + egui::vec2(12.0, 6.0),
            egui::Align2::LEFT_TOP,
            format!("Terminal {}", query.name),
            egui::FontId::proportional(11.0),
            text_color,
        );
    }
}
