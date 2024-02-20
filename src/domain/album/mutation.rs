use async_graphql::*;
use async_graphql_relay::RelayNodeID;

use crate::{
    db::ModelManager,
    graphql::{AuthGuard, Error},
};

use super::{
    db_model::{Album as DbAlbum, AlbumDao},
    graphql_model::{CreateAlbum, UpdateAlbum},
    Album,
};

#[derive(Default)]
pub struct AlbumMutation;

#[Object]
impl AlbumMutation {
    #[graphql(guard = "AuthGuard")]
    async fn create_album(&self, ctx: &Context<'_>, input: CreateAlbum) -> Result<Album> {
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext.into()),
        };

        let album = AlbumDao::create(mm, input.into())
            .map(|album: DbAlbum| -> Album { album.into() })
            .map_err(|e| -> Error { e.into() })?;

        Ok(album)
    }

    #[graphql(guard = "AuthGuard")]
    async fn update_album(
        &self,
        ctx: &Context<'_>,
        id: RelayNodeID<Album>,
        input: UpdateAlbum,
    ) -> Result<Album> {
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext.into()),
        };

        let album = AlbumDao::update(mm, &id.to_uuid(), input.into())
            .map(|album: DbAlbum| -> Album { album.into() })
            .map_err(|e| -> Error { e.into() })?;

        Ok(album)
    }

    #[graphql(guard = "AuthGuard")]
    async fn delete_album(&self, ctx: &Context<'_>, id: RelayNodeID<Album>) -> Result<bool> {
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext.into()),
        };

        AlbumDao::delete(mm, &id.to_uuid()).map_err(|e| -> Error { e.into() })?;

        Ok(true)
    }
}
