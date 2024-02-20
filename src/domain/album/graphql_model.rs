use async_graphql::{ComplexObject, Context};
use async_graphql::{InputObject, SimpleObject};
use async_graphql_relay::{RelayNode, RelayNodeID, RelayNodeObject};
use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::image::{Image, ImageDao};
use crate::{
    db::ModelManager,
    graphql::{node::Node, Error, Identifiable},
};

use super::db_model::{
    Album as DbAlbum, AlbumDao, CreateAlbum as DbCreateAlbum, UpdateAlbum as DbUpdateAlbum,
};

#[derive(SimpleObject, RelayNodeObject, Debug, Clone)]
#[graphql(complex)]
#[relay(node_suffix = "al")]
pub struct Album {
    pub id: RelayNodeID<Self>,
    pub title: String,
    pub description: Option<String>,
    pub original_title: String,
    pub is_uploaded: bool,
    pub prev_image_id: Option<Uuid>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[ComplexObject]
impl Album {
    async fn images(&self, ctx: &Context<'_>) -> Result<Vec<Image>, Error> {
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext),
        };
        let images = AlbumDao::get_album_images_sorted(mm, &self.id.to_uuid())
            .map_err(|e| -> Error { e.into() })?;
        Ok(images.into_iter().map(|r| r.into()).collect())
    }

    async fn image(&self, ctx: &Context<'_>) -> Result<Option<Image>, Error> {
        let mm = ctx.data_opt::<ModelManager>();
        let mm = match mm {
            Some(mm) => mm,
            None => return Err(Error::ModalManagerNotInContext),
        };
        let image = match self.prev_image_id {
            Some(image_id) => {
                let img = ImageDao::get_by_id(mm, &image_id).map_err(|e| -> Error { e.into() })?;
                Some(img.into())
            }
            None => None,
        };
        Ok(image)
    }
}

impl From<DbAlbum> for Album {
    fn from(album: DbAlbum) -> Self {
        Self {
            id: RelayNodeID::new(album.id),
            title: album.title,
            description: album.description,
            original_title: album.original_title,
            is_uploaded: album.is_uploaded,
            prev_image_id: album.prev_image_id,
            created_at: album.created_at,
            updated_at: album.updated_at,
        }
    }
}

impl Identifiable for Album {
    fn get_id(&self) -> uuid::Uuid {
        self.id.to_uuid()
    }
}

#[async_trait]
impl RelayNode for Album {
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
        let album = AlbumDao::get_by_id(mm, &id.to_uuid())
            .map(|album: DbAlbum| -> Album { album.into() })
            .map_err(|e| -> Error { e.into() })?;

        Ok(Some(album.into()))
    }
}

#[derive(InputObject)]
pub struct CreateAlbum {
    pub title: String,
    pub description: Option<String>,
    pub original_title: String,
    pub photoed_at: Option<chrono::NaiveDateTime>,
}

impl From<CreateAlbum> for DbCreateAlbum {
    fn from(val: CreateAlbum) -> Self {
        DbCreateAlbum {
            id: Uuid::new_v4(),
            title: val.title,
            description: val.description,
            original_title: val.original_title,
        }
    }
}

#[derive(InputObject)]
pub struct UpdateAlbum {
    pub title: Option<String>,
    pub description: Option<String>,
    pub original_title: Option<String>,
    pub is_uploaded: Option<bool>,
    pub is_ready: Option<bool>,
    pub prev_image_id: Option<Uuid>,
    pub photoed_at: Option<chrono::NaiveDateTime>,
}

impl From<UpdateAlbum> for DbUpdateAlbum {
    fn from(val: UpdateAlbum) -> Self {
        DbUpdateAlbum {
            title: val.title,
            description: val.description,
            original_title: val.original_title,
            is_uploaded: val.is_uploaded,
            prev_image_id: val.prev_image_id,
        }
    }
}
