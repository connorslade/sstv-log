use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use axum::{
    Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use tokio::{net::TcpListener, sync::broadcast::Receiver};
use tower_http::services::ServeDir;
use turso::Connection;

use crate::sstv::decode::SstvEvent;

mod events;
mod image;
use events::events;
use image::{image, images};

type AnyResult<T> = axum::response::Result<T, AnyError>;
struct AnyError(anyhow::Error);

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

    db.execute(include_str!("../sql/init_images.sql"), ())
        .await?;
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
                        "INSERT INTO images (timestamp, protocol, image) VALUES (?, ?, ?)",
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
        .route("/images", get(images))
        .route("/image/{id}", get(image))
        .fallback_service(serve)
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    axum::serve(listener, service).await?;
    Ok(())
}

impl IntoResponse for AnyError {
    fn into_response(self) -> Response {
        let msg = format!("Internal server error: {}", self.0);
        (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
    }
}

impl<E> From<E> for AnyError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
