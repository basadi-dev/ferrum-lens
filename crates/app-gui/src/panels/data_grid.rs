use crate::theme::colors::*;
use egui::{RichText, Ui};
use egui_extras::{Column, TableBuilder};

#[derive(Default)]
pub struct DataGridPanel {
    pub sql_query: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub is_loading: bool,
    pub execution_time_ms: u64,
    pub error_message: Option<String>,
}

impl DataGridPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, ui: &mut Ui, event_tx: &std::sync::mpsc::Sender<crate::app::AppEvent>) {
        // Cmd+Enter keyboard shortcut
        let kb_shortcut = ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.command);

        // Main Tab Bar
        egui::TopBottomPanel::top("editor_tab_bar")
            .frame(
                egui::Frame::NONE
                    .fill(SURFACE_CONTAINER_LOW)
                    .inner_margin(egui::Margin {
                        left: 4,
                        right: 0,
                        top: 4,
                        bottom: 0,
                    }),
            )
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;

                    // Active Tab
                    let tab_rect = ui.allocate_space(egui::vec2(140.0, 26.0)).1;
                    ui.painter().rect_filled(
                        tab_rect,
                        egui::CornerRadius {
                            nw: 4,
                            ne: 4,
                            sw: 0,
                            se: 0,
                        },
                        SURFACE_CONTAINER_LOWEST,
                    );
                    ui.painter().rect_stroke(
                        tab_rect,
                        egui::CornerRadius {
                            nw: 4,
                            ne: 4,
                            sw: 0,
                            se: 0,
                        },
                        egui::Stroke::new(1.0, OUTLINE_VARIANT.linear_multiply(0.2)),
                        egui::StrokeKind::Inside,
                    );

                    ui.painter().text(
                        tab_rect.min + egui::vec2(12.0, 6.0),
                        egui::Align2::LEFT_TOP,
                        "Terminal  query_01.sql",
                        egui::FontId::proportional(11.0),
                        SECONDARY,
                    );

                    // Add button
                    if ui
                        .add(
                            egui::Button::new(RichText::new("➕").color(ON_SURFACE_VARIANT))
                                .frame(false),
                        )
                        .clicked()
                    {}
                });
            });

        // Top: Query Editor Panel
        egui::TopBottomPanel::top("query_editor_panel")
            .resizable(true)
            .min_height(120.0)
            .height_range(120.0..=500.0)
            .frame(
                egui::Frame::NONE
                    .fill(SURFACE_CONTAINER_LOWEST)
                    .inner_margin(8.0),
            )
            .show_inside(ui, |ui| {
                // Custom SQL syntax highlighter (bypasses egui_extras::highlight
                // which breaks TextEdit interaction)
                let text_color = ui.visuals().text_color();
                let mut layouter = |_ui: &egui::Ui, string: &str, wrap_width: f32| {
                    let job = Self::highlight_sql(string, 13.0, text_color);
                    let mut job = job;
                    job.wrap.max_width = wrap_width;
                    _ui.fonts(|f| f.layout_job(job))
                };

                let response = ui.add(
                    egui::TextEdit::multiline(&mut self.sql_query)
                        .font(egui::TextStyle::Monospace)
                        .layouter(&mut layouter)
                        .desired_width(f32::INFINITY)
                        .desired_rows(8)
                        .hint_text("-- Write your SQL here and press ▶ EXECUTE or Cmd+Enter"),
                );

                // Cmd+Enter: execute
                if response.has_focus() && kb_shortcut && !self.sql_query.trim().is_empty() {
                    tracing::info!("[UI] Execute triggered via Cmd+Enter");
                    self.error_message = None;
                    let _ = event_tx.send(crate::app::AppEvent::ExecuteQuery(
                        self.sql_query.clone(),
                    ));
                }
            });

        // Resizer (implicit via resizable panel above)
        // Bottom: Results Panel Header
        egui::TopBottomPanel::top("results_tab_bar")
            .frame(
                egui::Frame::NONE
                    .fill(SURFACE_CONTAINER_LOW)
                    .inner_margin(egui::Margin::symmetric(0, 0)),
            )
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    let tab_rect = ui.allocate_space(egui::vec2(100.0, 32.0)).1;
                    ui.painter().text(
                        tab_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "QUERY RESULT",
                        egui::FontId::proportional(10.0),
                        SECONDARY,
                    );
                    ui.painter().line_segment(
                        [tab_rect.left_bottom(), tab_rect.right_bottom()],
                        egui::Stroke::new(2.0, SECONDARY),
                    );

                    ui.add_space(16.0);
                    ui.label(
                        RichText::new("MESSAGES")
                            .size(10.0)
                            .strong()
                            .color(ON_SURFACE_VARIANT),
                    );
                    ui.add_space(16.0);
                    ui.label(
                        RichText::new("EXECUTION PLAN")
                            .size(10.0)
                            .strong()
                            .color(ON_SURFACE_VARIANT),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);

                        ui.add(
                            egui::Button::new(RichText::new("⚙").color(ON_SURFACE_VARIANT))
                                .frame(false),
                        )
                        .clicked();
                        ui.add(
                            egui::Button::new(RichText::new("⬇").color(ON_SURFACE_VARIANT))
                                .frame(false),
                        )
                        .clicked();

                        let time_rect = ui.allocate_space(egui::vec2(60.0, 20.0)).1;
                        ui.painter()
                            .rect_filled(time_rect, 2.0, SURFACE_CONTAINER_LOWEST);
                        ui.painter().rect_stroke(
                            time_rect,
                            2.0,
                            egui::Stroke::new(1.0, OUTLINE_VARIANT.linear_multiply(0.2)),
                            egui::StrokeKind::Inside,
                        );
                        ui.painter().text(
                            time_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("⏱ {}ms", self.execution_time_ms),
                            egui::FontId::proportional(10.0),
                            ON_SURFACE_VARIANT,
                        );
                    });
                });
                ui.painter().line_segment(
                    [ui.min_rect().left_bottom(), ui.min_rect().right_bottom()],
                    egui::Stroke::new(1.0, OUTLINE_VARIANT.linear_multiply(0.2)),
                );
            });

        // Bottom: Results Grid
        egui::Frame::NONE
            .fill(SURFACE_CONTAINER_LOWEST.linear_multiply(0.5))
            .show(ui, |ui| {
                // Error banner
                if let Some(err) = &self.error_message {
                    let frame = egui::Frame::NONE
                        .fill(egui::Color32::from_rgb(60, 20, 20))
                        .inner_margin(10.0)
                        .corner_radius(4.0);
                    frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("⚠ Error:")
                                    .color(egui::Color32::from_rgb(255, 130, 130))
                                    .strong(),
                            );
                            ui.label(
                                RichText::new(err).color(egui::Color32::from_rgb(255, 180, 180)),
                            );
                        });
                    });
                    ui.add_space(8.0);
                }

                if self.is_loading {
                    ui.centered_and_justified(|ui| {
                        ui.spinner();
                    });
                    return;
                }

                if self.columns.is_empty() && self.error_message.is_none() {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            RichText::new("Run a query to see results here.")
                                .color(ON_SURFACE_VARIANT),
                        );
                    });
                    return;
                }

                // Virtual scrolling table
                let mut table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .min_scrolled_height(0.0);

                // Row Number column
                table = table.column(Column::initial(40.0).clip(true));

                for _ in &self.columns {
                    table = table.column(Column::initial(150.0).clip(true));
                }

                table
                    .header(28.0, |mut header| {
                        header.col(|ui| {
                            ui.centered_and_justified(|ui| {
                                ui.label(
                                    RichText::new("#")
                                        .strong()
                                        .size(11.0)
                                        .color(ON_SURFACE_VARIANT),
                                );
                            });
                        });
                        for col_name in &self.columns {
                            header.col(|ui| {
                                ui.label(
                                    RichText::new(col_name)
                                        .strong()
                                        .size(11.0)
                                        .color(ON_SURFACE_VARIANT),
                                );
                            });
                        }
                    })
                    .body(|body| {
                        body.rows(28.0, self.rows.len(), |mut row| {
                            let row_index = row.index();
                            if let Some(row_data) = self.rows.get(row_index) {
                                row.col(|ui| {
                                    ui.centered_and_justified(|ui| {
                                        ui.label(
                                            RichText::new((row_index + 1).to_string())
                                                .color(ON_SURFACE_VARIANT.linear_multiply(0.5))
                                                .size(11.0),
                                        );
                                    });
                                });

                                for (i, cell) in row_data.iter().enumerate() {
                                    row.col(|ui| {
                                        if cell == "NULL" {
                                            ui.label(
                                                RichText::new(cell)
                                                    .color(ON_SURFACE_VARIANT.linear_multiply(0.5))
                                                    .italics()
                                                    .size(11.0),
                                            );
                                        } else {
                                            // Make first valid column secondary color
                                            let color = if i == 0 { SECONDARY } else { ON_SURFACE };
                                            ui.label(RichText::new(cell).color(color).size(11.0));
                                        }
                                    });
                                }
                            }
                        });
                    });
            });
    }

    /// Custom SQL syntax highlighter that produces a LayoutJob with correct byte ranges.
    /// Bypasses egui_extras::syntax_highlighting::highlight which breaks TextEdit interaction.
    fn highlight_sql(
        text: &str,
        font_size: f32,
        default_color: egui::Color32,
    ) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();
        job.text = text.into();

        if text.is_empty() {
            return job;
        }

        let font_id = egui::FontId::monospace(font_size);
        let kw_color = egui::Color32::from_rgb(86, 156, 214); // blue — keywords
        let str_color = egui::Color32::from_rgb(206, 145, 120); // orange — strings
        let num_color = egui::Color32::from_rgb(181, 206, 168); // green — numbers
        let cmt_color = egui::Color32::from_rgb(106, 153, 85); // dim green — comments
        let op_color = egui::Color32::from_rgb(180, 180, 180); // light grey — operators
        let fn_color = egui::Color32::from_rgb(220, 220, 170); // yellow — functions

        let keywords: &[&str] = &[
            "SELECT", "FROM", "WHERE", "AND", "OR", "NOT", "IN", "INSERT", "UPDATE", "DELETE",
            "CREATE", "DROP", "ALTER", "TABLE", "INTO", "VALUES", "SET", "JOIN", "LEFT", "RIGHT",
            "INNER", "OUTER", "FULL", "CROSS", "ON", "AS", "ORDER", "BY", "GROUP", "HAVING",
            "LIMIT", "OFFSET", "DISTINCT", "UNION", "ALL", "EXISTS", "BETWEEN", "LIKE", "ILIKE",
            "IS", "NULL", "TRUE", "FALSE", "CASE", "WHEN", "THEN", "ELSE", "END", "ASC", "DESC",
            "WITH", "RECURSIVE", "RETURNING", "IF", "BEGIN", "COMMIT", "ROLLBACK", "INDEX",
            "UNIQUE", "PRIMARY", "KEY", "FOREIGN", "REFERENCES", "CONSTRAINT", "DEFAULT",
            "CHECK", "CASCADE", "RESTRICT", "TRUNCATE", "EXPLAIN", "ANALYZE", "VACUUM",
            "GRANT", "REVOKE", "SCHEMA", "DATABASE", "VIEW", "TRIGGER", "FUNCTION", "PROCEDURE",
            "INTERVAL", "TYPE", "CAST", "COALESCE", "NULLIF", "OVER", "PARTITION", "ROW",
            "ROWS", "RANGE", "UNBOUNDED", "PRECEDING", "FOLLOWING", "CURRENT",
        ];

        let builtins: &[&str] = &[
            "COUNT", "SUM", "AVG", "MIN", "MAX", "NOW", "UPPER", "LOWER", "LENGTH", "TRIM",
            "SUBSTRING", "REPLACE", "CONCAT", "ROUND", "CEIL", "FLOOR", "ABS", "EXTRACT",
            "DATE_TRUNC", "TO_CHAR", "TO_DATE", "ARRAY_AGG", "STRING_AGG", "ROW_NUMBER",
            "RANK", "DENSE_RANK", "LAG", "LEAD", "FIRST_VALUE", "LAST_VALUE", "NTILE",
        ];

        let push_section =
            |job: &mut egui::text::LayoutJob,
             range: std::ops::Range<usize>,
             color: egui::Color32,
             font: &egui::FontId| {
                job.sections.push(egui::text::LayoutSection {
                    leading_space: 0.0,
                    byte_range: range,
                    format: egui::TextFormat::simple(font.clone(), color),
                });
            };

        let bytes = text.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            let start = i;

            // -- line comments
            if i + 1 < bytes.len() && bytes[i] == b'-' && bytes[i + 1] == b'-' {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                push_section(&mut job, start..i, cmt_color, &font_id);
                continue;
            }

            // /* block comments */
            if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                if i + 1 < bytes.len() {
                    i += 2;
                }
                push_section(&mut job, start..i, cmt_color, &font_id);
                continue;
            }

            // 'string literals'
            if bytes[i] == b'\'' {
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == b'\'' {
                        i += 1;
                        // handle escaped '' inside strings
                        if i < bytes.len() && bytes[i] == b'\'' {
                            i += 1;
                            continue;
                        }
                        break;
                    }
                    i += 1;
                }
                push_section(&mut job, start..i, str_color, &font_id);
                continue;
            }

            // Numbers (integers, decimals)
            if bytes[i].is_ascii_digit()
                || (bytes[i] == b'.' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit())
            {
                while i < bytes.len()
                    && (bytes[i].is_ascii_digit() || bytes[i] == b'.' || bytes[i] == b'e' || bytes[i] == b'E')
                {
                    i += 1;
                }
                push_section(&mut job, start..i, num_color, &font_id);
                continue;
            }

            // Words (keywords, identifiers, functions)
            if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                let word = &text[start..i];
                let upper = word.to_uppercase();
                let color = if keywords.contains(&upper.as_str()) {
                    kw_color
                } else if builtins.contains(&upper.as_str()) {
                    fn_color
                } else {
                    default_color
                };
                push_section(&mut job, start..i, color, &font_id);
                continue;
            }

            // Operators and punctuation
            if matches!(
                bytes[i],
                b'=' | b'<' | b'>' | b'!' | b'+' | b'-' | b'*' | b'/' | b'%' | b'(' | b')'
                    | b',' | b';' | b'.'
            ) {
                i += 1;
                push_section(&mut job, start..i, op_color, &font_id);
                continue;
            }

            // Everything else (whitespace, etc.) — single byte at a time
            i += 1;
            push_section(&mut job, start..i, default_color, &font_id);
        }

        job
    }
}
