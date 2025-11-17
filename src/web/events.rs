use std::sync::Arc;

use axum::{
    extract::{State, WebSocketUpgrade, ws::Message},
    response::IntoResponse,
};
use tokio::sync::broadcast::error::RecvError;

use crate::{sstv::decode::SstvEvent, web::App};

pub async fn events(State(app): State<Arc<App>>, ws: WebSocketUpgrade) -> impl IntoResponse {
    let mut rx = app.rx.resubscribe();
    ws.on_upgrade(async move |mut socket| {
        loop {
            let event = match rx.recv().await {
                Ok(x) => x,
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => break,
            };

            let msg = match event {
                SstvEvent::Start(mode) => {
                    Message::Text(format!("decode_start:{}", mode.name()).into())
                }
                SstvEvent::Progress(p) => Message::Text(format!("decode_progress:{p}").into()),
                SstvEvent::End(_, image) => Message::Binary(image),
            };

            if socket.send(msg).await.is_err() {
                break;
            };
        }
    })
}
