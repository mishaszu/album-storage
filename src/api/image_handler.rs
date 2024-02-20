use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap},
    response::IntoResponse,
};
use serde::Deserialize;
use tracing::error;

use super::{api_handler::ApiState, error::Error};

use crate::{config::config, services::lust::Lust, utils::read_file};

#[derive(Deserialize)]
pub struct Image {
    size: Option<String>,
}

pub async fn get_image(
    State(context): State<ApiState>,
    Path(image_id): Path<String>,
    Query(payload): Query<Image>,
) -> Result<impl IntoResponse, Error> {
    let client = context.reqwest_client;
    let mut params = vec![];

    match payload.size {
        Some(size) => params.push(("size".to_string(), size)),
        None => (),
    }

    let (response, headers) =
        Lust::get_file(&client, &config().LUST_BUCKET, &image_id, Some(params))
            .await
            .map_err(|e| Error::ServiceError(e.to_string()))?;

    let mut file = response.into_response();
    let new_headers: &mut HeaderMap = file.headers_mut();
    headers.into_iter().for_each(|(k, v)| {
        if let Some(header_name) = k {
            new_headers.insert(header_name, v);
        }
    });

    Ok(file)
}

pub async fn get_raw_image(
    Path((album_id, image_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let image = format!("{}/{}", album_id, image_id);
    let file = read_file(&image).await.map_err(|e| {
        error!("Failed to read file: {:?}", e);
        Error::FailedToReadFile
    });

    match file {
        Ok(file) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("image/jpeg"),
            );

            Ok((headers, file))
        }
        Err(e) => Err(e),
    }
}
