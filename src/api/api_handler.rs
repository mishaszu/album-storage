use axum::{extract::DefaultBodyLimit, middleware, response::IntoResponse, routing::get, Router};
use reqwest::Client;

use crate::{api::auth_middleware::mw_ctx_require, db::ModelManager};

use super::image_handler::{get_image, get_raw_image};

#[derive(Clone)]
pub struct ApiState {
    pub reqwest_client: Client,
    pub mm: ModelManager,
}

pub fn routes(mm: ModelManager) -> Router {
    let reqwest_client = Client::new();
    Router::new()
        .route("/test", get(test))
        .route("/image/:image_id", get(get_image))
        .route("/raw_image/:album_id/:image_id", get(get_raw_image))
        .layer(middleware::from_fn_with_state(mm.clone(), mw_ctx_require))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 10))
        .with_state(ApiState { reqwest_client, mm })
}

async fn test() -> impl IntoResponse {
    "User logged in".to_string()
}
