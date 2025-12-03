// Web server for chat graph visualization
#[cfg(feature = "web")]
use anyhow::Result;
#[cfg(feature = "web")]
use axum::{
    extract::State,
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
#[cfg(feature = "web")]
use std::sync::Arc;
#[cfg(feature = "web")]
use tower_http::services::ServeDir;

#[cfg(feature = "web")]
use crate::conversation::{GraphData, PromptHistory};

#[cfg(feature = "web")]
#[derive(Clone)]
pub struct AppState {
    pub graph_data: Arc<GraphData>,
    pub prompt_history: Arc<PromptHistory>,
}

#[cfg(feature = "web")]
pub async fn start_chat_server(
    graph_data: GraphData,
    prompt_history: PromptHistory,
    port: u16,
) -> Result<()> {
    let state = AppState {
        graph_data: Arc::new(graph_data),
        prompt_history: Arc::new(prompt_history),
    };

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/graph", get(graph_handler))
        .route("/api/prompts", get(prompts_handler))
        .nest_service("/static", ServeDir::new("ui"))
        .with_state(state);

    let addr = format!("127.0.0.1:{}", port);
    println!("🌐 会話グラフUIを起動: http://{}", addr);
    println!("💡 Ctrl+Cで終了");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(feature = "web")]
async fn index_handler() -> impl IntoResponse {
    let html = include_str!("../ui/chat-graph.html");
    Html(html)
}

#[cfg(feature = "web")]
async fn graph_handler(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.graph_data.as_ref().clone())
}

#[cfg(feature = "web")]
async fn prompts_handler(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.prompt_history.as_ref().clone())
}



#[cfg(not(feature = "web"))]
pub async fn start_chat_server(
    _graph_data: crate::conversation::graph_exporter::GraphData,
    _prompt_history: crate::conversation::PromptHistory,
    _port: u16,
) -> anyhow::Result<()> {
    anyhow::bail!("Web機能が無効です。--features webでビルドしてください")
}
