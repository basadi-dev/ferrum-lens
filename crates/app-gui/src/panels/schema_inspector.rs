use egui::{Ui, RichText};
use egui_extras::{Column, TableBuilder};
use crate::theme::colors::*;

#[derive(Default)]
pub struct SchemaInspectorPanel {
    active_tab: usize, // 0 = Columns, 1 = Indexes, 2 = Constraints, 3 = DDL
    pub table_name: String,
    pub columns: Vec<app_core::schema::ColumnMetadata>,
}

impl SchemaInspectorPanel {
    pub fn set_columns(&mut self, table_name: String, columns: Vec<app_core::schema::ColumnMetadata>) {
        self.table_name = table_name;
        self.columns = columns;
    }

    pub fn show(&mut self, ui: &mut Ui, table_name: &str) {
        ui.vertical(|ui| {
            // Header Context
            ui.horizontal(|ui| {
                ui.heading(RichText::new(table_name).color(ON_SURFACE).strong().size(20.0));
                ui.label(RichText::new("Table").color(ON_SURFACE_VARIANT).size(12.0));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if !self.columns.is_empty() {
                        ui.label(RichText::new(format!("{} Columns", self.columns.len())).color(ON_SURFACE_VARIANT).size(12.0));
                    }
                });
            });
            
            ui.add_space(16.0);

            // Custom Tab Bar
            ui.horizontal(|ui| {
                let tabs = ["Columns", "Indexes", "Constraints", "DDL"];
                for (i, tab) in tabs.iter().enumerate() {
                    let is_active = self.active_tab == i;
                    let text_color = if is_active { SECONDARY } else { ON_SURFACE_VARIANT };
                    
                    if ui.selectable_label(is_active, RichText::new(*tab).color(text_color).strong()).clicked() {
                        self.active_tab = i;
                    }
                }
            });
            ui.separator();
            ui.add_space(16.0);

            // Tab Content
            match self.active_tab {
                0 => self.render_columns(ui),
                1 => self.render_indexes(ui),
                2 => self.render_constraints(ui),
                3 => self.render_ddl(ui),
                _ => {}
            }
        });
    }

    fn render_columns(&mut self, ui: &mut Ui) {
        let _text_height = egui::TextStyle::Body.resolve(ui.style()).size;

        if self.columns.is_empty() {
            ui.label(RichText::new("Loading column data...").color(ON_SURFACE_VARIANT));
            return;
        }

        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::initial(150.0).resizable(true)) // NAME
            .column(Column::initial(120.0).resizable(true)) // TYPE
            .column(Column::initial(60.0).resizable(true))  // NULL
            .column(Column::initial(180.0).resizable(true)) // DEFAULT
            .column(Column::remainder().at_least(100.0))    // EXTRAS
            .header(24.0, |mut header| {
                let col_header = |text| RichText::new(text).strong().size(11.0).color(ON_SURFACE_VARIANT);
                header.col(|ui| { ui.label(col_header("NAME")); });
                header.col(|ui| { ui.label(col_header("TYPE")); });
                header.col(|ui| { ui.label(col_header("NULL")); });
                header.col(|ui| { ui.label(col_header("DEFAULT")); });
                header.col(|ui| { ui.label(col_header("EXTRAS")); });
            })
            .body(|mut body| {
                for col in &self.columns {
                    body.row(28.0, |mut row| {
                        row.col(|ui| { ui.label(RichText::new(&col.name).color(ON_SURFACE).strong()); });
                        row.col(|ui| { ui.label(RichText::new(&col.data_type).color(PRIMARY)); });
                        
                        let nullable_str = if col.is_nullable { "YES" } else { "NO" };
                        row.col(|ui| { ui.label(RichText::new(nullable_str).color(ON_SURFACE_VARIANT)); });
                        
                        let default_str = col.default_value.clone().unwrap_or_else(|| "NULL".to_string());
                        row.col(|ui| { ui.label(RichText::new(default_str).color(ON_SURFACE_VARIANT)); });
                        
                        row.col(|ui| { 
                            if col.is_primary {
                                ui.label(RichText::new("PRIMARY KEY").color(SECONDARY).size(10.0));
                            }
                        });
                    });
                }
            });
    }

    fn render_indexes(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Index visualization coming soon.").color(ON_SURFACE_VARIANT));
    }

    fn render_constraints(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Constraints coming soon.").color(ON_SURFACE_VARIANT));
    }

    fn render_ddl(&mut self, ui: &mut Ui) {
        let code = "-- Mock DDL\nCREATE TABLE auth_users (\n    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),\n    email VARCHAR(255) NOT NULL UNIQUE,\n    password_hash VARCHAR(255) NOT NULL,\n    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),\n    last_login TIMESTAMPTZ\n);";
        let mut text = code.to_string();
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut text)
                    .font(egui::TextStyle::Monospace)
                    .code_editor()
                    .desired_rows(10)
                    .desired_width(f32::INFINITY)
                    .interactive(false)
            );
        });
    }
}
