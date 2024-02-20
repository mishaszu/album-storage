use reqwest::Url;

use super::error::{Error, Result};

pub fn build_url(url: &str, params: Option<Vec<(String, String)>>) -> Result<Url> {
    let url = match params {
        Some(params) => Url::parse_with_params(url, params),
        None => Url::parse(url),
    };

    url.map_err(|e| Error::UrlParseFailed(e.to_string()))
}
