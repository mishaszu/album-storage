use async_graphql::*;

use crate::{config::config, graphql::AuthGuard, utils::delete_file};

#[derive(Default)]
pub struct RawAlbumMutation;

#[derive(SimpleObject)]
pub struct DeleteResult {
    path: String,
    success: bool,
}

#[Object]
impl RawAlbumMutation {
    #[graphql(guard = "AuthGuard")]
    async fn delete_raw_files(
        &self,
        _ctx: &Context<'_>,
        paths: Vec<String>,
    ) -> FieldResult<Vec<DeleteResult>> {
        let root = config().IMAGES_DIR.clone();
        let mut deleted = Vec::new();
        for path in paths {
            let path = format!("{}/{}", root, path);
            let res = delete_file(&path).await.is_ok();
            deleted.push(DeleteResult { path, success: res });
        }
        Ok(deleted)
    }

    #[graphql(guard = "AuthGuard")]
    async fn delete_dir(&self, _ctx: &Context<'_>, path: String) -> FieldResult<DeleteResult> {
        let root = config().IMAGES_DIR.clone();
        let path = format!("{}/{}", root, path);
        let res = delete_file(&path).await.is_ok();
        Ok(DeleteResult { path, success: res })
    }
}
