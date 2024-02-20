use async_graphql::*;

use crate::graphql::{stringIdentifiedQuery, AuthGuard, CursorParams, StringConnectionResult};

use super::{RawAlbum, RawAlbumString};

#[derive(Default)]
pub struct RawAlbumQuery;

#[Object]
impl RawAlbumQuery {
    #[graphql(guard = "AuthGuard")]
    async fn dirs(
        &self,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> StringConnectionResult<RawAlbumString> {
        stringIdentifiedQuery(
            RawAlbumString::read(None).await?.into_iter(),
            CursorParams::new(after, before, first, last),
            10,
        )
        .await
    }
    #[graphql(guard = "AuthGuard")]
    async fn dir(&self, title: String) -> Result<RawAlbum> {
        Ok(RawAlbum::read(title).await?)
    }
}
