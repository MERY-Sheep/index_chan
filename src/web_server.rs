#[cfg(feature = "web")]
pub mod server {
    use axum::{
        extract::State,
        response::{Html, IntoResponse, Json},
        routing::get,
        Router,
    };
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tower_http::cors::CorsLayer;
    use tower_http::services::ServeDir;
    use anyhow::Result;

    use crate::graph::CodeGraph;

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
}
