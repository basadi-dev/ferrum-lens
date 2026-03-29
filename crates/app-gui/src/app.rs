

use std::sync::mpsc::{channel, Sender, Receiver};
use app_core::db_pool::DbPool;
use app_core::models::connection::ConnectionConfig;
use app_core::queries::{SavedQuery, SavedQueryStore};

pub enum ActiveView {
    #[allow(dead_code)]
    Welcome,
    ConnectionManager,
    QueryGrid,
    SchemaInspector(String), // table name
}

pub enum ConnectionState {
    Disconnected,
    Connected(DbPool, ConnectionConfig),
}

pub enum AppEvent {
    Connect(DbPool, ConnectionConfig),
    OpenTable(String),
    OpenQueryEditor,
    Disconnect,
    TablesLoaded(Vec<app_core::schema::TableMetadata>),
    ColumnsLoaded(String, Vec<app_core::schema::ColumnMetadata>),
    ExecuteQuery(String),
    QueryResultsLoaded(app_core::query::QueryResult, String), // result, query string
    QueryError(String),
    OpenSavedQuery(String, String), // title, sql content
    LoadColumnsForSidebar(String),
    ColumnsLoadedForSidebar(String, Vec<app_core::schema::ColumnMetadata>),
    ExecuteQuerySilently(String),
    OpenQueryEditorWithText(String),
    SaveQuery(String, String), // name, content
    DeleteQuery(String), // id
    QueriesLoaded(Vec<SavedQuery>),
    SaveActiveQueryContent(String), // content
}

pub struct FerrumApp {
    active_view: ActiveView,
    connection_state: ConnectionState,
    
    event_tx: Sender<AppEvent>,
    event_rx: Receiver<AppEvent>,

    connection_panel: crate::panels::connection_manager::ConnectionManagerPanel,
    explorer_panel: crate::panels::query_explorer::QueryExplorerPanel,
    schema_panel: crate::panels::schema_inspector::SchemaInspectorPanel,
    data_grid_panel: crate::panels::data_grid::DataGridPanel,
    query_store: SavedQueryStore,
}

impl FerrumApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        crate::theme::setup_theme(&cc.egui_ctx);
        let (event_tx, event_rx) = channel();

        Self {
            active_view: ActiveView::ConnectionManager,
            connection_state: ConnectionState::Disconnected,
            event_tx,
            event_rx,
            connection_panel: crate::panels::connection_manager::ConnectionManagerPanel::default(),
            explorer_panel: crate::panels::query_explorer::QueryExplorerPanel::default(),
            schema_panel: crate::panels::schema_inspector::SchemaInspectorPanel::default(),
            data_grid_panel: crate::panels::data_grid::DataGridPanel::new(),
            query_store: SavedQueryStore::new().expect("Failed to initialize query store"),
        }
    }
}

