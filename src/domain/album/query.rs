use async_graphql::*;
use async_graphql_relay::RelayNodeID;

use crate::graphql::{uuidIdentifiedQuery, AuthGuard, ConnectionResult, CursorParams};
use crate::{db::ModelManager, graphql::Error};

use super::{db_model::AlbumDao, Album, DbAlbum};

#[derive(Default)]
pub struct AlbumQuery;

#[Object]
impl AlbumQuery {
    #[graphql(guard = "AuthGuard")]
    async fn album(&self, ctx: &Context<'_>, id: RelayNodeID<Album>) -> Result<Album> {
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext.into()),
        };

        let album = AlbumDao::get_by_id(mm, &id.to_uuid())
            .map(|album: DbAlbum| -> Album { album.into() })
            .map_err(|e| -> Error { e.into() })?;

        Ok(album)
    }

    #[graphql(guard = "AuthGuard")]
    async fn albums(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> ConnectionResult<Album> {
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext.into()),
        };

        let albums = AlbumDao::list(mm)
            .map(|albums: Vec<DbAlbum>| -> Vec<Album> {
                albums.into_iter().map(|album| album.into()).collect()
            })
            .map_err(|e| -> Error { e.into() })?;

        uuidIdentifiedQuery(
            albums.into_iter(),
            CursorParams::new(after, before, first, last),
            10,
        )
        .await
    }
}
