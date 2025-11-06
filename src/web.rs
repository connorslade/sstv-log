use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

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
use turso::Connection;

use crate::sstv::decode::SstvEvent;

struct App {
    rx: Receiver<SstvEvent>,
    db: Connection,
}

pub async fn web_server(rx: Receiver<SstvEvent>) -> Result<()> {
    let db = turso::Builder::new_local("data.db")
        .with_io("io_uring".into())
        .build()
        .await?
        .connect()?;

    db.execute(include_str!("sql/init_images.sql"), ()).await?;
    let state = Arc::new(App { rx, db });

    let this = state.clone();
    tokio::spawn(async move {
        let mut rx = this.rx.resubscribe();
        loop {
            let event = rx.recv().await.unwrap();
            if let SstvEvent::End(mode, image) = event {
                let time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                this.db
                    .execute(
                        "INSERT INTO images VALUES (?, ?, ?)",
                        (time, mode.to_vis(), &image[..]),
                    )
                    .await
                    .unwrap();
            }
        }
    });

    let serve = ServeDir::new("web").append_index_html_on_directories(true);
    let service = Router::new()
        .route("/events", get(events))
        .fallback_service(serve)
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    axum::serve(listener, service).await?;
    Ok(())
}

async fn events(ws: WebSocketUpgrade, State(app): State<Arc<App>>) -> impl IntoResponse {
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
