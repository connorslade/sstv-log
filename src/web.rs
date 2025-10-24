use std::sync::Arc;

use anyhow::Result;
use axum::{
    Router,
    extract::{State, WebSocketUpgrade, ws::Message},
    response::IntoResponse,
    routing::get,
};
use tokio::{
    net::TcpListener,
    sync::broadcast::{Receiver, error::RecvError},
};
use tower_http::services::ServeDir;

use crate::sstv::decode::SstvEvent;

pub async fn web_server(rx: Receiver<SstvEvent>) -> Result<()> {
    let serve = ServeDir::new("web").append_index_html_on_directories(true);
    let service = Router::new()
        .route("/events", get(events))
        .fallback_service(serve)
        .with_state(Arc::new(rx));

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    axum::serve(listener, service).await?;
    Ok(())
}

async fn events(
    ws: WebSocketUpgrade,
    State(rx): State<Arc<Receiver<SstvEvent>>>,
) -> impl IntoResponse {
    let mut rx = rx.resubscribe();
    ws.on_upgrade(async move |mut socket| {
        loop {
            let event = match rx.recv().await {
                Ok(x) => x,
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => break,
            };

            let msg = match event {
                SstvEvent::Start => Message::Text("decode_start".into()),
                SstvEvent::Progress(p) => Message::Text(format!("decode_progress:{p}").into()),
                SstvEvent::End(image) => Message::Binary(image),
            };

            if socket.send(msg).await.is_err() {
                break;
            };
        }
    })
}
