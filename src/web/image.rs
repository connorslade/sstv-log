use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::{
    sstv::modes::SstvMode,
    web::{AnyResult, App},
};

#[derive(Serialize, Deserialize)]
struct Image {
    timestamp: u64,
    mode: SstvMode,
}

pub async fn images(State(app): State<Arc<App>>) -> AnyResult<impl IntoResponse> {
    let mut entries = (app.db)
        .query("SELECT id, timestamp, protocol FROM images;", ())
        .await?;

    let mut images = HashMap::<u64, Image>::new();
    while let Some(entry) = entries.next().await? {
        images.insert(
            entry.get(0)?,
            Image {
                timestamp: entry.get(1)?,
                mode: SstvMode::from_vis(entry.get::<u32>(2)? as u8),
            },
        );
    }

    Ok(Json(images))
}

pub async fn image(
    State(app): State<Arc<App>>,
    Path(id): Path<u64>,
) -> AnyResult<impl IntoResponse> {
    let mut result = (app.db)
        .query("SELECT image FROM images WHERE id = ?;", [id])
        .await?;
    let image = (result.next().await?)
        .context("ID not found.")?
        .get::<Vec<u8>>(0)?;
    Ok(image)
}
