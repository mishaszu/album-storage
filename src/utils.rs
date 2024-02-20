use tokio::fs;
use tracing::error;

use crate::config;

#[derive(Debug)]
pub enum Error {
    FailedToReadFile,
}

pub async fn read_file(file: &str) -> Result<Vec<u8>, Error> {
    fs::read(&format!("{}/{}", &config::config().IMAGES_DIR, &file))
        .await
        .map_err(|_| Error::FailedToReadFile)
}

pub async fn delete_file(file: &str) -> Result<(), Error> {
    fs::remove_file(&format!("{}/{}", &config::config().IMAGES_DIR, &file))
        .await
        .map_err(|e| {
            error!("{:<12} - failed to delete file: {}", "FILE", e);
            Error::FailedToReadFile
        })
}
