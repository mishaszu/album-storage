use std::fs::Metadata;

use async_graphql::SimpleObject;
use tokio::fs::read_dir;

use crate::config;
use crate::graphql::{Error, IdentifiableString, Result};

mod mutation;
mod query;

pub use mutation::RawAlbumMutation;
pub use query::RawAlbumQuery;

#[derive(SimpleObject)]
pub struct DirItem {
    pub name: String,
    pub is_dir: bool,
    pub created_at: Option<chrono::NaiveDateTime>,
}

fn get_file_created_time(meta: Metadata) -> Result<i64> {
    let time = meta
        .created()
        .map_err(|_| Error::FailedToReadDir)?
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map_err(|_| Error::FailedToReadDir)?;
    Ok(time.as_secs() as i64)
}

async fn read_path(path: Option<String>) -> Result<Vec<DirItem>> {
    let root = config::config().IMAGES_DIR.clone();
    let path = if let Some(path) = path {
        root + "/" + &path
    } else {
        root
    };

    let mut dirs = read_dir(path).await.map_err(|_| Error::FailedToReadDir)?;

    let mut files = Vec::new();

    while let Some(dir) = dirs
        .next_entry()
        .await
        .map_err(|_| Error::FailedToReadDir)?
    {
        let d = dir.metadata().await.map_err(|_| Error::FailedToReadDir)?;
        if d.is_dir() {
            files.push(DirItem {
                name: dir
                    .file_name()
                    .into_string()
                    .map_err(|_| Error::FailedToReadDir)?,
                is_dir: true,
                created_at: chrono::NaiveDateTime::from_timestamp_opt(get_file_created_time(d)?, 0),
            });
        } else if d.is_file() {
            files.push(DirItem {
                name: dir
                    .file_name()
                    .into_string()
                    .map_err(|_| Error::FailedToReadDir)?,
                is_dir: false,
                created_at: chrono::NaiveDateTime::from_timestamp_opt(get_file_created_time(d)?, 0),
            });
        }
    }

    Ok(files)
}

#[derive(SimpleObject, Clone, Debug)]
pub struct RawAlbumString {
    title: String,
    created_at: Option<chrono::NaiveDateTime>,
}

impl IdentifiableString for RawAlbumString {
    fn get_id(&self) -> String {
        self.title.clone()
    }
}

impl RawAlbumString {
    pub async fn read(path: Option<String>) -> Result<Vec<Self>> {
        let dirs = read_path(path)
            .await?
            .into_iter()
            .filter(|d| d.is_dir)
            .map(|d| Self {
                title: d.name,
                created_at: d.created_at,
            })
            .collect();

        Ok(dirs)
    }
}

#[derive(SimpleObject)]
pub struct RawAlbum {
    title: String,
    items: Vec<DirItem>,
}

impl RawAlbum {
    pub async fn read(title: String) -> Result<Self> {
        let mut dirs = read_path(Some(title.clone())).await?;
        dirs.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(Self { title, items: dirs })
    }
}