impl eframe::App for FerrumApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle events
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                AppEvent::Connect(pool, config) => {
                    let schema_or_db = match &config.engine {
                        app_core::models::connection::DatabaseEngine::PostgreSql => "public".to_string(),
                        _ => config.database.clone(),
                    };
                    self.connection_state = ConnectionState::Connected(pool.clone(), config);
                    self.active_view = ActiveView::QueryGrid;
                    
                    // Fetch schema aggressively upon connection
                    let tx = self.event_tx.clone();
                    tokio::spawn(async move {
                        if let Ok(tables) = app_core::schema::list_tables(&pool, &schema_or_db).await {
                            let _ = tx.send(AppEvent::TablesLoaded(tables));
                        }
                    });
                }
                AppEvent::TablesLoaded(tables) => {
                    self.explorer_panel.set_tables(tables);
                }
                AppEvent::OpenTable(table_name) => {
                    self.active_view = ActiveView::SchemaInspector(table_name.clone());
                    
                    if let ConnectionState::Connected(pool, config) = &self.connection_state {
                        let tx = self.event_tx.clone();
                        let db_name = config.database.clone();
                        let pool_clone = pool.clone();
                        
                        tokio::spawn(async move {
                            if let Ok(columns) = app_core::schema::list_columns(&pool_clone, &db_name, &table_name).await {
                                let _ = tx.send(AppEvent::ColumnsLoaded(table_name, columns));
                            }
                        });
                    }
                }
                AppEvent::ColumnsLoaded(table_name, columns) => {
                    self.schema_panel.set_columns(table_name, columns);
                }
                AppEvent::ExecuteQuery(query) => {
                    tracing::info!("[EVENT] ExecuteQuery received: {}", &query);
                    self.data_grid_panel.is_loading = true;
                    if let ConnectionState::Connected(pool, _config) = &self.connection_state {
                        let tx = self.event_tx.clone();
                        let pool_clone = pool.clone();
                        let sql = query.clone();
                        let repaint_ctx = ctx.clone();
                        
                        tokio::spawn(async move {
                            tracing::info!("[ASYNC] Executing query...");
                            match app_core::query::execute_query(&pool_clone, &sql).await {
                                Ok(res) => {
                                    tracing::info!("[ASYNC] Query OK — {} cols, {} rows", res.columns.len(), res.rows.len());
                                    let _ = tx.send(AppEvent::QueryResultsLoaded(res, sql));
                                }
                                Err(e) => {
                                    tracing::error!("[ASYNC] Query error: {}", e);
                                    let _ = tx.send(AppEvent::QueryError(format!("{}", e)));
                                }
                            }
                            repaint_ctx.request_repaint();
                        });
                    } else {
                        tracing::warn!("[EVENT] ExecuteQuery but not connected!");
                        self.data_grid_panel.is_loading = false;
                    }
                }
                AppEvent::QueryResultsLoaded(result, _sql) => {
                    tracing::info!("[EVENT] QueryResultsLoaded — {} cols, {} rows, {}ms", result.columns.len(), result.rows.len(), result.execution_time_ms);
                    self.data_grid_panel.is_loading = false;
                    self.data_grid_panel.error_message = None;
                    self.data_grid_panel.columns = result.columns;
                    self.data_grid_panel.rows = result.rows;
                    self.data_grid_panel.execution_time_ms = result.execution_time_ms;
                }
                AppEvent::QueryError(msg) => {
                    tracing::error!("[EVENT] QueryError: {}", &msg);
                    self.data_grid_panel.is_loading = false;
                    self.data_grid_panel.error_message = Some(msg);
                }
                AppEvent::OpenQueryEditor => {
                    self.active_view = ActiveView::QueryGrid;
                }
                AppEvent::OpenSavedQuery(_title, sql) => {
                    self.data_grid_panel.sql_query = sql;
                    self.active_view = ActiveView::QueryGrid;
                }
                AppEvent::LoadColumnsForSidebar(table_name) => {
                    if let ConnectionState::Connected(pool, config) = &self.connection_state {
                        let tx = self.event_tx.clone();
                        let db_name = config.database.clone();
                        let pool_clone = pool.clone();
                        
                        tokio::spawn(async move {
                            if let Ok(columns) = app_core::schema::list_columns(&pool_clone, &db_name, &table_name).await {
                                let _ = tx.send(AppEvent::ColumnsLoadedForSidebar(table_name, columns));
                            }
                        });
                    }
                }
                AppEvent::ColumnsLoadedForSidebar(table_name, columns) => {
                    self.explorer_panel.add_columns_to_cache(table_name, columns);
                }
                AppEvent::ExecuteQuerySilently(query) => {
                    if let ConnectionState::Connected(pool, _config) = &self.connection_state {
                        let pool_clone = pool.clone();
                        tokio::spawn(async move {
                            let _ = app_core::query::execute_query(&pool_clone, &query).await;
                        });
                    }
                }
                AppEvent::OpenQueryEditorWithText(text) => {
                    self.data_grid_panel.sql_query = text;
                    self.active_view = ActiveView::QueryGrid;
                }
                AppEvent::SaveQuery(name, content) => {
                    let mut queries = self.query_store.load_queries().unwrap_or_default();
                    queries.push(SavedQuery {
                        id: uuid::Uuid::new_v4().to_string(),
                        name,
                        content,
                        created_at: std::time::SystemTime::now(),
                    });
                    let _ = self.query_store.save_queries(&queries);
                }
                AppEvent::SaveActiveQueryContent(content) => {
                    if let Some(active_name) = &self.explorer_panel.active_item
                        && let Some(query) = self.explorer_panel.saved_queries.iter_mut().find(|q| &q.name == active_name) {
                            query.content = content;
                            self.explorer_panel.queries_changed = true;
                            // Optionally, update the store immediately:
                            let _ = self.query_store.save_queries(&self.explorer_panel.saved_queries);
                            tracing::info!("Saved modified content for query: {}", active_name);
                        }
                }
                AppEvent::DeleteQuery(id) => {
                    let mut queries = self.query_store.load_queries().unwrap_or_default();
                    queries.retain(|q| q.id != id);
                    let _ = self.query_store.save_queries(&queries);
                }
                AppEvent::QueriesLoaded(_) => {}
                AppEvent::Disconnect => {
                    self.connection_state = ConnectionState::Disconnected;
                    self.active_view = ActiveView::ConnectionManager;
                }
            }
        }

        // Menu Bar
        egui::TopBottomPanel::top("menu_bar")
            .frame(egui::Frame::NONE.fill(crate::theme::colors::SURFACE_CONTAINER_LOW).inner_margin(egui::Margin::symmetric(8, 4)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let btn = |text: &str| egui::Button::new(egui::RichText::new(text).size(11.0).color(crate::theme::colors::ON_SURFACE)).frame(false);
                    ui.add(btn("File")).clicked();
                    ui.add(btn("Edit")).clicked();
                    ui.add(btn("View")).clicked();
                    ui.add(egui::Button::new(egui::RichText::new("Database").size(11.0).color(crate::theme::colors::SECONDARY).strong()).frame(false)).clicked();
                    ui.add(btn("Window")).clicked();
                    ui.add(btn("Help")).clicked();

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let ConnectionState::Connected(_, _) = &self.connection_state
                            && ui.button(egui::RichText::new("DISCONNECT").size(10.0).strong().color(crate::theme::colors::ERROR)).clicked() {
                                let _ = self.event_tx.send(AppEvent::Disconnect);
                            }
                    });
                });
            });

        // App Toolbar
        egui::TopBottomPanel::top("app_toolbar")
            .frame(egui::Frame::NONE.fill(crate::theme::colors::SURFACE).inner_margin(egui::Margin::symmetric(12, 6)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let ConnectionState::Connected(_, _) = &self.connection_state {
                        if ui.add(egui::Button::new(egui::RichText::new("▶ EXECUTE").size(11.0).strong().color(crate::theme::colors::ON_SECONDARY)).fill(crate::theme::colors::SECONDARY)).clicked()
                            && matches!(self.active_view, ActiveView::QueryGrid) {
                                let query = self.data_grid_panel.sql_query.clone();
                                let _ = self.event_tx.send(AppEvent::ExecuteQuery(query));
                            }
                        ui.add_space(8.0);
                        if ui.button("💾").on_hover_text("Save Query (Cmd+S)").clicked()
                            && matches!(self.active_view, ActiveView::QueryGrid) {
                                let content = self.data_grid_panel.sql_query.clone();
                                let _ = self.event_tx.send(AppEvent::SaveActiveQueryContent(content));
                            }
                        if ui.button("▤").clicked() {
                            let _ = self.event_tx.send(AppEvent::OpenQueryEditor);
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // DB Name
                        if let ConnectionState::Connected(_, config) = &self.connection_state {
                            ui.label(egui::RichText::new(&config.database).color(crate::theme::colors::ON_SURFACE_VARIANT).size(11.0));
                            ui.label(egui::RichText::new("🗄").color(crate::theme::colors::SUCCESS_BADGE));
                        }
                    });
                });
            });

        // Only show sidebar if connected
        if let ConnectionState::Connected(_, _) = &self.connection_state {
            if self.explorer_panel.saved_queries.is_empty()
                && let Ok(queries) = self.query_store.load_queries() {
                    self.explorer_panel.set_saved_queries(queries);
                }

            egui::SidePanel::left("sidebar")
                .frame(egui::Frame::NONE.fill(crate::theme::colors::SURFACE_CONTAINER_LOW))
                .default_width(240.0)
                .show(ctx, |ui| {
                    self.explorer_panel.show(ui, &self.event_tx);
                });
            
            // Re-sync queries back if they changed
            if self.explorer_panel.queries_changed {
                let _ = self.query_store.save_queries(&self.explorer_panel.saved_queries);
                self.explorer_panel.queries_changed = false;
            }
        }

        // OS Status Bar MUST BE BEFORE CentralPanel to preserve layout limits
        egui::TopBottomPanel::bottom("status_bar")
            .frame(egui::Frame::NONE.fill(crate::theme::colors::ON_PRIMARY).inner_margin(egui::Margin::symmetric(12, 4))) // Use dark blue #002e6a
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    match &self.connection_state {
                        ConnectionState::Connected(_, config) => {
                            let status = format!("Connected: {}:{}", config.host, config.port.unwrap_or(0));
                            ui.label(egui::RichText::new("●").color(crate::theme::colors::SUCCESS_BADGE).size(10.0));
                            ui.label(egui::RichText::new(status).color(egui::Color32::from_white_alpha(200)).size(10.0));
                        }
                        ConnectionState::Disconnected => {
                            ui.label(egui::RichText::new("○ Disconnected").color(egui::Color32::from_white_alpha(100)).size(10.0));
                        }
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("v0.1.0-stable").color(crate::theme::colors::SUCCESS_BADGE).size(10.0));
                        if matches!(self.active_view, ActiveView::QueryGrid) {
                            let rows_text = format!("Rows: {}", self.data_grid_panel.rows.len());
                            ui.label(egui::RichText::new(rows_text).color(egui::Color32::from_white_alpha(200)).size(10.0));
                            let time_text = format!("Time: {}ms", self.data_grid_panel.execution_time_ms);
                            ui.label(egui::RichText::new(time_text).color(egui::Color32::from_white_alpha(200)).size(10.0));
                        }
                    });
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(crate::theme::colors::SURFACE))
            .show(ctx, |ui| {
            match &mut self.active_view {
                ActiveView::Welcome => {
                    ui.centered_and_justified(|ui| {
                        if ui.button("New Connection").clicked() {
                            self.active_view = ActiveView::ConnectionManager;
                        }
                    });
                }
                ActiveView::ConnectionManager => {
                    self.connection_panel.show(ui, &self.event_tx);
                }
                ActiveView::SchemaInspector(table_name) => {
                    self.schema_panel.show(ui, table_name);
                }
                ActiveView::QueryGrid => {
                    self.data_grid_panel.show(ui, &self.event_tx);
                }
            }
        });
    }
}
