use async_graphql::{InputObject, SimpleObject};
use async_graphql_relay::{RelayNode, RelayNodeID, RelayNodeObject};
use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

use super::{AlbumImageDao, DbAlbumImage, DbCreateAlbumImage, DbUpdateAlbumImage};
use crate::{
    db::ModelManager,
    graphql::{node::Node, Error, Identifiable},
};

#[derive(SimpleObject, RelayNodeObject, Debug)]
#[relay(node_suffix = "ai")]
pub struct AlbumImage {
    pub id: RelayNodeID<Self>,
    pub album_id: Uuid,
    pub image_id: Uuid,
    pub order_index: i32,
    pub highlighted: bool,
    pub is_primary_album: bool,
}

impl From<DbAlbumImage> for AlbumImage {
    fn from(album_image: DbAlbumImage) -> Self {
        Self {
            id: RelayNodeID::new(album_image.id),
            album_id: album_image.album_id,
            image_id: album_image.image_id,
            order_index: album_image.order_index,
            highlighted: album_image.highlighted,
            is_primary_album: album_image.is_primary_album,
        }
    }
}

impl Identifiable for AlbumImage {
    fn get_id(&self) -> Uuid {
        self.id.to_uuid()
    }
}

#[async_trait]
impl RelayNode for AlbumImage {
    type TNode = Node;

    async fn get(
        ctx: async_graphql_relay::RelayContext,
        id: RelayNodeID<Self>,
    ) -> async_graphql::Result<Option<Self::TNode>> {
        let mm = ctx.get::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext.into()),
        };
        let options = AlbumImageDao::get_by_id(mm, &id.to_uuid())
            .map(|options: DbAlbumImage| -> AlbumImage { options.into() })
            .map_err(|e| -> Error { e.into() })?;

        Ok(Some(options.into()))
    }
}

#[derive(InputObject, Deserialize, Debug)]
pub struct CreateAlbumImage {
    pub album_id: Uuid,
    pub image_id: Uuid,
    pub order_index: i32,
    pub highlighted: bool,
    pub is_primary_album: bool,
}

impl Into<DbCreateAlbumImage> for CreateAlbumImage {
    fn into(self) -> DbCreateAlbumImage {
        DbCreateAlbumImage {
            id: Uuid::new_v4(),
            album_id: self.album_id,
            image_id: self.image_id,
            order_index: self.order_index,
            highlighted: self.highlighted,
            is_primary_album: self.is_primary_album,
        }
    }
}

#[derive(InputObject, Deserialize, Debug)]
pub struct UpdateAlbumImage {
    pub album_id: Option<Uuid>,
    pub order_index: Option<i32>,
    pub highlighted: Option<bool>,
    pub is_primary_album: Option<bool>,
}

impl Into<DbUpdateAlbumImage> for UpdateAlbumImage {
    fn into(self) -> DbUpdateAlbumImage {
        DbUpdateAlbumImage {
            album_id: self.album_id,
            order_index: self.order_index,
            highlighted: self.highlighted,
            is_primary_album: self.is_primary_album,
        }
    }
}

pub struct UpdateOrder {
    pub id: RelayNodeID<AlbumImage>,
    pub order_index: i32,
}

impl Into<DbUpdateAlbumImage> for UpdateOrder {
    fn into(self) -> DbUpdateAlbumImage {
        DbUpdateAlbumImage {
            album_id: None,
            order_index: Some(self.order_index),
            highlighted: None,
            is_primary_album: None,
        }
    }
}
