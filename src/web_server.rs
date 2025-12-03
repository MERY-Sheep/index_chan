#[cfg(feature = "web")]
pub mod server {
    use axum::{
        extract::{Query, State},
        http::StatusCode,
        response::{Html, IntoResponse, Json},
        routing::{get, post},
        Router,
    };
    use serde::{Deserialize, Serialize};
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tower_http::cors::CorsLayer;
    use tower_http::services::ServeDir;
    use anyhow::Result;

    use crate::graph::CodeGraph;
    use crate::filter::GraphFilter;

    #[derive(Clone)]
    pub struct AppState {
        pub graph: Arc<CodeGraph>,
    }

    pub async fn start_server(
        graph: CodeGraph,
        port: u16,
    ) -> Result<()> {
        let state = AppState {
            graph: Arc::new(graph),
        };

        let app = Router::new()
            .route("/", get(index_handler))
            .route("/api/graph", get(graph_handler))
            .route("/api/filter", post(filter_handler))
            .route("/api/filter/keywords", get(filter_keywords_handler))
            .route("/api/filter/dead-code", get(filter_dead_code_handler))
            .route("/api/filter/file", get(filter_file_handler))
            .nest_service("/static", ServeDir::new("static"))
            .layer(CorsLayer::permissive())
            .with_state(state);

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        println!("🌐 Webサーバー起動: http://{}", addr);
        println!("📊 3D依存関係グラフを表示中...");
        println!("💡 Ctrl+Cで終了");

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }

    async fn index_handler() -> impl IntoResponse {
        Html(include_str!("../static/index.html"))
    }

    async fn graph_handler(State(state): State<AppState>) -> impl IntoResponse {
        Json((*state.graph).clone())
    }

    // Filter request/response types
    #[derive(Deserialize)]
    struct FilterRequest {
        query: String,
        #[serde(default)]
        include_dependencies: bool,
        #[serde(default)]
        use_llm: bool,
    }

    #[derive(Serialize)]
    struct FilterResponse {
        graph: CodeGraph,
        stats: FilterStats,
    }

    #[derive(Serialize)]
    struct FilterStats {
        original_nodes: usize,
        filtered_nodes: usize,
        original_edges: usize,
        filtered_edges: usize,
        filter_query: String,
    }

    #[derive(Deserialize)]
    struct KeywordsQuery {
        keywords: String,
        #[serde(default)]
        include_dependencies: bool,
    }

    #[derive(Deserialize)]
    struct FileQuery {
        file_pattern: String,
        #[serde(default)]
        include_dependencies: bool,
    }

    // Filter by natural language query (POST /api/filter)
    async fn filter_handler(
        State(state): State<AppState>,
        Json(req): Json<FilterRequest>,
    ) -> Result<Json<FilterResponse>, (StatusCode, String)> {
        let filter = GraphFilter::new();
        
        // Parse keywords from query (simple split for now)
        let keywords: Vec<String> = req.query
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let filtered_graph = filter
            .filter_by_keywords(&state.graph, &keywords, req.include_dependencies)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let stats = FilterStats {
            original_nodes: state.graph.nodes.len(),
            filtered_nodes: filtered_graph.nodes.len(),
            original_edges: state.graph.edges.len(),
            filtered_edges: filtered_graph.edges.len(),
            filter_query: req.query.clone(),
        };

        Ok(Json(FilterResponse {
            graph: filtered_graph,
            stats,
        }))
    }

    // Filter by keywords (GET /api/filter/keywords?keywords=llm,analyzer&include_dependencies=true)
    async fn filter_keywords_handler(
        State(state): State<AppState>,
        Query(params): Query<KeywordsQuery>,
    ) -> Result<Json<FilterResponse>, (StatusCode, String)> {
        let filter = GraphFilter::new();
        
        let keywords: Vec<String> = params.keywords
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let filtered_graph = filter
            .filter_by_keywords(&state.graph, &keywords, params.include_dependencies)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let stats = FilterStats {
            original_nodes: state.graph.nodes.len(),
            filtered_nodes: filtered_graph.nodes.len(),
            original_edges: state.graph.edges.len(),
            filtered_edges: filtered_graph.edges.len(),
            filter_query: params.keywords.clone(),
        };

        Ok(Json(FilterResponse {
            graph: filtered_graph,
            stats,
        }))
    }

    // Filter dead code (GET /api/filter/dead-code)
    async fn filter_dead_code_handler(
        State(state): State<AppState>,
    ) -> Result<Json<FilterResponse>, (StatusCode, String)> {
        let filter = GraphFilter::new();
        
        let filtered_graph = filter
            .filter_dead_code(&state.graph)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let stats = FilterStats {
            original_nodes: state.graph.nodes.len(),
            filtered_nodes: filtered_graph.nodes.len(),
            original_edges: state.graph.edges.len(),
            filtered_edges: filtered_graph.edges.len(),
            filter_query: "dead_code".to_string(),
        };

        Ok(Json(FilterResponse {
            graph: filtered_graph,
            stats,
        }))
    }

    // Filter by file (GET /api/filter/file?file_pattern=scanner.rs&include_dependencies=true)
    async fn filter_file_handler(
        State(state): State<AppState>,
        Query(params): Query<FileQuery>,
    ) -> Result<Json<FilterResponse>, (StatusCode, String)> {
        let filter = GraphFilter::new();
        
        let filtered_graph = filter
            .filter_by_file(&state.graph, &params.file_pattern, params.include_dependencies)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let stats = FilterStats {
            original_nodes: state.graph.nodes.len(),
            filtered_nodes: filtered_graph.nodes.len(),
            original_edges: state.graph.edges.len(),
            filtered_edges: filtered_graph.edges.len(),
            filter_query: format!("file:{}", params.file_pattern),
        };

        Ok(Json(FilterResponse {
            graph: filtered_graph,
            stats,
        }))
    }
}
