pub mod app;
pub mod routes;

pub async fn serve(config: amp_core::AppConfig) {
    let addr = format!("{}:{}", config.host, config.port);
    let app = app::create(config).await;
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
