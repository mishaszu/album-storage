use axum::{
    body::{Body as AxumBody, Bytes},
    http::{HeaderMap, HeaderName, HeaderValue},
    response::IntoResponse,
};
use derive_more::Display;
use reqwest::{
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    Body, Client, StatusCode, Url,
};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::config::config;

use super::error::{Error, Result};
use super::req_client;

#[derive(Debug, Display, Serialize, Clone)]
pub enum LustError {
    #[display(fmt = "Bad new image format")]
    Post400BadFormat,

    #[display(fmt = "Unauthorized")]
    Post401Unauthorized,

    #[display(fmt = "Bucket not found")]
    Post404BucketNotFound,

    #[display(fmt = "Payload too large")]
    Post413PayloadTooLarge,

    #[display(fmt = "Bad request, bucket: {}, image: {}", _0, _1)]
    BadRequest(String, String),

    #[display(fmt = "Not found, bucket: {}, image: {}", _0, _1)]
    NotFound(String, String),

    #[display(fmt = "Undefined error")]
    Undefined,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LustSizing {
    pub sizing_id: i32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LustResponse {
    pub image_id: String,
    pub processing_time: f64,
    pub io_time: f64,
    pub checksum: i64,
    pub images: Vec<LustSizing>,
    pub bucket_id: i64,
}

enum LustResult {
    Success,
    Error(LustError),
}

impl From<StatusCode> for LustResult {
    fn from(status: StatusCode) -> Self {
        match status {
            StatusCode::OK => LustResult::Success,
            StatusCode::BAD_REQUEST => LustResult::Error(LustError::Post400BadFormat),
            StatusCode::UNAUTHORIZED => LustResult::Error(LustError::Post401Unauthorized),
            StatusCode::NOT_FOUND => LustResult::Error(LustError::Post404BucketNotFound),
            StatusCode::PAYLOAD_TOO_LARGE => LustResult::Error(LustError::Post413PayloadTooLarge),
            _ => LustResult::Error(LustError::Undefined),
        }
    }
}

impl IntoResponse for LustResponse {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::to_string(&self).unwrap();
        axum::response::IntoResponse::into_response(body)
    }
}

pub struct Lust;

impl Lust {
    pub fn build_post_url(bucket: &str, params: Option<Vec<(String, String)>>) -> Result<Url> {
        let url = format!("{}/{}", &config().LUST_URL, bucket);
        req_client::build_url(&url, params)
    }

    pub fn build_get_url(
        bucket: &str,
        params: Option<Vec<(String, String)>>,
        image: &str,
    ) -> Result<Url> {
        let url = format!("{}/{}/{}", &config().LUST_URL, bucket, image);
        req_client::build_url(&url, params)
    }

    pub async fn post_stream(
        client: &Client,
        bucket: &str,
        size: u64,
        file: Bytes,
    ) -> Result<LustResponse> {
        let url = Self::build_post_url(bucket, None)?;
        debug!("{:<12} - LUST creating file", "LUST");
        let body = Body::from(file);
        let res = client
            .post(url)
            .header(CONTENT_TYPE, "application/octet-stream")
            .header(CONTENT_LENGTH, size.to_string())
            .body(body)
            .send()
            .await
            .map_err(|e| e.into());

        match res {
            Ok(res) => match res.status().into() {
                LustResult::Success => res.json::<LustResponse>().await.map_err(|e| e.into()),
                LustResult::Error(LustError::Undefined) => {
                    debug!(
                        "{:<12} - LUST undefined error: {} for bucket: {}",
                        "LUST",
                        res.status(),
                        bucket
                    );
                    Err(LustError::Undefined.into())
                }
                LustResult::Error(e) => Err(e.into()),
            },
            Err(e) => Err(e),
        }
    }

    pub async fn post_file(client: &Client, bucket: &str, file: Vec<u8>) -> Result<LustResponse> {
        let url = Self::build_post_url(
            bucket,
            Some(vec![("format".to_string(), "jpeg".to_string())]),
        )?;
        debug!("{:<12} - LUST creating file", "LUST");
        let size = file.len();
        println!("url: {}, size: {}", url, size);
        let res = client
            .post(url)
            .header(CONTENT_TYPE, "application/octet-stream")
            .header(CONTENT_LENGTH, size.to_string())
            .body(file)
            .send()
            .await
            .map_err(|e| e.into());

        match res {
            Ok(res) => match res.status().into() {
                LustResult::Success => res.json::<LustResponse>().await.map_err(|e| e.into()),
                LustResult::Error(LustError::Undefined) => {
                    debug!(
                        "{:<12} - LUST undefined error: {} for bucket: {}",
                        "LUST",
                        res.status(),
                        bucket
                    );
                    Err(LustError::Undefined.into())
                }
                LustResult::Error(e) => Err(e.into()),
            },
            Err(e) => Err(e),
        }
    }

    pub async fn get_file(
        client: &Client,
        bucket: &str,
        image_id: &str,
        params: Option<Vec<(String, String)>>,
    ) -> Result<(impl IntoResponse, HeaderMap)> {
        let url = Self::build_get_url(bucket, params, image_id)?;
        debug!("{:<12} - LUST getting file - {}", "LUST", &url);
        let res = client
            .get(url)
            .send()
            .await
            .map_err(|e| Error::ReqwestFailed(e.to_string()))?;

        match res.status() {
            StatusCode::OK => {
                let headers = res.headers().clone();
                let mut axum_headers = HeaderMap::new();
                headers
                    .into_iter()
                    .for_each(|(k, v)| match (k, v.to_str()) {
                        (Some(k), Ok(v)) => {
                            let name = k.to_string();
                            let name = HeaderName::from_bytes(name.as_bytes());
                            let value = v.to_string();
                            let value = HeaderValue::from_bytes(value.as_bytes());

                            match (name, value) {
                                (Ok(name), Ok(value)) => {
                                    axum_headers.insert(name, value);
                                }
                                _ => (),
                            }
                        }
                        _ => (),
                    });
                let body = AxumBody::from_stream(res.bytes_stream());
                Ok((body, axum_headers))
            }
            StatusCode::NOT_FOUND => {
                Err(LustError::NotFound(bucket.to_string(), image_id.to_string()).into())
            }
            StatusCode::BAD_REQUEST => {
                Err(LustError::BadRequest(bucket.to_string(), image_id.to_string()).into())
            }
            _ => {
                debug!(
                    "{:<12} - LUST undefined error: {} for image: {}",
                    "LUST",
                    res.status(),
                    image_id
                );
                Err(LustError::Undefined.into())
            }
        }
    }

    pub async fn delete_file(client: &Client, bucket: &str, image_id: &str) -> Result<()> {
        let url = Self::build_get_url(bucket, None, &image_id)?;
        debug!("{:<12} - LUST deleting file", "LUST");
        let res = client
            .delete(url)
            .send()
            .await
            .map_err(|e| Error::ReqwestFailed(e.to_string()))?;

        match res.status() {
            StatusCode::OK => {
                debug!("{:<12} - LUST file deleted", "REDIRECT");
                Ok(())
            }
            StatusCode::NOT_FOUND => {
                Err(LustError::NotFound(bucket.to_string(), image_id.to_string()).into())
            }
            StatusCode::BAD_REQUEST => {
                Err(LustError::BadRequest(bucket.to_string(), image_id.to_string()).into())
            }
            _ => {
                debug!(
                    "{:<12} - LUST undefined error: {} for image: {}",
                    "LUST",
                    res.status(),
                    image_id
                );
                Err(LustError::Undefined.into())
            }
        }
    }
}
